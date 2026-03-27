//! IWG — Inferred World Geometry: body plan + terrain visual constants.

/// Lateral spread multiplier for limb pairs relative to body radius.
pub const LIMB_SPREAD_RATIO: f32 = 1.2;

/// Z-axis spacing between successive limb pairs along the spine.
pub const LIMB_PAIR_Z_SPACING: f32 = 0.6;

/// Kleiber-style allometric exponent for organ scaling (sublinear).
pub const ALLOMETRIC_EXPONENT: f32 = 0.75;

/// Minimum allowed organ scale after allometric calculation.
pub const ORGAN_SCALE_MIN: f32 = 0.05;

/// Maximum allowed organ scale after allometric calculation.
pub const ORGAN_SCALE_MAX: f32 = 4.0;

/// Base scale per OrganRole variant (indexed by `OrganRole as usize`).
/// Order: Stem, Root, Core, Leaf, Petal, Sensory, Thorn, Shell, Fruit, Bud, Limb, Fin.
pub const ROLE_BASE_SCALE: [f32; 12] = [
    1.0, 0.6, 1.2, 0.5, 0.4, 0.3, 0.25, 0.8, 0.35, 0.2, 0.7, 0.5,
];

// --- Inferred World Geometry: Terrain Visuals ---

/// Base RGB per element band (indexed by band 0..7).
pub const TERRAIN_BAND_COLOR: [[f32; 3]; 8] = [
    [0.45, 0.38, 0.28], // Band 0: Terra
    [0.20, 0.45, 0.65], // Band 1: Aqua
    [0.65, 0.28, 0.15], // Band 2: Ignis
    [0.30, 0.55, 0.25], // Band 3: Flora
    [0.50, 0.50, 0.55], // Band 4: Aer
    [0.60, 0.55, 0.35], // Band 5: Lux
    [0.35, 0.30, 0.45], // Band 6: Umbra
    [0.55, 0.55, 0.50], // Band 7: Neutral
];

/// Slope above which shadow darkening is applied.
pub const SLOPE_SHADOW_THRESHOLD: f32 = 0.3;

/// Multiplicative darkening factor for steep slopes.
pub const SLOPE_SHADOW_FACTOR: f32 = 0.7;

/// Minimum brightness from qe normalization.
pub const QE_BRIGHTNESS_MIN: f32 = 0.5;

/// Maximum brightness from qe normalization.
pub const QE_BRIGHTNESS_MAX: f32 = 1.0;

/// Saturation per MatterState variant: Solid, Liquid, Gas, Plasma.
pub const STATE_SATURATION: [f32; 4] = [1.0, 0.7, 0.4, 0.3];

// --- Inferred World Geometry: Atmosphere ---

/// Peak directional light intensity [lux] when sun is directly overhead.
pub const SUN_BASE_INTENSITY: f32 = 20000.0;

/// Minimum directional light intensity [lux] at very low sun angles.
pub const SUN_MIN_INTENSITY: f32 = 500.0;

/// Fog start distance as ratio of world radius.
pub const FOG_START_RATIO: f32 = 0.6;

/// Fog end distance as ratio of world radius.
pub const FOG_END_RATIO: f32 = 1.2;

/// Minimum fog start distance [world units].
pub const FOG_MIN_START: f32 = 10.0;

/// Maximum fog end distance [world units].
pub const FOG_MAX_END: f32 = 200.0;

/// Bloom intensity per unit of average qe_norm.
pub const BLOOM_QE_SCALE: f32 = 0.2;

/// Hard ceiling for bloom intensity.
pub const BLOOM_MAX: f32 = 0.4;

/// Ticks between atmosphere inference updates.
pub const ATMOSPHERE_UPDATE_INTERVAL: u32 = 30;

/// Base ambient light intensity before canopy/sun modulation.
pub const AMBIENT_BASE_INTENSITY: f32 = 0.15;

/// Fraction of ambient occluded by full canopy density.
pub const AMBIENT_CANOPY_REDUCTION: f32 = 0.5;

// --- Inferred World Geometry: Water Surface ---

/// Y offset above the average liquid terrain height.
pub const WATER_SURFACE_OFFSET: f32 = 0.2;

/// Subdivision count for the water plane grid (vertices = (N+1)^2).
pub const WATER_SUBDIVISIONS: u32 = 8;

/// Minimum number of liquid cells required to generate a water surface.
pub const WATER_MIN_CELLS: u32 = 4;

/// Depth threshold below which water is considered shallow.
pub const WATER_SHALLOW_DEPTH: f32 = 0.5;

/// Depth threshold above which water is considered deep.
pub const WATER_DEEP_DEPTH: f32 = 2.0;

/// RGB color for shallow water.
pub const WATER_COLOR_SHALLOW: [f32; 3] = [0.3, 0.6, 0.8];

/// RGB color for medium-depth water.
pub const WATER_COLOR_MEDIUM: [f32; 3] = [0.1, 0.3, 0.6];

/// RGB color for deep water.
pub const WATER_COLOR_DEEP: [f32; 3] = [0.05, 0.15, 0.4];

// --- Inferred World Geometry: Atmosphere Sync ---

/// Sun rotation speed (radians per simulation tick).
pub const SUN_ROTATION_SPEED: f32 = 0.001;

/// Default latitude for sun angle (0 = equator, 1 = pole).
pub const DEFAULT_LATITUDE: f32 = 0.2;

/// Distance to place the directional light source from origin.
pub const SUN_PLACEMENT_DISTANCE: f32 = 100.0;

/// Density threshold fraction (cells above this % of max qe count as "dense").
pub const ATMOSPHERE_DENSITY_THRESHOLD_RATIO: f32 = 0.1;
