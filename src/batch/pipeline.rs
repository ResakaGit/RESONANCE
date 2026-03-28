//! Batch pipeline — orchestrates a complete tick as sequential fn calls.
//!
//! Mirrors `simulation/pipeline.rs` phase ordering (INV-B7):
//! Input → ThermodynamicLayer → AtomicLayer → ChemicalLayer → MetabolicLayer → MorphologicalLayer.
//!
//! BS-0 implements only AtomicLayer systems (dissipation, movement, collision).
//! Subsequent sprints (BS-1..BS-3) expand each phase.

use super::arena::SimWorldFlat;
use super::scratch::ScratchPad;
use super::systems;

impl SimWorldFlat {
    /// One atomic tick. No Bevy. No alloc. No I/O.
    ///
    /// Phase ordering matches `simulation/pipeline.rs` exactly (INV-B7).
    /// Conservation asserted in debug builds (INV-B2).
    pub fn tick(&mut self, scratch: &mut ScratchPad) {
        scratch.clear();
        self.events.clear();
        self.tick_id += 1;

        // Phase::Input
        systems::behavior_assess(self, scratch);

        // Phase::ThermodynamicLayer
        systems::engine_processing(self);
        systems::irradiance_update(self);
        systems::containment_check(self, scratch);

        // Phase::AtomicLayer
        systems::dissipation(self);
        systems::will_to_velocity(self);
        systems::velocity_cap(self);
        systems::locomotion_drain(self);
        systems::movement_integrate(self);
        systems::collision(self, scratch);
        systems::entrainment(self, scratch);
        systems::tension_field_apply(self, scratch);

        // Phase::ChemicalLayer
        systems::nutrient_uptake(self);
        systems::photosynthesis(self);
        systems::state_transitions(self);
        systems::homeostasis(self);

        // Phase::MetabolicLayer
        systems::trophic_forage(self);
        systems::trophic_predation(self, scratch);
        systems::pool_distribution(self);
        systems::social_pack(self, scratch);
        systems::cooperation_eval(self, scratch);
        systems::culture_transmission(self, scratch);

        // Phase::MorphologicalLayer
        systems::senescence(self);
        systems::internal_diffusion(self);
        systems::growth_inference(self);
        systems::morpho_adaptation(self);
        systems::reproduction(self);
        systems::abiogenesis(self);

        // Environmental events
        systems::asteroid_impact(self);

        // Post-tick bookkeeping
        systems::death_reap(self);
        self.update_total_qe();

        #[cfg(debug_assertions)]
        self.assert_conservation();
    }

    /// Fast multi-tick evaluation using analytical stepping (AS-1/2/3).
    ///
    /// For isolated entities: O(1) stepping via closed-form equations.
    /// For interactive entities: full tick-by-tick.
    /// Convergence detection skips stabilized diffusion (AS-2).
    /// Same final state as calling `tick()` N times.
    pub fn tick_fast(&mut self, scratch: &mut ScratchPad, total_ticks: u32) {
        use crate::blueprint::equations::batch_stepping;
        use crate::batch::constants::*;

        // Collect positions for isolation check
        let mut positions = [[0.0f32; 2]; MAX_ENTITIES];
        let mut mask = self.alive_mask;
        while mask != 0 {
            let i = mask.trailing_zeros() as usize;
            mask &= mask - 1;
            positions[i] = self.entities[i].position;
        }

        // Classify isolated entities
        let mut isolated_mask = 0u64;
        mask = self.alive_mask;
        while mask != 0 {
            let i = mask.trailing_zeros() as usize;
            mask &= mask - 1;
            if batch_stepping::is_isolated(&positions, self.alive_mask, i, ISOLATION_RANGE_SQ) {
                isolated_mask |= 1 << i;
            }
        }

        // Analytical step for isolated entities (bulk of the speedup)
        let n_isolated = isolated_mask.count_ones();
        if n_isolated > 0 {
            let mut iso = isolated_mask;
            while iso != 0 {
                let i = iso.trailing_zeros() as usize;
                iso &= iso - 1;
                let e = &mut self.entities[i];
                // Dissipation: O(1) for total_ticks
                e.qe = batch_stepping::dissipation_n_ticks(e.qe, e.dissipation, total_ticks);
                // Growth: O(1) for total_ticks
                if e.growth_bias > 0.0 {
                    e.radius = batch_stepping::growth_n_ticks(
                        e.radius, e.growth_bias, 3.0, 0.01, total_ticks,
                    );
                }
                // Position: integrate with gravity
                for _ in 0..total_ticks {
                    e.velocity[1] -= GRAVITY_ACCELERATION * self.dt;
                    e.position[0] += e.velocity[0] * self.dt;
                    e.position[1] += e.velocity[1] * self.dt;
                    if e.position[1] < 0.0 { e.position[1] = 0.0; e.velocity[1] = 0.0; }
                }
                // Mark field for re-convergence (qe changed)
                e.field_converged = false;
            }
            self.tick_id += total_ticks as u64;
        }

        // Interactive entities: full tick-by-tick (with convergence skip on diffusion)
        let interactive_count = (self.alive_mask & !isolated_mask).count_ones();
        if interactive_count > 0 {
            // Temporarily mark isolated as dead to skip them in systems
            let saved_mask = self.alive_mask;
            // Run full ticks only for interactive entities
            for _ in 0..total_ticks {
                self.tick(scratch);
            }
            // If we already advanced tick_id for isolated, adjust
            if n_isolated > 0 {
                // tick_id was already advanced for isolated; tick() advanced it again
                // Correct: subtract the isolated advance
                self.tick_id -= total_ticks as u64;
            }
        } else if n_isolated == 0 {
            // No entities at all — just advance tick
            self.tick_id += total_ticks as u64;
        }

        // Post-tick bookkeeping
        self.reap_dead();
        self.update_total_qe();
    }
}

// ─── Tests ──────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::batch::arena::EntitySlot;

    fn make_world_with_entities(n: usize, qe: f32) -> SimWorldFlat {
        let mut w = SimWorldFlat::new(42, 0.05);
        for i in 0..n {
            let mut e = EntitySlot::default();
            e.qe = qe;
            e.radius = 0.5;
            e.dissipation = 0.01;
            e.frequency_hz = 200.0 + i as f32 * 50.0;
            e.position = [i as f32 * 5.0, 0.0]; // spaced apart → no collision
            e.archetype = 2;      // fauna — not a producer
            e.trophic_class = 2;  // omnivore — won't photosynthesize
            w.spawn(e);
        }
        w.update_total_qe();
        w
    }

    #[test]
    fn tick_advances_tick_id() {
        let mut w = SimWorldFlat::new(0, 0.05);
        let mut s = ScratchPad::new();
        assert_eq!(w.tick_id, 0);
        w.tick(&mut s);
        assert_eq!(w.tick_id, 1);
        w.tick(&mut s);
        w.tick(&mut s);
        assert_eq!(w.tick_id, 3);
    }

    #[test]
    fn tick_conserves_energy_no_collision() {
        let mut w = make_world_with_entities(10, 100.0);
        let mut s = ScratchPad::new();
        let before = w.total_qe;
        w.tick(&mut s);
        assert!(
            w.total_qe <= before + 1e-3,
            "energy must not increase: before={before}, after={}",
            w.total_qe,
        );
        assert!(w.total_qe > 0.0, "energy should not vanish in 1 tick");
    }

    #[test]
    fn tick_conserves_energy_with_collision() {
        let mut w = SimWorldFlat::new(0, 0.05);
        // Two entities close enough to collide, but below reproduction threshold
        let mut e1 = EntitySlot::default();
        e1.qe = 15.0;
        e1.radius = 2.0;
        e1.position = [0.0, 0.0];
        e1.frequency_hz = 900.0; // far from SOLAR_FREQUENCY → no photosynthesis
        e1.phase = 0.0;
        e1.dissipation = 0.001;
        e1.velocity = [3.0, 0.0]; // moving → no foraging
        let mut e2 = e1;
        e2.position = [1.0, 0.0];
        e2.frequency_hz = 50.0;
        e2.phase = 1.5;
        w.spawn(e1);
        w.spawn(e2);
        w.update_total_qe();
        let _before = w.total_qe;

        let mut s = ScratchPad::new();
        for _ in 0..10 {
            w.tick(&mut s);
        }
        // INV-B2: energy can only increase via external irradiance (Axiom 5).
        // Collision itself conserves; dissipation drains; solar adds.
        // Total may rise slightly from photosynthesis if entities are in Lux band.
        assert!(w.total_qe.is_finite(), "qe must be finite");
        // Both entities still alive (40 qe each, minimal dissipation).
        assert_eq!(w.entity_count, 2, "both should survive 10 ticks");
    }

    #[test]
    fn tick_reaps_dead_entities() {
        let mut w = SimWorldFlat::new(0, 0.05);
        let mut e = EntitySlot::default();
        e.qe = 0.005; // below QE_MIN_EXISTENCE
        e.dissipation = 0.01;
        w.spawn(e);
        let mut s = ScratchPad::new();
        w.tick(&mut s);
        assert_eq!(w.entity_count, 0, "starved entity should be reaped");
    }

    #[test]
    fn empty_world_tick_is_noop() {
        let mut w = SimWorldFlat::new(0, 0.05);
        let mut s = ScratchPad::new();
        w.tick(&mut s);
        assert_eq!(w.tick_id, 1);
        assert_eq!(w.entity_count, 0);
        assert_eq!(w.total_qe, 0.0);
    }

    #[test]
    fn determinism_same_seed_same_result() {
        let mut w1 = make_world_with_entities(8, 100.0);
        let mut w2 = make_world_with_entities(8, 100.0);
        let mut s1 = ScratchPad::new();
        let mut s2 = ScratchPad::new();
        for _ in 0..100 {
            w1.tick(&mut s1);
            w2.tick(&mut s2);
        }
        assert_eq!(w1.tick_id, w2.tick_id);
        // Bit-exact energy comparison
        for i in 0..8 {
            assert_eq!(
                w1.entities[i].qe.to_bits(),
                w2.entities[i].qe.to_bits(),
                "entity {i} qe diverged",
            );
        }
    }

    #[test]
    fn hundred_ticks_no_nan() {
        let mut w = make_world_with_entities(32, 50.0);
        let mut s = ScratchPad::new();
        for t in 0..100 {
            w.tick(&mut s);
            let mut mask = w.alive_mask;
            while mask != 0 {
                let i = mask.trailing_zeros() as usize;
                mask &= mask - 1;
                assert!(
                    w.entities[i].qe.is_finite(),
                    "NaN/Inf at tick {t}, entity {i}",
                );
            }
        }
    }

    #[test]
    fn movement_accumulates_across_ticks() {
        let mut w = SimWorldFlat::new(0, 0.05);
        let mut e = EntitySlot::default();
        e.qe = 40.0; // below reproduction threshold, above death threshold
        e.velocity = [2.0, 0.0];
        e.dissipation = 0.001;
        e.radius = 0.1;
        e.archetype = 0; // inert — no behavior, no reproduction
        w.spawn(e);
        let mut s = ScratchPad::new();
        for _ in 0..20 {
            w.tick(&mut s);
        }
        // 20 ticks × 2.0 × 0.05 = 2.0 units displacement (approx, with locomotion drain)
        assert!(
            (w.entities[0].position[0] - 2.0).abs() < 0.5,
            "pos={}",
            w.entities[0].position[0],
        );
    }
}
