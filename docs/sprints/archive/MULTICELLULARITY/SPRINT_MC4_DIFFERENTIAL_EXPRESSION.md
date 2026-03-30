# Sprint MC-4 — Differential Expression: especialización celular por posición

**Módulo:** `src/blueprint/equations/positional_signal.rs` (extensión de MC-3)
**Tipo:** Pure math, stateless, TDD.
**Estado:** ⏳ Pendiente
**Bloqueado por:** MC-3

---

## Objetivo

Función pura que modifica la expression_mask de una célula basándose en su señal posicional.
Células borde: silencian growth, expresan resilience (defensa).
Células interior: silencian resilience, expresan growth (metabolismo).
La especialización emerge de la combinación signal × mask.

## Diseño

### `modulate_expression(border_signal, current_mask, rate) → [f32; 4]`

```rust
/// Adjust expression mask based on positional signal.
///
/// border_signal ~1.0 (exposed): push mask toward defense profile [low, low, low, high]
/// border_signal ~0.0 (interior): push mask toward growth profile [high, high, high, low]
///
/// Rate = how fast cells specialize (EXPRESSION_MODULATION_RATE).
/// Axiom 6: specialization emerges from gradient, not from cell type labels.
pub fn modulate_expression(
    border_signal: f32,
    current_mask: &[f32; 4],
    rate: f32,
) -> [f32; 4] {
    // Target profiles (derived, not hardcoded):
    // Border:   [1-KLEIBER, 1-KLEIBER, 1-KLEIBER, 1.0]  → high resilience
    // Interior: [1.0, 1.0, 1.0, 1-KLEIBER]               → high growth/mobility/branching
    //
    // Interpolate: target = lerp(interior_target, border_target, border_signal)
    // New mask = current + rate × (target - current)
    // Clamped to [0, 1].
}
```

### Specialization emerges because:

```
Border cell:
  → border_signal = 0.8
  → expression_mask shifts toward [0.25, 0.25, 0.25, 1.0]
  → resilience fully expressed → shell/armor capabilities unlock
  → growth/mobility silenced → cell doesn't grow or move
  → → DEFENSE SPECIALIST (skin/armor analog)

Interior cell:
  → border_signal = 0.1
  → expression_mask shifts toward [1.0, 1.0, 1.0, 0.25]
  → growth/metabolism fully expressed → metabolic graph stronger
  → resilience silenced → no armor
  → → METABOLIC SPECIALIST (organ analog)
```

### Integration with existing EpigeneticState

```
ANTES: epigenetic_adaptation_system modifica mask por ambiente externo
AHORA: modulate_expression modifica mask por posición en colonia (señal interna)

Las dos fuentes se componen: ambiente × posición → mask final.
No se reemplazan — se multiplican.
```

## Tests

### Contrato
- `modulation_stays_in_unit` — all mask values ∈ [0, 1]
- `zero_rate_no_change` — rate=0 → mask unchanged
- `identity_at_neutral_signal` — border_signal=0.5 → minimal shift

### Lógica (specialization)
- `border_cell_high_resilience` — signal=1.0 → mask[3] increases
- `border_cell_low_growth` — signal=1.0 → mask[0] decreases
- `interior_cell_high_growth` — signal=0.0 → mask[0] increases
- `interior_cell_low_resilience` — signal=0.0 → mask[3] decreases
- `gradual_specialization` — 10 applications → mask converges to target

### Errores
- `nan_signal_safe` — NaN → treated as 0.5 (neutral)
- `extreme_mask_clamped` — mask > 1.0 after modulation → clamped

## Criterios de aceptación

- `modulate_expression` es `(f32, &[f32;4], f32) → [f32;4]`, stateless.
- Target profiles derivados de `KLEIBER_EXPONENT` (not hardcoded).
- Compatible con `EpigeneticState.expression_mask` (same type).
- 10+ tests.
