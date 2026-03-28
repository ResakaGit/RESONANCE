# Sprint AD-1 — InternalEnergyField Component

**Módulo:** `src/layers/`
**Tipo:** Nuevo componente ECS
**Eje axiomático:** Axiom 1 (entity = energy distribution, not energy point)
**Estado:** ⏳ Pendiente
**Bloqueado por:** Nada
**Esfuerzo:** Bajo (~30min)

---

## Objetivo

Agregar componente `InternalEnergyField` con 8 nodos de energía por entidad. Convierte entidades de "puntos de energía" a "campos de energía con estructura interna".

## Tareas

1. Crear `src/layers/internal_field.rs` con componente (≤4 fields: `nodes: [f32; 8]`)
2. Re-exportar desde `src/layers/mod.rs`
3. Insertar en `awakening_system` cuando entidad despierta: `nodes[i] = qe / 8.0`
4. Insertar en `materialization/spawn.rs` al spawn: misma distribución uniforme
5. Tests: uniform distribution, total matches qe, zero-energy stays zero

## Criterio de cierre

- Componente existe con `#[derive(Component, Reflect, Debug, Clone)]`
- Insertado en awakening + materialization
- 3+ tests unitarios
