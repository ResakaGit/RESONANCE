/// Intake del núcleo reducido por daño estructural. damage ∈ [0,1].
/// `intake_effective = intake_base × (1 − damage)`
pub fn nucleus_effective_intake(intake_base: f32, structural_damage: f32) -> f32 {
    (intake_base * (1.0 - structural_damage.clamp(0.0, 1.0))).max(0.0)
}

/// ¿El núcleo sigue siendo viable? false = victoria inminente.
pub fn is_nucleus_viable(qe: f32, qe_min: f32) -> bool {
    qe > qe_min
}

/// Potencial de comeback: inverso de masa inercial. Mayor = más reversible.
pub fn comeback_potential(mean_inertial_mass: f32) -> f32 {
    if mean_inertial_mass <= 0.0 {
        return 0.0;
    }
    1.0 / mean_inertial_mass
}

/// Ventaja energética: positivo = team_a va ganando.
pub fn energy_advantage(total_qe_a: f32, total_qe_b: f32) -> f32 {
    total_qe_a - total_qe_b
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::blueprint::constants::QE_NUCLEUS_VIABILITY_THRESHOLD;

    #[test]
    fn intake_no_damage() {
        assert!((nucleus_effective_intake(100.0, 0.0) - 100.0).abs() < 1e-5);
    }

    #[test]
    fn intake_half_damage() {
        assert!((nucleus_effective_intake(100.0, 0.5) - 50.0).abs() < 1e-5);
    }

    #[test]
    fn intake_full_damage() {
        assert!(nucleus_effective_intake(100.0, 1.0) < 1e-5);
    }

    #[test]
    fn viable_above_threshold() {
        assert!(is_nucleus_viable(101.0, 100.0));
    }

    #[test]
    fn not_viable_below() {
        assert!(!is_nucleus_viable(50.0, 100.0));
    }

    #[test]
    fn comeback_high_mass_low_potential() {
        assert!(comeback_potential(0.1) > comeback_potential(10.0));
    }

    #[test]
    fn comeback_zero_mass() {
        assert_eq!(comeback_potential(0.0), 0.0);
    }

    #[test]
    fn energy_advantage_positive_when_a_leads() {
        assert!(energy_advantage(500.0, 300.0) > 0.0);
    }

    #[test]
    fn nucleus_viability_threshold_constant_used() {
        assert!(!is_nucleus_viable(QE_NUCLEUS_VIABILITY_THRESHOLD - 1.0, QE_NUCLEUS_VIABILITY_THRESHOLD));
    }
}
