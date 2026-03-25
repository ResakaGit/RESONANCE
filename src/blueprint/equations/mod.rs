//! Matemática pura del motor, particionada por dominio bajo `equations/<dominio>/`.
//! La superficie pública de `crate::blueprint::equations` se mantiene vía `pub use`.

mod finite_helpers;

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

// MG-1 — re-export morphogenesis (descubrimiento vía `equations::`)
pub use crate::blueprint::morphogenesis::{
    albedo_luminosity_blend, bounded_fineness_descent, carnot_efficiency, entropy_production,
    exergy_balance, heat_capacity, inferred_albedo, inferred_drag_coefficient,
    inferred_surface_rugosity, irradiance_effective_for_albedo, rugosity_to_detail_multiplier,
    shape_cost, surface_dissipation_power, vascular_transport_cost,
};

#[cfg(test)]
mod tests;
