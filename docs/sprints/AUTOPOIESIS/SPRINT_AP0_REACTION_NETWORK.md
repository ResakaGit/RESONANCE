# Sprint AP-0: Reaction Network Substrate

**ADR:** [ADR-037](../../arquitectura/ADR/ADR-037-reaction-network-substrate.md)
**Esfuerzo:** 1.5 semanas
**Bloqueado por:** Nada (fundación del track)
**Desbloquea:** AP-1, AP-3

## Contexto

Hoy `particle_lab` modela partículas individuales (Coulomb + LJ → moléculas). Esto demuestra formación pero no permite cinética química a escala de sopa: no hay "concentración de A", "tasa de A+B→C", ni difusión. Sin estos primitivos no se puede detectar un set autocatalítico (Kauffman) ni medir su persistencia (Pross).

## Principio agnóstico

Una **especie** no es un "tipo de partícula" — es un canal de energía localizado:

```
species[cell][s] = qe de la especie s en la celda  (f32)
reaction r: reactants[r] + products[r] @ rate k[r], frequency f_r

Δ(s) = Σ_r (stoich[r,s] × rate_r(species[cell], k[r], f_r))
rate_r = k[r] × Π reactants^stoich × alignment(f_r, cell.freq)    ← Axiom 8
                                    × (1 - DISSIPATION_LIQUID)    ← Axiom 4
```

Cada reacción consume reactivos, produce productos, disipa la diferencia. Conservación local enforced.

## Entregable

1. `species: [f32; MAX_SPECIES]` añadido a celda del `NutrientFieldGrid` (canal nuevo)
2. `Reaction { reactants: [(SpeciesId, u8); 4], products: [(SpeciesId, u8); 4], k: f32, freq: f32 }` — packed, `Copy`, 32 bytes
3. `ReactionNetwork(Vec<Reaction>)` — Resource (no component)
4. `mass_action_rate(species, reaction) → f32` — pure fn
5. `apply_reaction(species: &mut [f32], r: &Reaction, dt: f32)` — pure fn, conservación enforced
6. `diffuse_species(grid, dt)` — Laplacian discreto (4-vecino), tasa `DISSIPATION_LIQUID`
7. `reaction_step_system` — ChemicalLayer
8. `species_diffusion_system` — ChemicalLayer (after reaction)

## Tareas

| # | Tarea | Archivo | Tests |
|---|-------|---------|-------|
| 1 | `MAX_SPECIES = 32` const + `SpeciesId(u8)` | `src/blueprint/constants/chemistry.rs` | 1 |
| 2 | Extender grid con `species: [f32; 32]` | `src/batch/scratch.rs` | 2 |
| 3 | `Reaction` struct packed 32B | `src/layers/reaction.rs` | 3 |
| 4 | `ReactionNetwork` resource + loader RON | `src/resources/reaction_network.rs` | 4 |
| 5 | `mass_action_rate` pure fn | `src/blueprint/equations/reaction_kinetics.rs` | 6 |
| 6 | `apply_reaction` pure fn (conserv. enforced) | `src/blueprint/equations/reaction_kinetics.rs` | 8 |
| 7 | `diffuse_species` pure fn | `src/blueprint/equations/reaction_kinetics.rs` | 4 |
| 8 | Reaction step system | `src/simulation/chemical/reaction_step.rs` | 3 integration |
| 9 | Diffusion system | `src/simulation/chemical/species_diffusion.rs` | 2 integration |
| 10 | RON loader + sample networks | `assets/reactions/raf_minimal.ron` | — |

## Criterios de aceptación

- [ ] Conservación local: para cada step, `Δ(qe) ≤ 0` y `|Δ(qe)| ≈ DISSIPATION_LIQUID × consumed`
- [ ] Difusión homogeneiza `species[s]` en sopa cerrada (test: varianza ↓)
- [ ] Reacción A+B→C con k=1.0 satisface `[C](t) ≈ [A]_0 × (1 - exp(-k·t))` para [B]_0 ≫ [A]_0
- [ ] `Reaction` es `Copy` y ≤32 bytes (assert `size_of`)
- [ ] Zero allocation en hot path (reaction_step_system)
- [ ] RON loader carga `raf_minimal.ron` (3 reacciones formando ciclo cerrado)
