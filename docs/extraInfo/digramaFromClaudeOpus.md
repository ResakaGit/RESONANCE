# Resonance — Arquitectura Completa (Estado Actual)

> Actualizado: 2026-03-27 | Estado: SF ✅ · ET parcial (ET-2,3,5,6,7,8,9 wired; T3-T4 stubs) · AC ✅ 5/5 · GS parcial (3/9) · Batch ✅ (34 systems, 165 tests) · Stellar ✅ · Energy Cycle ✅ · Awakening ✅ · Bevy Decoupled (math_types) · Axiomatic Inference ✅ ARCHIVED (7/7) · Internal Energy Field ✅ · Axiom-Pure Behavior ✅ · 2473+ tests

---

## Flujo de Arquitectura

```
╔══════════════════════════════════════════════════════════════════════════════╗
║                    RESONANCE — FLUJO DE ARQUITECTURA                       ║
║                         (estado actual 2026-03-27)                         ║
╚══════════════════════════════════════════════════════════════════════════════╝

┌─────────────────────────────────────────────────────────────────────────────┐
│                          STARTUP (una vez)                                  │
│                                                                             │
│  RON Map ──→ MapConfig ──→ EnergyFieldGrid (32×32 arena / 128×128 stellar)│
│                         ──→ NutrientFieldGrid                               │
│                         ──→ TerrainField (altitude, slope, drainage)        │
│                         ──→ Spawn EnergyNucleus + VictoryNucleus marker    │
│                         ──→ Spawn ControlNode (3-5 por mapa) ⏳ GS-6       │
│                         ──→ AlchemicalAlmanac (11 elementos)                │
│                         ──→ ArchetypeRegistry ← assets/characters/*.ron ⏳ GS-8
│                                                                             │
│  Warmup (ticks) ──→ Propagar campo (multi-tick, SF-6 ✅)                   │
│                  ──→ Materializar entidades (spawn_from_config ⏳ GS-8)    │
│                  ──→ GameState::Playing ──→ PlayState::Active               │
│                  ──→ CheckpointConfig::from_env() guarda estado (SF-5 ✅)  │
│                                                                             │
│  Stellar mode:   spawn_star (L0 1M qe + L11 InverseSquare + EnergyNucleus)│
│                  spawn_planet (orbital velocity + AmbientPressure surface)  │
│                  Map: stellar_system.ron (128×128 AU, 1 star + planets)    │
└─────────────────────────────────────────────────────────────────────────────┘
                                    │
                                    ▼
╔═════════════════════════════════════════════════════════════════════════════╗
║                     FixedUpdate (cada tick, determinista)                   ║
╠═════════════════════════════════════════════════════════════════════════════╣
║                                                                             ║
║  ┌──────────────────── SimulationClockSet ──────────────────┐              ║
║  │  tick_id++ ──→ bridge_phase_tick ──→ metrics_log         │              ║
║  │                ──→ simulation_health_system (SF-1 ✅)     │              ║
║  └──────────────────────────┬───────────────────────────────┘              ║
║                             ▼                                               ║
║  ┌──────────────────── Phase::Input ────────────────────────┐              ║
║  │                                                           │              ║
║  │  [GS-1 ✅] lockstep_input_gate                           │              ║
║  │  PlatformWill: click ──→ PathRequestEvent ──→ NavPath    │              ║
║  │  D5 Sensory: SpatialIndex.query_radius() ──→ Awareness  │              ║
║  │  [ET-2 ✅] theory_of_mind_update_system                  │              ║
║  │  D1 Behavior (assess ──→ decide):                        │              ║
║  │    Axiom 6: mobility_bias gates behavior (no tags)        │              ║
║  │    Axiom 6: food = qe < self × FOOD_QE_RATIO             │              ║
║  │    Axiom 6: threat = qe > self × THREAT_QE_RATIO         │              ║
║  │    Axiom 6: hunt if mobility_bias > HUNT_MOBILITY_THRESHOLD│             ║
║  │  [GS-3 ✅] nash_target_select                            │              ║
║  │  [ET-3 ✅] culture_transmission_spread                   │              ║
║  │    [AC-3 ✅] × freq_imitation_affinity × coherence_bonus │              ║
║  │  SimulationRest: grimoire split, ElementId sync          │              ║
║  └──────────────────────────┬───────────────────────────────┘              ║
║                             ▼                                               ║
║  ┌──────────── Phase::ThermodynamicLayer ───────────────────┐              ║
║  │  Containment, Structural, Resonance, Engine              │              ║
║  │  Irradiance (external solar source, SOLAR_FLUX_BASE)     │              ║
║  │  radiation_pressure, NucleusReservoir                    │              ║
║  │  [GS-5 ✅] nucleus_intake_decay                          │              ║
║  └──────────────────────────┬───────────────────────────────┘              ║
║                             ▼                                               ║
║  ┌──────────────── Phase::AtomicLayer ──────────────────────┐              ║
║  │  dissipation ──→ will_to_velocity ──→ velocity_cap       │              ║
║  │  ──→ locomotion_drain ──→ movement_integrate             │              ║
║  │  ──→ SpatialIndex ──→ TensionField ──→ collision         │              ║
║  │  [AC-2 ✅] entrainment (Kuramoto)                        │              ║
║  └──────────────────────────┬───────────────────────────────┘              ║
║                             ▼                                               ║
║  ┌──────────────── Phase::ChemicalLayer ────────────────────┐              ║
║  │  Nutrient: osmosis ──→ uptake (speed² < FORAGE_MAX)      │              ║
║  │  Photosynthesis: Gaussian resonance con SOLAR_FREQUENCY   │              ║
║  │    gain = irr × area × efficiency × solar_resonance       │              ║
║  │    producers deposit NUTRIENT_DEPOSIT_FRACTION to soil    │              ║
║  │  State transitions, Catalysis, Homeostasis                │              ║
║  └──────────────────────────┬───────────────────────────────┘              ║
║                             ▼                                               ║
║  ┌──────────────── Phase::MetabolicLayer ───────────────────┐              ║
║  │  basal_drain, senescence_death, Growth, Stress            │              ║
║  │  Trophic: energy dominance predation (no trophic tags)    │              ║
║  │    Axiom 6: pred.qe > target.qe × PREDATION_DOMINANCE    │              ║
║  │    Axiom 3: drain × interference().abs()                  │              ║
║  │  [AC-5 ✅] cooperation (dissipation reduction, Axiom 5)  │              ║
║  │  [ET-5,6,8,9 ✅] symbiosis, epigenetics, coalitions, niche║             ║
║  │  Social pack: oscillatory affinity (no faction tags)      │              ║
║  │  Culture: blend expression masks by freq affinity         │              ║
║  │  [GS-5 ✅] victory_check                                 │              ║
║  └──────────────────────────┬───────────────────────────────┘              ║
║                             ▼                                               ║
║  ┌──────────── Phase::MorphologicalLayer ───────────────────┐              ║
║  │  senescence ──→ internal_diffusion (8-node qe field)     │              ║
║  │  ──→ growth_inference ──→ morpho_adaptation               │              ║
║  │  ──→ reproduction (genome inheritance + mutation)         │              ║
║  │  ──→ abiogenesis (coherence > dissipation → spawn)        │              ║
║  │  ──→ death_reap (nutrients return to grid)                │              ║
║  │                                                           │              ║
║  │  Internal Energy Field (8 body-axis nodes):               │              ║
║  │    genome_to_profile() → distribute_to_field()            │              ║
║  │    → field_diffuse() per tick → emergent gradients        │              ║
║  │    → field_to_radii() → variable-thickness geometry       │              ║
║  │                                                           │              ║
║  │  Constructal body plan, IWG terrain/water                 │              ║
║  └──────────────────────────┬───────────────────────────────┘              ║
║                             ▼                                               ║
║  [SF-4 ✅] metrics_export: SimulationHealthDashboard → CSV/JSON           ║
╠═════════════════════════════════════════════════════════════════════════════╣
║                     SimWorld Boundary (sim_world.rs) ✅                     ║
║                                                                             ║
║  SimWorld::new(SimConfig)  ──→ headless App + FixedUpdate manual  ✅       ║
║  SimWorld::tick(&[InputCommand]) ──→ FixedUpdate schedule         ✅       ║
║  SimWorld::snapshot() ──→ WorldSnapshot (owned, no ECS types)     ✅       ║
║  SimWorld::energy_hash() ──→ u64  (determinism check)             ✅       ║
║                                                                             ║
║  INV-1: zero render deps   INV-4: deterministic   INV-5: renderer read-only║
║  INV-6: events live 1 tick INV-7: conservation    INV-8: tick_id only clock║
╠═════════════════════════════════════════════════════════════════════════════╣
║                                                                             ║
║  ┌──────────────────── Update (visual) ─────────────────────┐              ║
║  │  sync_visual_from_sim ──→ entity_shape_inference:         │              ║
║  │    GF1 spine + variable-radius mesh (from qe_field)       │              ║
║  │    + per organ sub-mesh ──→ merge_meshes → Mesh3d         │              ║
║  │  shape_color_inference, growth_morphology, phenology      │              ║
║  └───────────────────────────────────────────────────────────┘              ║
╚═════════════════════════════════════════════════════════════════════════════╝

┌─────────────────────────────────────────────────────────────────────────────┐
│              BATCH SIMULATOR (src/batch/ — headless, no Bevy)               │
│                                                                             │
│  SimWorldFlat: 64 EntitySlots (qe_field[8] + freq_field[8] internal)      │
│  34 stateless systems (6 phases) — call blueprint/equations/ for math      │
│  internal_diffusion: 8-node energy field → emergent organ-like gradients  │
│  GenomeBlob: 4 biases + archetype → mutate + crossover                     │
│  GeneticHarness: evaluate → select → reproduce (tournament + elitism)      │
│  WorldBatch: N worlds in parallel via rayon                                │
│  GenomeBlob ↔ Bevy components (lossless round-trip via bridge.rs)          │
│  creature_builder: genome + qe_field → GF1 variable-radius mesh + branches│
│  165 tests · batch_benchmark (criterion)                                   │
│                                                                             │
│  100% axiom-pure behavior:                                                 │
│    Predation: energy dominance (no trophic tags)                           │
│    Photosynthesis: Gaussian resonance with SOLAR_FREQUENCY                 │
│    Foraging: speed² < FORAGE_MAX_SPEED_SQ (slow = graze)                  │
│    Behavior: mobility_bias gates (no archetype tags)                       │
│    Social: oscillatory affinity (no faction tags)                          │
│    All constants in batch/constants.rs (zero magic numbers in systems)     │
└─────────────────────────────────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────────────────────────────────┐
│              8 AXIOMAS (5 primitivos + 3 derivados)                        │
│                                                                             │
│  PRIMITIVOS (independientes, irreducibles):                                │
│  1. Everything is Energy    — all entities are qe                           │
│  2. Pool Invariant          — Σ children ≤ parent                           │
│  4. Dissipation (2nd Law)   — all processes lose energy                     │
│  7. Distance Attenuation    — interaction decays with distance              │
│  8. Oscillatory Nature      — every qe oscillates at frequency f            │
│                                                                             │
│  DERIVADOS (consecuencias, elevados por utilidad de diseño):               │
│  3. Competition as Primitive — de Axiom 8 aplicado a transferencia          │
│  5. Conservation             — de Axiom 2 + 4 (pool + dissipation)         │
│  6. Emergence at Scale       — meta-regla (constrañe al dev, no al motor)  │
│                                                                             │
│  Los 3 derivados no producen comportamiento adicional.                     │
│  Existen como guard rails contra hardcoding.                               │
├─────────────────────────────────────────────────────────────────────────────┤
│              4 CONSTANTES (2 física + 2 calibración)                       │
│                                                                             │
│  FÍSICA (no tocar para calibrar):                                          │
│  KLEIBER_EXPONENT = 0.75              (biológico universal)                │
│  DISSIPATION_SOLID   = 0.005          (Segunda Ley, ratios 1:4:16:50)     │
│  DISSIPATION_LIQUID  = 0.02                                                │
│  DISSIPATION_GAS     = 0.08                                                │
│  DISSIPATION_PLASMA  = 0.25                                                │
│                                                                             │
│  CALIBRACIÓN (grid/game, recalibrar si cambia cell_size o bandas):        │
│  COHERENCE_BANDWIDTH = 50.0 Hz        (ventana de observación, Axiom 8)   │
│  DENSITY_SCALE       = 20.0           (geometría del grid, Axiom 1)       │
│  SOLAR_FREQUENCY     = 400.0 Hz       (resonancia fotosintética, Axiom 8) │
└─────────────────────────────────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────────────────────────────────┐
│              EMERGENCE PIPELINE (axiom-derived, bottom-up)                   │
│                                                                             │
│  ENERGY CYCLE (closed loop):                                               │
│    Solar (external) → resonant entities absorb (Gaussian around 400 Hz)   │
│    → producers deposit nutrients → slow entities forage                    │
│    → dominant entities drain weaker (energy ratio, not tags)              │
│    → dead return nutrients to grid → cycle restarts                       │
│                                                                             │
│  Internal energy field (8 nodes):                                          │
│    genome → distribution profile → diffusion → emergent gradients         │
│    → variable-radius geometry (organ-like bulges where qe concentrates)   │
│    → branching at qe peaks (branching_bias × field nodes)                 │
│                                                                             │
│  Trophic succession: NOT programmed — emerges from energy dominance       │
│  Cooperation: reduces dissipation (Axiom 5), not free qe                  │
│  Social packs: oscillatory affinity > 0.3, not faction tag                │
│  Culture: expression mask blending by freq affinity                       │
│  Morphology: growth_bias→tips, resilience→center, branching→lobes        │
└─────────────────────────────────────────────────────────────────────────────┘
```

---

## Las 14 Capas Ortogonales

```
L0  BaseEnergy ──────── qe (existencia)          ← TODO toca esto
L1  SpatialVolume ───── radius (colisión)         ← allometric growth
L2  OscillatorySignature freq, phase (resonancia) ← homeostasis, catalysis, solar resonance
L3  FlowVector ──────── velocity, drag            ← physics, locomotion, orbital
L4  MatterCoherence ─── state, bond_energy        ← state transitions
L5  AlchemicalEngine ── buffer, valves             ← engine processing, nucleus
L6  AmbientPressure ─── delta_qe, viscosity       ← climate, terrain, vacuum (stellar)
L7  WillActuator ────── intent, channeling         ← behavior AI
L8  AlchemicalInjector  projected_qe, freq         ← spell payload
L9  MobaIdentity ────── faction, tags, crit        ← game rules
L10 ResonanceLink ───── buff/debuff overlay        ← spell side effects
L11 TensionField ────── attract/repel force        ← gravity/magnetic, stellar orbits (1/r²)
L12 Homeostasis ─────── freq adaptation + cost     ← chemical layer
L13 StructuralLink ──── spring joint, stress       ← structural constraint

+ Auxiliares: OrganManifest, MetabolicGraph, GrowthBudget, Grimoire,
  TrophicState, NutrientProfile, InferenceProfile, BodyPlanLayout,
  PerformanceCachePolicy, MorphogenesisShapeParams, MorphogenesisSurface,
  InferredAlbedo, HasInferredShape, ShapeInferred, SenescenceProfile,
  PackMembership, VictoryNucleus ✅, ControlNode ⏳ GS-6,
  InputPacket (SparseSet) ✅ GS-1

+ ET layers: OtherModelSet, SymbiosisLink, SenescenceProfile,
  EpigeneticState, NicheProfile, LanguageCapacity, SelfModel, CulturalMemory
```

---

## Morphogenesis Pipeline (shape from energy)

```
FixedUpdate / MorphologicalLayer:
  shape_optimization_system     → MorphogenesisShapeParams.fineness_ratio
  surface_rugosity_system       → MorphogenesisSurface.rugosity
  albedo_inference_system       → InferredAlbedo.albedo
  epigenetic_adaptation_system  → EpigeneticState.expression_mask (ET-6)
  internal_diffusion_system     → qe_field[8] gradients (emergent organs)
  constructal_body_plan_system  → BodyPlanLayout (N limbs from cost minimization)

Update / after sync_visual:
  entity_shape_inference_system:
    torso = build_flow_spine → build_flow_mesh_variable_radius (qe_field → radii)
    organs = for each slot in BodyPlanLayout:
             organ_slot_scale() → sub-influence → sub-mesh
    final = merge_meshes([torso, organs...]) → V6VisualRoot.Mesh3d

Batch viewer (creature_builder.rs):
  genome biases + qe_field → trunk_params_from_genome() + branch_plan_from_genome()
  → build_flow_spine() + build_flow_mesh_variable_radius(field_to_radii())
  → merge_meshes([trunk, branches...])
  Zero hardcoded shapes. All geometry from energy composition.
```

---

## Test Coverage

```
2473+ tests total:
  blueprint/equations/     → ~600+ pure math tests (all domains)
  simulation/              → ~800+ system tests (MinimalPlugins pattern)
  worldgen/                → ~300+ field/materialization tests
  layers/                  → ~200+ component tests
  batch/                   → 165 headless simulator tests (34 systems)
  tests/                   → ~100+ integration (probe_animal, property_conservation, etc.)
  emergence/               → ~100+ equations + system tests
  internal_field/          → 22 diffusion + profile + radii tests
  creature_builder/        → 6 mesh generation tests
  proptest                 → 19 property-based (conservation, pool equations)
```

---

## Bevy Decoupling Status

```
BEVY-FREE (ready to extract as resonance_core):
├── math_types.rs          ← glam 0.29 re-exports, 0 bevy imports
├── blueprint/equations/   ← 180+ files, 0 bevy::math
│   ├── batch_fitness.rs   ← GA fitness + genome→geometry mapping
│   ├── internal_field.rs  ← 8-node diffusion + radii + distribution
│   └── determinism.rs     ← hashing + RNG (next_u64, gaussian_f32)
├── blueprint/constants/   ← 100% bevy-free
├── batch/                 ← 100% bevy-free (34 systems, rayon parallel)
│   ├── arena.rs           ← EntitySlot (qe_field[8], freq_field[8])
│   ├── systems/           ← 34 stateless systems, axiom-pure
│   ├── harness.rs         ← GeneticHarness (evolutionary loop)
│   ├── bridge.rs          ← GenomeBlob ↔ Bevy (lossless round-trip)
│   └── genome.rs          ← GenomeBlob (mutate, crossover, hash)
├── geometry_flow/
│   ├── creature_builder.rs ← genome + field → mesh (desacoplado)
│   └── mod.rs             ← build_flow_mesh_variable_radius (new)
├── topology/ (pure math)  ← 6 files decoupled
└── eco/ (math)            ← 2 files decoupled

BEVY-COUPLED (rendering + ECS):
├── layers/                ← #[derive(Component, Reflect)]
├── simulation/            ← Query<>, Res<>, Commands
├── plugins/               ← Bevy plugin registration
├── rendering/             ← Mesh, Material, Camera
└── runtime_platform/      ← Input, windowing, navmesh
```

---

## Headless Runners

```bash
# Batch evolution → view evolved creatures (GF1 inferred geometry)
cargo run --release --bin evolve_and_view -- --worlds 500 --gens 300 --ticks 1000 --seed 77

# Batch evolution only (headless, save genomes)
cargo run --release --bin evolve -- --worlds 1000 --gens 500 --ticks 1000 --seed 42

# Bevy simulation with specific map
RESONANCE_MAP=genesis_validation cargo run --release --bin headless_sim -- --ticks 10000 --scale 8 --out world.ppm
```
