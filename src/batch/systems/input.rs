//! Phase::Input batch systems — behavioral assess + decide.
//!
//! Simplified behavioral AI for batch: assess threats/food via N² scan,
//! decide action via utility scoring, write intent to will_intent.

use crate::batch::arena::SimWorldFlat;
use crate::batch::constants::MAX_ENTITIES;
use crate::batch::scratch::ScratchPad;
use crate::blueprint::{constants, equations};

/// Behavioral assessment: scan neighbors for nearest threat and nearest food.
///
/// Writes results into scratch.neighbors (nearest food idx, nearest threat idx).
/// Entities with mobility capacity exhibit behavior (Axiom 6: emergence from composition).
pub fn behavior_assess(world: &mut SimWorldFlat, scratch: &mut ScratchPad) {
    let mask = world.alive_mask;

    // Collect decisions into scratch buffer, then apply.
    // This avoids borrow conflicts between reading entities and writing will_intent.
    // Reuse scratch.pairs as (entity_idx, action, target_idx) packed into (u8, u8).
    scratch.pairs_len = 0;

    let mut mi = mask;
    while mi != 0 {
        let i = mi.trailing_zeros() as usize;
        mi &= mi - 1;
        // Axiom 6: behavior emerges from composition, not top-down tags.
        // Entities with mobility capacity (mobility_bias > 0) exhibit behavior.
        if world.entities[i].mobility_bias <= 0.01 { continue; }

        let (hunger, _energy_ratio) = equations::assess_energy(
            world.entities[i].engine_buffer, world.entities[i].engine_max.max(0.01),
        );

        let mut best_food_dist_sq = f32::MAX;
        let mut best_food_idx: u8 = 255;
        let mut best_threat_dist_sq = f32::MAX;
        let mut best_threat_idx: u8 = 255;

        let mut mj = mask;
        while mj != 0 {
            let j = mj.trailing_zeros() as usize;
            mj &= mj - 1;
            if i == j { continue; }

            let dx = world.entities[i].position[0] - world.entities[j].position[0];
            let dy = world.entities[i].position[1] - world.entities[j].position[1];
            let dist_sq = dx * dx + dy * dy;

            // Axiom 6: food identified by trophic hierarchy, not archetype tag.
            // Lower trophic class = potential food. Photosynthetic band (200-600 Hz) = plant-like.
            if world.entities[j].trophic_class < world.entities[i].trophic_class
                || (world.entities[j].frequency_hz >= 200.0
                    && world.entities[j].frequency_hz <= 600.0
                    && world.entities[j].trophic_class == 0)
            {
                if dist_sq < best_food_dist_sq {
                    best_food_dist_sq = dist_sq;
                    best_food_idx = j as u8;
                }
            }
            if world.entities[j].trophic_class > world.entities[i].trophic_class
                && world.entities[j].qe > world.entities[i].qe
            {
                if dist_sq < best_threat_dist_sq {
                    best_threat_dist_sq = dist_sq;
                    best_threat_idx = j as u8;
                }
            }
        }

        let food_dist = best_food_dist_sq.sqrt();
        let threat_dist = best_threat_dist_sq.sqrt();

        let mut scores = [0.0_f32; 5];
        scores[0] = if hunger < constants::HUNGER_THRESHOLD_FRACTION {
            constants::IDLE_SATIATED_SCORE
        } else {
            constants::IDLE_DEFAULT_SCORE
        };

        if best_food_idx < MAX_ENTITIES as u8 {
            scores[1] = equations::utility_forage(hunger, food_dist, world.entities[i].mobility_bias);
            if world.entities[i].trophic_class >= 3 {
                scores[3] = equations::utility_hunt(
                    world.entities[best_food_idx as usize].qe,
                    food_dist, world.entities[i].qe, world.entities[i].mobility_bias,
                );
            }
        }
        if best_threat_idx < MAX_ENTITIES as u8 {
            scores[2] = equations::utility_flee(
                equations::threat_level(world.entities[best_threat_idx as usize].qe, world.entities[i].qe),
                threat_dist, constants::HUNT_MAX_RANGE, world.entities[i].resilience,
            );
        }

        let mut action = equations::select_best_action(&scores) as u8;
        if scores[2] >= constants::PANIC_THRESHOLD { action = 2; }

        // Encode: action in pair.0 high nibble, target idx in pair.1
        let target = match action {
            1 | 3 => best_food_idx,
            2 => best_threat_idx,
            _ => 255,
        };
        if scratch.pairs_len < scratch.pairs.len() {
            scratch.pairs[scratch.pairs_len] = (i as u8, (action << 4) | (target & 0x0F));
            scratch.pairs_len += 1;
        }
    }

    // Apply decisions
    for p in 0..scratch.pairs_len {
        let i = scratch.pairs[p].0 as usize;
        let packed = scratch.pairs[p].1;
        let action = packed >> 4;
        let target = (packed & 0x0F) as usize;

        match action {
            1 if target < MAX_ENTITIES && world.alive_mask & (1 << target) != 0 => {
                let dx = world.entities[target].position[0] - world.entities[i].position[0];
                let dy = world.entities[target].position[1] - world.entities[i].position[1];
                let len = (dx * dx + dy * dy).sqrt().max(0.01);
                world.entities[i].will_intent = [dx / len, dy / len];
            }
            2 if target < MAX_ENTITIES && world.alive_mask & (1 << target) != 0 => {
                let dx = world.entities[i].position[0] - world.entities[target].position[0];
                let dy = world.entities[i].position[1] - world.entities[target].position[1];
                let len = (dx * dx + dy * dy).sqrt().max(0.01);
                world.entities[i].will_intent = [
                    dx / len * constants::BEHAVIOR_PANIC_FACTOR,
                    dy / len * constants::BEHAVIOR_PANIC_FACTOR,
                ];
            }
            3 if target < MAX_ENTITIES && world.alive_mask & (1 << target) != 0 => {
                let dx = world.entities[target].position[0] - world.entities[i].position[0];
                let dy = world.entities[target].position[1] - world.entities[i].position[1];
                let len = (dx * dx + dy * dy).sqrt().max(0.01);
                world.entities[i].will_intent = [
                    dx / len * constants::BEHAVIOR_SPRINT_FACTOR,
                    dy / len * constants::BEHAVIOR_SPRINT_FACTOR,
                ];
            }
            _ => {
                world.entities[i].will_intent = [0.0, 0.0];
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::batch::arena::EntitySlot;

    fn carnivore(w: &mut SimWorldFlat, qe: f32, pos: [f32; 2]) -> usize {
        let mut e = EntitySlot::default();
        e.qe = qe;
        e.radius = 1.0;
        e.position = pos;
        e.archetype = 2;
        e.trophic_class = 3; // carnivore
        e.engine_max = 50.0;
        e.engine_buffer = 5.0;
        e.mobility_bias = 0.7;
        e.resilience = 0.5;
        w.spawn(e).unwrap()
    }

    fn herbivore(w: &mut SimWorldFlat, qe: f32, pos: [f32; 2]) -> usize {
        let mut e = EntitySlot::default();
        e.qe = qe;
        e.radius = 0.8;
        e.position = pos;
        e.archetype = 2;
        e.trophic_class = 1; // herbivore
        e.engine_max = 40.0;
        e.engine_buffer = 5.0;
        e.mobility_bias = 0.5;
        e.resilience = 0.3;
        w.spawn(e).unwrap()
    }

    #[test]
    fn carnivore_intends_toward_prey() {
        let mut w = SimWorldFlat::new(0, 0.05);
        let pred = carnivore(&mut w, 200.0, [0.0, 0.0]);
        herbivore(&mut w, 50.0, [5.0, 0.0]); // prey to the right
        let mut scratch = ScratchPad::new();
        behavior_assess(&mut w, &mut scratch);
        // Predator should intend rightward (positive x)
        assert!(w.entities[pred].will_intent[0] > 0.0, "should move toward prey");
    }

    #[test]
    fn herbivore_flees_from_threat() {
        let mut w = SimWorldFlat::new(0, 0.05);
        let prey = herbivore(&mut w, 50.0, [0.0, 0.0]);
        carnivore(&mut w, 500.0, [3.0, 0.0]); // big predator nearby
        let mut scratch = ScratchPad::new();
        behavior_assess(&mut w, &mut scratch);
        // Herbivore should flee leftward (negative x)
        assert!(w.entities[prey].will_intent[0] < 0.0, "should flee from threat");
    }

    #[test]
    fn flora_has_no_behavior() {
        let mut w = SimWorldFlat::new(0, 0.05);
        let mut e = EntitySlot::default();
        e.qe = 100.0;
        e.archetype = 1; // flora
        let idx = w.spawn(e).unwrap();
        let mut scratch = ScratchPad::new();
        behavior_assess(&mut w, &mut scratch);
        assert_eq!(w.entities[idx].will_intent, [0.0, 0.0]);
    }

    #[test]
    fn isolated_entity_idles() {
        let mut w = SimWorldFlat::new(0, 0.05);
        let idx = carnivore(&mut w, 200.0, [0.0, 0.0]);
        let mut scratch = ScratchPad::new();
        behavior_assess(&mut w, &mut scratch);
        // No targets → idle
        assert_eq!(w.entities[idx].will_intent, [0.0, 0.0]);
    }
}
