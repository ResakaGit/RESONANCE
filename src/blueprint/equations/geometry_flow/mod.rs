use bevy::math::Vec3;

// ═══════════════════════════════════════════════
// Geometry Flow — spine flora / tendencia vs resistencia (stateless)
// ═══════════════════════════════════════════════

/// Cuánto mezclar `least_resistance_dir` al **romper** la tendencia recta (GF1).
pub const FLOW_BREAK_STEER_BLEND: f32 = 0.35;

/// Proyección no negativa del vector energía sobre la tangente (empuje a lo largo del eje).
#[inline]
pub fn flow_push_along_tangent(energy_vector: Vec3, tangent_unit: Vec3) -> f32 {
    if !energy_vector.is_finite() || !tangent_unit.is_finite() {
        return 0.0;
    }
    let t = tangent_unit.normalize_or_zero();
    if t.length_squared() < 1e-12 {
        return 0.0;
    }
    energy_vector.dot(t).max(0.0)
}

/// `true` si el tramo **mantiene** la tangente actual; si `false`, se aplica steering.
#[inline]
pub fn flow_maintain_straight_segment(push_along: f32, resistance: f32) -> bool {
    if !push_along.is_finite() || !resistance.is_finite() {
        return true;
    }
    push_along >= resistance
}

/// Mezcla la tangente actual con la dirección de menor resistencia (`blend` en \([0,1]\)).
#[inline]
pub fn flow_steered_tangent(tangent_current: Vec3, least_resistance_dir: Vec3, blend: f32) -> Vec3 {
    let t = tangent_current.normalize_or_zero();
    let r = least_resistance_dir.normalize_or_zero();
    let b = blend.clamp(0.0, 1.0);
    if t.length_squared() < 1e-12 && r.length_squared() < 1e-12 {
        return Vec3::Y;
    }
    if t.length_squared() < 1e-12 {
        return r;
    }
    if r.length_squared() < 1e-12 {
        return t;
    }
    (t * (1.0 - b) + r * b).normalize_or_zero()
}
