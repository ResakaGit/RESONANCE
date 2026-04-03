//! C4: Mesh Export — GF1 evolved meshes to OBJ format for 3D printing / Blender.
//!
//! Pure data transform: GenomeBlob → radial field → GF1 mesh → OBJ string.

use crate::batch::bridge;
use crate::batch::genome::GenomeBlob;
use crate::blueprint::equations::radial_field;
use crate::geometry_flow::creature_builder;

/// Export an evolved genome as OBJ text.
///
/// Returns OBJ content as a String — positions, normals, and faces.
/// Consumer writes to file. No I/O here (pure function).
pub fn genome_to_obj(genome: &GenomeBlob) -> String {
    let freq = bridge::genome_to_components(genome).2.frequency_hz();
    // Viewer normalization: growth ∈ [0,1] → qe ∈ [20, 100] for stable mesh rendering
    let qe = 20.0 + genome.growth_bias * 80.0;

    let field = radial_field::build_viewer_field(
        genome.growth_bias,
        genome.resilience,
        genome.branching_bias,
        qe,
    );
    let freq_field = radial_field::build_viewer_freq_field(freq);

    let mesh = creature_builder::build_creature_mesh_with_field(
        genome.growth_bias,
        genome.mobility_bias,
        genome.branching_bias,
        genome.resilience,
        freq,
        &field,
        &freq_field,
    );

    bevy_mesh_to_obj(&mesh)
}

/// Convert a Bevy Mesh to OBJ text format.
fn bevy_mesh_to_obj(mesh: &bevy::render::mesh::Mesh) -> String {
    use bevy::render::mesh::VertexAttributeValues;

    let positions = match mesh.attribute(bevy::render::mesh::Mesh::ATTRIBUTE_POSITION) {
        Some(VertexAttributeValues::Float32x3(p)) => p,
        _ => return String::from("# Error: no positions in mesh\n"),
    };

    let normals = mesh.attribute(bevy::render::mesh::Mesh::ATTRIBUTE_NORMAL);

    let indices: Vec<u32> = match mesh.indices() {
        Some(bevy::render::mesh::Indices::U32(i)) => i.clone(),
        Some(bevy::render::mesh::Indices::U16(i)) => i.iter().map(|&x| x as u32).collect(),
        None => return String::from("# Error: no indices in mesh\n"),
    };

    let mut obj = String::with_capacity(positions.len() * 40 + indices.len() * 15);
    obj.push_str("# Resonance GF1 — evolved creature mesh\n");
    obj.push_str(&format!(
        "# vertices: {} faces: {}\n",
        positions.len(),
        indices.len() / 3
    ));

    for [x, y, z] in positions {
        obj.push_str(&format!("v {x:.6} {y:.6} {z:.6}\n"));
    }

    if let Some(VertexAttributeValues::Float32x3(norms)) = normals {
        for [nx, ny, nz] in norms {
            obj.push_str(&format!("vn {nx:.6} {ny:.6} {nz:.6}\n"));
        }
    }

    let has_normals = normals.is_some();
    for tri in indices.chunks(3) {
        if tri.len() == 3 {
            let (a, b, c) = (tri[0] + 1, tri[1] + 1, tri[2] + 1);
            if has_normals {
                obj.push_str(&format!("f {a}//{a} {b}//{b} {c}//{c}\n"));
            } else {
                obj.push_str(&format!("f {a} {b} {c}\n"));
            }
        }
    }

    obj
}
