//! C1: Fossil Record — timeline of evolved morphology with slider.
//!
//! Usage: `cargo run --release --bin fossil_record -- --gens 200 --worlds 200`

use bevy::prelude::*;
use resonance::batch::bridge;
use resonance::blueprint::equations;
use resonance::blueprint::equations::radial_field;
use resonance::geometry_flow::creature_builder;
use resonance::use_cases::cli::{archetype_label, parse_arg};
use resonance::use_cases::experiments::fossil::{self, FossilRecord};

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let gens = parse_arg(&args, "--gens", 200);
    let worlds = parse_arg(&args, "--worlds", 200);
    let ticks = parse_arg(&args, "--ticks", 500);
    let seed = parse_arg(&args, "--seed", 42);

    println!("╔══════════════════════════════════════════╗");
    println!("║  RESONANCE — Fossil Record                ║");
    println!("╚══════════════════════════════════════════╝\n");

    let record = fossil::run(
        &resonance::use_cases::presets::EARTH,
        seed as u64,
        worlds as usize,
        gens as u32,
        ticks as u32,
    );
    resonance::use_cases::presenters::terminal::print_fossil(&record);

    if record.fossils.is_empty() {
        println!("\n  No fossils captured.\n");
        return;
    }

    println!("\n  Launching timeline viewer (LEFT/RIGHT to navigate)...\n");

    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Resonance — Fossil Record".to_string(),
                resolution: (1280.0, 720.0).into(),
                ..default()
            }),
            ..default()
        }))
        .insert_resource(Timeline {
            record,
            current: 0,
            dirty: true,
        })
        .add_systems(Startup, setup_scene)
        .add_systems(Update, (slider_input, rebuild_creature, update_hud))
        .run();
}

#[derive(Resource)]
struct Timeline {
    record: FossilRecord,
    current: usize,
    dirty: bool,
}

#[derive(Component)]
struct CreatureMesh;

#[derive(Component)]
struct HudText;

fn setup_scene(mut commands: Commands) {
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(0.0, 5.0, 10.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));
    commands.spawn((
        DirectionalLight {
            illuminance: 12000.0,
            shadows_enabled: true,
            ..default()
        },
        Transform::from_rotation(Quat::from_euler(EulerRot::XYZ, -0.7, 0.4, 0.0)),
    ));
    commands.insert_resource(AmbientLight {
        color: Color::srgb(0.3, 0.3, 0.4),
        brightness: 500.0,
    });

    // HUD
    commands.spawn((
        HudText,
        Text::new("Gen 0"),
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(10.0),
            left: Val::Px(10.0),
            ..default()
        },
    ));
}

fn slider_input(keys: Res<ButtonInput<KeyCode>>, mut tl: ResMut<Timeline>) {
    let max = tl.record.fossils.len().saturating_sub(1);
    let prev = tl.current;
    if keys.just_pressed(KeyCode::ArrowRight) {
        tl.current = (tl.current + 1).min(max);
    }
    if keys.just_pressed(KeyCode::ArrowLeft) {
        tl.current = tl.current.saturating_sub(1);
    }
    // Jump 10 with shift
    if keys.pressed(KeyCode::ShiftLeft) {
        if keys.just_pressed(KeyCode::ArrowRight) {
            tl.current = (tl.current + 10).min(max);
        }
        if keys.just_pressed(KeyCode::ArrowLeft) {
            tl.current = tl.current.saturating_sub(10);
        }
    }
    if tl.current != prev {
        tl.dirty = true;
    }
}

fn rebuild_creature(
    mut commands: Commands,
    mut tl: ResMut<Timeline>,
    existing: Query<Entity, With<CreatureMesh>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    if !tl.dirty {
        return;
    }
    tl.dirty = false;

    // Despawn old
    for e in &existing {
        commands.entity(e).despawn();
    }

    let Some(fossil) = tl.record.fossils.get(tl.current) else {
        return;
    };
    let genome = &fossil.genome;
    let freq = bridge::genome_to_components(genome).2.frequency_hz();
    let qe = 20.0 + genome.growth_bias * 80.0;

    let field = radial_field::build_viewer_field(
        genome.growth_bias,
        genome.resilience,
        genome.branching_bias,
        qe,
    );
    let freq_field = radial_field::build_viewer_freq_field(freq);

    let mesh = creature_builder::build_creature_mesh_with_field(
        genome.growth_bias,
        genome.mobility_bias,
        genome.branching_bias,
        genome.resilience,
        freq,
        &field,
        &freq_field,
    );

    let tint = equations::frequency_to_tint_rgb(freq);
    let base_color = Color::srgb(
        tint[0].clamp(0.05, 0.95),
        tint[1].clamp(0.05, 0.95),
        tint[2].clamp(0.05, 0.95),
    );
    let mat = materials.add(StandardMaterial {
        base_color,
        emissive: base_color.into(),
        perceptual_roughness: 0.3,
        double_sided: true,
        cull_mode: None,
        ..default()
    });

    commands.spawn((
        CreatureMesh,
        Mesh3d(meshes.add(mesh)),
        MeshMaterial3d(mat),
        Transform::from_xyz(0.0, 0.5, 0.0),
    ));
}

fn update_hud(tl: Res<Timeline>, mut text: Query<&mut Text, With<HudText>>) {
    let Some(fossil) = tl.record.fossils.get(tl.current) else {
        return;
    };
    for mut t in &mut text {
        *t = Text::new(format!(
            "Gen {}/{} | {} | fit={:.3} | div={:.3} | spp={:.1} | [LEFT/RIGHT]",
            fossil.generation,
            tl.record.fossils.len(),
            archetype_label(fossil.genome.archetype),
            fossil.fitness,
            fossil.diversity,
            fossil.species,
        ));
    }
}
