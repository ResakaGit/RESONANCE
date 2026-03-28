# Sprint AD-5 — Deprecate reproduction_spawn_system

**Módulo:** `src/simulation/reproduction/`, `src/plugins/morphological_plugin.rs`
**Tipo:** Eliminación de sistema hardcodeado
**Eje axiomático:** Axiom 6 (emergence, not programming)
**Estado:** ✅ Cerrado (2026-03-27)
**Bloqueado por:** AD-4 (split system must be working first)
**Esfuerzo:** Medio (~45min)

---

## Objetivo

Eliminar el sistema de reproducción hardcodeado y todas sus constantes. La "reproducción" ya no es un evento programado — es la consecuencia de un campo interno que se desconecta.

## Qué se elimina

| Constante/Sistema | Valor actual | Por qué sobra |
|-------------------|-------------|----------------|
| `reproduction_spawn_system` | Sistema completo | Reemplazado por `axiomatic_split_system` |
| `reproduction_cooldown_tick_system` | Sistema completo | No hay cooldown — split es físico |
| `ReproductionCooldown` | Component SparseSet | No hay cooldown |
| `FAUNA_REPRODUCTION_QE_MIN` | 200.0 | No hay threshold — valley ≤ 0 decide |
| `SEED_ENERGY_FRACTION` | 0.15 | No hay fracción — cada mitad se lleva sus nodos |
| `REPRODUCTION_COOLDOWN_TICKS` | 60 | No hay cooldown |
| `FAUNA_OFFSPRING_INITIAL_RADIUS` | 0.35 | Radius derivado de sum(child_nodes) |
| `MAX_REPRODUCTIONS_PER_FRAME` | 2 | 1 split per entity per tick (structural) |
| `REPRODUCTION_RADIUS_FACTOR` | constante | No se chequea radius — se chequea campo |
| `CapabilitySet::REPRODUCE` | bit flag | No hay capability de reproducción — todo puede dividirse si su campo se desconecta |

## Qué permanece

- `CapabilitySet::BRANCH` → reinterpretado: "puede desarrollar valleys" (capacidad de campo, no de reproducción)
- `InferenceProfile.branching_bias` → ahora influye en cómo el campo distribuye energía internamente (más branching = más peaks)
- `CulturalMemory` inheritance → se mueve a `axiomatic_split_system`

## Tareas

1. Remover `reproduction_spawn_system` y `reproduction_cooldown_tick_system` del MorphologicalPlugin
2. Remover `ReproductionCooldown` component
3. Remover constantes de reproducción de `simulation/reproduction/constants.rs`
4. Mover herencia cultural al split system (si no está ya)
5. Verificar que el split system produce offspring a tasa comparable
6. Correr demo: `big_bang` + `civilization_test` — verificar que la población se sostiene

## Criterio de cierre

- `grep -rn "reproduction_spawn_system" src/plugins/` → 0 matches
- `grep -rn "REPRODUCTION_COOLDOWN\|FAUNA_REPRODUCTION_QE_MIN\|SEED_ENERGY_FRACTION" src/` → 0 matches (except docs/archive)
- Demo `civilization_test` produce poblaciones comparables (±30% del baseline anterior)
- Cero regresión en tests existentes
