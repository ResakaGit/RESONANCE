//! C2: Petri Dish — click a creature to see its 16×8 radial energy field as heatmap.
//!
//! Usage: `cargo run --release --bin petri_dish -- --gens 100`

use bevy::prelude::*;
use resonance::batch::batch::BatchConfig;
use resonance::batch::bridge;
use resonance::batch::genome::GenomeBlob;
use resonance::batch::harness::GeneticHarness;
use resonance::blueprint::equations;
use resonance::blueprint::equations::radial_field;
use resonance::geometry_flow::creature_builder;
use resonance::use_cases::cli::{archetype_label, parse_arg};

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let gens = parse_arg(&args, "--gens", 100);
    let seed = parse_arg(&args, "--seed", 42);

    println!("╔══════════════════════════════════════════╗");
    println!("║  RESONANCE — Petri Dish                   ║");
    println!("╚══════════════════════════════════════════╝\n");

    let config = BatchConfig {
        world_count: 200,
        ticks_per_eval: 600,
        initial_entities: 16,
        max_generations: gens as u32,
        seed: seed as u64,
        ..Default::default()
    };
    let mut harness = GeneticHarness::new(config);
    let genomes = harness.run();
    println!(
        "  {} genomes evolved. Click a creature to inspect.\n",
        genomes.len()
    );

    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Resonance — Petri Dish (click to inspect)".to_string(),
                resolution: (1280.0, 720.0).into(),
                ..default()
            }),
            ..default()
        }))
        .insert_resource(EvolvedData(genomes))
        .insert_resource(SelectedCreature {
            index: None,
            dirty: true,
        })
        .add_systems(Startup, (setup_scene, spawn_creatures))
        .add_systems(Update, (cycle_selection, update_heatmap, update_hud))
        .run();
}

#[derive(Resource)]
struct EvolvedData(Vec<GenomeBlob>);

#[derive(Resource)]
struct SelectedCreature {
    index: Option<usize>,
    dirty: bool,
}

#[derive(Component)]
struct CreatureIndex(#[allow(dead_code)] usize);

#[derive(Component)]
struct HeatmapOverlay;

#[derive(Component)]
struct HudText;

fn setup_scene(mut commands: Commands) {
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(0.0, 12.0, 16.0).looking_at(Vec3::ZERO, Vec3::Y),
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
        Text::new("[TAB] to cycle creatures"),
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
    genomes: Res<EvolvedData>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // Ground
    commands.spawn((
        Mesh3d(meshes.add(Plane3d::default().mesh().size(50.0, 50.0))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::srgb(0.08, 0.12, 0.06),
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
            CreatureIndex(i),
            Mesh3d(meshes.add(mesh)),
            MeshMaterial3d(mat),
            Transform::from_xyz(x, 0.05, z),
        ));
    }
}

fn cycle_selection(
    keys: Res<ButtonInput<KeyCode>>,
    genomes: Res<EvolvedData>,
    mut sel: ResMut<SelectedCreature>,
) {
    if keys.just_pressed(KeyCode::Tab) {
        let max = genomes.0.len().min(10);
        let next = sel.index.map(|i| (i + 1) % max).unwrap_or(0);
        sel.index = Some(next);
        sel.dirty = true;
    }
}

fn update_heatmap(
    mut commands: Commands,
    genomes: Res<EvolvedData>,
    mut sel: ResMut<SelectedCreature>,
    existing: Query<Entity, With<HeatmapOverlay>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    if !sel.dirty {
        return;
    }
    sel.dirty = false;

    // Despawn old heatmap
    for e in &existing {
        commands.entity(e).despawn();
    }

    let Some(idx) = sel.index else {
        return;
    };
    let Some(genome) = genomes.0.get(idx) else {
        return;
    };

    let qe = 20.0 + genome.growth_bias * 80.0;
    let field = radial_field::build_viewer_field(
        genome.growth_bias,
        genome.resilience,
        genome.branching_bias,
        qe,
    );

    // Render field as 16×8 grid of colored cubes (heatmap)
    let max_qe = field
        .iter()
        .flatten()
        .copied()
        .fold(0.0f32, f32::max)
        .max(0.01);
    let cell_size = 0.25;
    let origin_x = 8.0;
    let origin_z = -4.0;

    for a in 0..radial_field::AXIAL {
        for r in 0..radial_field::RADIAL {
            let val = field[a][r] / max_qe;
            let color = Color::srgb(val, val * 0.3, (1.0 - val) * 0.5);
            let x = origin_x + a as f32 * cell_size;
            let z = origin_z + r as f32 * cell_size;
            let y = val * 2.0;

            commands.spawn((
                HeatmapOverlay,
                Mesh3d(meshes.add(Cuboid::new(cell_size * 0.9, y.max(0.05), cell_size * 0.9))),
                MeshMaterial3d(materials.add(StandardMaterial {
                    base_color: color,
                    emissive: color.into(),
                    ..default()
                })),
                Transform::from_xyz(x, y * 0.5, z),
            ));
        }
    }
}

fn update_hud(
    sel: Res<SelectedCreature>,
    genomes: Res<EvolvedData>,
    mut text: Query<&mut Text, With<HudText>>,
) {
    let Some(idx) = sel.index else {
        return;
    };
    let Some(g) = genomes.0.get(idx) else {
        return;
    };
    for mut t in &mut text {
        *t = Text::new(format!(
            "#{} {} | g={:.2} m={:.2} b={:.2} r={:.2} | [TAB] next",
            idx,
            archetype_label(g.archetype),
            g.growth_bias,
            g.mobility_bias,
            g.branching_bias,
            g.resilience,
        ));
    }
}
