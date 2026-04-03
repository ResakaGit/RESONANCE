//! Constantes de referencia para bandas de frecuencia elemental.
//! Derivadas del diseño del almanac — no de los 4 fundamentales.
//!
//! Reference constants for elemental frequency bands.
//! Derived from almanac design — not from the 4 fundamentals.
//!
//! Uso: tests, fixtures, valores por defecto cuando el almanac no está cargado.

/// Frecuencia central de cada elemento [Hz].
/// Central frequency per element [Hz].
pub const FREQ_UMBRA: f32 = 20.0;
pub const FREQ_TERRA: f32 = 75.0;
pub const FREQ_AQUA: f32 = 250.0;
pub const FREQ_IGNIS: f32 = 450.0;
pub const FREQ_VENTUS: f32 = 700.0;
pub const FREQ_LUX: f32 = 1000.0;

/// Bandas de estabilidad (min, max) [Hz].
/// Stability bands (min, max) [Hz].
pub const BAND_UMBRA: (f32, f32) = (10.0, 30.0);
pub const BAND_TERRA: (f32, f32) = (50.0, 84.0);
pub const BAND_AQUA: (f32, f32) = (200.0, 300.0);
pub const BAND_IGNIS: (f32, f32) = (400.0, 500.0);
pub const BAND_VENTUS: (f32, f32) = (600.0, 800.0);
pub const BAND_LUX: (f32, f32) = (900.0, 1100.0);
