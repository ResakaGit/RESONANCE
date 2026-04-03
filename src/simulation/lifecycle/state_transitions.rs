//! State transitions del juego — owner único del state machine.
//! Game state transitions — single owner of the state machine.
//!
//! DC-3: Repatriado desde worldgen/systems/startup.rs.
//! Worldgen señaliza readiness vía WorldgenReady resource;
//! estos sistemas deciden cuándo transicionar.

use crate::simulation::states::{GameState, PlayState};
use crate::worldgen::WorldgenReady;
use bevy::prelude::*;

/// Transiciona `GameState::Loading → Playing`.
/// Ejecutado en Startup, antes del warmup.
///
/// Transitions GameState::Loading → Playing. Runs in Startup, before warmup.
pub fn enter_game_state_playing_system(mut next: ResMut<NextState<GameState>>) {
    next.set(GameState::Playing);
}

/// Transiciona `PlayState::Warmup → Active` cuando worldgen está listo.
/// Usa exclusive world access porque en Startup chain, los sub-states
/// no están disponibles vía SystemParam (GameState aún no fue aplicado).
///
/// Transitions PlayState::Warmup → Active when worldgen is ready.
pub fn transition_to_active_system(world: &mut World) {
    if !world.contains_resource::<WorldgenReady>() {
        return;
    }
    if let Some(mut next) = world.get_resource_mut::<NextState<PlayState>>() {
        next.set(PlayState::Active);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::simulation::states::GameState;

    /// Helper: crea app con states inicializados correctamente (Bevy 0.15).
    /// StatesPlugin es necesario para que init_state funcione.
    fn app_with_states() -> App {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, bevy::state::app::StatesPlugin));
        app.init_state::<GameState>();
        app.add_sub_state::<PlayState>();
        app
    }

    /// Helper: fuerza GameState::Playing y ejecuta un update para que el sub-state se active.
    fn force_playing(app: &mut App) {
        app.world_mut()
            .resource_mut::<NextState<GameState>>()
            .set(GameState::Playing);
        app.update(); // Apply GameState transition
        app.update(); // Sub-state becomes available
    }

    #[test]
    fn enter_playing_sets_game_state() {
        let mut app = app_with_states();
        app.add_systems(Update, enter_game_state_playing_system);

        app.update(); // System sets NextState
        app.update(); // Bevy applies transition

        let gs = app.world().resource::<State<GameState>>();
        assert_eq!(*gs.get(), GameState::Playing);
    }

    #[test]
    fn transition_noop_without_worldgen_ready() {
        let mut app = app_with_states();
        force_playing(&mut app);

        app.add_systems(Update, transition_to_active_system);
        // NO WorldgenReady inserted
        app.update();
        app.update();

        let ps = app.world().resource::<State<PlayState>>();
        assert_eq!(
            *ps.get(),
            PlayState::Warmup,
            "Should remain Warmup without WorldgenReady"
        );
    }

    #[test]
    fn transition_to_active_when_worldgen_ready() {
        let mut app = app_with_states();
        force_playing(&mut app);

        app.insert_resource(WorldgenReady {
            completed_at_tick: 100,
        });
        app.add_systems(Update, transition_to_active_system);
        app.update(); // System sets NextState via exclusive access
        app.update(); // Bevy applies transition

        let ps = app.world().resource::<State<PlayState>>();
        assert_eq!(*ps.get(), PlayState::Active);
    }

    #[test]
    fn transition_idempotent_when_already_active() {
        let mut app = app_with_states();
        force_playing(&mut app);

        app.insert_resource(WorldgenReady {
            completed_at_tick: 50,
        });
        app.add_systems(Update, transition_to_active_system);
        app.update();
        app.update();
        // Run again — should not panic
        app.update();

        let ps = app.world().resource::<State<PlayState>>();
        assert_eq!(*ps.get(), PlayState::Active);
    }

    #[test]
    fn transition_noop_before_game_state_playing() {
        let mut app = app_with_states();
        // GameState is Loading — NextState<PlayState> may not exist
        app.insert_resource(WorldgenReady {
            completed_at_tick: 10,
        });
        app.add_systems(Update, transition_to_active_system);
        // Should NOT panic — gracefully handles missing NextState<PlayState>
        app.update();
        app.update();

        let gs = app.world().resource::<State<GameState>>();
        assert_eq!(*gs.get(), GameState::Loading);
    }
}
