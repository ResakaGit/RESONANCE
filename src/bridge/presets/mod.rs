//! Configuración data-driven del Bridge Optimizer (RON + presets + hot reload).
//! Patrón alineado a `blueprint::almanac` — ver sprint B8 y `docs/design/BRIDGE_OPTIMIZER.md` §12.
//!
//! `RigidityPreset` (RON `preset`) y `Rigidity` (RON `rigidity`) son isomorfos: sirven para el mismo
//! eje de tuning; ver `resolve_template_preset` para precedencia al mezclar desde `BridgeConfigPartialRon`.

use std::collections::HashMap;
use std::path::Path;

use bevy::asset::{AssetLoader, AssetPath, LoadContext, io::Reader};
use bevy::prelude::*;
use bevy::reflect::TypePath;
use serde::{Deserialize, Serialize};

use crate::bridge::cache::{BridgeCache, register_bridge_cache};
use crate::bridge::config::{
    BandDef, BridgeConfig, BridgeKind, CachePolicy, CatalysisBridge, CollisionTransferBridge,
    CompetitionNormBridge, DensityBridge, DissipationBridge, DragBridge, EngineBridge,
    InterferenceBridge, OsmosisBridge, PhaseTransitionBridge, Rigidity, TemperatureBridge,
    WillBridge, validate_bands,
};
use crate::bridge::context_fill::{
    BridgePhase, BridgePhaseChanged, BridgePhaseConfig, BridgePhaseState,
    apply_bridge_phase_side_effects,
};
use crate::bridge::metrics::{
    BridgeMetrics, BridgeMetricsConfig, BridgeMetricsSummary, BridgeOptimizerPhaseLogState,
    bridge_layer_name,
};

// --- Preset público (sprint B8): distinto nombre que `Rigidity` en disco semántico ----------

/// Perfil de tuning predefinido — se mapea a `Rigidity` salvo override en RON.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RigidityPreset {
    Rigid,
    #[default]
    Moderate,
    Flexible,
    Transparent,
}

impl From<RigidityPreset> for Rigidity {
    fn from(p: RigidityPreset) -> Self {
        match p {
            RigidityPreset::Rigid => Rigidity::Rigid,
            RigidityPreset::Moderate => Rigidity::Moderate,
            RigidityPreset::Flexible => Rigidity::Flexible,
            RigidityPreset::Transparent => Rigidity::Transparent,
        }
    }
}

/// Convierte `Rigidity` (campo en disco / `BridgeConfig`) → preset de plantilla (1:1).
#[inline]
fn rigidity_preset_from_rigidity(r: Rigidity) -> RigidityPreset {
    match r {
        Rigidity::Rigid => RigidityPreset::Rigid,
        Rigidity::Moderate => RigidityPreset::Moderate,
        Rigidity::Flexible => RigidityPreset::Flexible,
        Rigidity::Transparent => RigidityPreset::Transparent,
    }
}

/// `rigidity` en RON gobierna bandas/histéresis/cache de plantilla si está presente; `preset` solo si no hay `rigidity`.
/// Si ambos están y discrepan → `warn!` y prevalece **`rigidity`** (fuente de verdad para la plantilla).
fn resolve_template_preset(partial: &BridgeConfigPartialRon, bridge_key: &str) -> RigidityPreset {
    let template = match partial.rigidity {
        Some(r) => rigidity_preset_from_rigidity(r),
        None => partial.preset.unwrap_or(RigidityPreset::Moderate),
    };

    if let (Some(p), Some(r)) = (partial.preset, partial.rigidity) {
        let from_preset = Rigidity::from(p);
        if from_preset != r {
            bevy::log::warn!(
                target: "bridge_config",
                bridge = bridge_key,
                ?p,
                rigidity = ?r,
                "preset and rigidity disagree; template (bands, base hysteresis, cache) follows rigidity"
            );
        }
    }

    template
}

// --- Payload RON (sin genérico; el loader instancia `BridgeConfig<B>` por puente) ------------

/// Campos opcionales en `bridge_config.ron`.
///
/// **Contrato:** `rigidity` (si existe) define la **plantilla** de cuantización (vía `config_for_preset` equivalente).
/// `preset` rellena la misma plantilla solo cuando `rigidity` está ausente. Overrides numéricos (`hysteresis_margin`, …)
/// se aplican después. `RigidityPreset` y `Rigidity` son isomorfos; duplicar en serde evita ambigüedad de nombre en RON.
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(default)]
pub struct BridgeConfigPartialRon {
    #[serde(default)]
    pub preset: Option<RigidityPreset>,
    #[serde(default)]
    pub bands: Option<Vec<BandDef>>,
    #[serde(default)]
    pub hysteresis_margin: Option<f32>,
    #[serde(default)]
    pub cache_capacity: Option<usize>,
    #[serde(default)]
    pub cache_policy: Option<CachePolicy>,
    #[serde(default)]
    pub enabled: Option<bool>,
    #[serde(default)]
    pub rigidity: Option<Rigidity>,
}

#[derive(Asset, TypePath, Clone, Debug, Deserialize, Serialize)]
pub struct BridgeConfigAsset {
    /// Claves: `density`, `temperature`, `phase_transition`, … (ver `BridgeDefaults::FILE_KEY`).
    #[serde(default)]
    pub bridges: HashMap<String, BridgeConfigPartialRon>,
}

#[derive(Default)]
pub struct BridgeConfigLoader;

impl AssetLoader for BridgeConfigLoader {
    type Asset = BridgeConfigAsset;
    type Settings = ();
    type Error = ron::error::SpannedError;

    async fn load(
        &self,
        reader: &mut dyn Reader,
        _settings: &(),
        _load_context: &mut LoadContext<'_>,
    ) -> Result<Self::Asset, Self::Error> {
        let mut bytes = Vec::new();
        reader.read_to_end(&mut bytes).await?;
        ron::de::from_bytes::<BridgeConfigAsset>(&bytes)
    }

    fn extensions(&self) -> &[&str] {
        &["ron"]
    }
}

// --- Defaults por puente --------------------------------------------------------------------

/// Defaults por tipo de puente — bandas base *Moderate* y variantes por `RigidityPreset`.
pub trait BridgeDefaults: BridgeKind + Sized {
    const FILE_KEY: &'static str;

    fn config_for_preset(preset: RigidityPreset) -> BridgeConfig<Self>;
}

pub(super) fn hysteresis_for(preset: RigidityPreset, base: f32) -> f32 {
    match preset {
        RigidityPreset::Rigid => base * 1.5,
        RigidityPreset::Moderate => base,
        RigidityPreset::Flexible => base * 0.5,
        RigidityPreset::Transparent => 0.0,
    }
}

pub(super) fn cache_cap_for(preset: RigidityPreset, base: usize) -> usize {
    match preset {
        RigidityPreset::Rigid => (base / 2).max(4),
        RigidityPreset::Moderate => base,
        RigidityPreset::Flexible => base.saturating_mul(2),
        RigidityPreset::Transparent => base,
    }
}

pub(super) fn policy_for(preset: RigidityPreset) -> CachePolicy {
    match preset {
        RigidityPreset::Transparent => CachePolicy::Lru,
        _ => CachePolicy::ContextFill,
    }
}

/// Une bandas adyacentes de a pares hasta quedar ~la mitad (rigidez alta).
fn merge_pairs_wide(bands: &[BandDef]) -> Vec<BandDef> {
    if bands.len() <= 2 {
        return bands.to_vec();
    }
    let mut out = Vec::new();
    let mut i = 0;
    while i < bands.len() {
        if i + 1 < bands.len() {
            let a = &bands[i];
            let b = &bands[i + 1];
            let mid_canon = (a.canonical + b.canonical) * 0.5;
            out.push(BandDef {
                min: a.min,
                max: b.max,
                canonical: mid_canon,
                stable: a.stable && b.stable,
            });
            i += 2;
        } else {
            out.push(bands[i]);
            i += 1;
        }
    }
    out
}

/// Parte cada banda por la mitad (más segmentos, más estrechos).
fn split_bands_narrow(bands: &[BandDef]) -> Vec<BandDef> {
    let mut out = Vec::new();
    for b in bands {
        if b.max - b.min < 1e-3 {
            out.push(*b);
            continue;
        }
        let mid = (b.min + b.max) * 0.5;
        let c1 = (b.min + mid) * 0.5;
        let c2 = (mid + b.max) * 0.5;
        out.push(BandDef { min: b.min, max: mid, canonical: c1, stable: b.stable });
        out.push(BandDef { min: mid,   max: b.max, canonical: c2, stable: b.stable });
    }
    out
}

pub(super) fn bands_for_equation_preset(moderate: &[BandDef], preset: RigidityPreset) -> Vec<BandDef> {
    match preset {
        RigidityPreset::Moderate => moderate.to_vec(),
        RigidityPreset::Rigid => {
            let once = merge_pairs_wide(moderate);
            if once.len() > 3 { merge_pairs_wide(&once) } else { once }
        }
        RigidityPreset::Flexible => split_bands_narrow(moderate),
        RigidityPreset::Transparent => vec![BandDef {
            min: 0.0,
            max: 100_000.0,
            canonical: 0.0,
            stable: true,
        }],
    }
}

pub(super) fn finish_config<B: BridgeKind>(
    bands: Vec<BandDef>,
    hysteresis_margin: f32,
    cache_capacity: usize,
    policy: CachePolicy,
    enabled: bool,
    rigidity: Rigidity,
) -> BridgeConfig<B> {
    BridgeConfig {
        bands,
        hysteresis_margin,
        cache_capacity,
        policy,
        enabled,
        rigidity,
        _marker: core::marker::PhantomData,
    }
}

macro_rules! impl_bridge_defaults {
    ($ty:ty, $key:literal, $moderate_bands:expr, $h_base:expr, $cap_base:expr) => {
        impl $crate::bridge::presets::BridgeDefaults for $ty {
            const FILE_KEY: &'static str = $key;

            fn config_for_preset(preset: $crate::bridge::presets::RigidityPreset) -> $crate::bridge::config::BridgeConfig<Self> {
                let bands = $crate::bridge::presets::bands_for_equation_preset(&$moderate_bands, preset);
                let h = $crate::bridge::presets::hysteresis_for(preset, $h_base);
                let cap = $crate::bridge::presets::cache_cap_for(preset, $cap_base);
                let policy = $crate::bridge::presets::policy_for(preset);
                let (enabled, rigidity) = match preset {
                    $crate::bridge::presets::RigidityPreset::Transparent => (false, $crate::bridge::config::Rigidity::Transparent),
                    _ => (true, $crate::bridge::config::Rigidity::from(preset)),
                };
                $crate::bridge::presets::finish_config::<Self>(bands, h, cap, policy, enabled, rigidity)
            }
        }
    };
}

pub mod combat;
pub mod ecosystem;
pub mod physics;

// --- Merge RON → config tipada -------------------------------------------------------------

/// Aplica defaults del preset al config completo (tests y reset explícito).
pub fn apply_preset<B: BridgeDefaults>(config: &mut BridgeConfig<B>, preset: RigidityPreset) {
    *config = B::config_for_preset(preset);
}

fn validate_or_fallback_bands<B: BridgeDefaults>(bands: &[BandDef], ctx: &str) -> Vec<BandDef> {
    if let Err(e) = validate_bands(bands) {
        bevy::log::warn!(
            target: "bridge_config",
            bridge = ctx,
            ?e,
            "invalid bands (gap/overlap/order); using bridge Moderate bands"
        );
        return B::config_for_preset(RigidityPreset::Moderate).bands;
    }
    bands.to_vec()
}

/// Construye `BridgeConfig<B>` desde entrada RON parcial.
pub fn build_config_from_partial<B: BridgeDefaults>(
    partial: &BridgeConfigPartialRon,
) -> BridgeConfig<B> {
    let template = resolve_template_preset(partial, B::FILE_KEY);
    let mut cfg = B::config_for_preset(template);

    if let Some(ref b) = partial.bands {
        cfg.bands = validate_or_fallback_bands::<B>(b, B::FILE_KEY);
    }
    if let Some(h) = partial.hysteresis_margin {
        cfg.hysteresis_margin = h;
    }
    if let Some(c) = partial.cache_capacity {
        cfg.cache_capacity = c;
    }
    if let Some(p) = partial.cache_policy {
        cfg.policy = p;
    }
    if let Some(e) = partial.enabled {
        cfg.enabled = e;
    }

    cfg
}

/// Aplica un asset completo al `World`: actualiza cada `BridgeConfig<B>` y opcionalmente limpia caches.
pub fn apply_bridge_config_asset(world: &mut World, asset: &BridgeConfigAsset, clear_caches: bool) {
    apply_bridge_config_for::<DensityBridge>(world, asset, clear_caches);
    apply_bridge_config_for::<TemperatureBridge>(world, asset, clear_caches);
    apply_bridge_config_for::<PhaseTransitionBridge>(world, asset, clear_caches);
    apply_bridge_config_for::<InterferenceBridge>(world, asset, clear_caches);
    apply_bridge_config_for::<DissipationBridge>(world, asset, clear_caches);
    apply_bridge_config_for::<DragBridge>(world, asset, clear_caches);
    apply_bridge_config_for::<EngineBridge>(world, asset, clear_caches);
    apply_bridge_config_for::<WillBridge>(world, asset, clear_caches);
    apply_bridge_config_for::<CatalysisBridge>(world, asset, clear_caches);
    apply_bridge_config_for::<CollisionTransferBridge>(world, asset, clear_caches);
    apply_bridge_config_for::<OsmosisBridge>(world, asset, clear_caches);
    apply_bridge_config_for::<CompetitionNormBridge>(world, asset, clear_caches);

    if world.contains_resource::<BridgePhaseState>() {
        let phase = world.resource::<BridgePhaseState>().phase;
        apply_bridge_phase_side_effects(world, phase);
    }
}

fn apply_bridge_config_for<B: BridgeDefaults>(
    world: &mut World,
    asset: &BridgeConfigAsset,
    clear_caches: bool,
) {
    match asset.bridges.get(B::FILE_KEY) {
        Some(partial) => {
            let cfg = build_config_from_partial::<B>(partial);
            world.insert_resource(cfg);
        }
        None => {
            bevy::log::warn!(
                target: "bridge_config",
                key = B::FILE_KEY,
                "bridge key missing in RON; using Moderate preset by default"
            );
            world.insert_resource(B::config_for_preset(RigidityPreset::Moderate));
        }
    }

    if clear_caches {
        if let Some(mut cache) = world.remove_resource::<BridgeCache<B>>() {
            cache.clear();
            world.insert_resource(cache);
        }
    }
}

// --- Bevy: estado + sistemas ----------------------------------------------------------------

#[derive(Resource, Debug, Default)]
pub struct BridgeConfigAssetState {
    pub initialized: bool,
    pub built: bool,
    /// `true` cuando hay que volver a leer el asset (carga inicial o `AssetEvent`).
    pub dirty: bool,
    pub handle: Option<Handle<BridgeConfigAsset>>,
}

fn bridge_config_path_exists() -> bool {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("assets")
        .join("bridge_config.ron")
        .is_file()
}

/// Startup: encola `bridge_config.ron` (si existe).
pub fn init_bridge_config_assets_system(
    asset_server: Res<AssetServer>,
    mut state: ResMut<BridgeConfigAssetState>,
) {
    if state.initialized {
        return;
    }
    state.initialized = true;
    if !bridge_config_path_exists() {
        state.built = true;
        state.handle = None;
        return;
    }
    let h: Handle<BridgeConfigAsset> = asset_server.load(AssetPath::from("bridge_config.ron"));
    state.handle = Some(h);
    state.built = false;
    state.dirty = true;
}

/// Marca pending cuando el asset cambia o aún no se aplicó la primera carga.
pub fn bridge_config_mark_dirty_system(
    mut state: ResMut<BridgeConfigAssetState>,
    mut events: EventReader<AssetEvent<BridgeConfigAsset>>,
) {
    if !state.initialized {
        return;
    }
    for event in events.read() {
        match event {
            AssetEvent::Added { .. }
            | AssetEvent::Modified { .. }
            | AssetEvent::Removed { .. }
            | AssetEvent::Unused { .. }
            | AssetEvent::LoadedWithDependencies { .. } => {
                state.dirty = true;
            }
        }
    }
    if !state.built {
        state.dirty = true;
    }
}

/// Aplica `bridge_config.ron` al `World` (exclusivo): primera carga sin limpiar caches; reload sí.
pub fn bridge_config_hot_reload_system(world: &mut World) {
    let Some(st) = world.get_resource::<BridgeConfigAssetState>() else {
        return;
    };
    if !st.initialized || !st.dirty {
        return;
    }

    let handle = st.handle.clone();
    let had_built_before = st.built;

    let Some(handle) = handle else {
        if let Some(mut s) = world.get_resource_mut::<BridgeConfigAssetState>() {
            s.dirty = false;
            s.built = true;
        }
        return;
    };

    let Some(asset) = world
        .get_resource::<Assets<BridgeConfigAsset>>()
        .and_then(|a| a.get(&handle))
        .cloned()
    else {
        return;
    };

    let clear_caches = had_built_before;
    apply_bridge_config_asset(&mut *world, &asset, clear_caches);

    if let Some(mut s) = world.get_resource_mut::<BridgeConfigAssetState>() {
        s.dirty = false;
        s.built = true;
    }
}

fn insert_default_bridge_resources(app: &mut App) {
    app.insert_resource(DensityBridge::config_for_preset(RigidityPreset::Moderate))
        .insert_resource(TemperatureBridge::config_for_preset(RigidityPreset::Moderate))
        .insert_resource(PhaseTransitionBridge::config_for_preset(RigidityPreset::Moderate))
        .insert_resource(InterferenceBridge::config_for_preset(RigidityPreset::Moderate))
        .insert_resource(DissipationBridge::config_for_preset(RigidityPreset::Moderate))
        .insert_resource(DragBridge::config_for_preset(RigidityPreset::Moderate))
        .insert_resource(EngineBridge::config_for_preset(RigidityPreset::Moderate))
        .insert_resource(WillBridge::config_for_preset(RigidityPreset::Moderate))
        .insert_resource(CatalysisBridge::config_for_preset(RigidityPreset::Moderate))
        .insert_resource(CollisionTransferBridge::config_for_preset(RigidityPreset::Moderate))
        .insert_resource(OsmosisBridge::config_for_preset(RigidityPreset::Moderate))
        .insert_resource(CompetitionNormBridge::config_for_preset(RigidityPreset::Moderate));
}

fn register_all_bridge_caches(app: &mut App) {
    register_bridge_cache::<DensityBridge>(
        app, DensityBridge::config_for_preset(RigidityPreset::Moderate).cache_capacity, CachePolicy::ContextFill,
    );
    register_bridge_cache::<TemperatureBridge>(
        app, TemperatureBridge::config_for_preset(RigidityPreset::Moderate).cache_capacity, CachePolicy::ContextFill,
    );
    register_bridge_cache::<PhaseTransitionBridge>(
        app, PhaseTransitionBridge::config_for_preset(RigidityPreset::Moderate).cache_capacity, CachePolicy::ContextFill,
    );
    register_bridge_cache::<InterferenceBridge>(
        app, InterferenceBridge::config_for_preset(RigidityPreset::Moderate).cache_capacity, CachePolicy::ContextFill,
    );
    register_bridge_cache::<DissipationBridge>(
        app, DissipationBridge::config_for_preset(RigidityPreset::Moderate).cache_capacity, CachePolicy::ContextFill,
    );
    register_bridge_cache::<DragBridge>(
        app, DragBridge::config_for_preset(RigidityPreset::Moderate).cache_capacity, CachePolicy::ContextFill,
    );
    register_bridge_cache::<EngineBridge>(
        app, EngineBridge::config_for_preset(RigidityPreset::Moderate).cache_capacity, CachePolicy::ContextFill,
    );
    register_bridge_cache::<WillBridge>(
        app, WillBridge::config_for_preset(RigidityPreset::Moderate).cache_capacity, CachePolicy::ContextFill,
    );
    register_bridge_cache::<CatalysisBridge>(
        app, CatalysisBridge::config_for_preset(RigidityPreset::Moderate).cache_capacity, CachePolicy::ContextFill,
    );
    register_bridge_cache::<CollisionTransferBridge>(
        app, CollisionTransferBridge::config_for_preset(RigidityPreset::Moderate).cache_capacity, CachePolicy::ContextFill,
    );
    register_bridge_cache::<OsmosisBridge>(
        app, OsmosisBridge::config_for_preset(RigidityPreset::Moderate).cache_capacity, CachePolicy::ContextFill,
    );
    register_bridge_cache::<CompetitionNormBridge>(
        app, CompetitionNormBridge::config_for_preset(RigidityPreset::Moderate).cache_capacity, CachePolicy::ContextFill,
    );
}

fn register_all_bridge_metrics(app: &mut App) {
    app.insert_resource(BridgeMetrics::<DensityBridge>::new(bridge_layer_name::<DensityBridge>()))
        .insert_resource(BridgeMetrics::<TemperatureBridge>::new(bridge_layer_name::<TemperatureBridge>()))
        .insert_resource(BridgeMetrics::<PhaseTransitionBridge>::new(bridge_layer_name::<PhaseTransitionBridge>()))
        .insert_resource(BridgeMetrics::<InterferenceBridge>::new(bridge_layer_name::<InterferenceBridge>()))
        .insert_resource(BridgeMetrics::<DissipationBridge>::new(bridge_layer_name::<DissipationBridge>()))
        .insert_resource(BridgeMetrics::<DragBridge>::new(bridge_layer_name::<DragBridge>()))
        .insert_resource(BridgeMetrics::<EngineBridge>::new(bridge_layer_name::<EngineBridge>()))
        .insert_resource(BridgeMetrics::<WillBridge>::new(bridge_layer_name::<WillBridge>()))
        .insert_resource(BridgeMetrics::<CatalysisBridge>::new(bridge_layer_name::<CatalysisBridge>()))
        .insert_resource(BridgeMetrics::<CollisionTransferBridge>::new(bridge_layer_name::<CollisionTransferBridge>()))
        .insert_resource(BridgeMetrics::<OsmosisBridge>::new(bridge_layer_name::<OsmosisBridge>()))
        .insert_resource(BridgeMetrics::<CompetitionNormBridge>::new(bridge_layer_name::<CompetitionNormBridge>()));
}

/// Plugin: asset RON + Resources `BridgeConfig<B>` + `BridgeCache<B>` por puente.
pub struct BridgeConfigPlugin;

impl Plugin for BridgeConfigPlugin {
    fn build(&self, app: &mut App) {
        app.init_asset::<BridgeConfigAsset>()
            .init_asset_loader::<BridgeConfigLoader>()
            .init_resource::<BridgeConfigAssetState>()
            .insert_resource(BridgePhaseConfig::default())
            .insert_resource(BridgePhaseState::default())
            .add_event::<BridgePhaseChanged>();

        insert_default_bridge_resources(app);
        register_all_bridge_caches(app);
        register_all_bridge_metrics(app);
        app.insert_resource(BridgeMetricsConfig::default())
            .init_resource::<BridgeMetricsSummary>()
            .init_resource::<BridgeOptimizerPhaseLogState>();
        apply_bridge_phase_side_effects(app.world_mut(), BridgePhase::Warmup);

        app.add_systems(Startup, init_bridge_config_assets_system)
            .add_systems(
                Update,
                (
                    bridge_config_mark_dirty_system,
                    bridge_config_hot_reload_system,
                )
                    .chain(),
            );
    }
}

// --- Tests ----------------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bridge_config_ron_on_disk_parses() {
        let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("assets/bridge_config.ron");
        let s = std::fs::read_to_string(path).expect("bridge_config.ron");
        let parsed: BridgeConfigAsset = ron::de::from_str(&s).expect("parse RON");
        assert!(!parsed.bridges.is_empty());
        assert!(parsed.bridges.contains_key(OsmosisBridge::FILE_KEY));
    }

    #[test]
    fn each_bridge_resource_type_after_apply() {
        let mut world = World::new();
        let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("assets/bridge_config.ron");
        let s = std::fs::read_to_string(path).unwrap();
        let asset: BridgeConfigAsset = ron::de::from_str(&s).unwrap();
        apply_bridge_config_asset(&mut world, &asset, false);
        assert!(world.get_resource::<BridgeConfig<DensityBridge>>().is_some());
        assert!(world.get_resource::<BridgeConfig<TemperatureBridge>>().is_some());
        assert!(world.get_resource::<BridgeConfig<PhaseTransitionBridge>>().is_some());
        assert!(world.get_resource::<BridgeConfig<InterferenceBridge>>().is_some());
        assert!(world.get_resource::<BridgeConfig<DissipationBridge>>().is_some());
        assert!(world.get_resource::<BridgeConfig<DragBridge>>().is_some());
        assert!(world.get_resource::<BridgeConfig<EngineBridge>>().is_some());
        assert!(world.get_resource::<BridgeConfig<WillBridge>>().is_some());
        assert!(world.get_resource::<BridgeConfig<CatalysisBridge>>().is_some());
        assert!(world.get_resource::<BridgeConfig<CollisionTransferBridge>>().is_some());
        assert!(world.get_resource::<BridgeConfig<OsmosisBridge>>().is_some());
    }

    #[test]
    fn apply_preset_rigid_wider_than_moderate() {
        let m = DensityBridge::config_for_preset(RigidityPreset::Moderate);
        let mut r = m.clone();
        apply_preset(&mut r, RigidityPreset::Rigid);
        let mw: f32 = m.bands.iter().map(|b| b.max - b.min).sum::<f32>() / m.bands.len() as f32;
        let rw: f32 = r.bands.iter().map(|b| b.max - b.min).sum::<f32>() / r.bands.len() as f32;
        assert!(rw >= mw * 0.99);
        assert!(r.hysteresis_margin > m.hysteresis_margin);
    }

    #[test]
    fn apply_preset_transparent_disables() {
        let mut c = DensityBridge::config_for_preset(RigidityPreset::Moderate);
        apply_preset(&mut c, RigidityPreset::Transparent);
        assert!(!c.enabled);
    }

    #[test]
    fn preset_plus_explicit_override_hysteresis() {
        let partial = BridgeConfigPartialRon {
            preset: Some(RigidityPreset::Moderate),
            hysteresis_margin: Some(7.0),
            ..Default::default()
        };
        let cfg = build_config_from_partial::<DensityBridge>(&partial);
        assert!((cfg.hysteresis_margin - 7.0).abs() < 1e-5);
        assert_eq!(
            cfg.bands,
            DensityBridge::config_for_preset(RigidityPreset::Moderate).bands
        );
    }

    /// `rigidity` manda la plantilla si entra en conflicto con `preset` (revisión contrato B8).
    #[test]
    fn rigidity_overrides_preset_for_template_bands() {
        let partial = BridgeConfigPartialRon {
            preset: Some(RigidityPreset::Moderate),
            rigidity: Some(Rigidity::Rigid),
            ..Default::default()
        };
        let cfg = build_config_from_partial::<DensityBridge>(&partial);
        let rigid_cfg = DensityBridge::config_for_preset(RigidityPreset::Rigid);
        let moderate_cfg = DensityBridge::config_for_preset(RigidityPreset::Moderate);
        assert_eq!(cfg.bands, rigid_cfg.bands);
        assert_eq!(cfg.rigidity, Rigidity::Rigid);
        assert_ne!(cfg.bands, moderate_cfg.bands);
        assert_eq!(cfg.hysteresis_margin, rigid_cfg.hysteresis_margin);
    }

    #[test]
    fn gap_bands_fallback_to_moderate() {
        let bad_bands = vec![
            BandDef { min: 0.0, max: 1.0, canonical: 0.5, stable: true },
            BandDef { min: 2.0, max: 3.0, canonical: 2.5, stable: true },
        ];
        let partial = BridgeConfigPartialRon {
            preset: Some(RigidityPreset::Moderate),
            bands: Some(bad_bands),
            ..Default::default()
        };
        let cfg = build_config_from_partial::<DensityBridge>(&partial);
        let moderate = DensityBridge::config_for_preset(RigidityPreset::Moderate);
        assert_eq!(cfg.bands, moderate.bands);
    }

    #[test]
    fn overlap_bands_fallback_to_moderate() {
        let bad_bands = vec![
            BandDef { min: 0.0, max: 2.0, canonical: 1.0, stable: true },
            BandDef { min: 1.0, max: 3.0, canonical: 2.0, stable: true },
        ];
        let partial = BridgeConfigPartialRon {
            bands: Some(bad_bands),
            ..Default::default()
        };
        let cfg = build_config_from_partial::<DensityBridge>(&partial);
        assert_eq!(
            cfg.bands,
            DensityBridge::config_for_preset(RigidityPreset::Moderate).bands
        );
    }

    #[test]
    fn missing_bridge_key_uses_moderate_default() {
        let asset = BridgeConfigAsset { bridges: HashMap::new() };
        let mut world = World::new();
        apply_bridge_config_for::<DensityBridge>(&mut world, &asset, false);
        assert_eq!(
            *world.resource::<BridgeConfig<DensityBridge>>(),
            DensityBridge::config_for_preset(RigidityPreset::Moderate)
        );
    }

    #[test]
    fn hot_reload_clears_density_cache() {
        let mut world = World::new();
        let mut cache = BridgeCache::<DensityBridge>::new(8, CachePolicy::Lru);
        cache.insert(1, crate::bridge::cache::CachedValue::Scalar(1.0));
        world.insert_resource(cache);
        let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("assets/bridge_config.ron");
        let s = std::fs::read_to_string(path).unwrap();
        let asset: BridgeConfigAsset = ron::de::from_str(&s).unwrap();
        apply_bridge_config_asset(&mut world, &asset, true);
        let cache = world.resource::<BridgeCache<DensityBridge>>();
        assert_eq!(cache.stats().len, 0);
    }
}
