//! Bridge presets — física: densidad, temperatura, transición de fase.

use crate::bridge::config::{BandDef, DensityBridge, PhaseTransitionBridge, TemperatureBridge};

const DENSITY_MOD: [BandDef; 5] = [
    BandDef {
        min: 0.0,
        max: 10.0,
        canonical: 5.0,
        stable: true,
    },
    BandDef {
        min: 10.0,
        max: 30.0,
        canonical: 20.0,
        stable: true,
    },
    BandDef {
        min: 30.0,
        max: 60.0,
        canonical: 45.0,
        stable: true,
    },
    BandDef {
        min: 60.0,
        max: 100.0,
        canonical: 80.0,
        stable: true,
    },
    BandDef {
        min: 100.0,
        max: 500.0,
        canonical: 200.0,
        stable: true,
    },
];

const TEMPERATURE_MOD: [BandDef; 5] = [
    BandDef {
        min: 0.0,
        max: 100.0,
        canonical: 50.0,
        stable: true,
    },
    BandDef {
        min: 100.0,
        max: 200.0,
        canonical: 150.0,
        stable: true,
    },
    BandDef {
        min: 200.0,
        max: 300.0,
        canonical: 250.0,
        stable: true,
    },
    BandDef {
        min: 300.0,
        max: 400.0,
        canonical: 350.0,
        stable: true,
    },
    BandDef {
        min: 400.0,
        max: 600.0,
        canonical: 500.0,
        stable: true,
    },
];

const PHASE_MOD: [BandDef; 4] = [
    BandDef {
        min: 0.0,
        max: 30.0,
        canonical: 15.0,
        stable: true,
    },
    BandDef {
        min: 30.0,
        max: 100.0,
        canonical: 60.0,
        stable: true,
    },
    BandDef {
        min: 100.0,
        max: 300.0,
        canonical: 180.0,
        stable: true,
    },
    BandDef {
        min: 300.0,
        max: 10_000.0,
        canonical: 400.0,
        stable: true,
    },
];

// Histéresis base 2.0 — `blueprint_layer_bridge_optimizer.md` §13 (density).
impl_bridge_defaults!(DensityBridge, "density", DENSITY_MOD, 2.0, 100);
impl_bridge_defaults!(TemperatureBridge, "temperature", TEMPERATURE_MOD, 1.0, 20);
impl_bridge_defaults!(
    PhaseTransitionBridge,
    "phase_transition",
    PHASE_MOD,
    5.0,
    70
);
