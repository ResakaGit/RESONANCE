//! ThermodynamicPlugin — Phase::ThermodynamicLayer systems.
//!
//! Extracted from `pipeline.rs` in sprint Q5.
//! Pure registrar: no state, no resources. Ordering preserved exactly.

use bevy::prelude::*;

use crate::eco::climate::{climate_config_hot_reload_system, climate_tick_system};
use crate::eco::eco_boundaries_system;
use crate::simulation::states::{GameState, PlayState};
use crate::simulation::{self, Phase};
use crate::topology::config::terrain_config_loader_system;

/// Registers all Phase::ThermodynamicLayer systems.
pub struct ThermodynamicPlugin;

impl Plugin for ThermodynamicPlugin {
    fn build(&self, app: &mut App) {
        let run_gameplay = in_state(GameState::Playing).and(in_state(PlayState::Active));

        app.init_resource::<simulation::sensory::AttentionGrid>();

        app.add_systems(
            FixedUpdate,
            (
                terrain_config_loader_system,
                climate_config_hot_reload_system,
                climate_tick_system,
            )
                .chain()
                .in_set(Phase::ThermodynamicLayer)
                .run_if(run_gameplay.clone())
                .before(crate::worldgen::systems::terrain::terrain_mutation_system)
                .before(eco_boundaries_system)
                .before(simulation::containment::containment_system),
        )
        .add_systems(
            FixedUpdate,
            simulation::sensory::attention_convergence_system
                .in_set(Phase::ThermodynamicLayer)
                .run_if(run_gameplay.clone()),
        )
        .add_systems(
            FixedUpdate,
            (
                simulation::containment::containment_system,
                simulation::structural_runtime::structural_constraint_system,
                simulation::containment::containment_overlap_system,
                simulation::containment::containment_thermal_system,
                simulation::containment::containment_drag_system,
                simulation::pre_physics::reset_resonance_overlay_system,
                simulation::pre_physics::resonance_link_system,
                simulation::pre_physics::sync_injector_projected_qe_system,
                simulation::pre_physics::engine_processing_system,
                simulation::photosynthesis::irradiance_update_system,
                simulation::pre_physics::perception_system,
            )
                .chain()
                .in_set(Phase::ThermodynamicLayer)
                .run_if(run_gameplay.clone())
                .after(
                    crate::worldgen::systems::visual::flush_pending_energy_visual_rebuild_system,
                ),
        );

        // Radiation pressure: excess energy above threshold pushes outward.
        app.add_systems(
            FixedUpdate,
            crate::worldgen::systems::radiation_pressure::radiation_pressure_system
                .in_set(Phase::ThermodynamicLayer)
                .run_if(run_gameplay.clone()),
        );

        // Day/night cycle: angular modulation of solar energy.
        // After propagation + pressure, before materialization.
        // Only runs if DayNightConfig resource exists.
        app.add_systems(
            FixedUpdate,
            (
                crate::worldgen::systems::planetary_formation::planetary_formation_system
                    .after(crate::worldgen::systems::radiation_pressure::radiation_pressure_system),
                crate::worldgen::systems::day_night::day_night_modulation_system.after(
                    crate::worldgen::systems::planetary_formation::planetary_formation_system,
                ),
                crate::worldgen::systems::water_cycle::water_cycle_system
                    .after(crate::worldgen::systems::day_night::day_night_modulation_system),
            )
                .in_set(Phase::ThermodynamicLayer)
                .run_if(run_gameplay),
        );
    }
}
