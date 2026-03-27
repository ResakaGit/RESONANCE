# Sprints — Gamedev Patterns ✅ CERRADO

Índice maestro: [`../README.md`](../README.md). Filosofía MOBA ↔ energía: `docs/design/GAMEDEV_IMPLEMENTATION.md`.

**Cerrados:** G1–G4, G6, G7 — `SPRINT_*.md` eliminados. **G10** (minimap) y **G12** (fog) — implementados en `runtime_platform/hud/minimap.rs`, `world/fog_of_war.rs`, `simulation/fog_of_war.rs`; sprints eliminados.

## Todos los sprints cerrados

| Sprint | Implementación | Estado |
|--------|---------------|--------|
| G1–G4, G6, G7, G10, G12 | (anteriores) | ✅ |
| **G5** | `oxidized_navigation`; `NavAgent`/`NavPath`; 3 sistemas en `InputChannelSet::PlatformWill`; bridge → `WillActuator` | ✅ |
| **G8** | Guards en `set_state`, `set_bond_energy_eb`, `set_thermal_conductivity`, `set_radius` + todos los setters de capas | ✅ |
| **G9** | Tabla producer→consumer 15 eventos en `events.rs`; tabla cross-phase en `pipeline.rs` | ✅ |
| **G11** | `ChampionId`, `WorldEntityId`, `EffectId`, `PoolId`, `OrganId`, `AgentId`; `IdGenerator` extendido; 10 integration tests | ✅ |

## Referencias

- `docs/design/GAMEDEV_PATTERNS.md`
- `docs/arquitectura/blueprint_gamedev_patterns.md`
