pub mod demo_clouds;
pub mod demo_level;
pub mod demos;
pub mod fog_of_war;
pub mod grimoire_presets;
pub mod marker;
pub mod perception;
pub mod space;

pub use demo_clouds::{
    DemoCloudSpawnerState, demo_cloud_context_spawn_system, demo_cloud_motion_system,
    spawn_demo_clouds_startup_system,
};
pub use demo_level::{
    enforce_rosa_focus_system, pin_rosa_lod_focus_system, spawn_demo_flora,
    spawn_demo_level_startup_system, stabilize_rosa_growth_system,
};
pub use demos::{
    COMPETITION_ARENA_SLUG, spawn_competition_demo_startup_system,
    DEMO_ANIMAL_SLUG, spawn_demo_animal_startup_system,
    DemoMetricsHud, ensure_demo_metrics_hud_system, sync_demo_metrics_hud_system,
    DEMO_CELULA_SLUG, spawn_demo_celula_startup_system,
    DEMO_PLANTA_SLUG, spawn_demo_planta_startup_system,
    DEMO_VIRUS_SLUG, spawn_demo_virus_startup_system,
    INFERRED_WORLD_SLUG, spawn_inferred_world_startup_system,
    MORPHOGENESIS_DEMO_SLUG, spawn_morphogenesis_demo_startup_system,
    ROUND_WORLD_ROSA_SLUG, enforce_round_world_rosa_focus_system,
    round_world_rosa_pin_lod_focus_for_inference_system, spawn_round_world_rosa_demo,
    spawn_round_world_rosa_startup_system, stabilize_round_world_rosa_energy_system,
    SIGNAL_DEMO_SLUG, spawn_signal_demo_startup_system,
};
pub use fog_of_war::{FogOfWarGrid, NUM_FOG_TEAMS, faction_for_fog_team, fog_team_index};
pub use marker::Scoreboard;
pub use perception::PerceptionCache;
pub use space::{
    SpatialEntry, SpatialIndex, update_spatial_index_after_move_system, update_spatial_index_system,
};
