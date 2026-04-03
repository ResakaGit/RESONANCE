//! GenomeBlob — minimal DNA for batch reproduction and future genetic harness.
//!
//! 20 bytes. Copy. Deterministic. Represents `InferenceProfile` + archetype.

use crate::batch::arena::EntitySlot;
use crate::blueprint::equations::determinism;

/// Minimal genome encoding. Copy + repr(C).
#[derive(Clone, Copy, Debug, PartialEq)]
#[repr(C)]
pub struct GenomeBlob {
    pub archetype: u8,
    pub trophic_class: u8,
    pub growth_bias: f32,
    pub mobility_bias: f32,
    pub branching_bias: f32,
    pub resilience: f32,
    /// Self-adaptive mutation step (Schwefel 1981). Evolves with the genome.
    pub sigma: f32,
}

impl GenomeBlob {
    /// Generate a random genome from a deterministic seed.
    ///
    /// Archetype and trophic_class are **coherent**: archetype determines
    /// the valid trophic range. No "flora + carnivore" chimeras.
    pub fn random(rng_state: u64) -> Self {
        let s0 = determinism::next_u64(rng_state);
        let s1 = determinism::next_u64(s0);
        let s2 = determinism::next_u64(s1);
        let s3 = determinism::next_u64(s2);
        let s4 = determinism::next_u64(s3);

        // Archetype distribution: 30% flora, 40% fauna, 20% cell, 10% virus
        let arch_roll = determinism::unit_f32(s0);
        let archetype = if arch_roll < 0.30 {
            1
        }
        // flora
        else if arch_roll < 0.70 {
            2
        }
        // fauna
        else if arch_roll < 0.90 {
            3
        }
        // cell
        else {
            4
        }; // virus

        // Trophic coherent with archetype
        let trophic_class = match archetype {
            1 => 0, // flora → always primary producer
            2 => {
                // fauna → herbivore(60%), carnivore(30%), omnivore(10%)
                let t = determinism::unit_f32(s1);
                if t < 0.60 {
                    1
                } else if t < 0.90 {
                    3
                } else {
                    2
                }
            }
            3 => {
                // cell → producer(50%), detritivore(50%)
                if determinism::unit_f32(s1) < 0.5 {
                    0
                } else {
                    4
                }
            }
            4 => 3, // virus → always carnivore (parasitic)
            _ => 0,
        };

        Self {
            archetype,
            trophic_class,
            growth_bias: determinism::unit_f32(s2),
            mobility_bias: determinism::unit_f32(s3),
            branching_bias: determinism::unit_f32(s4),
            resilience: determinism::unit_f32(determinism::next_u64(s4)),
            sigma: 0.15,
        }
    }

    /// Extract genome from a live EntitySlot.
    pub fn from_slot(slot: &EntitySlot) -> Self {
        Self {
            archetype: slot.archetype,
            trophic_class: slot.trophic_class,
            growth_bias: slot.growth_bias,
            mobility_bias: slot.mobility_bias,
            branching_bias: slot.branching_bias,
            resilience: slot.resilience,
            sigma: 0.15,
        }
    }

    /// Inject genome into an EntitySlot (overwrites genome fields only).
    pub fn apply(&self, slot: &mut EntitySlot) {
        slot.archetype = self.archetype;
        slot.trophic_class = self.trophic_class;
        slot.growth_bias = self.growth_bias;
        slot.mobility_bias = self.mobility_bias;
        slot.branching_bias = self.branching_bias;
        slot.resilience = self.resilience;
    }

    /// Self-adaptive mutation (Schwefel 1981).
    ///
    /// Sigma mutates first (log-normal), then biases mutate with new sigma.
    /// If `sigma_override > 0`, uses that instead of self.sigma.
    pub fn mutate(&self, rng_state: u64, sigma_override: f32) -> Self {
        use crate::blueprint::equations::batch_fitness;
        let mut g = *self;
        let effective_sigma = if sigma_override > 0.0 {
            sigma_override
        } else {
            g.sigma
        };
        let biases = [
            g.growth_bias,
            g.mobility_bias,
            g.branching_bias,
            g.resilience,
        ];
        let (new_biases, new_sigma) =
            batch_fitness::self_adaptive_mutate(&biases, effective_sigma, rng_state, 0.001, 0.3);
        g.growth_bias = new_biases[0];
        g.mobility_bias = new_biases[1];
        g.branching_bias = new_biases[2];
        g.resilience = new_biases[3];
        g.sigma = new_sigma;
        g
    }

    /// Uniform crossover between two genomes.
    ///
    /// Archetype+trophic inherited as a unit from one parent (no chimeras).
    /// Biases crossed uniformly.
    pub fn crossover(&self, other: &Self, rng_state: u64) -> Self {
        use crate::blueprint::equations::batch_fitness;
        let a = [
            self.growth_bias,
            self.mobility_bias,
            self.branching_bias,
            self.resilience,
        ];
        let b = [
            other.growth_bias,
            other.mobility_bias,
            other.branching_bias,
            other.resilience,
        ];
        let child_biases = batch_fitness::crossover_uniform(&a, &b, rng_state);
        // Inherit archetype+trophic as a coherent pair from one parent
        let (arch, troph) = if determinism::unit_f32(rng_state) < 0.5 {
            (self.archetype, self.trophic_class)
        } else {
            (other.archetype, other.trophic_class)
        };
        Self {
            archetype: arch,
            trophic_class: troph,
            growth_bias: child_biases[0],
            mobility_bias: child_biases[1],
            branching_bias: child_biases[2],
            resilience: child_biases[3],
            sigma: (self.sigma + other.sigma) * 0.5,
        }
    }

    /// Euclidean distance in 4D bias space.
    pub fn distance(&self, other: &Self) -> f32 {
        let dg = self.growth_bias - other.growth_bias;
        let dm = self.mobility_bias - other.mobility_bias;
        let db = self.branching_bias - other.branching_bias;
        let dr = self.resilience - other.resilience;
        (dg * dg + dm * dm + db * db + dr * dr).sqrt()
    }

    /// Deterministic hash for comparison.
    pub fn hash(&self) -> u64 {
        determinism::hash_f32_slice(&[
            self.growth_bias,
            self.mobility_bias,
            self.branching_bias,
            self.resilience,
        ])
    }

    /// Biases as array (for crossover and fitness equations).
    pub fn biases(&self) -> [f32; 4] {
        [
            self.growth_bias,
            self.mobility_bias,
            self.branching_bias,
            self.resilience,
        ]
    }
}

impl Default for GenomeBlob {
    fn default() -> Self {
        Self {
            archetype: 0,
            trophic_class: 0,
            growth_bias: 0.5,
            mobility_bias: 0.5,
            branching_bias: 0.5,
            resilience: 0.5,
            sigma: 0.15,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn from_slot_round_trip() {
        let mut slot = EntitySlot::default();
        slot.archetype = 2;
        slot.trophic_class = 3;
        slot.growth_bias = 0.8;
        slot.mobility_bias = 0.3;
        slot.branching_bias = 0.6;
        slot.resilience = 0.9;
        let g = GenomeBlob::from_slot(&slot);
        let mut slot2 = EntitySlot::default();
        g.apply(&mut slot2);
        assert_eq!(slot2.archetype, 2);
        assert_eq!(slot2.trophic_class, 3);
        assert!((slot2.growth_bias - 0.8).abs() < 1e-5);
        assert!((slot2.resilience - 0.9).abs() < 1e-5);
    }

    #[test]
    fn mutate_self_adaptive_uses_genome_sigma() {
        let g = GenomeBlob::default(); // sigma=0.15
        let m = g.mutate(42, 0.0); // sigma_override=0 → use self.sigma
        // With self-adaptive, genome changes (sigma mutates first)
        assert_ne!(g.sigma, m.sigma, "sigma should self-adapt");
    }

    #[test]
    fn mutate_produces_variation() {
        let g = GenomeBlob::default();
        let m = g.mutate(42, 0.1);
        // At least one bias should differ
        assert!(
            m.growth_bias != g.growth_bias
                || m.mobility_bias != g.mobility_bias
                || m.branching_bias != g.branching_bias
                || m.resilience != g.resilience,
        );
    }

    #[test]
    fn mutate_stays_in_bounds() {
        let g = GenomeBlob {
            growth_bias: 0.99,
            mobility_bias: 0.01,
            ..Default::default()
        };
        for seed in 0..100 {
            let m = g.mutate(seed, 0.5);
            assert!(m.growth_bias >= 0.0 && m.growth_bias <= 1.0);
            assert!(m.mobility_bias >= 0.0 && m.mobility_bias <= 1.0);
        }
    }

    #[test]
    fn mutate_deterministic() {
        let g = GenomeBlob::default();
        assert_eq!(g.mutate(42, 0.1), g.mutate(42, 0.1));
    }

    #[test]
    fn hash_differs_for_different_genomes() {
        let a = GenomeBlob {
            growth_bias: 0.5,
            ..Default::default()
        };
        let b = GenomeBlob {
            growth_bias: 0.6,
            ..Default::default()
        };
        assert_ne!(a.hash(), b.hash());
    }

    #[test]
    fn random_different_seeds_differ() {
        let a = GenomeBlob::random(42);
        let b = GenomeBlob::random(43);
        assert_ne!(a.hash(), b.hash());
    }

    #[test]
    fn random_deterministic() {
        assert_eq!(GenomeBlob::random(42), GenomeBlob::random(42));
    }

    #[test]
    fn random_biases_in_range() {
        for seed in 0..100 {
            let g = GenomeBlob::random(seed);
            assert!(g.growth_bias >= 0.0 && g.growth_bias < 1.0);
            assert!(g.mobility_bias >= 0.0 && g.mobility_bias < 1.0);
            assert!(g.resilience >= 0.0 && g.resilience < 1.0);
        }
    }

    #[test]
    fn crossover_contains_parent_genes() {
        let a = GenomeBlob {
            growth_bias: 0.0,
            mobility_bias: 0.0,
            branching_bias: 0.0,
            resilience: 0.0,
            ..Default::default()
        };
        let b = GenomeBlob {
            growth_bias: 1.0,
            mobility_bias: 1.0,
            branching_bias: 1.0,
            resilience: 1.0,
            ..Default::default()
        };
        let c = a.crossover(&b, 42);
        for bias in c.biases() {
            assert!(
                bias == 0.0 || bias == 1.0,
                "child should have parent genes exactly"
            );
        }
    }

    #[test]
    fn crossover_deterministic() {
        let a = GenomeBlob::random(10);
        let b = GenomeBlob::random(20);
        assert_eq!(a.crossover(&b, 42), a.crossover(&b, 42));
    }

    #[test]
    fn distance_identical_is_zero() {
        let g = GenomeBlob::default();
        assert_eq!(g.distance(&g), 0.0);
    }

    #[test]
    fn distance_symmetric() {
        let a = GenomeBlob::random(1);
        let b = GenomeBlob::random(2);
        assert!((a.distance(&b) - b.distance(&a)).abs() < 1e-5);
    }

    #[test]
    fn distance_max_is_two() {
        let a = GenomeBlob {
            growth_bias: 0.0,
            mobility_bias: 0.0,
            branching_bias: 0.0,
            resilience: 0.0,
            ..Default::default()
        };
        let b = GenomeBlob {
            growth_bias: 1.0,
            mobility_bias: 1.0,
            branching_bias: 1.0,
            resilience: 1.0,
            ..Default::default()
        };
        assert!(
            (a.distance(&b) - 2.0).abs() < 1e-3,
            "max distance = sqrt(4) = 2"
        );
    }
}
