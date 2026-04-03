//! Body plan equations: symmetry inference, lateral organ placement, allometric scaling.

use std::f32::consts::TAU;

// DEBT: Bevy prelude for Reflect derive + Vec3. Pure math should use crate::math_types
// and extract Reflect to bridge layer. Low priority: struct defs only, no runtime coupling.
use bevy::prelude::*;

use crate::blueprint::constants::inferred_world_geometry::{
    ALLOMETRIC_EXPONENT, LIMB_PAIR_Z_SPACING, LIMB_SPREAD_RATIO, ORGAN_SCALE_MAX, ORGAN_SCALE_MIN,
    ROLE_BASE_SCALE,
};
use crate::blueprint::{MAX_ORGANS_PER_ENTITY, OrganRole};
use crate::geometry_flow::SpineNode;
use crate::layers::OrganManifest;
use crate::layers::body_plan_layout::BodyPlanLayout;
use crate::worldgen::organ_inference::{
    ORGAN_ATTACHMENT_ZONE, organ_attachment_points, organ_orientation,
};

/// Body symmetry mode inferred from limb count.
#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Default, Reflect)]
pub enum SymmetryMode {
    #[default]
    Bilateral = 0,
    Radial = 1,
    Asymmetric = 2,
}

/// Infer symmetry mode from the number of limbs.
pub fn infer_symmetry_mode(limb_count: u8) -> SymmetryMode {
    match limb_count {
        0 => SymmetryMode::Bilateral,
        1 => SymmetryMode::Asymmetric,
        2 | 4 => SymmetryMode::Bilateral,
        _ => SymmetryMode::Radial,
    }
}

/// Compute lateral offset for an organ given its index, total count, symmetry, spine tangent, and spread radius.
pub fn lateral_offset(
    organ_index: u8,
    organ_count: u8,
    symmetry: SymmetryMode,
    spine_tangent: Vec3,
    spread_radius: f32,
) -> Vec3 {
    let spread = spread_radius.max(0.01);
    let count = organ_count.max(1);

    match symmetry {
        SymmetryMode::Bilateral => {
            let tangent_n = spine_tangent.normalize_or_zero();
            let side = tangent_n.cross(Vec3::Y).normalize_or_zero();
            let sign = if organ_index % 2 == 0 { 1.0 } else { -1.0 };
            let lateral = side * sign * spread;
            let pair_z = (organ_index / 2) as f32 * LIMB_PAIR_Z_SPACING * tangent_n;
            lateral + pair_z
        }
        SymmetryMode::Radial => {
            let tangent_n = spine_tangent.normalize_or_zero();
            let angle = TAU * organ_index as f32 / count as f32;
            // Build orthonormal basis in the plane perpendicular to tangent
            let u = tangent_n.any_orthonormal_vector();
            let v = tangent_n.cross(u);
            (u * angle.cos() + v * angle.sin()) * spread
        }
        SymmetryMode::Asymmetric => Vec3::ZERO,
    }
}

/// Allometric organ scale: sublinear scaling from biomass, divided by organ count, weighted by role.
pub fn allometric_organ_scale(role: OrganRole, biomass: f32, organ_count: u8) -> f32 {
    let base = biomass.max(0.01).powf(ALLOMETRIC_EXPONENT);
    let count = organ_count.max(1) as f32;
    let role_weight = ROLE_BASE_SCALE[role as usize];
    (base / count * role_weight).clamp(ORGAN_SCALE_MIN, ORGAN_SCALE_MAX)
}

/// Count limb-role organs across an entire manifest.
pub fn count_limbs_in_manifest(manifest: &OrganManifest) -> u8 {
    manifest
        .iter()
        .filter(|spec| spec.role() == OrganRole::Limb)
        .map(|spec| spec.count())
        .fold(0u8, |acc, c| acc.saturating_add(c))
}

/// Compute a full body plan layout from manifest, spine, spread radius, and energy direction.
pub fn compute_body_plan_layout(
    manifest: &OrganManifest,
    spine: &[SpineNode],
    spread_radius: f32,
    energy_direction: Vec3,
) -> BodyPlanLayout {
    let mut positions = [Vec3::ZERO; MAX_ORGANS_PER_ENTITY];
    let mut directions = [Vec3::Y; MAX_ORGANS_PER_ENTITY];
    let mut slot = 0usize;

    if manifest.is_empty() || spine.len() < 2 {
        return BodyPlanLayout::default();
    }

    let limb_count = count_limbs_in_manifest(manifest);
    let symmetry = infer_symmetry_mode(limb_count);

    // For lateral offset on limbs we need a running index per limb-role spec.
    let mut limb_global_index: u8 = 0;

    for spec in manifest.iter() {
        let role = spec.role();
        // Skip trunk roles (Stem, Core) — their mesh comes from the branching tree.
        if matches!(role, OrganRole::Stem | OrganRole::Core) {
            continue;
        }
        let zone = ORGAN_ATTACHMENT_ZONE[role as usize];
        let attachments = organ_attachment_points(spine, spec.count(), zone);

        for attachment in &attachments {
            if slot >= MAX_ORGANS_PER_ENTITY {
                break;
            }

            let base_pos = attachment.position;
            let offset = if role == OrganRole::Limb {
                let off = lateral_offset(
                    limb_global_index,
                    limb_count,
                    symmetry,
                    attachment.tangent,
                    spread_radius * LIMB_SPREAD_RATIO,
                );
                limb_global_index = limb_global_index.saturating_add(1);
                off
            } else {
                Vec3::ZERO
            };

            let (normal_out, _tangent_out) = organ_orientation(role, attachment, energy_direction);

            positions[slot] = base_pos + offset;
            directions[slot] = normal_out;
            slot += 1;
        }

        if slot >= MAX_ORGANS_PER_ENTITY {
            break;
        }
    }

    BodyPlanLayout::new(positions, directions, symmetry, slot as u8)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::blueprint::OrganRole;
    use crate::layers::{OrganManifest, OrganSpec};

    // --- infer_symmetry_mode ---

    #[test]
    fn infer_symmetry_mode_zero_returns_bilateral() {
        assert_eq!(infer_symmetry_mode(0), SymmetryMode::Bilateral);
    }

    #[test]
    fn infer_symmetry_mode_one_returns_asymmetric() {
        assert_eq!(infer_symmetry_mode(1), SymmetryMode::Asymmetric);
    }

    #[test]
    fn infer_symmetry_mode_two_returns_bilateral() {
        assert_eq!(infer_symmetry_mode(2), SymmetryMode::Bilateral);
    }

    #[test]
    fn infer_symmetry_mode_four_returns_bilateral() {
        assert_eq!(infer_symmetry_mode(4), SymmetryMode::Bilateral);
    }

    #[test]
    fn infer_symmetry_mode_three_returns_radial() {
        assert_eq!(infer_symmetry_mode(3), SymmetryMode::Radial);
    }

    #[test]
    fn infer_symmetry_mode_five_returns_radial() {
        assert_eq!(infer_symmetry_mode(5), SymmetryMode::Radial);
    }

    #[test]
    fn infer_symmetry_mode_six_returns_radial() {
        assert_eq!(infer_symmetry_mode(6), SymmetryMode::Radial);
    }

    // --- lateral_offset ---

    #[test]
    fn bilateral_four_limbs_symmetric_pairs() {
        let tangent = Vec3::Z;
        let offsets: Vec<Vec3> = (0..4)
            .map(|i| lateral_offset(i, 4, SymmetryMode::Bilateral, tangent, 1.0))
            .collect();
        // Pair 0: indices 0 and 1 should mirror in lateral (x) axis
        assert!((offsets[0].x + offsets[1].x).abs() < 1e-5);
        // Pair 1: indices 2 and 3 should also mirror
        assert!((offsets[2].x + offsets[3].x).abs() < 1e-5);
    }

    #[test]
    fn radial_five_limbs_equidistant_angles() {
        let tangent = Vec3::Y;
        let offsets: Vec<Vec3> = (0..5)
            .map(|i| lateral_offset(i, 5, SymmetryMode::Radial, tangent, 1.0))
            .collect();
        // All offsets should have the same magnitude (spread_radius)
        for off in &offsets {
            assert!(
                (off.length() - 1.0).abs() < 1e-4,
                "length was {}",
                off.length()
            );
        }
        // Angle between consecutive should be equal (TAU/5)
        let expected_angle = TAU / 5.0;
        for i in 0..5 {
            let a = offsets[i];
            let b = offsets[(i + 1) % 5];
            let cos_angle = a.dot(b) / (a.length() * b.length());
            let angle = cos_angle.clamp(-1.0, 1.0).acos();
            assert!(
                (angle - expected_angle).abs() < 1e-3,
                "angle between {i} and {} was {angle}, expected {expected_angle}",
                (i + 1) % 5
            );
        }
    }

    #[test]
    fn asymmetric_returns_zero() {
        let off = lateral_offset(0, 1, SymmetryMode::Asymmetric, Vec3::Z, 2.0);
        assert_eq!(off, Vec3::ZERO);
    }

    #[test]
    fn spread_radius_zero_guard() {
        // Should not panic or produce NaN with zero spread
        let off = lateral_offset(0, 2, SymmetryMode::Bilateral, Vec3::Z, 0.0);
        assert!(off.x.is_finite());
        assert!(off.y.is_finite());
        assert!(off.z.is_finite());
    }

    #[test]
    fn lateral_offset_determinism() {
        let a = lateral_offset(2, 6, SymmetryMode::Radial, Vec3::new(1.0, 0.5, 0.0), 0.8);
        let b = lateral_offset(2, 6, SymmetryMode::Radial, Vec3::new(1.0, 0.5, 0.0), 0.8);
        assert_eq!(a, b);
    }

    // --- allometric_organ_scale ---

    #[test]
    fn allometric_scale_sublinear() {
        // Use Sensory (base_scale 0.3) so results stay well within [MIN, MAX]
        let s1 = allometric_organ_scale(OrganRole::Sensory, 1.0, 1);
        let s8 = allometric_organ_scale(OrganRole::Sensory, 8.0, 1);
        // 8^0.75 = 4.7568..., both values unclamped → ratio ≈ 4.76
        let ratio = s8 / s1;
        assert!(
            (ratio - 4.756).abs() < 0.1,
            "ratio was {ratio}, expected ~4.756"
        );
    }

    #[test]
    fn allometric_scale_divides_by_count() {
        let s1 = allometric_organ_scale(OrganRole::Leaf, 10.0, 1);
        let s4 = allometric_organ_scale(OrganRole::Leaf, 10.0, 4);
        assert!((s1 / s4 - 4.0).abs() < 0.01);
    }

    #[test]
    fn allometric_scale_clamped() {
        let tiny = allometric_organ_scale(OrganRole::Bud, 0.001, 16);
        assert!(tiny >= ORGAN_SCALE_MIN);
        let huge = allometric_organ_scale(OrganRole::Core, 1_000_000.0, 1);
        assert!(huge <= ORGAN_SCALE_MAX);
    }

    #[test]
    fn allometric_scale_zero_biomass() {
        let s = allometric_organ_scale(OrganRole::Stem, 0.0, 1);
        assert!(s >= ORGAN_SCALE_MIN);
        assert!(s.is_finite());
    }

    // --- count_limbs_in_manifest ---

    #[test]
    fn count_limbs_in_manifest_sums_limb_specs() {
        let mut manifest = OrganManifest::default();
        manifest.push(OrganSpec::new(OrganRole::Limb, 4, 1.0));
        manifest.push(OrganSpec::new(OrganRole::Leaf, 6, 0.5));
        manifest.push(OrganSpec::new(OrganRole::Limb, 2, 0.8));
        assert_eq!(count_limbs_in_manifest(&manifest), 6);
    }

    #[test]
    fn count_limbs_in_manifest_empty_returns_zero() {
        let manifest = OrganManifest::default();
        assert_eq!(count_limbs_in_manifest(&manifest), 0);
    }

    // --- compute_body_plan_layout ---

    fn sample_spine() -> Vec<SpineNode> {
        (0..10)
            .map(|i| {
                let t = i as f32 / 9.0;
                SpineNode {
                    position: Vec3::new(0.0, t * 3.0, 0.0),
                    tangent: Vec3::Y,
                    tint_rgb: [0.5, 0.5, 0.5],
                    qe_norm: 0.5,
                }
            })
            .collect()
    }

    #[test]
    fn compute_body_plan_layout_bilateral_quadruped() {
        let mut manifest = OrganManifest::default();
        manifest.push(OrganSpec::new(OrganRole::Core, 1, 1.0));
        manifest.push(OrganSpec::new(OrganRole::Stem, 1, 1.0));
        manifest.push(OrganSpec::new(OrganRole::Limb, 4, 1.0));
        manifest.push(OrganSpec::new(OrganRole::Sensory, 1, 0.5));

        let spine = sample_spine();
        let layout = compute_body_plan_layout(&manifest, &spine, 0.5, Vec3::Y);

        // Trunk (Core, Stem) skipped. Limb(4) + Sensory(1) = 5 slots.
        assert_eq!(layout.active_count(), 5);
        assert_eq!(layout.symmetry(), SymmetryMode::Bilateral);

        // Limb organs (indices 0..4): pairs should have mirrored lateral offsets.
        let p0 = layout.position(0);
        let p1 = layout.position(1);
        // Bilateral limbs: even index one side, odd index other side.
        assert!(
            (p0.x + p1.x).abs() < 0.5 || (p0.z + p1.z).abs() < 0.5,
            "first limb pair should be approximately mirrored"
        );
    }

    #[test]
    fn compute_body_plan_layout_radial_five() {
        let mut manifest = OrganManifest::default();
        manifest.push(OrganSpec::new(OrganRole::Core, 1, 1.0));
        manifest.push(OrganSpec::new(OrganRole::Limb, 5, 1.0));

        let spine = sample_spine();
        let layout = compute_body_plan_layout(&manifest, &spine, 0.5, Vec3::Y);

        // Core skipped. Limb(5) = 5 slots.
        assert_eq!(layout.active_count(), 5);
        assert_eq!(layout.symmetry(), SymmetryMode::Radial);
    }

    #[test]
    fn compute_body_plan_layout_empty_manifest() {
        let manifest = OrganManifest::default();
        let spine = sample_spine();
        let layout = compute_body_plan_layout(&manifest, &spine, 0.5, Vec3::Y);
        assert_eq!(layout.active_count(), 0);
    }

    #[test]
    fn compute_body_plan_layout_determinism() {
        let mut manifest = OrganManifest::default();
        manifest.push(OrganSpec::new(OrganRole::Stem, 1, 1.0));
        manifest.push(OrganSpec::new(OrganRole::Limb, 4, 1.0));
        manifest.push(OrganSpec::new(OrganRole::Leaf, 3, 0.7));

        let spine = sample_spine();
        let a = compute_body_plan_layout(&manifest, &spine, 0.8, Vec3::new(1.0, 0.5, 0.0));
        let b = compute_body_plan_layout(&manifest, &spine, 0.8, Vec3::new(1.0, 0.5, 0.0));
        assert_eq!(a, b);
    }
}
