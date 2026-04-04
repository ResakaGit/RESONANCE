# TT-1: Estadísticas de Ventana Deslizante

**Objetivo:** Funciones puras de estadística sobre ventanas deslizantes: varianza, autocorrelación lag-1, entropía de Shannon, información de Fisher. Base matemática para todos los normalizadores del Telescopio.

**Estado:** ✅ COMPLETADO (2026-04-04)
**Esfuerzo:** Bajo (math pura, sin ECS)
**Bloqueado por:** —
**Desbloquea:** TT-3 (normalizers), TT-5 (telescope state)

---

## Entregables

### 1. `src/blueprint/equations/temporal_telescope.rs`

```rust
/// Varianza de una ventana de f32. Retorna 0.0 si len < 2.
pub fn sliding_variance(window: &[f32]) -> f32

/// Autocorrelación lag-1. Mide inercia del sistema. ρ₁ → 1.0 = critical slowing down.
pub fn sliding_autocorrelation_lag1(window: &[f32]) -> f32

/// Entropía de Shannon sobre distribución normalizada. H = -Σ pᵢ ln(pᵢ).
/// Input: slice de qe por celda (NO normalizado — la función normaliza internamente).
pub fn shannon_entropy(distribution: &[f32]) -> f32

/// Información de Fisher. Mide sensibilidad distribucional al cambio temporal.
/// F(t) = Σ (1/pᵢ) × (Δpᵢ/Δt)². Inputs: distribución actual y anterior.
pub fn fisher_information(current: &[f32], previous: &[f32], dt: f32) -> f32

/// Tasa de cambio de entropía (primera derivada, diferencia finita).
pub fn entropy_rate(h_current: f32, h_previous: f32, dt: f32) -> f32

/// Aceleración de entropía (segunda derivada, diferencia finita).
pub fn entropy_acceleration(h_current: f32, h_previous: f32, h_before_previous: f32, dt: f32) -> f32
```

### 2. `src/blueprint/constants/temporal_telescope.rs`

```rust
/// Tamaño de ventana para estadísticas (ticks). Derivado: potencia de 2 para bitwise modulo.
pub const TELESCOPE_WINDOW_SIZE: usize = 128;

/// Umbral de autocorrelación para "alta inercia". Derivado: exp(-DISSIPATION_SOLID × 10).
pub const RHO1_HIGH_INERTIA: f32 = /* derivar */;

/// Umbral de Fisher para "spike distribucional". Derivado: 3.0 (3 sigma sobre mediana).
pub const FISHER_SPIKE_MULTIPLIER: f32 = 3.0;

/// Epsilon para aceleración de entropía. Derivado: DISSIPATION_SOLID² (cuadrado de la menor tasa).
pub const ENTROPY_ACCELERATION_EPSILON: f32 = /* derivar */;
```

---

## Contrato stateless

Todas las funciones reciben `&[f32]` y retornan `f32`. Sin estado mutable. Sin side effects. Sin ECS. Sin Bevy. Testeables con `#[test]` puro.

---

## Preguntas para tests

1. `sliding_variance(&[1.0; 100])` → ¿debe retornar exactamente 0.0?
2. `sliding_variance(&[])` → ¿retorna 0.0 sin panic?
3. `sliding_autocorrelation_lag1` de una constante → ¿ρ₁ = 0.0 o NaN? (debe ser 0.0)
4. `sliding_autocorrelation_lag1` de una sinusoidal pura → ¿ρ₁ cercano a cos(2π/período)?
5. `shannon_entropy(&[1.0, 0.0, 0.0])` → ¿H = 0.0? (toda la energía en una celda)
6. `shannon_entropy(&[1.0, 1.0, 1.0, 1.0])` → ¿H = ln(4)? (distribución uniforme)
7. `shannon_entropy(&[0.0; 100])` → ¿retorna 0.0 sin NaN? (edge case: todo cero)
8. `fisher_information` con distribuciones idénticas → ¿F = 0.0?
9. `fisher_information` con distribución que cambió en 1 celda → ¿F proporcional al cambio²?
10. `entropy_acceleration` = 0 cuando H(t) es lineal en t
11. Todas las funciones son `const`-safe respecto a NaN (nunca retornan NaN para inputs finitos)

---

## Integración con código existente

- **Consume:** Nada (funciones puras aisladas)
- **Consumido por:** TT-3 (normalizers), TT-5 (telescope state), TT-7 (calibration bridge)
- **No modifica:** Ningún archivo existente
- **Patrón:** Mismo que `blueprint/equations/conservation.rs` — math pura sin dependencias ECS
