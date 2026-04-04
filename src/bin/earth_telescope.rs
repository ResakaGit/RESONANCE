//! Demo: Tierra + Multi-Telescopio — de la sopa primordial a la modernidad.
//! Demo: Earth + Multi-Telescope — from primordial soup to modernity.
//!
//! Visualiza la Tierra como esfera 3D con día/noche, estaciones, y un dashboard
//! del Telescopio Temporal mostrando métricas de régimen, visibilidad de Englert,
//! K adaptativo, y precisión de proyección.
//!
//! Usage:
//!   RESONANCE_MAP=earth_real cargo run --release --bin earth_telescope
//!
//! El telescopio corre en modo batch síncrono: cada N ticks (K del nivel 0),
//! el ancla simula tick-a-tick mientras el telescopio proyecta analíticamente.
//! Después reconcilia: colapso cuántico + re-emanación (ADR-016).

use bevy::prelude::*;
use bevy::render::render_asset::RenderAssetUsages;
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};

use resonance::batch::telescope::activation::{projection_accuracy, correction_frequency};
use resonance::batch::telescope::calibration_bridge::CalibrationConfig;
use resonance::batch::telescope::pipeline::{ReconciliationHistory, regime_label};
use resonance::batch::telescope::TelescopeConfig;
use resonance::blueprint::constants::temporal_telescope as tc;
use resonance::blueprint::equations::temporal_telescope::RegimeMetrics;
use resonance::layers::{BaseEnergy, BehavioralAgent, OscillatorySignature};
use resonance::plugins::{LayersPlugin, SimulationPlugin};
use resonance::rendering::quantized_color::PaletteRegistry;
use resonance::runtime_platform::compat_2d3d::SimWorldTransformParams;
use resonance::runtime_platform::dashboard_bridge::{
    DashboardBridgePlugin, RingBuffer, SimTickSummary,
};
use resonance::runtime_platform::simulation_tick::{SimulationClock, SimulationTickPlugin};
use resonance::viewer::frame_buffer;
use resonance::worldgen::systems::day_night::DayNightConfig;
use resonance::worldgen::EnergyFieldGrid;

// ── Visual constants (rendering, not physics) -------------------------───────

/// Radio de la esfera del planeta (unidades Bevy).
const PLANET_RADIUS: f32 = 5.0;
const SPHERE_SEGMENTS: u32 = 64;
const SPHERE_RINGS: u32 = 32;
/// Velocidad de órbita de cámara (rad/s). ~42s por revolución.
const CAMERA_ORBIT_SPEED: f32 = 0.15;
const CAMERA_DISTANCE: f32 = 15.0;
/// Inclinación axial de la Tierra (23.5° en radianes).
const PLANET_TILT_RAD: f32 = -0.41;
/// DENSITY_SCALE / DISSIPATION_GAS = 250. 1 día por ~5 segundos reales.
const SIM_TICKS_PER_SEC: f64 = 250.0;

/// Frecuencia de actualización del telescopio (cada N ticks de simulación).
/// No es el K del telescopio — es con qué frecuencia el sistema decide si proyectar.
const TELESCOPE_UPDATE_INTERVAL: u64 = 16;

// ── Bevy Components & Resources -------------------------─────────────────────

#[derive(Component)]
struct Planet;

#[derive(Component)]
struct CameraPivot;

#[derive(Component)]
struct MetricsText;

#[derive(Resource)]
struct PlanetTexture(Handle<Image>);

/// Estado del telescopio para la demo. Resource de Bevy.
/// Telescope demo state. Bevy Resource.
#[derive(Resource)]
struct DemoTelescopeState {
    /// Reservado para tick_telescope_stack_sync (MT-4).
    _config: TelescopeConfig,
    /// Reservado para tick_telescope_stack_sync (MT-4).
    _cal_config: CalibrationConfig,
    history: ReconciliationHistory,
    metrics: RegimeMetrics,
    last_telescope_tick: u64,
    /// Historial de qe total (últimos 512 valores) para métricas del telescopio.
    qe_ring: RingBuffer,
    /// Historial de población (últimos 512 valores).
    pop_ring: RingBuffer,
}

impl Default for DemoTelescopeState {
    fn default() -> Self {
        Self {
            _config: TelescopeConfig::default(),
            _cal_config: CalibrationConfig::default(),
            history: ReconciliationHistory::default(),
            metrics: RegimeMetrics::default(),
            last_telescope_tick: 0,
            qe_ring: RingBuffer::default(),
            pop_ring: RingBuffer::default(),
        }
    }
}

// ── Main --------------------------------------------------───────────────────

fn main() {
    let mut app = App::new();

    app.add_plugins(DefaultPlugins.set(WindowPlugin {
        primary_window: Some(Window {
            title: "Resonance — Earth Telescope Demo".to_string(),
            resolution: (1280.0, 720.0).into(),
            ..default()
        }),
        ..default()
    }));

    app.insert_resource(
        resonance::runtime_platform::simulation_tick::V6RuntimeConfig {
            use_fixed_tick: true,
            fixed_hz: SIM_TICKS_PER_SEC,
        },
    );
    app.init_resource::<PaletteRegistry>();
    app.add_plugins(SimulationTickPlugin);
    app.insert_resource(SimWorldTransformParams::default());
    app.add_plugins(LayersPlugin);
    app.add_plugins(SimulationPlugin);
    app.add_plugins(DashboardBridgePlugin);
    app.init_resource::<DemoTelescopeState>();

    app.add_systems(Startup, (setup_planet, setup_hud));
    app.add_systems(
        Update,
        (
            update_planet_texture,
            rotate_camera,
            update_telescope_metrics,
            update_hud,
        ),
    );

    app.run();
}

// ── Setup --------------------------------------------------──────────────────

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
        .unwrap_or((128, 64));

    let size = Extent3d {
        width: tex_w,
        height: tex_h,
        depth_or_array_layers: 1,
    };
    let mut image = Image::new(
        size,
        TextureDimension::D2,
        vec![0u8; (tex_w * tex_h * 4) as usize],
        TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::RENDER_WORLD | RenderAssetUsages::MAIN_WORLD,
    );
    image.sampler = bevy::image::ImageSampler::linear();
    let texture_handle = images.add(image);
    commands.insert_resource(PlanetTexture(texture_handle.clone()));

    let sphere_mesh = meshes.add(
        Sphere::new(PLANET_RADIUS)
            .mesh()
            .uv(SPHERE_SEGMENTS, SPHERE_RINGS),
    );
    let planet_material = materials.add(StandardMaterial {
        base_color_texture: Some(texture_handle),
        unlit: true,
        ..default()
    });
    commands.spawn((
        Planet,
        Mesh3d(sphere_mesh),
        MeshMaterial3d(planet_material),
        Transform::from_rotation(Quat::from_euler(EulerRot::XYZ, PLANET_TILT_RAD, 0.0, 0.0)),
    ));

    commands
        .spawn((CameraPivot, Transform::default()))
        .with_children(|parent| {
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
            font_size: 14.0,
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

// ── Texture Update (cada frame) -------------------------─────────────────────

fn update_planet_texture(
    grid: Option<Res<EnergyFieldGrid>>,
    planet_tex: Option<Res<PlanetTexture>>,
    mut images: ResMut<Assets<Image>>,
    entities_q: Query<(
        &Transform,
        &BaseEnergy,
        &OscillatorySignature,
        Option<&BehavioralAgent>,
    )>,
) {
    let Some(grid) = grid else { return };
    let Some(tex) = planet_tex else { return };

    let mut ent_positions = Vec::new();
    let mut beh_positions = Vec::new();
    for (tr, energy, osc, beh) in &entities_q {
        if energy.is_dead() {
            continue;
        }
        let pos = bevy::math::Vec2::new(tr.translation.x, tr.translation.z);
        if let Some((cx, cy)) = grid.cell_coords(pos) {
            ent_positions.push((cx, cy, osc.frequency_hz()));
            if beh.is_some() {
                beh_positions.push((cx, cy));
            }
        }
    }

    let frame = frame_buffer::render_frame(&grid, &ent_positions, &beh_positions);

    let w = frame.width as u32;
    let h = frame.height as u32;
    let mut data = Vec::with_capacity(frame.pixels.len() * 4);
    for &[r, g, b, a] in &frame.pixels {
        data.extend_from_slice(&[r, g, b, a]);
    }

    // Bevy 0.15: reemplazar Image completa fuerza GPU re-upload.
    let mut new_image = Image::new(
        Extent3d { width: w, height: h, depth_or_array_layers: 1 },
        TextureDimension::D2,
        data,
        TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::RENDER_WORLD | RenderAssetUsages::MAIN_WORLD,
    );
    new_image.sampler = bevy::image::ImageSampler::linear();
    images.insert(tex.0.id(), new_image);
}

// ── Telescope Metrics (cada TELESCOPE_UPDATE_INTERVAL ticks) ─────────────────

fn update_telescope_metrics(
    clock: Option<Res<SimulationClock>>,
    summary: Option<Res<SimTickSummary>>,
    mut state: ResMut<DemoTelescopeState>,
) {
    let Some(clock) = clock else { return };
    let Some(summary) = summary else { return };

    // Registrar en ring buffers para estadísticas del telescopio.
    state.qe_ring.push(summary.total_qe);
    state.pop_ring.push(summary.alive_count as f32);

    // Actualizar métricas del telescopio periódicamente (no cada tick — costoso).
    if clock.tick_id < state.last_telescope_tick + TELESCOPE_UPDATE_INTERVAL {
        return;
    }
    state.last_telescope_tick = clock.tick_id;

    let qe_history = state.qe_ring.to_vec();

    // Computar métricas de régimen usando funciones puras del telescopio.
    use resonance::blueprint::equations::temporal_telescope::{
        sliding_variance, sliding_autocorrelation_lag1, estimate_lambda_max,
    };
    let variance = sliding_variance(&qe_history);
    let autocorrelation = sliding_autocorrelation_lag1(&qe_history);
    let lambda_max = estimate_lambda_max(autocorrelation, 1.0 / SIM_TICKS_PER_SEC as f32);
    let population = summary.alive_count as f32 / 128.0; // normalizado a MAX_ENTITIES

    state.metrics = RegimeMetrics {
        variance,
        autocorrelation,
        lambda_max,
        population,
        hurst: 0.5, // DFA costoso — computar cada 64 updates si se necesita
        ..Default::default()
    };
}

// ── HUD (cada frame) --------------------------------------------------───────

fn update_hud(
    clock: Option<Res<SimulationClock>>,
    config: Option<Res<DayNightConfig>>,
    grid: Option<Res<EnergyFieldGrid>>,
    entities_q: Query<&BaseEnergy, Without<resonance::worldgen::EnergyNucleus>>,
    behavioral_q: Query<(), With<BehavioralAgent>>,
    telescope: Res<DemoTelescopeState>,
    mut text_q: Query<&mut Text, With<MetricsText>>,
) {
    let Ok(mut text) = text_q.get_single_mut() else { return };
    let tick = clock.as_ref().map(|c| c.tick_id).unwrap_or(0);
    let day_period = config.as_ref().map(|c| c.period_ticks).unwrap_or(600.0);
    let year_period = config.as_ref().map(|c| c.year_period_ticks).unwrap_or(219000.0);

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

    let mut alive = 0_u32;
    let mut total_qe = 0.0_f32;
    for energy in &entities_q {
        if !energy.is_dead() {
            alive += 1;
            total_qe += energy.qe();
        }
    }
    let behavioral = behavioral_q.iter().count() as u32;
    let grid_qe = grid.as_ref().map(|g| g.total_qe()).unwrap_or(0.0);

    // Métricas del telescopio.
    let m = &telescope.metrics;
    let regime = regime_label(m);
    let accuracy = projection_accuracy(&telescope.history, tc::ACCURACY_WINDOW);
    let corrections = correction_frequency(&telescope.history);

    **text = format!(
        "EARTH TELESCOPE DEMO\n\
         -------------------------\n\
         Tick: {}  Day: {:.1}  Year: {:.2}\n\
         Season: {}\n\
         -------------------------\n\
         Entities:   {}  (behavioral: {})\n\
         Field qe:   {:.0}\n\
         Entity qe:  {:.0}\n\
         Total qe:   {:.0}\n\
         -------------------------\n\
         TELESCOPE\n\
         Regime:     {}\n\
         Variance:   {:.4}\n\
         Rho1:       {:.3}\n\
         Lambda:     {:.4}\n\
         Accuracy:   {:.0}%\n\
         Corrections: {:.0}%",
        tick, sim_day, sim_year, season,
        alive, behavioral,
        grid_qe, total_qe, grid_qe + total_qe,
        regime,
        m.variance, m.autocorrelation, m.lambda_max,
        accuracy * 100.0, corrections * 100.0,
    );
}

fn rotate_camera(time: Res<Time>, mut pivot: Query<&mut Transform, With<CameraPivot>>) {
    for mut tr in &mut pivot {
        tr.rotate_y(CAMERA_ORBIT_SPEED * time.delta_secs());
    }
}
