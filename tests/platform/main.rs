//! Suite `platform` — contratos de plataforma, patrones y assets.
//!
//! Criterio de asignación (ADR-048 §2.1): *contrato de infraestructura*, no
//! cero-App — `require_marker_hierarchy` levanta `App` + `MinimalPlugins`.
//! Runtime trivial en conjunto: es el escalón barato de la escalera de
//! `AXIOM_LAYER_VALIDATION_MATRIX.md` §3.
//!
//! Filtrar un módulo concreto:
//! `cargo test --test platform demo_flow_maps::`

mod require_marker_hierarchy;
mod sparse_set_transient_markers;
mod g11_strong_ids;
mod q3_pub_field_api;
mod t9_terrain_config_hot_reload;
mod wgsl_cell_field_snapshot_valid;
mod demo_flow_maps;
