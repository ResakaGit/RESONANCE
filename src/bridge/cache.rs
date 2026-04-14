//! Generic per-bridge cache — dispensable Resource (no effect on results if cleared).
//! See `docs/sprints/BRIDGE_OPTIMIZER/README.md` and `docs/design/BRIDGE_OPTIMIZER.md` §5.3.
//!
//! **B2:** eviction = LRU by `last_used` counter only.
//! `ContextFill` disables eviction during warmup (B7).
//! **B7:** `eviction_enabled` (Filling phase) disables eviction when inserting into a full cache.

use core::marker::PhantomData;

use bevy::prelude::*;
use fxhash::FxHashMap;

use crate::bridge::config::{BridgeKind, CachePolicy};
use crate::layers::MatterState;

/// Cached output per equation — union of types used in physics/interference bridges.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum CachedValue {
    Scalar(f32),
    State(MatterState),
    Vector(Vec2),
}

const _: () = assert!(core::mem::size_of::<CachedValue>() <= 12);

#[derive(Clone, Copy, Debug, PartialEq)]
struct CacheEntry {
    key: u64,
    value: CachedValue,
    last_used: u64,
}

/// Internal backend: linear Vec (N ≤ 256) or map (N > 256). Same LRU semantics via `last_used`.
#[derive(Debug)]
enum CacheBackend {
    Small { entries: Vec<CacheEntry> },
    Large { entries: FxHashMap<u64, CacheEntry> },
}

impl CacheBackend {
    fn new(capacity: usize) -> Self {
        if capacity <= 256 {
            Self::Small {
                entries: Vec::with_capacity(capacity.min(256)),
            }
        } else {
            Self::Large {
                entries: FxHashMap::default(),
            }
        }
    }

    fn len(&self) -> usize {
        match self {
            CacheBackend::Small { entries } => entries.len(),
            CacheBackend::Large { entries } => entries.len(),
        }
    }

    fn clear(&mut self) {
        match self {
            CacheBackend::Small { entries } => entries.clear(),
            CacheBackend::Large { entries } => entries.clear(),
        }
    }

    fn get_mut(&mut self, key: u64) -> Option<&mut CacheEntry> {
        match self {
            CacheBackend::Small { entries } => entries.iter_mut().find(|e| e.key == key),
            CacheBackend::Large { entries } => entries.get_mut(&key),
        }
    }

    /// Index or key of the candidate with minimum `last_used` (LRU eviction).
    fn eviction_victim(&self) -> Option<Victim> {
        match self {
            CacheBackend::Small { entries } => entries
                .iter()
                .enumerate()
                .min_by(|(ia, ea), (ib, eb)| {
                    ea.last_used.cmp(&eb.last_used).then_with(|| ia.cmp(ib))
                })
                .map(|(i, _)| Victim::Small(i)),
            CacheBackend::Large { entries } => entries
                .iter()
                .min_by(|(ka, ea), (kb, eb)| {
                    ea.last_used.cmp(&eb.last_used).then_with(|| ka.cmp(kb))
                })
                .map(|(k, _)| Victim::Large(*k)),
        }
    }

    fn remove_victim(&mut self, victim: Victim) {
        match (self, victim) {
            (CacheBackend::Small { entries }, Victim::Small(i)) => {
                entries.swap_remove(i);
            }
            (CacheBackend::Large { entries }, Victim::Large(k)) => {
                entries.remove(&k);
            }
            _ => {}
        }
    }

    fn insert_entry(&mut self, entry: CacheEntry) {
        match self {
            CacheBackend::Small { entries } => {
                if let Some(e) = entries.iter_mut().find(|e| e.key == entry.key) {
                    *e = entry;
                } else {
                    entries.push(entry);
                }
            }
            CacheBackend::Large { entries } => {
                entries.insert(entry.key, entry);
            }
        }
    }
}

#[derive(Clone, Copy, Debug)]
enum Victim {
    Small(usize),
    Large(u64),
}

/// Usage statistics — cache is dispensable: clearing it does not change pipeline results.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CacheStats {
    pub hits: u64,
    pub misses: u64,
    pub evictions: u64,
    pub hit_rate: f32,
    pub len: usize,
    pub capacity: usize,
}

/// Resource por tipo de bridge — aislamiento en compile time, sin registry global.
#[derive(Resource, Debug)]
pub struct BridgeCache<B: BridgeKind> {
    capacity: usize,
    policy: CachePolicy,
    /// Si es `false`, no se evicta al insertar con cache llena (fase Filling, sprint B7).
    eviction_enabled: bool,
    backend: CacheBackend,
    /// Monotonic counter for LRU timestamp (not the simulation tick).
    clock: u64,
    hits: u64,
    misses: u64,
    evictions: u64,
    _marker: PhantomData<B>,
}

impl<B: BridgeKind> BridgeCache<B> {
    pub fn new(capacity: usize, policy: CachePolicy) -> Self {
        Self {
            capacity,
            policy,
            eviction_enabled: true,
            backend: CacheBackend::new(capacity),
            clock: 0,
            hits: 0,
            misses: 0,
            evictions: 0,
            _marker: PhantomData,
        }
    }

    #[inline]
    pub fn set_eviction_enabled(&mut self, enabled: bool) {
        self.eviction_enabled = enabled;
    }

    #[inline]
    pub fn eviction_enabled(&self) -> bool {
        self.eviction_enabled
    }

    /// Exposed for Small/Large selection tests (capacity threshold 256).
    #[cfg(test)]
    pub(crate) fn backend_kind(&self) -> &'static str {
        match &self.backend {
            CacheBackend::Small { .. } => "Small",
            CacheBackend::Large { .. } => "Large",
        }
    }

    fn next_clock(&mut self) -> u64 {
        self.clock = self.clock.wrapping_add(1);
        self.clock
    }

    pub fn lookup(&mut self, key: u64) -> Option<CachedValue> {
        let tick = self.next_clock();
        if let Some(e) = self.backend.get_mut(key) {
            e.last_used = tick;
            self.hits += 1;
            Some(e.value)
        } else {
            self.misses += 1;
            None
        }
    }

    pub fn insert(&mut self, key: u64, value: CachedValue) {
        if self.capacity == 0 {
            return;
        }

        let tick = self.next_clock();

        if let Some(e) = self.backend.get_mut(key) {
            e.value = value;
            e.last_used = tick;
            return;
        }

        while self.eviction_enabled && self.backend.len() >= self.capacity {
            if let Some(victim) = self.backend.eviction_victim() {
                self.backend.remove_victim(victim);
                self.evictions += 1;
            } else {
                break;
            }
        }

        if !self.eviction_enabled && self.backend.len() >= self.capacity {
            // No eviction and full: do not insert new keys (only updates above).
            return;
        }

        self.backend.insert_entry(CacheEntry {
            key,
            value,
            last_used: tick,
        });
    }

    pub fn clear(&mut self) {
        self.backend.clear();
        self.hits = 0;
        self.misses = 0;
        self.evictions = 0;
        self.clock = 0;
    }

    pub fn stats(&self) -> CacheStats {
        let denom = self.hits + self.misses;
        let hit_rate = if denom > 0 {
            self.hits as f32 / denom as f32
        } else {
            0.0
        };
        CacheStats {
            hits: self.hits,
            misses: self.misses,
            evictions: self.evictions,
            hit_rate,
            len: self.backend.len(),
            capacity: self.capacity,
        }
    }

    /// Resets period counters (hits/misses/evictions) without clearing entries — windowed metrics (B9).
    pub fn reset_usage_counters(&mut self) {
        self.hits = 0;
        self.misses = 0;
        self.evictions = 0;
    }

    pub fn policy(&self) -> CachePolicy {
        self.policy
    }
}

/// Inserts `BridgeCache<B>` as a Resource. If not registered, `ResMut`/`Res` will panic at runtime (Bevy).
pub fn register_bridge_cache<B: BridgeKind>(app: &mut App, capacity: usize, policy: CachePolicy) {
    app.insert_resource(BridgeCache::<B>::new(capacity, policy));
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bridge::config::DensityBridge;

    #[derive(Debug)]
    struct OtherBridge;
    impl BridgeKind for OtherBridge {}

    #[test]
    fn insert_lookup_returns_value() {
        let mut c = BridgeCache::<DensityBridge>::new(8, CachePolicy::Lru);
        c.insert(1, CachedValue::Scalar(3.5));
        assert_eq!(c.lookup(1), Some(CachedValue::Scalar(3.5)));
    }

    #[test]
    fn cached_value_state_roundtrip() {
        let mut c = BridgeCache::<DensityBridge>::new(8, CachePolicy::Lru);
        c.insert(7, CachedValue::State(MatterState::Liquid));
        assert_eq!(c.lookup(7), Some(CachedValue::State(MatterState::Liquid)));
    }

    #[test]
    fn cached_value_vector_roundtrip() {
        let mut c = BridgeCache::<DensityBridge>::new(8, CachePolicy::Lru);
        let v = Vec2::new(1.25, -3.5);
        c.insert(8, CachedValue::Vector(v));
        assert_eq!(c.lookup(8), Some(CachedValue::Vector(v)));
    }

    #[test]
    fn filling_phase_no_evict_skips_new_keys_when_full() {
        let mut c = BridgeCache::<DensityBridge>::new(2, CachePolicy::Lru);
        c.set_eviction_enabled(false);
        c.insert(1, CachedValue::Scalar(1.0));
        c.insert(2, CachedValue::Scalar(2.0));
        c.insert(3, CachedValue::Scalar(3.0));
        assert_eq!(c.stats().len, 2);
        assert_eq!(c.lookup(1), Some(CachedValue::Scalar(1.0)));
        assert_eq!(c.lookup(3), None);
        assert_eq!(c.stats().evictions, 0);
    }

    #[test]
    fn full_cache_evicts_lru_oldest() {
        let mut c = BridgeCache::<DensityBridge>::new(2, CachePolicy::Lru);
        c.insert(10, CachedValue::Scalar(1.0));
        c.insert(20, CachedValue::Scalar(2.0));
        // Full; victim is the entry with minimum last_used (10).
        c.insert(30, CachedValue::Scalar(3.0));
        assert_eq!(c.lookup(10), None);
        assert_eq!(c.lookup(20), Some(CachedValue::Scalar(2.0)));
        assert_eq!(c.lookup(30), Some(CachedValue::Scalar(3.0)));
        assert!(c.stats().evictions >= 1);
    }

    #[test]
    fn clear_empties_and_resets_stats() {
        let mut c = BridgeCache::<DensityBridge>::new(4, CachePolicy::Lru);
        c.insert(1, CachedValue::Scalar(1.0));
        let _ = c.lookup(1);
        c.clear();
        let s = c.stats();
        assert_eq!(s.hits, 0);
        assert_eq!(s.misses, 0);
        assert_eq!(s.evictions, 0);
        assert_eq!(s.len, 0);
        assert_eq!(c.lookup(1), None);
    }

    #[test]
    fn reset_usage_counters_keeps_entries_clears_hits_misses_evictions() {
        let mut c = BridgeCache::<DensityBridge>::new(4, CachePolicy::Lru);
        c.insert(1, CachedValue::Scalar(1.0));
        let _ = c.lookup(1);
        let _ = c.lookup(2);
        assert_eq!(c.stats().hits, 1);
        assert_eq!(c.stats().misses, 1);
        c.reset_usage_counters();
        let s = c.stats();
        assert_eq!(s.hits, 0);
        assert_eq!(s.misses, 0);
        assert_eq!(s.len, 1);
        assert_eq!(c.lookup(1), Some(CachedValue::Scalar(1.0)));
    }

    #[test]
    fn hit_rate_three_hits_one_miss() {
        let mut c = BridgeCache::<DensityBridge>::new(8, CachePolicy::Lru);
        c.insert(1, CachedValue::Scalar(42.0));
        assert_eq!(c.lookup(1), Some(CachedValue::Scalar(42.0)));
        assert_eq!(c.lookup(1), Some(CachedValue::Scalar(42.0)));
        assert_eq!(c.lookup(1), Some(CachedValue::Scalar(42.0)));
        assert_eq!(c.lookup(999), None);
        let s = c.stats();
        assert_eq!(s.hits, 3);
        assert_eq!(s.misses, 1);
        assert!((s.hit_rate - 0.75).abs() < 1e-5);
    }

    #[test]
    fn empty_lookup_miss() {
        let mut c = BridgeCache::<DensityBridge>::new(4, CachePolicy::Lru);
        assert_eq!(c.lookup(0), None);
        assert_eq!(c.stats().misses, 1);
    }

    #[test]
    fn double_insert_same_key_overwrites() {
        let mut c = BridgeCache::<DensityBridge>::new(4, CachePolicy::Lru);
        c.insert(5, CachedValue::Scalar(1.0));
        c.insert(5, CachedValue::Scalar(2.0));
        assert_eq!(c.lookup(5), Some(CachedValue::Scalar(2.0)));
        assert_eq!(c.stats().len, 1);
    }

    #[test]
    fn generic_bridges_are_isolated() {
        let mut a = BridgeCache::<DensityBridge>::new(4, CachePolicy::Lru);
        let mut b = BridgeCache::<OtherBridge>::new(4, CachePolicy::Lru);
        a.insert(1, CachedValue::Scalar(10.0));
        b.insert(1, CachedValue::Scalar(99.0));
        assert_eq!(a.lookup(1), Some(CachedValue::Scalar(10.0)));
        assert_eq!(b.lookup(1), Some(CachedValue::Scalar(99.0)));
    }

    #[test]
    fn backend_small_vs_large_threshold() {
        let small = BridgeCache::<DensityBridge>::new(256, CachePolicy::Lru);
        assert_eq!(small.backend_kind(), "Small");
        let large = BridgeCache::<DensityBridge>::new(257, CachePolicy::Lru);
        assert_eq!(large.backend_kind(), "Large");
    }

    #[test]
    fn register_bridge_cache_inserts_resource() {
        let mut app = App::new();
        register_bridge_cache::<DensityBridge>(&mut app, 100, CachePolicy::ContextFill);
        let world = app.world();
        assert!(world.get_resource::<BridgeCache<DensityBridge>>().is_some());
    }
}
