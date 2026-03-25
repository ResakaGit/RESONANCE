//! HUD de gameplay (MOBA) — capa presentación sobre datos ECS.

mod ability_bar;
mod minimap;
mod minimap_constants;

pub use ability_bar::AbilityHudPlugin;
pub use minimap::{
    MinimapIcon, MinimapPlugin, MinimapScreenRect, minimap_cursor_blocks_primary_pick,
};
