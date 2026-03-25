//! Wrappers `SystemParam` para ecuaciones de interferencia / catálisis / transferencia (sprint B5).
//!
//! No compone [`crate::layers::oscillatory::InterferenceOps`] porque anidar otro `SystemParam` con las
//! mismas `Query` rompería el borrow checker de Bevy; la semántica replica `InterferenceOps` (mismas
//! queries + `compute_interference_total`).
//!
//! Ver `docs/sprints/BRIDGE_OPTIMIZER/README.md` y
//! `docs/arquitectura/blueprint_layer_bridge_optimizer.md` §5, §13.

use std::f32::consts::TAU;

use bevy::ecs::system::SystemParam;
use bevy::prelude::*;

use crate::blueprint::AlchemicalAlmanac;
use crate::blueprint::equations;
use crate::bridge::cache::{BridgeCache, CachedValue};
use crate::bridge::config::{
    BridgeConfig, CatalysisBridge, CollisionTransferBridge, InterferenceBridge, OsmosisBridge,
};
// SSOT en `bridge::constants`; reexport para rutas `bridge::bridged_ops::INTERFERENCE_*`.
pub use crate::bridge::constants::{INTERFERENCE_PHASE_SECTORS, INTERFERENCE_TIME_QUANT_S};
use crate::bridge::context_fill::{BridgePhase, BridgePhaseState};
use crate::bridge::decorator::{Bridgeable, bridge_compute, hash_inputs};
use crate::bridge::normalize::{normalize_scalar, quantize_precision};
use crate::layers::identity::MobaIdentity;
use crate::layers::oscillatory::{OscillatorySignature, compose_interference};
use crate::runtime_platform::simulation_tick::SimulationElapsed;

// --- Inputs -----------------------------------------------------------------

/// Entrada cruda para `equations::interference` (5 escalares).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct InterferenceEquationInput {
    pub f1: f32,
    pub phase1: f32,
    pub f2: f32,
    pub phase2: f32,
    pub t: f32,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CatalysisEquationInput {
    pub projected_qe: f32,
    pub interference: f32,
    pub critical_multiplier: f32,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CollisionTransferEquationInput {
    pub qe_a: f32,
    pub qe_b: f32,
    pub interference: f32,
    pub conductivity: f32,
    pub dt: f32,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct OsmosisEquationInput {
    pub concentration_a: f32,
    pub concentration_b: f32,
    pub membrane_permeability: f32,
}

// --- Frecuencia: Almanac + fallback bandas -----------------------------------

/// Canonical de banda elemental si existe; si no, bandas del `BridgeConfig` (Moderate / RON).
#[inline]
pub fn canonicalize_frequency_hz(
    almanac: &AlchemicalAlmanac,
    freq_hz: f32,
    bands: &[crate::bridge::config::BandDef],
    hysteresis: f32,
) -> f32 {
    if let Some(def) = almanac.find_stable_band(freq_hz) {
        def.frequency_hz
    } else {
        normalize_scalar(freq_hz, bands, hysteresis, None).0
    }
}

#[inline]
fn quantize_time_window(t: f32, window_s: f32) -> f32 {
    if !t.is_finite() || !window_s.is_finite() || window_s <= 0.0 {
        return t;
    }
    (t / window_s).floor() * window_s
}

#[inline]
fn quantize_phase_sector(phase: f32, sectors: u8) -> f32 {
    if sectors < 2 {
        return phase;
    }
    let s = f32::from(sectors);
    let p = phase.rem_euclid(TAU);
    let idx = (p / TAU * s).floor().clamp(0.0, s - 1.0);
    (idx / s) * TAU
}

/// Graba interferencia exacta (ondas crudas) con clave alineada al pipeline activo (almanac + bandas).
fn interference_warmup_insert(
    cache: &mut BridgeCache<InterferenceBridge>,
    cfg: &BridgeConfig<InterferenceBridge>,
    almanac: &AlchemicalAlmanac,
    raw_input: InterferenceEquationInput,
    raw_out: f32,
) {
    let bands = &cfg.bands;
    let h = cfg.hysteresis_margin;
    let mut kin = raw_input;
    kin.f1 = canonicalize_frequency_hz(almanac, kin.f1, bands, h);
    kin.f2 = canonicalize_frequency_hz(almanac, kin.f2, bands, h);
    let norm = <InterferenceBridge as Bridgeable>::normalize(kin, cfg, None);
    let key = <InterferenceBridge as Bridgeable>::cache_key(norm);
    cache.insert(key, CachedValue::Scalar(raw_out));
}

fn interference_time_window(config: &BridgeConfig<InterferenceBridge>) -> f32 {
    // 0.1s objetivo sprint; si el preset usa histéresis 0.05, 2× da 0.1 coherente con §13.
    let w = config.hysteresis_margin * 2.0;
    if (0.08..=0.12).contains(&w) {
        w
    } else {
        INTERFERENCE_TIME_QUANT_S
    }
}

// --- InterferenceBridge -----------------------------------------------------

impl Bridgeable for InterferenceBridge {
    type Input = InterferenceEquationInput;
    type Output = f32;

    fn normalize(
        input: Self::Input,
        config: &BridgeConfig<Self>,
        _band_hint: Option<usize>,
    ) -> Self::Input {
        let h = config.hysteresis_margin;
        let f1 = normalize_scalar(input.f1, &config.bands, h, None).0;
        let f2 = normalize_scalar(input.f2, &config.bands, h, None).0;
        let phase1 = quantize_phase_sector(input.phase1, INTERFERENCE_PHASE_SECTORS);
        let phase2 = quantize_phase_sector(input.phase2, INTERFERENCE_PHASE_SECTORS);
        let tw = interference_time_window(config);
        let t = quantize_time_window(input.t, tw);
        Self::Input {
            f1,
            phase1,
            f2,
            phase2,
            t,
        }
    }

    fn cache_key(normalized: Self::Input) -> u64 {
        hash_inputs(&[
            f32::to_bits(normalized.f1) as u64,
            f32::to_bits(normalized.phase1) as u64,
            f32::to_bits(normalized.f2) as u64,
            f32::to_bits(normalized.phase2) as u64,
            f32::to_bits(normalized.t) as u64,
        ])
    }

    fn compute(normalized: Self::Input) -> Self::Output {
        equations::interference(
            normalized.f1,
            normalized.phase1,
            normalized.f2,
            normalized.phase2,
            normalized.t,
        )
    }

    fn into_cached(value: Self::Output) -> CachedValue {
        CachedValue::Scalar(value)
    }

    fn from_cached(value: CachedValue) -> Option<Self::Output> {
        match value {
            CachedValue::Scalar(s) => Some(s),
            _ => None,
        }
    }
}

// --- CatalysisBridge --------------------------------------------------------

impl Bridgeable for CatalysisBridge {
    type Input = CatalysisEquationInput;
    type Output = f32;

    fn normalize(
        input: Self::Input,
        config: &BridgeConfig<Self>,
        _band_hint: Option<usize>,
    ) -> Self::Input {
        let h = config.hysteresis_margin;
        let projected_qe = normalize_scalar(input.projected_qe, &config.bands, h, None).0;
        let interference = quantize_precision(input.interference, 2);
        let critical_multiplier = quantize_precision(input.critical_multiplier, 2);
        Self::Input {
            projected_qe,
            interference,
            critical_multiplier,
        }
    }

    fn cache_key(normalized: Self::Input) -> u64 {
        hash_inputs(&[
            f32::to_bits(normalized.projected_qe) as u64,
            f32::to_bits(normalized.interference) as u64,
            f32::to_bits(normalized.critical_multiplier) as u64,
        ])
    }

    fn compute(normalized: Self::Input) -> Self::Output {
        equations::catalysis_result(
            normalized.projected_qe,
            normalized.interference,
            normalized.critical_multiplier,
        )
    }

    fn into_cached(value: Self::Output) -> CachedValue {
        CachedValue::Scalar(value)
    }

    fn from_cached(value: CachedValue) -> Option<Self::Output> {
        match value {
            CachedValue::Scalar(s) => Some(s),
            _ => None,
        }
    }
}

// --- CollisionTransferBridge ------------------------------------------------

impl Bridgeable for CollisionTransferBridge {
    type Input = CollisionTransferEquationInput;
    type Output = f32;

    fn normalize(
        input: Self::Input,
        config: &BridgeConfig<Self>,
        _band_hint: Option<usize>,
    ) -> Self::Input {
        let h = config.hysteresis_margin;
        let qe_a = normalize_scalar(input.qe_a, &config.bands, h, None).0;
        let qe_b = normalize_scalar(input.qe_b, &config.bands, h, None).0;
        let interference = quantize_precision(input.interference, 2);
        let conductivity = quantize_precision(input.conductivity, 3);
        let dt = quantize_precision(input.dt, 4);
        Self::Input {
            qe_a,
            qe_b,
            interference,
            conductivity,
            dt,
        }
    }

    fn cache_key(normalized: Self::Input) -> u64 {
        hash_inputs(&[
            f32::to_bits(normalized.qe_a) as u64,
            f32::to_bits(normalized.qe_b) as u64,
            f32::to_bits(normalized.interference) as u64,
            f32::to_bits(normalized.conductivity) as u64,
            f32::to_bits(normalized.dt) as u64,
        ])
    }

    fn compute(normalized: Self::Input) -> Self::Output {
        equations::collision_transfer(
            normalized.qe_a,
            normalized.qe_b,
            normalized.interference,
            normalized.conductivity,
            normalized.dt,
        )
    }

    fn into_cached(value: Self::Output) -> CachedValue {
        CachedValue::Scalar(value)
    }

    fn from_cached(value: CachedValue) -> Option<Self::Output> {
        match value {
            CachedValue::Scalar(s) => Some(s),
            _ => None,
        }
    }
}

impl Bridgeable for OsmosisBridge {
    type Input = OsmosisEquationInput;
    type Output = f32;

    fn normalize(
        input: Self::Input,
        config: &BridgeConfig<Self>,
        _band_hint: Option<usize>,
    ) -> Self::Input {
        let h = config.hysteresis_margin;
        let concentration_a = normalize_scalar(input.concentration_a, &config.bands, h, None).0;
        let concentration_b = normalize_scalar(input.concentration_b, &config.bands, h, None).0;
        let membrane_permeability = quantize_precision(input.membrane_permeability, 4);
        Self::Input {
            concentration_a,
            concentration_b,
            membrane_permeability,
        }
    }

    fn cache_key(normalized: Self::Input) -> u64 {
        hash_inputs(&[
            f32::to_bits(normalized.concentration_a) as u64,
            f32::to_bits(normalized.concentration_b) as u64,
            f32::to_bits(normalized.membrane_permeability) as u64,
        ])
    }

    fn compute(normalized: Self::Input) -> Self::Output {
        equations::osmotic_pressure_delta(
            normalized.concentration_a,
            normalized.concentration_b,
            normalized.membrane_permeability,
        )
    }

    fn into_cached(value: Self::Output) -> CachedValue {
        CachedValue::Scalar(value)
    }

    fn from_cached(value: CachedValue) -> Option<Self::Output> {
        match value {
            CachedValue::Scalar(s) => Some(s),
            _ => None,
        }
    }
}

// --- Fast-path colisión (misma clave empaquetada que el cache) ---------------

#[derive(Clone, Copy, Debug, Default)]
pub struct CollisionTransferScratch {
    last_packed: Option<u64>,
    last_out: f32,
}

impl CollisionTransferScratch {
    #[inline]
    pub fn try_hit(&mut self, packed: u64) -> Option<f32> {
        if self.last_packed == Some(packed) {
            return Some(self.last_out);
        }
        None
    }

    #[inline]
    pub fn record(&mut self, packed: u64, out: f32) {
        self.last_packed = Some(packed);
        self.last_out = out;
    }

    #[inline]
    pub fn clear(&mut self) {
        self.last_packed = None;
    }
}

// --- SystemParam ------------------------------------------------------------

#[derive(SystemParam)]
pub struct BridgedInterferenceOps<'w, 's> {
    waves: Query<'w, 's, &'static OscillatorySignature>,
    identities: Query<'w, 's, &'static MobaIdentity>,
    phase_state: Res<'w, BridgePhaseState>,
    sim_elapsed: Option<Res<'w, SimulationElapsed>>,
    time: Res<'w, Time>,
    almanac: Res<'w, AlchemicalAlmanac>,
    interference_config: Res<'w, BridgeConfig<InterferenceBridge>>,
    interference_cache: ResMut<'w, BridgeCache<InterferenceBridge>>,
    catalysis_config: Res<'w, BridgeConfig<CatalysisBridge>>,
    catalysis_cache: ResMut<'w, BridgeCache<CatalysisBridge>>,
    collision_config: Res<'w, BridgeConfig<CollisionTransferBridge>>,
    collision_cache: ResMut<'w, BridgeCache<CollisionTransferBridge>>,
    collision_scratch: Local<'s, CollisionTransferScratch>,
}

impl<'w, 's> BridgedInterferenceOps<'w, 's> {
    #[inline]
    fn phase_time_secs(&self) -> f32 {
        self.sim_elapsed
            .as_ref()
            .map(|sim| sim.secs)
            .unwrap_or_else(|| self.time.elapsed_secs())
    }

    /// Replica `InterferenceOps::between` con bridges opcionales en la componente física pura.
    pub fn between(&mut self, a: Entity, b: Entity) -> Option<f32> {
        let wave_a = self.waves.get(a).ok()?;
        let wave_b = self.waves.get(b).ok()?;
        let t = self.phase_time_secs();
        let faction_mod = match (self.identities.get(a).ok(), self.identities.get(b).ok()) {
            (Some(id_a), Some(id_b)) => id_a.faction_modifier(id_b),
            _ => 0.0,
        };

        let mut input = InterferenceEquationInput {
            f1: wave_a.frequency_hz,
            phase1: wave_a.phase,
            f2: wave_b.frequency_hz,
            phase2: wave_b.phase,
            t,
        };

        if self.interference_config.enabled {
            let bands = &self.interference_config.bands;
            let h = self.interference_config.hysteresis_margin;
            input.f1 = canonicalize_frequency_hz(&self.almanac, input.f1, bands, h);
            input.f2 = canonicalize_frequency_hz(&self.almanac, input.f2, bands, h);
        }

        if self.phase_state.phase == BridgePhase::Warmup {
            let raw_out = equations::interference(
                wave_a.frequency_hz,
                wave_a.phase,
                wave_b.frequency_hz,
                wave_b.phase,
                t,
            );
            interference_warmup_insert(
                &mut self.interference_cache,
                &self.interference_config,
                &self.almanac,
                input,
                raw_out,
            );
            return Some(compose_interference(raw_out, faction_mod));
        }

        let raw = bridge_compute(
            input,
            &self.interference_config,
            &mut self.interference_cache,
        );
        Some(compose_interference(raw, faction_mod))
    }

    pub fn catalysis_result(&mut self, input: CatalysisEquationInput) -> f32 {
        if self.phase_state.phase == BridgePhase::Warmup {
            let raw = equations::catalysis_result(
                input.projected_qe,
                input.interference,
                input.critical_multiplier,
            );
            let n = <CatalysisBridge as Bridgeable>::normalize(input, &self.catalysis_config, None);
            let k = <CatalysisBridge as Bridgeable>::cache_key(n);
            self.catalysis_cache.insert(k, CachedValue::Scalar(raw));
            return raw;
        }
        bridge_compute(input, &self.catalysis_config, &mut self.catalysis_cache)
    }

    pub fn collision_transfer(&mut self, input: CollisionTransferEquationInput) -> f32 {
        let cfg = &self.collision_config;
        let scratch = &mut *self.collision_scratch;

        if self.phase_state.phase == BridgePhase::Warmup {
            let out = equations::collision_transfer(
                input.qe_a,
                input.qe_b,
                input.interference,
                input.conductivity,
                input.dt,
            );
            let n = <CollisionTransferBridge as Bridgeable>::normalize(input, cfg, None);
            let k = <CollisionTransferBridge as Bridgeable>::cache_key(n);
            self.collision_cache.insert(k, CachedValue::Scalar(out));
            scratch.record(k, out);
            return out;
        }

        if !cfg.enabled {
            scratch.clear();
            return equations::collision_transfer(
                input.qe_a,
                input.qe_b,
                input.interference,
                input.conductivity,
                input.dt,
            );
        }

        let normalized = <CollisionTransferBridge as Bridgeable>::normalize(input, cfg, None);
        let packed = <CollisionTransferBridge as Bridgeable>::cache_key(normalized);
        if let Some(v) = scratch.try_hit(packed) {
            return v;
        }

        let out = bridge_compute(input, cfg, &mut self.collision_cache);
        scratch.record(packed, out);
        out
    }

    pub fn interference_equation(&mut self, input: InterferenceEquationInput) -> f32 {
        if self.phase_state.phase == BridgePhase::Warmup {
            let raw_out =
                equations::interference(input.f1, input.phase1, input.f2, input.phase2, input.t);
            interference_warmup_insert(
                &mut self.interference_cache,
                &self.interference_config,
                &self.almanac,
                input,
                raw_out,
            );
            return raw_out;
        }
        bridge_compute(
            input,
            &self.interference_config,
            &mut self.interference_cache,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::ecs::system::SystemState;
    use bevy::prelude::World;

    use crate::bridge::Bridgeable;
    use crate::bridge::config::CachePolicy;
    use crate::bridge::presets::{BridgeDefaults, RigidityPreset};
    use crate::layers::oscillatory::{InterferenceOps, compute_interference_total};

    fn interference_cfg(enabled: bool) -> BridgeConfig<InterferenceBridge> {
        let mut c = InterferenceBridge::config_for_preset(RigidityPreset::Moderate);
        c.enabled = enabled;
        c
    }

    fn catalysis_cfg(enabled: bool) -> BridgeConfig<CatalysisBridge> {
        let mut c = CatalysisBridge::config_for_preset(RigidityPreset::Moderate);
        c.enabled = enabled;
        c
    }

    fn collision_cfg(enabled: bool) -> BridgeConfig<CollisionTransferBridge> {
        let mut c = CollisionTransferBridge::config_for_preset(RigidityPreset::Moderate);
        c.enabled = enabled;
        c
    }

    #[test]
    fn interference_bridge_disabled_bit_exact() {
        let cfg = interference_cfg(false);
        let mut cache = BridgeCache::<InterferenceBridge>::new(64, CachePolicy::Lru);
        let input = InterferenceEquationInput {
            f1: 111.3,
            phase1: 0.4,
            f2: 333.7,
            phase2: 1.9,
            t: 12.345,
        };
        let out = bridge_compute(input, &cfg, &mut cache);
        let direct =
            equations::interference(input.f1, input.phase1, input.f2, input.phase2, input.t);
        assert_eq!(out, direct);
    }

    #[test]
    fn interference_enabled_matches_normalized_equation() {
        let cfg = interference_cfg(true);
        let mut cache = BridgeCache::<InterferenceBridge>::new(512, CachePolicy::Lru);
        let input = InterferenceEquationInput {
            f1: 450.0,
            phase1: 0.31,
            f2: 700.0,
            phase2: 1.12,
            t: 3.33,
        };
        let bridged = bridge_compute(input, &cfg, &mut cache);
        let n = <InterferenceBridge as Bridgeable>::normalize(input, &cfg, None);
        let expected = equations::interference(n.f1, n.phase1, n.f2, n.phase2, n.t);
        assert!(
            (bridged - expected).abs() < 1e-4,
            "b={} e={}",
            bridged,
            expected
        );
    }

    #[test]
    fn interference_cache_hit_rate_over_50_percent() {
        let cfg = interference_cfg(true);
        let mut cache = BridgeCache::<InterferenceBridge>::new(1000, CachePolicy::Lru);
        let base = InterferenceEquationInput {
            f1: 400.0,
            phase1: 0.5,
            f2: 500.0,
            phase2: 1.0,
            t: 1.0,
        };
        for i in 0..200 {
            let mut x = base;
            x.t = base.t + (i as f32) * 0.001;
            let _ = bridge_compute(x, &cfg, &mut cache);
        }
        let s = cache.stats();
        let total = s.hits + s.misses;
        assert!(total > 0);
        let rate = s.hits as f32 / total as f32;
        assert!(
            rate > 0.5,
            "hit_rate={} hits={} misses={}",
            rate,
            s.hits,
            s.misses
        );
    }

    #[test]
    fn interference_packed_key_deterministic() {
        let n = InterferenceEquationInput {
            f1: 100.0,
            phase1: 0.0,
            f2: 300.0,
            phase2: 0.0,
            t: 1.0,
        };
        let a = <InterferenceBridge as Bridgeable>::cache_key(n);
        let b = <InterferenceBridge as Bridgeable>::cache_key(n);
        assert_eq!(a, b);
    }

    #[test]
    fn catalysis_cache_key_deterministic() {
        let n = CatalysisEquationInput {
            projected_qe: 42.0,
            interference: 0.5,
            critical_multiplier: 1.5,
        };
        let k1 = <CatalysisBridge as Bridgeable>::cache_key(n);
        let k2 = <CatalysisBridge as Bridgeable>::cache_key(n);
        assert_eq!(k1, k2);
    }

    #[test]
    fn collision_cache_key_deterministic() {
        let n = CollisionTransferEquationInput {
            qe_a: 10.0,
            qe_b: 20.0,
            interference: 0.25,
            conductivity: 0.4,
            dt: 1.0 / 60.0,
        };
        let k1 = <CollisionTransferBridge as Bridgeable>::cache_key(n);
        let k2 = <CollisionTransferBridge as Bridgeable>::cache_key(n);
        assert_eq!(k1, k2);
    }

    #[test]
    fn catalysis_random_100_matches_normalized_equation() {
        let cfg = catalysis_cfg(true);
        let mut cache = BridgeCache::<CatalysisBridge>::new(256, CachePolicy::Lru);
        for i in 0..100 {
            let s = i as f32 * 0.713;
            let input = CatalysisEquationInput {
                projected_qe: 10.0 + (s * 17.0).sin() * 50.0,
                interference: -0.9 + (s * 3.0).sin() * 0.15,
                critical_multiplier: 1.0 + (i % 5) as f32 * 0.1,
            };
            let b = bridge_compute(input, &cfg, &mut cache);
            let n = <CatalysisBridge as Bridgeable>::normalize(input, &cfg, None);
            let e =
                equations::catalysis_result(n.projected_qe, n.interference, n.critical_multiplier);
            assert!((b - e).abs() < 1e-4, "i={} b={} e={}", i, b, e);
        }
    }

    #[test]
    fn catalysis_disabled_matches_equation() {
        let cfg = catalysis_cfg(false);
        let mut cache = BridgeCache::<CatalysisBridge>::new(16, CachePolicy::Lru);
        let input = CatalysisEquationInput {
            projected_qe: 78.3,
            interference: 0.72,
            critical_multiplier: 2.0,
        };
        let b = bridge_compute(input, &cfg, &mut cache);
        assert_eq!(b, equations::catalysis_result(78.3, 0.72, 2.0));
    }

    #[test]
    fn collision_transfer_random_matches_normalized_equation() {
        let cfg = collision_cfg(true);
        let mut cache = BridgeCache::<CollisionTransferBridge>::new(256, CachePolicy::Lru);
        for i in 0..80 {
            let input = CollisionTransferEquationInput {
                qe_a: 50.0 + i as f32,
                qe_b: 40.0 + (i as f32) * 0.5,
                interference: 0.3 + (i as f32 * 0.07).sin() * 0.2,
                conductivity: 0.5,
                dt: 1.0 / 60.0,
            };
            let b = bridge_compute(input, &cfg, &mut cache);
            let n = <CollisionTransferBridge as Bridgeable>::normalize(input, &cfg, None);
            let e =
                equations::collision_transfer(n.qe_a, n.qe_b, n.interference, n.conductivity, n.dt);
            assert!((b - e).abs() < 1e-3, "b={} e={}", b, e);
        }
    }

    #[test]
    fn collision_transfer_disabled_bit_exact() {
        let cfg = collision_cfg(false);
        let mut cache = BridgeCache::<CollisionTransferBridge>::new(16, CachePolicy::Lru);
        let input = CollisionTransferEquationInput {
            qe_a: 88.0,
            qe_b: 77.0,
            interference: -0.4,
            conductivity: 0.35,
            dt: 1.0 / 120.0,
        };
        let b = bridge_compute(input, &cfg, &mut cache);
        assert_eq!(
            b,
            equations::collision_transfer(
                input.qe_a,
                input.qe_b,
                input.interference,
                input.conductivity,
                input.dt,
            )
        );
    }

    #[test]
    fn collision_scratch_skips_second_hash_lookup_simulation() {
        let cfg = collision_cfg(true);
        let mut cache = BridgeCache::<CollisionTransferBridge>::new(128, CachePolicy::Lru);
        let mut scratch = CollisionTransferScratch::default();
        let input = CollisionTransferEquationInput {
            qe_a: 100.0,
            qe_b: 90.0,
            interference: 0.5,
            conductivity: 0.2,
            dt: 1.0 / 60.0,
        };
        let n = <CollisionTransferBridge as Bridgeable>::normalize(input, &cfg, None);
        let packed = <CollisionTransferBridge as Bridgeable>::cache_key(n);
        let o1 = bridge_compute(input, &cfg, &mut cache);
        scratch.record(packed, o1);
        assert_eq!(scratch.try_hit(packed), Some(o1));
    }

    #[test]
    fn compose_interference_total_matches_bridged_normalized_pipeline() {
        let cfg = interference_cfg(true);
        let mut cache = BridgeCache::<InterferenceBridge>::new(64, CachePolicy::Lru);
        let f1 = 450.0_f32;
        let p1 = 0.2_f32;
        let f2 = 700.0_f32;
        let p2 = 1.1_f32;
        let t = 0.5_f32;
        let faction = 0.15_f32;
        let input = InterferenceEquationInput {
            f1,
            phase1: p1,
            f2,
            phase2: p2,
            t,
        };
        let raw_b = bridge_compute(input, &cfg, &mut cache);
        let n = <InterferenceBridge as Bridgeable>::normalize(input, &cfg, None);
        let total_ref = compose_interference(
            equations::interference(n.f1, n.phase1, n.f2, n.phase2, n.t),
            faction,
        );
        let total_br = compose_interference(raw_b, faction);
        assert!((total_ref - total_br).abs() < 1e-4);
        // Con bridge desactivado, coincide con el cómputo exacto sin cuantizar (véase `interference_bridge_disabled_bit_exact`).
        let total_exact = compute_interference_total(f1, p1, f2, p2, t, faction);
        let raw_disabled = bridge_compute(
            input,
            &interference_cfg(false),
            &mut BridgeCache::<InterferenceBridge>::new(16, CachePolicy::Lru),
        );
        assert!((compose_interference(raw_disabled, faction) - total_exact).abs() < 1e-5);
    }

    /// Paridad `InterferenceOps::between` vs `BridgedInterferenceOps::between` con bridge desactivado.
    #[test]
    fn between_matches_interference_ops_when_bridge_disabled() {
        let mut world = World::new();
        world.insert_resource(Time::<()>::default());
        world.insert_resource(AlchemicalAlmanac::default());
        let mut icfg = InterferenceBridge::config_for_preset(RigidityPreset::Moderate);
        icfg.enabled = false;
        world.insert_resource(icfg);
        world.insert_resource(CatalysisBridge::config_for_preset(RigidityPreset::Moderate));
        world.insert_resource(CollisionTransferBridge::config_for_preset(
            RigidityPreset::Moderate,
        ));
        world.insert_resource(BridgeCache::<InterferenceBridge>::new(64, CachePolicy::Lru));
        world.insert_resource(BridgeCache::<CatalysisBridge>::new(64, CachePolicy::Lru));
        world.insert_resource(BridgeCache::<CollisionTransferBridge>::new(
            64,
            CachePolicy::Lru,
        ));
        world.insert_resource(crate::bridge::context_fill::BridgePhaseState::active_only());

        let a = world.spawn(OscillatorySignature::new(400.0, 0.3)).id();
        let b = world.spawn(OscillatorySignature::new(500.0, 1.2)).id();

        let v_io = {
            let mut st = SystemState::<InterferenceOps>::new(&mut world);
            let p = st.get_mut(&mut world);
            p.between(a, b).expect("between io")
        };

        let v_bio = {
            let mut st = SystemState::<BridgedInterferenceOps>::new(&mut world);
            let mut p = st.get_mut(&mut world);
            p.between(a, b).expect("between bridged")
        };

        assert!((v_io - v_bio).abs() < 1e-5, "io={} bio={}", v_io, v_bio);
    }
}
