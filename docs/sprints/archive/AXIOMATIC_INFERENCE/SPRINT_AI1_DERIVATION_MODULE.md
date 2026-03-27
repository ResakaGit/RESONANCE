# Sprint AI-1 — Derivation Module

**Módulo:** `src/blueprint/equations/derived_thresholds.rs`
**Tipo:** Ecuaciones puras (zero Bevy deps)
**Eje axiomático:** Todos (Axiom 1-8 combinados)
**Estado:** ✅ Cerrado (2026-03-27)
**Bloqueado por:** Nada
**Esfuerzo:** Bajo (~1h)

---

## Contexto

Existe un borrador en `derived_thresholds.rs` con `const fn` que no compila (Rust stable no permite `f32` ops en const). Necesita refactor a `#[inline] fn`.

## Objetivo

Crear el módulo canónico que computa TODAS las constantes derivadas desde los 4 fundamentales. Cada función es pura, `#[inline]`, sin deps de Bevy.

## Tareas

1. **Refactor `const fn` → `#[inline] fn`** para todas las funciones que usan `f32::exp()`, `f32::powf()`, etc.
2. **Documentar cada función** con la derivación algebraica en doc comment.
3. **Tests:** Cada función tiene al menos 1 test que valida la relación con los fundamentales.
4. **Eliminar `pow_f32` / `ln_approx` / `exp_approx`** — usar `f32::powf()` y `f32::exp()` directamente.
5. **Re-exportar** desde `blueprint/equations/mod.rs` como `pub mod derived_thresholds`.

## Criterio de cierre

- `cargo test --lib derived_thresholds` — 12+ tests pasan
- Cada función devuelve valores positivos y finitos
- Relaciones monotónicas validadas: `solid < liquid < gas < plasma` para density thresholds
- Relaciones inversas validadas: `max_age_fauna < max_age_flora < max_age_materialized`

## Constantes que calcula

| Función | Derivada de | Reemplaza |
|---------|------------|-----------|
| `basal_drain_rate()` | DISSIPATION_SOLID × amplification | `BASAL_RATE = 1.0` |
| `liquid_density_threshold()` | DISSIPATION ratios ^ (1/Kleiber) × scale | `LIQUID_DENSITY_THRESHOLD = 80` |
| `gas_density_threshold()` | ídem | `GAS_DENSITY_THRESHOLD = 300` |
| `plasma_density_threshold()` | ídem | `PLASMA_DENSITY_THRESHOLD = 800` |
| `move_density_min/max()` | liquid/gas thresholds × factor | `MOVE_DENSITY_MIN/MAX` |
| `sense_coherence_min()` | DISSIPATION_SOLID noise floor | `SENSE_COHERENCE_MIN = 0.4` |
| `branch_qe_min()` | 2 × self_sustaining_qe_min | `BRANCH_QE_MIN = 30` |
| `spawn_potential_threshold()` | 1/3 (algebraic break-even) | `AXIOMATIC_SPAWN_THRESHOLD = 0.3` |
| `self_sustaining_qe_min()` | dissipation × area / coherence | `SELF_SUSTAINING_QE_MIN = 20` |
| `senescence_coeff_*()` | dissipation rate per state | `SENESCENCE_COEFF_*` |
| `max_age_*()` | 1/coeff (Gompertz inverse) | `SENESCENCE_MAX_AGE_*` |
| `radiation_pressure_threshold()` | gas_density_threshold | `RADIATION_PRESSURE_THRESHOLD_QE = 100` |
| `radiation_pressure_transfer_rate()` | DISSIPATION_GAS | `RADIATION_PRESSURE_TRANSFER_RATE = 0.05` |
| `survival_probability_threshold()` | exp(-2) (Gompertz) | `SURVIVAL_THRESHOLD = 0.05` |
