# Changelog

## [Unreleased] — 2026-03-25

### Added — AC-1–AC-5: Axiomatic Closure (cross-axiom dynamics)
- **AC-1**: `metabolic_interference_factor` / `apply_metabolic_interference` in `blueprint/equations/energy_competition/metabolic_interference.rs` (15 tests). `trophic_predation_attempt_system` scales assimilation by oscillatory alignment between predator and prey (Axiom 3×8)
- **AC-2**: Kuramoto entrainment in `blueprint/equations/emergence/entrainment.rs` (12 tests) + `simulation/emergence/entrainment.rs` system (4 integration tests). Distance-weighted frequency alignment via `Phase::AtomicLayer`
- **AC-3**: `frequency_imitation_affinity` + `should_imitate_with_affinity` in `blueprint/equations/emergence/culture.rs` (12 tests). `cultural_transmission_system` gates imitation by oscillatory affinity (Axiom 6×8)
- **AC-4**: `frequency_purity_at_distance` + `entrainment_coupling_at_distance` in `blueprint/equations/signal_propagation.rs` (11 tests). Exponential spatial decay for entrainment coupling (Axiom 7×8)
- **AC-5**: `cooperation_is_beneficial` + `extraction_estimate_in_group` + `defection_temptation` in `blueprint/equations/emergence/symbiosis.rs` (10 tests). `cooperation_evaluation_system` in `simulation/cooperation.rs` (3 integration tests). Nash-stable alliance detection with AC-1 interference cost
- **Events**: `AllianceProposedEvent`, `AllianceDefectEvent` in `events.rs`, registered in bootstrap
- **Constants**: `METABOLIC_INTERFERENCE_FLOOR`, `KURAMOTO_BASE_COUPLING`, `ENTRAINMENT_SCAN_RADIUS`, `COOPERATION_GROUP_BONUS`, `COOPERATION_DEFECT_THRESHOLD`, `CULTURE_COHERENCE_IMITATION_BONUS_CAP`
- **Plugins**: `AtomicPlugin` registers entrainment after spatial index; `MetabolicPlugin` registers cooperation after trophic
- **Test count**: 1721 → 2150

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
