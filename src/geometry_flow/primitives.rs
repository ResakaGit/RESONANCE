use std::f32::consts::{PI, TAU};

use bevy::math::Vec3;
use bevy::prelude::Mesh;
use bevy::render::mesh::{Indices, PrimitiveTopology};
use bevy::render::render_asset::RenderAssetUsages;

use crate::blueprint::equations::BranchRole;
use crate::geometry_flow::{
    GeometryInfluence, SpineNode, build_flow_mesh, vertex_along_flow_color,
};
use crate::layers::organ::{GeometryPrimitive, OrganSpec};

#[cfg(test)]
use bevy::render::mesh::VertexAttributeValues;

/// Curvatura cóncava por defecto para pétalos.
pub const PETAL_CURVATURE: f32 = 0.3;
/// Apertura máxima de pétalo externo (~57°); internas se cierran proporcionalmente.
pub const PETAL_DEFAULT_OPENING: f32 = 1.0;
/// Vértices transversales por fila de pétalo (curvatura cóncava requiere ≥ 4).
pub const PETAL_CROSS_VERTS: u32 = 4;
/// Curvatura sinusoidal por defecto para hojas.
pub const LEAF_CURVATURE: f32 = 0.15;
/// Elongación por defecto para bulbo.
pub const BULB_DEFAULT_ELONGATION: f32 = 1.3;
/// Ángulo áureo en radianes.
pub const GOLDEN_ANGLE: f32 = 2.399_963;

pub const PRIM_MIN_SUBDIVISIONS: u32 = 2;
pub const PRIM_MAX_SUBDIVISIONS: u32 = 8;
pub const BULB_MIN_RINGS: u32 = 3;
pub const BULB_MAX_RINGS: u32 = 8;
pub const BULB_MIN_SECTORS: u32 = 4;
pub const BULB_MAX_SECTORS: u32 = 12;
pub const MAX_PETAL_COUNT: u8 = 24;

/// Parámetros comunes de construcción para primitivas de órgano.
#[derive(Debug, Clone, Copy)]
pub struct OrganPrimitiveParams {
    pub origin: Vec3,
    pub direction: Vec3,
    pub tangent: Vec3,
    pub base_radius: f32,
    pub tint_rgb: [f32; 3],
    pub qe_norm: f32,
    pub detail: f32,
}

fn empty_mesh() -> Mesh {
    Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::default(),
    )
}

fn normalize_or(v: Vec3, fallback: Vec3) -> Vec3 {
    let n = v.normalize_or_zero();
    if n.length_squared() < 1e-12 {
        fallback
    } else {
        n
    }
}

fn orthonormal_basis(primary: Vec3, secondary_hint: Vec3) -> (Vec3, Vec3, Vec3) {
    let y = normalize_or(primary, Vec3::Y);
    let mut x = secondary_hint - y * secondary_hint.dot(y);
    if x.length_squared() < 1e-12 {
        let aux = if y.dot(Vec3::X).abs() < 0.9 {
            Vec3::X
        } else {
            Vec3::Z
        };
        x = aux - y * aux.dot(y);
    }
    let x = normalize_or(x, Vec3::X);
    let z = normalize_or(y.cross(x), Vec3::Z);
    (x, y, z)
}

fn detail_to_steps(detail: f32, min: u32, max: u32) -> u32 {
    let d = detail.clamp(0.0, 1.0);
    (min as f32 + d * (max as f32 - min as f32))
        .round()
        .clamp(min as f32, max as f32) as u32
}

#[derive(Default)]
struct MeshBuffers {
    positions: Vec<[f32; 3]>,
    normals: Vec<[f32; 3]>,
    uvs: Vec<[f32; 2]>,
    colors: Vec<[f32; 4]>,
    indices: Vec<u32>,
}

impl MeshBuffers {
    fn with_capacity(vertex_capacity: usize, index_capacity: usize) -> Self {
        Self {
            positions: Vec::with_capacity(vertex_capacity),
            normals: Vec::with_capacity(vertex_capacity),
            uvs: Vec::with_capacity(vertex_capacity),
            colors: Vec::with_capacity(vertex_capacity),
            indices: Vec::with_capacity(index_capacity),
        }
    }

    fn into_mesh(self) -> Mesh {
        let mut mesh = empty_mesh();
        mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, self.positions);
        mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, self.normals);
        mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, self.uvs);
        mesh.insert_attribute(Mesh::ATTRIBUTE_COLOR, self.colors);
        mesh.insert_indices(Indices::U32(self.indices));
        mesh
    }
}

/// Construye una superficie plana subdividida, doble cara.
pub fn build_flat_surface(
    origin: Vec3,
    normal: Vec3,
    tangent: Vec3,
    length: f32,
    width: f32,
    subdivisions: u32,
    tint_rgb: [f32; 3],
    qe_norm: f32,
) -> Mesh {
    let sub = subdivisions.clamp(PRIM_MIN_SUBDIVISIONS, PRIM_MAX_SUBDIVISIONS);
    if length <= 0.0 || width <= 0.0 {
        return empty_mesh();
    }

    let (x_axis, y_axis, z_axis) = orthonormal_basis(normal, tangent);
    let tangent_axis = normalize_or(z_axis.cross(y_axis), x_axis);
    let half_w = width * 0.5;
    let front_vtx = ((sub + 1) * 2) as usize;
    let mut buffers = MeshBuffers::with_capacity(front_vtx * 2, (sub * 12) as usize);

    for i in 0..=sub {
        let u = i as f32 / sub as f32;
        let center = origin + tangent_axis * ((u - 0.5) * length);
        let base_curve = (u * PI).sin() * LEAF_CURVATURE * length * 0.1;
        for side in 0..=1u32 {
            let v = side as f32;
            let lateral = (v * 2.0 - 1.0) * half_w;
            let p = center + z_axis * lateral + y_axis * base_curve;
            buffers.positions.push(p.to_array());
            buffers.normals.push(y_axis.to_array());
            buffers.uvs.push([u, v]);
            buffers
                .colors
                .push(vertex_along_flow_color(qe_norm, tint_rgb, u, v));
        }
    }
    for i in 0..front_vtx {
        buffers.positions.push(buffers.positions[i]);
        buffers.normals.push((-y_axis).to_array());
        buffers.uvs.push(buffers.uvs[i]);
        buffers.colors.push(buffers.colors[i]);
    }

    for i in 0..sub {
        let row = i * 2;
        let next = (i + 1) * 2;
        let a = row;
        let b = row + 1;
        let c = next;
        let d = next + 1;
        buffers.indices.extend_from_slice(&[a, c, b, b, c, d]);
        let off = front_vtx as u32;
        buffers
            .indices
            .extend_from_slice(&[b + off, c + off, a + off, d + off, c + off, b + off]);
    }

    buffers.into_mesh()
}

/// Construye un abanico radial de pétalos en espiral áurea con curvatura cóncava y capas concéntricas.
///
/// Pétalos internos (índice bajo): más cerrados, cortos y saturados.
/// Pétalos externos (índice alto): más abiertos, largos y claros.
/// Cada pétalo tiene [`PETAL_CROSS_VERTS`] vértices transversales para curvatura 3D.
pub fn build_petal_fan(
    center: Vec3,
    up: Vec3,
    petal_count: u8,
    petal_length: f32,
    petal_width: f32,
    opening_angle: f32,
    subdivisions: u32,
    tint_rgb: [f32; 3],
    qe_norm: f32,
) -> Mesh {
    let petal_count = petal_count.clamp(1, MAX_PETAL_COUNT);
    if petal_length <= 0.0 || petal_width <= 0.0 {
        return empty_mesh();
    }
    let sub = subdivisions.clamp(PRIM_MIN_SUBDIVISIONS, PRIM_MAX_SUBDIVISIONS);
    let up_axis = normalize_or(up, Vec3::Y);
    let (x_axis, _, z_axis) = orthonormal_basis(up_axis, Vec3::X);
    let cv = PETAL_CROSS_VERTS;

    let per_petal_verts = ((sub + 1) * cv) as usize;
    let per_petal_quads = sub as usize * (cv as usize - 1);
    let mut buffers = MeshBuffers::with_capacity(
        per_petal_verts * petal_count as usize,
        petal_count as usize * per_petal_quads * 6,
    );

    let denom = petal_count.max(2) as f32;
    for p in 0..petal_count as u32 {
        let theta = p as f32 * GOLDEN_ANGLE;
        let ring_t = p as f32 / denom;

        let petal_open = opening_angle * (0.25 + 0.75 * ring_t);
        let petal_len = petal_length * (0.45 + 0.55 * ring_t);
        let petal_w = petal_width * (0.40 + 0.60 * ring_t);
        let base_lift = (1.0 - ring_t) * petal_length * 0.25;

        let radial = normalize_or(x_axis * theta.cos() + z_axis * theta.sin(), x_axis);
        let tangent = normalize_or(
            radial * petal_open.sin() + up_axis * petal_open.cos(),
            up_axis,
        );
        let width_axis = normalize_or(up_axis.cross(tangent), radial);
        let base_idx = buffers.positions.len() as u32;

        for i in 0..=sub {
            let u = i as f32 / sub as f32;
            let along = tangent * (u * petal_len);

            for j in 0..cv {
                let v = j as f32 / (cv - 1) as f32;
                let signed = v * 2.0 - 1.0;
                let lateral = width_axis * (signed * petal_w * 0.5);

                let cross_lift = (1.0 - signed * signed) * PETAL_CURVATURE * petal_len * 0.4
                    * (u * PI).sin();
                let curl = u * u * u * petal_len * (0.08 + 0.12 * ring_t);
                let edge_droop = signed.abs() * u * PETAL_CURVATURE * petal_len * 0.12;
                let lift = up_axis * (base_lift + cross_lift - edge_droop - curl);
                let pos = center + along + lateral + lift;

                buffers.positions.push(pos.to_array());
                buffers.normals.push(
                    normalize_or(tangent.cross(width_axis), up_axis).to_array(),
                );
                buffers.uvs.push([u, v]);

                let ring_shade = 0.70 + 0.30 * ring_t;
                let along_shade = 0.75 + 0.25 * u;
                let shade = ring_shade * along_shade;
                let tinted = [
                    (tint_rgb[0] * shade).clamp(0.0, 1.0),
                    (tint_rgb[1] * shade).clamp(0.0, 1.0),
                    (tint_rgb[2] * shade).clamp(0.0, 1.0),
                ];
                buffers.colors.push(vertex_along_flow_color(qe_norm, tinted, u, v));
            }
        }

        for i in 0..sub {
            for j in 0..cv - 1 {
                let row = base_idx + i * cv + j;
                let next = base_idx + (i + 1) * cv + j;
                let a = row;
                let b = row + 1;
                let c = next;
                let d = next + 1;
                buffers.indices.extend_from_slice(&[a, c, b, b, c, d]);
            }
        }
    }

    buffers.into_mesh()
}

/// Construye un ovoide por esfera UV deformada.
pub fn build_bulb(
    center: Vec3,
    up: Vec3,
    radius: f32,
    elongation: f32,
    rings: u32,
    sectors: u32,
    tint_rgb: [f32; 3],
    qe_norm: f32,
) -> Mesh {
    if radius <= 0.0 {
        return empty_mesh();
    }
    let ring_n = rings.clamp(BULB_MIN_RINGS, BULB_MAX_RINGS);
    let sec_n = sectors.clamp(BULB_MIN_SECTORS, BULB_MAX_SECTORS);
    let elong = elongation.max(0.1);
    let (x_axis, y_axis, z_axis) = orthonormal_basis(up, Vec3::X);

    let vert_count = ((ring_n + 1) * (sec_n + 1)) as usize;
    let mut buffers = MeshBuffers::with_capacity(vert_count, (ring_n * sec_n * 6) as usize);

    for i in 0..=ring_n {
        let v = i as f32 / ring_n as f32;
        let phi = v * PI;
        let sy = phi.cos();
        let sr = phi.sin();

        for j in 0..=sec_n {
            let u = j as f32 / sec_n as f32;
            let theta = u * TAU;
            let sx = theta.cos() * sr;
            let sz = theta.sin() * sr;
            let local = x_axis * sx + y_axis * (sy * elong) + z_axis * sz;
            let p = center + local * radius;
            buffers.positions.push(p.to_array());
            let n_local = x_axis * sx + y_axis * (sy / elong.max(1e-4)) + z_axis * sz;
            buffers.normals.push(normalize_or(n_local, y_axis).to_array());
            buffers.uvs.push([u, v]);
            let lat = (sy * 0.5 + 0.5).clamp(0.0, 1.0);
            buffers
                .colors
                .push(vertex_along_flow_color(qe_norm, tint_rgb, v, lat));
        }
    }

    let stride = sec_n + 1;
    for i in 0..ring_n {
        for j in 0..sec_n {
            let a = i * stride + j;
            let b = a + 1;
            let c = (i + 1) * stride + j;
            let d = c + 1;
            buffers.indices.extend_from_slice(&[a, c, b, b, c, d]);
        }
    }

    buffers.into_mesh()
}

/// Construye una primitiva en función del `OrganSpec`.
pub fn build_organ_primitive(spec: &OrganSpec, params: &OrganPrimitiveParams) -> Mesh {
    let scale = spec.scale_factor().max(0.1);
    let detail = params.detail.clamp(0.0, 1.0);
    let subdivisions = detail_to_steps(detail, PRIM_MIN_SUBDIVISIONS, PRIM_MAX_SUBDIVISIONS);
    let rings = detail_to_steps(detail, BULB_MIN_RINGS, BULB_MAX_RINGS);
    let sectors = detail_to_steps(detail, BULB_MIN_SECTORS, BULB_MAX_SECTORS);

    match spec.primitive() {
        GeometryPrimitive::Tube => {
            let dir = normalize_or(params.direction, Vec3::Y);
            let length = params.base_radius.max(0.01) * 2.2 * scale;
            let tube_max_segments = detail_to_steps(detail, 4, 12);
            let spine = vec![
                SpineNode {
                    position: params.origin,
                    tangent: dir,
                    tint_rgb: params.tint_rgb,
                    qe_norm: params.qe_norm.clamp(0.0, 1.0),
                },
                SpineNode {
                    position: params.origin + dir * length,
                    tangent: dir,
                    tint_rgb: params.tint_rgb,
                    qe_norm: params.qe_norm.clamp(0.0, 1.0),
                },
            ];
            let inf = GeometryInfluence {
                detail,
                energy_direction: dir,
                energy_strength: 1.0,
                resistance: 0.0,
                least_resistance_direction: dir,
                length_budget: length,
                max_segments: tube_max_segments,
                radius_base: params.base_radius.max(0.01) * 0.35 * scale,
                start_position: params.origin,
                qe_norm: params.qe_norm.clamp(0.0, 1.0),
                tint_rgb: params.tint_rgb,
                branch_role: BranchRole::Stem,
            };
            build_flow_mesh(&spine, &inf)
        }
        GeometryPrimitive::FlatSurface => build_flat_surface(
            params.origin,
            params.direction,
            params.tangent,
            params.base_radius.max(0.01) * 2.4 * scale,
            params.base_radius.max(0.01) * 1.4 * scale,
            subdivisions,
            params.tint_rgb,
            params.qe_norm,
        ),
        GeometryPrimitive::PetalFan => build_petal_fan(
            params.origin,
            params.direction,
            spec.count().clamp(1, MAX_PETAL_COUNT),
            params.base_radius.max(0.01) * 3.0 * scale,
            params.base_radius.max(0.01) * 1.6 * scale,
            PETAL_DEFAULT_OPENING,
            subdivisions,
            params.tint_rgb,
            params.qe_norm,
        ),
        GeometryPrimitive::Bulb => build_bulb(
            params.origin,
            params.direction,
            params.base_radius.max(0.01) * 0.9 * scale,
            BULB_DEFAULT_ELONGATION,
            rings,
            sectors,
            params.tint_rgb,
            params.qe_norm,
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn pos(mesh: &Mesh) -> &[[f32; 3]] {
        let Some(VertexAttributeValues::Float32x3(v)) = mesh.attribute(Mesh::ATTRIBUTE_POSITION) else {
            panic!("missing positions");
        };
        v.as_slice()
    }

    fn nrm(mesh: &Mesh) -> &[[f32; 3]] {
        let Some(VertexAttributeValues::Float32x3(v)) = mesh.attribute(Mesh::ATTRIBUTE_NORMAL) else {
            panic!("missing normals");
        };
        v.as_slice()
    }

    fn uv(mesh: &Mesh) -> &[[f32; 2]] {
        let Some(VertexAttributeValues::Float32x2(v)) = mesh.attribute(Mesh::ATTRIBUTE_UV_0) else {
            panic!("missing uv");
        };
        v.as_slice()
    }

    fn col(mesh: &Mesh) -> &[[f32; 4]] {
        let Some(VertexAttributeValues::Float32x4(v)) = mesh.attribute(Mesh::ATTRIBUTE_COLOR) else {
            panic!("missing color");
        };
        v.as_slice()
    }

    fn tri_count(mesh: &Mesh) -> usize {
        match mesh.indices() {
            Some(Indices::U32(id)) => id.len() / 3,
            Some(Indices::U16(id)) => id.len() / 3,
            None => 0,
        }
    }

    fn assert_mesh_coherent(mesh: &Mesh) {
        let p = pos(mesh);
        assert_eq!(p.len(), nrm(mesh).len());
        assert_eq!(p.len(), uv(mesh).len());
        assert_eq!(p.len(), col(mesh).len());
        let Some(Indices::U32(id)) = mesh.indices() else {
            panic!("indices missing");
        };
        let max_idx = p.len().saturating_sub(1) as u32;
        assert!(id.iter().all(|i| *i <= max_idx), "index out of bounds");
    }

    fn sample_params(detail: f32) -> OrganPrimitiveParams {
        OrganPrimitiveParams {
            origin: Vec3::ZERO,
            direction: Vec3::Y,
            tangent: Vec3::X,
            base_radius: 1.0,
            tint_rgb: [0.8, 0.3, 0.4],
            qe_norm: 0.7,
            detail,
        }
    }

    #[test]
    fn build_flat_surface_has_expected_attributes_with_subdivisions_two() {
        let mesh = build_flat_surface(
            Vec3::ZERO,
            Vec3::Y,
            Vec3::X,
            2.0,
            1.0,
            2,
            [0.2, 0.8, 0.3],
            0.5,
        );
        assert_mesh_coherent(&mesh);
        assert_eq!(pos(&mesh).len(), 12);
    }

    #[test]
    fn build_flat_surface_is_double_sided() {
        let subdivisions = 2;
        let mesh = build_flat_surface(
            Vec3::ZERO,
            Vec3::Y,
            Vec3::X,
            2.0,
            1.0,
            subdivisions,
            [0.2, 0.8, 0.3],
            0.5,
        );
        assert_eq!(tri_count(&mesh), (subdivisions * 2 * 2) as usize);
    }

    #[test]
    fn build_petal_fan_five_petals_respects_golden_angle_spacing() {
        let mesh = build_petal_fan(
            Vec3::ZERO,
            Vec3::Y,
            5,
            1.5,
            0.6,
            1.0,
            2,
            [0.9, 0.2, 0.4],
            0.8,
        );
        let p = pos(&mesh);
        let cv = PETAL_CROSS_VERTS as usize;
        let per_petal = (2 + 1) * cv;
        let mut centroids = Vec::new();
        for i in 0..5usize {
            let start = i * per_petal;
            let tip_start = start + per_petal - cv;
            let mut tip = Vec3::ZERO;
            for k in 0..cv {
                tip += Vec3::from_array(p[tip_start + k]);
            }
            tip /= cv as f32;
            centroids.push(tip);
        }
        for i in 1..centroids.len() {
            let a0 = centroids[i - 1].z.atan2(centroids[i - 1].x);
            let a1 = centroids[i].z.atan2(centroids[i].x);
            let mut diff = (a1 - a0).abs().rem_euclid(TAU);
            if diff > PI {
                diff = TAU - diff;
            }
            let expected = GOLDEN_ANGLE.min(TAU - GOLDEN_ANGLE);
            let delta = (diff - expected).abs();
            assert!(delta < 0.25, "petal spacing delta too large: {delta}");
        }
    }

    #[test]
    fn build_petal_fan_is_deterministic() {
        let a = build_petal_fan(
            Vec3::new(1.0, 2.0, -3.0),
            Vec3::Y,
            5,
            1.2,
            0.5,
            0.9,
            3,
            [0.8, 0.2, 0.3],
            0.4,
        );
        let b = build_petal_fan(
            Vec3::new(1.0, 2.0, -3.0),
            Vec3::Y,
            5,
            1.2,
            0.5,
            0.9,
            3,
            [0.8, 0.2, 0.3],
            0.4,
        );
        assert_eq!(pos(&a), pos(&b));
        assert_eq!(nrm(&a), nrm(&b));
        assert_eq!(uv(&a), uv(&b));
        assert_eq!(col(&a), col(&b));
        let Some(Indices::U32(ia)) = a.indices() else {
            panic!("indices a missing");
        };
        let Some(Indices::U32(ib)) = b.indices() else {
            panic!("indices b missing");
        };
        assert_eq!(ia, ib);
    }

    #[test]
    fn build_bulb_elongation_one_is_near_spherical() {
        let mesh = build_bulb(Vec3::ZERO, Vec3::Y, 2.0, 1.0, 6, 8, [0.4, 0.7, 0.2], 0.3);
        for p in pos(&mesh) {
            let d = Vec3::from_array(*p).length();
            assert!((d - 2.0).abs() < 0.05, "distance {d} is not near radius");
        }
    }

    #[test]
    fn build_bulb_elongation_two_is_taller_than_wide() {
        let mesh = build_bulb(Vec3::ZERO, Vec3::Y, 1.0, 2.0, 6, 8, [0.4, 0.7, 0.2], 0.3);
        let mut max_y = 0.0f32;
        let mut max_xz = 0.0f32;
        for p in pos(&mesh) {
            let v = Vec3::from_array(*p);
            max_y = max_y.max(v.y.abs());
            max_xz = max_xz.max((v.x * v.x + v.z * v.z).sqrt());
        }
        assert!(max_y > max_xz, "bulb should be taller than wide");
    }

    #[test]
    fn all_primitives_have_synchronized_attributes_and_valid_indices() {
        let flat = build_flat_surface(
            Vec3::ZERO,
            Vec3::Y,
            Vec3::X,
            1.5,
            0.6,
            3,
            [0.2, 0.9, 0.4],
            0.6,
        );
        let fan = build_petal_fan(
            Vec3::ZERO,
            Vec3::Y,
            4,
            1.0,
            0.4,
            1.0,
            3,
            [0.8, 0.2, 0.5],
            0.6,
        );
        let bulb = build_bulb(Vec3::ZERO, Vec3::Y, 1.0, 1.4, 4, 6, [0.4, 0.5, 0.9], 0.6);
        assert_mesh_coherent(&flat);
        assert_mesh_coherent(&fan);
        assert_mesh_coherent(&bulb);
    }

    #[test]
    fn triangle_count_is_monotonic_with_detail() {
        let low = sample_params(0.0);
        let high = sample_params(1.0);
        let spec_leaf = OrganSpec::new(crate::layers::organ::OrganRole::Leaf, 1, 1.0);
        let m_low = build_organ_primitive(&spec_leaf, &low);
        let m_high = build_organ_primitive(&spec_leaf, &high);
        assert!(tri_count(&m_high) >= tri_count(&m_low));
    }

    #[test]
    fn build_organ_primitive_dispatches_to_expected_shape_family() {
        let params = sample_params(0.8);
        let tube = build_organ_primitive(
            &OrganSpec::new(crate::layers::organ::OrganRole::Stem, 1, 1.0),
            &params,
        );
        let flat = build_organ_primitive(
            &OrganSpec::new(crate::layers::organ::OrganRole::Leaf, 1, 1.0),
            &params,
        );
        let fan = build_organ_primitive(
            &OrganSpec::new(crate::layers::organ::OrganRole::Petal, 5, 1.0),
            &params,
        );
        let bulb = build_organ_primitive(
            &OrganSpec::new(crate::layers::organ::OrganRole::Bud, 1, 1.0),
            &params,
        );

        assert!(tri_count(&tube) > 0);
        assert!(tri_count(&flat) > 0);
        assert!(tri_count(&fan) > 0);
        assert!(tri_count(&bulb) > 0);
        assert!(tri_count(&fan) > tri_count(&flat), "petal fan should be richer");
    }

    #[test]
    fn build_organ_primitive_petal_count_is_clamped() {
        let params = sample_params(1.0);
        let clamped = build_organ_primitive(
            &OrganSpec::new(crate::layers::organ::OrganRole::Petal, MAX_PETAL_COUNT, 1.0),
            &params,
        );
        let overflow = build_organ_primitive(
            &OrganSpec::new(crate::layers::organ::OrganRole::Petal, u8::MAX, 1.0),
            &params,
        );
        assert_eq!(tri_count(&clamped), tri_count(&overflow));
    }
}
