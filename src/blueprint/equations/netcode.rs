//! GS-1: Netcode lockstep — pure equations for input delay and desync detection.

/// Delay de input en ticks para absorber RTT máximo del grupo.
/// `delay = ceil(rtt_half_ms / 1000 * tick_rate_hz)`, mínimo 1.
#[inline]
pub fn input_delay_ticks(max_rtt_ms: f32, tick_rate_hz: f32) -> u32 {
    ((max_rtt_ms * 0.5 / 1000.0) * tick_rate_hz).ceil().max(1.0) as u32
}

/// ¿Es el delay aceptable para juego competitivo? (≤6 ticks = aceptable a 20Hz)
#[inline]
pub fn is_delay_acceptable(delay_ticks: u32) -> bool {
    delay_ticks <= crate::blueprint::constants::LOCKSTEP_MAX_ACCEPTABLE_DELAY_TICKS
}

/// Hash rápido del estado para detección de desync. No criptográfico.
#[inline]
pub fn tick_checksum(energy_snapshot: &[f32]) -> u64 {
    crate::blueprint::equations::determinism::hash_f32_slice(energy_snapshot)
}

/// Costo de corrección en ticks (para rollback planning).
#[inline]
pub fn correction_cost_ticks(ticks_since_divergence: u32) -> u32 {
    ticks_since_divergence
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn input_delay_100ms_20hz() {
        assert_eq!(input_delay_ticks(100.0, 20.0), 1);
    }

    #[test]
    fn input_delay_300ms_20hz() {
        assert_eq!(input_delay_ticks(300.0, 20.0), 3);
    }

    #[test]
    fn input_delay_zero_rtt_returns_minimum_one() {
        assert_eq!(input_delay_ticks(0.0, 20.0), 1);
    }

    #[test]
    fn delay_acceptable_6() {
        assert!(is_delay_acceptable(6));
    }

    #[test]
    fn delay_not_acceptable_9() {
        assert!(!is_delay_acceptable(9));
    }

    #[test]
    fn delay_acceptable_boundary_exactly_max() {
        assert!(is_delay_acceptable(
            crate::blueprint::constants::LOCKSTEP_MAX_ACCEPTABLE_DELAY_TICKS
        ));
    }

    #[test]
    fn checksum_deterministic() {
        assert_eq!(tick_checksum(&[1.0, 2.0]), tick_checksum(&[1.0, 2.0]));
    }

    #[test]
    fn checksum_order_matters() {
        assert_ne!(tick_checksum(&[1.0, 2.0]), tick_checksum(&[2.0, 1.0]));
    }

    #[test]
    fn correction_cost_identity() {
        assert_eq!(correction_cost_ticks(7), 7);
        assert_eq!(correction_cost_ticks(0), 0);
    }
}
