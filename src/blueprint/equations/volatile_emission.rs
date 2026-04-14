//! Emisión de volátiles — cualquier órgano gaseoso con overflow emite.
//! Volatile emission — any gas-density organ with overflow emits.
//!
//! Not role-specific: emission condition is density < GAS_THRESHOLD and overflow > 0.

use crate::blueprint::constants::plant_physiology::*;
use crate::blueprint::equations::derived_thresholds::*;

/// Whether an organ can emit volatiles based on its density.
/// Si un órgano puede emitir volátiles basado en su densidad.
///
/// Only low-density (gas-like) organs emit. Dense organs retain energy.
#[inline]
pub fn can_emit_volatile(organ_density: f32) -> bool {
    organ_density > 0.0 && organ_density < gas_density_threshold()
}

/// Compute emission rate from overflow energy.
/// Calcular tasa de emisión desde energía excedente.
///
/// `rate = overflow × DISSIPATION_GAS × efficiency`
#[inline]
pub fn volatile_emission_rate(overflow_qe: f32) -> f32 {
    if overflow_qe <= 0.0 {
        return 0.0;
    }
    overflow_qe * VOLATILE_DECAY_RATE * VOLATILE_EFFICIENCY
}

/// Decay volatile signal by one tick.
/// Decaer señal volátil por un tick.
#[inline]
pub fn volatile_decay(signal: f32) -> f32 {
    (signal * (1.0 - VOLATILE_DECAY_RATE)).max(0.0)
}

/// Perceive volatile signal via frequency alignment (Axiom 8).
/// Percibir señal volátil via alineamiento de frecuencia.
///
/// Returns perceived intensity [0, signal]. Zero if frequencies misaligned.
#[inline]
pub fn perceive_volatile(
    signal: f32,
    volatile_freq: f32,
    sensor_freq: f32,
) -> f32 {
    if signal <= 0.0 {
        return 0.0;
    }
    let df = volatile_freq - sensor_freq;
    let bw = COHERENCE_BANDWIDTH;
    let alignment = (-0.5 * (df * df) / (bw * bw)).exp();
    signal * alignment
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn can_emit_low_density_true() {
        assert!(can_emit_volatile(1.0)); // very low density
    }

    #[test]
    fn can_emit_high_density_false() {
        assert!(!can_emit_volatile(10000.0)); // way above gas threshold
    }

    #[test]
    fn can_emit_zero_density_false() {
        assert!(!can_emit_volatile(0.0));
    }

    #[test]
    fn emission_rate_positive_on_overflow() {
        let rate = volatile_emission_rate(10.0);
        assert!(rate > 0.0);
    }

    #[test]
    fn emission_rate_zero_on_no_overflow() {
        assert_eq!(volatile_emission_rate(0.0), 0.0);
        assert_eq!(volatile_emission_rate(-5.0), 0.0);
    }

    #[test]
    fn decay_reduces_signal() {
        let after = volatile_decay(100.0);
        assert!(after < 100.0);
        assert!(after > 90.0); // ~92
    }

    #[test]
    fn decay_eight_ticks_halves() {
        let mut s = 100.0;
        for _ in 0..8 {
            s = volatile_decay(s);
        }
        assert!((s - 51.3).abs() < 2.0, "expected ~51, got {s}");
    }

    #[test]
    fn perceive_aligned_full_signal() {
        let p = perceive_volatile(100.0, 400.0, 400.0);
        assert!((p - 100.0).abs() < 1e-3);
    }

    #[test]
    fn perceive_misaligned_near_zero() {
        let p = perceive_volatile(100.0, 400.0, 700.0); // 300 Hz away, 6 bandwidths
        assert!(p < 1.0, "expected near zero, got {p}");
    }

    #[test]
    fn perceive_zero_signal_zero() {
        assert_eq!(perceive_volatile(0.0, 400.0, 400.0), 0.0);
    }
}
