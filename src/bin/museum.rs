//! C3: Museum Mode — fullscreen ecosystem, orbital camera, continuous evolution.
//!
//! Usage: `cargo run --release --bin museum`
//!
//! No UI. No input. Just an evolving ecosystem projected on screen.

use bevy::prelude::*;
use resonance::batch::batch::BatchConfig;
use resonance::batch::bridge;
use resonance::blueprint::equations;
use resonance::blueprint::equations::radial_field;
use resonance::geometry_flow::creature_builder;
use resonance::use_cases::cli::parse_arg;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let gens = parse_arg(&args, "--gens", 50);
    let seed = parse_arg(&args, "--seed", 42);

    println!("╔══════════════════════════════════════════╗");
    println!("║  RESONANCE — Museum Mode                  ║");
    println!("╚══════════════════════════════════════════╝\n");

    let config = BatchConfig {
        world_count: 200,
        ticks_per_eval: 600,
        initial_entities: 16,
        max_generations: gens as u32,
        seed: seed as u64,
        ..Default::default()
    };
    let mut harness = resonance::batch::harness::GeneticHarness::new(config);
    let genomes = harness.run();
    println!(
        "  {} genomes evolved. Launching exhibition...\n",
        genomes.len()
    );

    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Resonance — Museum Mode".to_string(),
                mode: bevy::window::WindowMode::BorderlessFullscreen(MonitorSelection::Primary),
                ..default()
            }),
            ..default()
        }))
        .insert_resource(EvolvedGenomes(genomes))
        .add_systems(Startup, (setup_scene, spawn_creatures))
        .add_systems(Update, orbit_camera)
        .run();
}

#[derive(Resource)]
struct EvolvedGenomes(Vec<resonance::batch::genome::GenomeBlob>);

fn setup_scene(mut commands: Commands) {
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(0.0, 12.0, 20.0).looking_at(Vec3::ZERO, Vec3::Y),
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
        color: Color::srgb(0.2, 0.2, 0.3),
        brightness: 400.0,
    });
    commands.insert_resource(ClearColor(Color::srgb(0.01, 0.01, 0.02)));
}

fn spawn_creatures(
    mut commands: Commands,
    genomes: Res<EvolvedGenomes>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    commands.spawn((
        Mesh3d(meshes.add(Plane3d::default().mesh().size(80.0, 80.0))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::srgb(0.03, 0.05, 0.02),
            perceptual_roughness: 0.98,
            ..default()
        })),
    ));

    let count = genomes.0.len().min(10);
    for (i, genome) in genomes.0.iter().take(count).enumerate() {
        let angle = (i as f32 / count as f32) * std::f32::consts::TAU;
        let radius = 5.0 + (i as f32 * 1.5);
        let x = angle.cos() * radius;
        let z = angle.sin() * radius;
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
            Mesh3d(meshes.add(mesh)),
            MeshMaterial3d(mat),
            Transform::from_xyz(x, 0.05, z),
        ));
    }
}

fn orbit_camera(time: Res<Time>, mut q: Query<&mut Transform, With<Camera3d>>) {
    for mut t in &mut q {
        let a = time.elapsed_secs() * 0.05;
        let r = 22.0 + (time.elapsed_secs() * 0.02).sin() * 5.0;
        t.translation = Vec3::new(a.cos() * r, 8.0 + (a * 0.3).sin() * 3.0, a.sin() * r);
        t.look_at(Vec3::new(0.0, 2.0, 0.0), Vec3::Y);
    }
}
