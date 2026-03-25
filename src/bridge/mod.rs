//! Bridge Optimizer — cuantización estable + cache por capa (ver sprints `docs/sprints/BRIDGE_OPTIMIZER/`).

pub mod constants;

#[cfg(feature = "bridge_optimizer")]
pub mod benchmark_harness;
pub mod impls;
pub mod cache;
pub mod config;
pub mod context_fill;
pub mod decorator;
pub mod metrics;
pub mod normalize;
pub mod presets;
pub mod macros;

pub use impls::ops::{
    BridgedInterferenceOps, CatalysisEquationInput, CollisionTransferEquationInput,
    CollisionTransferScratch, CompetitionNormEquationInput, InterferenceEquationInput,
    OsmosisEquationInput, canonicalize_frequency_hz,
};
#[cfg(feature = "bridge_optimizer")]
pub use impls::physics::{BridgedPhysicsOps, phase_band_hint_from_state};
pub use cache::{BridgeCache, CacheStats, CachedValue, register_bridge_cache};
pub use config::{
    BandDef, BandValidationError, BridgeConfig, BridgeKind, CachePolicy, CatalysisBridge,
    CollisionTransferBridge, CompetitionNormBridge, DensityBridge, DissipationBridge, DragBridge,
    EngineBridge, EvolutionSurrogateBridge, EvictionPolicy, InterferenceBridge, OsmosisBridge,
    PhaseTransitionBridge, Rigidity, TemperatureBridge, WillBridge, validate_bands,
};
pub use constants::{
    INTERFERENCE_PHASE_SECTORS, INTERFERENCE_TIME_QUANT_S, VEC2_DIRECTION_ZERO_EPS_SQ,
    VEC2_STATIC_SECTOR,
};
pub use context_fill::{
    BridgePhase, BridgePhaseChanged, BridgePhaseConfig, BridgePhaseState,
    apply_bridge_phase_side_effects, bridge_caches_max_fill_ratio, bridge_phase_apply_transition,
    bridge_phase_force, bridge_phase_reset, bridge_phase_tick, clear_all_bridge_caches,
};
pub use decorator::{
    Bridgeable, bridge_compute, bridge_compute_with_hint, bridge_warmup_record,
    bridge_warmup_record_with_hint, hash_inputs,
};
pub use metrics::{
    BridgeLayerRow, BridgeMetrics, BridgeMetricsConfig, BridgeMetricsSnapshot,
    BridgeMetricsSummary, BridgeOptimizerPhaseLogState, absorb_cache_stats_into_metrics,
    bridge_cache_fill_report, bridge_layer_name, bridge_metrics_collect_all,
    bridge_metrics_collect_system, bridge_optimizer_enter_active_log_system,
    hit_rate_quality_prefix, rebuild_bridge_metrics_summary,
};
pub use normalize::{
    CANONICAL_DIRECTIONS_8, CANONICAL_DIRECTIONS_16, CANONICAL_DIRECTIONS_32,
    band_contains_half_open, band_index_of, direction_sector, normalize_direction,
    normalize_magnitude, normalize_scalar, normalize_vec2, quantize_precision, vec2_cache_key,
};
pub use presets::{
    BridgeConfigAsset, BridgeConfigAssetState, BridgeConfigLoader, BridgeConfigPartialRon,
    BridgeConfigPlugin, RigidityPreset, apply_bridge_config_asset, apply_preset,
    bridge_config_hot_reload_system, bridge_config_mark_dirty_system, build_config_from_partial,
    init_bridge_config_assets_system,
};
