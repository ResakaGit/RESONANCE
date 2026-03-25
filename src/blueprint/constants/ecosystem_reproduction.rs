// ── Ecosistema: reproducción (EA6) — `equations::can_reproduce` / `mutate_bias` + tuning compartido con `simulation::reproduction` ──
/// Peso de `branching_bias` en el factor efectivo: `reproduction_radius_factor * (1.0 - branching_bias * BETA)`.
pub const REPRODUCTION_BRANCHING_BETA: f32 = 0.4;
/// Piso del factor efectivo antes de multiplicar `base_radius`.
pub const REPRODUCTION_EFFECTIVE_FACTOR_FLOOR: f32 = 1.5;
/// Sustituto si `value` no es finito en `mutate_bias`.
pub const MUTATE_BIAS_NONFINITE_VALUE_FALLBACK: f32 = 0.5;
/// Factor de umbral respecto a `base_radius` (mismo valor que sprint EA6; lo pasa el sistema a `can_reproduce`).
pub const REPRODUCTION_RADIUS_FACTOR: f32 = 3.0;
