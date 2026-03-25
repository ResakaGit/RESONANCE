//! Contrato público Q3: lectura vía accessors (crate externo `tests/` no ve `pub(crate)`).

use bevy::prelude::*;

use resonance::layers::{
    AlchemicalEngine, BaseEnergy, FlowVector, MatterCoherence, MatterState, OscillatorySignature,
};
use resonance::worldgen::{EnergyNucleus, FrequencyContribution, PropagationDecay};

#[test]
fn base_energy_qe_accessor() {
    let e = BaseEnergy::new(42.0);
    assert!((e.qe() - 42.0).abs() < 1e-5);
}

#[test]
fn matter_coherence_state_accessor() {
    let m = MatterCoherence::new(MatterState::Liquid, 1000.0, 0.4);
    assert_eq!(m.state(), MatterState::Liquid);
}

#[test]
fn flow_vector_velocity_accessor() {
    let f = FlowVector::new(Vec2::new(3.0, 4.0), 2.0);
    assert_eq!(f.velocity(), Vec2::new(3.0, 4.0));
}

#[test]
fn oscillatory_frequency_accessor() {
    let o = OscillatorySignature::new(440.0, 0.25);
    assert!((o.frequency_hz() - 440.0).abs() < 1e-5);
}

#[test]
fn alchemical_engine_buffer_accessors() {
    let eng = AlchemicalEngine::new(100.0, 10.0, 5.0, 20.0);
    assert!((eng.buffer_level() - 20.0).abs() < 1e-5);
    assert!((eng.buffer_cap() - 100.0).abs() < 1e-5);
}

#[test]
fn energy_nucleus_frequency_accessor() {
    let n = EnergyNucleus::new(80.0, 1.0, 2.0, PropagationDecay::Flat);
    assert!((n.frequency_hz() - 80.0).abs() < 1e-5);
}

#[test]
fn frequency_contribution_accessors() {
    let c = FrequencyContribution::new(Entity::from_raw(7), 123.0, 4.0);
    assert_eq!(c.source_entity(), Entity::from_raw(7));
    assert!((c.frequency_hz() - 123.0).abs() < 1e-5);
    assert!((c.intensity_qe() - 4.0).abs() < 1e-5);
}
