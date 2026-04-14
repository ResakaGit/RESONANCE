//! Ecuaciones de energía per-órgano — distribución por densidad, no por rol.
//! Per-organ energy equations — distribution by density, not role.
//!
//! Every organ is a packet of energy with physical state. Its behavior
//! (priority, senescence rate, volatility) derives from density and matter_state.

use crate::blueprint::constants::ORGAN_DEATH_THRESHOLD;
use crate::blueprint::equations::derived_thresholds::*;

/// Density of an organ slot. Returns 0.0 if volume <= 0.
/// Densidad del slot de órgano. Retorna 0.0 si volumen <= 0.
#[inline]
pub fn organ_density(qe: f32, volume: f32) -> f32 {
    if volume <= 0.0 || qe <= 0.0 {
        return 0.0;
    }
    qe / volume
}

/// Priority of an organ within its entity, derived from density ratio.
/// Prioridad de un órgano, derivada de su densidad relativa.
///
/// Dense organs (solid-like) retain more energy under stress.
/// Low-density organs (gas-like) lose energy first.
#[inline]
pub fn organ_priority(density: f32, total_density: f32) -> f32 {
    if total_density <= 0.0 {
        return 0.0;
    }
    (density / total_density).clamp(0.0, 1.0)
}

/// Distribute entity energy across organ slots proportional to density.
/// Distribuir energía de la entidad entre órganos proporcional a densidad.
///
/// Returns array of organ energies. `sum(result) <= entity_qe` guaranteed.
pub fn distribute_organ_energy(entity_qe: f32, densities: &[f32], len: usize) -> [f32; 12] {
    let mut result = [0.0_f32; 12];
    if entity_qe <= 0.0 || len == 0 {
        return result;
    }
    let total: f32 = densities.iter().take(len).sum();
    if total <= 0.0 {
        // Equal distribution if all densities are zero
        let share = entity_qe / len as f32;
        for slot in result.iter_mut().take(len) {
            *slot = share;
        }
        return result;
    }
    for i in 0..len.min(12) {
        result[i] = entity_qe * (densities[i] / total);
    }
    result
}

/// Enforce pool invariant: sum(organ_qe) must not exceed entity_qe.
/// Normalizes proportionally if exceeded. Axiom 2 enforcement.
pub fn enforce_pool_invariant(organ_qe: &mut [f32; 12], len: usize, entity_qe: f32) {
    let sum: f32 = organ_qe.iter().take(len).sum();
    if sum <= entity_qe || sum <= 0.0 {
        return;
    }
    let scale = entity_qe / sum;
    for slot in organ_qe.iter_mut().take(len) {
        *slot *= scale;
    }
}

/// Senescence rate for an organ, derived from its density via matter_state.
/// Tasa de senescencia de un órgano, derivada de su densidad via matter_state.
///
/// Gas-density organs age 16× faster than solid-density (DISSIPATION_GAS/SOLID).
#[inline]
pub fn organ_senescence_rate(density: f32) -> f32 {
    let gas = gas_density_threshold();
    let liquid = liquid_density_threshold();
    if density >= gas {
        DISSIPATION_GAS
    } else if density >= liquid {
        DISSIPATION_LIQUID
    } else {
        DISSIPATION_SOLID
    }
}

/// Whether an organ is still alive (has enough energy).
/// Si un órgano sigue vivo (tiene suficiente energía).
#[inline]
pub fn organ_alive(organ_qe: f32) -> bool {
    organ_qe >= ORGAN_DEATH_THRESHOLD
}

/// Apply senescence drain to a single organ. Returns new qe after drain.
/// Aplicar drenaje de senescencia a un órgano. Retorna qe después del drenaje.
#[inline]
pub fn organ_senescence_drain(organ_qe: f32, senescence_rate: f32) -> f32 {
    (organ_qe * (1.0 - senescence_rate)).max(0.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn density_zero_volume_returns_zero() {
        assert_eq!(organ_density(10.0, 0.0), 0.0);
        assert_eq!(organ_density(0.0, 5.0), 0.0);
        assert_eq!(organ_density(-1.0, 5.0), 0.0);
    }

    #[test]
    fn density_positive_computes_ratio() {
        let d = organ_density(20.0, 4.0);
        assert!((d - 5.0).abs() < 1e-5);
    }

    #[test]
    fn priority_proportional_to_density() {
        let p = organ_priority(5.0, 10.0);
        assert!((p - 0.5).abs() < 1e-5);
    }

    #[test]
    fn priority_zero_total_returns_zero() {
        assert_eq!(organ_priority(5.0, 0.0), 0.0);
    }

    #[test]
    fn distribute_conserves_energy() {
        let densities = [3.0, 1.0, 0.5, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0];
        let result = distribute_organ_energy(100.0, &densities, 3);
        let sum: f32 = result.iter().sum();
        assert!((sum - 100.0).abs() < 1e-3, "sum={sum}");
        assert!(result[0] > result[1]); // higher density gets more
        assert!(result[1] > result[2]);
    }

    #[test]
    fn distribute_zero_qe_returns_zeros() {
        let densities = [1.0; 12];
        let result = distribute_organ_energy(0.0, &densities, 3);
        assert!(result.iter().all(|&v| v == 0.0));
    }

    #[test]
    fn distribute_zero_densities_equal_share() {
        let densities = [0.0; 12];
        let result = distribute_organ_energy(120.0, &densities, 3);
        assert!((result[0] - 40.0).abs() < 1e-3);
        assert!((result[1] - 40.0).abs() < 1e-3);
        assert!((result[2] - 40.0).abs() < 1e-3);
    }

    #[test]
    fn enforce_invariant_normalizes_when_exceeded() {
        let mut qe = [60.0, 60.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0];
        enforce_pool_invariant(&mut qe, 2, 100.0);
        let sum: f32 = qe[..2].iter().sum();
        assert!((sum - 100.0).abs() < 1e-3);
    }

    #[test]
    fn enforce_invariant_noop_when_under() {
        let mut qe = [30.0, 20.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0];
        enforce_pool_invariant(&mut qe, 2, 100.0);
        assert!((qe[0] - 30.0).abs() < 1e-5);
    }

    #[test]
    fn senescence_rate_scales_with_density() {
        let solid_rate = organ_senescence_rate(1.0); // very low density = solid
        let gas_rate = organ_senescence_rate(1000.0); // very high density = gas
        assert!(gas_rate > solid_rate);
        assert!((gas_rate / solid_rate - 16.0).abs() < 1.0);
    }

    #[test]
    fn organ_alive_above_threshold() {
        assert!(organ_alive(1.0));
        assert!(!organ_alive(0.01));
    }

    #[test]
    fn senescence_drain_reduces_qe() {
        let after = organ_senescence_drain(100.0, 0.08);
        assert!((after - 92.0).abs() < 1e-3);
    }

    #[test]
    fn senescence_drain_never_negative() {
        let after = organ_senescence_drain(0.001, 0.99);
        assert!(after >= 0.0);
    }
}
