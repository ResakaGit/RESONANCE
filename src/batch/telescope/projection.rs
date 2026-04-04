//! Motor de proyección (TT-6).
//! Projection engine (TT-6).
//!
//! Proyecta un SimWorldFlat K ticks al futuro usando solvers analíticos existentes
//! ponderados por normalizadores. No modifica el mundo original.

use crate::batch::arena::{EntitySlot, SimWorldFlat};
use crate::batch::constants::GRID_CELLS;
use crate::blueprint::equations::temporal_telescope::{NormalizerWeights, RegimeMetrics, project_qe};
use crate::blueprint::equations::{batch_stepping, macro_analytics};

/// Proyecta una entidad individual K ticks al futuro.
/// Projects a single entity K ticks into the future.
///
/// Usa solvers analíticos existentes: exponential_decay, allometric_radius.
/// Pondera por Hurst weight via project_qe.
#[inline]
pub fn project_entity(
    entity: &EntitySlot,
    metrics: &RegimeMetrics,
    weights: &NormalizerWeights,
    k: u32,
    dt: f32,
) -> EntitySlot {
    if !entity.alive || k == 0 {
        return *entity;
    }
    let mut projected = *entity;

    // QE: frequency-aware decay (Axiom 8) + conservation-bounded projection (Axiom 4+5).
    // Step 1: Modulate dissipation by solar resonance (resonant entities decay less).
    // Step 2: Apply exponential decay (Second Law, always reduces).
    // Step 3: Hurst-weighted trend correction.
    // Step 4: Clamp to [base_decay, current_qe] — never creates energy.
    use crate::blueprint::equations::temporal_telescope::{
        conservation_bounded_project, frequency_aware_decay_rate,
    };
    let effective_rate = frequency_aware_decay_rate(
        entity.dissipation,
        entity.frequency_hz,
        crate::batch::constants::SOLAR_FREQUENCY,
        crate::batch::constants::SOLAR_BANDWIDTH,
        crate::batch::constants::PHOTOSYNTHESIS_EFFICIENCY,
    );
    let base_decay = batch_stepping::dissipation_n_ticks(entity.qe, effective_rate, k);
    let hurst_projected = project_qe(base_decay, 0.0, metrics, weights, k);
    projected.qe = conservation_bounded_project(entity.qe, base_decay, hurst_projected);

    // Radius: allometric growth.
    if entity.growth_bias > 0.0 {
        let max_r = entity.growth_bias * crate::blueprint::constants::temporal_telescope::PROJECTION_MAX_ALLOMETRIC_RADIUS;
        projected.radius = macro_analytics::allometric_radius(
            entity.radius,
            max_r,
            entity.growth_bias * crate::blueprint::constants::temporal_telescope::PROJECTION_ALLOMETRIC_SCALE,
            k,
        );
    }

    // Position: linear extrapolation from velocity (conservative).
    let k_f = k as f32 * dt;
    projected.position[0] += entity.velocity[0] * k_f;
    projected.position[1] += entity.velocity[1] * k_f;

    // Death check: if projected qe below existence threshold, mark dead.
    if projected.qe < crate::batch::constants::QE_MIN_EXISTENCE {
        projected.alive = false;
    }

    projected
}

/// Proyecta el grid de nutrientes K ticks (decaimiento simple).
/// Projects nutrient grid K ticks (simple decay).
#[inline]
pub fn project_nutrient_grid(grid: &[f32; GRID_CELLS], k: u32, _dt: f32) -> [f32; GRID_CELLS] {
    if k == 0 {
        return *grid;
    }
    let mut result = *grid;
    // Nutrient regeneration is slow (geological) — approximate as constant for small K.
    // For large K, apply mild exponential decay (nutrient consumption by entities).
    let decay_rate = crate::blueprint::constants::temporal_telescope::PROJECTION_NUTRIENT_DECAY_RATE;
    let factor = (1.0 - decay_rate).powi(k as i32);
    for cell in &mut result {
        *cell *= factor;
    }
    result
}

/// Proyecta el grid de irradiancia K ticks (modulación estacional).
/// Projects irradiance grid K ticks (seasonal modulation).
#[inline]
pub fn project_irradiance_grid(
    grid: &[f32; GRID_CELLS],
    tick_id: u64,
    k: u32,
) -> [f32; GRID_CELLS] {
    if k == 0 {
        return *grid;
    }
    // Irradiance is externally driven (solar). Project seasonal modulation.
    let season_rate = crate::batch::constants::SEASON_RATE;
    let season_amp = crate::batch::constants::SEASON_AMPLITUDE;
    let current_season = ((tick_id as f32) * season_rate).sin() * season_amp + 1.0;
    let future_season = (((tick_id + k as u64) as f32) * season_rate).sin() * season_amp + 1.0;

    if current_season.abs() < f32::EPSILON {
        return *grid;
    }
    let ratio = future_season / current_season.max(crate::blueprint::constants::temporal_telescope::PROJECTION_IRRADIANCE_SEASON_FLOOR);

    let mut result = *grid;
    for cell in &mut result {
        *cell = (*cell * ratio).max(0.0);
    }
    result
}

/// Proyecta un mundo completo K ticks al futuro.
/// Projects a complete world K ticks into the future.
///
/// Retorna copia proyectada. El mundo original no se toca.
/// Stateless: misma entrada → misma salida.
pub fn project_world(
    world: &SimWorldFlat,
    metrics: &RegimeMetrics,
    weights: &NormalizerWeights,
    k: u32,
) -> SimWorldFlat {
    if k == 0 {
        return world.clone();
    }

    let mut projected = world.clone();
    projected.tick_id = world.tick_id + k as u64;

    // Proyectar entidades vivas.
    let mut mask = world.alive_mask;
    while mask != 0 {
        let i = mask.trailing_zeros() as usize;
        mask &= mask - 1;
        projected.entities[i] = project_entity(
            &world.entities[i],
            metrics,
            weights,
            k,
            world.dt,
        );
        // Actualizar alive_mask si la entidad murió.
        if !projected.entities[i].alive {
            projected.alive_mask &= !(1u128 << i);
            projected.entity_count = projected.entity_count.saturating_sub(1);
        }
    }

    // Proyectar grids.
    projected.nutrient_grid = project_nutrient_grid(&world.nutrient_grid, k, world.dt);
    projected.irradiance_grid = project_irradiance_grid(&world.irradiance_grid, world.tick_id, k);

    // Recalcular total_qe.
    projected.update_total_qe();

    projected
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_world_with_entity(qe: f32, dissipation: f32) -> SimWorldFlat {
        let mut w = SimWorldFlat::new(42, 0.05);
        let mut e = EntitySlot::default();
        e.qe = qe;
        e.dissipation = dissipation;
        e.radius = 1.0;
        w.spawn(e);
        w
    }

    #[test]
    fn project_world_k0_returns_clone() {
        let w = make_world_with_entity(100.0, 0.01);
        let m = RegimeMetrics::default();
        let weights = NormalizerWeights::default();
        let p = project_world(&w, &m, &weights, 0);
        assert_eq!(p.entities[0].qe, w.entities[0].qe);
        assert_eq!(p.tick_id, w.tick_id);
    }

    #[test]
    fn project_world_energy_does_not_increase() {
        let mut w = make_world_with_entity(100.0, 0.01);
        w.update_total_qe();
        let m = RegimeMetrics { hurst: 0.5, autocorrelation: 0.5, ..Default::default() };
        let weights = NormalizerWeights::default();
        let p = project_world(&w, &m, &weights, 100);
        // Axioma 5: proyección nunca crea energía (con H=0.5, no extrapola tendencia).
        assert!(p.total_qe <= w.total_qe + 1.0,
            "projection should not create energy: {} vs {}", p.total_qe, w.total_qe);
    }

    #[test]
    fn project_entity_dead_stays_dead() {
        let e = EntitySlot::default(); // alive=false
        let m = RegimeMetrics::default();
        let w = NormalizerWeights::default();
        let p = project_entity(&e, &m, &w, 100, 0.05);
        assert!(!p.alive);
    }

    #[test]
    fn project_entity_growth_increases_radius() {
        let mut e = EntitySlot::default();
        e.alive = true;
        e.qe = 100.0;
        e.radius = 0.5;
        e.growth_bias = 0.8;
        let m = RegimeMetrics::default();
        let w = NormalizerWeights::default();
        let p = project_entity(&e, &m, &w, 100, 0.05);
        assert!(p.radius > e.radius, "radius should grow: {} vs {}", p.radius, e.radius);
    }

    #[test]
    fn project_entity_position_moves() {
        let mut e = EntitySlot::default();
        e.alive = true;
        e.qe = 100.0;
        e.velocity = [1.0, 0.0];
        let m = RegimeMetrics::default();
        let w = NormalizerWeights::default();
        let p = project_entity(&e, &m, &w, 10, 0.05);
        assert!(p.position[0] > e.position[0]);
    }

    #[test]
    fn project_entity_decays_with_dissipation() {
        let mut e = EntitySlot::default();
        e.alive = true;
        e.qe = 100.0;
        e.dissipation = 0.1; // strong decay
        // With H=0.5 (neutral), inertia=0.5: blends current with decayed.
        let m = RegimeMetrics { hurst: 0.5, autocorrelation: 0.5, ..Default::default() };
        let w = NormalizerWeights::default();
        let p = project_entity(&e, &m, &w, 50, 0.05);
        assert!(p.qe < e.qe, "qe should decrease under dissipation: {} vs {}", p.qe, e.qe);
    }

    #[test]
    fn project_nutrient_k0_unchanged() {
        let grid = [1.0_f32; GRID_CELLS];
        let result = project_nutrient_grid(&grid, 0, 0.05);
        assert_eq!(result, grid);
    }

    #[test]
    fn project_irradiance_k0_unchanged() {
        let grid = [2.0_f32; GRID_CELLS];
        let result = project_irradiance_grid(&grid, 0, 0);
        assert_eq!(result, grid);
    }

    #[test]
    fn project_world_tick_id_advances() {
        let w = make_world_with_entity(100.0, 0.01);
        let m = RegimeMetrics::default();
        let weights = NormalizerWeights::default();
        let p = project_world(&w, &m, &weights, 64);
        assert_eq!(p.tick_id, w.tick_id + 64);
    }

    #[test]
    fn project_world_deterministic() {
        let w = make_world_with_entity(100.0, 0.01);
        let m = RegimeMetrics { hurst: 0.7, autocorrelation: 0.5, ..Default::default() };
        let weights = NormalizerWeights::default();
        let p1 = project_world(&w, &m, &weights, 64);
        let p2 = project_world(&w, &m, &weights, 64);
        assert_eq!(p1.entities[0].qe, p2.entities[0].qe);
        assert_eq!(p1.total_qe, p2.total_qe);
    }

    // ── Axiom Property Tests ─────────────────────────────────────

    #[test]
    fn axiom4_dissipation_always_reduces_qe() {
        // Axioma 4: toda entidad con disipación > 0 pierde energía.
        let mut w = SimWorldFlat::new(42, 0.05);
        for i in 0..10 {
            let mut e = EntitySlot::default();
            e.qe = 50.0 + i as f32 * 10.0;
            e.dissipation = 0.005 + i as f32 * 0.01;
            e.radius = 1.0;
            w.spawn(e);
        }
        w.update_total_qe();
        let m = RegimeMetrics { hurst: 0.5, autocorrelation: 0.5, ..Default::default() };
        let weights = NormalizerWeights::default();
        let p = project_world(&w, &m, &weights, 50);

        for i in 0..10 {
            if w.entities[i].alive {
                assert!(p.entities[i].qe <= w.entities[i].qe,
                    "Axiom 4: entity {i} qe increased: {} → {}", w.entities[i].qe, p.entities[i].qe);
            }
        }
    }

    #[test]
    fn axiom5_total_qe_never_increases_with_neutral_hurst() {
        // Axioma 5: energía total nunca crece (sin fuente externa).
        let mut w = SimWorldFlat::new(42, 0.05);
        for _ in 0..20 {
            let mut e = EntitySlot::default();
            e.qe = 100.0;
            e.dissipation = 0.01;
            e.radius = 1.0;
            w.spawn(e);
        }
        w.update_total_qe();
        let initial_qe = w.total_qe;

        let m = RegimeMetrics { hurst: 0.5, autocorrelation: 0.5, ..Default::default() };
        let weights = NormalizerWeights::default();

        for k in [1, 10, 50, 100, 500] {
            let p = project_world(&w, &m, &weights, k);
            assert!(p.total_qe <= initial_qe + 1.0,
                "Axiom 5: total qe increased at K={k}: {initial_qe} → {}", p.total_qe);
        }
    }

    #[test]
    fn project_entity_below_existence_threshold_dies() {
        let mut e = EntitySlot::default();
        e.alive = true;
        e.qe = 0.0001; // barely above threshold
        e.dissipation = 0.5;
        e.radius = 0.1;
        let m = RegimeMetrics { hurst: 0.5, ..Default::default() };
        let w = NormalizerWeights::default();
        let p = project_entity(&e, &m, &w, 10, 0.05);
        assert!(!p.alive, "entity with qe={} after dissipation should die", e.qe);
    }

    #[test]
    fn project_entity_negative_velocity_moves_backward() {
        let mut e = EntitySlot::default();
        e.alive = true;
        e.qe = 100.0;
        e.velocity = [-2.0, -1.0];
        e.position = [10.0, 10.0];
        let m = RegimeMetrics::default();
        let w = NormalizerWeights::default();
        let p = project_entity(&e, &m, &w, 10, 0.05);
        assert!(p.position[0] < e.position[0], "x should decrease with negative vx");
        assert!(p.position[1] < e.position[1], "y should decrease with negative vy");
    }

    #[test]
    fn project_nutrient_grid_decays_over_k() {
        let grid = [10.0_f32; GRID_CELLS];
        let p = project_nutrient_grid(&grid, 100, 0.05);
        for (i, &v) in p.iter().enumerate() {
            assert!(v < grid[i], "nutrient cell {i} should decay: {v} >= {}", grid[i]);
            assert!(v > 0.0, "nutrient cell {i} should not go negative");
        }
    }

    // ── alive_mask integrity ─────────────────────────────────────

    #[test]
    fn project_world_alive_mask_consistent() {
        let mut w = SimWorldFlat::new(42, 0.05);
        for _ in 0..10 {
            let mut e = EntitySlot::default();
            e.qe = 50.0;
            e.dissipation = 0.01;
            e.radius = 1.0;
            w.spawn(e);
        }
        let m = RegimeMetrics::default();
        let weights = NormalizerWeights::default();
        let p = project_world(&w, &m, &weights, 100);

        // Verify alive_mask matches entities[i].alive for all slots.
        for i in 0..128 {
            let bit_alive = p.alive_mask & (1u128 << i) != 0;
            assert_eq!(bit_alive, p.entities[i].alive,
                "alive_mask mismatch at slot {i}: bit={bit_alive}, field={}", p.entities[i].alive);
        }
    }
}
