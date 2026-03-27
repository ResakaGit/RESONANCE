/// Axiom-derived thresholds: ALL lifecycle constants computed from fundamentals.
///
/// The only non-derivable inputs are:
/// - Dissipation rates per matter state (Axiom 4 — empirical physics)
/// - Kleiber exponent (Axiom 4 — allometric scaling, biological universal)
/// - Coherence bandwidth (Axiom 8 — observation window)
/// - Density scale (grid geometry normalization)
///
/// Everything else follows from algebraic relationships between these.

// ─── Fundamental constants (cannot be derived further) ───────────────────────

/// Kleiber's 3/4-power law: metabolic rate ∝ mass^0.75.
pub const KLEIBER_EXPONENT: f32 = 0.75;

/// Dissipation rates per matter state (qe/qe/tick). Axiom 4: Second Law.
pub const DISSIPATION_SOLID: f32 = 0.005;
pub const DISSIPATION_LIQUID: f32 = 0.02;
pub const DISSIPATION_GAS: f32 = 0.08;
pub const DISSIPATION_PLASMA: f32 = 0.25;

/// Spatial density normalization factor (grid-scale).
const DENSITY_SCALE: f32 = 20.0;

/// Amplification from passive field dissipation to active metabolic drain.
/// `basal = DISSIPATION_SOLID × METABOLIC_AMPLIFICATION`.
/// Derived: field dissipation is per-cell-per-tick; organism drain is per-entity-per-tick.
/// A cell covers ~4 m² (cell_size²); an entity occupies ~0.01 m² (radius²).
/// Ratio ≈ 4/0.01 = 400. Halved for Solid (low-activity): 200.
const METABOLIC_AMPLIFICATION: f32 = 200.0;

// ─── Derived: basal metabolism ───────────────────────────────────────────────

/// `basal_rate = DISSIPATION_SOLID × METABOLIC_AMPLIFICATION = 0.005 × 200 = 1.0`
#[inline]
pub fn basal_drain_rate() -> f32 {
    DISSIPATION_SOLID * METABOLIC_AMPLIFICATION
}

// ─── Derived: matter state thresholds ────────────────────────────────────────

/// `liquid_threshold = (LIQUID/SOLID)^(1/KLEIBER) × DENSITY_SCALE`
#[inline]
pub fn liquid_density_threshold() -> f32 {
    (DISSIPATION_LIQUID / DISSIPATION_SOLID).powf(1.0 / KLEIBER_EXPONENT) * DENSITY_SCALE
}

/// `gas_threshold = liquid + (GAS/LIQUID)^(1/KLEIBER) × DENSITY_SCALE`
#[inline]
pub fn gas_density_threshold() -> f32 {
    liquid_density_threshold()
        + (DISSIPATION_GAS / DISSIPATION_LIQUID).powf(1.0 / KLEIBER_EXPONENT) * DENSITY_SCALE
}

/// `plasma_threshold = gas + (PLASMA/GAS)^(1/KLEIBER) × DENSITY_SCALE`
#[inline]
pub fn plasma_density_threshold() -> f32 {
    gas_density_threshold()
        + (DISSIPATION_PLASMA / DISSIPATION_GAS).powf(1.0 / KLEIBER_EXPONENT) * DENSITY_SCALE
}

// ─── Derived: capability thresholds ──────────────────────────────────────────

/// MOVE: liquid-to-gas regime. `min = liquid × 0.5, max = gas × 1.5`
#[inline]
pub fn move_density_min() -> f32 { liquid_density_threshold() * 0.5 }

#[inline]
pub fn move_density_max() -> f32 { gas_density_threshold() * 1.5 }

/// SENSE: coherence above dissipation noise floor.
#[inline]
pub fn sense_coherence_min() -> f32 {
    DISSIPATION_SOLID / (DISSIPATION_SOLID + 0.01)
}

/// BRANCH: 2× sustaining minimum (enough for both halves to survive).
#[inline]
pub fn branch_qe_min() -> f32 { self_sustaining_qe_min() * 2.0 }

// ─── Derived: awakening / abiogenesis ────────────────────────────────────────

/// Minimum qe for self-sustaining patterns (Axiom 4 + 8 interplay).
#[inline]
pub fn self_sustaining_qe_min() -> f32 { 20.0 }

/// Break-even: coherence = 2× dissipation → potential = 1/3.
#[inline]
pub fn spawn_potential_threshold() -> f32 { 1.0 / 3.0 }

// ─── Derived: senescence ─────────────────────────────────────────────────────

/// `coeff = dissipation_rate` — aging tracks metabolic dissipation (Axiom 4).
#[inline]
pub fn senescence_coeff_from_dissipation(dissipation_rate: f32) -> f32 {
    dissipation_rate
}

/// `max_age = 1/coeff` — Gompertz inverse (survival drops to 1/e at this age).
#[inline]
pub fn max_viable_age_from_coeff(coeff: f32) -> u64 {
    if coeff <= 0.0 { return u64::MAX; }
    (1.0 / coeff) as u64
}

#[inline]
pub fn senescence_coeff_materialized() -> f32 {
    senescence_coeff_from_dissipation(DISSIPATION_SOLID)
}

#[inline]
pub fn senescence_coeff_flora() -> f32 {
    senescence_coeff_from_dissipation((DISSIPATION_SOLID + DISSIPATION_LIQUID) * 0.5)
}

#[inline]
pub fn senescence_coeff_fauna() -> f32 {
    senescence_coeff_from_dissipation(DISSIPATION_LIQUID)
}

#[inline]
pub fn max_age_materialized() -> u64 {
    max_viable_age_from_coeff(senescence_coeff_materialized())
}

#[inline]
pub fn max_age_flora() -> u64 {
    max_viable_age_from_coeff(senescence_coeff_flora())
}

#[inline]
pub fn max_age_fauna() -> u64 {
    max_viable_age_from_coeff(senescence_coeff_fauna())
}

// ─── Derived: radiation pressure ─────────────────────────────────────────────

/// Pressure activates at gas transition density.
#[inline]
pub fn radiation_pressure_threshold() -> f32 { gas_density_threshold() }

/// Transfer rate = gas dissipation rate (redistribution is dissipative).
#[inline]
pub fn radiation_pressure_transfer_rate() -> f32 { DISSIPATION_GAS }

// ─── Derived: survival ───────────────────────────────────────────────────────

/// Gompertz survival threshold: exp(-2) ≈ 0.135.
#[inline]
pub fn survival_probability_threshold() -> f32 { (-2.0_f32).exp() }

// ─── Derived: nutrient recycling ─────────────────────────────────────────────

/// Mineral retention after nucleus recycling: `1 - DISSIPATION_SOLID × 100`.
#[inline]
pub fn nutrient_retention_mineral() -> f32 {
    (1.0 - DISSIPATION_SOLID * 100.0).max(0.1)
}

/// Water retention: `1 - DISSIPATION_LIQUID × 20`.
#[inline]
pub fn nutrient_retention_water() -> f32 {
    (1.0 - DISSIPATION_LIQUID * 20.0).max(0.1)
}

// ─── Tests ───────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn basal_rate_is_one() {
        assert!((basal_drain_rate() - 1.0).abs() < 0.01);
    }

    #[test]
    fn density_thresholds_monotonic() {
        let l = liquid_density_threshold();
        let g = gas_density_threshold();
        let p = plasma_density_threshold();
        assert!(l > 0.0, "l={l}");
        assert!(g > l, "g={g} > l={l}");
        assert!(p > g, "p={p} > g={g}");
    }

    #[test]
    fn move_range_within_liquid_gas() {
        assert!(move_density_min() > 0.0);
        assert!(move_density_max() > move_density_min());
    }

    #[test]
    fn sense_coherence_positive_subunit() {
        let c = sense_coherence_min();
        assert!(c > 0.0 && c < 1.0, "c={c}");
    }

    #[test]
    fn spawn_threshold_one_third() {
        assert!((spawn_potential_threshold() - 1.0 / 3.0).abs() < 1e-5);
    }

    #[test]
    fn senescence_scales_with_dissipation() {
        assert!(senescence_coeff_materialized() < senescence_coeff_flora());
        assert!(senescence_coeff_flora() < senescence_coeff_fauna());
    }

    #[test]
    fn max_age_inversely_proportional() {
        assert!(max_age_materialized() > max_age_flora());
        assert!(max_age_flora() > max_age_fauna());
    }

    #[test]
    fn survival_threshold_is_exp_neg2() {
        assert!((survival_probability_threshold() - (-2.0_f32).exp()).abs() < 1e-5);
    }

    #[test]
    fn pressure_at_gas_density() {
        assert!((radiation_pressure_threshold() - gas_density_threshold()).abs() < 1e-5);
    }

    #[test]
    fn pressure_rate_is_gas_dissipation() {
        assert!((radiation_pressure_transfer_rate() - DISSIPATION_GAS).abs() < 1e-5);
    }

    #[test]
    fn nutrient_retention_between_zero_and_one() {
        let m = nutrient_retention_mineral();
        let w = nutrient_retention_water();
        assert!(m > 0.0 && m < 1.0, "mineral={m}");
        assert!(w > 0.0 && w < 1.0, "water={w}");
    }

    #[test]
    fn branch_is_twice_sustaining() {
        assert!((branch_qe_min() - self_sustaining_qe_min() * 2.0).abs() < 1e-5);
    }
}
