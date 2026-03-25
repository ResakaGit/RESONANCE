//! Clima global: estaciones, perfiles data-driven y offsets efectivos por tick.
//! Ver `docs/sprints/ECO_BOUNDARIES/README.md` y `docs/design/ECO_BOUNDARIES.md` §7.

use bevy::asset::{AssetLoader, LoadContext, io::Reader};
use bevy::prelude::*;
use bevy::reflect::TypePath;
use serde::{Deserialize, Serialize};

/// Estación del ciclo anual (hemisferio único; el mapa define escala temporal vía RON).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[repr(u8)]
pub enum Season {
    Spring = 0,
    Summer = 1,
    Autumn = 2,
    Winter = 3,
}

impl Season {
    pub fn from_index(idx: usize) -> Self {
        match idx % 4 {
            0 => Season::Spring,
            1 => Season::Summer,
            2 => Season::Autumn,
            _ => Season::Winter,
        }
    }

    pub fn next(self) -> Season {
        match self {
            Season::Spring => Season::Summer,
            Season::Summer => Season::Autumn,
            Season::Autumn => Season::Winter,
            Season::Winter => Season::Spring,
        }
    }
}

/// Modificadores de una estación (offsets sobre valores base de zona; E5 consumirá esto).
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct SeasonProfile {
    pub temperature_offset: f32,
    pub precipitation_factor: f32,
    pub wind_intensity: f32,
}

impl SeasonProfile {
    /// Clima desactivado u offsets nulos (no confundir con default RON: multiplicadores neutros = 1.0).
    pub const ZERO: Self = Self {
        temperature_offset: 0.0,
        precipitation_factor: 0.0,
        wind_intensity: 0.0,
    };

    fn lerp(a: Self, b: Self, t: f32) -> Self {
        let t = t.clamp(0.0, 1.0);
        Self {
            temperature_offset: a.temperature_offset
                + (b.temperature_offset - a.temperature_offset) * t,
            precipitation_factor: a.precipitation_factor
                + (b.precipitation_factor - a.precipitation_factor) * t,
            wind_intensity: a.wind_intensity + (b.wind_intensity - a.wind_intensity) * t,
        }
    }
}

fn default_season_duration() -> u32 {
    100
}

fn default_transition_window() -> f32 {
    0.2
}

fn default_climate_enabled() -> bool {
    true
}

/// Configuración cargada desde `assets/climate_config.ron`.
#[derive(Asset, TypePath, Clone, Debug, Serialize, Deserialize)]
pub struct ClimateConfig {
    #[serde(default = "default_season_duration")]
    pub season_duration_ticks: u32,
    /// Fracción [0,1] del final de cada estación dedicada a interpolar hacia la siguiente.
    #[serde(default = "default_transition_window")]
    pub transition_window: f32,
    #[serde(default = "default_climate_enabled")]
    pub enabled: bool,
    #[serde(default)]
    pub spring: SeasonProfile,
    #[serde(default)]
    pub summer: SeasonProfile,
    #[serde(default)]
    pub autumn: SeasonProfile,
    #[serde(default)]
    pub winter: SeasonProfile,
}

impl Default for SeasonProfile {
    /// Campos omitidos en RON: sin offset térmico, multiplicadores neutros (§7 blueprint).
    fn default() -> Self {
        Self {
            temperature_offset: 0.0,
            precipitation_factor: 1.0,
            wind_intensity: 1.0,
        }
    }
}

impl Default for ClimateConfig {
    fn default() -> Self {
        Self {
            season_duration_ticks: default_season_duration(),
            transition_window: default_transition_window(),
            enabled: default_climate_enabled(),
            spring: SeasonProfile {
                temperature_offset: 0.0,
                precipitation_factor: 1.0,
                wind_intensity: 1.0,
            },
            summer: SeasonProfile {
                temperature_offset: 20.0,
                precipitation_factor: 0.5,
                wind_intensity: 0.8,
            },
            autumn: SeasonProfile {
                temperature_offset: -5.0,
                precipitation_factor: 1.2,
                wind_intensity: 1.0,
            },
            winter: SeasonProfile {
                temperature_offset: -20.0,
                precipitation_factor: 1.5,
                wind_intensity: 1.2,
            },
        }
    }
}

fn profile_for_season(cfg: &ClimateConfig, season: Season) -> SeasonProfile {
    match season {
        Season::Spring => cfg.spring,
        Season::Summer => cfg.summer,
        Season::Autumn => cfg.autumn,
        Season::Winter => cfg.winter,
    }
}

/// Avanza un tick de simulación: estación, progreso y offsets efectivos (una vez por tick, no por entidad).
pub fn step_climate_state(state: &mut ClimateState, cfg: &ClimateConfig) {
    let tick = state.current_tick;
    apply_tick_snapshot(state, cfg, tick);
    state.current_tick = state.current_tick.saturating_add(1);
}

/// Calcula snapshot (season, cycle_progress, effective) para un `tick` global sin mutar `current_tick`.
pub fn snapshot_for_tick(cfg: &ClimateConfig, tick: u64) -> (Season, f32, SeasonProfile) {
    if !cfg.enabled {
        return (
            Season::from_index((tick / cfg.season_duration_ticks.max(1) as u64) as usize),
            0.0,
            SeasonProfile::ZERO,
        );
    }

    let duration = cfg.season_duration_ticks.max(1);
    let duration_f = duration as f32;
    let season_index = (tick / duration as u64) as usize;
    let season = Season::from_index(season_index);
    let tick_in_season = (tick % duration as u64) as u32;

    let cycle_progress = tick_in_season as f32 / duration_f;

    let tw = cfg.transition_window.clamp(0.0, 1.0);
    let pure_cutoff = ((1.0 - tw) * duration_f).floor() as u32;
    let pure_cutoff = pure_cutoff.min(duration.saturating_sub(1));

    let current = profile_for_season(cfg, season);
    let next = profile_for_season(cfg, season.next());

    let effective = if tick_in_season < pure_cutoff {
        current
    } else {
        let transition_start = pure_cutoff;
        let transition_len = duration.saturating_sub(transition_start).max(1);
        let t = (tick_in_season.saturating_sub(transition_start)) as f32 / transition_len as f32;
        SeasonProfile::lerp(current, next, t)
    };

    (season, cycle_progress, effective)
}

fn apply_tick_snapshot(state: &mut ClimateState, cfg: &ClimateConfig, tick: u64) {
    let (season, cycle_progress, effective) = snapshot_for_tick(cfg, tick);
    state.season = season;
    state.cycle_progress = cycle_progress;
    state.effective = effective;
}

/// Estado runtime del clima (recalculado por `climate_tick_system` cada tick de simulación).
#[derive(Resource, Debug)]
pub struct ClimateState {
    pub config_handle: Handle<ClimateConfig>,
    pub current_tick: u64,
    pub season: Season,
    pub cycle_progress: f32,
    pub effective: SeasonProfile,
}

impl ClimateState {
    pub fn new(config_handle: Handle<ClimateConfig>) -> Self {
        Self {
            config_handle,
            current_tick: 0,
            season: Season::Spring,
            cycle_progress: 0.0,
            effective: SeasonProfile::ZERO,
        }
    }

    /// Offsets ya interpolados para el último `step` aplicado (O(1) para consumidores).
    pub fn effective_offsets(&self) -> SeasonProfile {
        self.effective
    }
}

/// Handle de carga de `climate_config.ron` (hot-reload vía `AssetEvent` como el almanaque).
#[derive(Resource, Debug, Default)]
pub struct ClimateAssetState {
    pub initialized: bool,
    pub handle: Option<Handle<ClimateConfig>>,
}

#[derive(Default)]
pub struct ClimateConfigLoader;

impl AssetLoader for ClimateConfigLoader {
    type Asset = ClimateConfig;
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
        ron::de::from_bytes::<ClimateConfig>(&bytes)
    }

    fn extensions(&self) -> &[&str] {
        &["ron"]
    }
}

/// Startup: encola `climate_config.ron` y crea [`ClimateState`] cuando el asset existe.
pub fn init_climate_config_system(
    asset_server: Res<AssetServer>,
    mut asset_state: ResMut<ClimateAssetState>,
    mut commands: Commands,
    existing: Option<Res<ClimateState>>,
) {
    if asset_state.initialized {
        return;
    }
    asset_state.initialized = true;
    let handle: Handle<ClimateConfig> = asset_server.load("climate_config.ron");
    asset_state.handle = Some(handle.clone());

    if existing.is_none() {
        commands.insert_resource(ClimateState::new(handle));
    }
}

/// Hot-reload: si cambió nuestro `climate_config.ron`, recalcula offsets del **mismo** `current_tick`
/// antes de que [`climate_tick_system`] avance el contador (orden: este sistema va primero en PrePhysics).
pub fn climate_config_hot_reload_system(
    mut events: EventReader<AssetEvent<ClimateConfig>>,
    mut climate: ResMut<ClimateState>,
    configs: Res<Assets<ClimateConfig>>,
) {
    let ours = climate.config_handle.id();
    let mut hit = false;
    for ev in events.read() {
        let id_opt = match ev {
            AssetEvent::Added { id }
            | AssetEvent::Modified { id }
            | AssetEvent::LoadedWithDependencies { id } => Some(*id),
            _ => None,
        };
        if id_opt == Some(ours) {
            hit = true;
        }
    }
    if hit {
        if let Some(cfg) = configs.get(&climate.config_handle) {
            // `step_climate_state` aplica snapshot(t) y luego incrementa `current_tick` → el clima
            // visible corresponde al tick `current_tick - 1` (ver tests de integración).
            let tick = climate.current_tick.saturating_sub(1);
            apply_tick_snapshot(&mut climate, cfg, tick);
        }
    }
}

/// Un tick de simulación: lee config vigente y actualiza offsets efectivos.
pub fn climate_tick_system(mut climate: ResMut<ClimateState>, configs: Res<Assets<ClimateConfig>>) {
    let Some(cfg) = configs.get(&climate.config_handle) else {
        return;
    };
    step_climate_state(&mut climate, cfg);
}

#[cfg(test)]
mod tests {
    use super::*;

    fn cfg_test_100() -> ClimateConfig {
        ClimateConfig {
            season_duration_ticks: 100,
            transition_window: 0.2,
            enabled: true,
            spring: SeasonProfile {
                temperature_offset: 10.0,
                precipitation_factor: 1.0,
                wind_intensity: 1.0,
            },
            summer: SeasonProfile {
                temperature_offset: 30.0,
                precipitation_factor: 2.0,
                wind_intensity: 2.0,
            },
            autumn: SeasonProfile::ZERO,
            winter: SeasonProfile::ZERO,
        }
    }

    #[test]
    fn season_next_cycles() {
        assert_eq!(Season::Spring.next(), Season::Summer);
        assert_eq!(Season::Summer.next(), Season::Autumn);
        assert_eq!(Season::Autumn.next(), Season::Winter);
        assert_eq!(Season::Winter.next(), Season::Spring);
    }

    #[test]
    fn season_changes_at_duration_boundary() {
        let cfg = cfg_test_100();
        let (s99, _, _) = snapshot_for_tick(&cfg, 99);
        let (s100, _, _) = snapshot_for_tick(&cfg, 100);
        assert_eq!(s99, Season::Spring);
        assert_eq!(s100, Season::Summer);
    }

    #[test]
    fn cycle_progress_linear_within_season() {
        let cfg = cfg_test_100();
        let (_, p0, _) = snapshot_for_tick(&cfg, 0);
        let (_, p50, _) = snapshot_for_tick(&cfg, 50);
        let (_, p99, _) = snapshot_for_tick(&cfg, 99);
        assert!((p0 - 0.0).abs() < 1e-5);
        assert!((p50 - 0.5).abs() < 1e-5);
        assert!((p99 - 0.99).abs() < 1e-4);
    }

    #[test]
    fn first_80_percent_pure_season_profile() {
        let cfg = cfg_test_100();
        // duration 100, tw 0.2 → pure tick_in < 80 → ticks 0..79
        let (_, _, eff79) = snapshot_for_tick(&cfg, 79);
        assert_eq!(eff79, cfg.spring);
    }

    #[test]
    fn last_20_percent_interpolates_toward_next() {
        let cfg = cfg_test_100();
        let (_, _, eff80) = snapshot_for_tick(&cfg, 80);
        let (_, _, eff90) = snapshot_for_tick(&cfg, 90);
        // t=0 en 80 → spring; t>0 después
        assert_eq!(eff80, cfg.spring);
        assert!(eff90.temperature_offset > cfg.spring.temperature_offset);
        assert!(eff90.temperature_offset < cfg.summer.temperature_offset);
    }

    #[test]
    fn first_tick_of_new_season_matches_next_pure_profile() {
        let cfg = cfg_test_100();
        let (_, _, eff100) = snapshot_for_tick(&cfg, 100);
        assert_eq!(eff100, cfg.summer);
    }

    #[test]
    fn disabled_yields_zero_offsets() {
        let mut cfg = cfg_test_100();
        cfg.enabled = false;
        let (_, _, eff) = snapshot_for_tick(&cfg, 50);
        assert_eq!(eff, SeasonProfile::ZERO);
    }

    #[test]
    fn climate_config_ron_parses() {
        let raw = include_str!("../../assets/climate_config.ron");
        let parsed: ClimateConfig = ron::de::from_str(raw).expect("parse climate_config.ron");
        assert!(parsed.season_duration_ticks > 0);
        assert!((parsed.summer.temperature_offset - 20.0).abs() < 1e-3);
    }

    #[test]
    fn hot_reload_semantics_new_config_reapplies_at_same_tick() {
        let mut climate = ClimateState {
            config_handle: Handle::default(),
            current_tick: 79,
            season: Season::Spring,
            cycle_progress: 0.0,
            effective: SeasonProfile::ZERO,
        };
        let cfg_a = cfg_test_100();
        apply_tick_snapshot(&mut climate, &cfg_a, 79);
        assert_eq!(climate.effective, cfg_a.spring);

        let mut cfg_b = cfg_a.clone();
        cfg_b.spring.temperature_offset = 123.0;
        apply_tick_snapshot(&mut climate, &cfg_b, 79);
        assert_eq!(climate.effective.temperature_offset, 123.0);
        assert_eq!(climate.current_tick, 79);
    }

    #[test]
    fn after_step_effective_matches_snapshot_at_previous_counter() {
        let cfg = cfg_test_100();
        let mut climate = ClimateState {
            config_handle: Handle::default(),
            current_tick: 0,
            season: Season::Spring,
            cycle_progress: 0.0,
            effective: SeasonProfile::ZERO,
        };
        for _ in 0..50 {
            step_climate_state(&mut climate, &cfg);
        }
        let (_, _, expected) = snapshot_for_tick(&cfg, 49);
        assert_eq!(climate.effective, expected);
        assert_eq!(climate.current_tick, 50);
    }

    #[test]
    fn season_profile_default_ron_omission_is_neutral_multipliers() {
        let d: SeasonProfile = SeasonProfile::default();
        assert_eq!(d.precipitation_factor, 1.0);
        assert_eq!(d.wind_intensity, 1.0);
        assert_eq!(d.temperature_offset, 0.0);
    }
}
