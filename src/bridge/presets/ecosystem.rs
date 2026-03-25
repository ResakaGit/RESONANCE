//! Bridge presets — ecosistema: ósmosis, catálisis, disipación, drag, motor, norma de competición.

use crate::bridge::config::{
    BandDef, CatalysisBridge, CompetitionNormBridge, DissipationBridge, DragBridge, EngineBridge,
    OsmosisBridge,
};

use super::impl_bridge_defaults;

const DISSIPATION_MOD: [BandDef; 4] = [
    BandDef { min: 0.0,  max: 0.25, canonical: 0.12, stable: true },
    BandDef { min: 0.25, max: 0.5,  canonical: 0.37, stable: true },
    BandDef { min: 0.5,  max: 1.0,  canonical: 0.75, stable: true },
    BandDef { min: 1.0,  max: 4.0,  canonical: 2.0,  stable: true },
];

const DRAG_MOD: [BandDef; 4] = [
    BandDef { min: 0.0,  max: 1.0,   canonical: 0.5,  stable: true },
    BandDef { min: 1.0,  max: 5.0,   canonical: 3.0,  stable: true },
    BandDef { min: 5.0,  max: 20.0,  canonical: 12.0, stable: true },
    BandDef { min: 20.0, max: 100.0, canonical: 50.0, stable: true },
];

const ENGINE_MOD: [BandDef; 4] = [
    BandDef { min: 0.0,  max: 0.25, canonical: 0.12, stable: true },
    BandDef { min: 0.25, max: 0.5,  canonical: 0.37, stable: true },
    BandDef { min: 0.5,  max: 1.0,  canonical: 0.75, stable: true },
    BandDef { min: 1.0,  max: 2.0,  canonical: 1.5,  stable: true },
];

const CATALYSIS_MOD: [BandDef; 4] = [
    BandDef { min: 0.0,   max: 50.0,   canonical: 25.0,  stable: true },
    BandDef { min: 50.0,  max: 150.0,  canonical: 100.0, stable: true },
    BandDef { min: 150.0, max: 400.0,  canonical: 275.0, stable: true },
    BandDef { min: 400.0, max: 2000.0, canonical: 800.0, stable: true },
];

const OSMOSIS_MOD: [BandDef; 5] = [
    BandDef { min: 0.0,   max: 10.0,  canonical: 5.0,   stable: true },
    BandDef { min: 10.0,  max: 30.0,  canonical: 20.0,  stable: true },
    BandDef { min: 30.0,  max: 60.0,  canonical: 45.0,  stable: true },
    BandDef { min: 60.0,  max: 120.0, canonical: 90.0,  stable: true },
    BandDef { min: 120.0, max: 500.0, canonical: 250.0, stable: true },
];

// raw_score domain [0, ∞) — logistic midpoint typically ~1.0, so cover [0, 10].
const COMPETITION_NORM_MOD: [BandDef; 5] = [
    BandDef { min: 0.0, max: 1.0,  canonical: 0.5,  stable: true },
    BandDef { min: 1.0, max: 2.0,  canonical: 1.5,  stable: true },
    BandDef { min: 2.0, max: 4.0,  canonical: 3.0,  stable: true },
    BandDef { min: 4.0, max: 7.0,  canonical: 5.5,  stable: true },
    BandDef { min: 7.0, max: 50.0, canonical: 15.0, stable: true },
];

impl_bridge_defaults!(DissipationBridge,    "dissipation",    DISSIPATION_MOD,    0.5,  80);
impl_bridge_defaults!(DragBridge,           "drag",           DRAG_MOD,           0.3,  800);
impl_bridge_defaults!(EngineBridge,         "engine",         ENGINE_MOD,         1.0,  200);
impl_bridge_defaults!(CatalysisBridge,      "catalysis",      CATALYSIS_MOD,      0.01, 1000);
impl_bridge_defaults!(OsmosisBridge,        "osmosis",        OSMOSIS_MOD,        0.5,  512);
impl_bridge_defaults!(CompetitionNormBridge,"competition_norm",COMPETITION_NORM_MOD,0.1, 128);
