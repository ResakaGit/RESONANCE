//! ET-15: Language (Symbolic Communication) — ecuaciones puras. Sin deps de Bevy.

/// Fitness de un símbolo: información transmitida menos costo de encoding.
pub fn symbol_fitness(information_bits: f32, reception_rate: f32, encoding_cost: f32) -> f32 {
    information_bits * reception_rate - encoding_cost
}

/// Vocabulario compartido entre dos entidades: intersección normalizada.
pub fn shared_vocabulary_ratio(vocab_a: &[u32], vocab_b: &[u32]) -> f32 {
    if vocab_a.is_empty() || vocab_b.is_empty() {
        return 0.0;
    }
    let shared = vocab_a.iter().filter(|s| vocab_b.contains(s)).count();
    shared as f32 / vocab_a.len().max(vocab_b.len()) as f32
}

/// Eficiencia de comunicación: vocabulario compartido × alcance / ruido.
pub fn communication_efficiency(shared_ratio: f32, signal_range: f32, noise_level: f32) -> f32 {
    if noise_level <= 0.0 {
        return shared_ratio * signal_range;
    }
    shared_ratio * signal_range / (1.0 + noise_level)
}

/// Tasa de deriva semántica: velocidad de cambio del significado de un símbolo.
pub fn semantic_drift_rate(
    symbol_usage_frequency: f32,
    population_size: f32,
    isolation_factor: f32,
) -> f32 {
    if symbol_usage_frequency <= 0.0 {
        return isolation_factor;
    }
    isolation_factor / (symbol_usage_frequency * population_size.sqrt())
}

/// Complejidad gramatical emergente: cuántos símbolos se combinan por mensaje.
pub fn grammar_complexity(vocab_size: u8, interaction_frequency: f32) -> f32 {
    (vocab_size as f32).ln() * interaction_frequency.sqrt()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn symbol_fitness_positive_useful_symbol() {
        assert!((symbol_fitness(4.0, 0.8, 1.0) - 2.2).abs() < 1e-5);
    }

    #[test]
    fn symbol_fitness_negative_costly_symbol() {
        assert!(symbol_fitness(1.0, 0.5, 2.0) < 0.0);
    }

    #[test]
    fn shared_vocabulary_ratio_partial_overlap() {
        let r = shared_vocabulary_ratio(&[1, 2, 3], &[2, 3, 4]);
        assert!((r - 2.0 / 3.0).abs() < 1e-5);
    }

    #[test]
    fn shared_vocabulary_ratio_empty_returns_zero() {
        assert_eq!(shared_vocabulary_ratio(&[], &[1, 2]), 0.0);
    }

    #[test]
    fn communication_efficiency_no_noise() {
        assert!((communication_efficiency(0.8, 10.0, 0.0) - 8.0).abs() < 1e-5);
    }

    #[test]
    fn communication_efficiency_with_noise() {
        let e = communication_efficiency(0.8, 10.0, 0.5);
        assert!((e - 8.0 / 1.5).abs() < 1e-4);
    }

    #[test]
    fn grammar_complexity_grows_with_vocab() {
        let small = grammar_complexity(4, 1.0);
        let large = grammar_complexity(8, 1.0);
        assert!(large > small);
    }

    #[test]
    fn grammar_complexity_eight_vocab() {
        let c = grammar_complexity(8, 1.0);
        assert!((c - 8.0f32.ln()).abs() < 1e-5);
    }
}
