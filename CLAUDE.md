# Resonance — Claude Code Instructions

## Project

Resonance is an alchemical MOBA in Rust/Bevy 0.15 where everything is energy (qe). 14 orthogonal ECS layers define all entities by composition. Gameplay is 100% emergent from energy interactions; interfaces are standard MOBA (Dota/LoL style).

## Architecture

- **14 layers** (L0 BaseEnergy → L13 StructuralLink). See `layers/` (+ aux: nutrient, irradiance, inference, vision_fog, growth, …).
- **Pipeline:** `FixedUpdate` + `Time<Fixed>` / `SimulationTickPlugin`. Phases = `SystemSet`s in `simulation/pipeline.rs`:
  `SimulationClockSet` → `Phase::Input` → `Phase::ThermodynamicLayer` → `Phase::AtomicLayer` → `Phase::ChemicalLayer` → `Phase::MetabolicLayer` → `Phase::MorphologicalLayer`.
- **Pure math** in `blueprint/equations/` (`mod.rs` + domains). NEVER inline formulas in systems.
- **Constants** in `{module}/constants.rs` or `{module}/constants/mod.rs` (+ domain shards).
- **Stateless-first:** Pure functions + Resources. Components hold state, systems transform it.
- **Pattern: Layered ECS with Vertical Slices.** NOT hexagonal. Components = domain, systems = use cases, Bevy = infrastructure. No ports/adapters.

## Stack (Hard Constraints)

| Layer | Technology | Version | Notes |
|-------|-----------|---------|-------|
| Language | Rust | stable 2024 edition | MSRV 1.85 |
| Engine | Bevy | 0.15.x | ECS + rendering + input |
| Math | glam 0.29 (direct) | `math_types.rs` | Vec2, Vec3, f32 ops — decoupled from bevy::math |
| Async | None | — | Bevy schedule only, no tokio/async-std |

## Module Map (`src/lib.rs`)

```
math_types.rs       → Engine-agnostic glam re-exports (Vec2, Vec3, Quat). All non-ECS code imports from here.
batch/              → Batch simulator: millions of worlds without Bevy (rayon parallel)
  arena.rs          → EntitySlot (flat entity, repr(C)), SimWorldFlat (64 slots + grids)
  systems/          → 33 stateless systems (6 phases), call blueprint/equations/ for math
  genome.rs         → GenomeBlob (DNA: 4 biases + archetype), mutate, crossover
  harness.rs        → GeneticHarness (evaluate → select → reproduce), FitnessReport
  bridge.rs         → GenomeBlob ↔ Bevy components (lossless round-trip), save/load binary
  batch.rs          → WorldBatch (N worlds), BatchConfig, rayon par_iter_mut
  lineage.rs        → LineageId (deterministic ancestry), TrackedGenome (genome + parent_id)
  census.rs         → EntitySnapshot (per-entity state), PopulationCensus (per-gen capture, HOF distribution/mean)
blueprint/          → Types, equations/, constants/, almanac/, abilities, recipes, ids, validator, morphogenesis
  equations/        → Pure math facade (45+ domain files). Key domains:
    abiogenesis/    → Legacy potential + axiomatic.rs (coherence-driven, axiom-derived)
    batch_fitness.rs → composite_fitness, tournament_select, crossover_uniform
    core_physics/   → interference, density, dissipation, state transitions
    determinism.rs  → hash_f32_slice, next_u64, unit_f32, range_f32, gaussian_f32
    entity_shape.rs → GF1 influence, constructal optimizer, organ_slot_scale(mobility)
    radiation_pressure.rs → Frequency-coherent outward push (Axiom 8)
    awakening.rs      → Awakening potential (coherence vs dissipation threshold)
    derived_thresholds.rs → ALL lifecycle constants from 4 fundamentals (12 tests)
  morphogenesis/    → Constructal (shape_cost, drag, fineness), surface (rugosity, albedo), thermodynamics
bridge/             → Cache optimizer (BridgeCache<B>, 11 equation kinds) + constants.rs
eco/                → Eco-boundaries, zones, climate + systems.rs
entities/           → EntityBuilder (.wave_from_hz for axiomatic), archetypes (spawn_*), composition
events.rs           → Event contracts (cast, catalysis, path, death, worldgen, …); see bootstrap.rs
geometry_flow/      → GF1 flora-tube (branching stateless), merge_meshes (canonical), deformation
layers/             → 14 ECS layers + auxiliaries (24+ files)
plugins/            → SimulationPlugin, LayersPlugin, DebugPlugin, MorphologicalPlugin
rendering/          → quantized_color (+ QuantizedColorPlugin in main.rs)
runtime_platform/   → 17 sub-modules: compat_2d3d, tick, input, camera, HUD, fog_overlay, …
simulation/         → pipeline, bootstrap, pathfinding, fog, growth, photosynthesis, …
  abiogenesis/      → Axiomatic abiogenesis: coherence_gain > dissipation → spawn (any frequency band)
  emergence/        → ET systems: theory_of_mind, symbiosis_effect, epigenetic_adaptation, niche_adaptation,
                      culture, entrainment, coalitions (+ stubs: infrastructure, institutions, etc.)
  lifecycle/        → constructal_body_plan, entity_shape_inference (compound mesh), body_plan_layout
  awakening.rs      → Inert entities gain BehavioralAgent when coherence > threshold (axiom-derived)
  metabolic/        → basal_drain (passive qe cost), senescence_death (age-based mortality),
                      trophic (herbivore/carnivore/decomposer), growth_budget, metabolic_stress
  reproduction/     → Flora seed dispersal + fauna offspring (inherits mutated InferenceProfile incl. mobility_bias)
topology/           → Terrain: noise, slope, drainage, classifier, hydraulics, mutations, config
world/              → SpatialIndex, demos, presets; maps = assets/maps/*.ron
worldgen/           → V7: field_grid, nucleus (+NucleusReservoir), propagation, materialization, shape_inference, nutrient_field
  systems/          → startup, prephysics, propagation, materialization, terrain, visual, performance,
                      radiation_pressure (non-linear outward push), nucleus_recycling (nutrient→new nucleus)
```

**Maps:** `RESONANCE_MAP` → `assets/maps/{name}.ron` (`worldgen/map_config.rs`).
**Headless:** `cargo run --bin headless_sim -- --ticks 10000 --scale 8 --out world.ppm` (sim → PPM image, no GPU).
**Events:** `simulation/bootstrap.rs` — 15 `Event` types (incl. `TerrainMutationEvent`); `PathRequestEvent` in `Compat2d3dPlugin`.
**Docs:** `docs/arquitectura/README.md` (module blueprints). Folder structure: `docs/design/FOLDER_STRUCTURE.md`.

## The 8 Foundational Axioms

**⚠️ INVIOLABLE: No change, feature, refactor, or optimization may contradict, bypass, or weaken ANY of the 8 axioms or 4 fundamental constants. If a proposed change conflicts with an axiom, the change is WRONG — not the axiom. This is the constitution of the project. No exceptions. No DEBT. No "temporary" violations.**

All simulation behavior MUST derive from these. No arbitrary constants, no per-element special cases.

### Primitive axioms (independent, irreducible)

1. **Everything is Energy** — All entities are qe. No separate HP/mana/stats.
2. **Pool Invariant** — `Σ energy(children) ≤ energy(parent)`. Conservation absolute.
4. **Dissipation (2nd Law)** — All processes lose energy. `loss ≥ qe × rate`. No 100% efficiency.
7. **Distance Attenuation** — `interaction_intensity` monotonically decreasing in distance.
8. **Oscillatory Nature** — Every concentration oscillates at frequency f. Interaction modulated by `cos(Δf × t + Δφ)`.

### Derived axioms (consequences of primitives, elevated for clarity)

3. **Competition as Primitive** — `magnitude = base × interference_factor`. *Derived from Axiom 8 applied to energy transfer. Kept as axiom because it's the most cited design constraint.*
5. **Conservation** — Energy never created, only transferred/dissipated. Total qe monotonically decreases. *Derived from Axiom 2 + Axiom 4. Kept as axiom because INV-7 (SimWorld) enforces it explicitly.*
6. **Emergence at Scale** — Behavior at scale N = consequence of interactions at scale N-1. No top-down programming. *Meta-rule constraining the developer, not the physics. Kept as axiom because it prevents hardcoded trophic classes, faction tags, and behavior scripts.*

> **Note:** The 3 derived axioms produce zero additional simulation behavior. Removing them from the list would not change a single tick of simulation output. They exist as guard rails for design decisions, not as physics laws.

**Cross-axiom compositions:** `docs/design/AXIOMATIC_CLOSURE.md`. Runtime contracts: `docs/arquitectura/blueprint_axiomatic_closure.md`.

## The 4 Fundamental Constants

The 8 axioms define the **rules**. These 4 constants are the **parameters** — the only numeric values that cannot be derived further. Everything else is computed algebraically from these via `blueprint/equations/derived_thresholds.rs`.

### Truly fundamental (physics)

| Constant | Value | Axiom | Justification |
|----------|-------|-------|---------------|
| `KLEIBER_EXPONENT` | 0.75 | Axiom 4 | Biological universal: metabolic rate ∝ mass^0.75 (validated across 27 orders of magnitude) |
| `DISSIPATION_{SOLID,LIQUID,GAS,PLASMA}` | 0.005, 0.02, 0.08, 0.25 | Axiom 4 | Second Law dissipation rate per matter state (empirical physics, ratios 1:4:16:50) |

### Grid/game calibration (not derivable from physics, but constrained by axioms)

| Constant | Value | Axiom | Justification |
|----------|-------|-------|---------------|
| `COHERENCE_BANDWIDTH` | 50.0 Hz | Axiom 8 | Observation window for frequency interference. Defines elemental band width. Circular with game design (Terra band = 85-110 Hz). |
| `DENSITY_SCALE` | 20.0 | Axiom 1 | Spatial normalization factor. Depends on grid cell_size. Artefact of spatial resolution, not physics. |

> **Note:** KLEIBER + DISSIPATION_RATIOS are physics. BANDWIDTH + DENSITY_SCALE are grid/game calibration. If you change cell_size or frequency bands, recalibrate these two — but never touch KLEIBER or the dissipation ratios for calibration purposes.

**Derivation chain** (computed, not hardcoded):
```
Fundamentals (4)
├── KLEIBER + DISSIPATION ratios → matter state thresholds (Solid→Liquid→Gas→Plasma)
├── DISSIPATION_SOLID → basal_drain_rate, senescence_coeff_materialized, bond_energy_scale
├── DISSIPATION_LIQUID → senescence_coeff_fauna, nutrient_retention_water
├── DISSIPATION_GAS → radiation_pressure_threshold, radiation_pressure_transfer_rate
├── Threshold ratios → move_density_min/max, sense_coherence_min, branch_qe_min
├── 1/coeff → max_viable_age (Gompertz inverse)
├── exp(-2) → survival_probability_threshold (Gompertz 1/e² point)
└── 1/3 → spawn_potential_threshold (algebraic break-even)
```

See `docs/sprints/AXIOMATIC_INFERENCE/` for full sprint docs. Implementation: `src/blueprint/equations/derived_thresholds.rs` (12 tests).

## Morphogenesis Pipeline

Shapes emerge from energy composition, not templates. Full pipeline:

```
FixedUpdate / MorphologicalLayer:
  shape_optimization_system    → MorphogenesisShapeParams.fineness_ratio (bounded_fineness_descent)
  surface_rugosity_system      → MorphogenesisSurface.rugosity (inferred_surface_rugosity)
  albedo_inference_system      → InferredAlbedo.albedo (inferred_albedo)
  epigenetic_adaptation_system → EpigeneticState.expression_mask (env → gene silencing)
  constructal_body_plan_system → BodyPlanLayout (optimal_appendage_count → N limbs)

Update / after sync_visual:
  entity_shape_inference_system:
    torso = build_flow_spine → build_flow_mesh (main GF1 tube)
    organs = for each slot in BodyPlanLayout:
             organ_slot_scale(slot, count, mobility_bias) → sub-influence → sub-mesh
    final = merge_meshes([torso, organs...]) → V6VisualRoot.Mesh3d
```

**Key equations:** `optimal_appendage_count` (drag × thrust_efficiency + maintenance), `organ_slot_scale` (front/rear asymmetry from mobility_bias), `frequency_alignment` (Gaussian coherence).

## Axiomatic Abiogenesis

Life emerges where `coherence_gain(neighbors) > dissipation_loss(local)`. Frequency-agnostic — any band can produce life.

```
Axiom 8: neighbor coherence = Σ qe_i × alignment(f_center, f_i) × attenuation(d_i)
Axiom 4: dissipation cost = cell_qe × dissipation_rate(matter_state)
Axiom 1: matter_state = f(energy_density), capabilities = f(density, coherence)
→ potential = (coherence - dissipation) / (coherence - dissipation + qe)
→ spawn if potential > threshold
```

Entity properties derived from energy state: matter_state_from_density, capabilities_from_energy, inference_profile_from_energy. No per-band constants.

## Energy Cycle (Closed Loop)

```
Nucleus (finite reservoir) → emits to field → diffusion + radiation pressure
    ↓                                                    ↓
Reservoir depletes (→0)                    Entities materialize (SenescenceProfile)
    ↓                                                    ↓
Zone cools down                            Live (basal_drain) → die (senescence/starvation)
    ↓                                                    ↓
                        Nutrients return to grid (nutrient_return_on_death_system)
                                     ↓
                        Threshold reached → nucleus_recycling_system → new finite nucleus
                                     ↓
                                 Cycle restarts
```

**Key systems:**
- `NucleusReservoir` (SparseSet): finite fuel, drained per tick by `propagate_nuclei_system`
- `basal_drain_system` (MetabolicLayer): passive qe cost ∝ radius^0.75 × age_factor (Kleiber)
- `senescence_death_system` (MetabolicLayer): hard age limit + Gompertz hazard
- `radiation_pressure_system` (ThermodynamicLayer): frequency-coherent outward push (Axiom 8)
- `nucleus_recycling_system` (MorphologicalLayer): nutrients accumulate → spawn new nucleus
- `awakening_system` (MorphologicalLayer): inert entities gain BehavioralAgent when coherence > threshold

**Axiom-derived constants:** `blueprint/equations/derived_thresholds.rs` — ALL lifecycle constants computed from 4 fundamentals:
- `KLEIBER_EXPONENT` (0.75), `DISSIPATION_{SOLID,LIQUID,GAS,PLASMA}`, `COHERENCE_BANDWIDTH`, `DENSITY_SCALE`
- Sprint `AXIOMATIC_INFERENCE` ✅ ARCHIVED (7/7 sprints) — see `docs/sprints/archive/AXIOMATIC_INFERENCE/`
- Visual calibration (rendering tuning, not physics): `src/worldgen/visual_calibration.rs`

## Evolution & Emergence Pipeline

Reproduction, mutation, selection, and group behavior — all axiom-derived.

```
reproduction_spawn_system (MorphologicalLayer):
  Flora: BRANCH cap → seed with mutated InferenceProfile (growth, mobility, branching, resilience)
  Fauna: MOVE + REPRODUCE caps + qe > 200 → offspring with full behavior stack + mutated profile
  Conservation: parent drained, offspring qe ≤ drained amount (Axiom 5)
  Mutation: deterministic from entity index (no RNG crate), all 4 biases mutate

Emergence systems (registered in plugins, active in runtime):
  Phase::Input:
    theory_of_mind_update_system    → OtherModelSet predictions from observed neighbors (ET-2)
  Phase::MetabolicLayer:
    symbiosis_effect_system         → mutualism/parasitism drain/benefit on SymbiosisLink (ET-5)
    niche_adaptation_system         → character displacement under competitive pressure (ET-9)
  Phase::MorphologicalLayer:
    epigenetic_adaptation_system    → environment modulates expression_mask (ET-6)

Already functional (from prior sprints):
  entrainment_system (AC-2)         → Kuramoto frequency sync between neighbors
  cooperation_evaluation_system (AC-5) → Nash alliance detection
  cultural_transmission_system (ET-3) → meme spread by imitation
  coalition_stability_system (ET-8) → coalition eval + intake bonus
```

**Stellar archetypes:** `spawn_star` (L0 high qe + L11 InverseSquare + EnergyNucleus) + `spawn_planet` (orbital velocity + surface conditions). Map: `stellar_system.ron`.

## The 14 Orthogonal Layers

```
L0  BaseEnergy           (existence — qe)
L1  SpatialVolume        (spatial volume — radius)
L2  OscillatorySignature (wave signature — frequency, phase)
L3  FlowVector           (flow — velocity, dissipation)
L4  MatterCoherence      (structural integrity — state, bond energy)
L5  AlchemicalEngine     (mana processor — buffer, valves)
L6  AmbientPressure      (terrain — delta_qe, viscosity)
L7  WillActuator         (will — intent, channeling)
L8  AlchemicalInjector   (spell payload — projected qe, forced freq)
L9  MobaIdentity         (game rules — faction, tags, crit)
L10 ResonanceLink        (buff/debuff — effect → target)
L11 TensionField         (gravity/magnetic force at distance)
L12 Homeostasis          (frequency adaptation with qe cost)
L13 StructuralLink       (spring joint between entities)
```

## Coding Rules

1. **English identifiers only.** Linter translates Spanish→English.
2. **Max 4 fields per component.** More = split into layers.
3. **One system, one transformation.** No god-systems (>5 component types).
4. **`SparseSet`** for transient components (buffs, markers, one-shot flags).
5. **Guard change detection.** `if val != new { val = new; }` or `set_if_neq`.
6. **Chain events.** Producer `.before()` or `.chain()` with consumer. Never unordered.
7. **Phase assignment required.** Every gameplay system → `.in_set(Phase::X)`.
8. **Math in `blueprint/equations/`.** Systems call pure fns, don't inline formulas.
9. **Component group factories** for spawning (`entities/component_groups.rs`). Pure functions returning tuples, composable via nested bundles. `EntityBuilder` for legacy; prefer factories for new code.
10. **Constants in constants.** Tuning values centralized per module. Algorithmic arrays stay in-file.
11. **`With<T>`/`Without<T>`** over `Option<&T>` for filter-only queries.
12. **Minimal query width.** Only request components you read/write.
13. **No `Vec<T>` in components** unless genuinely variable-length.
14. **`#[derive(Component, Reflect, Debug, Clone)]`** on every component. Register: `app.register_type::<T>()`.

## Hard Blocks (Defaults, Not Laws)

Rules are strong defaults. Violating one is allowed **if and only if** you document the justification inline with `// DEBT: <reason>`. Unjustified violations are still zero tolerance.

**Absolute (never violate):**
1. **NO `unsafe`** — zero tolerance. No exceptions.
2. **NO external crates** without approval — only what's in Cargo.toml.
3. **NO `async`/`await`** — Bevy schedule only.
4. **NO `Arc<Mutex<T>>`** — use `Resource` or `Local`.
5. **NO shared mutable state outside Resources** — no `static mut`, no `lazy_static! { Mutex }`.

**Strong defaults (violate with `// DEBT:` justification):**
6. **NO `HashMap` in hot paths** — sorted `Vec` or Entity indexing. *Prove it's hot before optimizing; benchmark first.*
7. **NO `String` in components** — enums, `u32` IDs, or `&'static str`.
8. **NO `Box<dyn Trait>`** in components — enums for closed sets.
9. **NO `#[derive(Bundle)]`** — Bevy 0.15 uses tuples or `#[require(...)]`.
10. **NO `ResMut` when `Res` suffices** — minimize write locks.
11. **NO `unwrap()`/`expect()`/`panic!()` in systems** — `let-else` or `if-let`. Tests OK. *If a spawn invariant guarantees Some, `// DEBT: invariant held by spawn_X` + unwrap is acceptable.*
12. **NO inline formulas** in systems — all math in `blueprint/equations/`.
13. **NO storing derived values** as components — compute at point of use.
14. **NO trait objects for game logic** — components + systems.
15. **NO component methods with side effects** — pure `&self` only, systems do work.
16. **NO `Entity` as persistent/network ID** — strong newtype IDs.
17. **NO systems in `Update` for gameplay** — `FixedUpdate` + `Phase` (except visual derivation).

## Bevy 0.15 Patterns

```rust
commands.spawn((CompA::new(), CompB::new()));                    // Tuple spawn (no Bundle)
app.add_systems(FixedUpdate, sys.in_set(Phase::MetabolicLayer)); // System registration
for (entity, mut energy, vol) in &mut query { ... }             // Query iteration
commands.spawn(Camera2d);                                        // Camera
app.register_type::<MyComp>();                                   // Reflect registration

#[require(BaseEnergy)]                                           // Component dependencies
#[component(storage = "SparseSet")]                              // Transient components
StateScoped(GameState::Playing)                                  // Auto-cleanup on state exit

if target.field != new_val { target.field = new_val; }          // Guard mutation
target.set_if_neq(new_val);                                     // Guard shorthand
Query<&Comp, Changed<Comp>>                                     // Skip unchanged archetypes
Query<&Energy, (With<Alive>, Without<Dead>)>                    // Filter without data access
OnAdd, OnRemove                                                  // Entity lifecycle observers
```

## Rust 2024 Edition Idioms

```rust
// Let chains
if let Some(e) = energy_q.get(id) && let Some(v) = vol_q.get(id) && e.qe > 0.0 { ... }

// Let-else
let Some(target) = query.get(entity) else { return; };

// Exhaustive match — NO _ wildcard on enums (compiler catches new variants)
match state { MatterState::Solid => .., Liquid => .., Gas => .., Plasma => .. }

// Iterator chains over index loops
let total: f32 = query.iter().filter(|(_, e)| !e.is_dead()).map(|(_, e)| e.qe).sum();

// Stack over heap: [f32; 3] not Vec | f32 not f64 | const fn where possible
```

## Code Aesthetic (Yanagi)

1. **Dense but readable** — one empty line between blocks, never two.
2. **Vertical alignment** — align similar patterns in columns.
3. **Let types speak** — skip comments when type is clear.
4. **Functional over imperative** — iterator chains, `map`, `filter`. No index loops.
5. **Early return** — guard clauses top, happy path unindented.
6. **`///` doc comments** on public fns — one line max. Include equation for math fns.
7. **Inline comments** only for non-obvious math or invariants.
8. **Imports grouped:** bevy → crate → super.
9. **Naming:** `SCREAMING_SNAKE` constants, `PascalCase` types, `snake_case` fns. Domain abbreviations OK: `qe`, `eb`, `freq`, `dt`.
10. **Bilingual comments** — All `///` doc comments must have both Spanish and English, one line each. Spanish first, English below. Minimalist: one sentence per language. Inline `//` comments can be single-language if trivial.

## Design Templates

```rust
// === System ===
use bevy::prelude::*;
use crate::layers::*;
use crate::blueprint::{constants, equations};

/// [What this system transforms and why].
pub fn my_system(
    mut query: Query<(&mut Target, &Source), Without<Dead>>,
    config: Res<MyConfig>,
) {
    for (mut target, source) in &mut query {
        let result = equations::some_calc(source.field, config.param);
        if target.field != result { target.field = result; }
    }
}

// === Component ===
/// Layer [N]: [Name] — [one-line purpose].
#[derive(Component, Reflect, Debug, Clone)]
#[reflect(Component)]
pub struct MyComp {
    field_a: f32,  // max 4 fields
    field_b: f32,
}
impl MyComp {
    pub fn new(a: f32, b: f32) -> Self { Self { field_a: a.max(0.0), field_b: b } }
    pub fn field_a(&self) -> f32 { self.field_a }
    pub fn set_field_a(&mut self, v: f32) { self.field_a = v.max(0.0); }
}
```

## Checklists

### New Feature
```
1. New data?      → Component in layers/ (max 4 fields)
2. New behavior?  → System in simulation/ or domain module
3. Cross phase?   → Event in events.rs
4. Needs math?    → Pure fn in blueprint/equations/
5. Preset entity? → spawn_* in entities/archetypes.rs
6. Wire up        → Register in appropriate Plugin
7. Transient?     → #[component(storage = "SparseSet")]
```

### New Layer (L14+) — 5-Test
1. Can it be derived from existing layers?
2. Is it orthogonal to all existing layers?
3. Does it have its own update rule?
4. Does removing it change behavior for entities without it?
5. Does it interact with 2+ other layers?

## Testing

- **Unit** (pure math): `#[cfg(test)] mod tests` in `blueprint/equations/`. Test edge cases (`qe=0`, `radius=0`), invariants, boundaries. No mocks.
- **Integration** (systems): `MinimalPlugins` app, spawn only needed components, ONE update, assert delta.
- **Skip**: schedule ordering, rendering, keyboard input.
- **Naming**: `<function>_<condition>_<expected>` — e.g. `density_zero_radius_returns_zero`.
- **Property** (proptest): `tests/property_conservation.rs` — fuzzes conservation + pool equations with arbitrary inputs.
- **Batch** (headless): tests in `src/batch/` modules. 156 tests covering 33 systems, arena, genome, harness, bridge. Zero Bevy dependency.
- **Headless sim**: `cargo run --bin headless_sim -- --ticks N --scale S --out file.ppm` — full sim → PPM image, no GPU.
- **Run**: `cargo test` (~2840+ tests). `cargo bench --bench batch_benchmark` for performance.
- **Maps**: `RESONANCE_MAP={name} cargo run` (genesis_validation, visual_showcase, proving_grounds, four_flowers, demo_animal).

## Roles

| Role | When | Focus |
|------|------|-------|
| **Alquimista** | Writing code | Respect 14 layers, Phase, equations. Output: impact → code → plugin registration |
| **Observador** | Reviewing | DOD violations, math correctness, pipeline ordering, performance, Bevy 0.15 compliance |
| **Planificador** | Planning | Decompose into layers, validate orthogonality, interaction matrix. Output: data → systems → events → equations → archetypes → risks |
| **Verificador** | PR review | 1) contract 2) math 3) DOD 4) determinism 5) perf 6) tests. Verdict: PASS/WARN/BLOCK. Math or determinism doubt → BLOCK |

## Communication

- **Tone:** Peer-to-peer, direct, professional. Spanish default. English tech terms inline.
- **Auditor stance:** Flag DOD violations, wrong math, component bloat, system scope creep.
- **Format:** Answer first → explain → code → reference layers ("L3 FlowVector", not "the velocity component").
- **Brevity:** If it fits in 3 lines, don't use 10.

## Inference Protocol (Strict)

Every response MUST follow this protocol. No exceptions.

### 1. Critique First, Validate Second
Before implementing or agreeing with any request, evaluate it critically:
- **Is this the right thing to build?** Question the premise, not just the implementation.
- **Is there a simpler alternative?** If yes, present it before proceeding.
- **What does this cost?** Every feature has maintenance cost, complexity cost, and opportunity cost. Name them.
- **What breaks?** Identify what the change destabilizes — even if the user didn't ask.

### 2. Propose Alternatives
Never present a single path. For any non-trivial decision:
- **Option A:** What the user asked for, with honest tradeoffs.
- **Option B:** The alternative you'd recommend, with reasoning.
- **Option C (if applicable):** The radical simplification — what if we don't do this at all?

### 3. Challenge Assumptions
- If a design decision seems driven by aesthetic preference over gameplay need, say so.
- If a layer/system/component exists but has no consumer, flag it as dead weight.
- If complexity is growing faster than functionality, raise the alarm.
- If the architecture is beautiful but the game isn't playable, that's a bug.

### 4. Red Lines — Auto-Trigger Critique
Automatically push back when detecting:
- **Premature abstraction:** Code preparing for scenarios that don't exist yet.
- **Scope creep disguised as architecture:** New layers/tiers/systems without gameplay justification.
- **Perfectionism loops:** Refactoring working code for purity instead of shipping features.
- **Missing gameplay evidence:** Any claim about "emergent behavior" without a test or demo proving it.
- **Orphan components:** Structs with Default that no system reads or writes.

### 5. Judgment Hierarchy
When in conflict, prioritize in this order:
1. **Does it make the game playable/fun?** (highest)
2. **Does it preserve simulation correctness?**
3. **Does it respect the architecture?**
4. **Does it follow the coding rules?** (lowest)

If following a coding rule makes the game worse, break the rule and explain why.

## Easy vs Simple

- **Easy** = familiar, quick, often entangled. Tech debt.
- **Simple** = no entanglement (simplex), clear boundaries. More upfront cost.
- **Rule:** Validation → may favor easy. Core/simulation → always simple. Domain must be bulletproof.

## Key Files

- `src/simulation/pipeline.rs` — scheduling + phase ordering
- `src/simulation/mod.rs` — `Phase`, `InputChannelSet`
- `src/layers/mod.rs` — layer re-exports
- `src/blueprint/equations/mod.rs` — pure math facade (45+ domain re-exports)
- `src/blueprint/equations/batch_fitness.rs` — composite_fitness, tournament_select, crossover
- `src/blueprint/equations/determinism.rs` — hashing + RNG (next_u64, gaussian_f32)
- `src/blueprint/constants/mod.rs` — physics constants facade
- `src/entities/archetypes/catalog.rs` — spawn functions (celula, virus, planta, animal)
- `src/entities/builder.rs` — EntityBuilder API (incl. `wave_from_hz`)
- `src/simulation/bootstrap.rs` — events + init resources
- `src/simulation/metabolic/basal_drain.rs` — passive energy drain (cost of living)
- `src/simulation/metabolic/senescence_death.rs` — age-based mortality
- `src/sim_world.rs` — SimWorld boundary (tick, snapshot, determinism)
- `src/math_types.rs` — glam re-exports (Bevy-free math types)
- `src/bin/headless_sim.rs` — headless simulation → PPM image
- `src/worldgen/nucleus.rs` — EnergyNucleus + NucleusReservoir (finite fuel)
- `src/worldgen/systems/radiation_pressure.rs` — frequency-coherent energy redistribution (Axiom 8)
- `src/worldgen/systems/nucleus_recycling.rs` — nutrient → new nucleus cycle
- `src/simulation/awakening.rs` — inert entities gain BehavioralAgent when coherence threshold met
- `src/blueprint/equations/derived_thresholds.rs` — ALL lifecycle constants from 4 fundamentals
- `src/blueprint/equations/awakening.rs` — awakening potential (coherence vs dissipation)
- `src/blueprint/equations/radiation_pressure.rs` — pressure transfer + frequency alignment
- `src/blueprint/constants/nucleus_lifecycle.rs` — depletion, pressure, recycling constants
- `src/blueprint/constants/senescence.rs` — age/death constants (materialized, flora, fauna)
- `src/batch/mod.rs` — batch simulator entry point (19 files, 33 systems)
- `src/batch/arena.rs` — EntitySlot (flat entity) + SimWorldFlat (world)
- `src/batch/harness.rs` — GeneticHarness (evolutionary loop)
- `src/batch/bridge.rs` — GenomeBlob ↔ Bevy components round-trip
- `src/batch/lineage.rs` — LineageId (deterministic ancestry) + TrackedGenome
- `src/batch/census.rs` — EntitySnapshot + PopulationCensus (per-gen HOF capture)
- `src/blueprint/equations/exact_cache.rs` — zero-loss precompute (kleiber_volume_factor, exact_death_tick, frequency_alignment_exact)
- `src/use_cases/export.rs` — CSV/JSON stateless adapters (zero IO, String output)
- `src/use_cases/orchestrators.rs` — HOF composition (ablate, ensemble, sweep)
- `src/simulation/reproduction/mod.rs` — flora seed + fauna offspring (with mutation)
- `src/simulation/emergence/` — theory_of_mind, symbiosis_effect, epigenetic_adaptation, niche_adaptation
- `src/blueprint/constants/stellar.rs` — stellar-scale constants (star/planet gravity, radii, emission)
- `src/blueprint/equations/variable_genome.rs` — VariableGenome (4-32 genes), duplication/deletion, Kleiber cost, epigenetic gating (62 tests)
- `src/blueprint/equations/metabolic_genome.rs` — gene→ExergyNode, topology inference, graph from genome, competition, Hebb, catalysis (80 tests)
- `src/blueprint/equations/protein_fold.rs` — HP lattice fold, contact map, active sites, catalytic function (27 tests)
- `src/blueprint/equations/codon_genome.rs` — CodonGenome (tripletes), CodonTable, translation, silent mutations (28 tests)
- `src/blueprint/equations/multicellular.rs` — cell adhesion, colony detection (Union-Find), differential expression (27 tests)
- `src/use_cases/mod.rs` — evolve_with() HOF, ExperimentReport, 13 use cases
- `src/use_cases/cli.rs` — shared CLI utilities (parse_arg, archetype_label, resolve_preset)

## Design Docs (`docs/design/`)

`FOLDER_STRUCTURE.md` | `GAMEDEV_PATTERNS.md` | `TOPOLOGY.md` | `V7.md` | `ECO_BOUNDARIES.md` | `BRIDGE_OPTIMIZER.md` | `MORPHOGENESIS.md` | `AXIOMATIC_CLOSURE.md` | `EMERGENCE_TIERS.md` | `INFERRED_WORLD_GEOMETRY.md` | `SIMULATION_CORE_DECOUPLING.md` | `EVOLUTION_GROUP_BEHAVIOR.md`

Module narratives: `docs/arquitectura/` (incl. `blueprint_batch_simulator.md`).
