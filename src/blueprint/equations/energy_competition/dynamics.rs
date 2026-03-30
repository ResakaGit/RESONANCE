//! EC-5: Dinámica competitiva — análisis analítico puro de pools jerárquicos.
//! EC-5A: Matriz de competencia N×N (stack-allocated) + índice de Gini.
//! EC-5B: Detección de equilibrio, dominancia estable (ESS), y colapso.
//! EC-5C: Predicción lineal de trayectoria.

// DEBT: Reflect required because PoolHealthStatus is stored in Component (competition_dynamics.rs).
use bevy::prelude::Reflect;

use crate::blueprint::constants::{
    COLLAPSE_WARNING_TICKS, EXTRACTION_EPSILON, MAX_COMPETITION_MATRIX,
};
use super::{is_pool_equilibrium, ticks_to_collapse};

// ─── Tipos ───────────────────────────────────────────────────────────────────

/// Estado de salud de un pool energético.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Reflect)]
pub enum PoolHealthStatus {
    /// Intake cubre extracción + pérdida (net drain ≤ 0).
    Healthy,
    /// Drenando pero sin colapso inminente (ticks_to_collapse ≥ COLLAPSE_WARNING_TICKS).
    Stressed,
    /// Net drain > 0 y ticks_to_collapse < COLLAPSE_WARNING_TICKS.
    Collapsing,
    /// pool = 0.
    Collapsed,
}

/// Trayectoria proyectada del pool a tasa constante (lineal, v1).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PoolTrajectory {
    /// Ticks estimados hasta colapso. `u32::MAX` si estable o creciente.
    pub ticks_to_collapse: u32,
    /// Ticks estimados hasta capacidad máxima. `u32::MAX` si drenando.
    pub ticks_to_full: u32,
    /// Rate neto por tick: positivo = creciendo, negativo = drenando.
    pub net_change_per_tick: f32,
}

// ─── EC-5A: Matriz de Competencia ─────────────────────────────────────────────

/// Stack-allocated competition matrix type: N×N, max `MAX_COMPETITION_MATRIX` entries.
pub type CompetitionMatrix = [[f32; MAX_COMPETITION_MATRIX]; MAX_COMPETITION_MATRIX];

/// Calcula la matriz de competencia C[i][j] para N hijos (N ≤ MAX_COMPETITION_MATRIX).
/// C[i][j] = efecto de la extracción de hijo i sobre la disponibilidad de hijo j.
/// C[i][i] = energía neta retenida por hijo i.
/// Stack-allocated: [[f32; 16]; 16] = 1 KiB.
pub fn competition_matrix(
    extractions: &[f32],
    available: f32,
) -> [[f32; MAX_COMPETITION_MATRIX]; MAX_COMPETITION_MATRIX] {
    let mut m = [[0.0f32; MAX_COMPETITION_MATRIX]; MAX_COMPETITION_MATRIX];
    let n     = extractions.len().min(MAX_COMPETITION_MATRIX);
    let avail = available.max(EXTRACTION_EPSILON);
    for i in 0..n {
        let sum_others: f32 = (0..n).filter(|&k| k != i).map(|k| extractions[k]).sum();
        for j in 0..n {
            m[i][j] = if i == j {
                extractions[i] - extractions[i] * sum_others / avail
            } else {
                -extractions[i] / avail
            };
        }
    }
    m
}

/// Índice de competencia: Gini coefficient sobre las extracciones.
/// 0.0 = perfectamente equitativo, 1.0 = un solo hijo toma todo.
/// Gini = Σ|x_i − x_j| / (2·N·Σx_i). Guard: todos 0 → 0.0.
pub fn competition_intensity(extractions: &[f32]) -> f32 {
    let n = extractions.len();
    if n == 0 { return 0.0; }
    let total: f32 = extractions.iter().copied().sum();
    if total <= EXTRACTION_EPSILON { return 0.0; }
    let mut sum_abs = 0.0f32;
    for i in 0..n {
        for j in 0..n {
            sum_abs += (extractions[i] - extractions[j]).abs();
        }
    }
    (sum_abs / (2.0 * n as f32 * total)).clamp(0.0, 1.0)
}

// ─── EC-5B: Equilibrio y Dominancia ───────────────────────────────────────────

/// ¿El pool está en equilibrio estable? Wrapper de EC-1 `is_pool_equilibrium`.
#[inline]
pub fn detect_equilibrium(
    intake: f32,
    total_extracted: f32,
    dissipation_loss: f32,
    epsilon: f32,
) -> bool {
    is_pool_equilibrium(intake, total_extracted, dissipation_loss, epsilon)
}

/// ¿Hijo `dominant_index` tiene dominancia estable (ESS)?
/// Requiere extracción estrictamente mayor que todos los hermanos, y pool viable.
pub fn detect_dominance(
    extractions: &[f32],
    dominant_index: usize,
    pool_viable: bool,
) -> bool {
    if !pool_viable { return false; }
    let n = extractions.len();
    if n == 0 || dominant_index >= n { return false; }
    let dom = extractions[dominant_index];
    (0..n).filter(|&k| k != dominant_index).all(|k| dom > extractions[k])
}

/// Diagnostica el estado de salud del pool a partir del net drain.
/// - `Collapsed`:  pool ≤ 0.
/// - `Collapsing`: net drain > 0 y ticks_to_collapse < COLLAPSE_WARNING_TICKS.
/// - `Stressed`:   net drain > 0 y ticks_to_collapse ≥ COLLAPSE_WARNING_TICKS.
/// - `Healthy`:    net drain ≤ 0 (intake cubre extracción + pérdida).
pub fn detect_collapse(
    pool: f32,
    intake: f32,
    total_extracted: f32,
    loss: f32,
) -> PoolHealthStatus {
    if pool <= 0.0 { return PoolHealthStatus::Collapsed; }
    let net_drain = (total_extracted + loss) - intake;
    if net_drain > 0.0 {
        let ttc = ticks_to_collapse(pool, net_drain);
        if ttc < COLLAPSE_WARNING_TICKS {
            PoolHealthStatus::Collapsing
        } else {
            PoolHealthStatus::Stressed
        }
    } else {
        PoolHealthStatus::Healthy
    }
}

// ─── EC-5C: Predicción de Trayectoria ─────────────────────────────────────────

/// Proyección lineal del pool a tasa constante.
/// `net_drain_per_tick > 0` → pool decrece; `< 0` → pool crece.
/// `net_change_per_tick` almacenado con signo opuesto (positivo = creciendo).
pub fn predict_pool_trajectory(
    pool: f32,
    net_drain_per_tick: f32,
    capacity: f32,
) -> PoolTrajectory {
    let p   = pool.max(0.0);
    let cap = capacity.max(p);
    let ttc = if net_drain_per_tick > 0.0 {
        ticks_to_collapse(p, net_drain_per_tick)
    } else {
        u32::MAX
    };
    let ttf = if net_drain_per_tick < 0.0 {
        let gain = (-net_drain_per_tick).max(EXTRACTION_EPSILON);
        let room = (cap - p).max(0.0);
        if room <= 0.0 { 0 } else { (room / gain).ceil() as u32 }
    } else {
        u32::MAX
    };
    PoolTrajectory {
        ticks_to_collapse:  ttc,
        ticks_to_full:      ttf,
        net_change_per_tick: -net_drain_per_tick,
    }
}

// ─── Tests ───────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::blueprint::constants::COLLAPSE_WARNING_TICKS;

    // ── EC-5A: competition_matrix ────────────────────────────────────────────

    #[test]
    fn competition_matrix_zero_available_no_panic() {
        let m = competition_matrix(&[100.0, 100.0], 0.0);
        assert!(m[0][0].is_finite());
        assert!(m[0][1].is_finite());
    }

    #[test]
    fn competition_matrix_single_child() {
        let m = competition_matrix(&[100.0], 1000.0);
        // C[0][0] = 100 - 100 * 0 / 1000 = 100
        assert!((m[0][0] - 100.0).abs() < 1e-3, "got {}", m[0][0]);
        // All other cells zero (N=1, so no i!=j terms)
        for j in 1..MAX_COMPETITION_MATRIX {
            assert_eq!(m[0][j], 0.0);
        }
    }

    #[test]
    fn competition_matrix_uniform_extractions_symmetric() {
        let extr = [100.0, 100.0, 100.0];
        let m = competition_matrix(&extr, 600.0);
        // C[i][j] == C[j][i] for equal extractions → symmetric
        for i in 0..3 {
            for j in 0..3 {
                assert!((m[i][j] - m[j][i]).abs() < 1e-4, "not symmetric at [{i}][{j}]");
            }
        }
    }

    #[test]
    fn competition_matrix_diagonal_positive_typical() {
        let extr = [200.0, 100.0, 50.0];
        let m = competition_matrix(&extr, 1000.0);
        // Diagonal: C[i][i] = extr[i] - extr[i] * sum_others / avail
        // For i=0: 200 - 200*(150)/1000 = 200 - 30 = 170
        assert!(m[0][0] > 0.0, "diagonal must be positive: {}", m[0][0]);
        assert!(m[1][1] > 0.0);
        assert!(m[2][2] > 0.0);
    }

    #[test]
    fn competition_matrix_off_diagonal_negative() {
        let extr = [200.0, 100.0];
        let m = competition_matrix(&extr, 1000.0);
        // C[0][1] = -200/1000 = -0.2
        assert!(m[0][1] < 0.0, "off-diagonal must be negative: {}", m[0][1]);
        assert!(m[1][0] < 0.0);
    }

    // ── EC-5A: competition_intensity ────────────────────────────────────────

    #[test]
    fn competition_intensity_uniform_returns_zero() {
        let v = [100.0f32, 100.0, 100.0];
        let g = competition_intensity(&v);
        assert!(g.abs() < 1e-5, "uniform → 0.0, got {g}");
    }

    #[test]
    fn competition_intensity_monopoly_approx_two_thirds() {
        // [300, 0, 0]: one child takes all → Gini ≈ 0.667
        let v = [300.0f32, 0.0, 0.0];
        let g = competition_intensity(&v);
        assert!((g - 2.0 / 3.0).abs() < 0.01, "got {g}");
    }

    #[test]
    fn competition_intensity_empty_slice_returns_zero() {
        assert_eq!(competition_intensity(&[]), 0.0);
    }

    #[test]
    fn competition_intensity_all_zero_returns_zero() {
        assert_eq!(competition_intensity(&[0.0, 0.0, 0.0]), 0.0);
    }

    #[test]
    fn competition_intensity_range_always_unit() {
        let cases: &[&[f32]] = &[
            &[1.0, 2.0, 3.0, 4.0],
            &[1000.0, 0.0],
            &[50.0, 50.0],
            &[1.0],
        ];
        for v in cases {
            let g = competition_intensity(v);
            assert!(g >= 0.0 && g <= 1.0, "out of [0,1]: {g} for {v:?}");
        }
    }

    // ── EC-5B: detect_equilibrium ────────────────────────────────────────────

    #[test]
    fn detect_equilibrium_balanced() {
        assert!(detect_equilibrium(100.0, 90.0, 10.0, 1e-3));
    }

    #[test]
    fn detect_equilibrium_not_balanced() {
        assert!(!detect_equilibrium(100.0, 50.0, 10.0, 1e-3));
    }

    // ── EC-5B: detect_dominance ──────────────────────────────────────────────

    #[test]
    fn detect_dominance_clear_winner() {
        assert!(detect_dominance(&[500.0, 300.0, 200.0], 0, true));
    }

    #[test]
    fn detect_dominance_pool_not_viable_returns_false() {
        assert!(!detect_dominance(&[500.0, 300.0, 200.0], 0, false));
    }

    #[test]
    fn detect_dominance_tie_returns_false() {
        // [300, 300, 200]: index 0 ties with index 1 → not strictly dominant
        assert!(!detect_dominance(&[300.0, 300.0, 200.0], 0, true));
    }

    #[test]
    fn detect_dominance_empty_slice_returns_false() {
        assert!(!detect_dominance(&[], 0, true));
    }

    #[test]
    fn detect_dominance_out_of_bounds_returns_false() {
        assert!(!detect_dominance(&[100.0], 5, true));
    }

    // ── EC-5B: detect_collapse ───────────────────────────────────────────────

    #[test]
    fn detect_collapse_zero_pool_is_collapsed() {
        assert_eq!(detect_collapse(0.0, 100.0, 50.0, 5.0), PoolHealthStatus::Collapsed);
    }

    #[test]
    fn detect_collapse_healthy_when_intake_covers() {
        // intake=200 > extracted=50 + loss=10 → net_drain = -140 → Healthy
        assert_eq!(
            detect_collapse(1000.0, 200.0, 50.0, 10.0),
            PoolHealthStatus::Healthy
        );
    }

    #[test]
    fn detect_collapse_collapsing_imminent() {
        // pool=100, net_drain=(200+10)-10=200, ttc=ceil(100/200)=1 < 100
        assert_eq!(
            detect_collapse(100.0, 10.0, 200.0, 10.0),
            PoolHealthStatus::Collapsing
        );
    }

    #[test]
    fn detect_collapse_stressed_slow_drain() {
        // pool=20000, net_drain=(200+10)-50=160, ttc=ceil(20000/160)=125 >= 100
        assert_eq!(
            detect_collapse(20000.0, 50.0, 200.0, 10.0),
            PoolHealthStatus::Stressed
        );
    }

    #[test]
    fn detect_collapse_boundary_just_below_warning_is_collapsing() {
        // ttc = COLLAPSE_WARNING_TICKS - 1 → Collapsing
        let warn   = COLLAPSE_WARNING_TICKS as f32;
        let pool   = warn - 1.0;
        let drain  = 1.0;
        // ttc = ceil(pool/drain) = warn-1 < warn → Collapsing
        assert_eq!(
            detect_collapse(pool, 0.0, drain, 0.0),
            PoolHealthStatus::Collapsing
        );
    }

    #[test]
    fn detect_collapse_boundary_at_warning_is_stressed() {
        // ttc = COLLAPSE_WARNING_TICKS → Stressed (not < warning)
        let warn  = COLLAPSE_WARNING_TICKS as f32;
        let pool  = warn;
        let drain = 1.0;
        // ttc = ceil(warn/1) = warn → NOT < warn → Stressed
        assert_eq!(
            detect_collapse(pool, 0.0, drain, 0.0),
            PoolHealthStatus::Stressed
        );
    }

    // ── EC-5C: predict_pool_trajectory ──────────────────────────────────────

    #[test]
    fn predict_trajectory_draining_ten_ticks() {
        let t = predict_pool_trajectory(1000.0, 100.0, 2000.0);
        assert_eq!(t.ticks_to_collapse, 10);
        assert_eq!(t.ticks_to_full, u32::MAX);
    }

    #[test]
    fn predict_trajectory_filling_twenty_ticks() {
        let t = predict_pool_trajectory(1000.0, -50.0, 2000.0);
        assert_eq!(t.ticks_to_full, 20);
        assert_eq!(t.ticks_to_collapse, u32::MAX);
    }

    #[test]
    fn predict_trajectory_stable_both_max() {
        let t = predict_pool_trajectory(1000.0, 0.0, 2000.0);
        assert_eq!(t.ticks_to_collapse, u32::MAX);
        assert_eq!(t.ticks_to_full, u32::MAX);
    }

    #[test]
    fn predict_trajectory_already_full_zero_ticks_to_full() {
        let t = predict_pool_trajectory(2000.0, -50.0, 2000.0);
        assert_eq!(t.ticks_to_full, 0);
    }

    #[test]
    fn predict_trajectory_net_change_sign_convention() {
        // draining → net_change_per_tick should be negative
        let t = predict_pool_trajectory(500.0, 10.0, 1000.0);
        assert!(t.net_change_per_tick < 0.0, "draining: expected negative, got {}", t.net_change_per_tick);
        // filling → net_change_per_tick should be positive
        let t2 = predict_pool_trajectory(500.0, -10.0, 1000.0);
        assert!(t2.net_change_per_tick > 0.0, "filling: expected positive, got {}", t2.net_change_per_tick);
    }
}
