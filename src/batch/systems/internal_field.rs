//! Internal energy field system — 2D radial diffusion for emergent morphology.
//!
//! Tier 1: per-entity, no interaction. SIMD-friendly.
//! Phase: MorphologicalLayer, before growth_inference.

use crate::batch::arena::SimWorldFlat;
use crate::batch::constants::*;
use crate::blueprint::equations::radial_field;

/// Distribute entity qe into 2D radial field, run diffusion + freq entrainment.
///
/// Axiom 6: bilateral peaks emerge from isotropic init + diffusion.
/// Axiom 5: conservation — radial_total always matches qe.
pub fn internal_diffusion(world: &mut SimWorldFlat) {
    let dt = world.dt;
    let mut mask = world.alive_mask;
    while mask != 0 {
        let i = mask.trailing_zeros() as usize;
        mask &= mask - 1;
        let e = &mut world.entities[i];

        // If field uninitialized, distribute from genome (isotropic → bilateral emerges)
        let field_sum = radial_field::radial_total(&e.qe_field);
        if field_sum < 1e-6 && e.qe > 1e-6 {
            e.qe_field = radial_field::distribute_to_radial(
                e.qe, e.growth_bias, e.resilience, e.branching_bias,
            );
            // Initialize freq_field with slight axial + radial variation
            for a in 0..radial_field::AXIAL {
                for r in 0..radial_field::RADIAL {
                    e.freq_field[a][r] = e.frequency_hz
                        + (a as f32 - (radial_field::AXIAL as f32 - 1.0) / 2.0) * FREQ_FIELD_SPREAD
                        + (r as f32 - (radial_field::RADIAL as f32 - 1.0) / 2.0) * FREQ_FIELD_SPREAD * 0.5;
                }
            }
        } else if e.qe > 1e-6 {
            radial_field::radial_rescale(&mut e.qe_field, e.qe);
        }

        // 2D diffusion (axial + radial neighbors)
        e.qe_field = radial_field::radial_diffuse(
            &e.qe_field, INTERNAL_DIFFUSION_CONDUCTIVITY, dt,
        );

        // 2D frequency entrainment
        e.freq_field = radial_field::radial_freq_entrain(
            &e.freq_field, INTERNAL_FREQ_COUPLING, dt,
        );

        // Conservation: update cached qe from field
        e.qe = radial_field::radial_total(&e.qe_field);
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
        e.branching_bias = 0.6;
        e.frequency_hz = 440.0;
        w.spawn(e).unwrap()
    }

    #[test]
    fn initializes_2d_field_from_qe() {
        let mut w = SimWorldFlat::new(0, 0.05);
        spawn(&mut w, 100.0);
        internal_diffusion(&mut w);
        let total = radial_field::radial_total(&w.entities[0].qe_field);
        assert!((total - 100.0).abs() < 1e-2, "field should sum to qe: {total}");
    }

    #[test]
    fn conserves_energy_across_ticks() {
        let mut w = SimWorldFlat::new(0, 0.05);
        spawn(&mut w, 100.0);
        for _ in 0..50 {
            internal_diffusion(&mut w);
        }
        let total = radial_field::radial_total(&w.entities[0].qe_field);
        assert!((total - w.entities[0].qe).abs() < 1e-2);
        assert!((total - 100.0).abs() < 1e-1, "conservation: {total}");
    }

    #[test]
    fn bilateral_symmetry_emerges() {
        let mut w = SimWorldFlat::new(0, 0.05);
        let idx = spawn(&mut w, 100.0);
        w.entities[idx].branching_bias = 1.0; // strong lateral emphasis
        internal_diffusion(&mut w);
        // Sectors 1 (right) and 3 (left) should be equal (isotropic init)
        for a in 0..radial_field::AXIAL {
            let right = w.entities[idx].qe_field[a][1];
            let left = w.entities[idx].qe_field[a][3];
            assert!((right - left).abs() < 1e-2,
                "station {a}: right={right} left={left} — bilateral should be symmetric");
        }
    }

    #[test]
    fn lateral_peaks_form_with_branching() {
        let mut w = SimWorldFlat::new(0, 0.05);
        let idx = spawn(&mut w, 100.0);
        w.entities[idx].branching_bias = 1.0;
        internal_diffusion(&mut w);
        // Lateral sectors (1,3) should have more energy than dorsal/ventral (0,2)
        let lateral: f32 = (0..radial_field::AXIAL)
            .map(|a| w.entities[idx].qe_field[a][1] + w.entities[idx].qe_field[a][3])
            .sum();
        let dv: f32 = (0..radial_field::AXIAL)
            .map(|a| w.entities[idx].qe_field[a][0] + w.entities[idx].qe_field[a][2])
            .sum();
        assert!(lateral > dv, "lateral={lateral} should exceed dorsal/ventral={dv}");
    }

    #[test]
    fn skips_dead_entities() {
        let mut w = SimWorldFlat::new(0, 0.05);
        spawn(&mut w, 100.0);
        w.kill(0);
        internal_diffusion(&mut w); // no panic
    }

    #[test]
    fn freq_field_initialized_2d() {
        let mut w = SimWorldFlat::new(0, 0.05);
        let idx = spawn(&mut w, 100.0);
        internal_diffusion(&mut w);
        assert!(w.entities[idx].freq_field[0][0] > 0.0);
        // Different axial stations should have different freq
        assert_ne!(
            w.entities[idx].freq_field[0][0].to_bits(),
            w.entities[idx].freq_field[4][0].to_bits(),
        );
    }
}
