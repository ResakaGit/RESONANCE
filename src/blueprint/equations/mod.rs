//! Matemática pura del motor, particionada por dominio bajo `equations/<dominio>/`.
//! La superficie pública de `crate::blueprint::equations` se mantiene vía `pub use`.

mod finite_helpers;
pub mod macro_analytics;
pub use macro_analytics::*;
pub mod conservation;
pub use conservation::{conservation_error, global_conservation_error, has_invalid_values, is_valid_qe};

mod core_physics;
mod growth_engine;
mod ability_runtime;
mod contact;
mod metabolic_graph;
mod flux;
mod spatial;
mod combat_will;
mod field_body;
mod moba_ui;
mod geometry_flow;
mod geometry_deformation;
mod math_util;
mod phenology;
mod field_color;
mod quantized_color;
mod morphogenesis_shape;
mod ecology;
mod lifecycle;
mod trophic;
mod organ_inference;
mod population;
mod abiogenesis;
mod behavior;
mod homeostasis;
mod locomotion;
mod sensory;
mod morpho_adaptation;
mod social_communication;
mod ecology_dynamics;
mod energy_competition;
mod simulation_quality;
mod tactical_ai;
mod game_loop;
mod netcode;
pub mod awakening;
mod radiation_pressure;
mod signal_propagation;
mod culture;
pub mod emergence;
pub mod inferred_world_geometry;
pub mod calibration;
pub mod batch_fitness;
pub mod determinism;
pub mod internal_field;
pub mod morph_robustness;
pub mod observability;
pub mod sensitivity;
pub mod surrogate_error;

pub use ability_runtime::*;
pub use abiogenesis::*;
pub use behavior::*;
pub use homeostasis::*;
pub use locomotion::*;
pub use sensory::*;
pub use social_communication::*;
pub use combat_will::*;
pub use contact::*;
pub use core_physics::*;
pub use ecology::*;
pub use trophic::*;
pub use field_body::*;
pub use field_color::*;
pub use flux::*;
pub use geometry_flow::*;
pub use geometry_deformation::*;
pub use growth_engine::*;
pub use lifecycle::*;
pub use math_util::*;
pub use metabolic_graph::*;
pub use moba_ui::*;
pub use organ_inference::*;
pub use phenology::*;
pub use population::*;
pub use quantized_color::*;
pub use spatial::*;
pub use morpho_adaptation::*;
pub use morphogenesis_shape::*;
pub use ecology_dynamics::*;
pub use energy_competition::*;
pub use simulation_quality::*;
pub use tactical_ai::*;
pub use game_loop::*;
pub use netcode::*;
pub use radiation_pressure::*;
pub use signal_propagation::*;
pub use culture::{
    CulturalPhase,
    group_frequency_coherence,
    freq_interference,
    internal_synthesis_rate,
    pattern_resilience,
    group_longevity_norm,
    culture_index,
    cultural_phase,
    entrainment_possible,
    culture_emergent,
    inter_group_conflict_potential,
    conflict_active,
};
pub use calibration::*;
pub use determinism::*;
pub use morph_robustness::*;
pub use observability::*;
pub use sensitivity::*;
pub use surrogate_error::*;
pub use inferred_world_geometry::{
    SymmetryMode, allometric_organ_scale, count_limbs_in_manifest,
    infer_symmetry_mode, lateral_offset, compute_body_plan_layout,
    terrain_cell_color, build_terrain_visuals,
    water_surface_height, water_depth_color, build_water_mesh,
    inferred_sun_direction, inferred_sun_intensity, inferred_fog_params,
    inferred_fog_color, inferred_bloom_intensity, inferred_ambient_light,
};
mod entity_shape;
pub use entity_shape::{
    bilateral_quadruped_attachments, entity_geometry_influence, entity_lod_detail,
    fineness_to_spine_params, frequency_to_tint_rgb, matter_to_gf1_resistance,
    optimal_appendage_count, organ_slot_scale, projected_area_with_limbs,
    shape_cache_signature,
};

// MG-1 — re-export morphogenesis (descubrimiento vía `equations::`)
pub use crate::blueprint::morphogenesis::{
    albedo_luminosity_blend, bounded_fineness_descent, carnot_efficiency, entropy_production,
    exergy_balance, heat_capacity, inferred_albedo, inferred_drag_coefficient,
    inferred_surface_rugosity, irradiance_effective_for_albedo, rugosity_to_detail_multiplier,
    shape_cost, surface_dissipation_power, vascular_transport_cost,
};

#[cfg(test)]
mod tests;
