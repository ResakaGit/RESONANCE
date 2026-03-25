//! Umbrales y tuning compartidos del sustrato topológico (T1+).

/// Altitud mínima por defecto tras `normalize_heightmap` (docs/design/TOPOLOGY.md / T2).
pub const ALTITUDE_MIN_DEFAULT: f32 = -50.0;

/// Altitud máxima por defecto tras `normalize_heightmap` (docs/design/TOPOLOGY.md / T2).
pub const ALTITUDE_MAX_DEFAULT: f32 = 200.0;

/// Acumulación mínima para clasificar celda como lecho de río (T5+).
pub const RIVER_THRESHOLD: f32 = 100.0;

/// Pendiente en grados a partir de la cual se considera acantilado.
pub const CLIFF_SLOPE_THRESHOLD: f32 = 60.0;

/// Altitud de referencia para modulación V7 (emisión).
pub const REFERENCE_ALTITUDE: f32 = 50.0;

/// Escala de cómo la altitud modula la emisión efectiva.
pub const ALTITUDE_EMISSION_SCALE: f32 = 0.005;

/// Escala de cómo la pendiente modula la difusión efectiva.
pub const SLOPE_DIFFUSION_SCALE: f32 = 0.3;

/// Límite superior (exclusivo) de acumulación para clase `Dry`.
pub const DRAINAGE_DRY: f32 = 10.0;

/// Límite superior (exclusivo) de acumulación para clase `Moist`.
pub const DRAINAGE_MOIST: f32 = 50.0;

/// Límite superior (inclusivo hasta RIVER) para clase `Wet`; `River` es estrictamente mayor que `RIVER_THRESHOLD`.
pub const DRAINAGE_WET: f32 = 100.0;
