# RFC-001: Prueba comportamental de Ax6 sobre el motor real (entrainment ECS)

**Estado:** Superseded → `docs/arquitectura/ADR/ADR-047-ax6-ecs-emergence-probe.md`
**Fecha:** 2026-08-17 · implementado 2026-08-17

El contenido íntegro de este RFC (problema, propuesta, métrica, escenario,
alternativas, constantes, criterios de aceptación) vive ahora en ADR-047, que
además carga el veredicto medido del spike en su §10. Este archivo se conserva
sólo por el registro de crítica de abajo — la trazabilidad de *por qué* cada
decisión es como es.

---

## Registro de crítica

**Ronda 1** (crítico física/ECS, 2026-08-17): 3 MAJOR + 5 MINOR, veredicto
REVISAR. Aplicado: A2 reencuadrada de "decay Ax7" a gate duro del broadphase
con aserción `S == 0` (el decay solo NO suprime: S≈0.95); derivación de ticks
corregida de lineal a exponencial + modo lento, valor 200→400 declarado
empírico; schedule fijado a `Update` (patrón r2; `FixedUpdate` +
`MinimalPlugins` rompe el conteo de ticks); piso de S recalculado vía escalera
congelada (S≈0.75, no 0.875); spacing 6→8 para eliminar truncación de vecinos
y su sesgo por spawn index; atribución de determinismo corregida
(`query_radius`/`to_bits` + spawn order, no el sort interno del snapshot); φ
declarado don't-care y stream de ω avanzando 2 estados por draw gaussiano;
`build_index` marcado como patrón a replicar, no import.

**Ronda 2** (crítico arquitectura/convenciones, 2026-08-17): 1 BLOCK +
5 MAJOR + 1 MINOR, veredicto REVISAR. Aplicado: `COHERENCE_BANDWIDTH × 0.16`
eliminado (número mágico invertido — BLOCK); spread rederivado de
`KURAMOTO_LOCK_THRESHOLD_HZ` con cotas gateadas en r10; destino final ADR-047
declarado + secciones ADR agregadas (alternativas A/B/C, no-viola-axiomas,
costos, revisable-cuando, veredicto §10 propio); métrica y verdict movidos a
`equations/emergence/synchronization.rs` como fns puras (no inline en el test)
+ gates r10 para las constantes nuevas; EM-1.5 como ítem nuevo (EM-1.4 declaró
el probe Out-of-scope — no se retro-expande un ítem cerrado); ADR-046 §10
restringido a sus propios números analíticos (el veredicto del probe vive en
ADR-047 §10); entrada `[[test]]` de Cargo.toml eliminada (convención: explícito
sólo para `required-features`); prefijo `ECS_SYNC_*` → `SYNC_ECS_*`.

**Ronda 3** (crítico adversarial de implementabilidad, 2026-08-17): 1 BLOCK +
3 MAJOR + 6 MINOR, veredicto REVISAR → con fixes aplicados, listo para
implementar. Todas las citas file:line y la aritmética central verificadas sin
errores. Aplicado: `SYNC_ECS_CENTER_HZ = 75.0` (BLOCK — `gaussian_f32` centra
en 0 y L2 clampea a `max(0.0)`: sin offset, gaussiana rectificada y experimento
corrupto en silencio; gate `center − 3·spread > 0` en r10); protocolo de
aserción del `S == 0` (recolección canónica por `Entity::index` + identidad
bit a bit vía `hash_f32_slice`); bordes de `frequency_collapse_s` definidos
(σ₀≤0→0, no-finitos→0, clamp [0,1]); guard estructural con exclusión de self
(`query_radius` devuelve al propio nodo); `frequency_std` agregada con
convención n−1; estructura de 3 Apps + radio de `SpatialEntry` explicitados;
aritmética de ticks corregida (13.5, no 11; `1/n` vecinos, no N población);
verdict llamado una vez por ablación; límite "régimen de truncación no
ejercido" agregado a §8; criterio 2 endurecido a "todas las seeds" y path de
`DIAGNOSIS_EM1_5.md` fijado.
