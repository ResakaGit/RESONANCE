//! Sprint G6: `#[require]` — contrato de composición para spawns mínimos.
//! Ver `docs/sprints/GAMEDEV_PATTERNS/README.md` (G6 cerrado).

use bevy::prelude::*;
use resonance::blueprint::constants::{
    DEFAULT_BASE_ENERGY, DEFAULT_BOND_ENERGY, DEFAULT_DISSIPATION_RATE, DEFAULT_FREQUENCY_HZ,
    DEFAULT_SPHERE_RADIUS, DEFAULT_THERMAL_CONDUCTIVITY, ENGINE_DEFAULT_INPUT_VALVE,
    ENGINE_DEFAULT_MAX_BUFFER, ENGINE_DEFAULT_OUTPUT_VALVE, LINK_NEUTRAL_MULTIPLIER,
    QE_MIN_EXISTENCE, VOLUME_MIN_RADIUS,
};
use resonance::layers::{
    AlchemicalBase, AlchemicalEngine, AlchemicalInjector, AmbientPressure, BaseEnergy, Champion,
    FlowVector, MatterCoherence, MatterState, MobaIdentity, MobileEntity, OscillatorySignature,
    SpatialVolume, WaveEntity, WillActuator,
};

/// `LayersPlugin` no afecta `#[require]` al spawn; `MinimalPlugins` basta para estos tests.
fn minimal_app_for_require_tests() -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app
}

#[test]
fn spawn_champion_inserts_required_stack_and_markers() {
    let mut app = minimal_app_for_require_tests();
    let id = app.world_mut().spawn(Champion).id();

    let world = app.world();
    assert!(world.get::<Champion>(id).is_some());
    assert!(world.get::<MobileEntity>(id).is_some());
    assert!(world.get::<WaveEntity>(id).is_some());
    assert!(world.get::<AlchemicalBase>(id).is_some());

    assert!(world.get::<Transform>(id).is_some());
    assert!(world.get::<Visibility>(id).is_some());

    assert!(world.get::<BaseEnergy>(id).is_some());
    assert!(world.get::<SpatialVolume>(id).is_some());
    assert!(world.get::<OscillatorySignature>(id).is_some());
    assert!(world.get::<FlowVector>(id).is_some());
    assert!(world.get::<MatterCoherence>(id).is_some());
    assert!(world.get::<AlchemicalEngine>(id).is_some());
    assert!(world.get::<WillActuator>(id).is_some());
    assert!(world.get::<MobaIdentity>(id).is_some());
}

#[test]
fn spawn_champion_does_not_insert_optional_layers() {
    let mut app = minimal_app_for_require_tests();
    let id = app.world_mut().spawn(Champion).id();
    let world = app.world();
    assert!(
        world.get::<AmbientPressure>(id).is_none(),
        "L6 no forma parte de la cadena G6"
    );
    assert!(
        world.get::<AlchemicalInjector>(id).is_none(),
        "L8 no forma parte de la cadena G6"
    );
}

#[test]
fn spawn_wave_entity_stops_before_matter_engine_will_moba() {
    let mut app = minimal_app_for_require_tests();
    let id = app.world_mut().spawn(WaveEntity).id();
    let w = app.world();

    assert!(w.get::<WaveEntity>(id).is_some());
    assert!(w.get::<AlchemicalBase>(id).is_some());
    assert!(w.get::<OscillatorySignature>(id).is_some());
    assert!(w.get::<FlowVector>(id).is_some());

    assert!(w.get::<MatterCoherence>(id).is_none());
    assert!(w.get::<AlchemicalEngine>(id).is_none());
    assert!(w.get::<WillActuator>(id).is_none());
    assert!(w.get::<MobaIdentity>(id).is_none());
    assert!(w.get::<MobileEntity>(id).is_none());
    assert!(w.get::<Champion>(id).is_none());
}

#[test]
fn spawn_mobile_entity_includes_engine_but_not_moba_identity() {
    let mut app = minimal_app_for_require_tests();
    let id = app.world_mut().spawn(MobileEntity).id();
    let w = app.world();

    assert!(w.get::<MobileEntity>(id).is_some());
    assert!(w.get::<MatterCoherence>(id).is_some());
    assert!(w.get::<AlchemicalEngine>(id).is_some());
    assert!(w.get::<WillActuator>(id).is_some());

    assert!(w.get::<MobaIdentity>(id).is_none());
    assert!(w.get::<Champion>(id).is_none());
}

#[test]
fn spawn_champion_defaults_are_valid_for_simulation() {
    let mut app = minimal_app_for_require_tests();
    let id = app.world_mut().spawn(Champion).id();
    let world = app.world();

    let qe = world.get::<BaseEnergy>(id).unwrap().qe();
    assert!(qe.is_finite());
    assert!(
        qe > QE_MIN_EXISTENCE,
        "qe por defecto debe mantener existencia (> QE_MIN_EXISTENCE)"
    );
    assert!((qe - DEFAULT_BASE_ENERGY).abs() < 1e-3);

    let vol = world.get::<SpatialVolume>(id).unwrap();
    let r = vol.radius;
    assert!(r >= VOLUME_MIN_RADIUS && r.is_finite());
    assert!((r - DEFAULT_SPHERE_RADIUS).abs() < 1e-4);

    let osc = world.get::<OscillatorySignature>(id).unwrap();
    assert!(osc.frequency_hz() >= 0.0 && osc.frequency_hz().is_finite());
    assert!((osc.frequency_hz() - DEFAULT_FREQUENCY_HZ).abs() < 1e-3);

    let flow = world.get::<FlowVector>(id).unwrap();
    assert_eq!(flow.velocity(), Vec2::ZERO);
    assert!(flow.dissipation_rate().is_finite() && flow.dissipation_rate() >= 0.0);
    assert!((flow.dissipation_rate() - DEFAULT_DISSIPATION_RATE).abs() < 1e-5);

    let matter = world.get::<MatterCoherence>(id).unwrap();
    assert_eq!(matter.state(), MatterState::Solid);
    assert!((matter.bond_energy_eb() - DEFAULT_BOND_ENERGY).abs() < 1e-3);
    assert!((matter.thermal_conductivity() - DEFAULT_THERMAL_CONDUCTIVITY).abs() < 1e-3);

    let engine = world.get::<AlchemicalEngine>(id).unwrap();
    assert!((engine.buffer_cap() - ENGINE_DEFAULT_MAX_BUFFER).abs() < 1e-3);
    assert!((engine.valve_in_rate() - ENGINE_DEFAULT_INPUT_VALVE).abs() < 1e-3);
    assert!((engine.valve_out_rate() - ENGINE_DEFAULT_OUTPUT_VALVE).abs() < 1e-3);
    assert!((engine.buffer_level() - 0.0).abs() < 1e-6);

    let id_moba = world.get::<MobaIdentity>(id).unwrap();
    assert!((id_moba.critical_multiplier() - LINK_NEUTRAL_MULTIPLIER).abs() < 1e-5);
}

#[test]
fn spawn_alchemical_base_inserts_transform_energy_volume() {
    let mut app = minimal_app_for_require_tests();
    let id = app.world_mut().spawn(AlchemicalBase).id();
    let world = app.world();
    assert!(world.get::<AlchemicalBase>(id).is_some());
    assert!(world.get::<Transform>(id).is_some());
    assert!(world.get::<BaseEnergy>(id).is_some());
    assert!(world.get::<SpatialVolume>(id).is_some());
    let r = world.get::<SpatialVolume>(id).unwrap().radius;
    assert!((r - DEFAULT_SPHERE_RADIUS).abs() < 1e-4);
}
