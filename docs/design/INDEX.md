# Design Docs — Index

> **Primary documentation:** [`docs/ARCHITECTURE.md`](../ARCHITECTURE.md)
>
> 110K LOC · 3,066 tests · Rust 2024 / Bevy 0.15

The canonical architecture document is [`docs/ARCHITECTURE.md`](../ARCHITECTURE.md). It covers axioms, constants, module map, drug design pipeline, adaptive controller, Bozic validation, emergence status, and honest limitations.

The files below are **historical design specs** from earlier development phases. They remain as reference but may contain outdated metrics or superseded designs. When in conflict, ARCHITECTURE.md and CLAUDE.md are the sources of truth.

## Historical Design Specs

| Document | Topic | Status |
|----------|-------|--------|
| [BLUEPRINT.md](BLUEPRINT.md) | 14-layer foundational philosophy | Reference |
| [SIMULATION_CORE_DECOUPLING.md](SIMULATION_CORE_DECOUPLING.md) | SimWorld boundary contract | Reference |
| [V7.md](V7.md) | Worldgen V7: field grid, nucleus, materialization | Reference |
| [MORPHOGENESIS.md](MORPHOGENESIS.md) | Constructal body plan + mesh inference | Reference |
| [TOPOLOGY.md](TOPOLOGY.md) | Procedural terrain | Reference |
| [EMERGENCE_TIERS.md](EMERGENCE_TIERS.md) | 16 emergence modules (9 active, 7 not registered) | Reference |
| [AXIOMATIC_CLOSURE.md](AXIOMATIC_CLOSURE.md) | Cross-axiom dynamics | Reference |
| [PLANETARY_SIMULATION.md](PLANETARY_SIMULATION.md) | Day/night, seasons, water cycle | Reference |
| [USE_CASE_ARCHITECTURE.md](USE_CASE_ARCHITECTURE.md) | HOF experiment pattern | Reference |

## Module Contracts

See [`docs/arquitectura/`](../arquitectura/) for per-module runtime contracts.

## Sprint History

See [`docs/sprints/`](../sprints/) for sprint backlog and archive.

## Paper

See [`docs/paper/`](../paper/) for arXiv submission source.
