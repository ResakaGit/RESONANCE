//! Phase::AtomicLayer batch systems — dissipation, movement, collision,
//! velocity cap, will-to-velocity, locomotion drain.
//!
//! All math delegated to `blueprint::equations`. No inline formulas.

use crate::batch::arena::SimWorldFlat;
use crate::batch::constants::{
    COLLISION_EXCHANGE_FRACTION, GRAVITY_ACCELERATION, TENSION_FORCE_SCALE,
    TENSION_RADIUS_MULTIPLIER,
};
use crate::batch::scratch::ScratchPad;
use crate::blueprint::equations::emergence::entrainment as entrainment_eq;
use crate::blueprint::{constants, equations};

/// L3→L0: entropy drain per tick.
///
/// `loss = dissipation_loss(qe, dissipation_rate)`.
/// Calls `equations::dissipation_loss` which clamps rate to [MIN, MAX].
pub fn dissipation(world: &mut SimWorldFlat) {
    let mut mask = world.alive_mask;
    while mask != 0 {
        let i = mask.trailing_zeros() as usize;
        mask &= mask - 1;
        let e = &mut world.entities[i];
        let loss = equations::dissipation_loss(e.qe, e.dissipation);
        let new_qe = (e.qe - loss).max(0.0);
        if e.qe != new_qe {
            e.qe = new_qe;
        }
    }
}

/// MD-0 Verlet position half-step: x += v*dt + 0.5*a_old*dt².
///
/// Uses stored `old_acceleration` from previous tick (gravity + external forces).
/// Runs BEFORE force systems so forces are evaluated at the new position.
///
/// Axiom 7: gravitational pull = distance attenuation from ground.
/// Axiom 4: ground collision dissipates kinetic energy.
pub fn verlet_position_step(world: &mut SimWorldFlat) {
    let dt = world.dt;
    let mut mask = world.alive_mask;
    while mask != 0 {
        let i = mask.trailing_zeros() as usize;
        mask &= mask - 1;
        let e = &mut world.entities[i];
        e.position[0] =
            equations::verlet::position_step(e.position[0], e.velocity[0], e.old_acceleration[0], dt);
        e.position[1] =
            equations::verlet::position_step(e.position[1], e.velocity[1], e.old_acceleration[1], dt);
        // Ground collision: floor at y=0 (free-space only — PBC has no floor)
        if world.sim_box.is_none() && e.position[1] < 0.0 {
            e.position[1] = 0.0;
            e.velocity[1] = 0.0;
            e.old_acceleration[1] = 0.0;
        }
    }
}

/// MD-2: wrap positions into periodic box [0, L). No-op if sim_box is None.
///
/// Runs after verlet_position_step. Torus topology: particle leaving one
/// edge reappears on the opposite side. Axiom 7 holds on the torus.
pub fn wrap_positions(world: &mut SimWorldFlat) {
    let Some(bl) = world.sim_box else { return };
    let mut mask = world.alive_mask;
    while mask != 0 {
        let i = mask.trailing_zeros() as usize;
        mask &= mask - 1;
        let e = &mut world.entities[i];
        e.position[0] = equations::pbc::wrap(e.position[0], bl[0]);
        e.position[1] = equations::pbc::wrap(e.position[1], bl[1]);
    }
}

/// MD-0 Verlet velocity finish: v += 0.5*(a_old + a_new)*dt.
///
/// Runs AFTER force systems. Applies gravity via Verlet averaging.
/// Stores `a_new` into `old_acceleration` for the next tick's position step.
/// Gravity disabled under PBC (no floor, no preferred direction).
///
/// Particle forces (Coulomb/LJ) contribute Euler kicks to velocity between
/// position_step and velocity_finish — acceptable hybrid for MD-0.
/// Full force-accumulator Verlet deferred to MD-2.
pub fn verlet_velocity_finish(world: &mut SimWorldFlat) {
    let dt = world.dt;
    let has_pbc = world.sim_box.is_some();
    let mut mask = world.alive_mask;
    while mask != 0 {
        let i = mask.trailing_zeros() as usize;
        mask &= mask - 1;
        let e = &mut world.entities[i];
        // PBC: no gravity (no preferred direction on torus).
        // Free-space: gravity toward y=0.
        let a_new = if has_pbc {
            [0.0_f32, 0.0]
        } else {
            [0.0_f32, -GRAVITY_ACCELERATION]
        };
        e.velocity[0] =
            equations::verlet::velocity_step(e.velocity[0], e.old_acceleration[0], a_new[0], dt);
        e.velocity[1] =
            equations::verlet::velocity_step(e.velocity[1], e.old_acceleration[1], a_new[1], dt);
        e.old_acceleration = a_new;
        // Ground: entity on floor cannot acquire downward velocity (free-space only)
        if !has_pbc && e.position[1] <= 0.0 && e.velocity[1] < 0.0 {
            e.velocity[1] = 0.0;
        }
    }
}

/// N² collision: radius overlap → energy exchange via oscillatory interference.
///
/// 1. Collect overlapping pairs into `scratch.pairs`.
/// 2. For each pair, compute `equations::interference` at t=0.
/// 3. Transfer a fraction of energy based on interference sign.
///
/// Conservation: energy transferred from A→B equals energy lost by A.
pub fn collision(world: &mut SimWorldFlat, scratch: &mut ScratchPad) {
    // ── Phase 1: detect overlapping pairs ───────────────────────────────────
    scratch.pairs_len = 0;
    let entities = &world.entities;
    let mask = world.alive_mask;

    let mut mi = mask;
    while mi != 0 {
        let i = mi.trailing_zeros() as usize;
        mi &= mi - 1;

        let mut mj = mi; // only j > i (mi already has i cleared)
        while mj != 0 {
            let j = mj.trailing_zeros() as usize;
            mj &= mj - 1;

            let dx = entities[i].position[0] - entities[j].position[0];
            let dy = entities[i].position[1] - entities[j].position[1];
            let dist_sq = dx * dx + dy * dy;
            let r_sum = entities[i].radius + entities[j].radius;

            if dist_sq < r_sum * r_sum && scratch.pairs_len < scratch.pairs.len() {
                scratch.pairs[scratch.pairs_len] = (i as u8, j as u8);
                scratch.pairs_len += 1;
            }
        }
    }

    // ── Phase 2: resolve energy exchange ────────────────────────────────────
    for p in 0..scratch.pairs_len {
        let (ai, bi) = scratch.pairs[p];
        let (a, b) = (ai as usize, bi as usize);

        let transfer = equations::interference(
            world.entities[a].frequency_hz,
            world.entities[a].phase,
            world.entities[b].frequency_hz,
            world.entities[b].phase,
            0.0, // instantaneous
        );

        let donor_qe = if transfer > 0.0 {
            world.entities[a].qe
        } else {
            world.entities[b].qe
        };
        let amount = transfer.abs() * donor_qe * COLLISION_EXCHANGE_FRACTION;
        let safe_amount = amount.min(donor_qe);

        if safe_amount <= 0.0 {
            continue;
        }

        if transfer > 0.0 {
            world.entities[a].qe -= safe_amount;
            world.entities[b].qe += safe_amount;
        } else {
            world.entities[b].qe -= safe_amount;
            world.entities[a].qe += safe_amount;
        }
    }
}

/// Clamp velocity magnitude to MAX_GLOBAL_VELOCITY.
pub fn velocity_cap(world: &mut SimWorldFlat) {
    let max_v = constants::MAX_GLOBAL_VELOCITY;
    let max_v_sq = max_v * max_v;
    let mut mask = world.alive_mask;
    while mask != 0 {
        let i = mask.trailing_zeros() as usize;
        mask &= mask - 1;
        let e = &mut world.entities[i];
        let speed_sq = e.velocity[0] * e.velocity[0] + e.velocity[1] * e.velocity[1];
        if speed_sq > max_v_sq {
            let inv_speed = max_v / speed_sq.sqrt();
            e.velocity[0] *= inv_speed;
            e.velocity[1] *= inv_speed;
        }
    }
}

/// L7 WillActuator → L3 FlowVector: apply will intent as acceleration.
///
/// Uses `equations::will_force(intent, buffer, max_buffer)` → force vector,
/// then `equations::integrate_velocity(velocity, force, qe, dt)`.
pub fn will_to_velocity(world: &mut SimWorldFlat) {
    let dt = world.dt;
    let mut mask = world.alive_mask;
    while mask != 0 {
        let i = mask.trailing_zeros() as usize;
        mask &= mask - 1;
        let e = &mut world.entities[i];
        let intent_sq = e.will_intent[0] * e.will_intent[0] + e.will_intent[1] * e.will_intent[1];
        if intent_sq < 1e-10 {
            continue;
        }
        let intent = glam::Vec2::new(e.will_intent[0], e.will_intent[1]);
        let force = equations::will_force(intent, e.engine_buffer, e.engine_max);
        let vel = glam::Vec2::new(e.velocity[0], e.velocity[1]);
        let new_vel = equations::integrate_velocity(
            vel,
            force,
            e.qe.max(crate::batch::constants::GUARD_EPSILON),
            dt,
        );
        if new_vel.is_finite() {
            e.velocity[0] = new_vel.x;
            e.velocity[1] = new_vel.y;
        }
    }
}

/// L3 FlowVector → L0 BaseEnergy: movement costs energy.
///
/// `cost = locomotion_energy_cost(qe, speed, terrain_factor)`.
/// Terrain factor simplified to 1.0 for batch (no terrain context).
pub fn locomotion_drain(world: &mut SimWorldFlat) {
    let mut mask = world.alive_mask;
    while mask != 0 {
        let i = mask.trailing_zeros() as usize;
        mask &= mask - 1;
        let e = &mut world.entities[i];
        let speed = (e.velocity[0] * e.velocity[0] + e.velocity[1] * e.velocity[1]).sqrt();
        if speed < 1e-4 {
            continue;
        }
        let cost = equations::locomotion_energy_cost(e.qe, speed, 1.0);
        let drain = cost.min(e.qe);
        if drain > 0.0 {
            e.qe -= drain;
        }
    }
}

/// AC-2 Kuramoto entrainment: nearby entities synchronize frequency.
///
/// For each pair within `ENTRAINMENT_SCAN_RADIUS`, apply phase coupling.
/// Calls `entrainment_eq::kuramoto_pair_delta`.
pub fn entrainment(world: &mut SimWorldFlat, scratch: &mut ScratchPad) {
    let range_sq = constants::ENTRAINMENT_SCAN_RADIUS * constants::ENTRAINMENT_SCAN_RADIUS;
    scratch.pairs_len = 0;

    let mut mi = world.alive_mask;
    while mi != 0 {
        let i = mi.trailing_zeros() as usize;
        mi &= mi - 1;

        let mut mj = mi;
        while mj != 0 {
            let j = mj.trailing_zeros() as usize;
            mj &= mj - 1;

            let dx = world.entities[i].position[0] - world.entities[j].position[0];
            let dy = world.entities[i].position[1] - world.entities[j].position[1];
            let dist_sq = dx * dx + dy * dy;
            if dist_sq >= range_sq {
                continue;
            }

            let dist = dist_sq.sqrt();
            let coupling = equations::entrainment_coupling_at_distance(
                constants::KURAMOTO_BASE_COUPLING,
                dist,
                constants::ENTRAINMENT_COHERENCE_LAMBDA,
            );
            if coupling < 1e-6 {
                continue;
            }

            let delta_i = entrainment_eq::kuramoto_pair_delta(
                world.entities[i].frequency_hz,
                world.entities[j].frequency_hz,
                coupling,
            );
            let delta_j = -delta_i;

            world.entities[i].frequency_hz += delta_i * world.dt;
            world.entities[j].frequency_hz += delta_j * world.dt;
        }
    }
}

/// L11 TensionField: gravity/magnetic force between nearby entities.
///
/// Entities with nonzero tension radius attract/repel neighbors.
pub fn tension_field_apply(world: &mut SimWorldFlat, _scratch: &mut ScratchPad) {
    let dt = world.dt;
    let mut mi = world.alive_mask;
    while mi != 0 {
        let i = mi.trailing_zeros() as usize;
        mi &= mi - 1;

        // Only entities that have a tension field active (adapt_rate > 0 as proxy)
        // In batch, use pressure_dqe > 0 as indicator of active tension
        let t_radius = world.entities[i].radius * TENSION_RADIUS_MULTIPLIER;
        let t_radius_sq = t_radius * t_radius;

        let mut mj = world.alive_mask & !((1u128 << i) | ((1u128 << i) - 1));
        while mj != 0 {
            let j = mj.trailing_zeros() as usize;
            mj &= mj - 1;

            let dx = world.entities[j].position[0] - world.entities[i].position[0];
            let dy = world.entities[j].position[1] - world.entities[i].position[1];
            let dist_sq = dx * dx + dy * dy;
            if dist_sq >= t_radius_sq || dist_sq < crate::batch::constants::GUARD_EPSILON {
                continue;
            }

            let dist = dist_sq.sqrt();
            let force_mag =
                (world.entities[i].qe * world.entities[j].qe) / (dist_sq * TENSION_FORCE_SCALE);
            let force_mag = force_mag.min(1.0); // cap force

            let nx = dx / dist;
            let ny = dy / dist;

            // Attractive: pull together (gravity-like)
            world.entities[i].velocity[0] += nx * force_mag * dt;
            world.entities[i].velocity[1] += ny * force_mag * dt;
            world.entities[j].velocity[0] -= nx * force_mag * dt;
            world.entities[j].velocity[1] -= ny * force_mag * dt;
        }
    }
}

/// Containment check: overlapping entities apply thermal drag.
///
/// Larger entity drags the smaller one, transferring thermal energy.
pub fn containment_check(world: &mut SimWorldFlat, _scratch: &mut ScratchPad) {
    let mut mi = world.alive_mask;
    while mi != 0 {
        let i = mi.trailing_zeros() as usize;
        mi &= mi - 1;

        let mut mj = mi;
        while mj != 0 {
            let j = mj.trailing_zeros() as usize;
            mj &= mj - 1;

            let dx = world.entities[i].position[0] - world.entities[j].position[0];
            let dy = world.entities[i].position[1] - world.entities[j].position[1];
            let dist = (dx * dx + dy * dy).sqrt();
            let r_sum = world.entities[i].radius + world.entities[j].radius;

            if dist >= r_sum {
                continue;
            }

            // Overlap → drag the smaller entity
            let overlap = r_sum - dist;
            let drag = overlap * super::super::constants::CONTAINMENT_DRAG_COEFF * world.dt;
            if world.entities[i].radius < world.entities[j].radius {
                world.entities[i].velocity[0] *= (1.0 - drag).max(0.0);
                world.entities[i].velocity[1] *= (1.0 - drag).max(0.0);
            } else {
                world.entities[j].velocity[0] *= (1.0 - drag).max(0.0);
                world.entities[j].velocity[1] *= (1.0 - drag).max(0.0);
            }
        }
    }
}

// ─── Tests ──────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::batch::arena::EntitySlot;

    fn spawn_entity(w: &mut SimWorldFlat, qe: f32, x: f32, y: f32, radius: f32) -> usize {
        let mut e = EntitySlot::default();
        e.qe = qe;
        e.position = [x, y];
        e.radius = radius;
        e.frequency_hz = 440.0;
        e.dissipation = 0.01;
        w.spawn(e).unwrap()
    }

    // ── dissipation ─────────────────────────────────────────────────────────

    #[test]
    fn dissipation_reduces_energy() {
        let mut w = SimWorldFlat::new(0, 0.05);
        spawn_entity(&mut w, 100.0, 0.0, 0.0, 1.0);
        let before = w.entities[0].qe;
        dissipation(&mut w);
        assert!(w.entities[0].qe < before, "energy should decrease");
        assert!(w.entities[0].qe > 0.0, "energy should stay positive");
    }

    #[test]
    fn dissipation_skips_dead_entities() {
        let mut w = SimWorldFlat::new(0, 0.05);
        spawn_entity(&mut w, 100.0, 0.0, 0.0, 1.0);
        w.kill(0);
        dissipation(&mut w);
        assert_eq!(w.entities[0].qe, 0.0, "dead entity untouched");
    }

    #[test]
    fn dissipation_uses_equation() {
        let mut w = SimWorldFlat::new(0, 0.05);
        let idx = spawn_entity(&mut w, 100.0, 0.0, 0.0, 1.0);
        w.entities[idx].dissipation = 0.05;
        let expected_loss = equations::dissipation_loss(100.0, 0.05);
        dissipation(&mut w);
        let actual = 100.0 - w.entities[idx].qe;
        assert!(
            (actual - expected_loss).abs() < 1e-5,
            "loss={actual}, expected={expected_loss}",
        );
    }

    #[test]
    fn dissipation_never_negative() {
        let mut w = SimWorldFlat::new(0, 0.05);
        let idx = spawn_entity(&mut w, 0.001, 0.0, 0.0, 1.0);
        w.entities[idx].dissipation = 0.5; // max rate
        dissipation(&mut w);
        assert!(w.entities[idx].qe >= 0.0);
    }

    // ── verlet_position_step + verlet_velocity_finish ─────────────────────

    /// Helper: run full Verlet step (position + velocity) for tests without
    /// intermediate force recomputation.
    fn verlet_full(w: &mut SimWorldFlat) {
        verlet_position_step(w);
        verlet_velocity_finish(w);
    }

    #[test]
    fn verlet_displaces_position_with_gravity() {
        let mut w = SimWorldFlat::new(0, 0.05);
        let idx = spawn_entity(&mut w, 100.0, 0.0, 5.0, 1.0);
        w.entities[idx].velocity = [10.0, 20.0];
        verlet_full(&mut w);
        // x: v*dt + 0.5*0*dt² = 10*0.05 = 0.5 (old_acceleration starts at 0)
        assert!(
            (w.entities[idx].position[0] - 0.5).abs() < 1e-3,
            "x ~ vx * dt, got {}",
            w.entities[idx].position[0],
        );
        // y: upward velocity should move up (gravity partially counteracts)
        assert!(
            w.entities[idx].position[1] > 5.0,
            "upward velocity should move up",
        );
    }

    #[test]
    fn verlet_skips_dead_entities() {
        let mut w = SimWorldFlat::new(0, 0.05);
        let idx = spawn_entity(&mut w, 100.0, 5.0, 5.0, 1.0);
        w.entities[idx].velocity = [10.0, 10.0];
        w.kill(idx);
        verlet_full(&mut w);
        assert_eq!(w.entities[idx].position[0], 0.0);
    }

    #[test]
    fn verlet_zero_velocity_falls() {
        let mut w = SimWorldFlat::new(0, 0.05);
        spawn_entity(&mut w, 100.0, 3.0, 7.0, 1.0);
        // Tick 1: old_acceleration=[0,0] → position stays, velocity acquires -g/2*dt
        // Tick 2: old_acceleration=[0,-g] → position advances downward
        verlet_full(&mut w);
        verlet_full(&mut w);
        assert!(
            (w.entities[0].position[0] - 3.0).abs() < 1e-5,
            "x unchanged",
        );
        assert!(w.entities[0].position[1] < 7.0, "gravity should pull down");
    }

    #[test]
    fn verlet_floor_collision() {
        let mut w = SimWorldFlat::new(0, 0.05);
        let idx = spawn_entity(&mut w, 100.0, 3.0, 0.01, 1.0);
        w.entities[idx].velocity = [0.0, -10.0];
        verlet_full(&mut w);
        assert_eq!(w.entities[idx].position[1], 0.0, "should stop at floor");
        assert_eq!(w.entities[idx].velocity[1], 0.0, "vertical velocity zeroed");
    }

    #[test]
    fn verlet_energy_drift_harmonic_1k() {
        // Harmonic oscillator via Verlet: energy drift must be bounded.
        let k = 1.0_f32;
        let dt = 0.01_f32;
        let (mut x, mut v) = (1.0_f32, 0.0_f32);
        let e0 = 0.5 * k * x * x + 0.5 * v * v;
        let mut max_drift: f32 = 0.0;
        for _ in 0..1_000 {
            let a_old = -k * x;
            x = equations::verlet::position_step(x, v, a_old, dt);
            let a_new = -k * x;
            v = equations::verlet::velocity_step(v, a_old, a_new, dt);
            let drift = ((0.5 * k * x * x + 0.5 * v * v - e0) / e0).abs();
            if drift > max_drift {
                max_drift = drift;
            }
        }
        assert!(max_drift < 1e-4, "energy drift {max_drift} exceeds 1e-4");
    }

    // ── collision ───────────────────────────────────────────────────────────

    #[test]
    fn collision_detects_overlapping_pair() {
        let mut w = SimWorldFlat::new(0, 0.05);
        spawn_entity(&mut w, 100.0, 0.0, 0.0, 1.0);
        spawn_entity(&mut w, 100.0, 1.0, 0.0, 1.0); // distance=1, radii sum=2 → overlap
        let mut scratch = ScratchPad::new();
        collision(&mut w, &mut scratch);
        assert_eq!(scratch.pairs_len, 1);
    }

    #[test]
    fn collision_ignores_distant_pair() {
        let mut w = SimWorldFlat::new(0, 0.05);
        spawn_entity(&mut w, 100.0, 0.0, 0.0, 1.0);
        spawn_entity(&mut w, 100.0, 10.0, 0.0, 1.0); // distance=10, radii sum=2 → no overlap
        let mut scratch = ScratchPad::new();
        collision(&mut w, &mut scratch);
        assert_eq!(scratch.pairs_len, 0);
    }

    #[test]
    fn collision_conserves_total_energy() {
        let mut w = SimWorldFlat::new(0, 0.05);
        spawn_entity(&mut w, 100.0, 0.0, 0.0, 1.0);
        spawn_entity(&mut w, 50.0, 0.5, 0.0, 1.0);
        let total_before = w.entities[0].qe + w.entities[1].qe;
        let mut scratch = ScratchPad::new();
        collision(&mut w, &mut scratch);
        let total_after = w.entities[0].qe + w.entities[1].qe;
        assert!(
            (total_after - total_before).abs() < 1e-4,
            "before={total_before}, after={total_after}",
        );
    }

    #[test]
    fn collision_skips_dead_entities() {
        let mut w = SimWorldFlat::new(0, 0.05);
        spawn_entity(&mut w, 100.0, 0.0, 0.0, 1.0);
        spawn_entity(&mut w, 100.0, 0.5, 0.0, 1.0);
        w.kill(1);
        let mut scratch = ScratchPad::new();
        collision(&mut w, &mut scratch);
        assert_eq!(scratch.pairs_len, 0);
    }

    #[test]
    fn collision_energy_stays_non_negative() {
        let mut w = SimWorldFlat::new(0, 0.05);
        let a = spawn_entity(&mut w, 0.02, 0.0, 0.0, 1.0);
        let b = spawn_entity(&mut w, 1000.0, 0.5, 0.0, 1.0);
        w.entities[a].frequency_hz = 100.0;
        w.entities[b].frequency_hz = 500.0;
        let mut scratch = ScratchPad::new();
        collision(&mut w, &mut scratch);
        assert!(w.entities[a].qe >= 0.0);
        assert!(w.entities[b].qe >= 0.0);
    }

    // ── velocity_cap ────────────────────────────────────────────────────────

    #[test]
    fn velocity_cap_clamps_fast_entity() {
        let mut w = SimWorldFlat::new(0, 0.05);
        let idx = spawn_entity(&mut w, 100.0, 0.0, 0.0, 1.0);
        w.entities[idx].velocity = [1000.0, 0.0];
        velocity_cap(&mut w);
        let speed =
            (w.entities[idx].velocity[0].powi(2) + w.entities[idx].velocity[1].powi(2)).sqrt();
        assert!(
            (speed - constants::MAX_GLOBAL_VELOCITY).abs() < 1e-3,
            "speed={speed}, max={}",
            constants::MAX_GLOBAL_VELOCITY,
        );
    }

    #[test]
    fn velocity_cap_preserves_slow_entity() {
        let mut w = SimWorldFlat::new(0, 0.05);
        let idx = spawn_entity(&mut w, 100.0, 0.0, 0.0, 1.0);
        w.entities[idx].velocity = [3.0, 4.0]; // speed=5, below cap
        velocity_cap(&mut w);
        assert!((w.entities[idx].velocity[0] - 3.0).abs() < 1e-5);
        assert!((w.entities[idx].velocity[1] - 4.0).abs() < 1e-5);
    }

    #[test]
    fn velocity_cap_preserves_direction() {
        let mut w = SimWorldFlat::new(0, 0.05);
        let idx = spawn_entity(&mut w, 100.0, 0.0, 0.0, 1.0);
        w.entities[idx].velocity = [300.0, 400.0]; // direction = (0.6, 0.8)
        velocity_cap(&mut w);
        let vx = w.entities[idx].velocity[0];
        let vy = w.entities[idx].velocity[1];
        let ratio = vx / vy;
        assert!(
            (ratio - 0.75).abs() < 1e-3,
            "direction preserved: ratio={ratio}"
        );
    }

    // ── will_to_velocity ────────────────────────────────────────────────────

    #[test]
    fn will_to_velocity_accelerates_entity() {
        let mut w = SimWorldFlat::new(0, 0.05);
        let idx = spawn_entity(&mut w, 100.0, 0.0, 0.0, 1.0);
        w.entities[idx].will_intent = [1.0, 0.0];
        w.entities[idx].engine_buffer = 10.0;
        w.entities[idx].engine_max = 50.0;
        will_to_velocity(&mut w);
        assert!(w.entities[idx].velocity[0] > 0.0, "should accelerate in x");
    }

    #[test]
    fn will_to_velocity_zero_intent_noop() {
        let mut w = SimWorldFlat::new(0, 0.05);
        let idx = spawn_entity(&mut w, 100.0, 0.0, 0.0, 1.0);
        w.entities[idx].will_intent = [0.0, 0.0];
        will_to_velocity(&mut w);
        assert_eq!(w.entities[idx].velocity[0], 0.0);
        assert_eq!(w.entities[idx].velocity[1], 0.0);
    }

    // ── locomotion_drain ────────────────────────────────────────────────────

    #[test]
    fn locomotion_drain_reduces_energy_for_moving_entity() {
        let mut w = SimWorldFlat::new(0, 0.05);
        let idx = spawn_entity(&mut w, 100.0, 0.0, 0.0, 1.0);
        w.entities[idx].velocity = [10.0, 0.0];
        let before = w.entities[idx].qe;
        locomotion_drain(&mut w);
        assert!(w.entities[idx].qe < before, "moving should cost energy");
    }

    #[test]
    fn locomotion_drain_zero_velocity_no_cost() {
        let mut w = SimWorldFlat::new(0, 0.05);
        spawn_entity(&mut w, 100.0, 0.0, 0.0, 1.0);
        locomotion_drain(&mut w);
        assert_eq!(w.entities[0].qe, 100.0);
    }

    #[test]
    fn locomotion_drain_never_negative() {
        let mut w = SimWorldFlat::new(0, 0.05);
        let idx = spawn_entity(&mut w, 0.02, 0.0, 0.0, 1.0);
        w.entities[idx].velocity = [100.0, 100.0];
        locomotion_drain(&mut w);
        assert!(w.entities[idx].qe >= 0.0);
    }

    // ── entrainment ─────────────────────────────────────────────────────────

    #[test]
    fn entrainment_pulls_frequencies_together() {
        let mut w = SimWorldFlat::new(0, 0.05);
        let a = spawn_entity(&mut w, 100.0, 0.0, 0.0, 1.0);
        let b = spawn_entity(&mut w, 100.0, 2.0, 0.0, 1.0);
        w.entities[a].frequency_hz = 440.0;
        w.entities[b].frequency_hz = 445.0;
        let gap_before = (w.entities[a].frequency_hz - w.entities[b].frequency_hz).abs();
        let mut scratch = ScratchPad::new();
        entrainment(&mut w, &mut scratch);
        let gap_after = (w.entities[a].frequency_hz - w.entities[b].frequency_hz).abs();
        assert!(
            gap_after < gap_before,
            "frequencies should converge: {gap_before} → {gap_after}"
        );
    }

    #[test]
    fn entrainment_distant_no_effect() {
        let mut w = SimWorldFlat::new(0, 0.05);
        let a = spawn_entity(&mut w, 100.0, 0.0, 0.0, 1.0);
        let b = spawn_entity(&mut w, 100.0, 50.0, 0.0, 1.0); // beyond range
        w.entities[a].frequency_hz = 440.0;
        w.entities[b].frequency_hz = 445.0;
        let mut scratch = ScratchPad::new();
        entrainment(&mut w, &mut scratch);
        assert!((w.entities[a].frequency_hz - 440.0).abs() < 1e-5);
    }

    // ── containment_check ───────────────────────────────────────────────────

    #[test]
    fn containment_drags_smaller_entity() {
        let mut w = SimWorldFlat::new(0, 0.05);
        let small = spawn_entity(&mut w, 50.0, 0.0, 0.0, 0.5);
        let big = spawn_entity(&mut w, 200.0, 0.3, 0.0, 2.0); // overlapping
        w.entities[small].velocity = [10.0, 0.0];
        w.entities[big].velocity = [0.0, 0.0];
        let mut scratch = ScratchPad::new();
        containment_check(&mut w, &mut scratch);
        assert!(
            w.entities[small].velocity[0] < 10.0,
            "small entity should be dragged"
        );
    }
}
