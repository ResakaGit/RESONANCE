//! Criterion benchmarks for batch simulator performance.
//!
//! Run: `cargo bench --bench batch_benchmark`

use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};

use resonance::batch::arena::{EntitySlot, SimWorldFlat};
use resonance::batch::batch::{BatchConfig, WorldBatch};
use resonance::batch::harness::GeneticHarness;
use resonance::batch::scratch::ScratchPad;
use resonance::blueprint::equations::determinism;

fn populate_demo(world: &mut SimWorldFlat, n: u8) {
    for i in 0..n {
        let seed = determinism::next_u64(42 ^ (i as u64));
        let mut e = EntitySlot::default();
        e.qe = 30.0;
        e.radius = 0.5;
        e.dissipation = 0.01;
        e.frequency_hz = determinism::range_f32(seed, 100.0, 900.0);
        e.engine_max = 20.0;
        e.input_valve = 0.5;
        e.output_valve = 0.5;
        e.archetype = 2;
        e.trophic_class = (i % 4) as u8;
        e.growth_bias = determinism::unit_f32(seed);
        e.mobility_bias = 0.5;
        e.resilience = 0.5;
        let s1 = determinism::next_u64(seed);
        let s2 = determinism::next_u64(s1);
        e.position = [
            determinism::range_f32(s1, 1.0, 15.0),
            determinism::range_f32(s2, 1.0, 15.0),
        ];
        world.spawn(e);
    }
    for cell in &mut world.nutrient_grid { *cell = 5.0; }
    world.update_total_qe();
}

fn bench_single_world_tick(c: &mut Criterion) {
    let mut world = SimWorldFlat::new(42, 0.05);
    populate_demo(&mut world, 32);
    let mut scratch = ScratchPad::new();
    c.bench_function("single_world_32ent", |b| {
        b.iter(|| world.tick(&mut scratch))
    });
}

fn bench_batch_tick(c: &mut Criterion) {
    let mut group = c.benchmark_group("batch_tick_parallel");
    group.sample_size(10);
    for &count in &[100, 1_000, 10_000] {
        let config = BatchConfig {
            world_count: count,
            initial_entities: 8,
            ..Default::default()
        };
        let mut batch = WorldBatch::new(config);
        group.bench_with_input(
            BenchmarkId::from_parameter(count),
            &count,
            |b, _| b.iter(|| batch.tick_all()),
        );
    }
    group.finish();
}

fn bench_batch_tick_sequential(c: &mut Criterion) {
    let mut group = c.benchmark_group("batch_tick_sequential");
    group.sample_size(10);
    for &count in &[100, 1_000] {
        let config = BatchConfig {
            world_count: count,
            initial_entities: 8,
            ..Default::default()
        };
        let mut batch = WorldBatch::new(config);
        group.bench_with_input(
            BenchmarkId::from_parameter(count),
            &count,
            |b, _| b.iter(|| batch.tick_all_sequential()),
        );
    }
    group.finish();
}

fn bench_genetic_step(c: &mut Criterion) {
    let config = BatchConfig {
        world_count: 100,
        ticks_per_eval: 50,
        initial_entities: 4,
        max_generations: 100,
        ..Default::default()
    };
    let mut harness = GeneticHarness::new(config);
    c.bench_function("genetic_step_100w_50t", |b| {
        b.iter(|| harness.step())
    });
}

criterion_group!(
    benches,
    bench_single_world_tick,
    bench_batch_tick,
    bench_batch_tick_sequential,
    bench_genetic_step,
);
criterion_main!(benches);
