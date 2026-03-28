use std::env;
use std::fs;
use std::path::PathBuf;

use crate::math_types::Vec2;
use bevy::prelude::Resource;
use serde::{Deserialize, Serialize};

use crate::worldgen::constants::{DEFAULT_MAP_NAME, MAPS_DIR};
use crate::worldgen::{EnergyNucleus, PropagationDecay};

/// Nombre lógico del mapa cargado (slug de `RESONANCE_MAP` o `default`).
/// Útil para dispatch de demos sin releer env en plugins tardíos.
#[derive(Debug, Clone, Resource, PartialEq, Eq)]
pub struct ActiveMapName(pub String);

#[inline]
pub fn active_map_slug_from_env() -> String {
    env::var("RESONANCE_MAP")
        .ok()
        .filter(|v| !v.trim().is_empty())
        .unwrap_or_else(|| DEFAULT_MAP_NAME.to_string())
}

/// `RESONANCE_MAP` / [`ActiveMapName`]: demo planeta esférico + atmósfera + rosa (`world/demos/round_world_rosa`).
pub const ROUND_WORLD_ROSA_MAP_SLUG: &str = "round_world_rosa";

#[derive(Debug, Clone, Resource, Serialize, Deserialize)]
pub struct MapConfig {
    pub width_cells: u32,
    pub height_cells: u32,
    pub cell_size: f32,
    pub origin: [f32; 2],
    pub warmup_ticks: Option<u32>,
    pub seed: Option<u64>,
    pub nuclei: Vec<NucleusConfig>,
    /// Si es `false`, no se crea [`crate::world::FogOfWarGrid`] ni el overlay 3D (demos / sandbox).
    #[serde(default = "default_fog_of_war_enabled")]
    pub fog_of_war: bool,
    #[serde(default)]
    pub seasons: Vec<SeasonPreset>,
    /// Celdas de borde del grid con contexto eco Void (barrera lógica); clamp del jugador al interior.
    #[serde(default)]
    pub playfield_margin_cells: Option<u32>,
    /// Big Bang mode: initial qe seeded uniformly in all cells (no nuclei required).
    /// When set, field is pre-seeded with this qe + initial_field_freq before warmup.
    #[serde(default)]
    pub initial_field_qe: Option<f32>,
    /// Frequency of the initial uniform field (Hz). Default: 85.0 (Terra band).
    #[serde(default)]
    pub initial_field_freq: Option<f32>,
    /// Initial nutrient water saturation [0,1] for Big Bang bootstrap.
    #[serde(default)]
    pub initial_nutrient_water: Option<f32>,
    /// Day/night cycle period in ticks. When set, solar energy rotates.
    #[serde(default)]
    pub day_period_ticks: Option<f32>,
    /// Cosmological anchor override. None = default (20.0).
    #[serde(default)]
    pub self_sustaining_qe: Option<f32>,
    /// Emission scaling for all nuclei. None = no scaling (1.0).
    /// Use grid_area / reference_area for proportional scaling on larger grids.
    #[serde(default)]
    pub emission_scale: Option<f32>,
    /// Orbital year period in ticks. Enables seasonal irradiance modulation.
    #[serde(default)]
    pub year_period_ticks: Option<f32>,
    /// Axial tilt [0, 1]. 0 = no seasons, 0.4 ≈ Earth-like (23.5°/90° ≈ 0.26).
    #[serde(default)]
    pub axial_tilt: Option<f32>,
}

fn default_fog_of_war_enabled() -> bool {
    true
}

impl Default for MapConfig {
    fn default() -> Self {
        Self {
            width_cells: 64,
            height_cells: 64,
            cell_size: 2.0,
            origin: [-64.0, -64.0],
            warmup_ticks: None,
            seed: None,
            fog_of_war: true,
            nuclei: vec![
                NucleusConfig {
                    name: "terra_nucleus".to_string(),
                    position: [-12.0, -8.0],
                    frequency_hz: 75.0,
                    emission_rate_qe_s: 120.0,
                    propagation_radius: 20.0,
                    decay: PropagationDecay::InverseSquare,
                    ambient_pressure: None,
                    reservoir: None,
                },
                NucleusConfig {
                    name: "ignis_nucleus".to_string(),
                    position: [16.0, 12.0],
                    frequency_hz: 450.0,
                    emission_rate_qe_s: 140.0,
                    propagation_radius: 18.0,
                    decay: PropagationDecay::InverseLinear,
                    ambient_pressure: None,
                    reservoir: None,
                },
            ],
            seasons: Vec::new(),
            playfield_margin_cells: None,
            initial_field_qe: None,
            initial_field_freq: None,
            initial_nutrient_water: None,
            day_period_ticks: None,
            self_sustaining_qe: None,
            emission_scale: None,
            year_period_ticks: None,
            axial_tilt: None,
        }
    }
}

impl MapConfig {
    pub fn origin_vec2(&self) -> Vec2 {
        Vec2::new(self.origin[0], self.origin[1])
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NucleusConfig {
    pub name: String,
    pub position: [f32; 2],
    pub frequency_hz: f32,
    pub emission_rate_qe_s: f32,
    pub propagation_radius: f32,
    pub decay: PropagationDecay,
    pub ambient_pressure: Option<AmbientPressureConfig>,
    /// Per-nucleus fuel reservoir (qe). None = infinite emission (no NucleusReservoir).
    #[serde(default)]
    pub reservoir: Option<f32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AmbientPressureConfig {
    pub delta_qe: f32,
    pub viscosity: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SeasonPreset {
    pub name: String,
    #[serde(default)]
    pub nucleus_deltas: Vec<NucleusDelta>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NucleusDelta {
    pub nucleus_name: String,
    pub frequency_hz_delta: Option<f32>,
    pub emission_rate_delta: Option<f32>,
    pub propagation_radius_delta: Option<f32>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ValidationError {
    EmptyNuclei,
    InvalidWorldSize,
    InvalidCellSize,
    DuplicateNucleusName(String),
    NucleusOutOfBounds { name: String },
    NonPositiveFrequency { name: String },
    NegativeEmissionRate { name: String },
    NonPositiveRadius { name: String },
    InvalidDecayK { name: String },
    InvalidAmbientPressure { name: String },
    PlayfieldMarginTooLarge,
}

pub fn selected_map_path_from_env() -> PathBuf {
    let map_name = env::var("RESONANCE_MAP")
        .ok()
        .filter(|v| !v.trim().is_empty())
        .unwrap_or_else(|| DEFAULT_MAP_NAME.to_string());
    PathBuf::from(format!("{MAPS_DIR}/{map_name}.ron"))
}

pub fn parse_map_config(contents: &str) -> Result<MapConfig, String> {
    let options =
        ron::Options::default().with_default_extension(ron::extensions::Extensions::IMPLICIT_SOME);
    options
        .from_str(contents)
        .or_else(|_| serde_json::from_str(contents))
        .map_err(|err| format!("MapConfig parse error (RON/JSON): {err}"))
}

pub fn validate_map_config(config: &MapConfig) -> Result<(), Vec<ValidationError>> {
    let mut errors = Vec::new();
    if config.width_cells == 0 || config.height_cells == 0 {
        errors.push(ValidationError::InvalidWorldSize);
    }
    if !config.cell_size.is_finite() || config.cell_size <= 0.0 {
        errors.push(ValidationError::InvalidCellSize);
    }
    // Empty nuclei allowed in Big Bang mode (initial_field_qe seeds the field instead).
    if config.nuclei.is_empty() && config.initial_field_qe.is_none() {
        errors.push(ValidationError::EmptyNuclei);
    }

    let margin = config.playfield_margin_cells.unwrap_or(0);
    let min_dim = config.width_cells.min(config.height_cells);
    if margin > 0 && 2 * margin >= min_dim {
        errors.push(ValidationError::PlayfieldMarginTooLarge);
    }

    let world_width = config.width_cells as f32 * config.cell_size;
    let world_height = config.height_cells as f32 * config.cell_size;
    let min_x = config.origin[0];
    let min_y = config.origin[1];
    let max_x = min_x + world_width;
    let max_y = min_y + world_height;

    let mut names = std::collections::HashSet::new();
    for nucleus in &config.nuclei {
        if !names.insert(nucleus.name.clone()) {
            errors.push(ValidationError::DuplicateNucleusName(nucleus.name.clone()));
        }
        if !nucleus.frequency_hz.is_finite() || nucleus.frequency_hz <= 0.0 {
            errors.push(ValidationError::NonPositiveFrequency {
                name: nucleus.name.clone(),
            });
        }
        if !nucleus.emission_rate_qe_s.is_finite() || nucleus.emission_rate_qe_s < 0.0 {
            errors.push(ValidationError::NegativeEmissionRate {
                name: nucleus.name.clone(),
            });
        }
        if !nucleus.propagation_radius.is_finite() || nucleus.propagation_radius <= 0.0 {
            errors.push(ValidationError::NonPositiveRadius {
                name: nucleus.name.clone(),
            });
        }
        if let PropagationDecay::Exponential { k } = nucleus.decay
            && (!k.is_finite() || k < 0.0)
        {
            errors.push(ValidationError::InvalidDecayK {
                name: nucleus.name.clone(),
            });
        }

        if let Some(ap) = &nucleus.ambient_pressure {
            if !ap.delta_qe.is_finite() || !ap.viscosity.is_finite() {
                errors.push(ValidationError::InvalidAmbientPressure {
                    name: nucleus.name.clone(),
                });
            }
        }

        let x = nucleus.position[0];
        let y = nucleus.position[1];
        if !x.is_finite() || !y.is_finite() || x < min_x || x >= max_x || y < min_y || y >= max_y {
            errors.push(ValidationError::NucleusOutOfBounds {
                name: nucleus.name.clone(),
            });
        }
    }

    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
}

pub fn load_map_config_from_env() -> MapConfig {
    load_map_config_from_env_result().unwrap_or_default()
}

pub fn load_map_config_from_env_result() -> Result<MapConfig, String> {
    let path = selected_map_path_from_env();
    let contents = fs::read_to_string(&path)
        .map_err(|err| format!("Failed to read '{}': {err}", path.display()))?;
    parse_map_config(&contents).map_err(|err| format!("Error in '{}': {err}", path.display()))
}

pub fn load_default_map_asset() -> Result<MapConfig, String> {
    let path = PathBuf::from(format!("{MAPS_DIR}/{DEFAULT_MAP_NAME}.ron"));
    let contents = fs::read_to_string(&path)
        .map_err(|err| format!("Failed to read '{}': {err}", path.display()))?;
    parse_map_config(&contents).map_err(|err| format!("Error in '{}': {err}", path.display()))
}

#[derive(Debug, Clone)]
pub struct SpawnedNucleusSpec {
    pub name: String,
    pub nucleus: EnergyNucleus,
    pub position: Vec2,
    /// Capa 6 opcional: volumen de presión compartido con el radio de propagación (bioma del emisor).
    pub ambient_pressure: Option<AmbientPressureConfig>,
    /// Per-nucleus reservoir override (None = infinite emission).
    pub reservoir: Option<f32>,
}

pub fn resolve_nuclei_for_spawn(config: &MapConfig) -> Vec<SpawnedNucleusSpec> {
    let mut rng = config.seed.map(SeededRng::new);
    let world_width = config.width_cells as f32 * config.cell_size;
    let world_height = config.height_cells as f32 * config.cell_size;
    let min_x = config.origin[0];
    let min_y = config.origin[1];
    let max_x = min_x + world_width;
    let max_y = min_y + world_height;
    let max_inside_x = max_x - f32::EPSILON;
    let max_inside_y = max_y - f32::EPSILON;
    config
        .nuclei
        .iter()
        .map(|entry| {
            let mut pos = Vec2::new(entry.position[0], entry.position[1]);
            let mut emission = entry.emission_rate_qe_s;
            if let Some(r) = &mut rng {
                let jitter = Vec2::new(r.next_range(-0.25, 0.25), r.next_range(-0.25, 0.25));
                pos += jitter * config.cell_size;
                emission += r.next_range(-0.1, 0.1) * entry.emission_rate_qe_s;
                emission = emission.max(0.0);
            }
            pos.x = pos.x.clamp(min_x, max_inside_x);
            pos.y = pos.y.clamp(min_y, max_inside_y);
            SpawnedNucleusSpec {
                name: entry.name.clone(),
                nucleus: EnergyNucleus::new(
                    entry.frequency_hz,
                    emission,
                    entry.propagation_radius,
                    entry.decay,
                ),
                position: pos,
                ambient_pressure: entry.ambient_pressure.clone(),
                reservoir: entry.reservoir,
            }
        })
        .collect()
}

#[derive(Debug, Clone)]
struct SeededRng {
    state: u64,
}

impl SeededRng {
    fn new(seed: u64) -> Self {
        // Evitar colisión seed 0 vs 1 (ambas quedaban en state 1 con max(1)).
        Self {
            state: seed.rotate_left(17).wrapping_add(0xA076_1D64_78BD_642F) | 1,
        }
    }

    fn next_u32(&mut self) -> u32 {
        self.state = self.state.wrapping_mul(6364136223846793005).wrapping_add(1);
        (self.state >> 32) as u32
    }

    fn next_unit(&mut self) -> f32 {
        (self.next_u32() as f32) / (u32::MAX as f32)
    }

    fn next_range(&mut self, min: f32, max: f32) -> f32 {
        min + self.next_unit() * (max - min)
    }
}

#[cfg(test)]
mod tests {
    use super::{
        MapConfig, NucleusConfig, ValidationError, load_map_config_from_env, parse_map_config,
        resolve_nuclei_for_spawn, validate_map_config,
    };
    use crate::worldgen::PropagationDecay;

    #[test]
    fn parse_default_map_rosa_lifecycle_ok() {
        let ron = include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/assets/maps/default.ron"
        ));
        let cfg = parse_map_config(ron).expect("valid RON");
        validate_map_config(&cfg).expect("valid default map");
        assert_eq!(cfg.nuclei.len(), 2);
        assert_eq!(cfg.width_cells, 32);
        assert!((cfg.cell_size - 0.5).abs() < 0.01);
        let terra = cfg.nuclei.iter().find(|n| n.name == "terra_soil").expect("terra nucleus");
        assert!((terra.frequency_hz - 75.0).abs() < 0.01);
        let lux = cfg.nuclei.iter().find(|n| n.name == "lux_sun").expect("lux nucleus");
        assert!((lux.frequency_hz - 1000.0).abs() < 0.01);
    }

    #[test]
    fn parse_morphogenesis_demo_map_three_nuclei_ok() {
        let ron = include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/assets/maps/morphogenesis_demo.ron"
        ));
        let cfg = parse_map_config(ron).expect("valid morphogenesis_demo RON");
        validate_map_config(&cfg).expect("valid morphogenesis demo map");
        assert_eq!(cfg.nuclei.len(), 3);
        assert_eq!(cfg.width_cells, 32);
        assert!(!cfg.fog_of_war);

        let ocean = cfg.nuclei.iter().find(|n| n.name == "ocean_deep").expect("ocean nucleus");
        let ap = ocean.ambient_pressure.as_ref().expect("ocean ambient_pressure");
        assert!((ap.viscosity - 2.5).abs() < 0.01);

        let desert = cfg.nuclei.iter().find(|n| n.name == "scorched_desert").expect("desert nucleus");
        let dap = desert.ambient_pressure.as_ref().expect("desert ambient_pressure");
        assert!((dap.delta_qe - (-2.0)).abs() < 0.01);

        let forest = cfg.nuclei.iter().find(|n| n.name == "temperate_forest").expect("forest nucleus");
        let fap = forest.ambient_pressure.as_ref().expect("forest ambient_pressure");
        assert!((fap.delta_qe - 1.0).abs() < 0.01);
    }

    #[test]
    fn seeded_rng_zero_and_one_differ() {
        let mut cfg = MapConfig {
            seed: Some(0),
            ..Default::default()
        };
        let a = resolve_nuclei_for_spawn(&cfg);
        cfg.seed = Some(1);
        let b = resolve_nuclei_for_spawn(&cfg);
        assert_eq!(a.len(), b.len());
        assert_ne!(a[0].position, b[0].position);
    }

    #[test]
    fn validate_map_config_detects_out_of_bounds() {
        let cfg = MapConfig {
            nuclei: vec![NucleusConfig {
                name: "bad".to_string(),
                position: [10_000.0, 0.0],
                frequency_hz: 75.0,
                emission_rate_qe_s: 100.0,
                propagation_radius: 12.0,
                decay: PropagationDecay::Flat,
                ambient_pressure: None,
                reservoir: None,
            }],
            ..Default::default()
        };
        let err = validate_map_config(&cfg).expect_err("expected failure");
        assert!(
            err.iter().any(
                |e| matches!(e, ValidationError::NucleusOutOfBounds { name } if name == "bad")
            )
        );
    }

    #[test]
    fn validate_map_config_detects_playfield_margin_too_large() {
        let mut cfg = MapConfig::default();
        cfg.width_cells = 4;
        cfg.height_cells = 4;
        cfg.cell_size = 1.0;
        cfg.origin = [0.0, 0.0];
        cfg.playfield_margin_cells = Some(2);
        cfg.nuclei = vec![NucleusConfig {
            name: "n".to_string(),
            position: [1.0, 1.0],
            frequency_hz: 75.0,
            emission_rate_qe_s: 10.0,
            propagation_radius: 0.5,
            decay: PropagationDecay::Flat,
            ambient_pressure: None,
            reservoir: None,
        }];
        let err = validate_map_config(&cfg).expect_err("margin");
        assert!(err.contains(&ValidationError::PlayfieldMarginTooLarge));
    }

    #[test]
    fn validate_map_config_detects_non_positive_frequency() {
        let cfg = MapConfig {
            nuclei: vec![NucleusConfig {
                name: "bad_freq".to_string(),
                position: [0.0, 0.0],
                frequency_hz: -1.0,
                emission_rate_qe_s: 100.0,
                propagation_radius: 12.0,
                decay: PropagationDecay::Flat,
                ambient_pressure: None,
                reservoir: None,
            }],
            ..Default::default()
        };
        let err = validate_map_config(&cfg).expect_err("expected failure");
        assert!(err.iter().any(
            |e| matches!(e, ValidationError::NonPositiveFrequency { name } if name == "bad_freq")
        ));
    }

    #[test]
    fn resolve_nuclei_for_spawn_same_seed_is_reproducible() {
        let cfg = MapConfig {
            seed: Some(777),
            ..Default::default()
        };
        let a = resolve_nuclei_for_spawn(&cfg);
        let b = resolve_nuclei_for_spawn(&cfg);
        assert_eq!(a.len(), b.len());
        assert!(a.iter().zip(&b).all(|(left, right)| {
            left.name == right.name
                && left.nucleus.frequency_hz() == right.nucleus.frequency_hz()
                && left.nucleus.emission_rate_qe_s() == right.nucleus.emission_rate_qe_s()
                && (left.position.x - right.position.x).abs() < f32::EPSILON
                && (left.position.y - right.position.y).abs() < f32::EPSILON
        }));
    }

    #[test]
    fn load_map_config_missing_file_falls_back_to_default() {
        let prev = std::env::var("RESONANCE_MAP").ok();
        unsafe {
            std::env::set_var("RESONANCE_MAP", "__nonexistent_map__");
        }
        let cfg = load_map_config_from_env();
        assert!(!cfg.nuclei.is_empty());
        match prev {
            Some(value) => unsafe {
                std::env::set_var("RESONANCE_MAP", value);
            },
            None => unsafe {
                std::env::remove_var("RESONANCE_MAP");
            },
        }
    }

    #[test]
    fn resolve_nuclei_for_spawn_seeded_stays_inside_world_bounds() {
        let cfg = MapConfig {
            seed: Some(1234),
            width_cells: 2,
            height_cells: 2,
            cell_size: 1.0,
            origin: [0.0, 0.0],
            nuclei: vec![NucleusConfig {
                name: "edge".to_string(),
                position: [0.0, 0.0],
                frequency_hz: 75.0,
                emission_rate_qe_s: 10.0,
                propagation_radius: 1.0,
                decay: PropagationDecay::Flat,
                ambient_pressure: None,
                reservoir: None,
            }],
            ..Default::default()
        };
        let spawned = resolve_nuclei_for_spawn(&cfg);
        assert_eq!(spawned.len(), 1);
        let pos = spawned[0].position;
        assert!((0.0..2.0).contains(&pos.x));
        assert!((0.0..2.0).contains(&pos.y));
    }
}
