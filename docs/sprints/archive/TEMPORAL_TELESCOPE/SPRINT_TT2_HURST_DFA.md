# TT-2: Exponente de Hurst via DFA

**Objetivo:** Implementar Detrended Fluctuation Analysis (DFA) para computar el exponente de Hurst H de una serie temporal. H mide persistencia: H > 0.5 = tendencias persisten (seguro extrapolar), H < 0.5 = tendencias se revierten.

**Estado:** ✅ COMPLETADO (2026-04-04)
**Esfuerzo:** Medio (algoritmo DFA es O(N×log N), requiere regresión lineal)
**Bloqueado por:** —
**Desbloquea:** TT-3 (normalizers — el Hurst normalizer depende de H)

---

## Entregable

### En `src/blueprint/equations/temporal_telescope.rs` (mismo archivo que TT-1)

```rust
/// Exponente de Hurst via Detrended Fluctuation Analysis.
/// Mide persistencia de la serie temporal.
///
/// Algoritmo:
///   1. Integrar serie: Y(k) = Σᵢ₌₁ᵏ (xᵢ - x̄)
///   2. Dividir en ventanas de tamaño n
///   3. En cada ventana: fit lineal, computar residuo
///   4. F(n) = sqrt(mean(residuos²))
///   5. Repetir para n ∈ [min_box, max_box]
///   6. H = pendiente de log(F) vs log(n)
///
/// Retorna H ∈ [0, 1]. Returns 0.5 si la serie es muy corta.
pub fn hurst_dfa(window: &[f32], min_box: usize, max_box: usize) -> f32

/// Regresión lineal simple: pendiente de y = mx + b.
/// Auxiliar para DFA y para fits dentro de ventanas.
/// Retorna (slope, intercept). Retorna (0.0, 0.0) si len < 2.
pub fn linear_regression(x: &[f32], y: &[f32]) -> (f32, f32)
```

---

## Contrato stateless

`hurst_dfa` recibe `&[f32]` y retorna `f32`. Sin allocaciones de heap — usa arrays stack-allocated para las escalas log-space (máximo ~10 puntos para box sizes entre 8 y 512). `linear_regression` es O(N) single-pass.

---

## Preguntas para tests

1. `hurst_dfa` de ruido blanco (generado con determinism::next_u64) → ¿H ≈ 0.5 ± 0.1?
2. `hurst_dfa` de random walk (suma acumulada de ruido blanco) → ¿H ≈ 1.0 ± 0.15? (nota: DFA de random walk da α ≈ 1.5, pero H del incremento = 0.5; el walk mismo tiene H ≈ 1.0)
3. `hurst_dfa` de serie constante → ¿H = 0.5? (no hay tendencia)
4. `hurst_dfa` de serie con tendencia lineal perfecta → ¿H cercano a 1.0?
5. `hurst_dfa(&[], 8, 128)` → ¿retorna 0.5 sin panic?
6. `hurst_dfa` con min_box > len → ¿retorna 0.5? (datos insuficientes)
7. `linear_regression` de puntos colineales → ¿pendiente exacta?
8. `linear_regression(&[], &[])` → ¿(0.0, 0.0) sin panic?
9. ¿El resultado es determinista? Misma entrada → mismo H bit-exacto
10. ¿Performance aceptable para window de 512 puntos? (debe ser < 1ms)

---

## Referencia

- Peng, C.-K. et al. (1994). "Mosaic organization of DNA nucleotides." *Phys. Rev. E*, 49(2), 1685.
- H = α - 1 para series no-estacionarias (random walk), H = α para estacionarias (incrementos).
- Resonance usa H para normalizar la agresividad de extrapolación del Telescopio.

---

## Integración

- **Consume:** Nada
- **Consumido por:** TT-3 (Hurst normalizer), TT-5 (telescope metrics)
- **No modifica:** Nada existente
- **Patrón:** Función pura en `blueprint/equations/`. Mismo archivo que TT-1.
