//! ET-1: Asociative Memory — puras. Sin deps de Bevy.

/// Fuerza de asociación tras `elapsed_ticks` aplicando decay exponencial.
/// `outcome_qe`: valor observado. `decay_rate`: fracción perdida por tick.
pub fn association_strength(outcome_qe: f32, elapsed_ticks: u64, decay_rate: f32) -> f32 {
    let decay = (-decay_rate * elapsed_ticks as f32).exp();
    outcome_qe * decay
}

/// Valor esperado de un estímulo dado el historial de entradas y fuerzas.
/// `stimuli`: hashes de estímulos. `strengths`: fuerza asociada a cada uno.
/// `query_hash`: estímulo consultado. Retorna 0.0 si no está en historial.
pub fn expected_stimulus_value(stimuli: &[u32], strengths: &[f32], query_hash: u32) -> f32 {
    stimuli
        .iter()
        .zip(strengths.iter())
        .find(|&(&s, _)| s == query_hash)
        .map(|(_, &v)| v)
        .unwrap_or(0.0)
}

/// Costo de mantener `entry_count` asociaciones activas por tick.
pub fn memory_maintenance_cost(entry_count: u8, cost_per_entry: f32) -> f32 {
    entry_count as f32 * cost_per_entry
}

/// Hash deterministico de un estímulo a partir de frecuencia + posición relativa.
/// Usa multiplicación de números primos — sin floats en la clave, sin colisiones frecuentes.
pub fn stimulus_hash(freq_band: u8, rel_x_band: i8, rel_z_band: i8) -> u32 {
    let a = freq_band as u32;
    let b = (rel_x_band as i32 + 128) as u32;
    let c = (rel_z_band as i32 + 128) as u32;
    a.wrapping_mul(73856093) ^ b.wrapping_mul(19349663) ^ c.wrapping_mul(83492791)
}

/// Índice LRU: posición de la entrada con menor strength (candidata a reemplazo).
pub fn lru_victim_index(strengths: &[f32]) -> usize {
    strengths
        .iter()
        .enumerate()
        .min_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal))
        .map(|(i, _)| i)
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn association_strength_zero_elapsed_returns_outcome() {
        assert!((association_strength(10.0, 0, 0.001) - 10.0).abs() < 1e-4);
    }

    #[test]
    fn association_strength_decays_over_time() {
        let s0 = association_strength(10.0, 0, 0.001);
        let s1 = association_strength(10.0, 1000, 0.001);
        assert!(s1 < s0);
    }

    #[test]
    fn association_strength_zero_decay_never_decays() {
        let s = association_strength(5.0, 100_000, 0.0);
        assert!((s - 5.0).abs() < 1e-4);
    }

    #[test]
    fn expected_stimulus_value_found() {
        assert!((expected_stimulus_value(&[1, 2, 3], &[1.0, 2.0, 3.0], 2) - 2.0).abs() < 1e-5);
    }

    #[test]
    fn expected_stimulus_value_missing_returns_zero() {
        assert_eq!(expected_stimulus_value(&[1, 2], &[1.0, 2.0], 99), 0.0);
    }

    #[test]
    fn memory_maintenance_cost_proportional() {
        assert!((memory_maintenance_cost(4, 0.5) - 2.0).abs() < 1e-5);
    }

    #[test]
    fn stimulus_hash_deterministic() {
        assert_eq!(stimulus_hash(5, 10, -3), stimulus_hash(5, 10, -3));
    }

    #[test]
    fn stimulus_hash_different_inputs_differ() {
        assert_ne!(stimulus_hash(5, 10, -3), stimulus_hash(5, 10, -4));
    }

    #[test]
    fn lru_victim_index_finds_minimum() {
        assert_eq!(lru_victim_index(&[3.0, 1.0, 5.0, 2.0]), 1);
    }

    #[test]
    fn lru_victim_single_entry_returns_zero() {
        assert_eq!(lru_victim_index(&[99.0]), 0);
    }
}
