//! `BridgedPhysicsOps` — wrapper de [`crate::layers::PhysicsOps`] con cuantización + cache (sprint B4).
//! Requiere feature `bridge_optimizer`. Ver `docs/sprints/BRIDGE_OPTIMIZER/README.md` y `docs/arquitectura/blueprint_layer_bridge_optimizer.md` §5.

use bevy::ecs::system::SystemParam;
use bevy::prelude::*;

use crate::blueprint::equations;
use crate::bridge::cache::{BridgeCache, CachedValue};
use crate::bridge::config::{
    BridgeConfig, DensityBridge, PhaseTransitionBridge, TemperatureBridge,
};
use crate::bridge::context_fill::{BridgePhase, BridgePhaseState};
use crate::bridge::decorator::{
    Bridgeable, bridge_compute, bridge_compute_with_hint, bridge_warmup_record,
    bridge_warmup_record_with_hint, hash_inputs,
};
use crate::bridge::normalize::normalize_scalar;
use crate::layers::{MatterState, PhysicsOps};

// --- Bridgeable: densidad / temperatura / fase ------------------------------------------------

impl Bridgeable for DensityBridge {
    type Input = f32;
    type Output = f32;

    fn normalize(
        input: Self::Input,
        config: &BridgeConfig<Self>,
        band_hint: Option<usize>,
    ) -> Self::Input {
        normalize_scalar(input, &config.bands, config.hysteresis_margin, band_hint).0
    }

    fn cache_key(normalized: Self::Input) -> u64 {
        hash_inputs(&[f32::to_bits(normalized) as u64])
    }

    fn compute(normalized: Self::Input) -> Self::Output {
        normalized
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

impl Bridgeable for TemperatureBridge {
    type Input = f32;
    type Output = f32;

    fn normalize(
        input: Self::Input,
        config: &BridgeConfig<Self>,
        band_hint: Option<usize>,
    ) -> Self::Input {
        normalize_scalar(input, &config.bands, config.hysteresis_margin, band_hint).0
    }

    fn cache_key(normalized: Self::Input) -> u64 {
        hash_inputs(&[f32::to_bits(normalized) as u64])
    }

    fn compute(normalized: Self::Input) -> Self::Output {
        equations::equivalent_temperature(normalized)
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

impl Bridgeable for PhaseTransitionBridge {
    type Input = (f32, f32);
    type Output = MatterState;

    fn normalize(
        input: Self::Input,
        config: &BridgeConfig<Self>,
        band_hint: Option<usize>,
    ) -> Self::Input {
        let (temp, bond) = input;
        let (t_canon, _) =
            normalize_scalar(temp, &config.bands, config.hysteresis_margin, band_hint);
        (t_canon, bond)
    }

    fn cache_key(normalized: Self::Input) -> u64 {
        let (t, b) = normalized;
        hash_inputs(&[f32::to_bits(t) as u64, f32::to_bits(b) as u64])
    }

    fn compute(normalized: Self::Input) -> Self::Output {
        equations::state_from_temperature(normalized.0, normalized.1)
    }

    fn into_cached(value: Self::Output) -> CachedValue {
        CachedValue::State(value)
    }

    fn from_cached(value: CachedValue) -> Option<Self::Output> {
        match value {
            CachedValue::State(s) => Some(s),
            _ => None,
        }
    }
}

/// Índice de banda de temperatura alineado a `PHASE_MOD` en `presets.rs`.
#[inline]
pub fn phase_band_hint_from_state(state: MatterState) -> Option<usize> {
    Some(match state {
        MatterState::Solid => 0,
        MatterState::Liquid => 1,
        MatterState::Gas => 2,
        MatterState::Plasma => 3,
    })
}

// --- SystemParam ------------------------------------------------------------------------------

#[derive(SystemParam)]
pub struct BridgedPhysicsOps<'w, 's> {
    pub inner: PhysicsOps<'w, 's>,
    pub phase_state: Res<'w, BridgePhaseState>,
    pub density_cfg: Res<'w, BridgeConfig<DensityBridge>>,
    pub density_cache: ResMut<'w, BridgeCache<DensityBridge>>,
    pub temperature_cfg: Res<'w, BridgeConfig<TemperatureBridge>>,
    pub temperature_cache: ResMut<'w, BridgeCache<TemperatureBridge>>,
    pub phase_cfg: Res<'w, BridgeConfig<PhaseTransitionBridge>>,
    pub phase_cache: ResMut<'w, BridgeCache<PhaseTransitionBridge>>,
}

impl<'w, 's> BridgedPhysicsOps<'w, 's> {
    /// Misma lectura que [`PhysicsOps::matter_state`] (sin bridge).
    #[inline]
    pub fn matter_state(&self, entity: Entity) -> Option<MatterState> {
        self.inner.matter_state(entity)
    }

    pub fn density(&mut self, entity: Entity) -> Option<f32> {
        let raw = self.inner.density(entity)?;
        Some(match self.phase_state.phase {
            // Salida exacta al caller; en cache va el canónico para que los hits en Active coincidan con `compute(normalize(·))`.
            BridgePhase::Warmup => {
                let norm = <DensityBridge as Bridgeable>::normalize(raw, &self.density_cfg, None);
                let key = <DensityBridge as Bridgeable>::cache_key(norm);
                let stored = <DensityBridge as Bridgeable>::compute(norm);
                self.density_cache
                    .insert(key, <DensityBridge as Bridgeable>::into_cached(stored));
                raw
            }
            _ => bridge_compute::<DensityBridge>(raw, &self.density_cfg, &mut self.density_cache),
        })
    }

    pub fn temperature(&mut self, entity: Entity) -> Option<f32> {
        match self.phase_state.phase {
            BridgePhase::Warmup => {
                let d = self.density(entity)?;
                Some(bridge_warmup_record::<TemperatureBridge>(
                    d,
                    &self.temperature_cfg,
                    &mut self.temperature_cache,
                ))
            }
            _ => {
                let d = self.density(entity)?;
                Some(bridge_compute::<TemperatureBridge>(
                    d,
                    &self.temperature_cfg,
                    &mut self.temperature_cache,
                ))
            }
        }
    }

    /// Histéresis: `band_hint` = estado actual de la entidad (`matter_state`).
    pub fn matter_state_from_temperature(
        &mut self,
        entity: Entity,
        temp: f32,
        bond_energy: f32,
    ) -> MatterState {
        let hint = self
            .inner
            .matter_state(entity)
            .and_then(phase_band_hint_from_state);
        match self.phase_state.phase {
            BridgePhase::Warmup => bridge_warmup_record_with_hint::<PhaseTransitionBridge>(
                (temp, bond_energy),
                &self.phase_cfg,
                &mut self.phase_cache,
                hint,
            ),
            _ => bridge_compute_with_hint::<PhaseTransitionBridge>(
                (temp, bond_energy),
                &self.phase_cfg,
                &mut self.phase_cache,
                hint,
            ),
        }
    }

    pub fn velocity_limit(&self, entity: Entity) -> f32 {
        self.inner.velocity_limit(entity)
    }

    pub fn is_solid(&self, entity: Entity) -> bool {
        self.inner.is_solid(entity)
    }

    pub fn dissipation_multiplier(&self, entity: Entity) -> f32 {
        self.inner.dissipation_multiplier(entity)
    }

    pub fn conductivity(&self, entity: Entity) -> f32 {
        self.inner.conductivity(entity)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bridge::config::{BandDef, CachePolicy, Rigidity};
    use crate::bridge::context_fill::BridgePhaseState;
    use crate::bridge::presets::{BridgeDefaults, RigidityPreset};
    use crate::layers::{BaseEnergy, MatterCoherence, SpatialVolume};
    use bevy::ecs::system::SystemState;

    fn narrow_bands() -> Vec<BandDef> {
        vec![
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
        ]
    }

    #[test]
    fn enabled_density_and_temperature_near_physics_ops_on_entity() {
        let mut world = World::new();
        world.insert_resource(DensityBridge::config_for_preset(RigidityPreset::Moderate));
        world.insert_resource(TemperatureBridge::config_for_preset(
            RigidityPreset::Moderate,
        ));
        world.insert_resource(PhaseTransitionBridge::config_for_preset(
            RigidityPreset::Moderate,
        ));
        world.insert_resource(BridgeCache::<DensityBridge>::new(256, CachePolicy::Lru));
        world.insert_resource(BridgeCache::<TemperatureBridge>::new(256, CachePolicy::Lru));
        world.insert_resource(BridgeCache::<PhaseTransitionBridge>::new(
            256,
            CachePolicy::Lru,
        ));
        world.insert_resource(BridgePhaseState::active_only());

        let entity = world
            .spawn((
                BaseEnergy::new(100.0),
                SpatialVolume::new(2.0),
                MatterCoherence::default(),
            ))
            .id();

        let cfg_d = DensityBridge::config_for_preset(RigidityPreset::Moderate);
        let cfg_t = TemperatureBridge::config_for_preset(RigidityPreset::Moderate);
        let max_d = max_band_span(&cfg_d.bands);
        let max_t = max_band_span(&cfg_t.bands);

        let mut state = SystemState::<(PhysicsOps, BridgedPhysicsOps)>::new(&mut world);
        let (physics, mut bridged) = state.get_mut(&mut world);

        let d0 = physics.density(entity).unwrap();
        let d1 = bridged.density(entity).unwrap();
        assert!((d1 - d0).abs() <= max_d + 1e-3);

        let t0 = physics.temperature(entity).unwrap();
        let t1 = bridged.temperature(entity).unwrap();
        assert!((t1 - t0).abs() <= max_d + max_t + 1e-3);
    }

    #[test]
    fn disabled_matches_physics_ops_bitwise() {
        let mut world = World::new();
        world.insert_resource(
            BridgeConfig::<DensityBridge>::new(
                narrow_bands(),
                1.0,
                32,
                CachePolicy::Lru,
                false,
                Rigidity::Moderate,
            )
            .expect("bands"),
        );
        world.insert_resource(
            BridgeConfig::<TemperatureBridge>::new(
                narrow_bands(),
                1.0,
                32,
                CachePolicy::Lru,
                false,
                Rigidity::Moderate,
            )
            .expect("bands"),
        );
        world.insert_resource(
            BridgeConfig::<PhaseTransitionBridge>::new(
                narrow_bands(),
                1.0,
                32,
                CachePolicy::Lru,
                false,
                Rigidity::Moderate,
            )
            .expect("bands"),
        );
        world.insert_resource(BridgeCache::<DensityBridge>::new(32, CachePolicy::Lru));
        world.insert_resource(BridgeCache::<TemperatureBridge>::new(32, CachePolicy::Lru));
        world.insert_resource(BridgeCache::<PhaseTransitionBridge>::new(
            32,
            CachePolicy::Lru,
        ));
        world.insert_resource(BridgePhaseState::active_only());

        let entity = world
            .spawn((
                BaseEnergy::new(100.0),
                SpatialVolume::new(2.0),
                MatterCoherence::default(),
            ))
            .id();

        let mut state = SystemState::<(PhysicsOps, BridgedPhysicsOps)>::new(&mut world);
        let (physics, mut bridged) = state.get_mut(&mut world);

        assert_eq!(
            physics.density(entity).unwrap(),
            bridged.density(entity).unwrap()
        );
        assert_eq!(
            physics.temperature(entity).unwrap(),
            bridged.temperature(entity).unwrap()
        );
    }

    #[test]
    fn density_second_call_cache_hit() {
        let mut world = World::new();
        world.insert_resource(DensityBridge::config_for_preset(RigidityPreset::Moderate));
        world.insert_resource(TemperatureBridge::config_for_preset(
            RigidityPreset::Moderate,
        ));
        world.insert_resource(PhaseTransitionBridge::config_for_preset(
            RigidityPreset::Moderate,
        ));
        world.insert_resource(BridgeCache::<DensityBridge>::new(
            DensityBridge::config_for_preset(RigidityPreset::Moderate).cache_capacity,
            CachePolicy::Lru,
        ));
        world.insert_resource(BridgeCache::<TemperatureBridge>::new(
            TemperatureBridge::config_for_preset(RigidityPreset::Moderate).cache_capacity,
            CachePolicy::Lru,
        ));
        world.insert_resource(BridgeCache::<PhaseTransitionBridge>::new(
            PhaseTransitionBridge::config_for_preset(RigidityPreset::Moderate).cache_capacity,
            CachePolicy::Lru,
        ));
        world.insert_resource(BridgePhaseState::active_only());

        let entity = world
            .spawn((
                BaseEnergy::new(100.0),
                SpatialVolume::new(2.0),
                MatterCoherence::default(),
            ))
            .id();

        {
            let mut state = SystemState::<BridgedPhysicsOps>::new(&mut world);
            let mut bridged = state.get_mut(&mut world);
            let _ = bridged.density(entity);
            let _ = bridged.density(entity);
        }

        let hits = world.resource::<BridgeCache<DensityBridge>>().stats().hits;
        assert!(hits >= 1);
    }

    #[test]
    fn phase_hysteresis_near_threshold() {
        let mut world = World::new();
        world.insert_resource(DensityBridge::config_for_preset(RigidityPreset::Moderate));
        world.insert_resource(TemperatureBridge::config_for_preset(
            RigidityPreset::Moderate,
        ));
        world.insert_resource(PhaseTransitionBridge::config_for_preset(
            RigidityPreset::Moderate,
        ));
        world.insert_resource(BridgeCache::<DensityBridge>::new(64, CachePolicy::Lru));
        world.insert_resource(BridgeCache::<TemperatureBridge>::new(64, CachePolicy::Lru));
        world.insert_resource(BridgeCache::<PhaseTransitionBridge>::new(
            64,
            CachePolicy::Lru,
        ));
        world.insert_resource(BridgePhaseState::active_only());

        let entity = world
            .spawn((
                BaseEnergy::new(100.0),
                SpatialVolume::new(2.0),
                MatterCoherence::new(MatterState::Solid, 100.0, 0.5),
            ))
            .id();

        let bond = 100.0_f32;
        let temp_borderline = 30.5;
        assert_eq!(
            equations::state_from_temperature(temp_borderline, bond),
            MatterState::Liquid
        );

        let mut state = SystemState::<BridgedPhysicsOps>::new(&mut world);
        let mut bridged = state.get_mut(&mut world);
        assert_eq!(
            bridged.matter_state_from_temperature(entity, temp_borderline, bond),
            MatterState::Solid
        );
        assert_eq!(
            bridged.matter_state_from_temperature(entity, 36.0, bond),
            MatterState::Liquid
        );
    }

    #[test]
    fn random_inputs_within_bridge_epsilon_per_equation() {
        let mut world = World::new();
        world.insert_resource(DensityBridge::config_for_preset(RigidityPreset::Moderate));
        world.insert_resource(TemperatureBridge::config_for_preset(
            RigidityPreset::Moderate,
        ));
        world.insert_resource(PhaseTransitionBridge::config_for_preset(
            RigidityPreset::Moderate,
        ));
        world.insert_resource(BridgeCache::<DensityBridge>::new(256, CachePolicy::Lru));
        world.insert_resource(BridgeCache::<TemperatureBridge>::new(256, CachePolicy::Lru));
        world.insert_resource(BridgeCache::<PhaseTransitionBridge>::new(
            256,
            CachePolicy::Lru,
        ));

        let cfg_d0 = world.resource::<BridgeConfig<DensityBridge>>().clone();
        let lo = cfg_d0.bands.first().expect("bands").min;
        let hi = cfg_d0.bands.last().expect("bands").max;

        for i in 0..1000 {
            // Mantener densidad dentro del dominio cubierto por bandas; fuera de rango el clamp
            // a canónicos extremos puede exceder el ancho de una banda (ver `normalize_scalar`).
            let t = (i as f32 * 0.017).sin() * 0.5 + 0.5;
            let d_raw = lo + t * (hi - lo);
            let cfg_d = world.resource::<BridgeConfig<DensityBridge>>().clone();
            let cfg_t = world.resource::<BridgeConfig<TemperatureBridge>>().clone();
            let mut cd = BridgeCache::<DensityBridge>::new(256, CachePolicy::Lru);
            let mut ct = BridgeCache::<TemperatureBridge>::new(256, CachePolicy::Lru);
            let d_b = bridge_compute::<DensityBridge>(d_raw, &cfg_d, &mut cd);
            let max_d = max_band_span(&cfg_d.bands);
            assert!((d_b - d_raw).abs() <= max_d + 1e-3, "density i={i}");

            // Temperatura: entrada = densidad ya pasada por puente de densidad (misma cadena que `BridgedPhysicsOps`).
            let t_exact_on_bridged_d = equations::equivalent_temperature(d_b);
            let t_b = bridge_compute::<TemperatureBridge>(d_b, &cfg_t, &mut ct);
            let max_t = max_band_span(&cfg_t.bands);
            assert!(
                (t_b - t_exact_on_bridged_d).abs() <= max_t + 1e-3,
                "temperature i={i} t_exact={t_exact_on_bridged_d} t_b={t_b}"
            );
        }
    }

    /// Cota conservadora: |raw − canónico| no supera el ancho de la banda más ancha.
    fn max_band_span(bands: &[BandDef]) -> f32 {
        bands.iter().map(|b| b.max - b.min).fold(0.0_f32, f32::max)
    }

    #[test]
    fn phase_disabled_matches_equations() {
        let mut cfg = PhaseTransitionBridge::config_for_preset(RigidityPreset::Moderate);
        cfg.enabled = false;
        let mut cache = BridgeCache::<PhaseTransitionBridge>::new(64, CachePolicy::Lru);
        for i in 0..200 {
            let t = (i as f32 * 0.37).sin() * 200.0 + 100.0;
            let b = 50.0 + (i as f32 * 0.11).cos() * 40.0;
            let out =
                bridge_compute_with_hint::<PhaseTransitionBridge>((t, b), &cfg, &mut cache, None);
            assert_eq!(out, equations::state_from_temperature(t, b));
        }
    }
}
