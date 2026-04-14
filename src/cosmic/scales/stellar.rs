//! Escala S1 — gravedad entre estrellas + nucleosíntesis emergente.
//! Scale S1 — inter-stellar gravity + emergent nucleosynthesis.
//!
//! CT-4. Diferencia vs S0 cosmológico (`cosmological.rs`):
//! - Sin expansión Hubble (escala sub-cosmológica).
//! - Con nucleosíntesis: `freq *= (qe_initial/qe_current)^0.25` por edad.
//! Shared math: `cosmic_gravity` + `verlet`.

use crate::blueprint::equations::cosmic_gravity as grav;
use crate::blueprint::equations::derived_thresholds::DISSIPATION_PLASMA;
use crate::blueprint::equations::stellar_dynamics as sd;
use crate::blueprint::equations::verlet::{position_step_3d, velocity_step_3d};
use crate::cosmic::scale_manager::CosmicWorld;

/// Umbral de qe por debajo del cual una entidad se considera gas del disco
/// (no experimenta nucleosíntesis ni dissipation estelar).
/// qe threshold below which entities are treated as disk gas (no nucleosynthesis).
pub const DISK_GAS_THRESHOLD: f64 = 1.0;

/// Config del tick estelar.
/// Stellar tick config.
#[derive(Clone, Copy, Debug)]
pub struct StellarConfig {
    pub dissipation_rate: f64,
    pub dt: f64,
}

impl Default for StellarConfig {
    fn default() -> Self {
        // dt más pequeño que S0: estrellas se mueven más rápido en su propio frame.
        Self { dissipation_rate: DISSIPATION_PLASMA as f64 * 0.1, dt: 0.01 }
    }
}

/// Un paso estelar: gravedad N-body + Verlet + nucleosíntesis + dissipation.
/// One stellar step: N-body gravity + Verlet + nucleosynthesis + dissipation.
pub fn stellar_tick(world: &mut CosmicWorld, config: &StellarConfig) {
    let alive_indices: Vec<usize> = world
        .entities
        .iter()
        .enumerate()
        .filter(|(_, e)| e.alive)
        .map(|(i, _)| i)
        .collect();
    let n = alive_indices.len();
    if n < 2 { return; }

    let positions: Vec<[f64; 3]> = alive_indices.iter().map(|&i| world.entities[i].position).collect();
    let masses: Vec<f64> = alive_indices.iter().map(|&i| world.entities[i].qe).collect();
    let freqs: Vec<f64> = alive_indices.iter().map(|&i| world.entities[i].frequency_hz).collect();

    let acc = grav::gravity_accelerations(&positions, &masses, &freqs);

    let dt = config.dt;
    let zero = [0.0_f64; 3];
    for (k, &idx) in alive_indices.iter().enumerate() {
        let e = &mut world.entities[idx];
        let qe_before = e.qe;
        let freq_before = e.frequency_hz;

        e.position = position_step_3d(e.position, e.velocity, acc[k], dt);
        e.velocity = velocity_step_3d(e.velocity, zero, acc[k], dt);

        let is_star = qe_before >= DISK_GAS_THRESHOLD;
        if is_star {
            e.qe = grav::apply_dissipation_qe(qe_before, config.dissipation_rate, dt);
            // Nucleosíntesis: usa qe_initial del mundo como referencia absoluta
            // (todas las estrellas nacen juntas tras expand_cluster).
            if world.total_qe_initial > 0.0 && qe_before > 0.0 {
                let ratio_initial = world.total_qe_initial / (n as f64);
                e.frequency_hz = sd::nucleosynthesis_shift(freq_before, ratio_initial, e.qe);
            }
        }

        e.age_ticks = e.age_ticks.saturating_add(1);
        if e.qe <= 0.0 { e.alive = false; }
    }
    world.tick_id = world.tick_id.saturating_add(1);
}

// ─── Tests ──────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::blueprint::equations::derived_thresholds::COHERENCE_BANDWIDTH;
    use crate::cosmic::bridges::cosmo_to_stellar::expand_cluster;
    use crate::cosmic::scale_manager::CosmicEntity;

    fn sample_stellar_world() -> CosmicWorld {
        let cluster = CosmicEntity {
            qe: 3000.0,
            radius: 30.0,
            frequency_hz: 40.0,
            phase: 0.0,
            position: [0.0; 3],
            velocity: [0.0; 3],
            dissipation: 0.25,
            age_ticks: 0,
            entity_id: 1,
            alive: true,
        };
        expand_cluster(&cluster, 11, COHERENCE_BANDWIDTH as f64, 20, 60).unwrap().world
    }

    #[test]
    fn stellar_tick_is_qe_monotone_non_increasing() {
        let mut world = sample_stellar_world();
        let cfg = StellarConfig::default();
        let mut prev = world.total_qe();
        for _ in 0..30 {
            stellar_tick(&mut world, &cfg);
            let now = world.total_qe();
            assert!(now <= prev + 1e-9, "qe grew: {prev} -> {now}");
            prev = now;
        }
    }

    #[test]
    fn stellar_tick_advances_age() {
        let mut world = sample_stellar_world();
        let cfg = StellarConfig::default();
        let age_before = world.entities[0].age_ticks;
        stellar_tick(&mut world, &cfg);
        assert_eq!(world.entities[0].age_ticks, age_before + 1);
    }

    #[test]
    fn nucleosynthesis_shifts_frequencies_up_over_time() {
        let mut world = sample_stellar_world();
        let stars_before: Vec<f64> = world
            .entities
            .iter()
            .filter(|e| e.qe >= DISK_GAS_THRESHOLD)
            .map(|e| e.frequency_hz)
            .collect();
        let cfg = StellarConfig {
            dissipation_rate: 0.05,
            dt: 0.1,
        };
        for _ in 0..50 { stellar_tick(&mut world, &cfg); }
        let stars_after: Vec<f64> = world
            .entities
            .iter()
            .filter(|e| e.qe >= DISK_GAS_THRESHOLD)
            .map(|e| e.frequency_hz)
            .collect();
        assert_eq!(stars_before.len(), stars_after.len());
        let mean_before: f64 = stars_before.iter().sum::<f64>() / stars_before.len() as f64;
        let mean_after: f64 = stars_after.iter().sum::<f64>() / stars_after.len() as f64;
        assert!(mean_after > mean_before, "blue shift not observed: {mean_before} -> {mean_after}");
    }

    #[test]
    fn stellar_tick_no_op_on_single_body() {
        let mut world = CosmicWorld::new(1);
        world.spawn(CosmicEntity {
            qe: 10.0,
            radius: 1.0,
            frequency_hz: 50.0,
            phase: 0.0,
            position: [1.0, 2.0, 3.0],
            velocity: [0.1, 0.0, 0.0],
            dissipation: 0.0,
            age_ticks: 0,
            entity_id: 0,
            alive: true,
        });
        let before = world.entities[0].position;
        stellar_tick(&mut world, &StellarConfig::default());
        assert_eq!(world.entities[0].position, before, "single body must not drift under N-body loop");
    }
}
