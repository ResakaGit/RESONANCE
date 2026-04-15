use bevy::prelude::*;

// --- Subdirectorios temáticos ---
pub mod emergence;
pub mod lifecycle;
pub mod metabolic;
pub mod thermodynamic;

// Re-exports transparentes: preservan `crate::simulation::module` para todo el codebase.
pub use thermodynamic::containment;
pub use thermodynamic::homeostasis_thermo;
pub use thermodynamic::locomotion;
pub use thermodynamic::osmosis;
pub use thermodynamic::physics;
pub use thermodynamic::pre_physics;
pub use thermodynamic::sensory;
pub use thermodynamic::structural_runtime;

pub use metabolic::atmosphere_inference;
pub use metabolic::competition_dynamics;
pub use metabolic::ecology_dynamics;
pub use metabolic::growth_budget;
pub use metabolic::metabolic_stress;
pub use metabolic::morphogenesis;
pub use metabolic::nutrient_uptake;
pub use metabolic::photosynthesis;
pub use metabolic::pool_conservation;
pub use metabolic::pool_distribution;
pub use metabolic::scale_composition;
pub use metabolic::social_communication;
pub use metabolic::trophic;

pub use lifecycle::allometric_growth;
pub use lifecycle::competitive_exclusion;
pub use lifecycle::env_scenario;
pub use lifecycle::evolution_surrogate;
pub use lifecycle::inference_growth;
pub use lifecycle::morpho_adaptation;
pub use lifecycle::organ_lifecycle;

// --- Módulos en raíz (cross-cutting) ---
pub mod ability_targeting;
pub mod abiogenesis;
pub mod atomic;
pub mod awakening;
pub mod behavior;
mod bootstrap;
pub mod checkpoint_system;
pub mod cooperation;
pub mod culture_observation;
#[cfg(test)]
mod eco_e5_simulation_tests;
pub mod element_layer2;
#[cfg(test)]
mod event_ordering_tests;
pub mod fog_of_war;
pub mod game_loop;
pub mod grimoire_enqueue;
pub mod input;
pub mod netcode;
pub mod observability;
pub(crate) mod observers;
pub mod pathfinding;
pub mod pipeline;
pub mod player_controlled;
pub mod post;
pub mod reactions;
pub mod species_to_qe;
#[cfg(test)]
mod regression;
pub mod reproduction;
pub mod sensory_perception;
pub mod states;
#[cfg(test)]
pub(crate) mod test_support;
pub mod time_compat;
#[cfg(test)]
mod verify_wave_gate;

pub use bootstrap::init_simulation_bootstrap;
pub use states::{GameState, PlayState};

pub use player_controlled::PlayerControlled;
pub use reactions::SpellMarker;

/// Fases del pipeline de simulación (5 Capas Strict).
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub enum Phase {
    Input,
    ThermodynamicLayer,
    AtomicLayer,
    ChemicalLayer,
    MetabolicLayer,
    MorphologicalLayer,
}

/// Orden dentro de `Phase::Input`: la plataforma escribe `WillActuator` antes que el resto de simulación.
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub enum InputChannelSet {
    PlatformWill,
    SimulationRest,
}
