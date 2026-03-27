//! AtomicPlugin — Phase::AtomicLayer systems (physics chain).
//!
//! Extracted from `pipeline.rs` in sprint Q5. Wraps the existing
//! `physics::register_physics_phase_systems` delegation.
//! Pure registrar: no state, no resources.

use bevy::prelude::*;

use crate::simulation::{emergence, physics};
use crate::simulation::Phase;
use crate::simulation::states::{GameState, PlayState};
use crate::world::space::update_spatial_index_after_move_system;

/// Registers all Phase::AtomicLayer systems (physics chain).
pub struct AtomicPlugin;

impl Plugin for AtomicPlugin {
    fn build(&self, app: &mut App) {
        physics::register_physics_phase_systems(app, FixedUpdate);

        // AC-2: Kuramoto entrainment — after spatial index updated, before MetabolicLayer.
        let run_gameplay = in_state(GameState::Playing).and(in_state(PlayState::Active));
        app.add_systems(
            FixedUpdate,
            emergence::entrainment::entrainment_system
                .in_set(Phase::AtomicLayer)
                .run_if(run_gameplay)
                .after(update_spatial_index_after_move_system),
        );
    }
}
