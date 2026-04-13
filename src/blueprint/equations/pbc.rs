//! Periodic Boundary Conditions — pure math.
//!
//! MD-2: torus topology for bulk simulation. No surface artifacts.
//!
//! Axiom 7: distance attenuation holds on the torus manifold.
//! The physical distance IS the minimum image distance.

/// Wrap coordinate into [0, length). Torus topology.
#[inline]
pub fn wrap(x: f32, length: f32) -> f32 {
    x - length * (x / length).floor()
}

/// Minimum image displacement: shortest signed distance on a 1D torus.
///
/// Maps `dr` into (-length/2, length/2].
/// Standard MD convention: `dr - length * round(dr / length)`.
#[inline]
pub fn minimum_image(dr: f32, length: f32) -> f32 {
    dr - length * (dr / length + 0.5).floor()
}

/// Minimum image displacement vector in 2D. Returns (dx, dy).
#[inline]
pub fn minimum_image_2d(
    pos_a: [f32; 2],
    pos_b: [f32; 2],
    box_lengths: [f32; 2],
) -> [f32; 2] {
    [
        minimum_image(pos_b[0] - pos_a[0], box_lengths[0]),
        minimum_image(pos_b[1] - pos_a[1], box_lengths[1]),
    ]
}

// ─── f64 3D (MD-7) ─────────────────────────────────────────────────────────

/// Wrap f64 coordinate into [0, length). Torus topology.
#[inline]
pub fn wrap_f64(x: f64, length: f64) -> f64 {
    x - length * (x / length).floor()
}

/// Minimum image displacement (f64): shortest signed distance on a 1D torus.
#[inline]
pub fn minimum_image_f64(dr: f64, length: f64) -> f64 {
    dr - length * (dr / length + 0.5).floor()
}

/// Minimum image displacement vector in 3D (f64).
#[inline]
pub fn minimum_image_3d(
    pos_a: [f64; 3],
    pos_b: [f64; 3],
    box_lengths: [f64; 3],
) -> [f64; 3] {
    [
        minimum_image_f64(pos_b[0] - pos_a[0], box_lengths[0]),
        minimum_image_f64(pos_b[1] - pos_a[1], box_lengths[1]),
        minimum_image_f64(pos_b[2] - pos_a[2], box_lengths[2]),
    ]
}

/// Wrap 3D position into box [0, L)^3.
#[inline]
pub fn wrap_3d(pos: [f64; 3], box_lengths: [f64; 3]) -> [f64; 3] {
    [
        wrap_f64(pos[0], box_lengths[0]),
        wrap_f64(pos[1], box_lengths[1]),
        wrap_f64(pos[2], box_lengths[2]),
    ]
}

// ─── Tests ──────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    // ── wrap ────────────────────────────────────────────────────────────────

    #[test]
    fn wrap_identity_inside_box() {
        let l = 10.0;
        assert!((wrap(3.5, l) - 3.5).abs() < 1e-6);
        assert!((wrap(0.0, l) - 0.0).abs() < 1e-6);
        assert!((wrap(9.99, l) - 9.99).abs() < 1e-6);
    }

    #[test]
    fn wrap_positive_overflow() {
        assert!((wrap(10.1, 10.0) - 0.1).abs() < 1e-5);
        assert!((wrap(25.3, 10.0) - 5.3).abs() < 1e-4);
    }

    #[test]
    fn wrap_negative_underflow() {
        let w = wrap(-0.1, 10.0);
        assert!((w - 9.9).abs() < 1e-5, "got {w}");
        let w2 = wrap(-15.3, 10.0);
        assert!((w2 - 4.7).abs() < 1e-4, "got {w2}");
    }

    // ── minimum_image ──────────────────────────────────────────────────────

    #[test]
    fn minimum_image_small_displacement() {
        // dr < L/2 → unchanged
        let dr = minimum_image(2.0, 10.0);
        assert!((dr - 2.0).abs() < 1e-6);
        let dr_neg = minimum_image(-3.0, 10.0);
        assert!((dr_neg - (-3.0)).abs() < 1e-6);
    }

    #[test]
    fn minimum_image_large_displacement() {
        // dr = 8.0 on L=10 → should be -2.0 (wrap around)
        let dr = minimum_image(8.0, 10.0);
        assert!((dr - (-2.0)).abs() < 1e-5, "got {dr}");
    }

    #[test]
    fn minimum_image_negative_large() {
        // dr = -7.0 on L=10 → should be +3.0
        let dr = minimum_image(-7.0, 10.0);
        assert!((dr - 3.0).abs() < 1e-5, "got {dr}");
    }

    #[test]
    fn minimum_image_half_box() {
        // dr = L/2 exactly → maps to -L/2 or +L/2 (implementation-dependent)
        let dr = minimum_image(5.0, 10.0);
        assert!(dr.abs() <= 5.0 + 1e-6);
    }

    // ── minimum_image_2d ───────────────────────────────────────────────────

    #[test]
    fn minimum_image_2d_symmetric() {
        let bl = [10.0, 10.0];
        let d_ab = minimum_image_2d([1.0, 1.0], [9.0, 9.0], bl);
        let d_ba = minimum_image_2d([9.0, 9.0], [1.0, 1.0], bl);
        assert!((d_ab[0] + d_ba[0]).abs() < 1e-5, "dx symmetric");
        assert!((d_ab[1] + d_ba[1]).abs() < 1e-5, "dy symmetric");
    }

    #[test]
    fn minimum_image_2d_near_boundary() {
        // Particles at (0.1, 0) and (9.9, 0) in box L=10
        // Raw dx = 9.8, but minimum image dx = -0.2
        let d = minimum_image_2d([0.1, 0.0], [9.9, 0.0], [10.0, 10.0]);
        assert!((d[0] - (-0.2)).abs() < 1e-4, "dx={}", d[0]);
        assert!(d[1].abs() < 1e-6);
    }

    // ── 3D f64 (MD-7) ───────────────────────────────────────────────────────

    #[test]
    fn wrap_f64_positive_overflow() {
        assert!((wrap_f64(10.1, 10.0) - 0.1).abs() < 1e-12);
    }

    #[test]
    fn minimum_image_3d_symmetric() {
        let bl = [10.0, 10.0, 10.0];
        let a = [1.0, 1.0, 1.0];
        let b = [9.0, 9.0, 9.0];
        let d_ab = minimum_image_3d(a, b, bl);
        let d_ba = minimum_image_3d(b, a, bl);
        for dim in 0..3 {
            assert!((d_ab[dim] + d_ba[dim]).abs() < 1e-12, "dim {dim} symmetric");
        }
    }

    #[test]
    fn minimum_image_3d_near_boundary() {
        let d = minimum_image_3d([0.1, 5.0, 0.1], [9.9, 5.0, 9.9], [10.0, 10.0, 10.0]);
        assert!((d[0] - (-0.2)).abs() < 1e-10, "dx={}", d[0]);
        assert!(d[1].abs() < 1e-12, "dy={}", d[1]);
        assert!((d[2] - (-0.2)).abs() < 1e-10, "dz={}", d[2]);
    }

    #[test]
    fn wrap_3d_all_dimensions() {
        let p = wrap_3d([-0.5, 10.5, 5.0], [10.0, 10.0, 10.0]);
        assert!((p[0] - 9.5).abs() < 1e-12);
        assert!((p[1] - 0.5).abs() < 1e-12);
        assert!((p[2] - 5.0).abs() < 1e-12);
    }

    #[test]
    fn minimum_image_2d_distance_shorter_than_naive() {
        let bl = [10.0, 10.0];
        let a = [0.5, 0.5];
        let b = [9.5, 9.5];
        let d = minimum_image_2d(a, b, bl);
        let mi_dist_sq = d[0] * d[0] + d[1] * d[1];
        let naive_dx = b[0] - a[0];
        let naive_dy = b[1] - a[1];
        let naive_dist_sq = naive_dx * naive_dx + naive_dy * naive_dy;
        assert!(
            mi_dist_sq < naive_dist_sq,
            "minimum image {mi_dist_sq} should be shorter than naive {naive_dist_sq}",
        );
    }
}
