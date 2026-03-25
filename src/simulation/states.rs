//! Estados globales de aplicación (Sprint G2): `GameState` + sub-estado `PlayState` bajo `Playing`.
//! La simulación de combate corre solo en `Playing` + `Active`; el campo V7 puede propagar en todo `Playing`.
//!
//! `Copy` exige que sigan siendo enums triviales; payloads pesados → `Resource` aparte.

use bevy::prelude::*;

/// Ciclo de vida app-level: carga, partida, pausa, post-partida.
#[derive(States, Clone, Copy, PartialEq, Eq, Hash, Debug, Default)]
pub enum GameState {
    #[default]
    Loading,
    Playing,
    Paused,
    PostGame,
}

/// Fase dentro de `GameState::Playing`: warmup de worldgen vs gameplay activo.
#[derive(SubStates, Clone, Copy, PartialEq, Eq, Hash, Debug, Default)]
#[source(GameState = GameState::Playing)]
pub enum PlayState {
    /// Propagación / materialización inicial (puede extenderse a multi-tick).
    #[default]
    Warmup,
    Active,
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::state::app::StatesPlugin;

    #[derive(Resource, Default)]
    struct BumpCount(u32);

    fn bump(mut c: ResMut<BumpCount>) {
        c.0 += 1;
    }

    fn app_with_state_plugins() -> App {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, StatesPlugin));
        app.init_state::<GameState>().add_sub_state::<PlayState>();
        app.init_resource::<BumpCount>();
        app
    }

    #[test]
    fn gameplay_gate_skips_during_loading() {
        let mut app = app_with_state_plugins();
        // `Update`: mismo `run_if` que producción en `FixedUpdate`, pero sin depender de
        // `Time<Fixed>` (MinimalPlugins no avanza stepping fijo en tests).
        app.add_systems(
            Update,
            bump.run_if(in_state(GameState::Playing).and(in_state(PlayState::Active))),
        );
        app.update();
        assert_eq!(app.world().resource::<BumpCount>().0, 0);
    }

    #[test]
    fn gameplay_gate_runs_after_playing_and_active() {
        let mut app = app_with_state_plugins();
        app.add_systems(
            Update,
            bump.run_if(in_state(GameState::Playing).and(in_state(PlayState::Active))),
        );
        app.world_mut()
            .resource_mut::<NextState<GameState>>()
            .set(GameState::Playing);
        app.world_mut()
            .resource_mut::<NextState<PlayState>>()
            .set(PlayState::Active);
        // `Main` aplica `NextState` antes de `Update`; no usar `run_schedule(StateTransition)` aislado.
        app.update();
        assert_eq!(app.world().resource::<BumpCount>().0, 1);
    }

    #[test]
    fn playing_warmup_skips_active_only_gate() {
        let mut app = app_with_state_plugins();
        app.add_systems(
            Update,
            bump.run_if(in_state(GameState::Playing).and(in_state(PlayState::Active))),
        );
        app.world_mut()
            .resource_mut::<NextState<GameState>>()
            .set(GameState::Playing);
        // Sub-state default = Warmup tras entrar en Playing.
        app.update();
        assert_eq!(app.world().resource::<BumpCount>().0, 0);
        app.world_mut()
            .resource_mut::<NextState<PlayState>>()
            .set(PlayState::Active);
        app.update();
        assert_eq!(app.world().resource::<BumpCount>().0, 1);
    }
}
