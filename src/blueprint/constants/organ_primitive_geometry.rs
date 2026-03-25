// ── Organ primitive geometry base ratios (energy-driven sizing) ──

/// Base length-to-radius ratio for PetalFan primitive.
pub const PETAL_FAN_LENGTH_RATIO: f32 = 3.0;
/// Base width-to-radius ratio for PetalFan primitive.
pub const PETAL_FAN_WIDTH_RATIO: f32 = 1.8;
/// Base length-to-radius ratio for FlatSurface primitive (Leaf).
pub const FLAT_SURFACE_LENGTH_RATIO: f32 = 2.2;
/// Base width-to-radius ratio for FlatSurface primitive (Leaf).
pub const FLAT_SURFACE_WIDTH_RATIO: f32 = 1.2;
/// Base length-to-radius ratio for Tube primitive.
pub const TUBE_LENGTH_RATIO: f32 = 2.2;
/// Base radius scale for Tube primitive.
pub const TUBE_RADIUS_SCALE: f32 = 0.35;
/// Base radius scale for Bulb primitive.
pub const BULB_RADIUS_SCALE: f32 = 0.9;

/// Biomass contribution scale for organ energy-driven sizing.
pub const ORGAN_ENERGY_BIOMASS_SCALE: f32 = 1.0;
/// Minimum organ scale from energy equation (ensures visibility).
pub const ORGAN_ENERGY_SCALE_MIN: f32 = 0.15;
/// Maximum organ scale amplification from biomass.
pub const ORGAN_ENERGY_SCALE_MAX: f32 = 3.5;
/// Floor fraction of qe_norm contribution (prevents zero-energy organs from vanishing).
pub const ORGAN_ENERGY_QE_FLOOR: f32 = 0.3;
