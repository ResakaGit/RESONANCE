//! Suite `probes` — probes de arquetipos + materialización.
//!
//! Criterio de asignación (ADR-048 §2.1): `App` + spawn + N updates, con
//! observación del comportamiento resultante (no un umbral axiomático).
//!
//! Filtrar un módulo concreto:
//! `cargo test --test probes probe_animal::`

mod probe_animal;
mod probe_celula;
mod probe_mono_constructal;
mod probe_planta;
mod probe_virus;
mod senescence_materialization_check;
mod test_senescence_insert;
