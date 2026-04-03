//! GF2A — Tensores de deformación termodinámica (stateless).
//!
//! Toda la física de deformación del spine vive aquí.
//! Ningún símbolo ECS, ningún estado.

use crate::math_types::Vec3;

/// Delta de deformación por tensores físicos.
///
/// `bond_energy` alto → material rígido → delta pequeño.
/// `tangent` es el eje axial del nodo; el delta se aplica perpendicular a él.
///
/// `δ = tangent_perp · (1 − bond) · normalize(t_energy + t_gravity)`
pub fn deformation_delta(tangent: Vec3, t_energy: Vec3, t_gravity: Vec3, bond_energy: f32) -> Vec3 {
    let combined = t_energy + t_gravity;
    if combined.length_squared() < 1e-12 {
        return Vec3::ZERO;
    }
    let combined_n = combined.normalize_or_zero();
    let rigidity = 1.0 - bond_energy.clamp(0.0, 1.0);
    // Componente perpendicular a la tangente del spine.
    let t_n = tangent.normalize_or_zero();
    let parallel = t_n * combined_n.dot(t_n);
    let perp = combined_n - parallel;
    perp * rigidity
}

/// Vector de tropismo: energía + gravedad, BRANCHLESS.
///
/// Retorna `(energy_tropism, gravity_tropism)`.
/// `energy_tropism = energy_direction * (absorbed_energy / (bond_energy + 1.0))`
/// `gravity_tropism = Vec3::NEG_Y * gravity_scale * absorbed_energy`
/// Gradiente de energía desde vecinos 4-connected (diferencias finitas centrales).
///
/// `∇qe ≈ ((right − left) / (2·cs), 0, (up − down) / (2·cs))`
pub fn energy_gradient_from_neighbors(
    left_qe: f32,
    right_qe: f32,
    down_qe: f32,
    up_qe: f32,
    cell_size: f32,
) -> Vec3 {
    let inv_2cs = 1.0 / (2.0 * cell_size.max(1e-6));
    Vec3::new(
        (right_qe - left_qe) * inv_2cs,
        0.0,
        (up_qe - down_qe) * inv_2cs,
    )
}

pub fn calculate_tropism_vector(
    absorbed_energy: f32,
    bond_energy: f32,
    energy_direction: Vec3,
    gravity_scale: f32,
) -> (Vec3, Vec3) {
    let energy_tropism = energy_direction * (absorbed_energy / (bond_energy + 1.0));
    let gravity_tropism = Vec3::NEG_Y * gravity_scale * absorbed_energy;
    (energy_tropism, gravity_tropism)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deformation_delta_high_bond_energy_returns_near_zero() {
        let tangent = Vec3::Y;
        let t_energy = Vec3::X * 5.0;
        let t_gravity = Vec3::NEG_Y * 9.8;
        let delta = deformation_delta(tangent, t_energy, t_gravity, 1.0);
        assert!(
            delta.length() < 1e-6,
            "bond_energy=1 must yield zero delta, got {delta:?}"
        );
    }

    #[test]
    fn deformation_delta_zero_bond_energy_returns_significant() {
        let tangent = Vec3::Y;
        let t_energy = Vec3::X * 5.0;
        let t_gravity = Vec3::ZERO;
        let delta = deformation_delta(tangent, t_energy, t_gravity, 0.0);
        assert!(
            delta.length() > 0.1,
            "bond_energy=0 must yield non-zero delta, got {delta:?}"
        );
    }

    #[test]
    fn deformation_delta_no_forces_returns_zero() {
        let delta = deformation_delta(Vec3::Y, Vec3::ZERO, Vec3::ZERO, 0.0);
        assert_eq!(delta, Vec3::ZERO);
    }

    #[test]
    fn calculate_tropism_vector_is_deterministic() {
        let a = calculate_tropism_vector(1.5, 0.3, Vec3::new(0.5, 1.0, -0.2), 9.8);
        let b = calculate_tropism_vector(1.5, 0.3, Vec3::new(0.5, 1.0, -0.2), 9.8);
        assert_eq!(a.0, b.0);
        assert_eq!(a.1, b.1);
    }

    #[test]
    fn calculate_tropism_vector_zero_energy_returns_zero_vectors() {
        let (et, gt) = calculate_tropism_vector(0.0, 0.5, Vec3::Y, 9.8);
        assert_eq!(et, Vec3::ZERO);
        assert_eq!(gt, Vec3::ZERO);
    }

    #[test]
    fn calculate_tropism_vector_gravity_always_points_neg_y() {
        let (_et, gt) = calculate_tropism_vector(1.0, 0.0, Vec3::X, 1.0);
        assert!(gt.y < 0.0, "gravity tropism must have negative Y component");
        assert_eq!(gt.x, 0.0);
        assert_eq!(gt.z, 0.0);
    }

    #[test]
    fn energy_gradient_uniform_field_returns_zero() {
        let g = energy_gradient_from_neighbors(10.0, 10.0, 10.0, 10.0, 2.0);
        assert!(g.length() < 1e-6, "uniform field must have zero gradient");
    }

    #[test]
    fn energy_gradient_right_higher_points_positive_x() {
        let g = energy_gradient_from_neighbors(0.0, 100.0, 50.0, 50.0, 1.0);
        assert!(g.x > 0.0, "gradient must point toward higher energy");
        assert_eq!(g.y, 0.0);
    }

    #[test]
    fn energy_gradient_up_higher_points_positive_z() {
        let g = energy_gradient_from_neighbors(50.0, 50.0, 0.0, 100.0, 1.0);
        assert!(g.z > 0.0, "gradient must point toward higher energy (up)");
        assert_eq!(g.x, 0.0);
    }
}
