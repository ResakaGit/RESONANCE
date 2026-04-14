//! CT-8 Cosmic Telescope — viewer 3D multi-escala.
//! CT-8 Cosmic Telescope — 3D multi-scale viewer.
//!
//! Big Bang → estrellas → planetas → vida → proteínas. Click en entidad =
//! colapso observacional (zoom-in). Escape = agregación (zoom-out). Cada
//! escala tiene estilo visual propio; las transiciones son fade-out + fade-in
//! sobre los meshes reales — nunca un teleport.
//!
//! Controles:
//!   Click izquierdo   → zoom-in a la entidad apuntada
//!   Escape            → zoom-out
//!   1..=5             → saltar a S0..S4 (si está instanciada)
//!   Space             → pause/resume
//!   Tab               → siguiente seed (nuevo universo)
//!   Right-drag        → orbitar cámara
//!   Scroll            → distancia cámara
//!
//! La lógica de seeding y bridges vive en `cosmic::observer`; este binario
//! sólo maneja presentación y input (ADR-036 §D6).

use bevy::input::mouse::{MouseMotion, MouseWheel};
use bevy::prelude::*;

use resonance::blueprint::equations::derived_thresholds::COHERENCE_BANDWIDTH;
use resonance::cosmic::constants::scale_camera_distance;
use resonance::cosmic::multiverse::ROOT_PARENT_ID;
use resonance::cosmic::scale_manager::CosmicEntity;
use resonance::cosmic::scales::coarsening::{
    background_coarsening_system, CosmicBackgroundClock,
};
use resonance::cosmic::zoom::zoom_out_system;
use resonance::cosmic::{
    largest_entity_in, rebranch_observed, scale_label, scale_short, seed_universe,
    zoom_via_bridge, BigBangParams, BranchSnapshot, CosmicPlugin, MultiverseBranch, MultiverseLog,
    ScaleLevel, ScaleManager, ZoomOutEvent, ALL_SCALES,
};

// ─── Visual constants ──────────────────────────────────────────────────────

/// Radio de referencia al que normalizamos cada escala en view-space.
const VIEW_RADIUS: f32 = 8.0;
/// Distancia inicial de la cámara al origen.
const CAMERA_DISTANCE_INITIAL: f32 = 22.0;
const CAMERA_DISTANCE_MIN: f32 = 6.0;
const CAMERA_DISTANCE_MAX: f32 = 80.0;
/// Sensibilidad del drag-orbit (rad por píxel).
const ORBIT_SENSITIVITY: f32 = 0.006;
/// Sensibilidad del scroll-zoom (factor por tick).
const SCROLL_SENSITIVITY: f32 = 0.92;
/// Duración de cada fase de fade (segundos).
const FADE_DURATION: f32 = 0.25;
/// Color de fondo — negro profundo con un toque azulado.
const CLEAR_COLOR: Color = Color::srgb(0.01, 0.01, 0.03);

/// Teclas 1..5 ↔ `ALL_SCALES` por índice. Orden fijo por convención.
const SCALE_JUMP_KEYS: [KeyCode; 5] = [
    KeyCode::Digit1,
    KeyCode::Digit2,
    KeyCode::Digit3,
    KeyCode::Digit4,
    KeyCode::Digit5,
];

// ─── Components ────────────────────────────────────────────────────────────

#[derive(Component)]
struct SceneRoot;

/// Marca un mesh spawneado para una entidad cósmica en la escala observada.
#[derive(Component, Clone, Copy)]
struct ScaleEntityView {
    entity_id: u32,
    level: ScaleLevel,
    /// Escala nominal (sin fade). `Transform.scale = base_scale * fade_alpha`.
    base_scale: f32,
}

/// Marca el pivote de la cámara orbital.
#[derive(Component)]
struct OrbitCamera {
    yaw: f32,
    pitch: f32,
    distance: f32,
    distance_target: f32,
}

impl Default for OrbitCamera {
    fn default() -> Self {
        Self {
            yaw: 0.5,
            pitch: 0.35,
            distance: CAMERA_DISTANCE_INITIAL,
            distance_target: CAMERA_DISTANCE_INITIAL,
        }
    }
}

#[derive(Component)]
struct HudText;

/// Root del breadcrumb bar clicable. Contiene los segmentos como hijos.
#[derive(Component)]
struct BreadcrumbBar;

/// Segmento clicable del breadcrumb. `back_steps = None` marca la escala
/// observada (no navegable). `Some(n)` = zoom-out n veces al pulsar.
#[derive(Component, Clone, Copy)]
struct BreadcrumbSegment {
    back_steps: Option<u8>,
}

/// Firma del estado reflejado en el bar — evita rebuilds superfluos.
#[derive(Resource, Default)]
struct BreadcrumbLayout {
    signature: u64,
}

/// Panel lateral de comparación (CT-9 §E3). Alterna con la tecla `C`.
#[derive(Component)]
struct ComparisonPanel;

// ─── Resources ─────────────────────────────────────────────────────────────

/// Estado del viewer. `scene` conduce el fade; `breadcrumb` refleja el path
/// recorrido; `paused` gobierna el coarsening; `comparison_open` alterna el
/// panel de branch anterior (CT-9 §E3).
#[derive(Resource, Default)]
struct ViewerState {
    paused: bool,
    comparison_open: bool,
    breadcrumb: Vec<BreadcrumbStep>,
    scene: ScenePhase,
}

#[derive(Clone, Copy, Debug)]
struct BreadcrumbStep {
    from: ScaleLevel,
    parent_id: u32,
}

#[derive(Clone, Copy, Debug, Default)]
enum ScenePhase {
    /// No hay nada renderizado todavía (arranque).
    #[default]
    Empty,
    /// Hay una escala renderizada y estable en pantalla.
    Steady { rendered: ScaleLevel },
    /// Desvaneciendo los meshes de `from` para luego saltar a `to`.
    FadeOut { from: ScaleLevel, to: ScaleLevel, t: f32 },
    /// Apareciendo los meshes recién spawneados de `to`.
    FadeIn { to: ScaleLevel, t: f32 },
}

impl ScenePhase {
    fn rendered_level(self) -> Option<ScaleLevel> {
        match self {
            ScenePhase::Empty => None,
            ScenePhase::Steady { rendered } => Some(rendered),
            ScenePhase::FadeOut { from, .. } => Some(from),
            ScenePhase::FadeIn { to, .. } => Some(to),
        }
    }

    fn is_transitioning(self) -> bool {
        matches!(self, ScenePhase::FadeOut { .. } | ScenePhase::FadeIn { .. })
    }
}

// ─── Main ──────────────────────────────────────────────────────────────────

fn main() {
    let mut app = App::new();
    app.add_plugins(DefaultPlugins.set(WindowPlugin {
        primary_window: Some(Window {
            title: "Resonance — Cosmic Telescope".to_string(),
            resolution: (1280.0, 720.0).into(),
            ..default()
        }),
        ..default()
    }))
    .insert_resource(ClearColor(CLEAR_COLOR))
    .init_resource::<ViewerState>()
    .init_resource::<BreadcrumbLayout>()
    .add_plugins(CosmicPlugin)
    .init_resource::<CosmicBackgroundClock>();

    app.add_systems(
        Startup,
        (
            setup_camera_and_light,
            setup_hud,
            setup_breadcrumb_bar,
            setup_comparison_panel,
            seed_initial_universe.after(setup_camera_and_light),
        ),
    );

    app.add_systems(
        Update,
        (
            handle_breadcrumb_clicks,
            input_keyboard,
            input_mouse,
            coarsening_gated,
            zoom_out_system,
            drive_scene_phase,
            animate_scene_entities,
            animate_camera,
            rebuild_breadcrumb_bar,
            style_breadcrumb_segments,
            update_hud,
            update_comparison_panel,
        )
            .chain(),
    );

    app.run();
}

// ─── Startup ───────────────────────────────────────────────────────────────

fn setup_camera_and_light(mut commands: Commands) {
    commands.spawn((
        Camera3d::default(),
        OrbitCamera::default(),
        Transform::from_xyz(0.0, 5.0, CAMERA_DISTANCE_INITIAL).looking_at(Vec3::ZERO, Vec3::Y),
    ));

    // Luz ambiente tenue — las escalas cósmicas son mayormente emissive.
    commands.insert_resource(AmbientLight {
        color: Color::srgb(0.9, 0.9, 1.0),
        brightness: 35.0,
    });

    commands.spawn((
        DirectionalLight {
            illuminance: 800.0,
            shadows_enabled: false,
            ..default()
        },
        Transform::from_xyz(5.0, 10.0, 5.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));

    commands.spawn((SceneRoot, Transform::default(), Visibility::Visible));
}

fn setup_hud(mut commands: Commands) {
    commands.spawn((
        HudText,
        Text::new("Initializing universe..."),
        TextFont { font_size: 13.0, ..default() },
        TextColor(Color::srgb(0.85, 0.92, 1.0)),
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(10.0),
            left: Val::Px(10.0),
            ..default()
        },
    ));
}

fn setup_breadcrumb_bar(mut commands: Commands) {
    commands.spawn((
        BreadcrumbBar,
        Node {
            position_type: PositionType::Absolute,
            bottom: Val::Px(16.0),
            left: Val::Px(16.0),
            right: Val::Px(16.0),
            flex_direction: FlexDirection::Row,
            align_items: AlignItems::Center,
            column_gap: Val::Px(4.0),
            ..default()
        },
    ));
}

fn seed_initial_universe(mut mgr: ResMut<ScaleManager>) {
    seed_universe(&mut mgr, &BigBangParams::interactive(initial_seed()));
}

fn initial_seed() -> u64 {
    std::env::args().nth(1).and_then(|s| s.parse().ok()).unwrap_or(42)
}

// ─── Input ─────────────────────────────────────────────────────────────────

fn input_keyboard(
    keys: Res<ButtonInput<KeyCode>>,
    mut state: ResMut<ViewerState>,
    mut mgr: ResMut<ScaleManager>,
    mut log: ResMut<MultiverseLog>,
    mut zoom_out_ev: EventWriter<ZoomOutEvent>,
) {
    if state.scene.is_transitioning() {
        return; // bloqueamos input durante la animación — evita saltos encadenados.
    }

    if keys.just_pressed(KeyCode::Space) {
        state.paused = !state.paused;
    }

    if keys.just_pressed(KeyCode::KeyC) {
        state.comparison_open = !state.comparison_open;
    }

    if keys.just_pressed(KeyCode::Escape) && state.breadcrumb.pop().is_some() {
        zoom_out_ev.send(ZoomOutEvent);
    }

    if keys.just_pressed(KeyCode::Tab) {
        cycle_seed(&mut mgr, &mut log, &mut state);
    }

    // Digit1..Digit5 ↔ ALL_SCALES por índice — una sola fuente de verdad.
    for (key, target) in SCALE_JUMP_KEYS.iter().zip(ALL_SCALES.iter()) {
        if keys.just_pressed(*key) && mgr.has(*target) && mgr.observed != *target {
            jump_to(&mut state, &mut mgr, *target, &mut zoom_out_ev);
            break;
        }
    }
}

/// Tab — `cycle seed` del branch actual (CT-9 §E2). En S0 re-siembra el
/// universo entero; en cualquier otra escala re-ejecuta el bridge de la
/// escala observada con `seed+1`, conservando el breadcrumb. El snapshot
/// del branch abandonado queda registrado en `MultiverseLog`.
fn cycle_seed(
    mgr: &mut ScaleManager,
    log: &mut MultiverseLog,
    state: &mut ViewerState,
) {
    let bandwidth = COHERENCE_BANDWIDTH as f64;
    let observed = mgr.observed;

    // 1. Snapshot del branch que estamos abandonando.
    if let Some(inst) = mgr.get(observed) {
        log.record(MultiverseBranch::from_instance(inst, bandwidth));
    }

    // 2. Nueva realidad — rama divergente en el punto actual.
    match observed.parent() {
        None => {
            let next = mgr.universe_seed.wrapping_add(1);
            state.breadcrumb.clear();
            seed_universe(mgr, &BigBangParams::interactive(next));
        }
        Some(_) => {
            let current_seed = mgr
                .get(observed)
                .map(|i| i.zoom_seed)
                .unwrap_or(mgr.universe_seed);
            rebranch_observed(mgr, current_seed.wrapping_add(1));
        }
    }
}

fn jump_to(
    state: &mut ViewerState,
    mgr: &mut ScaleManager,
    target: ScaleLevel,
    zoom_out_ev: &mut EventWriter<ZoomOutEvent>,
) {
    let observed_depth = mgr.observed.depth();
    let target_depth = target.depth();
    if target_depth < observed_depth {
        // Subir por zoom-out iterado; el sistema descargará los niveles en cadena.
        let steps = (observed_depth - target_depth) as usize;
        for _ in 0..steps {
            if state.breadcrumb.pop().is_some() {
                zoom_out_ev.send(ZoomOutEvent);
            }
        }
    } else if target_depth > observed_depth {
        // Bajar por zoom-in iterado usando la entidad dominante en cada escala.
        let mut from = mgr.observed;
        while from != target {
            let Some(pid) = largest_entity_in(mgr, from) else { break; };
            state.breadcrumb.push(BreadcrumbStep { from, parent_id: pid });
            let Some(child) = zoom_via_bridge(mgr, pid, from) else {
                state.breadcrumb.pop();
                break;
            };
            from = child;
        }
    }
}

fn input_mouse(
    buttons: Res<ButtonInput<MouseButton>>,
    mut motion: EventReader<MouseMotion>,
    mut wheel: EventReader<MouseWheel>,
    windows: Query<&Window>,
    mut cam_q: Query<(&mut OrbitCamera, &Transform, &Camera)>,
    views_q: Query<(&Transform, &ScaleEntityView), Without<OrbitCamera>>,
    mut state: ResMut<ViewerState>,
    mut mgr: ResMut<ScaleManager>,
) {
    let Ok((mut orbit, cam_tf, camera)) = cam_q.get_single_mut() else { return; };

    // Scroll → zoom de cámara.
    let scroll: f32 = wheel.read().map(|w| w.y).sum();
    if scroll != 0.0 {
        let factor = SCROLL_SENSITIVITY.powf(scroll);
        orbit.distance_target = (orbit.distance_target * factor)
            .clamp(CAMERA_DISTANCE_MIN, CAMERA_DISTANCE_MAX);
    }

    // Right-drag → orbit.
    if buttons.pressed(MouseButton::Right) {
        let delta: Vec2 = motion.read().map(|m| m.delta).sum();
        orbit.yaw -= delta.x * ORBIT_SENSITIVITY;
        orbit.pitch = (orbit.pitch - delta.y * ORBIT_SENSITIVITY)
            .clamp(-1.4, 1.4);
    } else {
        // Consumir motion aunque no orbitemos — evita ráfagas acumuladas.
        motion.clear();
    }

    // Left-click → zoom-in sobre la entidad apuntada.
    if buttons.just_pressed(MouseButton::Left) && !state.scene.is_transitioning() {
        if let Some(hit) = pick_entity(&windows, camera, cam_tf, &views_q) {
            let from = hit.level;
            state.breadcrumb.push(BreadcrumbStep { from, parent_id: hit.entity_id });
            if zoom_via_bridge(&mut mgr, hit.entity_id, from).is_none() {
                state.breadcrumb.pop();
            }
        }
    }
}

struct PickHit { level: ScaleLevel, entity_id: u32 }

fn pick_entity(
    windows: &Query<&Window>,
    camera: &Camera,
    cam_tf: &Transform,
    views_q: &Query<(&Transform, &ScaleEntityView), Without<OrbitCamera>>,
) -> Option<PickHit> {
    let window = windows.get_single().ok()?;
    let cursor = window.cursor_position()?;
    let ray = camera.viewport_to_world(&GlobalTransform::from(*cam_tf), cursor).ok()?;

    let mut best: Option<(PickHit, f32)> = None;
    for (tf, view) in views_q.iter() {
        let center = tf.translation;
        let radius = tf.scale.max_element().max(0.15);
        // Intersección rayo-esfera (t min ≥ 0).
        let oc = ray.origin - center;
        let b = oc.dot(*ray.direction);
        let c = oc.length_squared() - radius * radius;
        let disc = b * b - c;
        if disc < 0.0 { continue; }
        let t = -b - disc.sqrt();
        if t < 0.0 { continue; }
        let better = best.as_ref().map(|(_, best_t)| t < *best_t).unwrap_or(true);
        if better {
            best = Some((PickHit { level: view.level, entity_id: view.entity_id }, t));
        }
    }
    best.map(|(hit, _)| hit)
}

// ─── Coarsening (gated on pause) ───────────────────────────────────────────

fn coarsening_gated(
    state: Res<ViewerState>,
    clock: ResMut<CosmicBackgroundClock>,
    mgr: ResMut<ScaleManager>,
) {
    if state.paused { return; }
    background_coarsening_system(clock, mgr);
}

// ─── Scene phase driver ────────────────────────────────────────────────────

fn drive_scene_phase(
    time: Res<Time>,
    mut state: ResMut<ViewerState>,
    mgr: Res<ScaleManager>,
    mut commands: Commands,
    meshes: ResMut<Assets<Mesh>>,
    materials: ResMut<Assets<StandardMaterial>>,
    mut orbit_q: Query<&mut OrbitCamera>,
    views_q: Query<Entity, With<ScaleEntityView>>,
) {
    let dt = time.delta_secs();
    let observed = mgr.observed;
    let rendered = state.scene.rendered_level();

    // Avanzar transición en curso.
    let next = match state.scene {
        ScenePhase::FadeOut { from, to, mut t } => {
            t += dt;
            if t >= FADE_DURATION {
                despawn_views(&mut commands, &views_q);
                match mgr.get(to) {
                    Some(inst) => {
                        spawn_scale_view(
                            &mut commands,
                            meshes,
                            materials,
                            inst.world.entities.as_slice(),
                            to,
                        );
                        nudge_camera_for_scale(&mut orbit_q, to);
                        ScenePhase::FadeIn { to, t: 0.0 }
                    }
                    // La escala destino desapareció (ej. zoom-out más allá);
                    // el próximo frame reconvergerá hacia `observed` actual.
                    None => ScenePhase::Empty,
                }
            } else {
                ScenePhase::FadeOut { from, to, t }
            }
        }
        ScenePhase::FadeIn { to, mut t } => {
            t += dt;
            if t >= FADE_DURATION {
                ScenePhase::Steady { rendered: to }
            } else {
                ScenePhase::FadeIn { to, t }
            }
        }
        ScenePhase::Empty | ScenePhase::Steady { .. } => {
            if rendered == Some(observed) {
                state.scene
            } else {
                match rendered {
                    None => {
                        // Primer spawn: sin fade-out previo.
                        if let Some(inst) = mgr.get(observed) {
                            spawn_scale_view(
                                &mut commands,
                                meshes,
                                materials,
                                inst.world.entities.as_slice(),
                                observed,
                            );
                            nudge_camera_for_scale(&mut orbit_q, observed);
                        }
                        ScenePhase::FadeIn { to: observed, t: 0.0 }
                    }
                    Some(from) => ScenePhase::FadeOut { from, to: observed, t: 0.0 },
                }
            }
        }
    };
    state.scene = next;
}

fn despawn_views(commands: &mut Commands, views_q: &Query<Entity, With<ScaleEntityView>>) {
    for e in views_q.iter() {
        commands.entity(e).despawn_recursive();
    }
}

fn nudge_camera_for_scale(orbit_q: &mut Query<&mut OrbitCamera>, level: ScaleLevel) {
    let Ok(mut orbit) = orbit_q.get_single_mut() else { return; };
    orbit.distance_target =
        scale_camera_distance(level).clamp(CAMERA_DISTANCE_MIN, CAMERA_DISTANCE_MAX);
}

// ─── Per-scale spawning ────────────────────────────────────────────────────

fn spawn_scale_view(
    commands: &mut Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    entities: &[CosmicEntity],
    level: ScaleLevel,
) {
    let alive: Vec<&CosmicEntity> = entities.iter().filter(|e| e.alive).collect();
    if alive.is_empty() { return; }

    let positions = layout_positions(&alive, level);
    let unit_sphere = meshes.add(Sphere::new(1.0).mesh().uv(24, 12));

    for (i, entity) in alive.iter().enumerate() {
        let pos = positions[i];
        let style = style_for(level, entity);
        let material = materials.add(style.material.clone());
        commands.spawn((
            ScaleEntityView {
                entity_id: entity.entity_id,
                level,
                base_scale: style.base_scale,
            },
            Mesh3d(unit_sphere.clone()),
            MeshMaterial3d(material),
            // Empieza en escala 0 — se anima en FadeIn.
            Transform::from_translation(pos).with_scale(Vec3::ZERO),
        ));

        // Halos para escalas cósmicas/estelares — esfera translúcida más grande.
        if let Some(halo_material) = style.halo {
            let halo = materials.add(halo_material);
            commands.spawn((
                ScaleEntityView {
                    entity_id: entity.entity_id,
                    level,
                    base_scale: style.base_scale * 2.2,
                },
                Mesh3d(unit_sphere.clone()),
                MeshMaterial3d(halo),
                Transform::from_translation(pos).with_scale(Vec3::ZERO),
            ));
        }
    }
}

struct EntityStyle {
    material: StandardMaterial,
    halo: Option<StandardMaterial>,
    base_scale: f32,
}

fn style_for(level: ScaleLevel, entity: &CosmicEntity) -> EntityStyle {
    let qe = entity.qe.max(1e-9) as f32;
    match level {
        ScaleLevel::Cosmological => {
            let hue = (entity.frequency_hz * 0.01).sin().abs() as f32;
            let base = 0.25 + 0.25 * qe.log10().clamp(0.0, 4.0) / 4.0;
            EntityStyle {
                material: emissive(Color::srgb(0.6 + 0.3 * hue, 0.7, 1.0), 3.5),
                halo: Some(translucent(Color::srgba(0.3, 0.5, 1.0, 0.12))),
                base_scale: base.clamp(0.25, 1.0),
            }
        }
        ScaleLevel::Stellar => {
            let t = (qe.log10().clamp(0.0, 4.0) / 4.0).clamp(0.0, 1.0);
            let color = Color::srgb(1.0, 0.6 + 0.3 * t, 0.2 + 0.4 * t);
            EntityStyle {
                material: emissive(color, 5.0),
                halo: Some(translucent(Color::srgba(1.0, 0.7, 0.3, 0.10))),
                base_scale: (0.18 + 0.22 * t).clamp(0.15, 0.6),
            }
        }
        ScaleLevel::Planetary => {
            let t = (entity.frequency_hz as f32).rem_euclid(1.0);
            let color = Color::srgb(0.3 + 0.4 * t, 0.5, 0.7 - 0.4 * t);
            EntityStyle {
                material: matte(color, 0.4),
                halo: None,
                base_scale: 0.35,
            }
        }
        ScaleLevel::Ecological => EntityStyle {
            material: matte(Color::srgb(0.3, 0.8, 0.4), 0.5),
            halo: None,
            base_scale: 1.2,
        },
        ScaleLevel::Molecular => {
            let hue = ((entity.entity_id as f32) * 0.17).sin().abs();
            let color = Color::srgb(0.8, 0.4 + 0.5 * hue, 0.3 + 0.5 * (1.0 - hue));
            EntityStyle {
                material: matte(color, 0.2),
                halo: None,
                base_scale: 0.25,
            }
        }
    }
}

fn emissive(color: Color, strength: f32) -> StandardMaterial {
    StandardMaterial {
        base_color: color,
        emissive: color.to_linear() * strength,
        unlit: false,
        ..default()
    }
}

fn translucent(color: Color) -> StandardMaterial {
    StandardMaterial {
        base_color: color,
        emissive: color.to_linear() * 0.4,
        alpha_mode: AlphaMode::Blend,
        unlit: true,
        ..default()
    }
}

fn matte(color: Color, metallic: f32) -> StandardMaterial {
    StandardMaterial {
        base_color: color,
        perceptual_roughness: 0.7,
        metallic,
        ..default()
    }
}

// ─── Layout ────────────────────────────────────────────────────────────────

/// Normaliza las posiciones f64 de los entities a view-space f32 (bbox →
/// `VIEW_RADIUS`). Para escalas con todas las posiciones coincidentes genera
/// una distribución Fibonacci esférica determinista sobre `entity_id`.
fn layout_positions(alive: &[&CosmicEntity], level: ScaleLevel) -> Vec<Vec3> {
    let raw: Vec<Vec3> = alive
        .iter()
        .map(|e| Vec3::new(e.position[0] as f32, e.position[1] as f32, e.position[2] as f32))
        .collect();

    let extent = bbox_extent(&raw);
    if extent > 1e-4 {
        let centroid = raw.iter().copied().sum::<Vec3>() / raw.len() as f32;
        let scale = VIEW_RADIUS / extent;
        return raw.iter().map(|p| (*p - centroid) * scale).collect();
    }

    // Fallback: Fibonacci sphere para escalas sin geometría propia (Molecular/Ecological).
    let radius = match level {
        ScaleLevel::Molecular => VIEW_RADIUS * 0.6,
        ScaleLevel::Ecological => 0.0,
        _ => VIEW_RADIUS * 0.5,
    };
    alive
        .iter()
        .enumerate()
        .map(|(i, _)| fibonacci_sphere(i, alive.len(), radius))
        .collect()
}

fn bbox_extent(points: &[Vec3]) -> f32 {
    let mut min = Vec3::splat(f32::INFINITY);
    let mut max = Vec3::splat(f32::NEG_INFINITY);
    for p in points {
        min = min.min(*p);
        max = max.max(*p);
    }
    (max - min).max_element()
}

fn fibonacci_sphere(i: usize, n: usize, radius: f32) -> Vec3 {
    if n <= 1 || radius <= 0.0 { return Vec3::ZERO; }
    let phi = std::f32::consts::PI * (3.0 - (5.0_f32).sqrt());
    let y = 1.0 - (i as f32 / (n - 1) as f32) * 2.0;
    let r = (1.0 - y * y).sqrt();
    let theta = phi * i as f32;
    Vec3::new(theta.cos() * r, y, theta.sin() * r) * radius
}

// ─── Animation ─────────────────────────────────────────────────────────────

fn animate_scene_entities(
    state: Res<ViewerState>,
    mut q: Query<(&ScaleEntityView, &mut Transform)>,
) {
    let alpha = fade_alpha(state.scene);
    for (view, mut tf) in &mut q {
        let target = view.base_scale * alpha;
        let scale = Vec3::splat(target);
        if tf.scale != scale { tf.scale = scale; }
    }
}

fn fade_alpha(phase: ScenePhase) -> f32 {
    match phase {
        ScenePhase::Empty => 0.0,
        ScenePhase::Steady { .. } => 1.0,
        ScenePhase::FadeOut { t, .. } => (1.0 - t / FADE_DURATION).clamp(0.0, 1.0),
        ScenePhase::FadeIn { t, .. } => (t / FADE_DURATION).clamp(0.0, 1.0),
    }
}

fn animate_camera(time: Res<Time>, mut q: Query<(&mut OrbitCamera, &mut Transform)>) {
    let Ok((mut orbit, mut tf)) = q.get_single_mut() else { return; };
    // Lerp exponencial de la distancia hacia distance_target.
    let k = 1.0 - (-6.0 * time.delta_secs()).exp();
    orbit.distance += (orbit.distance_target - orbit.distance) * k;

    let yaw = orbit.yaw;
    let pitch = orbit.pitch;
    let d = orbit.distance;
    let pos = Vec3::new(
        d * pitch.cos() * yaw.sin(),
        d * pitch.sin(),
        d * pitch.cos() * yaw.cos(),
    );
    tf.translation = pos;
    *tf = tf.looking_at(Vec3::ZERO, Vec3::Y);
}

// ─── Breadcrumb bar ────────────────────────────────────────────────────────

/// Firma de los datos mostrados en el bar (path + observed). Cambios fuerzan
/// rebuild; idempotencia entre frames estables evita churn de entidades UI.
fn breadcrumb_signature(state: &ViewerState, observed: ScaleLevel) -> u64 {
    let mut h = state.breadcrumb.len() as u64;
    for step in &state.breadcrumb {
        h = h.wrapping_mul(0x100000001B3) ^ step.parent_id as u64;
        h = h.wrapping_mul(0x100000001B3) ^ step.from.depth() as u64;
    }
    h.wrapping_mul(0x100000001B3) ^ observed.depth() as u64
}

fn rebuild_breadcrumb_bar(
    state: Res<ViewerState>,
    mgr: Res<ScaleManager>,
    mut layout: ResMut<BreadcrumbLayout>,
    mut commands: Commands,
    bar_q: Query<Entity, With<BreadcrumbBar>>,
    children_q: Query<Entity, With<BreadcrumbSegment>>,
) {
    let sig = breadcrumb_signature(&state, mgr.observed);
    if sig == layout.signature { return; }
    layout.signature = sig;

    let Ok(bar) = bar_q.get_single() else { return; };
    // Despawn segmentos existentes. Separadores están marcados también como segmentos
    // con `back_steps = None` y label " > "; los distinguimos por label al respawnear.
    for e in children_q.iter() {
        commands.entity(e).despawn_recursive();
    }

    let total_back = state.breadcrumb.len() as u8;
    commands.entity(bar).with_children(|parent| {
        for (i, step) in state.breadcrumb.iter().enumerate() {
            let back = total_back - i as u8;
            spawn_segment(parent, scale_short(step.from), Some(back));
            spawn_separator(parent);
        }
        spawn_segment(parent, scale_short(mgr.observed), None);
    });
}

fn spawn_segment(parent: &mut ChildBuilder, label: &str, back_steps: Option<u8>) {
    parent
        .spawn((
            Button,
            BreadcrumbSegment { back_steps },
            Node {
                padding: UiRect::axes(Val::Px(8.0), Val::Px(4.0)),
                border: UiRect::all(Val::Px(1.0)),
                ..default()
            },
            BorderColor(Color::srgba(0.3, 0.45, 0.7, 0.4)),
            BorderRadius::all(Val::Px(3.0)),
            BackgroundColor(segment_bg(back_steps, Interaction::None)),
        ))
        .with_children(|b| {
            b.spawn((
                Text::new(label),
                TextFont { font_size: 13.0, ..default() },
                TextColor(segment_fg(back_steps)),
            ));
        });
}

fn spawn_separator(parent: &mut ChildBuilder) {
    parent.spawn((
        // Marcar como segmento no-clicable para que el rebuild lo limpie también.
        BreadcrumbSegment { back_steps: None },
        Node { padding: UiRect::horizontal(Val::Px(2.0)), ..default() },
        Text::new(">"),
        TextFont { font_size: 13.0, ..default() },
        TextColor(Color::srgba(0.6, 0.7, 0.9, 0.5)),
    ));
}

fn segment_bg(back_steps: Option<u8>, interaction: Interaction) -> Color {
    match (back_steps, interaction) {
        (None, _) => Color::srgba(0.25, 0.45, 0.9, 0.35),            // current (no clic)
        (Some(_), Interaction::Pressed) => Color::srgba(0.35, 0.55, 0.95, 0.55),
        (Some(_), Interaction::Hovered) => Color::srgba(0.2, 0.35, 0.65, 0.45),
        (Some(_), Interaction::None) => Color::srgba(0.1, 0.15, 0.25, 0.3),
    }
}

fn segment_fg(back_steps: Option<u8>) -> Color {
    match back_steps {
        None => Color::srgb(1.0, 1.0, 1.0),
        Some(_) => Color::srgb(0.85, 0.92, 1.0),
    }
}

fn style_breadcrumb_segments(
    mut q: Query<
        (&Interaction, &BreadcrumbSegment, &mut BackgroundColor),
        (Changed<Interaction>, With<Button>),
    >,
) {
    for (interaction, seg, mut bg) in &mut q {
        let color = segment_bg(seg.back_steps, *interaction);
        if bg.0 != color { bg.0 = color; }
    }
}

fn handle_breadcrumb_clicks(
    interactions: Query<(&Interaction, &BreadcrumbSegment), (Changed<Interaction>, With<Button>)>,
    mut state: ResMut<ViewerState>,
    mut zoom_out_ev: EventWriter<ZoomOutEvent>,
) {
    if state.scene.is_transitioning() { return; }
    for (interaction, seg) in interactions.iter() {
        if *interaction != Interaction::Pressed { continue; }
        let Some(steps) = seg.back_steps else { continue; };
        for _ in 0..steps {
            if state.breadcrumb.pop().is_some() {
                zoom_out_ev.send(ZoomOutEvent);
            }
        }
    }
}

// ─── HUD ───────────────────────────────────────────────────────────────────

fn setup_comparison_panel(mut commands: Commands) {
    commands.spawn((
        ComparisonPanel,
        Text::new(""),
        TextFont { font_size: 13.0, ..default() },
        TextColor(Color::srgb(1.0, 0.9, 0.7)),
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(10.0),
            right: Val::Px(10.0),
            width: Val::Px(360.0),
            padding: UiRect::all(Val::Px(8.0)),
            ..default()
        },
        BackgroundColor(Color::srgba(0.05, 0.05, 0.1, 0.65)),
        BorderColor(Color::srgba(0.8, 0.6, 0.3, 0.5)),
        BorderRadius::all(Val::Px(4.0)),
        Visibility::Hidden,
    ));
}

fn update_comparison_panel(
    state: Res<ViewerState>,
    mgr: Res<ScaleManager>,
    log: Res<MultiverseLog>,
    mut q: Query<(&mut Text, &mut Visibility), With<ComparisonPanel>>,
) {
    let Ok((mut text, mut vis)) = q.get_single_mut() else { return; };
    let desired = if state.comparison_open { Visibility::Visible } else { Visibility::Hidden };
    if *vis != desired { *vis = desired; }
    if !state.comparison_open { return; }

    **text = render_comparison(&mgr, &log);
}

fn render_comparison(mgr: &ScaleManager, log: &MultiverseLog) -> String {
    let observed = mgr.observed;
    let Some(current_inst) = mgr.get(observed) else {
        return "COMPARISON [C]\n(no active branch)".to_string();
    };
    let bandwidth = COHERENCE_BANDWIDTH as f64;
    let current = BranchSnapshot::from_instance(current_inst, bandwidth);
    let parent_id = current_inst.parent_entity_id.unwrap_or(ROOT_PARENT_ID);
    let current_seed = current_inst.zoom_seed;

    let prev = log.most_recent_for(parent_id, observed);
    let life_prob = log.life_probability(parent_id, observed);
    let n_branches = log.branches_for(parent_id, observed).count();
    let summary = log.summary();

    let prev_block = match prev {
        Some(b) => format!(
            "◀ PREVIOUS BRANCH (seed={})\n\
             qe:          {:>10.2}\n\
             entities:    {}\n\
             species:     {}\n\
             has_life:    {}\n\
             q_folding:   {:.3}",
            b.seed, b.snapshot.total_qe, b.snapshot.n_entities,
            b.snapshot.species_count, b.snapshot.has_life, b.snapshot.max_q_folding,
        ),
        None => "◀ PREVIOUS BRANCH\n(no prior branch for this parent+scale)".to_string(),
    };

    format!(
        "COMPARISON VIEW [C]\n\
         Parent: #{parent_id}  Scale: {:?}\n\
         ────────────────────────────────────\n\
         ▶ CURRENT BRANCH (seed={current_seed})\n\
         qe:          {:>10.2}\n\
         entities:    {}\n\
         species:     {}\n\
         has_life:    {}\n\
         q_folding:   {:.3}\n\
         ────────────────────────────────────\n\
         {prev_block}\n\
         ────────────────────────────────────\n\
         EMERGENT OBSERVABLES (same cluster)\n\
         branches:       {n_branches}\n\
         life_prob:      {:.2}\n\
         ────────────────────────────────────\n\
         MULTIVERSE TOTALS\n\
         branches:       {}\n\
         life_ratio:     {:.2}\n\
         mean_qe:        {:>10.2}\n\
         mean_species:   {:.2}\n\
         mean_q_folding: {:.3}",
        observed,
        current.total_qe, current.n_entities, current.species_count,
        current.has_life, current.max_q_folding,
        life_prob,
        summary.n_branches, summary.life_ratio, summary.mean_qe,
        summary.mean_species_count, summary.mean_q_folding,
    )
}

fn update_hud(
    mgr: Res<ScaleManager>,
    state: Res<ViewerState>,
    mut q: Query<&mut Text, With<HudText>>,
) {
    let Ok(mut text) = q.get_single_mut() else { return; };
    let observed = mgr.observed;
    let inst = mgr.get(observed);
    let entities = inst.map(|i| i.world.n_alive()).unwrap_or(0);
    let scale_qe: f64 = inst.map(|i| i.world.total_qe()).unwrap_or(0.0);
    let total_qe = mgr.total_qe_across_scales();
    let age = inst.map(|i| i.world.tick_id).unwrap_or(0);

    let label = scale_label(observed);
    let tag = scale_short(observed);
    let paused = if state.paused { " [PAUSED]" } else { "" };
    let phase = phase_label(state.scene);

    **text = format!(
        "COSMIC TELESCOPE{paused}\n\
         ────────────────────────────────────────\n\
         Scale:       {label} ({tag})\n\
         Seed:        {}\n\
         Universe age (ticks @ scale): {age}\n\
         Scale qe:    {scale_qe:>10.2}\n\
         Total qe:    {total_qe:>10.2}\n\
         Entities:    {entities}\n\
         Phase:       {phase}\n\
         ────────────────────────────────────────\n\
         {}\n\
         ────────────────────────────────────────\n\
         [Click entity] zoom-in   [Esc] zoom-out\n\
         [1-5] jump scale  [Space] pause  [Tab] cycle-seed\n\
         [C] comparison panel     [RMB drag] orbit  [Scroll] distance\n\
         Breadcrumb bar (below): click any step to return",
        mgr.universe_seed,
        scale_map(&mgr),
    );
}

fn phase_label(p: ScenePhase) -> &'static str {
    match p {
        ScenePhase::Empty => "empty",
        ScenePhase::Steady { .. } => "steady",
        ScenePhase::FadeOut { .. } => "fade-out",
        ScenePhase::FadeIn { .. } => "fade-in",
    }
}

fn scale_map(mgr: &ScaleManager) -> String {
    use std::fmt::Write;
    let mut out = String::with_capacity(64);
    for (i, lvl) in ALL_SCALES.iter().enumerate() {
        let marker = if *lvl == mgr.observed { "●" }
        else if mgr.has(*lvl) { "◉" }
        else { "○" };
        let _ = write!(out, "[{}]{} {}  ", i + 1, scale_short(*lvl), marker);
    }
    out
}
