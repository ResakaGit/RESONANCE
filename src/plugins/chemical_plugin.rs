//! ChemicalPlugin — Phase::ChemicalLayer systems (reactions chain).
//!
//! Extracted from `pipeline.rs` in sprint Q5. Wraps the existing
//! `reactions::register_reactions_phase_systems` delegation.
//! Pure registrar: no state, no resources.

use bevy::prelude::*;

use crate::simulation::reactions;

/// Registers all Phase::ChemicalLayer systems (reactions chain).
pub struct ChemicalPlugin;

impl Plugin for ChemicalPlugin {
    fn build(&self, app: &mut App) {
        reactions::register_reactions_phase_systems(app, FixedUpdate);
    }
}
