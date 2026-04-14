# Sprint PP-0: Organ Sub-Pools — Paquetes de energía por órgano con estado físico

**ADR:** [ADR-033](../../arquitectura/ADR/ADR-033-organ-sub-pools.md)
**Esfuerzo:** 1 semana
**Bloqueado por:** Nada (fundación del track)
**Desbloquea:** PP-1, PP-4, PP-5, PP-6, PP-7

## Contexto

OrganManifest infiere órganos pero no tienen energía propia. Sin sub-pools no
hay senescencia diferencial, ni redistribución bajo estrés, ni emisión de volátiles.

## Principio agnóstico

Un órgano no es un "tipo" con propiedades predefinidas — es un **packet de
energía con estado físico**. Su comportamiento se deriva de `qe + volume + bond_energy`:

```
OrganSlot {
    qe: f32,           // energía
    volume: f32,       // volumen
    bond_energy: f32,  // rigidez
}

density = qe / volume
matter_state = state_from_density(density)     // derived_thresholds.rs
dissipation = dissipation_rate(matter_state)   // 4 fundamentals
priority = density / sum(densities)            // más denso = más prioritario
```

## Entregable

1. `OrganSlot { qe, volume, bond_energy }` — reemplaza `OrganSpec`
2. `OrganManifest.slots: [OrganSlot; MAX_ORGANS_PER_ENTITY]` — inline array, zero alloc
3. `organ_density(slot) → f32` — pure fn
4. `organ_priority(density, total_density) → f32` — pure fn
5. `distribute_organ_energy(entity_qe, slots) → [f32; 12]` — pure fn
6. `enforce_pool_invariant(slots, entity_qe)` — normaliza si excede
7. `organ_distribution_system` — MorphologicalLayer, after organ_lifecycle

## Tareas

| # | Tarea | Archivo | Tests |
|---|-------|---------|-------|
| 1 | `OrganSlot` struct con 3 fields físicos | `src/layers/organ.rs` | 2 |
| 2 | `organ_density`, `organ_priority` pure fns | `src/blueprint/equations/organ_energy.rs` | 4 |
| 3 | `distribute_organ_energy` pure fn | `src/blueprint/equations/organ_energy.rs` | 6 |
| 4 | `enforce_pool_invariant` pure fn | `src/blueprint/equations/organ_energy.rs` | 4 |
| 5 | `organ_distribution_system` | `src/simulation/metabolic/organ_distribution.rs` | 2 integration |
| 6 | Register en MorphologicalPlugin | `src/plugins/morphological_plugin.rs` | — |

## Criterios de aceptación

- [ ] `sum(organ_qe) ≤ entity_qe` nunca violado (property test)
- [ ] Bajo estrés (entity_qe = 10%), órganos de baja densidad → qe ≈ 0, órganos densos → qe > 0
- [ ] Zero allocation en hot path
- [ ] Ninguna referencia a OrganRole en la distribución de energía
- [ ] `cargo test` pasa sin regresiones
