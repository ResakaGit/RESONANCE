//! Pipeline context-fill: Warmup → Filling → Active (`docs/sprints/BRIDGE_OPTIMIZER/README.md`, blueprint §7).
//!
//! Orquesta flags `BridgeConfig::enabled`, `BridgeCache::eviction_enabled` y el decorador
//! `bridge_warmup_record*` sin tocar `equations.rs`.

use bevy::prelude::*;

use crate::bridge::cache::BridgeCache;
use crate::bridge::config::{
    BridgeConfig, CatalysisBridge, CollisionTransferBridge, DensityBridge, DissipationBridge,
    DragBridge, EngineBridge, InterferenceBridge, OsmosisBridge, PhaseTransitionBridge,
    TemperatureBridge, WillBridge,
};

// --- Fase y eventos ---------------------------------------------------------------------------

/// Fases globales del optimizer (una sola máquina de estados para todos los puentes).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Reflect)]
pub enum BridgePhase {
    #[default]
    Warmup,
    Filling,
    Active,
}

/// Umbrales de transición — data-driven (resource), no hardcode en lógica.
#[derive(Resource, Clone, Debug, PartialEq)]
pub struct BridgePhaseConfig {
    pub warmup_ticks: u32,
    pub filling_max_ticks: u32,
    /// Transición Filling → Active si `max(len/capacity)` ≥ este valor.
    pub fill_ratio_for_active: f32,
}

impl Default for BridgePhaseConfig {
    fn default() -> Self {
        Self {
            warmup_ticks: 100,
            filling_max_ticks: 50,
            fill_ratio_for_active: 0.8,
        }
    }
}

/// Estado runtime del pipeline (contadores por fase).
#[derive(Resource, Clone, Debug, PartialEq)]
pub struct BridgePhaseState {
    pub phase: BridgePhase,
    pub ticks_in_warmup: u32,
    pub ticks_in_filling: u32,
}

impl Default for BridgePhaseState {
    fn default() -> Self {
        Self {
            phase: BridgePhase::Warmup,
            ticks_in_warmup: 0,
            ticks_in_filling: 0,
        }
    }
}

impl BridgePhaseState {
    /// Saltar Warmup/Filling (tests y escenarios que necesitan hits inmediatos).
    pub fn active_only() -> Self {
        Self {
            phase: BridgePhase::Active,
            ticks_in_warmup: 0,
            ticks_in_filling: 0,
        }
    }
}

#[derive(Event, Clone, Copy, Debug, PartialEq, Eq)]
pub struct BridgePhaseChanged {
    pub from: BridgePhase,
    pub to: BridgePhase,
}

// --- Fill ratio agregado ----------------------------------------------------------------------

/// Máximo de `len/capacity` entre todas las caches de puente registradas en el `World`.
pub fn bridge_caches_max_fill_ratio(world: &World) -> f32 {
    let mut max_r = 0.0_f32;
    macro_rules! scan {
        ($($B:ty),* $(,)?) => {$(
            if let Some(c) = world.get_resource::<BridgeCache<$B>>() {
                let s = c.stats();
                if s.capacity > 0 {
                    max_r = max_r.max(s.len as f32 / s.capacity as f32);
                }
            }
        )*};
    }
    scan!(
        DensityBridge,
        TemperatureBridge,
        PhaseTransitionBridge,
        DissipationBridge,
        DragBridge,
        EngineBridge,
        WillBridge,
        InterferenceBridge,
        CatalysisBridge,
        CollisionTransferBridge,
        OsmosisBridge,
    );
    max_r
}

// --- Efectos por fase -------------------------------------------------------------------------

/// Aplica flags a **todos** los `BridgeConfig<B>` y `BridgeCache<B>` conocidos.
pub fn apply_bridge_phase_side_effects(world: &mut World, phase: BridgePhase) {
    let (configs_enabled, eviction) = match phase {
        BridgePhase::Warmup => (false, true),
        BridgePhase::Filling => (true, false),
        BridgePhase::Active => (true, true),
    };

    macro_rules! each {
        ($($B:ty),* $(,)?) => {$(
            if let Some(mut cfg) = world.get_resource_mut::<BridgeConfig<$B>>() {
                cfg.enabled = configs_enabled;
            }
            if let Some(mut cache) = world.get_resource_mut::<BridgeCache<$B>>() {
                cache.set_eviction_enabled(eviction);
            }
        )*};
    }
    each!(
        DensityBridge,
        TemperatureBridge,
        PhaseTransitionBridge,
        DissipationBridge,
        DragBridge,
        EngineBridge,
        WillBridge,
        InterferenceBridge,
        CatalysisBridge,
        CollisionTransferBridge,
        OsmosisBridge,
    );
}

pub fn clear_all_bridge_caches(world: &mut World) {
    macro_rules! each {
        ($($B:ty),* $(,)?) => {$(
            if let Some(mut c) = world.get_resource_mut::<BridgeCache<$B>>() {
                c.clear();
            }
        )*};
    }
    each!(
        DensityBridge,
        TemperatureBridge,
        PhaseTransitionBridge,
        DissipationBridge,
        DragBridge,
        EngineBridge,
        WillBridge,
        InterferenceBridge,
        CatalysisBridge,
        CollisionTransferBridge,
        OsmosisBridge,
    );
}

fn send_phase_changed(world: &mut World, from: BridgePhase, to: BridgePhase) {
    if from == to {
        return;
    }
    if let Some(mut events) = world.get_resource_mut::<Events<BridgePhaseChanged>>() {
        events.send(BridgePhaseChanged { from, to });
    }
}

/// Transición interna: actualiza `BridgePhaseState`, efectos y evento.
pub fn bridge_phase_apply_transition(world: &mut World, to: BridgePhase) {
    let from = world
        .get_resource::<BridgePhaseState>()
        .map(|s| s.phase)
        .unwrap_or(to);
    if from == to {
        apply_bridge_phase_side_effects(world, to);
        return;
    }
    if let Some(mut state) = world.get_resource_mut::<BridgePhaseState>() {
        state.phase = to;
        if to == BridgePhase::Filling {
            state.ticks_in_filling = 0;
        }
    }
    apply_bridge_phase_side_effects(world, to);
    send_phase_changed(world, from, to);
}

/// Fuerza fase (debug / tests). Resetea contadores para no disparar transiciones inmediatas.
pub fn bridge_phase_force(world: &mut World, to: BridgePhase) {
    let from = world
        .get_resource::<BridgePhaseState>()
        .map(|s| s.phase)
        .unwrap_or(to);
    if let Some(mut state) = world.get_resource_mut::<BridgePhaseState>() {
        state.phase = to;
        state.ticks_in_warmup = 0;
        state.ticks_in_filling = 0;
    }
    apply_bridge_phase_side_effects(world, to);
    send_phase_changed(world, from, to);
}

/// Vuelve a Warmup, limpia caches y sincroniza flags.
pub fn bridge_phase_reset(world: &mut World) {
    let from = world
        .get_resource::<BridgePhaseState>()
        .map(|s| s.phase)
        .unwrap_or(BridgePhase::Warmup);
    clear_all_bridge_caches(world);
    if let Some(mut state) = world.get_resource_mut::<BridgePhaseState>() {
        state.phase = BridgePhase::Warmup;
        state.ticks_in_warmup = 0;
        state.ticks_in_filling = 0;
    }
    apply_bridge_phase_side_effects(world, BridgePhase::Warmup);
    send_phase_changed(world, from, BridgePhase::Warmup);
}

/// Tick exclusivo: evalúa transiciones por contador y fill ratio.
pub fn bridge_phase_tick(world: &mut World) {
    if !world.contains_resource::<BridgePhaseState>()
        || !world.contains_resource::<BridgePhaseConfig>()
    {
        return;
    }

    let cfg = world.resource::<BridgePhaseConfig>().clone();
    let next = match world.resource::<BridgePhaseState>().phase {
        BridgePhase::Warmup => {
            let mut state = world.resource_mut::<BridgePhaseState>();
            state.ticks_in_warmup = state.ticks_in_warmup.saturating_add(1);
            if state.ticks_in_warmup >= cfg.warmup_ticks {
                Some(BridgePhase::Filling)
            } else {
                None
            }
        }
        BridgePhase::Filling => {
            let tf = {
                let mut state = world.resource_mut::<BridgePhaseState>();
                state.ticks_in_filling = state.ticks_in_filling.saturating_add(1);
                state.ticks_in_filling
            };
            let fill = bridge_caches_max_fill_ratio(world);
            if fill >= cfg.fill_ratio_for_active || tf >= cfg.filling_max_ticks {
                Some(BridgePhase::Active)
            } else {
                None
            }
        }
        BridgePhase::Active => None,
    };

    if let Some(to) = next {
        bridge_phase_apply_transition(world, to);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bridge::config::BridgeKind;
    use crate::bridge::config::{CachePolicy, Rigidity};
    use crate::bridge::decorator::bridge_warmup_record;

    #[derive(Clone, Copy, Debug)]
    struct TinyBridge;
    impl BridgeKind for TinyBridge {}

    // Reutilizamos el patrón DoubleBridge de decorator tests — duplicado mínimo para World.
    use crate::bridge::config::BandDef;
    use crate::bridge::decorator::Bridgeable;
    use crate::bridge::decorator::hash_inputs;
    use crate::bridge::normalize_scalar;

    impl Bridgeable for TinyBridge {
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

        fn compute(x: Self::Input) -> Self::Output {
            x * 2.0
        }

        fn into_cached(value: Self::Output) -> crate::bridge::cache::CachedValue {
            crate::bridge::cache::CachedValue::Scalar(value)
        }

        fn from_cached(value: crate::bridge::cache::CachedValue) -> Option<Self::Output> {
            match value {
                crate::bridge::cache::CachedValue::Scalar(s) => Some(s),
                _ => None,
            }
        }
    }

    fn tiny_bands() -> Vec<BandDef> {
        vec![
            BandDef {
                min: 0.0,
                max: 1.0,
                canonical: 0.5,
                stable: true,
            },
            BandDef {
                min: 1.0,
                max: 2.0,
                canonical: 1.5,
                stable: true,
            },
        ]
    }

    #[test]
    fn initial_phase_is_warmup() {
        let s = BridgePhaseState::default();
        assert_eq!(s.phase, BridgePhase::Warmup);
    }

    #[test]
    fn warmup_to_filling_after_n_ticks() {
        let mut world = World::new();
        world.insert_resource(BridgePhaseConfig {
            warmup_ticks: 3,
            filling_max_ticks: 99,
            fill_ratio_for_active: 0.99,
        });
        world.insert_resource(BridgePhaseState::default());
        world.init_resource::<Events<BridgePhaseChanged>>();

        bridge_phase_tick(&mut world);
        bridge_phase_tick(&mut world);
        assert_eq!(
            world.resource::<BridgePhaseState>().phase,
            BridgePhase::Warmup
        );
        bridge_phase_tick(&mut world);
        assert_eq!(
            world.resource::<BridgePhaseState>().phase,
            BridgePhase::Filling
        );
    }

    #[test]
    fn filling_to_active_on_fill_ratio() {
        let mut world = World::new();
        world.insert_resource(BridgePhaseConfig {
            warmup_ticks: 0,
            filling_max_ticks: 99,
            fill_ratio_for_active: 0.51,
        });
        world.insert_resource(BridgePhaseState {
            phase: BridgePhase::Filling,
            ticks_in_warmup: 0,
            ticks_in_filling: 0,
        });
        world.insert_resource(
            BridgeConfig::<DensityBridge>::new(
                tiny_bands(),
                0.25,
                2,
                CachePolicy::Lru,
                true,
                Rigidity::Moderate,
            )
            .expect("bands"),
        );
        world.insert_resource(BridgeCache::<DensityBridge>::new(2, CachePolicy::Lru));
        world.init_resource::<Events<BridgePhaseChanged>>();

        bridge_phase_tick(&mut world);
        assert_eq!(
            world.resource::<BridgePhaseState>().phase,
            BridgePhase::Filling
        );
        world
            .resource_mut::<BridgeCache<DensityBridge>>()
            .insert(1, crate::bridge::cache::CachedValue::Scalar(1.0));
        bridge_phase_tick(&mut world);
        assert_eq!(
            world.resource::<BridgePhaseState>().phase,
            BridgePhase::Filling
        );
        world
            .resource_mut::<BridgeCache<DensityBridge>>()
            .insert(2, crate::bridge::cache::CachedValue::Scalar(2.0));
        bridge_phase_tick(&mut world);
        assert_eq!(
            world.resource::<BridgePhaseState>().phase,
            BridgePhase::Active
        );
    }

    #[test]
    fn filling_to_active_after_m_ticks_even_if_low_fill() {
        let mut world = World::new();
        world.insert_resource(BridgePhaseConfig {
            warmup_ticks: 0,
            filling_max_ticks: 2,
            fill_ratio_for_active: 0.99,
        });
        world.insert_resource(BridgePhaseState {
            phase: BridgePhase::Filling,
            ticks_in_warmup: 0,
            ticks_in_filling: 0,
        });
        world.insert_resource(
            BridgeConfig::<DensityBridge>::new(
                tiny_bands(),
                0.25,
                100,
                CachePolicy::Lru,
                true,
                Rigidity::Moderate,
            )
            .expect("bands"),
        );
        world.insert_resource(BridgeCache::<DensityBridge>::new(100, CachePolicy::Lru));
        world.init_resource::<Events<BridgePhaseChanged>>();

        bridge_phase_tick(&mut world);
        assert_eq!(
            world.resource::<BridgePhaseState>().phase,
            BridgePhase::Filling
        );
        bridge_phase_tick(&mut world);
        assert_eq!(
            world.resource::<BridgePhaseState>().phase,
            BridgePhase::Active
        );
    }

    #[test]
    fn warmup_exact_and_cache_nonzero() {
        let cfg = BridgeConfig::<TinyBridge>::new(
            tiny_bands(),
            0.25,
            16,
            CachePolicy::Lru,
            true,
            Rigidity::Moderate,
        )
        .expect("bands");
        let mut cache = BridgeCache::<TinyBridge>::new(16, CachePolicy::Lru);
        let out = bridge_warmup_record(0.3_f32, &cfg, &mut cache);
        // Exacto sobre crudo: 0.3 * 2; la clave de cache usa canónico 0.5 → 1.0 en fase activa.
        assert!((out - 0.6).abs() < 1e-5, "out={out}");
        assert!(cache.stats().len > 0);
    }

    #[test]
    fn force_phase_active_jumps() {
        let mut world = World::new();
        world.insert_resource(BridgePhaseConfig::default());
        world.insert_resource(BridgePhaseState::default());
        world.insert_resource(
            BridgeConfig::<DensityBridge>::new(
                tiny_bands(),
                0.25,
                8,
                CachePolicy::Lru,
                false,
                Rigidity::Moderate,
            )
            .expect("bands"),
        );
        world.insert_resource(BridgeCache::<DensityBridge>::new(8, CachePolicy::Lru));
        world.init_resource::<Events<BridgePhaseChanged>>();

        bridge_phase_force(&mut world, BridgePhase::Active);
        assert_eq!(
            world.resource::<BridgePhaseState>().phase,
            BridgePhase::Active
        );
        assert!(world.resource::<BridgeConfig<DensityBridge>>().enabled);
    }

    #[test]
    fn reset_clears_cache_and_warmup() {
        let mut world = World::new();
        world.insert_resource(BridgePhaseConfig::default());
        world.insert_resource(BridgePhaseState {
            phase: BridgePhase::Active,
            ticks_in_warmup: 5,
            ticks_in_filling: 2,
        });
        world.insert_resource(
            BridgeConfig::<DensityBridge>::new(
                tiny_bands(),
                0.25,
                8,
                CachePolicy::Lru,
                true,
                Rigidity::Moderate,
            )
            .expect("bands"),
        );
        world.insert_resource(BridgeCache::<DensityBridge>::new(8, CachePolicy::Lru));
        world
            .resource_mut::<BridgeCache<DensityBridge>>()
            .insert(9, crate::bridge::cache::CachedValue::Scalar(1.0));
        world.init_resource::<Events<BridgePhaseChanged>>();

        bridge_phase_reset(&mut world);
        assert_eq!(
            world.resource::<BridgePhaseState>().phase,
            BridgePhase::Warmup
        );
        assert_eq!(
            world.resource::<BridgeCache<DensityBridge>>().stats().len,
            0
        );
    }

    #[test]
    fn filling_eviction_disabled_active_reenabled() {
        let mut world = World::new();
        world.insert_resource(BridgePhaseConfig::default());
        world.insert_resource(BridgePhaseState::default());
        world.insert_resource(
            BridgeConfig::<DensityBridge>::new(
                tiny_bands(),
                0.25,
                2,
                CachePolicy::Lru,
                true,
                Rigidity::Moderate,
            )
            .expect("bands"),
        );
        world.insert_resource(BridgeCache::<DensityBridge>::new(2, CachePolicy::Lru));

        bridge_phase_apply_transition(&mut world, BridgePhase::Filling);
        assert!(
            !world
                .resource::<BridgeCache<DensityBridge>>()
                .eviction_enabled()
        );

        bridge_phase_apply_transition(&mut world, BridgePhase::Active);
        assert!(
            world
                .resource::<BridgeCache<DensityBridge>>()
                .eviction_enabled()
        );
    }

    #[test]
    fn phase_changed_emitted_on_transitions() {
        use bevy::MinimalPlugins;

        #[derive(Resource, Default)]
        struct Collected(Vec<BridgePhaseChanged>);

        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .add_event::<BridgePhaseChanged>()
            .insert_resource(BridgePhaseConfig {
                warmup_ticks: 1,
                filling_max_ticks: 1,
                fill_ratio_for_active: 1.0,
            })
            .insert_resource(BridgePhaseState::default())
            .insert_resource(
                BridgeConfig::<DensityBridge>::new(
                    tiny_bands(),
                    0.25,
                    4,
                    CachePolicy::Lru,
                    true,
                    Rigidity::Moderate,
                )
                .expect("bands"),
            )
            .insert_resource(BridgeCache::<DensityBridge>::new(4, CachePolicy::Lru))
            .insert_resource(Collected::default())
            .add_systems(Update, |world: &mut World| bridge_phase_tick(world))
            .add_systems(
                PostUpdate,
                |mut r: EventReader<BridgePhaseChanged>, mut c: ResMut<Collected>| {
                    for e in r.read() {
                        c.0.push(*e);
                    }
                },
            );

        app.update();
        app.update();
        let collected = std::mem::take(&mut app.world_mut().resource_mut::<Collected>().0);
        assert!(collected.iter().any(|e| e.to == BridgePhase::Filling));
        assert!(collected.iter().any(|e| e.to == BridgePhase::Active));
    }
}
