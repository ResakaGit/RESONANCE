//! MC-1/2/3/4: Cell Adhesion + Colony Detection + Positional Signal + Differential Expression.
//!
//! Axiom 7: adhesion decays with distance. Axiom 8: frequency alignment for bonding.
//! Axiom 6: specialization emerges from position, not templates.
//! Axiom 4: bonds cost energy to maintain.

use crate::blueprint::constants::{
    ADHESION_COST, ADHESION_FREQ_BANDWIDTH, ADHESION_THRESHOLD,
    BOND_STRENGTH_SCALE, BORDER_TARGET,
    INTERIOR_TARGET, MIN_COLONY_SIZE,
};
use crate::batch::constants::MAX_ENTITIES;

// ─── MC-1: Cell Adhesion ────────────────────────────────────────────────────

/// Adhesion affinity ∈ [0,1]. Axiom 7 (distance) × Axiom 8 (frequency).
pub fn adhesion_affinity(freq_a: f32, freq_b: f32, distance: f32, radius_a: f32, radius_b: f32) -> f32 {
    if !freq_a.is_finite() || !freq_b.is_finite() || !distance.is_finite() { return 0.0; }
    let contact = radius_a + radius_b;
    if distance > contact * 2.0 { return 0.0; }
    let proximity = (1.0 - distance / (contact * 2.0).max(0.01)).clamp(0.0, 1.0);
    let freq_align = gaussian_alignment(freq_a, freq_b, ADHESION_FREQ_BANDWIDTH);
    proximity * freq_align
}

/// Bond strength from affinity + energy. Axiom 1: energy determines bond.
pub fn bond_strength(affinity: f32, qe_a: f32, qe_b: f32) -> f32 {
    affinity * qe_a.min(qe_b).max(0.0).sqrt() * BOND_STRENGTH_SCALE
}

/// Maintenance cost per bond per tick. Axiom 4.
pub fn bond_cost(strength: f32) -> f32 {
    strength.max(0.0) * ADHESION_COST
}

/// Should two cells bond? Pure decision.
pub fn should_bond(affinity: f32) -> bool {
    affinity > ADHESION_THRESHOLD
}

// ─── MC-2: Colony Detection (Union-Find, fixed-size) ────────────────────────

/// Colony assignment. Stack-allocated.
#[derive(Clone, Copy, Debug)]
pub struct ColonyMap {
    pub colony_id: [u8; MAX_ENTITIES],
    pub colony_count: u8,
    pub max_colony_size: u8,
}

/// Detect connected components via Union-Find. No heap.
pub fn detect_colonies(adjacency: &[[bool; MAX_ENTITIES]; MAX_ENTITIES], alive_mask: u128) -> ColonyMap {
    let mut parent = [0u8; MAX_ENTITIES];
    for i in 0..MAX_ENTITIES { parent[i] = i as u8; }

    // Union pass
    let mut mask = alive_mask;
    while mask != 0 {
        let i = mask.trailing_zeros() as usize;
        mask &= mask - 1;
        let mut mask2 = if i >= 127 { 0 } else { alive_mask & !((1u128 << (i + 1)) - 1) };
        while mask2 != 0 {
            let j = mask2.trailing_zeros() as usize;
            mask2 &= mask2 - 1;
            if adjacency[i][j] { union(&mut parent, i, j); }
        }
    }

    // Assign colony IDs
    let mut colony_id = [0u8; MAX_ENTITIES];
    let mut label = 1u8;
    let mut root_to_label = [0u8; MAX_ENTITIES];
    let mut colony_sizes = [0u8; MAX_ENTITIES];

    let mut mask = alive_mask;
    while mask != 0 {
        let i = mask.trailing_zeros() as usize;
        mask &= mask - 1;
        let root = find(&parent, i);
        if root_to_label[root] == 0 {
            root_to_label[root] = label;
            label = label.saturating_add(1);
        }
        colony_id[i] = root_to_label[root];
        colony_sizes[root_to_label[root] as usize] = colony_sizes[root_to_label[root] as usize].saturating_add(1);
    }

    // Filter: only colonies ≥ MIN_COLONY_SIZE count
    let mut real_count = 0u8;
    let mut max_size = 0u8;
    for i in 1..label {
        if colony_sizes[i as usize] >= MIN_COLONY_SIZE {
            real_count += 1;
            max_size = max_size.max(colony_sizes[i as usize]);
        } else {
            // Too small — reset to 0 (not a real colony)
            let lid = i;
            for c in colony_id.iter_mut() { if *c == lid { *c = 0; } }
        }
    }

    ColonyMap { colony_id, colony_count: real_count, max_colony_size: max_size }
}

fn find(parent: &[u8; MAX_ENTITIES], mut i: usize) -> usize {
    while parent[i] as usize != i { i = parent[i] as usize; }
    i
}

fn union(parent: &mut [u8; MAX_ENTITIES], a: usize, b: usize) {
    let ra = find(parent, a);
    let rb = find(parent, b);
    if ra != rb { parent[rb] = ra as u8; }
}

// ─── MC-3: Positional Signal ────────────────────────────────────────────────

/// Border signal ∈ [0,1]. 1.0 = exposed, 0.0 = interior. Axiom 7: local only.
pub fn border_signal(neighbor_count: u8, max_neighbors: u8) -> f32 {
    if max_neighbors == 0 { return 1.0; }
    1.0 - (neighbor_count as f32 / max_neighbors as f32).clamp(0.0, 1.0)
}

/// Compute border signal for all entities. Pure.
pub fn positional_gradient(
    adjacency: &[[bool; MAX_ENTITIES]; MAX_ENTITIES],
    colony_ids: &[u8; MAX_ENTITIES],
    alive_mask: u128,
) -> [f32; MAX_ENTITIES] {
    let mut signals = [0.0f32; MAX_ENTITIES];
    let mut mask = alive_mask;
    while mask != 0 {
        let i = mask.trailing_zeros() as usize;
        mask &= mask - 1;
        if colony_ids[i] == 0 { continue; } // not in a colony

        let my_colony = colony_ids[i];
        let neighbors = (0..MAX_ENTITIES)
            .filter(|&j| j != i && adjacency[i][j] && colony_ids[j] == my_colony)
            .count() as u8;
        // Max possible neighbors in a 2D grid = 4
        // 4 = cardinal directions in 2D lattice (N/S/E/W)
        const MAX_NEIGHBORS_2D: u8 = 4;
        signals[i] = border_signal(neighbors, MAX_NEIGHBORS_2D);
    }
    signals
}

// ─── MC-4: Differential Expression ──────────────────────────────────────────

/// Modulate expression mask based on positional signal. Axiom 6: emerges from position.
///
/// Border (signal~1): push toward BORDER_TARGET [low, low, low, high] (defense).
/// Interior (signal~0): push toward INTERIOR_TARGET [high, high, high, low] (metabolism).
pub fn modulate_expression(
    border_sig: f32,
    current_mask: &[f32; 4],
    rate: f32,
) -> [f32; 4] {
    let sig = if border_sig.is_finite() { border_sig.clamp(0.0, 1.0) } else { 0.5 };
    let r = rate.clamp(0.0, 1.0);
    std::array::from_fn(|i| {
        let target = INTERIOR_TARGET[i] + (BORDER_TARGET[i] - INTERIOR_TARGET[i]) * sig;
        let new = current_mask[i] + r * (target - current_mask[i]);
        new.clamp(0.0, 1.0)
    })
}

/// Specialization index: how different are expression masks within a colony.
/// 0 = all identical. 1 = maximally different. Pure.
/// Specialization index: variance of expression within a colony. No heap.
pub fn specialization_index(
    masks: &[[f32; 4]],
    colony_ids: &[u8],
    target_colony: u8,
) -> f32 {
    let n = masks.len().min(colony_ids.len());
    // Two-pass: mean then variance. No Vec allocation.
    let count = (0..n).filter(|&i| colony_ids[i] == target_colony).count();
    if count < 2 { return 0.0; }
    let cf = count as f32;

    let mut variance_sum = 0.0f32;
    for dim in 0..4 {
        let mean: f32 = (0..n)
            .filter(|&i| colony_ids[i] == target_colony)
            .map(|i| masks[i][dim])
            .sum::<f32>() / cf;
        let var: f32 = (0..n)
            .filter(|&i| colony_ids[i] == target_colony)
            .map(|i| (masks[i][dim] - mean).powi(2))
            .sum::<f32>() / cf;
        variance_sum += var;
    }
    (variance_sum / 4.0).sqrt().min(1.0)
}

// ─── Helpers ────────────────────────────────────────────────────────────────

/// Axiom 8: frequency alignment. Centralized in determinism module.
fn gaussian_alignment(f_a: f32, f_b: f32, bandwidth: f32) -> f32 {
    super::determinism::gaussian_frequency_alignment(f_a, f_b, bandwidth)
}

// ─── Tests ──────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    // ── MC-1: Adhesion ──────────────────────────────────────────────────

    #[test] fn affinity_zero_when_far() {
        assert_eq!(adhesion_affinity(400.0, 400.0, 100.0, 1.0, 1.0), 0.0);
    }

    #[test] fn affinity_high_when_close_same_freq() {
        let a = adhesion_affinity(400.0, 400.0, 0.1, 1.0, 1.0);
        assert!(a > 0.9, "close + same freq = high affinity: {a}");
    }

    #[test] fn affinity_in_unit() {
        for d in [0.0, 0.5, 1.0, 2.0, 10.0] {
            let a = adhesion_affinity(400.0, 500.0, d, 1.0, 1.0);
            assert!(a >= 0.0 && a <= 1.0, "affinity out of range: {a}");
        }
    }

    #[test] fn affinity_decays_with_distance() {
        let near = adhesion_affinity(400.0, 400.0, 0.5, 1.0, 1.0);
        let far = adhesion_affinity(400.0, 400.0, 1.5, 1.0, 1.0);
        assert!(near > far, "Axiom 7: {near} > {far}");
    }

    #[test] fn affinity_decays_with_freq_diff() {
        let same = adhesion_affinity(400.0, 400.0, 0.5, 1.0, 1.0);
        let diff = adhesion_affinity(400.0, 800.0, 0.5, 1.0, 1.0);
        assert!(same > diff, "Axiom 8: {same} > {diff}");
    }

    #[test] fn bond_strength_positive() {
        assert!(bond_strength(0.8, 50.0, 40.0) > 0.0);
    }

    #[test] fn bond_cost_positive() {
        assert!(bond_cost(1.0) > 0.0);
    }

    #[test] fn should_bond_threshold() {
        assert!(!should_bond(0.1));
        assert!(should_bond(0.9));
    }

    #[test] fn affinity_nan_safe() {
        assert_eq!(adhesion_affinity(f32::NAN, 400.0, 1.0, 1.0, 1.0), 0.0);
    }

    // ── MC-2: Colony Detection ──────────────────────────────────────────

    fn empty_adj() -> [[bool; MAX_ENTITIES]; MAX_ENTITIES] { [[false; MAX_ENTITIES]; MAX_ENTITIES] }

    #[test] fn no_links_no_colonies() {
        let m = detect_colonies(&empty_adj(), 0b111);
        assert_eq!(m.colony_count, 0);
    }

    #[test] fn pair_too_small() {
        let mut adj = empty_adj();
        adj[0][1] = true; adj[1][0] = true;
        let m = detect_colonies(&adj, 0b11);
        assert_eq!(m.colony_count, 0, "pair < MIN_COLONY_SIZE=3");
    }

    #[test] fn three_linked_one_colony() {
        let mut adj = empty_adj();
        adj[0][1] = true; adj[1][0] = true;
        adj[1][2] = true; adj[2][1] = true;
        let m = detect_colonies(&adj, 0b111);
        assert_eq!(m.colony_count, 1);
        assert_eq!(m.max_colony_size, 3);
    }

    #[test] fn two_separate_groups() {
        let mut adj = empty_adj();
        // Group 1: 0-1-2
        adj[0][1] = true; adj[1][0] = true;
        adj[1][2] = true; adj[2][1] = true;
        // Group 2: 3-4-5
        adj[3][4] = true; adj[4][3] = true;
        adj[4][5] = true; adj[5][4] = true;
        let m = detect_colonies(&adj, 0b111111);
        assert_eq!(m.colony_count, 2);
    }

    #[test] fn dead_entities_excluded() {
        let mut adj = empty_adj();
        adj[0][1] = true; adj[1][0] = true;
        adj[1][2] = true; adj[2][1] = true;
        let m = detect_colonies(&adj, 0b011); // entity 2 dead
        assert_eq!(m.colony_count, 0, "only 2 alive = below MIN");
    }

    // ── MC-3: Positional Signal ─────────────────────────────────────────

    #[test] fn border_signal_no_neighbors_one() { assert_eq!(border_signal(0, 4), 1.0); }
    #[test] fn border_signal_all_neighbors_zero() { assert_eq!(border_signal(4, 4), 0.0); }
    #[test] fn border_signal_half() { assert!((border_signal(2, 4) - 0.5).abs() < 1e-5); }

    #[test] fn gradient_non_colony_zero() {
        let adj = empty_adj();
        let ids = [0u8; MAX_ENTITIES];
        let g = positional_gradient(&adj, &ids, 0b1);
        assert_eq!(g[0], 0.0, "not in colony → zero signal");
    }

    // ── MC-4: Differential Expression ───────────────────────────────────

    #[test] fn modulate_stays_in_unit() {
        let mask = [0.5; 4];
        for sig in [0.0, 0.5, 1.0] {
            let new = modulate_expression(sig, &mask, 0.1);
            for &v in &new { assert!(v >= 0.0 && v <= 1.0); }
        }
    }

    #[test] fn modulate_zero_rate_no_change() {
        let mask = [0.7, 0.3, 0.5, 0.9];
        let new = modulate_expression(0.5, &mask, 0.0);
        for i in 0..4 { assert!((new[i] - mask[i]).abs() < 1e-5); }
    }

    #[test] fn border_cell_high_resilience() {
        let mask = [0.5; 4];
        let new = modulate_expression(1.0, &mask, 0.5); // strong border signal
        assert!(new[3] > mask[3], "border → resilience up: {} > {}", new[3], mask[3]);
    }

    #[test] fn border_cell_low_growth() {
        let mask = [0.5; 4];
        let new = modulate_expression(1.0, &mask, 0.5);
        assert!(new[0] < mask[0], "border → growth down: {} < {}", new[0], mask[0]);
    }

    #[test] fn interior_cell_high_growth() {
        let mask = [0.5; 4];
        let new = modulate_expression(0.0, &mask, 0.5);
        assert!(new[0] > mask[0], "interior → growth up");
    }

    #[test] fn interior_cell_low_resilience() {
        let mask = [0.5; 4];
        let new = modulate_expression(0.0, &mask, 0.5);
        assert!(new[3] < mask[3], "interior → resilience down");
    }

    #[test] fn specialization_index_uniform_zero() {
        let masks = [[0.5; 4]; 3];
        let ids = [1u8, 1, 1];
        assert_eq!(specialization_index(&masks, &ids, 1), 0.0);
    }

    #[test] fn specialization_index_diverse_positive() {
        let masks = [[1.0, 0.0, 0.5, 0.0], [0.0, 1.0, 0.5, 1.0], [0.5, 0.5, 0.5, 0.5]];
        let ids = [1u8, 1, 1];
        assert!(specialization_index(&masks, &ids, 1) > 0.0);
    }

    #[test] fn modulate_nan_signal_safe() {
        let new = modulate_expression(f32::NAN, &[0.5; 4], 0.1);
        for &v in &new { assert!(v.is_finite()); }
    }
}
