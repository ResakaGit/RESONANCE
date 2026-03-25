//! Eco-Boundaries: topología de fronteras y contexto ambiental derivado del `EnergyFieldGrid`.
//! No es una capa ortogonal; es un derivado cacheado (ver `docs/design/ECO_BOUNDARIES.md`).

pub mod boundary_detector;
pub mod boundary_field;
pub mod climate;
pub mod constants;
pub mod context_lookup;
pub mod contracts;
pub mod systems;
pub mod zone_classifier;

pub use boundary_detector::{
    NEIGHBOR_OFFSETS, compute_gradient_factor, detect_boundary, infer_transition_type,
};
pub use boundary_field::{
    EcoBoundaryField, aggregate_zone_class_contexts, aggregate_zone_contexts,
};
pub use climate::{
    ClimateAssetState, ClimateConfig, ClimateConfigLoader, ClimateState, Season, SeasonProfile,
    climate_config_hot_reload_system, climate_tick_system, init_climate_config_system,
    snapshot_for_tick, step_climate_state,
};
pub use constants::{
    ATMOSPHERE_CEILING_HEIGHT, BOUNDARY_RECOMPUTE_COOLDOWN, DENSITY_JUMP_RELATIVE_MIN,
    ELEMENT_GRADIENT_HZ_SPAN, ELEMENT_ZONE_HZ_BREAK, GAS_TRANSITION, IGNIS_DOMINANT_MAX_HZ,
    IGNIS_DOMINANT_MIN_HZ, LIQUID_TRANSITION, PHASE_GRADIENT_TEMP_SPAN, QE_MIN_EXISTENCE,
    SOLID_TRANSITION, SUBAQUATIC_DENSITY_THRESHOLD, SUBTERRANEAN_DENSITY_THRESHOLD,
    THERMAL_SHOCK_GRADIENT, THIN_ATMOSPHERE_DENSITY_MAX, VOID_QE_THRESHOLD,
};
pub use context_lookup::{
    ContextLookup, EcoPlayfieldMargin, apply_season_to_zone_context, cell_index_for_pos,
    context_at_inner, context_response_legacy_baseline, eco_field_aligned_with_grid,
    is_boundary_at_inner, is_cell_in_logical_void_margin, is_void_at_inner, void_context_response,
    zone_at_inner,
};
pub use contracts::{BoundaryMarker, ContextResponse, TransitionType, ZoneClass, ZoneContext};
pub use systems::eco_boundaries_system;
pub use zone_classifier::classify_cell;
