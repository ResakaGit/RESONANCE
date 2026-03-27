//! ET-6: Epigenetic Expression — ecuaciones puras. Sin deps de Bevy.

/// ¿Debe expresarse este gen dado el entorno energético?
/// env_energy_ratio: field_qe / mean_field_qe — qué tan rico es el entorno.
pub fn should_express_gene(gene_benefit: f32, expression_cost: f32, env_energy_ratio: f32) -> bool {
    gene_benefit * env_energy_ratio > expression_cost
}

/// Fenotipo efectivo: producto del genotipo base por la máscara de expresión.
/// expression_level: [0,1] — 0 = silenciado, 1 = expresión completa.
pub fn effective_phenotype(genotype_val: f32, expression_level: f32) -> f32 {
    genotype_val * expression_level.clamp(0.0, 1.0)
}

/// Costo de silenciar un gen (reconfiguración metabólica).
pub fn silencing_cost(gene_complexity: f32, silencing_rate: f32) -> f32 {
    gene_complexity * silencing_rate
}

/// Velocidad de respuesta epigenética: EMA hacia el target (lag de adaptación).
pub fn epigenetic_lag(expression_current: f32, expression_target: f32, adaptation_speed: f32) -> f32 {
    expression_current + (expression_target - expression_current) * adaptation_speed
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_express_rich_environment() {
        assert!(should_express_gene(5.0, 1.0, 1.0));
    }

    #[test]
    fn should_not_express_poor_environment() {
        assert!(!should_express_gene(0.5, 1.0, 0.5));
    }

    #[test]
    fn effective_phenotype_full_expression() {
        assert!((effective_phenotype(10.0, 1.0) - 10.0).abs() < 1e-5);
    }

    #[test]
    fn effective_phenotype_silenced() {
        assert!((effective_phenotype(10.0, 0.0)).abs() < 1e-5);
    }

    #[test]
    fn effective_phenotype_partial() {
        assert!((effective_phenotype(10.0, 0.5) - 5.0).abs() < 1e-5);
    }

    #[test]
    fn epigenetic_lag_slow_convergence() {
        // 1.0 → 0.0 at speed 0.05 → 0.95
        assert!((epigenetic_lag(1.0, 0.0, 0.05) - 0.95).abs() < 1e-5);
    }

    #[test]
    fn epigenetic_lag_zero_speed_unchanged() {
        assert!((epigenetic_lag(0.5, 1.0, 0.0) - 0.5).abs() < 1e-5);
    }

    #[test]
    fn silencing_cost_proportional() {
        assert!((silencing_cost(2.0, 0.5) - 1.0).abs() < 1e-5);
    }
}
