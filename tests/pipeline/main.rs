//! Suite `pipeline` — integración del pipeline de simulación (App completa,
//! cadena de fases).
//!
//! Criterio de asignación (ADR-048 §2.1). Nombre `pipeline` y no
//! `integration`: todo `tests/` es integración por definición de cargo, 3
//! miembros ya llevan sufijo `_integration` (evita
//! `--test integration sf_integration::`), y "pipeline" es el vocabulario del
//! repo (CLAUDE.md §Pipeline).
//!
//! Filtrar un módulo concreto:
//! `cargo test --test pipeline sf_integration::`

mod sf_integration;
mod energy_competition_integration;
mod morphogenesis_integration;
mod planet_viewer_integration;
mod field_convergence;
mod input_event_flow;
mod pathfinding_multi_waypoint;
