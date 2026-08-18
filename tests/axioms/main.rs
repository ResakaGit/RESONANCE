//! Suite `axioms` — gates axiomáticos, property tests y equivalencias.
//!
//! Criterio de asignación (ADR-048 §2.1): el assert central es una invariante
//! de axioma o un umbral derivado. Varios miembros levantan `App` +
//! `MinimalPlugins` (r1/r2, emergence_ecs) — el criterio es el *tema*, no el
//! peso de runtime.
//!
//! Filtrar un módulo concreto:
//! `cargo test --test axioms property_conservation::`

mod r1_conservation;
mod r2_determinism;
mod r4_calibration;
mod r5_sensitivity;
mod r6_observability;
mod r7_morph_robustness;
mod r8_surrogate;
mod r9_ci_gates;
mod r10_emergence_gates;
mod property_conservation;
mod property_autopoiesis;
mod emergence_sync;
mod emergence_ecs;
mod chemistry_equivalence;
