# RFC-002: Consolidación de test binaries — 37 → 6 targets

**Estado:** Superseded → `docs/arquitectura/ADR/ADR-048-test-suite-consolidation.md`
**Fecha:** 2026-08-17 · implementado 2026-08-18

El contenido íntegro de este RFC (problema, propuesta, las 4 suites y su
criterio de asignación, mecánica del `git mv`, protocolo proptest, mapa de
invocación, alternativas, criterios de aceptación) vive ahora en ADR-048, que
además carga el veredicto medido de la implementación en su §10. Este archivo
se conserva sólo por el registro de crítica de abajo — la trazabilidad de *por
qué* cada decisión es como es.

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

**Post-implementación** (2026-08-18): la ejecución confirmó las 3 rondas sin
sorpresas de diseño — 6 targets, 272/272 tests, 4 suites verdes, ruta de
proptest exactamente la predicha por §2.3. Un solo hallazgo no anticipado: el
patrón de re-grep de §3 (`--test +[a-z0-9_]+`) no captura los `paths:` triggers
del nightly (líneas 23 y 37), que referencian `tests/property_autopoiesis.rs`
como **ruta de archivo**; sin actualizarlos el workflow habría dejado de
dispararse en silencio. Detalle en ADR-048 §10.
