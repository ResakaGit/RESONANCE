//! Tuning del sistema `abiogenesis_system` (EA5).
//! Calibración de potencial / perfiles / banda Hz → [`crate::blueprint::constants`].
//! Fixtures compartidos de tests (`qe` celda, `water_norm`) → `ABIOGENESIS_TEST_*` en blueprint.

use crate::blueprint::equations::derived_thresholds::{DISSIPATION_GAS, DISSIPATION_SOLID};

/// Presupuesto máximo de spawns por tick de simulación.
pub const MAX_ABIOGENESIS_PER_FRAME: usize = 2;

/// Celdas del campo visitadas por tick (tapa coste CPU).
pub const SCAN_BUDGET_CELLS: usize = 64;

// ── Valores iniciales del `EntityBuilder` (alineado a EA5 / demo flora) ──
#[allow(dead_code)]
pub const EMERGENT_INITIAL_RADIUS: f32 = 0.05;
/// Disipación flora emergente = fase sólida (Axiom 4).
/// Emergent flora dissipation = solid-phase (Axiom 4).
#[allow(dead_code)]
pub const EMERGENT_FLOW_DISSIPATION: f32 = DISSIPATION_SOLID; // 0.005
#[allow(dead_code)]
pub const EMERGENT_MATTER_THERMAL_CONDUCTIVITY: f32 = 0.05;
pub const EMERGENT_GROWTH_BIOMASS: f32 = 0.05;
pub const EMERGENT_GROWTH_LIMITER: u8 = 0;
pub const EMERGENT_GROWTH_EFFICIENCY: f32 = 0.8;

/// Escalas de nutriente respecto a `water_norm` de la celda.
pub const EMERGENT_NUTRIENT_CARBON_SCALE: f32 = 0.3;
pub const EMERGENT_NUTRIENT_NITROGEN_SCALE: f32 = 0.2;
pub const EMERGENT_NUTRIENT_PHOSPHORUS_SCALE: f32 = 0.15;
pub const EMERGENT_NUTRIENT_WATER_SCALE: f32 = 0.5;

// ── Fauna emergent defaults (EA5-F) ─────────────────────────────────────────
pub const FAUNA_EMERGENT_INITIAL_RADIUS: f32 = 0.35;
/// Disipación fauna emergente = 1.25× gas: metabolismo animal joven.
/// Emergent fauna dissipation = 1.25× gas: young animal metabolism.
pub const FAUNA_EMERGENT_FLOW_DISSIPATION: f32 = DISSIPATION_GAS * 1.25; // 0.10
pub const FAUNA_EMERGENT_MATTER_THERMAL_CONDUCTIVITY: f32 = 0.15;
pub const FAUNA_EMERGENT_BUF_MAX: f32 = 400.0;
pub const FAUNA_EMERGENT_IN_VALVE: f32 = 0.6;
pub const FAUNA_EMERGENT_OUT_VALVE: f32 = 0.5;
pub const FAUNA_EMERGENT_BUF_INIT: f32 = 100.0;
pub const FAUNA_EMERGENT_ADAPT_RATE: f32 = 4.0;
pub const FAUNA_EMERGENT_QE_COST_HZ: f32 = 0.15;
pub const FAUNA_EMERGENT_STAB_BAND: f32 = 6.0;
