# Sprint AP-4: Fission Trigger — Cuando producción interna excede cohesión

**ADR:** [ADR-039](../../arquitectura/ADR/ADR-039-fission-criterion.md)
**Esfuerzo:** 1 semana
**Bloqueado por:** AP-3
**Desbloquea:** AP-5

## Contexto

Sin replicación, una vesícula es solo un atractor estático. Para cerrar el invariante "lo que persiste copió antes de disiparse", la vesícula debe **dividirse** cuando supera su capacidad de contención.

No se puede hardcodear "if size > X then split" — viola Axiom 6. La fisión debe emerger de la física: presión interna (producción de la closure) excede tensión de membrana (cohesión).

## Principio

Análogo a `electroweak symmetry breaking` o a la fisión binaria bacteriana. Una burbuja se divide cuando la presión interna supera la tensión superficial:

```
Para cada blob (componente conexa de membrane_strength > THRESHOLD):
  internal_production = Σ_cell∈blob Σ_r∈C rate_r × stoich_out
  cohesion_capacity   = perimeter(blob) × mean_membrane_strength(blob)
  pressure_ratio      = internal_production / max(cohesion_capacity, ε)

  if pressure_ratio > FISSION_PRESSURE_RATIO:
    pinch_axis = principal_eigenvector(covariance(blob))
    split blob along pinch_axis:
      copy species[cell] to two new connected regions
      lineage_id_new = hash(lineage_id_parent, tick)
```

`FISSION_PRESSURE_RATIO = DISSIPATION_PLASMA / DISSIPATION_SOLID = 50.0` — la única constante "calibrable" del track.

## Entregable

1. `BlobIndex { id: u32, cells: Vec<(u16, u16)>, lineage: u64 }` — Resource (regenerada cada N=50 ticks)
2. `find_blobs(membrane_grid, threshold) → Vec<BlobIndex>` — flood-fill, pure fn
3. `pressure_ratio(blob, species, network) → f32` — pure fn
4. `pinch_axis(blob_cells) → Vec2` — PCA 2D, pure fn
5. `apply_fission(grid, blob, axis, lineage_parent) → (lineage_a, lineage_b)` — copia mass-conserving
6. `fission_system` — every 50 ticks, dispara `FissionEvent { parent, children: [u64; 2], tick }`

## Tareas

| # | Tarea | Archivo | Tests |
|---|-------|---------|-------|
| 1 | `BlobIndex` + flood-fill | `src/blueprint/equations/blob_topology.rs` | 5 |
| 2 | `pressure_ratio` pure fn | `src/blueprint/equations/fission.rs` | 5 |
| 3 | `pinch_axis` 2D PCA pure fn | `src/blueprint/equations/fission.rs` | 4 |
| 4 | `apply_fission` (conservativa) | `src/blueprint/equations/fission.rs` | 6 |
| 5 | `FissionEvent` + system | `src/simulation/chemical/fission.rs` | 3 integration |
| 6 | `LineageRegistry` resource | `src/resources/lineage.rs` | 3 |

## Criterios de aceptación

- [ ] Blob con producción baja → pressure_ratio < 50 → no fission
- [ ] Blob saturado (producción ≫ cohesión) → fission en ≤ 50 ticks
- [ ] Conservación masa: `Σ species post-fission == Σ species pre-fission` (within rounding)
- [ ] Linaje: ambos hijos heredan `lineage_parent` en `LineageRegistry`
- [ ] Pinch axis es el eje principal de la covarianza espacial (test con elipse alargada)
- [ ] Determinismo: misma seed → misma cadena de fissions
- [ ] Property: post-fission, ambos hijos tienen `find_raf` ≠ ∅ (la closure se preserva)
