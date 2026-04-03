//! GF2D — Sistema ECS de deformación geométrica (derivación visual, `Update`).
//!
//! Transforma mallas usando el motor de deformación termodinámica.
//! Corre en `Update` (derivación visual explícita, no simulación).

use bevy::prelude::*;

use crate::blueprint::equations::{
    BranchRole, calculate_tropism_vector, energy_gradient_from_neighbors,
};
use crate::geometry_flow::deformation::{
    DeformationPayload, apply_spine_to_mesh, deform_spine, deformation_fingerprint,
};
use crate::geometry_flow::deformation_cache::GeometryDeformationCache;
use crate::geometry_flow::{GeometryInfluence, build_flow_spine};
use crate::layers::{BaseEnergy, OscillatorySignature};
use crate::worldgen::EnergyFieldGrid;
use crate::worldgen::contracts::Materialized;

/// Capacidad del cache de deformación (parallel-array, modulo entity index).
const GF2_CACHE_CAPACITY: usize = 4096;
/// Escala de gravedad por defecto [m/s²].
const GF2_DEFAULT_GRAVITY_SCALE: f32 = 9.8;

/// Deforma las mallas de entidades materializadas usando tensores termodinámicos.
///
/// Lee energía + oscilación de las capas ECS, muestrea el gradiente del campo V7,
/// calcula el spine deformado y escribe las posiciones en el asset de `Mesh`.
pub fn geometry_deformation_system(
    mut meshes: ResMut<Assets<Mesh>>,
    mut cache: ResMut<GeometryDeformationCache>,
    grid: Option<Res<EnergyFieldGrid>>,
    query: Query<(
        Entity,
        &BaseEnergy,
        &OscillatorySignature,
        &Mesh3d,
        &Materialized,
    )>,
) {
    let grid_ref = grid.as_deref();

    for (entity, energy, oscillation, mesh3d, mat) in &query {
        let absorbed_energy = energy.qe().max(0.0);
        let bond_energy = (1.0 - oscillation.phase().sin().abs()).clamp(0.0, 1.0);

        let field_energy_dir = sample_field_gradient(grid_ref, mat);

        let (t_energy, t_gravity) = calculate_tropism_vector(
            absorbed_energy,
            bond_energy,
            field_energy_dir,
            GF2_DEFAULT_GRAVITY_SCALE,
        );

        let influence = build_minimal_influence(energy, oscillation, field_energy_dir);
        let base_spine = build_flow_spine(&influence);

        let payload = DeformationPayload {
            base_spine,
            t_energy,
            t_gravity,
            bond_energy,
            gravity_scale: GF2_DEFAULT_GRAVITY_SCALE,
        };

        let fingerprint = deformation_fingerprint(&payload);
        let tensor_magnitude = (t_energy + t_gravity).length();

        let cache_idx = entity.index() as usize % GF2_CACHE_CAPACITY;

        if cache
            .lookup(fingerprint, tensor_magnitude, cache_idx)
            .is_some()
        {
            cache.record_hit(cache_idx);
            continue;
        }

        let deformed = deform_spine(&payload);

        let Some(mesh) = meshes.get_mut(&mesh3d.0) else {
            continue;
        };

        let Some(positions_attr) = mesh.attribute(Mesh::ATTRIBUTE_POSITION) else {
            continue;
        };

        let bevy::render::mesh::VertexAttributeValues::Float32x3(base_positions) = positions_attr
        else {
            continue;
        };

        let base_positions_owned: Vec<[f32; 3]> = base_positions.to_vec();
        let new_positions = apply_spine_to_mesh(&base_positions_owned, &deformed);

        // Guard: sólo escribe si hay cambio real.
        if base_positions_owned != new_positions {
            mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, new_positions.clone());
            cache.update(cache_idx, fingerprint, new_positions, tensor_magnitude);
        }
    }
}

/// Muestrea el gradiente de energía del campo V7 en la celda de la entidad.
///
/// Diferencias finitas centrales sobre vecinos 4-connected.
/// Sin grid → `Vec3::ZERO` (sin deformación por campo).
fn sample_field_gradient(grid: Option<&EnergyFieldGrid>, mat: &Materialized) -> Vec3 {
    let Some(g) = grid else {
        return Vec3::ZERO;
    };
    if mat.cell_x < 0 || mat.cell_y < 0 {
        return Vec3::ZERO;
    }
    let ux = mat.cell_x as u32;
    let uy = mat.cell_y as u32;
    let center = g.cell_xy(ux, uy).map_or(0.0, |c| c.accumulated_qe);
    let left = if ux > 0 {
        g.cell_xy(ux - 1, uy).map_or(center, |c| c.accumulated_qe)
    } else {
        center
    };
    let right = g.cell_xy(ux + 1, uy).map_or(center, |c| c.accumulated_qe);
    let down = if uy > 0 {
        g.cell_xy(ux, uy - 1).map_or(center, |c| c.accumulated_qe)
    } else {
        center
    };
    let up = g.cell_xy(ux, uy + 1).map_or(center, |c| c.accumulated_qe);
    energy_gradient_from_neighbors(left, right, down, up, g.cell_size)
}

fn build_minimal_influence(
    energy: &BaseEnergy,
    oscillation: &OscillatorySignature,
    energy_direction: Vec3,
) -> GeometryInfluence {
    let qe_norm = (energy.qe() / 1000.0_f32).clamp(0.0, 1.0);
    let freq = oscillation.frequency_hz();
    let length = (freq * 0.001 + 0.5).clamp(0.2, 4.0);

    GeometryInfluence {
        detail: qe_norm,
        energy_direction,
        energy_strength: energy.qe().clamp(0.0, 100.0),
        resistance: 0.5,
        least_resistance_direction: Vec3::Y,
        length_budget: length,
        max_segments: 8,
        radius_base: 0.05,
        start_position: Vec3::ZERO,
        qe_norm,
        tint_rgb: [0.4, 0.8, 0.3],
        branch_role: BranchRole::Stem,
    }
}
