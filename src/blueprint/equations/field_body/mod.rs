use crate::blueprint::constants::*;
use crate::layers::FieldFalloffMode;
use crate::math_types::Vec2;

// ═══════════════════════════════════════════════
// Capa 11: Campo de Tensión
// ═══════════════════════════════════════════════

/// Atenuación de fuerza a distancia (Axioma 7: monótonamente decreciente).
/// Field force attenuation with distance (Axiom 7: monotonically decreasing).
///
/// Ambos modos (`1/d²`, `1/d`) son estrictamente decrecientes en `d > 0`; el
/// `max` con `min_distance`/`DIVISION_GUARD_EPSILON` evita la singularidad en 0
/// sin romper la monotonicidad (test `falloff_is_monotonically_decreasing`).
pub fn safe_falloff(distance: f32, mode: FieldFalloffMode, min_distance: f32) -> f32 {
    let d = distance.max(min_distance.max(DIVISION_GUARD_EPSILON));
    match mode {
        FieldFalloffMode::InverseSquare => 1.0 / (d * d),
        FieldFalloffMode::InverseLinear => 1.0 / d,
    }
}

#[allow(clippy::too_many_arguments)]
pub fn tension_field_acceleration(
    source_qe: f32,
    target_qe: f32,
    delta: Vec2,
    gravity_gain: f32,
    magnetic_gain: f32,
    interference: f32,
    mode: FieldFalloffMode,
    min_distance: f32,
) -> Vec2 {
    if source_qe <= 0.0 || target_qe <= 0.0 {
        return Vec2::ZERO;
    }

    let distance = delta.length();
    if distance <= 0.0 {
        return Vec2::ZERO;
    }

    let dir_to_source = delta / distance;
    let falloff = safe_falloff(distance, mode, min_distance);
    let grav = gravity_gain * source_qe * target_qe * falloff;
    let magnet = magnetic_gain * interference * source_qe * target_qe * falloff;
    let total_force = grav + magnet;

    if !total_force.is_finite() {
        return Vec2::ZERO;
    }

    // a = F / m  con m ~= qe_target.
    let acc = dir_to_source * (total_force / target_qe.max(DIVISION_GUARD_EPSILON));
    if acc.is_finite() { acc } else { Vec2::ZERO }
}

// ═══════════════════════════════════════════════
// Capa 12: Homeostasis
// ═══════════════════════════════════════════════

pub fn homeostasis_delta_hz(current_hz: f32, target_hz: f32, adapt_rate_hz: f32, dt: f32) -> f32 {
    if dt <= 0.0 || adapt_rate_hz <= 0.0 {
        return 0.0;
    }
    let max_step = adapt_rate_hz * dt;
    (target_hz - current_hz).clamp(-max_step, max_step)
}

pub fn homeostasis_qe_cost(delta_hz: f32, qe_cost_per_hz: f32) -> f32 {
    delta_hz.abs() * qe_cost_per_hz.max(0.0)
}

// ═══════════════════════════════════════════════
// Capa 13: Enlace Estructural
// ═══════════════════════════════════════════════

pub fn spring_force(delta: Vec2, rest_length: f32, stiffness: f32) -> Vec2 {
    let distance = delta.length();
    if distance <= DISTANCE_EPSILON || stiffness <= 0.0 {
        return Vec2::ZERO;
    }
    let dir = delta / distance;
    let extension = distance - rest_length.max(0.0);
    dir * (stiffness * extension)
}

pub fn structural_stress(extension: f32, shared_thermal_load: f32) -> f32 {
    extension.abs() + shared_thermal_load.abs()
}

/// Transferencia conservativa de `qe` por desbalance entre extremos de un enlace estructural.
#[inline]
pub fn structural_link_qe_transfer(qe_imbalance_abs: f32, stiffness: f32, dt: f32) -> f32 {
    if qe_imbalance_abs <= 0.0 || stiffness <= 0.0 || dt <= 0.0 {
        return 0.0;
    }
    let raw = qe_imbalance_abs * STRUCTURAL_LINK_QE_TRANSFER_COEF * stiffness * dt;
    raw.min(qe_imbalance_abs)
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── safe_falloff (Axioma 7) ──

    /// Ax7: la atenuación decrece monótonamente con la distancia, en ambos modos.
    #[test]
    fn falloff_is_monotonically_decreasing() {
        let eps = 0.01;
        for mode in [FieldFalloffMode::InverseSquare, FieldFalloffMode::InverseLinear] {
            let mut prev = f32::INFINITY;
            let mut d = 0.5;
            while d <= 50.0 {
                let f = safe_falloff(d, mode, eps);
                assert!(
                    f <= prev,
                    "Ax7 violado: falloff({d}) = {f} > previo {prev} en {mode:?}"
                );
                prev = f;
                d += 0.5;
            }
        }
    }

    #[test]
    fn falloff_softens_singularity_at_zero() {
        // Distancia 0 no produce NaN/Inf: se clampa a min_distance.
        let f = safe_falloff(0.0, FieldFalloffMode::InverseSquare, 0.1);
        assert!(f.is_finite() && f > 0.0);
    }

    // ── homeostasis_delta_hz ──

    #[test]
    fn homeostasis_adapts_frequency_toward_target() {
        let delta = homeostasis_delta_hz(100.0, 120.0, 5.0, 1.0);
        assert!(delta > 0.0, "should move toward higher target");
        assert!(delta <= 5.0, "clamped by adapt_rate * dt");

        let delta_down = homeostasis_delta_hz(120.0, 100.0, 5.0, 1.0);
        assert!(delta_down < 0.0, "should move toward lower target");
    }

    #[test]
    fn homeostasis_delta_hz_zero_rate_gives_zero() {
        assert_eq!(homeostasis_delta_hz(100.0, 120.0, 0.0, 1.0), 0.0);
    }

    #[test]
    fn homeostasis_delta_hz_zero_dt_gives_zero() {
        assert_eq!(homeostasis_delta_hz(100.0, 120.0, 5.0, 0.0), 0.0);
    }

    #[test]
    fn homeostasis_delta_hz_at_target_gives_zero() {
        assert_eq!(homeostasis_delta_hz(100.0, 100.0, 5.0, 1.0), 0.0);
    }

    // ── homeostasis_qe_cost ──

    #[test]
    fn homeostasis_drains_qe_proportional_to_delta() {
        let cost_small = homeostasis_qe_cost(1.0, 2.0);
        let cost_large = homeostasis_qe_cost(5.0, 2.0);
        assert!(cost_large > cost_small);
        assert!((cost_small - 2.0).abs() < 1e-5);
        assert!((cost_large - 10.0).abs() < 1e-5);
    }

    #[test]
    fn homeostasis_stops_adapting_when_no_qe() {
        // When delta is very large, cost exceeds any reasonable qe budget.
        let cost = homeostasis_qe_cost(100.0, 2.0);
        let available_qe = 10.0;
        assert!(
            cost > available_qe,
            "cost exceeds available qe — system should skip"
        );
    }

    #[test]
    fn homeostasis_qe_cost_negative_delta_uses_abs() {
        let cost_pos = homeostasis_qe_cost(3.0, 1.0);
        let cost_neg = homeostasis_qe_cost(-3.0, 1.0);
        assert!((cost_pos - cost_neg).abs() < 1e-5);
    }

    #[test]
    fn homeostasis_qe_cost_zero_rate_gives_zero() {
        assert_eq!(homeostasis_qe_cost(5.0, 0.0), 0.0);
    }
}
