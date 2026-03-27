# Track — Simulation Quality (SQ)

Master index: [`../README.md`](../README.md).

Refactor track for `src/simulation/` — code smells, anti-patterns, SRP violations.
Zero behavior change; pure quality improvement.

Extracted from STRUCTURE_MIGRATION after SM-1–SM-7 closure.

## Sprints

| Sprint | File | Scope | Status |
|--------|------|-------|--------|
| **SM-8** | [SPRINT_SM8_SIMULATION_CODE_QUALITY.md](SPRINT_SM8_SIMULATION_CODE_QUALITY.md) | `simulation/thermodynamic/`, `reactions.rs`, `input.rs` — god-systems, magic numbers, SRP violations | ✅ Cerrado 2026-03-25 |

### SM-8 Progress

- ✅ **SM-8A** Magic numbers extraction → `blueprint/constants/simulation_defaults.rs`
- ✅ **SM-8B** Change detection guards → `input.rs` (movement_intent), `reactions.rs` (frequency_hz)
- ✅ **SM-8C** Inline math extraction → `blueprint/equations/simulation_quality.rs` (`perception_signal_weighted`, `bond_weakening` + 10 tests)
- ✅ **SM-8D** God-system splits — `contained_thermal_transfer_system` → 3 sistemas (`containment_overlap`, `containment_thermal`, `containment_drag`); `grimoire_cast_intent_system` → 3 sistemas (ver SM-8G)
- ✅ **SM-8E** Pipeline registration refactoring
- ✅ **SM-8F** Lifecycle query documentation
- ✅ **SM-8G** Input SRP split — `grimoire_cast_intent_system` → `grimoire_slot_selection_system` + `grimoire_targeting_system` + `grimoire_channeling_start_system`; `SlotActivatedEvent` como señal interna de un tick

## Context

SM-8 was authored during the STRUCTURE_MIGRATION track as a post-audit. The `simulation/metabolic/` subdirectory scores 9.7/10 and is considered reference-quality. Work targets the remaining 30% surface: `thermodynamic/` (2 god-systems), root `reactions.rs`, and `input.rs`.

## References

- `CLAUDE.md` — Coding rules (max 4 fields, one system one transformation, math in blueprint/equations)
- `docs/arquitectura/` — Module contracts
