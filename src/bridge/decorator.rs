//! Patrón decorador: `Bridgeable` + `bridge_compute` — sin dependencia de Bevy (testeable en unit tests).
//! Los resultados con bridge activo pueden diferir del cómputo exacto en ±epsilon de banda
//! (rango de normalización por capa; ver `BridgeConfig::bands` y `docs/arquitectura/blueprint_layer_bridge_optimizer.md` §5–6).
//!
//! Ver `docs/sprints/BRIDGE_OPTIMIZER/README.md` y `docs/design/BRIDGE_OPTIMIZER.md`.

use crate::bridge::cache::{BridgeCache, CachedValue};
use crate::bridge::config::{BridgeConfig, BridgeKind};

/// Integración de una ecuación pura con normalización + cache. El struct marcador `B` también
/// tipa `BridgeConfig<B>` y `BridgeCache<B>` (aislamiento por tipo, sin registry global).
pub trait Bridgeable: BridgeKind + Sized {
    type Input: Copy;
    type Output: Copy;

    /// Cuantiza entradas según `config` (histéresis opcional vía `band_hint`).
    fn normalize(
        input: Self::Input,
        config: &BridgeConfig<Self>,
        band_hint: Option<usize>,
    ) -> Self::Input;

    /// Clave determinista para el input ya normalizado.
    fn cache_key(normalized: Self::Input) -> u64;

    /// Función original sobre el input normalizado.
    fn compute(normalized: Self::Input) -> Self::Output;

    fn into_cached(value: Self::Output) -> CachedValue;

    fn from_cached(value: CachedValue) -> Option<Self::Output>;
}

/// FNV-1a 64-bit sobre partes ya cuantizadas (p. ej. `f32::to_bits` como `u64`).
#[inline]
pub fn hash_inputs(parts: &[u64]) -> u64 {
    const FNV_OFFSET: u64 = 1469598103934665603;
    const FNV_PRIME: u64 = 1099511628211;
    let mut h = FNV_OFFSET;
    for &p in parts {
        h ^= p;
        h = h.wrapping_mul(FNV_PRIME);
    }
    h
}

/// Pipeline: `normalize` → lookup → en miss `compute` + `insert`. Con `config.enabled == false`,
/// bypass total: `compute(input)` sin normalizar ni tocar la semántica de la ecuación exacta.
#[inline]
pub fn bridge_compute<B: Bridgeable>(
    input: B::Input,
    config: &BridgeConfig<B>,
    cache: &mut BridgeCache<B>,
) -> B::Output {
    bridge_compute_with_hint(input, config, cache, None)
}

/// Variante con pista de banda para histéresis (`normalize_scalar` en capas escalares).
#[inline]
pub fn bridge_compute_with_hint<B: Bridgeable>(
    input: B::Input,
    config: &BridgeConfig<B>,
    cache: &mut BridgeCache<B>,
    band_hint: Option<usize>,
) -> B::Output {
    if !config.enabled {
        return B::compute(input);
    }

    let normalized = B::normalize(input, config, band_hint);
    let key = B::cache_key(normalized);

    if let Some(cached) = cache.lookup(key) {
        if let Some(out) = B::from_cached(cached) {
            return out;
        }
    }

    let out = B::compute(normalized);
    cache.insert(key, B::into_cached(out));
    out
}

/// Warmup (context-fill): sin lookup; `out = B::compute(input)` sobre **crudo**; clave =
/// `cache_key(normalize(input))`. Si `compute(crudo) ≠ compute(normalizado)` en el mismo bucket,
/// el hit en Active puede diferir del miss-path — ver especialización en `bridged_physics::density`.
/// Ver `docs/sprints/BRIDGE_OPTIMIZER/README.md` (B7 cerrado).
#[inline]
pub fn bridge_warmup_record_with_hint<B: Bridgeable>(
    input: B::Input,
    config: &BridgeConfig<B>,
    cache: &mut BridgeCache<B>,
    band_hint: Option<usize>,
) -> B::Output {
    // Exacto sobre entrada cruda (bypass de cuantización en el cómputo); la clave sigue el espacio normalizado.
    let out = B::compute(input);
    let normalized = B::normalize(input, config, band_hint);
    let key = B::cache_key(normalized);
    cache.insert(key, B::into_cached(out));
    out
}

#[inline]
pub fn bridge_warmup_record<B: Bridgeable>(
    input: B::Input,
    config: &BridgeConfig<B>,
    cache: &mut BridgeCache<B>,
) -> B::Output {
    bridge_warmup_record_with_hint(input, config, cache, None)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bridge::config::{BandDef, CachePolicy, Rigidity};

    /// Ecuación trivial: `double(x) = x * 2.0` — solo en tests.
    #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
    struct DoubleBridge;

    impl BridgeKind for DoubleBridge {}

    impl Bridgeable for DoubleBridge {
        type Input = f32;
        type Output = f32;

        fn normalize(
            input: Self::Input,
            config: &BridgeConfig<Self>,
            band_hint: Option<usize>,
        ) -> Self::Input {
            crate::bridge::normalize_scalar(
                input,
                &config.bands,
                config.hysteresis_margin,
                band_hint,
            )
            .0
        }

        fn cache_key(normalized: Self::Input) -> u64 {
            hash_inputs(&[f32::to_bits(normalized) as u64])
        }

        fn compute(normalized: Self::Input) -> Self::Output {
            normalized * 2.0
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

    fn bands_two_contiguous() -> Vec<BandDef> {
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

    fn test_config(enabled: bool) -> BridgeConfig<DoubleBridge> {
        BridgeConfig::new(
            bands_two_contiguous(),
            0.25,
            16,
            CachePolicy::Lru,
            enabled,
            Rigidity::Moderate,
        )
        .expect("test bands")
    }

    #[test]
    fn bridge_compute_matches_double_first_call_miss() {
        let cfg = test_config(true);
        let mut cache = BridgeCache::<DoubleBridge>::new(16, CachePolicy::Lru);
        let x = 0.3_f32;
        let out = bridge_compute(x, &cfg, &mut cache);
        assert!((out - 1.0).abs() < 1e-5, "canonical 0.5 * 2 = 1.0");
        let s = cache.stats();
        assert_eq!(s.misses, 1);
    }

    #[test]
    fn second_call_same_raw_input_cache_hit() {
        let cfg = test_config(true);
        let mut cache = BridgeCache::<DoubleBridge>::new(16, CachePolicy::Lru);
        let x = 0.3_f32;
        let a = bridge_compute(x, &cfg, &mut cache);
        let b = bridge_compute(x, &cfg, &mut cache);
        assert_eq!(a, b);
        let s = cache.stats();
        assert_eq!(s.hits, 1);
        assert_eq!(s.misses, 1);
    }

    #[test]
    fn different_band_cache_miss() {
        let cfg = test_config(true);
        let mut cache = BridgeCache::<DoubleBridge>::new(16, CachePolicy::Lru);
        let _ = bridge_compute(0.3, &cfg, &mut cache);
        let out = bridge_compute(1.4, &cfg, &mut cache);
        assert!((out - 3.0).abs() < 1e-5);
        let s = cache.stats();
        assert_eq!(s.misses, 2);
    }

    #[test]
    fn same_band_different_raw_cache_hit() {
        let cfg = test_config(true);
        let mut cache = BridgeCache::<DoubleBridge>::new(16, CachePolicy::Lru);
        let _ = bridge_compute(0.3, &cfg, &mut cache);
        let out = bridge_compute(0.7, &cfg, &mut cache);
        assert!((out - 1.0).abs() < 1e-5);
        let s = cache.stats();
        assert_eq!(s.hits, 1);
        assert_eq!(s.misses, 1);
    }

    #[test]
    fn disabled_bypasses_cache_and_normalization() {
        let cfg = test_config(false);
        let mut cache = BridgeCache::<DoubleBridge>::new(16, CachePolicy::Lru);
        let x = 0.3_f32;
        let out = bridge_compute(x, &cfg, &mut cache);
        assert!((out - 0.6).abs() < 1e-5);
        assert_eq!(cache.stats().misses, 0);
        assert_eq!(cache.stats().hits, 0);
    }

    #[test]
    fn hash_inputs_equal_unequal() {
        let a = hash_inputs(&[1u64, 2u64]);
        let b = hash_inputs(&[1u64, 2u64]);
        let c = hash_inputs(&[1u64, 3u64]);
        assert_eq!(a, b);
        assert_ne!(a, c);
    }

    #[test]
    fn bridge_compute_with_hint_runs_without_panic() {
        let cfg = test_config(true);
        let mut cache = BridgeCache::<DoubleBridge>::new(16, CachePolicy::Lru);
        let _ = bridge_compute_with_hint(0.3, &cfg, &mut cache, Some(0));
        let _ = bridge_compute_with_hint(0.3, &cfg, &mut cache, Some(0));
        assert!(cache.stats().hits >= 1);
    }

    #[test]
    fn warmup_record_exact_output_inserts_normalized_key() {
        let cfg = test_config(true);
        let mut cache = BridgeCache::<DoubleBridge>::new(16, CachePolicy::Lru);
        let x = 0.3_f32;
        let w = bridge_warmup_record(x, &cfg, &mut cache);
        assert!(
            (w - 0.6).abs() < 1e-5,
            "warmup: exact compute on raw (0.3×2)"
        );
        assert_eq!(cache.stats().hits, 0, "warmup no hace lookup");
        assert_eq!(cache.stats().len, 1);
        let c = bridge_compute(x, &cfg, &mut cache);
        assert!(
            (c - 0.6).abs() < 1e-5,
            "hit returns context-fill cached value, no re-quantize"
        );
        assert!(cache.stats().hits >= 1);
    }
}
