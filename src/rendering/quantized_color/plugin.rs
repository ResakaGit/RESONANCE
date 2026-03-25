//! Plugin: CPU Color Quantization.

use bevy::prelude::*;

use crate::simulation::states::{GameState, PlayState};

use super::registry::PaletteRegistry;
use super::systems::{
    factor_precision_system, palette_registry_cpu_sync_system,
    quantized_precision_ensure_system,
};

/// Motor de color cuantizado purificado (Sprint 14 refactorizado): paletas, ρ de LOD. 
pub struct QuantizedColorPlugin;

impl Plugin for QuantizedColorPlugin {
    fn build(&self, app: &mut App) {
        let run_game = in_state(GameState::Playing).and(in_state(PlayState::Active));

        app.init_resource::<PaletteRegistry>().add_systems(
            Update,
            (
                palette_registry_cpu_sync_system,
                quantized_precision_ensure_system,
                factor_precision_system.after(quantized_precision_ensure_system),
            )
                .chain()
                .run_if(run_game),
        );
    }
}
