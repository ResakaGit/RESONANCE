# ADR-037: Reaction Network Substrate — SoA per-cell vs entity-as-molecule vs component-per-species

**Estado:** Propuesto
**Fecha:** 2026-04-14
**Contexto:** AUTOPOIESIS track, sprint AP-0

## Contexto

El track AUTOPOIESIS necesita un sustrato químico que permita:

1. Concentraciones de N especies en cada punto del espacio.
2. Reacciones con cinética de mass-action y catálisis frequency-aligned.
3. Difusión Laplaciana inter-celular.
4. Determinismo, cero allocation en hot path, ≤256 entidades, ≤32 especies, 60Hz.
5. Detección topológica posterior (RAF, blobs).

El simulador ya tiene `NutrientFieldGrid` con 4 canales (C, N, P, W). Particle_lab modela partículas individuales. Ninguna pieza actual cubre la combinación arriba.

## Alternativas evaluadas

| Opción | Modelo | Pros | Contras |
|--------|--------|------|---------|
| **A: Entity-as-molecule** | Cada partícula es Entity con `Species` component. Reacciones via collisions (extiende particle_lab). | Pure emergence (Axiom 6 pleno). Visualmente intuitivo. | O(N²) sin Barnes-Hut a escala química real (10³–10⁵ partículas/celda). Determinismo frágil bajo ECS scheduling. RAF detection requeriría agregación constante. |
| **B: Component-per-species** | Entity con `[Concentration<S>; N]` components, query por especie. | ECS-idiomático. | Viola regla 13 (no `Vec` en components). Query width explota: 32 species → 32 component types. Cache-hostile. |
| **C: SoA per-cell** | `species: [f32; MAX_SPECIES]` añadido a `NutrientFieldGrid`, `ReactionNetwork` como Resource. Reacciones operan sobre slices. | Cache-friendly (32 × f32 = 128B = 2 cache lines). Determinismo trivial (orden fijo). RAF y blob detection naturales sobre grid. Difusión Laplaciana ya es patrón conocido (canales C/N/P). Zero allocation. | Pierde la "molécula como entidad" — solo concentraciones. Para visualización hay que reconstruir representación discreta. |

## Decision

**Opción C: SoA per-cell.**

```rust
// src/batch/scratch.rs (extender NutrientFieldGrid)
pub struct CellState {
    pub c: f32, pub n: f32, pub p: f32, pub w: f32,  // existing
    pub species: [f32; MAX_SPECIES],                  // NEW
    pub freq: f32,                                    // for catalysis alignment
}

// src/resources/reaction_network.rs
#[derive(Resource)]
pub struct ReactionNetwork(pub Vec<Reaction>);

#[derive(Copy, Clone, Debug)]
pub struct Reaction {
    pub reactants: [(SpeciesId, u8); 4],   // (id, stoich), id=255 sentinel
    pub products:  [(SpeciesId, u8); 4],
    pub k: f32,
    pub freq: f32,
}
// size_of::<Reaction>() == 32 bytes (1 cache line / 2)
```

### Justificación

1. **Performance.** A 256 celdas × 32 especies × 64 reacciones, un step completo es ~50K mults — sobra para 60Hz.
2. **Determinismo.** El orden de reacciones es el orden del `Vec`; no depende del scheduler ECS.
3. **Composición con telescope (ADR-015).** Las concentraciones son un campo escalar sobre el grid — el telescope ya sabe proyectar campos.
4. **No viola Axiom 6.** Las especies y reacciones son **datos** (cargados de RON), no hardcoded behavior. Una sopa aleatoria es válida. La autopoiesis emerge del juego de las reacciones, no de un componente "Cell".
5. **Compatible con particle_lab.** Particle_lab queda como "L-1 fundamental" (cómo se forman moléculas individuales). AP-0 opera al nivel "L0 cinético" (sopa de millones de moléculas resumida en concentraciones). Cada uno tiene su use case.

### Lo que NO hace este ADR

- **No** elimina particle_lab. Coexisten: particle_lab para chemistry-from-physics, AP-0 para chemistry-as-substrate.
- **No** introduce nueva phase. Reacciones corren en `ChemicalLayer` (existente).
- **No** modifica L0 BaseEnergy. Las concentraciones son del **grid**, no de entidades.

## No viola axiomas

| Axiom | Cumplimiento |
|-------|-------------|
| 1 | Especies son packets de qe. `Σ species[cell]` contribuye al qe total de la celda. |
| 2 | `apply_reaction` enforced: outputs ≤ inputs (con disipación). Pool invariant en cada step. |
| 4 | Cada reacción disipa `(1 - REACTION_EFFICIENCY) × consumed`. |
| 5 | Conservación local verificada en pure fn (debug_assert). |
| 6 | **Crítico.** Reacciones son datos cargados, no código. RAF emerge en AP-1 sin programarse. |
| 7 | Difusión Laplaciana = atenuación con distancia. |
| 8 | `rate_r ∝ alignment(reaction.freq, cell.freq)` — catálisis frequency-selective. |

## Archivos modificados

| Archivo | Cambio |
|---------|--------|
| `src/batch/scratch.rs` | Extender `CellState` con `species: [f32; 32]` y `freq: f32` |
| `src/blueprint/constants/chemistry.rs` | **NUEVO** — `MAX_SPECIES`, `MAX_REACTIONS_PER_NETWORK`, etc. |
| `src/layers/reaction.rs` | **NUEVO** — `Reaction` struct, `SpeciesId` newtype |
| `src/resources/reaction_network.rs` | **NUEVO** — `ReactionNetwork` resource + RON loader |
| `src/blueprint/equations/reaction_kinetics.rs` | **NUEVO** — `mass_action_rate`, `apply_reaction`, `diffuse_species` |
| `src/simulation/chemical/reaction_step.rs` | **NUEVO** — system, ChemicalLayer |
| `src/simulation/chemical/species_diffusion.rs` | **NUEVO** — system, ChemicalLayer (after reaction) |
| `assets/reactions/*.ron` | **NUEVO** — sample networks |

## Tests

- 26 unit tests (kinetics, conservación, difusión, loader)
- 5 integration tests (sopa cerrada, sopa abierta, regression, perf bench, determinismo)

## Costos

- ~+128 bytes/celda × 256 celdas = +32KB grid (despreciable).
- ~+1KB per ReactionNetwork (Resource único).
- Hot-path: ~50K f32 mults/tick @ N=64 reactions, 256 cells → < 0.5ms en target hardware.

## Decisión revisable cuando

- N supera 256 celdas con ≥1024 reacciones por celda → considerar GPU compute shader.
- Scaling a 3D necesita revisión de difusión (6-vecino vs 4-vecino).
