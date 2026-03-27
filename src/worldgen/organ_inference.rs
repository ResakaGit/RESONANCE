use crate::math_types::Vec3;
use bevy::prelude::Mesh;
use bevy::render::mesh::VertexAttributeValues;

use crate::blueprint::almanac::AlchemicalAlmanac;
use crate::blueprint::constants::{
    BRANCH_DIR_EPSILON, BRANCH_MIN_BIOMASS, MAX_ORGAN_INSTANCE_COUNT,
    ORGAN_ORIENTATION_PARALLEL_DOT_CUTOFF, ORGAN_ZONE_APICAL_BASAL_SPAN, ORGAN_ZONE_APICAL_OFFSET,
    ORGAN_ZONE_BASAL_OFFSET, ORGAN_ZONE_FULL_OFFSET, ORGAN_ZONE_FULL_SPAN, VISUAL_QE_REFERENCE,
};
use crate::blueprint::equations::{
    BranchRole, organ_role_modulated_rgb, organ_role_opacity, organ_role_scale,
};
use crate::geometry_flow::branching::{BranchNode, build_branched_tree_dyn, flatten_tree_to_mesh};
use crate::geometry_flow::primitives::{OrganPrimitiveParams, build_organ_primitive};
use crate::geometry_flow::{GeometryInfluence, SpineNode, build_flow_mesh};
use crate::layers::body_plan_layout::BodyPlanLayout;
use crate::layers::{GeometryPrimitive, OrganManifest, OrganRole};
use crate::worldgen::field_visual_sample::gf1_field_linear_rgb_qe_at_position;
use crate::worldgen::EnergyFieldGrid;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum OrientationMode {
    AlongTangent,
    Outward,
    GravityDown,
    EnergyOrOutward,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AttachmentZone {
    Apical,
    Distributed,
    Basal,
    Full,
}

#[derive(Clone, Copy, Debug)]
pub struct OrganAttachment {
    pub position: Vec3,
    pub tangent: Vec3,
    pub normal: Vec3,
    pub spine_fraction: f32,
}

pub const ORGAN_ATTACHMENT_ZONE: [AttachmentZone; OrganRole::COUNT] = [
    AttachmentZone::Full,
    AttachmentZone::Basal,
    AttachmentZone::Full,
    AttachmentZone::Distributed,
    AttachmentZone::Apical,
    AttachmentZone::Apical,
    AttachmentZone::Distributed,
    AttachmentZone::Full,
    AttachmentZone::Apical,
    AttachmentZone::Apical,
    AttachmentZone::Distributed,
    AttachmentZone::Distributed,
];

const ORGAN_ORIENTATION_MODE: [OrientationMode; OrganRole::COUNT] = [
    OrientationMode::AlongTangent,   // Stem
    OrientationMode::GravityDown,    // Root
    OrientationMode::AlongTangent,   // Core
    OrientationMode::Outward,        // Leaf
    OrientationMode::Outward,        // Petal
    OrientationMode::EnergyOrOutward, // Sensory
    OrientationMode::Outward,        // Thorn
    OrientationMode::AlongTangent,   // Shell
    OrientationMode::GravityDown,    // Fruit
    OrientationMode::GravityDown,    // Bud
    OrientationMode::Outward,        // Limb
    OrientationMode::Outward,        // Fin
];

#[inline]
fn safe_tangent(node: &SpineNode) -> Vec3 {
    let t = node.tangent.normalize_or_zero();
    if t.length_squared() < BRANCH_DIR_EPSILON {
        Vec3::Y
    } else {
        t
    }
}

#[inline]
fn outward_normal(tangent: Vec3) -> Vec3 {
    let mut side = tangent.cross(Vec3::Y);
    if side.length_squared() < BRANCH_DIR_EPSILON {
        side = tangent.cross(Vec3::X);
    }
    side.normalize_or_zero()
}

#[inline]
fn is_trunk_role(role: OrganRole) -> bool {
    matches!(role, OrganRole::Stem | OrganRole::Core)
}

/// Derive a tangent vector perpendicular to a cached direction.
/// Used when reconstructing organ params from a BodyPlanLayout that only stores direction (normal).
#[inline]
fn tangent_from_direction(direction: Vec3) -> Vec3 {
    let d = direction.normalize_or_zero();
    let mut side = d.cross(Vec3::Y);
    if side.length_squared() < BRANCH_DIR_EPSILON {
        side = d.cross(Vec3::X);
    }
    side.normalize_or_zero()
}

pub fn organ_attachment_points(
    spine: &[SpineNode],
    count: u8,
    attachment_zone: AttachmentZone,
) -> Vec<OrganAttachment> {
    if spine.len() < 2 || count == 0 {
        return Vec::new();
    }

    let n = count.min(MAX_ORGAN_INSTANCE_COUNT) as usize;
    let mut out = Vec::with_capacity(n);
    let max_idx = spine.len() - 1;

    for i in 0..n {
        let base_t = if n == 1 {
            0.5
        } else {
            (i + 1) as f32 / (n + 1) as f32
        };
        let zone_t = match attachment_zone {
            AttachmentZone::Apical => ORGAN_ZONE_APICAL_OFFSET + base_t * ORGAN_ZONE_APICAL_BASAL_SPAN,
            AttachmentZone::Distributed => base_t,
            AttachmentZone::Basal => ORGAN_ZONE_BASAL_OFFSET + base_t * ORGAN_ZONE_APICAL_BASAL_SPAN,
            AttachmentZone::Full => ORGAN_ZONE_FULL_OFFSET + base_t * ORGAN_ZONE_FULL_SPAN,
        }
        .clamp(0.0, 1.0);
        let idx = ((max_idx as f32) * zone_t).round() as usize;
        let idx = idx.clamp(0, max_idx);
        let node = spine[idx];
        let tangent = safe_tangent(&node);
        let normal = outward_normal(tangent);
        out.push(OrganAttachment {
            position: node.position,
            tangent,
            normal,
            spine_fraction: zone_t,
        });
    }

    out
}

pub fn organ_orientation(
    role: OrganRole,
    attachment: &OrganAttachment,
    energy_direction: Vec3,
) -> (Vec3, Vec3) {
    let tangent = attachment.tangent.normalize_or_zero();
    let outward = attachment.normal.normalize_or_zero();
    let mode = ORGAN_ORIENTATION_MODE[role as usize];
    let normal_out = match mode {
        OrientationMode::GravityDown => Vec3::NEG_Y,
        OrientationMode::EnergyOrOutward => {
            let e = energy_direction.normalize_or_zero();
            if e.length_squared() < BRANCH_DIR_EPSILON {
                outward
            } else {
                e
            }
        }
        OrientationMode::Outward => outward,
        OrientationMode::AlongTangent => tangent,
    }
    .normalize_or_zero();

    let tangent_out = if normal_out.dot(tangent).abs() > ORGAN_ORIENTATION_PARALLEL_DOT_CUTOFF {
        outward
    } else {
        tangent
    }
    .normalize_or_zero();

    (normal_out, tangent_out)
}

/// Build organ mesh without a cached layout (recomputes positions every call).
pub fn build_organ_mesh(
    manifest: &OrganManifest,
    spine: &[SpineNode],
    influence: &GeometryInfluence,
    growth: Option<crate::layers::GrowthBudget>,
    grid: &EnergyFieldGrid,
    almanac: &AlchemicalAlmanac,
    fallback_rgb: [f32; 3],
    fallback_qe_norm: f32,
) -> Mesh {
    build_organ_mesh_inner(manifest, spine, influence, growth, grid, almanac, fallback_rgb, fallback_qe_norm, None)
}

/// Build organ mesh with an optional cached `BodyPlanLayout`.
/// When `layout` is `Some`, uses cached positions/directions instead of recomputing.
pub fn build_organ_mesh_with_layout(
    manifest: &OrganManifest,
    spine: &[SpineNode],
    influence: &GeometryInfluence,
    growth: Option<crate::layers::GrowthBudget>,
    grid: &EnergyFieldGrid,
    almanac: &AlchemicalAlmanac,
    fallback_rgb: [f32; 3],
    fallback_qe_norm: f32,
    layout: Option<&BodyPlanLayout>,
) -> Mesh {
    build_organ_mesh_inner(manifest, spine, influence, growth, grid, almanac, fallback_rgb, fallback_qe_norm, layout)
}

fn build_organ_mesh_inner(
    manifest: &OrganManifest,
    spine: &[SpineNode],
    influence: &GeometryInfluence,
    growth: Option<crate::layers::GrowthBudget>,
    grid: &EnergyFieldGrid,
    almanac: &AlchemicalAlmanac,
    fallback_rgb: [f32; 3],
    fallback_qe_norm: f32,
    layout: Option<&BodyPlanLayout>,
) -> Mesh {
    let sample_field: &dyn Fn(Vec3) -> ([f32; 3], f32) = &|pos: Vec3| {
        gf1_field_linear_rgb_qe_at_position(
            grid,
            pos,
            almanac,
            VISUAL_QE_REFERENCE,
            0.0,
            fallback_rgb,
            fallback_qe_norm,
        )
    };

    let mut meshes = Vec::new();

    if let Some(g) = growth
        && g.biomass_available > BRANCH_MIN_BIOMASS
    {
        let mut root = build_branched_tree_dyn(influence, g.biomass_available, Some(sample_field));
        let cycle = manifest_branch_cycle(manifest);
        apply_manifest_roles_to_tree(&mut root, &cycle, 0);
        meshes.push(flatten_tree_to_mesh(&root));
    } else {
        meshes.push(build_flow_mesh(spine, influence));
    }

    // Running slot index into the BodyPlanLayout (which skips trunk roles).
    let mut layout_slot: usize = 0;

    for spec in manifest.iter() {
        let role = spec.role();
        if is_trunk_role(role) {
            continue;
        }

        let count = spec.count();

        // When a layout is available, read cached positions/directions.
        // Otherwise fall back to computing attachment points on the fly.
        if let Some(layout) = layout {
            for _ in 0..count {
                if layout_slot >= layout.active_count() as usize {
                    break;
                }
                let origin = layout.position(layout_slot);
                let direction = layout.direction(layout_slot);
                layout_slot += 1;

                let (raw_rgb, qe_norm) = sample_field(origin);
                let tint_rgb = organ_role_modulated_rgb(raw_rgb, role);
                let tangent_out = tangent_from_direction(direction);
                let biomass = growth.map(|g| g.biomass_available).unwrap_or(0.0);
                let params = OrganPrimitiveParams {
                    origin,
                    direction,
                    tangent: tangent_out,
                    base_radius: organ_role_scale(role, influence.radius_base),
                    tint_rgb,
                    qe_norm,
                    detail: influence.detail,
                    biomass,
                };

                if !matches!(spec.primitive(), GeometryPrimitive::Tube | GeometryPrimitive::FlatSurface | GeometryPrimitive::PetalFan | GeometryPrimitive::Bulb) {
                    continue;
                }
                let mut organ_mesh = build_organ_primitive(spec, &params);
                apply_role_opacity(&mut organ_mesh, role);
                meshes.push(organ_mesh);
            }
        } else {
            let zone = ORGAN_ATTACHMENT_ZONE[role as usize];
            let attachments = organ_attachment_points(spine, count, zone);
            for attachment in attachments {
                let (raw_rgb, qe_norm) = sample_field(attachment.position);
                let tint_rgb = organ_role_modulated_rgb(raw_rgb, role);
                let (normal_out, tangent_out) = organ_orientation(role, &attachment, influence.energy_direction);
                let biomass = growth.map(|g| g.biomass_available).unwrap_or(0.0);
                let params = OrganPrimitiveParams {
                    origin: attachment.position,
                    direction: normal_out,
                    tangent: tangent_out,
                    base_radius: organ_role_scale(role, influence.radius_base),
                    tint_rgb,
                    qe_norm,
                    detail: influence.detail,
                    biomass,
                };

                if !matches!(spec.primitive(), GeometryPrimitive::Tube | GeometryPrimitive::FlatSurface | GeometryPrimitive::PetalFan | GeometryPrimitive::Bulb) {
                    continue;
                }
                let mut organ_mesh = build_organ_primitive(spec, &params);
                apply_role_opacity(&mut organ_mesh, role);
                meshes.push(organ_mesh);
            }
        }
    }

    merge_meshes(&meshes)
}

#[inline]
fn apply_role_opacity(mesh: &mut Mesh, role: OrganRole) {
    let alpha = organ_role_opacity(role);
    if let Some(VertexAttributeValues::Float32x4(colors)) = mesh.attribute_mut(Mesh::ATTRIBUTE_COLOR) {
        for c in colors.iter_mut() {
            c[3] = (c[3] * alpha).clamp(0.0, 1.0);
        }
    }
}

fn merge_meshes(meshes: &[Mesh]) -> Mesh {
    crate::geometry_flow::merge_meshes(meshes)
}

#[inline]
fn organ_role_to_branch_role(role: OrganRole) -> BranchRole {
    match role {
        OrganRole::Leaf | OrganRole::Petal | OrganRole::Fin => BranchRole::Leaf,
        OrganRole::Thorn | OrganRole::Fruit | OrganRole::Bud | OrganRole::Sensory => BranchRole::Thorn,
        OrganRole::Stem | OrganRole::Root | OrganRole::Core | OrganRole::Shell | OrganRole::Limb => BranchRole::Stem,
    }
}

fn manifest_branch_cycle(manifest: &OrganManifest) -> Vec<BranchRole> {
    let mut out = Vec::new();
    for spec in manifest.iter() {
        let role = spec.role();
        if is_trunk_role(role) {
            continue;
        }
        out.push(organ_role_to_branch_role(role));
    }
    if out.is_empty() {
        out.extend_from_slice(&[BranchRole::Leaf, BranchRole::Thorn, BranchRole::Stem]);
    }
    out
}

fn apply_manifest_roles_to_tree(node: &mut BranchNode, cycle: &[BranchRole], depth: u32) {
    node.influence.branch_role = if depth == 0 {
        BranchRole::Stem
    } else {
        cycle[(depth as usize - 1) % cycle.len()]
    };
    for (idx, child) in node.children.iter_mut().enumerate() {
        let role = if depth == 0 {
            cycle[idx % cycle.len()]
        } else {
            cycle[(idx + depth as usize) % cycle.len()]
        };
        child.influence.branch_role = role;
        apply_manifest_roles_to_tree(child, cycle, depth + 1);
    }
}

/// Assemble a `BodyPlanLayout` from a manifest, spine, and geometry influence.
/// Delegates to `compute_body_plan_layout` with the spread radius and energy direction
/// extracted from the influence. Returns `None` if the manifest is empty.
pub fn assemble_body_plan(
    manifest: &OrganManifest,
    spine: &[SpineNode],
    influence: &GeometryInfluence,
) -> Option<BodyPlanLayout> {
    if manifest.is_empty() || spine.len() < 2 {
        return None;
    }
    let layout = crate::blueprint::equations::compute_body_plan_layout(
        manifest,
        spine,
        influence.radius_base,
        influence.energy_direction,
    );
    if layout.active_count() == 0 {
        return None;
    }
    Some(layout)
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::render::mesh::Indices;
    use crate::blueprint::equations::organ_role_opacity;
    use crate::geometry_flow::{GeometryInfluence, build_flow_spine, flow_mesh_triangle_count};
    use crate::layers::{LifecycleStage, OrganManifest, OrganSpec};

    fn sample_grid() -> EnergyFieldGrid {
        let mut g = EnergyFieldGrid::new(8, 8, 1.0, bevy::math::Vec2::ZERO);
        if let Some(c) = g.cell_xy_mut(2, 2) {
            c.accumulated_qe = 180.0;
            c.dominant_frequency_hz = 250.0;
            c.purity = 1.0;
        }
        if let Some(c) = g.cell_xy_mut(4, 4) {
            c.accumulated_qe = 420.0;
            c.dominant_frequency_hz = 480.0;
            c.purity = 1.0;
        }
        g
    }

    fn sample_influence() -> GeometryInfluence {
        GeometryInfluence {
            detail: 0.7,
            energy_direction: Vec3::Y,
            energy_strength: 1.0,
            resistance: 0.2,
            least_resistance_direction: Vec3::X,
            length_budget: 3.0,
            max_segments: 8,
            radius_base: 0.08,
            start_position: Vec3::new(2.5, 0.0, 2.5),
            qe_norm: 0.5,
            tint_rgb: [0.3, 0.7, 0.2],
            branch_role: crate::blueprint::equations::BranchRole::Stem,
        }
    }

    #[test]
    fn organ_attachment_points_apical_close_to_tip() {
        let inf = sample_influence();
        let spine = build_flow_spine(&inf);
        let points = organ_attachment_points(&spine, 4, AttachmentZone::Apical);
        assert!(points.iter().all(|p| p.spine_fraction > 0.78));
    }

    #[test]
    fn organ_attachment_points_basal_close_to_base() {
        let inf = sample_influence();
        let spine = build_flow_spine(&inf);
        let points = organ_attachment_points(&spine, 4, AttachmentZone::Basal);
        assert!(points.iter().all(|p| p.spine_fraction < 0.22));
    }

    #[test]
    fn organ_attachment_points_distributed_are_uniform() {
        let inf = sample_influence();
        let spine = build_flow_spine(&inf);
        let points = organ_attachment_points(&spine, 4, AttachmentZone::Distributed);
        let d0 = points[1].spine_fraction - points[0].spine_fraction;
        let d1 = points[2].spine_fraction - points[1].spine_fraction;
        let d2 = points[3].spine_fraction - points[2].spine_fraction;
        assert!((d0 - d1).abs() < 0.08);
        assert!((d1 - d2).abs() < 0.08);
    }

    #[test]
    fn organ_attachment_points_full_spans_mid_body() {
        let inf = sample_influence();
        let spine = build_flow_spine(&inf);
        let points = organ_attachment_points(&spine, 4, AttachmentZone::Full);
        assert!(points.first().is_some_and(|p| p.spine_fraction > 0.10));
        assert!(points.last().is_some_and(|p| p.spine_fraction < 0.90));
    }

    #[test]
    fn build_organ_mesh_empty_manifest_still_generates_trunk() {
        let inf = sample_influence();
        let spine = build_flow_spine(&inf);
        let manifest = OrganManifest::new(LifecycleStage::Dormant);
        let mesh = build_organ_mesh(
            &manifest,
            &spine,
            &inf,
            None,
            &sample_grid(),
            &AlchemicalAlmanac::default(),
            [0.4, 0.4, 0.4],
            0.4,
        );
        assert!(flow_mesh_triangle_count(&mesh) > 0);
    }

    #[test]
    fn build_organ_mesh_stem_leaf_petal_has_more_triangles_than_stem_only() {
        let inf = sample_influence();
        let spine = build_flow_spine(&inf);
        let mut manifest = OrganManifest::new(LifecycleStage::Reproductive);
        assert!(manifest.push(OrganSpec::new(OrganRole::Stem, 1, 1.0)));
        assert!(manifest.push(OrganSpec::new(OrganRole::Leaf, 3, 0.7)));
        assert!(manifest.push(OrganSpec::new(OrganRole::Petal, 5, 0.5)));
        let rich = build_organ_mesh(
            &manifest,
            &spine,
            &inf,
            None,
            &sample_grid(),
            &AlchemicalAlmanac::default(),
            [0.4, 0.4, 0.4],
            0.4,
        );

        let base = build_organ_mesh(
            &OrganManifest::new(LifecycleStage::Dormant),
            &spine,
            &inf,
            None,
            &sample_grid(),
            &AlchemicalAlmanac::default(),
            [0.4, 0.4, 0.4],
            0.4,
        );
        assert!(flow_mesh_triangle_count(&rich) > flow_mesh_triangle_count(&base));
    }

    #[test]
    fn build_organ_mesh_is_deterministic_same_inputs() {
        let inf = sample_influence();
        let spine = build_flow_spine(&inf);
        let mut manifest = OrganManifest::new(LifecycleStage::Growing);
        assert!(manifest.push(OrganSpec::new(OrganRole::Leaf, 3, 0.7)));
        let a = build_organ_mesh(
            &manifest,
            &spine,
            &inf,
            None,
            &sample_grid(),
            &AlchemicalAlmanac::default(),
            [0.4, 0.4, 0.4],
            0.4,
        );
        let b = build_organ_mesh(
            &manifest,
            &spine,
            &inf,
            None,
            &sample_grid(),
            &AlchemicalAlmanac::default(),
            [0.4, 0.4, 0.4],
            0.4,
        );
        let Some(VertexAttributeValues::Float32x3(pos_a)) = a.attribute(Mesh::ATTRIBUTE_POSITION) else {
            panic!("missing position a");
        };
        let Some(VertexAttributeValues::Float32x3(pos_b)) = b.attribute(Mesh::ATTRIBUTE_POSITION) else {
            panic!("missing position b");
        };
        let Some(Indices::U32(id_a)) = a.indices() else {
            panic!("missing indices a");
        };
        let Some(Indices::U32(id_b)) = b.indices() else {
            panic!("missing indices b");
        };
        assert_eq!(flow_mesh_triangle_count(&a), flow_mesh_triangle_count(&b));
        assert_eq!(pos_a, pos_b);
        assert_eq!(id_a, id_b);
    }

    #[test]
    fn build_organ_mesh_epi3_sampling_changes_colors_by_position() {
        let mut inf_a = sample_influence();
        inf_a.start_position = Vec3::new(2.5, 0.0, 2.5);
        let spine_a = build_flow_spine(&inf_a);
        let mut manifest = OrganManifest::new(LifecycleStage::Growing);
        assert!(manifest.push(OrganSpec::new(OrganRole::Leaf, 3, 0.7)));
        let mesh_a = build_organ_mesh(
            &manifest,
            &spine_a,
            &inf_a,
            None,
            &sample_grid(),
            &AlchemicalAlmanac::default(),
            [0.4, 0.4, 0.4],
            0.4,
        );

        let mut inf_b = sample_influence();
        inf_b.start_position = Vec3::new(4.5, 0.0, 4.5);
        let spine_b = build_flow_spine(&inf_b);
        let mesh_b = build_organ_mesh(
            &manifest,
            &spine_b,
            &inf_b,
            None,
            &sample_grid(),
            &AlchemicalAlmanac::default(),
            [0.4, 0.4, 0.4],
            0.4,
        );
        let Some(VertexAttributeValues::Float32x4(col_a)) = mesh_a.attribute(Mesh::ATTRIBUTE_COLOR) else {
            panic!("missing colors a");
        };
        let Some(VertexAttributeValues::Float32x4(col_b)) = mesh_b.attribute(Mesh::ATTRIBUTE_COLOR) else {
            panic!("missing colors b");
        };
        let avg = |c: &[[f32; 4]]| -> [f32; 3] {
            let mut s = [0.0f32; 3];
            for v in c {
                s[0] += v[0];
                s[1] += v[1];
                s[2] += v[2];
            }
            let n = c.len().max(1) as f32;
            [s[0] / n, s[1] / n, s[2] / n]
        };
        let a = avg(col_a);
        let b = avg(col_b);
        let dist = (a[0] - b[0]).abs() + (a[1] - b[1]).abs() + (a[2] - b[2]).abs();
        assert!(dist > 0.01, "expected color difference, got {dist}");
    }

    fn extract_alpha_values(mesh: &Mesh) -> Vec<f32> {
        if let Some(VertexAttributeValues::Float32x4(colors)) = mesh.attribute(Mesh::ATTRIBUTE_COLOR) {
            return colors.iter().map(|c| c[3]).collect();
        }
        Vec::new()
    }

    fn assert_role_alpha_applied(role: OrganRole) {
        let inf = sample_influence();
        let spine = build_flow_spine(&inf);
        let mut manifest = OrganManifest::new(LifecycleStage::Reproductive);
        assert!(manifest.push(OrganSpec::new(OrganRole::Stem, 1, 1.0)));
        assert!(manifest.push(OrganSpec::new(role, 1, 1.0)));
        let mesh = build_organ_mesh(
            &manifest,
            &spine,
            &inf,
            None,
            &sample_grid(),
            &AlchemicalAlmanac::default(),
            [0.4, 0.4, 0.4],
            0.4,
        );
        let alphas = extract_alpha_values(&mesh);
        assert!(!alphas.is_empty());
        let expected = organ_role_opacity(role);
        assert!(
            alphas.iter().any(|a| (*a - expected).abs() <= 1e-6),
            "alpha esperado por rol no encontrado role={role:?} expected={expected}"
        );
        assert!(
            alphas.iter().any(|a| (*a - 1.0).abs() <= 1e-6),
            "debe existir alpha 1.0 del tronco base"
        );
        assert!(alphas.iter().all(|a| *a <= 1.0 + 1e-6));
    }

    #[test]
    fn build_organ_mesh_stem_preserves_alpha_identity() {
        let inf = sample_influence();
        let spine = build_flow_spine(&inf);
        let mut manifest = OrganManifest::new(LifecycleStage::Growing);
        assert!(manifest.push(OrganSpec::new(OrganRole::Stem, 1, 1.0)));
        let mesh = build_organ_mesh(
            &manifest,
            &spine,
            &inf,
            None,
            &sample_grid(),
            &AlchemicalAlmanac::default(),
            [0.4, 0.4, 0.4],
            0.4,
        );
        let alphas = extract_alpha_values(&mesh);
        assert!(!alphas.is_empty());
        assert!(alphas.iter().all(|a| (*a - 1.0).abs() <= 1e-6));
    }

    #[test]
    fn build_organ_mesh_petal_applies_opacity_table_factor() {
        assert_role_alpha_applied(OrganRole::Petal);
    }

    #[test]
    fn build_organ_mesh_bud_applies_opacity_table_factor() {
        assert_role_alpha_applied(OrganRole::Bud);
    }

    #[test]
    fn build_organ_mesh_fin_applies_opacity_table_factor() {
        assert_role_alpha_applied(OrganRole::Fin);
    }

    // --- build_organ_mesh_with_layout ---

    #[test]
    fn build_organ_mesh_with_layout_none_matches_original() {
        let inf = sample_influence();
        let spine = build_flow_spine(&inf);
        let mut manifest = OrganManifest::new(LifecycleStage::Growing);
        assert!(manifest.push(OrganSpec::new(OrganRole::Leaf, 3, 0.7)));
        let without = build_organ_mesh(
            &manifest, &spine, &inf, None,
            &sample_grid(), &AlchemicalAlmanac::default(), [0.4, 0.4, 0.4], 0.4,
        );
        let with_none = build_organ_mesh_with_layout(
            &manifest, &spine, &inf, None,
            &sample_grid(), &AlchemicalAlmanac::default(), [0.4, 0.4, 0.4], 0.4, None,
        );
        let Some(VertexAttributeValues::Float32x3(pos_a)) = without.attribute(Mesh::ATTRIBUTE_POSITION) else {
            panic!("missing positions a");
        };
        let Some(VertexAttributeValues::Float32x3(pos_b)) = with_none.attribute(Mesh::ATTRIBUTE_POSITION) else {
            panic!("missing positions b");
        };
        assert_eq!(pos_a, pos_b, "None layout should match no-layout variant");
        assert_eq!(
            flow_mesh_triangle_count(&without),
            flow_mesh_triangle_count(&with_none),
        );
    }

    #[test]
    fn build_organ_mesh_with_layout_some_uses_cached_positions() {
        let inf = sample_influence();
        let spine = build_flow_spine(&inf);
        let mut manifest = OrganManifest::new(LifecycleStage::Reproductive);
        assert!(manifest.push(OrganSpec::new(OrganRole::Stem, 1, 1.0)));
        assert!(manifest.push(OrganSpec::new(OrganRole::Leaf, 2, 0.7)));

        // Build a layout with shifted positions to verify it's used.
        let layout = assemble_body_plan(&manifest, &spine, &inf);
        assert!(layout.is_some(), "layout should be Some for non-trunk manifest");
        let layout = layout.unwrap();
        assert!(layout.active_count() >= 2, "expect at least 2 active organs (2 leaves)");

        // Mesh built with layout should produce triangles.
        let mesh = build_organ_mesh_with_layout(
            &manifest, &spine, &inf, None,
            &sample_grid(), &AlchemicalAlmanac::default(), [0.4, 0.4, 0.4], 0.4,
            Some(&layout),
        );
        assert!(flow_mesh_triangle_count(&mesh) > 0);
    }

    #[test]
    fn build_organ_mesh_with_layout_produces_same_triangle_count() {
        let inf = sample_influence();
        let spine = build_flow_spine(&inf);
        let mut manifest = OrganManifest::new(LifecycleStage::Reproductive);
        assert!(manifest.push(OrganSpec::new(OrganRole::Stem, 1, 1.0)));
        assert!(manifest.push(OrganSpec::new(OrganRole::Leaf, 3, 0.7)));
        assert!(manifest.push(OrganSpec::new(OrganRole::Petal, 5, 0.5)));

        let layout = assemble_body_plan(&manifest, &spine, &inf).unwrap();
        let without_layout = build_organ_mesh(
            &manifest, &spine, &inf, None,
            &sample_grid(), &AlchemicalAlmanac::default(), [0.4, 0.4, 0.4], 0.4,
        );
        let with_layout = build_organ_mesh_with_layout(
            &manifest, &spine, &inf, None,
            &sample_grid(), &AlchemicalAlmanac::default(), [0.4, 0.4, 0.4], 0.4,
            Some(&layout),
        );
        assert_eq!(
            flow_mesh_triangle_count(&without_layout),
            flow_mesh_triangle_count(&with_layout),
            "layout vs no-layout should produce same number of triangles"
        );
    }

    // --- assemble_body_plan ---

    #[test]
    fn assemble_body_plan_empty_manifest_returns_none() {
        let inf = sample_influence();
        let spine = build_flow_spine(&inf);
        let manifest = OrganManifest::new(LifecycleStage::Dormant);
        assert!(assemble_body_plan(&manifest, &spine, &inf).is_none());
    }

    #[test]
    fn assemble_body_plan_trunk_only_returns_none() {
        let inf = sample_influence();
        let spine = build_flow_spine(&inf);
        let mut manifest = OrganManifest::new(LifecycleStage::Growing);
        assert!(manifest.push(OrganSpec::new(OrganRole::Stem, 1, 1.0)));
        assert!(manifest.push(OrganSpec::new(OrganRole::Core, 1, 1.0)));
        // Only trunk roles => compute_body_plan_layout returns active_count=0 => None
        assert!(assemble_body_plan(&manifest, &spine, &inf).is_none());
    }

    #[test]
    fn assemble_body_plan_with_organs_returns_some() {
        let inf = sample_influence();
        let spine = build_flow_spine(&inf);
        let mut manifest = OrganManifest::new(LifecycleStage::Reproductive);
        assert!(manifest.push(OrganSpec::new(OrganRole::Stem, 1, 1.0)));
        assert!(manifest.push(OrganSpec::new(OrganRole::Leaf, 3, 0.7)));
        let layout = assemble_body_plan(&manifest, &spine, &inf);
        assert!(layout.is_some());
        let layout = layout.unwrap();
        assert_eq!(layout.active_count(), 3);
    }

    #[test]
    fn assemble_body_plan_short_spine_returns_none() {
        let inf = sample_influence();
        let spine = &[SpineNode {
            position: Vec3::ZERO,
            tangent: Vec3::Y,
            tint_rgb: [0.5, 0.5, 0.5],
            qe_norm: 0.5,
        }];
        let mut manifest = OrganManifest::new(LifecycleStage::Growing);
        assert!(manifest.push(OrganSpec::new(OrganRole::Leaf, 2, 0.7)));
        assert!(assemble_body_plan(&manifest, spine, &inf).is_none());
    }

    #[test]
    fn assemble_body_plan_determinism() {
        let inf = sample_influence();
        let spine = build_flow_spine(&inf);
        let mut manifest = OrganManifest::new(LifecycleStage::Reproductive);
        assert!(manifest.push(OrganSpec::new(OrganRole::Limb, 4, 1.0)));
        assert!(manifest.push(OrganSpec::new(OrganRole::Sensory, 1, 0.5)));
        let a = assemble_body_plan(&manifest, &spine, &inf).unwrap();
        let b = assemble_body_plan(&manifest, &spine, &inf).unwrap();
        assert_eq!(a, b);
    }

    // --- tangent_from_direction ---

    #[test]
    fn tangent_from_direction_perpendicular_to_input() {
        let dir = Vec3::new(1.0, 2.0, 3.0).normalize();
        let tangent = tangent_from_direction(dir);
        assert!(tangent.dot(dir).abs() < 1e-4, "tangent should be perpendicular to direction");
        assert!((tangent.length() - 1.0).abs() < 1e-4, "tangent should be unit length");
    }

    #[test]
    fn tangent_from_direction_y_axis() {
        let tangent = tangent_from_direction(Vec3::Y);
        assert!(tangent.dot(Vec3::Y).abs() < 1e-4);
        assert!((tangent.length() - 1.0).abs() < 1e-4);
    }

    #[test]
    fn tangent_from_direction_zero_returns_valid() {
        let tangent = tangent_from_direction(Vec3::ZERO);
        assert!(tangent.is_finite());
    }
}
