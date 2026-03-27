pub mod systems;

pub mod cell_field_snapshot;
pub mod field_visual_sample;
pub mod archetypes;
pub mod constants;
pub mod contracts;
pub mod field_grid;
pub mod lod;
pub mod map_config;
pub(crate) mod materialization_rules;
pub mod nucleus;
pub mod nutrient_field;
pub mod organ_inference;
pub(crate) mod propagation;
pub mod propagation_mode;
pub mod shape_inference;
pub mod visual_calibration;
pub(crate) mod visual_derivation;

pub use archetypes::{
    DensityClass, ElementBand, PURE_ELEMENT_STATE_ARCHETYPES, WorldArchetype,
    archetype_from_signature,
};
pub use constants::{
    DENSITY_HIGH_THRESHOLD, DENSITY_LOW_THRESHOLD, FIELD_CELL_SIZE, FIELD_CONDUCTIVITY_SPREAD,
    FIELD_DECAY_RATE, MAX_FREQUENCY_CONTRIBUTIONS, MIN_CONTRIBUTION_INTENSITY,
    MIN_MATERIALIZATION_QE, PURITY_THRESHOLD, REFERENCE_DENSITY, WARMUP_TICKS,
};
pub use contracts::{
    BoundaryVisual, EnergyCell, EnergyVisual, FrequencyContribution, MaterializationResult,
    Materialized, PendingEnergyVisualRebuild, PhenologyPhaseCache, PhenologyVisualParams,
};
pub use field_grid::EnergyFieldGrid;
pub use map_config::{
    ActiveMapName, AmbientPressureConfig, MapConfig, ValidationError, ROUND_WORLD_ROSA_MAP_SLUG,
    active_map_slug_from_env, load_default_map_asset, load_map_config_from_env,
    load_map_config_from_env_result, resolve_nuclei_for_spawn, validate_map_config,
};
pub use materialization_rules::{
    boundary_marker_cache_tag, boundary_visual_from_marker, boundary_world_archetype,
    materialize_cell_at_time, materialize_cell_at_time_with_boundary,
};
pub use nucleus::{EnergyNucleus, NucleusReservoir, PropagationDecay};
pub use propagation_mode::{
    NucleusEmissionState, PropagationMode,
    diffuse_propagation_system, insert_nucleus_emission_state_system,
};
pub use nutrient_field::{
    COMPETITION_BASE_DRAIN_PER_EXTRA_COMPETITOR_QE, NUTRIENT_DEPLETION_RATE,
    NUTRIENT_REGEN_PER_TICK, NUTRIENT_RETURN_RATE, NUTRIENT_WRITE_EPS, NutrientCell,
    NutrientFieldGrid, apply_nucleus_bias, nutrient_bias_from_frequency,
    seed_nutrient_field_from_nuclei_system, sync_nutrient_field_len_system,
};
/// API pública para benches / integración: el módulo `propagation` sigue siendo `pub(crate)`.
pub use propagation::{field_dissipation, resolve_dominant_frequency};
pub use visual_derivation::{
    VisualProperties, apply_archetype_visual_profile, boundary_transition_emission_extra,
    color_lerp, compound_color_blend, derive_all, derive_color, derive_color_compound,
    derive_color_phenology, derive_emission, derive_opacity, derive_scale,
    energy_visual_boundary_flat_color, materialized_tile_spatial_density,
    neutral_visual_linear_rgb, visual_proxy_temperature, zone_class_display_color,
};

pub use shape_inference::{
    ShapeInferenceFrameState, ShapeInferred, growth_morphology_system,
    reset_shape_inference_frame_system, shape_color_inference_system,
};
pub use organ_inference::{AttachmentZone, OrganAttachment, build_organ_mesh, organ_attachment_points, organ_orientation};
pub use systems::materialization::{NucleusFreqTrack, SeasonTransition};
pub use cell_field_snapshot::{
    CellFieldSnapshot, CellFieldSnapshotCache, cell_field_snapshot_from_energy_cell,
    cell_field_snapshot_read, cell_field_snapshot_sync_system, frequency_contributions_fingerprint,
};
pub use cell_field_snapshot::gpu_layout::{
    CELL_FIELD_SNAPSHOT_GPU_SCHEMA_VERSION, CELL_FIELD_SNAPSHOT_WGSL_PATH, GpuCellFieldPacked,
    GpuCellFieldSnapshotHeader, GPU_CELL_FIELD_ROW_BYTES, GPU_SNAPSHOT_HEADER_BYTES,
    cell_field_snapshot_to_gpu_packed, gpu_cell_field_snapshot_bytes,
    gpu_packed_rows_from_cache_entries, initial_gpu_cell_field_snapshot_header,
};
// EPI2: mezcla fenológica en lineal = `linear_rgb_lerp` (+ `field_visual_mix_unit` internamente).
pub use crate::blueprint::equations::{field_visual_mix_unit, linear_rgb_lerp};
pub use field_visual_sample::{
    field_linear_rgb_from_cell, field_linear_rgb_from_cell_inputs,
    gf1_field_linear_rgb_qe_at_position, linear_rgb_from_derive_color,
};
pub use systems::performance::{
    MatBudgetCounters, MatCacheStats, MaterializationCellCache, PropagationWriteBudget,
    VisualDerivationFrameState, WorldgenLodContext, WorldgenPerfSettings,
};
pub use systems::phenology_visual::phenology_visual_apply_system;
pub use systems::startup::{StartupNucleus, WorldgenWarmupConfig};
