//! Matemática pura del Telescopio Temporal (ADR-015).
//! Pure math for the Temporal Telescope (ADR-015).
//!
//! Stateless functions: sliding statistics, Hurst DFA, projection normalizers.
//! No ECS, no Bevy, no side effects. All testable with `#[test]` alone.

// ─── TT-1: Sliding Window Statistics ─────────────────────────────────────────

/// Varianza de una ventana de f32. Retorna 0.0 si len < 2.
/// Variance of an f32 window. Returns 0.0 if len < 2.
///
/// `σ² = (1/N) Σ (xᵢ - x̄)²`
#[inline]
pub fn sliding_variance(window: &[f32]) -> f32 {
    let n = window.len();
    if n < 2 {
        return 0.0;
    }
    let inv_n = 1.0 / n as f32;
    let mean = window.iter().sum::<f32>() * inv_n;
    window.iter().map(|&x| (x - mean) * (x - mean)).sum::<f32>() * inv_n
}

/// Autocorrelación lag-1. Mide inercia del sistema. ρ₁ → 1.0 = critical slowing down.
/// Lag-1 autocorrelation. Measures system inertia. ρ₁ → 1.0 = critical slowing down.
///
/// `ρ₁ = Σ (xᵢ - x̄)(xᵢ₊₁ - x̄) / Σ (xᵢ - x̄)²`
#[inline]
pub fn sliding_autocorrelation_lag1(window: &[f32]) -> f32 {
    let n = window.len();
    if n < 3 {
        return 0.0;
    }
    let inv_n = 1.0 / n as f32;
    let mean = window.iter().sum::<f32>() * inv_n;
    let var: f32 = window.iter().map(|&x| (x - mean) * (x - mean)).sum();
    if var < f32::EPSILON {
        return 0.0;
    }
    let cov: f32 = window.windows(2)
        .map(|pair| (pair[0] - mean) * (pair[1] - mean))
        .sum();
    (cov / var).clamp(-1.0, 1.0)
}

/// Entropía de Shannon sobre distribución. H = -Σ pᵢ ln(pᵢ).
/// Shannon entropy over distribution. H = -Σ pᵢ ln(pᵢ).
///
/// Input: slice de qe por celda (NO normalizado). La función normaliza internamente.
/// Returns 0.0 if total is zero or slice is empty.
#[inline]
pub fn shannon_entropy(distribution: &[f32]) -> f32 {
    if distribution.is_empty() {
        return 0.0;
    }
    let total: f32 = distribution.iter().filter(|&&x| x > 0.0).sum();
    if total < f32::EPSILON {
        return 0.0;
    }
    let inv_total = 1.0 / total;
    distribution.iter()
        .filter(|&&x| x > 0.0)
        .map(|&x| {
            let p = x * inv_total;
            -p * p.ln()
        })
        .sum()
}

/// Información de Fisher. Mide sensibilidad distribucional al cambio temporal.
/// Fisher information. Measures distributional sensitivity to temporal change.
///
/// `F(t) = Σ (1/pᵢ) × (Δpᵢ/Δt)²`
#[inline]
pub fn fisher_information(current: &[f32], previous: &[f32], dt: f32) -> f32 {
    if current.len() != previous.len() || current.is_empty() || dt < f32::EPSILON {
        return 0.0;
    }
    let total_curr: f32 = current.iter().filter(|&&x| x > 0.0).sum();
    let total_prev: f32 = previous.iter().filter(|&&x| x > 0.0).sum();
    if total_curr < f32::EPSILON || total_prev < f32::EPSILON {
        return 0.0;
    }
    let inv_curr = 1.0 / total_curr;
    let inv_prev = 1.0 / total_prev;
    let inv_dt = 1.0 / dt;
    current.iter().zip(previous.iter())
        .map(|(&c, &p)| {
            let p_c = (c * inv_curr).max(f32::EPSILON);
            let p_p = p * inv_prev;
            let dp = (p_c - p_p) * inv_dt;
            dp * dp / p_c
        })
        .sum()
}

/// Tasa de cambio de entropía (primera derivada, diferencia finita).
/// Entropy rate (first derivative, finite difference).
#[inline]
pub fn entropy_rate(h_current: f32, h_previous: f32, dt: f32) -> f32 {
    if dt < f32::EPSILON {
        return 0.0;
    }
    (h_current - h_previous) / dt
}

/// Aceleración de entropía (segunda derivada, diferencia finita).
/// Entropy acceleration (second derivative, finite difference).
#[inline]
pub fn entropy_acceleration(h_current: f32, h_previous: f32, h_before_previous: f32, dt: f32) -> f32 {
    if dt < f32::EPSILON {
        return 0.0;
    }
    let dt_sq = dt * dt;
    (h_current - 2.0 * h_previous + h_before_previous) / dt_sq
}

// ─── TT-2: Hurst DFA ────────────────────────────────────────────────────────

/// Regresión lineal simple: pendiente y ordenada. Single-pass O(N).
/// Simple linear regression: slope and intercept. Single-pass O(N).
///
/// Retorna (0.0, 0.0) si len < 2 o longitudes distintas.
#[inline]
pub fn linear_regression(x: &[f32], y: &[f32]) -> (f32, f32) {
    let n = x.len().min(y.len());
    if n < 2 {
        return (0.0, 0.0);
    }
    let n_f = n as f32;
    let (sum_x, sum_y, sum_xy, sum_xx) = x.iter().zip(y.iter()).take(n)
        .fold((0.0_f32, 0.0_f32, 0.0_f32, 0.0_f32), |(sx, sy, sxy, sxx), (&xi, &yi)| {
            (sx + xi, sy + yi, sxy + xi * yi, sxx + xi * xi)
        });
    let denom = n_f * sum_xx - sum_x * sum_x;
    if denom.abs() < f32::EPSILON {
        return (0.0, sum_y / n_f);
    }
    let slope = (n_f * sum_xy - sum_x * sum_y) / denom;
    let intercept = (sum_y - slope * sum_x) / n_f;
    (slope, intercept)
}

/// Exponente de Hurst via Detrended Fluctuation Analysis.
/// Hurst exponent via Detrended Fluctuation Analysis.
///
/// Mide persistencia de la serie temporal.
/// H > 0.5 = persistente (tendencias continúan). H < 0.5 = anti-persistente.
/// Retorna 0.5 si datos insuficientes.
pub fn hurst_dfa(window: &[f32], min_box: usize, max_box: usize) -> f32 {
    let n = window.len();
    if n < min_box * 2 || min_box < 4 {
        return 0.5;
    }
    let max_box = max_box.min(n / 2);
    if max_box <= min_box {
        return 0.5;
    }

    // 1. Integrate: Y(k) = Σ (xᵢ - mean)
    let inv_n = 1.0 / n as f32;
    let mean = window.iter().sum::<f32>() * inv_n;
    let len = n.min(crate::blueprint::constants::temporal_telescope::DFA_MAX_SERIES);
    let mut integrated = [0.0_f32; crate::blueprint::constants::temporal_telescope::DFA_MAX_SERIES];
    let mut cumsum = 0.0_f32;
    for i in 0..len {
        cumsum += window[i] - mean;
        integrated[i] = cumsum;
    }

    // 2. Compute F(n) for logarithmically spaced box sizes
    let mut log_n = [0.0_f32; crate::blueprint::constants::temporal_telescope::DFA_MAX_SCALES];
    let mut log_f = [0.0_f32; crate::blueprint::constants::temporal_telescope::DFA_MAX_SCALES];
    let mut scale_count = 0;

    let log_min = (min_box as f32).ln();
    let log_max = (max_box as f32).ln();
    let spacing = crate::blueprint::constants::temporal_telescope::DFA_LOG_SPACING;
    let n_scales = ((log_max - log_min) / spacing.ln().abs()).ceil() as usize;
    let n_scales = n_scales.clamp(3, crate::blueprint::constants::temporal_telescope::DFA_MAX_SCALES);
    let step = (log_max - log_min) / (n_scales - 1) as f32;

    for s in 0..n_scales {
        let box_size = (log_min + s as f32 * step).exp() as usize;
        let box_size = box_size.clamp(min_box, max_box);
        let num_boxes = len / box_size;
        if num_boxes < 1 {
            continue;
        }

        let mut total_fluct = 0.0_f32;
        for b in 0..num_boxes {
            let start = b * box_size;
            // Linear detrend within box using local regression
            let mut sx = 0.0_f32;
            let mut sy = 0.0_f32;
            let mut sxy = 0.0_f32;
            let mut sxx = 0.0_f32;
            for j in 0..box_size {
                let xj = j as f32;
                let yj = integrated[start + j];
                sx += xj;
                sy += yj;
                sxy += xj * yj;
                sxx += xj * xj;
            }
            let bs = box_size as f32;
            let denom = bs * sxx - sx * sx;
            let (a, b_coef) = if denom.abs() > f32::EPSILON {
                let slope = (bs * sxy - sx * sy) / denom;
                let intercept = (sy - slope * sx) / bs;
                (slope, intercept)
            } else {
                (0.0, sy / bs)
            };

            let mut mse = 0.0_f32;
            for j in 0..box_size {
                let trend = a * j as f32 + b_coef;
                let residual = integrated[start + j] - trend;
                mse += residual * residual;
            }
            total_fluct += mse / bs;
        }

        let f_n = (total_fluct / num_boxes as f32).sqrt();
        if f_n > f32::EPSILON && box_size > 0 {
            log_n[scale_count] = (box_size as f32).ln();
            log_f[scale_count] = f_n.ln();
            scale_count += 1;
        }
    }

    if scale_count < 2 {
        return 0.5;
    }

    // 3. H = slope of log(F) vs log(n)
    let (slope, _) = linear_regression(&log_n[..scale_count], &log_f[..scale_count]);
    slope.clamp(0.0, 2.0)
}

// ─── TT-3: Projection Normalizers ────────────────────────────────────────────

/// Métricas de régimen del Telescopio. Datos puros, sin estado.
/// Telescope regime metrics. Pure data, no state.
#[derive(Clone, Copy, Debug, Default)]
pub struct RegimeMetrics {
    /// σ²(qe_total) — de sliding_variance.
    pub variance: f32,
    /// ρ₁ — de sliding_autocorrelation_lag1.
    pub autocorrelation: f32,
    /// H — de hurst_dfa.
    pub hurst: f32,
    /// F(t) — de fisher_information.
    pub fisher: f32,
    /// Mediana de F sobre ventana reciente.
    pub fisher_median: f32,
    /// d²H/dt² — de entropy_acceleration.
    pub entropy_accel: f32,
    /// Eigenvalor dominante estimado.
    pub lambda_max: f32,
    /// Entidades vivas (normalizado [0,1]).
    pub population: f32,
    /// (muertes + nacimientos) / tick estimado.
    pub event_rate: f32,
}

/// Pesos de los normalizadores. Calibrados por el puente.
/// Normalizer weights. Calibrated by the bridge.
#[derive(Clone, Copy, Debug)]
pub struct NormalizerWeights {
    /// Cuánto pesa H en la extrapolación. Default 1.0.
    pub hurst_weight: f32,
    /// Cuánto pesa ρ₁. Default 1.0.
    pub inertia_weight: f32,
    /// Multiplicador para F threshold. Default 1.0.
    pub fisher_sensitivity: f32,
    /// K máximo permitido por las métricas actuales.
    pub max_k: u32,
}

impl Default for NormalizerWeights {
    fn default() -> Self {
        Self {
            hurst_weight: 1.0,
            inertia_weight: 1.0,
            fisher_sensitivity: 1.0,
            max_k: crate::blueprint::constants::temporal_telescope::TELESCOPE_K_MAX,
        }
    }
}

/// Proyecta qe_total K ticks adelante usando Hurst + autocorrelación.
/// Projects qe_total K ticks ahead using Hurst + autocorrelation.
///
/// `trend` = dqe/dt promedio reciente.
#[inline]
pub fn project_qe(current: f32, trend: f32, metrics: &RegimeMetrics, weights: &NormalizerWeights, k: u32) -> f32 {
    if k == 0 {
        return current;
    }
    let h = metrics.hurst.clamp(0.0, 1.0);
    let rho = metrics.autocorrelation.clamp(0.0, 1.0);
    // Hurst-weighted extrapolation: persistent H extrapolates, anti-persistent reverts
    let hurst_factor = (2.0 * h - 1.0) * weights.hurst_weight;
    // Inertia blends between current state and trend-projected state
    let inertia = rho * weights.inertia_weight;
    let projected_delta = trend * k as f32 * hurst_factor.clamp(-1.0, 1.0);
    let blended = current * inertia + (current + projected_delta) * (1.0 - inertia);
    blended.max(0.0)
}

/// Proyecta population count K ticks adelante.
/// Projects population count K ticks ahead.
#[inline]
pub fn project_population(current: f32, trend: f32, metrics: &RegimeMetrics, weights: &NormalizerWeights, k: u32) -> f32 {
    project_qe(current, trend, metrics, weights, k).max(0.0)
}

/// Densidad de eventos esperada en K ticks (McTaggart normalizer).
/// Expected event density in K ticks (McTaggart normalizer).
///
/// `E = event_rate × K × population`
#[inline]
pub fn event_density(event_rate: f32, k: u32, population: f32) -> f32 {
    event_rate.max(0.0) * k as f32 * population.max(0.0)
}

/// Horizonte de confianza del Telescopio en ticks (Lyapunov normalizer).
/// Telescope confidence horizon in ticks (Lyapunov normalizer).
///
/// `horizon = |1/λ_max|` si λ < 0. Si λ ≥ 0, retorna k_min.
#[inline]
pub fn confidence_horizon(lambda_max: f32, k_min: u32) -> u32 {
    if lambda_max >= 0.0 {
        return k_min;
    }
    let horizon = (1.0 / lambda_max.abs()) as u32;
    horizon.max(k_min)
}

/// Estima λ_max desde ρ₁ y dt. λ ≈ ln(ρ₁) / dt.
/// Estimates λ_max from ρ₁ and dt. λ ≈ ln(ρ₁) / dt.
#[inline]
pub fn estimate_lambda_max(rho1: f32, dt: f32) -> f32 {
    if dt < f32::EPSILON || rho1 <= 0.0 || rho1 >= 1.0 {
        return 0.0;
    }
    rho1.ln() / dt
}

/// Dado RegimeMetrics + Weights, calcula el K óptimo.
/// Given RegimeMetrics + Weights, computes optimal K.
pub fn optimal_k(metrics: &RegimeMetrics, weights: &NormalizerWeights, k_min: u32, k_max: u32) -> u32 {
    let horizon = confidence_horizon(metrics.lambda_max, k_min);
    let e_density = event_density(metrics.event_rate, k_max, metrics.population);

    // Start from horizon, reduce for high event density and Fisher spikes
    let mut k = horizon.min(k_max);

    use crate::blueprint::constants::temporal_telescope as c;

    // High event density → reduce K (many discrete events → ordering matters)
    if e_density > c::EVENT_DENSITY_HIGH {
        k /= c::OPTIMAL_K_HIGH_EVENT_DIVISOR;
    } else if e_density > c::EVENT_DENSITY_MEDIUM {
        k /= c::OPTIMAL_K_MEDIUM_EVENT_DIVISOR;
    }

    // Fisher spike → reduce K (distribution reshaping in progress)
    if metrics.fisher > metrics.fisher_median * c::FISHER_SPIKE_MULTIPLIER
        && metrics.fisher_median > f32::EPSILON
    {
        k /= c::OPTIMAL_K_FISHER_DIVISOR;
    }

    // Entropy accelerating → reduce K
    if metrics.entropy_accel.abs() > c::ENTROPY_ACCELERATION_EPSILON {
        k = k * c::OPTIMAL_K_ENTROPY_NUMERATOR / c::OPTIMAL_K_ENTROPY_DENOMINATOR;
    }

    // High Hurst + low variance → safe to project far
    if metrics.hurst > c::HURST_SAFE_PERSISTENCE && metrics.variance < f32::EPSILON {
        k = (k as f32 * c::TELESCOPE_K_GROW_FACTOR) as u32;
    }

    k.clamp(k_min, k_max.min(weights.max_k))
}

// ─── MT-1: Quantum-Inspired Functions (ADR-016) ─────────────────────────────

/// Visibilidad especulativa (coherencia) del nivel. Englert: D²+V²≤1.
/// Speculative visibility (coherence) of a level. Englert: D²+V²≤1.
///
/// V=0: colapsado (certeza total). V=1: onda pura (máxima incertidumbre).
/// D = e^{-ticks/coherence_length}, V = sqrt(1 - D²).
#[inline]
pub fn speculative_visibility(ticks_to_anchor: u64, coherence_length: f32) -> f32 {
    if coherence_length <= 0.0 {
        return 1.0; // sin coherencia → pura onda
    }
    let d = (-(ticks_to_anchor as f32) / coherence_length).exp();
    (1.0 - d * d).sqrt().clamp(0.0, 1.0)
}

/// Proyección conservation-bounded. Axioma 4+5.
/// Conservation-bounded projection. Axiom 4+5.
///
/// Clamp: base_decay ≤ resultado ≤ current_qe.
/// La disipación siempre reduce (Axioma 4). La proyección nunca supera el input (Axioma 5).
#[inline]
pub fn conservation_bounded_project(current_qe: f32, base_decay: f32, projected: f32) -> f32 {
    let floor = base_decay.min(current_qe);
    let ceiling = current_qe;
    projected.clamp(floor, ceiling)
}

/// Tasa de decay efectiva modulada por resonancia solar. Axioma 8.
/// Effective decay rate modulated by solar resonance. Axiom 8.
///
/// Entidades resonantes decaen menos (subsidiadas por fotosíntesis).
/// `effective = base × (1 - resonance × efficiency)`
#[inline]
pub fn frequency_aware_decay_rate(
    base_dissipation: f32,
    entity_freq: f32,
    solar_freq: f32,
    solar_bandwidth: f32,
    photosynthesis_efficiency: f32,
) -> f32 {
    let resonance = crate::blueprint::equations::determinism::gaussian_frequency_alignment(
        entity_freq,
        solar_freq,
        solar_bandwidth,
    );
    let subsidy = resonance * photosynthesis_efficiency.clamp(0.0, 1.0);
    (base_dissipation * (1.0 - subsidy)).max(0.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── TT-1: Sliding statistics ─────────────────────────────────

    #[test]
    fn variance_constant_is_zero() {
        assert_eq!(sliding_variance(&[1.0; 100]), 0.0);
    }

    #[test]
    fn variance_empty_is_zero() {
        assert_eq!(sliding_variance(&[]), 0.0);
    }

    #[test]
    fn variance_single_is_zero() {
        assert_eq!(sliding_variance(&[42.0]), 0.0);
    }

    #[test]
    fn variance_known_value() {
        // [1, 2, 3, 4, 5] → mean=3, var = (4+1+0+1+4)/5 = 2.0
        let v = sliding_variance(&[1.0, 2.0, 3.0, 4.0, 5.0]);
        assert!((v - 2.0).abs() < 1e-5, "expected 2.0, got {v}");
    }

    #[test]
    fn autocorrelation_constant_is_zero() {
        assert_eq!(sliding_autocorrelation_lag1(&[5.0; 50]), 0.0);
    }

    #[test]
    fn autocorrelation_empty_is_zero() {
        assert_eq!(sliding_autocorrelation_lag1(&[]), 0.0);
    }

    #[test]
    fn autocorrelation_alternating_is_negative() {
        let alt: Vec<f32> = (0..100).map(|i| if i % 2 == 0 { 1.0 } else { -1.0 }).collect();
        let rho = sliding_autocorrelation_lag1(&alt);
        assert!(rho < -0.9, "alternating should be strongly negative: {rho}");
    }

    #[test]
    fn entropy_uniform_is_ln_n() {
        let uniform = [1.0_f32; 4];
        let h = shannon_entropy(&uniform);
        let expected = (4.0_f32).ln();
        assert!((h - expected).abs() < 1e-5, "expected ln(4)={expected}, got {h}");
    }

    #[test]
    fn entropy_concentrated_is_zero() {
        let h = shannon_entropy(&[1.0, 0.0, 0.0]);
        assert!(h.abs() < 1e-5, "single cell should have H=0, got {h}");
    }

    #[test]
    fn entropy_all_zero_is_zero() {
        assert_eq!(shannon_entropy(&[0.0; 100]), 0.0);
    }

    #[test]
    fn entropy_empty_is_zero() {
        assert_eq!(shannon_entropy(&[]), 0.0);
    }

    #[test]
    fn fisher_identical_is_zero() {
        let d = [1.0, 2.0, 3.0];
        let f = fisher_information(&d, &d, 1.0);
        assert!(f < 1e-5, "identical distributions should have F=0, got {f}");
    }

    #[test]
    fn fisher_change_is_positive() {
        let prev = [1.0, 1.0, 1.0];
        let curr = [2.0, 1.0, 1.0];
        let f = fisher_information(&curr, &prev, 1.0);
        assert!(f > 0.0, "changed distribution should have F>0, got {f}");
    }

    #[test]
    fn entropy_acceleration_linear_is_zero() {
        // H values on a line: 1.0, 2.0, 3.0 → d²H/dt² = 0
        let a = entropy_acceleration(3.0, 2.0, 1.0, 1.0);
        assert!(a.abs() < 1e-5, "linear entropy should have zero acceleration: {a}");
    }

    #[test]
    fn entropy_rate_positive_for_increasing() {
        let r = entropy_rate(2.0, 1.0, 1.0);
        assert!((r - 1.0).abs() < 1e-5);
    }

    #[test]
    fn all_statistics_no_nan() {
        let window = [0.0, 1.0, f32::MIN_POSITIVE, 1000.0];
        assert!(sliding_variance(&window).is_finite());
        assert!(sliding_autocorrelation_lag1(&window).is_finite());
        assert!(shannon_entropy(&window).is_finite());
        assert!(fisher_information(&window, &[1.0; 4], 1.0).is_finite());
        assert!(entropy_rate(1.0, 0.0, 1.0).is_finite());
        assert!(entropy_acceleration(1.0, 0.0, 0.0, 1.0).is_finite());
    }

    // ── TT-2: Hurst DFA ─────────────────────────────────────────

    #[test]
    fn linear_regression_colinear() {
        let x = [0.0, 1.0, 2.0, 3.0];
        let y = [1.0, 3.0, 5.0, 7.0]; // y = 2x + 1
        let (slope, intercept) = linear_regression(&x, &y);
        assert!((slope - 2.0).abs() < 1e-4, "slope: {slope}");
        assert!((intercept - 1.0).abs() < 1e-4, "intercept: {intercept}");
    }

    #[test]
    fn linear_regression_empty() {
        assert_eq!(linear_regression(&[], &[]), (0.0, 0.0));
    }

    #[test]
    fn hurst_empty_returns_half() {
        assert_eq!(hurst_dfa(&[], 8, 128), 0.5);
    }

    #[test]
    fn hurst_short_returns_half() {
        assert_eq!(hurst_dfa(&[1.0, 2.0, 3.0], 8, 128), 0.5);
    }

    #[test]
    fn hurst_constant_series() {
        let constant = [1.0_f32; 256];
        let h = hurst_dfa(&constant, 8, 64);
        // Constant series → integrated = 0 → F(n) ≈ 0 → H undefined, fallback 0.5
        // With noise-free data, DFA can return edge values; just verify finite & bounded.
        assert!(h >= 0.0 && h <= 2.0, "H should be bounded: {h}");
    }

    #[test]
    fn hurst_deterministic() {
        let series: Vec<f32> = (0..256).map(|i| (i as f32 * 0.1).sin()).collect();
        let h1 = hurst_dfa(&series, 8, 64);
        let h2 = hurst_dfa(&series, 8, 64);
        assert_eq!(h1, h2, "DFA must be deterministic");
    }

    // ── TT-3: Projection Normalizers ─────────────────────────────

    #[test]
    fn project_qe_zero_k_returns_current() {
        let m = RegimeMetrics::default();
        let w = NormalizerWeights::default();
        assert_eq!(project_qe(100.0, 1.0, &m, &w, 0), 100.0);
    }

    #[test]
    fn project_qe_never_negative() {
        let m = RegimeMetrics { hurst: 0.0, autocorrelation: 0.0, ..Default::default() };
        let w = NormalizerWeights::default();
        let result = project_qe(10.0, -100.0, &m, &w, 1000);
        assert!(result >= 0.0, "qe must be non-negative: {result}");
    }

    #[test]
    fn event_density_zero_rate() {
        assert_eq!(event_density(0.0, 100, 50.0), 0.0);
    }

    #[test]
    fn event_density_known_value() {
        let e = event_density(0.1, 100, 10.0);
        assert!((e - 100.0).abs() < 1e-3, "expected 100.0, got {e}");
    }

    #[test]
    fn confidence_horizon_contractive() {
        let h = confidence_horizon(-0.05, 4);
        assert_eq!(h, 20);
    }

    #[test]
    fn confidence_horizon_expansive_returns_kmin() {
        let h = confidence_horizon(0.01, 4);
        assert_eq!(h, 4);
    }

    #[test]
    fn estimate_lambda_from_rho() {
        let lambda = estimate_lambda_max(0.99, 1.0);
        assert!((lambda - (-0.01005)).abs() < 0.001, "expected ≈-0.01, got {lambda}");
    }

    #[test]
    fn estimate_lambda_edge_cases() {
        assert_eq!(estimate_lambda_max(0.0, 1.0), 0.0);
        assert_eq!(estimate_lambda_max(1.0, 1.0), 0.0);
        assert_eq!(estimate_lambda_max(0.5, 0.0), 0.0);
    }

    #[test]
    fn optimal_k_respects_bounds() {
        let m = RegimeMetrics { lambda_max: -0.01, event_rate: 0.0, population: 0.0, ..Default::default() };
        let w = NormalizerWeights::default();
        let k = optimal_k(&m, &w, 4, 1024);
        assert!(k >= 4 && k <= 1024, "K out of bounds: {k}");
    }

    #[test]
    fn optimal_k_reduces_for_high_event_density() {
        let low_events = RegimeMetrics { lambda_max: -0.001, event_rate: 0.0, population: 10.0, ..Default::default() };
        let high_events = RegimeMetrics { lambda_max: -0.001, event_rate: 1.0, population: 10.0, ..Default::default() };
        let w = NormalizerWeights::default();
        let k_low = optimal_k(&low_events, &w, 4, 512);
        let k_high = optimal_k(&high_events, &w, 4, 512);
        assert!(k_high < k_low, "high events should reduce K: low={k_low}, high={k_high}");
    }

    #[test]
    fn normalizer_weights_default() {
        let w = NormalizerWeights::default();
        assert_eq!(w.hurst_weight, 1.0);
        assert_eq!(w.inertia_weight, 1.0);
        assert_eq!(w.fisher_sensitivity, 1.0);
    }

    // ── Axiom Property Tests ─────────────────────────────────────

    #[test]
    fn axiom5_project_qe_neutral_hurst_never_increases() {
        // Axiom 5: with H=0.5 (neutral), no trend extrapolation → qe ≤ current.
        let m = RegimeMetrics { hurst: 0.5, autocorrelation: 0.5, ..Default::default() };
        let w = NormalizerWeights::default();
        for k in [1, 10, 100, 1000] {
            let result = project_qe(100.0, 0.0, &m, &w, k);
            assert!(result <= 100.0 + f32::EPSILON,
                "Axiom 5: neutral H projection must not create energy: {result} at K={k}");
        }
    }

    #[test]
    fn axiom5_project_qe_anti_persistent_decreases() {
        // H=0.0 (anti-persistent) with positive trend → revert → qe decreases.
        let m = RegimeMetrics { hurst: 0.0, autocorrelation: 0.0, ..Default::default() };
        let w = NormalizerWeights::default();
        let result = project_qe(100.0, 1.0, &m, &w, 100);
        assert!(result < 100.0, "anti-persistent should decrease: {result}");
    }

    #[test]
    fn axiom5_project_qe_persistent_with_negative_trend() {
        // H=1.0 (persistent) with negative trend → extrapolate downward.
        let m = RegimeMetrics { hurst: 1.0, autocorrelation: 0.0, ..Default::default() };
        let w = NormalizerWeights::default();
        let result = project_qe(100.0, -1.0, &m, &w, 50);
        assert!(result < 100.0, "persistent negative trend should decrease: {result}");
    }

    #[test]
    fn hurst_dfa_white_noise_near_half() {
        // White noise should produce H ≈ 0.5 ± 0.2.
        use crate::blueprint::equations::determinism;
        let mut state = 42_u64;
        let noise: Vec<f32> = (0..512).map(|_| {
            state = determinism::next_u64(state);
            determinism::unit_f32(state) - 0.5
        }).collect();
        let h = hurst_dfa(&noise, 8, 128);
        assert!(h > 0.2 && h < 0.8,
            "white noise H should be near 0.5: {h}");
    }

    #[test]
    fn hurst_dfa_random_walk_above_half() {
        // Random walk (cumsum of white noise) should produce H > 0.5.
        use crate::blueprint::equations::determinism;
        let mut state = 42_u64;
        let mut walk = Vec::with_capacity(512);
        let mut cumsum = 0.0_f32;
        for _ in 0..512 {
            state = determinism::next_u64(state);
            cumsum += determinism::unit_f32(state) - 0.5;
            walk.push(cumsum);
        }
        let h = hurst_dfa(&walk, 8, 128);
        assert!(h > 0.7, "random walk H should be > 0.7: {h}");
    }

    #[test]
    fn shannon_entropy_bounded_by_ln_n() {
        // H ≤ ln(N) for any distribution of N bins.
        let dist = [3.0, 1.0, 4.0, 1.0, 5.0, 9.0, 2.0, 6.0];
        let h = shannon_entropy(&dist);
        let max_h = (dist.len() as f32).ln();
        assert!(h <= max_h + 1e-5, "entropy should be ≤ ln(N): {h} > {max_h}");
    }

    #[test]
    fn autocorrelation_trending_is_positive() {
        // Monotonically increasing series → positive autocorrelation.
        let trending: Vec<f32> = (0..100).map(|i| i as f32).collect();
        let rho = sliding_autocorrelation_lag1(&trending);
        assert!(rho > 0.9, "trending series should have high positive rho: {rho}");
    }

    #[test]
    fn optimal_k_combined_reductions_dont_underflow() {
        // All reduction paths fire simultaneously → K should still be ≥ K_MIN.
        let m = RegimeMetrics {
            lambda_max: -0.0001,  // very weak contraction → large horizon
            event_rate: 1.0,
            population: 100.0,   // E = 1.0 × 1024 × 100 >> 5 (high density)
            fisher: 100.0,
            fisher_median: 1.0,  // spike: 100 > 3 × 1 (Fisher spike)
            entropy_accel: 1.0,  // > ENTROPY_ACCELERATION_EPSILON
            ..Default::default()
        };
        let w = NormalizerWeights::default();
        let k = optimal_k(&m, &w, 4, 1024);
        assert!(k >= 4, "K should never go below K_MIN even with all reductions: {k}");
    }

    // ── Integration Tests ────────────────────────────────────────

    #[test]
    fn regime_metrics_to_optimal_k_to_project_qe_pipeline() {
        // Full pipeline: metrics → optimal_k → project_qe.
        let m = RegimeMetrics {
            hurst: 0.8,
            autocorrelation: 0.3,
            lambda_max: -0.01,
            event_rate: 0.01,
            population: 0.5,
            ..Default::default()
        };
        let w = NormalizerWeights::default();
        let k = optimal_k(&m, &w, 4, 1024);
        assert!(k > 4, "with good metrics, K should be > K_MIN: {k}");

        let projected = project_qe(100.0, -0.1, &m, &w, k);
        assert!(projected.is_finite(), "projection must be finite");
        assert!(projected >= 0.0, "projection must be non-negative");
    }

    // ── MT-1: Quantum-Inspired Functions ─────────────────────────

    #[test]
    fn visibility_at_anchor_is_zero() {
        let v = speculative_visibility(0, 100.0);
        assert!(v < 0.01, "at anchor, V should be ~0: {v}");
    }

    #[test]
    fn visibility_far_away_is_one() {
        let v = speculative_visibility(1_000_000, 100.0);
        assert!(v > 0.99, "far from anchor, V should be ~1: {v}");
    }

    #[test]
    fn visibility_mid_range() {
        // ticks=100, cl=100 → D=e⁻¹≈0.368, V=sqrt(1-0.135)≈0.93
        let v = speculative_visibility(100, 100.0);
        assert!(v > 0.5 && v < 1.0, "mid-range V: {v}");
    }

    #[test]
    fn visibility_always_bounded() {
        for ticks in [0, 1, 10, 100, 1000, 1_000_000, u64::MAX / 2] {
            for cl in [0.1, 1.0, 100.0, 10_000.0] {
                let v = speculative_visibility(ticks, cl);
                assert!(v >= 0.0 && v <= 1.0, "V out of bounds at ticks={ticks}, cl={cl}: {v}");
            }
        }
    }

    #[test]
    fn englert_duality_holds() {
        // D² + V² ≤ 1 for all inputs
        for ticks in [0, 5, 50, 500, 5000, 50000] {
            let cl = 100.0_f32;
            let d = (-(ticks as f32) / cl).exp();
            let v = speculative_visibility(ticks, cl);
            let sum = d * d + v * v;
            assert!(sum <= 1.0 + 1e-5, "Englert violated at ticks={ticks}: D²+V²={sum}");
        }
    }

    #[test]
    fn visibility_zero_coherence_is_pure_wave() {
        assert_eq!(speculative_visibility(50, 0.0), 1.0);
    }

    #[test]
    fn conservation_bounded_within_range() {
        assert_eq!(conservation_bounded_project(100.0, 90.0, 95.0), 95.0);
    }

    #[test]
    fn conservation_bounded_clamps_above() {
        assert_eq!(conservation_bounded_project(100.0, 90.0, 120.0), 100.0);
    }

    #[test]
    fn conservation_bounded_clamps_below() {
        assert_eq!(conservation_bounded_project(100.0, 90.0, 80.0), 90.0);
    }

    #[test]
    fn conservation_bounded_never_exceeds_input() {
        use crate::blueprint::equations::determinism;
        let mut state = 42_u64;
        for _ in 0..1000 {
            state = determinism::next_u64(state);
            let current = determinism::unit_f32(state) * 200.0;
            state = determinism::next_u64(state);
            let decay = determinism::unit_f32(state) * current;
            state = determinism::next_u64(state);
            let projected = determinism::unit_f32(state) * 300.0;
            let result = conservation_bounded_project(current, decay, projected);
            assert!(result <= current, "exceeded input: {result} > {current}");
            assert!(result >= decay.min(current), "below floor: {result} < {}", decay.min(current));
        }
    }

    #[test]
    fn freq_aware_decay_resonant_is_lower() {
        let base = 0.01;
        let resonant = frequency_aware_decay_rate(base, 400.0, 400.0, 200.0, 0.4);
        let disonant = frequency_aware_decay_rate(base, 50.0, 400.0, 200.0, 0.4);
        assert!(resonant < disonant, "resonant should decay less: {resonant} >= {disonant}");
    }

    #[test]
    fn freq_aware_decay_never_negative() {
        let r = frequency_aware_decay_rate(0.01, 400.0, 400.0, 200.0, 1.0);
        assert!(r >= 0.0, "decay rate should never be negative: {r}");
    }

    #[test]
    fn freq_aware_decay_no_efficiency_equals_base() {
        let base = 0.05;
        let r = frequency_aware_decay_rate(base, 400.0, 400.0, 200.0, 0.0);
        assert!((r - base).abs() < 1e-6, "zero efficiency should equal base: {r} vs {base}");
    }

    #[test]
    fn freq_aware_decay_always_le_base() {
        for freq in [50.0, 100.0, 200.0, 400.0, 800.0, 1000.0] {
            let base = 0.02;
            let r = frequency_aware_decay_rate(base, freq, 400.0, 200.0, 0.4);
            assert!(r <= base + 1e-6, "freq={freq}: decay {r} > base {base}");
        }
    }
}
