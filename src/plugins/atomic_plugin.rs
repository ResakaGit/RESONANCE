//! AtomicPlugin — Phase::AtomicLayer systems (physics chain).
//!
//! Extracted from `pipeline.rs` in sprint Q5. Wraps the existing
//! `physics::register_physics_phase_systems` delegation.
//! Pure registrar: no state, no resources.

use bevy::prelude::*;

use crate::simulation::physics;

/// Registers all Phase::AtomicLayer systems (physics chain).
pub struct AtomicPlugin;

impl Plugin for AtomicPlugin {
    fn build(&self, app: &mut App) {
        physics::register_physics_phase_systems(app, FixedUpdate);
    }
}
