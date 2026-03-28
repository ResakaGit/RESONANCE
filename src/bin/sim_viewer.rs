//! Simulation viewer — real-time visualization of the simulation.
//!
//! Two render modes:
//!   --render terminal   ASCII art in terminal (zero deps, works everywhere)
//!   --render window     2D pixel window (requires --features pixel_viewer)
//!
//! Usage:
//!   RESONANCE_MAP=big_bang cargo run --release --bin sim_viewer -- --render terminal
//!   RESONANCE_MAP=civilization_test cargo run --release --features pixel_viewer --bin sim_viewer -- --render window

use bevy::prelude::*;

use resonance::layers::{BaseEnergy, BehavioralAgent, OscillatorySignature, SpatialVolume};
use resonance::plugins::{LayersPlugin, SimulationPlugin};
use resonance::rendering::quantized_color::PaletteRegistry;
use resonance::runtime_platform::compat_2d3d::SimWorldTransformParams;
use resonance::runtime_platform::simulation_tick::{SimulationClock, SimulationTickPlugin};
use resonance::viewer::frame_buffer;
use resonance::worldgen::EnergyFieldGrid;

fn main() {
    let render_mode = parse_arg_str("--render").unwrap_or_else(|| "terminal".to_string());

    eprintln!("=== Resonance Simulation Viewer ===");
    eprintln!("render: {render_mode}");

    let mut app = build_app();

    match render_mode.as_str() {
        "terminal" => run_terminal(&mut app),
        #[cfg(feature = "pixel_viewer")]
        "window" => run_window(&mut app),
        #[cfg(not(feature = "pixel_viewer"))]
        "window" => {
            eprintln!("error: --render window requires --features pixel_viewer");
            eprintln!("  cargo run --release --features pixel_viewer --bin sim_viewer -- --render window");
            std::process::exit(1);
        }
        other => {
            eprintln!("unknown render mode: {other}. Use 'terminal' or 'window'.");
            std::process::exit(1);
        }
    }
}

fn build_app() -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_plugins(bevy::state::app::StatesPlugin);
    app.add_plugins(bevy::asset::AssetPlugin::default());
    app.init_asset::<Mesh>();
    app.init_asset::<Image>();
    app.init_asset::<bevy::pbr::StandardMaterial>();
    app.init_resource::<PaletteRegistry>();
    app.init_resource::<ButtonInput<KeyCode>>();
    app.init_resource::<ButtonInput<MouseButton>>();
    app.add_plugins(SimulationTickPlugin);
    app.insert_resource(SimWorldTransformParams::default());
    app.add_plugins(LayersPlugin);
    app.add_plugins(SimulationPlugin);
    app.finish();
    app.cleanup();
    app.update(); // Startup.
    for _ in 0..5 {
        std::thread::sleep(std::time::Duration::from_millis(20));
        app.update();
    }
    app
}

/// Collect entity + behavioral positions from world.
fn collect_positions(world: &mut bevy::ecs::world::World, grid: &EnergyFieldGrid) -> (Vec<(u32, u32, f32)>, Vec<(u32, u32)>) {
    let mut entities = Vec::new();
    let mut behaviorals = Vec::new();

    let mut eq = world.query::<(&Transform, &BaseEnergy, &OscillatorySignature, Option<&BehavioralAgent>)>();
    for (tr, energy, osc, beh) in eq.iter(world) {
        if energy.is_dead() { continue; }
        let pos = bevy::math::Vec2::new(tr.translation.x, tr.translation.z);
        if let Some((cx, cy)) = grid.cell_coords(pos) {
            entities.push((cx, cy, osc.frequency_hz()));
            if beh.is_some() {
                behaviorals.push((cx, cy));
            }
        }
    }
    (entities, behaviorals)
}

fn snapshot_frame(app: &mut App) -> Option<frame_buffer::FrameBuffer> {
    let world = app.world_mut();
    let grid = world.get_resource::<EnergyFieldGrid>()?.clone();
    let (ents, behs) = collect_positions(world, &grid);
    Some(frame_buffer::render_frame(&grid, &ents, &behs))
}

// ─── Terminal mode ───────────────────────────────────────────────────────────

fn run_terminal(app: &mut App) {
    let sleep = std::time::Duration::from_millis(17);
    loop {
        std::thread::sleep(sleep);
        app.update();

        let clk = app.world()
            .get_resource::<SimulationClock>()
            .map(|c| c.tick_id)
            .unwrap_or(0);

        if let Some(frame) = snapshot_frame(app) {
            resonance::viewer::terminal::display_frame(&frame, clk);
        }
    }
}

// ─── Window mode ─────────────────────────────────────────────────────────────

#[cfg(feature = "pixel_viewer")]
fn run_window(app: &mut App) {
    let (w, h) = {
        let world = app.world();
        let grid = world.get_resource::<EnergyFieldGrid>();
        grid.map(|g| (g.width, g.height)).unwrap_or((32, 32))
    };

    let scale = parse_arg("--scale").unwrap_or(8);
    let sleep = std::time::Duration::from_millis(17);

    resonance::viewer::pixel_window::run_window(
        resonance::viewer::pixel_window::WindowConfig {
            title: "Resonance — Simulation Viewer".to_string(),
            scale,
        },
        w,
        h,
        move || {
            std::thread::sleep(sleep);
            app.update();
            snapshot_frame(app)
        },
    );
}

// ─── Arg parsing ─────────────────────────────────────────────────────────────

fn parse_arg(flag: &str) -> Option<u32> {
    let args: Vec<String> = std::env::args().collect();
    args.iter().position(|a| a == flag).and_then(|i| args.get(i + 1)).and_then(|v| v.parse().ok())
}

fn parse_arg_str(flag: &str) -> Option<String> {
    let args: Vec<String> = std::env::args().collect();
    args.iter().position(|a| a == flag).and_then(|i| args.get(i + 1).cloned())
}
