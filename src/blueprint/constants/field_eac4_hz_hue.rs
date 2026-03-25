// ── EAC4 — Hz → matiz (capa visual opcional; ver docs/sprints/ELEMENT_ALMANAC_CANON/README.md + blueprint_blueprint_math §EAC) ──
/// Solo color RON (`ElementDef.color`): sin mezcla con espectro Hz. Debe coincidir con `serde` default en `ElementDef`.
pub const FIELD_EAC4_HZ_IDENTITY_WEIGHT_RON_ONLY: f32 = 1.0;
/// `hz_identity_weight >= 1 - este valor` → solo color RON (sin mezcla espectro Hz).
pub const FIELD_EAC4_IDENTITY_FULL_RON_EPS: f32 = 1e-5;
/// Valor V en HSV [0,1] para el arco iris de juego (saturación viene de `purity` saneada).
pub const FIELD_EAC4_HZ_SPECTRUM_VALUE: f32 = 1.0;
/// Saturación HSV del arco en rama híbrida dentro de `field_linear_rgb_from_hz_purity` (pureza de celda aplica después).
pub const FIELD_EAC4_HYBRID_SPECTRUM_PURITY: f32 = 1.0;

#[cfg(test)]
pub mod eac4_test_thresholds {
    /// ε L1 RGB: `w_identity = 1` debe coincidir con `def_linear_rgb` (mezcla Hz desactivada).
    pub const MAX_L1_VS_DEF_RGB: f32 = 1e-4;
}

