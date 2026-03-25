//! Benchmarks del campo energético worldgen: materialización, disipación, resolución de frecuencia.
//! `cargo bench -p resonance --bench worldgen_field_perf`

use bevy::math::Vec2;
use bevy::prelude::Entity;
use criterion::{Criterion, black_box, criterion_group, criterion_main};
use resonance::blueprint::AlchemicalAlmanac;
use resonance::layers::MatterState;
use resonance::topology::TerrainType;
use resonance::worldgen::{
    EnergyCell, EnergyFieldGrid, FIELD_CELL_SIZE, FIELD_DECAY_RATE, FrequencyContribution,
    field_dissipation, materialize_cell_at_time, resolve_dominant_frequency,
};

fn filled_cell() -> EnergyCell {
    let mut c = EnergyCell::default();
    c.accumulated_qe = 120.0;
    c.dominant_frequency_hz = 75.0;
    c.purity = 0.95;
    c.temperature = 50.0;
    c.matter_state = MatterState::Solid;
    c.frequency_contributions
        .push(FrequencyContribution::new(Entity::from_raw(1), 75.0, 80.0));
    c
}

fn bench_materialize_k_cells(c: &mut Criterion) {
    let almanac = AlchemicalAlmanac::default();
    let cells: Vec<EnergyCell> = (0..100).map(|_| filled_cell()).collect();
    c.bench_function("materialize_100_dirty_cells", |b| {
        b.iter(|| {
            for cell in &cells {
                black_box(materialize_cell_at_time(
                    black_box(cell),
                    black_box(&almanac),
                    black_box(0.42_f32),
                    black_box(FIELD_CELL_SIZE),
                    black_box(None::<TerrainType>),
                ));
            }
        });
    });
}

fn bench_dissipate_full_grid_100(c: &mut Criterion) {
    let mut grid = EnergyFieldGrid::new(100, 100, 1.0, Vec2::ZERO);
    for cell in grid.iter_cells_mut() {
        cell.accumulated_qe = 55.0;
    }
    let dt = 1.0_f32 / 60.0;
    c.bench_function("field_dissipation_scan_100x100", |b| {
        b.iter(|| {
            let mut acc = 0.0_f32;
            for cell in grid.iter_cells() {
                acc += field_dissipation(
                    black_box(cell.accumulated_qe),
                    FIELD_DECAY_RATE,
                    black_box(dt),
                );
            }
            black_box(acc);
        });
    });
}

fn bench_resolve_dominant_10k(c: &mut Criterion) {
    let cell = filled_cell();
    c.bench_function("resolve_dominant_frequency_x10000", |b| {
        b.iter(|| {
            for _ in 0..10_000 {
                black_box(resolve_dominant_frequency(black_box(
                    cell.frequency_contributions(),
                )));
            }
        });
    });
}

criterion_group!(
    benches,
    bench_materialize_k_cells,
    bench_dissipate_full_grid_100,
    bench_resolve_dominant_10k
);
criterion_main!(benches);
