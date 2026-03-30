//! Survival Mode — sobrevive como una criatura evolucionada.
//! Survival Mode — survive as an evolved creature.
//!
//! Carga genomes desde archivo o genera nuevos. Spawna arena con física completa.
//! El jugador controla UNA entidad con WASD. Score = ticks sobrevividos.
//! Todo vive en este binario. Zero modificaciones a src/.
//!
//! Usage:
//!   cargo run --release --bin survival -- --genomes assets/evolved/seed_42.bin
//!   cargo run --release --bin survival -- --seed 77 --gens 100

use bevy::prelude::*;

use resonance::batch::bridge;
use resonance::batch::genome::GenomeBlob;
use resonance::blueprint::equations;
use resonance::events::DeathEvent;
use resonance::geometry_flow::creature_builder;
use resonance::layers::{BaseEnergy, WillActuator};
use resonance::plugins::{
    DashboardBridgePlugin, DashboardPanelsPlugin, LayersPlugin, SimulationPlugin,
    SimulationTickPlugin,
};
use resonance::runtime_platform::dashboard_bridge::SimTickSummary;
use resonance::use_cases::cli::{find_arg, parse_arg};

// ─── Resources (local al binario) ───────────────────────────────────────────

/// Estado de la partida. Solo existe en este binario.
#[derive(Resource, Debug)]
struct SurvivalState {
    score:          u64,
    alive:          bool,
    player_entity:  Option<Entity>,
}

impl SurvivalState {
    fn new() -> Self { Self { score: 0, alive: true, player_entity: None } }
}

/// Genomes cargados para spawn.
#[derive(Resource)]
struct LoadedGenomes(Vec<GenomeBlob>);

// ─── Markers (local al binario) ─────────────────────────────────────────────

/// Marca la entidad controlada por el jugador. SparseSet.
#[derive(Component)]
#[component(storage = "SparseSet")]
struct Player;

/// Marca el texto del HUD.
#[derive(Component)]
struct HudText;

/// Marca la UI de game over.
#[derive(Component)]
struct GameOverUi;

// ─── Constants (visual calibration, no physics) ────────────────────────────

const VIEWER_QE_MIN: f32     = 20.0;  // minimum qe for stable mesh rendering
const VIEWER_QE_RANGE: f32   = 80.0;  // growth_bias × RANGE + MIN = visual qe
const ARENA_CREATURE_CAP: usize = 12; // max creatures in arena
const SPAWN_RING_BASE: f32   = 6.0;   // inner radius of spawn ring
const SPAWN_RING_STEP: f32   = 0.5;   // radius increment per creature
const SPAWN_Y_OFFSET: f32    = 0.1;   // lift above ground plane

// ─── States (local al binario) ──────────────────────────────────────────────

#[derive(States, Clone, Copy, PartialEq, Eq, Hash, Debug, Default)]
enum Phase {
    #[default]
    Playing,
    Dead,
}

// ─── Main ───────────────────────────────────────────────────────────────────

fn main() {
    let args: Vec<String> = std::env::args().collect();

    let genomes = load_or_evolve(&args);
    if genomes.is_empty() {
        eprintln!("No genomes available. Use --genomes <path> or --seed <n>.");
        std::process::exit(1);
    }

    println!("╔═══════════════════════════════════════════╗");
    println!("║  RESONANCE — Survival Mode                ║");
    println!("╠═══════════════════════════════════════════╣");
    println!("║  WASD to move. Survive as long as you can.║");
    println!("║  R to restart after death.                 ║");
    println!("╚═══════════════════════════════════════════╝\n");
    println!("  {} genomes loaded. You are creature #0.\n", genomes.len());

    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Resonance — Survival".into(),
                resolution: (1280.0, 720.0).into(),
                ..default()
            }),
            ..default()
        }))
        .add_plugins(SimulationTickPlugin)
        .add_plugins(LayersPlugin)
        .add_plugins(SimulationPlugin)
        .add_plugins(DashboardBridgePlugin)
        .add_plugins(DashboardPanelsPlugin)
        .init_state::<Phase>()
        .insert_resource(LoadedGenomes(genomes))
        .insert_resource(SurvivalState::new())
        .add_systems(Startup, (setup_scene, spawn_creatures).chain())
        .add_systems(FixedUpdate, (
            player_input,
            score_tick,
            detect_death,
        ).chain().run_if(in_state(Phase::Playing)))
        .add_systems(OnEnter(Phase::Dead), spawn_game_over_ui)
        .add_systems(Update, restart_on_r.run_if(in_state(Phase::Dead)))
        .add_systems(Update, update_hud)
        .run();
}

// ─── Genome loading (stateless) ─────────────────────────────────────────────

/// Carga genomes desde archivo o evoluciona nuevos. Función pura sobre args.
fn load_or_evolve(args: &[String]) -> Vec<GenomeBlob> {
    // Intentar cargar desde archivo
    if let Some(path) = find_arg(args, "--genomes") {
        match bridge::load_genomes(std::path::Path::new(&path)) {
            Ok(g) if !g.is_empty() => {
                println!("  Loaded {} genomes from {path}", g.len());
                return g;
            }
            Ok(_) => eprintln!("  Warning: {path} contains 0 genomes"),
            Err(e) => eprintln!("  Warning: failed to load {path}: {e}"),
        }
    }

    // Fallback: evolucionar
    let seed = parse_arg(args, "--seed", 42) as u64;
    let gens = parse_arg(args, "--gens", 100) as u32;
    let worlds = parse_arg(args, "--worlds", 200) as usize;
    println!("  Evolving {worlds} worlds × {gens} gens (seed={seed})...");

    use resonance::batch::batch::BatchConfig;
    use resonance::batch::harness::GeneticHarness;

    let config = BatchConfig {
        world_count: worlds,
        max_generations: gens,
        seed,
        initial_entities: 12,
        ..Default::default()
    };
    let mut harness = GeneticHarness::new(config);
    harness.run()
}

// ─── Startup systems ────────────────────────────────────────────────────────

fn setup_scene(mut commands: Commands) {
    // Cámara cenital (follow-cam en Update sería mejor, pero esto es MVP)
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(0.0, 15.0, 12.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));
    commands.spawn((
        DirectionalLight {
            illuminance: 12000.0,
            shadows_enabled: true,
            ..default()
        },
        Transform::from_rotation(Quat::from_euler(EulerRot::XYZ, -0.8, 0.3, 0.0)),
    ));
    commands.insert_resource(AmbientLight {
        color: Color::srgb(0.3, 0.3, 0.4),
        brightness: 400.0,
    });

    // HUD
    commands.spawn((
        HudText,
        Text::new("Score: 0"),
        TextFont { font_size: 28.0, ..default() },
        TextColor(Color::WHITE),
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(10.0),
            left: Val::Px(10.0),
            ..default()
        },
    ));
}

fn spawn_creatures(
    mut commands: Commands,
    genomes: Res<LoadedGenomes>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut state: ResMut<SurvivalState>,
) {
    // Ground
    commands.spawn((
        Mesh3d(meshes.add(Plane3d::default().mesh().size(40.0, 40.0))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::srgb(0.08, 0.12, 0.06),
            perceptual_roughness: 0.95,
            ..default()
        })),
    ));

    let count = genomes.0.len().min(ARENA_CREATURE_CAP);
    for (i, genome) in genomes.0.iter().take(count).enumerate() {
        let angle = (i as f32 / count as f32) * std::f32::consts::TAU;
        let radius = SPAWN_RING_BASE + (i as f32 * SPAWN_RING_STEP);
        let x = angle.cos() * radius;
        let z = angle.sin() * radius;

        let (mesh_handle, mat_handle) = build_creature_visuals(genome, &mut meshes, &mut materials);
        let entity = commands.spawn((
            Mesh3d(mesh_handle),
            MeshMaterial3d(mat_handle),
            Transform::from_xyz(x, SPAWN_Y_OFFSET, z),
        )).id();

        // Marcar la primera criatura como player
        if i == 0 {
            commands.entity(entity).insert(Player);
            state.player_entity = Some(entity);
        }
    }
}

/// Construye mesh + material desde genome. Stateless.
/// Pattern duplicado de evolve_and_view.rs por diseño (binario standalone, zero coupling).
fn build_creature_visuals(
    genome: &GenomeBlob,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
) -> (Handle<Mesh>, Handle<StandardMaterial>) {
    let freq = bridge::genome_to_components(genome).2.frequency_hz();
    let qe = VIEWER_QE_MIN + genome.growth_bias * VIEWER_QE_RANGE;

    use resonance::blueprint::equations::radial_field;
    let field = radial_field::build_viewer_field(
        genome.growth_bias, genome.resilience, genome.branching_bias, qe,
    );
    let freq_field = radial_field::build_viewer_freq_field(freq);

    let mesh = creature_builder::build_creature_mesh_with_field(
        genome.growth_bias, genome.mobility_bias,
        genome.branching_bias, genome.resilience, freq,
        &field, &freq_field,
    );

    let tint = equations::frequency_to_tint_rgb(freq);
    let base_color = Color::srgb(
        tint[0].clamp(0.05, 0.95),
        tint[1].clamp(0.05, 0.95),
        tint[2].clamp(0.05, 0.95),
    );

    (
        meshes.add(mesh),
        materials.add(StandardMaterial {
            base_color,
            emissive: base_color.into(),
            perceptual_roughness: 0.3 + (1.0 - genome.resilience) * 0.5,
            metallic: genome.mobility_bias * 0.3,
            double_sided: true,
            cull_mode: None,
            ..default()
        }),
    )
}

// ─── Gameplay systems (FixedUpdate, Playing only) ───────────────────────────

/// Input WASD → WillActuator. Solo para la entidad Player.
fn player_input(
    keys: Res<ButtonInput<KeyCode>>,
    mut query: Query<&mut WillActuator, With<Player>>,
) {
    let Ok(mut will) = query.get_single_mut() else { return };
    let mut dir = Vec2::ZERO;
    if keys.pressed(KeyCode::KeyW) || keys.pressed(KeyCode::ArrowUp)    { dir.y += 1.0; }
    if keys.pressed(KeyCode::KeyS) || keys.pressed(KeyCode::ArrowDown)  { dir.y -= 1.0; }
    if keys.pressed(KeyCode::KeyA) || keys.pressed(KeyCode::ArrowLeft)  { dir.x -= 1.0; }
    if keys.pressed(KeyCode::KeyD) || keys.pressed(KeyCode::ArrowRight) { dir.x += 1.0; }
    will.set_movement_intent(dir.normalize_or_zero());
}

/// Score = ticks sobrevividos. Solo corre en Phase::Playing (run_if guard).
fn score_tick(mut state: ResMut<SurvivalState>) {
    state.score += 1;
}

/// Transiciona a muerte. Idempotente.
fn trigger_death(state: &mut SurvivalState, next_phase: &mut NextState<Phase>) {
    state.alive = false;
    next_phase.set(Phase::Dead);
}

/// Detecta muerte del player via DeathEvent o energía agotada.
fn detect_death(
    mut events: EventReader<DeathEvent>,
    player_q: Query<(Entity, &BaseEnergy), With<Player>>,
    mut state: ResMut<SurvivalState>,
    mut next_phase: ResMut<NextState<Phase>>,
) {
    if !state.alive { return; }
    let Some(player_entity) = state.player_entity else { return };

    // DeathEvent emitido por EnergyOps
    if events.read().any(|ev| ev.entity == player_entity) {
        trigger_death(&mut state, &mut next_phase);
        return;
    }

    // Fallback: entity despawned o energía agotada
    match player_q.get(player_entity) {
        Ok((_, energy)) if energy.qe() <= 0.0 => trigger_death(&mut state, &mut next_phase),
        Err(_) => trigger_death(&mut state, &mut next_phase),
        _ => {}
    }
}

// ─── HUD ────────────────────────────────────────────────────────────────────

fn update_hud(
    state: Res<SurvivalState>,
    summary: Res<SimTickSummary>,
    player_q: Query<&BaseEnergy, With<Player>>,
    mut text_q: Query<&mut Text, With<HudText>>,
) {
    let Ok(mut text) = text_q.get_single_mut() else { return };
    let player_qe = player_q.get_single().map(|e| e.qe()).unwrap_or(0.0);
    let status = if state.alive { "ALIVE" } else { "DEAD" };
    **text = format!(
        "Score: {} | qe: {:.1} | {status} | pop: {} | world qe: {:.0}",
        state.score, player_qe, summary.alive_count, summary.total_qe,
    );
}

// ─── Game Over (Phase::Dead) ────────────────────────────────────────────────

fn spawn_game_over_ui(mut commands: Commands, state: Res<SurvivalState>) {
    commands.spawn((
        GameOverUi,
        StateScoped(Phase::Dead),
        Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            flex_direction: FlexDirection::Column,
            ..default()
        },
        BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.7)),
    )).with_children(|parent| {
        parent.spawn((
            Text::new("GAME OVER"),
            TextFont { font_size: 72.0, ..default() },
            TextColor(Color::srgb(0.9, 0.2, 0.2)),
        ));
        parent.spawn((
            Text::new(format!("Score: {}", state.score)),
            TextFont { font_size: 36.0, ..default() },
            TextColor(Color::WHITE),
        ));
        parent.spawn((
            Text::new("Press R to restart"),
            TextFont { font_size: 24.0, ..default() },
            TextColor(Color::srgb(0.6, 0.6, 0.6)),
        ));
    });
}

fn restart_on_r(
    keys: Res<ButtonInput<KeyCode>>,
    mut state: ResMut<SurvivalState>,
    mut next_phase: ResMut<NextState<Phase>>,
) {
    if keys.just_pressed(KeyCode::KeyR) {
        state.score = 0;
        state.alive = true;
        state.player_entity = None;
        next_phase.set(Phase::Playing);
    }
}
