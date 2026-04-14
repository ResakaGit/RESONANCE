# Sprint AP-3: Emergent Membrane — Cohesión sin componente Membrana

**ADR:** [ADR-038](../../arquitectura/ADR/ADR-038-emergent-membrane.md)
**Esfuerzo:** 1.5 semanas
**Bloqueado por:** AP-0
**Desbloquea:** AP-4

## Contexto

Una RAF detectada se difunde y se diluye sin contención espacial. Sin membrana, la concentración cae bajo el umbral de auto-sostén y la closure muere. Pero declarar un componente `Membrane` viola Axiom 6 (todo emerge).

Solución: la membrana es la manifestación observable de **cohesión por gradiente de densidad** (TensionField, L11) acumulado por productos de la closure. No hay componente nuevo — solo un nuevo lector del campo.

## Principio

```
Para cada celda:
  density_gradient = ‖∇(Σ_s∈products [s])‖
  membrane_strength = density_gradient × bond_energy_avg
                     × (1 - DISSIPATION_LIQUID)

Para cada flux entre celdas vecinas:
  damped_flux = raw_flux × exp(-membrane_strength × MEMBRANE_DAMPING)
```

Cuando los productos de una closure se acumulan, su gradiente aumenta, la difusión hacia afuera se atenúa exponencialmente, y emerge una "burbuja" de alta concentración → vesícula. Sin componente. Sin script.

## Entregable

1. `local_gradient(grid, cell, species_mask) → Vec2` — pure fn (∇ discreto sobre suma ponderada de especies)
2. `membrane_strength(gradient_norm, bond_energy_avg) → f32` — pure fn
3. `damped_diffusion(raw_flux, membrane_strength) → f32` — pure fn
4. Modificar `diffuse_species` (de AP-0) para aplicar damping membrana-aware
5. `closure_membrane_mask` — Resource: `[bool; MAX_SPECIES]` indica qué especies forman membrana (productos de closures detectadas)
6. Visualization channel `membrane_strength` en grid para AP-6

## Tareas

| # | Tarea | Archivo | Tests |
|---|-------|---------|-------|
| 1 | `local_gradient` 2D | `src/blueprint/equations/membrane.rs` | 4 |
| 2 | `membrane_strength` pure fn | `src/blueprint/equations/membrane.rs` | 4 |
| 3 | `damped_diffusion` pure fn | `src/blueprint/equations/membrane.rs` | 4 |
| 4 | Modificar `diffuse_species` con damping | `src/blueprint/equations/reaction_kinetics.rs` | 3 (regression) |
| 5 | `closure_membrane_mask` resource + system | `src/simulation/chemical/membrane_inference.rs` | 3 integration |
| 6 | Channel viz en grid | `src/batch/scratch.rs` | 1 |

## Criterios de aceptación

- [ ] Sopa homogénea (∇=0) → damping=1 → difusión normal (regression)
- [ ] Closure activa concentra productos → gradiente↑ → flux out↓ → concentración se sostiene
- [ ] Sin closure detectada → ningún damping (mask vacía)
- [ ] Conservación: damping reduce flux pero no destruye qe — qe que no fluye queda en celda
- [ ] Property: para un blob estable, `qe_blob(t+N) ≥ 0.9 × qe_blob(t)` con N=100 ticks
- [ ] Visualización: heatmap de membrane_strength muestra contorno cerrado alrededor del blob
