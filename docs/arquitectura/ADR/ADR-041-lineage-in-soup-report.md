# ADR-041: Lineage tracking en `SoupReport` — wiring + per-cell tagging

**Estado:** Propuesto
**Fecha:** 2026-04-14
**Contexto:** AUTOPOIESIS track, sprint AP-6c (tarea #4 — lineage tree renderer)
**ADRs relacionados:** ADR-039 (fission criterion), ADR-040 (streaming SoupSim)

## 1. Contexto y problema

- Módulos afectados:
  - `src/use_cases/experiments/autopoiesis.rs:147-190` (`SoupReport` + `to_dot`)
  - `src/blueprint/equations/fission.rs:162-230` (`FissionOutcome`, `apply_fission`)
  - `src/layers/lineage_registry.rs:30-50` (`LineageRegistry`, `record_birth`)
  - `src/layers/species_grid.rs` (hoy no lleva lineage tag por celda)

- Evidencia del gap:
  - `SoupReport::to_dot()` documenta el agujero explícitamente (autopoiesis.rs:169):
    > _"la relación padre↔hijo por fission no está rastreada en `SoupReport` — futuro AP-6b"_
  - **La máquina de fisión existe y está testeada** — `apply_fission` (fission.rs:202) genera `FissionOutcome { lineage_a, lineage_b, dissipated_qe, cells_a, cells_b }` (fission.rs:166-177), conservando masa módulo tax.
  - **LineageRegistry existe** (layers/lineage_registry.rs:32) con `record_birth(lineage, parent, birth_tick)` y storage binary-searched.
  - **Pero el harness no las llama.** Grep `apply_fission|LineageRegistry` en `autopoiesis.rs` ⇒ **0 hits** en código (sólo un comentario en l.169). El harness detecta pressure crossings (autopoiesis.rs:323-334) y los cuenta en `ClosureFate::pressure_events`, pero jamás parte ningún blob.
  - Resultado: closures son detectadas, la sopa disipa, los fates se agregan — pero el tree genealógico del track no se construye. El criterio de cierre del track (README.md:124, _"≥ 1 fission"_) es inalcanzable porque _ningún_ fission ocurre hoy.

- Restricciones del stack (CLAUDE.md):
  - Hard Block #6: NO `HashMap` en hot paths — `LineageRegistry` ya cumple (Vec ordenada + binary search, l.30-34).
  - Coding rule #13: NO `Vec<T>` en componentes salvo longitud genuinamente variable — OK, `FissionEvent` es data de reporte, no componente.
  - Axiom 2 (pool invariant) y Axiom 4 (dissipation) ya honrados por `apply_fission`.

**Problema:** La viz de AP-6c no puede dibujar un árbol de linaje porque (a) el harness no hace fisiones y (b) `SoupReport` no lleva la lista de eventos ni per-cell lineage. El criterio de cierre del track no se satisface sin esto.

**Fuerzas:**
- Minimizar cambios — la máquina ya está; es cableado.
- Mantener determinismo y byte-estabilidad de `SoupReport` para seeds pre-existentes **en ausencia de fissions** (importa: AP-5 proptest corre sobre redes random donde muchas seeds no tendrán fission).
- Per-cell lineage es necesario: sin un "¿de quién es esta celda?" por celda del grid, no se puede saber **qué closure** fisionó cuando la presión cruza el umbral — sólo que _algo_ lo hizo.
- Budget sprint AP-6c: esto no puede convertirse en un sub-sprint de 2 semanas.

## 2. Decisión

Cableamos `apply_fission` en el harness + agregamos `lineage_of: Vec<u64>` paralelo al grid + extendemos `SoupReport` con eventos.

1. **Per-cell lineage tag** — nueva capa de datos paralela al grid:
   ```rust
   // src/layers/species_grid.rs (o módulo nuevo hermano)
   pub struct LineageGrid { tags: Vec<u64>, w: usize, h: usize }  // tags[y*w + x]
   ```
   Inicialmente todas las celdas → `lineage = 0` ("sopa primordial"). Al detectar una closure persistente por primera vez, sus celdas mask-marcadas reciben un `lineage_id` determinístico derivado del `closure.hash` (una sola función). Futuras fisiones reemplazan `lineage_id` en `cells_a`/`cells_b` con los valores de `FissionOutcome`.

2. **Trigger de fisión real en el harness** (actualmente sólo cuenta):
   Cuando `pressure_ratio(blob, …) > FISSION_PRESSURE_RATIO` (autopoiesis.rs:331), en vez de incrementar contador únicamente:
   ```rust
   let axis = pinch_axis(&blob.cells);
   let parent = dominant_lineage(&blob, &lineage_grid);
   let outcome = apply_fission(&mut grid, blob, axis, parent, tick);
   lineage_grid.stamp(&outcome.cells_a, outcome.lineage_a);
   lineage_grid.stamp(&outcome.cells_b, outcome.lineage_b);
   events.push(FissionEventRecord { tick, parent, children: [outcome.lineage_a, outcome.lineage_b], dissipated_qe: outcome.dissipated_qe });
   ```

3. **Extender `SoupReport`** (autopoiesis.rs:149):
   ```rust
   pub struct SoupReport {
       // … campos existentes
       pub fission_events: Vec<FissionEventRecord>,   // default: empty
   }
   #[derive(Clone, Debug, Serialize, Deserialize)]
   pub struct FissionEventRecord {
       pub tick: u64,
       pub parent: u64,
       pub children: [u64; 2],
       pub dissipated_qe: f32,
   }
   ```
   **`#[serde(default)]`** sobre `fission_events` para preservar compatibilidad con reports JSON pre-AP-6c.

4. **DOT export** `to_dot` (autopoiesis.rs:170): agregar edges `parent → child` por cada evento. Color de nodo se mantiene (verde = sobrevivió, rojo = murió). Raíces = lineage `0`.

**Alcance in-scope:**
- Wiring `apply_fission` al loop del harness (1 branch en ~5 líneas existentes).
- `LineageGrid` paralelo (nuevo, ~80 LOC).
- `FissionEventRecord` + Vec en `SoupReport`.
- Edges en `to_dot`.

**Alcance out-of-scope:**
- Reintegrar `LineageRegistry` como `Resource` Bevy — el harness NO usa Bevy (ADR-040 §3). El registry se reconstruye desde el Vec de eventos cuando/si la viz lo quiere como `Resource`.
- GC de linajes sin descendencia.
- Tree balancing / layout algorithms (responsibility de la viz, no del reporte).
- Multi-fisión simultánea en el mismo tick sobre el mismo blob (escenario patológico — por ahora, primero-que-cruza gana).

## 3. Alternativas consideradas

### A · Solo registrar eventos sin stamping per-cell
- **Cómo:** `fission_events: Vec<{tick, children}>` sin `parent`, sin `LineageGrid`.
- Pros: menor cambio. Cero memory extra por grid.
- Contras: **no se puede saber qué closure fisionó.** Los hijos son IDs sintéticos sin padre real ⇒ el árbol es un bosque de pares disjuntos. Inútil para la viz (no hay continuidad visual "blob rojo → blobs rojo-claro + rojo-oscuro").
- **Descarte:** el tree resultante no contiene la información que la viz necesita.

### B · `LineageRegistry` como `Resource` Bevy + Entity-per-closure
- **Cómo:** spawn entidad por closure, `Lineage` como Component, reuso total del registry.
- Pros: alinea con el modelo ECS del resto del simulador.
- Contras: **el harness es Bevy-free por diseño** (verificación: grep `use bevy` en autopoiesis.rs ⇒ 0 hits). Introducir Bevy rompe AP-5 proptest (headless). Además entidad-por-closure viola ADR-037 §"SoA por celda" — las closures son patrones del grid, no entities.
- **Descarte:** viola invariante 1 del track (_"zero 'Cell' component", README.md:110_).

### C · Per-cell tag vía bits altos de un campo existente
- **Cómo:** esteganografía en `SpeciesCell::freq` o similar.
- Pros: cero memoria extra.
- Contras: frágil, destruye precisión, invita bugs sutiles. Literalmente un anti-patrón.
- **Descarte:** code smell estructural.

### **Elegida — Decisión §2 (stamping + Vec de eventos)**
Combina (i) datos suficientes para reconstruir el tree + (ii) cambio mínimo (no nuevas entidades, no Bevy en harness) + (iii) backward-compat JSON via `#[serde(default)]`. Reutiliza 100% de `apply_fission` / `FissionOutcome` / `child_lineage` (fission.rs:150-189) que ya están testeados.

## 4. Consecuencias

**Positivas:**
- Criterio de cierre del track alcanzable: _"reporta ≥1 fission"_ (README.md:124) ahora es operacional.
- Viz puede dibujar edges `parent → [child_a, child_b]` leyendo `fission_events` + colorear el grid por `LineageGrid`.
- `to_dot` produce un DAG real (no un conjunto de nodos disjuntos).
- Cero Bevy en harness — AP-5 proptest sigue corriendo.

**Negativas:**
- `LineageGrid` duplica dimensiones de `SpeciesGrid` — `w·h·8 bytes` (u64) extra. A 128×128 = 128 KiB. Aceptable.
- `SoupReport` JSON crece por seed con fissions activas. Aceptable — typical run pre-AP-6c tiene `fates.len()` en orden de decenas.
- `dominant_lineage(&blob, &lineage_grid)` es un cálculo nuevo: moda sobre las celdas del blob. O(blob_size), barato.
- Byte-estabilidad: seeds donde **no** hay fission tienen `fission_events: []` ⇒ output idéntico antes/después con serde(default) bien puesto. Seeds con fission cambian deterministically ⇒ AP-5 golden debe regenerarse una vez.

**A vigilar:**
- **Idempotencia de la detección:** `raf_closures` corre cada `detection_every` ticks. Si una closure persiste 5 detection cycles y se fisiona en el 6to, el `lineage_id` original debe ser estable entre cycles. Garantía: `closure.hash` ya es determinístico sobre la topología de reacciones (existe en el código, usado por `fate.hash`); `stamp_initial_lineage(closure) = hash_to_lineage(closure.hash)`.
- **Double-stamping:** una celda puede pertenecer a 2 closures overlapping en el grid. Política: **primero-que-marca gana** por tick de detección. Evaluar si genera sesgo observable en AP-5.
- **Conservación de masa post-fission:** `apply_fission` ya la garantiza (fission.rs:196 "Σ species pre = Σ species post + dissipated_qe"). No hay que revalidar en harness.

**Deuda técnica asumida:**
- `LineageRegistry` (layers/lineage_registry.rs) queda **no usado por el harness**. Lo consume únicamente el ECS Bevy en otros binarios (grep confirma). Dejarlo — mezclar Bevy-resource con harness Bevy-free es peor. `// TODO(ADR-041): harness no usa LineageRegistry — la viz lo reconstruye desde SoupReport.fission_events`.
- Golden JSON de AP-5 cambia en seeds con fission. Regenerar con documentación.

## 5. Plan de implementación

**Pre-requisito:** ADR-040 mergeado (`SoupSim` existe). Esta decisión toca `SoupSim::step()`, no el wrapper batch.

1. **Agregar `FissionEventRecord` + `fission_events` a `SoupReport`** con `#[serde(default)]`. Correr `cargo test autopoiesis` — todo verde, sin cambios de comportamiento.

2. **`src/layers/lineage_grid.rs`** (nuevo): struct + `new(w,h)` + `stamp(&cells, id)` + `dominant(&blob) -> u64` + `get(x,y)`. Unit tests: tier B, 5-7 tests (empty, single stamp, overwrite, dominant con empate → menor id gana).

3. **Función pura** `hash_to_lineage(closure_hash: u64) -> u64` en `blueprint/equations/fission.rs` (hermana de `child_lineage`). Test de determinismo + disjunción respecto a raíz `0`.

4. **`SoupSim::step()`** (ADR-040 §2):
   - Añadir `lineage_grid: LineageGrid` y `fission_events: Vec<FissionEventRecord>` como campos.
   - Al snapshot inicial (hoy autopoiesis.rs:306-317): stampear las celdas del mask con `hash_to_lineage(closure.hash)`.
   - En el bloque pressure-check (hoy autopoiesis.rs:323-334): reemplazar el bool `pressure_crossed` por un loop que, **por cada blob que cruza el umbral**, llama `apply_fission` y pushea al Vec. `track.fate.pressure_events` sigue contando, así que AP-5 no pierde información.

5. **`SoupSim::finish()`** copia `fission_events` al `SoupReport`.

6. **`SoupReport::to_dot`** (autopoiesis.rs:170): agregar, después del loop de nodos, `for ev in &self.fission_events { for child in ev.children { writeln(f, "c{parent:016x} -> c{child:016x} [label=t{tick}]") } }`.

7. **Tests tier B nuevos:**
   - Fission con 1 blob que cruza umbral ⇒ `fission_events.len() == 1`.
   - Children pertenecen a `LineageRegistry` virtual (reconstruido) con `parent` correcto.
   - `to_dot` output parsea con `graphviz -Tsvg /dev/null` (CI, opcional) o al menos `dot.matches('{').count() == dot.matches('}').count()`.
   - Regresión: seed donde hoy no hay fission ⇒ `report.fission_events == []` ⇒ JSON legacy parsea sin error.

**Verificación:**
- `cargo run --release --bin autopoietic_lab -- --network assets/reactions/formose.ron --ticks 5000 --out r.json` produce `fission_events.len() >= 1`.
- `cargo test autopoiesis` verde.
- Golden AP-5 regenerado, diff revisable a mano (~pocos bytes por seed).

**Riesgos de migración:**
- Consumidores JSON externos (si existen). Mitigación: `#[serde(default)]`. Grep `serde_json::from_str::<SoupReport>` muestra 1 hit en el test existente (autopoietic_lab.rs:288) — OK.
- Fissions cascada: fisionar el hijo A antes de que termine el mismo tick. Política: **un fission por blob por detection tick.** Si A cruza presión de nuevo en el **siguiente** detection tick, fisiona ahí. Documentar en el código.

## 6. Referencias

- `SoupReport` + `to_dot`: `src/use_cases/experiments/autopoiesis.rs:147-190`
- `apply_fission` + `FissionOutcome`: `src/blueprint/equations/fission.rs:162-230`
- `child_lineage`: `src/blueprint/equations/fission.rs:150-160`
- `LineageRegistry` (no usado por harness): `src/layers/lineage_registry.rs:30-158`
- Invariante track: `docs/sprints/AUTOPOIESIS/README.md:110` ("zero 'Cell' component") y l.124 (criterio cierre)
- ADR previo: `docs/arquitectura/ADR/ADR-039-fission-criterion.md:62` (_"Cada fisión emite `FissionEvent { parent_lineage, child_lineages: [u64; 2] }`"_ — esta ADR hace operacional esa promesa)

## 7. Notas para el revisor humano (STRIP BEFORE MERGE)

- **Exploración realizada:**
  - Grep `apply_fission` en `src/use_cases/experiments/` ⇒ 0 hits (confirmado gap).
  - Grep `LineageRegistry` en `autopoiesis.rs` ⇒ 0 hits.
  - Leí `fission.rs:150-230` (child_lineage + FissionOutcome + apply_fission) y `lineage_registry.rs:1-50`.
  - Verifiqué Hard Block #13 (no Vec en componentes): `FissionEventRecord` vive en `SoupReport` que NO es Component — OK.

- **Suposiciones que requieren validación:**
  - **[ASSUMPTION]** `closure.hash` ya es determinístico entre detection cycles sobre la misma red. Lo infiero de que `fate.hash` se usa para matching across ticks (autopoiesis.rs:343 `|&h| h == track.fate.hash`) — si no fuera determinista, el tracking de fates ya estaría roto. Confirmar mirando `raf_closures` + Closure struct.
  - **[ASSUMPTION]** `formose.ron` produce fissions en <5000 ticks. No corrí la simulación; el AC del sprint lo afirma (SPRINT_AP6_AUTOPOIETIC_LAB.md:48) pero nadie lo validó empíricamente. Si falla, no es bug de esta ADR sino input empírico del asset — track-level risk.
  - **[ASSUMPTION]** `SoupReport` serializado byte-estable entre runs idénticos (ya flag en §7 de ADR-040).

- **Preguntas abiertas:**
  - ¿`LineageGrid` debería vivir en `src/layers/` (capa ECS) o en `src/use_cases/experiments/autopoiesis/` (adjunto al harness)? Propongo **layers/** — es data paralela al grid, mismo nivel de abstracción que `SpeciesGrid`/`ClosureMembraneMask`. Si se quiere puro "experimento aislado", mover al módulo del harness. Preferencia del usuario.
  - Política "primero gana" en double-stamping: ¿es aceptable o hay que registrar _todos_ los lineages que tocan una celda? Hoy dice que una celda tiene **un** dueño; arbitraje "primero" es simple pero sesga hacia closures antiguas. Alternativa: mayoritario por frecuencia de ocurrencia — pero requiere otro pass. Diferir a retro post-AP-6c.
  - ¿Vale la pena extraer `dominant_lineage` como función pura en `blueprint/equations/` para testear aislada?
