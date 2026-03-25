# Changelog

## [Unreleased] — 2026-03-25

### Added — Sprint MG-8: Morphogenesis Integration & Demo
- **EntityBuilder**: `with_organ_manifest()`, `with_metabolic_graph_inferred()`, `with_metabolic_graph()`, `irradiance()` methods for metabolic graph composition
- **Archetypes**: `spawn_aquatic_organism`, `spawn_desert_plant`, `spawn_desert_creature`, `spawn_forest_plant` with `MorphogenesisSpawnPreset` constants
- **Map**: `assets/maps/morphogenesis_demo.ron` — 3 biomes (ocean, desert, forest) for phenotype emergence
- **Benchmark**: `benches/morphogenesis_benchmark.rs` — 100 entities, 12-node DAG, 6 MG systems
- **Tests**: 25 new tests covering builder API, archetype spawn, phenotype convergence (fineness, albedo, rugosity), legacy entity regression

### Added — Rosa Lifecycle Demo
- **Default map** optimized for single rosa simulation (Terra + Lux nuclei, cell_size=0.5)
- **demo_level.rs**: Rosa lifecycle startup with `Materialized`, `EnergyVisual`, `LifecycleStageCache`, `QuantizedPrecision`, `InferenceProfile`, camera close-up
- **Systems**: `enforce_rosa_focus_system` (hides non-rosa entities), `stabilize_rosa_energy_system` (energy floor), `stabilize_rosa_growth_system` (LOD + mesh rebuild), `pin_rosa_lod_focus_system` (LOD anchor)
- **Debug telemetry**: `debug_rosa_inference_chain_system` — prints lifecycle stage, biomass, precision, capabilities every 2s

### Removed — Legacy Demo Cleanup
- **Maps deleted**: `demo_arena.ron`, `demo_floor.ron`, `demo_minimal.ron`, `demo_strata.ron`, `demo_river_plateau.ron`, `layer_ladder.ron`, `proving_grounds.ron`, `four_flowers.ron`
- **Code deleted**: `src/world/demos/layer_ladder.rs` and all references
- **Root docs deleted**: `DESIGNING.md`, `PLANT_SIMULATION.md`, `TOPOLOGY_AND_LAYERS.md` (content lives in `docs/`)

### Changed
- **demo_level.rs**: Simplified from 4-plant sandbox to single rosa lifecycle
- **debug_plugin.rs**: Removed layer_ladder dispatch, added rosa lifecycle systems (LOD pin, visibility filter, energy stabilizer)
- **default.ron**: Replaced 3-nucleus arena with 2-nucleus rosa garden (Terra + Lux, cell_size=0.5)
- Camera controls 70% slower for flora-scale navigation
