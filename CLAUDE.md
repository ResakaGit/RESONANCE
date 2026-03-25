# Resonance — Claude Code Instructions

## Project

Resonance is an alchemical MOBA in Rust/Bevy 0.15 where everything is energy (qe). 14 orthogonal ECS layers define all entities by composition. Gameplay is 100% emergent from energy interactions; interfaces are standard MOBA (Dota/LoL style).

## Architecture

- **14 layers** (L0 BaseEnergy → L13 StructuralLink). See `layers/` (+ auxiliares: nutrient, irradiance, inference, vision_fog, growth, …).
- **Pipeline:** `FixedUpdate` with fixed timestep (`Time<Fixed>` / `SimulationTickPlugin`). Phases are **`SystemSet`s encadenados** en `simulation/pipeline.rs`:
  `SimulationClockSet` → `Phase::Input` → `Phase::ThermodynamicLayer` → `Phase::AtomicLayer` → `Phase::ChemicalLayer` → `Phase::MetabolicLayer` → `Phase::MorphologicalLayer`.
  Los nombres legacy **PrePhysics / Physics / Reactions / PostPhysics** ya no existen en el enum (`simulation/mod.rs`).
- **Pure math** lives in `blueprint/equations/` (`mod.rs` + dominios). NEVER inline formulas in systems.
- **Constants** live in `{module}/constants.rs` or `{module}/constants/mod.rs` (+ domain shards) where the module has one — blueprint uses `src/blueprint/constants/`, plus bridge, eco, topology, worldgen, etc.
- **Stateless-first:** Pure functions + Resources. Components hold state, systems transform it.
- **Pattern: Layered ECS with Vertical Slices.** NOT hexagonal architecture. In ECS simulations: components ARE the domain, systems ARE the use cases, Bevy IS the infrastructure. No ports/adapters.

## Stack (Hard Constraints)

| Layer | Technology | Version | Notes |
|-------|-----------|---------|-------|
| Language | Rust | stable 2024 edition | MSRV 1.85 |
| Engine | Bevy | 0.15.x | ECS + rendering + input |
| Math | `bevy::math` (glam) | bundled | Vec2, Vec3, f32 ops |
| Async | None | — | Bevy schedule only, no tokio/async-std |

## Module Map (`src/lib.rs` — 14 `pub mod` top-level)

```
blueprint/          → Types, equations/, constants/, almanac/, abilities, recipes, ids, validator, morphogenesis
bridge/             → Cache optimizer (BridgeCache<B>, 11 equation kinds) + constants.rs
eco/                → Eco-boundaries, zones, climate + systems.rs
entities/           → EntityBuilder, archetypes (spawn_*), composition, lifecycle_observers
events.rs           → Event contracts (cast, catálisis, path, death, worldgen, …); ver bootstrap.rs
geometry_flow/      → GF1 flora-tubo (branching stateless)
layers/             → 14 capas ECS + auxiliares (24 archivos en layers/)
plugins/            → SimulationPlugin, LayersPlugin, DebugPlugin
rendering/          → quantized_color (+ QuantizedColorPlugin en main.rs)
runtime_platform/   → 17 sub-módulos (`mod.rs`): compat_2d3d, tick, input, camera, HUD, fog_overlay, …
simulation/         → pipeline, bootstrap, input→…→post, pathfinding, fog, crecimiento, fotosíntesis, …
topology/           → Terrain: noise, slope, drainage, classifier, hydraulics, mutations, config
world/              → SpatialIndex, demos (demo_level, nubes, fog grid, presets); mapas = assets/maps/*.ron
worldgen/           → V7: field_grid, nucleus, propagation, materialization, shape_inference, nutrient_field
  systems/          → startup, prephysics, propagation, materialization, terrain, visual, performance
```

**Mapas:** `RESONANCE_MAP` → `assets/maps/{nombre}.ron` (`worldgen/map_config.rs`).

**Eventos registrados:** `simulation/bootstrap.rs` — 15 tipos `Event` (incl. `TerrainMutationEvent`); `PathRequestEvent` en `Compat2d3dPlugin` (perfil full3d).

**Docs alineados al código:** `docs/arquitectura/README.md` (blueprints por módulo). Estructura de carpetas detallada: `docs/design/FOLDER_STRUCTURE.md`.

## The 14 Orthogonal Layers

```
Layer  0: BaseEnergy           (existence — qe)
Layer  1: SpatialVolume        (spatial volume — radius)
Layer  2: OscillatorySignature (wave signature — frequency, phase)
Layer  3: FlowVector           (flow — velocity, dissipation)
Layer  4: MatterCoherence      (structural integrity — state, bond energy)
Layer  5: AlchemicalEngine     (mana processor — buffer, valves)
Layer  6: AmbientPressure      (terrain — delta_qe, viscosity)
Layer  7: WillActuator         (will — intent, channeling)
Layer  8: AlchemicalInjector   (spell payload — projected qe, forced freq)
Layer  9: MobaIdentity         (game rules — faction, tags, crit)
Layer 10: ResonanceLink        (buff/debuff — effect → target)
Layer 11: TensionField         (gravity/magnetic force at distance)
Layer 12: Homeostasis          (frequency adaptation with qe cost)
Layer 13: StructuralLink       (spring joint between entities)
```

## Coding Rules

1. **English identifiers only.** A linter translates Spanish→English.
2. **Max 4 fields per component.** More fields = split into layers.
3. **One system, one transformation.** No god-systems (>5 component types).
4. **Use `SparseSet`** for transient components (buffs, markers, one-shot flags).
5. **Guard change detection.** Check equality before mutation: `if val != new { val = new; }`
6. **Chain events.** Producer `.before()` or `.chain()` with consumer. Never unordered.
7. **Phase assignment required.** Every gameplay system → `.in_set(Phase::X)` (u otro `SystemSet` explícito).
8. **Math in blueprint/equations.** Systems call pure functions, don't inline formulas.
9. **EntityBuilder** for spawning. Fluent API supporting all 14 layers.
10. **Constants in constants.** Tuning values centralized per module (`constants.rs` or `constants/` + `mod.rs`). Algorithmic arrays stay in-file.

## Hard Blocks (Zero Tolerance)

1. **NO `unsafe` blocks** — zero tolerance. If you need unsafe, the design is wrong.
2. **NO external crates** without explicit approval — only `bevy = "0.15"` + what's in Cargo.toml.
3. **NO `async`/`await`** — Bevy's schedule handles concurrency.
4. **NO `Arc<Mutex<T>>`** — use Bevy `Resource` or `Local` instead.
5. **NO `HashMap` in hot paths** — prefer sorted `Vec` or Bevy's `Entity` indexing.
6. **NO `String` in components** — use enums, `u32` IDs, or `&'static str`.
7. **NO `Box<dyn Trait>`** in components — use enums for closed sets.
8. **NO `#[derive(Bundle)]`** — removed in Bevy 0.15, use tuples or `#[require(...)]`.
9. **NO `ResMut` when `Res` suffices** — minimize write locks.
10. **NO `unwrap()`/`expect()`/`panic!()` in systems** — use `let-else` or `if-let`. Tests are fine.
11. **NO inline formulas** in systems — all math in `blueprint/equations/`.
12. **NO storing derived values** as components — density, temperature computed at point of use.
13. **NO trait objects for game logic** — use components + systems.
14. **NO component methods with side effects** — pure `&self` methods only, systems do the work.
15. **NO `Entity` as persistent/network ID** — create strong newtype IDs.
16. **NO systems in `Update` for gameplay logic** — use `FixedUpdate` + `Phase` (salvo derivación visual deliberada en `Update`).
17. **NO shared mutable state outside Resources** — no `static mut`, no `lazy_static! { Mutex }`.

## Bevy 0.15 Patterns

```rust
// Spawning: tuple of components (no Bundle derive)
commands.spawn((ComponentA::new(), ComponentB::new()));

// System registration
app.add_systems(FixedUpdate, my_system.in_set(Phase::MetabolicLayer));

// Queries
for (entity, mut energy, volume) in &mut query { ... }

// Camera
commands.spawn(Camera2d);

// Component dependencies
#[require(BaseEnergy)]

// Transient components
#[derive(Component)]
#[component(storage = "SparseSet")]
struct Stunned { remaining_ticks: u32 }

// Change detection guard
if target.field != new_val { target.field = new_val; }

// Observers for entity lifecycle
OnAdd, OnRemove

// State cleanup
StateScoped(GameState::Playing)
```

## Rust 2024 Edition Idioms

```rust
// Let chains (edition 2024)
if let Some(energy) = energy_query.get(entity)
    && let Some(volume) = volume_query.get(entity)
    && energy.qe > 0.0
{ ... }

// Let-else for early returns
let Some(target) = query.get(entity) else { return; };

// Exhaustive matching (NO _ wildcard on enums — compiler catches new variants)
match state {
    MatterState::Solid => { ... }
    MatterState::Liquid => { ... }
    MatterState::Gas => { ... }
    MatterState::Plasma => { ... }
}

// Iterator chains over index loops
let total_qe: f32 = query.iter()
    .filter(|(_, e)| !e.is_dead())
    .map(|(_, e)| e.qe)
    .sum();

// Stack over heap
let values: [f32; 3] = [a, b, c];  // NOT Vec

// f32 for game math (not f64)
// const fn where possible
```

## Code Aesthetic (Yanagi)

1. **Dense but readable** — no wasted lines. One empty line between logical blocks, never two.
2. **Vertical alignment** — align similar patterns in columns when it aids scanning.
3. **Let types speak** — if the type makes the intent clear, skip the comment.
4. **Functional over imperative** — prefer iterator chains, `map`, `filter`. Avoid index-based loops.
5. **Early return** — guard clauses at the top, happy path unindented.
6. **Doc comments (`///`)** on public functions — one line max. Include equation for math fns.
7. **Inline comments** only for non-obvious math or invariants. No comments on obvious code.
8. **Imports grouped:** bevy, crate, super.

## System Design Template

```rust
use bevy::prelude::*;
use crate::layers::*;
use crate::blueprint::{constants, equations};

/// [What this system transforms and why].
pub fn my_system(
    mut query: Query<(&mut TargetComponent, &SourceComponent), Without<Dead>>,
    config: Res<MyConfig>,
) {
    for (mut target, source) in &mut query {
        let result = equations::some_calculation(source.field, config.param);
        if target.field != result {
            target.field = result;
        }
    }
}
```

## Component Design Template

```rust
/// Capa [N]: [Nombre] — [one-line purpose].
#[derive(Component, Reflect, Debug, Clone)]
#[reflect(Component)]
pub struct MyComponent {
    field_a: f32,  // max 4 fields
    field_b: f32,
}

impl MyComponent {
    pub fn new(field_a: f32, field_b: f32) -> Self {
        Self { field_a: field_a.max(0.0), field_b }
    }
    pub fn field_a(&self) -> f32 { self.field_a }
    pub fn set_field_a(&mut self, val: f32) { self.field_a = val.max(0.0); }
}
```

## Adding a New Feature (Checklist)

```
1. Does it need new data?           → New component in layers/ (max 4 fields)
2. Does it need new behavior?       → New system in simulation/ or domain module
3. Does it cross phase boundaries?  → New event in events.rs
4. Does it need math?               → Pure function in blueprint/equations/
5. Does it need a preset entity?    → spawn_* function in entities/archetypes.rs
6. Wire it up                       → Register in the appropriate Plugin
7. Is it transient?                 → Use #[component(storage = "SparseSet")]
```

## Adding a New Layer (L14+)

Before adding, apply the **5-Test** from DESIGNING.md:
1. Can it be derived from existing layers?
2. Is it orthogonal to all existing layers?
3. Does it have its own update rule?
4. Does removing it change behavior for entities that don't have it?
5. Does it interact with 2+ other layers?

## Testing

### Strategy
- **Unit tests** (pure math): `#[cfg(test)] mod tests` at bottom of each file in `blueprint/equations/`. Fast, deterministic, no Bevy needed. Test edge cases, invariants, boundary values.
- **Integration tests** (systems): minimal Bevy App with `MinimalPlugins`. Spawn only needed components, run ONE update, assert delta.
- **Do NOT test**: Bevy schedule ordering, rendering, keyboard input.

### Commands
- `cargo test` — ~920+ tests en `--lib` (más `tests/` y benches según configuración).
- `cargo run` — mapa por defecto (`default.ron` si aplica).
- `RESONANCE_MAP=demo_arena cargo run` — arena multi-elemental (RON).
- `RESONANCE_MAP=proving_grounds cargo run` — stress map.
- `RESONANCE_MAP=flower_demo cargo run` — flor procedural (`geometry_flow`).
- `RESONANCE_MAP=four_flowers cargo run` — cuatro núcleos Terra-band (campo V7, grid 32×32).

### Test Naming
Format: `<function>_<condition>_<expected_result>` — e.g. `density_zero_radius_returns_zero`.

## Roles (Use as Mindset When Appropriate)

### Alquimista (Implementer)
When writing production code: respect 14 layers, SystemSets, equations in blueprint. Before writing: verify which layers intervene, which Phase, if math already exists. Output: impacto (qué capas) → código → registro en plugin.

### Observador (Auditor)
When reviewing code: check DOD violations (>4 fields, god-systems, derived values stored, trait objects), math correctness (matches blueprint, uses equations.rs), pipeline ordering (correct Phase, deterministic), performance (allocations in hot path, unnecessary &mut), Bevy 0.15 compliance.

### Planificador (Architect)
When planning features: decompose into layers, validate orthogonality, locate in pipeline, draw interaction matrix. Output: datos nuevos → sistemas nuevos → eventos → ecuaciones → arquetipos → dependencias → riesgos.

### Verificador (Verifier)
When reviewing PRs/diffs: auditor stance, assume happy-path bias. Verification loop: 1) module/phase contract, 2) math correctness, 3) ECS/DOD integrity, 4) determinism, 5) performance hot path, 6) test evidence. Verdict: PASS / WARN / BLOCK. Ante duda en correctitud matemática o determinismo → BLOCK.

## Communication

- **Tone:** Peer-to-peer, direct, professional. Spanish by default. English technical terms inline.
- **Auditor stance:** Flag DOD violations, incorrect math, component bloat, system scope creep.
- **Response format:** Answer first → explain → show code → reference layers ("Capa 3 FlowVector", not "the velocity component").
- **Keep it short:** If it fits in 3 lines, don't use 10.

## Easy vs Simple

- **Easy** = familiar, quick, but often entangled. Generates tech debt.
- **Simple** = no enredos (simplex), clear boundaries. Costs more initially.
- **Pragmatism:** Context decides. Validation → may favor easy. Core/simulation → always favor simple. The domain must be bulletproof for iteration, testing, scaling.

## Key Files for Context

When working on a feature, read these first:

- `src/simulation/pipeline.rs` — system scheduling and phase ordering
- `src/simulation/mod.rs` — `Phase`, `InputChannelSet`
- `src/layers/mod.rs` — layer re-exports
- `src/blueprint/equations/mod.rs` — all pure math (facade); dominios en `equations/*/`
- `src/blueprint/constants/mod.rs` — physics constants (facade); dominios en `constants/*.rs`
- `src/entities/archetypes.rs` — spawn functions
- `src/entities/builder.rs` — EntityBuilder API
- `src/simulation/bootstrap.rs` — events + init resources

## Design Docs

Detailed design docs in `docs/design/`:

- `FOLDER_STRUCTURE.md` — folder organization + migration status
- `GAMEDEV_PATTERNS.md` — Bevy patterns, MOBA patterns, anti-patterns
- `TOPOLOGY.md` — terrain generation
- `V7.md` — worldgen energy field
- `ECO_BOUNDARIES.md` — zone classification
- `BRIDGE_OPTIMIZER.md` — cache optimizer
- `MORPHOGENESIS.md` — inferred morphogenesis + Matrioska functional composition

Module-level narrative synced with code: `docs/arquitectura/`.
