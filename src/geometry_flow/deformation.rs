//! GF2B — Deformación termodinámica del spine (stateless).
//!
//! Opera sobre el [`SpineNode`] de GF1, sin ECS.

use crate::blueprint::equations::deformation_delta;
use crate::geometry_flow::SpineNode;

/// Payload completo para deformar un spine.
pub struct DeformationPayload {
    pub base_spine: Vec<SpineNode>,
    pub t_energy: bevy::math::Vec3,
    pub t_gravity: bevy::math::Vec3,
    pub bond_energy: f32,
    pub gravity_scale: f32,
}

/// Deforma el spine segmento a segmento con curvatura cuadrática.
///
/// - Nodo base (index 0): weight = 0 → sin desplazamiento.
/// - Punta (index n-1): weight = 1 → máximo desplazamiento.
/// - Cada nodo: `new_pos = base_pos + delta * weight²`
pub fn deform_spine(payload: &DeformationPayload) -> Vec<SpineNode> {
    let spine = &payload.base_spine;
    let n = spine.len();
    if n == 0 {
        return Vec::new();
    }
    if n == 1 {
        return spine.clone();
    }

    let denom = (n - 1) as f32;
    spine
        .iter()
        .enumerate()
        .map(|(i, node)| {
            let weight = i as f32 / denom;
            let weight_sq = weight * weight;
            let delta = deformation_delta(
                node.tangent,
                payload.t_energy,
                payload.t_gravity,
                payload.bond_energy,
            );
            SpineNode {
                position: node.position + delta * weight_sq,
                tangent: node.tangent,
                tint_rgb: node.tint_rgb,
                qe_norm: node.qe_norm,
            }
        })
        .collect()
}

/// Aplica el spine deformado a un array de posiciones de vértices.
///
/// Cada vértice se interpola linealmente en el eje del spine (índice normalizado).
/// Los vértices se mueven según el desplazamiento del nodo de spine más próximo.
pub fn apply_spine_to_mesh(
    base_positions: &[[f32; 3]],
    deformed_spine: &[SpineNode],
) -> Vec<[f32; 3]> {
    let spine_n = deformed_spine.len();
    if spine_n == 0 || base_positions.is_empty() {
        return base_positions.to_vec();
    }

    let denom = (spine_n - 1).max(1) as f32;
    base_positions
        .iter()
        .enumerate()
        .map(|(vi, &pos)| {
            // Índice normalizado a lo largo del spine basado en la posición del vértice en el array.
            let t = (vi as f32 / base_positions.len().max(1) as f32).clamp(0.0, 1.0);
            let spine_f = t * denom;
            let lo = (spine_f.floor() as usize).min(spine_n - 1);
            let hi = (spine_f.ceil() as usize).min(spine_n - 1);
            let frac = spine_f - spine_f.floor();

            let lo_node = &deformed_spine[lo];
            let hi_node = &deformed_spine[hi];

            // Desplazamiento interpolado respecto a posición base del spine.
            // La posición de vértice se desplaza proporcionalmente al nodo de spine.
            let disp_lo = lo_node.position - deformed_spine
                .first()
                .map(|n| n.position)
                .unwrap_or(lo_node.position);
            let disp_hi = hi_node.position - deformed_spine
                .first()
                .map(|n| n.position)
                .unwrap_or(hi_node.position);
            let disp = disp_lo.lerp(disp_hi, frac);

            [pos[0] + disp.x, pos[1] + disp.y, pos[2] + disp.z]
        })
        .collect()
}

/// Fingerprint determinista del payload para cache key.
///
/// Usa hash FNV-1a sobre los campos numéricos del payload.
pub fn deformation_fingerprint(payload: &DeformationPayload) -> u64 {
    const FNV_OFFSET: u64 = 14_695_981_039_346_656_037;
    const FNV_PRIME: u64 = 1_099_511_628_211;

    let mut hash = FNV_OFFSET;
    let mut mix = |bytes: &[u8]| {
        for &b in bytes {
            hash ^= b as u64;
            hash = hash.wrapping_mul(FNV_PRIME);
        }
    };

    mix(&(payload.bond_energy.to_bits()).to_le_bytes());
    mix(&(payload.gravity_scale.to_bits()).to_le_bytes());
    mix(&(payload.t_energy.x.to_bits()).to_le_bytes());
    mix(&(payload.t_energy.y.to_bits()).to_le_bytes());
    mix(&(payload.t_energy.z.to_bits()).to_le_bytes());
    mix(&(payload.t_gravity.x.to_bits()).to_le_bytes());
    mix(&(payload.t_gravity.y.to_bits()).to_le_bytes());
    mix(&(payload.t_gravity.z.to_bits()).to_le_bytes());

    for node in &payload.base_spine {
        mix(&(node.position.x.to_bits()).to_le_bytes());
        mix(&(node.position.y.to_bits()).to_le_bytes());
        mix(&(node.position.z.to_bits()).to_le_bytes());
        mix(&(node.qe_norm.to_bits()).to_le_bytes());
    }

    hash
}

#[cfg(test)]
mod tests {
    use bevy::math::Vec3;

    use super::*;
    use crate::geometry_flow::SpineNode;

    fn make_spine(n: usize) -> Vec<SpineNode> {
        (0..n)
            .map(|i| SpineNode {
                position: Vec3::new(0.0, i as f32, 0.0),
                tangent: Vec3::Y,
                tint_rgb: [0.5, 0.5, 0.5],
                qe_norm: 0.5,
            })
            .collect()
    }

    fn make_payload(
        spine: Vec<SpineNode>,
        gravity_scale: f32,
        t_energy: Vec3,
        t_gravity: Vec3,
        bond: f32,
    ) -> DeformationPayload {
        DeformationPayload {
            base_spine: spine,
            t_energy,
            t_gravity,
            bond_energy: bond,
            gravity_scale,
        }
    }

    #[test]
    fn deform_spine_gravity_zero_no_change_when_no_energy() {
        let spine = make_spine(5);
        let base_positions: Vec<Vec3> = spine.iter().map(|n| n.position).collect();
        let payload = make_payload(spine, 0.0, Vec3::ZERO, Vec3::ZERO, 0.5);
        let deformed = deform_spine(&payload);
        for (d, b) in deformed.iter().zip(base_positions.iter()) {
            assert!(
                (d.position - *b).length() < 1e-6,
                "no forces → no change: {:?} vs {:?}",
                d.position,
                b
            );
        }
    }

    #[test]
    fn deform_spine_output_length_equals_input_length() {
        let spine = make_spine(7);
        let payload = make_payload(spine, 0.0, Vec3::X, Vec3::ZERO, 0.2);
        let deformed = deform_spine(&payload);
        assert_eq!(deformed.len(), 7);
    }

    #[test]
    fn deform_spine_same_payload_is_deterministic() {
        let spine_a = make_spine(5);
        let spine_b = make_spine(5);
        let p_a = make_payload(spine_a, 9.8, Vec3::X * 2.0, Vec3::NEG_Y, 0.3);
        let p_b = make_payload(spine_b, 9.8, Vec3::X * 2.0, Vec3::NEG_Y, 0.3);
        let d_a = deform_spine(&p_a);
        let d_b = deform_spine(&p_b);
        for (a, b) in d_a.iter().zip(d_b.iter()) {
            assert_eq!(a.position, b.position, "deformation must be deterministic");
        }
    }

    #[test]
    fn deform_spine_base_node_has_zero_displacement() {
        let spine = make_spine(4);
        let base_origin = spine[0].position;
        let payload = make_payload(spine, 9.8, Vec3::X * 3.0, Vec3::NEG_Y, 0.0);
        let deformed = deform_spine(&payload);
        assert!(
            (deformed[0].position - base_origin).length() < 1e-6,
            "node 0 (weight=0) must not be displaced"
        );
    }

    #[test]
    fn deform_spine_tip_has_larger_displacement_than_base() {
        let spine = make_spine(5);
        let base: Vec<Vec3> = spine.iter().map(|n| n.position).collect();
        let payload = make_payload(spine, 0.0, Vec3::X * 5.0, Vec3::ZERO, 0.0);
        let deformed = deform_spine(&payload);
        let disp_base = (deformed[0].position - base[0]).length();
        let disp_tip = (deformed[4].position - base[4]).length();
        assert!(
            disp_tip > disp_base,
            "tip displacement ({disp_tip}) must exceed base ({disp_base})"
        );
    }

    #[test]
    fn deformation_fingerprint_same_payload_same_hash() {
        let p_a = make_payload(make_spine(4), 9.8, Vec3::X, Vec3::NEG_Y, 0.4);
        let p_b = make_payload(make_spine(4), 9.8, Vec3::X, Vec3::NEG_Y, 0.4);
        assert_eq!(
            deformation_fingerprint(&p_a),
            deformation_fingerprint(&p_b)
        );
    }

    #[test]
    fn deformation_fingerprint_different_payloads_differ() {
        let p_a = make_payload(make_spine(4), 9.8, Vec3::X, Vec3::NEG_Y, 0.4);
        let p_b = make_payload(make_spine(4), 9.8, Vec3::Y, Vec3::NEG_Y, 0.4);
        assert_ne!(
            deformation_fingerprint(&p_a),
            deformation_fingerprint(&p_b)
        );
    }
}
