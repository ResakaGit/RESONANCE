//! D5: Sensory & perception systems — frequency-based scan, threat memory, panic events.

use bevy::prelude::*;

use crate::blueprint::{constants, equations};
use crate::events::ThreatDetectedEvent;
use crate::layers::behavior::{BehavioralAgent, SensoryAwareness};
use crate::layers::{
    AlchemicalEngine, BaseEnergy, CapabilitySet, FlowVector, Faction, MobaIdentity,
    TrophicConsumer,
};
use crate::runtime_platform::compat_2d3d::SimWorldTransformParams;
use crate::runtime_platform::core_math_agnostic::sim_plane_pos;
use crate::world::SpatialIndex;

/// Throttle cursor for sensory frequency scan.
#[derive(Resource, Default)]
pub struct SensoryScanCursor {
    pub start_index: usize,
}

/// Persistent threat memory with spatial + temporal decay.
#[derive(Component, Reflect, Debug, Clone)]
#[component(storage = "SparseSet")]
pub struct ThreatMemory {
    pub last_threat_position: Vec2,
    pub ticks_since_seen: u32,
}

// ---------------------------------------------------------------------------
// S1: Frequency scan — detects threats and food via equations.
// ---------------------------------------------------------------------------

/// Scans nearby entities using frequency detection range and classifies threats/food.
/// Phase: Input, before BehaviorSet::Assess.
pub fn sensory_frequency_scan_system(
    layout: Res<SimWorldTransformParams>,
    spatial: Res<SpatialIndex>,
    mut cursor: ResMut<SensoryScanCursor>,
    mut commands: Commands,
    agents: Query<
        (Entity, &Transform, &CapabilitySet, &AlchemicalEngine),
        (With<BehavioralAgent>, Without<SensoryAwareness>),
    >,
    identities: Query<&MobaIdentity>,
    energies: Query<&BaseEnergy>,
    flows: Query<&FlowVector>,
    trophic_consumers: Query<&TrophicConsumer>,
) {
    let mut sorted: Vec<_> = agents.iter().collect();
    sorted.sort_by_key(|(e, ..)| e.to_bits());

    if sorted.is_empty() {
        cursor.start_index = 0;
        return;
    }

    let start = cursor.start_index.min(sorted.len());
    let mut budget = constants::SENSORY_SCAN_BUDGET;
    let mut scanned = 0_usize;

    let iter = sorted[start..].iter().chain(sorted[..start].iter());
    for &(entity, transform, caps, engine) in iter {
        if budget == 0 {
            break;
        }
        if !caps.can_sense() {
            continue;
        }
        budget -= 1;
        scanned += 1;

        let pos = sim_plane_pos(transform.translation, layout.use_xz_ground);
        let self_qe = energies.get(entity).map(|e| e.qe()).unwrap_or(0.0);
        let scan_range = equations::frequency_detection_range(
            constants::SENSORY_BASE_SENSITIVITY,
            self_qe,
            constants::SENSORY_NOISE_FLOOR,
        )
        .min(constants::SENSORY_MAX_SCAN_RANGE);

        let nearby = spatial.query_radius(pos, scan_range);
        let self_identity = identities.get(entity).ok();

        let (hunger, _) = equations::assess_energy(engine.buffer_level(), engine.buffer_cap());

        let mut best_threat: Option<Entity> = None;
        let mut best_threat_score: f32 = 0.0;
        let mut best_threat_dist: f32 = f32::MAX;
        let mut best_food: Option<Entity> = None;
        let mut best_food_score: f32 = 0.0;
        let mut best_food_dist: f32 = f32::MAX;

        for entry in &nearby {
            if entry.entity == entity {
                continue;
            }
            let dist = (entry.position - pos).length();

            let is_enemy = self_identity
                .and_then(|si| identities.get(entry.entity).ok().map(|oi| si.is_enemy(oi)))
                .unwrap_or(false);
            let is_predator = trophic_consumers
                .get(entry.entity)
                .map(|tc| tc.is_predator())
                .unwrap_or(false);

            if is_enemy || is_predator {
                let other_qe = energies.get(entry.entity).map(|e| e.qe()).unwrap_or(0.0);
                let speed = flows
                    .get(entry.entity)
                    .map(|f| f.velocity().length())
                    .unwrap_or(0.0);
                let score =
                    equations::threat_level_assessment(other_qe, speed, is_predator, dist);
                if score > best_threat_score {
                    best_threat_score = score;
                    best_threat = Some(entry.entity);
                    best_threat_dist = dist;
                }
            }

            let other_faction = identities.get(entry.entity).ok().map(|i| i.faction());
            let is_food = other_faction
                .map(|f| f == Faction::Neutral || f == Faction::Wild)
                .unwrap_or(false)
                && energies.get(entry.entity).is_ok();

            if is_food {
                let food_qe = energies.get(entry.entity).map(|e| e.qe()).unwrap_or(0.0);
                let score = equations::food_attractiveness(food_qe, dist, hunger);
                if score > best_food_score {
                    best_food_score = score;
                    best_food = Some(entry.entity);
                    best_food_dist = dist;
                }
            }
        }

        commands.entity(entity).insert(SensoryAwareness {
            hostile_entity: best_threat,
            hostile_distance: if best_threat.is_some() {
                best_threat_dist
            } else {
                f32::MAX
            },
            food_entity: best_food,
            food_distance: if best_food.is_some() {
                best_food_dist
            } else {
                f32::MAX
            },
        });
    }

    cursor.start_index = (start + scanned) % sorted.len().max(1);
}

// ---------------------------------------------------------------------------
// S2: Threat memory — persists threat position across ticks.
// ---------------------------------------------------------------------------

/// Updates threat memory: records position when threat detected, decays when lost.
/// Phase: Input, after S1.
pub fn sensory_threat_memory_system(
    mut commands: Commands,
    mut query: Query<
        (Entity, &SensoryAwareness, Option<&mut ThreatMemory>),
        With<BehavioralAgent>,
    >,
    transforms: Query<&Transform>,
    layout: Res<SimWorldTransformParams>,
) {
    for (entity, awareness, memory_opt) in &mut query {
        if let Some(threat_entity) = awareness.hostile_entity {
            let threat_pos = transforms
                .get(threat_entity)
                .map(|t| sim_plane_pos(t.translation, layout.use_xz_ground))
                .unwrap_or(Vec2::ZERO);

            if let Some(mut memory) = memory_opt {
                if memory.last_threat_position != threat_pos {
                    memory.last_threat_position = threat_pos;
                }
                if memory.ticks_since_seen != 0 {
                    memory.ticks_since_seen = 0;
                }
            } else {
                commands.entity(entity).insert(ThreatMemory {
                    last_threat_position: threat_pos,
                    ticks_since_seen: 0,
                });
            }
        } else if let Some(mut memory) = memory_opt {
            memory.ticks_since_seen += 1;
            if memory.ticks_since_seen > constants::SENSORY_MEMORY_DECAY_TICKS {
                commands.entity(entity).remove::<ThreatMemory>();
            }
        }
    }
}

// ---------------------------------------------------------------------------
// S3: Awareness event — emits ThreatDetectedEvent on panic threshold.
// ---------------------------------------------------------------------------

/// Emits ThreatDetectedEvent when threat level exceeds panic threshold.
/// Phase: Input, after S2.
pub fn sensory_awareness_event_system(
    query: Query<(Entity, &SensoryAwareness), With<BehavioralAgent>>,
    energies: Query<&BaseEnergy>,
    flows: Query<&FlowVector>,
    trophic_consumers: Query<&TrophicConsumer>,
    mut events: EventWriter<ThreatDetectedEvent>,
) {
    for (entity, awareness) in &query {
        let Some(threat_entity) = awareness.hostile_entity else {
            continue;
        };

        let threat_qe = energies.get(threat_entity).map(|e| e.qe()).unwrap_or(0.0);
        let speed = flows
            .get(threat_entity)
            .map(|f| f.velocity().length())
            .unwrap_or(0.0);
        let is_predator = trophic_consumers
            .get(threat_entity)
            .map(|tc| tc.is_predator())
            .unwrap_or(false);

        let level = equations::threat_level_assessment(
            threat_qe,
            speed,
            is_predator,
            awareness.hostile_distance,
        );

        if level > constants::SENSORY_PANIC_THRESHOLD {
            events.send(ThreatDetectedEvent {
                entity,
                threat: threat_entity,
                threat_level: level,
            });
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::events::DeathEvent;
    use crate::layers::behavior::{BehaviorCooldown, BehaviorIntent};
    use crate::layers::inference::TrophicClass;

    fn test_app() -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_event::<DeathEvent>();
        app.add_event::<ThreatDetectedEvent>();
        app.init_resource::<SpatialIndex>();
        app.init_resource::<SensoryScanCursor>();
        app.insert_resource(SimWorldTransformParams::default());
        app.add_systems(
            Update,
            (
                sensory_frequency_scan_system,
                sensory_threat_memory_system,
                sensory_awareness_event_system,
            )
                .chain(),
        );
        app
    }

    fn spawn_behavioral_agent(app: &mut App, pos: Vec3, qe: f32, sense: bool) -> Entity {
        let flags = if sense {
            CapabilitySet::MOVE | CapabilitySet::SENSE
        } else {
            CapabilitySet::MOVE
        };
        app.world_mut()
            .spawn((
                BehavioralAgent,
                BehaviorIntent::default(),
                BehaviorCooldown::default(),
                Transform::from_translation(pos),
                CapabilitySet::new(flags),
                BaseEnergy::new(qe),
                AlchemicalEngine::new(100.0, 10.0, 10.0, 50.0),
                MobaIdentity {
                    faction: Faction::Red,
                    relational_tags: 0,
                    critical_multiplier: 1.0,
                },
            ))
            .id()
    }

    fn spawn_target(
        app: &mut App,
        pos: Vec3,
        qe: f32,
        faction: Faction,
        radius: f32,
    ) -> Entity {
        app.world_mut()
            .spawn((
                Transform::from_translation(pos),
                BaseEnergy::new(qe),
                crate::layers::SpatialVolume::new(radius),
                MobaIdentity {
                    faction,
                    relational_tags: 0,
                    critical_multiplier: 1.0,
                },
            ))
            .id()
    }

    fn rebuild_spatial(app: &mut App) {
        let mut index = app.world_mut().resource_mut::<SpatialIndex>();
        index.clear();
        let mut entries = Vec::new();
        for (entity, transform, volume) in app
            .world_mut()
            .query::<(Entity, &Transform, &crate::layers::SpatialVolume)>()
            .iter(app.world())
        {
            entries.push(crate::world::SpatialEntry {
                entity,
                position: sim_plane_pos(transform.translation, false),
                radius: volume.radius,
            });
        }
        let mut index = app.world_mut().resource_mut::<SpatialIndex>();
        for entry in entries {
            index.insert(entry);
        }
    }

    // ── S1: frequency scan ──

    #[test]
    fn scan_detects_enemy_as_threat() {
        let mut app = test_app();

        let agent = spawn_behavioral_agent(&mut app, Vec3::ZERO, 500.0, true);
        // Also give agent spatial volume so it appears in index
        app.world_mut()
            .entity_mut(agent)
            .insert(crate::layers::SpatialVolume::new(1.0));

        let enemy = spawn_target(&mut app, Vec3::new(5.0, 0.0, 0.0), 300.0, Faction::Blue, 1.0);

        rebuild_spatial(&mut app);
        app.update();

        let awareness = app.world().get::<SensoryAwareness>(agent);
        assert!(awareness.is_some(), "agent should have SensoryAwareness");
        let a = awareness.unwrap();
        assert_eq!(a.hostile_entity, Some(enemy));
        assert!(a.hostile_distance < 10.0);
    }

    #[test]
    fn scan_detects_neutral_as_food() {
        let mut app = test_app();

        let agent = spawn_behavioral_agent(&mut app, Vec3::ZERO, 500.0, true);
        app.world_mut()
            .entity_mut(agent)
            .insert(crate::layers::SpatialVolume::new(1.0));

        let food = spawn_target(
            &mut app,
            Vec3::new(3.0, 0.0, 0.0),
            100.0,
            Faction::Neutral,
            1.0,
        );

        rebuild_spatial(&mut app);
        app.update();

        let a = app.world().get::<SensoryAwareness>(agent).unwrap();
        assert_eq!(a.food_entity, Some(food));
    }

    #[test]
    fn scan_skips_entities_without_sense() {
        let mut app = test_app();

        let agent = spawn_behavioral_agent(&mut app, Vec3::ZERO, 500.0, false);
        app.world_mut()
            .entity_mut(agent)
            .insert(crate::layers::SpatialVolume::new(1.0));

        spawn_target(&mut app, Vec3::new(5.0, 0.0, 0.0), 300.0, Faction::Blue, 1.0);

        rebuild_spatial(&mut app);
        app.update();

        assert!(
            app.world().get::<SensoryAwareness>(agent).is_none(),
            "agent without SENSE should not get SensoryAwareness from D5",
        );
    }

    // ── S2: threat memory ──

    #[test]
    fn threat_memory_created_on_threat_detection() {
        let mut app = test_app();

        let agent = spawn_behavioral_agent(&mut app, Vec3::ZERO, 500.0, true);
        app.world_mut()
            .entity_mut(agent)
            .insert(crate::layers::SpatialVolume::new(1.0));

        spawn_target(&mut app, Vec3::new(5.0, 0.0, 0.0), 300.0, Faction::Blue, 1.0);

        rebuild_spatial(&mut app);
        app.update();

        let memory = app.world().get::<ThreatMemory>(agent);
        assert!(memory.is_some(), "should create ThreatMemory on threat detection");
        assert_eq!(memory.unwrap().ticks_since_seen, 0);
    }

    #[test]
    fn threat_memory_decays_over_time() {
        let mut app = test_app();

        let agent = spawn_behavioral_agent(&mut app, Vec3::ZERO, 500.0, true);
        app.world_mut().entity_mut(agent).insert((
            crate::layers::SpatialVolume::new(1.0),
            ThreatMemory {
                last_threat_position: Vec2::new(5.0, 0.0),
                ticks_since_seen: 0,
            },
            // Insert awareness with no threat to test decay
            SensoryAwareness {
                hostile_entity: None,
                hostile_distance: f32::MAX,
                food_entity: None,
                food_distance: f32::MAX,
            },
        ));

        // Don't rebuild spatial (no targets → scan won't overwrite awareness since agent already has it)
        // But S2 reads existing awareness. We need to run just S2.
        // Simpler: create minimal app with just S2.
        let mut app2 = App::new();
        app2.add_plugins(MinimalPlugins);
        app2.insert_resource(SimWorldTransformParams::default());
        app2.add_systems(Update, sensory_threat_memory_system);

        let agent2 = app2
            .world_mut()
            .spawn((
                BehavioralAgent,
                SensoryAwareness {
                    hostile_entity: None,
                    hostile_distance: f32::MAX,
                    food_entity: None,
                    food_distance: f32::MAX,
                },
                ThreatMemory {
                    last_threat_position: Vec2::new(5.0, 0.0),
                    ticks_since_seen: 0,
                },
            ))
            .id();

        app2.update();

        let m = app2.world().get::<ThreatMemory>(agent2).unwrap();
        assert_eq!(m.ticks_since_seen, 1, "should increment ticks_since_seen");
    }

    #[test]
    fn threat_memory_removed_after_decay_threshold() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.insert_resource(SimWorldTransformParams::default());
        app.add_systems(Update, sensory_threat_memory_system);

        let agent = app
            .world_mut()
            .spawn((
                BehavioralAgent,
                SensoryAwareness {
                    hostile_entity: None,
                    hostile_distance: f32::MAX,
                    food_entity: None,
                    food_distance: f32::MAX,
                },
                ThreatMemory {
                    last_threat_position: Vec2::new(5.0, 0.0),
                    ticks_since_seen: constants::SENSORY_MEMORY_DECAY_TICKS,
                },
            ))
            .id();

        app.update();

        assert!(
            app.world().get::<ThreatMemory>(agent).is_none(),
            "memory should be removed after decay threshold",
        );
    }

    // ── S3: awareness event ──

    #[test]
    fn panic_event_emitted_for_high_threat() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_event::<DeathEvent>();
        app.add_event::<ThreatDetectedEvent>();
        app.add_systems(Update, sensory_awareness_event_system);

        // Spawn large predator threat very close
        let threat = app
            .world_mut()
            .spawn((
                BaseEnergy::new(5000.0),
                FlowVector::new(Vec2::new(10.0, 0.0), 0.1),
                TrophicConsumer::new(TrophicClass::Carnivore, 5.0),
            ))
            .id();

        app.world_mut().spawn((
            BehavioralAgent,
            SensoryAwareness {
                hostile_entity: Some(threat),
                hostile_distance: 1.0,
                food_entity: None,
                food_distance: f32::MAX,
            },
        ));

        app.update();

        let events: Vec<_> = app
            .world_mut()
            .resource_mut::<Events<ThreatDetectedEvent>>()
            .drain()
            .collect();
        assert!(
            !events.is_empty(),
            "should emit ThreatDetectedEvent for high threat",
        );
        assert!(events[0].threat_level > constants::SENSORY_PANIC_THRESHOLD);
    }

    #[test]
    fn no_panic_event_for_weak_threat() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_event::<DeathEvent>();
        app.add_event::<ThreatDetectedEvent>();
        app.add_systems(Update, sensory_awareness_event_system);

        let threat = app
            .world_mut()
            .spawn(BaseEnergy::new(10.0))
            .id();

        app.world_mut().spawn((
            BehavioralAgent,
            SensoryAwareness {
                hostile_entity: Some(threat),
                hostile_distance: 50.0,
                food_entity: None,
                food_distance: f32::MAX,
            },
        ));

        app.update();

        let events: Vec<_> = app
            .world_mut()
            .resource_mut::<Events<ThreatDetectedEvent>>()
            .drain()
            .collect();
        assert!(
            events.is_empty(),
            "should NOT emit event for weak distant threat",
        );
    }
}
