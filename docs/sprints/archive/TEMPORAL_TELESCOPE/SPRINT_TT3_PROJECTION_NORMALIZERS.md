# TT-3: Normalizadores de Proyección

**Objetivo:** Funciones puras que normalizan la proyección del Telescopio según el régimen dinámico actual. Cada normalizador toma métricas crudas (de TT-1/TT-2) y retorna un factor o valor de proyección. El Telescopio los compone para producir su estado especulativo.

**Estado:** ✅ COMPLETADO (2026-04-04)
**Esfuerzo:** Medio (6 normalizadores, cada uno es 5-15 líneas de math)
**Bloqueado por:** TT-1 (tipos/funciones), TT-2 (hurst_dfa)
**Desbloquea:** TT-6 (projection engine)

---

## Entregable

### En `src/blueprint/equations/temporal_telescope.rs` (mismo archivo)

```rust
/// Métricas de régimen del Telescopio. Datos puros, sin estado.
#[derive(Clone, Copy, Debug, Default)]
pub struct RegimeMetrics {
    pub variance: f32,           // σ²(qe_total) — de sliding_variance
    pub autocorrelation: f32,    // ρ₁ — de sliding_autocorrelation_lag1
    pub hurst: f32,              // H — de hurst_dfa
    pub fisher: f32,             // F(t) — de fisher_information
    pub fisher_median: f32,      // mediana de F sobre ventana reciente
    pub entropy_accel: f32,      // d²H/dt² — de entropy_acceleration
    pub lambda_max: f32,         // eigenvalor dominante estimado
    pub population: f32,         // entidades vivas (normalizado)
    pub event_rate: f32,         // (muertes + nacimientos) / tick estimado
}

/// Pesos de los normalizadores. Calibrados por el puente.
#[derive(Clone, Copy, Debug)]
pub struct NormalizerWeights {
    pub hurst_weight: f32,       // cuánto pesa H en la extrapolación (default 1.0)
    pub inertia_weight: f32,     // cuánto pesa ρ₁ (default 1.0)
    pub fisher_sensitivity: f32, // multiplicador para F threshold (default 1.0)
    pub max_k: u32,              // K máximo permitido por las métricas actuales
}

/// Proyecta qe_total K ticks adelante usando Hurst + autocorrelación.
/// `trend` = dqe/dt promedio reciente.
pub fn project_qe(current: f32, trend: f32, metrics: &RegimeMetrics, weights: &NormalizerWeights, k: u32) -> f32

/// Proyecta population count K ticks adelante.
pub fn project_population(current: f32, trend: f32, metrics: &RegimeMetrics, weights: &NormalizerWeights, k: u32) -> f32

/// Densidad de eventos esperada en K ticks (McTaggart normalizer).
/// E = event_rate × K × population. Clasificación: B-series (E<1), borderline, A-series (E>10).
pub fn event_density(event_rate: f32, k: u32, population: f32) -> f32

/// Horizonte de confianza del Telescopio en ticks (Lyapunov normalizer).
/// horizon = |1/λ_max| si λ < 0. Si λ ≥ 0, retorna K_min.
pub fn confidence_horizon(lambda_max: f32, k_min: u32) -> u32

/// Estima λ_max desde ρ₁ y dt. λ ≈ ln(ρ₁) / dt.
pub fn estimate_lambda_max(rho1: f32, dt: f32) -> f32

/// Dado RegimeMetrics + Weights, calcula el K óptimo.
pub fn optimal_k(metrics: &RegimeMetrics, weights: &NormalizerWeights, k_min: u32, k_max: u32) -> u32
```

---

## Contrato stateless

Todas las funciones son `fn(inputs) → output`. `RegimeMetrics` y `NormalizerWeights` son `Copy` — stack-allocated, sin heap. Composición: `optimal_k` llama a `confidence_horizon`, `event_density`, y usa `hurst` + `autocorrelation` de `RegimeMetrics`.

---

## Preguntas para tests

1. `project_qe` con H=1.0, trend=+1.0, K=10 → ¿qe aumenta linealmente?
2. `project_qe` con H=0.5, trend=+1.0 → ¿qe ≈ current? (random walk, no extrapolar)
3. `project_qe` con H=0.0, trend=+1.0 → ¿qe baja? (anti-persistente, revertir)
4. `project_qe` resultado nunca negativo (qe ≥ 0.0 siempre)
5. `event_density` con rate=0, cualquier K → ¿E = 0.0?
6. `event_density` con rate=0.1, K=100, pop=10 → ¿E = 100? (A-series)
7. `confidence_horizon` con λ=-0.05 → ¿horizon = 20?
8. `confidence_horizon` con λ=+0.01 → ¿retorna k_min?
9. `estimate_lambda_max` con ρ₁=0.99, dt=1.0 → ¿λ ≈ -0.01?
10. `optimal_k` reduce K cuando event_density > 5
11. `optimal_k` aumenta K cuando H > 0.7 y σ² baja
12. `NormalizerWeights::default()` tiene todos los pesos en 1.0

---

## Integración

- **Consume:** TT-1 (`sliding_variance`, etc.), TT-2 (`hurst_dfa`)
- **Consumido por:** TT-6 (projection engine), TT-7 (calibration bridge ajusta weights)
- **No modifica:** Nada existente
