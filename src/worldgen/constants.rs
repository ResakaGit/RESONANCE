// ── LOD / grid / mapas / propagación / materialización (M4) ──
/// Distancia máxima (mundo) para LOD cercano.
pub const LOD_NEAR_MAX: f32 = 30.0;

/// Distancia máxima para LOD medio (más allá = lejano).
pub const LOD_MID_MAX: f32 = 80.0;

/// Tamaño de chunk del grid de campo (celdas por lado de chunk).
pub const FIELD_GRID_CHUNK_SIZE: u32 = 16;

/// Nombre de mapa RON por defecto (`assets/maps/{name}.ron`).
pub(crate) const DEFAULT_MAP_NAME: &str = "default";

/// Directorio de definiciones de mapa.
pub(crate) const MAPS_DIR: &str = "assets/maps";

/// Mínima distancia en propagación para evitar división por cero (m).
pub(crate) const MIN_DISTANCE_M: f32 = 1e-4;

/// Duración en ticks de la transición de estación (materialización).
pub const SEASON_TRANSITION_TICKS: u32 = 60;

/// Mínimo de qe en celda para materializar. Derived: self_sustaining_qe_min / 2 = 10.0.
pub const MIN_MATERIALIZATION_QE: f32 = 10.0; // Matches derived_thresholds::min_materialization_qe()

/// Tamaño de celda del campo en unidades de mundo.
pub const FIELD_CELL_SIZE: f32 = 2.0;

/// `Name` de entidades de abiogénesis (EA5): anclan celda sin componente [`Materialized`](crate::worldgen::Materialized).
pub const ABIOGENESIS_FIELD_OCCUPANT_NAME: &str = "flora_emergent";

/// Pérdida base de qe por celda/s (Axiom 4). Matches derived_thresholds::field_decay_rate().
pub const FIELD_DECAY_RATE: f32 = 1.0;

/// Densidad de referencia visual (visual calibration, not axiom-derived).
pub const REFERENCE_DENSITY: f32 = super::visual_calibration::VISUAL_REFERENCE_DENSITY;

/// Umbral de densidad clase baja (Axiom 1). Matches DENSITY_SCALE fundamental.
pub const DENSITY_LOW_THRESHOLD: f32 = 20.0;

/// Umbral de densidad clase alta (visual calibration).
pub const DENSITY_HIGH_THRESHOLD: f32 = super::visual_calibration::VISUAL_DENSITY_HIGH;

/// Pureza para materialización pura vs compuesta (visual calibration).
pub const PURITY_THRESHOLD: f32 = super::visual_calibration::VISUAL_PURITY_THRESHOLD;

/// Cantidad de ticks de warmup antes de materializar mundo inicial.
pub const WARMUP_TICKS: u32 = 60;

/// Conductividad lateral del campo (visual calibration).
pub const FIELD_CONDUCTIVITY_SPREAD: f32 = super::visual_calibration::VISUAL_CONDUCTIVITY_SPREAD;

/// Máximo de contribuciones por celda para evitar crecimiento no acotado.
pub const MAX_FREQUENCY_CONTRIBUTIONS: usize = 8;

/// Umbral para descartar contribuciones insignificantes.
pub const MIN_CONTRIBUTION_INTENSITY: f32 = 0.001;

// ── Derivación visual (escala / emisión / opacidad desde densidad y estado) ──
/// Escala mínima tras sanitizar (evita sprites invisibles).
pub const VISUAL_MIN_SCALE: f32 = 0.05;

/// Canal sRGB del gris neutro cuando no hay banda válida o pureza 0 (canónico: `blueprint::constants::FIELD_VISUAL_NEUTRAL_GRAY_CHANNEL`).
pub const VISUAL_NEUTRAL_GRAY_CHANNEL: f32 =
    crate::blueprint::constants::FIELD_VISUAL_NEUTRAL_GRAY_CHANNEL;

pub const VISUAL_SCALE_SOLID_BASE: f32 = 0.7;

pub const VISUAL_SCALE_SOLID_RANGE: f32 = 0.9;

pub const VISUAL_SCALE_LIQUID: f32 = 1.0;

pub const VISUAL_SCALE_GAS_BASE: f32 = 1.4;

pub const VISUAL_SCALE_GAS_RANGE: f32 = 0.7;

pub const VISUAL_SCALE_PLASMA_BASE: f32 = 1.1;

pub const VISUAL_SCALE_PLASMA_RANGE: f32 = 0.5;

pub const VISUAL_EMISSION_PLASMA_OFFSET: f32 = 0.35;

/// Suavizado de emisión plasma: temp / (temp + este valor).
pub const VISUAL_EMISSION_PLASMA_TEMP_DIVISOR: f32 = 100.0;

pub const VISUAL_EMISSION_GAS_SCALE: f32 = 0.6;

pub const VISUAL_EMISSION_GAS_TEMP_DIVISOR: f32 = 300.0;

pub const VISUAL_OPACITY_LIQUID_BASE: f32 = 0.65;

pub const VISUAL_OPACITY_LIQUID_RANGE: f32 = 0.25;

pub const VISUAL_OPACITY_GAS_BASE: f32 = 0.15;

pub const VISUAL_OPACITY_GAS_RANGE: f32 = 0.30;

pub const VISUAL_OPACITY_PLASMA_BASE: f32 = 0.55;

pub const VISUAL_OPACITY_PLASMA_RANGE: f32 = 0.35;

/// Cuánto reduce la interferencia constructiva el peso del color secundario (canónico: `FIELD_COMPOUND_BLEND_CONSTRUCTIVE_INTERFERENCE_WEIGHT`).
pub const COMPOUND_BLEND_CONSTRUCTIVE_INTERFERENCE_WEIGHT: f32 =
    crate::blueprint::constants::FIELD_COMPOUND_BLEND_CONSTRUCTIVE_INTERFERENCE_WEIGHT;

/// Mezcla base primario/secundario en rama destructiva (canónico: `FIELD_COMPOUND_BLEND_DESTRUCTIVE_BASE`).
pub const COMPOUND_BLEND_DESTRUCTIVE_BASE: f32 =
    crate::blueprint::constants::FIELD_COMPOUND_BLEND_DESTRUCTIVE_BASE;

// ── Spawns desde grid de worldgen (visual calibration) ──
/// Bond energy for terrain tiles (visual calibration).
pub const MATERIALIZED_SPAWN_BOND_ENERGY: f32 = super::visual_calibration::VISUAL_SPAWN_BOND_ENERGY;

/// Thermal conductivity for terrain tiles (visual calibration).
pub const MATERIALIZED_SPAWN_THERMAL_CONDUCTIVITY: f32 = super::visual_calibration::VISUAL_SPAWN_THERMAL_CONDUCTIVITY;

/// Collider = half cell (geometric, Axiom 1).
pub const MATERIALIZED_COLLIDER_RADIUS_FACTOR: f32 = 0.5;

/// Min collider radius (geometric floor).
pub const MATERIALIZED_MIN_COLLIDER_RADIUS: f32 = 0.01;

/// Umbrales de aserción para tests EPI2 (`field_visual_sample`) y regresión campo→RGB lineal.
#[cfg(test)]
pub mod field_sample_test_thresholds {
    pub const RGB_APPROX_EPS: f32 = 1e-4;
    /// Misma magnitud que `blueprint::constants::eac3_test_thresholds::MIN_RGB_L1_DISTINCT`.
    pub const MIN_BAND_L1: f32 =
        crate::blueprint::constants::eac3_test_thresholds::MIN_RGB_L1_DISTINCT;
    pub const MIN_COMPOUND_L1: f32 = 0.05;
    /// Paso `t` en tests de interferencia: con Δf típico Ignis/Aqua (200 Hz) → fase π/2 en `cos(2π Δf t)`.
    pub const INTERFERENCE_TEST_PHASE_STEP_T: f32 = 0.00125;
}

#[cfg(test)]
mod tests {
    use super::{
        DENSITY_HIGH_THRESHOLD, DENSITY_LOW_THRESHOLD, MAX_FREQUENCY_CONTRIBUTIONS,
        MIN_CONTRIBUTION_INTENSITY, PURITY_THRESHOLD,
    };

    #[test]
    fn worldgen_constants_invariants_hold() {
        assert!((0.0..=1.0).contains(&PURITY_THRESHOLD));
        assert!(DENSITY_LOW_THRESHOLD < DENSITY_HIGH_THRESHOLD);
        assert!(MIN_CONTRIBUTION_INTENSITY > 0.0);
        assert!(MAX_FREQUENCY_CONTRIBUTIONS > 0);
    }
}
