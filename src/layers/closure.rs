//! AP-1/2: ClosureMetrics — seguimiento per-tick de una closure detectada.
//! AP-1/2: ClosureMetrics — per-tick tracking of a detected closure.
//!
//! Component SparseSet: sólo las entidades que *representan* una closure
//! detectada llevan este componente.  La detección misma corre sobre el
//! `SpeciesGrid` (AP-1) — no es por-entidad.  Este componente existe para que
//! AP-6 pueda asociar linaje, historia de K-stability y timestamp a cada
//! closure viva en la simulación.
//!
//! Ring buffer inline de 16 muestras — 64 B, cabe en 1 cache line.

use bevy::prelude::*;

/// Capacidad del historial corto de K-stability por closure.
pub const CLOSURE_HISTORY_LEN: usize = 16;

/// Métricas per-closure, actualizadas cada N ticks por `closure_metrics_system`
/// (wiring en AP-6).  Max 4 fields — el ring buffer es un `[f32; 16]` dentro
/// de la misma estructura, contando como un único campo compuesto.
#[derive(Component, Reflect, Debug, Clone)]
#[reflect(Component)]
#[component(storage = "SparseSet")]
pub struct ClosureMetrics {
    /// Hash FNV-1a de `(reactions, species)` — estable entre detecciones.
    pub hash: u64,
    /// Último valor computado de `kinetic_stability`.
    pub k_stability: f32,
    /// Ticks vivos desde la primera detección.
    pub age_ticks: u32,
    /// Historial corto de K (ring buffer).
    pub history: ClosureHistory,
}

impl ClosureMetrics {
    pub fn new(hash: u64, k_stability: f32) -> Self {
        let mut history = ClosureHistory::default();
        history.push(k_stability);
        Self { hash, k_stability, age_ticks: 0, history }
    }

    /// Actualiza `k_stability`, incrementa edad y empuja en el historial.
    /// `k` no-finito ⇒ no-op (no corrompe el estado).  Contrato:
    /// tras una llamada exitosa, `age_ticks` incrementa y `history.len()`
    /// aumenta (hasta capacidad) en exactamente 1.
    pub fn observe(&mut self, k: f32) {
        if !k.is_finite() { return; }
        self.k_stability = k;
        self.age_ticks = self.age_ticks.saturating_add(1);
        self.history.push(k);
    }

    /// Media móvil del historial — útil para detectar tendencia.
    #[inline]
    pub fn mean_k(&self) -> f32 { self.history.mean() }

    /// `true` si la media del historial supera el umbral de persistencia.
    #[inline]
    pub fn is_persistent(&self) -> bool {
        self.history.mean() >= crate::blueprint::constants::chemistry::KINETIC_STABILITY_PERSISTENT
    }
}

/// Ring buffer inline de `CLOSURE_HISTORY_LEN` muestras.  `Copy`, 68 B.
#[derive(Reflect, Debug, Clone, Copy)]
pub struct ClosureHistory {
    samples: [f32; CLOSURE_HISTORY_LEN],
    head: u8,
    filled: u8,
}

impl Default for ClosureHistory {
    fn default() -> Self {
        Self { samples: [0.0; CLOSURE_HISTORY_LEN], head: 0, filled: 0 }
    }
}

impl ClosureHistory {
    /// Agrega una muestra al ring buffer.  `sample` no-finito ⇒ no-op
    /// (no se empuja 0.0 silenciosamente — el historial refleja observaciones reales).
    pub fn push(&mut self, sample: f32) {
        if !sample.is_finite() { return; }
        let h = self.head as usize;
        self.samples[h] = sample;
        self.head = ((h + 1) % CLOSURE_HISTORY_LEN) as u8;
        if (self.filled as usize) < CLOSURE_HISTORY_LEN {
            self.filled += 1;
        }
    }

    #[inline] pub fn len(&self) -> usize { self.filled as usize }
    #[inline] pub fn is_empty(&self) -> bool { self.filled == 0 }
    #[inline] pub fn capacity(&self) -> usize { CLOSURE_HISTORY_LEN }

    pub fn mean(&self) -> f32 {
        let n = self.filled as usize;
        if n == 0 { return 0.0; }
        let sum: f32 = self.samples.iter().take(n).copied().sum();
        sum / n as f32
    }

    /// Iterador en orden cronológico (más viejo → más nuevo).
    pub fn iter_chrono(&self) -> impl Iterator<Item = f32> + '_ {
        let n = self.filled as usize;
        let head = self.head as usize;
        let start = if n < CLOSURE_HISTORY_LEN { 0 } else { head };
        (0..n).map(move |i| self.samples[(start + i) % CLOSURE_HISTORY_LEN])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn history_push_and_mean() {
        let mut h = ClosureHistory::default();
        assert!(h.is_empty());
        h.push(1.0); h.push(2.0); h.push(3.0);
        assert_eq!(h.len(), 3);
        assert!((h.mean() - 2.0).abs() < 1e-6);
    }

    #[test]
    fn history_wraps_at_capacity() {
        let mut h = ClosureHistory::default();
        for i in 0..(CLOSURE_HISTORY_LEN + 5) {
            h.push(i as f32);
        }
        assert_eq!(h.len(), CLOSURE_HISTORY_LEN);
        // Los primeros 5 fueron sobreescritos; los últimos 16 permanecen.
        let vals: Vec<f32> = h.iter_chrono().collect();
        assert_eq!(vals.first().copied(), Some(5.0));
        assert_eq!(vals.last().copied(), Some((CLOSURE_HISTORY_LEN + 4) as f32));
    }

    #[test]
    fn history_skips_non_finite() {
        let mut h = ClosureHistory::default();
        h.push(f32::NAN);
        h.push(f32::INFINITY);
        h.push(f32::NEG_INFINITY);
        assert!(h.is_empty(), "non-finite samples must not enter the buffer");
        h.push(2.0);
        h.push(f32::NAN);
        h.push(4.0);
        assert_eq!(h.len(), 2);
        assert!((h.mean() - 3.0).abs() < 1e-6);
    }

    #[test]
    fn observe_ignores_non_finite_and_preserves_age() {
        let mut m = ClosureMetrics::new(0, 1.0);
        let age_before = m.age_ticks;
        let hist_len_before = m.history.len();
        m.observe(f32::NAN);
        assert_eq!(m.age_ticks, age_before, "age must not advance on NaN");
        assert_eq!(m.history.len(), hist_len_before, "history must not grow on NaN");
        assert_eq!(m.k_stability, 1.0, "last valid k preserved");
    }

    #[test]
    fn metrics_observe_increments_age_and_updates_k() {
        let mut m = ClosureMetrics::new(0xdead_beef, 1.2);
        assert_eq!(m.age_ticks, 0);
        m.observe(1.5);
        assert_eq!(m.age_ticks, 1);
        assert_eq!(m.k_stability, 1.5);
        assert_eq!(m.history.len(), 2);
    }

    #[test]
    fn metrics_is_persistent_based_on_mean() {
        let mut m = ClosureMetrics::new(0, 0.5);
        for _ in 0..5 { m.observe(2.0); }
        assert!(m.is_persistent());
        let mut weak = ClosureMetrics::new(0, 0.2);
        for _ in 0..5 { weak.observe(0.3); }
        assert!(!weak.is_persistent());
    }

    #[test]
    fn iter_chrono_respects_insertion_order() {
        let mut h = ClosureHistory::default();
        for v in [1.0, 2.0, 3.0] { h.push(v); }
        let out: Vec<f32> = h.iter_chrono().collect();
        assert_eq!(out, vec![1.0, 2.0, 3.0]);
    }
}
