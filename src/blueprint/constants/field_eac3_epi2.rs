// ── EAC3 / EPI2 — RGB base de campo (almanaque × pureza × compuesto) ──
/// Canal sRGB del gris neutro (sin banda, pureza 0, no finitos). `worldgen::constants::VISUAL_NEUTRAL_GRAY_CHANNEL` debe reexportar este valor.
pub const FIELD_VISUAL_NEUTRAL_GRAY_CHANNEL: f32 = 0.5;
/// Constructivo: sesgo al primario en mezcla compuesta (misma semántica que `COMPOUND_BLEND_CONSTRUCTIVE_INTERFERENCE_WEIGHT` en worldgen).
pub const FIELD_COMPOUND_BLEND_CONSTRUCTIVE_INTERFERENCE_WEIGHT: f32 = 0.75;
/// Base de mezcla primario/secundario en interferencia destructiva.
pub const FIELD_COMPOUND_BLEND_DESTRUCTIVE_BASE: f32 = 0.5;
/// Alpha opaco: primarios de almanaque y neutro en mezcla compuesta (EPI2 expone RGB sin alpha en celda).
pub const FIELD_VISUAL_OPAQUE_ALPHA: f32 = 1.0;
/// Rango de `interference` en mezcla compuesta (salida de `cos`, constructiva/destructiva).
pub const FIELD_COMPOUND_INTERFERENCE_CLAMP_MIN: f32 = -1.0;
pub const FIELD_COMPOUND_INTERFERENCE_CLAMP_MAX: f32 = 1.0;

/// Umbrales solo para tests EAC3 (`equations`); alineado en magnitud con `worldgen::field_sample_test_thresholds::MIN_BAND_L1`.
#[cfg(test)]
pub mod eac3_test_thresholds {
    pub const MIN_RGB_L1_DISTINCT: f32 = 0.1;
}

