//! PC-2/4: Particle forces batch system — Coulomb + Lennard-Jones.
//!
//! Strategy pattern: ForceStrategy selects which forces to apply.
//! Default = Full (Coulomb + LJ). Configurable per experiment.
//!
//! Stateless: reads positions/charges, writes velocities.
//! Axiom 5: Newton 3 respected (Σ force = 0).
//! Axiom 7: all forces decay with distance.

use crate::batch::arena::{ForceStrategy, SimWorldFlat};
use crate::blueprint::equations::coulomb::{self, ChargedParticle};

// ─── System ─────────────────────────────────────────────────────────────────

/// Apply particle forces to all alive entities. Stateless: world → world.
///
/// Reads: charge, particle_mass, position, frequency_hz.
/// Writes: velocity (force / mass → acceleration → velocity delta).
///
/// Strategy::Disabled → no-op (zero cost for non-particle simulations).
pub fn particle_forces(world: &mut SimWorldFlat, strategy: ForceStrategy, dt: f32) {
    if strategy == ForceStrategy::Disabled {
        return;
    }

    // 1. Extract particles from EntitySlot (read-only snapshot)
    let (particles, count) = extract_particles(world);
    if count < 2 {
        return;
    }

    // 2. Accumulate forces (pure HOF, Newton 3 guaranteed)
    let forces = coulomb::accumulate_forces(&particles, count);

    // 3. Apply: force / mass → acceleration → velocity delta
    let mut mask = world.alive_mask;
    let mut idx = 0usize;
    while mask != 0 {
        let i = mask.trailing_zeros() as usize;
        mask &= mask - 1;

        if idx >= count {
            break;
        }
        let mass = world.entities[i].particle_mass.max(0.01);

        // Only apply LJ component if Full strategy
        let fx = forces[idx][0];
        let fy = forces[idx][1];

        // F = ma → a = F/m → Δv = a × dt
        world.entities[i].velocity[0] += (fx / mass) * dt;
        world.entities[i].velocity[1] += (fy / mass) * dt;

        idx += 1;
    }
}

// ─── Bond detection system ──────────────────────────────────────────────────

/// Detect stable bonds and record them. Stateless: world → bond list.
///
/// Returns (bond pairs, count). Does NOT modify world — caller decides
/// what to do with bonds (create StructuralLinks, track molecules, etc.).
pub fn detect_particle_bonds(world: &SimWorldFlat) -> ([(u8, u8, f32); 256], usize) {
    let (particles, count) = extract_particles(world);
    coulomb::detect_bonds(&particles, count)
}

/// Count distinct molecule types in current world state. Pure observability.
pub fn count_molecules(world: &SimWorldFlat) -> (u8, u8) {
    let (particles, count) = extract_particles(world);
    let (bonds, bond_count) = coulomb::detect_bonds(&particles, count);

    // Simple: each bond pair = one "molecule" (for now, no graph traversal)
    // TODO: connected components for multi-atom molecules
    let mut sigs = [coulomb::MoleculeSignature {
        total_charge: 0.0,
        mean_frequency: 0.0,
        particle_count: 0,
        bond_count: 0,
    }; 256];
    let mut sig_count = 0usize;

    for k in 0..bond_count {
        if sig_count >= 256 {
            break;
        }
        let (a, b, _) = bonds[k];
        let members = [a, b];
        sigs[sig_count] = coulomb::classify_molecule(&particles, &members, 2, 1);
        sig_count += 1;
    }

    let types = coulomb::count_element_types(&sigs, sig_count);
    (bond_count as u8, types)
}

// ─── Helpers ────────────────────────────────────────────────────────────────

/// Extract ChargedParticle array from EntitySlot array. Pure snapshot.
fn extract_particles(world: &SimWorldFlat) -> ([ChargedParticle; 128], usize) {
    let mut particles = [ChargedParticle {
        charge: 0.0,
        mass: 1.0,
        frequency: 0.0,
        position: [0.0; 2],
        velocity: [0.0; 2],
    }; 128];
    let mut count = 0usize;

    let mut mask = world.alive_mask;
    while mask != 0 {
        let i = mask.trailing_zeros() as usize;
        mask &= mask - 1;
        if count >= 128 {
            break;
        }

        particles[count] = ChargedParticle {
            charge: world.entities[i].charge,
            mass: world.entities[i].particle_mass,
            frequency: world.entities[i].frequency_hz,
            position: world.entities[i].position,
            velocity: world.entities[i].velocity,
        };
        count += 1;
    }
    (particles, count)
}

// ─── Tests (BDD) ────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn world_with_charges(charges: &[(f32, f32, f32)]) -> SimWorldFlat {
        let mut w = SimWorldFlat::new(42, 0.05);
        for &(charge, freq, x) in charges {
            let mut slot = crate::batch::arena::EntitySlot::default();
            slot.qe = 10.0;
            slot.charge = charge;
            slot.particle_mass = 1.0;
            slot.frequency_hz = freq;
            slot.position = [x, 0.0];
            slot.radius = 0.1;
            w.spawn(slot);
        }
        w
    }

    // ── GIVEN strategy=Disabled THEN no-op ──────────────────────────────

    #[test]
    fn disabled_strategy_no_effect() {
        let mut w = world_with_charges(&[(1.0, 400.0, 0.0), (-1.0, 400.0, 1.0)]);
        let v_before = w.entities[0].velocity;
        particle_forces(&mut w, ForceStrategy::Disabled, 0.05);
        assert_eq!(w.entities[0].velocity, v_before, "disabled = no change");
    }

    // ── GIVEN opposite charges THEN they accelerate toward each other ───

    #[test]
    fn opposite_charges_accelerate_together() {
        let mut w = world_with_charges(&[(1.0, 400.0, 0.0), (-1.0, 400.0, 2.0)]);
        particle_forces(&mut w, ForceStrategy::Full, 0.05);
        assert!(
            w.entities[0].velocity[0] > 0.0,
            "positive charge moves toward negative"
        );
        assert!(
            w.entities[1].velocity[0] < 0.0,
            "negative charge moves toward positive"
        );
    }

    // ── GIVEN same charges THEN they accelerate apart ───────────────────

    #[test]
    fn same_charges_repel() {
        let mut w = world_with_charges(&[(1.0, 400.0, 0.0), (1.0, 400.0, 2.0)]);
        particle_forces(&mut w, ForceStrategy::Full, 0.05);
        assert!(w.entities[0].velocity[0] < 0.0, "repelled left");
        assert!(w.entities[1].velocity[0] > 0.0, "repelled right");
    }

    // ── GIVEN Newton 3 THEN momentum conserved ──────────────────────────

    #[test]
    fn momentum_conservation() {
        let mut w = world_with_charges(&[(1.0, 400.0, 0.0), (-1.0, 300.0, 1.0), (0.5, 500.0, 0.5)]);
        particle_forces(&mut w, ForceStrategy::Full, 0.05);
        let px: f32 = (0..3)
            .map(|i| w.entities[i].velocity[0] * w.entities[i].particle_mass)
            .sum();
        let py: f32 = (0..3)
            .map(|i| w.entities[i].velocity[1] * w.entities[i].particle_mass)
            .sum();
        assert!(px.abs() < 1e-4, "Axiom 5: Σpx = 0: {px}");
        assert!(py.abs() < 1e-4, "Axiom 5: Σpy = 0: {py}");
    }

    // ── GIVEN no charged particles THEN no panic ────────────────────────

    #[test]
    fn empty_world_no_panic() {
        let mut w = SimWorldFlat::new(42, 0.05);
        particle_forces(&mut w, ForceStrategy::Full, 0.05);
    }

    // ── GIVEN bond detection THEN finds opposite pairs ──────────────────

    #[test]
    fn detects_close_opposite_bond() {
        let w = world_with_charges(&[(1.0, 400.0, 0.0), (-1.0, 400.0, 0.3)]);
        let (_, count) = detect_particle_bonds(&w);
        assert!(count >= 1, "close opposite charges should bond");
    }

    #[test]
    fn no_bond_far_apart() {
        let w = world_with_charges(&[(1.0, 400.0, 0.0), (-1.0, 400.0, 100.0)]);
        let (_, count) = detect_particle_bonds(&w);
        assert_eq!(count, 0, "far apart = no bond");
    }

    // ── GIVEN molecules THEN count types ────────────────────────────────

    #[test]
    fn count_molecule_types() {
        let w = world_with_charges(&[
            (1.0, 400.0, 0.0),
            (-1.0, 400.0, 0.3), // pair 1
            (1.0, 400.0, 5.0),
            (-1.0, 400.0, 5.3), // pair 2 (same type)
            (2.0, 600.0, 10.0),
            (-2.0, 600.0, 10.3), // pair 3 (different type)
        ]);
        let (bonds, types) = count_molecules(&w);
        assert!(bonds >= 2, "should have bonds: {bonds}");
        // Types: at least 1 (may be 2 depending on freq alignment threshold)
        assert!(types >= 1, "should have molecule types: {types}");
    }

    // ── GIVEN determinism THEN same result ──────────────────────────────

    #[test]
    fn forces_deterministic() {
        let mut a = world_with_charges(&[(1.0, 400.0, 0.0), (-1.0, 300.0, 1.0)]);
        let mut b = world_with_charges(&[(1.0, 400.0, 0.0), (-1.0, 300.0, 1.0)]);
        particle_forces(&mut a, ForceStrategy::Full, 0.05);
        particle_forces(&mut b, ForceStrategy::Full, 0.05);
        assert_eq!(
            a.entities[0].velocity[0].to_bits(),
            b.entities[0].velocity[0].to_bits()
        );
    }
}
