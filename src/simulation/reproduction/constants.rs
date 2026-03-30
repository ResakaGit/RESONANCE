//! Tuning EA6 — alinear con `blueprint::constants` y valores numéricos de `abiogenesis/constants.rs` (Flora).

pub use crate::blueprint::constants::ABIOGENESIS_FLORA_ELEMENT_SYMBOL as FLORA_ELEMENT_SYMBOL;
pub use crate::blueprint::constants::REPRODUCTION_RADIUS_FACTOR;

/// Distancia de dispersión de la semilla en el plano de simulación (XZ).
pub const SEED_DISPERSAL_DISTANCE: f32 = 3.0;
/// Ticks de cooldown entre reproducciones por entidad.
pub const REPRODUCTION_COOLDOWN_TICKS: u32 = 120;
/// Máximo de semillas emitidas por tick (presupuesto global).
pub const MAX_REPRODUCTIONS_PER_FRAME: usize = 2;
/// Amplitud máxima de mutación en sesgos del perfil inferido.
pub const MUTATION_MAX_DRIFT: f32 = 0.1;
/// Fracción del `qe` parental transferida a la semilla (`EnergyOps::drain`).
pub const SEED_ENERGY_FRACTION: f32 = 0.3;

/// Paso angular tipo golden angle por índice de entidad (dispersión determinista).
pub const SEED_DISPERSAL_ANGLE_STEP: f32 = 2.399963;
/// Escala del índice en `sin` para drift de mutación.
pub const REPRODUCTION_MUTATION_INDEX_SCALE: f32 = 0.1;
/// Coeficiente del drift aplicado a `mobility_bias` (same direction as growth drift).
pub const REPRODUCTION_MUTATION_MOBILITY_SCALE: f32 = 0.4;
/// Coeficiente del drift aplicado a `branching_bias` (opuesto al drift principal).
pub const REPRODUCTION_MUTATION_BRANCHING_SCALE: f32 = 0.5;
/// Coeficiente del drift aplicado a `resilience`.
pub const REPRODUCTION_MUTATION_RESILIENCE_SCALE: f32 = 0.3;

pub const SEED_ENTITY_NAME: &str = "flora_seed";
pub const SEED_INITIAL_RADIUS: f32 = 0.08;

/// Bond fijo “semilla dura” (EA6); abiogénesis usa bond dinámico por celda.
pub const SEED_MATTER_BOND_EB: f32 = 800.0;

pub const SEED_GROWTH_BIOMASS: f32 = 0.12;
pub const SEED_GROWTH_LIMITER: u8 = 0;
/// Mismo valor que `abiogenesis::constants::EMERGENT_GROWTH_EFFICIENCY`.
pub const SEED_GROWTH_EFFICIENCY: f32 = 0.8;
/// Mismo valor que `abiogenesis::constants::EMERGENT_MATTER_THERMAL_CONDUCTIVITY`.
pub const SEED_MATTER_THERMAL_CONDUCTIVITY: f32 = 0.05;
/// Mismo valor que `abiogenesis::constants::EMERGENT_FLOW_DISSIPATION`.
pub const SEED_FLOW_DISSIPATION: f32 = 0.005;

pub const SEED_NUTRIENT_CARBON: f32 = 32.0;
pub const SEED_NUTRIENT_NITROGEN: f32 = 24.0;
pub const SEED_NUTRIENT_PHOSPHORUS: f32 = 18.0;
pub const SEED_NUTRIENT_WATER: f32 = 48.0;

// ── Fauna reproduction (EV-1) ───────────────────────────────────────────────
/// Minimum qe for fauna to reproduce (higher than flora — animals are costlier).
pub const FAUNA_REPRODUCTION_QE_MIN: f32 = 200.0;
/// Fraction of parent qe transferred to fauna offspring.
pub const FAUNA_SEED_ENERGY_FRACTION: f32 = 0.25;
/// Initial radius for fauna offspring (smaller than adult).
pub const FAUNA_OFFSPRING_INITIAL_RADIUS: f32 = 0.2;
/// Base frequency offset for fauna offspring (Hz).
pub const FAUNA_OFFSPRING_FREQ_BASE: f32 = 75.0;
/// Frequency scale from parent mobility_bias (Hz).
pub const FAUNA_OFFSPRING_FREQ_SCALE: f32 = 400.0;
/// Flow dissipation rate for fauna offspring.
pub const FAUNA_OFFSPRING_FLOW_DISSIPATION: f32 = 0.08;
/// Bond energy for fauna offspring (Solid matter).
pub const FAUNA_OFFSPRING_BOND_EB: f32 = 1000.0;
/// Thermal conductivity for fauna offspring.
pub const FAUNA_OFFSPRING_THERMAL_CONDUCTIVITY: f32 = 1.5;
/// Motor buffer max for fauna offspring.
pub const FAUNA_OFFSPRING_MOTOR_BUF_MAX: f32 = 400.0;
/// Motor input valve for fauna offspring.
pub const FAUNA_OFFSPRING_MOTOR_IN_VALVE: f32 = 0.6;
/// Motor output valve for fauna offspring.
pub const FAUNA_OFFSPRING_MOTOR_OUT_VALVE: f32 = 0.5;
/// Fraction of drained qe used for motor buffer init.
pub const FAUNA_OFFSPRING_MOTOR_BUF_INIT_FRACTION: f32 = 0.3;
/// Trophic intake rate for fauna offspring.
pub const FAUNA_OFFSPRING_TROPHIC_INTAKE: f32 = 12.0;
/// Initial satiation for fauna offspring.
pub const FAUNA_OFFSPRING_INITIAL_SATIATION: f32 = 0.5;
