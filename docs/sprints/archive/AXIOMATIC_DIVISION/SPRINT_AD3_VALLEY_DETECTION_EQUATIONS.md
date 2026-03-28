# Sprint AD-3 — Valley Detection Equations

**Módulo:** `src/blueprint/equations/`
**Tipo:** Ecuaciones puras (zero Bevy)
**Eje axiomático:** Axiom 1 (qe ≤ 0 = no existence = disconnection)
**Estado:** ✅ Cerrado (2026-03-27)
**Bloqueado por:** Nada (ecuaciones independientes)
**Esfuerzo:** Bajo (~30min)

---

## Objetivo

Ecuaciones puras para detectar valleys, evaluar viabilidad de split, y particionar el campo. Cero thresholds hardcodeados — `valley.qe ≤ 0` es Axiom 1 puro.

## Funciones

```rust
pub fn find_valleys(field: &[f32; 8]) -> Vec<usize>
pub fn is_split_viable(field: &[f32; 8], valley_idx: usize) -> bool  // field[idx] ≤ 0.0
pub fn split_field_at(field: &[f32; 8], valley_idx: usize) -> ([f32; 8], [f32; 8])
```

## Tareas

1. Crear `src/blueprint/equations/field_division.rs`
2. Re-exportar desde `equations/mod.rs`
3. Tests: 6+ (no valleys in monotonic, center valley, edge valley, conservation, budding, split viable/not viable)

## Criterio de cierre

- Cero imports de Bevy
- `is_split_viable` usa solo `≤ 0.0` (Axiom 1)
- `split_field_at` conserva energía (Axiom 2): `sum(left) + sum(right) ≤ sum(original)`
- 6+ tests
