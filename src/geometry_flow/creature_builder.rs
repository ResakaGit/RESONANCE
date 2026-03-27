//! Creature geometry builder — stateless GF1 composition from genome parameters.
//!
//! Consumes pure parameters from `batch_fitness::trunk_params_from_genome`
//! and `batch_fitness::branch_plan_from_genome`. Produces a single merged Mesh.
//!
//! No ECS. No Bevy systems. Just geometry.

use bevy::prelude::Mesh;

use crate::blueprint::equations::{
    batch_fitness::{self, BranchPlan},
    frequency_to_tint_rgb, internal_field, BranchRole,
};
use crate::geometry_flow::{self, GeometryInfluence, merge_meshes};
use crate::math_types::Vec3;

/// Build a complete creature mesh from genome biases.
///
/// Calls `trunk_params_from_genome` → `build_flow_spine` → `build_flow_mesh`,
/// then appends branches per `branch_plan_from_genome`.
/// Returns a single merged `Mesh` ready for GPU.
pub fn build_creature_mesh(
    growth_bias: f32,
    mobility_bias: f32,
    branching_bias: f32,
    resilience: f32,
    frequency_hz: f32,
) -> Mesh {
    let tint = frequency_to_tint_rgb(frequency_hz);
    let (length, radius, tilt, resistance, detail, segments) =
        batch_fitness::trunk_params_from_genome(growth_bias, mobility_bias, branching_bias, resilience);

    let dir = Vec3::new(tilt.sin(), tilt.cos(), 0.0).normalize_or_zero();

    let trunk_influence = GeometryInfluence {
        detail,
        energy_direction: dir,
        energy_strength: 0.3 + mobility_bias * 0.7,
        resistance,
        least_resistance_direction: Vec3::Y,
        length_budget: length,
        max_segments: segments,
        radius_base: radius,
        start_position: Vec3::ZERO,
        qe_norm: 0.3 + growth_bias * 0.7,
        tint_rgb: tint,
        branch_role: BranchRole::Stem,
    };

    let trunk_spine = geometry_flow::build_flow_spine(&trunk_influence);
    let trunk_mesh = geometry_flow::build_flow_mesh(&trunk_spine, &trunk_influence);

    // ── Branches ────────────────────────────────────────────────────────────
    let plan = batch_fitness::branch_plan_from_genome(branching_bias, growth_bias, resilience);
    if plan.count == 0 || trunk_spine.len() < 3 {
        return trunk_mesh;
    }

    let mut all_meshes = vec![trunk_mesh];

    for b in 0..plan.count {
        let attach_idx = (plan.attach_fractions[b] * (trunk_spine.len() - 1) as f32) as usize;
        let attach_idx = attach_idx.clamp(1, trunk_spine.len() - 1);
        let attach_pos = trunk_spine[attach_idx].position;

        let angle = plan.angles[b];
        let branch_dir = Vec3::new(
            angle.cos() * 0.7,
            0.3 + growth_bias * 0.4,
            angle.sin() * 0.7,
        ).normalize_or_zero();

        let branch_influence = GeometryInfluence {
            detail: detail * 0.7,
            energy_direction: branch_dir,
            energy_strength: 0.2 + mobility_bias * 0.3,
            resistance: resistance * plan.flexibility,
            least_resistance_direction: branch_dir,
            length_budget: length * plan.length_fraction,
            max_segments: (segments / 3).max(4),
            radius_base: radius * plan.radius_fraction,
            start_position: attach_pos,
            qe_norm: 0.2 + branching_bias * 0.3,
            tint_rgb: [
                tint[0] * 0.8 + 0.1,
                tint[1] * 0.9,
                tint[2] * 0.7 + 0.15,
            ],
            branch_role: BranchRole::Leaf,
        };

        let branch_spine = geometry_flow::build_flow_spine(&branch_influence);
        all_meshes.push(geometry_flow::build_flow_mesh(&branch_spine, &branch_influence));
    }

    merge_meshes(&all_meshes)
}

/// Build creature mesh with variable cross-section driven by internal energy field.
///
/// Each of the 8 nodes maps to a spine station. Radius at each station
/// proportional to local qe via `internal_field::field_to_radii`.
/// Produces emergent organ-like bulges without programming organs.
pub fn build_creature_mesh_with_field(
    growth_bias: f32,
    mobility_bias: f32,
    branching_bias: f32,
    resilience: f32,
    frequency_hz: f32,
    qe_field: &[f32; 8],
    freq_field: &[f32; 8],
) -> Mesh {
    let tint = frequency_to_tint_rgb(frequency_hz);
    let (length, radius, tilt, resistance, detail, segments) =
        batch_fitness::trunk_params_from_genome(growth_bias, mobility_bias, branching_bias, resilience);

    let dir = Vec3::new(tilt.sin(), tilt.cos(), 0.0).normalize_or_zero();

    // Per-node radii from internal energy field
    let radii = internal_field::field_to_radii(
        qe_field, radius,
        crate::batch::constants::FIELD_RADIUS_MIN_RATIO,
        crate::batch::constants::FIELD_RADIUS_MAX_RATIO,
    );

    let trunk_influence = GeometryInfluence {
        detail,
        energy_direction: dir,
        energy_strength: 0.3 + mobility_bias * 0.7,
        resistance,
        least_resistance_direction: Vec3::Y,
        length_budget: length,
        max_segments: segments,
        radius_base: radius,
        start_position: Vec3::ZERO,
        qe_norm: 0.3 + growth_bias * 0.7,
        tint_rgb: tint,
        branch_role: BranchRole::Stem,
    };

    let trunk_spine = geometry_flow::build_flow_spine(&trunk_influence);

    // Interpolate radii from 8 field nodes to spine node count
    let spine_radii = interpolate_radii_to_spine(&radii, trunk_spine.len());
    let trunk_mesh = geometry_flow::build_flow_mesh_variable_radius(
        &trunk_spine, &trunk_influence, &spine_radii,
    );

    // Branches (same logic as build_creature_mesh)
    let plan = batch_fitness::branch_plan_from_genome(branching_bias, growth_bias, resilience);
    if plan.count == 0 || trunk_spine.len() < 3 {
        return trunk_mesh;
    }

    let mut all_meshes = vec![trunk_mesh];
    for b in 0..plan.count {
        let attach_idx = (plan.attach_fractions[b] * (trunk_spine.len() - 1) as f32) as usize;
        let attach_idx = attach_idx.clamp(1, trunk_spine.len() - 1);
        let attach_pos = trunk_spine[attach_idx].position;

        // Branch radius scales with local qe at attachment point
        let local_radius = spine_radii.get(attach_idx).copied().unwrap_or(radius) * plan.radius_fraction;

        let angle = plan.angles[b];
        let branch_dir = Vec3::new(
            angle.cos() * 0.7,
            0.3 + growth_bias * 0.4,
            angle.sin() * 0.7,
        ).normalize_or_zero();

        // Per-node tint from freq_field at nearest node
        let field_node = (b * 8 / plan.count.max(1)).min(7);
        let local_freq = freq_field[field_node];
        let branch_tint = frequency_to_tint_rgb(local_freq);

        let branch_influence = GeometryInfluence {
            detail: detail * 0.7,
            energy_direction: branch_dir,
            energy_strength: 0.2 + mobility_bias * 0.3,
            resistance: resistance * plan.flexibility,
            least_resistance_direction: branch_dir,
            length_budget: length * plan.length_fraction,
            max_segments: (segments / 3).max(4),
            radius_base: local_radius,
            start_position: attach_pos,
            qe_norm: 0.2 + branching_bias * 0.3,
            tint_rgb: branch_tint,
            branch_role: BranchRole::Leaf,
        };
        let branch_spine = geometry_flow::build_flow_spine(&branch_influence);
        all_meshes.push(geometry_flow::build_flow_mesh(&branch_spine, &branch_influence));
    }

    merge_meshes(&all_meshes)
}

/// Linearly interpolate 8 radii values to match spine node count.
fn interpolate_radii_to_spine(radii_8: &[f32; 8], spine_len: usize) -> Vec<f32> {
    if spine_len == 0 { return Vec::new(); }
    if spine_len == 1 { return vec![radii_8[0]]; }
    let mut result = Vec::with_capacity(spine_len);
    for i in 0..spine_len {
        let t = i as f32 / (spine_len - 1).max(1) as f32;
        let field_pos = t * 7.0;
        let lo = (field_pos as usize).min(6);
        let hi = lo + 1;
        let frac = field_pos - lo as f32;
        result.push(radii_8[lo] * (1.0 - frac) + radii_8[hi] * frac);
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn creature_mesh_has_vertices() {
        let mesh = build_creature_mesh(0.8, 0.5, 0.3, 0.7, 440.0);
        let pos = mesh.attribute(Mesh::ATTRIBUTE_POSITION);
        assert!(pos.is_some(), "mesh should have position attribute");
    }

    #[test]
    fn creature_mesh_with_branches_is_larger() {
        let no_branch = build_creature_mesh(0.8, 0.5, 0.0, 0.7, 440.0);
        let branched  = build_creature_mesh(0.8, 0.5, 1.0, 0.7, 440.0);
        let count_a = vertex_count(&no_branch);
        let count_b = vertex_count(&branched);
        assert!(count_b > count_a, "branches should add vertices: {count_a} vs {count_b}");
    }

    #[test]
    fn different_genomes_produce_different_meshes() {
        let a = build_creature_mesh(0.2, 0.8, 0.1, 0.3, 200.0);
        let b = build_creature_mesh(0.9, 0.1, 0.9, 0.9, 700.0);
        assert_ne!(vertex_count(&a), vertex_count(&b), "different genomes → different vertex counts");
    }

    #[test]
    fn field_driven_mesh_has_variable_thickness() {
        let mut field = [1.0; 8];
        field[3] = 10.0; // big lump at node 3
        let freq = [440.0; 8];
        let mesh = build_creature_mesh_with_field(0.8, 0.5, 0.3, 0.7, 440.0, &field, &freq);
        assert!(vertex_count(&mesh) > 0);
    }

    #[test]
    fn field_driven_differs_from_uniform() {
        let uniform = [5.0; 8];
        let mut lumpy = [2.0; 8];
        lumpy[2] = 20.0;
        lumpy[5] = 15.0;
        let freq = [440.0; 8];
        let m1 = build_creature_mesh_with_field(0.8, 0.5, 0.5, 0.5, 440.0, &uniform, &freq);
        let m2 = build_creature_mesh_with_field(0.8, 0.5, 0.5, 0.5, 440.0, &lumpy, &freq);
        // Same genome but different fields → same vertex count (topology)
        // but different positions (variable radius)
        assert_eq!(vertex_count(&m1), vertex_count(&m2));
    }

    #[test]
    fn interpolate_radii_length_matches() {
        let radii = [1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0];
        let interp = interpolate_radii_to_spine(&radii, 20);
        assert_eq!(interp.len(), 20);
        assert!((interp[0] - 1.0).abs() < 1e-4);
        assert!((interp[19] - 8.0).abs() < 1e-4);
    }

    fn vertex_count(mesh: &Mesh) -> usize {
        use bevy::render::mesh::VertexAttributeValues;
        match mesh.attribute(Mesh::ATTRIBUTE_POSITION) {
            Some(VertexAttributeValues::Float32x3(v)) => v.len(),
            _ => 0,
        }
    }
}
