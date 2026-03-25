//! Constantes globales de la simulación Resonance (tuning centralizado).
//! Segmentadas por dominio (encabezados del monolito `constants.rs` histórico).
//! API estable: constantes en la raíz del módulo vía `pub use` y el submódulo `morphogenesis`
//! (p. ej. `crate::blueprint::constants::FRICTION_COEF`, `crate::blueprint::constants::morphogenesis::STEFAN_BOLTZMANN`).

mod almanac_coherence;
mod allometry_tl6;
mod containment_geometry;
mod derived_link_volume_will;
mod ecosystem_abiogenesis;
mod ecosystem_reproduction;
mod element_id_fnv;
mod environment_trophic_li8;
mod field_eac3_epi2;
mod field_eac4_hz_hue;
mod fog_of_war_g12;
mod general;
mod gf1_branch_role_tint;
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
mod metabolic_graph_mg2;
mod metabolic_graph_mg3;
mod metabolic_graph_mg6;
mod morphogenesis_track;
mod numeric_math;
mod organ_attachment_li6;
mod organ_inference_li3;
mod organ_role_visual_li6;
mod perception;
mod shape_color_inference_gf1;
mod simulation_actuator_collision;
mod simulation_defaults;
mod thermal_transfer;
mod behavior_d1;
mod trophic_predation_d2;
mod homeostasis_d4;
mod locomotion_d3;
mod sensory_d5;
mod social_communication_d6;
mod morpho_adaptation_d8;
mod ecology_dynamics_d9;
mod energy_competition_ec;
mod visual_quantization_phenology;

pub use almanac_coherence::*;
pub use allometry_tl6::*;
pub use containment_geometry::*;
pub use derived_link_volume_will::*;
pub use ecosystem_abiogenesis::*;
pub use ecosystem_reproduction::*;
pub use element_id_fnv::*;
pub use environment_trophic_li8::*;
pub use field_eac3_epi2::*;
pub use field_eac4_hz_hue::*;
pub use fog_of_war_g12::*;
pub use general::*;
pub use gf1_branch_role_tint::*;
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
pub use metabolic_graph_mg2::*;
pub use metabolic_graph_mg3::*;
pub use metabolic_graph_mg6::*;
pub use numeric_math::*;
pub use organ_attachment_li6::*;
pub use organ_inference_li3::*;
pub use organ_role_visual_li6::*;
pub use perception::*;
pub use shape_color_inference_gf1::*;
pub use simulation_actuator_collision::*;
pub use simulation_defaults::*;
pub use thermal_transfer::*;
pub use behavior_d1::*;
pub use trophic_predation_d2::*;
pub use homeostasis_d4::*;
pub use locomotion_d3::*;
pub use sensory_d5::*;
pub use social_communication_d6::*;
pub use morpho_adaptation_d8::*;
pub use ecology_dynamics_d9::*;
pub use energy_competition_ec::*;
pub use visual_quantization_phenology::*;

pub use morphogenesis_track::morphogenesis;
pub use morphogenesis::{
    ALBEDO_EPSILON, ALBEDO_FALLBACK, ALBEDO_IRRADIANCE_FLUX_EPS, ALBEDO_LUMINOSITY_ALBEDO_WEIGHT,
    ALBEDO_LUMINOSITY_BASE_WEIGHT, ALBEDO_MAX, ALBEDO_MIN, DEFAULT_CONVECTION_COEFF,
    DEFAULT_EMISSIVITY, DRAG_COEFF_BASE, DRAG_COEFF_MIN, DRAG_FINENESS_SCALE, FINENESS_DEFAULT,
    FINENESS_MAX, FINENESS_MIN, MAX_SEGMENTS_PER_ENTITY, RUGOSITY_EPSILON, RUGOSITY_MAX,
    RUGOSITY_MAX_DETAIL_MULTIPLIER, RUGOSITY_MIN, SHAPE_FD_DELTA, SHAPE_OPTIMIZER_DAMPING,
    SHAPE_OPTIMIZER_EPSILON, SHAPE_OPTIMIZER_MAX_ITER, SPECIFIC_HEAT_FACTOR, STEFAN_BOLTZMANN,
};
