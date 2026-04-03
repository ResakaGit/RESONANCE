//! IWG-5 — Water surface inference equations: height, color, mesh generation.
//!
//! DEBT: This equation file imports Bevy Mesh/Indices/PrimitiveTopology to construct
//! render-ready meshes. The pure math (height, color interpolation) is Bevy-free,
//! but `water_surface_mesh()` returns `bevy::Mesh` directly. Proper fix: return raw
//! vertex/index arrays and let a system/bridge construct the Mesh. Low priority because
//! this is cold-path (called once per water body, not per tick).

use crate::math_types::Vec3;
use bevy::prelude::Mesh;
use bevy::render::mesh::{Indices, PrimitiveTopology};
use bevy::render::render_asset::RenderAssetUsages;

use crate::blueprint::constants::inferred_world_geometry::{
    WATER_COLOR_DEEP, WATER_COLOR_MEDIUM, WATER_COLOR_SHALLOW, WATER_DEEP_DEPTH,
    WATER_SHALLOW_DEPTH, WATER_SURFACE_OFFSET,
};
use crate::blueprint::equations::field_color::linear_rgb_lerp_preclamped;

/// Water surface height from terrain heights of liquid cells.
///
/// `height = max(avg(heights), min_terrain_height) + WATER_SURFACE_OFFSET`, clamped to [0, 200].
pub fn water_surface_height(liquid_terrain_heights: &[f32], min_terrain_height: f32) -> f32 {
    if liquid_terrain_heights.is_empty() {
        return min_terrain_height;
    }
    let avg =
        liquid_terrain_heights.iter().copied().sum::<f32>() / liquid_terrain_heights.len() as f32;
    (avg.max(min_terrain_height) + WATER_SURFACE_OFFSET).clamp(0.0, 200.0)
}

/// Water color based on depth (distance between surface and terrain).
///
/// Shallow (< 0.5): `WATER_COLOR_SHALLOW`.
/// Medium (0.5..2.0): lerp from shallow to medium.
/// Deep (>= 2.0): `WATER_COLOR_DEEP`.
pub fn water_depth_color(depth: f32) -> [f32; 3] {
    let d = depth.max(0.0);
    if d < WATER_SHALLOW_DEPTH {
        WATER_COLOR_SHALLOW
    } else if d < WATER_DEEP_DEPTH {
        let t = (d - WATER_SHALLOW_DEPTH) / (WATER_DEEP_DEPTH - WATER_SHALLOW_DEPTH);
        linear_rgb_lerp_preclamped(WATER_COLOR_SHALLOW, WATER_COLOR_MEDIUM, t)
    } else {
        WATER_COLOR_DEEP
    }
}

/// Builds a flat water mesh for a rectangular region with depth-based vertex colors.
///
/// Grid of `(subdivisions+1)^2` vertices in the XZ plane, all at `water_height` Y.
/// Vertex color derived from `water_depth_color(water_height - terrain_height_at(x,z))`.
pub fn build_water_mesh(
    bounds_min: Vec3,
    bounds_max: Vec3,
    water_height: f32,
    subdivisions: u32,
    terrain_heights: &[f32],
    rows: u32,
    cols: u32,
) -> Mesh {
    let subs = subdivisions.max(1);
    let verts_per_side = subs + 1;
    let n = (verts_per_side * verts_per_side) as usize;

    let x_range = bounds_max.x - bounds_min.x;
    let z_range = bounds_max.z - bounds_min.z;

    let mut positions: Vec<[f32; 3]> = Vec::with_capacity(n);
    let mut normals: Vec<[f32; 3]> = Vec::with_capacity(n);
    let mut uvs: Vec<[f32; 2]> = Vec::with_capacity(n);
    let mut colors: Vec<[f32; 4]> = Vec::with_capacity(n);

    for iy in 0..verts_per_side {
        for ix in 0..verts_per_side {
            let x_norm = ix as f32 / subs as f32;
            let z_norm = iy as f32 / subs as f32;
            let px = bounds_min.x + x_range * x_norm;
            let pz = bounds_min.z + z_range * z_norm;

            positions.push([px, water_height, pz]);
            normals.push([0.0, 1.0, 0.0]);
            uvs.push([x_norm, z_norm]);

            let terrain_h = sample_terrain_height_nearest(
                px,
                pz,
                bounds_min.x,
                bounds_min.z,
                bounds_max.x,
                bounds_max.z,
                terrain_heights,
                rows,
                cols,
            );
            let depth = (water_height - terrain_h).max(0.0);
            let [r, g, b] = water_depth_color(depth);
            colors.push([r, g, b, 0.7]);
        }
    }

    let mut indices: Vec<u32> = Vec::with_capacity((subs * subs * 6) as usize);
    for iy in 0..subs {
        for ix in 0..subs {
            let cur = iy * verts_per_side + ix;
            let next_row = (iy + 1) * verts_per_side + ix;
            indices.extend_from_slice(&[cur, next_row, cur + 1, cur + 1, next_row, next_row + 1]);
        }
    }

    let mut mesh = Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::default(),
    );
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
    mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
    mesh.insert_attribute(Mesh::ATTRIBUTE_COLOR, colors);
    mesh.insert_indices(Indices::U32(indices));
    mesh
}

/// Nearest-neighbor terrain height sample from a flat row-major array.
fn sample_terrain_height_nearest(
    world_x: f32,
    world_z: f32,
    min_x: f32,
    min_z: f32,
    max_x: f32,
    max_z: f32,
    terrain_heights: &[f32],
    rows: u32,
    cols: u32,
) -> f32 {
    if rows == 0 || cols == 0 || terrain_heights.is_empty() {
        return 0.0;
    }
    let x_range = (max_x - min_x).max(1e-6);
    let z_range = (max_z - min_z).max(1e-6);
    let tx = ((world_x - min_x) / x_range).clamp(0.0, 1.0);
    let tz = ((world_z - min_z) / z_range).clamp(0.0, 1.0);
    let col = ((tx * (cols as f32 - 1.0)).round() as u32).min(cols - 1);
    let row = ((tz * (rows as f32 - 1.0)).round() as u32).min(rows - 1);
    let idx = row as usize * cols as usize + col as usize;
    terrain_heights.get(idx).copied().unwrap_or(0.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn water_surface_height_average_plus_offset() {
        let heights = [1.0, 2.0, 3.0, 4.0];
        let h = water_surface_height(&heights, 0.0);
        // avg = 2.5, + 0.2 = 2.7
        assert!((h - 2.7).abs() < 1e-5);
    }

    #[test]
    fn water_surface_height_empty_returns_min() {
        let h = water_surface_height(&[], 5.0);
        assert!((h - 5.0).abs() < 1e-5);
    }

    #[test]
    fn water_depth_color_shallow() {
        let c = water_depth_color(0.0);
        assert_eq!(c, WATER_COLOR_SHALLOW);
        let c2 = water_depth_color(0.3);
        assert_eq!(c2, WATER_COLOR_SHALLOW);
    }

    #[test]
    fn water_depth_color_deep() {
        let c = water_depth_color(2.0);
        assert_eq!(c, WATER_COLOR_DEEP);
        let c2 = water_depth_color(10.0);
        assert_eq!(c2, WATER_COLOR_DEEP);
    }

    #[test]
    fn water_depth_color_medium_lerp() {
        // Midpoint between shallow and deep thresholds.
        let mid = (WATER_SHALLOW_DEPTH + WATER_DEEP_DEPTH) / 2.0;
        let c = water_depth_color(mid);
        // Should be halfway between shallow and medium colors.
        for i in 0..3 {
            let expected = (WATER_COLOR_SHALLOW[i] + WATER_COLOR_MEDIUM[i]) / 2.0;
            assert!(
                (c[i] - expected).abs() < 1e-5,
                "channel {i}: expected {expected}, got {}",
                c[i]
            );
        }
    }

    #[test]
    fn build_water_mesh_positions_normals_uvs_colors_synced() {
        use bevy::render::mesh::VertexAttributeValues;

        let terrain = vec![0.0; 4]; // 2x2 flat at 0
        let mesh = build_water_mesh(
            Vec3::new(0.0, 0.0, 0.0),
            Vec3::new(10.0, 0.0, 10.0),
            2.0,
            2,
            &terrain,
            2,
            2,
        );

        let pos_count = match mesh.attribute(Mesh::ATTRIBUTE_POSITION) {
            Some(VertexAttributeValues::Float32x3(v)) => v.len(),
            _ => panic!("expected Float32x3 positions"),
        };
        let norm_count = match mesh.attribute(Mesh::ATTRIBUTE_NORMAL) {
            Some(VertexAttributeValues::Float32x3(v)) => v.len(),
            _ => panic!("expected Float32x3 normals"),
        };
        let uv_count = match mesh.attribute(Mesh::ATTRIBUTE_UV_0) {
            Some(VertexAttributeValues::Float32x2(v)) => v.len(),
            _ => panic!("expected Float32x2 uvs"),
        };
        let col_count = match mesh.attribute(Mesh::ATTRIBUTE_COLOR) {
            Some(VertexAttributeValues::Float32x4(v)) => v.len(),
            _ => panic!("expected Float32x4 colors"),
        };

        // (2+1)^2 = 9 vertices
        assert_eq!(pos_count, 9);
        assert_eq!(norm_count, 9);
        assert_eq!(uv_count, 9);
        assert_eq!(col_count, 9);
    }

    #[test]
    fn build_water_mesh_normals_are_up() {
        use bevy::render::mesh::VertexAttributeValues;

        let terrain = vec![0.0; 4];
        let mesh = build_water_mesh(
            Vec3::new(0.0, 0.0, 0.0),
            Vec3::new(5.0, 0.0, 5.0),
            1.0,
            2,
            &terrain,
            2,
            2,
        );

        let VertexAttributeValues::Float32x3(normals) =
            mesh.attribute(Mesh::ATTRIBUTE_NORMAL).expect("normals")
        else {
            panic!("expected Float32x3 normals");
        };
        for n in normals {
            assert!((n[0]).abs() < 1e-6);
            assert!((n[1] - 1.0).abs() < 1e-6);
            assert!((n[2]).abs() < 1e-6);
        }
    }

    #[test]
    fn water_surface_determinism() {
        let heights = [1.0, 3.0, 2.0, 4.0, 0.5];
        let h_a = water_surface_height(&heights, 0.5);
        let h_b = water_surface_height(&heights, 0.5);
        assert_eq!(h_a, h_b);

        let c_a = water_depth_color(1.2);
        let c_b = water_depth_color(1.2);
        assert_eq!(c_a, c_b);

        let terrain = vec![0.0, 1.0, 0.5, 2.0];
        let mesh_a = build_water_mesh(Vec3::ZERO, Vec3::new(4.0, 0.0, 4.0), 2.0, 2, &terrain, 2, 2);
        let mesh_b = build_water_mesh(Vec3::ZERO, Vec3::new(4.0, 0.0, 4.0), 2.0, 2, &terrain, 2, 2);

        use bevy::render::mesh::VertexAttributeValues;
        let VertexAttributeValues::Float32x3(pos_a) =
            mesh_a.attribute(Mesh::ATTRIBUTE_POSITION).expect("pos")
        else {
            panic!("expected Float32x3");
        };
        let VertexAttributeValues::Float32x3(pos_b) =
            mesh_b.attribute(Mesh::ATTRIBUTE_POSITION).expect("pos")
        else {
            panic!("expected Float32x3");
        };
        assert_eq!(pos_a, pos_b);
    }
}
