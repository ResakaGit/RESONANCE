//! IWG — Inferred World Geometry: body plan, terrain visuals, atmosphere, water surface.

pub mod atmosphere;
pub mod body_plan;
pub mod terrain_visuals;
pub mod water_surface;
pub use atmosphere::*;
pub use body_plan::*;
pub use terrain_visuals::*;
pub use water_surface::*;
