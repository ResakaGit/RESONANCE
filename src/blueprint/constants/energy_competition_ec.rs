//! Constantes del track Energy Competition (EC-1E … EC-7B).
//! Pool model, extraction thresholds, conservation tolerances,
//! competitive dynamics, scale-invariant fitness, fenotipo presets.

// --- Energy Competition: Pool ---
pub const DISSIPATION_RATE_MIN: f32 = 0.001;
pub const DISSIPATION_RATE_MAX: f32 = 0.5;
pub const DISSIPATION_RATE_DEFAULT: f32 = 0.01;
pub const POOL_CAPACITY_MIN: f32 = 1.0;

// --- AC-1: Metabolic Interference ---
/// Minimum extraction efficiency under maximum destructive interference.
/// A metabolic extractor always retains basal friction — never full lockout.
pub const METABOLIC_INTERFERENCE_FLOOR: f32 = 0.05;

// --- Energy Competition: Extraction ---
pub const EXTRACTION_EPSILON: f32 = 1e-6;
pub const REGULATED_AGGRESSIVE_MULT: f32 = 1.5;
pub const REGULATED_THROTTLE_MULT: f32 = 0.3;
pub const REGULATED_THRESHOLD_LOW_DEFAULT: f32 = 0.3;
pub const REGULATED_THRESHOLD_HIGH_DEFAULT: f32 = 0.7;
pub const AGGRESSION_FACTOR_DEFAULT: f32 = 0.5;
pub const DAMAGE_RATE_DEFAULT: f32 = 0.1;

// --- Energy Competition: Conservation ---
pub const POOL_CONSERVATION_EPSILON: f32 = 1e-3;

// --- Energy Competition: Composition ---
/// Número máximo de modificadores en un ExtractionProfile (DOD: max 4).
pub const MAX_EXTRACTION_MODIFIERS: usize = 4;

// --- EC-5: Competitive Dynamics ---
/// Ticks de advertencia antes de colapso de pool. Umbral entre Stressed y Collapsing.
pub const COLLAPSE_WARNING_TICKS: u32 = 100;
/// Dimensión máxima de la matriz de competencia N×N (stack-allocated).
pub const MAX_COMPETITION_MATRIX: usize = 16;

// --- EC-7A: Scale-Invariant Fitness ---
/// Peso del bonus de complejidad estructural en fitness inferido.
pub const COMPLEXITY_FITNESS_WEIGHT: f32 = 0.1;
/// Techo del bonus de complejidad estructural (evita dominancia por complejidad pura).
pub const COMPLEXITY_CAP: f32 = 0.5;
/// Valor máximo del fitness inferido de un pool.
pub const FITNESS_MAX: f32 = 1.5;

/// Rate de blend por tick para convergencia de fitness en PoolParentLink.
pub const FITNESS_BLEND_RATE: f32 = 0.1;

// --- EC-7B: Competitive Regime Thresholds ---
/// Intensidad de competencia por debajo de la cual el régimen es Abundance.
pub const REGIME_ABUNDANCE_INTENSITY_THRESHOLD: f32 = 0.3;
/// Intensidad de competencia por encima de la cual el régimen es Dominance.
pub const REGIME_DOMINANCE_INTENSITY_THRESHOLD: f32 = 0.6;

// --- EC-3F: Fenotipo Presets ---
/// Umbral de pool_ratio para activar stress response en opportunistic_generalist.
pub const OPPORTUNISTIC_STRESS_THRESHOLD: f32 = 0.4;
/// Umbral de stress response para resilient_homeostatic.
pub const HOMEOSTATIC_STRESS_THRESHOLD: f32 = 0.3;
/// Multiplicador de extracción bajo estrés en resilient_homeostatic.
pub const HOMEOSTATIC_STRESS_MULT: f32 = 1.2;
/// Factor de escala de extracción para apex_predator (domina en abundancia).
pub const APEX_PREDATOR_SCALE_FACTOR: f32 = 2.0;
