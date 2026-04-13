//! Batch simulation systems — stateless functions over `SimWorldFlat`.
//!
//! Each system calls `blueprint::equations::*` for math. No inline formulas.
//! Organized by pipeline phase, mirroring `simulation/pipeline.rs` ordering.

mod atomic;
pub mod bonded_forces;
mod chemical;
mod input;
mod internal_field;
mod metabolic;
mod metabolic_graph;
mod morphological;
mod multicellular;
pub mod parallel_forces;
mod particle_forces;
mod protein;
pub mod remd;
mod thermostat;
mod thermodynamic;

// Phase::Input
pub use input::behavior_assess;

// Phase::ThermodynamicLayer
pub use thermodynamic::{engine_processing, grid_cell, irradiance_update};

// Phase::AtomicLayer
pub use atomic::{
    collision, containment_check, dissipation, entrainment, locomotion_drain,
    tension_field_apply, velocity_cap, verlet_position_step, verlet_velocity_finish,
    will_to_velocity, wrap_positions,
};

// Phase::ChemicalLayer
pub use chemical::{homeostasis, nutrient_uptake, photosynthesis, state_transitions};

// Phase::MetabolicLayer
pub use metabolic::{
    cooperation_eval, culture_transmission, pool_distribution, social_pack, trophic_forage,
    trophic_predation,
};

// Phase::MetabolicLayer (metabolic graph + protein fold)
pub use metabolic_graph::metabolic_graph_infer;
pub use multicellular::multicellular_step;
pub use particle_forces::{count_molecules, detect_particle_bonds, particle_forces};
pub use thermostat::langevin_thermostat;
pub use protein::protein_fold_infer;

// Phase::MorphologicalLayer (internal field)
pub use internal_field::internal_diffusion;

// Phase::MorphologicalLayer
pub use morphological::{
    abiogenesis, asteroid_impact, death_reap, growth_inference, morpho_adaptation, reproduction,
    senescence,
};
