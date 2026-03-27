//! Visual calibration constants — rendering tuning, NOT axiom-derived physics.
//!
//! These values control how the energy field renders visually. They are calibrated
//! by observing simulation output, not computed from the 4 fundamentals.
//!
//! The axiomatically derived equivalents exist in `derived_thresholds.rs` but
//! produce different visual results (different scale, saturation, diffusion speed).
//! These values represent the visual artist's choice, not the physicist's derivation.
//!
//! If you change a fundamental constant (e.g., DISSIPATION_SOLID), re-evaluate
//! whether these calibrations still produce acceptable visuals.

/// Reference density for visual scale derivation.
/// Axiom-derived equivalent: `liquid_density_threshold()` ≈ 127.
/// Calibrated lower (50) for wider visual dynamic range.
pub const VISUAL_REFERENCE_DENSITY: f32 = 50.0;

/// High density class boundary for visual classification.
/// Axiom-derived equivalent: `liquid_density_threshold()` ≈ 127.
/// Calibrated lower (100) for earlier visual transition.
pub const VISUAL_DENSITY_HIGH: f32 = 100.0;

/// Purity threshold for pure vs compound materialization.
/// Axiom-derived equivalent: `sense_coherence_min() × 2` ≈ 0.40.
/// Calibrated higher (0.7) for stricter visual purity — fewer "pure" tiles.
pub const VISUAL_PURITY_THRESHOLD: f32 = 0.7;

/// Field diffusion conductivity between neighbor cells.
/// Axiom-derived equivalent: `DISSIPATION_LIQUID` = 0.02.
/// Calibrated higher (0.1) for visible field spread during warmup.
pub const VISUAL_CONDUCTIVITY_SPREAD: f32 = 0.1;

/// Bond energy for materialized terrain spawns.
/// Axiom-derived equivalent: `1/DISSIPATION_SOLID` = 200.
/// Calibrated higher (1000) for robust terrain that resists phase transitions.
pub const VISUAL_SPAWN_BOND_ENERGY: f32 = 1000.0;

/// Thermal conductivity for materialized terrain spawns.
/// Axiom-derived equivalent: `DISSIPATION_SOLID × DENSITY_SCALE` = 0.1.
/// Calibrated higher (0.3) for visible thermal exchange between tiles.
pub const VISUAL_SPAWN_THERMAL_CONDUCTIVITY: f32 = 0.3;
