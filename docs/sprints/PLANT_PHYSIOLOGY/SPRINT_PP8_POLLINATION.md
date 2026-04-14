# Sprint PP-8: Cross-Transfer — Reproducción cruzada mediada por terceros

**ADR:** [ADR-035](../../arquitectura/ADR/ADR-035-volatile-field-protocol.md) (canal de atracción)
**Esfuerzo:** 2 semanas
**Bloqueado por:** PP-6 (volatile emission para atracción)
**Desbloquea:** —

## Contexto

Hoy flora se reproduce por seed dispatch autónomo. No hay interacción
reproductiva mediada por otra entidad. La transferencia cruzada permite
mezcla genética entre entidades distantes, mediada por un transportador.

## Principio agnóstico

No se dice "polinización flora↔fauna". El mecanismo es genérico:

1. Entidad A emite volátil (PP-6) → atrae entidad móvil con freq alineada
2. Entidad móvil toca A → `CollisionTransfer` deposita **EnergyTag** (qe + freq)
3. Entidad móvil con EnergyTag toca entidad B →
   - Si `alignment(tag_freq, B_freq) > TRANSFER_THRESHOLD` → reproducción cruzada
   - Si no → tag se pierde (disipa, Axiom 4)

El tag es un packet de energía con frecuencia (Axiom 1 + 8). Se disipa con
el tiempo (Axiom 4). Solo entidades compatibles por frecuencia lo aceptan
(Axiom 8). La especificidad emerge, no se programa.

```
EnergyTag {
    qe: f32,                    // energía del tag (se disipa)
    source_freq: f32,           // frecuencia del emisor
    source_profile: [f32; 4],   // growth, mobility, branch, resilience del emisor
    age_ticks: u32,             // decay counter
}

compatibility = alignment(tag.source_freq, target.freq)
tag.qe *= (1 - DISSIPATION_LIQUID)  // decay each tick while carried
tag muere cuando tag.qe < threshold OR age > LIFETIME
```

## Entregable

1. `EnergyTag { qe, source_freq, source_profile, age_ticks }` — transient component (SparseSet)
2. `transfer_compatibility(tag_freq, target_freq, bandwidth) → f32` — pure fn
3. `mix_profiles(a, b, weight) → [f32; 4]` — pure fn (crossover)
4. `tag_deposit_system` — on sessile↔mobile collision, mobile gains EnergyTag
5. `tag_transfer_system` — on mobile↔sessile collision, if compatible → cross-reproduce
6. `tag_decay_system` — EnergyTag disipa si no se usa

## Tareas

| # | Tarea | Archivo | Tests |
|---|-------|---------|-------|
| 1 | `EnergyTag` component | `src/layers/energy_tag.rs` | 1 |
| 2 | `transfer_compatibility` pure fn | `src/blueprint/equations/cross_transfer.rs` | 6 |
| 3 | `mix_profiles` pure fn | `src/blueprint/equations/cross_transfer.rs` | 4 |
| 4 | Deposit system | `src/simulation/reproduction/cross_transfer.rs` | 3 integration |
| 5 | Transfer system | `src/simulation/reproduction/cross_transfer.rs` | 3 integration |
| 6 | Decay system | `src/simulation/reproduction/cross_transfer.rs` | 2 |
| 7 | Register en MetabolicPlugin | `src/plugins/metabolic_plugin.rs` | — |

## Criterios de aceptación

- [ ] Entidad A + transportador + entidad B (freq compatible) → offspring con profile mixto
- [ ] Entidad A + transportador + entidad C (freq incompatible) → no offspring
- [ ] EnergyTag decae — transportador lento pierde el tag antes de llegar
- [ ] Atracción mediada por volátil (PP-6), no por distancia fija
- [ ] `TRANSFER_THRESHOLD = 0.5` (alignment mínimo para compatibilidad)
- [ ] `TAG_LIFETIME = 1.0 / DISSIPATION_LIQUID ≈ 50 ticks`
- [ ] Ninguna referencia a "polen", "flora", "fauna" — funciona entre cualquier par de entidades
