# ADR-048: Consolidación de test binaries — 37 → 6 targets

**Estado:** Aceptado — implementado y verificado 2026-08-18 (§10)
**Fecha:** 2026-08-18
**Origen:** `docs/rfc/RFC-002-test-binary-consolidation.md` (3 rondas de crítica
adversarial; el Registro de crítica queda en el stub del RFC)
**ADRs relacionados:** ADR-047 (creó `tests/emergence_ecs.rs`, consolidado aquí)
· ADR-042 (precedente de infra con ADR)
**Relación:** continúa la mitigación de OOM de 2026-08-17 (`[profile.test]
debug=0`, `jobs=4`, `test=false` en 29 bins) · implementa la política RAM de
`AXIOM_LAYER_VALIDATION_MATRIX.md` §3 a nivel estructural

---

## 1. Contexto y problema

`tests/` contenía 37 archivos (36 originales + `emergence_ecs.rs` de ADR-047)
→ cargo compilaba y linkeaba **37 binarios independientes**, cada uno
estatizando Bevy 0.15 completo. Tras los fixes de config quedaban ~62 link
units en `cargo test`; los 35 test binaries no gated eran el multiplicador
dominante restante (~85 % del costo de link). En una máquina de 16 GB eso
define el techo de lo corrible.

Costo secundario: overhead por proceso (cargo corre cada binario en serie), y
~36 artefactos por perfil en disco.

## 2. Decisión

Consolidar los **35 archivos no gated** en **4 suites** usando el layout
estándar de cargo (subdirectorio con `main.rs` = un solo target de
integración). Los 2 gated (`bridge_optimizer_equivalence`,
`gpu_cell_field_snapshot_palette_dispatch`) permanecen como binarios propios —
`required-features` es por target y no puede coexistir con tests no gated en el
mismo binario.

Resultado: **6 targets** (4 suites + 2 gated) — de 37 a 6, −31 links de Bevy.

### 2.1 Las 4 suites (criterio de asignación reproducible)

**`tests/axioms/`** (14) — gates axiomáticos, property tests y equivalencias.
Criterio: el assert central es una invariante de axioma o un umbral derivado
(varios usan App+`MinimalPlugins`, p.ej. r1/r2 — el criterio es el *tema*, no
el peso de runtime):

```
r1_conservation, r2_determinism, r4_calibration, r5_sensitivity,
r6_observability, r7_morph_robustness, r8_surrogate, r9_ci_gates,
r10_emergence_gates, property_conservation, property_autopoiesis,
emergence_sync, emergence_ecs, chemistry_equivalence
```

**`tests/probes/`** (7) — probes de arquetipos + materialización (App + spawn
+ N updates, observación de comportamiento):

```
probe_animal, probe_celula, probe_mono_constructal, probe_planta, probe_virus,
senescence_materialization_check, test_senescence_insert
```

**`tests/pipeline/`** (7) — integración del pipeline de simulación (App
completa, cadena de fases). Nombre `pipeline`, no `integration`: todo `tests/`
es integración por definición de cargo, 3 miembros ya llevan sufijo
`_integration` (evita `--test integration sf_integration::`), y "pipeline" es
el vocabulario del repo (CLAUDE.md §Pipeline):

```
sf_integration, energy_competition_integration, morphogenesis_integration,
planet_viewer_integration, field_convergence, input_event_flow,
pathfinding_multi_waypoint
```

**`tests/platform/`** (7) — contratos de plataforma, patrones y assets;
runtime trivial (no todos "estáticos": `require_marker_hierarchy` levanta
App+`MinimalPlugins` — el criterio es *contrato de infraestructura*, no
cero-App):

```
require_marker_hierarchy, sparse_set_transient_markers, g11_strong_ids,
q3_pub_field_api, t9_terrain_config_hot_reload, wgsl_cell_field_snapshot_valid,
demo_flow_maps
```

(`demo_flow_maps` asignado a platform: es contrato estático de assets RON
contra `docs/guides/DEMO_FLOW.md` — cero App, cero pipeline.)

Verificado: 35 no gated asignados exactamente una vez; cero test fns
duplicadas por suite; cero `DefaultPlugins`/render en `tests/` (todas las Apps
son `MinimalPlugins`; `planet_viewer_integration` ni siquiera levanta App); el
`#![cfg(test)]` de `chemistry_equivalence.rs:20` es inner attr válido como
módulo.

### 2.2 Mecánica (por suite)

0. **Gate duro:** `git status --porcelain tests/` vacío antes de empezar.
   `git mv` sobre un archivo untracked falla con `fatal: not under version
   control`, y mover archivos con diff pendiente mezcla contenido y move en el
   mismo diff.
1. `git mv tests/<file>.rs tests/<suite>/<file>.rs` — **git mv, no
   delete+add**, para preservar historia/blame.
2. `tests/<suite>/main.rs` nuevo: doc-comment de la suite + `mod <file>;` por
   archivo. Sin `fn main` (el harness de libtest la genera).
3. **`Cargo.toml` sin cambios** — cargo edition 2024 autodescubre
   `tests/<suite>/main.rs` como target `<suite>` (`autotests` default true;
   las 2 entradas gated existentes no lo desactivan). Convención fijada en
   ADR-047: `[[test]]` explícito SOLO para `required-features`.
4. Los archivos movidos **no cambian por dentro**: importan `resonance::*` como
   crate externo igual que antes; cada uno pasa a ser un módulo del binario de
   la suite, con su namespace propio.

### 2.3 proptest-regressions — la convención real

El default de proptest es `FileFailurePersistence::SourceParallel`
(proptest 1.11.0): sube el árbol desde el source file hasta el primer
directorio que contenga `lib.rs`/`main.rs` y persiste en un directorio
**hermano** `proptest-regressions/` con extensión `.txt`. Los sidecars vivían
como `tests/<name>.proptest-regressions` SOLO porque `tests/` no tenía
`main.rs` (fallback `WithSource`). **Al crear `tests/axioms/main.rs` la
resolución flipa** a `tests/proptest-regressions/<name>.txt` — mover el sidecar
junto al `.rs` (el plan ingenuo) lo dejaría huérfano y las regresiones
guardadas se perderían en silencio.

Protocolo aplicado (empírico primero, move después) — resultados en §10:

1. Probe **descartable** `tests/axioms/persistence_probe.rs` (proptest que
   siempre falla) + `mod` temporal → observar dónde escribe proptest. Borrar
   probe Y sidecar. El probe nunca entra al diff final.
2. `git mv` de los 2 sidecars existentes a la ruta observada.
3. Confirmar que proptest **lee** las seeds desde la ruta nueva.

La verificación "que no aparezca un sidecar nuevo" NO sirve: el archivo solo
se escribe on-failure, esa comprobación pasa siempre.

### 2.4 Runtime RAM y paralelismo intra-binario (el trade-off real)

Antes: 37 procesos **seriales** (cargo corre un binario a la vez), cada uno con
paralelismo interno. Después: 1 proceso por suite con **todos** sus tests en
threads paralelos → el pico de RAM en runtime puede subir (varias `App` Bevy
concurrentes en `probes`/`pipeline`).

Mitigación: todas las Apps de test usan `MinimalPlugins` (sin render/GPU,
~decenas de MB por App). Si una suite pica, el knob es
`cargo test --test pipeline -- --test-threads=4` — runtime, no estructura.
Este ADR **no** agrega serialización global. Medido en §10: ninguna suite
excede 0.7 s de runtime; el trade-off no se materializó.

### 2.5 Invocación — mapa viejo → nuevo

| Antes | Después |
|---|---|
| `cargo test --test emergence_sync` | `cargo test --test axioms emergence_sync::` |
| `cargo test --test property_autopoiesis` | `cargo test --test axioms property_autopoiesis::` |
| `cargo test --test emergence_ecs` | `cargo test --test axioms emergence_ecs::` |
| `cargo test --test sf_integration` | `cargo test --test pipeline sf_integration::` |
| `cargo test --test r10_emergence_gates` | `cargo test --test axioms r10_emergence_gates::` |

El filtro por prefijo de módulo (`<mod>::`) reproduce la granularidad anterior
exacta (libtest matchea por substring; sin módulos anidados en las suites, el
prefijo es unívoco). La escalera de la matriz (§3 paso 4: "un archivo a la
vez") pasa a ser "un módulo a la vez" — mismo costo de runtime, y el costo de
link del paso 4 baja de 1-de-37 a 1-de-4 posibles binarios.

## 3. Alternativas consideradas

| Opción | Contras |
|---|---|
| **A (elegida):** 4 suites por afinidad | los nombres `--test <archivo>` cambian (mitigado por filtros §2.5) |
| B: 1 solo binario `tests/all/` | pierde el knob de granularidad de la escalera; runtime RAM concentrada; un fallo de compilación bloquea TODO el testing |
| C: no consolidar, solo `jobs=4` | deja 35 links de Bevy; cada `cargo test` completo sigue costando ~30-40 min de link en frío |
| D: mover integración a `#[cfg(test)]` en `src/` (unit) | cambia visibilidad (los tests dejan de consumir la API pública como crate externo — pierde el valor de contrato de `q3_pub_field_api` et al.) |
| E: 3 suites (platform fusionada en pipeline) | ahorra 1 link, pero pierde el escalón de runtime ~0 de la escalera: platform corre en segundos sin Apps de pipeline; fusionada, cada corrida barata paga el arranque caro |
| F: 5 suites (partir probes/pipeline por peso) | +1 link sin ganancia: el pico de RAM runtime ya se acota con `--test-threads` (§2.4), no hace falta partición estructural |

## 4. No viola axiomas

Todos N/A razonado: el cambio es **topología de build** — ningún axioma
gobierna la organización de test targets, y el criterio de aceptación 3
(cuerpos de tests sin cambios, solo `git mv`) garantiza mecánicamente cero
efecto de simulación. Ningún threshold, constante ni system se toca.
Verificado en §10: los 37 moves figuran como renames puros (`R`) en git.

## 5. Referencias actualizadas

Re-grep obligatorio antes de editar, con un patrón que capture flags
intercalados y menciones sin prefijo:

```
grep -rnE -- '--test +[a-z0-9_]+' docs/ scripts/ .github/
```

(el patrón ingenuo `"cargo test --test"` pierde `--no-run --lib --test`,
`--release --test` y `` `--test x` `` inline.)

| Locus | Cambio |
|---|---|
| `.github/workflows/autopoiesis-nightly.yml:78` | build step `--test property_autopoiesis` → `--test axioms` |
| `.github/workflows/autopoiesis-nightly.yml:86` | run step → `--test axioms property_autopoiesis:: -- --nocapture` |
| `.github/workflows/autopoiesis-nightly.yml:23,37` | **`paths:` triggers** `tests/property_autopoiesis.rs` → `tests/axioms/property_autopoiesis.rs` (ver §10, hallazgo no previsto por el RFC) |
| `docs/design/AXIOM_LAYER_VALIDATION_MATRIX.md` §2 (tabla de tipos), §3 (escalera paso 4), fichas §5, apéndice §7, contrato de costo | sintaxis nueva |
| `docs/arquitectura/ADR/ADR-047-ax6-ecs-emergence-probe.md` | comandos + rutas + §6 costos (0 targets netos) |
| `docs/arquitectura/ADR/ADR-046-emergence-sync-axiom6.md` | comandos §8/§10 + rutas |
| `docs/arquitectura/ADR/ADR-045-chemistry-canonical-choice.md` | comando §10 + ruta §5 |
| `docs/sprints/EMERGENT_MEASUREMENTS/SPRINT_EM1_THREE_FIGURES.md` | comandos EM-1.4/EM-1.5 + rutas |
| `docs/sprints/AUTOPOIESIS/SPRINT_AP6_AUTOPOIETIC_LAB.md:152` | sprint ACTIVO — comando F-2 |
| `docs/regulatory/01_foundation/PROBLEM_RESOLUTION.md:209`, `SOFTWARE_MAINTENANCE_PLAN.md:201` | comando property_conservation |
| `CLAUDE.md` §Testing | 4 suites + patrón de filtro + regla "los tests nuevos van dentro de una suite" |
| Sprints archivados (`SPRINT_SF7`, `SPRINT_BS1`) | NO tocados — registro histórico |

`ci.yml:34` usa `--workspace` (no afectado); no hay Makefiles/justfiles/tasks
que invoquen `--test <nombre>`.

## 6. Costos

- **Implementación:** ~1-2 h mecánicas (git mv + 4 `main.rs` + grep de docs) +
  2 builds. Presupuestados en ~30-40 min c/u en frío con `jobs=4`; **medidos en
  43 s y 10 s** con `target/` caliente (§10).
- **Riesgo:** bajo — reversible con `git mv` inverso; el protocolo §2.3 es el
  único paso con pérdida potencial y está blindado.
- **Beneficio:** −31 links de Bevy por `cargo test`; suite completa pasa de
  inviable (OOM) a corrible en 16 GB.

## 7. Criterios de aceptación

1. `cargo metadata`: exactamente 6 test targets (4 suites + 2 gated),
   `Cargo.toml` sin diff.
2. **Cero tests perdidos**: (a) pre-move, `cargo test --tests --no-run` y
   después `cargo test --test <name> -- --list | grep -c ': test$'` por cada
   uno de los 35 → tabla baseline; (b) post-move, `--list` por suite comparando
   el conteo por prefijo `<mod>::` contra la tabla. (El conteo estático por
   grep NO sirve: los bloques `proptest!` generan fns de test invisibles al
   grep.)
3. Cero cambios en el cuerpo de los archivos movidos (solo `git mv` + nuevos
   `main.rs` + docs). Diff revisable por inspección.
4. Property tests reconocen sus regressions vía el protocolo empírico de §2.3.
   La verificación ingenua queda explícitamente prohibida.
5. CI nightly verde con las líneas del workflow actualizadas, disparado vía
   `workflow_dispatch`: el primer run paga link frío del target `axioms`
   (rust-cache cachea deps, no el artefacto del target renombrado) contra
   `timeout-minutes: 20`.
6. `CLAUDE.md`/matriz/ADR-045/046/047 actualizados en el mismo PR, previo
   re-grep (§5).
7. Fila de ADR-048 en `ADR/README.md`.

## 8. Decisión revisable cuando

- Un track nuevo acumule >3 tests propios con afinidad clara → evaluar 5ª
  suite (opción F deja de ser "+1 link sin ganancia").
- Cargo implemente dedup/caching de link para test targets que vuelva
  irrelevante el conteo de binarios.
- El presupuesto de RAM de la máquina de referencia cambie de orden.

## 9. Qué NO cubre este ADR

- **Rutas `tests/<file>.rs` en el corpus regulatorio.** ~60 referencias a
  `tests/property_conservation.rs` sobreviven en `docs/regulatory/**` (más
  `docs/design/SIMULATION_CORE_DECOUPLING.md`, `ADR-040`,
  `SPRINT_AP5_PERSISTENCE_PROPTEST.md`). Quedaron fuera a propósito: §5 es la
  lista cerrada del RFC (sintaxis de comando), y reescribir 40 archivos
  regulatorios rompería el criterio 3 ("diff revisable por inspección"). Es
  deuda documental real, no funcional — ningún build las consume. Follow-up
  sugerido: un `sed` masivo + revisión de trazabilidad en su propio PR.
- La política de `--test-threads` por suite: se deja como knob de runtime, sin
  valor fijado en config (§2.4).

## 10. Veredicto de implementación

**PASS — 37 → 6 targets, 272/272 tests preservados, 4 suites verdes.**

Ejecutado 2026-08-18 sobre HEAD `f75362a`, Windows 11, 16 GB / 16 cores,
`jobs=4`, `[profile.test] debug=0`.

### Targets (criterio 1)

`cargo metadata --no-deps` → **exactamente 6** test targets:

| Target | src_path |
|---|---|
| `axioms` | `tests/axioms/main.rs` |
| `pipeline` | `tests/pipeline/main.rs` |
| `platform` | `tests/platform/main.rs` |
| `probes` | `tests/probes/main.rs` |
| `bridge_optimizer_equivalence` | `tests/bridge_optimizer_equivalence.rs` (gated) |
| `gpu_cell_field_snapshot_palette_dispatch` | `tests/gpu_cell_field_snapshot_palette_dispatch.rs` (gated) |

`git diff -- Cargo.toml` **vacío** — autodiscovery confirmado en la práctica.

### Conteo de tests (criterio 2) — baseline vs post

Comparación programática por prefijo `<mod>::`: **diff vacío, 35/35 módulos,
272/272 tests**. Cero pérdidas.

| Suite | Módulos | Tests baseline | Tests post | Δ |
|---|---:|---:|---:|---:|
| `axioms` | 14 | 134 | 134 | 0 |
| `probes` | 7 | 31 | 31 | 0 |
| `pipeline` | 7 | 68 | 68 | 0 |
| `platform` | 7 | 39 | 39 | 0 |
| **Total** | **35** | **272** | **272** | **0** |

Per-módulo baseline (todos idénticos post-move) — axioms: r1_conservation 6,
r2_determinism 4, r4_calibration 6, r5_sensitivity 7, r6_observability 17,
r7_morph_robustness 18, r8_surrogate 12, r9_ci_gates 10, r10_emergence_gates
16, property_conservation 19, property_autopoiesis 3, emergence_sync 4,
emergence_ecs 8, chemistry_equivalence 4 · probes: probe_animal 5,
probe_celula 5, probe_mono_constructal 7, probe_planta 6, probe_virus 6,
senescence_materialization_check 1, test_senescence_insert 1 · pipeline:
sf_integration 11, energy_competition_integration 6,
morphogenesis_integration 6, planet_viewer_integration 28, field_convergence 4,
input_event_flow 7, pathfinding_multi_waypoint 6 · platform:
require_marker_hierarchy 6, sparse_set_transient_markers 4, g11_strong_ids 10,
q3_pub_field_api 7, t9_terrain_config_hot_reload 1,
wgsl_cell_field_snapshot_valid 1, demo_flow_maps 10.

### Wall-time de builds

| Build | Comando | Wall-time | Artefactos |
|---|---|---:|---|
| Baseline (pre-move) | `cargo test --tests --no-run` | **43.18 s** | lib + 35 test bins |
| Primer link de `axioms` | `cargo test --test axioms persistence_probe::` | **12.57 s** | 1 suite |
| Post-move | `cargo test --tests --no-run` | **10.07 s** | lib + 4 suites |

**Desviación respecto de §6:** ambos builds se presupuestaron en 30-40 min en
frío; se midieron en 43 s y 10 s porque `target/` estaba caliente del trabajo
de ADR-047 (deps de Bevy ya compiladas; sólo se rehizo el link). El número
frío del RFC no se refutó ni se confirmó — no se ejecutó un `cargo clean`.

### Runtime de las 4 suites (criterio de §2.4)

| Suite | Tests | Resultado | Runtime libtest | Wall-time |
|---|---:|---|---:|---:|
| `axioms` | 134 | ok — 130 passed, 0 failed, 4 ignored | 0.67 s | 2 s |
| `probes` | 31 | ok — 31 passed, 0 failed | 0.03 s | 2 s |
| `pipeline` | 68 | ok — 68 passed, 0 failed | 0.06 s | 2 s |
| `platform` | 39 | ok — 39 passed, 0 failed | 0.03 s | 1 s |

**0 fallos, 0 flakiness.** Los 4 `#[ignore]` de `axioms` son los tests de spike
de `chemistry_equivalence` (ADR-045 §5), ignorados también antes del move. El
riesgo de §2.4 (Apps Bevy concurrentes por el paralelismo intra-binario) **no
se materializó**: ninguna suite pasa de 0.7 s y no hizo falta `--test-threads`.

### Protocolo proptest (criterio 4)

1. **Ruta observada.** El probe descartable falló como se diseñó y proptest
   imprimió:
   `proptest: Saving this and future failures in C:\...\RESONANCE\tests\proptest-regressions\persistence_probe.txt`
   → confirma `SourceParallel`: la resolución flipó a
   `tests/proptest-regressions/<name>.txt`, exactamente como predijo §2.3. El
   plan ingenuo (sidecar junto al `.rs`) habría huerfanado las regresiones.
2. **Move.** `git mv` de los 2 sidecars →
   `tests/proptest-regressions/property_conservation.txt` y
   `.../property_autopoiesis.txt`. Ambos figuran como renames (`R`) en git.
3. **Replay confirmado sobre el archivo REAL** (no sólo sobre el probe): se
   append-eó una línea `cc` deliberadamente malformada al sidecar de
   `property_conservation` y proptest la reportó al leerlo —
   `proptest: C:\...\tests\proptest-regressions\property_conservation.txt:11: unparsable line, ignoring`
   — prueba directa de que el archivo se lee desde la ruta nueva. El sidecar se
   restauró byte a byte (verificado con `cmp`).

El probe y su sidecar fueron borrados; no figuran en el diff (criterio 3).

### Integridad del diff (criterio 3)

`git status --porcelain` muestra los 37 movimientos (35 `.rs` + 2 sidecars)
como **renames puros `R`** — similitud 100 %, cero cambios de contenido. Los
únicos archivos nuevos en `tests/` son los 4 `main.rs`.

### Nightly (criterio 5) — PENDIENTE de validación post-push

`workflow_dispatch` **no se puede disparar localmente**. El yml quedó
actualizado en 4 líneas (78, 86 y —hallazgo nuevo— los `paths:` triggers 23 y
37). **El primer run se valida post-push.** Dos vigilancias para ese run:

- **`timeout-minutes: 20`** se dejó **sin cambiar**: el build step pasa de
  compilar 1 archivo de test a compilar los 14 de `axioms`, pero no hay
  medición de CI que justifique un bump, y subirlo a ciegas contradice el
  criterio del RFC ("si roza el límite, subir"). Si el primer run roza los 20
  min, subir el timeout es el fix.
- El paso de upload de artefactos ya listaba `tests/proptest-regressions/`
  (líneas 94-95), por lo que la ruta nueva queda cubierta sin cambios.

### Hallazgo no previsto por el RFC

El re-grep prescrito (`--test +[a-z0-9_]+`) **no captura los `paths:` triggers
del workflow**, que referencian `tests/property_autopoiesis.rs` como ruta de
archivo (líneas 23 y 37). Sin actualizarlos el nightly habría dejado de
dispararse silenciosamente ante cambios del property test — falla funcional, no
cosmética. Se detectó con un segundo grep (`tests/[a-z0-9_]+\.rs`) y se
corrigió. **Lección para futuros moves de archivos: el grep de comandos no
sustituye al grep de rutas.**

## 11. Riesgos

| Riesgo | Mitigación | Estado post-implementación |
|---|---|---|
| Pérdida de regresiones proptest por convención `SourceParallel` | §2.3 protocolo empírico obligatorio | Cerrado — ruta observada y replay probado (§10) |
| Dos módulos declaran el mismo `#[test]` fn name → filtros ambiguos | 0 colisiones en 258 fns; el filtro `<mod>::` sigue siendo unívoco | Cerrado — 272 tests listados sin ambigüedad |
| Tests con estado de proceso compartido colisionan en paralelo | 0 usos de `set_var`/`current_dir`; único escritor de FS es `chemistry_equivalence` → `target/ai3_dissipation_curve.csv`, path único y test `#[ignore]` | Cerrado — 4 suites verdes en paralelo intra-binario |
| Runtime RAM por Apps concurrentes | §2.4; knob `--test-threads` documentado | No materializado — máx 0.67 s por suite |
| Pérdida silenciosa de un test en el move | Criterio 2 (baseline de `--list`) | Cerrado — diff de conteos vacío |
| Referencias editadas entre el snapshot y la ejecución | §5: re-grep obligatorio | Cerrado — re-grep ejecutado; reveló 1 locus extra |
| Rutas `tests/*.rs` stale en el corpus regulatorio | Fuera de alcance declarado (§9) | **Abierto** — deuda documental, sin efecto funcional |
| Nightly excede `timeout-minutes: 20` con el link de `axioms` | Subir el timeout si el primer run roza | **Abierto** — se valida post-push |
