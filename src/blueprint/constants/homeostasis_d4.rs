// ── D4: Homeostasis & Thermoregulation ──

/// Target body temperature for endotherms (~37C in internal units).
pub const ENDOTHERM_TARGET_TEMP: f32 = 310.0;
/// Ectotherm convergence rate toward ambient temperature per tick.
pub const ECTOTHERM_CONVERGENCE_RATE: f32 = 0.1;
/// Base insulation factor (dimensionless).
pub const INSULATION_BASE: f32 = 1.0;
/// Bonus insulation from shell/armor structures.
pub const INSULATION_ARMOR_BONUS: f32 = 0.5;
/// Minimum engine buffer fraction to spend on thermoregulation.
pub const THERMOREG_MIN_QE_FRACTION: f32 = 0.1;
/// Scale factor: AmbientPressure.delta_qe_constant to temperature offset.
pub const THERMOREG_DELTA_TO_TEMP_SCALE: f32 = 10.0;
