# ADR-033: Organ Sub-Pools — Per-Organ Energy Budgets

**Estado:** Propuesto
**Fecha:** 2026-04-13
**Contexto:** PLANT_PHYSIOLOGY track, sprints PP-0, PP-5

## Contexto

Hoy cada entidad tiene un único `BaseEnergy.qe` (L0). Los órganos se infieren
desde `OrganManifest` pero no tienen energía propia. Esto impide:

- Senescencia por órgano (órganos delgados se marchitan antes que los densos)
- Costo diferencial de mantenimiento
- Redistribución interna de energía bajo estrés

El pool invariant (Axiom 2) exige `sum(organ_qe) ≤ entity_qe`. Esto ya es
una restricción activa — solo falta subdividir el pool.

## Decision

### D1: Array fijo `[f32; MAX_ORGANS]` en OrganManifest

**Alternativas evaluadas:**

| Opción | Pros | Contras |
|--------|------|---------|
| A: `Vec<f32>` en component | Flexible | Viola regla 13 (no Vec en components), allocation en hot path |
| B: `[f32; 12]` inline | Cache-friendly, zero alloc, SoA compatible | Desperdicia slots vacíos |
| C: Component separado `OrganEnergy` | Desacoplado | Query más ancha, sync overhead |

**Decisión: B.** `[f32; MAX_ORGANS_PER_ENTITY]` dentro de `OrganManifest`. Los slots
vacíos son 0.0 — el costo es 48 bytes por entidad (12 × f32). Cache line friendly.

### D2: Organ como packet físico — distribución por densidad, no por rol

**Rechazada:** priority table por `OrganRole` — viola Axiom 6 (emergence). Decidir
que un pétalo tiene prioridad 0.25 es programar comportamiento top-down.

**Decisión:** Cada `OrganSlot` tiene propiedades físicas medibles. La prioridad
se **deriva** del estado material del órgano:

```
OrganSlot {
    qe: f32,           // energía del órgano
    volume: f32,       // volumen → densidad = qe/volume
    bond_energy: f32,  // rigidez estructural
}

density(organ) = organ.qe / organ.volume
matter_state(organ) = state_from_density(density)  // ya existe en derived_thresholds.rs
dissipation(organ) = dissipation_rate(matter_state) // from 4 fundamentals

// Distribución proporcional a densidad (los más densos retienen más energía):
priority(organ) = density(organ) / sum(densities)
organ_qe[i] = entity_qe × priority(organ[i])
```

Un órgano denso con alta bond_energy retiene energía (se comporta como tallo).
Un órgano con baja densidad pierde energía primero (se comporta como pétalo).
**Nadie le dijo qué es.** El comportamiento emerge de la física.

### D3: Pool invariant enforcement

```
assert: sum(organ_qe[0..len]) <= entity_qe  — every tick
```

Si la suma excede (por rounding), se normaliza proporcionalmente. Pure fn en
`blueprint/equations/organ_energy.rs`. Zero side effects.

## No viola axiomas

1. **Axiom 1:** Sub-pools son subdivisiones de qe. No crean energía nueva.
2. **Axiom 2:** `sum(organ_qe) ≤ entity_qe` — enforced por diseño.
3. **Axiom 4:** Cada órgano disipa a la tasa de su matter_state. Órganos gaseosos disipan 16× más que sólidos.
4. **Axiom 6:** Marchitamiento selectivo EMERGE de densidad, no se programa por rol.

## Archivos modificados

| Archivo | Cambio |
|---------|--------|
| `src/layers/organ.rs` | `OrganSlot { qe, volume, bond_energy }`, `OrganManifest` usa `[OrganSlot; 12]` |
| `src/blueprint/equations/organ_energy.rs` | **NUEVO** — `organ_density`, `organ_priority`, `distribute_energy`, `enforce_pool_invariant` |
| `src/simulation/metabolic/organ_distribution.rs` | **NUEVO** — system que redistribuye cada tick |

## Tests

- 12 tests unitarios (distribución por densidad, invariant, edge cases qe=0)
- 2 tests de integración (órganos de baja densidad pierden energía primero)
