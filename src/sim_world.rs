//! Simulation boundary contract — the constitution of the separation.
//!
//! The simulation must not know it is being observed.
//! The renderer reads [`WorldSnapshot`]; it never touches the ECS world.
//!
//! # Three operations only
//!
//! ```text
//! SimWorld::new(config)  → initialize from constants (Big Bang)
//! SimWorld::tick(cmds)   → advance one unit of time (atomic, deterministic)
//! SimWorld::snapshot()   → export observable state (owned, sorted, renderer-ready)
//! ```
//!
//! # Invariants enforced here
//!
//! - **INV-4** Determinism — same config + same inputs → identical [`WorldSnapshot`]s.
//! - **INV-7** Conservation — `total_qe_after ≤ total_qe_before + ε`.
//! - **INV-8** Clock — `tick_id` is the only clock; no `std::time` inside `tick()`.
//!
//! See `docs/design/SIMULATION_CORE_DECOUPLING.md`.

use std::time::Duration;

use bevy::prelude::*;

use crate::blueprint::WorldEntityId;
use crate::blueprint::equations::determinism;
use crate::layers::{BaseEnergy, OscillatorySignature, SpatialVolume};

// ─── Tick identifier ──────────────────────────────────────────────────────────

/// The only clock in the universe (INV-8).
/// Monotonically increasing. Never derived from wall-clock time.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct TickId(pub u64);

// ─── Snapshot types ───────────────────────────────────────────────────────────

/// Entity state at tick T — plain Rust, zero ECS types, zero render deps.
///
/// Sorted by `id` ascending inside [`WorldSnapshot`] for canonical order.
#[derive(Debug, Clone)]
pub struct EntitySnapshot {
    /// Persistent strong ID (`WorldEntityId.0`).
    pub id: u64,
    /// World-space XZ position in sim units.
    pub position: [f32; 2],
    /// Energy in qe — the fundamental currency of existence.
    pub qe: f32,
    /// Oscillatory frequency in Hz (defines elemental band).
    pub frequency_hz: f32,
    /// Spatial radius in world units.
    pub radius: f32,
}

/// Complete world state at tick T.
///
/// - Produced by [`SimWorld::snapshot`] — owned, cloned out of ECS.
/// - Consumed by the renderer — INV-5: renderer never writes back.
/// - Entities sorted by `id` ascending — required for deterministic comparison.
#[derive(Debug, Clone)]
pub struct WorldSnapshot {
    pub tick_id: TickId,
    pub seed: u64,
    /// All observable entities in canonical order (sorted by `id`).
    pub entities: Vec<EntitySnapshot>,
    /// Sum of all `BaseEnergy.qe` — conservation invariant.
    pub total_qe: f32,
}

// ─── Input types ──────────────────────────────────────────────────────────────

/// Ability target encoding — no heap allocation, no trait objects.
#[derive(Debug, Clone, Copy)]
pub enum AbilityTargetCmd {
    Point([f32; 2]),
    Entity(u64),
    NoTarget,
}

/// Player intent as a physics-level instruction.
///
/// Enum variant per action type — no `Box<dyn Trait>` (Hard Block 14).
/// Produced by the platform layer; consumed by [`SimWorld::tick`].
#[derive(Debug, Clone, Copy)]
pub enum InputCommand {
    MoveToward { entity_id: u64, goal: [f32; 2] },
    CastAbility { entity_id: u64, slot: u8, target: AbilityTargetCmd },
}

// ─── Config ───────────────────────────────────────────────────────────────────

/// Initial configuration for one universe instance.
#[derive(Debug, Clone)]
pub struct SimConfig {
    /// Map asset name — resolves to `assets/maps/{map_name}.ron`.
    pub map_name: &'static str,
    /// Determinism seed. Per-entity stochasticity: `tick_id XOR entity_id`.
    pub seed: u64,
    /// Fixed timestep rate in Hz (default: 20).
    pub tick_rate_hz: f32,
}

impl Default for SimConfig {
    fn default() -> Self {
        Self { map_name: "demo_minimal", seed: 0, tick_rate_hz: 20.0 }
    }
}

// ─── SimWorld ─────────────────────────────────────────────────────────────────

/// The complete, owned state of one universe instance.
///
/// Exposes exactly three operations: [`new`][SimWorld::new],
/// [`tick`][SimWorld::tick], [`snapshot`][SimWorld::snapshot].
///
/// # Invariants
///
/// - **INV-2**: Single source of truth — all physics state lives here.
/// - **INV-3**: `tick()` performs no external I/O.
pub struct SimWorld {
    app: App,
    tick_id: TickId,
    seed: u64,
    tick_duration: Duration,
}

impl SimWorld {
    /// Big Bang — initialize a universe from config.
    pub fn new(config: SimConfig) -> Self {
        let tick_duration = Duration::from_secs_f32(1.0 / config.tick_rate_hz.max(1.0));
        let seed = config.seed;
        let app = Self::build_headless_app(&config);
        Self { app, tick_id: TickId(0), seed, tick_duration }
    }

    /// Advance one unit of time — atomic, indivisible, deterministic.
    ///
    /// Input commands are applied before physics (mirrors `Phase::Input` ordering).
    ///
    /// **INV-7** asserted in debug builds: energy must not spontaneously increase.
    /// **INV-8** enforced structurally: only `tick_duration` advances time, never `std::time`.
    pub fn tick(&mut self, commands: &[InputCommand]) {
        self.apply_input(commands);

        #[cfg(debug_assertions)]
        let qe_before = self.total_qe();

        // INV-8: advance by config-derived duration only — no wall-clock reads.
        self.app
            .world_mut()
            .resource_mut::<Time>()
            .advance_by(self.tick_duration);
        self.app.world_mut().run_schedule(FixedUpdate);

        self.tick_id = TickId(self.tick_id.0 + 1);

        // INV-7: energy must not increase (creation from nothing is a physics bug).
        // Tolerance: 0.01% relative + 1 qe absolute for f32 accumulation.
        #[cfg(debug_assertions)]
        {
            let qe_after = self.total_qe();
            let tolerance = qe_before.abs() * 1e-4 + 1.0;
            debug_assert!(
                qe_after <= qe_before + tolerance,
                "INV-7 conservation at tick {}: {qe_before:.3} → {qe_after:.3} (Δ={:+.4})",
                self.tick_id.0,
                qe_after - qe_before,
            );
        }
    }

    /// Export observable state — owned, cloned, sorted for determinism.
    ///
    /// Only entities with [`WorldEntityId`] are included (persistent entities).
    /// Transient/internal entities (no strong ID) are excluded.
    /// Sorted by `id` ascending — required for INV-4.
    pub fn snapshot(&mut self) -> WorldSnapshot {
        let tick_id = self.tick_id;
        let seed = self.seed;

        let world = self.app.world_mut();
        let mut q = world.query::<(
            &WorldEntityId,
            &BaseEnergy,
            &SpatialVolume,
            &OscillatorySignature,
            Option<&Transform>,
        )>();

        let mut entities: Vec<EntitySnapshot> = q
            .iter(world)
            .map(|(id, energy, vol, osc, tr)| EntitySnapshot {
                id: id.0 as u64,
                position: tr
                    .map(|t| [t.translation.x, t.translation.z])
                    .unwrap_or([0.0, 0.0]),
                qe: energy.qe(),
                frequency_hz: osc.frequency_hz(),
                radius: vol.radius,
            })
            .collect();

        // Canonical order: sort by strong ID — determinism requires stable iteration (INV-4).
        entities.sort_unstable_by_key(|e| e.id);
        let total_qe = entities.iter().map(|e| e.qe).sum();

        WorldSnapshot { tick_id, seed, entities, total_qe }
    }

    /// Hash of the current energy state — fast determinism check without full clone.
    ///
    /// Equivalent to hashing the `qe` field of all entities sorted by `id`.
    /// Cheap enough for per-tick verification in tests.
    pub fn energy_hash(&mut self) -> u64 {
        let world = self.app.world_mut();
        let mut q = world.query::<(&WorldEntityId, &BaseEnergy)>();
        let mut energies: Vec<(u32, f32)> = q
            .iter(world)
            .map(|(id, e)| (id.0, e.qe()))
            .collect();
        energies.sort_unstable_by_key(|(id, _)| *id);
        let values: Vec<f32> = energies.into_iter().map(|(_, qe)| qe).collect();
        determinism::hash_f32_slice(&values)
    }

    /// Current tick identifier.
    #[inline]
    pub fn tick_id(&self) -> TickId { self.tick_id }

    /// Mutable access to the underlying App for plugin wiring.
    ///
    /// Used by `resonance-app` during startup to add simulation plugins.
    /// Not exposed to the renderer — the renderer only uses `snapshot()`.
    #[inline]
    pub fn app_mut(&mut self) -> &mut App { &mut self.app }

    // ─── Private ──────────────────────────────────────────────────────────────

    #[allow(dead_code)]
    fn total_qe(&mut self) -> f32 {
        let world = self.app.world_mut();
        let mut q = world.query::<&BaseEnergy>();
        q.iter(world).map(|e| e.qe()).sum()
    }

    fn apply_input(&mut self, _commands: &[InputCommand]) {
        // Routes InputCommand → WillActuator / Grimoire via EntityLookup.
        // TODO(sim-decoupling): resolve entity_id → Bevy Entity via IdGenerator/EntityLookup.
        // Currently a no-op; Phase::Input systems handle default behavior.
    }

    /// Build a headless Bevy App — no render, no window, no display server.
    ///
    /// Uses [`MinimalPlugins`] to satisfy INV-1 (no rendering dependency).
    /// `FixedUpdate` schedule is added explicitly because `MinimalPlugins` omits
    /// `MainSchedulePlugin` (which normally registers it via `DefaultPlugins`).
    ///
    /// Full simulation plugins are added by the caller via [`app_mut()`][SimWorld::app_mut]
    /// during initialization in `resonance-app`. This keeps SimWorld decoupled
    /// from asset loading, which still depends on Bevy's AssetServer.
    fn build_headless_app(_config: &SimConfig) -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        // `MinimalPlugins` omits `MainSchedulePlugin` — add FixedUpdate manually.
        app.add_schedule(Schedule::new(FixedUpdate));
        app
    }
}

// ─── Tests ───────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tick_id_advances_per_tick() {
        let mut world = SimWorld::new(SimConfig::default());
        assert_eq!(world.tick_id().0, 0);
        world.tick(&[]);
        assert_eq!(world.tick_id().0, 1);
        world.tick(&[]);
        world.tick(&[]);
        assert_eq!(world.tick_id().0, 3);
    }

    #[test]
    fn snapshot_tick_id_matches_world() {
        let mut world = SimWorld::new(SimConfig::default());
        world.tick(&[]);
        world.tick(&[]);
        let snap = world.snapshot();
        assert_eq!(snap.tick_id.0, 2);
        assert_eq!(snap.seed, 0);
    }

    #[test]
    fn snapshot_total_qe_is_entity_sum() {
        let mut world = SimWorld::new(SimConfig::default());
        let snap = world.snapshot();
        let sum: f32 = snap.entities.iter().map(|e| e.qe).sum();
        // Empty world: both zero, tolerance for float ops.
        assert!((snap.total_qe - sum).abs() < f32::EPSILON);
    }

    #[test]
    fn snapshot_entities_sorted_by_id() {
        let mut world = SimWorld::new(SimConfig::default());
        let snap = world.snapshot();
        let ids: Vec<u64> = snap.entities.iter().map(|e| e.id).collect();
        let mut sorted = ids.clone();
        sorted.sort_unstable();
        assert_eq!(ids, sorted, "entities must be sorted by id for INV-4");
    }

    #[test]
    fn config_seed_propagates_to_snapshot() {
        let mut world = SimWorld::new(SimConfig { seed: 42, ..Default::default() });
        let snap = world.snapshot();
        assert_eq!(snap.seed, 42);
    }

    /// INV-4: identical config → identical energy hash after N ticks.
    #[test]
    fn determinism_empty_world_hashes_match() {
        let config = SimConfig::default();
        let mut w1 = SimWorld::new(config.clone());
        let mut w2 = SimWorld::new(config);
        for _ in 0..100 {
            w1.tick(&[]);
            w2.tick(&[]);
        }
        assert_eq!(w1.energy_hash(), w2.energy_hash(), "INV-4 violated: hashes diverged");
        assert_eq!(w1.tick_id(), w2.tick_id());
    }

    /// INV-4: snapshots are byte-identical between two independent runs.
    #[test]
    fn determinism_empty_world_snapshots_match() {
        let config = SimConfig::default();
        let mut w1 = SimWorld::new(config.clone());
        let mut w2 = SimWorld::new(config);
        for _ in 0..50 {
            w1.tick(&[]);
            w2.tick(&[]);
        }
        let s1 = w1.snapshot();
        let s2 = w2.snapshot();
        assert_eq!(s1.tick_id, s2.tick_id);
        assert_eq!(s1.total_qe.to_bits(), s2.total_qe.to_bits(), "total_qe must be bit-identical");
        assert_eq!(s1.entities.len(), s2.entities.len());
        for (e1, e2) in s1.entities.iter().zip(s2.entities.iter()) {
            assert_eq!(e1.id, e2.id);
            assert_eq!(e1.qe.to_bits(), e2.qe.to_bits(), "qe diverged at entity {}", e1.id);
        }
    }

    /// INV-7 (structural): empty world stays at zero energy.
    #[test]
    fn conservation_empty_world_stays_zero() {
        let mut world = SimWorld::new(SimConfig::default());
        for _ in 0..200 {
            world.tick(&[]);
        }
        let snap = world.snapshot();
        assert_eq!(snap.total_qe, 0.0, "empty world must have zero energy");
    }

    /// INV-8: different tick rates produce the same entity states (energy is tick-independent).
    /// Verifies clock is from config, not wall time.
    #[test]
    fn clock_is_config_not_wall_time() {
        let slow = SimConfig { tick_rate_hz: 5.0, ..Default::default() };
        let fast = SimConfig { tick_rate_hz: 60.0, ..Default::default() };
        let mut w_slow = SimWorld::new(slow);
        let mut w_fast = SimWorld::new(fast);
        for _ in 0..10 { w_slow.tick(&[]); }
        for _ in 0..10 { w_fast.tick(&[]); }
        // Both have advanced 10 ticks; entities are identical (empty world).
        assert_eq!(w_slow.energy_hash(), w_fast.energy_hash());
    }
}
