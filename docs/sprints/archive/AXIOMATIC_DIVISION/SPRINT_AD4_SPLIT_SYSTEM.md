# Sprint AD-4 — Axiomatic Split System

**Módulo:** `src/simulation/lifecycle/`
**Tipo:** Nuevo sistema ECS
**Eje axiomático:** Axiom 1 (split at disconnection) + Axiom 2 (conservation)
**Estado:** ✅ Cerrado (2026-03-27)
**Bloqueado por:** AD-1, AD-2, AD-3
**Esfuerzo:** Medio (~1h)

---

## Objetivo

Sistema que detecta valleys ≤ 0 en el campo interno y ejecuta el split. No es "reproducción" — es ruptura física por desconexión energética.

## Comportamiento

```
Cada tick, para cada entidad con InternalEnergyField:
  valleys = find_valleys(field)
  para cada valley:
    si is_split_viable(field, valley) → NO threshold, solo qe ≤ 0:
      (left, right) = split_field_at(field, valley)
      si sum(left) < self_sustaining_qe_min() → skip (child no viable, Axiom 1)
      si sum(right) < self_sustaining_qe_min() → skip
      spawn child_A con left nodes
      spawn child_B con right nodes
      despawn parent
      break (max 1 split per tick)
```

## Herencia (todo inferido)

| Propiedad | De dónde viene | Hardcoded? |
|-----------|---------------|------------|
| Energy | sum(child_nodes) | No — viene del campo |
| Frequency | parent.frequency_hz ± drift proporcional a Axiom 8 | No — drift = diffusion_delta de frecuencia |
| InferenceProfile | derivado del campo (growth = tip energy, mobility = gradient) | No — inferido de la distribución |
| CulturalMemory | copiada al hijo con más qe total | No — el más grande hereda cultura |
| SenescenceProfile | tick_birth = now, coeff from dissipation state | No — derivado de Axiom 4 |
| Position | parent.pos ± offset proporcional a valley position | No — el offset viene de la geometría del split |

## Tareas

1. Crear `axiomatic_split_system` en `simulation/lifecycle/`
2. Registrar en MorphologicalPlugin (Phase::MorphologicalLayer, after diffusion, before abiogenesis)
3. Conservation assert: `sum(child_A) + sum(child_B) ≤ sum(parent) + epsilon`
4. Tests: split produces 2 children, conservation holds, no split when no valley, budding asymmetry

## Criterio de cierre

- Split ocurre SOLO por `valley.qe ≤ 0` (Axiom 1)
- Cero constantes de reproducción
- Conservation verificada (Axiom 2)
- Demo headless muestra splits en telemetry
