//! ET-14: Institutions — ecuaciones puras. Sin deps de Bevy.

/// Estabilidad de una institución: compliance sostenida menos costos administrativos.
pub fn institution_stability(
    compliance_rate: f32,
    enforcement_efficiency: f32,
    admin_cost: f32,
) -> f32 {
    compliance_rate * enforcement_efficiency - admin_cost
}

/// Incentivo de cumplimiento: beneficio de cumplir > penalización esperada de defectar.
pub fn compliance_incentive(
    member_benefit: f32,
    defection_gain: f32,
    detection_probability: f32,
    penalty: f32,
) -> f32 {
    let expected_defection = defection_gain - detection_probability * penalty;
    member_benefit - expected_defection
}

/// Eficiencia del enforcement: qe recuperado de defectores vs. costo de detección.
pub fn enforcement_efficiency(penalty_collected: f32, enforcement_cost: f32) -> f32 {
    if enforcement_cost <= 0.0 {
        return 0.0;
    }
    (penalty_collected - enforcement_cost) / enforcement_cost
}

/// ROI de fundar una institución.
pub fn institution_roi(
    surplus_per_tick: f32,
    member_count: u16,
    admin_cost_per_tick: f32,
    founding_cost: f32,
    horizon_ticks: u32,
) -> f32 {
    let _ = member_count; // simplificado: surplus ya es colectivo
    let total_benefit = surplus_per_tick * horizon_ticks as f32;
    let total_cost = founding_cost + admin_cost_per_tick * horizon_ticks as f32;
    total_benefit - total_cost
}

/// Distribución de surplus institucional (proporcional a contribución).
pub fn allocation_share(own_contribution: f32, total_contributions: f32) -> f32 {
    if total_contributions <= 0.0 {
        return 0.0;
    }
    own_contribution / total_contributions
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn institution_stability_positive_when_compliant() {
        assert!(institution_stability(0.9, 10.0, 5.0) > 0.0);
    }

    #[test]
    fn institution_stability_negative_when_costly() {
        assert!(institution_stability(0.1, 1.0, 5.0) < 0.0);
    }

    #[test]
    fn compliance_incentive_positive_when_membership_better() {
        assert!(compliance_incentive(10.0, 5.0, 0.5, 8.0) > 0.0);
    }

    #[test]
    fn enforcement_efficiency_zero_when_no_cost() {
        assert_eq!(enforcement_efficiency(10.0, 0.0), 0.0);
    }

    #[test]
    fn institution_roi_positive_long_horizon() {
        assert!(institution_roi(10.0, 5, 1.0, 50.0, 100) > 0.0);
    }

    #[test]
    fn allocation_share_proportional() {
        assert!((allocation_share(25.0, 100.0) - 0.25).abs() < 1e-5);
    }

    #[test]
    fn allocation_share_zero_when_no_contributions() {
        assert_eq!(allocation_share(10.0, 0.0), 0.0);
    }
}
