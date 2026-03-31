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

/// Tasas de disipación por estado de materia (qe/qe/tick). Axioma 4: Segunda Ley.
/// Dissipation rates per matter state (qe/qe/tick). Axiom 4: Second Law.
///
/// Ratios 1:4:16:50 — molecular mobility increases ~4× per phase transition
/// (solid→liquid→gas), consistent with self-diffusion coefficient scaling in
/// condensed matter (e.g., D_liquid/D_solid ≈ 10³ in metals, but here normalized
/// to simulation timescale). Plasma ratio elevated (50×) for unbound high-energy state.
/// These are physically motivated calibration values, not measured from a specific system.
pub const DISSIPATION_SOLID: f32 = 0.005;
pub const DISSIPATION_LIQUID: f32 = 0.02;
pub const DISSIPATION_GAS: f32 = 0.08;
pub const DISSIPATION_PLASMA: f32 = 0.25;

/// Spatial density normalization factor (grid-scale). Fundamental.
pub const DENSITY_SCALE: f32 = 20.0;

/// Ancho de banda de coherencia (Hz). Axiom 8: ventana de observación para interferencia de frecuencia.
/// Coherence bandwidth (Hz). Axiom 8: observation window for frequency interference.
pub const COHERENCE_BANDWIDTH: f32 = 50.0;

/// Amplification from passive field dissipation to active metabolic drain.
/// `amplification = 1 / DISSIPATION_SOLID` — the inverse of base dissipation.
/// At DISSIPATION_SOLID = 0.005: amplification = 200.
/// Derivation: field operates at dissipation scale; organisms at inverse scale.
/// `basal_rate = DISSIPATION_SOLID × (1/DISSIPATION_SOLID) = 1.0 qe/tick` — unit rate.

// ─── Derived: basal metabolism ───────────────────────────────────────────────

/// `basal_rate = DISSIPATION_SOLID × (1/DISSIPATION_SOLID) = 1.0 qe/tick`
#[inline]
pub fn basal_drain_rate() -> f32 {
    DISSIPATION_SOLID * (1.0 / DISSIPATION_SOLID)
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
/// `min = DISSIPATION_SOLID / (DISSIPATION_SOLID + DISSIPATION_LIQUID)`
/// Signal must exceed the noise floor of the next dissipation regime.
#[inline]
pub fn sense_coherence_min() -> f32 {
    DISSIPATION_SOLID / (DISSIPATION_SOLID + DISSIPATION_LIQUID)
}

/// BRANCH: 2× sustaining minimum (enough for both halves to survive).
#[inline]
pub fn branch_qe_min() -> f32 { self_sustaining_qe_min() * 2.0 }

// ─── Cosmological anchor (single calibration constant) ──────────────────────

// Runtime-tunable cosmological anchor. Default = DENSITY_SCALE (derived, not arbitrary).
// Bevy-coupled by design: allows calibration experiments without recompilation.
use bevy::prelude::Resource;

/// Ancla cosmológica: mínima qe para patrones auto-sustentables.
/// Cosmological anchor: minimum qe for self-sustaining patterns.
///
/// Default = DENSITY_SCALE (20.0). Derived: 1 normalized density unit.
/// Injectable as Bevy Resource for runtime calibration experiments.
/// All lifecycle thresholds scale from this single value.
#[derive(Resource, Debug, Clone, Copy)]
pub struct SelfSustainingQeMin(pub f32);

impl Default for SelfSustainingQeMin {
    fn default() -> Self { Self(20.0) }
}

// ─── Derived: awakening / abiogenesis ────────────────────────────────────────

/// Mínima qe para patrones auto-sustentables: 1 unidad de densidad normalizada.
/// Minimum qe for self-sustaining patterns: 1 normalized density unit.
///
/// Derived: `self_sustaining_qe_min = DENSITY_SCALE` — the minimum energy
/// that fills one grid cell at unit density. Below this, no stable structure.
/// Runtime-tunable via `SelfSustainingQeMin` Resource (default = DENSITY_SCALE).
#[inline]
pub fn self_sustaining_qe_min() -> f32 { SelfSustainingQeMin::default().0 }

/// Umbral de spawn derivado del Axiom 2 (Pool Invariant).
/// Spawn threshold derived from Axiom 2 (Pool Invariant).
///
/// At threshold: parent retains `min`, child receives `min`, so `qe = 2×min`.
/// Net surplus = `min`. `potential = min / (min + 2×min) = 1/3`.
/// A cell needs 3× the sustaining minimum (2 to keep, 1 surplus) to spawn.
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

/// Mineral retention after nucleus recycling.
/// `retention = 1 - DISSIPATION_SOLID / DISSIPATION_LIQUID`
/// Minerals (C/N/P) resist conversion proportional to Solid/Liquid dissipation ratio.
#[inline]
pub fn nutrient_retention_mineral() -> f32 {
    (1.0 - DISSIPATION_SOLID / DISSIPATION_LIQUID).max(0.1)
}

/// Water retention: more volatile, dissipates faster.
/// `retention = 1 - DISSIPATION_LIQUID / DISSIPATION_GAS`
#[inline]
pub fn nutrient_retention_water() -> f32 {
    (1.0 - DISSIPATION_LIQUID / DISSIPATION_GAS).max(0.1)
}

/// Nutrient threshold for recycling trigger.
/// Sum of mineral and water conversion losses.
/// `threshold = (1 - mineral_ret) + (1 - water_ret)`
#[inline]
pub fn recycling_nutrient_threshold() -> f32 {
    (1.0 - nutrient_retention_mineral()) + (1.0 - nutrient_retention_water())
}

/// Conversion efficiency when draining grid energy to nucleus reservoir.
/// `efficiency = 1 - DISSIPATION_SOLID` — Second Law cost (Axiom 4).
#[inline]
pub fn recycling_conversion_efficiency() -> f32 {
    1.0 - DISSIPATION_SOLID
}

/// Harvest radius in cells for energy drain during recycling.
/// `radius = sqrt(DENSITY_SCALE)` — spatial extent of the drain zone.
#[inline]
pub fn recycling_harvest_radius_cells() -> u32 {
    DENSITY_SCALE.sqrt() as u32
}

/// Fraction of each cell's qe drained during recycling.
/// Same as mineral consumed fraction: `1 - mineral_retention`.
#[inline]
pub fn recycling_drain_fraction() -> f32 {
    1.0 - nutrient_retention_mineral()
}

/// Recycled nucleus emission rate from reservoir size.
/// `emission = reservoir × DISSIPATION_GAS` — gas-state energy release.
#[inline]
pub fn recycled_emission_rate(reservoir_qe: f32) -> f32 {
    reservoir_qe.max(0.0) * DISSIPATION_GAS
}

/// Recycled nucleus propagation radius from reservoir size.
/// `radius = sqrt(reservoir / DENSITY_SCALE)` — spatial extent from energy.
#[inline]
pub fn recycled_propagation_radius(reservoir_qe: f32) -> f32 {
    (reservoir_qe.max(0.0) / DENSITY_SCALE).sqrt().max(2.0)
}

// ─── Derived: nutrient cycle rates ──────────────────────────────────────────

/// Nutrient depletion rate (entity consumption per tick).
/// `rate = DISSIPATION_LIQUID / DENSITY_SCALE` — liquid-state mobility at grid scale.
/// Entities extract nutrients at the rate liquid energy diffuses through the grid.
#[inline]
pub fn nutrient_depletion_rate() -> f32 {
    DISSIPATION_LIQUID / DENSITY_SCALE
}

/// Nutrient return rate on entity death.
/// `rate = depletion × mineral_retention` — Second Law: not all nutrients survive.
/// Return < depletion ensures net nutrient loss per lifecycle (Axiom 4).
#[inline]
pub fn nutrient_return_rate() -> f32 {
    nutrient_depletion_rate() * nutrient_retention_mineral()
}

/// Natural nutrient regeneration per tick (geological weathering).
/// `rate = DISSIPATION_SOLID × DISSIPATION_LIQUID` — solid dissolving into liquid.
/// Much slower than biological depletion (product of two small rates).
#[inline]
pub fn nutrient_regen_per_tick() -> f32 {
    DISSIPATION_SOLID * DISSIPATION_LIQUID
}

// ─── Derived: worldgen field constants ────────────────────────────────────────

/// Minimum qe in a cell to materialize an entity = self_sustaining_qe_min / 2.
/// Half the sustaining minimum — enough to be visible but not necessarily alive.
#[inline]
pub fn min_materialization_qe() -> f32 { self_sustaining_qe_min() * 0.5 }

/// Field decay rate (qe/s per cell). = basal_drain_rate (same scale as entity metabolism).
#[inline]
pub fn field_decay_rate() -> f32 { basal_drain_rate() }

/// Reference density for visual derivation = liquid threshold.
#[inline]
pub fn reference_density() -> f32 { liquid_density_threshold() }

/// Low density class boundary = DENSITY_SCALE (the fundamental grid unit).
#[inline]
pub fn density_low_threshold() -> f32 { DENSITY_SCALE }

/// High density class boundary = liquid_density_threshold.
#[inline]
pub fn density_high_threshold() -> f32 { liquid_density_threshold() }

/// Purity threshold for pure vs compound materialization.
/// Derived: at sense_coherence_min × 2, signal is clearly dominant.
#[inline]
pub fn purity_threshold() -> f32 {
    (sense_coherence_min() * 2.0).min(0.95)
}

/// Field conductivity spread between neighbors.
/// = DISSIPATION_LIQUID (lateral diffusion scales with liquid-state mobility).
#[inline]
pub fn field_conductivity_spread() -> f32 { DISSIPATION_LIQUID }

/// Bond energy for materialized spawns = 1/DISSIPATION_SOLID (inverse of base decay).
#[inline]
pub fn materialized_bond_energy() -> f32 { 1.0 / DISSIPATION_SOLID }

/// Thermal conductivity for materialized spawns = DISSIPATION_SOLID × DENSITY_SCALE.
#[inline]
pub fn materialized_thermal_conductivity() -> f32 { DISSIPATION_SOLID * DENSITY_SCALE }

/// Collider radius factor = 0.5 (half of cell — geometric).
pub const MATERIALIZED_COLLIDER_RADIUS_FACTOR: f32 = 0.5;

/// Minimum collider radius = DISSIPATION_SOLID (smallest stable structure scale).
#[inline]
pub fn materialized_min_collider_radius() -> f32 { DISSIPATION_SOLID }

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

    #[test]
    fn recycling_threshold_equals_conversion_losses() {
        let expected = (1.0 - nutrient_retention_mineral()) + (1.0 - nutrient_retention_water());
        assert!((recycling_nutrient_threshold() - expected).abs() < 1e-6);
    }

    #[test]
    fn recycling_conversion_under_one() {
        let e = recycling_conversion_efficiency();
        assert!(e > 0.9 && e < 1.0, "efficiency={e}");
    }

    #[test]
    fn recycled_emission_scales_with_reservoir() {
        let small = recycled_emission_rate(100.0);
        let large = recycled_emission_rate(1000.0);
        assert!(large > small, "{large} > {small}");
        assert!((small - 100.0 * DISSIPATION_GAS).abs() < 1e-5);
    }

    #[test]
    fn recycled_radius_scales_with_reservoir() {
        let r = recycled_propagation_radius(500.0);
        assert!(r >= 2.0, "radius={r}");
        assert!(r < 20.0, "radius={r}");
    }

    #[test]
    fn recycling_harvest_radius_positive() {
        assert!(recycling_harvest_radius_cells() >= 2);
    }
}
