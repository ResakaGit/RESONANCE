use crate::math_types::{Quat, Vec3};
use bevy::prelude::Mesh;
use bevy::render::mesh::{Indices, VertexAttributeValues};

use crate::blueprint::constants::{
    BRANCH_ANGLE_SPREAD, BRANCH_CHILD_BUDGET_DECAY, BRANCH_DETAIL_DECAY, BRANCH_DIR_EPSILON,
    BRANCH_ENERGY_DECAY, BRANCH_MAX_DEPTH, BRANCH_QE_DECAY, BRANCH_RADIUS_DECAY,
    MAX_TOTAL_BRANCHES,
};
use crate::blueprint::equations::{
    branch_attenuation_values, branch_budget, branch_child_role_from_branch_index,
};

use super::{
    GeometryInfluence, SpineNode, build_flow_mesh, build_flow_spine, build_flow_spine_painted,
    spine_paint_vertex_from_raw_field,
};

/// Árbol GF1. Con muestreo de campo (EPI3), el color en vértices sale de [`SpineNode::tint_rgb`] / `qe_norm`;
/// `influence.tint_rgb` puede no coincidir con el primer anillo pintado (metadatos de paquete / pivote hijo).
#[derive(Debug, Clone)]
pub struct BranchNode {
    pub spine: Vec<SpineNode>,
    pub influence: GeometryInfluence,
    pub children: Vec<BranchNode>,
}

pub fn branch_attenuation(
    parent_influence: &GeometryInfluence,
    depth: u32,
    max_depth: u32,
) -> GeometryInfluence {
    let mut child = *parent_influence;
    let (length, radius, energy, qe, detail) = branch_attenuation_values(
        child.length_budget,
        child.radius_base,
        child.energy_strength,
        child.qe_norm,
        child.detail,
        depth,
        max_depth,
        BRANCH_RADIUS_DECAY,
        BRANCH_ENERGY_DECAY,
        BRANCH_QE_DECAY,
        BRANCH_DETAIL_DECAY,
    );
    child.length_budget = length;
    child.radius_base = radius;
    child.energy_strength = energy;
    child.qe_norm = qe;
    child.detail = detail;
    child
}

pub fn build_branched_tree(influence: &GeometryInfluence, growth_budget: f32) -> BranchNode {
    build_branched_tree_dyn(influence, growth_budget, None)
}

/// Ramificación con muestreo opcional de campo en cada `Vec3` (XZ→celda en el llamador). EPI3.
pub fn build_branched_tree_dyn(
    influence: &GeometryInfluence,
    growth_budget: f32,
    branch_field: Option<&dyn Fn(Vec3) -> ([f32; 3], f32)>,
) -> BranchNode {
    let mut total_branches = 1u32;
    build_branched_tree_recursive(
        influence,
        growth_budget,
        0,
        &mut total_branches,
        branch_field,
    )
}

fn build_branched_tree_recursive(
    influence: &GeometryInfluence,
    growth_budget: f32,
    depth: u32,
    total_branches: &mut u32,
    branch_field: Option<&dyn Fn(Vec3) -> ([f32; 3], f32)>,
) -> BranchNode {
    let spine = match branch_field {
        Some(f) => build_flow_spine_painted(influence, |pos, inf| {
            spine_paint_vertex_from_raw_field(pos, inf, f)
        }),
        None => build_flow_spine(influence),
    };
    let mut node = BranchNode {
        spine,
        influence: *influence,
        children: Vec::new(),
    };
    if depth >= BRANCH_MAX_DEPTH || *total_branches >= MAX_TOTAL_BRANCHES {
        return node;
    }

    let branch_count = branch_budget(growth_budget, depth, BRANCH_MAX_DEPTH)
        .min(MAX_TOTAL_BRANCHES.saturating_sub(*total_branches));
    if branch_count == 0 || node.spine.len() < 3 {
        return node;
    }

    let branch_count_usize = branch_count as usize;
    for i in 0..branch_count_usize {
        if *total_branches >= MAX_TOTAL_BRANCHES {
            break;
        }
        let t = (i + 1) as f32 / (branch_count_usize + 1) as f32;
        let idx = ((node.spine.len() - 1) as f32 * t).round() as usize;
        let idx = idx.clamp(1, node.spine.len() - 2);
        let pivot = node.spine[idx];

        let sign = if i % 2 == 0 { 1.0 } else { -1.0 };
        let rotation = Quat::from_rotation_y(BRANCH_ANGLE_SPREAD * sign);
        let mut child = branch_attenuation(&node.influence, depth + 1, BRANCH_MAX_DEPTH);
        child.start_position = pivot.position;
        child.energy_direction = (rotation * pivot.tangent).normalize_or_zero();
        if child.energy_direction.length_squared() < BRANCH_DIR_EPSILON {
            child.energy_direction = Vec3::Y;
        }
        child.least_resistance_direction = child.energy_direction;
        child.branch_role = branch_child_role_from_branch_index(i, depth + 1);
        if let Some(sample) = branch_field {
            let (rgb, qn) = spine_paint_vertex_from_raw_field(child.start_position, &child, sample);
            child.tint_rgb = rgb;
            child.qe_norm = qn.clamp(0.0, 1.0);
        }

        *total_branches += 1;
        let child_budget = growth_budget * BRANCH_CHILD_BUDGET_DECAY;
        node.children.push(build_branched_tree_recursive(
            &child,
            child_budget,
            depth + 1,
            total_branches,
            branch_field,
        ));
    }

    node
}

pub fn flatten_tree_to_mesh(root: &BranchNode) -> Mesh {
    let mut positions: Vec<[f32; 3]> = Vec::new();
    let mut normals: Vec<[f32; 3]> = Vec::new();
    let mut uvs: Vec<[f32; 2]> = Vec::new();
    let mut colors: Vec<[f32; 4]> = Vec::new();
    let mut indices: Vec<u32> = Vec::new();

    append_tree_mesh(
        root,
        &mut positions,
        &mut normals,
        &mut uvs,
        &mut colors,
        &mut indices,
    );

    let mut mesh = Mesh::new(
        bevy::render::mesh::PrimitiveTopology::TriangleList,
        bevy::render::render_asset::RenderAssetUsages::default(),
    );
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
    mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
    mesh.insert_attribute(Mesh::ATTRIBUTE_COLOR, colors);
    mesh.insert_indices(Indices::U32(indices));
    mesh
}

fn append_tree_mesh(
    node: &BranchNode,
    positions: &mut Vec<[f32; 3]>,
    normals: &mut Vec<[f32; 3]>,
    uvs: &mut Vec<[f32; 2]>,
    colors: &mut Vec<[f32; 4]>,
    indices: &mut Vec<u32>,
) {
    let mesh = build_flow_mesh(&node.spine, &node.influence);
    append_mesh(&mesh, positions, normals, uvs, colors, indices);
    for child in &node.children {
        append_tree_mesh(child, positions, normals, uvs, colors, indices);
    }
}

fn append_mesh(
    mesh: &Mesh,
    positions: &mut Vec<[f32; 3]>,
    normals: &mut Vec<[f32; 3]>,
    uvs: &mut Vec<[f32; 2]>,
    colors: &mut Vec<[f32; 4]>,
    indices: &mut Vec<u32>,
) {
    let base = positions.len() as u32;

    if let Some(VertexAttributeValues::Float32x3(src)) = mesh.attribute(Mesh::ATTRIBUTE_POSITION) {
        positions.extend(src.iter().copied());
    }
    if let Some(VertexAttributeValues::Float32x3(src)) = mesh.attribute(Mesh::ATTRIBUTE_NORMAL) {
        normals.extend(src.iter().copied());
    }
    if let Some(VertexAttributeValues::Float32x2(src)) = mesh.attribute(Mesh::ATTRIBUTE_UV_0) {
        uvs.extend(src.iter().copied());
    }
    if let Some(VertexAttributeValues::Float32x4(src)) = mesh.attribute(Mesh::ATTRIBUTE_COLOR) {
        colors.extend(src.iter().copied());
    }

    if let Some(Indices::U32(src)) = mesh.indices() {
        indices.extend(src.iter().map(|i| i + base));
    }
}

pub fn branch_node_count(root: &BranchNode) -> u32 {
    let mut count = 1u32;
    for child in &root.children {
        count += branch_node_count(child);
    }
    count
}

pub fn estimate_branch_cost(growth_budget: f32) -> u32 {
    fn recur(growth_budget: f32, depth: u32, total: &mut u32) {
        if depth >= BRANCH_MAX_DEPTH || *total >= MAX_TOTAL_BRANCHES {
            return;
        }
        let branch_count = branch_budget(growth_budget, depth, BRANCH_MAX_DEPTH)
            .min(MAX_TOTAL_BRANCHES.saturating_sub(*total));
        for _ in 0..branch_count {
            if *total >= MAX_TOTAL_BRANCHES {
                break;
            }
            *total += 1;
            recur(growth_budget * BRANCH_CHILD_BUDGET_DECAY, depth + 1, total);
        }
    }
    let mut total = 1u32;
    recur(growth_budget, 0, &mut total);
    total.max(1)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::geometry_flow::flow_mesh_triangle_count;

    fn sample_influence() -> GeometryInfluence {
        GeometryInfluence {
            detail: 0.8,
            energy_direction: Vec3::Y,
            energy_strength: 2.0,
            resistance: 0.5,
            least_resistance_direction: Vec3::X,
            length_budget: 3.0,
            max_segments: 12,
            radius_base: 0.08,
            start_position: Vec3::ZERO,
            qe_norm: 0.8,
            tint_rgb: [0.3, 0.8, 0.2],
            branch_role: crate::blueprint::equations::BranchRole::Stem,
        }
    }

    #[test]
    fn build_branched_tree_budget_zero_returns_single_spine() {
        let root = build_branched_tree(&sample_influence(), 0.0);
        assert!(root.children.is_empty());
        assert!(!root.spine.is_empty());
    }

    #[test]
    fn build_branched_tree_budget_one_has_children() {
        let root = build_branched_tree(&sample_influence(), 1.0);
        assert!(!root.children.is_empty(), "expected recursive children");
    }

    #[test]
    fn flatten_tree_to_mesh_returns_valid_mesh() {
        let root = build_branched_tree(&sample_influence(), 4.0);
        let mesh = flatten_tree_to_mesh(&root);
        let tri = flow_mesh_triangle_count(&mesh);
        assert!(tri > 0, "triangles={tri}");
    }

    #[test]
    fn deterministic_same_input_same_triangle_count() {
        let inf = sample_influence();
        let a = flatten_tree_to_mesh(&build_branched_tree(&inf, 4.0));
        let b = flatten_tree_to_mesh(&build_branched_tree(&inf, 4.0));
        assert_eq!(flow_mesh_triangle_count(&a), flow_mesh_triangle_count(&b));
    }

    #[test]
    fn deterministic_same_input_same_mesh_positions() {
        let inf = sample_influence();
        let a = flatten_tree_to_mesh(&build_branched_tree(&inf, 4.0));
        let b = flatten_tree_to_mesh(&build_branched_tree(&inf, 4.0));
        let Some(VertexAttributeValues::Float32x3(ap)) = a.attribute(Mesh::ATTRIBUTE_POSITION)
        else {
            panic!("missing positions a");
        };
        let Some(VertexAttributeValues::Float32x3(bp)) = b.attribute(Mesh::ATTRIBUTE_POSITION)
        else {
            panic!("missing positions b");
        };
        assert_eq!(ap, bp);
    }

    #[test]
    fn branch_count_respects_global_cap() {
        let root = build_branched_tree(&sample_influence(), 200.0);
        assert!(branch_node_count(&root) <= MAX_TOTAL_BRANCHES);
    }

    #[test]
    fn build_branched_tree_dyn_field_sample_is_deterministic() {
        let inf = sample_influence();
        let f = |p: Vec3| ([(p.x * 0.1).clamp(0.0, 1.0), 0.2, 0.3], 0.6);
        let a = flatten_tree_to_mesh(&super::build_branched_tree_dyn(&inf, 4.0, Some(&f)));
        let b = flatten_tree_to_mesh(&super::build_branched_tree_dyn(&inf, 4.0, Some(&f)));
        let Some(VertexAttributeValues::Float32x4(ca)) = a.attribute(Mesh::ATTRIBUTE_COLOR) else {
            panic!("missing colors a");
        };
        let Some(VertexAttributeValues::Float32x4(cb)) = b.attribute(Mesh::ATTRIBUTE_COLOR) else {
            panic!("missing colors b");
        };
        assert_eq!(ca, cb);
    }

    #[test]
    fn flatten_tree_mesh_attributes_are_synchronized() {
        let mesh = flatten_tree_to_mesh(&build_branched_tree(&sample_influence(), 4.0));
        let Some(VertexAttributeValues::Float32x3(pos)) = mesh.attribute(Mesh::ATTRIBUTE_POSITION)
        else {
            panic!("missing positions");
        };
        let Some(VertexAttributeValues::Float32x3(nrm)) = mesh.attribute(Mesh::ATTRIBUTE_NORMAL)
        else {
            panic!("missing normals");
        };
        let Some(VertexAttributeValues::Float32x2(uv)) = mesh.attribute(Mesh::ATTRIBUTE_UV_0)
        else {
            panic!("missing uv");
        };
        let Some(VertexAttributeValues::Float32x4(col)) = mesh.attribute(Mesh::ATTRIBUTE_COLOR)
        else {
            panic!("missing color");
        };
        assert_eq!(pos.len(), nrm.len());
        assert_eq!(pos.len(), uv.len());
        assert_eq!(pos.len(), col.len());
        let Some(Indices::U32(id)) = mesh.indices() else {
            panic!("missing indices");
        };
        assert_eq!(id.len() % 3, 0);
        let max_idx = pos.len().saturating_sub(1) as u32;
        assert!(id.iter().all(|i| *i <= max_idx));
    }
}
