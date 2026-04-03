use bevy::app::AppExit;
use bevy::prelude::*;

#[cfg(feature = "v7_worldgen")]
use resonance::plugins::WorldgenPlugin;
use resonance::plugins::{DebugPlugin, LayersPlugin, SimulationPlugin};
#[cfg(feature = "gpu_cell_field_snapshot")]
use resonance::rendering::gpu_cell_field_snapshot::GpuCellFieldSnapshotPlugin;
use resonance::rendering::quantized_color::QuantizedColorPlugin;
use resonance::runtime_platform::compat_2d3d::{
    Compat2d3dPlugin, add_runtime_platform_plugins_by_profile,
};
use resonance::runtime_platform::hud::{AbilityHudPlugin, MinimapPlugin};

/// Cierra la ventana / proceso (demos y juego completo).
fn quit_on_esc(mut exit: EventWriter<AppExit>, input: Res<ButtonInput<KeyCode>>) {
    if input.just_pressed(KeyCode::Escape) {
        exit.send(AppExit::Success);
    }
}

fn main() {
    let mut app = App::new();
    app.add_plugins(DefaultPlugins.set(WindowPlugin {
        primary_window: Some(Window {
            title: "Resonance — Alchemical Simulation".into(),
            resolution: (1280.0, 720.0).into(),
            ..default()
        }),
        ..default()
    }))
    .add_plugins(LayersPlugin)
    // Core demo 3D: `Compat2d3dPlugin::default()` → `full3d` si el env está vacío.
    // 2D worldgen + sprites: `RESONANCE_RENDER_COMPAT_PROFILE=legacy2d cargo run`
    .add_plugins(Compat2d3dPlugin::default());

    // Plataforma runtime: tick, input capture y Time<Fixed> antes del pipeline de simulación.
    add_runtime_platform_plugins_by_profile(&mut app);

    app.add_plugins(SimulationPlugin);
    #[cfg(feature = "v7_worldgen")]
    app.add_plugins(WorldgenPlugin);
    app.add_plugins(QuantizedColorPlugin);
    #[cfg(feature = "gpu_cell_field_snapshot")]
    app.add_plugins(GpuCellFieldSnapshotPlugin);
    app.add_plugins((AbilityHudPlugin, MinimapPlugin))
        .add_plugins(DebugPlugin)
        .add_systems(Update, quit_on_esc);
    app.run();
}
