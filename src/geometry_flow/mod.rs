//! Motor de geometría **stateless** para eje tipo flora / flujo (sprint GF1).
//!
//! No lee ECS ni `EnergyFieldGrid`; el llamador inyecta [`GeometryInfluence`] en el hex boundary.

use std::f32::consts::TAU;

use crate::math_types::Vec3;
use bevy::prelude::Mesh;
use bevy::render::mesh::{Indices, PrimitiveTopology};
use bevy::render::render_asset::RenderAssetUsages;

use crate::blueprint::equations::{
    BranchRole, FLOW_BREAK_STEER_BLEND, branch_role_modulated_linear_rgb,
    flow_maintain_straight_segment, flow_push_along_tangent, flow_steered_tangent,
    vertex_flow_color,
};

pub mod branching;
pub mod deformation;
pub mod deformation_cache;
pub mod geometry_deformation_system;
pub mod primitives;

/// Mínimo de segmentos del spine (GF1).
pub const GF1_MIN_SEGMENTS: u32 = 4;
/// Mínimo de vértices por anillo del tubo.
pub const GF1_MIN_RING_VERTS: u32 = 4;
/// Máximo de vértices por anillo (techo con `detail = 1`).
pub const GF1_MAX_RING_VERTS: u32 = 12;

/// Paquete inyectado en el boundary: toda la semántica “de mundo” viene resuelta aquí.
#[derive(Debug, Clone, Copy)]
pub struct GeometryInfluence {
    /// LOD \([0,1]\): más alto → más segmentos y más vértices por sección.
    pub detail: f32,
    pub energy_direction: Vec3,
    pub energy_strength: f32,
    pub resistance: f32,
    pub least_resistance_direction: Vec3,
    pub length_budget: f32,
    pub max_segments: u32,
    pub radius_base: f32,
    pub start_position: Vec3,
    /// \([0,1]\) para inferencia de color (resuelto fuera del motor).
    pub qe_norm: f32,
    pub tint_rgb: [f32; 3],
    /// Rol de esta sub-rama / tronco (EPI3); modulación en puras del blueprint.
    pub branch_role: BranchRole,
}

impl GeometryInfluence {
    #[inline]
    pub fn clamped_detail(&self) -> f32 {
        self.detail.clamp(0.0, 1.0)
    }

    /// Número de segmentos (aristas) del spine; interpola entre mínimo y `max_segments` con `detail`.
    pub fn segment_count(&self) -> u32 {
        let d = self.clamped_detail();
        let min_s = GF1_MIN_SEGMENTS;
        let max_s = self.max_segments.max(min_s);
        let n = (min_s as f32 + d * (max_s as f32 - min_s as f32)).round() as u32;
        n.clamp(min_s, max_s)
    }

    /// Vértices por anillo perpendicular al spine.
    pub fn ring_vertex_count(&self) -> u32 {
        let d = self.clamped_detail();
        let min_r = GF1_MIN_RING_VERTS;
        let max_r = GF1_MAX_RING_VERTS;
        let n = (min_r as f32 + d * (max_r as f32 - min_r as f32)).round() as u32;
        n.clamp(min_r, max_r)
    }
}

/// Nodo del eje central con posición, tangente y tinte por nodo (EPI3).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SpineNode {
    pub position: Vec3,
    pub tangent: Vec3,
    /// RGB lineal por nodo (muestreo campo o fallback del influence).
    pub tint_rgb: [f32; 3],
    pub qe_norm: f32,
}

/// Construye la polilínea del flujo: recto mientras el empuje a lo largo de la tangente ≥ resistencia;
/// si no, mezcla hacia `least_resistance_direction`.
pub fn build_flow_spine(influence: &GeometryInfluence) -> Vec<SpineNode> {
    build_flow_spine_painted(influence, |_, inf| (inf.tint_rgb, inf.qe_norm))
}

/// EPI3: compone muestreo crudo de campo (RGB + qe) con [`GeometryInfluence::branch_role`] en un solo sitio.
#[inline]
pub fn spine_paint_vertex_from_raw_field(
    pos: Vec3,
    influence: &GeometryInfluence,
    field_raw: &dyn Fn(Vec3) -> ([f32; 3], f32),
) -> ([f32; 3], f32) {
    let (rgb, qn) = field_raw(pos);
    (branch_role_modulated_linear_rgb(rgb, influence.branch_role), qn)
}

/// Spine con color por nodo: `paint(pos, influence)` devuelve RGB lineal y `qe_norm` ya acotados por el llamador.
pub fn build_flow_spine_painted<F>(influence: &GeometryInfluence, mut paint: F) -> Vec<SpineNode>
where
    F: FnMut(Vec3, &GeometryInfluence) -> ([f32; 3], f32),
{
    let n = influence.segment_count();
    let step = (influence.length_budget / n.max(1) as f32).max(1e-4);
    let mut pos = influence.start_position;
    let mut tangent = influence.energy_direction.normalize_or_zero();
    if tangent.length_squared() < 1e-12 {
        tangent = Vec3::Y;
    }
    let energy_vec = influence.energy_direction.normalize_or_zero() * influence.energy_strength;
    let least = influence.least_resistance_direction;

    let mut out = Vec::with_capacity(n as usize + 1);
    let (t0, q0) = paint(pos, influence);
    out.push(SpineNode {
        position: pos,
        tangent,
        tint_rgb: t0,
        qe_norm: q0.clamp(0.0, 1.0),
    });

    for _ in 0..n {
        let push = flow_push_along_tangent(energy_vec, tangent);
        let straight = flow_maintain_straight_segment(push, influence.resistance);
        let blend = if straight {
            0.0
        } else {
            FLOW_BREAK_STEER_BLEND
        };
        tangent = flow_steered_tangent(tangent, least, blend);
        if tangent.length_squared() < 1e-12 {
            tangent = Vec3::Y;
        }
        pos += tangent * step;
        let (t, q) = paint(pos, influence);
        out.push(SpineNode {
            position: pos,
            tangent,
            tint_rgb: t,
            qe_norm: q.clamp(0.0, 1.0),
        });
    }
    out
}

fn orthonormal_ring_axes(tangent: Vec3) -> (Vec3, Vec3) {
    let t = tangent.normalize_or_zero();
    let up = if t.dot(Vec3::Y).abs() > 0.92 {
        Vec3::X
    } else {
        Vec3::Y
    };
    let mut n = t.cross(up);
    if n.length_squared() < 1e-12 {
        n = t.cross(Vec3::Z);
    }
    n = n.normalize_or_zero();
    if n.length_squared() < 1e-12 {
        n = Vec3::X;
    }
    let b = n.cross(t).normalize_or_zero();
    (n, b)
}

/// Tubo triangular alrededor del spine; atributos listos para PBR con vertex color.
pub fn build_flow_mesh(spine: &[SpineNode], influence: &GeometryInfluence) -> Mesh {
    let ring_n = influence.ring_vertex_count() as usize;
    let spine_n = spine.len();
    if spine_n < 2 || ring_n < 3 {
        return Mesh::new(
            PrimitiveTopology::TriangleList,
            RenderAssetUsages::default(),
        );
    }

    let radius = influence.radius_base.max(1e-4);
    let mut positions: Vec<[f32; 3]> = Vec::with_capacity(spine_n * ring_n);
    let mut normals: Vec<[f32; 3]> = Vec::with_capacity(spine_n * ring_n);
    let mut uvs: Vec<[f32; 2]> = Vec::with_capacity(spine_n * ring_n);
    let mut colors: Vec<[f32; 4]> = Vec::with_capacity(spine_n * ring_n);

    let denom_s = (spine_n - 1).max(1) as f32;

    for (i, node) in spine.iter().enumerate() {
        let (n_axis, b_axis) = orthonormal_ring_axes(node.tangent);
        let s_along = i as f32 / denom_s;
        for j in 0..ring_n {
            let theta = TAU * j as f32 / ring_n as f32;
            let c = theta.cos();
            let s = theta.sin();
            let radial_dir = n_axis * c + b_axis * s;
            let p = node.position + radial_dir * radius;
            positions.push(p.to_array());
            normals.push(radial_dir.normalize_or_zero().to_array());
            let u = j as f32 / ring_n as f32;
            let v = s_along;
            uvs.push([u, v]);
            let azimuth_t = j as f32 / ring_n as f32;
            colors.push(vertex_flow_color(
                node.qe_norm,
                node.tint_rgb,
                s_along,
                azimuth_t,
            ));
        }
    }

    let mut indices: Vec<u32> = Vec::with_capacity((spine_n - 1) * ring_n * 6);
    for i in 0..spine_n - 1 {
        for j in 0..ring_n {
            let jn = (j + 1) % ring_n;
            let a = (i * ring_n + j) as u32;
            let b = (i * ring_n + jn) as u32;
            let c = ((i + 1) * ring_n + j) as u32;
            let d = ((i + 1) * ring_n + jn) as u32;
            indices.extend_from_slice(&[a, c, b, b, c, d]);
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

/// Merges multiple meshes into one by concatenating vertex buffers and offsetting indices.
///
/// Handles POSITION, NORMAL, UV_0, COLOR (Float32x3/x2/x4). Missing attributes are synthesized
/// with sensible defaults. All indices are remapped to U32.
pub fn merge_meshes(meshes: &[Mesh]) -> Mesh {
    use bevy::render::mesh::VertexAttributeValues;

    let mut positions: Vec<[f32; 3]> = Vec::new();
    let mut normals:   Vec<[f32; 3]> = Vec::new();
    let mut uvs:       Vec<[f32; 2]> = Vec::new();
    let mut colors:    Vec<[f32; 4]> = Vec::new();
    let mut indices:   Vec<u32>      = Vec::new();

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

/// Cuenta triángulos (tests / presupuesto).
pub fn flow_mesh_triangle_count(mesh: &Mesh) -> usize {
    match mesh.indices() {
        Some(Indices::U32(id)) => id.len() / 3,
        Some(Indices::U16(id)) => id.len() / 3,
        None => 0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_influence(detail: f32) -> GeometryInfluence {
        GeometryInfluence {
            detail,
            energy_direction: Vec3::new(0.0, 1.0, 0.0),
            energy_strength: 2.0,
            resistance: 1.0,
            least_resistance_direction: Vec3::new(1.0, 0.0, 0.0),
            length_budget: 3.0,
            max_segments: 16,
            radius_base: 0.08,
            start_position: Vec3::ZERO,
            qe_norm: 0.7,
            tint_rgb: [0.2, 0.8, 0.3],
            branch_role: BranchRole::Stem,
        }
    }

    #[test]
    fn detail_monotonic_triangle_count() {
        let inf_lo = sample_influence(0.0);
        let inf_hi = sample_influence(1.0);
        let spine_lo = build_flow_spine(&inf_lo);
        let spine_hi = build_flow_spine(&inf_hi);
        let m_lo = build_flow_mesh(&spine_lo, &inf_lo);
        let m_hi = build_flow_mesh(&spine_hi, &inf_hi);
        let t_lo = flow_mesh_triangle_count(&m_lo);
        let t_hi = flow_mesh_triangle_count(&m_hi);
        assert!(
            t_hi >= t_lo,
            "más detail no debe reducir triángulos: {t_lo} vs {t_hi}"
        );
    }

    #[test]
    fn vertex_flow_color_uses_injected_qe_norm_not_implicit_state() {
        use crate::blueprint::equations::vertex_flow_color;
        let a = vertex_flow_color(0.2, [1.0, 0.0, 0.0], 0.5, 0.5);
        let b = vertex_flow_color(0.95, [1.0, 0.0, 0.0], 0.5, 0.5);
        assert!(
            b[0] > a[0],
            "más qe_norm debe aclarar canales vía parámetro explícito"
        );
    }

    #[test]
    fn same_influence_deterministic_mesh() {
        let inf = sample_influence(0.5);
        let s1 = build_flow_spine(&inf);
        let s2 = build_flow_spine(&inf);
        assert_eq!(s1, s2);
        let m1 = build_flow_mesh(&s1, &inf);
        let m2 = build_flow_mesh(&s2, &inf);
        assert_eq!(flow_mesh_triangle_count(&m1), flow_mesh_triangle_count(&m2));
    }

    #[test]
    fn build_flow_spine_painted_varies_tint_per_node() {
        let inf = sample_influence(0.4);
        let spine = build_flow_spine_painted(&inf, |pos, _| {
            let r = (pos.y * 0.1 + 0.2).clamp(0.0, 1.0);
            ([r, 0.5, 0.1], 0.5)
        });
        assert!(spine.len() >= 2);
        assert_ne!(spine[0].tint_rgb, spine[spine.len() - 1].tint_rgb);
    }
}
