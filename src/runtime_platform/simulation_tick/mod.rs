use bevy::prelude::*;
use bevy::time::Time;

use crate::simulation::time_compat::simulation_delta_secs;

/// Configuración de runtime para orquestación de simulación V6.
#[derive(Resource, Debug, Clone, Copy)]
pub struct V6RuntimeConfig {
    pub use_fixed_tick: bool,
    pub fixed_hz: f64,
}

impl Default for V6RuntimeConfig {
    fn default() -> Self {
        Self {
            // Verify wave P0: simulación siempre en paso fijo por defecto.
            use_fixed_tick: true,
            fixed_hz: 60.0,
        }
    }
}

/// Reloj monotónico de ticks de simulación (telemetría/contratos).
#[derive(Resource, Debug, Clone, Copy, Default)]
pub struct SimulationClock {
    pub tick_id: u64,
}

/// Tiempo acumulado de simulación (determinista, sin wall-clock).
#[derive(Resource, Debug, Clone, Copy, Default)]
pub struct SimulationElapsed {
    pub secs: f32,
}

/// Set para garantizar que el clock avance antes del pipeline.
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct SimulationClockSet;

/// Avanza tick lógico y tiempo de simulación discreto.
pub fn advance_simulation_clock_system(
    mut clock: ResMut<SimulationClock>,
    mut elapsed: ResMut<SimulationElapsed>,
    fixed: Option<Res<Time<Fixed>>>,
    time: Res<Time>,
) {
    let dt = simulation_delta_secs(fixed, &time);
    if dt <= 0.0 {
        return;
    }
    clock.tick_id = clock.tick_id.saturating_add(1);
    elapsed.secs += dt;
}

/// Plugin de orquestación (Sprint 04).
pub struct SimulationTickPlugin;

impl Plugin for SimulationTickPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<V6RuntimeConfig>()
            .init_resource::<SimulationClock>()
            .init_resource::<SimulationElapsed>();

        let config = app.world().resource::<V6RuntimeConfig>().to_owned();
        if config.use_fixed_tick {
            app.insert_resource(Time::<Fixed>::from_hz(config.fixed_hz));
        }
    }
}
