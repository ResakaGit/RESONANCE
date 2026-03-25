// ── Visual: color cuantizado + fenología (EA8 / bridge) ──
// (Bloque contiguo al sprint EA6 en el monolito histórico; dominio propio para tuning visual.)

/// ρ mínimo en banda Far para color cuantizado (debe ser > 0). Ver `precision_rho_from_lod_distance`.
pub const QUANTIZED_COLOR_RHO_MIN: f32 = 0.12;

/// Piso numérico para clamp de ρ antes de `ceil(n·ρ)` (paridad CPU / WGSL).
pub const QUANTIZED_COLOR_RHO_CLAMP_FLOOR: f32 = 1e-6;

/// Span mínimo Near→Mid al interpolar ρ (evita división por cero si constantes LOD están mal configuradas).
pub const QUANTIZED_LOD_RHO_SPAN_EPS: f32 = 1e-6;

/// Referencia única de `qe` para normalización visual (bridge 3D, payload cuantizado, tuning MOBA).
pub const VISUAL_QE_REFERENCE: f32 = 600.0;

/// Histeresis (Δ fase) por defecto para refrescar color fenológico en `EnergyVisual` (EA8).
pub const PHENOLOGY_DEFAULT_EPSILON: f32 = 0.02;

/// Techo de normalización del proxy de madurez sin `GrowthBudget` (`EnergyCell.accumulated_qe` / biomasa).
pub const PHENOLOGY_DEFAULT_GROWTH_NORM_CEILING: f32 = 400.0;

/// Suma de pesos fenológicos por debajo de esto ⇒ fase 0 (desactivado efectivo).
pub const PHENOLOGY_WEIGHT_SUM_EPSILON: f32 = 1e-8;
