# Sprints — Gamedev Patterns (pendiente)

Índice maestro: [`../README.md`](../README.md). Filosofía MOBA ↔ energía: `docs/design/GAMEDEV_IMPLEMENTATION.md`.

**Cerrados:** G1–G4, G6, G7 — `SPRINT_*.md` eliminados. **G10** (minimap) y **G12** (fog) — implementados en `runtime_platform/hud/minimap.rs`, `world/fog_of_war.rs`, `simulation/fog_of_war.rs`; sprints eliminados.

## Backlog

| Sprint | Archivo | Estado |
|--------|---------|--------|
| **G5** | `SPRINT_G5_PATHFINDING.md` | Parcial — NavMesh + follow; falta flowfield masivo / avoidance |
| **G8** | `SPRINT_G8_CHANGE_DETECTION.md` | Parcial — auditoría global `set_if_neq` |
| **G9** | `SPRINT_G9_EVENT_ORDERING.md` | Parcial — tabla producer→consumer documentada al 100% |
| **G11** | `SPRINT_G11_STRONG_IDS.md` | Parcial — `WorldEntityId` / `IdGenerator` en `blueprint/ids.rs`; red/save end-to-end |

## Referencias

- `docs/design/GAMEDEV_PATTERNS.md`
- `docs/arquitectura/blueprint_gamedev_patterns.md`
