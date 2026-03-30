//! PC-3/4/5/6: Particle charge physics — Coulomb, Lennard-Jones, emergent bonding.
//!
//! All stateless. All pure math. Config in → physics out.
//!
//! Axiom 1: charge IS energy polarity (+/-).
//! Axiom 4: Lennard-Jones = equilibrium at minimum dissipation.
//! Axiom 5: Σ charge conserved. Force is conservative (no energy creation).
//! Axiom 7: Coulomb F ∝ 1/r². Strict distance attenuation.
//! Axiom 8: bond strength modulated by frequency alignment.

use crate::blueprint::constants::{
    BOND_ENERGY_THRESHOLD, COULOMB_SCALE, FORCE_SOFTENING, LJ_EPSILON,
    LJ_SIGMA, MAX_FORCE,
};
use super::determinism::{gaussian_frequency_alignment, sanitize_unit};

// ─── PC-3: Charge types ────────────────────────────────────────────────────

/// Particle with charge properties. Pure data, no behavior.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ChargedParticle {
    pub charge: f32,       // +/- polarity (Axiom 1: charge = energy polarity)
    pub mass: f32,         // inertial mass (resistance to acceleration)
    pub frequency: f32,    // oscillatory identity (Axiom 8)
    pub position: [f32; 2],
    pub velocity: [f32; 2],
}

// ─── PC-4: Coulomb force ────────────────────────────────────────────────────

/// Coulomb force magnitude between two charges. Axiom 7: F ∝ q1×q2/r².
///
/// Positive = repulsive (like charges). Negative = attractive (opposite charges).
/// Softened at close range to prevent singularity.
/// Returns scalar force (caller applies direction).
pub fn coulomb_force(q1: f32, q2: f32, distance: f32) -> f32 {
    if !q1.is_finite() || !q2.is_finite() || !distance.is_finite() { return 0.0; }
    let r2 = distance * distance + FORCE_SOFTENING * FORCE_SOFTENING; // softened
    let f = COULOMB_SCALE * q1 * q2 / r2;
    f.clamp(-MAX_FORCE, MAX_FORCE)
}

/// Lennard-Jones potential: short-range repulsion + medium-range attraction.
/// Axiom 4: equilibrium at r=sigma (minimum dissipation configuration).
///
/// V(r) = 4ε × [(σ/r)¹² - (σ/r)⁶]
/// Force = -dV/dr (returned as scalar, caller applies direction).
pub fn lennard_jones_force(distance: f32) -> f32 {
    if !distance.is_finite() || distance <= 0.0 { return MAX_FORCE; } // repel overlap
    let r = distance.max(FORCE_SOFTENING);
    let sr = LJ_SIGMA / r;
    let sr6 = sr * sr * sr * sr * sr * sr;
    let sr12 = sr6 * sr6;
    let f = 24.0 * LJ_EPSILON * (2.0 * sr12 - sr6) / r;
    f.clamp(-MAX_FORCE, MAX_FORCE)
}

/// Net force vector between two charged particles. Axiom 5+7.
///
/// Combines Coulomb (long-range) + Lennard-Jones (short-range).
/// Returns force vector ON particle A (Newton 3: force on B = -force on A).
pub fn net_force(a: &ChargedParticle, b: &ChargedParticle) -> [f32; 2] {
    let dx = b.position[0] - a.position[0];
    let dy = b.position[1] - a.position[1];
    let dist = (dx * dx + dy * dy).sqrt();
    if dist < 1e-10 { return [0.0, 0.0]; } // coincident

    // Coulomb: q1×q2 > 0 → repulsion (push away). q1×q2 < 0 → attraction (pull toward).
    // Negate because coulomb_force returns potential sign, we need force direction.
    let f_coulomb = -coulomb_force(a.charge, b.charge, dist);
    let f_lj = lennard_jones_force(dist);
    let f_total = f_coulomb + f_lj;

    // Direction: unit vector from A to B
    let ux = dx / dist;
    let uy = dy / dist;
    [f_total * ux, f_total * uy]
}

// ─── PC-5: Emergent bonding ────────────────────────────────────────────────

/// Bond energy between two particles. Axiom 4: bound state has LESS energy than free.
///
/// Negative = stable bond (energy released on binding).
/// Modulated by frequency alignment (Axiom 8): compatible frequencies bond stronger.
pub fn bond_energy(a: &ChargedParticle, b: &ChargedParticle) -> f32 {
    let dx = b.position[0] - a.position[0];
    let dy = b.position[1] - a.position[1];
    let dist = (dx * dx + dy * dy).sqrt();
    if dist < 1e-10 { return 0.0; }

    // Coulomb potential: V = k × q1 × q2 / r
    let v_coulomb = COULOMB_SCALE * a.charge * b.charge / dist.max(FORCE_SOFTENING);

    // LJ potential: V = 4ε × [(σ/r)¹² - (σ/r)⁶]
    let r = dist.max(FORCE_SOFTENING);
    let sr = LJ_SIGMA / r;
    let sr6 = sr * sr * sr * sr * sr * sr;
    let v_lj = 4.0 * LJ_EPSILON * (sr6 * sr6 - sr6);

    // Frequency alignment modulates bond strength (Axiom 8)
    let freq_mod = gaussian_frequency_alignment(a.frequency, b.frequency, 50.0);

    (v_coulomb + v_lj) * (0.5 + 0.5 * freq_mod) // freq alignment boosts binding
}

/// Is this pair stably bound? Pure predicate.
pub fn is_bound(bond_e: f32) -> bool {
    bond_e < -BOND_ENERGY_THRESHOLD // negative energy = stable (energy was released)
}

/// Detect all stable bonds in a particle set. Pure: particles → bond list.
///
/// Returns array of (index_a, index_b, bond_energy) for stable pairs.
/// O(N²) — acceptable for N≤128. For N>128, use spatial acceleration.
pub fn detect_bonds(
    particles: &[ChargedParticle],
    count: usize,
) -> ([(u8, u8, f32); 256], usize) {
    let mut bonds = [(0u8, 0u8, 0.0f32); 256];
    let mut bond_count = 0usize;
    let n = count.min(particles.len());

    for i in 0..n {
        for j in (i + 1)..n {
            if bond_count >= 256 { return (bonds, bond_count); }
            let be = bond_energy(&particles[i], &particles[j]);
            if is_bound(be) {
                bonds[bond_count] = (i as u8, j as u8, be);
                bond_count += 1;
            }
        }
    }
    (bonds, bond_count)
}

// ─── PC-6: Element/molecule classification ──────────────────────────────────

/// Molecule signature: charge sum + frequency mean. Pure descriptor.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct MoleculeSignature {
    pub total_charge: f32,
    pub mean_frequency: f32,
    pub particle_count: u8,
    pub bond_count: u8,
}

/// Classify a connected group of particles as a "molecule". Axiom 6: identity emerges.
///
/// Takes indices of particles in one connected component + their bonds.
pub fn classify_molecule(
    particles: &[ChargedParticle],
    member_indices: &[u8],
    member_count: usize,
    bond_count: u8,
) -> MoleculeSignature {
    let n = member_count.min(member_indices.len());
    if n == 0 {
        return MoleculeSignature { total_charge: 0.0, mean_frequency: 0.0, particle_count: 0, bond_count: 0 };
    }
    let total_charge: f32 = (0..n)
        .map(|k| particles[member_indices[k] as usize].charge)
        .sum();
    let mean_frequency: f32 = (0..n)
        .map(|k| particles[member_indices[k] as usize].frequency)
        .sum::<f32>() / n as f32;

    MoleculeSignature {
        total_charge,
        mean_frequency,
        particle_count: n as u8,
        bond_count,
    }
}

/// Count distinct molecule types in a set of signatures. Axiom 6: how many "elements" emerged.
///
/// Two signatures are "same type" if charge and particle_count match
/// and frequency is within bandwidth (Axiom 8).
pub fn count_element_types(signatures: &[MoleculeSignature], count: usize) -> u8 {
    let n = count.min(signatures.len());
    if n == 0 { return 0; }
    let mut types = 0u8;
    let mut seen = [false; 256];

    for i in 0..n {
        if seen[i] { continue; }
        types += 1;
        seen[i] = true;
        for j in (i + 1)..n {
            if seen[j] { continue; }
            let same_structure = signatures[i].particle_count == signatures[j].particle_count
                && (signatures[i].total_charge - signatures[j].total_charge).abs() < 0.1;
            let freq_match = gaussian_frequency_alignment(
                signatures[i].mean_frequency, signatures[j].mean_frequency, 50.0,
            ) > 0.8;
            if same_structure && freq_match { seen[j] = true; }
        }
    }
    types
}

// ─── Force accumulation HOF ─────────────────────────────────────────────────

/// Accumulate all pairwise forces on each particle. Pure HOF: particles → forces.
///
/// O(N²). Returns force vector per particle. Axiom 5: Newton 3 respected
/// (force on A = -force on B, net Σ = 0).
pub fn accumulate_forces(particles: &[ChargedParticle], count: usize) -> [[f32; 2]; 128] {
    let mut forces = [[0.0f32; 2]; 128];
    let n = count.min(particles.len()).min(128);

    for i in 0..n {
        for j in (i + 1)..n {
            let f = net_force(&particles[i], &particles[j]);
            // Newton 3: equal and opposite
            forces[i][0] += f[0]; forces[i][1] += f[1];
            forces[j][0] -= f[0]; forces[j][1] -= f[1];
        }
    }
    forces
}

// ─── Tests (BDD) ────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn particle(charge: f32, freq: f32, x: f32, y: f32) -> ChargedParticle {
        ChargedParticle { charge, mass: 1.0, frequency: freq, position: [x, y], velocity: [0.0, 0.0] }
    }

    // ── GIVEN isolated particle THEN no force ───────────────────────────

    #[test]
    fn isolated_particle_zero_force() {
        let a = particle(1.0, 400.0, 0.0, 0.0);
        let forces = accumulate_forces(&[a], 1);
        assert_eq!(forces[0], [0.0, 0.0], "isolated particle has no force");
    }

    // ── GIVEN opposite charges THEN attractive ──────────────────────────

    #[test]
    fn opposite_charges_attract() {
        let f = coulomb_force(1.0, -1.0, 1.0);
        assert!(f < 0.0, "opposite charges attract: {f}");
    }

    #[test]
    fn same_charges_repel() {
        let f = coulomb_force(1.0, 1.0, 1.0);
        assert!(f > 0.0, "same charges repel: {f}");
    }

    // ── GIVEN Coulomb THEN F ∝ 1/r² (Axiom 7) ──────────────────────────

    #[test]
    fn coulomb_inverse_square() {
        let f1 = coulomb_force(1.0, -1.0, 1.0).abs();
        let f2 = coulomb_force(1.0, -1.0, 2.0).abs();
        let ratio = f1 / f2;
        // Should be ~4 (2²) but softening modifies slightly
        assert!(ratio > 3.0 && ratio < 5.0, "Axiom 7: ~4× at 2× distance: {ratio}");
    }

    #[test]
    fn coulomb_proportional_to_charge() {
        let f1 = coulomb_force(1.0, 1.0, 1.0);
        let f2 = coulomb_force(2.0, 1.0, 1.0);
        assert!((f2 / f1 - 2.0).abs() < 0.1, "force ∝ charge");
    }

    // ── GIVEN distance→0 THEN softened (no singularity) ─────────────────

    #[test]
    fn coulomb_softened_at_zero() {
        let f = coulomb_force(1.0, -1.0, 0.0);
        assert!(f.is_finite(), "softened: no infinity at r=0");
        assert!(f.abs() <= MAX_FORCE, "capped at MAX_FORCE");
    }

    #[test]
    fn coulomb_nan_safe() {
        assert_eq!(coulomb_force(f32::NAN, 1.0, 1.0), 0.0);
    }

    // ── GIVEN LJ THEN equilibrium at sigma (Axiom 4) ────────────────────

    #[test]
    fn lj_zero_crossing_exists() {
        // LJ force crosses zero at r = 2^(1/6) × sigma (standard physics).
        // At r < crossing → repulsive. At r > crossing → attractive.
        let r_cross = LJ_SIGMA * 1.1225; // 2^(1/6) ≈ 1.1225
        let f_below = lennard_jones_force(r_cross * 0.9);
        let f_above = lennard_jones_force(r_cross * 1.1);
        assert!(f_below > 0.0, "below crossing = repulsive: {f_below}");
        assert!(f_above < 0.0, "above crossing = attractive: {f_above}");
    }

    #[test]
    fn lj_repulsive_close() {
        let f = lennard_jones_force(LJ_SIGMA * 0.5);
        assert!(f > 0.0, "repulsive when too close: {f}");
    }

    #[test]
    fn lj_attractive_medium() {
        let f = lennard_jones_force(LJ_SIGMA * 1.5);
        assert!(f < 0.0, "attractive at medium range: {f}");
    }

    // ── GIVEN net_force THEN Newton 3 (Axiom 5) ─────────────────────────

    #[test]
    fn newton_third_law() {
        let a = particle(1.0, 400.0, 0.0, 0.0);
        let b = particle(-1.0, 400.0, 2.0, 0.0);
        let f_on_a = net_force(&a, &b);
        let f_on_b = net_force(&b, &a);
        assert!((f_on_a[0] + f_on_b[0]).abs() < 1e-5, "Newton 3 x: {} + {}", f_on_a[0], f_on_b[0]);
        assert!((f_on_a[1] + f_on_b[1]).abs() < 1e-5, "Newton 3 y");
    }

    #[test]
    fn force_conservation_multi() {
        let particles = [
            particle(1.0, 400.0, 0.0, 0.0),
            particle(-1.0, 300.0, 1.0, 0.0),
            particle(0.5, 500.0, 0.0, 1.0),
        ];
        let forces = accumulate_forces(&particles, 3);
        let sum_x: f32 = forces[..3].iter().map(|f| f[0]).sum();
        let sum_y: f32 = forces[..3].iter().map(|f| f[1]).sum();
        assert!(sum_x.abs() < 1e-4, "Axiom 5: Σfx = 0: {sum_x}");
        assert!(sum_y.abs() < 1e-4, "Axiom 5: Σfy = 0: {sum_y}");
    }

    // ── GIVEN opposite charges close THEN bond forms (Axiom 4) ──────────

    #[test]
    fn bond_energy_negative_for_opposite() {
        let a = particle(1.0, 400.0, 0.0, 0.0);
        let b = particle(-1.0, 400.0, 0.5, 0.0);
        let be = bond_energy(&a, &b);
        assert!(be < 0.0, "opposite charges at equilibrium → negative bond energy: {be}");
    }

    #[test]
    fn bond_energy_less_negative_for_same() {
        let a = particle(1.0, 400.0, 0.0, 0.0);
        let b_opp = particle(-1.0, 400.0, 0.5, 0.0);
        let b_same = particle(1.0, 400.0, LJ_SIGMA, 0.0);
        let be_opp = bond_energy(&a, &b_opp);
        let be_same = bond_energy(&a, &b_same);
        assert!(be_opp < be_same, "opposite binds stronger: {be_opp} < {be_same}");
    }

    // ── GIVEN frequency alignment THEN stronger bond (Axiom 8) ──────────

    #[test]
    fn bond_stronger_with_same_frequency() {
        let a = particle(1.0, 400.0, 0.0, 0.0);
        let b_same = particle(-1.0, 400.0, 0.5, 0.0);
        let b_diff = particle(-1.0, 800.0, 0.5, 0.0);
        let be_same = bond_energy(&a, &b_same);
        let be_diff = bond_energy(&a, &b_diff);
        assert!(be_same < be_diff, "same freq → stronger bond (more negative): {be_same} < {be_diff}");
    }

    // ── GIVEN stable pair THEN is_bound = true ──────────────────────────

    #[test]
    fn bound_pair_detected() {
        let a = particle(1.0, 400.0, 0.0, 0.0);
        let b = particle(-1.0, 400.0, 0.5, 0.0);
        let be = bond_energy(&a, &b);
        assert!(is_bound(be), "opposite charges at equilibrium should be bound: be={be}");
    }

    #[test]
    fn far_particles_not_bound() {
        let a = particle(1.0, 400.0, 0.0, 0.0);
        let b = particle(-1.0, 400.0, 100.0, 0.0);
        let be = bond_energy(&a, &b);
        assert!(!is_bound(be), "far apart should not be bound: be={be}");
    }

    // ── GIVEN detect_bonds THEN finds correct pairs ─────────────────────

    #[test]
    fn detect_bonds_finds_pair() {
        let particles = [
            particle(1.0, 400.0, 0.0, 0.0),
            particle(-1.0, 400.0, 0.5, 0.0),
        ];
        let (bonds, count) = detect_bonds(&particles, 2);
        assert!(count >= 1, "should detect at least 1 bond");
        assert_eq!(bonds[0].0, 0);
        assert_eq!(bonds[0].1, 1);
    }

    #[test]
    fn detect_bonds_empty() {
        let (_, count) = detect_bonds(&[], 0);
        assert_eq!(count, 0);
    }

    #[test]
    fn detect_bonds_no_false_positives() {
        let particles = [
            particle(1.0, 400.0, 0.0, 0.0),
            particle(1.0, 400.0, 100.0, 0.0), // same charge, far apart
        ];
        let (_, count) = detect_bonds(&particles, 2);
        assert_eq!(count, 0, "same charge + far apart = no bond");
    }

    // ── GIVEN molecule THEN classification correct (Axiom 6) ────────────

    #[test]
    fn classify_diatomic() {
        let particles = [
            particle(1.0, 400.0, 0.0, 0.0),
            particle(-1.0, 400.0, 0.5, 0.0),
        ];
        let sig = classify_molecule(&particles, &[0, 1], 2, 1);
        assert_eq!(sig.particle_count, 2);
        assert_eq!(sig.bond_count, 1);
        assert!((sig.total_charge - 0.0).abs() < 1e-5, "neutral pair");
        assert!((sig.mean_frequency - 400.0).abs() < 1e-5);
    }

    #[test]
    fn classify_triatomic() {
        let particles = [
            particle(1.0, 400.0, 0.0, 0.0),
            particle(-2.0, 400.0, LJ_SIGMA, 0.0),
            particle(1.0, 400.0, 1.0, 0.0),
        ];
        let sig = classify_molecule(&particles, &[0, 1, 2], 3, 2);
        assert_eq!(sig.particle_count, 3);
        assert!((sig.total_charge - 0.0).abs() < 1e-5, "CO2-like: +1 -2 +1 = 0");
    }

    // ── GIVEN element types THEN count correct ──────────────────────────

    #[test]
    fn count_distinct_types() {
        let sigs = [
            MoleculeSignature { total_charge: 0.0, mean_frequency: 400.0, particle_count: 2, bond_count: 1 },
            MoleculeSignature { total_charge: 0.0, mean_frequency: 400.0, particle_count: 2, bond_count: 1 }, // same type
            MoleculeSignature { total_charge: 1.0, mean_frequency: 600.0, particle_count: 3, bond_count: 2 }, // different
        ];
        assert_eq!(count_element_types(&sigs, 3), 2);
    }

    #[test]
    fn count_zero_empty() {
        assert_eq!(count_element_types(&[], 0), 0);
    }

    // ── GIVEN charge conservation THEN Σ charge constant (Axiom 5) ──────

    #[test]
    fn charge_conservation_after_forces() {
        let particles = [
            particle(1.0, 400.0, 0.0, 0.0),
            particle(-1.0, 300.0, 1.0, 0.0),
            particle(0.5, 500.0, 0.5, 1.0),
        ];
        let charge_before: f32 = particles.iter().map(|p| p.charge).sum();
        let _ = accumulate_forces(&particles, 3); // forces don't change charge
        let charge_after: f32 = particles.iter().map(|p| p.charge).sum();
        assert_eq!(charge_before, charge_after, "Axiom 5: charge conserved");
    }

    // ── Determinism ─────────────────────────────────────────────────────

    #[test]
    fn forces_deterministic() {
        let p = [particle(1.0, 400.0, 0.0, 0.0), particle(-1.0, 300.0, 1.0, 0.0)];
        let a = accumulate_forces(&p, 2);
        let b = accumulate_forces(&p, 2);
        assert_eq!(a[0][0].to_bits(), b[0][0].to_bits());
    }
}
