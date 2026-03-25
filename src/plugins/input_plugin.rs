//! InputPlugin — Phase::Input systems (D1 behavior, D5 sensory, element_layer2, grimoire).
//!
//! Extracted from `pipeline.rs` in sprint Q5.
//! Pure registrar: no state, no resources. Ordering preserved exactly.

use bevy::prelude::*;

use crate::blueprint::almanac_hot_reload_system;
use crate::simulation::{self, Phase};

/// Registers all Phase::Input systems.
pub struct InputPlugin;

impl Plugin for InputPlugin {
    fn build(&self, app: &mut App) {
        // D5: Sensory Perception — runs before D1 Assess so SensoryAwareness is ready.
        app.init_resource::<simulation::sensory_perception::SensoryScanCursor>();
        app.add_systems(
            FixedUpdate,
            (
                simulation::sensory_perception::sensory_frequency_scan_system,
                simulation::sensory_perception::sensory_threat_memory_system,
                simulation::sensory_perception::sensory_awareness_event_system,
            )
                .chain()
                .in_set(Phase::Input)
                .after(simulation::InputChannelSet::PlatformWill)
                .before(simulation::behavior::BehaviorSet::Assess)
                .run_if(simulation::behavior::has_behavioral_agents),
        );

        // D1: Behavioral Intelligence systems
        app.add_systems(
            FixedUpdate,
            (
                simulation::behavior::behavior_cooldown_tick_system,
                simulation::behavior::behavior_assess_needs_system,
                simulation::behavior::behavior_evaluate_threats_system,
            )
                .chain()
                .in_set(simulation::behavior::BehaviorSet::Assess),
        );
        app.add_systems(
            FixedUpdate,
            (
                simulation::behavior::behavior_decision_system,
                simulation::behavior::behavior_will_bridge_system,
            )
                .chain()
                .in_set(simulation::behavior::BehaviorSet::Decide),
        );

        app.add_systems(
            FixedUpdate,
            almanac_hot_reload_system.in_set(simulation::InputChannelSet::SimulationRest),
        )
        .add_systems(
            FixedUpdate,
            simulation::element_layer2::ensure_element_id_component_system
                .in_set(simulation::InputChannelSet::SimulationRest),
        )
        .add_systems(
            FixedUpdate,
            simulation::element_layer2::derive_frequency_from_element_id_system
                .in_set(simulation::InputChannelSet::SimulationRest)
                .after(simulation::element_layer2::ensure_element_id_component_system),
        )
        .add_systems(
            FixedUpdate,
            simulation::element_layer2::sync_element_id_from_frequency_system
                .in_set(simulation::InputChannelSet::SimulationRest)
                .after(simulation::element_layer2::derive_frequency_from_element_id_system),
        );

        app.add_systems(
            FixedUpdate,
            // Hotkeys primero: en el mismo tick Q+click, el targeting ya está armado antes del pick.
            (
                simulation::input::grimoire_cast_intent_system,
                simulation::ability_targeting::ability_point_target_pick_system,
            )
                .chain()
                .in_set(simulation::InputChannelSet::SimulationRest)
                .before(simulation::element_layer2::derive_frequency_from_element_id_system),
        );
    }
}
