//! Sustrato topológico: relieve procedural alineado al `EnergyFieldGrid` (docs/design/TOPOLOGY.md).

pub mod config;
pub mod constants;
pub mod contracts;
pub mod functions;
pub mod generators;
pub mod mutations;
pub mod terrain_field;
pub mod terrain_mesher;

pub use config::{
    ModulationParams, TerrainConfig, TerrainConfigAssetState, TerrainConfigRonLoader,
    TerrainConfigRuntime, init_terrain_config_system, sanitize_terrain_config,
    terrain_config_loader_system, terrain_config_wants_reload,
};
pub use constants::{
    ALTITUDE_EMISSION_SCALE, ALTITUDE_MAX_DEFAULT, ALTITUDE_MIN_DEFAULT, CLIFF_SLOPE_THRESHOLD,
    DRAINAGE_DRY, DRAINAGE_MOIST, DRAINAGE_WET, REFERENCE_ALTITUDE, RIVER_THRESHOLD,
    SLOPE_DIFFUSION_SCALE,
};
pub use contracts::{DrainageClass, TerrainSample, TerrainType};
pub use functions::{
    modulate_decay, modulate_decay_with_params, modulate_diffusion, modulate_diffusion_with_params,
    modulate_emission, modulate_emission_with_params,
};
pub use generators::{
    ClassificationThresholds, ErosionParams, NoiseParams, classify_all, classify_terrain,
    compute_flow_accumulation, compute_flow_direction, derive_aspect, derive_slope,
    derive_slope_aspect, erode_hydraulic, fill_pits, generate_heightmap, normalize_heightmap,
};
pub use mutations::{
    DirtyRegion, TerrainMutation, TerrainMutationEvent, apply_mutation, rederive_region,
};
pub use terrain_field::TerrainField;
pub use terrain_mesher::{TerrainVisuals, generate_terrain_mesh};
