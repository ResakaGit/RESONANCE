# Blueprint: Inferencia campo → muestra / partes (EPI)

Referencia de contrato alineada al track [`docs/sprints/ENERGY_PARTS_INFERENCE/README.md`](../sprints/ENERGY_PARTS_INFERENCE/README.md) y al contrato de almanaque [`docs/sprints/ELEMENT_ALMANAC_CANON/README.md`](../sprints/ELEMENT_ALMANAC_CANON/README.md) (EAC).  
Template base: [`00_contratos_glosario.md`](00_contratos_glosario.md).

## 1) Propósito y frontera

- **Qué resuelve:** proyección **derivada** del `EnergyFieldGrid` a estructuras baratas por celda (`CellFieldSnapshot`), RGB lineal canónico por celda / posición para visual e inferencia (`field_linear_rgb_from_cell`, `gf1_field_linear_rgb_qe_at_position`), y extensión GF1 para tintes por nodo vía callback (`build_flow_spine_painted`). Opcionalmente, copia del snapshot a GPU (SSBO) con layout fijado respecto a WGSL.
- **Qué no resuelve:** no sustituye la simulación ni la propagación del campo; no define taxonomías MOBA ni gameplay de habilidades. La GPU no escribe grids de simulación.

## 2) Superficie pública (contrato)

| Área | Ubicación | Export / API |
|------|-----------|--------------|
| EPI1 snapshot | `src/worldgen/cell_field_snapshot/` | `CellFieldSnapshot`, `CellFieldSnapshotCache`, `cell_field_snapshot_from_energy_cell`, `cell_field_snapshot_sync_system`, `cell_field_snapshot_read`, `frequency_contributions_fingerprint` |
| Layout GPU (EPI4) | `cell_field_snapshot/gpu_layout.rs` | `GpuCellFieldPacked`, `GpuCellFieldSnapshotHeader`, `cell_field_snapshot_to_gpu_packed`, `gpu_cell_field_snapshot_bytes`, constantes `CELL_FIELD_SNAPSHOT_*` |
| EPI2 muestreo | `src/worldgen/field_visual_sample.rs` | `field_linear_rgb_from_cell`, `field_linear_rgb_from_cell_inputs`, `gf1_field_linear_rgb_qe_at_position`, `linear_rgb_from_derive_color` |
| Puras compartidas | `src/blueprint/equations.rs` | `field_linear_rgb_from_hz_purity`, `compound_field_linear_rgba`, `field_linear_rgb_sanitize_finite`, fenología (`linear_rgb_lerp`, `field_visual_mix_unit`) |
| EPI3 GF1 | `src/geometry_flow/mod.rs`, `branching.rs` | `build_flow_spine_painted`, `GeometryInfluence::branch_role`, `vertex_along_flow_color` |
| Plugin GPU opcional | `src/rendering/gpu_cell_field_snapshot/` (feature `gpu_cell_field_snapshot`) | `GpuCellFieldSnapshotPlugin`; shader `assets/shaders/cell_field_snapshot.wgsl` |
| Wire en cadena worldgen | `src/worldgen/systems/prephysics.rs` | `cell_field_snapshot_sync_system` encadenado **después** de `derive_cell_state_system` (coherencia Hz/pureza y `materialized_entity`) |

## 3) Invariantes y precondiciones

- La cache EPI1 es **descartable**: debe poder reconstruirse solo desde grid + estado de celdas ya derivado + misma política de invalidación (`grid.generation` / opción A: rebuild completo documentado en sprint EPI1).
- **No** mezclar lectura directa de celda y snapshot sin contrato de `generation` (regla del README del track).
- WGSL y Rust de layout GPU deben mantenerse **1:1**; tests: `tests/wgsl_cell_field_snapshot_valid.rs`, test opcional `gpu_cell_field_snapshot_palette_dispatch` con `--features gpu_cell_field_snapshot`.

## 4) Comportamiento runtime

- **Fase worldgen:** sync del snapshot en la cadena de `prephysics` (no gameplay `Phase::`* principal salvo consumidores que lean la cache desde otros sistemas).
- **EPI4:** `Update` — sube SSBO cuando `CellFieldSnapshotCache.synced_generation` alinea con `EnergyFieldGrid.generation` (ver `gpu_cell_field_snapshot_upload_system`).
- **Determinismo:** muestreo y empaquetado sin RNG; orden de entidades en consumidores (p. ej. shape inference) documentado en sprints.

## 5) Implementación y trade-offs

- **Memoria:** vector denso `Vec<Option<CellFieldSnapshot>>` por celda — coste O(celdas); ver tests de tamaño máximo de struct en `cell_field_snapshot/mod.rs`.
- **Valor:** O(1) por lectura de celda; GF1 puede compartir misma semántica de color que `visual_derivation` vía puras EAC/EPI2.
- **EPI4:** trade-off DevX vs bandwidth — feature flag para no arrastrar pipeline GPU en builds mínimos.

## 6) Fallas y observabilidad

- **Stale cache:** si el orden de schedule invierte sync vs derive, Hz/pureza/materialized quedan desalineados; hay tests de cadena en el módulo snapshot.
- **Divergencia CPU/GPU:** mitigación = misma versión de schema (`CELL_FIELD_SNAPSHOT_GPU_SCHEMA_VERSION`) y tests de parseo WGSL.

## 7) Checklist de atomicidad

- ¿Responsabilidad principal? **Sí** — proyección campo → muestras reutilizables; GF1 permanece stateless con inyección.
- ¿Acopla dominios? **Bajo** en núcleo; el plugin GPU acopla Bevy rendering.

## 8) Referencias cruzadas

- `docs/arquitectura/blueprint_blueprint_math.md` — EAC / `field_linear_rgb_from_hz_purity`
- `docs/arquitectura/blueprint_geometry_flow.md` — GF1 y EPI3
- `docs/arquitectura/blueprint_v7.md` — `EnergyFieldGrid`, materialización
- `main.rs` — registro condicional de `GpuCellFieldSnapshotPlugin`
