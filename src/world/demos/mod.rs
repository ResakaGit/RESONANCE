//! Demos de validación manual (composición por capas, mapas dedicados).
pub mod round_world_rosa;

pub use round_world_rosa::{
    ROUND_WORLD_ROSA_SLUG, enforce_round_world_rosa_focus_system,
    round_world_rosa_pin_lod_focus_for_inference_system, spawn_round_world_rosa_demo,
    spawn_round_world_rosa_startup_system, stabilize_round_world_rosa_energy_system,
};
