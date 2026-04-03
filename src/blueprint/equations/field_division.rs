//! AD-3: Valley detection and field splitting for axiomatic cell division.
//!
//! Pure math. Zero Bevy. Zero thresholds.
//! Split condition: valley.qe ≤ 0 (Axiom 1: no energy = no existence).
//! Conservation: sum(left) + sum(right) ≤ sum(original) (Axiom 2).

use crate::blueprint::equations::derived_thresholds;

/// Find valley indices in a 1D field: nodes where qe is a local minimum.
/// Returns (index, qe_at_valley) pairs. A valley at i: field[i] < field[i-1] AND field[i] < field[i+1].
pub fn find_valleys(field: &[f32; 8]) -> [(usize, f32); 8] {
    let mut valleys = [(0usize, 0.0f32); 8];
    let mut count = 0;
    for i in 1..7 {
        if field[i] < field[i - 1] && field[i] < field[i + 1] && count < 8 {
            valleys[count] = (i, field[i]);
            count += 1;
        }
    }
    valleys
}

/// Count valid valleys (index > 0 or first entry has qe data).
pub fn valley_count(valleys: &[(usize, f32); 8]) -> usize {
    valleys.iter().take_while(|v| v.0 > 0 || v.1 != 0.0).count()
}

/// A valley is a split point when its qe ≤ 0.
/// Axiom 1 pure: no energy = no existence = disconnection.
/// Zero threshold. Zero constants.
#[inline]
pub fn is_split_viable(field: &[f32; 8], valley_idx: usize) -> bool {
    valley_idx > 0 && valley_idx < 7 && field[valley_idx] <= 0.0
}

/// Split the field at a valley index. Returns (left_nodes, right_nodes).
///
/// Conservation (Axiom 2): sum(left) + sum(right) ≤ sum(original).
/// The valley node's energy is lost (dissipated at the break point).
/// Each half is padded with zeros to fill [f32; 8].
pub fn split_field_at(field: &[f32; 8], valley_idx: usize) -> ([f32; 8], [f32; 8]) {
    let mut left = [0.0f32; 8];
    let mut right = [0.0f32; 8];

    // Left child gets nodes [0..valley_idx)
    for i in 0..valley_idx {
        left[i] = field[i];
    }
    // Right child gets nodes (valley_idx..8]
    for i in (valley_idx + 1)..8 {
        right[i - valley_idx - 1] = field[i];
    }
    // Valley node energy is dissipated (Axiom 4)
    (left, right)
}

/// Check if a child field has enough energy to sustain existence.
/// Uses self_sustaining_qe_min() from derived_thresholds (Axiom 1 + 4).
pub fn child_viable(field: &[f32; 8]) -> bool {
    let total: f32 = field.iter().sum();
    total >= derived_thresholds::self_sustaining_qe_min()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn no_valleys_in_monotonic() {
        let field = [1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0];
        assert_eq!(valley_count(&find_valleys(&field)), 0);
    }

    #[test]
    fn no_valleys_in_uniform() {
        let field = [5.0; 8];
        assert_eq!(valley_count(&find_valleys(&field)), 0);
    }

    #[test]
    fn center_valley_detected() {
        let field = [10.0, 15.0, 20.0, 2.0, 20.0, 15.0, 10.0, 5.0];
        let valleys = find_valleys(&field);
        assert_eq!(valley_count(&valleys), 1);
        assert_eq!(valleys[0].0, 3);
        assert!((valleys[0].1 - 2.0).abs() < 1e-5);
    }

    #[test]
    fn two_valleys() {
        let field = [10.0, 1.0, 10.0, 1.0, 10.0, 5.0, 3.0, 2.0];
        let valleys = find_valleys(&field);
        assert_eq!(valley_count(&valleys), 2);
    }

    #[test]
    fn split_viable_only_at_zero() {
        let field = [10.0, 5.0, 0.0, 5.0, 10.0, 5.0, 3.0, 2.0];
        assert!(is_split_viable(&field, 2));
        let field2 = [10.0, 5.0, 0.1, 5.0, 10.0, 5.0, 3.0, 2.0];
        assert!(!is_split_viable(&field2, 2));
    }

    #[test]
    fn split_conserves_energy() {
        let field = [10.0, 8.0, 0.0, 8.0, 10.0, 6.0, 4.0, 2.0];
        let original_total: f32 = field.iter().sum();
        let (left, right) = split_field_at(&field, 2);
        let left_total: f32 = left.iter().sum();
        let right_total: f32 = right.iter().sum();
        assert!(
            left_total + right_total <= original_total + 1e-5,
            "conservation: {} + {} ≤ {}",
            left_total,
            right_total,
            original_total
        );
    }

    #[test]
    fn split_at_center_produces_two_halves() {
        let field = [10.0, 8.0, 6.0, 0.0, 6.0, 8.0, 10.0, 5.0];
        let (left, right) = split_field_at(&field, 3);
        assert!((left[0] - 10.0).abs() < 1e-5);
        assert!((left[1] - 8.0).abs() < 1e-5);
        assert!((left[2] - 6.0).abs() < 1e-5);
        assert!((right[0] - 6.0).abs() < 1e-5);
        assert!((right[1] - 8.0).abs() < 1e-5);
    }

    #[test]
    fn child_viable_above_min() {
        let field = [5.0, 5.0, 5.0, 5.0, 0.0, 0.0, 0.0, 0.0]; // total = 20
        assert!(child_viable(&field));
    }

    #[test]
    fn child_not_viable_below_min() {
        let field = [1.0, 1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0]; // total = 2
        assert!(!child_viable(&field));
    }

    #[test]
    fn edge_valley_not_split_viable() {
        let field = [0.0, 5.0, 10.0, 10.0, 10.0, 5.0, 3.0, 0.0];
        assert!(!is_split_viable(&field, 0)); // edge
        assert!(!is_split_viable(&field, 7)); // edge
    }
}
