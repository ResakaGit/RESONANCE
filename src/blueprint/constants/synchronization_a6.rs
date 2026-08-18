//! Axioma 6 (Emergence at Scale) — constantes del experimento de sincronización.
//! Prueba de emergencia vía order parameter de Kuramoto + ablación causal.
//! Ver `blueprint/equations/emergence/synchronization.rs` y ADR-046.
//!
//! Dos familias, mismo axioma:
//! - `SYNC_*` — experimento analítico de fase, mean-field all-to-all (ADR-046).
//! - `SYNC_ECS_*` — probe del `entrainment_system` real sobre L2×N (ADR-047);
//!   acopla frecuencia, no fase, y su métrica es el colapso de σ_ω.

use super::entrainment_ac2::KURAMOTO_LOCK_THRESHOLD_HZ;
use crate::blueprint::equations::derived_thresholds::DISSIPATION_GAS;

/// Cuanto temporal del integrador de fase (Euler explícito).
/// Derivado de la escala de disipación de gas (Axioma 4): estabilidad exige
/// `coupling·dt ≪ 1`; con `K ≈ 0.4` da `0.4 × 0.08 = 0.032`.
pub const SYNC_DT: f32 = DISSIPATION_GAS;

/// Razón supercrítica del acoplamiento: `K = SYNC_SUPERCRITICAL_RATIO × freq_spread`.
/// El acoplamiento crítico mean-field es `K_c = σ·√(8/π) ≈ 1.6·σ`; 4× lo satura
/// con margen (~2.5·K_c), llevando R→~1 en el régimen acoplado.
pub const SYNC_SUPERCRITICAL_RATIO: f32 = 4.0;

/// Fracción final de la serie R(t) promediada como `r_final` (régimen estacionario).
pub const SYNC_TAIL_WINDOW_FRAC: f32 = 0.25;

/// Umbral de PASS: R en el régimen acoplado debe superar esto (sincronizado).
pub const SYNC_R_PASS_MIN: f32 = 0.5;

/// Umbral de control: R ablado (coupling = 0) debe quedar por debajo de esto.
pub const SYNC_ABLATED_R_MAX: f32 = 0.1;

/// Separación mínima exigida entre `r_final` acoplado y ablado.
/// Consistente por construcción: `SYNC_R_PASS_MIN - SYNC_ABLATED_R_MAX = 0.4`.
pub const SYNC_GAP_MIN: f32 = 0.4;

// ─── Probe ECS del motor real (ADR-047) ──────────────────────────────────────
// Real-engine ECS probe constants — `entrainment_system` over an L2 population.

/// Spread inicial de ω en unidades del umbral de lock del **motor bajo prueba**.
///
/// Cotas derivadas (ambas gateadas en `tests/r10_emergence_gates.rs`):
/// - `≫ 1` — garantiza arranque desordenado en la escala de la dinámica medida
///   (`KURAMOTO_LOCK_THRESHOLD_HZ`); con ratio ≤ 1 la población nacería ya lockeada
///   y el guard de `entrainment_system` la congelaría antes del primer paso.
/// - `≪ COHERENCE_BANDWIDTH / KURAMOTO_LOCK_THRESHOLD_HZ = 50` — mantiene la
///   población dentro de la ventana de coherencia de Ax8.
///
/// `[ASSUMPTION]` 8.0 es una calibración **dentro** de las cotas (1, 50), no una
/// derivación cerrada. Ver ADR-047 §4.
pub const SYNC_ECS_SPREAD_RATIO: f32 = 8.0;

/// Desviación estándar de la población inicial de frecuencias (Hz).
/// `SYNC_ECS_SPREAD_RATIO × KURAMOTO_LOCK_THRESHOLD_HZ` — derivada de la dinámica
/// bajo prueba (el umbral de lock del propio motor), NO de un ratio post-hoc
/// sobre `COHERENCE_BANDWIDTH`.
pub const SYNC_ECS_SPREAD_HZ: f32 = SYNC_ECS_SPREAD_RATIO * KURAMOTO_LOCK_THRESHOLD_HZ;

/// Umbral de PASS del colapso `S = 1 − σ_ω(T)/σ_ω(0)` en la condición acoplada.
///
/// Queda **debajo** del piso `S ≈ 0.75` del peor estado totalmente lockeado
/// (la "escalera congelada": el guard de lock es por PAR, así que un gradiente
/// sostenido de ≤ 1 Hz por hop a lo largo del diámetro del grafo — ~7 hops en la
/// grilla 8×8 — deja σ(T) ≈ 2 Hz), con margen para convergencia parcial.
pub const SYNC_ECS_S_PASS_MIN: f32 = 0.5;

/// Cota formal del colapso en las condiciones abladas (regla y alcance).
/// El probe exige de hecho `S == 0` exacto (identidad bit a bit de ω(0) vs ω(T));
/// esta cota vive sólo en el veredicto, tolerante a futuras fuentes de jitter.
pub const SYNC_ECS_ABLATED_S_MAX: f32 = 0.1;

/// Pasos de `Update` del probe antes de medir σ_ω(T).
///
/// Derivación (honesta): la contracción es **exponencial**. Por par vecino
/// `t ≈ ln(spread/lock) / (2·K_eff) = 2.079/0.154 ≈ 13.5` ticks con
/// `K_eff(d=8) = KURAMOTO_BASE_COUPLING · e^(−8/12) ≈ 0.077`. Pero la población
/// converge por el **modo más lento** del laplaciano del king-graph 8×8 con la
/// normalización `1/n` del paso (n = |vecinos| ≤ 8, no la población N):
/// tasa ~O(10⁻²)/tick → escala O(10²–10³). 400 deja el residuo del modo lento en
/// ~e⁻⁴ del spread. Valor **empírico inicial**, no derivación cerrada.
pub const SYNC_ECS_TICKS: u32 = 400;

/// Frecuencia central de la población del probe (Hz).
///
/// Don't-care dinámico: la regla del motor es lineal en las **diferencias** de ω,
/// invariante a traslación. Su única restricción física es
/// `SYNC_ECS_CENTER_HZ − 3·SYNC_ECS_SPREAD_HZ > 0` para que el clamp `max(0.0)` de
/// L2 `OscillatorySignature` sea inerte (sin offset, `gaussian_f32` centra en 0 y
/// la población nacería rectificada). Gateada en r10. 75.0 por precedente de los
/// tests de `simulation/emergence/entrainment.rs`.
pub const SYNC_ECS_CENTER_HZ: f32 = 75.0;
