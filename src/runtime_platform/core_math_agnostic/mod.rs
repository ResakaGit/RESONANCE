use bevy::prelude::{Vec2, Vec3};

/// Normaliza la política canónica de cero:
/// evita `-0.0` para mantener snapshots deterministas y comparables.
#[inline]
pub fn canonical_zero(value: f32) -> f32 {
    if value == 0.0 { 0.0 } else { value }
}

/// Aplica cero canónico componente a componente.
#[inline]
pub fn canonical_vec2(v: Vec2) -> Vec2 {
    Vec2::new(canonical_zero(v.x), canonical_zero(v.y))
}

/// Clampa un escalar al rango unitario [0, 1].
#[inline]
pub fn clamp_unit(value: f32) -> f32 {
    value.clamp(0.0, 1.0)
}

/// Normaliza un vector 2D si su magnitud es válida; si no, devuelve cero.
#[inline]
pub fn normalize_or_zero(v: Vec2) -> Vec2 {
    if v.length_squared() > 0.0 {
        canonical_vec2(v.normalize())
    } else {
        Vec2::ZERO
    }
}

/// Altura Y del “suelo” sim en modo 3D (alineada al cubo `V6GroundPlane` / demo).
pub const DEFAULT_SIM_STANDING_Y: f32 = 4.0;

/// Posición 2D del plano de simulación desde `Transform` según layout del mundo.
#[inline]
pub fn sim_plane_pos(translation: Vec3, use_xz_ground: bool) -> Vec2 {
    if use_xz_ground {
        flatten_xz(translation)
    } else {
        translation.truncate()
    }
}

/// Convierte vector 2D (XY) a 3D sobre plano XZ.
/// Mapeo: x -> x, y -> z, y(altura) = 0.
#[inline]
pub fn vec2_to_xz(v: Vec2) -> Vec3 {
    Vec3::new(canonical_zero(v.x), 0.0, canonical_zero(v.y))
}

/// Construye un vector 3D desde plano XZ + altura explícita.
#[inline]
pub fn xz_to_vec3(xz: Vec2, y: f32) -> Vec3 {
    Vec3::new(
        canonical_zero(xz.x),
        canonical_zero(y),
        canonical_zero(xz.y),
    )
}

/// Aplana un vector 3D al plano XZ.
#[inline]
pub fn flatten_xz(v: Vec3) -> Vec2 {
    canonical_vec2(Vec2::new(v.x, v.z))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn canonical_zero_negative_zero_returns_positive_zero() {
        let value = canonical_zero(-0.0);
        assert_eq!(value, 0.0);
        assert!(!value.is_sign_negative());
    }

    #[test]
    fn normalize_or_zero_zero_vector_returns_zero() {
        let result = normalize_or_zero(Vec2::ZERO);
        assert_eq!(result, Vec2::ZERO);
    }

    #[test]
    fn normalize_or_zero_diagonal_vector_is_unit() {
        let result = normalize_or_zero(Vec2::new(3.0, 4.0));
        assert!((result.length() - 1.0).abs() < 1e-6);
    }

    #[test]
    fn vec2_to_xz_and_flatten_roundtrip_keeps_coordinates() {
        let original = Vec2::new(1.25, -4.5);
        let world = vec2_to_xz(original);
        let roundtrip = flatten_xz(world);
        assert_eq!(roundtrip, original);
    }

    #[test]
    fn clamp_unit_limits_lower_and_upper_bounds() {
        assert_eq!(clamp_unit(-2.0), 0.0);
        assert_eq!(clamp_unit(2.0), 1.0);
        assert_eq!(clamp_unit(0.4), 0.4);
    }
}
