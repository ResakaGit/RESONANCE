/// D8: Morphological Adaptation — Bergmann, Allen, Wolff constants.

/// Bergmann: growth scale per unit thermal stress (dimensionless multiplier).
pub const BERGMANN_GROWTH_SCALE: f32 = 0.15;

/// Allen: limb/branching reduction scale per unit cold stress.
pub const ALLEN_LIMB_REDUCTION_SCALE: f32 = 0.10;

/// Wolff: adaptation rate for bond_energy under sustained mechanical load (qe/tick).
pub const WOLFF_ADAPTATION_RATE: f32 = 0.02;

/// Homeostatic load baseline — deviation from this drives Wolff adaptation.
pub const WOLFF_HOMEOSTATIC_LOAD: f32 = 0.3;

/// Max delta per tick on InferenceProfile biases (prevents jumps).
pub const MORPHO_ADAPTATION_RATE: f32 = 0.005;

/// Minimum InferenceProfile delta to trigger organ rebalance.
pub const MORPHO_REBALANCE_THRESHOLD: f32 = 0.05;

/// Default target temperature for Bergmann/Allen (normalized, ~body temp equivalent).
pub const MORPHO_TARGET_TEMPERATURE: f32 = 300.0;

/// Division guard epsilon for temperature ratios.
pub const MORPHO_TEMP_EPSILON: f32 = 1e-6;

/// Speed threshold below which entity counts as sedentary for Wolff.
pub const WOLFF_SEDENTARY_SPEED: f32 = 0.1;

/// Bond energy floor — Wolff never reduces below this.
pub const WOLFF_BOND_ENERGY_MIN: f32 = 10.0;
