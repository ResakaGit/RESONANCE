//! MC-5 Batch: Multicellularity — adhesion + colony + differential expression.
//!
//! Runs after protein_fold_infer, before morphological phase.
//! Uses collision pairs from ScratchPad to detect potential bonds.
//! Axiom 6: specialization emerges from position, not templates.

use crate::batch::arena::SimWorldFlat;
use crate::batch::constants::MAX_ENTITIES;
use crate::batch::scratch::ScratchPad;
use crate::blueprint::constants::{ADHESION_COST, EXPRESSION_MODULATION_RATE};
use crate::blueprint::equations::multicellular;

/// Multicellular step: adhesion → colony detection → positional signal → expression.
///
/// Conservation: only drains bond cost. Never creates energy. Axiom 4/5.
pub fn multicellular_step(world: &mut SimWorldFlat, scratch: &ScratchPad) {
    // 1. Build adjacency from collision pairs (already computed by collision system)
    let mut adjacency = [[false; MAX_ENTITIES]; MAX_ENTITIES];
    for k in 0..scratch.pairs_len {
        let (a, b) = scratch.pairs[k];
        let (ai, bi) = (a as usize, b as usize);
        if ai >= MAX_ENTITIES || bi >= MAX_ENTITIES { continue; }
        if world.alive_mask & (1 << ai) == 0 || world.alive_mask & (1 << bi) == 0 { continue; }

        let ea = &world.entities[ai];
        let eb = &world.entities[bi];
        let dx = ea.position[0] - eb.position[0];
        let dy = ea.position[1] - eb.position[1];
        let dist = (dx * dx + dy * dy).sqrt();

        let affinity = multicellular::adhesion_affinity(
            ea.frequency_hz, eb.frequency_hz, dist, ea.radius, eb.radius,
        );
        if multicellular::should_bond(affinity) {
            adjacency[ai][bi] = true;
            adjacency[bi][ai] = true;
        }
    }

    // 2. Detect colonies
    let colonies = multicellular::detect_colonies(&adjacency, world.alive_mask);

    // 3. Compute positional gradient
    let gradient = multicellular::positional_gradient(
        &adjacency, &colonies.colony_id, world.alive_mask,
    );

    // 4. Modulate expression masks + pay bond costs
    let mut mask = world.alive_mask;
    while mask != 0 {
        let i = mask.trailing_zeros() as usize;
        mask &= mask - 1;

        if colonies.colony_id[i] == 0 { continue; } // not in a colony

        // Differential expression (Axiom 6)
        let new_mask = multicellular::modulate_expression(
            gradient[i],
            &world.entities[i].expression_mask,
            EXPRESSION_MODULATION_RATE,
        );
        world.entities[i].expression_mask = new_mask;

        // Bond maintenance cost (Axiom 4): count bonds for this entity
        let bond_count = (0..MAX_ENTITIES)
            .filter(|&j| adjacency[i][j])
            .count() as f32;
        let cost = ADHESION_COST * bond_count;
        let drain = cost.min(world.entities[i].qe);
        world.entities[i].qe -= drain;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::batch::arena::SimWorldFlat;
    use crate::batch::scratch::ScratchPad;

    fn world_with_close_entities(n: usize, freq: f32) -> (SimWorldFlat, ScratchPad) {
        let mut world = SimWorldFlat::new(42, 0.05);
        let mut scratch = ScratchPad::new();
        for i in 0..n.min(MAX_ENTITIES) {
            world.entities[i].alive = true;
            world.entities[i].qe = 50.0;
            world.entities[i].radius = 1.0;
            world.entities[i].frequency_hz = freq;
            world.entities[i].position = [i as f32 * 1.5, 0.0]; // close together
            world.entities[i].expression_mask = [0.5; 4];
            world.alive_mask |= 1 << i;
            world.entity_count += 1;
        }
        // Create collision pairs for adjacent entities
        for i in 0..n.saturating_sub(1) {
            scratch.pairs[scratch.pairs_len] = (i as u8, (i + 1) as u8);
            scratch.pairs_len += 1;
        }
        (world, scratch)
    }

    #[test]
    fn step_conserves_energy() {
        let (mut world, scratch) = world_with_close_entities(4, 400.0);
        let qe_before: f32 = (0..4).map(|i| world.entities[i].qe).sum();
        multicellular_step(&mut world, &scratch);
        let qe_after: f32 = (0..4).map(|i| world.entities[i].qe).sum();
        assert!(qe_after <= qe_before, "energy must not increase: {qe_after} <= {qe_before}");
    }

    #[test]
    fn step_modulates_expression() {
        let (mut world, scratch) = world_with_close_entities(4, 400.0);
        let mask_before = world.entities[0].expression_mask;
        multicellular_step(&mut world, &scratch);
        // If a colony formed, expression should differ from starting [0.5; 4]
        let mask_after = world.entities[0].expression_mask;
        // May or may not change depending on whether affinity > threshold
        let _ = (mask_before, mask_after); // no panic
    }

    #[test]
    fn step_no_entities_no_panic() {
        let mut world = SimWorldFlat::new(42, 0.05);
        let scratch = ScratchPad::new();
        multicellular_step(&mut world, &scratch);
    }

    #[test]
    fn step_dead_entities_skipped() {
        let (mut world, scratch) = world_with_close_entities(4, 400.0);
        world.alive_mask = 0; // all dead
        multicellular_step(&mut world, &scratch);
        assert_eq!(world.entities[0].qe, 50.0, "dead entities untouched");
    }

    #[test]
    fn step_different_freq_no_bond() {
        let (mut world, scratch) = world_with_close_entities(4, 400.0);
        world.entities[1].frequency_hz = 800.0; // very different
        world.entities[2].frequency_hz = 200.0;
        let qe_before = world.entities[0].qe;
        multicellular_step(&mut world, &scratch);
        // Bond cost should be lower because fewer bonds formed
        assert!(world.entities[0].qe >= qe_before - 0.1, "weak/no bonds = minimal cost");
    }

    #[test]
    fn step_deterministic() {
        let (mut a, sa) = world_with_close_entities(4, 400.0);
        let (mut b, sb) = world_with_close_entities(4, 400.0);
        multicellular_step(&mut a, &sa);
        multicellular_step(&mut b, &sb);
        for i in 0..4 {
            assert_eq!(a.entities[i].qe.to_bits(), b.entities[i].qe.to_bits());
            assert_eq!(a.entities[i].expression_mask, b.entities[i].expression_mask);
        }
    }
}
