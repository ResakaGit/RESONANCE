use bevy::prelude::*;

use crate::blueprint::init_almanac_elements_system;
use crate::eco::climate::init_climate_config_system;
use crate::plugins::atomic_plugin::AtomicPlugin;
use crate::plugins::chemical_plugin::ChemicalPlugin;
use crate::plugins::input_plugin::InputPlugin;
use crate::plugins::metabolic_plugin::MetabolicPlugin;
use crate::plugins::morphological_plugin::MorphologicalPlugin;
use crate::plugins::thermodynamic_plugin::ThermodynamicPlugin;
use crate::runtime_platform::compat_2d3d::{RenderCompatProfile, SimWorldTransformParams};
use crate::runtime_platform::fog_overlay::spawn_fog_world_overlay_startup_system;
use crate::simulation::lifecycle::{enter_game_state_playing_system, transition_to_active_system};
use crate::simulation::{init_simulation_bootstrap, pipeline};
use crate::topology::config::init_terrain_config_system;
use crate::world::fog_of_war::init_fog_of_war_from_energy_field_system;
use crate::worldgen::systems::startup::{
    load_map_config_startup_system, spawn_nuclei_from_map_config_system, worldgen_warmup_system,
};
use crate::worldgen::systems::terrain::insert_terrain_field_startup_system;

/// Plugin principal que registra todos los sistemas de simulación
/// con el ordenamiento correcto.
pub struct SimulationPlugin;

impl Plugin for SimulationPlugin {
    fn build(&self, app: &mut App) {
        if !app.world().contains_resource::<SimWorldTransformParams>() {
            let profile = app
                .world()
                .get_resource::<RenderCompatProfile>()
                .copied()
                .unwrap_or_default();
            app.insert_resource(SimWorldTransformParams::from_profile(profile));
        }

        init_simulation_bootstrap(app);

        // SF-4: Metrics export — only inserted when RESONANCE_METRICS=1.
        if std::env::var("RESONANCE_METRICS").is_ok() {
            app.insert_resource(crate::simulation::observability::MetricsExportConfig::default());
        }

        // SF-5: Checkpoint — only inserted when env vars are set.
        if let Some(cfg) = crate::simulation::checkpoint_system::CheckpointConfig::from_env() {
            app.insert_resource(cfg);
        }

        // Phase ordering + clock + worldgen delegation (schedule-generic).
        pipeline::register_simulation_pipeline(app, FixedUpdate);

        // Domain plugins — each owns one Phase of the pipeline.
        app.add_plugins((
            InputPlugin,
            ThermodynamicPlugin,
            AtomicPlugin,
            ChemicalPlugin,
            MetabolicPlugin,
            MorphologicalPlugin,
        ));

        #[cfg(not(feature = "v7_worldgen"))]
        pipeline::register_visual_derivation_pipeline(app);

        app.add_systems(
            Startup,
            (
                crate::simulation::checkpoint_system::checkpoint_load_startup_system,
                init_almanac_elements_system,
                init_climate_config_system,
                init_terrain_config_system,
                load_map_config_startup_system,
                crate::worldgen::systems::startup::seed_initial_field_system,
                crate::worldgen::systems::startup::init_day_night_config_system,
                init_fog_of_war_from_energy_field_system,
                insert_terrain_field_startup_system,
                spawn_nuclei_from_map_config_system,
                crate::worldgen::seed_nutrient_field_from_nuclei_system,
                enter_game_state_playing_system,
                worldgen_warmup_system,
                transition_to_active_system,
            )
                .chain(),
        );
        // Fog overlay needs Assets<Image> (render plugin) — skip in headless.
        app.add_systems(
            Startup,
            spawn_fog_world_overlay_startup_system
                .run_if(resource_exists::<Assets<Image>>)
                .after(transition_to_active_system),
        );
    }
}
