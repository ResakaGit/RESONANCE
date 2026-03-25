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
