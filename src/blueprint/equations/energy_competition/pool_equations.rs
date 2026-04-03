use super::super::finite_helpers::{finite_non_negative, finite_unit};
use crate::blueprint::constants::*;

// ─── EC-1A: Conservación de Pool ────────────────────────────────────────────

/// `pool(t+1) = (pool + intake − extracted − pool × rate).max(0)`.
/// `total_extracted` clampeado a `pool + intake`; `dissipation_rate` a rango válido.
#[inline]
pub fn pool_next_tick(pool: f32, intake: f32, total_extracted: f32, dissipation_rate: f32) -> f32 {
    let p = finite_non_negative(pool);
    let i = finite_non_negative(intake);
    let rate = dissipation_rate.clamp(DISSIPATION_RATE_MIN, DISSIPATION_RATE_MAX);
    let cap = p + i;
    let extr = total_extracted.clamp(0.0, cap);
    (p + i - extr - p * rate).max(0.0)
}

/// `loss = pool × rate`. Segunda ley: siempre positiva cuando pool > 0.
#[inline]
pub fn dissipation_loss(pool: f32, dissipation_rate: f32) -> f32 {
    let p = finite_non_negative(pool);
    let rate = dissipation_rate.clamp(DISSIPATION_RATE_MIN, DISSIPATION_RATE_MAX);
    p * rate
}

/// `available = (pool + intake − loss).max(0)`. Lo que queda tras disipación obligatoria.
#[inline]
pub fn available_for_extraction(pool: f32, intake: f32, dissipation_rate: f32) -> f32 {
    let p = finite_non_negative(pool);
    let i = finite_non_negative(intake);
    (p + i - dissipation_loss(pool, dissipation_rate)).max(0.0)
}

// ─── EC-1B: Funciones de Extracción (5 Tipos) ────────────────────────────────

/// Type I — Fair Share: `available / max(n_siblings, 1)`.
#[inline]
pub fn extract_proportional(available: f32, n_siblings: u32) -> f32 {
    let a = finite_non_negative(available);
    let n = n_siblings.max(1) as f32;
    a / n
}

/// Type II — Capacity-Bounded: `min(available, capacity.max(0))`.
#[inline]
pub fn extract_greedy(available: f32, capacity: f32) -> f32 {
    let a = finite_non_negative(available);
    let c = finite_non_negative(capacity);
    a.min(c)
}

/// Type III — Relative Fitness: `available × fitness / max(total_fitness, ε)`.
/// Guard: `total_fitness ≤ 0` → 0; `fitness < 0` → clamped 0.
#[inline]
pub fn extract_competitive(available: f32, fitness: f32, total_fitness: f32) -> f32 {
    let a = finite_non_negative(available);
    let f = fitness.max(0.0);
    let tf = total_fitness;
    if !tf.is_finite() || tf <= EXTRACTION_EPSILON {
        return 0.0;
    }
    (a * f / tf).max(0.0)
}

/// Type IV — Pool-Damaging: `(taken, pool_damage)`.
/// `taken = available × aggression_factor.clamp(0,1)`.
/// `pool_damage = taken × damage_rate.clamp(0,1)` — reduce capacidad del padre.
#[inline]
pub fn extract_aggressive(available: f32, aggression_factor: f32, damage_rate: f32) -> (f32, f32) {
    let a = finite_non_negative(available);
    let agg = aggression_factor.clamp(0.0, 1.0);
    let dmg = damage_rate.clamp(0.0, 1.0);
    let taken = a * agg;
    (taken, taken * dmg)
}

/// Type V — Homeostatic: tasa modulada por `pool_ratio` relativa a umbrales.
/// `pool_ratio > hi → base_rate × REGULATED_AGGRESSIVE_MULT`.
/// `pool_ratio in [lo, hi] → base_rate`.
/// `pool_ratio < lo → base_rate × REGULATED_THROTTLE_MULT`.
#[inline]
pub fn extract_regulated(
    _available: f32,
    pool_ratio: f32,
    base_rate: f32,
    threshold_low: f32,
    threshold_high: f32,
) -> f32 {
    let ratio = finite_unit(pool_ratio);
    let rate = base_rate.max(0.0);
    let lo = threshold_low.clamp(0.0, 1.0);
    let hi = threshold_high.clamp(0.0, 1.0);
    if ratio > hi {
        rate * REGULATED_AGGRESSIVE_MULT
    } else if ratio < lo {
        rate * REGULATED_THROTTLE_MULT
    } else {
        rate
    }
}

// ─── EC-1C: Fitness y Scaling ─────────────────────────────────────────────────

/// Ratio de fitness relativo al total de hermanos. Fallback proporcional si todos son 0.
#[inline]
pub fn relative_fitness(fitness: f32, sibling_fitnesses: &[f32]) -> f32 {
    let total: f32 = sibling_fitnesses.iter().copied().sum();
    if total <= EXTRACTION_EPSILON {
        let n = sibling_fitnesses.len().max(1) as f32;
        return 1.0 / n;
    }
    (fitness / total).clamp(0.0, 1.0)
}

/// Escala in-place `extractions` para que `Σ ≤ available`. No-op si la suma ya cabe.
/// Invariante post: `Σextractions ≤ available + POOL_CONSERVATION_EPSILON`.
pub fn scale_extractions_to_available(extractions: &mut [f32], available: f32) {
    let sum: f32 = extractions.iter().copied().sum();
    if sum <= available + POOL_CONSERVATION_EPSILON {
        return;
    }
    let factor = available / sum.max(EXTRACTION_EPSILON);
    for v in extractions.iter_mut() {
        *v *= factor;
    }
}

// ─── EC-1D: Condiciones de Estado ────────────────────────────────────────────

/// El pool no cambia este tick: `|intake − extracted − loss| < epsilon`.
#[inline]
pub fn is_pool_equilibrium(intake: f32, total_extracted: f32, loss: f32, epsilon: f32) -> bool {
    (intake - total_extracted - loss).abs() < epsilon
}

/// El pool se vaciará este tick: `extracted + loss > intake + pool`.
#[inline]
pub fn is_host_collapsing(pool: f32, intake: f32, total_extracted: f32, loss: f32) -> bool {
    total_extracted + loss > intake + pool
}

/// Ticks restantes hasta colapso a tasa constante. `u32::MAX` si no colapsa.
#[inline]
pub fn ticks_to_collapse(pool: f32, net_drain_per_tick: f32) -> u32 {
    if net_drain_per_tick <= 0.0 {
        return u32::MAX;
    }
    let p = finite_non_negative(pool);
    (p / net_drain_per_tick).ceil() as u32
}

// ─── Tests ───────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    // EC-1A
    #[test]
    fn pool_next_tick_standard_case() {
        let result = pool_next_tick(1000.0, 50.0, 200.0, 0.01);
        assert!((result - 840.0).abs() < 1e-3, "got {result}");
    }

    #[test]
    fn pool_next_tick_extracted_clamped_to_available() {
        let result = pool_next_tick(100.0, 0.0, 200.0, 0.01);
        assert!(result >= 0.0, "no negatives: {result}");
    }

    #[test]
    fn pool_next_tick_never_negative() {
        let result = pool_next_tick(0.0, 0.0, 9999.0, 0.5);
        assert_eq!(result, 0.0);
    }

    #[test]
    fn dissipation_loss_standard() {
        assert!((dissipation_loss(1000.0, 0.01) - 10.0).abs() < 1e-5);
    }

    #[test]
    fn dissipation_loss_zero_pool() {
        assert_eq!(dissipation_loss(0.0, 0.01), 0.0);
    }

    #[test]
    fn dissipation_loss_rate_clamped_to_min() {
        let result = dissipation_loss(1000.0, 0.0);
        assert!((result - 1000.0 * DISSIPATION_RATE_MIN).abs() < 1e-5);
    }

    #[test]
    fn available_for_extraction_standard() {
        let result = available_for_extraction(1000.0, 50.0, 0.01);
        assert!((result - 1040.0).abs() < 1e-3, "got {result}");
    }

    // EC-1B
    #[test]
    fn extract_proportional_four_siblings() {
        assert!((extract_proportional(1000.0, 4) - 250.0).abs() < 1e-5);
    }

    #[test]
    fn extract_proportional_zero_siblings_returns_all() {
        assert!((extract_proportional(1000.0, 0) - 1000.0).abs() < 1e-5);
    }

    #[test]
    fn extract_greedy_below_capacity() {
        assert!((extract_greedy(1000.0, 500.0) - 500.0).abs() < 1e-5);
    }

    #[test]
    fn extract_greedy_above_capacity_clamped() {
        assert!((extract_greedy(1000.0, 2000.0) - 1000.0).abs() < 1e-5);
    }

    #[test]
    fn extract_competitive_standard() {
        assert!((extract_competitive(1000.0, 0.6, 1.0) - 600.0).abs() < 1e-3);
    }

    #[test]
    fn extract_competitive_zero_fitness() {
        assert_eq!(extract_competitive(1000.0, 0.0, 1.0), 0.0);
    }

    #[test]
    fn extract_competitive_zero_total_fitness() {
        assert_eq!(extract_competitive(1000.0, 0.5, 0.0), 0.0);
    }

    #[test]
    fn extract_aggressive_standard() {
        let (taken, dmg) = extract_aggressive(1000.0, 0.5, 0.1);
        assert!((taken - 500.0).abs() < 1e-5);
        assert!((dmg - 50.0).abs() < 1e-5);
    }

    #[test]
    fn extract_aggressive_zero_aggression() {
        let (taken, dmg) = extract_aggressive(1000.0, 0.0, 0.1);
        assert_eq!(taken, 0.0);
        assert_eq!(dmg, 0.0);
    }

    #[test]
    fn extract_regulated_aggressive_zone() {
        assert!((extract_regulated(1000.0, 0.8, 100.0, 0.3, 0.7) - 150.0).abs() < 1e-5);
    }

    #[test]
    fn extract_regulated_normal_zone() {
        assert!((extract_regulated(1000.0, 0.5, 100.0, 0.3, 0.7) - 100.0).abs() < 1e-5);
    }

    #[test]
    fn extract_regulated_throttle_zone() {
        assert!((extract_regulated(1000.0, 0.1, 100.0, 0.3, 0.7) - 30.0).abs() < 1e-5);
    }

    // EC-1C
    #[test]
    fn relative_fitness_standard() {
        let result = relative_fitness(0.6, &[0.6, 0.3, 0.1]);
        assert!((result - 0.6).abs() < 1e-5);
    }

    #[test]
    fn relative_fitness_all_zero_fallback() {
        let result = relative_fitness(0.0, &[0.0, 0.0]);
        assert!((result - 0.5).abs() < 1e-5);
    }

    #[test]
    fn scale_extractions_scales_down() {
        let mut v = [600.0, 300.0, 100.0];
        scale_extractions_to_available(&mut v, 500.0);
        assert!((v[0] - 300.0).abs() < 1e-3);
        assert!((v[1] - 150.0).abs() < 1e-3);
        assert!((v[2] - 50.0).abs() < 1e-3);
    }

    #[test]
    fn scale_extractions_noop_when_sum_below_available() {
        let mut v = [100.0_f32, 100.0];
        scale_extractions_to_available(&mut v, 500.0);
        assert!((v[0] - 100.0).abs() < 1e-5);
        assert!((v[1] - 100.0).abs() < 1e-5);
    }

    #[test]
    fn scale_extractions_invariant_post_scaling() {
        let cases: &[(&[f32], f32)] = &[
            (&[300.0, 400.0, 500.0], 600.0),
            (&[1000.0, 2000.0], 100.0),
            (&[50.0, 50.0, 50.0, 50.0], 150.0),
        ];
        for (init, avail) in cases {
            let mut v: Vec<f32> = init.to_vec();
            scale_extractions_to_available(&mut v, *avail);
            let sum: f32 = v.iter().sum();
            assert!(
                sum <= avail + POOL_CONSERVATION_EPSILON,
                "sum={sum} avail={avail}"
            );
        }
    }

    // EC-1D
    #[test]
    fn is_pool_equilibrium_balanced() {
        assert!(is_pool_equilibrium(100.0, 90.0, 10.0, 1e-3));
    }

    #[test]
    fn is_pool_equilibrium_not_balanced() {
        assert!(!is_pool_equilibrium(100.0, 50.0, 10.0, 1e-3));
    }

    #[test]
    fn is_host_collapsing_true() {
        assert!(is_host_collapsing(100.0, 50.0, 200.0, 10.0));
    }

    #[test]
    fn is_host_collapsing_false() {
        assert!(!is_host_collapsing(1000.0, 50.0, 200.0, 10.0));
    }

    #[test]
    fn ticks_to_collapse_standard() {
        assert_eq!(ticks_to_collapse(1000.0, 100.0), 10);
    }

    #[test]
    fn ticks_to_collapse_no_drain() {
        assert_eq!(ticks_to_collapse(1000.0, 0.0), u32::MAX);
    }
}
