# RFC-002: Consolidación de test binaries — 37 → 6 targets

**Estado:** FINAL — 3/3 rondas de crítica aplicadas (2026-08-17) — listo para
implementar (precondición dura: working tree de `tests/` commiteado, §2.2 paso 0)
**Fecha:** 2026-08-17
**Ubicación final:** `docs/arquitectura/ADR/ADR-048-test-suite-consolidation.md`
al aceptarse — es decisión arquitectónica (reestructura la topología de
`tests/`, fija dónde vive todo test futuro, tiene condiciones de reversión;
precedente de infra con ADR: ADR-042 bevy-viz-layout). La **ejecución**
mecánica (git mv + grep de docs) es un ítem de sprint track TU
(`docs/sprints/TOOL_USE/`). `docs/rfc/` es staging del proceso de crítica
(precedente RFC-001 → ADR-047).
**Relación:** continúa la mitigación de OOM de 2026-08-17 (`[profile.test]
debug=0`, `jobs=4`, `test=false` en 29 bins) · implementa la política RAM de
`AXIOM_LAYER_VALIDATION_MATRIX.md` §3 a nivel estructural · consolida el
`tests/emergence_ecs.rs` creado por ADR-047
**Autor:** Claude (sesión 2026-08-17), revisión Augusto

---

## 1. Problema

`tests/` contiene 37 archivos (36 originales + `emergence_ecs.rs` de ADR-047)
→ cargo compila y linkea **37 binarios independientes**, cada uno estatizando
Bevy 0.15 completo. Tras los fixes de config quedan ~62 link units en
`cargo test`; los 35 test binaries no gated son el multiplicador dominante
restante (~85 % del costo de link). En una máquina de 16 GB esto define el
techo de lo corrible.

Costo secundario: overhead por proceso (cargo corre cada binario en serie), y
~36 artefactos por perfil en disco.

## 2. Propuesta

Consolidar los **35 archivos no gated** en **4 suites** usando el layout
estándar de cargo (subdirectorio con `main.rs` = un solo target de
integración). Los 2 gated (`bridge_optimizer_equivalence`,
`gpu_cell_field_snapshot_palette_dispatch`, `Cargo.toml:292-300`) permanecen
como binarios propios — `required-features` es por target y no puede coexistir
con tests no gated en el mismo binario.

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

(`demo_flow_maps` reasignado desde pipeline en ronda 2: es contrato estático
de assets RON contra `docs/guides/DEMO_FLOW.md` — `tests/demo_flow_maps.rs:1-2`
— cero App, cero pipeline.)

Verificado (rondas 1-2): 35 no gated asignados exactamente una vez; cero test
fns duplicadas por suite (258 fns analizadas); cero `DefaultPlugins`/render en
`tests/` (todas las Apps son `MinimalPlugins`; `planet_viewer_integration` ni
siquiera levanta App — fns puras de frame_buffer); el `#![cfg(test)]` de
`chemistry_equivalence.rs:20` es inner attr válido como módulo.

### 2.2 Mecánica (por suite)

0. **Gate duro (BLOCK de ronda 3):** `git status --porcelain tests/` debe
   estar **vacío** — todo el trabajo de ADR-046/047 commiteado antes de
   empezar. `git mv` sobre un archivo untracked falla con
   `fatal: not under version control` (al 2026-08-17 hay 3 untracked en
   `tests/`, incluido `emergence_ecs.rs`), y mover archivos con diff pendiente
   mezcla contenido y move en el mismo diff, rompiendo el criterio 3.
1. `git mv tests/<file>.rs tests/<suite>/<file>.rs` — **git mv, no
   delete+add**, para preservar historia/blame.
2. `tests/<suite>/main.rs` nuevo: doc-comment de la suite + `mod <file>;` por
   archivo. Sin `fn main` (harness de libtest la genera).
3. **`Cargo.toml` sin cambios** — cargo edition 2024 autodescubre
   `tests/<suite>/main.rs` como target `<suite>` (`autotests` default true;
   las 2 entradas gated existentes no lo desactivan). Consistente con la
   convención fijada en RFC-001 R2 / ADR-047: `[[test]]` explícito SOLO para
   `required-features`. (La analogía con la política de bins declarados no
   aplica: esa existe por la necesidad funcional de `test=false`/`bench=false`,
   que los test targets no tienen.)
4. Los archivos movidos **no cambian por dentro**: importan `resonance::*` como
   crate externo igual que antes; cada uno pasa a ser un módulo del binario de
   la suite, con su namespace propio.

### 2.3 proptest-regressions — la convención REAL (corregido en ronda 1)

El default de proptest es `FileFailurePersistence::SourceParallel`
(`proptest-1.11.0/src/test_runner/failure_persistence/file.rs:42-58` — 1.11.0
es lo que resuelve el lock del requirement "1.4"): sube el árbol desde el
source file hasta el primer directorio que contenga `lib.rs`/`main.rs` y
persiste en un directorio **hermano** `proptest-regressions/` con extensión
`.txt`. Hoy los sidecars viven como `tests/<name>.proptest-regressions` SOLO
porque `tests/` no tiene `main.rs` (fallback `WithSource`). **Al crear
`tests/axioms/main.rs`, la resolución flipa** a
`tests/proptest-regressions/<name>.txt` — mover el sidecar junto al `.rs`
(el plan ingenuo) lo dejaría huérfano y las regresiones guardadas se
perderían en silencio.

Protocolo (empírico primero, move después):

1. Post-consolidación, ANTES de mover sidecars: crear un módulo probe
   **descartable** — `tests/axioms/persistence_probe.rs` con un proptest que
   siempre falla (`prop_assert!(x > 255)` sobre `u8`) + `mod persistence_probe;`
   temporal en `main.rs` — correrlo y **observar dónde escribe proptest**
   (esperado `tests/proptest-regressions/persistence_probe.txt`; la ruta
   exacta la decide la lib, no este doc). Borrar el probe Y su sidecar. El
   probe nunca entra al diff final — criterio 3 intacto, sin tocar el cuerpo
   de ningún archivo movido.
2. `git mv` de los 2 sidecars existentes a esa ruta observada
   (esperado: `tests/proptest-regressions/property_conservation.txt` y
   `property_autopoiesis.txt`).
3. Revertir el fallo inyectado y confirmar que proptest **lee** las seeds
   (replay del caso guardado).

La verificación "que no aparezca un sidecar nuevo" NO sirve: el archivo solo
se escribe on-failure, esa comprobación pasa siempre.

### 2.4 Runtime RAM y paralelismo intra-binario (el trade-off real)

Antes: 37 procesos **seriales** (cargo corre un binario a la vez), cada uno con
paralelismo interno. Después: 1 proceso por suite con **todos** sus tests en
threads paralelos → el pico de RAM en runtime puede subir (varias `App` Bevy
concurrentes en `probes`/`pipeline`).

Mitigación: todas las Apps de test usan `MinimalPlugins` (verificado por grep,
ronda 2 — sin render/GPU, ~decenas de MB por App); 7-8 concurrentes es
aceptable. Si una suite pica, el knob es
`cargo test --test pipeline -- --test-threads=4` — runtime, no estructura.
El RFC **no** agrega serialización global.

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
prefijo es unívoco — verificado en ronda 1). La escalera de la matriz (§3 paso
4: "un archivo a la vez") pasa a ser "un módulo a la vez" — mismo costo de
runtime, y el costo de link del paso 4 baja de 1-de-37 a 1-de-4 posibles
binarios.

## 3. Referencias que hay que actualizar

Los números de línea son del snapshot 2026-08-17; ADR-046/047 y SPRINT_EM1
fueron editados por la implementación de ADR-047 el mismo día — **el ítem de
sprint debe re-grep antes de editar, con un patrón que capture flags
intercalados y menciones sin prefijo:**

```
grep -rnE -- '--test +[a-z0-9_]+' docs/ scripts/ .github/
```

(el patrón ingenuo `"cargo test --test"` pierde `--no-run --lib --test`,
`--release --test` y `` `--test x` `` inline — 4 filas de esta misma tabla.)

| Locus | Cambio |
|---|---|
| `.github/workflows/autopoiesis-nightly.yml:78` | build step `cargo test --no-run --lib --test property_autopoiesis` → `--test axioms` (**sin esta fila el nightly rompe en el build, antes de llegar al run**) |
| `.github/workflows/autopoiesis-nightly.yml:86` | `--test property_autopoiesis -- --nocapture` → `--test axioms property_autopoiesis:: -- --nocapture` |
| `docs/design/AXIOM_LAYER_VALIDATION_MATRIX.md` §2 (tabla de tipos, `:65`), §3, §5 (comandos por ficha) | sintaxis nueva del paso 4 |
| `docs/arquitectura/ADR/ADR-047-ax6-ecs-emergence-probe.md` (comandos `--test emergence_ecs`, `--test r10_emergence_gates`) | → `--test axioms <mod>::` — el doc canónico, no el stub de staging |
| `docs/sprints/EMERGENT_MEASUREMENTS/SPRINT_EM1_THREE_FIGURES.md` (comandos EM-1.4/EM-1.5) | sintaxis nueva |
| `docs/arquitectura/ADR/ADR-046-emergence-sync-axiom6.md` (comandos §8/§10) | sintaxis nueva |
| `docs/arquitectura/ADR/ADR-045-chemistry-canonical-choice.md:105` | `cargo test --release --test chemistry_equivalence -- --ignored --nocapture` → `--test axioms chemistry_equivalence:: -- --ignored --nocapture` |
| `docs/sprints/AUTOPOIESIS/SPRINT_AP6_AUTOPOIETIC_LAB.md:152` | sprint ACTIVO (no archive) — actualizar comando |
| `docs/regulatory/01_foundation/PROBLEM_RESOLUTION.md:209`, `SOFTWARE_MAINTENANCE_PLAN.md:201` | comando property_conservation |
| `CLAUDE.md` §Testing | documentar las 4 suites + patrón de filtro |
| Sprints archivados (`SPRINT_SF7`, `SPRINT_BS1`) | NO tocar — registro histórico |

Verificado en ronda 1: `ci.yml:34` usa `--workspace` (no afectado); no hay
Makefiles/justfiles/tasks que invoquen `--test <nombre>`.

## 4. Interacción con ADR-047

ADR-047 (RFC-001) ya está implementado: `tests/emergence_ecs.rs` existe y este
RFC lo consolida en `tests/axioms/` actualizando los comandos del propio
ADR-047 (§3). Secuencia cumplida (001 → 002). La implementación de este RFC
arranca **solo después** de que la de ADR-047 esté terminada y verificada —
nunca dos agentes en paralelo sobre `tests/`.

## 5. Alternativas consideradas

| Opción | Contras |
|---|---|
| **A (elegida):** 4 suites por afinidad | los nombres `--test <archivo>` cambian (mitigado por filtros §2.5) |
| B: 1 solo binario `tests/all/` | pierde el knob de granularidad de la escalera; runtime RAM concentrada; un fallo de compilación bloquea TODO el testing |
| C: no consolidar, solo `jobs=4` | deja 35 links de Bevy; cada `cargo test` completo sigue costando ~30-40 min de link |
| D: mover integración a `#[cfg(test)]` en `src/` (unit) | cambia visibilidad (los tests dejan de consumir la API pública como crate externo — pierde el valor de contrato de `q3_pub_field_api` et al.) |
| E: 3 suites (platform fusionada en pipeline) | ahorra 1 link, pero pierde el escalón de runtime ~0 de la escalera: platform corre en segundos sin Apps de pipeline; fusionada, cada corrida barata paga el arranque caro |
| F: 5 suites (partir probes/pipeline por peso) | +1 link sin ganancia: el pico de RAM runtime ya se acota con `--test-threads` (§2.4), no hace falta partición estructural |

## 6. No viola axiomas

Todos N/A razonado: el cambio es **topología de build** — ningún axioma
gobierna la organización de test targets, y el criterio de aceptación 3
(cuerpos de tests sin cambios, solo `git mv`) garantiza mecánicamente cero
efecto de simulación. Ningún threshold, constante ni system se toca.

## 7. Costos

- **Implementación:** ~1-2 h mecánicas (git mv + 4 `main.rs` + grep de docs) +
  **2 builds** de ~30-40 min c/u frío con `jobs=4`: pre-move
  (`cargo test --tests --no-run`, baseline del criterio 2) y post-move
  (verificación).
- **Riesgo:** bajo — reversible con `git mv` inverso; el protocolo §2.3 es el
  único paso con pérdida potencial y está blindado.
- **Beneficio:** −31 links de Bevy por `cargo test`; suite completa pasa de
  inviable (OOM) a corrible en 16 GB.

## 8. Criterios de aceptación

1. `cargo metadata`: exactamente 6 test targets (4 suites + 2 gated),
   `Cargo.toml` sin diff.
2. **Cero tests perdidos**, en versión ejecutable en 16 GB — un
   `cargo test -- --list` global pre-move linkearía los 37 binarios, o sea el
   OOM que este RFC elimina: (a) pre-move, `cargo test --tests --no-run`
   (jobs=4, presupuestado en §7) y después
   `cargo test --test <name> -- --list | grep -c ': test$'` por cada uno de
   los 35 (binarios ya linkeados, costo ~0) → tabla baseline per-file en el
   PR; (b) post-move, `--list` por suite comparando el conteo por prefijo
   `<mod>::` contra la tabla. (El conteo estático por grep NO sirve: los
   bloques `proptest!` generan fns de test que el grep no ve.)
3. Cero cambios en el cuerpo de los archivos movidos (solo `git mv` + nuevos
   `main.rs` + docs). Diff revisable por inspección.
4. Property tests reconocen sus regressions vía el protocolo empírico de §2.3
   (fallo inyectado → observar ruta → move → confirmar replay). La
   verificación ingenua queda explícitamente prohibida.
5. CI nightly verde con las DOS líneas del workflow actualizadas (78 y 86),
   disparado vía `workflow_dispatch` en el mismo PR: el primer run paga link
   frío del target `axioms` (rust-cache cachea deps, no el artefacto del
   target renombrado) contra `timeout-minutes: 20`
   (`autopoiesis-nightly.yml:56`) — si roza el límite, subir el timeout en la
   misma fila de §3.
6. `CLAUDE.md`/matriz/ADR-045/046/047 actualizados en el mismo PR (no en
   "follow-up"), previo re-grep (§3).
7. Fila de ADR-048 en `ADR/README.md`.

## 9. Decisión revisable cuando

- Un track nuevo acumule >3 tests propios con afinidad clara → evaluar 5ª
  suite (opción F deja de ser "+1 link sin ganancia").
- Cargo implemente dedup/caching de link para test targets que vuelva
  irrelevante el conteo de binarios.
- El presupuesto de RAM de la máquina de referencia cambie de orden.

## 10. Veredicto de implementación

> A completar tras la ejecución: conteo real de targets (`cargo metadata`),
> conteo de `--list` antes/después, wall-time y pico de RAM de un
> `cargo test --no-run` completo, resultado del protocolo proptest §2.3, y
> estado del nightly.

## 11. Riesgos

| Riesgo | Mitigación |
|---|---|
| Pérdida de regresiones proptest por convención `SourceParallel` | §2.3 (era el BLOCK de la ronda 1; protocolo empírico obligatorio) |
| Dos módulos declaran el mismo `#[test]` fn name → filtros ambiguos | Verificado ronda 1: 0 colisiones en 258 fns; si un test futuro colisiona, el filtro `<mod>::` sigue siendo unívoco |
| Tests con estado de proceso compartido colisionan en paralelo | Verificado ronda 1: 0 usos de `set_var`/`current_dir`; único escritor de FS es `chemistry_equivalence.rs:106-108` → `target/ai3_dissipation_curve.csv`, path único vía `CARGO_MANIFEST_DIR` y test `#[ignore]` — sin colisión |
| Runtime RAM por Apps concurrentes | §2.4; knob `--test-threads` documentado |
| Pérdida silenciosa de un test en el move | Criterio 2 (baseline de `--list`) lo hace imposible de pasar por alto |
| Referencias editadas entre el snapshot y la ejecución (ADR-046/047, sprints) | §3: re-grep obligatorio antes de editar docs |

---

## Registro de crítica

**Ronda 1** (crítico mecánica cargo, 2026-08-17): 1 BLOCK + 1 MAJOR + 2 MINOR,
veredicto REVISAR. Aplicado: §2.3 reescrita — el plan original (sidecar junto
al `.rs`) perdía las regresiones en silencio porque el default de proptest es
`SourceParallel`, que al aparecer `tests/<suite>/main.rs` flipa la resolución a
`tests/proptest-regressions/<name>.txt`; se reemplazó por protocolo empírico
(fallo inyectado → observar ruta → move → replay) y se prohibió la
verificación ingenua. §3: agregada la línea 78 del nightly (build step, rompía
antes del run) + ADR-045:105 + ADR-046 + SPRINT_AP6:152 (activo, no archive);
el `[ASSUMPTION]` se reemplazó por verificación real (`ci.yml` no afectado).
§7 (hoy §11): t9 exonerado (cero FS, usa Assets en memoria); el escritor real
de FS es `chemistry_equivalence` (path único + `#[ignore]`, descartado).

**Ronda 2** (crítico arquitectura/convenciones, 2026-08-17): 3 MAJOR +
4 MINOR, veredicto REVISAR. Aplicado: paso `[[test]]` explícito ELIMINADO
(contradecía la convención fijada por RFC-001 R2: explícito solo para
`required-features`; cargo edition 2024 autodescubre `tests/<suite>/main.rs`,
y la analogía con los bins no aplica — esa política existe por
`test=false`/`bench=false`); destino declarado ADR-048 + ejecución como ítem
de sprint track TU, con secciones ADR agregadas (no-viola-axiomas N/A
razonado, costos, revisable-cuando, veredicto §10); §3 actualizada — ADR-047
ya existe en disco con comandos propios y la matriz §2:65 también cita la
sintaxis vieja; nota de re-grep obligatorio (docs editados entre rondas por la
implementación de ADR-047); `demo_flow_maps` reasignado a platform (contrato
estático de assets, cero App); descripciones de suite reformuladas con
criterio de asignación reproducible; `integration` renombrada `pipeline`
(todo tests/ es integración; 3 miembros con sufijo `_integration`; vocabulario
del repo); alternativas E (3 suites) y F (5 suites) agregadas con su descarte
razonado. Inventario 34→35 por `emergence_ecs` (ADR-047).

**Ronda 3** (crítico adversarial de implementabilidad, 2026-08-17): 1 BLOCK +
3 MAJOR + 1 MINOR, veredicto REVISAR → con fixes aplicados, listo para
implementar. Verificado contra el disco post-ADR-047: inventario 37 = 35+2
cierra (14/7/7/7), autodiscovery probado empíricamente in-repo (35 targets
autodescubiertos conviven con los 2 `[[test]]` con `path`), cero colisiones de
nombre con bins/benches, TODOS los números de línea de §3 vigentes. Aplicado:
gate duro de working tree limpio en `tests/` (BLOCK — `git mv` falla sobre los
3 untracked de ADR-047 y mezclaría diffs); patrón de re-grep corregido
(`--test +[a-z0-9_]+` — el ingenuo perdía 4 filas de la propia tabla);
criterio 2 reescrito en versión ejecutable (per-file `--list` sobre binarios
ya linkeados, no `--list` global que reintroduce el OOM; grep estático
descartado por `proptest!`); §2.3 paso 1 con mecanismo concreto (probe
descartable `persistence_probe.rs`, nunca entra al diff — criterio 3 intacto);
criterio 5 con `workflow_dispatch` + vigilancia del `timeout-minutes: 20` del
nightly.
