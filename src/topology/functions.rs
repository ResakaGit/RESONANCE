use crate::math_types::Vec2;

use crate::topology::config::ModulationParams;
use crate::topology::contracts::TerrainType;

fn aspect_direction(aspect_deg: f32) -> Vec2 {
    if !aspect_deg.is_finite() {
        return Vec2::X;
    }
    let radians = aspect_deg.to_radians();
    // Convención brújula: 0° = Norte(+Y), 90° = Este(+X).
    Vec2::new(radians.sin(), radians.cos()).normalize_or_zero()
}

/// Modula emisión efectiva según altitud (valles emiten más, crestas menos).
pub fn modulate_emission(base_rate: f32, altitude: f32) -> f32 {
    modulate_emission_with_params(base_rate, altitude, &ModulationParams::default())
}

/// Variante data-driven para runtime/hot-reload.
pub fn modulate_emission_with_params(
    base_rate: f32,
    altitude: f32,
    params: &ModulationParams,
) -> f32 {
    if !base_rate.is_finite() || !altitude.is_finite() {
        return 0.0;
    }
    let factor =
        (1.0 + (params.reference_altitude - altitude) * params.altitude_emission_scale).max(0.1);
    base_rate.max(0.0) * factor
}

/// Modula difusión por alineación con la pendiente local.
pub fn modulate_diffusion(base_diffusion: f32, slope: f32, direction: Vec2, aspect: f32) -> f32 {
    modulate_diffusion_with_params(
        base_diffusion,
        slope,
        direction,
        aspect,
        &ModulationParams::default(),
    )
}

/// Variante data-driven para runtime/hot-reload.
pub fn modulate_diffusion_with_params(
    base_diffusion: f32,
    slope: f32,
    direction: Vec2,
    aspect: f32,
    params: &ModulationParams,
) -> f32 {
    if !base_diffusion.is_finite() || !slope.is_finite() || !direction.is_finite() {
        return 0.0;
    }
    let slope = slope.max(0.0);
    let dir = direction.normalize_or_zero();
    if dir == Vec2::ZERO {
        return base_diffusion.max(0.0);
    }
    let aspect_dir = aspect_direction(aspect);
    let alignment = dir.dot(aspect_dir).clamp(-1.0, 1.0);
    let factor = (1.0 + alignment * slope * params.slope_diffusion_scale).max(0.1);
    base_diffusion.max(0.0) * factor
}

/// Modula disipación por exposición geomorfológica.
pub fn modulate_decay(base_decay: f32, terrain_type: TerrainType) -> f32 {
    modulate_decay_with_params(base_decay, terrain_type, &ModulationParams::default())
}

/// Variante data-driven para runtime/hot-reload.
pub fn modulate_decay_with_params(
    base_decay: f32,
    terrain_type: TerrainType,
    params: &ModulationParams,
) -> f32 {
    if !base_decay.is_finite() {
        return 0.0;
    }
    let multiplier = match terrain_type {
        TerrainType::Peak | TerrainType::Ridge | TerrainType::Cliff => params.decay_peak_factor,
        TerrainType::Valley | TerrainType::Basin => params.decay_valley_factor,
        TerrainType::Riverbed => params.decay_riverbed_factor,
        TerrainType::Slope | TerrainType::Plain | TerrainType::Plateau => 1.0,
    };
    base_decay.max(0.0) * multiplier
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::topology::config::ModulationParams;
    use crate::topology::constants::REFERENCE_ALTITUDE;

    fn approx_eq(a: f32, b: f32) {
        assert!((a - b).abs() < 1e-5, "left={a}, right={b}");
    }

    #[test]
    fn modulate_emission_reference_altitude_returns_base() {
        approx_eq(modulate_emission(120.0, REFERENCE_ALTITUDE), 120.0);
    }

    #[test]
    fn modulate_emission_valley_increases_rate() {
        assert!(modulate_emission(100.0, REFERENCE_ALTITUDE - 20.0) > 100.0);
    }

    #[test]
    fn modulate_emission_ridge_decreases_rate() {
        assert!(modulate_emission(100.0, REFERENCE_ALTITUDE + 20.0) < 100.0);
    }

    #[test]
    fn modulate_emission_is_clamped_to_ten_percent_floor() {
        let out = modulate_emission(100.0, REFERENCE_ALTITUDE + 10_000.0);
        assert!(out >= 10.0);
    }

    #[test]
    fn modulate_diffusion_zero_slope_keeps_base() {
        approx_eq(modulate_diffusion(0.2, 0.0, Vec2::X, 0.0), 0.2);
    }

    #[test]
    fn modulate_diffusion_aligned_with_slope_increases() {
        let out = modulate_diffusion(0.2, 20.0, Vec2::Y, 0.0);
        assert!(out > 0.2);
    }

    #[test]
    fn modulate_diffusion_against_slope_decreases() {
        let out = modulate_diffusion(0.2, 20.0, -Vec2::Y, 0.0);
        assert!(out < 0.2);
    }

    #[test]
    fn modulate_diffusion_non_finite_aspect_falls_back_without_zeroing() {
        let out = modulate_diffusion(0.2, 10.0, Vec2::Y, f32::NAN);
        assert!(out.is_finite());
        assert!(out > 0.0);
    }

    #[test]
    fn aspect_cardinal_mapping_matches_compass_convention() {
        assert!(modulate_diffusion(0.2, 10.0, Vec2::Y, 0.0) > 0.2);
        assert!(modulate_diffusion(0.2, 10.0, Vec2::X, 90.0) > 0.2);
        assert!(modulate_diffusion(0.2, 10.0, -Vec2::Y, 180.0) > 0.2);
        assert!(modulate_diffusion(0.2, 10.0, -Vec2::X, 270.0) > 0.2);
    }

    #[test]
    fn modulate_decay_plain_is_neutral() {
        approx_eq(modulate_decay(0.1, TerrainType::Plain), 0.1);
    }

    #[test]
    fn modulate_decay_peak_is_15x() {
        approx_eq(modulate_decay(0.1, TerrainType::Peak), 0.15);
    }

    #[test]
    fn modulate_decay_valley_is_07x() {
        approx_eq(modulate_decay(0.1, TerrainType::Valley), 0.07);
    }

    #[test]
    fn modulate_decay_riverbed_is_08x() {
        approx_eq(modulate_decay(0.1, TerrainType::Riverbed), 0.08);
    }

    #[test]
    fn modulate_emission_uses_runtime_params() {
        let params = ModulationParams {
            altitude_emission_scale: 0.01,
            ..ModulationParams::default()
        };
        let boosted =
            modulate_emission_with_params(100.0, params.reference_altitude - 10.0, &params);
        assert!(boosted > 100.0);
    }
}
