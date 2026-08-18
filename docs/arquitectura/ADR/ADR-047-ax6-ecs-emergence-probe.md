# ADR-047: Prueba comportamental de Ax6 sobre el motor real (entrainment ECS)

**Estado:** Aceptado — probe PASS en 5/5 seeds, ablaciones bit-idénticas (§10)
**Fecha:** 2026-08-17
**Contexto:** EMERGENT_MEASUREMENTS (Sprint EM-1, ítem EM-1.5)
**ADRs relacionados:** ADR-046 (experimento analítico de fase; este ADR ejecuta
su "Fase 4 opcional") · ADR-045 (patrón spike + veredicto)
**Origen:** `docs/rfc/RFC-001-ax6-ecs-emergence-probe.md` (3 rondas de crítica
adversarial; el Registro de crítica queda en el stub del RFC)

## 1. Contexto y problema

ADR-046 probó que **el modelo de Kuramoto sincroniza** — mean-field de fase
analítico, all-to-all, sin ECS (`use_cases/experiments/emergence_sync.rs`). Eso
es un resultado de 1975, no una propiedad de RESONANCE. Sin embargo:

- La matriz (`docs/design/AXIOM_LAYER_VALIDATION_MATRIX.md` §4) declaraba la fila
  `SYNC Kuramoto order param (L2×N) · Ax6 C PASS` — sugiere que L2
  `OscillatorySignature` fue validada. **L2 no participa en ese experimento.**
- El sprint EM-1.4 figuraba `IMPLEMENTED` y el ADR-046 §10 (veredicto) estaba vacío.

El motor real tiene su propia dinámica de sincronización:
`entrainment_system` (`src/simulation/emergence/entrainment.rs:28`, registrado
en `src/plugins/atomic_plugin.rs`), que difiere del experimento analítico
en los tres ejes que importan:

| | `emergence_sync` (analítico) | `entrainment_system` (motor) |
|---|---|---|
| Variable acoplada | fase θ | **frecuencia ω** (`set_frequency_hz`) |
| Régimen | `sin` completo | lineal (`sin x ≈ x`, `kuramoto_pair_delta`) |
| Topología | all-to-all mean-field | vecinos ≤ 8 (`ENTRAINMENT_MAX_NEIGHBOURS`), radio `ENTRAINMENT_SCAN_RADIUS = 12.0`, coupling decae con distancia (AC-4, Ax7) |

**Hipótesis nula (H0):** *el orden global aparece con o sin la regla local del
motor* — la sincronización no es consecuencia de la interacción N−1.

**Claim a probar:** el orden global (colapso de la dispersión de frecuencias de
una población L2) emerge de la regla local del motor real, y desaparece al
ablarla. Hasta que esto se mida, el par (L2, Ax6) es `GAP`, no `PASS`.

## 2. Experimento diseñado (ítem EM-1.5)

Test de integración headless (`MinimalPlugins`, sin GPU) que corre **el system
real** — la misma fn que registra `AtomicPlugin` — sobre N entidades con
`OscillatorySignature` real y `SpatialIndex` real, con **dos ablaciones
causales**:

- **A1 — ablación de regla:** mismo estado inicial, el system no se agrega al
  schedule. Contrafáctico N−1→N puro (estilo ADR-046).
- **A2 — ablación por alcance (gate espacial):** system ACTIVO, misma
  población, layout disperso a `spacing = 3 × ENTRAINMENT_SCAN_RADIUS`. A esa
  distancia el broadphase no devuelve ningún vecino — filtro duro
  `distance ≤ scan_radius + r` (`spatial_index_backend/mod.rs:89`; 36 > 12.5) —
  así que la regla **corre cada tick con vecindad vacía** y ω nunca se ajusta.
  Prueba que el orden requiere *interacción efectiva*, no la mera presencia del
  código en el schedule.
  **Encuadre honesto:** la supresión es el cutoff duro del radio de scan (Ax7
  como *alcance acotado*), NO la atenuación exponencial continua. Si el decay
  fuera el supresor, el sistema aún convergería
  (`K_eff(36) = 0.15·e⁻³ ≈ 0.0075` → S ≈ 0.95 a T grande) y el control
  fallaría. Por eso A2 aserta `S == 0` exacto, bit-idéntico a A1 en el
  resultado aunque el code path sea distinto (query + guard corren).

### 2.1 Métrica (la decisión central)

El order parameter de fase `R` **no aplica**: el system acopla ω, no θ (las
fases quedan con offsets constantes; `R` no colapsa limpio — exactamente el
contra que ADR-046 §3 le puso a su opción B). La métrica honesta para
*frequency entrainment* es el **colapso de la dispersión de frecuencias**:

```
S = 1 − σ_ω(T) / σ_ω(0)   ∈ [0, 1]
```

con `σ_ω(0)` **medida** sobre la población spawneada (σ muestral de N draws,
±~9 % entre seeds), no el nominal derivado.

| Observable | Métrica | Tolerancia |
|---|---|---|
| Orden acoplado | `S` (layout denso, regla activa) | `≥ SYNC_ECS_S_PASS_MIN` (0.5) en **todas** las seeds |
| Orden ablado A1 | `S` (system fuera del schedule) | `== 0` exacto; cota formal `≤ SYNC_ECS_ABLATED_S_MAX` (0.1) |
| Orden ablado A2 | `S` (vecindad vacía) | idem A1 |
| Métrica secundaria | fracción de pares vecinos con `entrainment_lock_achieved(ω_i, ω_j, KURAMOTO_LOCK_THRESHOLD_HZ)` | reportada, sin umbral |

**Protocolo de aserción del `S == 0`** — la suma f32 no es asociativa, el "0
exacto" exige recolección canónica: leer ω ordenado por `Entity::index` en ambas
mediciones y asertar identidad bit a bit del vector ω(0) vs ω(T) vía
`hash_f32_slice` (`determinism.rs:11`, precedente r2); `S == 0.0` se sigue
trivialmente. La cota `SYNC_ECS_ABLATED_S_MAX` vive sólo en el verdict,
tolerante a futuras fuentes de jitter.

**Hogar de la métrica — fns puras, no inline en el test** (regla 12), en
`blueprint/equations/emergence/synchronization.rs` con unit tests propios
(patrón ADR-046 §7-8: el test de integración *llama* la métrica, no la define):

- `frequency_std(freqs: &[f32]) -> f32` — σ muestral, convención `n−1`
  declarada (para S es indistinto; para el σ reportado en §10 no, ~0.8 %).
- `frequency_collapse_s(sigma_0, sigma_t) -> f32` — bordes definidos al estilo
  del módulo: `sigma_0 ≤ 0 → 0.0`, entradas no-finitas → `0.0`, resultado
  `.clamp(0.0, 1.0)` (cubre N=1: σ=0 → S=0). Unit test por borde.
- `sync_ecs_verdict(s_coupled, s_ablated) -> bool` — misma forma que
  `emergence_sync_verdict`; con dos ablaciones se llama **una vez por cada una**:
  `verdict(s, s_a1) && verdict(s, s_a2)`.

`tests/axioms/r10_emergence_gates.rs` se extiende con los gates de rango y
consistencia de las constantes `SYNC_ECS_*` — así las constantes tienen
consumidores en `src/` y validación de boundary, como sus hermanas `SYNC_*`.

**Piso de σ(T) — por qué el PASS es 0.5 y no 0.9:** el guard de lock es por
PAR (`entrainment.rs:81-86`: skip si *todos* los vecinos están a
`< KURAMOTO_LOCK_THRESHOLD_HZ = 1.0`). Eso admite estados congelados tipo
"escalera": gradiente ≤ 1 Hz por hop sostenido a lo largo del diámetro del
grafo (~7 hops en la grilla 8×8) → rango global congelado de hasta ~7 Hz,
σ(T) ≈ 2 Hz en el peor caso → `S ≈ 0.75` para *cualquier* estado final
totalmente locked. `SYNC_ECS_S_PASS_MIN = 0.5` queda debajo de ese piso con
margen para convergencia parcial; el spike mide el S real.

### 2.2 Escenario

- **N = 64** entidades: `(OscillatorySignature::new(ω_i, 0.0), SpatialVolume,
  Transform)`. **φ es don't-care declarado**: el system no lee fase (snapshot
  toma sólo `frequency_hz`, `entrainment.rs:34-39`; muta sólo
  `set_frequency_hz`, `:99`) y S es función sólo de ω → fase constante 0.
- **`ω_i = SYNC_ECS_CENTER_HZ + gaussian_f32(...)`** — `gaussian_f32` centra
  en 0 (`determinism.rs:57-62`) y L2 clampea a `max(0.0)`
  (`oscillatory.rs:43-47,64-69`): sin el offset la gaussiana queda
  **rectificada** (σ(0) ≈ 4.6, masa de osciladores congelada en 0 Hz) y el
  experimento se corrompe en silencio. El centro es don't-care dinámico (la
  regla lineal sólo ve diferencias de ω); su única restricción real es
  `center − 3·spread > 0` (clamp inerte), gateada en r10.
  El stream avanza **dos `next_u64` por draw** — `gaussian_f32` consume dos
  estados internos; avanzar de a uno reproduce la correlación intra-stream del
  bug de `emergence_sync.rs::init_state` (el `u2` del draw k pasa a ser el `u1`
  del draw k+1). **Ese bug se corrigió en el mismo cambio** (ver §7).
- **Layout denso: grilla 8×8 con `spacing = 8.0`** (⅔ del scan radius). A ese
  spacing un nodo interior tiene exactamente 8 candidatos en rango (4
  ortogonales a d=8, 4 diagonales a d=11.31; d=16 queda fuera) →
  `ENTRAINMENT_MAX_NEIGHBOURS = 8` **no trunca** y el grafo es el king-graph
  simétrico. A spacing 6 habría 12 candidatos y la truncación retendría los 8
  de menor índice de spawn (`query_radius` ordena por `Entity::to_bits`,
  `spatial_index_backend/mod.rs:97`) — grafo dirigido sesgado, evitado por
  construcción. `SpatialEntry` con radio 0.5 (infla el filtro a `d ≤ 12.5`;
  mantiene 11.31 adentro y 16 afuera). El radio que gobierna es el de
  `SpatialEntry` al insertar en el índice — el system NO lee `SpatialVolume`;
  trampa de copia: los tests del módulo insertan `radius: 1.0`
  (`entrainment.rs:142`).
- Layout A2: misma grilla con `spacing = 36.0`.
- **Estructura de runs:** acoplado, A1 y A2 son **tres Apps separadas** creadas
  por un builder compartido parametrizado (`spacing`, `with_system: bool`), con
  los MISMOS draws de ω por seed (garantizado por el PCG determinista);
  `σ_ω(0)` se mide por-run sobre las ω efectivamente spawneadas.
- `SpatialIndex` construido una vez — las entidades no se mueven (sin
  `FlowVector`) — **replicando** el patrón del `#[cfg(test)] build_index` del
  propio módulo (`entrainment.rs:119-129`; helper privado, no importable) vía
  API pública `SpatialIndex::new(ENTRAINMENT_SCAN_RADIUS)` + `insert`.
  `SimWorldTransformParams::default()` (plano XY).
- **Schedule: `add_systems(Update, entrainment_system)` + `SYNC_ECS_TICKS` ×
  `app.update()`** — el patrón real de `tests/axioms/r2_determinism.rs:29-40` y del
  `#[cfg(test)]` del propio módulo (`entrainment.rs:144`). **No usar
  `FixedUpdate` con `MinimalPlugins`**: corre 0..k veces por `update()` según
  wall-clock → ni conteo de ticks garantizado ni determinismo entre máquinas.
- Multi-seed: 5 seeds `[1, 2, 3, 100, 777]` (paridad con
  `tests/axioms/emergence_sync.rs::emergence_holds_across_seeds`); seed 11 para los
  tests de una sola corrida.

### 2.3 Determinismo

Un solo system en el schedule → sin ambigüedad de orden de ejecución. Las
fuentes de orden determinista son: (a) orden de spawn estable → orden de
iteración de query estable (precedente `tests/axioms/r2_determinism.rs`), y (b)
`query_radius` ordena vecinos por `Entity::to_bits`
(`spatial_index_backend/mod.rs:97`). Nota: el `sorted_snapshot` interno del
system (`entrainment.rs:49-51`) es un índice de lookup para `binary_search`,
no la garantía de orden — la atribución correcta es (a)+(b). El test incluye
una aserción de reproducibilidad bit a bit entre dos runs de la misma seed.

## 3. Decisión y alternativas

**Decisión:** probe del system real en schedule mínimo (opción A).

| Opción | Descripción | Contras |
|---|---|---|
| **A (elegida)** | `entrainment_system` solo, schedule `Update` mínimo, `SpatialIndex` estático | No cubre gating/fases del pipeline (deliberado: aísla la causa) |
| B | Probe sobre `AtomicPlugin` completo (pipeline real con fases y run-conditions) | Confounds: cualquier system de la cadena física puede tocar ω o mover entidades; la ablación deja de ser un contrafáctico limpio; más caro |
| C | No medir; dejar Ax6 como guard-rail estructural | El sobreclaim de la matriz (SYNC C PASS sin tocar L2) queda en pie — inaceptable |

## 4. Constantes nuevas

`src/blueprint/constants/synchronization_a6.rs` (mismo archivo — es el mismo
axioma; prefijo `SYNC_ECS_*`, que agrupa léxicamente con las `SYNC_*`
existentes — no existe prefijo `ECS_*` en el repo):

| Constante | Valor | Derivación |
|---|---|---|
| `SYNC_ECS_SPREAD_RATIO` | 8.0 | spread inicial en unidades del umbral de lock del motor: `≫ 1` garantiza arranque desordenado en la escala de la dinámica bajo prueba; `≪ COHERENCE_BANDWIDTH / KURAMOTO_LOCK_THRESHOLD_HZ = 50` mantiene la población dentro de la banda de coherencia. Ambas cotas gateadas en r10 |
| `SYNC_ECS_SPREAD_HZ` | 8.0 Hz | `SYNC_ECS_SPREAD_RATIO × KURAMOTO_LOCK_THRESHOLD_HZ` — derivada de la dinámica bajo prueba, NO de un ratio post-hoc sobre `COHERENCE_BANDWIDTH` |
| `SYNC_ECS_S_PASS_MIN` | 0.5 | ≤ piso `S ≈ 0.75` del peor estado locked (escalera congelada, §2.1), con margen para convergencia parcial |
| `SYNC_ECS_ABLATED_S_MAX` | 0.1 | cota formal; A1/A2 esperan `S == 0` exacto |
| `SYNC_ECS_TICKS` | 400 | empírico inicial (ver abajo) |
| `SYNC_ECS_CENTER_HZ` | 75.0 Hz | frecuencia central de la población. Don't-care dinámico (la regla lineal es invariante a traslación en ω); la única restricción física es `center − 3·spread > 0` para que el clamp `max(0.0)` de L2 sea inerte (gate en r10). 75.0 por precedente de los tests del módulo (`entrainment.rs:137,161`) |

**Derivación de ticks (honesta):** la dinámica es contracción **exponencial**,
no lineal. Por par: `t ≈ ln(spread/lock) / (2·K_eff) = 2.079/0.154 ≈ 13.5`
ticks con `K_eff(d=8) = 0.15·e^(−8/12) ≈ 0.077`. Pero la población converge
por el modo más lento del grafo (gap espectral del laplaciano del king-graph
8×8, con la normalización `1/n` del paso — n = |vecinos| ≤ 8, no la población
N (`equations/emergence/entrainment.rs:55,64`)): tasa ~O(0.01)/tick → escala
O(10²-10³). **400 es un valor empírico inicial** (deja el residuo del modo
lento en ~e⁻⁴ del spread), no una derivación cerrada.

`[ASSUMPTION]` `SYNC_ECS_SPREAD_RATIO = 8.0` es una calibración dentro de las
cotas derivadas (1, 50); los umbrales S fueron propuestos analíticamente y
**confirmados** por el spike (§10) sin ajuste post-hoc.

## 5. No viola axiomas

| Ax | Cumplimiento |
|---|---|
| 1 Energy | El probe lee/escribe sólo ω (L2); no crea ni consume qe |
| 2 Pool / 5 Conservation | No hay pools ni transferencias en el schedule mínimo |
| 3 Competition | No aplica — sin competencia por energía |
| 4 Dissipation | El system bajo prueba no toca qe; el test no introduce eficiencias |
| 6 **Emergence** | Eje central: S es propiedad de escala; A1/A2 son los contrafácticos N−1→N |
| 7 Distance | A2 usa el alcance acotado real del motor (scan radius); el decay continuo queda declarado fuera de alcance (§9) |
| 8 Oscillatory | L2 real es el sustrato; el system ES la consecuencia AC-2 de Ax8 |

## 6. Costos

- **Compilación:** **0 test targets netos** — el archivo nació como target propio
  (`tests/emergence_ecs.rs`, autodiscovery) y ADR-048 lo consolidó como módulo
  de la suite `axioms` (`tests/axioms/emergence_ecs.rs`). La convención del repo
  sigue siendo `[[test]]` explícito sólo para `required-features`.
- **Runtime:** 64 entidades × 400 ticks × 3 condiciones × 5 seeds, sin GPU.
- **Código:** +3 fns puras y +6 constantes. Complejidad baja.

## 7. Archivos

| Archivo | Cambio |
|---|---|
| `tests/axioms/emergence_ecs.rs` | NUEVO — probe + A1 + A2 + determinismo + multi-seed + guards estructurales |
| `src/blueprint/equations/emergence/synchronization.rs` | + `frequency_std` + `frequency_collapse_s` + `sync_ecs_verdict` puras con unit tests de bordes |
| `src/blueprint/constants/synchronization_a6.rs` | + 6 constantes `SYNC_ECS_*` |
| `tests/axioms/r10_emergence_gates.rs` | + gates de rango/consistencia de `SYNC_ECS_*` (consumidores + boundary) |
| `src/use_cases/experiments/emergence_sync.rs` | **fix pre-existente:** `init_state` avanzaba un solo estado PCG por draw gaussiano; `gaussian_f32` consume dos (Box-Muller) → el `u2` del draw k era el `u1` de la fase k+1 (correlación intra-stream ω↔θ). Ahora avanza tres estados por oscilador |
| `docs/design/AXIOM_LAYER_VALIDATION_MATRIX.md` | fila SYNC dividida: `SYNC-analytic` (referencia, no motor) + `SYNC-ECS (L2×N)` |
| `docs/arquitectura/ADR/ADR-046-...md` | §10 llenado **sólo** con los números analíticos; §9 anotada "Fase 4 ejecutada vía ADR-047" |
| `docs/sprints/EMERGENT_MEASUREMENTS/SPRINT_EM1_THREE_FIGURES.md` | EM-1.4 corrección de status/claim; **EM-1.5 nuevo** |
| `docs/rfc/RFC-001-...md` | stub `Superseded → ADR-047`, conservando el Registro de crítica |
| `Cargo.toml` | **sin cambios** — autodiscovery cubre tests no gated |

**No se tocó:** `entrainment_system` ni ninguna constante existente del motor.

## 8. Tests

- **Unit** (`cargo test --lib blueprint::equations::emergence::synchronization`):
  `frequency_std` (n−1 vs n, n<2, no-finitos, invarianza a traslación, valor
  conocido), `frequency_collapse_s` (colapso total, sin cambio, σ₀≤0,
  no-finitos, clamp al crecer), `sync_ecs_verdict` (boundary).
- **Integration** (`cargo test --test axioms emergence_ecs::`): guards estructurales
  (king-graph sin truncación, A2 sin vecinos, población no rectificada),
  acoplado colapsa en 5 seeds, A1 y A2 bit-idénticos, veredicto por ablación,
  reproducibilidad bit a bit.
- **Gate** (`cargo test --test axioms r10_emergence_gates::`): rangos + consistencia de
  derivación + `center − 3·spread > 0` + boundary de `sync_ecs_verdict`.

## 9. Qué NO es este ADR / decisión revisable cuando

- No valida el pipeline completo (`AtomicPlugin` + fases + gating) — sólo la
  transformación del system. El acople con
  `update_spatial_index_after_move_system` y el run-condition quedan fuera.
- No mide la transición crítica `R(K)` ni ruido (ADR-046 §9, sigue diferido);
  el barrido de `KURAMOTO_BASE_COUPLING` es hoy imposible sin parametrizar el
  system (requeriría refactor propio).
- No mide la atenuación *continua* de Ax7 (A2 es gate duro) — un barrido
  S(spacing) intra-rango queda como extensión futura.
- No ejercita el **régimen de truncación** de vecinos (>8 candidatos, con su
  sesgo por spawn index) — el spacing 8 se elige justamente para excluirlo;
  cubrirlo requeriría su propio diseño.
- No reemplaza el experimento analítico: queda como **referencia** contra la
  cual se compara el comportamiento del motor.
- Revisable si Bevy cambia el orden de iteración de queries o el backend
  espacial su orden (`to_bits`) — la aserción de determinismo lo detecta — o si
  se agrega ruido térmico / movimiento (invalida el índice estático).

## 10. Veredicto del spike

**PASS — H0 refutada para el motor real. Ax6 medido sobre L2×N.**

Medido con `cargo test --test axioms emergence_ecs:: -- --nocapture --test-threads=1`
(N=64 en grilla 8×8, ω ~ N(75, 8) Hz, 400 ticks de `Update`, tres Apps por seed).
Runtime de las 8 pruebas: **1.20 s** (criterio §8: < 60 s, sin GPU, un solo binario).

| seed | σ_ω(0) | σ_ω(400) | **S acoplado** | S A1 (regla) | S A2 (alcance) | lock frac (acoplado) |
|---:|---:|---:|---:|---:|---:|---:|
| 1 | 7.8203 | 1.0990 | **0.8595** | 0.0000 | 0.0000 | 1.0000 |
| 2 | 8.9090 | 1.1234 | **0.8739** | 0.0000 | 0.0000 | 1.0000 |
| 3 | 7.7011 | 1.1741 | **0.8475** | 0.0000 | 0.0000 | 1.0000 |
| 100 | 8.4415 | 1.7887 | **0.7881** | 0.0000 | 0.0000 | 1.0000 |
| 777 | 8.0786 | 1.0768 | **0.8667** | 0.0000 | 0.0000 | 1.0000 |
| **media** | **8.190** | **1.252** | **0.8471** | **0.0000** | **0.0000** | **1.0000** |

**Criterios de aceptación:**

1. ✅ Runtime 1.20 s < 60 s, headless, un solo test target.
2. ✅ Acoplado `S ≥ SYNC_ECS_S_PASS_MIN (0.5)` en **5/5 seeds** — mínimo observado
   0.7881, margen de +0.29 sobre el umbral.
3. ✅ A1 y A2: identidad **bit a bit** de ω(0) vs ω(T) vía `hash_f32_slice` en
   5/5 seeds → `S == 0.0` exacto (no "≈ 0"). El code path de A2 es distinto (query
   + guard corren cada tick) y aun así el resultado es idéntico al de A1.
4. ✅ Determinismo bit a bit entre dos corridas de la seed 11 (hash ω(0), hash ω(T)
   y `S.to_bits()` idénticos).
5. ✅ `cargo test --test axioms r10_emergence_gates::` 16/16 con los 9 gates `SYNC_ECS_*`
   nuevos (rangos, derivación `spread = ratio × lock`, cotas (1, 50) del ratio,
   `center − 3·spread > 0`, gap, boundary del verdict).
6. ✅ Guards estructurales verdes: king-graph **sin truncación** (interiores con
   exactamente 8 vecinos, ninguno por encima del cap), A2 con **0 vecinos** en los
   64 nodos, población inicial nunca rectificada por el clamp de L2.

**Lecturas del resultado.**

- **La predicción de §2.1 se cumplió, incluido su mecanismo.** El estado final
  tiene `lock fraction = 1.0000` — la población termina **completamente lockeada**,
  que es exactamente el régimen de "escalera congelada" que fija el piso teórico
  `S ≈ 0.75`. El S medido (0.788–0.874) queda apenas por encima de ese piso, con
  σ(T) ≈ 1.1–1.8 Hz ≈ 1–2 × `KURAMOTO_LOCK_THRESHOLD_HZ`. No es convergencia
  parcial: es el atractor real del guard por pares. Por eso `SYNC_ECS_S_PASS_MIN`
  se fijó en 0.5 y no en 0.9 — y por eso **no hubo que ajustar nada** post-hoc.
- **400 ticks alcanzan el plateau, no lo rozan.** Con lock fraction 1.0 el sistema
  ya no puede evolucionar: todos los nodos entran en el `continue` del guard. El
  riesgo "convergencia del modo lento > 400 ticks" (§11) no se materializó.
- **La seed 100 es la peor y sigue pasando** (S=0.7881, σ(T)=1.7887): su escalera
  congelada quedó más ancha, consistente con la varianza esperada del estado final.
- **A2 aísla el gate duro, no el decay.** `locked fraction = 0.0000` en A2 no
  significa "ningún par lockeado" sino que **no hay pares vecinos** (denominador 0
  → la fn devuelve 0.0 por contrato). En A1 la métrica sí tiene denominador y da
  0.043–0.081: esa es la fracción de pares que *nacen* accidentalmente dentro de
  1 Hz, la línea de base contra la que el 1.0000 del acoplado se lee.

**Consecuencia para la matriz:** el par (L2, Ax6) deja de ser implícito. La fila
`SYNC` de `AXIOM_LAYER_VALIDATION_MATRIX.md` se parte en `SYNC-analytic`
(referencia, no toca el motor) y `SYNC-ECS (L2×N)` (`K PASS`, este ADR).

## 11. Riesgos

| Riesgo | Mitigación |
|---|---|
| El guard de lock congela una "escalera" y S queda entre 0.5 y 0.75 | Está dentro del PASS por diseño (§2.1); el valor exacto se reporta en §10 |
| Convergencia del modo lento > 400 ticks | Se reporta S(400) real; si `0.1 < S < 0.5`, es null-result documentado, no excusa para subir ticks hasta que pase |
| Cambiar N/spacing reintroduce truncación de vecinos | Guard estructural en el test: `query_radius` **incluye al propio nodo** (d=0 pasa el filtro) → se cuenta con `filter(entry.entity != self)` y se aserta `≤ ENTRAINMENT_MAX_NEIGHBOURS` (`== 8` para interiores) |
| Query iteration order cambia entre versiones de Bevy | Aserción de determinismo lo detecta |
| Tentación de usar `FixedUpdate` "por realismo" | §2.2 lo prohíbe con la razón (0..k runs por update con MinimalPlugins); el gating real ya lo cubre `atomic_plugin` |
