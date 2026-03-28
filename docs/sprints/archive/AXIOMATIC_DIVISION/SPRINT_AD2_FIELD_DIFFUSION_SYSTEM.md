# Sprint AD-2 — Internal Field Diffusion System

**Módulo:** `src/simulation/lifecycle/`
**Tipo:** Nuevo sistema stateless
**Eje axiomático:** Axiom 4 (diffusion + dissipation between internal nodes)
**Estado:** ✅ Cerrado (2026-03-27)
**Bloqueado por:** AD-1
**Esfuerzo:** Bajo (~30min)

---

## Objetivo

Sistema que difunde energía entre nodos vecinos del campo interno cada tick. Usa `diffusion_delta()` existente + `dissipation_from_state()` derivada. Cero constantes nuevas.

## Tareas

1. Crear sistema `internal_field_diffusion_system` en `simulation/lifecycle/`
2. Registrar en `MorphologicalPlugin` (Phase::MorphologicalLayer, before split)
3. Difusión: `delta = diffusion_delta(node[i], node[i+1], dissipation_rate, 1.0)`
4. Disipación: `node[i] -= node[i] × dissipation_from_state(matter_state)`
5. Tests: gradient equalization, dissipation reduces total, uniform stays uniform

## Criterio de cierre

- Sistema registrado y corriendo
- Usa solo ecuaciones existentes (diffusion_delta, dissipation_from_state)
- 3+ tests unitarios
