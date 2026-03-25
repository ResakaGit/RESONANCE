# Resonance — Alchemical Simulation Engine

Wave-resonance physics simulation for an alchemical MOBA, built with **Rust** and **Bevy 0.15 ECS**.

## Concept

Every entity in the world is an assembly of **14 orthogonal layers** (L0 BaseEnergy — L13 StructuralLink). Layers are independent of each other, but their cross-interactions produce emergent behavior: elements, matter states, spells, collisions, and life all arise from the same thermodynamic equations.

There are no hardcoded stats like "HP", "ATK", or "DEF". Everything is energy (`qe`), frequency (`Hz`), phase (`φ`), density (`ρ`), and coherence (`Eb`).

## Architecture

```
src/
├── layers/          14 ECS layers + auxiliaries (24 files)
├── simulation/      FixedUpdate pipeline: Input → Thermo → Atomic → Chemical → Metabolic → Morphological
├── entities/        EntityBuilder, archetypes (spawn_*)
├── blueprint/       Pure math engine (equations/, constants/, almanac/)
├── bridge/          Cache optimizer (BridgeCache<B>, 12 equation kinds)
├── eco/             Eco-boundaries, zones, climate
├── geometry_flow/   GF1 flora-tube (stateless branching)
├── topology/        Terrain: noise, slope, drainage, hydraulics
├── worldgen/        V7: field_grid, nucleus, propagation, materialization/
├── rendering/       quantized_color
├── runtime_platform/ 17 sub-modules (compat 2D/3D, tick, input, camera, HUD, fog)
├── plugins/         SimulationPlugin + 6 domain plugins, LayersPlugin, DebugPlugin
└── events.rs        Bevy events (system contracts)
```

- **Design:** [docs/design/](./docs/design/) — High-level specs, index at [INDEX.md](./docs/design/INDEX.md).
- **Module contracts:** [docs/arquitectura/](./docs/arquitectura/) — 30 runtime blueprints.
- **Active backlog:** [docs/sprints/](./docs/sprints/) — Open sprint tracks.
- **Roots:** [TOPOLOGY_AND_LAYERS.md](./TOPOLOGY_AND_LAYERS.md), [PLANT_SIMULATION.md](./PLANT_SIMULATION.md), [DESIGNING.md](./DESIGNING.md).

## Requirements

- Rust 1.80+ (`rustup update stable`)
- macOS / Linux / Windows

## Run

```bash
cargo run
```

Startup loads worldgen from `assets/maps/default.ron` (or the map set via env), completes warmup, then spawns **one hero** (`demo_level.rs`). **Field colors** are the V7 materialized cells (3D bridge / 2D sprites); debug gizmos **do not** draw those cells to keep the mosaic visible.

**Minimal map (small grid, short warmup):**

```bash
RESONANCE_MAP=demo_minimal cargo run
```

**Guided demo (Terra + pressure + small grid):** see [docs/guides/DEMO_FLOW.md](./docs/guides/DEMO_FLOW.md).

```bash
RESONANCE_MAP=demo_floor cargo run
```

**Strata demo (Terra floor + Ventus atmosphere, orbs + sky slab, color mosaic):**

```bash
RESONANCE_MAP=demo_strata cargo run
```

**Four flowers (32×32 grid, four Terra-band nuclei, V7 interference visible):**

```bash
RESONANCE_MAP=four_flowers cargo run
```

**Procedural flower (geometry_flow + pistil, `flower_demo` map):**

```bash
RESONANCE_MAP=flower_demo cargo run
```

**Core 3D demo (default):** `cargo run` with no env vars → **`full3d`** profile (3D rig, bridge, `CameraRigTarget` on demo hero). Run from crate root for assets.

**2D / hybrid:** `RESONANCE_RENDER_COMPAT_PROFILE=legacy2d` or `=hybrid` (2D camera with zoom in legacy; `EnergyVisual` sprite sync in hybrid).

Values: `legacy2d`, `hybrid`, `full3d` (also `RESONANCE_V6_PROFILE` as alias). Standalone binary without `CARGO_MANIFEST_DIR`: set `BEVY_ASSET_ROOT` to the crate path.

## Tests

```bash
cargo test
```

1721 unit tests, 0 failures.

## License

Private — All rights reserved.

---

## Changelog

### 2026-03-25

**Q5 — SimulationPlugin domain split**
- Extracted 6 domain plugins from `SimulationPlugin`: `ThermodynamicPlugin`, `AtomicPlugin`, `ChemicalPlugin`, `InputPlugin`, `MetabolicPlugin`, `MorphologicalPlugin`
- `simulation/pipeline.rs` reduced from 554 → 126 LOC (pure phase ordering + clock wiring)
- Each plugin owns exactly one Phase slice; all `.chain()`/`.after()` ordering preserved

**SM-1 — worldgen materialization split**
- `worldgen/systems/materialization.rs` (1851 LOC) split into `spawn.rs` (cell delta machinery) + `season.rs` (nucleus/season lifecycle) + `mod.rs` (re-exports)
- Public API unchanged; two orthogonal concerns now in separate files ≤370 LOC each

**SM-5 — CompetitionNormBridge wired**
- Added `CompetitionNormBridge` to bridge infrastructure via `impl_bridgeable_scalar_io!`
- `CompetitionNormEquationInput { raw_score, midpoint, k }` — quantized normalize using logistic bands
- Wired in `bridge/presets/ecosystem.rs`, registered in all 4 preset fns, exported from `bridge/mod.rs`
- 4 new unit tests; 97 bridge tests total

**SM-6 — Constants consolidation**
- Deleted orphan aggregate files (`math_and_ids.rs`, `layer_defaults.rs`) that duplicated constants already registered in their domain shards

**SM-7 — Sprint docs cleanup**
- Archived completed tracks: ENERGY_COMPETITION, STRUCTURE_MIGRATION, CODE_QUALITY
- Updated sprint status tables (MG-1–MG-7 ✅, EC-1–EC-8 ✅, SM-1–SM-7 ✅, Q2/Q3/Q5/Q8 ✅)
- Fixed broken `docs/arquitectura/` links; added `blueprint_sensory_lod.md` to index

**Q2 — Named constants**
- 11 magic numbers named in `energy_competition_ec.rs` + `organ_inference_li3.rs`
- `dynamics.rs` and `scale.rs` wired into `blueprint/equations/energy_competition/mod.rs`

**Q3 — PoolConservationLedger encapsulation**
- All public fields privatized; `new()` constructor + typed getters added
- All call sites updated (`pool_distribution.rs`, `pool_conservation.rs`, `scale_composition.rs`)

**Q8 — Color extraction from mesh generators**
- `vertex_along_flow_color` extracted to `blueprint/equations/field_color/`
- Inline petal shading in `build_petal_fan` extracted to `petal_shaded_flow_color`

**EC-1–EC-8 — Energy competition system**
- Hierarchical energy pools with extraction registry and conservation ledger
- Scale-invariant pool composition; competition dynamics and trajectory classification
- `PoolConservationLedger`, `EnergyPoolComponent`, `PoolDistributionSystem` implemented

**MG-1–MG-7 — Inferred morphogenesis**
- Thermodynamic equations (Carnot efficiency, exergy, entropy production)
- `MetabolicGraph` DAG with temporal step, entropy constraint, writer-monad ledger
- Shape optimization (MG-4), surface rugosity (MG-7), albedo inference (MG-5)
- `MorphologicalPlugin` registers all morphogenesis systems in `Phase::MorphologicalLayer`
