//! Cultura como observable energético — inferida desde axiomas existentes.
//!
//! **Sin componentes nuevos.** Todo deriva de:
//! - L2  `OscillatorySignature` (frequency_hz, phase)
//! - L12 `Homeostasis`          (adapt_rate como coupling_strength)
//! - Catalysis                  (interferencia entre frecuencias)
//! - `SimulationClock`          (tick_age para longevidad)
//!
//! Cultura(G) = coherencia(G) × síntesis(G) × resiliencia(G) × longevidad(G)
//!
//! Todos los inputs son escalares o slices de `f32` extraídos por el caller (sistema).
//! Esta capa no toca ECS.
//!
//! Complejidad: `group_frequency_coherence` O(n), `internal_synthesis_rate` O(n²).
//! Usar para observación/analytics, no en hot path por entidad.

use std::f32::consts::PI;

use super::finite_helpers::{finite_non_negative, finite_unit};
use crate::blueprint::constants::{
    CULTURE_COHERENCE_MIN, CULTURE_CONFLICT_THRESHOLD, CULTURE_FREQ_BANDWIDTH_HZ,
    CULTURE_GROUP_MIN_SIZE, CULTURE_MAX_EXPECTED_AGE_TICKS, CULTURE_PERCOLATION_CONNECTIVITY,
    CULTURE_PHASE_GAS_MAX, CULTURE_PHASE_SOLID_MIN, CULTURE_RESILIENCE_MIN, CULTURE_SYNTHESIS_MIN,
    DIVISION_GUARD_EPSILON,
};

// ── Tipo de salida ────────────────────────────────────────────────────────────

/// Estado de fase cultural: análogo a `MatterState` pero en espacio de frecuencias.
///
/// Transición de fase derivada de coherencia, no de densidad+temperatura.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CulturalPhase {
    /// Frecuencias dispersas — alta entropía, sin identidad grupal sostenida.
    Gas,
    /// Coaliciones temporales — coherencia parcial, fluida, inestable bajo perturbación.
    Liquid,
    /// Identidad grupal estable — coherencia resistente, se propaga a nuevas entidades.
    Solid,
}

// ── Coherencia de frecuencia ──────────────────────────────────────────────────

/// Coherencia de frecuencia del grupo G: `1 - CV(frequencies)`.
///
/// CV = σ/μ (coeficiente de variación).
/// - `→ 1.0`: todas en la misma frecuencia ("sólido cultural")
/// - `→ 0.0`: frecuencias dispersas ("gas cultural")
///
/// Retorna `0.0` si el slice está vacío o μ ≈ 0.
pub fn group_frequency_coherence(frequencies: &[f32]) -> f32 {
    let n = frequencies.len();
    if n == 0 {
        return 0.0;
    }
    let mean = frequencies.iter().copied().sum::<f32>() / n as f32;
    if mean < DIVISION_GUARD_EPSILON {
        return 0.0;
    }
    let variance = frequencies
        .iter()
        .map(|&f| (finite_non_negative(f) - mean).powi(2))
        .sum::<f32>()
        / n as f32;
    let cv = variance.sqrt() / mean;
    (1.0 - cv).clamp(0.0, 1.0)
}

// ── Síntesis catalítica ───────────────────────────────────────────────────────

/// Interferencia normalizada entre dos frecuencias: `cos(π × |Δf| / bandwidth)`.
///
/// - `+1.0` → misma frecuencia (constructiva, refuerzo mutuo)
/// - `-1.0` → máxima separación (destructiva, daño mutuo)
/// - ` 0.0` → cuarto del ancho de banda (ortogonal, sin interacción)
#[inline]
pub fn freq_interference(fa: f32, fb: f32) -> f32 {
    let delta = (finite_non_negative(fa) - finite_non_negative(fb))
        .abs()
        .min(CULTURE_FREQ_BANDWIDTH_HZ);
    (PI * delta / CULTURE_FREQ_BANDWIDTH_HZ).cos()
}

/// Tasa de síntesis catalítica interna: fracción de pares con interferencia constructiva.
///
/// Itera todos los pares `(i, j)` con `i < j` — O(n²).
/// Usar para snapshots/observabilidad, no en hot path.
/// Retorna `0.0` si `n < 2`.
pub fn internal_synthesis_rate(frequencies: &[f32]) -> f32 {
    let n = frequencies.len();
    if n < 2 {
        return 0.0;
    }
    let total_pairs = (n * (n - 1)) / 2;
    let constructive = frequencies
        .iter()
        .enumerate()
        .flat_map(|(i, &fa)| {
            frequencies[i + 1..]
                .iter()
                .map(move |&fb| freq_interference(fa, fb))
        })
        .filter(|&interference| interference > 0.0)
        .count();
    constructive as f32 / total_pairs as f32
}

// ── Resiliencia del patrón ────────────────────────────────────────────────────

/// Resiliencia del patrón: `coherence_after / coherence_before`, clampeada a `[0, 1]`.
///
/// Mide cuánta coherencia recupera el grupo después de una perturbación externa.
/// Retorna `0.0` si `coherence_before ≈ 0`.
pub fn pattern_resilience(coherence_before: f32, coherence_after: f32) -> f32 {
    let before = finite_unit(coherence_before);
    if before < DIVISION_GUARD_EPSILON {
        return 0.0;
    }
    (finite_unit(coherence_after) / before).clamp(0.0, 1.0)
}

// ── Longevidad normalizada ────────────────────────────────────────────────────

/// Longevidad normalizada del grupo: `mean_tick_age / CULTURE_MAX_EXPECTED_AGE_TICKS`.
///
/// Proxy de calidad de vida interna: grupos longevos tienen más ciclos disponibles
/// para entrainment y menos presión survival que disuelve la coherencia.
pub fn group_longevity_norm(mean_tick_age: f32) -> f32 {
    let age = finite_non_negative(mean_tick_age);
    (age / CULTURE_MAX_EXPECTED_AGE_TICKS).clamp(0.0, 1.0)
}

// ── Índice cultural unificado ─────────────────────────────────────────────────

/// Índice cultural: producto de cuatro observables normalizados `[0, 1]`.
///
/// `= coherence × synthesis × resilience × longevity`
///
/// - `0.0` si cualquiera falla (el producto colapsa ante cualquier condición ausente)
/// - `→ 1.0` cuando todos convergen simultáneamente
pub fn culture_index(coherence: f32, synthesis: f32, resilience: f32, longevity: f32) -> f32 {
    let c = finite_unit(coherence);
    let s = finite_unit(synthesis);
    let r = finite_unit(resilience);
    let l = finite_unit(longevity);
    (c * s * r * l).clamp(0.0, 1.0)
}

// ── Transición de fase cultural ───────────────────────────────────────────────

/// Fase cultural derivada del índice de coherencia.
///
/// Análogo directo a `MatterState` en L4, pero aplicado al espacio de frecuencias.
pub fn cultural_phase(coherence: f32) -> CulturalPhase {
    let c = finite_unit(coherence);
    if c >= CULTURE_PHASE_SOLID_MIN {
        CulturalPhase::Solid
    } else if c > CULTURE_PHASE_GAS_MAX {
        CulturalPhase::Liquid
    } else {
        CulturalPhase::Gas
    }
}

// ── Entrainment y emergencia ──────────────────────────────────────────────────

/// ¿Pueden dos entidades sincronizar frecuencias dado el coupling de Homeostasis?
///
/// Condición de Kuramoto: `|ω_a - ω_b| < coupling_strength / (2π)`.
/// `coupling_strength` = `Homeostasis.adapt_rate` del par de entidades.
pub fn entrainment_possible(freq_a: f32, freq_b: f32, coupling_strength: f32) -> bool {
    let delta_omega = (finite_non_negative(freq_a) - finite_non_negative(freq_b)).abs() * 2.0 * PI;
    delta_omega < finite_non_negative(coupling_strength)
}

/// ¿Ha emergido cultura en el grupo? Todos los umbrales simultáneamente.
///
/// `spatial_connectivity` = fracción de pares en rango de catalysis / total_pares.
pub fn culture_emergent(
    coherence: f32,
    synthesis: f32,
    resilience: f32,
    group_size: usize,
    spatial_connectivity: f32,
) -> bool {
    finite_unit(coherence) >= CULTURE_COHERENCE_MIN
        && finite_unit(synthesis) >= CULTURE_SYNTHESIS_MIN
        && finite_unit(resilience) >= CULTURE_RESILIENCE_MIN
        && group_size >= CULTURE_GROUP_MIN_SIZE
        && finite_unit(spatial_connectivity) >= CULTURE_PERCOLATION_CONNECTIVITY
}

// ── Conflicto inter-grupo ─────────────────────────────────────────────────────

/// Potencial de conflicto entre dos grupos: interferencia media entre todos los pares cruzados.
///
/// O(na × nb) — usar para observabilidad, no en hot path.
/// Retorna `[-1, 1]`. Negativo → destructivo → conflicto potencial.
pub fn inter_group_conflict_potential(freqs_a: &[f32], freqs_b: &[f32]) -> f32 {
    let na = freqs_a.len();
    let nb = freqs_b.len();
    if na == 0 || nb == 0 {
        return 0.0;
    }
    let total = (na * nb) as f32;
    let sum: f32 = freqs_a
        .iter()
        .flat_map(|&fa| freqs_b.iter().map(move |&fb| freq_interference(fa, fb)))
        .sum();
    (sum / total).clamp(-1.0, 1.0)
}

/// ¿Hay conflicto activo entre dos grupos?
///
/// `cos(Δfreq) < CULTURE_CONFLICT_THRESHOLD` → los grupos se dañan mutuamente en catalysis.
#[inline]
pub fn conflict_active(inter_group_potential: f32) -> bool {
    inter_group_potential < CULTURE_CONFLICT_THRESHOLD
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    // ── group_frequency_coherence ─────────────────────────────────────────────

    #[test]
    fn coherence_identical_frequencies_returns_one() {
        let freqs = [20.0_f32, 20.0, 20.0, 20.0];
        let c = group_frequency_coherence(&freqs);
        assert!((c - 1.0).abs() < 1e-5, "coherence={c}");
    }

    #[test]
    fn coherence_empty_slice_returns_zero() {
        assert_eq!(group_frequency_coherence(&[]), 0.0);
    }

    #[test]
    fn coherence_single_entity_returns_one() {
        assert!((group_frequency_coherence(&[75.0]) - 1.0).abs() < 1e-5);
    }

    #[test]
    fn coherence_all_element_bands_is_gas_level() {
        // Umbra(20) + Terra(75) + Aqua(250) + Ignis(450) + Lux(1000) — dispersión máxima
        let freqs = [20.0_f32, 75.0, 250.0, 450.0, 1000.0];
        let c = group_frequency_coherence(&freqs);
        assert!(c < CULTURE_PHASE_GAS_MAX + 0.1, "gas cultural: c={c}");
    }

    #[test]
    fn coherence_tight_band_is_higher_than_wide_band() {
        let tight = [98.0_f32, 100.0, 102.0, 99.0];
        let loose = [50.0_f32, 200.0, 500.0, 900.0];
        assert!(group_frequency_coherence(&tight) > group_frequency_coherence(&loose));
    }

    #[test]
    fn coherence_near_solid_threshold_for_same_band() {
        // Grupo Terra homogéneo → debe superar C_min
        let terra = [72.0_f32, 74.0, 76.0, 75.0, 73.0];
        let c = group_frequency_coherence(&terra);
        assert!(
            c >= CULTURE_COHERENCE_MIN,
            "grupo Terra homogéneo debe tener cultura: c={c}"
        );
    }

    // ── freq_interference ─────────────────────────────────────────────────────

    #[test]
    fn freq_interference_same_frequency_returns_one() {
        let i = freq_interference(100.0, 100.0);
        assert!((i - 1.0).abs() < 1e-5, "i={i}");
    }

    #[test]
    fn freq_interference_max_separation_returns_neg_one() {
        let i = freq_interference(0.0, CULTURE_FREQ_BANDWIDTH_HZ);
        assert!((i + 1.0).abs() < 1e-5, "i={i}");
    }

    #[test]
    fn freq_interference_half_bandwidth_is_orthogonal() {
        // cos(π × BW/2 / BW) = cos(π/2) = 0 → punto ortogonal exacto
        let i = freq_interference(0.0, CULTURE_FREQ_BANDWIDTH_HZ / 2.0);
        assert!(
            i.abs() < 1e-5,
            "mitad del ancho de banda = ortogonal: i={i}"
        );
    }

    #[test]
    fn freq_interference_umbra_vs_lux_is_destructive() {
        let i = freq_interference(20.0, 1000.0);
        assert!(i < 0.0, "Umbra vs Lux debe ser destructivo: i={i}");
    }

    #[test]
    fn freq_interference_within_terra_band_is_constructive() {
        let i = freq_interference(70.0, 85.0);
        assert!(i > 0.0, "dentro de Terra debe ser constructivo: i={i}");
    }

    // ── internal_synthesis_rate ───────────────────────────────────────────────

    #[test]
    fn synthesis_all_same_frequency_returns_one() {
        let freqs = [75.0_f32, 75.0, 75.0];
        assert!((internal_synthesis_rate(&freqs) - 1.0).abs() < 1e-5);
    }

    #[test]
    fn synthesis_empty_returns_zero() {
        assert_eq!(internal_synthesis_rate(&[]), 0.0);
    }

    #[test]
    fn synthesis_single_entity_returns_zero() {
        assert_eq!(internal_synthesis_rate(&[100.0]), 0.0);
    }

    #[test]
    fn synthesis_opposing_bands_is_below_threshold() {
        // Umbra vs Lux — interferencia casi completamente destructiva
        let freqs = [20.0_f32, 1000.0];
        let s = internal_synthesis_rate(&freqs);
        assert!(s < CULTURE_SYNTHESIS_MIN, "s={s}");
    }

    #[test]
    fn synthesis_homogeneous_group_exceeds_threshold() {
        let terra = [72.0_f32, 74.0, 76.0, 75.0];
        assert!(internal_synthesis_rate(&terra) >= CULTURE_SYNTHESIS_MIN);
    }

    // ── pattern_resilience ────────────────────────────────────────────────────

    #[test]
    fn resilience_no_change_returns_one() {
        assert!((pattern_resilience(0.8, 0.8) - 1.0).abs() < 1e-5);
    }

    #[test]
    fn resilience_total_loss_returns_zero() {
        assert!(pattern_resilience(0.8, 0.0).abs() < 1e-5);
    }

    #[test]
    fn resilience_zero_before_returns_zero() {
        assert_eq!(pattern_resilience(0.0, 0.5), 0.0);
    }

    #[test]
    fn resilience_growth_clamped_to_one() {
        // Si la coherencia aumentó tras perturbación → no puede > 1.0
        assert_eq!(pattern_resilience(0.3, 0.9), 1.0);
    }

    // ── group_longevity_norm ──────────────────────────────────────────────────

    #[test]
    fn longevity_zero_age_returns_zero() {
        assert_eq!(group_longevity_norm(0.0), 0.0);
    }

    #[test]
    fn longevity_at_max_expected_returns_one() {
        assert!((group_longevity_norm(CULTURE_MAX_EXPECTED_AGE_TICKS) - 1.0).abs() < 1e-5);
    }

    #[test]
    fn longevity_clamped_above_one() {
        assert_eq!(
            group_longevity_norm(CULTURE_MAX_EXPECTED_AGE_TICKS * 2.0),
            1.0
        );
    }

    #[test]
    fn longevity_monotonic_with_age() {
        assert!(group_longevity_norm(5000.0) > group_longevity_norm(1000.0));
    }

    // ── culture_index ─────────────────────────────────────────────────────────

    #[test]
    fn culture_index_all_ones_returns_one() {
        assert!((culture_index(1.0, 1.0, 1.0, 1.0) - 1.0).abs() < 1e-5);
    }

    #[test]
    fn culture_index_any_zero_collapses_to_zero() {
        assert_eq!(culture_index(0.0, 1.0, 1.0, 1.0), 0.0);
        assert_eq!(culture_index(1.0, 0.0, 1.0, 1.0), 0.0);
        assert_eq!(culture_index(1.0, 1.0, 0.0, 1.0), 0.0);
        assert_eq!(culture_index(1.0, 1.0, 1.0, 0.0), 0.0);
    }

    #[test]
    fn culture_index_equals_product_of_components() {
        let (c, s, r, l) = (0.8_f32, 0.7, 0.9, 0.6);
        let expected = c * s * r * l;
        assert!((culture_index(c, s, r, l) - expected).abs() < 1e-5);
    }

    #[test]
    fn culture_index_nan_inputs_produce_zero() {
        assert_eq!(culture_index(f32::NAN, 1.0, 1.0, 1.0), 0.0);
    }

    // ── cultural_phase ────────────────────────────────────────────────────────

    #[test]
    fn cultural_phase_solid_at_high_coherence() {
        assert_eq!(cultural_phase(0.85), CulturalPhase::Solid);
        assert_eq!(
            cultural_phase(CULTURE_PHASE_SOLID_MIN),
            CulturalPhase::Solid
        );
        assert_eq!(cultural_phase(1.0), CulturalPhase::Solid);
    }

    #[test]
    fn cultural_phase_gas_at_low_coherence() {
        assert_eq!(cultural_phase(0.0), CulturalPhase::Gas);
        assert_eq!(cultural_phase(0.10), CulturalPhase::Gas);
        assert_eq!(cultural_phase(CULTURE_PHASE_GAS_MAX), CulturalPhase::Gas);
    }

    #[test]
    fn cultural_phase_liquid_between_thresholds() {
        assert_eq!(cultural_phase(0.45), CulturalPhase::Liquid);
        assert_eq!(cultural_phase(0.26), CulturalPhase::Liquid);
        assert_eq!(cultural_phase(0.69), CulturalPhase::Liquid);
    }

    #[test]
    fn cultural_phase_transitions_are_monotonic() {
        // Gas → Liquid → Solid con coherencia creciente
        assert!(cultural_phase(0.1) == CulturalPhase::Gas);
        assert!(cultural_phase(0.5) == CulturalPhase::Liquid);
        assert!(cultural_phase(0.9) == CulturalPhase::Solid);
    }

    // ── entrainment_possible ──────────────────────────────────────────────────

    #[test]
    fn entrainment_same_frequency_always_possible() {
        assert!(entrainment_possible(100.0, 100.0, 0.001));
    }

    #[test]
    fn entrainment_blocked_by_cross_band_gap() {
        // |20 - 1000| × 2π ≈ 5655 >> coupling=1.0
        assert!(!entrainment_possible(20.0, 1000.0, 1.0));
    }

    #[test]
    fn entrainment_possible_within_same_band_with_moderate_coupling() {
        // Terra: 70-80 Hz, gap ≈ 10 Hz; con coupling=100.0 >> 62.8
        assert!(entrainment_possible(70.0, 80.0, 100.0));
    }

    #[test]
    fn entrainment_boundary_exactly_at_coupling() {
        // gap × 2π == coupling → no es posible (strict <)
        let gap = 5.0_f32;
        let coupling = gap * 2.0 * PI;
        assert!(!entrainment_possible(0.0, gap, coupling));
        assert!(entrainment_possible(0.0, gap, coupling + 0.001));
    }

    // ── culture_emergent ──────────────────────────────────────────────────────

    #[test]
    fn culture_emergent_all_conditions_met_returns_true() {
        assert!(culture_emergent(0.80, 0.70, 0.60, 5, 0.70));
    }

    #[test]
    fn culture_not_emergent_coherence_below_min() {
        assert!(!culture_emergent(0.30, 0.70, 0.60, 5, 0.70));
    }

    #[test]
    fn culture_not_emergent_synthesis_below_min() {
        assert!(!culture_emergent(0.80, 0.30, 0.60, 5, 0.70));
    }

    #[test]
    fn culture_not_emergent_resilience_below_min() {
        assert!(!culture_emergent(0.80, 0.70, 0.20, 5, 0.70));
    }

    #[test]
    fn culture_not_emergent_group_too_small() {
        assert!(!culture_emergent(0.80, 0.70, 0.60, 2, 0.70));
    }

    #[test]
    fn culture_not_emergent_low_connectivity() {
        assert!(!culture_emergent(0.80, 0.70, 0.60, 5, 0.30));
    }

    // ── inter_group_conflict_potential ───────────────────────────────────────

    #[test]
    fn conflict_same_frequency_groups_is_positive() {
        let a = [75.0_f32, 75.0, 75.0];
        let b = [74.0_f32, 75.0, 76.0];
        assert!(inter_group_conflict_potential(&a, &b) > 0.0);
    }

    #[test]
    fn conflict_opposing_bands_below_conflict_threshold() {
        let umbra = [20.0_f32, 22.0, 18.0];
        let lux = [1000.0_f32, 990.0, 1010.0];
        let pot = inter_group_conflict_potential(&umbra, &lux);
        assert!(pot < CULTURE_CONFLICT_THRESHOLD, "pot={pot}");
    }

    #[test]
    fn conflict_empty_group_a_returns_zero() {
        assert_eq!(inter_group_conflict_potential(&[], &[100.0]), 0.0);
    }

    #[test]
    fn conflict_empty_group_b_returns_zero() {
        assert_eq!(inter_group_conflict_potential(&[100.0], &[]), 0.0);
    }

    #[test]
    fn conflict_active_detects_destructive_inter_group() {
        assert!(conflict_active(-0.5));
        assert!(!conflict_active(0.3));
        assert!(!conflict_active(CULTURE_CONFLICT_THRESHOLD + 0.01));
    }

    #[test]
    fn conflict_symmetric_between_groups() {
        let a = [20.0_f32, 25.0];
        let b = [950.0_f32, 1000.0];
        let ab = inter_group_conflict_potential(&a, &b);
        let ba = inter_group_conflict_potential(&b, &a);
        assert!(
            (ab - ba).abs() < 1e-5,
            "conflicto debe ser simétrico: {ab} vs {ba}"
        );
    }

    // ── integración: ciclo de vida completo ───────────────────────────────────

    #[test]
    fn full_culture_pipeline_terra_group_emergent() {
        // Grupo Terra homogéneo en suelo templado
        let freqs = [72.0_f32, 74.0, 76.0, 75.0, 73.0];
        let coherence = group_frequency_coherence(&freqs);
        let synthesis = internal_synthesis_rate(&freqs);
        let resilience = pattern_resilience(coherence, coherence * 0.95); // perturbación leve
        let longevity = group_longevity_norm(3000.0);

        let idx = culture_index(coherence, synthesis, resilience, longevity);
        let phase = cultural_phase(coherence);

        assert!(idx > 0.0, "índice debe ser positivo: {idx}");
        assert_eq!(
            phase,
            CulturalPhase::Solid,
            "Terra homogéneo → sólido cultural"
        );
        assert!(culture_emergent(coherence, synthesis, resilience, 5, 0.65));
    }

    #[test]
    fn full_culture_pipeline_cross_band_conflict() {
        let umbra = [20.0_f32, 18.0, 22.0];
        let lux = [1000.0_f32, 990.0, 1010.0];
        let pot = inter_group_conflict_potential(&umbra, &lux);
        assert!(
            conflict_active(pot),
            "Umbra vs Lux debe activar conflicto: {pot}"
        );
    }
}
