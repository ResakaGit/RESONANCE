# Sprint AI-4 — Senescence Coefficients from Metabolic Rate

**Módulo:** `src/blueprint/constants/senescence.rs`, spawn paths (startup.rs, spawn.rs, abiogenesis/mod.rs)
**Tipo:** Constantes eliminadas → llamadas a derivación
**Eje axiomático:** Axiom 4 (dissipation = aging) + Kleiber (metabolic rate ∝ mass^0.75)
**Estado:** ✅ Cerrado (2026-03-27)
**Bloqueado por:** AI-1
**Esfuerzo:** Medio (~45min, toca 4 archivos)

---

## Qué existe hoy

```rust
// blueprint/constants/senescence.rs — 6 constantes arbitrarias:
pub const SENESCENCE_COEFF_MATERIALIZED: f32 = 0.0001;  // EMPIRICALLY-CALIBRATED
pub const SENESCENCE_MAX_AGE_MATERIALIZED: u64 = 8_000;  // EMPIRICALLY-CALIBRATED
pub const SENESCENCE_COEFF_FLORA: f32 = 0.0002;          // ratio 1:2 sin justificación
pub const SENESCENCE_MAX_AGE_FLORA: u64 = 5_000;
pub const SENESCENCE_COEFF_FAUNA: f32 = 0.0005;          // ratio 1:5 sin justificación
pub const SENESCENCE_MAX_AGE_FAUNA: u64 = 3_000;
```

## Derivación axiomática

El envejecimiento es disipación acumulada (Axiom 4). La tasa de envejecimiento escala con la tasa metabólica del estado de materia de la entidad:

```
senescence_coeff(entity) = dissipation_rate(matter_state) × 1.0
max_viable_age(entity) = 1.0 / senescence_coeff

Terreno (Solid):  coeff = 0.005 → max_age = 200 ticks
Flora (Solid-Liquid mix): coeff = 0.0125 → max_age = 80 ticks
Fauna (Liquid):   coeff = 0.02 → max_age = 50 ticks
```

**Nota:** Estos valores son mucho más cortos que los actuales (8000/5000/3000). El efecto será generaciones más rápidas — más turnover, más evolución. Si es demasiado rápido, el `DENSITY_SCALE` (el fundamental) se ajusta, NO los coeficientes.

## Tareas

1. Reemplazar 6 constantes en `senescence.rs` con re-exports de `derived_thresholds::*()`.
2. Actualizar 3 spawn paths que leen `SENESCENCE_COEFF_*` y `SENESCENCE_MAX_AGE_*`:
   - `worldgen/systems/startup.rs`
   - `worldgen/systems/materialization/spawn.rs`
   - `simulation/abiogenesis/mod.rs`
3. Actualizar `simulation/metabolic/senescence_death.rs`: `SURVIVAL_THRESHOLD` → `survival_probability_threshold()`.
4. Correr demo headless y verificar que el ciclo de vida sigue visible (con turnover más rápido).

## Criterio de cierre

- `senescence.rs` solo contiene re-exports, no valores numéricos propios
- `grep -r "0.0001\|0.0002\|0.0005\|8_000\|5_000\|3_000" src/blueprint/constants/senescence.rs` → 0 matches
- Demo headless muestra turnover generacional visible
