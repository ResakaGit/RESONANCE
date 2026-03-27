# Resonance — Arquitectura Completa (Estado Actual)

> Actualizado: 2026-03-27 | Estado: SF ✅ · ET parcial (ET-2,3,5,6,7,8,9 wired; T3-T4 stubs) · AC ✅ 5/5 · GS parcial (3/9) · Batch ✅ · Stellar ✅ · Energy Cycle ✅ · Awakening ✅ · Bevy Decoupled (math_types) · Derived Thresholds ✅ (AI-1 done) · 2472+ tests

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
║  │                                                           │              ║
║  │  PlatformWill:                                            │              ║
║  │    click ──→ PathRequestEvent ──→ pathfinding ──→ NavPath │              ║
║  │                                                           │              ║
║  │  D5 Sensory: SpatialIndex.query_radius() ──→ SensoryAwareness           ║
║  │    freq matching + distance ──→ ThreatMemory              │              ║
║  │    [SF-3 ✅] signal_latency(dist, medium) → delay         │              ║
║  │                                                           │              ║
║  │  [ET-2 ✅] theory_of_mind_update_system:                  │              ║
║  │    OtherModelSet[4 slots] → observe neighbors → update    │              ║
║  │    predictions → evict unprofitable → debit qe cost       │              ║
║  │                                                           │              ║
║  │  D1 Behavior (cooldown ──→ assess ──→ threats ──→ decide):│              ║
║  │    BaseEnergy + TrophicState + Awareness + OtherModelSet  │              ║
║  │      ──→ BehaviorMode (Idle|Hunt|Flee|Eat|Forage)         │              ║
║  │      ──→ BehaviorIntent ──→ WillActuator                  │              ║
║  │                                                           │              ║
║  │  [GS-3 ✅] nash_target_select (BehaviorSet::Decide)       │              ║
║  │  [ET-3 ✅] culture_transmission_spread (after D6)          │              ║
║  │    MemeVector → spread if interference > threshold        │              ║
║  │    [AC-3 ✅] × freq_imitation_affinity × coherence_bonus  │              ║
║  │                                                           │              ║
║  │  SimulationRest:                                          │              ║
║  │    [SM-8G ✅] grimoire split → 3 SRP systems              │              ║
║  │    ElementId ↔ frequency sync                             │              ║
║  └──────────────────────────┬───────────────────────────────┘              ║
║                             ▼                                               ║
║  ┌──────────── Phase::ThermodynamicLayer ───────────────────┐              ║
║  │                                                           │              ║
║  │  Containment:  ContainedIn ──→ thermal equilibrium        │              ║
║  │  Structural:   StructuralLink stress ──→ break event      │              ║
║  │  Resonance:    ResonanceLink ──→ overlay (motor/therm/flow)│             ║
║  │  Engine:       AlchemicalEngine buffer ──→ drain/overload  │              ║
║  │  Irradiance:   sun + ambient ──→ photosynthesis budget     │              ║
║  │  Spell resolve: CastPending ──→ spawn projectile entity    │              ║
║  │  WaveFront:    PropagationMode::WaveFront (SF-6 ✅)        │              ║
║  │  [GS-5 ✅] nucleus_intake_decay                           │              ║
║  │  ✅ radiation_pressure_system: qe > threshold → push out  │              ║
║  │  ✅ NucleusReservoir: finite fuel, drained per tick       │              ║
║  └──────────────────────────┬───────────────────────────────┘              ║
║                             ▼                                               ║
║  ┌──────────────── Phase::AtomicLayer ──────────────────────┐              ║
║  │                                                           │              ║
║  │  dissipation ──→ drag ──→ terrain_effects                 │              ║
║  │  ──→ locomotion drain ──→ movement_integrate              │              ║
║  │  ──→ update SpatialIndex                                  │              ║
║  │  ──→ TensionField (attract/repel, InverseSquare for stellar)            ║
║  │  ──→ collision_interference ──→ DeathEvent                │              ║
║  │                                                           │              ║
║  │  [AC-2 ✅] entrainment_system (Kuramoto freq sync)        │              ║
║  │  integrate_velocity (forward Euler)                       │              ║
║  │  integrate_velocity_verlet_half (symplectic, for orbits)  │              ║
║  │  Transform.translation += FlowVector.velocity × dt        │              ║
║  └──────────────────────────┬───────────────────────────────┘              ║
║                             ▼                                               ║
║  ┌──────────────── Phase::ChemicalLayer ────────────────────┐              ║
║  │                                                           │              ║
║  │  Nutrient: osmosis ──→ regen ──→ uptake ──→ depletion     │              ║
║  │  Photosynthesis: irradiance ──→ qe contribution           │              ║
║  │  State transitions: density+temp ──→ Solid↔Liquid↔Gas     │              ║
║  │  Catalysis chain: spatial_filter ──→ interference ──→ qe  │              ║
║  │  Homeostasis: freq adaptation (L12) + cost                │              ║
║  └──────────────────────────┬───────────────────────────────┘              ║
║                             ▼                                               ║
║  ┌──────────────── Phase::MetabolicLayer ───────────────────┐              ║
║  │                                                           │              ║
║  │  ✅ basal_drain_system: passive qe drain ∝ radius^0.75     │              ║
║  │  ✅ senescence_death_system: hard age limit + Gompertz    │              ║
║  │  Growth: GrowthBudget (TL3 ✅) + Liebig Law               │              ║
║  │  Stress: metabolic_stress ──→ DeathEvent if insolvent     │              ║
║  │  Trophic: satiation ──→ forage ──→ predation              │              ║
║  │    [AC-1 ✅] predation × metabolic_interference_factor    │              ║
║  │  [AC-5 ✅] cooperation_evaluation (Nash alliance detect)  │              ║
║  │  [ET-5 ✅] symbiosis_effect_system:                       │              ║
║  │    SymbiosisLink → mutualism/parasitism qe transfer       │              ║
║  │    unstable links auto-removed                            │              ║
║  │  [ET-9 ✅] niche_adaptation_system:                       │              ║
║  │    NicheProfile overlap → character displacement           │              ║
║  │    competitive pressure separates overlapping niches       │              ║
║  │  [ET-7] senescence_tick (SenescenceProfile age drain)     │              ║
║  │  [ET-8 ✅] coalition_stability (BridgeCache Large 512)    │              ║
║  │  [CE ✅] culture_observation (every 30 ticks)             │              ║
║  │  MetabolicDAG: graph_step ──→ entropy_constraint ──→ ledger│             ║
║  │  faction_identity ──→ bridge_metrics_collect              │              ║
║  │  [GS-5 ✅] victory_check                                 │              ║
║  └──────────────────────────┬───────────────────────────────┘              ║
║                             ▼                                               ║
║  ┌──────────── Phase::MorphologicalLayer ───────────────────┐              ║
║  │                                                           │              ║
║  │  Shape: shape_opt ──→ rugosity ──→ albedo inference        │              ║
║  │  [ET-6 ✅] epigenetic_adaptation_system:                   │              ║
║  │    AmbientPressure → EpigeneticState.expression_mask       │              ║
║  │    silencing costs qe (Axiom 4)                            │              ║
║  │  Constructal body plan (axiomatic):                        │              ║
║  │    optimal_appendage_count(v, ρ, r) → N limbs             │              ║
║  │    organ_slot_scale(slot, count, mobility) → proportions   │              ║
║  │  Growth: intent ──→ allometric_growth (TL6 ✅)            │              ║
║  │  Organs: viability ──→ lifecycle stage                     │              ║
║  │  Reproduction (flora + fauna):                             │              ║
║  │    Flora: BRANCH → seed + mutated InferenceProfile         │              ║
║  │    Fauna: MOVE+REPRODUCE + qe>200 → offspring              │              ║
║  │    All 4 biases mutate (growth, mobility, branch, resil.)  │              ║
║  │  Abiogenesis (axiomatic):                                  │              ║
║  │    coherence_gain(neighbors) > dissipation(local) → spawn  │              ║
║  │    ANY frequency band, properties from energy density      │              ║
║  │  ✅ nucleus_recycling_system: nutrient threshold → new nucleus            ║
║  │  ✅ awakening_system: coherence > threshold → BehavioralAgent            ║
║  │  D8 morpho adaptation (every 16 ticks)                     │              ║
║  │  IWG: terrain_mesh_gen ──→ water_surface                   │              ║
║  │  bridge_metrics_collect                                    │              ║
║  └──────────────────────────┬───────────────────────────────┘              ║
║                             ▼                                               ║
║  [SF-4 ✅] metrics_export: SimulationHealthDashboard → CSV/JSON             ║
╠═════════════════════════════════════════════════════════════════════════════╣
║                     SimWorld Boundary (sim_world.rs) ✅                     ║
║                                                                             ║
║  SimWorld::new(SimConfig)  ──→ headless App + FixedUpdate manual  ✅       ║
║  SimWorld::tick(&[InputCommand]) ──→ FixedUpdate schedule         ✅       ║
║  SimWorld::snapshot() ──→ WorldSnapshot (owned, no ECS types)     ✅       ║
║  SimWorld::energy_hash() ──→ u64  (determinism check)             ✅       ║
║  checkpoint_save/load                                            ✅ SF-5  ║
║                                                                             ║
║  INV-1: zero render deps   INV-4: deterministic   INV-5: renderer read-only║
║  INV-6: events live 1 tick INV-7: conservation    INV-8: tick_id only clock║
╠═════════════════════════════════════════════════════════════════════════════╣
║                     FixedUpdate → Update bridge                             ║
║                                                                             ║
║  V6RenderSnapshot ──→ sync_visual_from_sim_system                          ║
║  TerrainMeshResource ──→ terrain_mesh_sync_system                          ║
║  WaterMeshResource ──→ water_mesh_sync_system                              ║
║  AtmosphereState ──→ atmosphere_sync_system (IWG-6 ✅)                     ║
╠═════════════════════════════════════════════════════════════════════════════╣
║                                                                             ║
║  ┌──────────────────── Update (visual) ─────────────────────┐              ║
║  │                                                           │              ║
║  │  body_plan_layout_inference (fallback bilateral)          │              ║
║  │  entity_shape_inference (compound mesh):                  │              ║
║  │    torso = GF1 tube                                       │              ║
║  │    + per organ in BodyPlanLayout: sub-spine → sub-mesh    │              ║
║  │    → merge_meshes([torso, organs...]) → Mesh3d            │              ║
║  │    rugosity → mesh detail, albedo → tint brightness       │              ║
║  │  shape_color_inference (frequency → palette) ✅           │              ║
║  │  growth_morphology (organ → mesh deformation) ✅          │              ║
║  │  phenology_visual (seasonal tint) ✅                      │              ║
║  │  terrain_mesh_sync + water_mesh_sync + atmosphere_sync    │              ║
║  └───────────────────────────────────────────────────────────┘              ║
╚═════════════════════════════════════════════════════════════════════════════╝

┌─────────────────────────────────────────────────────────────────────────────┐
│              BATCH SIMULATOR (src/batch/ — headless, no Bevy)               │
│                                                                             │
│  SimWorldFlat: 64 EntitySlots + EnergyFieldMini + NutrientFieldMini        │
│  33 stateless systems (6 phases) — call blueprint/equations/ for math      │
│  GenomeBlob: 4 biases + archetype → mutate + crossover                     │
│  GeneticHarness: evaluate → select → reproduce (tournament + elitism)      │
│  WorldBatch: N worlds in parallel via rayon                                │
│  GenomeBlob ↔ Bevy components (lossless round-trip via bridge.rs)          │
│  156 tests · batch_benchmark                                               │
└─────────────────────────────────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────────────────────────────────┐
│              8 AXIOMAS (reglas del universo)                                │
│                                                                             │
│  1. Everything is Energy    — all entities are qe                           │
│  2. Pool Invariant          — Σ children ≤ parent                           │
│  3. Competition as Primitive — magnitude = base × interference              │
│  4. Dissipation (2nd Law)   — all processes lose energy                     │
│  5. Conservation            — energy never created                          │
│  6. Emergence at Scale      — N emerges from N-1                            │
│  7. Distance Attenuation    — interaction decays with distance              │
│  8. Oscillatory Nature      — every qe oscillates at frequency f            │
├─────────────────────────────────────────────────────────────────────────────┤
│              4 CONSTANTES FUNDAMENTALES (parámetros irreducibles)           │
│                                                                             │
│  KLEIBER_EXPONENT = 0.75              (Axiom 4 — biológico universal)      │
│  DISSIPATION_SOLID   = 0.005          (Axiom 4 — Segunda Ley por estado)   │
│  DISSIPATION_LIQUID  = 0.02                                                │
│  DISSIPATION_GAS     = 0.08                                                │
│  DISSIPATION_PLASMA  = 0.25                                                │
│  COHERENCE_BANDWIDTH = 50.0 Hz        (Axiom 8 — ventana de observación)   │
│  DENSITY_SCALE       = 20.0           (Axiom 1 — geometría del grid)       │
│                                                                             │
│  Los 8 axiomas definen las REGLAS.                                         │
│  Las 4 constantes son los PARÁMETROS.                                      │
│  TODO lo demás se COMPUTA: derived_thresholds.rs (12 tests)               │
└─────────────────────────────────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────────────────────────────────┐
│              EMERGENCE PIPELINE (axiom-derived, bottom-up)                   │
│                                                                             │
│  ENERGY CYCLE (closed loop, validated at 5k ticks):                        │
│    Nucleus (finite reservoir) → emit → field → diffusion + rad. pressure  │
│    → reservoir depletes → zone cools → entities die (senescence/drain)    │
│    → nutrients return → threshold → nucleus_recycling → NEW nucleus       │
│    → cycle restarts                                                        │
│                                                                             │
│  Energy field (nuclei emit) ──→ coherence > dissipation ──→ ABIOGENESIS    │
│    → entity with state/caps/profile derived from density                    │
│  Trophic succession: sessile → herbivore → carnivore                       │
│  Life systems: basal_drain + senescence + awakening (inert → alive)       │
│  Reproduction: parent drains qe → offspring inherits mutated profile       │
│  Selection: competition (Ax3) + dissipation (Ax4) → less fit die           │
│  Entrainment (AC-2): neighbors sync frequency (Kuramoto)                   │
│  Cooperation (AC-5): Nash alliance if benefit > solo                       │
│  Theory of Mind (ET-2): predict neighbor behavior                          │
│  Culture (ET-3): memes spread by imitation                                 │
│  Symbiosis (ET-5): mutualism/parasitism energy exchange                    │
│  Epigenetics (ET-6): environment modulates gene expression                 │
│  Coalitions (ET-8): stable alliances with intake bonus                     │
│  Niche displacement (ET-9): competitors diverge                            │
│  [gap] T3-T4: timescales, institutions, language → stubs                   │
└─────────────────────────────────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────────────────────────────────┐
│              STELLAR MODE (same axioms, different scale)                     │
│                                                                             │
│  Star = BaseEnergy(1M) + TensionField(InverseSquare, 200 AU)              │
│       + EnergyNucleus(Lux 1000Hz, emission 10K qe/s)                      │
│  Planet = BaseEnergy(1K) + FlowVector(orbital) + AmbientPressure(surface)  │
│  Habitable zone = emergent: where irradiance is moderate                   │
│  Life on planets = axiomatic abiogenesis (same equations as arena)         │
│  AmbientPressure::vacuum() for deep space (near-zero dissipation)          │
│  integrate_velocity_verlet_half for orbital angular momentum               │
│  Map: stellar_system.ron (128×128 AU)                                      │
└─────────────────────────────────────────────────────────────────────────────┘
```

---

## Las 14 Capas Ortogonales

```
L0  BaseEnergy ──────── qe (existencia)          ← TODO toca esto
L1  SpatialVolume ───── radius (colisión)         ← allometric growth
L2  OscillatorySignature freq, phase (resonancia) ← homeostasis, catalysis, Nash, cultura
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

+ ET layers (components):
  OtherModelSet (4 models), SymbiosisLink, SenescenceProfile,
  EpigeneticState (expression_mask[4]), NicheProfile (Hutchinson 4D),
  LanguageCapacity, SelfModel, CulturalMemory
```

---

## Morphogenesis Pipeline (shape from energy)

```
FixedUpdate / MorphologicalLayer:
  shape_optimization_system     → MorphogenesisShapeParams.fineness_ratio
  surface_rugosity_system       → MorphogenesisSurface.rugosity
  albedo_inference_system       → InferredAlbedo.albedo
  epigenetic_adaptation_system  → EpigeneticState.expression_mask (ET-6)
  constructal_body_plan_system  → BodyPlanLayout (N limbs from cost minimization)

Update / after sync_visual:
  entity_shape_inference_system:
    torso = build_flow_spine → build_flow_mesh (main GF1 tube)
    organs = for each slot in BodyPlanLayout:
             organ_slot_scale(slot, count, mobility_bias) → sub-influence → sub-mesh
    final = merge_meshes([torso, organs...]) → V6VisualRoot.Mesh3d
    rugosity → mesh detail multiplier, albedo → tint brightness
```

---

## Test Coverage

```
2472+ tests total:
  blueprint/equations/     → ~600+ pure math tests (all domains)
  simulation/              → ~800+ system tests (MinimalPlugins pattern)
  worldgen/                → ~300+ field/materialization tests
  layers/                  → ~200+ component tests
  batch/                   → 156 headless simulator tests
  tests/                   → ~100+ integration (probe_animal, probe_mono, property_conservation, etc.)
  emergence/               → ~100+ equations + system tests
  proptest                 → 19 property-based (conservation, pool equations)
  headless_sim             → binary for headless simulation → PPM image (no GPU)
```

---

## Bevy Decoupling Status

```
BEVY-FREE (ready to extract as resonance_core):
├── math_types.rs          ← glam 0.29 re-exports, 0 bevy imports
├── blueprint/equations/   ← 178+ files, 0 bevy::math (2 use bevy::color/render)
├── blueprint/constants/   ← 100% bevy-free
├── topology/ (pure math)  ← 6 files decoupled
├── geometry_flow/ (math)  ← 3 files decoupled
├── eco/ (math)            ← 2 files decoupled
└── bridge/ (normalize)    ← 1 file decoupled

BEVY-COUPLED (phase 2 — future extraction):
├── layers/                ← #[derive(Component, Reflect)]
├── simulation/            ← Query<>, Res<>, Commands
├── plugins/               ← Bevy plugin registration
├── rendering/             ← Mesh, Material, Camera
└── runtime_platform/      ← Input, windowing, navmesh
```

## Headless Runner

```bash
# Run simulation headless (no window, no GPU)
RESONANCE_MAP=genesis_validation cargo run --release --bin headless_sim -- --ticks 10000 --scale 8 --out world.ppm

# Available validation maps:
#   genesis_validation — 11 nuclei, optimized for abiogenesis + energy cycle
#   visual_showcase    — 14 nuclei, all 6 frequency bands, max visual richness
#   proving_grounds    — 7 nuclei, standard test environment
#   headless_stress    — 256×256, high emission stress test
#   optimal_inference  — 6 bands equal emission, calibrated for full cycle in 5k ticks
```

## Axiom-Derived Constants (`derived_thresholds.rs`)

```
4 FUNDAMENTALS (irreducible inputs):
├── KLEIBER_EXPONENT = 0.75          (biological universal, Axiom 4)
├── DISSIPATION_SOLID  = 0.005       (Second Law per state, Axiom 4)
│   DISSIPATION_LIQUID = 0.02
│   DISSIPATION_GAS    = 0.08
│   DISSIPATION_PLASMA = 0.25
├── COHERENCE_BANDWIDTH = 50.0 Hz    (observation window, Axiom 8)
└── DENSITY_SCALE = 20.0             (grid geometry normalization)

ALL DERIVED (12 functions, 12 tests):
├── basal_drain_rate()                = SOLID × 200 = 1.0 qe/tick
├── liquid_density_threshold()        = (LIQUID/SOLID)^(1/Kleiber) × scale ≈ 127
├── gas_density_threshold()           = liquid + (GAS/LIQUID)^(1/Kleiber) × scale ≈ 254
├── plasma_density_threshold()        = gas + (PLASMA/GAS)^(1/Kleiber) × scale ≈ 345
├── move_density_min/max()            = liquid×0.5 / gas×1.5
├── sense_coherence_min()             = SOLID / (SOLID + 0.01)
├── branch_qe_min()                   = self_sustaining × 2
├── spawn_potential_threshold()       = 1/3 (algebraic break-even)
├── senescence_coeff_{mat,flora,fauna}() = dissipation rate per state
├── max_age_{mat,flora,fauna}()       = 1/coeff (Gompertz inverse)
├── radiation_pressure_threshold()    = gas_density_threshold
├── radiation_pressure_transfer_rate()= DISSIPATION_GAS
├── survival_probability_threshold()  = exp(-2) ≈ 0.135
└── nutrient_retention_{mineral,water}() = 1 - dissipation × factor

Sprint: AXIOMATIC_INFERENCE (6 sprints) wires these into all consumers.
Status: AI-1 ✅ (module + 12 tests), AI-2–AI-6 ⏳ pending.
```
