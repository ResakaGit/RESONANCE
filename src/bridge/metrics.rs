//! Métricas del bridge optimizer — ventana por periodo (B9). Sin hot path extra.
//!
//! Lee `BridgeCache::stats()`, copia a `BridgeMetrics<B>`, reinicia contadores de uso en la cache.

use core::marker::PhantomData;

use bevy::prelude::*;

use crate::bridge::cache::BridgeCache;
use crate::bridge::config::{
    BridgeKind, CatalysisBridge, CollisionTransferBridge, DensityBridge, DissipationBridge,
    DragBridge, EngineBridge, InterferenceBridge, OsmosisBridge, PhaseTransitionBridge,
    TemperatureBridge, WillBridge,
};
use crate::bridge::context_fill::{BridgePhase, BridgePhaseState};
use crate::runtime_platform::simulation_tick::SimulationClock;

/// Intervalo de recolección en ticks de simulación (`SimulationClock::tick_id`).
#[derive(Resource, Debug, Clone, Copy, PartialEq, Eq)]
pub struct BridgeMetricsConfig {
    pub collect_interval_ticks: u32,
    /// Evicciones en la ventana para disparar hint de capacidad si además fill ≈ 100%.
    pub eviction_spike_threshold: u64,
}

impl Default for BridgeMetricsConfig {
    fn default() -> Self {
        Self {
            collect_interval_ticks: 60,
            eviction_spike_threshold: 5,
        }
    }
}

/// Captura inmutable para logging / serialización.
#[derive(Clone, Debug, PartialEq)]
pub struct BridgeMetricsSnapshot {
    pub layer_name: &'static str,
    pub window_hits: u64,
    pub window_misses: u64,
    pub window_evictions: u64,
    pub fill_len: usize,
    pub fill_capacity: usize,
    pub computations_saved: u64,
    pub total_lookups: u64,
}

/// Fila agregada para HUD / resumen.
#[derive(Clone, Debug, PartialEq)]
pub struct BridgeLayerRow {
    pub name: &'static str,
    pub hit_rate: f32,
    pub fill_level: f32,
    pub hits: u64,
    pub misses: u64,
    pub evictions: u64,
    pub recommendations: Vec<String>,
}

/// Agregación de todas las capas de puente registradas.
#[derive(Resource, Debug, Clone, Default)]
pub struct BridgeMetricsSummary {
    pub layers: Vec<BridgeLayerRow>,
    pub phase_label: &'static str,
}

/// Métricas de la última ventana recolectada para un tipo de puente.
#[derive(Resource, Debug, Clone)]
pub struct BridgeMetrics<B: BridgeKind> {
    pub layer_name: &'static str,
    pub window_hits: u64,
    pub window_misses: u64,
    pub window_evictions: u64,
    pub fill_len: usize,
    pub fill_capacity: usize,
    pub computations_saved: u64,
    pub total_lookups: u64,
    _marker: PhantomData<B>,
}

impl<B: BridgeKind> BridgeMetrics<B> {
    pub fn new(layer_name: &'static str) -> Self {
        Self {
            layer_name,
            window_hits: 0,
            window_misses: 0,
            window_evictions: 0,
            fill_len: 0,
            fill_capacity: 0,
            computations_saved: 0,
            total_lookups: 0,
            _marker: PhantomData,
        }
    }

    pub fn hit_rate(&self) -> f32 {
        let d = self.window_hits.saturating_add(self.window_misses);
        if d == 0 {
            0.0
        } else {
            self.window_hits as f32 / d as f32
        }
    }

    pub fn fill_level(&self) -> f32 {
        if self.fill_capacity == 0 {
            0.0
        } else {
            self.fill_len as f32 / self.fill_capacity as f32
        }
    }

    pub fn snapshot(&self) -> BridgeMetricsSnapshot {
        BridgeMetricsSnapshot {
            layer_name: self.layer_name,
            window_hits: self.window_hits,
            window_misses: self.window_misses,
            window_evictions: self.window_evictions,
            fill_len: self.fill_len,
            fill_capacity: self.fill_capacity,
            computations_saved: self.computations_saved,
            total_lookups: self.total_lookups,
        }
    }
}

/// Copia stats del periodo desde la cache al resource de métricas y reinicia contadores de uso.
pub fn absorb_cache_stats_into_metrics<B: BridgeKind>(
    cache: &mut BridgeCache<B>,
    metrics: &mut BridgeMetrics<B>,
) {
    let s = cache.stats();
    metrics.window_hits = s.hits;
    metrics.window_misses = s.misses;
    metrics.window_evictions = s.evictions;
    metrics.fill_len = s.len;
    metrics.fill_capacity = s.capacity;
    metrics.computations_saved = s.hits;
    metrics.total_lookups = s.hits.saturating_add(s.misses);
    cache.reset_usage_counters();
}

fn phase_label(world: &World) -> &'static str {
    world
        .get_resource::<BridgePhaseState>()
        .map(|s| match s.phase {
            BridgePhase::Warmup => "Warmup",
            BridgePhase::Filling => "Filling",
            BridgePhase::Active => "Active",
        })
        .unwrap_or("n/a")
}

fn row_recommendations(
    hit_rate: f32,
    fill_level: f32,
    evictions: u64,
    threshold: u64,
) -> Vec<String> {
    let mut r = Vec::new();
    if fill_level >= 1.0 - f32::EPSILON && evictions >= threshold {
        r.push("consider increasing cache_capacity".to_string());
    }
    if hit_rate < 0.5 {
        r.push("consider wider bands or more cache".to_string());
    }
    r
}

/// Reporte multilínea para consola al entrar en Active.
pub fn bridge_cache_fill_report(world: &World) -> String {
    let mut lines = Vec::new();
    macro_rules! line {
        ($($B:ty),* $(,)?) => {$(
            if let Some(c) = world.get_resource::<BridgeCache<$B>>() {
                let s = c.stats();
                lines.push(format!(
                    "{}: len={}/{} hits={} misses={}",
                    bridge_layer_name::<$B>(),
                    s.len,
                    s.capacity,
                    s.hits,
                    s.misses
                ));
            }
        )*};
    }
    line!(
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
    format!(
        "Bridge optimizer → Active — snapshot caches:\n{}",
        lines.join("\n")
    )
}

pub fn bridge_layer_name<B: BridgeKind>() -> &'static str {
    // Nombres estables para overlay / logs (coinciden con claves de preset donde aplica).
    macro_rules! n {
        ($($T:ty => $s:literal),* $(,)?) => {
            $(if core::any::TypeId::of::<B>() == core::any::TypeId::of::<$T>() {
                return $s;
            })*
        };
    }
    n!(
        DensityBridge => "density",
        TemperatureBridge => "temperature",
        PhaseTransitionBridge => "phase_transition",
        DissipationBridge => "dissipation",
        DragBridge => "drag",
        EngineBridge => "engine",
        WillBridge => "will",
        InterferenceBridge => "interference",
        CatalysisBridge => "catalysis",
        CollisionTransferBridge => "collision_transfer",
        OsmosisBridge => "osmosis",
    );
    "unknown_bridge"
}

fn collect_one_bridge<B: BridgeKind>(world: &mut World) {
    if !world.contains_resource::<BridgeCache<B>>()
        || !world.contains_resource::<BridgeMetrics<B>>()
    {
        return;
    }
    let (hits, misses, evictions, len, cap) = {
        let cache = world.resource::<BridgeCache<B>>();
        let s = cache.stats();
        (s.hits, s.misses, s.evictions, s.len, s.capacity)
    };
    {
        let mut cache = world.resource_mut::<BridgeCache<B>>();
        cache.reset_usage_counters();
    }
    let mut metrics = world.resource_mut::<BridgeMetrics<B>>();
    metrics.window_hits = hits;
    metrics.window_misses = misses;
    metrics.window_evictions = evictions;
    metrics.fill_len = len;
    metrics.fill_capacity = cap;
    metrics.computations_saved = hits;
    metrics.total_lookups = hits.saturating_add(misses);
}

/// Recolecta todas las caches tipadas conocidas (solo si existen ambos resources).
pub fn bridge_metrics_collect_all(world: &mut World) {
    collect_one_bridge::<DensityBridge>(world);
    collect_one_bridge::<TemperatureBridge>(world);
    collect_one_bridge::<PhaseTransitionBridge>(world);
    collect_one_bridge::<DissipationBridge>(world);
    collect_one_bridge::<DragBridge>(world);
    collect_one_bridge::<EngineBridge>(world);
    collect_one_bridge::<WillBridge>(world);
    collect_one_bridge::<InterferenceBridge>(world);
    collect_one_bridge::<CatalysisBridge>(world);
    collect_one_bridge::<CollisionTransferBridge>(world);
    collect_one_bridge::<OsmosisBridge>(world);
}

fn bridge_metrics_layer_rows(world: &World, cfg: &BridgeMetricsConfig) -> Vec<BridgeLayerRow> {
    let mut layers = Vec::new();
    macro_rules! push_row {
        ($($B:ty),* $(,)?) => {$(
            if let Some(m) = world.get_resource::<BridgeMetrics<$B>>() {
                let hr = m.hit_rate();
                let fl = m.fill_level();
                let rec = row_recommendations(hr, fl, m.window_evictions, cfg.eviction_spike_threshold);
                layers.push(BridgeLayerRow {
                    name: m.layer_name,
                    hit_rate: hr,
                    fill_level: fl,
                    hits: m.window_hits,
                    misses: m.window_misses,
                    evictions: m.window_evictions,
                    recommendations: rec,
                });
            }
        )*};
    }
    push_row!(
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
    layers
}

pub fn rebuild_bridge_metrics_summary(world: &mut World, cfg: &BridgeMetricsConfig) {
    let phase = phase_label(world);
    let layers = bridge_metrics_layer_rows(world, cfg);
    let Some(mut summary) = world.get_resource_mut::<BridgeMetricsSummary>() else {
        return;
    };
    summary.phase_label = phase;
    summary.layers = layers;
}

impl BridgeMetricsSummary {
    pub fn efficiency_report(&self) -> String {
        let mut s = String::from("Bridge optimizer — efficiency report\n");
        s.push_str(&format!("Phase: {}\n", self.phase_label));
        for row in &self.layers {
            s.push_str(&format!(
                "  {:<20} hit={:.1}% fill={:.1}% h={} m={} ev={}\n",
                row.name,
                row.hit_rate * 100.0,
                row.fill_level * 100.0,
                row.hits,
                row.misses,
                row.evictions
            ));
            for hint in &row.recommendations {
                s.push_str(&format!("    → {hint}\n"));
            }
        }
        s
    }
}

/// Hit rate con color UI aproximado (texto): G/Y/R.
pub fn hit_rate_quality_prefix(hit_rate: f32) -> &'static str {
    if hit_rate >= 0.8 {
        "[G] "
    } else if hit_rate >= 0.5 {
        "[Y] "
    } else {
        "[R] "
    }
}

/// Sistema exclusivo: corre solo cada `collect_interval_ticks` (post-frame simulación).
pub fn bridge_metrics_collect_system(world: &mut World) {
    let cfg = world
        .get_resource::<BridgeMetricsConfig>()
        .copied()
        .unwrap_or_default();
    if cfg.collect_interval_ticks == 0 {
        return;
    }
    let tick_id = world
        .get_resource::<SimulationClock>()
        .map(|c| c.tick_id)
        .unwrap_or(0);
    if tick_id == 0 {
        return;
    }
    if tick_id % cfg.collect_interval_ticks as u64 != 0 {
        return;
    }
    bridge_metrics_collect_all(world);
    rebuild_bridge_metrics_summary(world, &cfg);
}

/// Estado mínimo para detectar transición a Active sin `Local` (sistemas exclusivos solo `World`).
#[derive(Resource, Debug, Clone, Copy, Default)]
pub struct BridgeOptimizerPhaseLogState {
    pub last: Option<BridgePhase>,
}

/// Log `info!` al entrar en fase Active (desde Filling o Warmup).
pub fn bridge_optimizer_enter_active_log_system(world: &mut World) {
    let Some(st) = world.get_resource::<BridgePhaseState>().map(|s| s.phase) else {
        return;
    };
    let mut log_state = world.resource_mut::<BridgeOptimizerPhaseLogState>();
    let prev = log_state.last;
    log_state.last = Some(st);

    if st != BridgePhase::Active {
        return;
    }
    if prev.is_none() || prev == Some(BridgePhase::Active) {
        return;
    }

    info!(target: "bridge::optimizer", "{}", bridge_cache_fill_report(world));
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bridge::config::DensityBridge;

    #[derive(Debug)]
    struct OtherBridge;
    impl BridgeKind for OtherBridge {}

    #[test]
    fn hit_rate_after_80_hits_20_misses_window() {
        let mut cache =
            BridgeCache::<DensityBridge>::new(256, crate::bridge::config::CachePolicy::Lru);
        for i in 0..80 {
            cache.insert(i, crate::bridge::cache::CachedValue::Scalar(i as f32));
        }
        for i in 0..80 {
            let _ = cache.lookup(i);
        }
        for _ in 0..20 {
            let _ = cache.lookup(9999);
        }
        let mut m = BridgeMetrics::<DensityBridge>::new("density");
        absorb_cache_stats_into_metrics(&mut cache, &mut m);
        assert!((m.hit_rate() - 0.8).abs() < 1e-5);
        assert_eq!(m.total_lookups, 100);
        assert_eq!(m.computations_saved, 80);
        // Tras absorber, contadores cache en cero; nuevas lecturas solo ventana nueva
        let _ = cache.lookup(0);
        let _ = cache.lookup(1);
        let mut m2 = BridgeMetrics::new("density");
        absorb_cache_stats_into_metrics(&mut cache, &mut m2);
        assert!((m2.hit_rate() - 1.0).abs() < 1e-5);
    }

    #[test]
    fn fill_level_matches_occupancy() {
        let mut cache =
            BridgeCache::<DensityBridge>::new(10, crate::bridge::config::CachePolicy::Lru);
        for i in 0..3 {
            cache.insert(i, crate::bridge::cache::CachedValue::Scalar(1.0));
        }
        let mut m = BridgeMetrics::<DensityBridge>::new("density");
        absorb_cache_stats_into_metrics(&mut cache, &mut m);
        assert!((m.fill_level() - 0.3).abs() < 1e-5);
    }

    #[test]
    fn efficiency_report_readable() {
        let mut world = World::new();
        world.insert_resource(BridgeMetricsSummary::default());
        world.insert_resource(BridgeMetrics::<DensityBridge>::new("density"));
        world.insert_resource(BridgeMetrics::<OtherBridge>::new("other"));
        world.insert_resource(BridgeCache::<DensityBridge>::new(
            4,
            crate::bridge::config::CachePolicy::Lru,
        ));
        world.insert_resource(BridgeCache::<OtherBridge>::new(
            4,
            crate::bridge::config::CachePolicy::Lru,
        ));
        world.insert_resource(BridgeMetricsConfig::default());

        {
            let mut c = world.resource_mut::<BridgeCache<DensityBridge>>();
            c.insert(0, crate::bridge::cache::CachedValue::Scalar(1.0));
            let _ = c.lookup(0);
            let _ = c.lookup(1);
        }
        bridge_metrics_collect_all(&mut world);
        rebuild_bridge_metrics_summary(&mut world, &BridgeMetricsConfig::default());
        let rep = world.resource::<BridgeMetricsSummary>().efficiency_report();
        assert!(rep.contains("density"));
        assert!(rep.contains("hit="));
    }

    #[test]
    fn sliding_window_after_reset_counters() {
        let mut cache =
            BridgeCache::<DensityBridge>::new(8, crate::bridge::config::CachePolicy::Lru);
        cache.insert(1, crate::bridge::cache::CachedValue::Scalar(1.0));
        let _ = cache.lookup(1);
        let _ = cache.lookup(2);
        let mut m = BridgeMetrics::<DensityBridge>::new("density");
        absorb_cache_stats_into_metrics(&mut cache, &mut m);
        assert!((m.hit_rate() - 0.5).abs() < 1e-5);
        let _ = cache.lookup(1);
        let _ = cache.lookup(1);
        absorb_cache_stats_into_metrics(&mut cache, &mut m);
        assert!((m.hit_rate() - 1.0).abs() < 1e-5);
    }

    #[test]
    fn collect_interval_skips_intermediate_ticks() {
        let mut world = World::new();
        world.insert_resource(SimulationClock { tick_id: 59 });
        world.insert_resource(BridgeMetricsConfig {
            collect_interval_ticks: 60,
            eviction_spike_threshold: 5,
        });
        world.insert_resource(BridgeMetrics::<DensityBridge>::new("density"));
        let mut c = BridgeCache::<DensityBridge>::new(8, crate::bridge::config::CachePolicy::Lru);
        c.insert(0, crate::bridge::cache::CachedValue::Scalar(1.0));
        let _ = c.lookup(0);
        world.insert_resource(c);
        world.insert_resource(BridgeMetricsSummary::default());

        bridge_metrics_collect_system(&mut world);
        assert_eq!(
            world.resource::<BridgeMetrics<DensityBridge>>().window_hits,
            0
        );

        world.resource_mut::<SimulationClock>().tick_id = 60;
        bridge_metrics_collect_system(&mut world);
        assert_eq!(
            world.resource::<BridgeMetrics<DensityBridge>>().window_hits,
            1
        );
    }

    #[test]
    fn metrics_isolation_between_bridge_types() {
        let mut cd = BridgeCache::<DensityBridge>::new(8, crate::bridge::config::CachePolicy::Lru);
        let mut co = BridgeCache::<OtherBridge>::new(8, crate::bridge::config::CachePolicy::Lru);
        cd.insert(1, crate::bridge::cache::CachedValue::Scalar(1.0));
        let _ = cd.lookup(1);
        co.insert(1, crate::bridge::cache::CachedValue::Scalar(2.0));
        let _ = co.lookup(9);

        let mut md = BridgeMetrics::<DensityBridge>::new("density");
        let mut mo = BridgeMetrics::<OtherBridge>::new("other");
        absorb_cache_stats_into_metrics(&mut cd, &mut md);
        absorb_cache_stats_into_metrics(&mut co, &mut mo);
        assert_eq!(md.window_hits, 1);
        assert_eq!(md.window_misses, 0);
        assert_eq!(mo.window_hits, 0);
        assert_eq!(mo.window_misses, 1);
    }

    #[test]
    fn summary_aggregates_all_layers() {
        let mut world = World::new();
        world.insert_resource(BridgeMetricsSummary::default());
        world.insert_resource(BridgeMetrics::<DensityBridge>::new("density"));
        world.insert_resource(BridgeMetrics::<OtherBridge>::new("other"));
        world.insert_resource(BridgeCache::<DensityBridge>::new(
            8,
            crate::bridge::config::CachePolicy::Lru,
        ));
        world.insert_resource(BridgeCache::<OtherBridge>::new(
            8,
            crate::bridge::config::CachePolicy::Lru,
        ));
        world.insert_resource(BridgePhaseState::default());
        world.insert_resource(BridgeMetricsConfig::default());

        bridge_metrics_collect_all(&mut world);
        rebuild_bridge_metrics_summary(&mut world, &BridgeMetricsConfig::default());
        let s = world.resource::<BridgeMetricsSummary>();
        assert_eq!(s.layers.len(), 1);
        assert_eq!(s.layers[0].name, "density");
    }

    #[test]
    fn summary_lists_two_production_bridges() {
        use crate::bridge::config::TemperatureBridge;

        let mut world = World::new();
        world.insert_resource(BridgeMetricsSummary::default());
        world.insert_resource(BridgeMetrics::<DensityBridge>::new("density"));
        world.insert_resource(BridgeMetrics::<TemperatureBridge>::new("temperature"));
        world.insert_resource(BridgeCache::<DensityBridge>::new(
            8,
            crate::bridge::config::CachePolicy::Lru,
        ));
        world.insert_resource(BridgeCache::<TemperatureBridge>::new(
            8,
            crate::bridge::config::CachePolicy::Lru,
        ));
        world.insert_resource(BridgePhaseState::default());
        world.insert_resource(BridgeMetricsConfig::default());

        bridge_metrics_collect_all(&mut world);
        rebuild_bridge_metrics_summary(&mut world, &BridgeMetricsConfig::default());
        let rep = world.resource::<BridgeMetricsSummary>().efficiency_report();
        assert!(rep.contains("density"));
        assert!(rep.contains("temperature"));
        assert_eq!(world.resource::<BridgeMetricsSummary>().layers.len(), 2);
    }

    #[test]
    fn efficiency_report_includes_recommendation_lines() {
        let mut summary = BridgeMetricsSummary::default();
        summary.layers.push(BridgeLayerRow {
            name: "density",
            hit_rate: 0.4,
            fill_level: 1.0,
            hits: 4,
            misses: 6,
            evictions: 10,
            recommendations: vec![
                "consider increasing cache_capacity".to_string(),
                "consider wider bands or more cache".to_string(),
            ],
        });
        let rep = summary.efficiency_report();
        assert!(rep.contains("wider"));
        assert!(rep.contains("cache_capacity"));
    }

    #[test]
    fn recommendation_full_cache_high_evictions() {
        let r = row_recommendations(0.9, 1.0, 10, 5);
        assert!(r.iter().any(|s| s.contains("cache_capacity")));
    }

    #[test]
    fn recommendation_low_hit_rate() {
        let r = row_recommendations(0.4, 0.5, 0, 5);
        assert!(r.iter().any(|s| s.contains("wider")));
    }

    #[test]
    fn bridge_layer_name_density() {
        assert_eq!(bridge_layer_name::<DensityBridge>(), "density");
    }

    #[test]
    fn summary_includes_osmosis_layer() {
        use crate::bridge::config::OsmosisBridge;

        let mut world = World::new();
        world.insert_resource(BridgeMetricsSummary::default());
        world.insert_resource(BridgeMetrics::<OsmosisBridge>::new("osmosis"));
        world.insert_resource(BridgeCache::<OsmosisBridge>::new(
            8,
            crate::bridge::config::CachePolicy::Lru,
        ));
        world.insert_resource(BridgePhaseState::default());
        world.insert_resource(BridgeMetricsConfig::default());

        bridge_metrics_collect_all(&mut world);
        rebuild_bridge_metrics_summary(&mut world, &BridgeMetricsConfig::default());
        let summary = world.resource::<BridgeMetricsSummary>();
        assert!(summary.layers.iter().any(|l| l.name == "osmosis"));
    }
}
