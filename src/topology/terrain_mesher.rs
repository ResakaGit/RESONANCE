//! Malla de terreno **stateless**: geometría solo desde [`TerrainField`], tintes desde [`TerrainVisuals`].
//!
//! Contrato (docs/design/TERRAIN_MESHER.md): la altitud Y no mezcla V7 ni `accumulated_qe`; el color llega precruzado en `vertex_colors`.

use bevy::prelude::Mesh;
use bevy::render::mesh::{Indices, PrimitiveTopology};
use bevy::render::render_asset::RenderAssetUsages;

use super::TerrainField;

/// SoA de apariencia por celda (misma indexación row-major que `TerrainField::cell_index`).
#[derive(Clone, Debug)]
pub struct TerrainVisuals {
    pub vertex_colors: Vec<[f32; 4]>,
}

impl TerrainVisuals {
    /// Tinte uniforme neutro (hierba/roca apagado); útil como default hasta cruzar con V7 en ECS.
    pub fn neutral_flat(terrain: &TerrainField, rgba: [f32; 4]) -> Self {
        Self {
            vertex_colors: vec![rgba; terrain.total_cells()],
        }
    }
}

/// Construye malla triangular del relieve: posiciones Y = `terrain.altitude` exclusivamente.
///
/// Devuelve `None` si `visuals.vertex_colors.len() != terrain.total_cells()` (contrato DoD).
pub fn generate_terrain_mesh(terrain: &TerrainField, visuals: &TerrainVisuals) -> Option<Mesh> {
    let w = terrain.width as usize;
    let h = terrain.height as usize;
    if w < 2 || h < 2 {
        return Some(Mesh::new(
            PrimitiveTopology::TriangleList,
            RenderAssetUsages::default(),
        ));
    }
    let n = w * h;
    if visuals.vertex_colors.len() != n || terrain.altitude.len() != n {
        return None;
    }

    let cs = terrain.cell_size;
    let ox = terrain.origin.x;
    let oz = terrain.origin.y;

    let mut positions: Vec<[f32; 3]> = Vec::with_capacity(n);
    let mut normals: Vec<[f32; 3]> = vec![[0.0, 0.0, 0.0]; n];
    let mut uvs: Vec<[f32; 2]> = Vec::with_capacity(n);
    let mut colors: Vec<[f32; 4]> = Vec::with_capacity(n);

    for y in 0..h {
        for x in 0..w {
            let idx = y * w + x;
            let px = ox + x as f32 * cs;
            let pz = oz + y as f32 * cs;
            let py = terrain.altitude[idx];
            positions.push([px, py, pz]);
            uvs.push([
                x as f32 / (w - 1).max(1) as f32,
                y as f32 / (h - 1).max(1) as f32,
            ]);
            colors.push(visuals.vertex_colors[idx]);
        }
    }

    let mut indices: Vec<u32> = Vec::with_capacity((w - 1) * (h - 1) * 6);
    for y in 0..h - 1 {
        for x in 0..w - 1 {
            let cur = (y * w + x) as u32;
            let next_row = ((y + 1) * w + x) as u32;
            indices.extend_from_slice(&[cur, next_row, cur + 1, cur + 1, next_row, next_row + 1]);
        }
    }

    accumulate_smooth_normals(&positions, &indices, &mut normals);
    for nrm in &mut normals {
        let l = (nrm[0] * nrm[0] + nrm[1] * nrm[1] + nrm[2] * nrm[2]).sqrt();
        if l > 1e-8 {
            nrm[0] /= l;
            nrm[1] /= l;
            nrm[2] /= l;
        } else {
            *nrm = [0.0, 1.0, 0.0];
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
    Some(mesh)
}

fn accumulate_smooth_normals(positions: &[[f32; 3]], indices: &[u32], normals: &mut [[f32; 3]]) {
    for tri in indices.chunks_exact(3) {
        let i0 = tri[0] as usize;
        let i1 = tri[1] as usize;
        let i2 = tri[2] as usize;
        let Some(p0) = positions.get(i0) else {
            continue;
        };
        let Some(p1) = positions.get(i1) else {
            continue;
        };
        let Some(p2) = positions.get(i2) else {
            continue;
        };
        let e1 = [p1[0] - p0[0], p1[1] - p0[1], p1[2] - p0[2]];
        let e2 = [p2[0] - p0[0], p2[1] - p0[1], p2[2] - p0[2]];
        let nx = e1[1] * e2[2] - e1[2] * e2[1];
        let ny = e1[2] * e2[0] - e1[0] * e2[2];
        let nz = e1[0] * e2[1] - e1[1] * e2[0];
        for &i in tri {
            let Some(n) = normals.get_mut(i as usize) else {
                continue;
            };
            n[0] += nx;
            n[1] += ny;
            n[2] += nz;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::math::Vec2;
    use bevy::render::mesh::VertexAttributeValues;

    fn flat_terrain(w: u32, h: u32, altitude: f32, origin: Vec2) -> TerrainField {
        let mut t = TerrainField::new(w, h, 1.0, origin, 1);
        for a in &mut t.altitude {
            *a = altitude;
        }
        t
    }

    #[test]
    fn mesh_y_matches_terrain_altitude_only() {
        let mut t = TerrainField::new(3, 3, 2.0, Vec2::new(10.0, -5.0), 0);
        for (i, v) in t.altitude.iter_mut().enumerate() {
            *v = i as f32 * 0.5;
        }
        let vis = TerrainVisuals::neutral_flat(&t, [1.0, 0.0, 0.0, 1.0]);
        let mesh = generate_terrain_mesh(&t, &vis).expect("mesh");
        let VertexAttributeValues::Float32x3(pos) =
            mesh.attribute(Mesh::ATTRIBUTE_POSITION).expect("positions")
        else {
            panic!("expected Float32x3 positions");
        };
        let pos = pos.clone();
        assert_eq!(pos.len(), 9);
        for y in 0..3 {
            for x in 0..3 {
                let idx = y * 3 + x;
                let expected_y = t.altitude[idx];
                assert!(
                    (pos[idx][1] - expected_y).abs() < 1e-5,
                    "Y debe salir solo de TerrainField.altitude"
                );
                assert!((pos[idx][0] - (10.0 + x as f32 * 2.0)).abs() < 1e-5);
                assert!((pos[idx][2] - (-5.0 + y as f32 * 2.0)).abs() < 1e-5);
            }
        }
    }

    #[test]
    fn vertex_colors_follow_terrain_visuals_buffer() {
        let t = flat_terrain(2, 2, 0.0, Vec2::ZERO);
        let mut vis = TerrainVisuals::neutral_flat(&t, [0.0, 0.0, 0.0, 1.0]);
        vis.vertex_colors[0] = [0.1, 0.2, 0.3, 1.0];
        vis.vertex_colors[3] = [0.9, 0.8, 0.7, 1.0];
        let mesh = generate_terrain_mesh(&t, &vis).expect("mesh");
        let VertexAttributeValues::Float32x4(col) =
            mesh.attribute(Mesh::ATTRIBUTE_COLOR).expect("colors")
        else {
            panic!("expected Float32x4 colors");
        };
        let col = col.clone();
        assert_eq!(col[0], [0.1, 0.2, 0.3, 1.0]);
        assert_eq!(col[3], [0.9, 0.8, 0.7, 1.0]);
    }

    #[test]
    fn mismatched_visual_len_returns_none() {
        let t = flat_terrain(2, 2, 0.0, Vec2::ZERO);
        let bad = TerrainVisuals {
            vertex_colors: vec![[1.0; 4]; 3],
        };
        assert!(generate_terrain_mesh(&t, &bad).is_none());
    }
}
