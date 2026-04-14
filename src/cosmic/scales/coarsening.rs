//! Coarsening — ticks agregados para niveles no observados.
//! Coarsening — aggregated ticks for non-observed scales.
//!
//! CT-6 / ADR-036 §D3. Los niveles no observados siguen evolucionando a tasa
//! reducida (`k^distance` ticks por cada tick del observado). No se simulan
//! fuerzas individuales; solo se proyecta la evolución agregada: dissipation
//! acumulada + edad.
//!
//! Conservation (Ax 5): `coarse_tick` nunca incrementa `total_qe`.

use bevy::prelude::*;

use crate::blueprint::equations::coarsening as math;
use crate::cosmic::scale_manager::{CosmicWorld, ScaleManager};
use crate::cosmic::ScaleLevel;

/// Base K para ratios geométricos. Potencia de 2 para alineación con TelescopeStack.
/// Base K for geometric ratios. Power-of-2 aligns with TelescopeStack.
pub const COARSENING_BASE_K: u64 = 16;

/// Máxima distancia antes de considerar una escala "frozen" (no tickea).
/// Max distance before a scale is considered frozen (no tick).
pub const MAX_DISTANCE_FROZEN: u8 = 4;

/// Proyecta N ticks de dissipation sobre un `CosmicWorld`. No simula fuerzas.
/// Projects N dissipation ticks onto a `CosmicWorld`. No force simulation.
pub fn coarse_tick(world: &mut CosmicWorld, n_ticks: u64) {
    if n_ticks == 0 { return; }
    for e in &mut world.entities {
        if !e.alive { continue; }
        let new_qe = math::accumulated_dissipation(e.qe, e.dissipation, n_ticks);
        if e.qe != new_qe { e.qe = new_qe; }
        e.age_ticks = e.age_ticks.saturating_add(n_ticks);
        if e.qe <= 0.0 { e.alive = false; }
    }
    world.tick_id = world.tick_id.saturating_add(n_ticks);
}

/// Distancia entre dos escalas. Simétrica.
/// Distance between two scales. Symmetric.
#[inline]
pub fn scale_distance(a: ScaleLevel, b: ScaleLevel) -> u8 {
    let da = a.depth() as i16;
    let db = b.depth() as i16;
    (da - db).unsigned_abs() as u8
}

/// Ratio de coarsening para `target` relativo a `observed` con la `K` dada.
/// Coarsening ratio for `target` relative to `observed` with given `K`.
#[inline]
pub fn coarsening_ratio(observed: ScaleLevel, target: ScaleLevel, k: u64) -> Option<u64> {
    math::coarsening_ratio(scale_distance(observed, target), k, MAX_DISTANCE_FROZEN)
}

// ─── Background tick counter resource ──────────────────────────────────────

/// Contador independiente para disparar coarsening a cadencia correcta.
/// Independent counter to fire coarsening at the right cadence.
#[derive(Resource, Debug, Default)]
pub struct CosmicBackgroundClock {
    pub tick_id: u64,
}

// ─── System ─────────────────────────────────────────────────────────────────

/// Sistema que ejecuta coarse_tick en niveles no observados según ratio.
/// System that runs coarse_tick on non-observed scales at the right ratio.
pub fn background_coarsening_system(
    mut clock: ResMut<CosmicBackgroundClock>,
    mut mgr: ResMut<ScaleManager>,
) {
    clock.tick_id = clock.tick_id.saturating_add(1);
    let now = clock.tick_id;
    let observed = mgr.observed;

    for instance in mgr.instances.iter_mut() {
        if instance.level == observed { continue; }
        let Some(ratio) = coarsening_ratio(observed, instance.level, COARSENING_BASE_K) else {
            continue; // frozen
        };
        if ratio == 0 || now % ratio != 0 { continue; }
        coarse_tick(&mut instance.world, ratio);
    }
}

// ─── Tests ──────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cosmic::scale_manager::{CosmicEntity, ScaleInstance};

    fn sample_world(qe: f64, dissipation: f64, n: usize) -> CosmicWorld {
        let mut w = CosmicWorld::new(n);
        for i in 0..n {
            w.spawn(CosmicEntity {
                qe,
                radius: 1.0,
                frequency_hz: 50.0,
                phase: 0.0,
                position: [i as f64, 0.0, 0.0],
                velocity: [0.0; 3],
                dissipation,
                age_ticks: 0,
                entity_id: 0,
                alive: true,
            });
        }
        w.total_qe_initial = w.total_qe();
        w
    }

    #[test]
    fn coarse_tick_qe_monotone_non_increasing() {
        let mut w = sample_world(100.0, 0.01, 5);
        let before = w.total_qe();
        coarse_tick(&mut w, 50);
        assert!(w.total_qe() < before);
        assert!(w.total_qe() > 0.0);
    }

    #[test]
    fn coarse_tick_advances_age_exactly_n() {
        let mut w = sample_world(100.0, 0.01, 3);
        coarse_tick(&mut w, 42);
        for e in &w.entities { assert_eq!(e.age_ticks, 42); }
    }

    #[test]
    fn coarse_tick_zero_is_noop() {
        let mut w = sample_world(100.0, 0.01, 3);
        let before_qe = w.total_qe();
        coarse_tick(&mut w, 0);
        assert_eq!(w.total_qe(), before_qe);
        for e in &w.entities { assert_eq!(e.age_ticks, 0); }
    }

    #[test]
    fn coarsening_ratio_geometric() {
        assert_eq!(coarsening_ratio(ScaleLevel::Ecological, ScaleLevel::Ecological, 16), Some(1));
        assert_eq!(coarsening_ratio(ScaleLevel::Ecological, ScaleLevel::Planetary, 16), Some(16));
        assert_eq!(coarsening_ratio(ScaleLevel::Ecological, ScaleLevel::Stellar, 16), Some(256));
        assert_eq!(coarsening_ratio(ScaleLevel::Ecological, ScaleLevel::Cosmological, 16), Some(4096));
    }

    #[test]
    fn coarsening_ratio_frozen_at_max_distance() {
        // Ecological ↔ Molecular: distance 1 (normal case, not frozen).
        // Cosmological ↔ Molecular: distance 4, MAX_DISTANCE_FROZEN=4 → frozen.
        assert_eq!(coarsening_ratio(ScaleLevel::Cosmological, ScaleLevel::Molecular, 16), None);
    }

    #[test]
    fn coarsened_matches_iterated_within_tolerance() {
        let mut w1 = sample_world(1000.0, 0.01, 1);
        let mut w2 = sample_world(1000.0, 0.01, 1);

        // Fine path: 100 single-tick iterations.
        for _ in 0..100 { coarse_tick(&mut w1, 1); }
        // Coarse path: 1 call with n=100.
        coarse_tick(&mut w2, 100);

        let q1 = w1.entities[0].qe;
        let q2 = w2.entities[0].qe;
        let rel = (q1 - q2).abs() / q1.max(1e-18);
        assert!(rel < 1e-9, "rel error {rel} exceeds tolerance");
    }

    #[test]
    fn entity_dies_when_qe_drops_to_zero() {
        let mut w = sample_world(1e-300, 0.1, 1);
        coarse_tick(&mut w, 10_000);
        assert!(!w.entities[0].alive);
    }

    // ─── System-level tests ─────────────────────────────────────────────────

    fn seed_two_levels() -> (App, u32) {
        let mut app = App::new();
        app.add_plugins(crate::cosmic::CosmicPlugin);
        app.init_resource::<CosmicBackgroundClock>();
        app.add_systems(Update, background_coarsening_system);

        let mut mgr = app.world_mut().resource_mut::<ScaleManager>();
        let mut eco = ScaleInstance::new(ScaleLevel::Ecological, 4, 1);
        eco.world.spawn(CosmicEntity {
            qe: 100.0, radius: 1.0, frequency_hz: 50.0, phase: 0.0,
            position: [0.0; 3], velocity: [0.0; 3], dissipation: 0.01,
            age_ticks: 0, entity_id: 0, alive: true,
        });
        eco.world.total_qe_initial = eco.world.total_qe();
        let eco_id = 0;
        mgr.insert(eco);

        let mut planetary = ScaleInstance::new(ScaleLevel::Planetary, 4, 1);
        planetary.world.spawn(CosmicEntity {
            qe: 500.0, radius: 1.0, frequency_hz: 50.0, phase: 0.0,
            position: [0.0; 3], velocity: [0.0; 3], dissipation: 0.005,
            age_ticks: 0, entity_id: 0, alive: true,
        });
        planetary.world.total_qe_initial = planetary.world.total_qe();
        mgr.insert(planetary);
        mgr.observed = ScaleLevel::Ecological;

        (app, eco_id)
    }

    #[test]
    fn system_observed_level_unaffected() {
        let (mut app, _eco_id) = seed_two_levels();
        let eco_before = app
            .world()
            .resource::<ScaleManager>()
            .get(ScaleLevel::Ecological)
            .unwrap()
            .world
            .total_qe();
        // Run many ticks to let the coarsening system fire.
        for _ in 0..32 { app.update(); }
        let eco_after = app
            .world()
            .resource::<ScaleManager>()
            .get(ScaleLevel::Ecological)
            .unwrap()
            .world
            .total_qe();
        // Observed level only touched by coarsening if distance 0 AND ratio match:
        // distance=0 → ratio=1 (fires every tick). But spec says observed is skipped.
        // Our system skips `level == observed`, so qe unchanged.
        assert_eq!(eco_before, eco_after);
    }

    #[test]
    fn system_background_level_evolves_at_slower_rate() {
        let (mut app, _) = seed_two_levels();
        let before = app
            .world()
            .resource::<ScaleManager>()
            .get(ScaleLevel::Planetary)
            .unwrap()
            .world
            .total_qe();
        // distance(Ecological, Planetary) = 1; ratio = 16. Trigger at tick 16.
        for _ in 0..16 { app.update(); }
        let after = app
            .world()
            .resource::<ScaleManager>()
            .get(ScaleLevel::Planetary)
            .unwrap()
            .world
            .total_qe();
        assert!(after < before, "planetary level didn't evolve: {before} -> {after}");
    }

    #[test]
    fn conservation_preserved_across_mixed_levels() {
        let (mut app, _) = seed_two_levels();
        let before = app.world().resource::<ScaleManager>().total_qe_across_scales();
        for _ in 0..64 { app.update(); }
        let after = app.world().resource::<ScaleManager>().total_qe_across_scales();
        assert!(after <= before + 1e-9, "universe qe grew: {before} -> {after}");
    }
}
