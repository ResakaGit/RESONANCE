//! Bridge S2 → S3 — planeta ecológico usando `worldgen::MapConfig` existente.
//! Bridge S2 → S3 — planet-as-ecology using existing `worldgen::MapConfig`.
//!
//! CT-5 / ADR-036 §D4. Wrapper delgado: el worldgen existente hace todo.
//! Solo traduce `PlanetSpec` → `MapConfig` válido + central nucleus = la estrella madre.

use crate::blueprint::domain_enums::MatterState;
use crate::cosmic::bridges::stellar_to_planetary::PlanetSpec;
use crate::worldgen::map_config::{AmbientPressureConfig, MapConfig, NucleusConfig};
use crate::worldgen::nucleus::PropagationDecay;

/// Tamaño de grid por default para un planeta (cells). Calibrado para una sola
/// escena ecológica típica. Sobrescribible por el caller.
/// Default grid size per planet. Overridable by caller.
pub const DEFAULT_GRID_CELLS: u32 = 32;
pub const DEFAULT_CELL_SIZE: f32 = 0.5;

/// Convierte un `PlanetSpec` en `MapConfig` válido (pasa `validate_map_config`).
/// Converts a `PlanetSpec` into a valid `MapConfig`.
pub fn planet_to_map_config(spec: &PlanetSpec, seed: u64) -> MapConfig {
    let width_cells = DEFAULT_GRID_CELLS;
    let height_cells = DEFAULT_GRID_CELLS;
    let cell_size = DEFAULT_CELL_SIZE;
    let world_half = width_cells as f32 * cell_size * 0.5;

    let nucleus = build_planet_nucleus(spec, world_half);

    MapConfig {
        width_cells,
        height_cells,
        cell_size,
        origin: [-world_half, -world_half],
        warmup_ticks: None,
        seed: Some(seed),
        nuclei: vec![nucleus],
        fog_of_war: false,
        seasons: Vec::new(),
        playfield_margin_cells: None,
        initial_field_qe: Some((spec.qe as f32).max(0.0)),
        initial_field_freq: Some(spec.frequency_hz as f32),
        initial_nutrient_water: Some(nutrient_water_from_state(spec.matter_state)),
        day_period_ticks: None,
        self_sustaining_qe: None,
        emission_scale: None,
        year_period_ticks: None,
        axial_tilt: None,
        solar_emission_qe_s: None,
    }
}

/// Núcleo central representando al propio planeta (estado dominante + freq).
/// Central nucleus representing the planet itself (dominant state + freq).
fn build_planet_nucleus(spec: &PlanetSpec, world_half: f32) -> NucleusConfig {
    let propagation_radius = world_half * 0.6;
    let emission = (spec.qe as f32).max(1.0) * emission_scale_for_state(spec.matter_state);
    let frequency_hz = (spec.frequency_hz as f32).max(1.0);
    NucleusConfig {
        name: format!("planet_r{:.2}", spec.orbital_radius),
        position: [0.0, 0.0],
        frequency_hz,
        emission_rate_qe_s: emission,
        propagation_radius,
        decay: PropagationDecay::InverseSquare,
        ambient_pressure: ambient_pressure_for_state(spec.matter_state),
        reservoir: Some((spec.qe as f32).max(1.0)),
    }
}

/// Saturación de agua inicial según estado — derivable de axiomas.
/// Initial water saturation per matter state.
fn nutrient_water_from_state(state: MatterState) -> f32 {
    match state {
        MatterState::Solid => 0.1,
        MatterState::Liquid => 0.7,
        MatterState::Gas => 0.3,
        MatterState::Plasma => 0.0,
    }
}

/// Escala de emisión por estado: plasma emite más, sólido menos.
/// Emission scale per state.
fn emission_scale_for_state(state: MatterState) -> f32 {
    match state {
        MatterState::Solid => 0.5,
        MatterState::Liquid => 1.0,
        MatterState::Gas => 1.5,
        MatterState::Plasma => 2.0,
    }
}

/// Presión ambiente opcional: solo estados con efectos marcados (ocean/fire).
/// Optional ambient pressure for notable matter states.
fn ambient_pressure_for_state(state: MatterState) -> Option<AmbientPressureConfig> {
    match state {
        MatterState::Liquid => Some(AmbientPressureConfig { delta_qe: 1.0, viscosity: 2.5 }),
        MatterState::Gas => Some(AmbientPressureConfig { delta_qe: -0.5, viscosity: 0.5 }),
        MatterState::Plasma => Some(AmbientPressureConfig { delta_qe: -2.0, viscosity: 0.1 }),
        MatterState::Solid => None,
    }
}

// ─── Tests ──────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::worldgen::map_config::validate_map_config;

    fn sample_planet(state: MatterState) -> PlanetSpec {
        PlanetSpec {
            qe: 100.0,
            frequency_hz: 85.0,
            orbital_radius: 3.5,
            temperature: 0.015,
            matter_state: state,
        }
    }

    #[test]
    fn map_config_is_valid_for_liquid_planet() {
        let cfg = planet_to_map_config(&sample_planet(MatterState::Liquid), 42);
        validate_map_config(&cfg).expect("MapConfig should validate");
    }

    #[test]
    fn map_config_is_valid_for_all_states() {
        for state in [MatterState::Solid, MatterState::Liquid, MatterState::Gas, MatterState::Plasma] {
            let cfg = planet_to_map_config(&sample_planet(state), 7);
            validate_map_config(&cfg).unwrap_or_else(|e| panic!("invalid for {state:?}: {e:?}"));
        }
    }

    #[test]
    fn map_config_seed_propagates() {
        let cfg = planet_to_map_config(&sample_planet(MatterState::Liquid), 1234);
        assert_eq!(cfg.seed, Some(1234));
    }

    #[test]
    fn liquid_planet_has_ambient_pressure() {
        let cfg = planet_to_map_config(&sample_planet(MatterState::Liquid), 1);
        assert!(cfg.nuclei[0].ambient_pressure.is_some());
    }

    #[test]
    fn solid_planet_has_no_ambient_pressure() {
        let cfg = planet_to_map_config(&sample_planet(MatterState::Solid), 1);
        assert!(cfg.nuclei[0].ambient_pressure.is_none());
    }

    #[test]
    fn water_saturation_highest_for_liquid() {
        assert!(nutrient_water_from_state(MatterState::Liquid)
            > nutrient_water_from_state(MatterState::Solid));
        assert!(nutrient_water_from_state(MatterState::Liquid)
            > nutrient_water_from_state(MatterState::Gas));
        assert_eq!(nutrient_water_from_state(MatterState::Plasma), 0.0);
    }
}
