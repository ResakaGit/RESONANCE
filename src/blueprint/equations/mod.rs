//! Matemática pura del motor, particionada por dominio bajo `equations/<dominio>/`.
//! La superficie pública de `crate::blueprint::equations` se mantiene vía `pub use`.

mod finite_helpers;
pub mod macro_analytics;
pub use macro_analytics::*;
pub mod conservation;
pub use conservation::{
    conservation_error, global_conservation_error, has_invalid_values, is_valid_qe,
};

mod ability_runtime;
mod abiogenesis;
pub mod awakening;
pub mod batch_fitness;
pub mod batch_stepping;
pub mod bonded;
mod behavior;
pub mod calibration;
pub mod clinical_calibration;
pub mod codon_genome;
mod combat_will;
mod contact;
mod core_physics;
pub mod constraints;
pub mod coarsening;
pub mod cosmic_gravity;
pub mod coulomb;
mod culture;
pub mod derived_thresholds;
pub mod determinism;
mod ecology;
mod ecology_dynamics;
pub mod emergence;
mod energy_competition;
pub mod ewald;
pub mod exact_cache;
pub mod go_model;
pub mod implicit_solvent;

mod field_body;
mod field_color;
pub mod field_division;
mod flux;
mod game_loop;
mod geometry_deformation;
mod geometry_flow;
mod growth_engine;
mod homeostasis;
pub mod inferred_world_geometry;
pub mod internal_field;
mod lifecycle;
mod locomotion;
mod math_util;
pub mod md_analysis;
pub mod md_observables;
pub mod metabolic_genome;
mod metabolic_graph;
mod moba_ui;
pub mod morph_robustness;
mod morpho_adaptation;
mod morphogenesis_shape;
pub mod multicellular;
mod netcode;
pub mod observability;
mod organ_inference;
pub mod pathway_inhibitor;
pub mod pbc;
mod phenology;
pub mod planetary_formation;
pub mod planetary_rotation;
pub mod planetary_system;
mod population;
pub mod protein_fold;
pub mod proteome_inference;
mod quantized_color;
pub mod radial_field;
mod radiation_pressure;
pub mod respa;
pub mod scale_inference;
pub mod scale_temporal;
pub mod sensitivity;
pub mod special_functions;
pub mod stellar_dynamics;
mod sensory;
mod signal_propagation;
mod simulation_quality;
mod social_communication;
mod spatial;
pub mod spatial_tree;
pub mod surrogate_error;
mod tactical_ai;
pub mod temporal_telescope;
pub mod thermostat;
mod trophic;
pub mod variable_genome;
pub mod verlet;
pub mod vision;

pub use ability_runtime::*;
pub use abiogenesis::*;
pub use behavior::*;
pub use calibration::*;
pub use combat_will::*;
pub use contact::*;
pub use core_physics::*;
pub use culture::{
    CulturalPhase, conflict_active, cultural_phase, culture_emergent, culture_index,
    entrainment_possible, freq_interference, group_frequency_coherence, group_longevity_norm,
    inter_group_conflict_potential, internal_synthesis_rate, pattern_resilience,
};
pub use determinism::*;
pub use ecology::*;
pub use ecology_dynamics::*;
pub use energy_competition::*;
pub use field_body::*;
pub use field_color::*;
pub use flux::*;
pub use game_loop::*;
pub use geometry_deformation::*;
pub use geometry_flow::*;
pub use growth_engine::*;
pub use homeostasis::*;
pub use inferred_world_geometry::{
    SymmetryMode, allometric_organ_scale, build_terrain_visuals, build_water_mesh,
    compute_body_plan_layout, count_limbs_in_manifest, infer_symmetry_mode, inferred_ambient_light,
    inferred_bloom_intensity, inferred_fog_color, inferred_fog_params, inferred_sun_direction,
    inferred_sun_intensity, lateral_offset, terrain_cell_color, water_depth_color,
    water_surface_height,
};
pub use lifecycle::*;
pub use locomotion::*;
pub use math_util::*;
pub use metabolic_graph::*;
pub use moba_ui::*;
pub use morph_robustness::*;
pub use morpho_adaptation::*;
pub use morphogenesis_shape::*;
pub use netcode::*;
pub use observability::*;
pub use organ_inference::*;
pub use phenology::*;
pub use population::*;
pub use quantized_color::*;
pub use radiation_pressure::*;
pub use sensitivity::*;
pub use sensory::*;
pub use signal_propagation::*;
pub use simulation_quality::*;
pub use social_communication::*;
pub use spatial::*;
pub use surrogate_error::*;
pub use tactical_ai::*;
pub use trophic::*;
pub use vision::terrain_blocks_vision;
mod entity_shape;
pub use entity_shape::{
    bilateral_quadruped_attachments, entity_geometry_influence, entity_lod_detail,
    fineness_to_spine_params, frequency_to_tint_rgb, matter_to_gf1_resistance,
    optimal_appendage_count, organ_slot_scale, projected_area_with_limbs, shape_cache_signature,
    shape_cache_signature_with_surface,
};

// MG-1 — re-export morphogenesis (descubrimiento vía `equations::`)
pub use crate::blueprint::morphogenesis::{
    albedo_luminosity_blend, bounded_fineness_descent, carnot_efficiency, entropy_production,
    exergy_balance, heat_capacity, inferred_albedo, inferred_drag_coefficient,
    inferred_surface_rugosity, irradiance_effective_for_albedo, rugosity_to_detail_multiplier,
    shape_cost, surface_dissipation_power, vascular_transport_cost,
};

mod organ_energy;
pub use organ_energy::*;

mod spectral_absorption;
pub use spectral_absorption::*;
mod phototropism;
pub use phototropism::*;

mod tissue_growth;
pub use tissue_growth::*;
mod volatile_emission;
pub use volatile_emission::*;
mod subterranean_morphology;
pub use subterranean_morphology::*;
mod cross_transfer;
pub use cross_transfer::*;

// AUTOPOIESIS track (AP-0/1/2) — kinetics + RAF detection + Pross metric.
pub mod reaction_kinetics;
pub mod raf;
pub use reaction_kinetics::{
    ReactionOutcome, apply_reaction, diffuse_species, frequency_alignment, mass_action_rate,
    step_cell_reactions, step_grid_reactions,
};
pub use raf::{
    Closure, closure_hash, find_raf, food_mask, food_set_from_totals, kinetic_stability,
    raf_closures,
};

// AUTOPOIESIS track (AP-3) — emergent membrane pure fns (ADR-038).
pub mod membrane;
pub use membrane::{
    compute_membrane_field, damped_flux_factor, local_gradient, membrane_strength,
};

#[cfg(test)]
mod tests;
