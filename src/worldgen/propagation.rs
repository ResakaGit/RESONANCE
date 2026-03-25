use bevy::math::Vec2;

use crate::blueprint::equations;
use crate::layers::MatterState;
use crate::worldgen::constants::{MIN_CONTRIBUTION_INTENSITY, MIN_DISTANCE_M};
use crate::worldgen::{FrequencyContribution, PropagationDecay};

/// Intensidad de un núcleo en una celda [qe/s].
///
/// `emission_rate_qe_s`: tasa de emisión del núcleo [qe/s].
/// `radius_m`: radio máximo de influencia [m].
pub fn nucleus_intensity_at(
    nucleus_pos: Vec2,
    cell_pos: Vec2,
    emission_rate_qe_s: f32,
    radius_m: f32,
    decay: PropagationDecay,
) -> f32 {
    if !nucleus_pos.is_finite()
        || !cell_pos.is_finite()
        || !emission_rate_qe_s.is_finite()
        || !radius_m.is_finite()
        || emission_rate_qe_s <= 0.0
        || radius_m <= 0.0
    {
        return 0.0;
    }

    let distance = nucleus_pos.distance(cell_pos);
    if distance > radius_m {
        return 0.0;
    }
    let d = distance.max(MIN_DISTANCE_M);

    match decay {
        PropagationDecay::Flat => emission_rate_qe_s,
        PropagationDecay::InverseLinear => emission_rate_qe_s / d,
        PropagationDecay::InverseSquare => emission_rate_qe_s / (d * d),
        PropagationDecay::Exponential { k } => {
            if !k.is_finite() {
                return 0.0;
            }
            let k = k.max(0.0);
            emission_rate_qe_s * (-k * d).exp()
        }
    }
}

/// Resuelve frecuencia dominante y pureza [0..1] desde contribuciones.
///
/// `intensity_qe` debe venir integrada a energía por tick (`qe`), no a tasa (`qe/s`).
pub fn resolve_dominant_frequency(contributions: &[FrequencyContribution]) -> (f32, f32) {
    let mut weighted_sum: f32 = 0.0;
    let mut total_intensity: f32 = 0.0;
    let mut max_intensity: f32 = 0.0;

    for c in contributions {
        if !c.frequency_hz().is_finite()
            || !c.intensity_qe().is_finite()
            || c.intensity_qe() < MIN_CONTRIBUTION_INTENSITY
        {
            continue;
        }
        total_intensity += c.intensity_qe();
        weighted_sum += c.frequency_hz() * c.intensity_qe();
        max_intensity = max_intensity.max(c.intensity_qe());
    }

    if total_intensity <= 0.0 {
        return (0.0, 0.0);
    }

    let dominant_hz = weighted_sum / total_intensity;
    let purity = (max_intensity / total_intensity).clamp(0.0, 1.0);
    (dominant_hz, purity)
}

/// Disipación por Segunda Ley. Nunca retorna energía negativa.
pub fn field_dissipation(accumulated_qe: f32, decay_rate_qe_s: f32, dt_s: f32) -> f32 {
    if !accumulated_qe.is_finite() || !decay_rate_qe_s.is_finite() || !dt_s.is_finite() {
        return 0.0;
    }
    let qe = accumulated_qe.max(0.0);
    let decay = decay_rate_qe_s.max(0.0);
    let dt = dt_s.max(0.0);
    (qe - decay * dt).max(0.0)
}

/// Volumen implícito de celda (modelo cúbico 3D).
pub fn cell_volume(cell_size_m: f32) -> f32 {
    if !cell_size_m.is_finite() {
        return 0.0;
    }
    let size = cell_size_m.max(0.0);
    size * size * size
}

/// Densidad de celda ρ = qe / volume.
pub fn cell_density(accumulated_qe: f32, cell_size_m: f32) -> f32 {
    if !accumulated_qe.is_finite() || !cell_size_m.is_finite() {
        return 0.0;
    }
    let volume = cell_volume(cell_size_m);
    if volume <= 0.0 {
        return 0.0;
    }
    accumulated_qe.max(0.0) / volume
}

/// Temperatura equivalente reutilizando la ecuación canónica.
pub fn cell_temperature(density: f32) -> f32 {
    if !density.is_finite() {
        return 0.0;
    }
    equations::equivalent_temperature(density.max(0.0))
}

/// Estado de materia reutilizando ecuación canónica.
pub fn cell_matter_state(temperature: f32, bond_energy: f32) -> MatterState {
    if !temperature.is_finite() || !bond_energy.is_finite() {
        return MatterState::Solid;
    }
    equations::state_from_temperature(temperature.max(0.0), bond_energy.max(0.0))
}

/// Transferencia neta de A hacia B por difusión lateral [qe].
///
/// Simétrica: `diffusion_transfer(a, b, c, dt) == -diffusion_transfer(b, a, c, dt)`.
pub fn diffusion_transfer(cell_qe: f32, neighbor_qe: f32, conductivity: f32, dt_s: f32) -> f32 {
    if !cell_qe.is_finite()
        || !neighbor_qe.is_finite()
        || !conductivity.is_finite()
        || !dt_s.is_finite()
    {
        return 0.0;
    }
    let k = conductivity.clamp(0.0, 1.0);
    let dt = dt_s.max(0.0);
    (cell_qe - neighbor_qe) * k * dt
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::Entity;

    fn approx_eq(a: f32, b: f32) {
        assert!((a - b).abs() < 1e-5, "left={a}, right={b}");
    }

    #[test]
    fn nucleus_intensity_at_supports_all_decay_variants() {
        let origin = Vec2::ZERO;
        let point = Vec2::new(2.0, 0.0);
        let emission = 100.0;
        let radius = 10.0;

        let flat = nucleus_intensity_at(origin, point, emission, radius, PropagationDecay::Flat);
        let inv_lin = nucleus_intensity_at(
            origin,
            point,
            emission,
            radius,
            PropagationDecay::InverseLinear,
        );
        let inv_sq = nucleus_intensity_at(
            origin,
            point,
            emission,
            radius,
            PropagationDecay::InverseSquare,
        );
        let exp = nucleus_intensity_at(
            origin,
            point,
            emission,
            radius,
            PropagationDecay::Exponential { k: 0.5 },
        );

        approx_eq(flat, 100.0);
        approx_eq(inv_lin, 50.0);
        approx_eq(inv_sq, 25.0);
        assert!(exp > 0.0 && exp < emission);
    }

    #[test]
    fn nucleus_intensity_at_is_zero_outside_radius() {
        let result = nucleus_intensity_at(
            Vec2::ZERO,
            Vec2::new(100.0, 0.0),
            50.0,
            5.0,
            PropagationDecay::Flat,
        );
        approx_eq(result, 0.0);
    }

    #[test]
    fn nucleus_intensity_at_handles_zero_distance_without_singularity() {
        let lin = nucleus_intensity_at(
            Vec2::ZERO,
            Vec2::ZERO,
            100.0,
            5.0,
            PropagationDecay::InverseLinear,
        );
        let sq = nucleus_intensity_at(
            Vec2::ZERO,
            Vec2::ZERO,
            100.0,
            5.0,
            PropagationDecay::InverseSquare,
        );
        assert!(lin.is_finite() && sq.is_finite());
        assert!(lin > 0.0 && sq > 0.0);
    }

    #[test]
    fn nucleus_intensity_at_invalid_inputs_return_zero() {
        approx_eq(
            nucleus_intensity_at(
                Vec2::ZERO,
                Vec2::new(1.0, 1.0),
                0.0,
                5.0,
                PropagationDecay::Flat,
            ),
            0.0,
        );
        approx_eq(
            nucleus_intensity_at(
                Vec2::ZERO,
                Vec2::new(1.0, 1.0),
                10.0,
                -1.0,
                PropagationDecay::Flat,
            ),
            0.0,
        );
    }

    #[test]
    fn resolve_dominant_frequency_single_contribution_has_purity_one() {
        let contributions = [FrequencyContribution::new(Entity::from_raw(1), 420.0, 10.0)];
        let (hz, purity) = resolve_dominant_frequency(&contributions);
        approx_eq(hz, 420.0);
        approx_eq(purity, 1.0);
    }

    #[test]
    fn resolve_dominant_frequency_equal_contributions_has_purity_half() {
        let contributions = [
            FrequencyContribution::new(Entity::from_raw(1), 100.0, 10.0),
            FrequencyContribution::new(Entity::from_raw(2), 300.0, 10.0),
        ];
        let (hz, purity) = resolve_dominant_frequency(&contributions);
        approx_eq(hz, 200.0);
        approx_eq(purity, 0.5);
    }

    #[test]
    fn resolve_dominant_frequency_empty_returns_zeroes() {
        let (hz, purity) = resolve_dominant_frequency(&[]);
        approx_eq(hz, 0.0);
        approx_eq(purity, 0.0);
    }

    #[test]
    fn resolve_dominant_frequency_skips_non_finite_values() {
        // Tras `new()`, intensidades no finitas quedan en 0 y se filtran por umbral mínimo.
        let contributions = [
            FrequencyContribution::new(Entity::from_raw(1), 100.0, f32::NAN),
            FrequencyContribution::new(Entity::from_raw(2), 200.0, f32::INFINITY),
        ];
        let (hz, purity) = resolve_dominant_frequency(&contributions);
        approx_eq(hz, 0.0);
        approx_eq(purity, 0.0);
    }

    #[test]
    fn field_dissipation_never_returns_negative() {
        let qe = field_dissipation(1.0, 10.0, 1.0);
        approx_eq(qe, 0.0);
    }

    #[test]
    fn diffusion_transfer_is_symmetric() {
        let ab = diffusion_transfer(20.0, 5.0, 0.1, 0.5);
        let ba = diffusion_transfer(5.0, 20.0, 0.1, 0.5);
        approx_eq(ab, -ba);
    }

    #[test]
    fn cell_volume_density_temperature_and_state_are_consistent() {
        approx_eq(cell_volume(2.0), 8.0);
        let density = cell_density(40.0, 2.0);
        approx_eq(density, 5.0);
        let temp = cell_temperature(density);
        let state = cell_matter_state(temp, 10.0);
        assert!(matches!(
            state,
            MatterState::Solid | MatterState::Liquid | MatterState::Gas | MatterState::Plasma
        ));
    }
}
