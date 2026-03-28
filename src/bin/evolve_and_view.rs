//! Evolve creatures in batch, visualize with inferred GF1 geometry.
//!
//! `cargo run --release --bin evolve_and_view`
//! `cargo run --release --bin evolve_and_view -- --gens 200 --worlds 500`
//!
//! Phase 1: Batch evolution (headless).
//! Phase 2: Bevy window — geometry from `creature_builder::build_creature_mesh`.
//!
//! The binary is a thin consumer. All logic lives in:
//! - `batch/harness.rs` — evolution loop
//! - `blueprint/equations/batch_fitness.rs` — genome→influence mapping (pure math)
//! - `geometry_flow/creature_builder.rs` — GF1 spine+mesh+branches (stateless builder)
//! - `batch/bridge.rs` — genome→components conversion

use bevy::prelude::*;
use resonance::batch::batch::BatchConfig;
use resonance::batch::bridge;
use resonance::batch::harness::GeneticHarness;
use resonance::blueprint::equations;
use resonance::geometry_flow::creature_builder;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let worlds = parse_arg(&args, "--worlds", 300);
    let gens   = parse_arg(&args, "--gens", 100);
    let ticks  = parse_arg(&args, "--ticks", 800);
    let seed   = parse_arg(&args, "--seed", 42);

    println!("╔══════════════════════════════════════════╗");
    println!("║  RESONANCE — Evolve & View (GF1)        ║");
    println!("╚══════════════════════════════════════════╝\n");
    println!("  worlds={worlds} gens={gens} ticks={ticks} seed={seed}\n");

    // ── Phase 1: Evolve ─────────────────────────────────────────────────────
    println!("Phase 1: Evolving...");
    let config = BatchConfig {
        world_count:      worlds as usize,
        ticks_per_eval:   ticks as u32,
        initial_entities: 16,
        max_generations:  gens as u32,
        seed:             seed as u64,
        ..Default::default()
    };
    let mut harness = GeneticHarness::new(config);
    let genomes = harness.run();

    println!("  {} genomes evolved.\n", genomes.len());
    for (i, g) in genomes.iter().enumerate() {
        let arch = match g.archetype {
            1 => "flora", 2 => "fauna", 3 => "cell", 4 => "virus", _ => "inert",
        };
        println!("  #{i:>2}: {arch:<5} g={:.2} m={:.2} b={:.2} r={:.2}",
            g.growth_bias, g.mobility_bias, g.branching_bias, g.resilience);
    }
    if let Some(last) = harness.history.last() {
        println!("\n  Final: fitness={:.3} diversity={:.3} species={:.1}",
            last.best_fitness, last.diversity, last.species_mean);
    }

    // ── Phase 2: Visualize ──────────────────────────────────────────────────
    println!("\nPhase 2: Rendering...\n");
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Resonance — Evolved Creatures (GF1)".to_string(),
                resolution: (1280.0, 720.0).into(),
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
        Transform::from_xyz(0.0, 10.0, 16.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));
    commands.spawn((
        DirectionalLight {
            illuminance: 15000.0,
            shadows_enabled: true,
            ..default()
        },
        Transform::from_rotation(Quat::from_euler(EulerRot::XYZ, -0.7, 0.4, 0.0)),
    ));
    commands.insert_resource(AmbientLight {
        color: Color::srgb(0.35, 0.35, 0.45),
        brightness: 500.0,
    });
}

/// Spawn evolved creatures. Thin — delegates all logic.
fn spawn_creatures(
    mut commands: Commands,
    genomes: Res<EvolvedGenomes>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // Ground
    commands.spawn((
        Mesh3d(meshes.add(Plane3d::default().mesh().size(50.0, 50.0))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::srgb(0.10, 0.15, 0.07),
            perceptual_roughness: 0.95,
            ..default()
        })),
    ));

    let count = genomes.0.len().min(10);
    for (i, genome) in genomes.0.iter().take(count).enumerate() {
        let angle = (i as f32 / count as f32) * std::f32::consts::TAU;
        let x = angle.cos() * 7.0;
        let z = angle.sin() * 7.0;
        let freq = bridge::genome_to_components(genome).2.frequency_hz();

        use resonance::blueprint::equations::radial_field;
        // Viewer normalization: growth ∈ [0,1] → qe ∈ [20, 100] for stable mesh rendering
        let qe = 20.0 + genome.growth_bias * 80.0;
        let field = radial_field::build_viewer_field(
            genome.growth_bias, genome.resilience, genome.branching_bias, qe,
        );
        let freq_field = radial_field::build_viewer_freq_field(freq);

        // Geometry: genome + field → creature_builder (desacoplado)
        let mesh = creature_builder::build_creature_mesh_with_field(
            genome.growth_bias, genome.mobility_bias,
            genome.branching_bias, genome.resilience, freq,
            &field, &freq_field,
        );
        let mesh_handle = meshes.add(mesh);

        // Material: frequency → tint (Axiom 8)
        let tint = equations::frequency_to_tint_rgb(freq);
        let base_color = Color::srgb(
            tint[0].clamp(0.05, 0.95),
            tint[1].clamp(0.05, 0.95),
            tint[2].clamp(0.05, 0.95),
        );
        let mat = materials.add(StandardMaterial {
            base_color,
            emissive: base_color.into(),
            perceptual_roughness: 0.3 + (1.0 - genome.resilience) * 0.5,
            metallic: genome.mobility_bias * 0.4,
            double_sided: true,
            cull_mode: None,
            ..default()
        });

        commands.spawn((Mesh3d(mesh_handle), MeshMaterial3d(mat), Transform::from_xyz(x, 0.05, z)));
    }
}

fn orbit_camera(time: Res<Time>, mut q: Query<&mut Transform, With<Camera3d>>) {
    for mut t in &mut q {
        let a = time.elapsed_secs() * 0.1;
        t.translation = Vec3::new(a.cos() * 18.0, 8.0, a.sin() * 18.0);
        t.look_at(Vec3::new(0.0, 1.5, 0.0), Vec3::Y);
    }
}

fn parse_arg(args: &[String], flag: &str, default: i64) -> i64 {
    args.windows(2)
        .find(|w| w[0] == flag)
        .and_then(|w| w[1].parse().ok())
        .unwrap_or(default)
}
