//! Benchmarks multi-escala del motor de simulación Resonance.
//!
//! Tabla de umbrales de referencia (hardware: developer laptop, 2026):
//! | Escala  | Entidades | Sistema        | Objetivo  |
//! |---------|-----------|----------------|-----------|
//! | Micro   | 1+3       | Pipeline EC    | < 10 µs   |
//! | Meso    | 10+100    | Pipeline EC    | < 100 µs  |
//! | Macro   | 100+1000  | Distribution   | < 1 ms    |

use bevy::prelude::*;
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use resonance::blueprint::equations::macro_analytics::{
    euler_vs_exponential_error, exponential_decay,
};
use resonance::layers::{BaseEnergy, EnergyPool, ExtractionType, PoolParentLink};
use resonance::simulation::competition_dynamics::competition_dynamics_system;
use resonance::simulation::metabolic::pool_conservation::pool_conservation_system;
use resonance::simulation::metabolic::pool_distribution::{
    pool_dissipation_system, pool_distribution_system, pool_intake_system,
};
use resonance::simulation::metabolic::scale_composition::scale_composition_system;

// ─── Tipos de extracción variados ────────────────────────────────────────────

const ET_CYCLE: [ExtractionType; 5] = [
    ExtractionType::Proportional,
    ExtractionType::Greedy,
    ExtractionType::Competitive,
    ExtractionType::Aggressive,
    ExtractionType::Regulated,
];

fn param_for(etype: ExtractionType) -> f32 {
    match etype {
        ExtractionType::Proportional => 0.0,
        ExtractionType::Greedy       => 80.0,
        ExtractionType::Competitive  => 0.5,
        ExtractionType::Aggressive   => 0.3,
        ExtractionType::Regulated    => 30.0,
    }
}

// ─── App factory ─────────────────────────────────────────────────────────────

/// Crea una App con el pipeline EC completo registrado en Update.
fn make_ec_pipeline_app() -> App {
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

/// Crea una App con solo pool_distribution_system (hot path macro).
fn make_distribution_only_app() -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_systems(Update, (
        pool_intake_system,
        pool_distribution_system.after(pool_intake_system),
    ));
    app
}

// ─── Spawn helpers ────────────────────────────────────────────────────────────

/// Spawna `n_pools` pools, cada uno con `children_per_pool` hijos con tipos variados.
fn spawn_pools_with_children(app: &mut App, n_pools: u32, children_per_pool: u32) {
    for i in 0..n_pools {
        let x = (i % 10) as f32 * 4.0;
        let y = (i / 10) as f32 * 4.0;
        let _ = (x, y); // posición no relevante para EC puro

        let mut commands = app.world_mut().commands();
        let parent = commands.spawn(EnergyPool::new(1000.0, 5000.0, 50.0, 0.001)).id();
        drop(commands);

        for j in 0..children_per_pool {
            let etype = ET_CYCLE[(j % 5) as usize];
            let param  = param_for(etype);
            let mut commands = app.world_mut().commands();
            commands.spawn((
                BaseEnergy::new(0.0),
                PoolParentLink::new(parent, etype, param),
            ));
        }
    }
}

// ─── Micro: 1 pool + 3 hijos ─────────────────────────────────────────────────

/// Micro: 1 pool + 3 hijos con tipos de extracción variados.
/// Pipeline EC completo: intake → distribution → dissipation → conservation.
fn bench_micro_1_pool_3_children(c: &mut Criterion) {
    let mut app = make_ec_pipeline_app();
    spawn_pools_with_children(&mut app, 1, 3);

    // Warmup: 3 ticks para insertar PoolConservationLedger via commands.
    app.update();
    app.update();
    app.update();

    c.bench_function("micro_1_pool_3_children", |b| {
        b.iter(|| {
            app.update();
        });
    });
}

// ─── Meso: 10 pools + 10 hijos cada uno (100 hijos total) ────────────────────

/// Meso: 10 pools × 10 hijos = 100 hijos total. Pipeline EC completo.
fn bench_meso_10_pools_100_children(c: &mut Criterion) {
    let mut app = make_ec_pipeline_app();
    spawn_pools_with_children(&mut app, 10, 10);

    app.update();
    app.update();
    app.update();

    c.bench_function("meso_10_pools_100_children", |b| {
        b.iter(|| {
            app.update();
        });
    });
}

// ─── Macro: 100 pools + 10 hijos cada uno (1000 hijos total) ─────────────────

/// Macro: 100 pools × 10 hijos = 1000 hijos total.
/// Solo pool_distribution_system (el más costoso del EC).
fn bench_macro_100_pools_1000_children(c: &mut Criterion) {
    let mut app = make_distribution_only_app();
    spawn_pools_with_children(&mut app, 100, 10);

    app.update();
    app.update();
    app.update();

    c.bench_function("macro_100_pools_1000_children", |b| {
        b.iter(|| {
            app.update();
        });
    });
}

// ─── M5: macro-step exponential vs Euler iteration ───────────────────────────

/// M5-A: O(1) closed-form exponential decay over 100 ticks.
fn bench_exponential_decay(c: &mut Criterion) {
    c.bench_function("macro_step_exponential_decay_100ticks", |b| {
        b.iter(|| exponential_decay(black_box(1000.0), black_box(0.01), black_box(100)))
    });
}

/// M5-B: 100 Euler iterations (tick-by-tick loop equivalent).
fn bench_euler_simulation(c: &mut Criterion) {
    c.bench_function("euler_100_steps_manual", |b| {
        b.iter(|| {
            let mut qe = black_box(1000.0_f32);
            let rate = black_box(0.01_f32);
            for _ in 0..100 {
                qe -= qe * rate;
            }
            black_box(qe)
        })
    });
}

/// M5-C: Relative error between 100-tick Euler and continuous exact solution.
fn bench_euler_error(c: &mut Criterion) {
    c.bench_function("euler_vs_exponential_error_100ticks", |b| {
        b.iter(|| {
            euler_vs_exponential_error(black_box(1000.0), black_box(0.01), black_box(100))
        })
    });
}

// ─── Registration ─────────────────────────────────────────────────────────────

criterion_group!(
    benches,
    bench_micro_1_pool_3_children,
    bench_meso_10_pools_100_children,
    bench_macro_100_pools_1000_children,
    bench_exponential_decay,
    bench_euler_simulation,
    bench_euler_error,
);
criterion_main!(benches);
