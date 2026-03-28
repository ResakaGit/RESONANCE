//! Phase::MetabolicLayer batch systems — pool distribution, trophic,
//! social packs, cooperation, culture, ecology census.

use crate::batch::arena::SimWorldFlat;
use crate::batch::constants::*;
use crate::batch::scratch::ScratchPad;
use crate::blueprint::{constants, equations};
use crate::blueprint::equations::emergence::culture as culture_eq;

/// L5 engine buffer → L0 qe: distribute buffered energy back to entity.
///
/// Each tick, `output_valve` fraction of buffer returns to qe.
/// Calls `equations::extract_proportional` for fair-share distribution.
pub fn pool_distribution(world: &mut SimWorldFlat) {
    let mut mask = world.alive_mask;
    while mask != 0 {
        let i = mask.trailing_zeros() as usize;
        mask &= mask - 1;
        let e = &mut world.entities[i];
        if e.engine_buffer <= 0.0 || e.output_valve <= 0.0 { continue; }
        let available = e.engine_buffer * e.output_valve;
        let release = equations::extract_proportional(available, 1);
        let clamped = release.min(e.engine_buffer);
        if clamped <= 0.0 { continue; }
        e.engine_buffer -= clamped;
        e.qe += clamped;
    }
}

/// Trophic forage: slow entities extract from nutrient grid.
///
/// Axiom 6: foraging ability from being slow (composition), not trophic tag.
/// Stationary or slow-moving entities can graze; fast ones cannot.
pub fn trophic_forage(world: &mut SimWorldFlat) {
    use crate::batch::systems::thermodynamic::grid_cell;
    let mut mask = world.alive_mask;
    while mask != 0 {
        let i = mask.trailing_zeros() as usize;
        mask &= mask - 1;
        let spd_sq = world.entities[i].velocity[0].powi(2) + world.entities[i].velocity[1].powi(2);
        if spd_sq > FORAGE_MAX_SPEED_SQ { continue; }
        let cell = grid_cell(world.entities[i].position);
        if cell >= GRID_CELLS { continue; }
        let available = world.nutrient_grid[cell];
        if available <= 0.0 { continue; }
        let intake = (world.entities[i].radius * NUTRIENT_UPTAKE_RATE).min(available);
        world.nutrient_grid[cell] -= intake;
        world.entities[i].qe += intake;
        world.entities[i].satiation += equations::satiation_gain_from_meal(intake);
    }
}

/// Trophic predation: energy-dominant entities drain weaker ones in range.
///
/// Axiom 6: predation from energy dominance, not trophic tags.
/// Axiom 3: drain modulated by interference. Axiom 8: oscillatory.
pub fn trophic_predation(world: &mut SimWorldFlat, scratch: &mut ScratchPad) {
    let range_sq = PREDATION_RANGE * PREDATION_RANGE;
    scratch.pairs_len = 0;

    let mut mi = world.alive_mask;
    while mi != 0 {
        let i = mi.trailing_zeros() as usize;
        mi &= mi - 1;
        if world.entities[i].satiation > SATIATION_WELL_FED { continue; }
        let pred_qe = world.entities[i].qe;

        let mut mj = world.alive_mask;
        while mj != 0 {
            let j = mj.trailing_zeros() as usize;
            mj &= mj - 1;
            if i == j { continue; }
            if world.entities[j].qe >= pred_qe * PREDATION_DOMINANCE_RATIO { continue; }

            let dx = world.entities[i].position[0] - world.entities[j].position[0];
            let dy = world.entities[i].position[1] - world.entities[j].position[1];
            let dist_sq = dx * dx + dy * dy;
            if dist_sq < range_sq && scratch.pairs_len < scratch.pairs.len() {
                scratch.pairs[scratch.pairs_len] = (i as u8, j as u8);
                scratch.pairs_len += 1;
            }
        }
    }

    // Resolve predation — Axiom 3: magnitude = base × interference_factor
    // Axiom 8: interaction modulated by cos(Δf × t + Δφ)
    for p in 0..scratch.pairs_len {
        let (pi, qi) = (scratch.pairs[p].0 as usize, scratch.pairs[p].1 as usize);
        let interference = equations::interference(
            world.entities[pi].frequency_hz,
            world.entities[pi].phase,
            world.entities[qi].frequency_hz,
            world.entities[qi].phase,
            0.0,
        ).abs();
        let drain = world.entities[qi].qe * PREDATION_DRAIN_FRACTION * interference;
        let safe_drain = drain.min(world.entities[qi].qe);
        if safe_drain <= 0.0 { continue; }
        let assimilated = safe_drain * CARNIVORE_ASSIMILATION;
        world.entities[qi].qe -= safe_drain;
        world.entities[pi].qe += assimilated;
        world.entities[pi].satiation += equations::satiation_gain_from_meal(assimilated);
    }
}

/// Social pack: same-faction entities nearby pull toward centroid.
///
/// Applies cohesion force to velocity.
pub fn social_pack(world: &mut SimWorldFlat, _scratch: &mut ScratchPad) {
    let range_sq = PACK_SCAN_RADIUS * PACK_SCAN_RADIUS;
    let mask = world.alive_mask;

    // Axiom 6: packs emerge from oscillatory affinity, not top-down faction tags.
    // Axiom 8: cohesion modulated by cos(Δf × t + Δφ).
    let mut mi = mask;
    while mi != 0 {
        let i = mi.trailing_zeros() as usize;
        mi &= mi - 1;

        let mut cx = 0.0_f32;
        let mut cy = 0.0_f32;
        let mut weight_sum = 0.0_f32;

        let mut mj = mask;
        while mj != 0 {
            let j = mj.trailing_zeros() as usize;
            mj &= mj - 1;
            if i == j { continue; }
            let dx = world.entities[i].position[0] - world.entities[j].position[0];
            let dy = world.entities[i].position[1] - world.entities[j].position[1];
            if dx * dx + dy * dy >= range_sq { continue; }

            // Axiom 8: affinity from oscillatory interference
            let affinity = equations::interference(
                world.entities[i].frequency_hz,
                world.entities[i].phase,
                world.entities[j].frequency_hz,
                world.entities[j].phase,
                0.0,
            );
            if affinity <= 0.3 { continue; } // only constructive → cohesion

            cx += world.entities[j].position[0] * affinity;
            cy += world.entities[j].position[1] * affinity;
            weight_sum += affinity;
        }

        if weight_sum < GUARD_EPSILON { continue; }
        cx /= weight_sum;
        cy /= weight_sum;

        let fx = (cx - world.entities[i].position[0]) * PACK_COHESION_STRENGTH;
        let fy = (cy - world.entities[i].position[1]) * PACK_COHESION_STRENGTH;
        world.entities[i].velocity[0] += fx * world.dt;
        world.entities[i].velocity[1] += fy * world.dt;
    }
}

/// Cooperation eval: Nash-stable pairs within range.
///
/// Two entities cooperate if the shared bonus exceeds solo benefit.
/// Cooperation strengthens both (small qe boost).
pub fn cooperation_eval(world: &mut SimWorldFlat, scratch: &mut ScratchPad) {
    let range_sq = COOPERATION_SCAN_RADIUS * COOPERATION_SCAN_RADIUS;
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
            if dx * dx + dy * dy >= range_sq { continue; }

            // Oscillatory affinity determines cooperation benefit
            let affinity = equations::interference(
                world.entities[i].frequency_hz,
                world.entities[i].phase,
                world.entities[j].frequency_hz,
                world.entities[j].phase,
                0.0,
            );

            // Axiom 5: cooperation reduces dissipation, never creates energy.
            // Axiom 3: magnitude modulated by interference factor.
            if affinity > 0.0 && scratch.pairs_len < scratch.pairs.len() {
                let reduction = affinity * constants::COOPERATION_GROUP_BONUS * 0.001;
                world.entities[i].dissipation = (world.entities[i].dissipation - reduction).max(DISSIPATION_FLOOR);
                world.entities[j].dissipation = (world.entities[j].dissipation - reduction).max(DISSIPATION_FLOOR);
                scratch.pairs[scratch.pairs_len] = (i as u8, j as u8);
                scratch.pairs_len += 1;
            }
        }
    }
}

/// Culture transmission: nearby entities with high oscillatory affinity
/// influence each other's expression mask.
pub fn culture_transmission(world: &mut SimWorldFlat, _scratch: &mut ScratchPad) {
    let range_sq = CULTURE_SCAN_RADIUS * CULTURE_SCAN_RADIUS;
    let mask = world.alive_mask;

    let mut mi = mask;
    while mi != 0 {
        let i = mi.trailing_zeros() as usize;
        mi &= mi - 1;

        let mut mj = mi;
        while mj != 0 {
            let j = mj.trailing_zeros() as usize;
            mj &= mj - 1;

            let dx = world.entities[i].position[0] - world.entities[j].position[0];
            let dy = world.entities[i].position[1] - world.entities[j].position[1];
            if dx * dx + dy * dy >= range_sq { continue; }

            let affinity = culture_eq::frequency_imitation_affinity(
                world.entities[i].frequency_hz,
                world.entities[i].phase,
                world.entities[j].frequency_hz,
                world.entities[j].phase,
                0.0,
            );

            if affinity <= CULTURE_AFFINITY_MIN { continue; }

            // Blend expression masks toward each other (small step)
            let blend = affinity * CULTURE_BLEND_RATE;
            for d in 0..4 {
                let delta = world.entities[j].expression_mask[d]
                          - world.entities[i].expression_mask[d];
                world.entities[i].expression_mask[d] += delta * blend;
            }
        }
    }
}

/// Ecology census: count distinct frequency bands (species proxy).
///
/// No N² interaction — single scan. Writes nothing to entities,
/// just updates a world-level observable.
pub(super) fn ecology_census(world: &SimWorldFlat) -> u8 {
    let mut bands = [false; 16];
    let mut mask = world.alive_mask;
    while mask != 0 {
        let i = mask.trailing_zeros() as usize;
        mask &= mask - 1;
        let band = (world.entities[i].frequency_hz / 100.0).min(15.0) as usize;
        bands[band] = true;
    }
    bands.iter().filter(|&&b| b).count() as u8
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::batch::arena::EntitySlot;

    #[test]
    fn pool_distribution_releases_buffer_to_qe() {
        let mut w = SimWorldFlat::new(0, 0.05);
        let mut e = EntitySlot::default();
        e.qe = 50.0;
        e.engine_buffer = 20.0;
        e.engine_max = 50.0;
        e.output_valve = 0.5;
        let idx = w.spawn(e).unwrap();
        let total_before = w.entities[idx].qe + w.entities[idx].engine_buffer;
        pool_distribution(&mut w);
        let total_after = w.entities[idx].qe + w.entities[idx].engine_buffer;
        assert!(w.entities[idx].engine_buffer < 20.0, "buffer should decrease");
        assert!(w.entities[idx].qe > 50.0, "qe should increase");
        assert!((total_after - total_before).abs() < 1e-5, "energy conserved");
    }

    #[test]
    fn pool_distribution_zero_buffer_noop() {
        let mut w = SimWorldFlat::new(0, 0.05);
        let mut e = EntitySlot::default();
        e.qe = 50.0;
        e.engine_buffer = 0.0;
        e.output_valve = 1.0;
        w.spawn(e);
        pool_distribution(&mut w);
        assert_eq!(w.entities[0].qe, 50.0);
    }

    #[test]
    fn pool_distribution_zero_valve_noop() {
        let mut w = SimWorldFlat::new(0, 0.05);
        let mut e = EntitySlot::default();
        e.qe = 50.0;
        e.engine_buffer = 20.0;
        e.output_valve = 0.0;
        w.spawn(e);
        pool_distribution(&mut w);
        assert_eq!(w.entities[0].engine_buffer, 20.0);
    }

    // ── trophic_forage ──────────────────────────────────────────────────────

    fn herbivore(w: &mut SimWorldFlat, qe: f32, pos: [f32; 2]) -> usize {
        let mut e = EntitySlot::default();
        e.qe = qe;
        e.radius = 1.0;
        e.position = pos;
        e.archetype = 2;
        e.trophic_class = 1; // herbivore
        w.spawn(e).unwrap()
    }

    fn carnivore(w: &mut SimWorldFlat, qe: f32, pos: [f32; 2]) -> usize {
        let mut e = EntitySlot::default();
        e.qe = qe;
        e.radius = 1.0;
        e.position = pos;
        e.archetype = 2;
        e.trophic_class = 3; // carnivore
        e.satiation = 0.0;
        w.spawn(e).unwrap()
    }

    #[test]
    fn trophic_forage_herbivore_gains_from_grid() {
        use crate::batch::systems::thermodynamic::grid_cell;
        let mut w = SimWorldFlat::new(0, 0.05);
        let idx = herbivore(&mut w, 50.0, [3.0, 3.0]);
        let cell = grid_cell([3.0, 3.0]);
        w.nutrient_grid[cell] = 20.0;
        let before = w.entities[idx].qe;
        trophic_forage(&mut w);
        assert!(w.entities[idx].qe > before, "herbivore should gain qe");
        assert!(w.nutrient_grid[cell] < 20.0, "grid should deplete");
    }

    #[test]
    fn fast_entity_skips_foraging() {
        use crate::batch::systems::thermodynamic::grid_cell;
        let mut w = SimWorldFlat::new(0, 0.05);
        let idx = carnivore(&mut w, 100.0, [3.0, 3.0]);
        w.entities[idx].velocity = [5.0, 5.0]; // fast → can't forage
        let cell = grid_cell([3.0, 3.0]);
        w.nutrient_grid[cell] = 20.0;
        trophic_forage(&mut w);
        assert_eq!(w.nutrient_grid[cell], 20.0, "fast entity can't forage");
    }

    // ── trophic_predation ───────────────────────────────────────────────────

    #[test]
    fn dominant_entity_drains_weaker() {
        let mut w = SimWorldFlat::new(0, 0.05);
        let strong = carnivore(&mut w, 200.0, [0.0, 0.0]);
        let weak = herbivore(&mut w, 50.0, [1.0, 0.0]); // qe < 200 * 0.7
        let total_before = w.entities[strong].qe + w.entities[weak].qe;
        let mut scratch = ScratchPad::new();
        trophic_predation(&mut w, &mut scratch);
        assert!(w.entities[strong].qe > 200.0, "dominant should gain");
        assert!(w.entities[weak].qe < 50.0, "weak should lose");
        let total_after = w.entities[strong].qe + w.entities[weak].qe;
        assert!(total_after <= total_before + 1e-3, "no energy creation");
    }

    #[test]
    fn predation_out_of_range_noop() {
        let mut w = SimWorldFlat::new(0, 0.05);
        carnivore(&mut w, 100.0, [0.0, 0.0]);
        herbivore(&mut w, 80.0, [20.0, 0.0]); // far away
        let mut scratch = ScratchPad::new();
        trophic_predation(&mut w, &mut scratch);
        assert_eq!(scratch.pairs_len, 0);
    }

    // ── social_pack ─────────────────────────────────────────────────────────

    #[test]
    fn pack_cohesion_pulls_aligned_frequencies() {
        let mut w = SimWorldFlat::new(0, 0.05);
        let mut e1 = EntitySlot::default();
        e1.qe = 100.0;
        e1.position = [0.0, 0.0];
        e1.frequency_hz = 440.0;
        e1.phase = 0.0;
        let mut e2 = e1;
        e2.position = [4.0, 0.0];
        // Same freq + phase → constructive interference → cohesion
        let i1 = w.spawn(e1).unwrap();
        w.spawn(e2);
        let mut scratch = ScratchPad::new();
        social_pack(&mut w, &mut scratch);
        assert!(w.entities[i1].velocity[0] > 0.0, "aligned freq → pull toward");
    }

    #[test]
    fn pack_no_cohesion_for_destructive_interference() {
        let mut w = SimWorldFlat::new(0, 0.05);
        let mut e1 = EntitySlot::default();
        e1.qe = 100.0;
        e1.position = [0.0, 0.0];
        e1.frequency_hz = 440.0;
        e1.phase = 0.0;
        let mut e2 = e1;
        e2.position = [4.0, 0.0];
        e2.frequency_hz = 440.0;
        e2.phase = std::f32::consts::PI; // opposite phase → destructive
        let i1 = w.spawn(e1).unwrap();
        w.spawn(e2);
        let mut scratch = ScratchPad::new();
        social_pack(&mut w, &mut scratch);
        assert_eq!(w.entities[i1].velocity[0], 0.0, "destructive → no cohesion");
    }

    // ── cooperation_eval ────────────────────────────────────────────────────

    #[test]
    fn cooperation_reduces_dissipation() {
        let mut w = SimWorldFlat::new(0, 0.05);
        let mut e1 = EntitySlot::default();
        e1.qe = 100.0;
        e1.dissipation = 0.05;
        e1.position = [0.0, 0.0];
        e1.frequency_hz = 440.0;
        e1.phase = 0.0;
        let mut e2 = e1;
        e2.position = [2.0, 0.0];
        e2.frequency_hz = 440.0; // same freq → constructive interference
        e2.phase = 0.0;
        let i1 = w.spawn(e1).unwrap();
        let i2 = w.spawn(e2).unwrap();
        let d_before_1 = w.entities[i1].dissipation;
        let d_before_2 = w.entities[i2].dissipation;
        let mut scratch = ScratchPad::new();
        cooperation_eval(&mut w, &mut scratch);
        // Axiom 5: cooperation reduces dissipation, never creates energy
        assert!(w.entities[i1].dissipation < d_before_1, "dissipation should decrease");
        assert!(w.entities[i2].dissipation < d_before_2, "both should benefit");
        assert_eq!(w.entities[i1].qe, 100.0, "qe must not change — Axiom 5");
    }

    // ── culture_transmission ────────────────────────────────────────────────

    #[test]
    fn culture_blends_expression_masks() {
        let mut w = SimWorldFlat::new(0, 0.05);
        let mut e1 = EntitySlot::default();
        e1.qe = 100.0;
        e1.position = [0.0, 0.0];
        e1.frequency_hz = 440.0;
        e1.phase = 0.0;
        e1.expression_mask = [0.0, 0.0, 0.0, 0.0];
        let mut e2 = e1;
        e2.position = [2.0, 0.0];
        e2.expression_mask = [1.0, 1.0, 1.0, 1.0];
        let i1 = w.spawn(e1).unwrap();
        w.spawn(e2);
        let mut scratch = ScratchPad::new();
        culture_transmission(&mut w, &mut scratch);
        // e1's mask should have moved slightly toward e2's
        assert!(w.entities[i1].expression_mask[0] > 0.0, "should blend toward neighbor");
    }

    // ── ecology_census ──────────────────────────────────────────────────────

    #[test]
    fn ecology_census_counts_distinct_bands() {
        let mut w = SimWorldFlat::new(0, 0.05);
        let mut e1 = EntitySlot::default();
        e1.qe = 100.0;
        e1.frequency_hz = 150.0; // band 1
        let mut e2 = e1;
        e2.frequency_hz = 450.0; // band 4
        let mut e3 = e1;
        e3.frequency_hz = 160.0; // band 1 (same as e1)
        w.spawn(e1);
        w.spawn(e2);
        w.spawn(e3);
        assert_eq!(ecology_census(&w), 2, "two distinct bands");
    }

    #[test]
    fn ecology_census_empty_world() {
        let w = SimWorldFlat::new(0, 0.05);
        assert_eq!(ecology_census(&w), 0);
    }
}
