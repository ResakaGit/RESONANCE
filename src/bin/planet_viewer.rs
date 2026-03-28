//! Planet viewer — 3D globe with energy field as surface texture + metrics HUD.
//!
//! The simulation runs headless. Each frame, the energy grid is rendered
//! onto a sphere as an equirectangular texture. The sphere rotates with
//! the day/night cycle. A directional light simulates the sun.
//!
//! Metrics overlay shows real-time planetary stats calibrated to Earth ratios.
//!
//! Usage:
//!   RESONANCE_MAP=earth_128 cargo run --release --bin planet_viewer

use bevy::prelude::*;
use bevy::render::render_asset::RenderAssetUsages;
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};

use resonance::layers::{BaseEnergy, BehavioralAgent, OscillatorySignature, StructuralLink};
use resonance::plugins::{LayersPlugin, SimulationPlugin};
use resonance::rendering::quantized_color::PaletteRegistry;
use resonance::runtime_platform::compat_2d3d::SimWorldTransformParams;
use resonance::runtime_platform::simulation_tick::{SimulationClock, SimulationTickPlugin};
use resonance::viewer::frame_buffer;
use resonance::worldgen::{EnergyFieldGrid, NutrientFieldGrid};
use resonance::worldgen::systems::day_night::DayNightConfig;

// ── Visual constants (rendering, not physics) ────────────────────────────────
const PLANET_RADIUS: f32 = 5.0;
const SPHERE_SEGMENTS: u32 = 64;
const SPHERE_RINGS: u32 = 32;
const CAMERA_ORBIT_SPEED: f32 = 0.02;
const CAMERA_DISTANCE: f32 = 15.0;
const SIM_TICKS_PER_SEC: f64 = 600.0; // 1 day per 2 real seconds

fn main() {
    let mut app = App::new();

    app.add_plugins(DefaultPlugins.set(WindowPlugin {
        primary_window: Some(Window {
            title: "Resonance — Planet Viewer".to_string(),
            resolution: (1280.0, 720.0).into(),
            ..default()
        }),
        ..default()
    }));

    // Simulation speed: SIM_TICKS_PER_SEC ticks/s.
    // FixedUpdate runs at this rate; Bevy caps catch-up to ~250ms by default.
    // At 120 Hz, max ~30 ticks/frame before dropping — smooth enough.
    app.insert_resource(resonance::runtime_platform::simulation_tick::V6RuntimeConfig {
        use_fixed_tick: true,
        fixed_hz: SIM_TICKS_PER_SEC,
    });
    app.init_resource::<PaletteRegistry>();
    app.add_plugins(SimulationTickPlugin);
    app.insert_resource(SimWorldTransformParams::default());
    app.add_plugins(LayersPlugin);
    app.add_plugins(SimulationPlugin);

    app.add_systems(Startup, (setup_planet, setup_hud));
    app.add_systems(Update, (update_planet_texture, rotate_planet, rotate_camera, update_hud));

    app.run();
}

#[derive(Component)]
struct Planet;

#[derive(Component)]
struct CameraPivot;

#[derive(Component)]
struct MetricsText;

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

    let sphere_mesh = meshes.add(Sphere::new(PLANET_RADIUS).mesh().uv(SPHERE_SEGMENTS, SPHERE_RINGS));
    let planet_material = materials.add(StandardMaterial {
        base_color_texture: Some(texture_handle.clone()),
        emissive_texture: Some(texture_handle),
        emissive: bevy::color::LinearRgba::WHITE,
        unlit: true,
        ..default()
    });

    commands.spawn((
        Planet,
        Mesh3d(sphere_mesh),
        MeshMaterial3d(planet_material),
        Transform::from_rotation(Quat::from_euler(EulerRot::XYZ, -0.4, 0.0, 0.0)),
    ));

    commands.spawn((
        DirectionalLight {
            illuminance: 15000.0,
            color: Color::srgb(1.0, 0.95, 0.85),
            shadows_enabled: true,
            ..default()
        },
        Transform::from_rotation(Quat::from_euler(EulerRot::XYZ, -0.3, 0.8, 0.0)),
    ));

    commands.insert_resource(AmbientLight {
        color: Color::srgb(0.15, 0.15, 0.25),
        brightness: 50.0,
    });

    commands.spawn((
        CameraPivot,
        Transform::default(),
    )).with_children(|parent| {
        parent.spawn((
            Camera3d::default(),
            Transform::from_xyz(0.0, 3.0, CAMERA_DISTANCE).looking_at(Vec3::ZERO, Vec3::Y),
        ));
    });

    commands.insert_resource(ClearColor(Color::srgb(0.02, 0.02, 0.05)));
}

fn setup_hud(mut commands: Commands) {
    commands.spawn((
        MetricsText,
        Text::new("Initializing..."),
        TextFont {
            font_size: 16.0,
            ..default()
        },
        TextColor(Color::srgb(0.8, 0.9, 1.0)),
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(10.0),
            left: Val::Px(10.0),
            ..default()
        },
    ));
}

fn update_planet_texture(
    grid: Option<Res<EnergyFieldGrid>>,
    planet_tex: Option<Res<PlanetTexture>>,
    mut images: ResMut<Assets<Image>>,
    entities_q: Query<(&Transform, &BaseEnergy, &OscillatorySignature, Option<&BehavioralAgent>)>,
) {
    let Some(grid) = grid else { return };
    let Some(tex) = planet_tex else { return };
    let Some(image) = images.get_mut(&tex.0) else { return };

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

fn update_hud(
    clock: Option<Res<SimulationClock>>,
    config: Option<Res<DayNightConfig>>,
    grid: Option<Res<EnergyFieldGrid>>,
    nutrients: Option<Res<NutrientFieldGrid>>,
    entities_q: Query<&BaseEnergy, Without<resonance::worldgen::EnergyNucleus>>,
    behavioral_q: Query<(), With<BehavioralAgent>>,
    linked_q: Query<(), With<StructuralLink>>,
    mut text_q: Query<&mut Text, With<MetricsText>>,
) {
    let Ok(mut text) = text_q.get_single_mut() else { return };
    let tick = clock.as_ref().map(|c| c.tick_id).unwrap_or(0);
    let day_period = config.as_ref().map(|c| c.period_ticks).unwrap_or(1200.0);
    let year_period = config.as_ref().map(|c| c.year_period_ticks).unwrap_or(438000.0);

    // Time metrics.
    let sim_day = tick as f32 / day_period;
    let sim_year = tick as f32 / year_period;
    let season = if year_period > 0.0 {
        let phase = (tick as f32 / year_period).fract();
        match (phase * 4.0) as u32 {
            0 => "Spring",
            1 => "Summer",
            2 => "Autumn",
            _ => "Winter",
        }
    } else {
        "N/A"
    };

    // Population metrics.
    let mut alive = 0u32;
    let mut total_qe = 0.0f32;
    for energy in &entities_q {
        if !energy.is_dead() {
            alive += 1;
            total_qe += energy.qe();
        }
    }
    let behavioral = behavioral_q.iter().count() as u32;
    let linked = linked_q.iter().count() as u32;

    // Energy metrics.
    let grid_qe = grid.as_ref().map(|g| g.total_qe()).unwrap_or(0.0);

    // Water coverage.
    let water_coverage = nutrients.as_ref().map(|n| {
        let total_cells = (n.width * n.height) as f32;
        if total_cells <= 0.0 { return 0.0; }
        let wet_cells = (0..n.height).flat_map(|y| (0..n.width).map(move |x| (x, y)))
            .filter_map(|(x, y)| n.cell_xy(x, y))
            .filter(|c| c.water_norm > 0.3)
            .count() as f32;
        wet_cells / total_cells * 100.0
    }).unwrap_or(0.0);

    **text = format!(
        "EARTH SIMULATION\n\
         -------------------------\n\
         Tick: {}  Day: {:.1}  Year: {:.2}\n\
         Season: {}\n\
         -------------------------\n\
         Entities:   {}\n\
         Behavioral: {}\n\
         Multicelular: {} (linked)\n\
         -------------------------\n\
         Field qe:   {:.0}\n\
         Entity qe:  {:.0}\n\
         Total qe:   {:.0}\n\
         -------------------------\n\
         Water cover: {:.1}%\n\
         Avg qe/entity: {:.1}",
        tick, sim_day, sim_year,
        season,
        alive,
        behavioral,
        linked / 2, // bidirectional links → divide by 2
        grid_qe,
        total_qe,
        grid_qe + total_qe,
        water_coverage,
        if alive > 0 { total_qe / alive as f32 } else { 0.0 },
    );
}

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

fn rotate_camera(
    time: Res<Time>,
    mut pivot: Query<&mut Transform, With<CameraPivot>>,
) {
    for mut tr in &mut pivot {
        tr.rotate_y(CAMERA_ORBIT_SPEED * time.delta_secs());
    }
}
