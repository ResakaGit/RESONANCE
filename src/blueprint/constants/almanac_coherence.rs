// ── Almanaque: tolerancia tests coherencia RON ↔ const (EAC1) ──
/// Comparación Hz absoluta en `almanac_contract` (bandas O(10²) Hz en contenido actual).
pub const ALMANAC_COHERENCE_EPS_HZ: f32 = 1e-4;
/// Prefijo estable del error de símbolos duplicados (CI / grep).
pub const EAC1_ERR_DUPLICATE_SYMBOL_PREFIX: &str = "duplicate element symbol(s): ";

