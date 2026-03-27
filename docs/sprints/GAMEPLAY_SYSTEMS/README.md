# Track — Gameplay Systems (GS)

**Alineacion:** Blueprint "What The Simulation Needs" §4–9. Construye sobre SF (fundaciones) y sim_world.rs (contrato). Agrega el estrato competitivo: netcode, AI tactica, game loop, legibilidad visual, arquetipos y onboarding.
**Metodologia:** TDD, funciones puras en `blueprint/equations/`, sistemas de una transformacion, zero unsafe. Stateless-first.
**Prioridad:** Después de SF-7 (Replay). GS-1/GS-5 pueden arrancar en paralelo con SF-5.

> **GS NO duplica SF.** SF cubre fundaciones (observabilidad, serialización, latencia, replay). GS construye encima para el juego competitivo. Dependencia: SF-5 (checkpoint) bloquea GS-1 (lockstep).

---

## Objetivo del track

Cerrar los gaps que convierten el motor de simulación en un MOBA jugable y reproducible:

1. **Netcode** — Lockstep determinista + rollback para multijugador competitivo sin estado derivado transmitido.
2. **AI táctica** — Targeting Nash-optimal + formación de manada. Extiende D1 (BehaviorMode) al ámbito de equipo.
3. **Game Loop** — Condición de victoria como estado físico: núcleo energético bajo umbral de viabilidad.
4. **Visual Legibility** — Contrato inyectivo: estado físico → señal perceptual. Sin ambigüedad táctica.
5. **Character Design** — Arquetipos como configuraciones de física. Balance por constantes, no por código.
6. **Onboarding** — Secuencia de experiencias que construyen intuición sin exponer ecuaciones.

---

## Principio fundamental

> "El juego no tiene reglas. Tiene física. Las reglas emergen porque la física es consistente."

Un MOBA donde la victoria, el balance y las mecánicas emergen de las mismas seis ecuaciones que generan la vida.

---

## Auditoría de lo existente

| Sistema | Estado | Qué cambia con GS |
|---------|--------|-------------------|
| `sim_world.rs` — SimWorld boundary | ✅ Implementado | Base para netcode + rollback |
| SF-2/SF-5 — Serde + Checkpoint | ⏳ SF track | Prerequisito GS-1/GS-2 |
| D1 `simulation/behavior.rs` | ✅ Corre en Phase::Input | GS-3 extiende con Nash targeting |
| `layers/social_communication.rs` — PackMembership | ✅ Existe | GS-4 formaliza formación y cohesión |
| `worldgen/EnergyNucleus` | ✅ Existe, genera vida | GS-5 lo promueve a objetivo de victoria |
| `simulation/post.rs::faction_identity_system` | ✅ Rastreo de facciones | GS-5 agrega victory_check |
| `rendering/quantized_color/` | ✅ Frequency→palette | GS-7 formaliza el contrato injective completo |
| `worldgen/visual_derivation.rs` | ✅ Energy→visual | GS-7 lo documenta y extiende |
| `entities/archetypes/` | ✅ spawn_* functions | GS-8 agrega ArchetypeConfig RON |
| `world/demos/` | ✅ Demos proceduales | GS-9 usa como base para tutorial |
| Netcode | ❌ No existe | GS-1/GS-2 lo crean desde cero |

---

## Grafo de dependencias

```
[SF-5 Checkpoint]   [SF-7 Replay]   [sim_world.rs]   [D1 Behavior]
      │                   │                │                │
      ▼                   ▼                ▼                ▼
 GS-1 Lockstep      GS-5 Victoria    GS-2 Rollback    GS-3 Nash AI
      │                   │                │                │
      └───────────────────┤                │           GS-4 Pack
                          │                │
                   GS-6 Map Energy    GS-2 (via GS-1)
                          │
                ┌─────────┴──────────┐
                ▼                    ▼
         GS-7 Visual          GS-8 Arquetipos
                │                    │
                └─────────┬──────────┘
                          ▼
                    GS-9 Onboarding
```

---

## Ondas de ejecucion

| Onda | Sprints | Qué habilita | Precondicion | Estado |
|------|---------|-------------|-------------|--------|
| **0** | GS-1 ✅, GS-3 ✅, GS-5 ✅ | Ecuaciones + tipos fundacionales | SF-5, D1 existente | ✅ COMPLETA |
| **A** | GS-2, GS-4, GS-6 (paralelo) | Sistemas: rollback, pack, mapa | GS-1 ✅, GS-3 ✅, GS-5 ✅ | ⏳ Desbloqueada |
| **B** | GS-7, GS-8 (paralelo) | Contratos visual + arquetipos | GS-5 ✅, GS-6 | 🔒 |
| **C** | GS-9 | Onboarding completo | GS-7, GS-8 | 🔒 |

---

## Indice de sprints

| Sprint | Modulo principal | Onda | Dependencias | Estado |
|--------|-----------------|------|--------------|--------|
| GS-1 | `simulation/netcode/` | 0 | SF-5, sim_world | ✅ [Archivado](../archive/GAMEPLAY_SYSTEMS/SPRINT_GS1_NETCODE_LOCKSTEP.md) |
| [GS-2](SPRINT_GS2_NETCODE_ROLLBACK.md) | `simulation/netcode/` | A | GS-1 ✅ | ⏳ |
| GS-3 | `blueprint/equations/tactical_ai.rs` | 0 | D1 existente | ✅ [Archivado](../archive/GAMEPLAY_SYSTEMS/SPRINT_GS3_NASH_AI_TARGETING.md) |
| [GS-4](SPRINT_GS4_PACK_DYNAMICS.md) | `simulation/metabolic/social_communication.rs` | A | GS-3 ✅ | ⏳ |
| GS-5 | `simulation/game_loop.rs` | 0 | EnergyNucleus, GameState | ✅ [Archivado](../archive/GAMEPLAY_SYSTEMS/SPRINT_GS5_VICTORY_NUCLEUS.md) |
| [GS-6](SPRINT_GS6_MAP_ENERGY.md) | `worldgen/systems/node_control.rs` | A | GS-5 ✅ | ⏳ |
| [GS-7](SPRINT_GS7_VISUAL_CONTRACT.md) | `rendering/`, `worldgen/visual_derivation.rs` | B | GS-5 ✅, GS-6 | 🔒 |
| [GS-8](SPRINT_GS8_ARCHETYPE_CONFIG.md) | `entities/archetypes/`, RON | B | GS-5 ✅, GS-6 | 🔒 |
| [GS-9](SPRINT_GS9_ONBOARDING.md) | `plugins/`, `world/demos/` | C | GS-7, GS-8 | 🔒 |

---

## Paralelismo seguro

| | GS-1 | GS-2 | GS-3 | GS-4 | GS-5 | GS-6 | GS-7 | GS-8 | GS-9 |
|---|---|---|---|---|---|---|---|---|---|
| **GS-1** | — | | ✅ | ✅ | ✅ | | | | |
| **GS-2** | | — | ✅ | ✅ | ✅ | | | | |
| **GS-3** | ✅ | ✅ | — | | ✅ | | | | |
| **GS-4** | ✅ | ✅ | | — | ✅ | | | | |
| **GS-5** | ✅ | ✅ | ✅ | ✅ | — | | ✅ | ✅ | |
| **GS-6** | | | | | | — | ✅ | ✅ | |
| **GS-7** | | | | | ✅ | ✅ | — | ✅ | |
| **GS-8** | | | | | ✅ | ✅ | ✅ | — | |
| **GS-9** | | | | | | | | | — |

Onda 0: GS-1, GS-3, GS-5 son paralelos (módulos distintos, sin overlap).
Onda A: GS-2, GS-4, GS-6 son paralelos (distintos subsistemas).
Onda B: GS-7, GS-8 son paralelos.

---

## Invariantes del track

1. **Determinismo absoluto.** Toda lógica de AI, victoria y netcode usa sólo physics observables — cero RNG excepto gated por `tick_id XOR entity_id`.
2. **AI no lee conceptos de juego.** `BehaviorMode` se deriva de `qe`, `velocity`, `frequency`, `structural_damage`. Zero "kill counters" o "health bars".
3. **Victoria es estado físico.** La partida termina cuando `qe(nucleus_B) < QE_MIN_EXISTENCE`. No hay timer ni reglas externas.
4. **Visual nunca escribe física.** El renderer consume `WorldSnapshot`. Zero retroescritura.
5. **Arquetipos = constantes RON.** Balance sin tocar código. Todo tuning en `assets/characters/*.ron`.
6. **Netcode inputs-only.** Solo inputs cruzan la red en estado estacionario. World state se transmite únicamente para sync inicial y recuperación de desync.
7. **Zero crates nuevos.** Reutiliza `serde` + `ron` (ya en Cargo.toml).
8. **Phase assignment.** Cada sistema nuevo en `Phase::X` explícito.
9. **Max 4 campos por componente.** VictoryNucleus, InputPacket, etc.

---

## Contrato de pipeline GS

```
FixedUpdate:
  Phase::Input
    [existente] advance_simulation_clock_system
    [GS-1]      lockstep_input_collect_system    (.before PlatformWill)
    [GS-3/4]    nash_target_select_system         (.in_set BehaviorSet::Decide)
    [GS-4]      pack_cohesion_force_system        (.after BehaviorSet::Decide)

  Phase::MetabolicLayer
    [GS-5]      victory_nucleus_check_system      (.after metabolic_stress_death_system)
    [GS-6]      node_control_update_system        (.after faction_identity_system)
    [GS-4]      pack_formation_apply_system       (.before pack_cohesion_force_system)

  Phase::MorphologicalLayer
    [GS-7]      visual_contract_sync_system       (read-only, outputs to VisualHints Resource)

PostUpdate (on-demand):
    [GS-1/2]    lockstep_tick_checksum_system
    [GS-2]      rollback_detect_and_apply_system
```

---

## Referencias cruzadas

- `docs/design/SIMULATION_CORE_DECOUPLING.md` — SimWorld boundary (base de GS-1/GS-2)
- `docs/design/BLUEPRINT.md` — 14 capas ortogonales (base de GS-3/GS-8)
- `docs/sprints/SIMULATION_FOUNDATIONS/README.md` — Prerrequisitos: SF-5 (checkpoint) y SF-7 (replay)
- `docs/extraInfo/digramaFromClaudeOpus.md` — Diagrama de arquitectura actualizado
- `src/simulation/behavior.rs` — D1 BehaviorMode (GS-3 extiende)
- `src/layers/social_communication.rs` — PackMembership (GS-4 usa)
- `src/worldgen/EnergyNucleus` — Núcleo (GS-5 promueve a objetivo)
- `src/rendering/quantized_color/` — Paleta (GS-7 formaliza)
- `src/entities/archetypes/` — Spawn functions (GS-8 extiende con RON config)
