//! ET-8: Dynamic Coalitions — ecuaciones puras. Sin deps de Bevy.

pub const MAX_COALITION_BONUS: f32 = 2.5;
pub const MAX_COALITION_MEMBERS: u8 = 8;

/// Estabilidad Nash de la coalición: mínima ganancia individual de pertenecer.
pub fn coalition_stability(intake_with: &[f32], intake_without: &[f32]) -> f32 {
    intake_with.iter().zip(intake_without.iter())
        .map(|(w, wo)| w - wo)
        .fold(f32::MAX, f32::min)
}

/// Incentivo de deserción: ganancia neta de un miembro al irse a otra coalición.
pub fn defection_incentive(intake_current: f32, intake_alternative: f32, switching_cost: f32) -> f32 {
    (intake_alternative - intake_current - switching_cost).max(0.0)
}

/// Bonus de intake por tamaño de coalición (economía de escala, satura logarítmicamente).
pub fn coalition_intake_bonus(base_intake: f32, member_count: u8, scale_factor: f32) -> f32 {
    let scale = (1.0 + (member_count as f32).ln().max(0.0) * scale_factor).min(MAX_COALITION_BONUS);
    base_intake * scale
}

/// Tamaño óptimo de coalición dado el costo de coordinación por miembro.
pub fn optimal_coalition_size(marginal_benefit_per_member: f32, coordination_cost_per_member: f32) -> u8 {
    if coordination_cost_per_member <= 0.0 { return MAX_COALITION_MEMBERS; }
    let optimal = (marginal_benefit_per_member / coordination_cost_per_member).ceil() as u8;
    optimal.clamp(2, MAX_COALITION_MEMBERS)
}

/// Distribución equitativa de beneficio colectivo (Shapley simplificado: 1/n).
pub fn shapley_share(total_benefit: f32, member_count: u8) -> f32 {
    if member_count == 0 { return 0.0; }
    total_benefit / member_count as f32
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn coalition_stability_positive_when_all_gain() {
        assert!((coalition_stability(&[10.0, 10.0], &[5.0, 5.0]) - 5.0).abs() < 1e-5);
    }

    #[test]
    fn coalition_stability_negative_when_one_loses() {
        assert!((coalition_stability(&[10.0, 3.0], &[5.0, 5.0]) - (-2.0)).abs() < 1e-5);
    }

    #[test]
    fn defection_incentive_positive_when_better_outside() {
        assert!((defection_incentive(10.0, 15.0, 2.0) - 3.0).abs() < 1e-5);
    }

    #[test]
    fn defection_incentive_zero_when_not_better() {
        assert_eq!(defection_incentive(10.0, 8.0, 1.0), 0.0);
    }

    #[test]
    fn coalition_intake_bonus_one_member_no_ln_bonus() {
        // ln(1) = 0 → scale = 1.0 → no bonus
        assert!((coalition_intake_bonus(100.0, 1, 0.15) - 100.0).abs() < 1e-3);
    }

    #[test]
    fn coalition_intake_bonus_capped_at_max() {
        assert!(coalition_intake_bonus(100.0, u8::MAX, 10.0) <= 100.0 * MAX_COALITION_BONUS);
    }

    #[test]
    fn optimal_coalition_size_reasonable() {
        let size = optimal_coalition_size(5.0, 1.0);
        assert!(size >= 2 && size <= MAX_COALITION_MEMBERS);
    }

    #[test]
    fn shapley_share_equal_distribution() {
        assert!((shapley_share(100.0, 4) - 25.0).abs() < 1e-5);
    }

    #[test]
    fn shapley_share_zero_members() {
        assert_eq!(shapley_share(100.0, 0), 0.0);
    }
}
