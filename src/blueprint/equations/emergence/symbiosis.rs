//! ET-5 + AC-5: Obligate Symbiosis + Cooperation Emergence — ecuaciones puras.
//!
//! AC-5 adds Nash-equilibrium cooperation evaluation (Axiom 3 game theory):
//! entities form alliances when the Nash stable condition is met AND the
//! expected gain exceeds the defection threshold.

/// Beneficio de la simbiosis mutualista: qe/tick adicional con partner presente.
pub fn mutualism_benefit(own_intake: f32, partner_bonus_factor: f32) -> f32 {
    own_intake * partner_bonus_factor
}

/// Costo del parasitismo: qe/tick extraído del host por el parásito.
pub fn parasitism_drain(host_qe: f32, drain_rate: f32) -> f32 {
    host_qe * drain_rate
}

/// ¿La dependencia es obligada? Sí si el intake sin partner cae bajo la disipación base.
pub fn is_obligate_dependency(intake_without_partner: f32, base_dissipation: f32) -> bool {
    intake_without_partner < base_dissipation
}

/// Estabilidad Nash de la simbiosis: ninguna parte gana más rompiendo la relación.
pub fn is_symbiosis_stable(
    a_with_b: f32,
    a_without_b: f32,
    b_with_a: f32,
    b_without_a: f32,
) -> bool {
    a_with_b >= a_without_b && b_with_a >= b_without_a
}

/// Presión de coevolución: cuánto presiona B a A a adaptarse.
pub fn coevolution_pressure(extraction_b_on_a: f32, resistance_a: f32) -> f32 {
    (extraction_b_on_a - resistance_a).max(0.0)
}

// ── AC-5: Cooperation Emergence (Axiom 3 game theory) ────────────────────────

/// Estimated extraction rate for an entity within a group (cooperation payoff).
///
/// Group members share resource access — each member's effective extraction
/// is the group-pooled access scaled by their individual rate.
pub fn extraction_estimate_in_group(
    individual_rate: f32,
    group_size: f32,
    group_bonus: f32,
) -> f32 {
    let effective_size = group_size.max(1.0);
    individual_rate * (1.0 + group_bonus / effective_size)
}

/// Estimated extraction rate for a solo entity (defection payoff).
///
/// Invariant: always returns `individual_rate` — baseline payoff without group.
#[inline]
pub fn extraction_estimate_solo(individual_rate: f32) -> f32 {
    individual_rate
}

/// Whether forming/maintaining a cooperation is a Nash-stable decision.
///
/// Cooperation is beneficial when:
/// 1. Nash condition: both parties gain more together than alone (`is_symbiosis_stable`)
/// 2. Cooperation surplus exceeds the interference cost of being in proximity
///    (entities that share space must share resources → some crowding penalty).
///
/// `interference_cost`: reduction from AC-1 cross-frequency interference at
///   group proximity (pre-computed from `metabolic_interference_factor`).
pub fn cooperation_is_beneficial(
    a_rate_solo: f32,
    a_rate_in_group: f32,
    b_rate_solo: f32,
    b_rate_in_group: f32,
    interference_cost: f32,
) -> bool {
    let surplus_a = a_rate_in_group - a_rate_solo - interference_cost.max(0.0);
    let surplus_b = b_rate_in_group - b_rate_solo - interference_cost.max(0.0);
    surplus_a > 0.0 && surplus_b > 0.0
}

/// Defection temptation: how much entity A could gain by defecting from a group.
///
/// Positive → defection is tempting; negative → staying is clearly better.
pub fn defection_temptation(a_rate_solo: f32, a_rate_in_group: f32) -> f32 {
    a_rate_solo - a_rate_in_group
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn is_symbiosis_stable_mutual_benefit() {
        assert!(is_symbiosis_stable(10.0, 5.0, 10.0, 5.0));
    }

    #[test]
    fn is_symbiosis_stable_one_side_loses() {
        assert!(!is_symbiosis_stable(5.0, 10.0, 10.0, 5.0));
    }

    #[test]
    fn is_obligate_dependency_true_when_intake_low() {
        assert!(is_obligate_dependency(2.0, 5.0));
    }

    #[test]
    fn is_obligate_dependency_false_when_self_sufficient() {
        assert!(!is_obligate_dependency(10.0, 5.0));
    }

    #[test]
    fn mutualism_benefit_proportional() {
        assert!((mutualism_benefit(100.0, 0.2) - 20.0).abs() < 1e-5);
    }

    #[test]
    fn parasitism_drain_proportional() {
        assert!((parasitism_drain(200.0, 0.05) - 10.0).abs() < 1e-5);
    }

    #[test]
    fn coevolution_pressure_zero_when_resistant() {
        assert_eq!(coevolution_pressure(3.0, 5.0), 0.0);
    }

    #[test]
    fn coevolution_pressure_positive_when_vulnerable() {
        assert!(coevolution_pressure(10.0, 3.0) > 0.0);
    }

    // ── AC-5: cooperation equations ──────────────────────────────────────────

    #[test]
    fn extraction_in_group_exceeds_solo_with_positive_bonus() {
        let solo = extraction_estimate_solo(10.0);
        let group = extraction_estimate_in_group(10.0, 3.0, 6.0);
        assert!(group > solo, "solo={solo} group={group}");
    }

    #[test]
    fn extraction_solo_returns_individual_rate() {
        assert!((extraction_estimate_solo(7.5) - 7.5).abs() < 1e-5);
    }

    #[test]
    fn extraction_in_group_single_member_is_full_bonus() {
        // group_size=1 → bonus / 1 = full bonus
        let r = extraction_estimate_in_group(10.0, 1.0, 2.0);
        assert!((r - 30.0).abs() < 1e-5, "got {r}"); // 10 × (1 + 2/1) = 30
    }

    #[test]
    fn cooperation_beneficial_when_mutual_gain_exceeds_cost() {
        // Both gain from group, small interference cost
        assert!(cooperation_is_beneficial(5.0, 8.0, 5.0, 8.0, 0.5));
    }

    #[test]
    fn cooperation_not_beneficial_when_interference_too_high() {
        // Group gain = 3, but interference cost = 4 → net loss
        assert!(!cooperation_is_beneficial(5.0, 8.0, 5.0, 8.0, 4.0));
    }

    #[test]
    fn cooperation_not_beneficial_when_one_side_loses() {
        // A gains, B loses
        assert!(!cooperation_is_beneficial(5.0, 8.0, 10.0, 7.0, 0.0));
    }

    #[test]
    fn defection_temptation_positive_when_solo_better() {
        assert!(defection_temptation(10.0, 6.0) > 0.0);
    }

    #[test]
    fn defection_temptation_negative_when_group_better() {
        assert!(defection_temptation(6.0, 10.0) < 0.0);
    }

    #[test]
    fn defection_temptation_zero_when_equal() {
        assert!((defection_temptation(8.0, 8.0)).abs() < 1e-5);
    }
}
