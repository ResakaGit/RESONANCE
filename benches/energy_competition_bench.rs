use bevy::prelude::*;
use criterion::{criterion_group, criterion_main, Criterion};
use resonance::layers::{BaseEnergy, EnergyPool, ExtractionType, PoolParentLink};
use resonance::simulation::competition_dynamics::competition_dynamics_system;
use resonance::simulation::metabolic::pool_conservation::pool_conservation_system;
use resonance::simulation::metabolic::pool_distribution::{
    pool_dissipation_system, pool_distribution_system, pool_intake_system,
};
use resonance::simulation::metabolic::scale_composition::scale_composition_system;

fn make_bench_app() -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_systems(Update, (
        pool_intake_system,
        pool_distribution_system.after(pool_intake_system),
        pool_dissipation_system.after(pool_distribution_system),
        pool_conservation_system.after(pool_dissipation_system),
        competition_dynamics_system.after(pool_dissipation_system),
        scale_composition_system.after(pool_conservation_system),
    ));
    app
}

/// Benchmark: 100 pools × 10 hijos cada uno (1000 entidades totales).
/// Mide tiempo por tick del pipeline EC completo.
/// Target: < 1ms por tick.
fn bench_pool_distribution(c: &mut Criterion) {
    let mut app = make_bench_app();

    {
        for i in 0..100u32 {
            let x = (i % 10) as f32 * 4.0;
            let y = (i / 10) as f32 * 4.0;
            let mut commands = app.world_mut().commands();
            let parent = commands.spawn(EnergyPool::new(1000.0, 5000.0, 50.0, 0.001)).id();
            drop(commands);

            let et_cycle = [
                ExtractionType::Proportional,
                ExtractionType::Greedy,
                ExtractionType::Competitive,
                ExtractionType::Aggressive,
                ExtractionType::Regulated,
            ];
            for j in 0..10u32 {
                let etype = et_cycle[(j % 5) as usize];
                let param  = match etype {
                    ExtractionType::Proportional => 0.0,
                    ExtractionType::Greedy       => 80.0,
                    ExtractionType::Competitive  => 0.5,
                    ExtractionType::Aggressive   => 0.3,
                    ExtractionType::Regulated    => 30.0,
                };
                let mut commands = app.world_mut().commands();
                commands.spawn((
                    Transform::from_translation(Vec3::new(x, y, j as f32)),
                    Visibility::default(),
                    BaseEnergy::new(0.0),
                    PoolParentLink::new(parent, etype, param),
                ));
            }
        }
    }

    // Pre-warm: 3 ticks para insertar PoolConservationLedger + PoolDiagnostic via commands.
    app.update();
    app.update();
    app.update();

    c.bench_function("ec_pipeline_100_pools_10_children", |b| {
        b.iter(|| {
            app.update();
        });
    });
}

/// Benchmark: competition_matrix para N=16 (stack-allocated).
fn bench_competition_matrix(c: &mut Criterion) {
    use resonance::blueprint::equations::competition_matrix;

    let extractions: [f32; 16] = [
        200.0, 150.0, 100.0, 80.0, 70.0, 60.0, 50.0, 40.0,
        30.0, 25.0, 20.0, 15.0, 10.0, 8.0, 5.0, 2.0,
    ];
    let available = 1000.0_f32;

    c.bench_function("competition_matrix_n16", |b| {
        b.iter(|| {
            let m = competition_matrix(&extractions, available);
            criterion::black_box(m[0][0])
        });
    });
}

/// Benchmark: single extraction step (hot path — available.min(demand)).
fn bench_pool_extraction(c: &mut Criterion) {
    c.bench_function("pool_extraction_single", |b| {
        b.iter(|| {
            let available = criterion::black_box(1000.0_f32);
            let demand    = criterion::black_box(150.0_f32);
            criterion::black_box(available.min(demand))
        })
    });
}

criterion_group!(benches, bench_pool_distribution, bench_competition_matrix, bench_pool_extraction);
criterion_main!(benches);
