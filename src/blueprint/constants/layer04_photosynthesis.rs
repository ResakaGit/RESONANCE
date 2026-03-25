// ── Capa 4: Fotosíntesis (TL4) ──
/// Temperatura óptima normalizada para máximo rendimiento fotosintético.
pub const PHOTO_OPTIMAL_TEMP_NORM: f32 = 0.4;

/// Dispersión de la campana gaussiana de eficiencia térmica.
pub const PHOTO_TEMP_SIGMA: f32 = 0.25;

/// Escala base de conversión fotón -> qe por segundo.
pub const PHOTO_YIELD_SCALE: f32 = 0.05;

/// Bonus aditivo de biomasa por irradiancia absorbida en `growth_budget_system`.
pub const PHOTO_GROWTH_BONUS: f32 = 0.02;

/// Tope de bonus fotosintético para que Liebig siga siendo el limitante principal.
pub const PHOTO_GROWTH_BONUS_CAP: f32 = 0.5;

/// Decaimiento espacial de irradiancia emitida por núcleos Lux.
pub const IRRADIANCE_LUX_DECAY: f32 = 0.08;

/// Presupuesto máximo de entidades con update de irradiancia por frame.
pub const MAX_IRRADIANCE_PER_FRAME: u32 = 128;

/// Referencia para normalizar temperatura equivalente a [0, 1] en fotosíntesis.
pub const PHOTO_TEMP_NORM_REFERENCE: f32 = 1000.0;

/// Escala de consumo de agua por qe producido por fotosíntesis.
pub const PHOTO_WATER_CONSUMPTION_PER_QE: f32 = 0.01;

/// Umbral mínimo para considerar irradiancia efectiva en Capa 4.
pub const IRRADIANCE_MIN_EFFECTIVE: f32 = 1e-6;

/// Tope de densidad de fotones acumulada por entidad/tick (estabilidad numérica).
pub const PHOTO_MAX_PHOTON_DENSITY: f32 = 1_000_000.0;

/// Banda frecuencial Lux [Hz] usada para fuentes de luz fotosintética.
pub const LUX_BAND_MIN_HZ: f32 = 900.0;
pub const LUX_BAND_MAX_HZ: f32 = 1100.0;

