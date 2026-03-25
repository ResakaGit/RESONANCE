//! Motor de color cuantizado: paletas CPU, ρ por LOD (Sprint 14 purificado).
mod archetype_element;
mod camera_plane;
pub mod constants;
mod palette_gen;
mod plugin;
mod registry;
mod systems;

pub use archetype_element::element_id_for_world_archetype;
pub use palette_gen::{PaletteBlock, generate_palette, magenta_fallback_rgba};
pub use plugin::QuantizedColorPlugin;
pub use registry::PaletteRegistry;

pub use systems::{
    factor_precision_system, palette_registry_cpu_sync_system,
    quantized_precision_ensure_system,
};

use bevy::prelude::Component;

/// Factor ρ de precisión cromática por distancia (SparseSet: sin thrash de arquetipo).
#[derive(Component, Clone, Copy, Debug, Default, PartialEq)]
#[component(storage = "SparseSet")]
pub struct QuantizedPrecision(pub f32);
