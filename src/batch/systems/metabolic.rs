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

/// Trophic forage: herbivores extract from nutrient grid.
///
/// Herbivores (trophic_class=1) drain nutrients at their position,
/// gain qe and satiation. Calls `equations::satiation_gain_from_meal`.
pub fn trophic_forage(world: &mut SimWorldFlat) {
    use crate::batch::systems::thermodynamic::grid_cell;
    let mut mask = world.alive_mask;
    while mask != 0 {
        let i = mask.trailing_zeros() as usize;
        mask &= mask - 1;
        let tc = world.entities[i].trophic_class;
        if tc != 1 && tc != 2 { continue; } // herbivore or omnivore only
        let cell = grid_cell(world.entities[i].position);
        if cell >= GRID_CELLS { continue; }
        let available = world.nutrient_grid[cell];
        if available <= 0.0 { continue; }
        let intake = (world.entities[i].radius * 0.5).min(available);
        world.nutrient_grid[cell] -= intake;
        world.entities[i].qe += intake;
        world.entities[i].satiation += equations::satiation_gain_from_meal(intake);
    }
}

/// Trophic predation: carnivores attack prey in range.
///
/// N² pair scan. Carnivore (trophic_class>=3) drains herbivore/omnivore.
/// Conservation: prey loses exactly what predator gains / assimilation.
pub fn trophic_predation(world: &mut SimWorldFlat, scratch: &mut ScratchPad) {
    let range_sq = PREDATION_RANGE * PREDATION_RANGE;
    scratch.pairs_len = 0;

    // Collect predator-prey pairs
    let mut mi = world.alive_mask;
    while mi != 0 {
        let i = mi.trailing_zeros() as usize;
        mi &= mi - 1;
        if world.entities[i].trophic_class < 3 { continue; } // only carnivores
        if world.entities[i].satiation > 0.7 { continue; } // well-fed skip

        let mut mj = world.alive_mask;
        while mj != 0 {
            let j = mj.trailing_zeros() as usize;
            mj &= mj - 1;
            if i == j { continue; }
            if world.entities[j].trophic_class >= 3 { continue; } // not prey

            let dx = world.entities[i].position[0] - world.entities[j].position[0];
            let dy = world.entities[i].position[1] - world.entities[j].position[1];
            let dist_sq = dx * dx + dy * dy;
            if dist_sq < range_sq && scratch.pairs_len < scratch.pairs.len() {
                scratch.pairs[scratch.pairs_len] = (i as u8, j as u8);
                scratch.pairs_len += 1;
            }
        }
    }

    // Resolve predation
    for p in 0..scratch.pairs_len {
        let (pi, qi) = (scratch.pairs[p].0 as usize, scratch.pairs[p].1 as usize);
        let drain = world.entities[qi].qe * PREDATION_DRAIN_FRACTION;
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
pub fn social_pack(world: &mut SimWorldFlat, scratch: &mut ScratchPad) {
    let range_sq = PACK_SCAN_RADIUS * PACK_SCAN_RADIUS;
    let mask = world.alive_mask;

    let mut mi = mask;
    while mi != 0 {
        let i = mi.trailing_zeros() as usize;
        mi &= mi - 1;
        if world.entities[i].faction == 0 { continue; } // no faction

        // Compute centroid of same-faction neighbors
        let mut cx = 0.0_f32;
        let mut cy = 0.0_f32;
        let mut count = 0u32;

        let mut mj = mask;
        while mj != 0 {
            let j = mj.trailing_zeros() as usize;
            mj &= mj - 1;
            if i == j { continue; }
            if world.entities[j].faction != world.entities[i].faction { continue; }
            let dx = world.entities[i].position[0] - world.entities[j].position[0];
            let dy = world.entities[i].position[1] - world.entities[j].position[1];
            if dx * dx + dy * dy < range_sq {
                cx += world.entities[j].position[0];
                cy += world.entities[j].position[1];
                count += 1;
            }
        }

        if count == 0 { continue; }
        cx /= count as f32;
        cy /= count as f32;

        // Pull toward centroid
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

            // Cooperate if constructive interference (affinity > 0)
            if affinity > 0.0 && scratch.pairs_len < scratch.pairs.len() {
                let bonus = affinity * constants::COOPERATION_GROUP_BONUS * 0.01;
                world.entities[i].qe += bonus;
                world.entities[j].qe += bonus;
                scratch.pairs[scratch.pairs_len] = (i as u8, j as u8);
                scratch.pairs_len += 1;
            }
        }
    }
}

/// Culture transmission: nearby entities with high oscillatory affinity
/// influence each other's expression mask.
pub fn culture_transmission(world: &mut SimWorldFlat, scratch: &mut ScratchPad) {
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

            if affinity <= 0.3 { continue; } // low affinity → no imitation

            // Blend expression masks toward each other (small step)
            let blend = affinity * 0.01; // 1% per tick scaled by affinity
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
pub fn ecology_census(world: &SimWorldFlat) -> u8 {
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
    fn trophic_forage_carnivore_skips() {
        use crate::batch::systems::thermodynamic::grid_cell;
        let mut w = SimWorldFlat::new(0, 0.05);
        carnivore(&mut w, 100.0, [3.0, 3.0]);
        let cell = grid_cell([3.0, 3.0]);
        w.nutrient_grid[cell] = 20.0;
        trophic_forage(&mut w);
        assert_eq!(w.nutrient_grid[cell], 20.0, "carnivore shouldn't forage");
    }

    // ── trophic_predation ───────────────────────────────────────────────────

    #[test]
    fn predation_transfers_energy() {
        let mut w = SimWorldFlat::new(0, 0.05);
        let pred = carnivore(&mut w, 100.0, [0.0, 0.0]);
        let prey = herbivore(&mut w, 80.0, [1.0, 0.0]); // within range
        let total_before = w.entities[pred].qe + w.entities[prey].qe;
        let mut scratch = ScratchPad::new();
        trophic_predation(&mut w, &mut scratch);
        assert!(w.entities[pred].qe > 100.0, "predator should gain");
        assert!(w.entities[prey].qe < 80.0, "prey should lose");
        // Total decreases (assimilation < 1.0)
        let total_after = w.entities[pred].qe + w.entities[prey].qe;
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
    fn pack_cohesion_pulls_toward_centroid() {
        let mut w = SimWorldFlat::new(0, 0.05);
        let mut e1 = EntitySlot::default();
        e1.qe = 100.0;
        e1.position = [0.0, 0.0];
        e1.faction = 1;
        let mut e2 = e1;
        e2.position = [4.0, 0.0];
        let i1 = w.spawn(e1).unwrap();
        w.spawn(e2);
        let mut scratch = ScratchPad::new();
        social_pack(&mut w, &mut scratch);
        // e1 should be pulled rightward (toward e2)
        assert!(w.entities[i1].velocity[0] > 0.0, "should pull toward ally");
    }

    #[test]
    fn pack_different_faction_no_cohesion() {
        let mut w = SimWorldFlat::new(0, 0.05);
        let mut e1 = EntitySlot::default();
        e1.qe = 100.0;
        e1.position = [0.0, 0.0];
        e1.faction = 1;
        let mut e2 = e1;
        e2.position = [4.0, 0.0];
        e2.faction = 2;
        let i1 = w.spawn(e1).unwrap();
        w.spawn(e2);
        let mut scratch = ScratchPad::new();
        social_pack(&mut w, &mut scratch);
        assert_eq!(w.entities[i1].velocity[0], 0.0, "different factions → no pull");
    }

    // ── cooperation_eval ────────────────────────────────────────────────────

    #[test]
    fn cooperation_boosts_aligned_entities() {
        let mut w = SimWorldFlat::new(0, 0.05);
        let mut e1 = EntitySlot::default();
        e1.qe = 100.0;
        e1.position = [0.0, 0.0];
        e1.frequency_hz = 440.0;
        e1.phase = 0.0;
        let mut e2 = e1;
        e2.position = [2.0, 0.0];
        e2.frequency_hz = 440.0; // same freq → constructive interference
        e2.phase = 0.0;
        let i1 = w.spawn(e1).unwrap();
        let i2 = w.spawn(e2).unwrap();
        let mut scratch = ScratchPad::new();
        cooperation_eval(&mut w, &mut scratch);
        assert!(w.entities[i1].qe > 100.0, "should gain from cooperation");
        assert!(w.entities[i2].qe > 100.0, "both should gain");
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
