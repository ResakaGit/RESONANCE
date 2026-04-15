//! ChemicalPlugin — Phase::ChemicalLayer systems (reactions chain + AI-1 bridge).
//!
//! Extracted from `pipeline.rs` in sprint Q5. Wraps the existing
//! `reactions::register_reactions_phase_systems` delegation.
//!
//! AI-1 (ADR-043): adicionalmente registra el bridge `species → qe` que
//! lee `SpeciesGrid` (resource opt-in cargado por el track AUTOPOIESIS)
//! y proyecta concentraciones al `EnergyFieldGrid` via Ax 8.  No-op si
//! `SpeciesGrid` no está presente — preserva determinismo de tracks que
//! no usan química explícita.

use bevy::prelude::*;

use crate::simulation::Phase;
use crate::simulation::reactions;
use crate::simulation::species_to_qe::species_to_qe_injection_system;

/// Registers Phase::ChemicalLayer systems: reactions chain + AI-1 bridge.
pub struct ChemicalPlugin;

impl Plugin for ChemicalPlugin {
    fn build(&self, app: &mut App) {
        reactions::register_reactions_phase_systems(app, FixedUpdate);
        app.add_systems(
            FixedUpdate,
            species_to_qe_injection_system.in_set(Phase::ChemicalLayer),
        );
    }
}
