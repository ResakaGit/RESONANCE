//! Creature geometry builder — stateless GF1 composition from genome parameters.
//!
//! Consumes pure parameters from `batch_fitness::trunk_params_from_genome`
//! and `batch_fitness::branch_plan_from_genome`. Produces a single merged Mesh.
//!
//! No ECS. No Bevy systems. Just geometry.

use bevy::prelude::Mesh;

use crate::blueprint::equations::{
    batch_fitness::{self, BranchPlan},
    frequency_to_tint_rgb, BranchRole,
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

    fn vertex_count(mesh: &Mesh) -> usize {
        use bevy::render::mesh::VertexAttributeValues;
        match mesh.attribute(Mesh::ATTRIBUTE_POSITION) {
            Some(VertexAttributeValues::Float32x3(v)) => v.len(),
            _ => 0,
        }
    }
}
