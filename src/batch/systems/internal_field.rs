//! Internal energy field system — distributes qe along 8 body-axis nodes
//! and runs diffusion each tick.
//!
//! Tier 1: per-entity, no interaction. SIMD-friendly.
//! Phase: MorphologicalLayer, before growth_inference.

use crate::batch::arena::SimWorldFlat;
use crate::batch::constants::*;
use crate::blueprint::equations::internal_field;

/// Distribute entity qe into internal field, run diffusion + freq entrainment.
///
/// Axiom 6: organ-like peaks emerge from diffusion dynamics, not programming.
/// Axiom 5: conservation — field_total always matches qe after this system.
pub fn internal_diffusion(world: &mut SimWorldFlat) {
    let dt = world.dt;
    let mut mask = world.alive_mask;
    while mask != 0 {
        let i = mask.trailing_zeros() as usize;
        mask &= mask - 1;
        let e = &mut world.entities[i];

        // If field is uninitialized (all zeros but qe > 0), distribute from genome
        let field_sum = internal_field::field_total(&e.qe_field);
        if field_sum < 1e-6 && e.qe > 1e-6 {
            let profile = internal_field::genome_to_profile(
                e.growth_bias, e.resilience, e.branching_bias,
            );
            e.qe_field = internal_field::distribute_to_field(e.qe, &profile);
            // Initialize freq_field from entity frequency with slight variation
            for n in 0..internal_field::NODE_COUNT {
                e.freq_field[n] = e.frequency_hz
                    + (n as f32 - (internal_field::NODE_COUNT as f32 - 1.0) / 2.0)
                    * FREQ_FIELD_SPREAD;
            }
        } else if e.qe > 1e-6 {
            // Sync: if other systems changed qe, rescale field to match
            internal_field::rescale_field(&mut e.qe_field, e.qe);
        }

        // Diffuse energy between adjacent nodes
        e.qe_field = internal_field::field_diffuse(
            &e.qe_field, INTERNAL_DIFFUSION_CONDUCTIVITY, dt,
        );

        // Entrain frequencies between adjacent nodes
        e.freq_field = internal_field::freq_field_entrain(
            &e.freq_field, INTERNAL_FREQ_COUPLING, dt,
        );

        // Update cached qe from field (conservation)
        e.qe = internal_field::field_total(&e.qe_field);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::batch::arena::EntitySlot;

    fn spawn(w: &mut SimWorldFlat, qe: f32) -> usize {
        let mut e = EntitySlot::default();
        e.qe = qe;
        e.radius = 1.0;
        e.dissipation = 0.01;
        e.growth_bias = 0.8;
        e.resilience = 0.5;
        e.branching_bias = 0.3;
        e.frequency_hz = 440.0;
        w.spawn(e).unwrap()
    }

    #[test]
    fn initializes_field_from_qe() {
        let mut w = SimWorldFlat::new(0, 0.05);
        let idx = spawn(&mut w, 100.0);
        assert_eq!(internal_field::field_total(&w.entities[idx].qe_field), 0.0);
        internal_diffusion(&mut w);
        let total = internal_field::field_total(&w.entities[idx].qe_field);
        assert!((total - 100.0).abs() < 1e-3, "field should sum to qe: {total}");
    }

    #[test]
    fn conserves_energy_across_ticks() {
        let mut w = SimWorldFlat::new(0, 0.05);
        spawn(&mut w, 100.0);
        for _ in 0..50 {
            internal_diffusion(&mut w);
        }
        let total = internal_field::field_total(&w.entities[0].qe_field);
        assert!((total - w.entities[0].qe).abs() < 1e-3, "field={total} qe={}", w.entities[0].qe);
        assert!((total - 100.0).abs() < 1e-2, "conservation: {total}");
    }

    #[test]
    fn field_develops_gradient_from_genome() {
        let mut w = SimWorldFlat::new(0, 0.05);
        let idx = spawn(&mut w, 100.0);
        w.entities[idx].growth_bias = 1.0;   // tip emphasis
        w.entities[idx].resilience = 0.0;     // no center emphasis
        internal_diffusion(&mut w);
        // Tips should have more energy than center
        assert!(
            w.entities[idx].qe_field[0] > w.entities[idx].qe_field[3],
            "tips should have more: tip={} center={}",
            w.entities[idx].qe_field[0], w.entities[idx].qe_field[3],
        );
    }

    #[test]
    fn diffusion_smooths_over_time() {
        let mut w = SimWorldFlat::new(0, 0.05);
        let idx = spawn(&mut w, 100.0);
        internal_diffusion(&mut w);
        let variance_before: f32 = {
            let mean = 100.0 / 8.0;
            w.entities[idx].qe_field.iter().map(|v| (v - mean).powi(2)).sum::<f32>() / 8.0
        };
        for _ in 0..200 {
            internal_diffusion(&mut w);
        }
        let variance_after: f32 = {
            let mean = w.entities[idx].qe / 8.0;
            w.entities[idx].qe_field.iter().map(|v| (v - mean).powi(2)).sum::<f32>() / 8.0
        };
        assert!(variance_after < variance_before, "should smooth: {variance_before} → {variance_after}");
    }

    #[test]
    fn skips_dead_entities() {
        let mut w = SimWorldFlat::new(0, 0.05);
        spawn(&mut w, 100.0);
        w.kill(0);
        internal_diffusion(&mut w); // should not panic
    }

    #[test]
    fn freq_field_initialized() {
        let mut w = SimWorldFlat::new(0, 0.05);
        let idx = spawn(&mut w, 100.0);
        internal_diffusion(&mut w);
        assert!(w.entities[idx].freq_field[0] > 0.0, "freq should be initialized");
    }
}
