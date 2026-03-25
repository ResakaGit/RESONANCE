use bevy::prelude::*;

use crate::blueprint::constants::{
    EXHAUSTION_BUFFER_FRACTION, EXHAUSTION_REST_TICKS, LOCOMOTION_MIN_SPEED_THRESHOLD,
};
use crate::blueprint::equations;
use crate::layers::behavior::{BehaviorCooldown, BehaviorIntent, BehaviorMode, BehavioralAgent};
use crate::layers::{
    AlchemicalEngine, BaseEnergy, EnergyOps, FlowVector, MatterCoherence, MatterState,
    WillActuator,
};
use crate::runtime_platform::compat_2d3d::SimWorldTransformParams;
use crate::runtime_platform::core_math_agnostic::sim_plane_pos;
use crate::simulation::time_compat::simulation_delta_secs;
use crate::topology::TerrainField;

/// Drains energy from buffer (L5) or base (L0) proportional to movement speed.
pub fn locomotion_energy_drain_system(
    fixed: Option<Res<Time<Fixed>>>,
    time: Res<Time>,
    layout: Res<SimWorldTransformParams>,
    terrain: Option<Res<TerrainField>>,
    mut energy_ops: EnergyOps,
    mut query: Query<
        (
            Entity,
            &FlowVector,
            &mut AlchemicalEngine,
            &Transform,
            Option<&MatterCoherence>,
        ),
        With<WillActuator>,
    >,
) {
    let dt = simulation_delta_secs(fixed, &time);
    if dt <= 0.0 {
        return;
    }
    let xz = layout.use_xz_ground;

    for (entity, flow, mut engine, transform, matter_opt) in &mut query {
        let speed = flow.speed();
        if speed < LOCOMOTION_MIN_SPEED_THRESHOLD {
            continue;
        }

        let Some(mass) = energy_ops.qe(entity) else {
            continue;
        };
        if mass <= 0.0 {
            continue;
        }

        let terrain_factor = compute_terrain_factor(
            transform,
            xz,
            terrain.as_deref(),
            matter_opt,
        );

        let cost = equations::locomotion_energy_cost(mass, speed, terrain_factor) * dt;
        if cost <= 0.0 {
            continue;
        }

        let buffer_available = engine.buffer_level();
        if buffer_available >= cost {
            engine.try_subtract_buffer(cost);
        } else {
            if buffer_available > 0.0 {
                engine.try_subtract_buffer(buffer_available);
            }
            let remainder = cost - buffer_available;
            if remainder > 0.0 {
                energy_ops.drain(entity, remainder, crate::events::DeathCause::Dissipation);
            }
        }
    }
}

/// Forces idle + decision cooldown when buffer is critically low and entity is moving.
pub fn locomotion_exhaustion_system(
    mut query: Query<
        (
            &AlchemicalEngine,
            &FlowVector,
            &mut BehaviorIntent,
            &mut BehaviorCooldown,
        ),
        With<BehavioralAgent>,
    >,
) {
    for (engine, flow, mut intent, mut cooldown) in &mut query {
        let buffer_fraction = if engine.buffer_cap() > 0.0 {
            engine.buffer_level() / engine.buffer_cap()
        } else {
            0.0
        };

        if buffer_fraction >= EXHAUSTION_BUFFER_FRACTION
            || flow.speed() <= LOCOMOTION_MIN_SPEED_THRESHOLD
        {
            continue;
        }

        if intent.mode != BehaviorMode::Idle {
            intent.mode = BehaviorMode::Idle;
            intent.target_entity = None;
        }
        if cooldown.decision_cooldown < EXHAUSTION_REST_TICKS {
            cooldown.decision_cooldown = EXHAUSTION_REST_TICKS;
        }
    }
}

/// Computes terrain locomotion factor from terrain sample + matter state.
fn compute_terrain_factor(
    transform: &Transform,
    use_xz: bool,
    terrain: Option<&TerrainField>,
    matter_opt: Option<&MatterCoherence>,
) -> f32 {
    let Some(terrain) = terrain else {
        return 1.0;
    };
    let pos = sim_plane_pos(transform.translation, use_xz);
    let Some(sample) = terrain.sample_at_world(pos) else {
        return 1.0;
    };

    let state = matter_opt
        .map(|m| m.state())
        .unwrap_or(MatterState::Solid);
    let slope_normalized = (sample.slope / 90.0).clamp(0.0, 1.0);

    equations::terrain_locomotion_factor(slope_normalized, 1.0, state)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::blueprint::constants::{
        EXHAUSTION_BUFFER_FRACTION, EXHAUSTION_REST_TICKS, LOCOMOTION_MIN_SPEED_THRESHOLD,
    };
    use crate::events::DeathEvent;
    use crate::layers::behavior::{BehaviorCooldown, BehaviorIntent, BehaviorMode, BehavioralAgent};
    use crate::layers::{AlchemicalEngine, BaseEnergy, FlowVector, WillActuator};
    use bevy::math::Vec2;
    use std::time::Duration;

    /// Helper: creates a minimal App with time advanced to a known dt for drain tests.
    fn drain_test_app() -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_event::<DeathEvent>();
        app.insert_resource(SimWorldTransformParams::default());
        let mut fixed = Time::<Fixed>::from_seconds(1.0 / 60.0);
        fixed.advance_by(Duration::from_secs_f64(1.0 / 60.0));
        app.insert_resource(fixed);
        app
    }

    // -----------------------------------------------------------------------
    // S1: locomotion_energy_drain_system
    // -----------------------------------------------------------------------

    #[test]
    fn drain_skips_stationary_entities() {
        let mut app = drain_test_app();

        let id = app
            .world_mut()
            .spawn((
                BaseEnergy::new(500.0),
                FlowVector::new(Vec2::ZERO, 0.01),
                AlchemicalEngine::new(100.0, 10.0, 10.0, 80.0),
                Transform::default(),
                WillActuator::default(),
            ))
            .id();

        app.add_systems(Update, locomotion_energy_drain_system);
        app.update();

        let engine = app.world().get::<AlchemicalEngine>(id).unwrap();
        assert!((engine.buffer_level() - 80.0).abs() < 1e-5, "buffer unchanged for stationary");
    }

    #[test]
    fn drain_consumes_buffer_when_moving() {
        let mut app = drain_test_app();

        let id = app
            .world_mut()
            .spawn((
                BaseEnergy::new(500.0),
                FlowVector::new(Vec2::new(5.0, 0.0), 0.01),
                AlchemicalEngine::new(100.0, 10.0, 10.0, 80.0),
                Transform::default(),
                WillActuator::default(),
            ))
            .id();

        app.add_systems(Update, locomotion_energy_drain_system);
        app.update();

        let engine = app.world().get::<AlchemicalEngine>(id).unwrap();
        assert!(
            engine.buffer_level() < 80.0,
            "buffer should decrease when moving: got {}",
            engine.buffer_level()
        );
    }

    #[test]
    fn drain_falls_back_to_base_energy_when_buffer_empty() {
        let mut app = drain_test_app();

        let id = app
            .world_mut()
            .spawn((
                BaseEnergy::new(500.0),
                FlowVector::new(Vec2::new(5.0, 0.0), 0.01),
                AlchemicalEngine::new(100.0, 10.0, 10.0, 0.0),
                Transform::default(),
                WillActuator::default(),
            ))
            .id();

        app.add_systems(Update, locomotion_energy_drain_system);
        app.update();

        let energy = app.world().get::<BaseEnergy>(id).unwrap();
        assert!(
            energy.qe() < 500.0,
            "base energy should decrease as fallback: got {}",
            energy.qe()
        );
    }

    #[test]
    fn drain_ignores_entities_without_will_actuator() {
        let mut app = drain_test_app();

        let id = app
            .world_mut()
            .spawn((
                BaseEnergy::new(500.0),
                FlowVector::new(Vec2::new(5.0, 0.0), 0.01),
                AlchemicalEngine::new(100.0, 10.0, 10.0, 80.0),
                Transform::default(),
                // No WillActuator — projectile/passive entity
            ))
            .id();

        app.add_systems(Update, locomotion_energy_drain_system);
        app.update();

        let engine = app.world().get::<AlchemicalEngine>(id).unwrap();
        assert!(
            (engine.buffer_level() - 80.0).abs() < 1e-5,
            "passive entities should not pay locomotion cost"
        );
    }

    #[test]
    fn drain_below_speed_threshold_skipped() {
        let mut app = drain_test_app();

        let id = app
            .world_mut()
            .spawn((
                BaseEnergy::new(500.0),
                FlowVector::new(
                    Vec2::new(LOCOMOTION_MIN_SPEED_THRESHOLD * 0.5, 0.0),
                    0.01,
                ),
                AlchemicalEngine::new(100.0, 10.0, 10.0, 80.0),
                Transform::default(),
                WillActuator::default(),
            ))
            .id();

        app.add_systems(Update, locomotion_energy_drain_system);
        app.update();

        let engine = app.world().get::<AlchemicalEngine>(id).unwrap();
        assert!(
            (engine.buffer_level() - 80.0).abs() < 1e-5,
            "no drain below speed threshold"
        );
    }

    // -----------------------------------------------------------------------
    // S3: locomotion_exhaustion_system
    // -----------------------------------------------------------------------

    #[test]
    fn exhaustion_forces_idle_when_buffer_empty() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);

        let id = app
            .world_mut()
            .spawn((
                BehavioralAgent,
                BehaviorIntent {
                    mode: BehaviorMode::Forage { urgency: 0.8 },
                    target_entity: None,
                },
                BehaviorCooldown::default(),
                AlchemicalEngine::new(100.0, 10.0, 10.0, 0.0), // empty buffer
                FlowVector::new(Vec2::new(3.0, 0.0), 0.01),
            ))
            .id();

        app.add_systems(Update, locomotion_exhaustion_system);
        app.update();

        let intent = app.world().get::<BehaviorIntent>(id).unwrap();
        assert_eq!(intent.mode, BehaviorMode::Idle, "should force idle when exhausted");
        assert!(intent.target_entity.is_none());

        let cooldown = app.world().get::<BehaviorCooldown>(id).unwrap();
        assert_eq!(
            cooldown.decision_cooldown, EXHAUSTION_REST_TICKS,
            "should lock decision for rest period"
        );
    }

    #[test]
    fn exhaustion_no_effect_when_buffer_adequate() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);

        let id = app
            .world_mut()
            .spawn((
                BehavioralAgent,
                BehaviorIntent {
                    mode: BehaviorMode::Forage { urgency: 0.8 },
                    target_entity: None,
                },
                BehaviorCooldown::default(),
                AlchemicalEngine::new(100.0, 10.0, 10.0, 50.0), // 50% full
                FlowVector::new(Vec2::new(3.0, 0.0), 0.01),
            ))
            .id();

        app.add_systems(Update, locomotion_exhaustion_system);
        app.update();

        let intent = app.world().get::<BehaviorIntent>(id).unwrap();
        assert!(
            matches!(intent.mode, BehaviorMode::Forage { .. }),
            "should keep foraging when buffer adequate"
        );
    }

    #[test]
    fn exhaustion_no_effect_when_stationary() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);

        let id = app
            .world_mut()
            .spawn((
                BehavioralAgent,
                BehaviorIntent {
                    mode: BehaviorMode::Forage { urgency: 0.8 },
                    target_entity: None,
                },
                BehaviorCooldown::default(),
                AlchemicalEngine::new(100.0, 10.0, 10.0, 1.0), // near empty
                FlowVector::new(Vec2::ZERO, 0.01),              // but stationary
            ))
            .id();

        app.add_systems(Update, locomotion_exhaustion_system);
        app.update();

        let intent = app.world().get::<BehaviorIntent>(id).unwrap();
        assert!(
            matches!(intent.mode, BehaviorMode::Forage { .. }),
            "stationary entity should not be forced idle"
        );
    }

    #[test]
    fn exhaustion_preserves_higher_existing_cooldown() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);

        let id = app
            .world_mut()
            .spawn((
                BehavioralAgent,
                BehaviorIntent {
                    mode: BehaviorMode::Forage { urgency: 0.5 },
                    target_entity: None,
                },
                BehaviorCooldown {
                    decision_cooldown: EXHAUSTION_REST_TICKS + 10,
                    action_cooldown: 0,
                },
                AlchemicalEngine::new(100.0, 10.0, 10.0, 0.0),
                FlowVector::new(Vec2::new(3.0, 0.0), 0.01),
            ))
            .id();

        app.add_systems(Update, locomotion_exhaustion_system);
        app.update();

        let cooldown = app.world().get::<BehaviorCooldown>(id).unwrap();
        assert_eq!(
            cooldown.decision_cooldown,
            EXHAUSTION_REST_TICKS + 10,
            "should not lower an existing higher cooldown"
        );
    }

    #[test]
    fn exhaustion_threshold_boundary() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);

        // Exactly at threshold: buffer_fraction = 5/100 = 0.05 = EXHAUSTION_BUFFER_FRACTION
        let id = app
            .world_mut()
            .spawn((
                BehavioralAgent,
                BehaviorIntent {
                    mode: BehaviorMode::Forage { urgency: 0.5 },
                    target_entity: None,
                },
                BehaviorCooldown::default(),
                AlchemicalEngine::new(
                    100.0,
                    10.0,
                    10.0,
                    EXHAUSTION_BUFFER_FRACTION * 100.0,
                ),
                FlowVector::new(Vec2::new(3.0, 0.0), 0.01),
            ))
            .id();

        app.add_systems(Update, locomotion_exhaustion_system);
        app.update();

        let intent = app.world().get::<BehaviorIntent>(id).unwrap();
        assert!(
            matches!(intent.mode, BehaviorMode::Forage { .. }),
            "at exact threshold boundary, should NOT trigger (>= check)"
        );
    }
}
