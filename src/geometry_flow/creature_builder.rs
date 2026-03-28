//! Creature geometry builder — stateless GF1 composition from genome parameters.
//!
//! Consumes pure parameters from `batch_fitness::trunk_params_from_genome`
//! and `batch_fitness::branch_plan_from_genome`. Produces a single merged Mesh.
//!
//! No ECS. No Bevy systems. Just geometry.

use bevy::prelude::Mesh;

use crate::blueprint::equations::{
    batch_fitness,
    frequency_to_tint_rgb, radial_field, BranchRole,
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
/// Build creature mesh from 2D radial energy field.
///
/// Trunk: variable-radius tube from axial profile.
/// Appendages: each detected peak spawns a sub-mesh.
/// Shape determined by peak aspect ratio (bulb/tube/taper).
/// Axiom 6: peaks emerge from field, not programmed.
pub fn build_creature_mesh_with_field(
    growth_bias: f32,
    mobility_bias: f32,
    branching_bias: f32,
    resilience: f32,
    frequency_hz: f32,
    qe_field: &[[f32; 4]; 8],
    freq_field: &[[f32; 4]; 8],
) -> Mesh {
    let tint = frequency_to_tint_rgb(frequency_hz);
    let (length, radius, tilt, resistance, detail, segments) =
        batch_fitness::trunk_params_from_genome(growth_bias, mobility_bias, branching_bias, resilience);

    let dir = Vec3::new(tilt.sin(), tilt.cos(), 0.0).normalize_or_zero();

    // Per-station radii from 2D field (averaged across sectors)
    let radii = radial_field::radial_to_axial_radii(
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

    // Appendages from 2D field peaks (Axiom 6: emergent, not planned)
    let peaks = radial_field::detect_peaks(qe_field, crate::batch::constants::PEAK_THRESHOLD_FACTOR);
    let n_peaks = radial_field::peak_count(&peaks);
    if n_peaks == 0 || trunk_spine.len() < 3 {
        return trunk_mesh;
    }

    let spine_positions: Vec<Vec3> = trunk_spine.iter().map(|n| n.position).collect();
    let total_field_qe = radial_field::radial_total(qe_field);

    let mut all_meshes = vec![trunk_mesh];
    for p in 0..n_peaks {
        let (peak_ax, peak_rad, peak_qe) = peaks[p];
        if peak_qe < crate::batch::constants::APPENDAGE_QE_MIN { continue; }

        // Pure functions: 2D field peak → 3D geometry params (EM-3A + EM-3B)
        let (attach_pos, branch_dir) = radial_field::peak_to_3d_offset(
            peak_ax, peak_rad, &spine_positions,
        );
        let ar = radial_field::peak_aspect_ratio(qe_field, peak_ax, peak_rad);
        let (app_length, local_radius, _) = radial_field::peak_to_spine_params(
            peak_qe, ar, length, radius, total_field_qe,
        );

        let local_freq = freq_field[peak_ax as usize][peak_rad as usize];
        let branch_tint = frequency_to_tint_rgb(local_freq);

        let appendage_influence = GeometryInfluence {
            detail: detail * 0.7,
            energy_direction: branch_dir,
            energy_strength: 0.2 + mobility_bias * 0.3,
            resistance: resistance * (1.0 - resilience * 0.5),
            least_resistance_direction: branch_dir,
            length_budget: app_length,
            max_segments: (segments / 3).max(4),
            radius_base: local_radius,
            start_position: attach_pos,
            qe_norm: (peak_qe / radial_field::radial_total(qe_field).max(1e-6)).min(1.0),
            tint_rgb: branch_tint,
            branch_role: BranchRole::Leaf,
        };
        let app_spine = geometry_flow::build_flow_spine(&appendage_influence);

        // EM-4: Joint articulation — extract profile, detect joints, segment
        let direction: i8 = if peak_ax < (radial_field::AXIAL / 2) as u8 { -1 } else { 1 };
        let (profile, prof_len) = radial_field::extract_appendage_profile(
            qe_field, peak_ax, peak_rad, direction,
        );
        let joints = radial_field::detect_appendage_joints(&profile, prof_len);
        let n_joints = radial_field::appendage_joint_count(&joints);

        if n_joints > 0 && app_spine.len() > 2 {
            // Segmented: apply joint-thinned radii
            let seg_radii = radial_field::segmented_radii(
                local_radius, &joints, n_joints, app_spine.len(),
            );
            let spine_radii = interpolate_radii_to_spine(
                &seg_radii, app_spine.len(),
            );
            all_meshes.push(geometry_flow::build_flow_mesh_variable_radius(
                &app_spine, &appendage_influence, &spine_radii,
            ));
        } else {
            // No joints: single tube (existing behavior)
            all_meshes.push(geometry_flow::build_flow_mesh(&app_spine, &appendage_influence));
        }
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
    fn radial_field_mesh_has_vertices() {
        let field = radial_field::distribute_to_radial(100.0, 0.8, 0.5, 0.6);
        let freq = [[440.0; 4]; 8];
        let mesh = build_creature_mesh_with_field(0.8, 0.5, 0.6, 0.5, 440.0, &field, &freq);
        assert!(vertex_count(&mesh) > 0);
    }

    #[test]
    fn radial_field_with_peaks_has_appendages() {
        let mut field = [[2.0; 4]; 8];
        field[3][1] = 30.0; // strong lateral peak → appendage
        field[3][3] = 30.0; // bilateral
        let freq = [[440.0; 4]; 8];
        let with_peaks = build_creature_mesh_with_field(0.8, 0.5, 0.8, 0.5, 440.0, &field, &freq);
        let without = build_creature_mesh_with_field(0.8, 0.5, 0.0, 0.5, 440.0,
            &[[3.0; 4]; 8], &freq);
        assert!(vertex_count(&with_peaks) > vertex_count(&without),
            "peaks should add appendage vertices");
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
