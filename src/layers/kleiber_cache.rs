//! Cache exacta del factor de volumen Kleiber (`radius^0.75`).
//! Exact cache of Kleiber volume factor (`radius^0.75`).
//!
//! Elimina `powf()` per-tick en `basal_drain_system`. Actualización on-demand:
//! solo se recomputa cuando `SpatialVolume::radius` cambia (growth events).
//! SparseSet porque solo entidades con `SenescenceProfile` lo necesitan.

use bevy::prelude::*;

/// Cache del factor `radius^KLEIBER_EXPONENT`. SparseSet, co-localizado con SenescenceProfile.
/// Cache of `radius^KLEIBER_EXPONENT` factor. SparseSet, co-located with SenescenceProfile.
#[derive(Component, Reflect, Debug, Clone, Copy, PartialEq)]
#[reflect(Component, PartialEq)]
#[component(storage = "SparseSet")]
pub struct KleiberCache {
    vol_factor: f32,
    last_radius: f32,
}

impl Default for KleiberCache {
    fn default() -> Self {
        // NaN guarantees first update() triggers recompute (NaN != any f32).
        Self {
            vol_factor: 0.0,
            last_radius: f32::NAN,
        }
    }
}

impl KleiberCache {
    /// Factor de volumen cacheado (`radius^0.75`).
    /// Cached volume factor (`radius^0.75`).
    #[inline]
    pub fn vol_factor(&self) -> f32 {
        self.vol_factor
    }

    /// Radius que generó el factor cacheado.
    /// Radius that generated the cached factor.
    #[inline]
    pub fn last_radius(&self) -> f32 {
        self.last_radius
    }

    /// Actualiza el cache si el radius cambió. Retorna `true` si se recomputó.
    /// Updates cache if radius changed. Returns `true` if recomputed.
    #[inline]
    pub fn update(&mut self, current_radius: f32) -> bool {
        if self.last_radius == current_radius {
            return false;
        }
        self.vol_factor =
            crate::blueprint::equations::exact_cache::kleiber_volume_factor(current_radius);
        self.last_radius = current_radius;
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_triggers_update_on_any_radius() {
        let mut c = KleiberCache::default();
        assert!(
            c.update(1.0),
            "default last_radius=NaN should trigger update"
        );
    }

    #[test]
    fn same_radius_does_not_recompute() {
        let mut c = KleiberCache::default();
        c.update(2.0);
        assert!(!c.update(2.0), "same radius should not trigger recompute");
    }

    #[test]
    fn different_radius_recomputes() {
        let mut c = KleiberCache::default();
        c.update(2.0);
        assert!(c.update(3.0), "different radius should trigger recompute");
    }

    #[test]
    fn vol_factor_matches_exact_computation() {
        let mut c = KleiberCache::default();
        c.update(2.0);
        let expected = crate::blueprint::equations::exact_cache::kleiber_volume_factor(2.0);
        assert_eq!(c.vol_factor(), expected);
    }

    #[test]
    fn vol_factor_zero_radius() {
        let mut c = KleiberCache::default();
        c.update(0.0);
        assert_eq!(c.vol_factor(), 0.0);
    }

    #[test]
    fn vol_factor_nan_radius_returns_zero() {
        let mut c = KleiberCache::default();
        c.update(f32::NAN);
        assert_eq!(c.vol_factor(), 0.0);
    }

    #[test]
    fn vol_factor_infinity_radius_returns_zero() {
        let mut c = KleiberCache::default();
        c.update(f32::INFINITY);
        assert_eq!(c.vol_factor(), 0.0);
    }

    #[test]
    fn initialized_cache_is_partial_eq_to_itself() {
        let mut c = KleiberCache::default();
        c.update(2.0);
        assert_eq!(c, c, "initialized cache should eq itself");
    }

    // ── BS-5: Integration — cache matches uncached across radii ────────

    #[test]
    fn bs5_cache_matches_uncached_powf_across_radii() {
        let radii = [0.01, 0.1, 0.5, 1.0, 2.0, 5.0, 10.0, 50.0, 100.0];
        let mut cache = KleiberCache::default();
        for &r in &radii {
            cache.update(r);
            let cached = cache.vol_factor();
            let uncached = r.max(0.01).powf(
                crate::blueprint::equations::derived_thresholds::KLEIBER_EXPONENT,
            );
            let delta = (cached - uncached).abs();
            assert!(
                delta < 1e-6,
                "BS-5: KleiberCache diverged at r={r}: cached={cached} uncached={uncached}"
            );
        }
    }
}
