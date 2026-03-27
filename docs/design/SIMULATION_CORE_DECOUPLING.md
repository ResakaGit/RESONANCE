# Simulation Core Decoupling
**Resonance Engine — Architectural Separation Reference**

---

## 0. The Single Governing Principle

> **The simulation must not know it is being observed.**

Every line of physics code that imports a rendering concept is a violation of this principle.
Every system that requires a window to run is a liability.
The simulation is a universe. A universe does not need a camera to exist.

This document describes the extraction of that universe from its current host.

---

## 1. What Is Being Separated

The codebase contains two fundamentally different programs occupying the same process:

```
Program A — The Universe
    Deterministic tick loop
    Energy dynamics, wave propagation, competition
    Chemical catalysis, metabolic graphs, morphology
    Spatial indexing, field propagation, abiogenesis
    Has no opinion about pixels, windows, or frames
    Could run on a server, in a test, in a parallel thread
    Is the valuable thing

Program B — The Window
    Mesh generation and synchronization
    Visual derivation from entity state
    Atmosphere, lighting, fog, bloom
    Input capture and translation
    Depends entirely on Program A
    Program A does not depend on it at all
    Is the face of the valuable thing
```

The goal is to make the boundary between them explicit, enforced by the compiler, and permanent.

---

## 2. The Boundary Contract

The separation is defined by a single interface — the **Snapshot**:

```
Universe produces:   WorldSnapshot   (state at tick T)
Renderer consumes:   WorldSnapshot   (renders tick T)
Renderer produces:   InputCommands   (player intent)
Universe consumes:   InputCommands   (applied at tick T+1 via tick(cmds))
```

Nothing else crosses the boundary. The renderer never writes physics state. The universe never reads visual state. This contract is enforced at the type level — if it compiles, it is correct.

---

## 3. The SimWorld Contract

**Implementation:** `src/sim_world.rs`

The universe is fully described by a single owned struct:

```rust
pub struct SimWorld {
    app: App,         // private Bevy App (MinimalPlugins, no render)
    tick_id: TickId,  // monotonic clock
    seed: u64,        // determinism seed
    tick_duration: Duration,  // derived from SimConfig.tick_rate_hz
}
```

### Public API

```rust
SimWorld::new(config: SimConfig) → SimWorld
    // Big Bang — initialize from constants.

SimWorld::tick(&mut self, commands: &[InputCommand])
    // Advance one unit of time. Commands applied before physics.
    // INV-7 asserted in debug builds. INV-8 enforced structurally.

SimWorld::snapshot(&mut self) → WorldSnapshot
    // Export observable state. Owned, sorted by id, renderer-ready.

SimWorld::energy_hash(&mut self) → u64
    // Fast determinism check — hash of all (entity_id, qe) pairs sorted by id.
    // Uses blueprint::equations::determinism::hash_f32_slice.

SimWorld::tick_id(&self) → TickId
    // Current tick. Read-only, no &mut.

SimWorld::app_mut(&mut self) → &mut App
    // Plugin wiring during startup only (resonance-app).
    // Not exposed to the renderer.
```

### Supporting Types

```rust
TickId(pub u64)
    // The only clock (INV-8). Monotonically increasing. Never wall-clock.

SimConfig { map_name: &'static str, seed: u64, tick_rate_hz: f32 }
    // Default: map "demo_minimal", seed 0, 20 Hz.

InputCommand
    // MoveToward { entity_id: u64, goal: [f32; 2] }
    // CastAbility { entity_id: u64, slot: u8, target: AbilityTargetCmd }

AbilityTargetCmd
    // Point([f32; 2]) | Entity(u64) | NoTarget

EntitySnapshot { id: u64, position: [f32; 2], qe: f32, frequency_hz: f32, radius: f32 }
    // Plain Rust — zero ECS types, zero render deps.

WorldSnapshot { tick_id: TickId, seed: u64, entities: Vec<EntitySnapshot>, total_qe: f32 }
    // Entities sorted by id ascending. total_qe = sum of all qe.
```

---

## 4. The Tick Contract

A tick is the atomic unit of causality. It is indivisible.

```
Within one tick(commands):
    1. Apply input     — route InputCommands to physics intent (currently no-op; TODO)
    2. Advance time    — Time::advance_by(tick_duration) (INV-8)
    3. Run physics     — World::run_schedule(FixedUpdate) (all Phase sets)
    4. Increment clock — tick_id += 1
    5. Assert INV-7    — debug_assert: qe_after <= qe_before + tolerance

Tolerance for INV-7:
    qe_before.abs() * 1e-4 + 1.0   (0.01% relative + 1 qe absolute for f32 noise)

After tick() returns:
    The universe is in a valid, consistent state
    All events from this tick are consumed and gone
    No partial state exists
    The same inputs on the same initial state produce the same result
    Always. Without exception.
```

The last point — determinism — is not a feature. It is the foundation of everything else:
replay, netcode, parallel simulation, automated balance search, and debugging all depend on it absolutely.

---

## 5. Determinism as a First-Class Invariant

Determinism means: given identical initial state and identical input sequence, two instances of SimWorld produce byte-identical snapshots at every tick.

This requires:

**No hidden state.** Every value that affects physics must live inside SimWorld.
No thread-local state, no global mutable state, no time-based randomness.

**No floating point non-determinism.** All arithmetic must produce identical results
across platforms and compilations. This means: no auto-vectorization of physics loops
without explicit control, no `f64::sin()` calls that differ by platform,
consistent operation ordering enforced by code structure not compiler assumptions.

**Ordered iteration.** Entity processing order must be deterministic.
Sorted by `WorldEntityId` before every physics pass. `HashMap` is forbidden in hot paths
— its iteration order is undefined. `BTreeMap` or sorted `Vec` only.

**Seeded randomness.** All stochastic processes (mutation drift, abiogenesis probability)
use a deterministic PRNG seeded from `tick_id XOR entity_id`.
The seed is part of SimWorld state. It is serialized. It is reproduced exactly.

Determinism is tested, not assumed (see `tests/r2_determinism.rs` and `sim_world::tests`):
```
run_sim(config, seed, 10_000 ticks) == run_sim(config, seed, 10_000 ticks)
```
This test must pass on every commit. It is the most important test in the codebase.

---

## 6. The Serialization Contract

A universe that cannot be saved does not exist for anyone but its creator.

Serialization serves four distinct purposes, each with different requirements:

**Full Snapshot** — complete world state at tick T
Used for: demo sharing, experiment reproduction, debugging, time travel
Requirement: byte-perfect reconstruction of SimWorld from snapshot alone
Frequency: on demand, not every tick

**Delta Snapshot** — changes since tick T-1
Used for: netcode state synchronization
Requirement: minimal size, fast encode/decode
Frequency: every tick in multiplayer

**Config Snapshot** — constants + initial conditions only
Used for: parallel simulation runs, balance search, experiment design
Requirement: human-readable (RON already handles this)
Frequency: once per simulation run

**Input Log** — ordered sequence of InputCommands with tick timestamps
Used for: replay (config snapshot + input log = perfect replay)
Requirement: append-only, minimal size
Key insight: a full replay requires only the initial config + input log,
not a snapshot per tick, because determinism guarantees the rest

The combination of determinism + serialization produces replay as a free consequence.
No replay system needs to be built. It emerges from the architecture.

---

## 7. The Observability Contract

Three observation channels (see `src/simulation/observability.rs`):

**Real-time metrics stream**
Selected aggregate values exported every N ticks:
total energy, entity count by scale, death/birth rates,
dominant phenotype distribution, field gradient variance.
Consumer: `SimulationHealthDashboard` resource.

**Event log**
All DeathEvents, ReproductionEvents, PhaseTransitionEvents, written to append-only log.
Consumer: post-mortem analysis, replay annotation, ML training data.

**Entity inspector**
On-demand query: given EntityId, return full component state at current tick.
Consumer: debugging, spectator mode, developer tools.

The observation channels are read-only projections of SimWorld state.
They do not affect the simulation.

---

## 8. Headless as the Default Mode

The simulation must run without a window, without a GPU, without a display server.

Headless is not a special mode. It is the default.
Rendering is the special mode — an optional observer attached to a running universe.

Current status:
- `SimWorld::new()` calls `build_headless_app()` — `MinimalPlugins` only (no render, no window)
- `FixedUpdate` schedule added manually (MinimalPlugins omits MainSchedulePlugin)
- Time advanced via `Time::advance_by(tick_duration)` + `World::run_schedule(FixedUpdate)`
- Full simulation plugins wired by `resonance-app` via `SimWorld::app_mut()`

This enables:
- **Parallel universe search** — N SimWorld instances on N threads
- **Server-side simulation** — authoritative headless, clients are renderers
- **Automated testing at scale** — 10,000 ticks in a unit test

---

## 9. The Renderer's Role After Separation

The renderer's job:
```
Read WorldSnapshot
Translate physics state to visual language:
    energy magnitude   → color temperature
    frequency          → visual palette (elemental band)
    radius             → visual scale
    structural_damage  → decay visual effects
    velocity           → motion blur, particle trails
Make the invisible universe visible to a human
```

The renderer knows nothing about the physics that produced the snapshot.
It only knows how to make a snapshot beautiful.

---

## 10. What the Separation Unlocks — In Order of Value

```
Unlocked immediately upon separation:

1. Headless unit tests          — verify physics without a window
2. Determinism verification     — run same sim twice, diff the output
3. Serialization foundation     — SimWorld can be snapshot cleanly
4. Parallel balance search      — N universes on N threads
5. CI/CD physics testing        — emergence verified on every commit

Unlocked after serialization:

6. Replay system                — config + input log = perfect replay
7. Time travel debugging        — load snapshot from tick T, inspect state
8. Demo sharing                 — serialize interesting universe moments

Unlocked after netcode:

9. Server authority             — cheat-proof multiplayer
10. Rollback netcode            — low-latency competitive play
11. Spectator mode              — renderer observes server universe
```

---

## 11. Invariants the Architecture Must Enforce

```
INV-1   resonance-sim has zero runtime dependency on any rendering library
        Enforced by: MinimalPlugins only in SimWorld::build_headless_app()

INV-2   SimWorld is the single source of truth for all physics state
        Enforced by: no global mutable state outside SimWorld

INV-3   tick() is pure with respect to external I/O
        Enforced by: no network calls, file reads, or system time inside tick()

INV-4   Identical inputs produce identical outputs
        Enforced by: determinism tests in sim_world::tests + tests/r2_determinism.rs

INV-5   The renderer never writes to SimWorld
        Enforced by: snapshot() returns owned data, not references to SimWorld

INV-6   Events live exactly one tick
        Enforced by: Bevy event buffers drained at end of tick(), not carried forward

INV-7   Conservation laws hold after every tick
        Enforced by: debug_assert!(qe_after <= qe_before + qe_before.abs() * 1e-4 + 1.0)

INV-8   tick_id is the only clock
        Enforced by: Time::advance_by(tick_duration) only — no std::time in tick()
```

---

## 12. The Migration Principle

The separation is not a rewrite. It is a reclassification.

Every system in the current codebase is already one of two things:
```
Type A:   Physics — operates on energy, entities, fields, time
          Belongs in resonance-sim
          Current location: src/simulation/, src/blueprint/, src/layers/, src/bridge/

Type B:   Visual — operates on meshes, materials, transforms, lights
          Belongs in resonance-app
          Current location: src/rendering/, worldgen/systems/visual*, Update-schedule systems
```

There is no third type. A system that seems to be both types is a system
where the boundary has been blurred — it needs to be split into its two components,
one of each type, connected only through the Snapshot contract.

The migration is complete when:
```
cargo build -p resonance-sim   compiles with zero Bevy render dependencies
cargo test  -p resonance-sim   passes with zero GPU, zero window, zero display
```

**Current status**: SimWorld contract implemented in `src/sim_world.rs`.
Next step: extract `resonance-sim` as a workspace crate with `bevy` (no render features).

---

## 13. Vocabulary Reference

| Term | Definition |
|---|---|
| **SimWorld** | The complete, owned state of one universe instance |
| **TickId** | `TickId(pub u64)` — monotonically increasing tick counter, the only clock (INV-8) |
| **WorldSnapshot** | Read-only export of SimWorld state at tick T — entities sorted by id |
| **EntitySnapshot** | Single entity: id, position, qe, frequency_hz, radius |
| **InputCommand** | Player intent: MoveToward or CastAbility — no trait objects |
| **AbilityTargetCmd** | Point, Entity, or NoTarget — enum, stack-allocated |
| **SimConfig** | Initial conditions: map_name, seed, tick_rate_hz |
| **Boundary** | The interface between simulation and renderer — Snapshot only |
| **Headless** | Running SimWorld without any rendering dependency (the default mode) |
| **Determinism** | Same inputs + same initial state = identical output, always |
| **Delta Snapshot** | Minimal state diff between ticks — foundation of netcode |
| **Input Log** | Ordered record of InputCommands — enables perfect replay |
| **Invariant** | A constraint enforced by the compiler or by tests, never by convention |
| **Type A System** | Physics logic — belongs in resonance-sim |
| **Type B System** | Visual logic — belongs in resonance-app |
