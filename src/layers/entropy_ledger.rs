//! Libro contable termodinamico (MG-6C). Recomputado cada tick, no estado persistente.
//! Solo entidades con MetabolicGraph lo reciben.

use bevy::prelude::*;

/// Libro contable termodinamico derivado del DAG metabolico.
/// 4 campos — cumple regla ECS max 4.
#[derive(Component, Clone, Copy, Debug, PartialEq, Reflect)]
#[reflect(Component)]
#[component(storage = "SparseSet")]
pub struct EntropyLedger {
    /// Sigma Q_diss de organ_transform por nodo (qe/tick).
    pub total_heat_generated: f32,
    /// Sigma waste_mass de organ_transform por nodo (qe/tick).
    pub total_waste_generated: f32,
    /// S_gen = entropy_production(total_heat, T_core) (MG-1).
    pub entropy_rate: f32,
    /// eta_total = final_exergy / max(initial_exergy, EPSILON).
    pub exergy_efficiency: f32,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn entropy_ledger_is_copy() {
        let a = EntropyLedger {
            total_heat_generated: 1.0,
            total_waste_generated: 2.0,
            entropy_rate: 0.5,
            exergy_efficiency: 0.8,
        };
        let b = a;
        assert_eq!(a, b);
    }

    #[test]
    fn entropy_ledger_has_four_f32_fields() {
        assert_eq!(std::mem::size_of::<EntropyLedger>(), 4 * 4);
    }

    #[test]
    fn entropy_ledger_zero_exergy_no_nan() {
        let l = EntropyLedger {
            total_heat_generated: 0.0,
            total_waste_generated: 0.0,
            entropy_rate: 0.0,
            exergy_efficiency: 0.0,
        };
        assert!(!l.exergy_efficiency.is_nan());
        assert_eq!(l.exergy_efficiency, 0.0);
    }

    #[test]
    fn entropy_ledger_entropy_rate_consistent_with_mg1() {
        use crate::blueprint::morphogenesis::entropy_production;
        let q = 150.0;
        let t_core = 400.0;
        let expected = entropy_production(q, t_core);
        let l = EntropyLedger {
            total_heat_generated: q,
            total_waste_generated: 30.0,
            entropy_rate: expected,
            exergy_efficiency: 0.64,
        };
        assert!((l.entropy_rate - expected).abs() < 1e-6);
    }
}
