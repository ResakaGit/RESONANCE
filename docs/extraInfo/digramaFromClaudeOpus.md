# Resonance — Arquitectura Completa (Estado Actual)

> Actualizado: 2026-03-27 | Estado: SF ✅ · ET parcial (systems wired for ET-2,3,5,6,8,9; T3-T4 stubs) · AC ✅ 5/5 · GS parcial (3/9) · Batch ✅ · Stellar ✅ · 2408+ tests

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
│              8 AXIOMAS FUNDAMENTALES                                        │
│                                                                             │
│  1. Everything is Energy    — all entities are qe                           │
│  2. Pool Invariant          — Σ children ≤ parent                           │
│  3. Competition as Primitive — magnitude = base × interference              │
│  4. Dissipation (2nd Law)   — all processes lose energy                     │
│  5. Conservation            — energy never created                          │
│  6. Emergence at Scale      — N emerges from N-1                            │
│  7. Distance Attenuation    — interaction decays with distance              │
│  8. Oscillatory Nature      — every qe oscillates at frequency f            │
└─────────────────────────────────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────────────────────────────────┐
│              EMERGENCE PIPELINE (axiom-derived, bottom-up)                   │
│                                                                             │
│  Energy field (nuclei emit) ──→ coherence > dissipation ──→ ABIOGENESIS    │
│    → entity with state/caps/profile derived from density                    │
│  Trophic succession: sessile → herbivore → carnivore                       │
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
2408+ tests total:
  blueprint/equations/     → ~600+ pure math tests (all domains)
  simulation/              → ~800+ system tests (MinimalPlugins pattern)
  worldgen/                → ~300+ field/materialization tests
  layers/                  → ~200+ component tests
  batch/                   → 156 headless simulator tests
  tests/                   → ~100+ integration (probe_animal, probe_mono, etc.)
  emergence/               → ~100+ equations + system tests
```
