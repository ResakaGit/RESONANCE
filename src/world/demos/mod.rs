//! Demos de validación manual (composición por capas, mapas dedicados).
pub mod competition_arena;
pub mod demo_animal;
pub mod demo_celula;
pub mod demo_metrics;
pub mod demo_planta;
pub mod demo_virus;
pub mod inferred_world;
pub mod morphogenesis_demo;
pub mod round_world_rosa;
pub mod signal_demo;

pub use competition_arena::{
    COMPETITION_ARENA_SLUG, spawn_competition_demo_startup_system,
};
pub use demo_animal::{DEMO_ANIMAL_SLUG, spawn_demo_animal_startup_system};
pub use demo_metrics::{DemoMetricsHud, ensure_demo_metrics_hud_system, sync_demo_metrics_hud_system};
pub use demo_celula::{DEMO_CELULA_SLUG, spawn_demo_celula_startup_system};
pub use demo_planta::{DEMO_PLANTA_SLUG, spawn_demo_planta_startup_system};
pub use demo_virus::{DEMO_VIRUS_SLUG, spawn_demo_virus_startup_system};
pub use inferred_world::{
    INFERRED_WORLD_SLUG, spawn_inferred_world_startup_system,
};
pub use morphogenesis_demo::{
    MORPHOGENESIS_DEMO_SLUG, spawn_morphogenesis_demo_startup_system,
};
pub use round_world_rosa::{
    ROUND_WORLD_ROSA_SLUG, enforce_round_world_rosa_focus_system,
    round_world_rosa_pin_lod_focus_for_inference_system, spawn_round_world_rosa_demo,
    spawn_round_world_rosa_startup_system, stabilize_round_world_rosa_energy_system,
};
pub use signal_demo::{SIGNAL_DEMO_SLUG, spawn_signal_demo_startup_system};
