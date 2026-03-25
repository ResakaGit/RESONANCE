//! Config topológica data-driven (RON + Bevy Asset), Sprint T9 — patrón `eco::climate`.

use std::env;

use bevy::asset::{AssetId, AssetLoader, LoadContext, io::Reader};
use bevy::prelude::*;
use bevy::reflect::TypePath;
use serde::{Deserialize, Serialize};
use tracing::warn;

use super::constants::{
    ALTITUDE_EMISSION_SCALE, ALTITUDE_MAX_DEFAULT, ALTITUDE_MIN_DEFAULT, REFERENCE_ALTITUDE,
    SLOPE_DIFFUSION_SCALE,
};
pub use super::generators::classifier::ClassificationThresholds;
use super::generators::hydraulics::ErosionParams;
use super::generators::noise::NoiseParams;

fn default_terrain_enabled() -> bool {
    true
}

/// Parámetros de modulación V7 (emisión / difusión / decay) — BLUEPRINT §7.
#[derive(Clone, Copy, PartialEq, Debug, Serialize, Deserialize)]
pub struct ModulationParams {
    pub altitude_emission_scale: f32,
    pub slope_diffusion_scale: f32,
    pub reference_altitude: f32,
    pub decay_peak_factor: f32,
    pub decay_valley_factor: f32,
    pub decay_riverbed_factor: f32,
}

impl Default for ModulationParams {
    fn default() -> Self {
        Self {
            altitude_emission_scale: ALTITUDE_EMISSION_SCALE,
            slope_diffusion_scale: SLOPE_DIFFUSION_SCALE,
            reference_altitude: REFERENCE_ALTITUDE,
            decay_peak_factor: 1.5,
            decay_valley_factor: 0.7,
            decay_riverbed_factor: 0.8,
        }
    }
}

/// Configuración serializable y cargable como asset único (`terrain_config.ron`).
#[derive(Asset, TypePath, Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct TerrainConfig {
    pub seed: u64,
    #[serde(default)]
    pub noise: NoiseParams,
    #[serde(default)]
    pub erosion: ErosionParams,
    #[serde(default)]
    pub classification: ClassificationThresholds,
    #[serde(default)]
    pub modulation: ModulationParams,
    #[serde(default = "default_terrain_enabled")]
    pub enabled: bool,
}

impl Default for TerrainConfig {
    fn default() -> Self {
        Self {
            seed: 0,
            noise: NoiseParams::default(),
            erosion: ErosionParams {
                cycles: 50,
                strength: 0.3,
                deposition_rate: 0.5,
                evaporation: 0.1,
            },
            classification: ClassificationThresholds::default(),
            modulation: ModulationParams::default(),
            enabled: default_terrain_enabled(),
        }
    }
}

/// Copia saneada lista para consumidores (T2+); `generation` sube en cada carga / hot-reload.
#[derive(Resource, Debug, Clone)]
pub struct TerrainConfigRuntime {
    pub handle: Handle<TerrainConfig>,
    pub effective: Option<TerrainConfig>,
    pub generation: u64,
}

#[derive(Resource, Debug, Default)]
pub struct TerrainConfigAssetState {
    pub initialized: bool,
    pub built: bool,
    pub handle: Option<Handle<TerrainConfig>>,
}

#[derive(Default)]
pub struct TerrainConfigRonLoader;

impl AssetLoader for TerrainConfigRonLoader {
    type Asset = TerrainConfig;
    type Settings = ();
    type Error = ron::error::SpannedError;

    async fn load(
        &self,
        reader: &mut dyn Reader,
        _settings: &(),
        _load_context: &mut LoadContext<'_>,
    ) -> Result<Self::Asset, Self::Error> {
        let mut bytes = Vec::new();
        reader.read_to_end(&mut bytes).await?;
        ron::de::from_bytes::<TerrainConfig>(&bytes)
    }

    fn extensions(&self) -> &[&str] {
        &["ron"]
    }
}

/// Corrige valores hostiles; loguea advertencias (no panic).
pub fn sanitize_terrain_config(raw: &TerrainConfig) -> TerrainConfig {
    let mut cfg = raw.clone();
    let mut n = cfg.noise;
    if n.octaves == 0 {
        warn!("terrain_config: noise.octaves=0 invalid; using default (6)");
        n.octaves = NoiseParams::default().octaves;
    }
    if !n.frequency.is_finite() || n.frequency <= 0.0 {
        warn!("terrain_config: noise.frequency invalid; using default");
        n.frequency = NoiseParams::default().frequency;
    }
    if !n.amplitude.is_finite() {
        warn!("terrain_config: noise.amplitude not finite; using default");
        n.amplitude = NoiseParams::default().amplitude;
    }
    if !n.lacunarity.is_finite() || n.lacunarity <= 0.0 {
        warn!("terrain_config: noise.lacunarity invalid; using default");
        n.lacunarity = NoiseParams::default().lacunarity;
    }
    if !n.persistence.is_finite() || n.persistence < 0.0 {
        warn!("terrain_config: noise.persistence invalid; using default");
        n.persistence = NoiseParams::default().persistence;
    }
    if !n.min_height.is_finite() {
        n.min_height = ALTITUDE_MIN_DEFAULT;
    }
    if !n.max_height.is_finite() {
        n.max_height = ALTITUDE_MAX_DEFAULT;
    }
    if n.min_height >= n.max_height {
        warn!("terrain_config: min_height >= max_height; resetting to BLUEPRINT range");
        n.min_height = ALTITUDE_MIN_DEFAULT;
        n.max_height = ALTITUDE_MAX_DEFAULT;
    }
    cfg.noise = n;

    let mut e = cfg.erosion;
    if e.strength < 0.0 || !e.strength.is_finite() {
        warn!("terrain_config: erosion.strength invalid; using default (0.3)");
        e.strength = ErosionParams::default().strength;
    }
    if !e.deposition_rate.is_finite() || e.deposition_rate < 0.0 {
        e.deposition_rate = ErosionParams::default().deposition_rate;
    }
    if !e.evaporation.is_finite() || e.evaporation < 0.0 {
        e.evaporation = ErosionParams::default().evaporation;
    }
    cfg.erosion = e;

    let mut m = cfg.modulation;
    if !m.altitude_emission_scale.is_finite() || m.altitude_emission_scale < 0.0 {
        warn!("terrain_config: modulation.altitude_emission_scale invalid; default");
        m.altitude_emission_scale = ModulationParams::default().altitude_emission_scale;
    }
    if !m.slope_diffusion_scale.is_finite() || m.slope_diffusion_scale < 0.0 {
        warn!("terrain_config: modulation.slope_diffusion_scale invalid; default");
        m.slope_diffusion_scale = ModulationParams::default().slope_diffusion_scale;
    }
    if !m.reference_altitude.is_finite() {
        m.reference_altitude = ModulationParams::default().reference_altitude;
    }
    if !m.decay_peak_factor.is_finite() || m.decay_peak_factor < 0.0 {
        m.decay_peak_factor = ModulationParams::default().decay_peak_factor;
    }
    if !m.decay_valley_factor.is_finite() || m.decay_valley_factor < 0.0 {
        m.decay_valley_factor = ModulationParams::default().decay_valley_factor;
    }
    if !m.decay_riverbed_factor.is_finite() || m.decay_riverbed_factor < 0.0 {
        m.decay_riverbed_factor = ModulationParams::default().decay_riverbed_factor;
    }
    cfg.modulation = m;

    let mut c = cfg.classification;
    if !c.peak_altitude.is_finite() {
        c.peak_altitude = ClassificationThresholds::default().peak_altitude;
    }
    if !c.ridge_altitude.is_finite() {
        c.ridge_altitude = ClassificationThresholds::default().ridge_altitude;
    }
    if !c.plateau_altitude.is_finite() {
        c.plateau_altitude = ClassificationThresholds::default().plateau_altitude;
    }
    if !c.cliff_slope.is_finite() || c.cliff_slope < 0.0 {
        c.cliff_slope = ClassificationThresholds::default().cliff_slope;
    }
    if !c.slope_threshold.is_finite() || c.slope_threshold < 0.0 {
        c.slope_threshold = ClassificationThresholds::default().slope_threshold;
    }
    if !c.river_accumulation.is_finite() || c.river_accumulation < 0.0 {
        c.river_accumulation = ClassificationThresholds::default().river_accumulation;
    }
    if !c.basin_max_slope.is_finite() || c.basin_max_slope < 0.0 {
        c.basin_max_slope = ClassificationThresholds::default().basin_max_slope;
    }
    if !c.basin_max_altitude.is_finite() {
        c.basin_max_altitude = ClassificationThresholds::default().basin_max_altitude;
    }
    if !c.valley_max_altitude.is_finite() {
        c.valley_max_altitude = ClassificationThresholds::default().valley_max_altitude;
    }
    cfg.classification = c;

    cfg
}

/// `true` si hay que re-leer el asset: primera vez (`!built`) o evento nuestro (T9 hot-reload).
pub fn terrain_config_wants_reload(
    our_id: AssetId<TerrainConfig>,
    built: bool,
    events: impl Iterator<Item = AssetEvent<TerrainConfig>>,
) -> bool {
    let mut should_apply = !built;
    for ev in events {
        let hit = match ev {
            AssetEvent::Added { id }
            | AssetEvent::Modified { id }
            | AssetEvent::LoadedWithDependencies { id } => id == our_id,
            _ => false,
        };
        if hit {
            should_apply = true;
        }
    }
    should_apply
}

/// Startup: encola `terrain_config.ron` e inserta [`TerrainConfigRuntime`].
pub fn init_terrain_config_system(
    asset_server: Res<AssetServer>,
    mut asset_state: ResMut<TerrainConfigAssetState>,
    mut commands: Commands,
    existing: Option<Res<TerrainConfigRuntime>>,
) {
    if asset_state.initialized {
        return;
    }
    asset_state.initialized = true;
    let path = env::var("RESONANCE_TERRAIN_CONFIG")
        .ok()
        .filter(|s| !s.trim().is_empty())
        .unwrap_or_else(|| "terrain_config.ron".to_string());
    let handle: Handle<TerrainConfig> = asset_server.load(path);
    asset_state.handle = Some(handle.clone());
    asset_state.built = false;

    if existing.is_none() {
        commands.insert_resource(TerrainConfigRuntime {
            handle,
            effective: None,
            generation: 0,
        });
    }
}

/// Carga inicial + hot-reload vía `AssetEvent` (mismo patrón que `climate_config_hot_reload_system`).
pub fn terrain_config_loader_system(
    mut asset_state: ResMut<TerrainConfigAssetState>,
    mut runtime: ResMut<TerrainConfigRuntime>,
    assets: Res<Assets<TerrainConfig>>,
    mut events: EventReader<AssetEvent<TerrainConfig>>,
) {
    let Some(handle) = asset_state.handle.clone() else {
        return;
    };
    runtime.handle = handle.clone();
    let ours = handle.id();

    let pending: Vec<AssetEvent<TerrainConfig>> = events.read().cloned().collect();
    let should_apply = terrain_config_wants_reload(ours, asset_state.built, pending.into_iter());

    if !should_apply {
        return;
    }

    let Some(raw) = assets.get(&handle) else {
        // Reintentar en el siguiente frame si el evento llegó antes que el asset en `Assets`.
        if should_apply {
            asset_state.built = false;
        }
        return;
    };

    let effective = sanitize_terrain_config(raw);
    runtime.effective = Some(effective);
    runtime.generation = runtime.generation.saturating_add(1);
    asset_state.built = true;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::topology::constants::{ALTITUDE_MAX_DEFAULT, ALTITUDE_MIN_DEFAULT};

    fn assert_noise_sane(n: &NoiseParams) {
        assert!(n.octaves > 0);
        assert!(n.frequency.is_finite() && n.frequency > 0.0);
        assert!(n.amplitude.is_finite());
        assert!(n.lacunarity.is_finite() && n.lacunarity > 0.0);
        assert!(n.persistence.is_finite() && n.persistence >= 0.0);
        assert!(n.min_height.is_finite() && n.max_height.is_finite());
        assert!(n.min_height < n.max_height);
    }

    fn assert_erosion_sane(e: &ErosionParams) {
        assert!(e.strength >= 0.0 && e.strength.is_finite());
        assert!(e.deposition_rate >= 0.0 && e.deposition_rate.is_finite());
        assert!(e.evaporation >= 0.0 && e.evaporation.is_finite());
    }

    #[test]
    fn terrain_config_ron_parses() {
        let raw = include_str!("../../assets/terrain_config.ron");
        let parsed: TerrainConfig = ron::de::from_str(raw).expect("parse terrain_config.ron");
        let s = sanitize_terrain_config(&parsed);
        assert_noise_sane(&s.noise);
        assert_erosion_sane(&s.erosion);
    }

    #[test]
    fn enabled_false_parses() {
        let raw = r#"(
            seed: 0,
            enabled: false,
        )"#;
        let parsed: TerrainConfig = ron::de::from_str(raw).expect("parse minimal RON");
        assert!(!parsed.enabled);
    }

    #[test]
    fn invalid_octaves_zero_uses_default_six() {
        let mut cfg = TerrainConfig::default();
        cfg.noise.octaves = 0;
        let s = sanitize_terrain_config(&cfg);
        assert_eq!(s.noise.octaves, 6);
    }

    #[test]
    fn invalid_erosion_strength_negative_uses_default() {
        let mut cfg = TerrainConfig::default();
        cfg.erosion.strength = -1.0;
        let s = sanitize_terrain_config(&cfg);
        assert_eq!(s.erosion.strength, ErosionParams::default().strength);
    }

    #[test]
    fn invalid_noise_min_ge_max_resets_blueprint_altitude_range() {
        let mut cfg = TerrainConfig::default();
        cfg.noise.min_height = 100.0;
        cfg.noise.max_height = 20.0;
        let s = sanitize_terrain_config(&cfg);
        assert_eq!(s.noise.min_height, ALTITUDE_MIN_DEFAULT);
        assert_eq!(s.noise.max_height, ALTITUDE_MAX_DEFAULT);
    }

    #[test]
    fn preset_mountain_valley_has_positive_erosion_cycles() {
        let raw = include_str!("../../assets/terrain_presets/mountain_valley.ron");
        let parsed: TerrainConfig = ron::de::from_str(raw).expect("parse");
        assert!(parsed.erosion.cycles > 0);
    }

    #[test]
    fn preset_flat_plains_has_low_amplitude() {
        let raw = include_str!("../../assets/terrain_presets/flat_plains.ron");
        let parsed: TerrainConfig = ron::de::from_str(raw).expect("parse");
        assert!(
            parsed.noise.amplitude < 45.0,
            "flat_plains must be gentle relief"
        );
    }

    #[test]
    fn terrain_config_ron_round_trip_identical() {
        let c = TerrainConfig::default();
        let s = ron::ser::to_string(&c).expect("serialize");
        let back: TerrainConfig = ron::de::from_str(&s).expect("deserialize");
        assert_eq!(c, back);
    }

    #[test]
    fn wants_reload_when_not_yet_built_even_without_events() {
        let mut assets = Assets::<TerrainConfig>::default();
        let h = assets.add(TerrainConfig::default());
        let id = h.id();
        assert!(terrain_config_wants_reload(id, false, std::iter::empty()));
    }

    #[test]
    fn wants_reload_false_when_built_and_no_matching_events() {
        let mut assets = Assets::<TerrainConfig>::default();
        let h = assets.add(TerrainConfig::default());
        let id = h.id();
        assert!(!terrain_config_wants_reload(id, true, std::iter::empty()));
    }

    #[test]
    fn wants_reload_true_on_modified_matching_asset_id() {
        let mut assets = Assets::<TerrainConfig>::default();
        let h = assets.add(TerrainConfig::default());
        let id = h.id();
        let ev = AssetEvent::Modified { id };
        assert!(terrain_config_wants_reload(id, true, std::iter::once(ev)));
    }

    #[test]
    fn wants_reload_false_on_modified_other_asset_semantics() {
        let mut assets = Assets::<TerrainConfig>::default();
        let a = assets.add(TerrainConfig::default());
        let b = assets.add(TerrainConfig {
            seed: 9,
            ..Default::default()
        });
        let id_a = a.id();
        let id_b = b.id();
        let ev = AssetEvent::Modified { id: id_b };
        assert!(!terrain_config_wants_reload(
            id_a,
            true,
            std::iter::once(ev)
        ));
    }

    #[test]
    fn apply_sanitized_config_bumps_runtime_generation_contract() {
        let mut assets = Assets::<TerrainConfig>::default();
        let h = assets.add(TerrainConfig {
            seed: 777,
            ..Default::default()
        });
        let mut runtime = TerrainConfigRuntime {
            handle: h.clone(),
            effective: None,
            generation: 10,
        };
        let raw = assets.get(&h).expect("asset");
        runtime.effective = Some(sanitize_terrain_config(raw));
        runtime.generation = runtime.generation.saturating_add(1);
        assert_eq!(runtime.generation, 11);
        assert_eq!(runtime.effective.as_ref().unwrap().seed, 777);
    }

    #[test]
    fn preset_archipelago_parses_and_high_frequency() {
        let raw = include_str!("../../assets/terrain_presets/archipelago.ron");
        let parsed: TerrainConfig = ron::de::from_str(raw).expect("parse");
        assert!(parsed.noise.frequency > 0.025);
        assert!(parsed.noise.amplitude < 80.0);
    }
}
