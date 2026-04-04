//! Constantes globales de la simulación Resonance (tuning centralizado).
//! Segmentadas por dominio (encabezados del monolito `constants.rs` histórico).
//! API estable: constantes en la raíz del módulo vía `pub use` y el submódulo `morphogenesis`
//! (p. ej. `crate::blueprint::constants::FRICTION_COEF`, `crate::blueprint::constants::morphogenesis::STEFAN_BOLTZMANN`).

mod allometry_tl6;
mod almanac_coherence;
mod behavior_d1;
pub mod calibration;
mod codon;
mod containment_geometry;
mod cooperation_ac5;
mod culture;
mod derived_link_volume_will;
mod ecology_dynamics_d9;
mod ecosystem_abiogenesis;
mod ecosystem_reproduction;
mod element_id_fnv;
mod energy_competition_ec;
mod entrainment_ac2;
mod environment_trophic_li8;
mod field_eac3_epi2;
mod field_eac4_hz_hue;
mod fog_of_war_g12;
mod game_loop;
mod general;
mod gf1_branch_role_tint;
mod homeostasis_d4;
pub mod inferred_world_geometry;
mod layer00_base_energy;
mod layer01_faction;
mod layer02_08_interference;
mod layer02_oscillation;
mod layer03_flow_dissipation;
mod layer03_friction_drag;
mod layer03_osmosis;
mod layer04_coherence_state_tables;
mod layer04_growth_budget;
mod layer04_phase_transition;
mod layer04_photosynthesis;
mod layer05_branching_recursive;
mod layer05_engine_defaults;
mod layer05_engine_overload;
mod layer06_biome_pressure;
mod layer07_motor_movement;
mod layer08_catalysis;
mod layer08_injector;
mod layer13_structural_link;
mod lifecycle_li2;
mod locomotion_d3;
mod metabolic_graph_mg2;
mod metabolic_graph_mg3;
mod metabolic_graph_mg6;
mod morpho_adaptation_d8;
mod morphogenesis_track;
mod multicellular;
mod netcode;
pub mod nucleus_lifecycle;
mod numeric_math;
mod organ_attachment_li6;
mod organ_inference_li3;
mod organ_primitive_geometry;
mod organ_role_visual_li6;
mod particle_charge;
pub mod pathway_inhibitor;
mod perception;
mod senescence;
mod sensory_d5;
mod shape_color_inference_gf1;
mod simulation_actuator_collision;
mod simulation_defaults;
mod simulation_foundations;
mod social_communication_d6;
pub mod stellar;
mod surrogate;
mod tactical_ai;
pub mod temporal_telescope;
mod thermal_transfer;
mod trophic_predation_d2;
pub mod units;
mod visual_quantization_phenology;

pub use allometry_tl6::*;
pub use almanac_coherence::*;
pub use behavior_d1::*;
pub use codon::*;
pub use containment_geometry::*;
pub use cooperation_ac5::*;
pub use culture::*;
pub use derived_link_volume_will::*;
pub use ecology_dynamics_d9::*;
pub use ecosystem_abiogenesis::*;
pub use ecosystem_reproduction::*;
pub use element_id_fnv::*;
pub use energy_competition_ec::*;
pub use entrainment_ac2::*;
pub use environment_trophic_li8::*;
pub use field_eac3_epi2::*;
pub use field_eac4_hz_hue::*;
pub use fog_of_war_g12::*;
pub use game_loop::*;
pub use general::*;
pub use gf1_branch_role_tint::*;
pub use homeostasis_d4::*;
pub use layer00_base_energy::*;
pub use layer01_faction::*;
pub use layer02_08_interference::*;
pub use layer02_oscillation::*;
pub use layer03_flow_dissipation::*;
pub use layer03_friction_drag::*;
pub use layer03_osmosis::*;
pub use layer04_coherence_state_tables::*;
pub use layer04_growth_budget::*;
pub use layer04_phase_transition::*;
pub use layer04_photosynthesis::*;
pub use layer05_branching_recursive::*;
pub use layer05_engine_defaults::*;
pub use layer05_engine_overload::*;
pub use layer06_biome_pressure::*;
pub use layer07_motor_movement::*;
pub use layer08_catalysis::*;
pub use layer08_injector::*;
pub use layer13_structural_link::*;
pub use lifecycle_li2::*;
pub use locomotion_d3::*;
pub use metabolic_graph_mg2::*;
pub use metabolic_graph_mg3::*;
pub use metabolic_graph_mg6::*;
pub use morpho_adaptation_d8::*;
pub use multicellular::*;
pub use netcode::*;
pub use nucleus_lifecycle::*;
pub use numeric_math::*;
pub use organ_attachment_li6::*;
pub use organ_inference_li3::*;
pub use organ_primitive_geometry::*;
pub use organ_role_visual_li6::*;
pub use particle_charge::*;
pub use perception::*;
pub use senescence::*;
pub use sensory_d5::*;
pub use shape_color_inference_gf1::*;
pub use simulation_actuator_collision::*;
pub use simulation_defaults::*;
pub use simulation_foundations::*;
pub use social_communication_d6::*;
pub use surrogate::*;
pub use tactical_ai::*;
pub use thermal_transfer::*;
pub use trophic_predation_d2::*;
pub use units::*;
pub use visual_quantization_phenology::*;

pub mod element_bands;
pub use element_bands::*;
pub mod emergence;
pub use emergence::*;

// Re-export fundamental constants from derived_thresholds for ergonomic access.
pub use super::equations::derived_thresholds::COHERENCE_BANDWIDTH;

pub use morphogenesis::{
    ALBEDO_EPSILON, ALBEDO_FALLBACK, ALBEDO_IRRADIANCE_FLUX_EPS, ALBEDO_LUMINOSITY_ALBEDO_WEIGHT,
    ALBEDO_LUMINOSITY_BASE_WEIGHT, ALBEDO_MAX, ALBEDO_MIN, DEFAULT_CONVECTION_COEFF,
    DEFAULT_EMISSIVITY, DRAG_COEFF_BASE, DRAG_COEFF_MIN, DRAG_FINENESS_SCALE, FINENESS_DEFAULT,
    FINENESS_MAX, FINENESS_MIN, MAX_SEGMENTS_PER_ENTITY, RUGOSITY_EPSILON, RUGOSITY_MAX,
    RUGOSITY_MAX_DETAIL_MULTIPLIER, RUGOSITY_MIN, SHAPE_FD_DELTA, SHAPE_OPTIMIZER_DAMPING,
    SHAPE_OPTIMIZER_EPSILON, SHAPE_OPTIMIZER_MAX_ITER, SPECIFIC_HEAT_FACTOR, STEFAN_BOLTZMANN,
};
pub use morphogenesis_track::morphogenesis;
