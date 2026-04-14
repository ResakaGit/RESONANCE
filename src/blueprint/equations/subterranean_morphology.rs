//! Morfología subterránea — topología constructal por gradiente de nutrientes.
//! Subterranean morphology — constructal topology from nutrient gradient.
//!
//! Not root-specific: any underground organ follows this optimization.

use crate::blueprint::constants::plant_physiology::CONCENTRATION_THRESHOLD;

/// Compute nutrient gradient direction from grid samples.
/// Calcular dirección del gradiente de nutrientes desde muestras del grid.
///
/// Returns (direction_x, direction_y, gradient_strength).
pub fn nutrient_gradient_direction(
    _center_value: f32,
    north_value: f32,
    south_value: f32,
    east_value: f32,
    west_value: f32,
) -> (f32, f32, f32) {
    let gx = east_value - west_value;
    let gy = north_value - south_value;
    let mag = (gx * gx + gy * gy).sqrt();
    if mag < 1e-6 {
        return (0.0, 0.0, 0.0);
    }
    (gx / mag, gy / mag, mag)
}

/// Optimal subterranean branch count from gradient strength and available energy.
/// Cantidad óptima de ramas subterráneas desde fuerza del gradiente y energía.
///
/// Strong gradient (concentrated deep) → 1 long structure (minimizes distance).
/// Weak gradient (dispersed) → N short structures (maximizes coverage).
/// Returns (count, relative_length) where length ∈ [0.3, 1.0].
pub fn constructal_branch_count(gradient_strength: f32, available_qe: f32) -> (u8, f32) {
    if available_qe <= 0.0 {
        return (0, 0.0);
    }
    if gradient_strength > CONCENTRATION_THRESHOLD {
        // Concentrated: 1 long tap structure
        (1, 1.0)
    } else {
        // Dispersed: multiple short structures, count scales with energy
        let count = (available_qe / 10.0).clamp(2.0, 8.0) as u8;
        let length = 0.3
            + 0.2 * (1.0 - gradient_strength / CONCENTRATION_THRESHOLD).clamp(0.0, 1.0);
        (count, length)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn gradient_uniform_returns_zero() {
        let (_, _, strength) = nutrient_gradient_direction(5.0, 5.0, 5.0, 5.0, 5.0);
        assert!(strength < 1e-5);
    }

    #[test]
    fn gradient_east_heavy_points_east() {
        let (gx, _, strength) = nutrient_gradient_direction(5.0, 5.0, 5.0, 10.0, 0.0);
        assert!(gx > 0.9);
        assert!(strength > 0.0);
    }

    #[test]
    fn strong_gradient_single_long_branch() {
        let (count, length) = constructal_branch_count(1.0, 50.0); // above threshold
        assert_eq!(count, 1);
        assert!((length - 1.0).abs() < 1e-5);
    }

    #[test]
    fn weak_gradient_multiple_short_branches() {
        let (count, length) = constructal_branch_count(0.1, 50.0); // below threshold
        assert!(count >= 2);
        assert!(length < 1.0);
    }

    #[test]
    fn zero_energy_no_branches() {
        let (count, _) = constructal_branch_count(0.5, 0.0);
        assert_eq!(count, 0);
    }
}
