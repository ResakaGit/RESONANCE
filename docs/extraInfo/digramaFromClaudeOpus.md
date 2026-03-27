# Resonance — Arquitectura Completa (Estado Actual)

> Actualizado: 2026-03-25 | Estado real: SF ✅ · ET ✅ 16/16 · AC ✅ 5/5 · GS parcial (3/9) · 2150 tests

---

## Flujo de Arquitectura

```
╔══════════════════════════════════════════════════════════════════════════════╗
║                    RESONANCE — FLUJO DE ARQUITECTURA                       ║
║                         (estado actual 2026-03-25)                         ║
╚══════════════════════════════════════════════════════════════════════════════╝

┌─────────────────────────────────────────────────────────────────────────────┐
│                          STARTUP (una vez)                                  │
│                                                                             │
│  RON Map ──→ MapConfig ──→ EnergyFieldGrid (32×32)                         │
│                         ──→ NutrientFieldGrid                               │
│                         ──→ TerrainField (altitude, slope, drainage)        │
│                         ──→ Spawn EnergyNucleus + VictoryNucleus marker    │
│                         ──→ Spawn ControlNode (3-5 por mapa) ⏳ GS-6       │
│                         ──→ AlchemicalAlmanac (elementos)                   │
│                         ──→ ArchetypeRegistry ← assets/characters/*.ron ⏳ GS-8
│                                                                             │
│  Warmup (ticks) ──→ Propagar campo (multi-tick, SF-6 ✅)                   │
│                  ──→ Materializar entidades (spawn_from_config ⏳ GS-8)    │
│                  ──→ GameState::Playing ──→ PlayState::Active               │
│                  ──→ CheckpointConfig::from_env() guarda estado (SF-5 ✅)  │
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
║  │    InputBuffer.tick_id == target → LockstepRunCondition  │              ║
║  │    can_advance=false → tick bloqueado hasta tener inputs  │              ║
║  │                                                           │              ║
║  │  PlatformWill:                                            │              ║
║  │    click ──→ PathRequestEvent ──→ pathfinding ──→ NavPath │              ║
║  │                                                           │              ║
║  │  D5 Sensory (scan ──→ memory ──→ threat_event):           │              ║
║  │    SpatialIndex.query_radius() ──→ SensoryAwareness       │              ║
║  │    freq matching + distance ──→ ThreatMemory              │              ║
║  │    [SF-3 ✅] signal_latency(dist, medium) → delay         │              ║
║  │                                                           │              ║
║  │  D1 Behavior (cooldown ──→ assess ──→ threats ──→ decide):│              ║
║  │    BaseEnergy + TrophicState + Awareness                  │              ║
║  │      ──→ BehaviorMode (Idle|Hunt|Flee|Eat|Forage)         │              ║
║  │      ──→ BehaviorIntent ──→ WillActuator                  │              ║
║  │                                                           │              ║
║  │  [GS-3 ✅] nash_target_select (BehaviorSet::Decide):      │              ║
║  │    Hunt/FocusFire → argmin(qe / effective_extraction)     │              ║
║  │    resonance_factor(freq_a, freq_b) × capacity            │              ║
║  │      ──→ BehaviorMode::FocusFire { target, priority }     │              ║
║  │                                                           │              ║
║  │  [ET-1 ✅] associative_memory_encode/decay (BehaviorSet): │              ║
║  │    stimulus_hash → AssociativeMemory[ring] → decay        │              ║
║  │  [ET-2 ✅] other_model_update (after ET-1):               │              ║
║  │    OtherModel[target_id] → predicted_qe, predicted_freq   │              ║
║  │  [ET-3 ✅] culture_transmission_spread (after D6):        │              ║
║  │    MemeVector → spread if interference > threshold        │              ║
║  │    [AC-3 ✅] × freq_imitation_affinity × coherence_bonus  │              ║
║  │  [ET-4 ✅] field_infrastructure_mod (after ET-3):         │              ║
║  │    FieldModRecord → EnergyFieldGrid delta                 │              ║
║  │                                                           │              ║
║  │  [GS-4 ⏳] pack_regroup: pendiente Oleada 2               │              ║
║  │                                                           │              ║
║  │  SimulationRest:                                          │              ║
║  │    [SM-8G ✅] grimoire split → 3 SRP systems              │              ║
║  │    slot_selection ──→ targeting ──→ channeling_start      │              ║
║  │    ──→ CastPending events                                 │              ║
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
║  │  Climate:      season tick ──→ AmbientPressure delta       │              ║
║  │  Spell resolve: CastPending ──→ spawn projectile entity    │              ║
║  │  WaveFront:    PropagationMode::WaveFront (SF-6 ✅)        │              ║
║  │                                                           │              ║
║  │  [GS-5 ✅] nucleus_intake_decay:                          │              ║
║  │    structural_damage → effective_intake = base × (1-dmg)  │              ║
║  └──────────────────────────┬───────────────────────────────┘              ║
║                             ▼                                               ║
║  ┌──────────────── Phase::AtomicLayer ──────────────────────┐              ║
║  │                                                           │              ║
║  │  dissipation ──→ drag ──→ terrain_effects                 │              ║
║  │  ──→ locomotion drain ──→ movement_integrate              │              ║
║  │  ──→ update SpatialIndex                                  │              ║
║  │  ──→ TensionField (attract/repel)                         │              ║
║  │  ──→ collision_interference ──→ DeathEvent                │              ║
║  │                                                           │              ║
║  │  [AC-2 ✅] entrainment_system (after spatial_index):      │              ║
║  │    Kuramoto(neighbours) × coupling(d,λ) → Δfreq          │              ║
║  │  [AC-4 ✅] freq_purity = exp(-d/λ) — SSOT for AC-2       │              ║
║  │                                                           │              ║
║  │  Transform.translation += FlowVector.velocity × dt        │              ║
║  └──────────────────────────┬───────────────────────────────┘              ║
║                             ▼                                               ║
║  ┌──────────────── Phase::ChemicalLayer ────────────────────┐              ║
║  │                                                           │              ║
║  │  Nutrient: osmosis ──→ regen ──→ uptake ──→ depletion     │              ║
║  │  Photosynthesis: irradiance ──→ qe contribution           │              ║
║  │  State transitions: density+temp ──→ Solid↔Liquid↔Gas     │              ║
║  │                                                           │              ║
║  │  Catalysis chain:                                         │              ║
║  │    spatial_filter ──→ CatalysisRequest                    │              ║
║  │    ──→ math (interference = cos(Δfreq × t + Δphase))      │              ║
║  │    ──→ DeltaEnergyCommit                                  │              ║
║  │    ──→ energy_reducer (drain/inject BaseEnergy)           │              ║
║  │    ──→ side_effects (spawn debuffs, ResonanceLink)        │              ║
║  │    ──→ nutrient_return_on_death (qe → NutrientField)      │              ║
║  │                                                           │              ║
║  │  Homeostasis: freq adaptation (L12) + cost                │              ║
║  └──────────────────────────┬───────────────────────────────┘              ║
║                             ▼                                               ║
║  ┌──────────────── Phase::MetabolicLayer ───────────────────┐              ║
║  │                                                           │              ║
║  │  Growth:     GrowthBudget (TL3 ✅) + Liebig Law           │              ║
║  │  Stress:     metabolic_stress ──→ DeathEvent if insolvent │              ║
║  │                                                           │              ║
║  │  Trophic:    satiation decay ──→ forage ──→ predation     │              ║
║  │    [AC-1 ✅] predation × metabolic_interference_factor    │              ║
║  │              ──→ decomposer (recycle dead entities)        │              ║
║  │  [AC-5 ✅] cooperation_evaluation (after trophic):        │              ║
║  │    Nash(a_solo, a_group, b_solo, b_group, AC-1 cost)     │              ║
║  │    → AllianceProposedEvent / AllianceDefectEvent          │              ║
║  │                                                           │              ║
║  │  Social:     pack_formation ──→ cohesion ──→ dominance    │              ║
║  │  [GS-4 ⏳]   pack_cohesion_force: pendiente Oleada 2      │              ║
║  │                                                           │              ║
║  │  [ET-5 ✅] symbiosis_system (after social):               │              ║
║  │    SymbiosisLink → qe transfer + freq entrainment         │              ║
║  │  [ET-6 ✅] epigenetics_expression (after ET-5):           │              ║
║  │    EpigeneticMask × InferenceProfile → phenotype override │              ║
║  │  [ET-7 ✅] senescence_tick (every 16 ticks):              │              ║
║  │    SenescenceClock.age → capacity degradation curve       │              ║
║  │  [ET-8 ✅] coalition_check (every N, BridgeCache Large):  │              ║
║  │    CoalitionBridge O(n²) Nash → CoalitionId assignment    │              ║
║  │  [ET-9 ✅] niche_overlap_update (after census):           │              ║
║  │    NicheVector dot product → competitive exclusion delta  │              ║
║  │                                                           │              ║
║  │  Ecology:    census (every N) ──→ carrying_capacity       │              ║
║  │              ──→ succession (biome shift)                  │              ║
║  │                                                           │              ║
║  │  MetabolicDAG: graph_step ──→ entropy_constraint ──→ ledger│             ║
║  │  Macro-step:   analytical decay for distant entities       │              ║
║  │                                                           │              ║
║  │  [CE ✅] culture_observation (every 30 ticks):            │              ║
║  │    OscillatorySignature per faction → coherence × synthesis│             ║
║  │    × resilience × longevity → CultureEmergenceEvent       │              ║
║  │    inter_group cos(Δfreq) < -0.25 → CultureConflictEvent  │              ║
║  │                                                           │              ║
║  │  faction_identity ──→ bridge_metrics_collect              │              ║
║  │  [SF-1/4 ✅] metrics_snapshot + health_dashboard          │              ║
║  │                                                           │              ║
║  │  [GS-6 ⏳] node_control_update: pendiente Oleada 2        │              ║
║  │  [GS-5 ✅] victory_check:                                 │              ║
║  │    qe(nucleus) < QE_MIN → VictoryEvent → PlayState::Victory│             ║
║  └──────────────────────────┬───────────────────────────────┘              ║
║                             ▼                                               ║
║  ┌──────────── Phase::MorphologicalLayer ───────────────────┐              ║
║  │                                                           │              ║
║  │  Shape: shape_opt ──→ rugosity ──→ albedo inference        │              ║
║  │  Growth: intent ──→ allometric_growth (TL6 ✅)            │              ║
║  │  Organs: viability ──→ lifecycle stage (emerge→mature→decay)│             ║
║  │  Evolution: surrogate phenotype sampling                   │              ║
║  │  Adaptation: Bergmann/Allen/Wolff (every 16 ticks)         │              ║
║  │  Reproduction: spawn child if radius ≥ threshold           │              ║
║  │              inherit InferenceProfile + mutation drift     │              ║
║  │  Abiogenesis: spontaneous life from leftover qe            │              ║
║  │                                                           │              ║
║  │  [IWG-2 ✅] body_plan_assembler:                          │              ║
║  │    InferenceProfile → BodyPlanLayout (bilateral assembly)  │              ║
║  │  [IWG-6 ✅] atmosphere_inference:                         │              ║
║  │    EnergyFieldGrid → sun/fog/bloom (every 30 ticks)        │              ║
║  │  IWG ✅: terrain_mesh_gen ──→ water_surface               │              ║
║  │                                                           │              ║
║  │  [ET-10 ✅] timescale_lod (every LOD interval):           │              ║
║  │    TimescaleLOD → macro-step for distant entities          │              ║
║  │  [ET-11 ✅] aggregate_signal_update (after timescale):     │              ║
║  │    AggSignal per cell → multi-scale info compression       │              ║
║  │  [ET-12 ✅] tectonic_stress_apply (every 1000 ticks):      │              ║
║  │    TectonicStress → TerrainField mutation                  │              ║
║  │  [ET-13 ✅] geological_lod_adjust (after tectonic):        │              ║
║  │    LODPhysics per region → skip atomic for far entities    │              ║
║  │  [ET-14 ✅] institution_rule_apply (after coalitions):     │              ║
║  │    InstitutionRole → qe tax / bonus on members             │              ║
║  │  [ET-15 ✅] symbol_compress (after ET-3):                  │              ║
║  │    SymbolSet → compressed meme transmission                │              ║
║  │  [ET-16 ✅] self_model_update (after ET-2):                │              ║
║  │    SelfModel → agency_index = coherence × planning_depth   │              ║
║  │                                                           │              ║
║  │  [GS-7 ⏳] visual_contract_sync: pendiente Oleada 3       │              ║
║  │    (diseñado: freq→hue | qe→luminance | dmg→saturation)   │              ║
║  └──────────────────────────┬───────────────────────────────┘              ║
║                             ▼                                               ║
║  [SF-4 ✅] metrics_export: SimulationHealthDashboard → CSV/JSON             ║
║  [SF-7 ✅] 11 integration tests: roundtrip+causalidad+health+CSV            ║
║                                                                             ║
╠═════════════════════════════════════════════════════════════════════════════╣
║                     SimWorld Boundary (sim_world.rs) ✅                     ║
║                                                                             ║
║  SimWorld::new(SimConfig)  ──→ headless App + FixedUpdate manual  ✅       ║
║  SimWorld::tick(&[InputCommand]) ──→ FixedUpdate schedule         ✅       ║
║  SimWorld::snapshot() ──→ WorldSnapshot (owned, no ECS types)     ✅       ║
║  SimWorld::energy_hash() ──→ u64  (determinism check)             ✅       ║
║  checkpoint_save_system / checkpoint_load_startup_system          ✅ SF-5  ║
║  apply_input(commands) ──→ TODO entity_id→Entity lookup           ⚠       ║
║                                                                             ║
║  INV-1: zero render deps   INV-4: deterministic   INV-5: renderer read-only║
║  INV-6: events live 1 tick INV-7: conservation    INV-8: tick_id only clock║
╠═════════════════════════════════════════════════════════════════════════════╣
║                     PostUpdate (on-demand)                                  ║
║                                                                             ║
║  [GS-1 ✅] lockstep_checksum_record: energies sorted → hash → ChecksumLog  ║
║  [GS-1 ✅] lockstep_desync_check:    checksums → DesyncEvent if differ     ║
║  [GS-2 ⏳] rollback_frame_save:      pendiente Oleada 2                    ║
║  [GS-2 ⏳] rollback_detect:          pendiente Oleada 2                    ║
║  [GS-2 ⏳] rollback_apply:           pendiente Oleada 2                    ║
╠═════════════════════════════════════════════════════════════════════════════╣
║                     FixedUpdate → Update bridge                             ║
║                                                                             ║
║  TerrainMeshResource ──────┐                                               ║
║  WaterMeshResource ────────┼──→ Snapshot pattern (Resource → Update) ✅    ║
║  AtmosphereState ──────────┘    (IWG-6 ✅: inferred desde EnergyField)     ║
║  VisualHints ──────────────────→ (GS-7 ⏳: pendiente Oleada 3)             ║
╠═════════════════════════════════════════════════════════════════════════════╣
║                                                                             ║
║  ┌──────────────────── Update (visual) ─────────────────────┐              ║
║  │                                                           │              ║
║  │  shape_color_inference (frequency → palette) ✅           │              ║
║  │  growth_morphology (organ → mesh deformation) ✅          │              ║
║  │  body_plan_visual (BodyPlanLayout → bilateral mesh) ✅     │              ║
║  │  phenology_visual (seasonal tint) ✅                      │              ║
║  │                                                           │              ║
║  │  [GS-7 ⏳] VisualHints sync: freq/qe/dmg/speed → HLSA    │              ║
║  │                                                           │              ║
║  │  terrain_mesh_sync ──→ Mesh3d (relieve + vertex colors)   │              ║
║  │  water_mesh_sync ──→ Mesh3d (plano azul, AlphaBlend)      │              ║
║  │  atmosphere_sync ──→ DirectionalLight + fog + bloom        │              ║
║  └───────────────────────────────────────────────────────────┘              ║
╚═════════════════════════════════════════════════════════════════════════════╝

┌─────────────────────────────────────────────────────────────────────────────┐
│              GAMEPLAY LAYER (GS track — 3/9 completos)                      │
│                                                                             │
│  ┌─── Netcode ───────────────────────────────────────────────────────────┐ │
│  │  GS-1 ✅ Lockstep:  InputBuffer + ChecksumLog + LockstepRunCondition  │ │
│  │  GS-2 ⏳ Rollback:  RollbackBuffer(circular) + detect + re-simulate   │ │
│  │  Protocolo: solo inputs cruzan la red — world state nunca transmitido  │ │
│  └───────────────────────────────────────────────────────────────────────┘ │
│                                                                             │
│  ┌─── AI Táctica ────────────────────────────────────────────────────────┐ │
│  │  GS-3 ✅ Nash:  FocusFire = argmin(qe / capacity × resonance_factor)  │ │
│  │  GS-4 ⏳ Pack:  cohesion_force + flee_vec → WillActuator::social_intent│ │
│  └───────────────────────────────────────────────────────────────────────┘ │
│                                                                             │
│  ┌─── Game Loop ─────────────────────────────────────────────────────────┐ │
│  │  GS-5 ✅ Victory:  VictoryNucleus + qe < QE_MIN → VictoryEvent        │ │
│  │  GS-6 ⏳ Map:      ControlNode + node_drain(EnergyFieldGrid) → snowball│ │
│  └───────────────────────────────────────────────────────────────────────┘ │
│                                                                             │
│  ┌─── Visual + Personajes + Onboarding ──────────────────────────────────┐ │
│  │  GS-7 ⏳ Visual:    VisualHints inyectivo — freq/qe/dmg/speed → HLSA  │ │
│  │  GS-8 ⏳ Archetype: ArchetypeConfig RON → spawn_from_config(14 capas) │ │
│  │  GS-9 ⏳ Onboard:   TutorialState × 5 escenas emergentes              │ │
│  └───────────────────────────────────────────────────────────────────────┘ │
└─────────────────────────────────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────────────────────────────────┐
│              SIMULATION FOUNDATIONS (SF track — ✅ completo)                │
│                                                                             │
│  SF-1 ✅ Observability:  SimulationHealthDashboard — métricas nombradas    │
│  SF-2 ✅ Serialization:  #[derive(Serialize, Deserialize)] en 14 capas     │
│  SF-3 ✅ Signal Latency: signal_speed(medium) → delay_ticks — física real  │
│  SF-4 ✅ Metrics Export: CSV/JSON a disco cada N ticks                     │
│  SF-5 ✅ Checkpoint:     save/load systems — env-gated                     │
│  SF-6 ✅ Propagation:    EnergyFieldGrid WaveFront multi-tick              │
│  SF-7 ✅ Integration:    11 tests — roundtrip+causalidad+health+CSV        │
└─────────────────────────────────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────────────────────────────────┐
│           EMERGENCE_TIERS (ET track — ✅ 16/16 completos)                  │
│                                                                             │
│  ┌─── ET Tier 1 — adaptación individual ✅ ─────────────────────────────┐ │
│  │  ET-1  ✅ Associative Memory     layers/memory.rs + eq/emergence/    │ │
│  │  ET-2  ✅ Theory of Mind         layers/other_model.rs               │ │
│  │  ET-3  ✅ Cultural Transmission  simulation/emergence/culture.rs     │ │
│  │  ET-4  ✅ Field Infrastructure   simulation/emergence/infrastructure │ │
│  └───────────────────────────────────────────────────────────────────────┘ │
│                                                                             │
│  ┌─── ET Tier 2 — organización colectiva ✅ ────────────────────────────┐ │
│  │  ET-5  ✅ Obligate Symbiosis     layers/symbiosis.rs                 │ │
│  │  ET-6  ✅ Epigenetic Expression  layers/epigenetics.rs               │ │
│  │  ET-7  ✅ Programmed Senescence  layers/senescence.rs                │ │
│  │  ET-8  ✅ Dynamic Coalitions     simulation/emergence/coalitions.rs  │ │
│  │  ET-9  ✅ Multidimensional Niche layers/niche.rs                     │ │
│  └───────────────────────────────────────────────────────────────────────┘ │
│                                                                             │
│  ┌─── ET Tier 3 — escala espaciotemporal ✅ ────────────────────────────┐ │
│  │  ET-10 ✅ Multiple Timescales    layers/timescale.rs                 │ │
│  │  ET-11 ✅ Multi-Scale Info       simulation/emergence/multiscale.rs  │ │
│  │  ET-12 ✅ Continental Drift      simulation/emergence/tectonics.rs   │ │
│  │  ET-13 ✅ Geological Time LOD    simulation/emergence/geological_lod │ │
│  └───────────────────────────────────────────────────────────────────────┘ │
│                                                                             │
│  ┌─── ET Tier 4 — meta-emergencia ✅ ───────────────────────────────────┐ │
│  │  ET-14 ✅ Institutions           simulation/emergence/institutions   │ │
│  │  ET-15 ✅ Language               layers/language.rs                  │ │
│  │  ET-16 ✅ Functional Consciousness layers/self_model.rs              │ │
│  └───────────────────────────────────────────────────────────────────────┘ │
│                                                                             │
│  127 unit tests · 16 BridgeKind markers · simulation/emergence/ wired      │
│  Archivado en: docs/sprints/archive/EMERGENCE_TIERS/                        │
└─────────────────────────────────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────────────────────────────────┐
│              CULTURA (CE track — ✅ implementado)                           │
│                                                                             │
│  Cultura(G) = coherencia × síntesis × resiliencia × longevidad             │
│  Derivada de OscillatorySignature (L2) + Homeostasis (L12) + catálisis     │
│                                                                             │
│  blueprint/equations/culture.rs   — 11 funciones puras, 50 tests           │
│  blueprint/constants/culture.rs   — 7 constantes derivadas de física       │
│  simulation/culture_observation.rs — system throttled each 30 ticks        │
│                                                                             │
│  CultureEmergenceEvent  (rising edge: coherencia × síntesis × resiliencia) │
│  CultureConflictEvent   (cos(Δfreq) < -0.25 entre dos facciones)           │
└─────────────────────────────────────────────────────────────────────────────┘
```

---

## Las 14 Capas Ortogonales

```
L0  BaseEnergy ──────── qe (existencia)          ← TODO toca esto
L1  SpatialVolume ───── radius (colisión)         ← allometric growth (TL6)
L2  OscillatorySignature freq, phase (resonancia) ← homeostasis, catalysis, Nash, cultura
L3  FlowVector ──────── velocity, drag            ← physics, locomotion
L4  MatterCoherence ─── state, bond_energy        ← state transitions
L5  AlchemicalEngine ── buffer, valves             ← engine processing, nucleus
L6  AmbientPressure ─── delta_qe, viscosity       ← climate, terrain, SF-3
L7  WillActuator ────── intent, channeling         ← behavior AI, pack ⏳ GS-4
L8  AlchemicalInjector  projected_qe, freq         ← spell payload
L9  MobaIdentity ────── faction, tags, crit        ← game rules, ⏳ GS-6 nodes
L10 ResonanceLink ───── buff/debuff overlay        ← spell side effects
L11 TensionField ────── attract/repel force        ← gravity/magnetic
L12 Homeostasis ─────── freq adaptation + cost     ← chemical layer (TL1)
L13 StructuralLink ──── spring joint, stress       ← structural constraint

+ Auxiliares: OrganManifest, MetabolicGraph, GrowthBudget, Grimoire,
  TrophicState, NutrientProfile, InferenceProfile, BodyPlanLayout,
  PackMembership, VictoryNucleus ✅, ControlNode ⏳ GS-6,
  InputPacket (SparseSet) ✅ GS-1

+ ET layers ✅ (implementados):
  AssociativeMemory, SymbiosisLink, SenescenceClock, TimescaleLOD,
  OtherModel, MemeVector, EpigeneticMask, NicheVector, AggSignal,
  FieldModRecord, CoalitionId, TectonicStress, LODPhysics,
  InstitutionRole, SymbolSet, SelfModel
```

---

## Flujo de Eventos

```
Input    ──→ PathRequestEvent ──→ pathfinding (mismo phase)
Input    ──→ SlotActivatedEvent ──→ targeting/channeling/resolve (SM-8G ✅)
Input    ──→ CastPending ──→ ThermodynamicLayer (spawn projectile)
Atomic   ──→ CollisionEvent ──→ telemetría
Chemical ──→ CatalysisRequest ──→ math ──→ DeltaEnergyCommit (chain)
Chemical ──→ PhaseTransitionEvent ──→ observabilidad
*        ──→ DeathEvent ──→ nutrient_return + faction_update
Metabolic──→ HungerEvent ──→ behavior
Metabolic──→ PreyConsumedEvent ──→ social
Metabolic──→ AllianceProposedEvent ──→ coalitions/AI   [AC-5 ✅]
Metabolic──→ AllianceDefectEvent ──→ reputation/AI     [AC-5 ✅]
Metabolic──→ CultureEmergenceEvent ──→ observabilidad / HUD  [CE ✅]
Metabolic──→ CultureConflictEvent  ──→ behavior AI           [CE ✅]
Metabolic──→ NodeCapturedEvent ──→ observabilidad + HUD       [GS-6 ⏳]
Metabolic──→ VictoryEvent ──→ PlayState::Victory + GameOutcome [GS-5 ✅]
PostUpdate──→ DesyncEvent ──→ rollback / resync request        [GS-1 ✅/GS-2 ⏳]

Invariante: productor siempre en Phase anterior al consumidor
Invariante: eventos viven 1 tick (INV-6)
Invariante: VictoryEvent emitido una sola vez (GameOutcome idempotente)
```

---

## Resources Clave

```
── Simulación base ✅ ────────────────────────────────────────────────
EnergyFieldGrid   ← propagation multi-tick (SF-6 ✅)
                  → terrain_visuals, water_surface, materialization
                  → atmosphere_inference (IWG-6 ✅), abiogenesis, sensory
                  → node_control drain (GS-6 ⏳)

TerrainField      ← startup (noise + hydraulics) ✅
                  → terrain_mesh_gen, water_surface, terrain_effects

SpatialIndex      ← update_spatial_index (AtomicLayer) ✅
                  → sensory_scan, catalysis_filter, trophic

SimulationClock   ← advance_clock (SimulationClockSet) ✅
                  → atmosphere throttle, ecology intervals, cooldowns
                  → lockstep tick gate (GS-1 ✅)
                  → culture_observation throttle (every 30 ticks, CE ✅)

AlchemicalAlmanac ← startup (RON elements) ✅
                  → frequency→element, catalysis, visual derivation

NutrientFieldGrid ← startup seed + osmosis regen (TL1/TL2 ✅)
                  → uptake, depletion, death return

SimulationHealthDashboard ← metrics nombradas (SF-1/4 ✅)
CheckpointConfig  ← env-gated save/load (SF-5 ✅)
PropagationMode   ← Legacy | WaveFront (SF-6 ✅)

── Gameplay Systems ──────────────────────────────────────────────────
LockstepConfig      ✅ GS-1
InputBuffer         ✅ GS-1
ChecksumLog         ✅ GS-1
LockstepRunCondition✅ GS-1
RollbackBuffer      ⏳ GS-2
RollbackState       ⏳ GS-2
NashTargetConfig    ✅ GS-3
PackDynamicsConfig  ⏳ GS-4
GameOutcome         ✅ GS-5
NodeControlConfig   ⏳ GS-6
VisualHints         ⏳ GS-7
ArchetypeRegistry   ⏳ GS-8
OnboardingConfig    ⏳ GS-9
```

---

## Métricas (estado actual)

```
~170 sistemas, 6 fases, 14 capas + 16 ET layers, ~55 Resources, ~24 tipos de evento.
Ecuaciones puras: 20K+ LOC en blueprint/equations/ (50+ módulos incl. emergence/).
Tests: 2,150 (lib) + integración (sf_integration×11, r1-r9, morphogenesis, energy_competition, g11, ...).
Todo determinista. Todo encadenado. Todo emergente desde energía.
Serializable (SF-2/5 ✅). Observable (SF-1/4 ✅). Causalidad verificada (SF-7 ✅).
ET completo: 16 capas de emergencia, 127 unit tests. Cultura CE: 50 unit tests.
AC completo: 5 dinámicas cross-axiom, 60+ equation tests, 7 integration tests.

Pendiente: GS-2/4/6/7/8/9 (gameplay layer).
```

---

## Scorecard Actual

| Dominio | Nota | Estado |
|---------|------|--------|
| Termodinámica (TL1–TL6) | A | Completo: osmosis, nutrientes, GrowthBudget, fotosíntesis, deformación, allometría |
| 14 Capas | A | Completas, read+write, max 4 fields cada una |
| Ecuaciones puras | A | 20K+ LOC, 50+ módulos, sin inline en sistemas |
| Morfogénesis (MG1–8) | A | DAG + entropía + forma + albedo + rugosidad |
| Worldgen V7 (IWG1–7) | A | Body plan bilateral + atmósfera + terrain mesh |
| Simulación Fundacional (SF1–7) | A | Observabilidad + checkpoint + wavefront + 11 tests ✅ |
| Energy Competition (EC1–8) | A- | Pools + conservation + scale |
| Emergence Tiers (ET1–16) | A | 16/16 sprints: memoria, mente, cultura, coalición, consciencia |
| Cultura CE | A | OscillatorySignature → coherencia, síntesis, conflicto (50 tests) |
| Axiomatic Closure (AC) | A | 5/5 cross-axiom dynamics: interference, entrainment, culture×freq, purity, cooperation (60+ tests) |
| Crecimiento/Lifecycle | A- | Sólido, reproductive loop funcional |
| Ecología | A- | Census + succession + trophic |
| Combate/Habilidades | A- | Sistema funcional, SRP completo (SM-8G ✅) |
| SimWorld Boundary | A- | 8 invariantes ✅, apply_input TODO pendiente ⚠ |
| Sensorial | B+ | Budgeted + signal latency (SF-3 ✅) |
| Netcode | B+ | Lockstep ✅ (GS-1), Rollback ⏳ (GS-2) |
| AI Táctica | B+ | Nash ✅ (GS-3), Pack ⏳ (GS-4) |
| Game Loop | B+ | Victory ✅ (GS-5), Map energy ⏳ (GS-6) |
| Visual Contract | C+ | Diseñado, pendiente implementación (GS-7 ⏳) |
| Arquetipos RON | C+ | Diseñado, pendiente implementación (GS-8 ⏳) |
| Code Quality | A | SM-8 completo — cero god-systems, cero magic numbers |

**Nota general: A-** — Motor físico A, emergence completo A, gameplay parcial B+.

---

## Las Dos Almas (invariante de diseño)

```
¿Podés remover MOBA y que la simulación funcione?
  → SÍ. La termodinámica, ecología, crecimiento, reproducción
    y morfogénesis son independientes del gameplay.

¿Podés remover simulación y que el MOBA funcione?
  → NO. El daño emerge de catalysis (interferencia de frecuencia).
    Sin termodinámica no hay combate. El MOBA ES la física.

Entanglement asimétrico (invariante permanente):
  MOBA       depende de Simulación (100%)
  Simulación NO depende de MOBA    (0%)
  Emergence  depende de Simulación (100%) — organs como implementación de capas
```

---

## Estado de oleadas

```
OLEADA 1 ✅        SF-7 ✅        ET ✅             OLEADA 2 ⏳        OLEADA 3 🔒
──────────────    ───────────    ──────────────    ───────────────    ─────────────
SF-4 ✅ CSV  ─┐                  ET-1..ET-16 ✅    GS-2 ⏳ Rollback ─┐
SF-5 ✅ Check─┤──→ SF-7 ✅       127 unit tests    GS-4 ⏳ Pack     ─┤  GS-7 ⏳ Visual─┐
SF-6 ✅ Wave ─┘  11 tests        (archivado)       GS-6 ⏳ Map NRG  ─┘  GS-8 ⏳ RON   ─┤──→ GS-9 ⏳ DEMO
                                                                        GS-9 ⏳ (7+8)  └┘
GS-1 ✅ Lock ─┐
GS-3 ✅ Nash ─┤
GS-5 ✅ Vic  ─┘

SM-8D/F/G ✅ ─┘

CE (Cultura) ✅:  culture_observation_system — 50 tests — wired MetabolicLayer

AC (Axiomatic Closure) ✅:  5 cross-axiom dynamics — 60+ tests — wired Atomic+Metabolic
  AC-1 ✅ Interference × Extraction    AC-4 ✅ Frequency Purity
  AC-2 ✅ Kuramoto Entrainment         AC-5 ✅ Cooperation Emergence
  AC-3 ✅ Culture × Frequency

Total: 31 ✅ completados · 7 ⏳ pendientes
```
