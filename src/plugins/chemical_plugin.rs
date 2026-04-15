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

use crate::events::FissionEvent;
use crate::layers::lineage_tag::LineageTag;
use crate::simulation::Phase;
use crate::simulation::autopoiesis_bridge::{
    FissionEventCursor, emit_fission_events_system, on_fission_spawn_entity,
    step_soup_sim_system,
};
use crate::simulation::reactions;
use crate::simulation::species_to_qe::species_to_qe_injection_system;

/// Registers Phase::ChemicalLayer systems: reactions chain + AI bridge.
///
/// AI-1 (ADR-043): `species_to_qe_injection_system` proyecta concentraciones
/// AP-* al campo qe del simulador principal.
///
/// AI-2 (ADR-044): `step_soup_sim_system` → `emit_fission_events_system` →
/// `on_fission_spawn_entity` convierten fisiones AP-* en entities ECS con
/// `BaseEnergy + OscillatorySignature + LineageTag`.  Toda la cadena es
/// opt-in via `Option<Res<...>>` — sin `SoupSimResource` los systems son no-op.
pub struct ChemicalPlugin;

impl Plugin for ChemicalPlugin {
    fn build(&self, app: &mut App) {
        reactions::register_reactions_phase_systems(app, FixedUpdate);

        // Registros AI-1 + AI-2.
        app.add_event::<FissionEvent>()
            .register_type::<FissionEvent>()
            .register_type::<LineageTag>()
            .init_resource::<FissionEventCursor>();

        // Orden: step (genera records) → species→qe (lee grid) →
        //        emit events (lee records nuevos) → spawn (consume events).
        app.add_systems(
            FixedUpdate,
            (
                step_soup_sim_system,
                species_to_qe_injection_system,
                emit_fission_events_system,
                on_fission_spawn_entity,
            )
                .chain()
                .in_set(Phase::ChemicalLayer),
        );
    }
}
