//! Harness reproducible para benchmarks B10 y tests de equivalencia (`docs/sprints/BRIDGE_OPTIMIZER/README.md`).
//! Requiere `feature = "bridge_optimizer"` — mismo flag que `BridgedPhysicsOps` / `BridgedInterferenceOps`.
//!
//! **Estabilidad:** API pensada para benches/tests; no garantizar compat semver en helpers hasta
//! que el pipeline de juego cablee los wrappers sin trabajo redundante.

use bevy::ecs::system::SystemState;
use bevy::prelude::*;

use crate::blueprint::equations;
use crate::bridge::cache::BridgeCache;
use crate::bridge::config::{
    BandDef, BridgeConfig, CatalysisBridge, CollisionTransferBridge, DensityBridge,
    InterferenceBridge, PhaseTransitionBridge, TemperatureBridge,
};
use crate::bridge::context_fill::{BridgePhase, BridgePhaseState};
use crate::bridge::decorator::Bridgeable;
use crate::bridge::impls::ops::{
    BridgedInterferenceOps, CatalysisEquationInput, CollisionTransferEquationInput,
    InterferenceEquationInput,
};
#[cfg(feature = "bridge_optimizer")]
use crate::bridge::impls::physics::BridgedPhysicsOps;
use crate::bridge::normalize::normalize_scalar;
use crate::bridge::presets::{BridgeDefaults, RigidityPreset};
use crate::layers::{BaseEnergy, MatterCoherence, MatterState, PhysicsOps, SpatialVolume};

/// Semilla documentada — escenarios deterministas entre máquinas (misma versión del harness).
pub const BENCHMARK_SCENARIO_SEED: u64 = 0xB10u64;

/// Objetivos de hit rate del blueprint §11 (referencia humana / benches; no assert duro en CI).
pub const BLUEPRINT_HIT_RATE_DENSITY: f32 = 0.982;
pub const BLUEPRINT_HIT_RATE_TEMPERATURE: f32 = 0.978;
pub const BLUEPRINT_HIT_RATE_PHASE: f32 = 0.999;
pub const BLUEPRINT_HIT_RATE_INTERFERENCE: f32 = 0.723;
pub const BLUEPRINT_HIT_RATE_DRAG: f32 = 0.891;
pub const BLUEPRINT_HIT_RATE_CATALYSIS: f32 = 0.95;
pub const BLUEPRINT_HIT_RATE_WILL: f32 = 0.995;

/// Métricas agregadas post-run (benches o tests de diagnóstico).
#[derive(Clone, Debug, Default, PartialEq)]
pub struct BenchmarkReport {
    /// `t_direct / t_bridged` si ambos > 0; None si no se midió.
    pub speedup_factor: Option<f32>,
    pub hit_rate_density: f32,
    pub hit_rate_temperature: f32,
    pub hit_rate_phase_transition: f32,
    pub hit_rate_interference: f32,
    pub hit_rate_catalysis: f32,
    pub hit_rate_collision_transfer: f32,
    /// Cota superior conservadora del footprint de entradas cacheadas (ver `estimate_bridge_cache_upper_bound_bytes`).
    pub memory_cache_upper_bound_bytes: usize,
    /// Ratio warmup vs directo (1.0 = igual); None si no aplica.
    pub warmup_overhead_ratio: Option<f32>,
}

/// SplitMix64 — determinista, sin `rand`.
#[inline]
fn splitmix64(mut x: u64) -> u64 {
    x = x.wrapping_add(0x9E3779B97F4A7C15);
    let mut z = x;
    z = (z ^ (z >> 30)).wrapping_mul(0xBF58476D1CE4E5B9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94D049BB133111EB);
    z ^ (z >> 31)
}

#[inline]
fn u01(i: usize, salt: u32) -> f32 {
    let h = splitmix64(BENCHMARK_SCENARIO_SEED ^ ((i as u64) << 32) ^ u64::from(salt));
    (h as f64 / u64::MAX as f64) as f32
}

fn max_band_span(bands: &[BandDef]) -> f32 {
    bands.iter().map(|b| b.max - b.min).fold(0.0_f32, f32::max)
}

/// Inserta configs Moderate + caches con capacidad del preset + fase Active.
pub fn bootstrap_physics_bridge_world(world: &mut World) {
    let d = DensityBridge::config_for_preset(RigidityPreset::Moderate);
    let t = TemperatureBridge::config_for_preset(RigidityPreset::Moderate);
    let p = PhaseTransitionBridge::config_for_preset(RigidityPreset::Moderate);
    world.insert_resource(d.clone());
    world.insert_resource(t.clone());
    world.insert_resource(p.clone());
    world.insert_resource(BridgeCache::<DensityBridge>::new(
        d.cache_capacity,
        d.policy,
    ));
    world.insert_resource(BridgeCache::<TemperatureBridge>::new(
        t.cache_capacity,
        t.policy,
    ));
    world.insert_resource(BridgeCache::<PhaseTransitionBridge>::new(
        p.cache_capacity,
        p.policy,
    ));
}

/// Fase del optimizer para benches (Warmup = graba sin lookup; Active = decorador completo).
pub fn set_bridge_phase(world: &mut World, phase: BridgePhase) {
    let state = match phase {
        BridgePhase::Active => BridgePhaseState::active_only(),
        BridgePhase::Warmup => BridgePhaseState::default(),
        BridgePhase::Filling => BridgePhaseState {
            phase: BridgePhase::Filling,
            ticks_in_warmup: 100,
            ticks_in_filling: 0,
        },
    };
    world.insert_resource(state);
}

/// Recursos para `BridgedInterferenceOps` (patrón alineado a tests en `bridged_ops.rs`).
pub fn bootstrap_interference_bridge_world(world: &mut World) {
    world.insert_resource(Time::<()>::default());
    world.insert_resource(crate::blueprint::AlchemicalAlmanac::default());
    let icfg = InterferenceBridge::config_for_preset(RigidityPreset::Moderate);
    let ccfg = CatalysisBridge::config_for_preset(RigidityPreset::Moderate);
    let colcfg = CollisionTransferBridge::config_for_preset(RigidityPreset::Moderate);
    world.insert_resource(icfg.clone());
    world.insert_resource(ccfg.clone());
    world.insert_resource(colcfg.clone());
    world.insert_resource(BridgeCache::<InterferenceBridge>::new(
        icfg.cache_capacity,
        icfg.policy,
    ));
    world.insert_resource(BridgeCache::<CatalysisBridge>::new(
        ccfg.cache_capacity,
        ccfg.policy,
    ));
    world.insert_resource(BridgeCache::<CollisionTransferBridge>::new(
        colcfg.cache_capacity,
        colcfg.policy,
    ));
    world.insert_resource(BridgePhaseState::active_only());
}

/// Distribución blueprint B10: 60% estático, 20% móvil, 15% proyectil, 5% evento.
pub fn generate_benchmark_scenario(world: &mut World, entity_count: usize) -> Vec<Entity> {
    bootstrap_physics_bridge_world(world);
    set_bridge_phase(world, BridgePhase::Active);
    let mut out = Vec::with_capacity(entity_count);
    for i in 0..entity_count {
        let u = u01(i, 0);
        let (qe, radius, state, bond) = if u < 0.60 {
            // Estático — alta redundancia numérica.
            let qe = 45.0 + u01(i, 1) * 12.0;
            let radius = 1.8 + u01(i, 2) * 1.2;
            (qe, radius, MatterState::Solid, 5000.0 + u01(i, 3) * 2000.0)
        } else if u < 0.80 {
            let qe = 85.0 + u01(i, 4) * 40.0;
            let radius = 0.85 + u01(i, 5) * 0.35;
            let st = if u01(i, 6) < 0.5 {
                MatterState::Liquid
            } else {
                MatterState::Gas
            };
            (qe, radius, st, 3500.0 + u01(i, 7) * 1500.0)
        } else if u < 0.95 {
            let qe = 22.0 + u01(i, 8) * 18.0;
            let radius = 0.25 + u01(i, 9) * 0.2;
            (qe, radius, MatterState::Liquid, 2800.0 + u01(i, 10) * 400.0)
        } else {
            let qe = 10.0 + u01(i, 11) * 90.0;
            let radius = 0.5 + u01(i, 12) * 2.5;
            let st = match i % 4 {
                0 => MatterState::Solid,
                1 => MatterState::Liquid,
                2 => MatterState::Gas,
                _ => MatterState::Plasma,
            };
            (qe, radius, st, 2000.0 + u01(i, 13) * 4000.0)
        };
        let e = world
            .spawn((
                BaseEnergy::new(qe),
                SpatialVolume::new(radius),
                MatterCoherence::new(state, bond, 0.5),
            ))
            .id();
        out.push(e);
    }
    out
}

/// Cadena alineada a lecturas derivadas del pipeline: densidad → temperatura → fase (sin decorador).
pub fn run_derived_chain_direct(world: &mut World, entities: &[Entity]) {
    let mut state = SystemState::<(PhysicsOps, Query<&MatterCoherence>)>::new(world);
    let (physics, coh_q) = state.get_mut(world);
    for &e in entities {
        let Some(d) = physics.density(e) else {
            continue;
        };
        let t = equations::equivalent_temperature(d);
        let bond = coh_q.get(e).map(|c| c.bond_energy_eb()).unwrap_or(5000.0);
        let s = equations::state_from_temperature(t, bond);
        core::hint::black_box((d, t, s));
    }
}

/// Misma cadena vía `BridgedPhysicsOps` (optimizer ON en configs Moderate).
pub fn run_derived_chain_bridged(world: &mut World, entities: &[Entity]) {
    let mut state = SystemState::<(BridgedPhysicsOps, Query<&MatterCoherence>)>::new(world);
    let (mut bridged, coh_q) = state.get_mut(world);
    for &e in entities {
        let Some(_d) = bridged.density(e) else {
            continue;
        };
        let Some(t) = bridged.temperature(e) else {
            continue;
        };
        let bond = coh_q.get(e).map(|c| c.bond_energy_eb()).unwrap_or(5000.0);
        let s = bridged.matter_state_from_temperature(e, t, bond);
        core::hint::black_box((t, s));
    }
}

/// Priming: un pase completo para llenar caches antes de medir Active.
pub fn warm_derived_chain_bridged(world: &mut World, entities: &[Entity]) {
    run_derived_chain_bridged(world, entities);
}

/// Cota superior: suma `capacity × sizeof(CacheEntry)` por puente Moderate (Small backend ≈ 24B/entrada + alineación).
pub fn estimate_bridge_cache_upper_bound_bytes() -> usize {
    const ENTRY_UPPER: usize = 32;
    let d = DensityBridge::config_for_preset(RigidityPreset::Moderate).cache_capacity;
    let t = TemperatureBridge::config_for_preset(RigidityPreset::Moderate).cache_capacity;
    let p = PhaseTransitionBridge::config_for_preset(RigidityPreset::Moderate).cache_capacity;
    let i = InterferenceBridge::config_for_preset(RigidityPreset::Moderate).cache_capacity;
    let c = CatalysisBridge::config_for_preset(RigidityPreset::Moderate).cache_capacity;
    let col = CollisionTransferBridge::config_for_preset(RigidityPreset::Moderate).cache_capacity;
    ENTRY_UPPER * (d + t + p + i + c + col)
}

/// Lee hit rates desde el `World` (post-ejecución).
pub fn collect_benchmark_report(
    world: &World,
    speedup: Option<f32>,
    warmup_ratio: Option<f32>,
) -> BenchmarkReport {
    BenchmarkReport {
        speedup_factor: speedup,
        hit_rate_density: world
            .resource::<BridgeCache<DensityBridge>>()
            .stats()
            .hit_rate,
        hit_rate_temperature: world
            .resource::<BridgeCache<TemperatureBridge>>()
            .stats()
            .hit_rate,
        hit_rate_phase_transition: world
            .resource::<BridgeCache<PhaseTransitionBridge>>()
            .stats()
            .hit_rate,
        hit_rate_interference: world
            .get_resource::<BridgeCache<InterferenceBridge>>()
            .map(|c| c.stats().hit_rate)
            .unwrap_or(0.0),
        hit_rate_catalysis: world
            .get_resource::<BridgeCache<CatalysisBridge>>()
            .map(|c| c.stats().hit_rate)
            .unwrap_or(0.0),
        hit_rate_collision_transfer: world
            .get_resource::<BridgeCache<CollisionTransferBridge>>()
            .map(|c| c.stats().hit_rate)
            .unwrap_or(0.0),
        memory_cache_upper_bound_bytes: estimate_bridge_cache_upper_bound_bytes(),
        warmup_overhead_ratio: warmup_ratio,
    }
}

/// Expuesto para tests: valida cotas ε de densidad/temperatura vs crudo.
pub fn assert_physics_bridge_epsilon(world: &mut World, entities: &[Entity]) {
    let cfg_d = world.resource::<BridgeConfig<DensityBridge>>().clone();
    let cfg_t = world.resource::<BridgeConfig<TemperatureBridge>>().clone();
    let cfg_p = world
        .resource::<BridgeConfig<PhaseTransitionBridge>>()
        .clone();
    let max_d = max_band_span(&cfg_d.bands);
    let max_t = max_band_span(&cfg_t.bands);

    let mut state =
        SystemState::<(PhysicsOps, BridgedPhysicsOps, Query<&MatterCoherence>)>::new(world);
    let (physics, mut bridged, coh_q) = state.get_mut(world);

    for &e in entities {
        let Some(d0) = physics.density(e) else {
            panic!("density missing for entity {e:?}");
        };
        let Some(d1) = bridged.density(e) else {
            panic!("bridged density missing for entity {e:?}");
        };
        assert!(
            (d1 - d0).abs() <= max_d + 1e-3,
            "density entity {:?} d0={} d1={}",
            e,
            d0,
            d1
        );

        let Some(t0) = physics.temperature(e) else {
            panic!("temperature missing for entity {e:?}");
        };
        let Some(t1) = bridged.temperature(e) else {
            panic!("bridged temperature missing for entity {e:?}");
        };
        assert!(
            (t1 - t0).abs() <= max_d + max_t + 1e-3,
            "temp entity {:?}",
            e
        );

        let bond = coh_q.get(e).map(|c| c.bond_energy_eb()).unwrap_or(5000.0);
        let hint = crate::bridge::phase_band_hint_from_state(
            coh_q
                .get(e)
                .map(|c| c.state())
                .unwrap_or(MatterState::Solid),
        );
        let (t_canon, _) = normalize_scalar(t0, &cfg_p.bands, cfg_p.hysteresis_margin, hint);
        let expected = equations::state_from_temperature(t_canon, bond);
        let got = bridged.matter_state_from_temperature(e, t0, bond);
        assert_eq!(got, expected, "fase entity {:?} t0={} bond={}", e, t0, bond);
    }
}

/// Paridad ecuaciones aisladas (interferencia / catálisis / transferencia) con bridge Moderate ON.
pub fn assert_isolated_ops_normalized_parity(world: &mut World) {
    const INTERFERENCE_CASES: &[InterferenceEquationInput] = &[
        InterferenceEquationInput {
            f1: 450.0,
            phase1: 0.31,
            f2: 700.0,
            phase2: 1.12,
            t: 3.33,
        },
        InterferenceEquationInput {
            f1: 100.0,
            phase1: 0.0,
            f2: 300.0,
            phase2: std::f32::consts::PI,
            t: 0.05,
        },
        InterferenceEquationInput {
            f1: 12.0,
            phase1: -1.0,
            f2: 900.0,
            phase2: 4.0,
            t: 120.0,
        },
    ];

    for (idx, &input_i) in INTERFERENCE_CASES.iter().enumerate() {
        let b = {
            let mut st = SystemState::<BridgedInterferenceOps>::new(world);
            let mut ops = st.get_mut(world);
            ops.interference_equation(input_i)
        };
        let cfg = world.resource::<BridgeConfig<InterferenceBridge>>().clone();
        let n = <InterferenceBridge as Bridgeable>::normalize(input_i, &cfg, None);
        let e = equations::interference(n.f1, n.phase1, n.f2, n.phase2, n.t);
        assert!((b - e).abs() < 1e-4, "interference idx={idx} b={b} e={e}");
    }

    const CATALYSIS_CASES: &[CatalysisEquationInput] = &[
        CatalysisEquationInput {
            projected_qe: 42.0,
            interference: 0.55,
            critical_multiplier: 1.8,
        },
        CatalysisEquationInput {
            projected_qe: 5.0,
            interference: -0.9,
            critical_multiplier: 1.0,
        },
        CatalysisEquationInput {
            projected_qe: 200.0,
            interference: 0.1,
            critical_multiplier: 2.5,
        },
    ];

    for (idx, &input_c) in CATALYSIS_CASES.iter().enumerate() {
        let b = {
            let mut st = SystemState::<BridgedInterferenceOps>::new(world);
            let mut ops = st.get_mut(world);
            ops.catalysis_result(input_c)
        };
        let cfg = world.resource::<BridgeConfig<CatalysisBridge>>().clone();
        let n = <CatalysisBridge as Bridgeable>::normalize(input_c, &cfg, None);
        let e = equations::catalysis_result(n.projected_qe, n.interference, n.critical_multiplier);
        assert!((b - e).abs() < 1e-4, "catalysis idx={idx} b={b} e={e}");
    }

    const COLLISION_CASES: &[CollisionTransferEquationInput] = &[
        CollisionTransferEquationInput {
            qe_a: 88.0,
            qe_b: 72.0,
            interference: -0.35,
            conductivity: 0.42,
            dt: 1.0 / 60.0,
        },
        CollisionTransferEquationInput {
            qe_a: 10.0,
            qe_b: 90.0,
            interference: 0.5,
            conductivity: 0.1,
            dt: 1.0 / 120.0,
        },
        CollisionTransferEquationInput {
            qe_a: 1.0,
            qe_b: 1.0,
            interference: 0.0,
            conductivity: 0.99,
            dt: 1.0 / 30.0,
        },
    ];

    for (idx, &input_col) in COLLISION_CASES.iter().enumerate() {
        let b = {
            let mut st = SystemState::<BridgedInterferenceOps>::new(world);
            let mut ops = st.get_mut(world);
            ops.collision_transfer(input_col)
        };
        let cfg = world
            .resource::<BridgeConfig<CollisionTransferBridge>>()
            .clone();
        let n = <CollisionTransferBridge as Bridgeable>::normalize(input_col, &cfg, None);
        let e = equations::collision_transfer(n.qe_a, n.qe_b, n.interference, n.conductivity, n.dt);
        assert!((b - e).abs() < 1e-3, "collision idx={idx} b={b} e={e}");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scenario_len_matches_entity_count() {
        let mut world = World::new();
        let n = 127;
        let ents = generate_benchmark_scenario(&mut world, n);
        assert_eq!(ents.len(), n);
    }

    #[test]
    fn cache_upper_bound_under_1mb_for_moderate_preset() {
        let b = estimate_bridge_cache_upper_bound_bytes();
        assert!(b < 1_000_000, "upper_bound={}", b);
    }

    #[test]
    fn epsilon_harness_matches_bridged_semantics() {
        let mut world = World::new();
        let ents = generate_benchmark_scenario(&mut world, 24);
        assert_physics_bridge_epsilon(&mut world, &ents);
    }
}
