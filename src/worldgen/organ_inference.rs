use bevy::math::Vec3;
use bevy::prelude::Mesh;
use bevy::render::mesh::{Indices, PrimitiveTopology, VertexAttributeValues};
use bevy::render::render_asset::RenderAssetUsages;

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

    for spec in manifest.iter() {
        let role = spec.role();
        if is_trunk_role(role) {
            continue;
        }

        let zone = ORGAN_ATTACHMENT_ZONE[role as usize];
        let attachments = organ_attachment_points(spine, spec.count(), zone);
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

            // El primitive viene inferido por LI1 en OrganSpec.
            if !matches!(spec.primitive(), GeometryPrimitive::Tube | GeometryPrimitive::FlatSurface | GeometryPrimitive::PetalFan | GeometryPrimitive::Bulb) {
                continue;
            }
            let mut organ_mesh = build_organ_primitive(spec, &params);
            apply_role_opacity(&mut organ_mesh, role);
            meshes.push(organ_mesh);
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
    let mut positions: Vec<[f32; 3]> = Vec::new();
    let mut normals: Vec<[f32; 3]> = Vec::new();
    let mut uvs: Vec<[f32; 2]> = Vec::new();
    let mut colors: Vec<[f32; 4]> = Vec::new();
    let mut indices: Vec<u32> = Vec::new();

    for mesh in meshes {
        let base = positions.len() as u32;
        let vtx_count = if let Some(VertexAttributeValues::Float32x3(src)) =
            mesh.attribute(Mesh::ATTRIBUTE_POSITION)
        {
            positions.extend(src.iter().copied());
            src.len()
        } else {
            0
        };
        if let Some(VertexAttributeValues::Float32x3(src)) = mesh.attribute(Mesh::ATTRIBUTE_NORMAL) {
            normals.extend(src.iter().copied());
        } else {
            normals.extend(std::iter::repeat_n([0.0, 1.0, 0.0], vtx_count));
        }
        if let Some(VertexAttributeValues::Float32x2(src)) = mesh.attribute(Mesh::ATTRIBUTE_UV_0) {
            uvs.extend(src.iter().copied());
        } else {
            uvs.extend(std::iter::repeat_n([0.0, 0.0], vtx_count));
        }
        if let Some(VertexAttributeValues::Float32x4(src)) = mesh.attribute(Mesh::ATTRIBUTE_COLOR) {
            colors.extend(src.iter().copied());
        } else {
            colors.extend(std::iter::repeat_n([1.0, 1.0, 1.0, 1.0], vtx_count));
        }
        if let Some(Indices::U32(src)) = mesh.indices() {
            indices.extend(src.iter().map(|idx| idx + base));
        }
    }

    let mut mesh = Mesh::new(PrimitiveTopology::TriangleList, RenderAssetUsages::default());
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
    mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
    mesh.insert_attribute(Mesh::ATTRIBUTE_COLOR, colors);
    mesh.insert_indices(Indices::U32(indices));
    mesh
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

#[cfg(test)]
mod tests {
    use super::*;
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
}
