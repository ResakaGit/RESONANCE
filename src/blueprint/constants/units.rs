//! Unidades internas del motor de simulación Resonance.
//! Tabla de referencia para conversiones y rangos válidos.
//!
//! Fuentes:
//! - `qe`: definición en `layers/energy.rs` (BaseEnergy). Escalar adimensional ≥ 0.
//! - `tick`: `Time<Fixed>` / `SimulationTickPlugin`; por defecto 64 Hz (≈ 15.625 ms).
//! - `TEMPERATURE_RANGE`: normalizada ±1 sobre `T_reference` (layer04 + thermal_transfer).
//! - `VISCOSITY_*`: valores SI orientativos (agua 20 °C ≈ 1000 mPa·s, aire ≈ 1.2 mPa·s).
//! - `MAX_EXTRACTION_RATIO`: contrato de conservación EC-1A/EC-4B.
//! - `CONSERVATION_ERROR_TOLERANCE`: igual a `POOL_CONSERVATION_EPSILON` (energy_competition_ec.rs).
//! - `QE_DEAD_THRESHOLD`: igual a `QE_MIN_EXISTENCE` (simulation_defaults.rs).

/// Unidad de energía fundamental. Escalar adimensional normalizado ≥ 0.
/// Rango típico de entidades: [0.0, 10_000.0].
pub const QE_UNIT: &str = "qe (quantum energy)";

/// Unidad de tiempo: un tick de FixedUpdate. Por defecto ≈ 1/64 segundos.
pub const TICK_UNIT: &str = "tick";

/// Rango de temperatura normalizada: [-1.0, 1.0] relativo a T_reference.
pub const TEMPERATURE_RANGE: (f32, f32) = (-1.0, 1.0);

/// Viscosidad del medio (unidades SI orientativas). Agua ≈ 1000 mPa·s.
pub const VISCOSITY_WATER: f32 = 1000.0;
/// Viscosidad del medio (unidades SI orientativas). Aire ≈ 1.2 mPa·s.
pub const VISCOSITY_AIR: f32 = 1.2;

/// Ratio máximo de extracción por tick: ningún hijo puede extraer más del pool completo.
pub const MAX_EXTRACTION_RATIO: f32 = 1.0;

/// Tolerancia de error de conservación por tick.
/// Coincide con `POOL_CONSERVATION_EPSILON` (energy_competition_ec.rs).
pub const CONSERVATION_ERROR_TOLERANCE: f32 = 1e-3;

/// Umbral de energía mínima: entidades por debajo se consideran "vacías".
/// Coincide con `QE_MIN_EXISTENCE` (simulation_defaults.rs).
pub const QE_DEAD_THRESHOLD: f32 = 1e-6;

// DETERMINISMO: el motor es determinista por diseño.
// - Sin RNG en código de producción.
// - El orden de extracción es topológico por Entity index.
// - Tests en tests/r2_determinism.rs verifican esto.
