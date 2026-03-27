//! Bridge presets — combate: colisión, will/voluntad, interferencia.

use crate::bridge::config::{BandDef, CollisionTransferBridge, InterferenceBridge, WillBridge};


const INTERFERENCE_MOD: [BandDef; 6] = [
    BandDef { min: 0.0,    max: 200.0,  canonical: 100.0,  stable: true },
    BandDef { min: 200.0,  max: 400.0,  canonical: 300.0,  stable: true },
    BandDef { min: 400.0,  max: 600.0,  canonical: 500.0,  stable: true },
    BandDef { min: 600.0,  max: 800.0,  canonical: 700.0,  stable: true },
    BandDef { min: 800.0,  max: 1000.0, canonical: 900.0,  stable: true },
    BandDef { min: 1000.0, max: 1200.0, canonical: 1100.0, stable: true },
];

const WILL_MOD: [BandDef; 4] = [
    BandDef { min: 0.0,  max: 0.25, canonical: 0.12, stable: true },
    BandDef { min: 0.25, max: 0.5,  canonical: 0.37, stable: true },
    BandDef { min: 0.5,  max: 1.0,  canonical: 0.75, stable: true },
    BandDef { min: 1.0,  max: 2.0,  canonical: 1.5,  stable: true },
];

const COLLISION_MOD: [BandDef; 4] = [
    BandDef { min: 0.0,   max: 50.0,   canonical: 25.0,   stable: true },
    BandDef { min: 50.0,  max: 200.0,  canonical: 125.0,  stable: true },
    BandDef { min: 200.0, max: 800.0,  canonical: 500.0,  stable: true },
    BandDef { min: 800.0, max: 5000.0, canonical: 2000.0, stable: true },
];

impl_bridge_defaults!(InterferenceBridge,      "interference",       INTERFERENCE_MOD, 0.05, 500);
impl_bridge_defaults!(WillBridge,              "will",               WILL_MOD,         0.0,  80);
impl_bridge_defaults!(CollisionTransferBridge, "collision_transfer", COLLISION_MOD,    0.5,  1000);
