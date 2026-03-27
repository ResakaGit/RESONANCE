//! Batch simulation systems — stateless functions over `SimWorldFlat`.
//!
//! Each system calls `blueprint::equations::*` for math. No inline formulas.
//! Organized by pipeline phase, mirroring `simulation/pipeline.rs` ordering.

mod atomic;
mod chemical;
mod input;
mod metabolic;
mod morphological;
mod thermodynamic;

// Phase::Input
pub use input::behavior_assess;

// Phase::ThermodynamicLayer
pub use thermodynamic::{engine_processing, grid_cell, irradiance_update};

// Phase::AtomicLayer
pub use atomic::{
    collision, containment_check, dissipation, entrainment, locomotion_drain,
    movement_integrate, tension_field_apply, velocity_cap, will_to_velocity,
};

// Phase::ChemicalLayer
pub use chemical::{homeostasis, nutrient_uptake, photosynthesis, state_transitions};

// Phase::MetabolicLayer
pub use metabolic::{
    cooperation_eval, culture_transmission, ecology_census, pool_distribution, social_pack,
    trophic_forage, trophic_predation,
};

// Phase::MorphologicalLayer
pub use morphological::{
    abiogenesis, death_reap, growth_inference, morpho_adaptation, reproduction, senescence,
};
