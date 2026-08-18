//! Axioma 6 (Emergence at Scale) — order parameter de Kuramoto + paso mean-field.
//! Axiom 6 — Kuramoto order parameter + mean-field phase step.
//!
//! Sin dependencias de Bevy — 100% testeable sin ECS.
//!
//! El order parameter `R = |(1/N)·Σ e^{iθ_j}| ∈ [0,1]` es una cantidad
//! **colectiva** que no existe a nivel individual: mide cuán sincronizada está la
//! población. En el paso mean-field `dθ_i = ω_i + K·R·sin(ψ − θ_i)`, el orden
//! global `R` retroalimenta a cada unidad — eso *es* emergencia a escala. Con la
//! regla local activa (`K > 0`) R sube de ~0 (desordenado) a ~1 (sincronizado);
//! ablando la regla (`K = 0`) R colapsa al piso de N finito (~1/√N). Ver ADR-046.
//!
//! El motor real (`simulation/emergence/entrainment.rs`) acopla **frecuencia**, no
//! fase: para él la métrica honesta es el colapso de la dispersión de frecuencias
//! `S = 1 − σ_ω(T)/σ_ω(0)` (`frequency_std` + `frequency_collapse_s` +
//! `sync_ecs_verdict`, consumidas por `tests/emergence_ecs.rs`). Ver ADR-047.

use core::f32::consts::TAU;

use crate::blueprint::constants::{
    SYNC_ABLATED_R_MAX, SYNC_ECS_ABLATED_S_MAX, SYNC_ECS_S_PASS_MIN, SYNC_R_PASS_MIN,
};

/// Order parameter de Kuramoto `R = |(1/N)·Σ e^{iθ_j}| ∈ [0,1]`.
///
/// Invariantes: `N=0 → 0` (sin osciladores no hay coherencia); `N=1 → 1`
/// (un oscilador es trivialmente coherente). Invariante bajo rotación global de
/// fase: `R(θ) == R(θ + c)`.
#[inline]
pub fn kuramoto_order_parameter(phases: &[f32]) -> f32 {
    kuramoto_order_parameter_complex(phases).0
}

/// Order parameter complejo: `(R, ψ)` con `R` la coherencia y `ψ` la fase media.
///
/// `ψ` alimenta el paso mean-field. Con `N=0` retorna `(0, 0)`.
pub fn kuramoto_order_parameter_complex(phases: &[f32]) -> (f32, f32) {
    let n = phases.len();
    if n == 0 {
        return (0.0, 0.0);
    }
    let inv_n = 1.0 / n as f32;
    let mut sum_cos = 0.0f32;
    let mut sum_sin = 0.0f32;
    for &theta in phases {
        sum_cos += theta.cos();
        sum_sin += theta.sin();
    }
    let mean_cos = sum_cos * inv_n;
    let mean_sin = sum_sin * inv_n;
    let r = (mean_cos * mean_cos + mean_sin * mean_sin).sqrt().clamp(0.0, 1.0);
    let psi = mean_sin.atan2(mean_cos);
    (r, psi)
}

/// Un paso del integrador de fase mean-field de Kuramoto.
///
/// `θ_i' = (θ_i + dt·(ω_i + K·R·sin(ψ − θ_i))) mod 2π`
///
/// El acoplamiento se clampa a `≥ 0` (calca `kuramoto_pair_delta`). `rem_euclid`
/// mantiene la fase en `[0, 2π)` para estabilidad numérica sobre 10⁴–10⁵ pasos.
#[inline]
pub fn kuramoto_meanfield_phase_step(
    phase_i: f32,
    omega_i: f32,
    r: f32,
    psi: f32,
    coupling: f32,
    dt: f32,
) -> f32 {
    let drive = coupling.max(0.0) * r * (psi - phase_i).sin();
    (phase_i + dt * (omega_i + drive)).rem_euclid(TAU)
}

/// Veredicto de emergencia (Ax6): sincronización con la regla y ausencia sin ella.
///
/// `PASS ⇔ r_final ≥ SYNC_R_PASS_MIN ∧ r_ablated ≤ SYNC_ABLATED_R_MAX`.
/// La separación mínima (`SYNC_GAP_MIN`) queda garantizada por la consistencia de
/// los umbrales (ver `tests/r10_emergence_gates.rs`).
#[inline]
pub fn emergence_sync_verdict(r_final: f32, r_ablated: f32) -> bool {
    r_final >= SYNC_R_PASS_MIN && r_ablated <= SYNC_ABLATED_R_MAX
}

// ─── Entrainment de frecuencia (probe ECS, ADR-047) ──────────────────────────
// Frequency entrainment metric (ECS probe).

/// Desviación estándar **muestral** (convención `n−1`) de una población de ω.
/// Sample standard deviation (`n−1` convention) of a frequency population.
///
/// La convención se declara porque el σ reportado depende de ella (~0.8 % con
/// N=64); para `S = 1 − σ(T)/σ(0)` es indistinto (se cancela).
/// Bordes: `n < 2 → 0.0` (un oscilador no tiene dispersión); cualquier entrada
/// no-finita → `0.0` (mismo criterio que `gaussian_frequency_alignment`).
pub fn frequency_std(freqs: &[f32]) -> f32 {
    let n = freqs.len();
    if n < 2 || freqs.iter().any(|f| !f.is_finite()) {
        return 0.0;
    }
    let mean = freqs.iter().sum::<f32>() / n as f32;
    let sum_sq: f32 = freqs
        .iter()
        .map(|&f| {
            let d = f - mean;
            d * d
        })
        .sum();
    (sum_sq / (n - 1) as f32).sqrt()
}

/// Colapso de la dispersión de frecuencias `S = 1 − σ_ω(T)/σ_ω(0) ∈ [0,1]`.
/// Frequency-spread collapse — the order parameter for frequency entrainment.
///
/// `S = 0` ⇔ la población no se estrechó; `S = 1` ⇔ colapso total (una sola ω).
/// Bordes: `σ(0) ≤ 0 → 0.0` (incluye N=1, donde σ=0 y no hay orden que medir);
/// entradas no-finitas → `0.0`; el resultado se clampa a `[0,1]` (una dispersión
/// que *crece* no es orden negativo, es ausencia de orden).
#[inline]
pub fn frequency_collapse_s(sigma_0: f32, sigma_t: f32) -> f32 {
    if !sigma_0.is_finite() || !sigma_t.is_finite() || sigma_0 <= 0.0 {
        return 0.0;
    }
    (1.0 - sigma_t / sigma_0).clamp(0.0, 1.0)
}

/// Veredicto de emergencia sobre el motor ECS (Ax6): colapso con la regla, nada sin ella.
/// ECS emergence verdict — collapse with the rule, none without it.
///
/// `PASS ⇔ s_coupled ≥ SYNC_ECS_S_PASS_MIN ∧ s_ablated ≤ SYNC_ECS_ABLATED_S_MAX`.
/// Misma forma que `emergence_sync_verdict`. Con dos ablaciones (regla y alcance)
/// se llama **una vez por cada una**: `verdict(s, s_a1) && verdict(s, s_a2)`.
#[inline]
pub fn sync_ecs_verdict(s_coupled: f32, s_ablated: f32) -> bool {
    s_coupled >= SYNC_ECS_S_PASS_MIN && s_ablated <= SYNC_ECS_ABLATED_S_MAX
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::blueprint::equations::determinism::next_u64;

    const EPS: f32 = 1e-4;

    #[test]
    fn identical_phases_give_full_coherence() {
        let phases = [1.3f32; 64];
        assert!((kuramoto_order_parameter(&phases) - 1.0).abs() < EPS);
    }

    #[test]
    fn antiphase_pair_gives_zero() {
        let phases = [0.0, TAU / 2.0];
        assert!(kuramoto_order_parameter(&phases) < EPS);
    }

    #[test]
    fn uniform_phases_are_incoherent() {
        // 512 fases equiespaciadas en [0, 2π) → R ≈ 0 (control uniforme).
        let n = 512;
        let phases: Vec<f32> = (0..n).map(|i| TAU * i as f32 / n as f32).collect();
        assert!(kuramoto_order_parameter(&phases) < 0.01);
    }

    #[test]
    fn empty_is_zero_single_is_one() {
        assert_eq!(kuramoto_order_parameter(&[]), 0.0);
        assert!((kuramoto_order_parameter(&[2.7]) - 1.0).abs() < EPS);
    }

    #[test]
    fn r_is_invariant_under_global_rotation() {
        let base = [0.1f32, 0.9, 1.7, 2.4, 3.1];
        let shifted: Vec<f32> = base.iter().map(|&t| t + 1.234).collect();
        let r0 = kuramoto_order_parameter(&base);
        let r1 = kuramoto_order_parameter(&shifted);
        assert!((r0 - r1).abs() < EPS, "R debe ser invariante a rotación global");
    }

    #[test]
    fn mean_phase_points_to_cluster() {
        // Fases agrupadas cerca de π/2 → ψ ≈ π/2.
        let phases = [1.4f32, 1.5, 1.6, 1.7];
        let (_r, psi) = kuramoto_order_parameter_complex(&phases);
        assert!((psi - TAU / 4.0).abs() < 0.2);
    }

    #[test]
    fn random_phases_stay_in_unit_range() {
        let mut state = 0xABCD_1234u64;
        let phases: Vec<f32> = (0..256)
            .map(|_| {
                state = next_u64(state);
                (state >> 40) as f32 / (1u64 << 24) as f32 * TAU
            })
            .collect();
        let r = kuramoto_order_parameter(&phases);
        assert!(r.is_finite() && (0.0..=1.0).contains(&r));
    }

    #[test]
    fn meanfield_step_pulls_toward_mean_phase() {
        // Con R alto y ψ por delante, la fase avanza hacia ψ (sin dispersión de ω).
        let phase = 0.0;
        let next = kuramoto_meanfield_phase_step(phase, 0.0, 0.9, TAU / 4.0, 0.5, 0.08);
        assert!(next > phase, "el acoplamiento debe empujar hacia ψ");
    }

    #[test]
    fn meanfield_zero_coupling_only_drifts_by_omega() {
        // Sin acoplamiento, la fase avanza sólo por su frecuencia propia.
        let next = kuramoto_meanfield_phase_step(0.0, 1.0, 0.9, 1.0, 0.0, 0.1);
        assert!((next - 0.1).abs() < EPS);
    }

    #[test]
    fn verdict_passes_when_coupled_syncs_and_ablated_stays_low() {
        assert!(emergence_sync_verdict(0.92, 0.05));
        assert!(!emergence_sync_verdict(0.40, 0.05)); // no sincroniza
        assert!(!emergence_sync_verdict(0.92, 0.30)); // ablado demasiado alto
    }

    // ── frequency_std ────────────────────────────────────────────────────────

    #[test]
    fn frequency_std_empty_is_zero() {
        assert_eq!(frequency_std(&[]), 0.0);
    }

    #[test]
    fn frequency_std_single_sample_is_zero() {
        // n−1 = 0 → indefinido; el contrato devuelve 0.0 (sin dispersión medible).
        assert_eq!(frequency_std(&[75.0]), 0.0);
    }

    #[test]
    fn frequency_std_identical_population_is_zero() {
        assert!(frequency_std(&[75.0f32; 64]) < EPS);
    }

    #[test]
    fn frequency_std_matches_known_sample_value() {
        // [2,4,4,4,5,5,7,9]: media 5, Σd² = 32, n−1 = 7 → σ = √(32/7) = 2.13809.
        let s = frequency_std(&[2.0, 4.0, 4.0, 4.0, 5.0, 5.0, 7.0, 9.0]);
        assert!((s - 2.138_09).abs() < 1e-4, "got {s}");
    }

    #[test]
    fn frequency_std_uses_n_minus_one_not_n() {
        // Población de 2: σ_(n−1) = |Δ|/√2 = 7.07107; σ_n sería 5.0.
        let s = frequency_std(&[70.0, 80.0]);
        assert!((s - 7.071_07).abs() < 1e-4, "n−1 esperado, got {s}");
    }

    #[test]
    fn frequency_std_non_finite_input_is_zero() {
        assert_eq!(frequency_std(&[75.0, f32::NAN, 80.0]), 0.0);
        assert_eq!(frequency_std(&[75.0, f32::INFINITY]), 0.0);
    }

    #[test]
    fn frequency_std_is_translation_invariant() {
        let base = [70.0f32, 72.5, 79.0, 81.25, 68.0];
        let shifted: Vec<f32> = base.iter().map(|&f| f + 500.0).collect();
        assert!((frequency_std(&base) - frequency_std(&shifted)).abs() < 1e-3);
    }

    // ── frequency_collapse_s ─────────────────────────────────────────────────

    #[test]
    fn collapse_s_total_collapse_is_one() {
        assert!((frequency_collapse_s(8.0, 0.0) - 1.0).abs() < EPS);
    }

    #[test]
    fn collapse_s_no_change_is_zero() {
        assert!(frequency_collapse_s(8.0, 8.0).abs() < EPS);
    }

    #[test]
    fn collapse_s_half_spread_is_half() {
        assert!((frequency_collapse_s(8.0, 4.0) - 0.5).abs() < EPS);
    }

    #[test]
    fn collapse_s_zero_initial_sigma_is_zero() {
        // N=1 o población degenerada: σ(0)=0 → no hay orden que colapsar.
        assert_eq!(frequency_collapse_s(0.0, 0.0), 0.0);
        assert_eq!(frequency_collapse_s(-1.0, 0.5), 0.0);
    }

    #[test]
    fn collapse_s_non_finite_inputs_are_zero() {
        assert_eq!(frequency_collapse_s(f32::NAN, 1.0), 0.0);
        assert_eq!(frequency_collapse_s(8.0, f32::NAN), 0.0);
        assert_eq!(frequency_collapse_s(f32::INFINITY, 1.0), 0.0);
    }

    #[test]
    fn collapse_s_growing_spread_clamps_to_zero() {
        // σ(T) > σ(0) → dispersión que crece: ausencia de orden, no orden negativo.
        assert_eq!(frequency_collapse_s(2.0, 10.0), 0.0);
    }

    #[test]
    fn collapse_s_stays_in_unit_range() {
        for (s0, st) in [(8.0f32, 0.1f32), (0.5, 0.5), (100.0, 1.0), (1.0, 1000.0)] {
            let s = frequency_collapse_s(s0, st);
            assert!((0.0..=1.0).contains(&s), "S={s} fuera de [0,1]");
        }
    }

    // ── sync_ecs_verdict ─────────────────────────────────────────────────────

    #[test]
    fn ecs_verdict_passes_when_coupled_collapses_and_ablated_does_not() {
        assert!(sync_ecs_verdict(0.9, 0.0));
        assert!(!sync_ecs_verdict(0.4, 0.0)); // no colapsa lo suficiente
        assert!(!sync_ecs_verdict(0.9, 0.5)); // la ablación no suprime el orden
    }
}
