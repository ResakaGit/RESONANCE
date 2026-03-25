//! Equivalencia bridge ON vs ecuaciones de referencia — sprint B10.
//! `cargo test -p resonance --features bridge_optimizer --test bridge_optimizer_equivalence`

use bevy::prelude::*;

use resonance::bridge::benchmark_harness::{
    assert_isolated_ops_normalized_parity, assert_physics_bridge_epsilon,
    bootstrap_interference_bridge_world, estimate_bridge_cache_upper_bound_bytes,
    generate_benchmark_scenario,
};
use resonance::bridge::config::DensityBridge;
use resonance::bridge::presets::{BridgeDefaults, RigidityPreset};
use resonance::layers::{BaseEnergy, SpatialVolume};

#[test]
fn test_equivalence_with_without_bridge_thousand_ticks() {
    let mut world = World::new();
    let entities = generate_benchmark_scenario(&mut world, 48);
    assert!(
        estimate_bridge_cache_upper_bound_bytes() < 1_000_000,
        "cota memoria B10 para preset Moderate"
    );

    for tick in 0..1000 {
        for &e in &entities {
            let mut ent = world.entity_mut(e);
            if let Some(mut be) = ent.get_mut::<BaseEnergy>() {
                let pulse = 0.08 * (tick as f32 * 0.031).sin();
                if pulse >= 0.0 {
                    be.inject(pulse);
                } else {
                    let _ = be.drain(-pulse);
                }
            }
        }
        assert_physics_bridge_epsilon(&mut world, &entities);
    }

    let mut world_ops = World::new();
    bootstrap_interference_bridge_world(&mut world_ops);
    assert_isolated_ops_normalized_parity(&mut world_ops);
}

#[test]
fn edge_cases_density_inside_bands_low_and_high() {
    // Fuera del rango [min,max] de bandas Moderate la cota ε de B4 deja de aplicar (ver tests unitarios).
    let cfg = DensityBridge::config_for_preset(RigidityPreset::Moderate);
    let lo = cfg.bands.first().expect("bands").min;
    let hi = cfg.bands.last().expect("bands").max;
    let r = 1.75_f32;
    let vol = SpatialVolume::new(r).volume();
    let qe_low = lo * vol;
    let qe_high = hi * vol * 0.98;

    let mut world = World::new();
    let e0 = world
        .spawn((
            BaseEnergy::new(qe_low.max(1e-4)),
            SpatialVolume::new(r),
            resonance::layers::MatterCoherence::new(
                resonance::layers::MatterState::Solid,
                100.0,
                0.5,
            ),
        ))
        .id();
    let e1 = world
        .spawn((
            BaseEnergy::new(qe_high),
            SpatialVolume::new(r),
            resonance::layers::MatterCoherence::new(
                resonance::layers::MatterState::Plasma,
                50.0,
                0.5,
            ),
        ))
        .id();
    resonance::bridge::benchmark_harness::bootstrap_physics_bridge_world(&mut world);
    resonance::bridge::benchmark_harness::set_bridge_phase(
        &mut world,
        resonance::bridge::context_fill::BridgePhase::Active,
    );
    assert_physics_bridge_epsilon(&mut world, &[e0, e1]);
}
