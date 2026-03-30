# Track: SURVIVAL_MODE

**Estado:** ✅ ARCHIVADO (2026-03-30) — 3/3 sprints completados. 0 DEBT.

Los documentos `SPRINT_SV1`…`SV3` se eliminaron al cerrar el track; la especificación queda en `src/bin/survival.rs` y `src/sim_world.rs`.

---

## Entregables

| Sprint | Entregable | Archivo |
|--------|-----------|---------|
| SV-1 | `apply_input()` wiring (InputCommand → WillActuator) | `src/sim_world.rs` |
| SV-2 | Survival binary: genome load, arena spawn, WASD control, score tracking | `src/bin/survival.rs` |
| SV-3 | Death detection (DeathEvent + qe fallback), game over UI, restart (R) | `src/bin/survival.rs` |

## Uso

```bash
cargo run --release --bin survival -- --genomes assets/evolved/seed_42.bin
cargo run --release --bin survival -- --seed 42 --gens 50 --worlds 100
```

WASD para mover. R para reiniciar. Score = ticks sobrevividos. Zero src/ modifications (excepto SV-1: 5 LOC en sim_world.rs).
