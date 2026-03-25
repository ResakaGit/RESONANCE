//! Benchmarks comparativos Bridge Optimizer (sprint B10).
//! `cargo bench -p resonance --bench bridge_benchmark --features bridge_optimizer`

use std::time::Instant;

use bevy::prelude::World;
use criterion::{BenchmarkId, Criterion, black_box, criterion_group, criterion_main};
use fxhash::FxHashMap;

use resonance::bridge::benchmark_harness::{
    BLUEPRINT_HIT_RATE_CATALYSIS, BLUEPRINT_HIT_RATE_DENSITY, BLUEPRINT_HIT_RATE_INTERFERENCE,
    BLUEPRINT_HIT_RATE_PHASE, BLUEPRINT_HIT_RATE_TEMPERATURE, bootstrap_interference_bridge_world,
    collect_benchmark_report, generate_benchmark_scenario, run_derived_chain_bridged,
    run_derived_chain_direct, set_bridge_phase, warm_derived_chain_bridged,
};
use resonance::bridge::bridged_ops::InterferenceEquationInput;
use resonance::bridge::cache::BridgeCache;
use resonance::bridge::clear_all_bridge_caches;
use resonance::bridge::config::{
    DensityBridge, InterferenceBridge, PhaseTransitionBridge, TemperatureBridge,
};
use resonance::bridge::context_fill::BridgePhase;
use resonance::bridge::decorator::{Bridgeable, bridge_compute};
use resonance::bridge::presets::{BridgeDefaults, RigidityPreset};

const TICKS_IN_PIPELINE_BENCH: usize = 32;

/// Comparación escalar (hint=None en todos los puentes):
/// - **bridged_all_hits:** en hit se evita `compute` pero se paga normalize + hash + lookup + LRU/stats.
/// - **normalized_pipeline_no_cache:** solo normalize + compute (equivalente al miss path sin insertar en cache).
/// No miden lo mismo que “raw equations” sin cuantizar; miden decorador+cache vs pipeline normalizado sin LRU.
fn bench_scalar_chain_hot_cache_vs_raw_equations(c: &mut Criterion) {
    let cfg_d = DensityBridge::config_for_preset(RigidityPreset::Moderate);
    let cfg_t = TemperatureBridge::config_for_preset(RigidityPreset::Moderate);
    let cfg_p = PhaseTransitionBridge::config_for_preset(RigidityPreset::Moderate);
    let mut cd = BridgeCache::<DensityBridge>::new(cfg_d.cache_capacity, cfg_d.policy);
    let mut ct = BridgeCache::<TemperatureBridge>::new(cfg_t.cache_capacity, cfg_t.policy);
    let mut cp = BridgeCache::<PhaseTransitionBridge>::new(cfg_p.cache_capacity, cfg_p.policy);

    let densities: Vec<f32> = (0..200)
        .map(|i| {
            let u = (i * 1103515245 + 12345) % 1000;
            12.0 + (u as f32) * 0.08
        })
        .collect();

    for &d in &densities {
        let d_b = bridge_compute::<DensityBridge>(d, &cfg_d, &mut cd);
        let t_b = bridge_compute::<TemperatureBridge>(d_b, &cfg_t, &mut ct);
        let _ = bridge_compute::<PhaseTransitionBridge>((t_b, 5000.0), &cfg_p, &mut cp);
    }

    let bond = 5000.0_f32;
    c.bench_function("scalar_chain_bridged_all_hits", |b| {
        b.iter(|| {
            for &d in black_box(&densities) {
                let d_b = bridge_compute::<DensityBridge>(d, black_box(&cfg_d), black_box(&mut cd));
                let t_b =
                    bridge_compute::<TemperatureBridge>(d_b, black_box(&cfg_t), black_box(&mut ct));
                black_box(bridge_compute::<PhaseTransitionBridge>(
                    (t_b, bond),
                    black_box(&cfg_p),
                    black_box(&mut cp),
                ));
            }
        });
    });

    c.bench_function("scalar_chain_normalized_pipeline_no_cache", |b| {
        b.iter(|| {
            for &d in black_box(&densities) {
                let d_n = <DensityBridge as Bridgeable>::normalize(d, black_box(&cfg_d), None);
                let d_c = <DensityBridge as Bridgeable>::compute(d_n);
                let t_n =
                    <TemperatureBridge as Bridgeable>::normalize(d_c, black_box(&cfg_t), None);
                let t_c = <TemperatureBridge as Bridgeable>::compute(t_n);
                let p_n = <PhaseTransitionBridge as Bridgeable>::normalize(
                    (t_c, bond),
                    black_box(&cfg_p),
                    None,
                );
                black_box(<PhaseTransitionBridge as Bridgeable>::compute(p_n));
            }
        });
    });
}

fn bench_pipeline_without_bridge(c: &mut Criterion) {
    let mut world = World::new();
    let entities = generate_benchmark_scenario(&mut world, 200);
    c.bench_function("bench_pipeline_without_bridge", |b| {
        b.iter(|| {
            for _ in 0..TICKS_IN_PIPELINE_BENCH {
                run_derived_chain_direct(&mut world, black_box(&entities));
            }
        });
    });
}

fn bench_pipeline_with_bridge(c: &mut Criterion) {
    let mut world = World::new();
    let entities = generate_benchmark_scenario(&mut world, 200);
    warm_derived_chain_bridged(&mut world, &entities);
    c.bench_function("bench_pipeline_with_bridge", |b| {
        b.iter(|| {
            for _ in 0..TICKS_IN_PIPELINE_BENCH {
                run_derived_chain_bridged(&mut world, black_box(&entities));
            }
        });
    });
}

fn bench_warmup_phase(c: &mut Criterion) {
    let mut group = c.benchmark_group("bench_warmup_phase");
    // Mismo escenario `generate_benchmark_scenario` en ambos brazos — solo cambia fase Warmup vs Active.
    let mut world_w = World::new();
    let entities_w = generate_benchmark_scenario(&mut world_w, 200);
    set_bridge_phase(&mut world_w, BridgePhase::Warmup);

    let mut world_d = World::new();
    let entities_d = generate_benchmark_scenario(&mut world_d, 200);

    group.bench_function("warmup_record_path", |b| {
        b.iter(|| {
            for _ in 0..8 {
                run_derived_chain_bridged(&mut world_w, black_box(&entities_w));
            }
        });
    });

    group.bench_function("direct_equations_same_entities", |b| {
        b.iter(|| {
            for _ in 0..8 {
                run_derived_chain_direct(&mut world_d, black_box(&entities_d));
            }
        });
    });
    group.finish();
}

/// `FxHashMap::get` es suelo de un lookup genérico; `bridge_compute` en hit incluye normalize + FNV + LRU.
/// No sustituye un bench de “registry global” — solo orden de magnitud (ver sprint B10).
fn bench_cache_lookup_overhead(c: &mut Criterion) {
    let cfg = DensityBridge::config_for_preset(RigidityPreset::Moderate);
    let mut cache = BridgeCache::<DensityBridge>::new(cfg.cache_capacity, cfg.policy);
    let input = 52.7_f32;
    let _ = bridge_compute::<DensityBridge>(input, &cfg, &mut cache);
    assert!(cache.stats().hits + cache.stats().misses >= 1);

    let norm = <DensityBridge as Bridgeable>::normalize(input, &cfg, None);
    let key = <DensityBridge as Bridgeable>::cache_key(norm);
    let mut map: FxHashMap<u64, f32> = FxHashMap::default();
    map.insert(key, 1.0);

    let mut group = c.benchmark_group("bench_cache_lookup_overhead");
    group.bench_function("bridge_cache_density_hit", |b| {
        b.iter(|| {
            black_box(bridge_compute::<DensityBridge>(
                black_box(input),
                black_box(&cfg),
                black_box(&mut cache),
            ));
        });
    });
    group.bench_function("fxhash_u64_lookup_scalar", |b| {
        b.iter(|| black_box(map.get(&black_box(key)).copied()));
    });
    group.finish();
}

fn bench_scaling_entities(c: &mut Criterion) {
    for n in [50usize, 200, 500, 1000] {
        let mut group = c.benchmark_group(format!("bench_scaling_entities_{n}"));
        group.bench_with_input(BenchmarkId::new("bridged_hot", n), &n, |b, &n| {
            let mut world = World::new();
            let entities = generate_benchmark_scenario(&mut world, n);
            warm_derived_chain_bridged(&mut world, &entities);
            b.iter(|| run_derived_chain_bridged(&mut world, black_box(&entities)));
        });
        group.bench_with_input(BenchmarkId::new("direct", n), &n, |b, &n| {
            let mut world = World::new();
            let entities = generate_benchmark_scenario(&mut world, n);
            b.iter(|| run_derived_chain_direct(&mut world, black_box(&entities)));
        });
        group.finish();
    }
}

fn bench_individual_equations(c: &mut Criterion) {
    let mut group = c.benchmark_group("bench_individual_bridges");
    let cfg_d = DensityBridge::config_for_preset(RigidityPreset::Moderate);
    let mut cache_d = BridgeCache::<DensityBridge>::new(cfg_d.cache_capacity, cfg_d.policy);
    let input_d = 48.0_f32;
    let _ = bridge_compute::<DensityBridge>(input_d, &cfg_d, &mut cache_d);

    group.bench_function("density_only_hot", |b| {
        b.iter(|| {
            black_box(bridge_compute::<DensityBridge>(
                black_box(input_d),
                black_box(&cfg_d),
                black_box(&mut cache_d),
            ));
        });
    });

    let cfg_i = InterferenceBridge::config_for_preset(RigidityPreset::Moderate);
    let mut cache_i = BridgeCache::<InterferenceBridge>::new(cfg_i.cache_capacity, cfg_i.policy);
    let input_i = InterferenceEquationInput {
        f1: 400.0,
        phase1: 0.5,
        f2: 500.0,
        phase2: 1.0,
        t: 1.0,
    };
    let _ = bridge_compute::<InterferenceBridge>(input_i, &cfg_i, &mut cache_i);
    group.bench_function("interference_only_hot", |b| {
        b.iter(|| {
            black_box(bridge_compute::<InterferenceBridge>(
                black_box(input_i),
                black_box(&cfg_i),
                black_box(&mut cache_i),
            ));
        });
    });
    group.finish();
}

fn bench_worst_case_misses(c: &mut Criterion) {
    let mut world = World::new();
    let entities = generate_benchmark_scenario(&mut world, 200);
    c.bench_function("bench_worst_case_changing_qe", |b| {
        let mut tick = 0u32;
        b.iter(|| {
            tick = tick.wrapping_add(1);
            for e in &entities {
                let mut ent = world.entity_mut(*e);
                if let Some(mut be) = ent.get_mut::<resonance::layers::BaseEnergy>() {
                    let pulse = (tick as f32 * 0.13).sin() * 2.0;
                    if pulse >= 0.0 {
                        be.inject(pulse);
                    } else {
                        be.drain(-pulse);
                    }
                }
            }
            clear_all_bridge_caches(&mut world);
            run_derived_chain_bridged(&mut world, black_box(&entities));
        });
    });
}

fn bench_speedup_snapshot(_c: &mut Criterion) {
    // Información humana: hit rates + tiempo informal (una muestra; no es estadística Criterion).
    let mut world_b = World::new();
    let entities_b = generate_benchmark_scenario(&mut world_b, 200);
    warm_derived_chain_bridged(&mut world_b, &entities_b);
    let t0 = Instant::now();
    for i in 0..64 {
        black_box(i);
        run_derived_chain_direct(&mut world_b, black_box(&entities_b));
    }
    let direct_ns = t0.elapsed().as_nanos();

    let t1 = Instant::now();
    for i in 0..64 {
        black_box(i);
        run_derived_chain_bridged(&mut world_b, black_box(&entities_b));
    }
    let bridged_ns = t1.elapsed().as_nanos();

    let ratio = direct_ns as f64 / bridged_ns.max(1) as f64;
    let report = collect_benchmark_report(&world_b, None, None);
    println!(
        "\n[B10 snapshot] hit_density={:.3} (blueprint {:.3}) | hit_temp={:.3} (blueprint {:.3}) | hit_phase={:.3} (blueprint {:.3}) | cache_upper_bound={} B\n[B10 snapshot] timing_informal_64_iters ratio_direct_over_bridged≈{ratio:.2}x (direct_ns={direct_ns} bridged_ns={bridged_ns}) — no CI\n",
        report.hit_rate_density,
        BLUEPRINT_HIT_RATE_DENSITY,
        report.hit_rate_temperature,
        BLUEPRINT_HIT_RATE_TEMPERATURE,
        report.hit_rate_phase_transition,
        BLUEPRINT_HIT_RATE_PHASE,
        report.memory_cache_upper_bound_bytes
    );

    let mut world_i = World::new();
    bootstrap_interference_bridge_world(&mut world_i);
    {
        let mut st =
            bevy::ecs::system::SystemState::<resonance::bridge::BridgedInterferenceOps>::new(
                &mut world_i,
            );
        let mut ops = st.get_mut(&mut world_i);
        let input = InterferenceEquationInput {
            f1: 450.0,
            phase1: 0.2,
            f2: 700.0,
            phase2: 1.1,
            t: 0.5,
        };
        for _ in 0..400 {
            let _ = ops.interference_equation(input);
        }
    }
    let hi = world_i
        .resource::<BridgeCache<InterferenceBridge>>()
        .stats()
        .hit_rate;
    println!(
        "[B10 snapshot] interference hit_rate={hi:.3} (blueprint {BLUEPRINT_HIT_RATE_INTERFERENCE:.3}) | catalysis blueprint {BLUEPRINT_HIT_RATE_CATALYSIS:.3}\n"
    );
}

criterion_group!(
    benches,
    bench_scalar_chain_hot_cache_vs_raw_equations,
    bench_pipeline_without_bridge,
    bench_pipeline_with_bridge,
    bench_warmup_phase,
    bench_cache_lookup_overhead,
    bench_scaling_entities,
    bench_individual_equations,
    bench_worst_case_misses,
    bench_speedup_snapshot,
);
criterion_main!(benches);
