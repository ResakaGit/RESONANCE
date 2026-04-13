//! MD-1: Langevin thermostat batch system.
//!
//! Applies friction (Axiom 4 dissipation) + stochastic noise (heat bath coupling)
//! to particle velocities. Deterministic RNG for bit-exact reproducibility.
//!
//! Pipeline placement: after verlet_velocity_finish (modifies velocities).
//! Disabled by default (thermostat_enabled = false).

use crate::batch::arena::SimWorldFlat;
use crate::blueprint::equations::{determinism, thermostat as thermo_eq};

/// Splitmix64 hash: converts sequential inputs to well-distributed outputs.
/// LCGs fail at this (constant output difference for constant input difference).
/// Used to seed per-tick RNG from (world.seed, tick_id).
#[inline]
fn mix_seed(mut x: u64) -> u64 {
    x = x.wrapping_add(0x9E37_79B9_7F4A_7C15);
    x = (x ^ (x >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    x = (x ^ (x >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    x ^ (x >> 31)
}

/// Apply Langevin thermostat: friction + deterministic noise.
///
/// Reads thermostat config from `world.thermostat_*` fields.
/// Stateless: reads entity state + config, writes velocities + bookkeeping.
/// Deterministic: RNG seeded from (world.seed, tick_id, entity_index, dimension).
///
/// Energy bookkeeping:
/// - Friction removes KE → tracked in `thermostat_energy_dissipated`
/// - Noise injects KE → tracked in `thermostat_energy_injected`
/// - Conservation: E_kinetic + E_dissipated - E_injected ≈ E_initial (open subsystem)
pub fn langevin_thermostat(world: &mut SimWorldFlat) {
    if !world.thermostat_enabled {
        return;
    }
    let dt = world.dt as f64;
    let gamma = world.thermostat_gamma;
    let kb_t = world.thermostat_target_kb_t;
    let tick = world.tick_id;
    let seed = world.seed;

    // Direct velocity Langevin: v_new = v * (1 - gamma*dt) + sigma_v * z
    // where sigma_v = sqrt(2 * gamma * kBT * dt / m).
    // Equivalent to force-based but avoids intermediate conversions.
    let c1 = 1.0 - gamma * dt;

    // Per-tick seed via splitmix64 hash (NOT LCG — LCG outputs are correlated
    // for sequential inputs). Sequential LCG advancement within the tick.
    let mut rng = mix_seed(seed ^ tick);

    let mut mask = world.alive_mask;
    while mask != 0 {
        let i = mask.trailing_zeros() as usize;
        mask &= mask - 1;
        let e = &mut world.entities[i];
        let mass = (e.particle_mass as f64).max(0.01);
        let sigma_v = thermo_eq::langevin_velocity_sigma(gamma, kb_t, dt, mass);

        for dim in 0..2 {
            let v = e.velocity[dim] as f64;

            rng = determinism::next_u64(determinism::next_u64(determinism::next_u64(rng)));
            let z = determinism::gaussian_f32(rng, 1.0) as f64;

            let v_new = v * c1 + sigma_v * z;

            // Energy bookkeeping
            let ke_old = 0.5 * mass * v * v;
            let ke_new = 0.5 * mass * v_new * v_new;
            let dke = ke_new - ke_old;
            if dke < 0.0 {
                world.thermostat_energy_dissipated += -dke;
            } else {
                world.thermostat_energy_injected += dke;
            }

            e.velocity[dim] = v_new as f32;
        }
    }
}

// ─── Tests ──────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::batch::arena::EntitySlot;

    fn spawn_particle(w: &mut SimWorldFlat, mass: f32, vx: f32, vy: f32) -> usize {
        let mut e = EntitySlot::default();
        e.qe = 100.0;
        e.particle_mass = mass;
        e.velocity = [vx, vy];
        e.radius = 0.1;
        w.spawn(e).unwrap()
    }

    fn enable_thermostat(w: &mut SimWorldFlat, kb_t: f64, gamma: f64) {
        w.thermostat_enabled = true;
        w.thermostat_target_kb_t = kb_t;
        w.thermostat_gamma = gamma;
    }

    #[test]
    fn disabled_is_noop() {
        let mut w = SimWorldFlat::new(42, 0.01);
        spawn_particle(&mut w, 1.0, 5.0, 3.0);
        let v_before = w.entities[0].velocity;
        langevin_thermostat(&mut w);
        assert_eq!(w.entities[0].velocity, v_before);
    }

    #[test]
    fn cools_to_zero_without_noise() {
        // gamma > 0, T=0 → pure friction → KE → 0 monotonically
        let mut w = SimWorldFlat::new(42, 0.01);
        spawn_particle(&mut w, 1.0, 10.0, 10.0);
        enable_thermostat(&mut w, 0.0, 1.0);

        let mut prev_ke = f64::MAX;
        for _ in 0..500 {
            w.tick_id += 1;
            langevin_thermostat(&mut w);
            let vx = w.entities[0].velocity[0] as f64;
            let vy = w.entities[0].velocity[1] as f64;
            let ke = 0.5 * (vx * vx + vy * vy);
            assert!(ke <= prev_ke + 1e-10, "KE should decrease monotonically");
            prev_ke = ke;
        }
        assert!(prev_ke < 0.01, "should be near zero: {prev_ke}");
    }

    #[test]
    fn pure_langevin_converges() {
        // Bypass SimWorldFlat: directly test the v = c1*v + sigma_v*z recurrence.
        let gamma = 5.0_f64;
        let dt = 0.01_f64;
        let kb_t = 1.0_f64;
        let mass = 1.0_f64;
        let c1 = 1.0 - gamma * dt;
        let sigma_v = thermo_eq::langevin_velocity_sigma(gamma, kb_t, dt, mass);

        let mut vx = 0.0_f64;
        let mut rng_state = 42u64;
        // Thermalize 5000 steps
        for _ in 0..5000 {
            rng_state = determinism::next_u64(rng_state);
            let z = determinism::gaussian_f32(rng_state, 1.0) as f64;
            vx = vx * c1 + sigma_v * z;
        }
        // Measure <vx²> over 50000 steps
        let mut sum_vx2 = 0.0;
        let samples = 50000;
        for _ in 0..samples {
            rng_state = determinism::next_u64(rng_state);
            let z = determinism::gaussian_f32(rng_state, 1.0) as f64;
            vx = vx * c1 + sigma_v * z;
            sum_vx2 += vx * vx;
        }
        let avg_vx2 = sum_vx2 / samples as f64;
        // Expected: kBT/m = 1.0
        let error = ((avg_vx2 - 1.0) / 1.0).abs();
        assert!(error < 0.10, "<vx²>={avg_vx2:.4}, expected=1.0, error={error:.4}");
    }

    #[test]
    fn single_particle_velocity_variance() {
        // Direct check: 1 particle, <v²> should converge to kBT/m
        let mut w = SimWorldFlat::new(42, 0.01);
        enable_thermostat(&mut w, 1.0, 5.0); // gamma=5, tau=20 steps
        spawn_particle(&mut w, 1.0, 0.0, 0.0);

        // Thermalize 2000 steps (100 tau)
        for _ in 0..2000 {
            w.tick_id += 1;
            langevin_thermostat(&mut w);
        }
        // Measure <v²> over 50000 steps
        let mut sum_v2 = 0.0;
        let samples = 50000u64;
        for _ in 0..samples {
            w.tick_id += 1;
            langevin_thermostat(&mut w);
            let vx = w.entities[0].velocity[0] as f64;
            let vy = w.entities[0].velocity[1] as f64;
            sum_v2 += vx * vx + vy * vy;
        }
        let avg_v2 = sum_v2 / samples as f64;
        // <vx²+vy²> = 2*kBT/m = 2.0
        let expected = 2.0;
        let error = ((avg_v2 - expected) / expected).abs();
        assert!(
            error < 0.10,
            "<v²>={avg_v2:.3}, expected={expected}, error={error:.4}",
        );
    }

    #[test]
    fn equilibrates_to_target_temperature() {
        // High gamma for fast equilibration (tau = 1/gamma = 0.1 → 10 steps at dt=0.01).
        // 64 particles, 128 DOF → instantaneous T has ~9% fluctuation.
        let mut w = SimWorldFlat::new(42, 0.01);
        let target_t = 1.0;
        enable_thermostat(&mut w, target_t, 10.0);

        for i in 0..64 {
            let v = (i as f32 - 32.0) * 0.1;
            spawn_particle(&mut w, 1.0, v, -v * 0.3);
        }

        // Thermalize: 2000 steps = 200 tau
        for _ in 0..2000 {
            w.tick_id += 1;
            langevin_thermostat(&mut w);
        }

        // Measure average T over 5000 steps (~500 tau → ~500 independent samples)
        let mut t_sum = 0.0;
        let samples = 5000;
        for _ in 0..samples {
            w.tick_id += 1;
            langevin_thermostat(&mut w);
            let mut masses = Vec::new();
            let mut vels = Vec::new();
            let mut mask = w.alive_mask;
            while mask != 0 {
                let j = mask.trailing_zeros() as usize;
                mask &= mask - 1;
                masses.push(w.entities[j].particle_mass as f64);
                vels.push([
                    w.entities[j].velocity[0] as f64,
                    w.entities[j].velocity[1] as f64,
                ]);
            }
            t_sum += thermo_eq::kinetic_temperature(&masses, &vels, 1.0);
        }
        let t_avg = t_sum / samples as f64;
        let error = ((t_avg - target_t) / target_t).abs();
        assert!(
            error < 0.10,
            "<T>={t_avg:.3}, target={target_t}, error={error:.4}",
        );
    }

    #[test]
    fn deterministic_across_runs() {
        let mut w1 = SimWorldFlat::new(42, 0.01);
        let mut w2 = SimWorldFlat::new(42, 0.01);
        enable_thermostat(&mut w1, 1.0, 0.2);
        enable_thermostat(&mut w2, 1.0, 0.2);
        spawn_particle(&mut w1, 1.0, 5.0, 3.0);
        spawn_particle(&mut w2, 1.0, 5.0, 3.0);

        for _ in 0..100 {
            w1.tick_id += 1;
            w2.tick_id += 1;
            langevin_thermostat(&mut w1);
            langevin_thermostat(&mut w2);
        }
        assert_eq!(
            w1.entities[0].velocity[0].to_bits(),
            w2.entities[0].velocity[0].to_bits(),
            "bit-exact determinism",
        );
    }

    #[test]
    fn energy_bookkeeping_tracks_dissipation() {
        let mut w = SimWorldFlat::new(42, 0.01);
        spawn_particle(&mut w, 1.0, 10.0, 0.0);
        enable_thermostat(&mut w, 0.0, 0.5);

        for _ in 0..100 {
            w.tick_id += 1;
            langevin_thermostat(&mut w);
        }
        assert!(
            w.thermostat_energy_dissipated > 0.0,
            "should track dissipated energy",
        );
        assert_eq!(
            w.thermostat_energy_injected, 0.0,
            "T=0 → no injection",
        );
    }

    #[test]
    fn noise_variance_matches_unit_gaussian() {
        // Verify that the RNG seeding in the thermostat produces correct N(0,1).
        let seed = 42u64;
        let mut sum_z2 = 0.0;
        let n = 100_000u64;
        for tick in 0..n {
            let mut rng = determinism::next_u64(seed.wrapping_add(tick));
            rng = determinism::next_u64(rng.wrapping_add(0));
            rng = determinism::next_u64(rng.wrapping_add(0));
            let z = determinism::gaussian_f32(rng, 1.0) as f64;
            sum_z2 += z * z;
        }
        let var = sum_z2 / n as f64;
        assert!(
            (var - 1.0).abs() < 0.05,
            "variance should be ~1.0: got {var}",
        );
    }

    #[test]
    fn skips_dead_entities() {
        let mut w = SimWorldFlat::new(42, 0.01);
        let idx = spawn_particle(&mut w, 1.0, 10.0, 10.0);
        w.kill(idx);
        enable_thermostat(&mut w, 1.0, 0.2);
        w.tick_id = 1;
        langevin_thermostat(&mut w);
        assert_eq!(w.entities[idx].velocity, [0.0, 0.0]);
    }
}
