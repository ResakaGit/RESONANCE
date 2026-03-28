//! Planet viewer — 3D globe with energy field as surface texture.
//!
//! The simulation runs headless. Each frame, the energy grid is rendered
//! onto a sphere as an equirectangular texture. The sphere rotates with
//! the day/night cycle. A directional light simulates the sun.
//!
//! Usage:
//!   RESONANCE_MAP=earth_128 cargo run --release --bin planet_viewer

use bevy::prelude::*;
use bevy::render::render_asset::RenderAssetUsages;
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};

use resonance::layers::{BaseEnergy, BehavioralAgent, OscillatorySignature};
use resonance::plugins::{LayersPlugin, SimulationPlugin};
use resonance::rendering::quantized_color::PaletteRegistry;
use resonance::runtime_platform::compat_2d3d::SimWorldTransformParams;
use resonance::runtime_platform::simulation_tick::SimulationTickPlugin;
use resonance::viewer::frame_buffer;
use resonance::worldgen::EnergyFieldGrid;
use resonance::worldgen::systems::day_night::DayNightConfig;

fn main() {
    let mut app = App::new();

    // Full Bevy with rendering.
    app.add_plugins(DefaultPlugins.set(WindowPlugin {
        primary_window: Some(Window {
            title: "Resonance — Planet Viewer".to_string(),
            resolution: (1280.0, 720.0).into(),
            ..default()
        }),
        ..default()
    }));

    // Simulation (headless logic — no game UI).
    app.init_resource::<PaletteRegistry>();
    app.add_plugins(SimulationTickPlugin);
    app.insert_resource(SimWorldTransformParams::default());
    app.add_plugins(LayersPlugin);
    app.add_plugins(SimulationPlugin);

    // Planet renderer.
    app.add_systems(Startup, setup_planet);
    app.add_systems(Update, (update_planet_texture, rotate_planet, rotate_camera));

    app.run();
}

/// Marker for the planet sphere.
#[derive(Component)]
struct Planet;

/// Marker for the orbiting camera pivot.
#[derive(Component)]
struct CameraPivot;

/// Handle to the dynamic texture.
#[derive(Resource)]
struct PlanetTexture(Handle<Image>);

fn setup_planet(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut images: ResMut<Assets<Image>>,
    grid: Option<Res<EnergyFieldGrid>>,
) {
    let (tex_w, tex_h) = grid
        .as_ref()
        .map(|g| (g.width, g.height))
        .unwrap_or((128, 128));

    // Dynamic texture — updated each frame from the energy grid.
    let size = Extent3d {
        width: tex_w,
        height: tex_h,
        depth_or_array_layers: 1,
    };
    let mut image = Image::new(
        size,
        TextureDimension::D2,
        vec![40u8; (tex_w * tex_h * 4) as usize],
        TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::RENDER_WORLD | RenderAssetUsages::MAIN_WORLD,
    );
    image.sampler = bevy::image::ImageSampler::linear();
    let texture_handle = images.add(image);
    commands.insert_resource(PlanetTexture(texture_handle.clone()));

    // Planet sphere.
    let sphere_mesh = meshes.add(Sphere::new(5.0).mesh().uv(64, 32));
    let planet_material = materials.add(StandardMaterial {
        base_color_texture: Some(texture_handle),
        perceptual_roughness: 0.8,
        metallic: 0.0,
        ..default()
    });

    commands.spawn((
        Planet,
        Mesh3d(sphere_mesh),
        MeshMaterial3d(planet_material),
        Transform::from_rotation(Quat::from_euler(EulerRot::XYZ, -0.4, 0.0, 0.0)),
    ));

    // Sun light — directional, warm.
    commands.spawn((
        DirectionalLight {
            illuminance: 15000.0,
            color: Color::srgb(1.0, 0.95, 0.85),
            shadows_enabled: true,
            ..default()
        },
        Transform::from_rotation(Quat::from_euler(EulerRot::XYZ, -0.3, 0.8, 0.0)),
    ));

    // Ambient light — space is dark but not pitch black.
    commands.insert_resource(AmbientLight {
        color: Color::srgb(0.15, 0.15, 0.25),
        brightness: 50.0,
    });

    // Camera pivot (orbits the planet).
    commands.spawn((
        CameraPivot,
        Transform::default(),
    )).with_children(|parent| {
        parent.spawn((
            Camera3d::default(),
            Transform::from_xyz(0.0, 3.0, 15.0).looking_at(Vec3::ZERO, Vec3::Y),
        ));
    });

    // Starfield background.
    commands.insert_resource(ClearColor(Color::srgb(0.02, 0.02, 0.05)));
}

/// Updates the planet texture from the energy field grid each frame.
fn update_planet_texture(
    grid: Option<Res<EnergyFieldGrid>>,
    planet_tex: Option<Res<PlanetTexture>>,
    mut images: ResMut<Assets<Image>>,
    entities_q: Query<(&Transform, &BaseEnergy, &OscillatorySignature, Option<&BehavioralAgent>)>,
) {
    let Some(grid) = grid else { return };
    let Some(tex) = planet_tex else { return };
    let Some(image) = images.get_mut(&tex.0) else { return };

    // Collect entity positions (reuse sim_viewer pattern).
    let mut ent_positions = Vec::new();
    let mut beh_positions = Vec::new();
    for (tr, energy, osc, beh) in &entities_q {
        if energy.is_dead() { continue; }
        let pos = bevy::math::Vec2::new(tr.translation.x, tr.translation.z);
        if let Some((cx, cy)) = grid.cell_coords(pos) {
            ent_positions.push((cx, cy, osc.frequency_hz()));
            if beh.is_some() {
                beh_positions.push((cx, cy));
            }
        }
    }

    let frame = frame_buffer::render_frame(&grid, &ent_positions, &beh_positions);

    // Write frame pixels into the Image data (RGBA).
    let expected_len = frame.width * frame.height * 4;
    if image.data.len() != expected_len { return; }
    for (i, &[r, g, b, a]) in frame.pixels.iter().enumerate() {
        let base = i * 4;
        image.data[base] = r;
        image.data[base + 1] = g;
        image.data[base + 2] = b;
        image.data[base + 3] = a;
    }
}

/// Rotates the planet on its axis (synced with day/night period).
fn rotate_planet(
    time: Res<Time>,
    config: Option<Res<DayNightConfig>>,
    mut planet: Query<&mut Transform, With<Planet>>,
) {
    let speed = config
        .as_ref()
        .filter(|c| c.period_ticks > 0.0)
        .map(|c| std::f32::consts::TAU / (c.period_ticks / 60.0))
        .unwrap_or(0.1);

    for mut tr in &mut planet {
        tr.rotate_y(-speed * time.delta_secs());
    }
}

/// Slowly orbits the camera around the planet.
fn rotate_camera(
    time: Res<Time>,
    mut pivot: Query<&mut Transform, With<CameraPivot>>,
) {
    for mut tr in &mut pivot {
        tr.rotate_y(0.02 * time.delta_secs());
    }
}
