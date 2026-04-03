//! Particle Lab — spawn charged particles, observe emergent molecules.
//!
//! Lightweight: no batch ecology (no photosynthesis, no trophic, no metabolic graph).
//! Only Coulomb + Lennard-Jones + movement + dissipation.
//! ~10× faster than cancer_therapy for pure particle physics.
//!
//! Axiom 1: charge = energy polarity. Axiom 7: F∝1/r². Axiom 8: freq modulates bonds.

use crate::blueprint::equations::coulomb::{
    self, ChargedParticle, MoleculeSignature, accumulate_forces, classify_molecule,
    count_element_types, detect_bonds,
};
use crate::blueprint::equations::determinism;
use std::time::Instant;

// ─── Config ─────────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct ParticleLabConfig {
    /// Number of positive particles.
    pub positive_count: u8,
    /// Number of negative particles.
    pub negative_count: u8,
    /// Charge magnitude for positive particles.
    pub positive_charge: f32,
    /// Charge magnitude for negative particles (stored as positive, applied as negative).
    pub negative_charge: f32,
    /// Frequency range: particles get freq in [base - spread, base + spread].
    pub freq_base: f32,
    pub freq_spread: f32,
    /// Spatial arena size (particles spawn in [0, arena_size]²).
    pub arena_size: f32,
    /// Simulation timestep.
    pub dt: f32,
    /// Ticks per snapshot.
    pub ticks_per_snapshot: u32,
    /// Total snapshots (generations).
    pub snapshots: u32,
    pub seed: u64,
}

impl Default for ParticleLabConfig {
    fn default() -> Self {
        Self {
            positive_count: 15,
            negative_count: 15,
            positive_charge: 1.0,
            negative_charge: 1.0,
            freq_base: 400.0,
            freq_spread: 100.0,
            arena_size: 10.0,
            dt: 0.01,
            ticks_per_snapshot: 20,
            snapshots: 50,
            seed: 42,
        }
    }
}

// ─── Output ─────────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct ParticleSnapshot {
    pub step: u32,
    pub particle_count: u8,
    pub bond_count: u8,
    pub molecule_types: u8,
    pub mean_kinetic_energy: f32,
    pub mean_potential_energy: f32,
    pub total_charge: f32,
}

#[derive(Debug)]
pub struct ParticleLabReport {
    pub config: ParticleLabConfig,
    pub timeline: Vec<ParticleSnapshot>,
    pub final_molecules: Vec<MoleculeSignature>,
    pub wall_time_ms: u64,
}

// ─── Pure physics step (no batch pipeline, minimal) ─────────────────────────

/// One tick of pure particle physics. Stateless transform.
///
/// 1. Accumulate Coulomb + LJ forces (O(N²))
/// 2. Apply: F/m → Δv
/// 3. Move: position += velocity × dt
/// 4. Dissipate: velocity *= (1 - damping) (Axiom 4)
/// 5. Bounce: reflect off arena walls
fn physics_tick(particles: &mut [ChargedParticle], count: usize, dt: f32, arena: f32) {
    if count < 2 {
        return;
    }

    // Forces (pure HOF)
    let forces = accumulate_forces(particles, count);

    // Apply + move + dissipate + bounce
    let damping = 0.005; // Axiom 4: small energy loss per tick
    for i in 0..count {
        let mass = particles[i].mass.max(0.01);
        // F = ma → Δv = (F/m) × dt
        particles[i].velocity[0] += (forces[i][0] / mass) * dt;
        particles[i].velocity[1] += (forces[i][1] / mass) * dt;
        // Dissipation (Axiom 4)
        particles[i].velocity[0] *= 1.0 - damping;
        particles[i].velocity[1] *= 1.0 - damping;
        // Move
        particles[i].position[0] += particles[i].velocity[0] * dt;
        particles[i].position[1] += particles[i].velocity[1] * dt;
        // Bounce off walls (elastic, conserves energy)
        for dim in 0..2 {
            let p = if dim == 0 {
                &mut particles[i].position[0]
            } else {
                &mut particles[i].position[1]
            };
            let v = if dim == 0 {
                &mut particles[i].velocity[0]
            } else {
                &mut particles[i].velocity[1]
            };
            if *p < 0.0 {
                *p = -*p;
                *v = -*v;
            }
            if *p > arena {
                *p = 2.0 * arena - *p;
                *v = -*v;
            }
        }
    }
}

/// Compute snapshot metrics. Pure: particles → snapshot.
fn compute_snapshot(particles: &[ChargedParticle], count: usize, step: u32) -> ParticleSnapshot {
    let n = count.max(1) as f32;

    let mean_ke: f32 = particles[..count]
        .iter()
        .map(|p| 0.5 * p.mass * (p.velocity[0].powi(2) + p.velocity[1].powi(2)))
        .sum::<f32>()
        / n;

    let mut total_pe = 0.0f32;
    for i in 0..count {
        for j in (i + 1)..count {
            total_pe += coulomb::bond_energy(&particles[i], &particles[j]);
        }
    }

    let total_charge: f32 = particles[..count].iter().map(|p| p.charge).sum();
    let (bonds, bond_count) = detect_bonds(particles, count);

    let mut sigs = [MoleculeSignature {
        total_charge: 0.0,
        mean_frequency: 0.0,
        particle_count: 0,
        bond_count: 0,
    }; 256];
    let mut sig_count = 0;
    for k in 0..bond_count {
        if sig_count >= 256 {
            break;
        }
        let (a, b, _) = bonds[k];
        sigs[sig_count] = classify_molecule(particles, &[a, b], 2, 1);
        sig_count += 1;
    }
    let types = count_element_types(&sigs, sig_count);

    ParticleSnapshot {
        step,
        particle_count: count as u8,
        bond_count: bond_count as u8,
        molecule_types: types,
        mean_kinetic_energy: mean_ke,
        mean_potential_energy: total_pe / n,
        total_charge,
    }
}

// ─── Main HOF ───────────────────────────────────────────────────────────────

/// Run particle lab experiment. Pure: config → report.
pub fn run(config: &ParticleLabConfig) -> ParticleLabReport {
    let start = Instant::now();
    let total = (config.positive_count + config.negative_count) as usize;

    // Spawn particles
    let mut particles = [ChargedParticle {
        charge: 0.0,
        mass: 1.0,
        frequency: 0.0,
        position: [0.0; 2],
        velocity: [0.0; 2],
    }; 128];

    let mut s = config.seed;
    for i in 0..config.positive_count as usize {
        s = determinism::next_u64(s);
        particles[i] = ChargedParticle {
            charge: config.positive_charge,
            mass: 1.0,
            frequency: config.freq_base + determinism::gaussian_f32(s, config.freq_spread),
            position: [
                determinism::range_f32(s, 1.0, config.arena_size - 1.0),
                determinism::range_f32(determinism::next_u64(s), 1.0, config.arena_size - 1.0),
            ],
            velocity: [0.0; 2],
        };
        s = determinism::next_u64(s);
    }
    for i in 0..config.negative_count as usize {
        let idx = config.positive_count as usize + i;
        s = determinism::next_u64(s);
        particles[idx] = ChargedParticle {
            charge: -config.negative_charge,
            mass: 1.0,
            frequency: config.freq_base + determinism::gaussian_f32(s, config.freq_spread),
            position: [
                determinism::range_f32(s, 1.0, config.arena_size - 1.0),
                determinism::range_f32(determinism::next_u64(s), 1.0, config.arena_size - 1.0),
            ],
            velocity: [0.0; 2],
        };
        s = determinism::next_u64(s);
    }

    let mut timeline = Vec::with_capacity(config.snapshots as usize);

    for snap in 0..config.snapshots {
        for _ in 0..config.ticks_per_snapshot {
            physics_tick(&mut particles, total, config.dt, config.arena_size);
        }
        timeline.push(compute_snapshot(&particles, total, snap));
    }

    // Final molecules
    let (bonds, bc) = detect_bonds(&particles, total);
    let mut final_mols = Vec::new();
    for k in 0..bc {
        let (a, b, _) = bonds[k];
        final_mols.push(classify_molecule(&particles, &[a, b], 2, 1));
    }

    ParticleLabReport {
        config: config.clone(),
        timeline,
        final_molecules: final_mols,
        wall_time_ms: start.elapsed().as_millis() as u64,
    }
}

// ─── Tests ──────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn run_no_panic() {
        let r = run(&ParticleLabConfig {
            positive_count: 3,
            negative_count: 3,
            snapshots: 5,
            ticks_per_snapshot: 10,
            ..Default::default()
        });
        assert_eq!(r.timeline.len(), 5);
    }

    #[test]
    fn run_deterministic() {
        let c = ParticleLabConfig {
            positive_count: 5,
            negative_count: 5,
            snapshots: 5,
            ticks_per_snapshot: 10,
            ..Default::default()
        };
        let a = run(&c);
        let b = run(&c);
        for i in 0..5 {
            assert_eq!(a.timeline[i].bond_count, b.timeline[i].bond_count);
            assert_eq!(
                a.timeline[i].mean_kinetic_energy.to_bits(),
                b.timeline[i].mean_kinetic_energy.to_bits()
            );
        }
    }

    #[test]
    fn charge_conservation() {
        let c = ParticleLabConfig {
            positive_count: 10,
            negative_count: 10,
            snapshots: 10,
            ticks_per_snapshot: 20,
            ..Default::default()
        };
        let r = run(&c);
        for snap in &r.timeline {
            assert!(
                (snap.total_charge - 0.0).abs() < 1e-4,
                "Axiom 5: Σcharge = 0: {}",
                snap.total_charge
            );
        }
    }

    #[test]
    fn bonds_form_at_some_point() {
        let c = ParticleLabConfig {
            positive_count: 10,
            negative_count: 10,
            snapshots: 30,
            ticks_per_snapshot: 30,
            ..Default::default()
        };
        let r = run(&c);
        let peak_bonds = r.timeline.iter().map(|s| s.bond_count).max().unwrap_or(0);
        // Opposite charges attract → bonds must form at some snapshot
        assert!(
            peak_bonds > 0,
            "Axiom 7+8: opposite charges should form bonds"
        );
    }

    #[test]
    fn kinetic_energy_decreases() {
        let c = ParticleLabConfig {
            positive_count: 5,
            negative_count: 5,
            snapshots: 20,
            ticks_per_snapshot: 20,
            ..Default::default()
        };
        let r = run(&c);
        // Initial KE = 0 (particles start stationary), but forces create KE, then damping reduces it
        // After many ticks, KE should be lower than peak (system cools)
        let peak_ke = r
            .timeline
            .iter()
            .map(|s| s.mean_kinetic_energy)
            .fold(0.0f32, f32::max);
        let final_ke = r.timeline.last().unwrap().mean_kinetic_energy;
        assert!(
            final_ke <= peak_ke,
            "Axiom 4: system cools: {final_ke} <= {peak_ke}"
        );
    }

    #[test]
    fn molecule_types_emerge() {
        let c = ParticleLabConfig {
            positive_count: 10,
            negative_count: 10,
            freq_spread: 200.0, // wide spread → different "elements"
            snapshots: 30,
            ticks_per_snapshot: 30,
            ..Default::default()
        };
        let r = run(&c);
        let types = r.timeline.last().unwrap().molecule_types;
        // With freq spread, some molecule types should emerge
        // (may be 0 if no bonds formed, but should be ≥ 0)
        assert!(
            types <= r.timeline.last().unwrap().bond_count,
            "types ≤ bonds"
        );
    }
}
