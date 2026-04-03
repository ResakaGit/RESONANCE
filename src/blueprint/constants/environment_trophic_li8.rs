// ── LI8 / EET1: sandbox ambiental + contratos tróficos ──
/// Ganancia mínima de intake ambiental (evita apagar totalmente la señal por entorno).
pub const ENV_INTAKE_GAIN_FLOOR: f32 = 0.1;
/// Peso de densidad de comida en intake ambiental.
pub const ENV_INTAKE_FOOD_WEIGHT: f32 = 0.75;
/// Peso de densidad de medio (agua/aire/suelo) en intake ambiental.
pub const ENV_INTAKE_MEDIUM_WEIGHT: f32 = 0.25;
/// Escala de penalización de mantenimiento por temperatura fuera de zona templada.
pub const ENV_MAINT_TEMPERATURE_SCALE: f32 = 0.8;
/// Escala de penalización de mantenimiento por presión de predación.
pub const ENV_MAINT_PREDATION_SCALE: f32 = 0.6;
/// Escala de estrés por predación.
pub const ENV_STRESS_PREDATION_SCALE: f32 = 0.7;
/// Escala de estrés por densidad del medio.
pub const ENV_STRESS_MEDIUM_SCALE: f32 = 0.3;
/// Peso del término de competencia en neto trófico.
pub const TROPHIC_COMPETITION_PENALTY_SCALE: f32 = 0.8;
/// Coeficiente de intake por clase trófica.
pub const TROPHIC_INTAKE_FACTOR: [f32; 5] = [1.0, 0.95, 0.85, 0.75, 0.65];
/// Penalty térmico de asimilación (0 = sin penalty).
pub const TROPHIC_ASSIMILATION_TEMP_PENALTY: f32 = 0.5;
/// Baseline de costo de mantenimiento.
pub const TROPHIC_MAINTENANCE_BASE: f32 = 0.25;
/// Peso de movilidad en costo de mantenimiento.
pub const TROPHIC_MAINTENANCE_MOBILITY_WEIGHT: f32 = 0.35;
/// Peso de armadura en costo de mantenimiento.
pub const TROPHIC_MAINTENANCE_ARMOR_WEIGHT: f32 = 0.30;
/// Peso de predación en costo de mantenimiento.
pub const TROPHIC_MAINTENANCE_PREDATION_WEIGHT: f32 = 0.20;
/// Peso de densidad de medio en costo de mantenimiento.
pub const TROPHIC_MAINTENANCE_MEDIUM_WEIGHT: f32 = 0.15;
/// Referencia de qe para la viabilidad base de órganos.
pub const ORGAN_BASE_VIABILITY_QE_REFERENCE: f32 = 600.0;
/// Peso del término energético en viabilidad base.
pub const ORGAN_BASE_VIABILITY_QE_WEIGHT: f32 = 0.7;
/// Peso del término de eficiencia metabólica en viabilidad base.
pub const ORGAN_BASE_VIABILITY_EFFICIENCY_WEIGHT: f32 = 0.3;
/// Presupuesto máximo de evaluaciones surrogate por frame (LI9).
pub const MAX_EVOLUTION_EVALS_PER_FRAME: u32 = 96;
/// Capacidad de cache surrogate para fitness aproximado (LI9).
pub const EVOLUTION_SURROGATE_CACHE_CAPACITY: usize = 2048;
/// Iteraciones máximas para convergencia surrogate en fixtures simples (LI9).
pub const EVOLUTION_SURROGATE_MAX_ITERATIONS: u32 = 8;
/// Peso del costo de mantenimiento en score agregado LI9.
pub const EVOLUTION_MAINTENANCE_WEIGHT: f32 = 0.5;
/// Bit de capacidad reproductiva en `role_mask` para LI9.
pub const EVOLUTION_ROLE_REPRODUCE_BIT: u8 = 6;
/// Factor de reducción de alimento en escenario "scarce" (LI9).
pub const EVOLUTION_SCARCE_FOOD_FACTOR: f32 = 0.5;
/// Competencia en escenario "scarce" (LI9).
pub const EVOLUTION_SCARCE_COMPETITION: f32 = 0.7;
/// Incremento de presión de predación en escenario "hostile" (LI9).
pub const EVOLUTION_HOSTILE_PREDATION_DELTA: f32 = 0.2;
/// Incremento de temperatura en escenario "hostile" (LI9).
pub const EVOLUTION_HOSTILE_TEMPERATURE_DELTA: f32 = 0.2;
/// Competencia en escenario "hostile" (LI9).
pub const EVOLUTION_HOSTILE_COMPETITION: f32 = 0.8;
