---
name: simulation_core_decoupling
description: Blueprint for separating the simulation core (resonance-sim) from the renderer (resonance-app), implemented as SimWorld contract in src/sim_world.rs
type: project
---

SimWorld boundary contract implemented in `src/sim_world.rs` (2026-03-25).

The three-operation interface: `new(SimConfig)`, `tick(&mut self, &[InputCommand])`, `snapshot(&mut self) -> WorldSnapshot`.

Key types: `TickId`, `EntitySnapshot`, `WorldSnapshot`, `InputCommand`, `AbilityTargetCmd`, `SimConfig`.

**Why:** Blueprint document defines the universe/observer separation. Simulation must not know it is being observed. Renderer reads WorldSnapshot only.

**How to apply:** The 8 invariants (INV-1..8) in `src/sim_world.rs` doc comment are the architectural constitution. Future work: extract as `resonance-sim` workspace crate using bevy without render features.

Design doc: `docs/design/SIMULATION_CORE_DECOUPLING.md`.

Technical note: `MinimalPlugins` does not register `FixedUpdate` schedule (that's `MainSchedulePlugin` inside `DefaultPlugins`). The headless app adds it manually via `app.add_schedule(Schedule::new(FixedUpdate))`.
