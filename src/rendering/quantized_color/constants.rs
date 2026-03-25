//! Constantes del motor de color cuantizado: reexportan blueprint donde aplica (SSOT).

pub use crate::blueprint::constants::{
    QUANTIZED_COLOR_RHO_CLAMP_FLOOR, QUANTIZED_COLOR_RHO_MIN, VISUAL_QE_REFERENCE,
};

/// Muestras por elemento al construir `PaletteRegistry` desde el almanac.
pub const DEFAULT_PALETTE_N_MAX: u32 = 64;

/// Umbral de cambio en ρ para evitar escrituras redundantes en `QuantizedPrecision`.
pub const QUANTIZED_RHO_WRITE_EPS: f32 = 1e-5;

// ── Muestreo de paleta (`palette_gen`) — tuning cromático, no física de simulación ──

/// Fracción mínima de `REFERENCE_DENSITY` en el eje de muestreo (índice bajo).
pub const PALETTE_SAMPLE_DENSITY_MIN_FRAC: f32 = 0.15;
/// Fracción que escala con `t` hasta `REFERENCE_DENSITY` al índice alto.
pub const PALETTE_SAMPLE_DENSITY_RANGE_FRAC: f32 = 0.85;

pub const PALETTE_SAMPLE_TEMP_BASE: f32 = 40.0;
pub const PALETTE_SAMPLE_TEMP_SPAN: f32 = 220.0;

/// Mezcla emisión → canal alpha del texel de paleta (proxy de “brillo perceptual”).
pub const PALETTE_ALPHA_EMISSION_WEIGHT: f32 = 0.35;
