//! Constructal body plan inference: appendage count from thermodynamic cost minimization.
//!
//! Replaces hardcoded bilateral quadruped when entity has full physics layers (L6).
//! Reads L0 (BaseEnergy), L1 (SpatialVolume), L3 (FlowVector), L6 (AmbientPressure),
//! MorphogenesisShapeParams, InferenceProfile, CapabilitySet.
//! Writes BodyPlanLayout via `compute_body_plan_layout` from existing body plan equations.

use bevy::prelude::*;

use crate::blueprint::constants::{
    CONSTRUCTAL_LIMB_LENGTH_RATIO, CONSTRUCTAL_LIMB_RADIUS_RATIO,
    CONSTRUCTAL_VISCOSITY, MAX_CONSTRUCTAL_LIMBS,
};
use crate::blueprint::equations::optimal_appendage_count;
use crate::geometry_flow::{build_flow_spine, GeometryInfluence};
use crate::blueprint::constants::FINENESS_DEFAULT;
use crate::layers::{
    AmbientPressure, BodyPlanLayout, CapabilitySet, FlowVector,
    HasInferredShape, InferenceProfile, MorphogenesisShapeParams, SpatialVolume,
};

/// Infers body plan from constructal optimization of appendage count.
///
/// Only runs for entities with HasInferredShape + MOVE capability + AmbientPressure (L6).
/// Disjoint from `body_plan_layout_inference_system` (which handles entities without L6).
pub fn constructal_body_plan_system(
    mut commands: Commands,
    query: Query<
        (
            Entity,
            &SpatialVolume,
            &FlowVector,
            &AmbientPressure,
            Option<&MorphogenesisShapeParams>,
            Option<&InferenceProfile>,
            &CapabilitySet,
        ),
        (
            With<HasInferredShape>,
            Or<(
                Changed<SpatialVolume>,
                Changed<FlowVector>,
                Changed<AmbientPressure>,
                Changed<MorphogenesisShapeParams>,
                Without<BodyPlanLayout>,
            )>,
        ),
    >,
) {
    for (entity, volume, flow, pressure, shape_opt, profile_opt, caps) in &query {
        if !caps.has(CapabilitySet::MOVE) { continue; }

        let radius    = volume.radius;
        let fineness  = shape_opt.map(|s| s.fineness_ratio()).unwrap_or(FINENESS_DEFAULT);
        let velocity  = flow.speed(); // Axiom 1: only real energy state drives form
        let density   = pressure.terrain_viscosity;
        let limb_len  = radius * CONSTRUCTAL_LIMB_LENGTH_RATIO;
        let limb_r    = radius * CONSTRUCTAL_LIMB_RADIUS_RATIO;

        let n = optimal_appendage_count(
            radius, fineness, density, velocity, CONSTRUCTAL_VISCOSITY,
            limb_len, limb_r, MAX_CONSTRUCTAL_LIMBS,
        );

        // Build minimal spine for layout computation
        let mobility = profile_opt.map(|p| p.mobility_bias).unwrap_or(0.5);
        let spread   = radius * (0.3 + mobility * 0.4);
        let influence = GeometryInfluence {
            detail: 0.5,
            energy_direction: Vec3::Y,
            energy_strength: velocity,
            resistance: 0.5,
            least_resistance_direction: Vec3::X,
            length_budget: radius * fineness * 2.0,
            max_segments: 4,
            radius_base: radius,
            start_position: Vec3::ZERO,
            qe_norm: 0.5,
            tint_rgb: [0.5; 3],
            branch_role: crate::blueprint::equations::BranchRole::Stem,
        };
        let spine = build_flow_spine(&influence);

        let layout = compute_body_plan_layout_from_count(n, &spine, spread, influence.energy_direction);
        commands.entity(entity).insert(layout);
    }
}

/// Builds a `BodyPlanLayout` from an optimized appendage count + spine.
fn compute_body_plan_layout_from_count(
    limb_count: u8,
    spine: &[crate::geometry_flow::SpineNode],
    spread: f32,
    forward: Vec3,
) -> BodyPlanLayout {
    use crate::blueprint::equations::SymmetryMode;
    use crate::layers::organ::MAX_ORGANS_PER_ENTITY;

    let mut positions  = [Vec3::ZERO; MAX_ORGANS_PER_ENTITY];
    let mut directions = [Vec3::Y;    MAX_ORGANS_PER_ENTITY];

    let spine_len = spine.len();
    let spine_mid = if spine_len > 2 { spine_len / 2 } else { 0 };
    let body_center = if spine_len > 0 { spine[spine_mid].position } else { Vec3::ZERO };
    let body_tip    = if spine_len > 1 { spine[spine_len - 1].position } else { body_center + forward * 0.5 };
    let body_rear   = if spine_len > 0 { spine[0].position } else { body_center - forward * 0.5 };

    let mut slot = 0usize;

    // Head — forward tip
    if slot < MAX_ORGANS_PER_ENTITY {
        positions[slot]  = body_tip;
        directions[slot] = forward;
        slot += 1;
    }

    // Tail — rear
    if slot < MAX_ORGANS_PER_ENTITY {
        positions[slot]  = body_rear;
        directions[slot] = -forward;
        slot += 1;
    }

    // Limbs — bilateral pairs distributed along spine
    let pairs = (limb_count / 2).min((MAX_ORGANS_PER_ENTITY - slot) as u8 / 2);
    let side = forward.cross(Vec3::Y).normalize_or(Vec3::X);

    for i in 0..pairs {
        let t = if pairs > 1 { i as f32 / (pairs - 1) as f32 } else { 0.5 };
        let spine_idx = ((spine_len as f32 * (0.3 + t * 0.4)) as usize).min(spine_len.saturating_sub(1));
        let anchor = if spine_idx < spine_len { spine[spine_idx].position } else { body_center };
        let leg_dir = (side * 0.3 + Vec3::NEG_Y * 0.9).normalize_or(Vec3::NEG_Y);

        if slot + 1 < MAX_ORGANS_PER_ENTITY {
            positions[slot]      = anchor + side * spread;
            directions[slot]     = leg_dir;
            positions[slot + 1]  = anchor - side * spread;
            directions[slot + 1] = Vec3::new(-leg_dir.x, leg_dir.y, leg_dir.z);
            slot += 2;
        }
    }

    // Odd limb — single appendage at center (tail fin, dorsal, etc.)
    if limb_count % 2 == 1 && slot < MAX_ORGANS_PER_ENTITY {
        positions[slot]  = body_center + Vec3::Y * spread * 0.5;
        directions[slot] = Vec3::Y;
        slot += 1;
    }

    let symmetry = if limb_count > 0 { SymmetryMode::Bilateral } else { SymmetryMode::Radial };
    BodyPlanLayout::new(positions, directions, symmetry, slot.min(u8::MAX as usize) as u8)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn zero_limbs_produces_head_and_tail_only() {
        use crate::geometry_flow::SpineNode;
        let spine = vec![
            SpineNode { position: Vec3::ZERO, tangent: Vec3::Y, tint_rgb: [0.5; 3], qe_norm: 0.5 },
            SpineNode { position: Vec3::Y, tangent: Vec3::Y, tint_rgb: [0.5; 3], qe_norm: 0.5 },
        ];
        let layout = compute_body_plan_layout_from_count(0, &spine, 0.3, Vec3::Y);
        assert_eq!(layout.active_count(), 2, "head + tail = 2 slots");
    }

    #[test]
    fn four_limbs_produces_bilateral_layout() {
        use crate::geometry_flow::SpineNode;
        let spine: Vec<SpineNode> = (0..6).map(|i| SpineNode {
            position: Vec3::Y * i as f32 * 0.2,
            tangent: Vec3::Y,
            tint_rgb: [0.5; 3],
            qe_norm: 0.5,
        }).collect();
        let layout = compute_body_plan_layout_from_count(4, &spine, 0.3, Vec3::Y);
        // head(1) + tail(1) + 2 pairs(4) = 6
        assert_eq!(layout.active_count(), 6, "expected 6 slots for 4 limbs, got {}", layout.active_count());
    }

    #[test]
    fn odd_limb_count_adds_dorsal() {
        use crate::geometry_flow::SpineNode;
        let spine = vec![
            SpineNode { position: Vec3::ZERO, tangent: Vec3::Y, tint_rgb: [0.5; 3], qe_norm: 0.5 },
            SpineNode { position: Vec3::Y, tangent: Vec3::Y, tint_rgb: [0.5; 3], qe_norm: 0.5 },
        ];
        let layout = compute_body_plan_layout_from_count(3, &spine, 0.3, Vec3::Y);
        // head(1) + tail(1) + 1 pair(2) + dorsal(1) = 5
        assert_eq!(layout.active_count(), 5, "expected 5 slots for 3 limbs, got {}", layout.active_count());
    }

    #[test]
    fn layout_positions_are_finite() {
        use crate::geometry_flow::SpineNode;
        let spine: Vec<SpineNode> = (0..4).map(|i| SpineNode {
            position: Vec3::Y * i as f32 * 0.3,
            tangent: Vec3::Y,
            tint_rgb: [0.5; 3],
            qe_norm: 0.5,
        }).collect();
        let layout = compute_body_plan_layout_from_count(6, &spine, 0.5, Vec3::Y);
        for i in 0..layout.active_count() as usize {
            let pos = layout.position(i);
            assert!(pos.is_finite(), "position {i} not finite: {pos}");
        }
    }
}
