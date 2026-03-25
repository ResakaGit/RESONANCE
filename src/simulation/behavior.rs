use bevy::prelude::*;

use crate::blueprint::constants::{
    BEHAVIOR_DECISION_INTERVAL, BEHAVIOR_PANIC_FACTOR, BEHAVIOR_SPRINT_FACTOR,
    DEFAULT_MOBILITY_BIAS, FORAGE_MAX_RANGE, HUNGER_THRESHOLD_FRACTION, HUNT_MAX_RANGE,
    IDLE_DEFAULT_SCORE, IDLE_SATIATED_SCORE, MAX_CHASE_TICKS, PANIC_THRESHOLD,
    SATIATED_THRESHOLD_FRACTION,
};
use crate::blueprint::equations;
use crate::layers::behavior::{
    BehaviorCooldown, BehaviorIntent, BehaviorMode, BehavioralAgent, EnergyAssessment,
    SensoryAwareness,
};
use crate::layers::{AlchemicalEngine, BaseEnergy, CapabilitySet, Faction, InferenceProfile, MobaIdentity, WillActuator};
use crate::runtime_platform::compat_2d3d::SimWorldTransformParams;
use crate::runtime_platform::core_math_agnostic::sim_plane_pos;
use crate::world::SpatialIndex;

/// Sub-phases for the behavior pipeline (auto-deferred between Assess → Decide).
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub enum BehaviorSet {
    Assess,
    Decide,
}

/// Run condition: at least one `BehavioralAgent` exists.
pub fn has_behavioral_agents(query: Query<(), With<BehavioralAgent>>) -> bool {
    !query.is_empty()
}

// ---------------------------------------------------------------------------
// S5: Cooldown tick — decrements decision and action cooldowns.
// ---------------------------------------------------------------------------

pub fn behavior_cooldown_tick_system(
    mut query: Query<&mut BehaviorCooldown, With<BehavioralAgent>>,
) {
    for mut cooldown in &mut query {
        if cooldown.decision_cooldown == 0 && cooldown.action_cooldown == 0 {
            continue;
        }
        if cooldown.decision_cooldown > 0 {
            cooldown.decision_cooldown -= 1;
        }
        if cooldown.action_cooldown > 0 {
            cooldown.action_cooldown -= 1;
        }
    }
}

// ---------------------------------------------------------------------------
// S1: Assess needs — evaluates internal energy state.
// ---------------------------------------------------------------------------

pub fn behavior_assess_needs_system(
    mut commands: Commands,
    query: Query<
        (Entity, &BaseEnergy, &AlchemicalEngine, &BehaviorCooldown),
        With<BehavioralAgent>,
    >,
) {
    for (entity, energy, engine, cooldown) in &query {
        if cooldown.decision_cooldown > 0 {
            continue;
        }
        let (hunger_fraction, energy_ratio) =
            equations::assess_energy(engine.buffer_level(), engine.buffer_cap());
        commands.entity(entity).insert(EnergyAssessment {
            hunger_fraction,
            energy_ratio,
            biomass: energy.qe(),
        });
    }
}

// ---------------------------------------------------------------------------
// S2: Evaluate threats — scans spatial neighbours for hostiles and food.
// ---------------------------------------------------------------------------

pub fn behavior_evaluate_threats_system(
    mut commands: Commands,
    agents: Query<
        (Entity, &Transform, &BehaviorCooldown),
        (With<BehavioralAgent>, Without<SensoryAwareness>),
    >,
    spatial: Res<SpatialIndex>,
    layout: Res<SimWorldTransformParams>,
    identities: Query<&MobaIdentity>,
    energies: Query<&BaseEnergy>,
) {
    let detection_range = FORAGE_MAX_RANGE.max(HUNT_MAX_RANGE);

    for (entity, transform, cooldown) in &agents {
        if cooldown.decision_cooldown > 0 {
            continue;
        }
        let pos = sim_plane_pos(transform.translation, layout.use_xz_ground);
        let nearby = spatial.query_radius(pos, detection_range);
        let self_identity = identities.get(entity).ok();

        let mut hostile_entity: Option<Entity> = None;
        let mut hostile_dist = f32::MAX;
        let mut food_entity: Option<Entity> = None;
        let mut food_dist = f32::MAX;

        for entry in &nearby {
            if entry.entity == entity {
                continue;
            }
            let dist = (entry.position - pos).length();

            let Some(self_id) = self_identity else {
                if energies.get(entry.entity).is_ok() && dist < food_dist {
                    food_entity = Some(entry.entity);
                    food_dist = dist;
                }
                continue;
            };
            let Ok(other_id) = identities.get(entry.entity) else { continue; };

            if self_id.is_enemy(other_id) && dist < hostile_dist {
                hostile_entity = Some(entry.entity);
                hostile_dist = dist;
            }
            let faction = other_id.faction();
            if (faction == Faction::Neutral || faction == Faction::Wild)
                && dist < food_dist
                && energies.get(entry.entity).is_ok()
            {
                food_entity = Some(entry.entity);
                food_dist = dist;
            }
        }

        commands.entity(entity).insert(SensoryAwareness {
            hostile_entity,
            hostile_distance: if hostile_entity.is_some() { hostile_dist } else { f32::MAX },
            food_entity,
            food_distance: if food_entity.is_some() { food_dist } else { f32::MAX },
        });
    }
}

// ---------------------------------------------------------------------------
// S3: Decision — computes utility scores and selects best action.
// ---------------------------------------------------------------------------

pub fn behavior_decision_system(
    mut commands: Commands,
    mut agents: Query<
        (
            Entity,
            &EnergyAssessment,
            &SensoryAwareness,
            &mut BehaviorIntent,
            &mut BehaviorCooldown,
        ),
        With<BehavioralAgent>,
    >,
    profiles: Query<&InferenceProfile>,
    capabilities: Query<&CapabilitySet>,
    energies: Query<&BaseEnergy>,
) {
    for (entity, assessment, awareness, mut intent, mut cooldown) in &mut agents {
        let profile = profiles.get(entity).ok();
        let caps = capabilities.get(entity).ok();
        let resilience = InferenceProfile::resilience_effective(profile);
        let mobility = profile.map(|p| p.mobility_bias).unwrap_or(DEFAULT_MOBILITY_BIAS);

        let mut scores = [0.0_f32; 5];

        // 0: Idle
        scores[0] = if assessment.energy_ratio > SATIATED_THRESHOLD_FRACTION {
            IDLE_SATIATED_SCORE
        } else {
            IDLE_DEFAULT_SCORE
        };

        // 1: Forage
        if awareness.food_entity.is_some() {
            let urgency = if assessment.hunger_fraction > HUNGER_THRESHOLD_FRACTION {
                assessment.hunger_fraction
            } else {
                0.0
            };
            scores[1] = equations::utility_forage(
                assessment.hunger_fraction,
                awareness.food_distance,
                urgency,
            );
        }

        // 2: Flee / 3: Hunt — from hostile entity
        if let Some(hostile) = awareness.hostile_entity {
            let hostile_qe = energies.get(hostile).map(|e| e.qe()).unwrap_or(0.0);
            let threat = equations::threat_level(hostile_qe, assessment.biomass);

            scores[2] = equations::utility_flee(
                threat,
                awareness.hostile_distance,
                HUNT_MAX_RANGE,
                resilience,
            );

            // Hunt only if hostile is weaker
            if hostile_qe < assessment.biomass {
                scores[3] = equations::utility_hunt(
                    hostile_qe,
                    awareness.hostile_distance,
                    assessment.energy_ratio,
                    mobility,
                );
            }
        }

        // 4: Reproduce
        let can_reproduce = caps.map(|c| c.can_reproduce()).unwrap_or(false);
        if can_reproduce {
            scores[4] = equations::utility_reproduce(assessment.biomass, 1.0, 1.0);
        }

        // Panic override: flee trumps all when threat is severe
        let best = if scores[2] >= PANIC_THRESHOLD {
            2
        } else {
            equations::select_best_action(&scores)
        };

        let new_mode = match best {
            1 => BehaviorMode::Forage {
                urgency: assessment.hunger_fraction,
            },
            2 => match awareness.hostile_entity {
                Some(threat) => BehaviorMode::Flee { threat },
                None => BehaviorMode::Idle,
            },
            3 => match awareness.hostile_entity {
                Some(prey) => {
                    let chase_ticks =
                        if let BehaviorMode::Hunt {
                            prey: old_prey,
                            chase_ticks,
                        } = &intent.mode
                        {
                            if *old_prey == prey {
                                chase_ticks + BEHAVIOR_DECISION_INTERVAL
                            } else {
                                0
                            }
                        } else {
                            0
                        };
                    if chase_ticks > MAX_CHASE_TICKS {
                        BehaviorMode::Idle
                    } else {
                        BehaviorMode::Hunt { prey, chase_ticks }
                    }
                }
                None => BehaviorMode::Idle,
            },
            4 => BehaviorMode::Reproduce,
            _ => BehaviorMode::Idle,
        };

        let new_target = match &new_mode {
            BehaviorMode::Hunt { prey, .. } => Some(*prey),
            BehaviorMode::Flee { threat } => Some(*threat),
            BehaviorMode::Forage { .. } => awareness.food_entity,
            _ => None,
        };

        if intent.mode != new_mode {
            intent.mode = new_mode;
        }
        if intent.target_entity != new_target {
            intent.target_entity = new_target;
        }

        cooldown.decision_cooldown = BEHAVIOR_DECISION_INTERVAL.saturating_sub(1);

        commands
            .entity(entity)
            .remove::<EnergyAssessment>()
            .remove::<SensoryAwareness>();
    }
}

// ---------------------------------------------------------------------------
// S4: Will bridge — translates BehaviorIntent into WillActuator movement.
// ---------------------------------------------------------------------------

pub fn behavior_will_bridge_system(
    mut agents: Query<
        (&BehaviorIntent, &GlobalTransform, &mut WillActuator),
        With<BehavioralAgent>,
    >,
    targets: Query<&GlobalTransform>,
    layout: Res<SimWorldTransformParams>,
) {
    for (intent, self_gtf, mut will) in &mut agents {
        let self_pos = sim_plane_pos(self_gtf.translation(), layout.use_xz_ground);

        let movement = match &intent.mode {
            BehaviorMode::Idle => Vec2::ZERO,
            BehaviorMode::Forage { .. } => {
                direction_to_target(intent.target_entity, self_pos, &targets, &layout)
                    .unwrap_or(Vec2::ZERO)
            }
            BehaviorMode::Hunt { .. } => {
                direction_to_target(intent.target_entity, self_pos, &targets, &layout)
                    .map(|d| d * BEHAVIOR_SPRINT_FACTOR)
                    .unwrap_or(Vec2::ZERO)
            }
            BehaviorMode::Flee { threat } => {
                direction_to_target(Some(*threat), self_pos, &targets, &layout)
                    .map(|d| -d * BEHAVIOR_PANIC_FACTOR)
                    .unwrap_or(Vec2::ZERO)
            }
            BehaviorMode::Reproduce => {
                direction_to_target(intent.target_entity, self_pos, &targets, &layout)
                    .unwrap_or(Vec2::ZERO)
            }
            BehaviorMode::Migrate { direction } => *direction,
        };

        if will.movement_intent() != movement {
            will.set_movement_intent(movement);
        }
    }
}

fn direction_to_target(
    target: Option<Entity>,
    self_pos: Vec2,
    targets: &Query<&GlobalTransform>,
    layout: &SimWorldTransformParams,
) -> Option<Vec2> {
    let target_entity = target?;
    let target_gtf = targets.get(target_entity).ok()?;
    let target_pos = sim_plane_pos(target_gtf.translation(), layout.use_xz_ground);
    let delta = target_pos - self_pos;
    if delta.length_squared() > 1e-6 {
        Some(delta.normalize())
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layers::behavior::{BehaviorCooldown, BehaviorIntent, BehavioralAgent};

    #[test]
    fn behavior_idle_when_satiated_and_safe() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);

        let id = app
            .world_mut()
            .spawn((
                BehavioralAgent,
                BehaviorIntent::default(),
                BehaviorCooldown::default(),
                BaseEnergy::new(1000.0),
                AlchemicalEngine::new(1000.0, 10.0, 10.0, 800.0),
            ))
            .id();

        // Manually run assess
        let assessment = EnergyAssessment {
            hunger_fraction: 0.2,
            energy_ratio: 0.8,
            biomass: 1000.0,
        };
        let awareness = SensoryAwareness {
            hostile_entity: None,
            hostile_distance: f32::MAX,
            food_entity: None,
            food_distance: f32::MAX,
        };
        app.world_mut()
            .entity_mut(id)
            .insert((assessment, awareness));

        app.add_systems(Update, behavior_decision_system);
        app.update();

        let intent = app.world().get::<BehaviorIntent>(id).unwrap();
        assert_eq!(intent.mode, BehaviorMode::Idle);
    }

    #[test]
    fn behavior_forage_when_hungry() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);

        let food = app
            .world_mut()
            .spawn(BaseEnergy::new(100.0))
            .id();

        let agent = app
            .world_mut()
            .spawn((
                BehavioralAgent,
                BehaviorIntent::default(),
                BehaviorCooldown::default(),
                BaseEnergy::new(500.0),
                AlchemicalEngine::new(1000.0, 10.0, 10.0, 100.0),
            ))
            .id();

        let assessment = EnergyAssessment {
            hunger_fraction: 0.9,
            energy_ratio: 0.1,
            biomass: 500.0,
        };
        let awareness = SensoryAwareness {
            hostile_entity: None,
            hostile_distance: f32::MAX,
            food_entity: Some(food),
            food_distance: 5.0,
        };
        app.world_mut()
            .entity_mut(agent)
            .insert((assessment, awareness));

        app.add_systems(Update, behavior_decision_system);
        app.update();

        let intent = app.world().get::<BehaviorIntent>(agent).unwrap();
        assert!(matches!(intent.mode, BehaviorMode::Forage { .. }));
        assert_eq!(intent.target_entity, Some(food));
    }

    #[test]
    fn behavior_flee_overrides_forage_when_panicking() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);

        let threat = app
            .world_mut()
            .spawn(BaseEnergy::new(5000.0))
            .id();

        let food = app
            .world_mut()
            .spawn(BaseEnergy::new(50.0))
            .id();

        let agent = app
            .world_mut()
            .spawn((
                BehavioralAgent,
                BehaviorIntent::default(),
                BehaviorCooldown::default(),
                BaseEnergy::new(200.0),
                AlchemicalEngine::new(1000.0, 10.0, 10.0, 100.0),
                InferenceProfile::new(0.5, 0.5, 0.0, 0.1),
            ))
            .id();

        let assessment = EnergyAssessment {
            hunger_fraction: 0.9,
            energy_ratio: 0.1,
            biomass: 200.0,
        };
        let awareness = SensoryAwareness {
            hostile_entity: Some(threat),
            hostile_distance: 2.0,
            food_entity: Some(food),
            food_distance: 5.0,
        };
        app.world_mut()
            .entity_mut(agent)
            .insert((assessment, awareness));

        app.add_systems(Update, behavior_decision_system);
        app.update();

        let intent = app.world().get::<BehaviorIntent>(agent).unwrap();
        assert!(matches!(intent.mode, BehaviorMode::Flee { .. }));
    }

    #[test]
    fn behavior_hunt_when_enemy_is_weaker() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);

        let prey = app
            .world_mut()
            .spawn(BaseEnergy::new(400.0))
            .id();

        let agent = app
            .world_mut()
            .spawn((
                BehavioralAgent,
                BehaviorIntent::default(),
                BehaviorCooldown::default(),
                BaseEnergy::new(1000.0),
                AlchemicalEngine::new(1000.0, 10.0, 10.0, 800.0),
                InferenceProfile::new(0.5, 0.9, 0.0, 0.8),
            ))
            .id();

        let assessment = EnergyAssessment {
            hunger_fraction: 0.2,
            energy_ratio: 0.8,
            biomass: 1000.0,
        };
        let awareness = SensoryAwareness {
            hostile_entity: Some(prey),
            hostile_distance: 2.0,
            food_entity: None,
            food_distance: f32::MAX,
        };
        app.world_mut()
            .entity_mut(agent)
            .insert((assessment, awareness));

        app.add_systems(Update, behavior_decision_system);
        app.update();

        let intent = app.world().get::<BehaviorIntent>(agent).unwrap();
        assert!(matches!(intent.mode, BehaviorMode::Hunt { .. }));
    }

    #[test]
    fn behavior_will_bridge_sets_correct_direction() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);

        let food = app
            .world_mut()
            .spawn((
                Transform::from_xyz(10.0, 0.0, 0.0),
                GlobalTransform::from(Transform::from_xyz(10.0, 0.0, 0.0)),
            ))
            .id();

        let agent = app
            .world_mut()
            .spawn((
                BehavioralAgent,
                BehaviorIntent {
                    mode: BehaviorMode::Forage { urgency: 0.5 },
                    target_entity: Some(food),
                },
                Transform::from_xyz(0.0, 0.0, 0.0),
                GlobalTransform::default(),
                WillActuator::default(),
            ))
            .id();

        app.insert_resource(SimWorldTransformParams::default());
        app.add_systems(Update, behavior_will_bridge_system);
        app.update();

        let will = app.world().get::<WillActuator>(agent).unwrap();
        let movement = will.movement_intent();
        assert!(movement.x > 0.9, "should move toward food on +x axis");
        assert!(movement.y.abs() < 0.1, "should not drift on y axis");
    }

    #[test]
    fn behavior_cooldown_tick_decrements() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);

        let id = app
            .world_mut()
            .spawn((
                BehavioralAgent,
                BehaviorCooldown {
                    decision_cooldown: 3,
                    action_cooldown: 2,
                },
            ))
            .id();

        app.add_systems(Update, behavior_cooldown_tick_system);
        app.update();

        let cd = app.world().get::<BehaviorCooldown>(id).unwrap();
        assert_eq!(cd.decision_cooldown, 2);
        assert_eq!(cd.action_cooldown, 1);
    }

    #[test]
    fn behavior_cooldown_tick_clamps_at_zero() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);

        let id = app
            .world_mut()
            .spawn((
                BehavioralAgent,
                BehaviorCooldown {
                    decision_cooldown: 0,
                    action_cooldown: 0,
                },
            ))
            .id();

        app.add_systems(Update, behavior_cooldown_tick_system);
        app.update();

        let cd = app.world().get::<BehaviorCooldown>(id).unwrap();
        assert_eq!(cd.decision_cooldown, 0);
        assert_eq!(cd.action_cooldown, 0);
    }
}
