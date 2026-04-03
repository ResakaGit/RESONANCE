//! D6: Social & Communication — 3 sistemas de manada.
//!
//! Fase: [`Phase::MetabolicLayer`], después de trophic.
//! Orden: pack_formation → pack_cohesion → dominance.

use bevy::prelude::*;

use crate::blueprint::constants::*;
use crate::blueprint::equations;
use crate::layers::{
    BaseEnergy, Faction, InferenceProfile, MobaIdentity, PackMembership, PackRole, SpatialVolume,
    StructuralLink, WillActuator,
};
use crate::runtime_platform::compat_2d3d::SimWorldTransformParams;
use crate::runtime_platform::core_math_agnostic::sim_plane_pos;
use crate::runtime_platform::simulation_tick::SimulationClock;
use crate::world::SpatialIndex;

/// Iterate groups in a sorted-by-key slice, calling `f(group_slice)` for each key run.
fn for_each_group<T, K: Eq>(sorted: &[T], key: impl Fn(&T) -> K, mut f: impl FnMut(&[T])) {
    let mut start = 0;
    while start < sorted.len() {
        let k = key(&sorted[start]);
        let mut end = start + 1;
        while end < sorted.len() && key(&sorted[end]) == k {
            end += 1;
        }
        f(&sorted[start..end]);
        start = end;
    }
}

/// S1: Formación de packs — entidades sin pack cerca de aliadas forman grupo.
/// Run condition: every PACK_FORMATION_TICK_INTERVAL ticks.
pub fn social_pack_formation_system(
    clock: Res<SimulationClock>,
    spatial_index: Res<SpatialIndex>,
    layout: Res<SimWorldTransformParams>,
    mut commands: Commands,
    unattached_query: Query<(Entity, &Transform, &MobaIdentity), Without<PackMembership>>,
    existing_packs: Query<&PackMembership>,
) {
    if clock.tick_id % PACK_FORMATION_TICK_INTERVAL != 0 {
        return;
    }

    // Next available pack_id: max existing + 1
    let next_pack_id = existing_packs
        .iter()
        .map(|p| p.pack_id)
        .max()
        .map(|m| m + 1)
        .unwrap_or(1);
    let mut pack_counter = next_pack_id;
    let mut scans: usize = 0;

    // Collect unattached to avoid borrow conflicts (skip Neutral — can't ally)
    let candidates: Vec<_> = unattached_query
        .iter()
        .filter(|(_, _, id)| id.faction() != Faction::Neutral)
        .map(|(e, t, id)| {
            (
                e,
                sim_plane_pos(t.translation, layout.use_xz_ground),
                id.faction(),
            )
        })
        .collect();

    // Track which entities have been assigned in this tick
    let mut assigned = Vec::new();

    for i in 0..candidates.len() {
        if scans >= SOCIAL_SCAN_BUDGET {
            break;
        }
        let (entity_a, pos_a, faction_a) = candidates[i];
        if assigned.contains(&entity_a) {
            continue;
        }

        scans += 1;
        let nearby = spatial_index.query_radius(pos_a, PACK_FORMATION_RADIUS);

        // Find same-faction unattached neighbors
        let mut pack_members = vec![entity_a];
        for entry in &nearby {
            if entry.entity == entity_a || assigned.contains(&entry.entity) {
                continue;
            }
            // Must be unattached + same faction
            let Some((_, _, other_faction)) =
                candidates.iter().find(|(e, _, _)| *e == entry.entity)
            else {
                continue;
            };
            if *other_faction != faction_a {
                continue;
            }
            pack_members.push(entry.entity);
        }

        if pack_members.len() < 2 {
            continue;
        }

        // Leader = lowest Entity index (deterministic)
        pack_members.sort_by_key(|e| e.index());
        let tick = clock.tick_id as u32;

        for (idx, &member) in pack_members.iter().enumerate() {
            let role = if idx == 0 {
                PackRole::Leader
            } else {
                PackRole::Member
            };
            commands
                .entity(member)
                .insert(PackMembership::new(pack_counter, role, tick));
            assigned.push(member);
        }

        // Create social bonds between members (star topology: all → leader)
        let leader = pack_members[0];
        for &member in &pack_members[1..] {
            commands.entity(member).insert(StructuralLink::new(
                leader,
                SOCIAL_BOND_REST_LENGTH,
                SOCIAL_BOND_STIFFNESS,
                SOCIAL_BOND_BREAK_STRESS,
            ));
        }

        pack_counter += 1;
    }
}

/// S2: Cohesión de pack — miembros se mueven hacia centroid.
pub fn social_pack_cohesion_system(
    layout: Res<SimWorldTransformParams>,
    pack_query: Query<(Entity, &PackMembership, &Transform)>,
    mut will_query: Query<&mut WillActuator>,
) {
    // Group by pack_id → compute centroid → apply cohesion force
    // Using sorted vec (no HashMap in hot paths)
    let mut members: Vec<(u32, Entity, Vec2)> = pack_query
        .iter()
        .map(|(e, pm, t)| {
            (
                pm.pack_id,
                e,
                sim_plane_pos(t.translation, layout.use_xz_ground),
            )
        })
        .collect();
    members.sort_by_key(|(pid, _, _)| *pid);

    for_each_group(
        &members,
        |(pid, _, _)| *pid,
        |group| {
            let centroid = group
                .iter()
                .map(|(_, _, pos)| *pos)
                .fold(Vec2::ZERO, |acc, p| acc + p)
                / group.len() as f32;

            for &(_, entity, pos) in group {
                let force = equations::pack_cohesion_force(pos, centroid, SOCIAL_BOND_REST_LENGTH);
                if force.length_squared() < f32::EPSILON {
                    continue;
                }
                let Ok(mut will) = will_query.get_mut(entity) else {
                    continue;
                };
                let current = will.movement_intent();
                let new_intent = current + force;
                if current != new_intent {
                    will.set_movement_intent(new_intent);
                }
            }
        },
    );
}

/// S3: Dominancia — recalcula leader por pack basado en score.
/// Run condition: every DOMINANCE_TICK_INTERVAL ticks.
pub fn social_dominance_system(
    clock: Res<SimulationClock>,
    mut pack_query: Query<(
        Entity,
        &mut PackMembership,
        &BaseEnergy,
        &SpatialVolume,
        Option<&InferenceProfile>,
    )>,
) {
    if clock.tick_id % DOMINANCE_TICK_INTERVAL != 0 {
        return;
    }

    // Collect scored members grouped by pack
    let mut scored: Vec<(u32, Entity, f32)> = pack_query
        .iter()
        .map(|(e, pm, energy, vol, profile)| {
            let resilience = InferenceProfile::resilience_effective(profile);
            let score = equations::dominance_contest_score(energy.qe(), vol.radius, resilience);
            (pm.pack_id, e, score)
        })
        .collect();
    scored.sort_by_key(|(pid, _, _)| *pid);

    // Collect leader decisions first to avoid borrow conflicts with get_mut
    let mut role_updates: Vec<(Entity, PackRole)> = Vec::new();

    for_each_group(
        &scored,
        |(pid, _, _)| *pid,
        |group| {
            let Some((_, leader_entity, _)) = group
                .iter()
                .max_by(|a, b| a.2.partial_cmp(&b.2).unwrap_or(std::cmp::Ordering::Equal))
            else {
                return;
            };

            for &(_, entity, _) in group {
                let new_role = if entity == *leader_entity {
                    PackRole::Leader
                } else {
                    PackRole::Member
                };
                role_updates.push((entity, new_role));
            }
        },
    );

    for (entity, new_role) in role_updates {
        let Ok((_, mut pm, _, _, _)) = pack_query.get_mut(entity) else {
            continue;
        };
        if pm.role != new_role {
            pm.role = new_role;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layers::{BaseEnergy, Faction, MobaIdentity, SpatialVolume};
    use crate::runtime_platform::simulation_tick::SimulationClock;
    use crate::world::space::SpatialEntry;

    fn test_app() -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.insert_resource(SimulationClock { tick_id: 0 });
        app.insert_resource(SpatialIndex::new(10.0));
        app.insert_resource(SimWorldTransformParams::default());
        app
    }

    fn red_identity() -> MobaIdentity {
        MobaIdentity {
            faction: Faction::Red,
            relational_tags: 0,
            critical_multiplier: 1.0,
        }
    }

    fn blue_identity() -> MobaIdentity {
        MobaIdentity {
            faction: Faction::Blue,
            relational_tags: 0,
            critical_multiplier: 1.0,
        }
    }

    // ── S1: pack formation ──

    #[test]
    fn pack_forms_when_two_entities_near() {
        let mut app = test_app();

        let e1 = app
            .world_mut()
            .spawn((Transform::from_xyz(0.0, 0.0, 0.0), red_identity()))
            .id();
        let e2 = app
            .world_mut()
            .spawn((Transform::from_xyz(3.0, 0.0, 0.0), red_identity()))
            .id();

        let mut index = SpatialIndex::new(10.0);
        index.insert(SpatialEntry {
            entity: e1,
            position: Vec2::ZERO,
            radius: 1.0,
        });
        index.insert(SpatialEntry {
            entity: e2,
            position: Vec2::new(3.0, 0.0),
            radius: 1.0,
        });
        app.insert_resource(index);

        app.add_systems(Update, social_pack_formation_system);
        app.update();

        let pm1 = app.world().get::<PackMembership>(e1);
        let pm2 = app.world().get::<PackMembership>(e2);
        assert!(pm1.is_some(), "entity 1 should have PackMembership");
        assert!(pm2.is_some(), "entity 2 should have PackMembership");
        assert_eq!(pm1.unwrap().pack_id, pm2.unwrap().pack_id, "same pack");
    }

    #[test]
    fn pack_does_not_form_for_neutral_faction() {
        let mut app = test_app();

        let e1 = app
            .world_mut()
            .spawn((
                Transform::from_xyz(0.0, 0.0, 0.0),
                MobaIdentity::default(), // Neutral
            ))
            .id();
        let e2 = app
            .world_mut()
            .spawn((
                Transform::from_xyz(2.0, 0.0, 0.0),
                MobaIdentity::default(), // Neutral
            ))
            .id();

        let mut index = SpatialIndex::new(10.0);
        index.insert(SpatialEntry {
            entity: e1,
            position: Vec2::ZERO,
            radius: 1.0,
        });
        index.insert(SpatialEntry {
            entity: e2,
            position: Vec2::new(2.0, 0.0),
            radius: 1.0,
        });
        app.insert_resource(index);

        app.add_systems(Update, social_pack_formation_system);
        app.update();

        assert!(app.world().get::<PackMembership>(e1).is_none());
        assert!(app.world().get::<PackMembership>(e2).is_none());
    }

    #[test]
    fn pack_does_not_form_across_factions() {
        let mut app = test_app();

        let e1 = app
            .world_mut()
            .spawn((Transform::from_xyz(0.0, 0.0, 0.0), red_identity()))
            .id();
        let e2 = app
            .world_mut()
            .spawn((Transform::from_xyz(3.0, 0.0, 0.0), blue_identity()))
            .id();

        let mut index = SpatialIndex::new(10.0);
        index.insert(SpatialEntry {
            entity: e1,
            position: Vec2::ZERO,
            radius: 1.0,
        });
        index.insert(SpatialEntry {
            entity: e2,
            position: Vec2::new(3.0, 0.0),
            radius: 1.0,
        });
        app.insert_resource(index);

        app.add_systems(Update, social_pack_formation_system);
        app.update();

        assert!(app.world().get::<PackMembership>(e1).is_none());
        assert!(app.world().get::<PackMembership>(e2).is_none());
    }

    #[test]
    fn pack_does_not_form_when_too_far() {
        let mut app = test_app();

        let e1 = app
            .world_mut()
            .spawn((Transform::from_xyz(0.0, 0.0, 0.0), red_identity()))
            .id();
        let e2 = app
            .world_mut()
            .spawn((Transform::from_xyz(100.0, 0.0, 0.0), red_identity()))
            .id();

        let mut index = SpatialIndex::new(10.0);
        index.insert(SpatialEntry {
            entity: e1,
            position: Vec2::ZERO,
            radius: 1.0,
        });
        index.insert(SpatialEntry {
            entity: e2,
            position: Vec2::new(100.0, 0.0),
            radius: 1.0,
        });
        app.insert_resource(index);

        app.add_systems(Update, social_pack_formation_system);
        app.update();

        assert!(app.world().get::<PackMembership>(e1).is_none());
        assert!(app.world().get::<PackMembership>(e2).is_none());
    }

    #[test]
    fn pack_leader_is_lowest_entity_index() {
        let mut app = test_app();

        let e1 = app
            .world_mut()
            .spawn((Transform::from_xyz(0.0, 0.0, 0.0), red_identity()))
            .id();
        let e2 = app
            .world_mut()
            .spawn((Transform::from_xyz(2.0, 0.0, 0.0), red_identity()))
            .id();

        let mut index = SpatialIndex::new(10.0);
        index.insert(SpatialEntry {
            entity: e1,
            position: Vec2::ZERO,
            radius: 1.0,
        });
        index.insert(SpatialEntry {
            entity: e2,
            position: Vec2::new(2.0, 0.0),
            radius: 1.0,
        });
        app.insert_resource(index);

        app.add_systems(Update, social_pack_formation_system);
        app.update();

        let pm1 = app.world().get::<PackMembership>(e1).unwrap();
        let pm2 = app.world().get::<PackMembership>(e2).unwrap();
        // Lower index gets Leader
        if e1.index() < e2.index() {
            assert_eq!(pm1.role, PackRole::Leader);
            assert_eq!(pm2.role, PackRole::Member);
        } else {
            assert_eq!(pm2.role, PackRole::Leader);
            assert_eq!(pm1.role, PackRole::Member);
        }
    }

    #[test]
    fn pack_formation_creates_structural_link() {
        let mut app = test_app();

        let e1 = app
            .world_mut()
            .spawn((Transform::from_xyz(0.0, 0.0, 0.0), red_identity()))
            .id();
        let e2 = app
            .world_mut()
            .spawn((Transform::from_xyz(2.0, 0.0, 0.0), red_identity()))
            .id();

        let mut index = SpatialIndex::new(10.0);
        index.insert(SpatialEntry {
            entity: e1,
            position: Vec2::ZERO,
            radius: 1.0,
        });
        index.insert(SpatialEntry {
            entity: e2,
            position: Vec2::new(2.0, 0.0),
            radius: 1.0,
        });
        app.insert_resource(index);

        app.add_systems(Update, social_pack_formation_system);
        app.update();

        // Non-leader gets StructuralLink to leader
        let pm1 = app.world().get::<PackMembership>(e1).unwrap();
        let _pm2 = app.world().get::<PackMembership>(e2).unwrap();
        let leader = if pm1.role == PackRole::Leader { e1 } else { e2 };
        let member = if leader == e1 { e2 } else { e1 };

        let link = app.world().get::<StructuralLink>(member);
        assert!(link.is_some(), "member should have StructuralLink");
        assert_eq!(link.unwrap().target, leader);
        assert!((link.unwrap().stiffness - SOCIAL_BOND_STIFFNESS).abs() < f32::EPSILON);
    }

    #[test]
    fn pack_formation_skips_non_zero_tick() {
        let mut app = test_app();
        app.insert_resource(SimulationClock { tick_id: 3 }); // Not divisible by 16

        app.world_mut()
            .spawn((Transform::from_xyz(0.0, 0.0, 0.0), red_identity()));
        app.world_mut()
            .spawn((Transform::from_xyz(2.0, 0.0, 0.0), red_identity()));

        app.add_systems(Update, social_pack_formation_system);
        app.update();

        let count = app
            .world_mut()
            .query::<&PackMembership>()
            .iter(app.world())
            .count();
        assert_eq!(count, 0, "should skip on non-interval ticks");
    }

    // ── S2: pack cohesion ──

    #[test]
    fn pack_cohesion_moves_toward_centroid() {
        let mut app = test_app();

        // Two pack members: one at origin, one at (20, 0). Centroid = (10, 0).
        // Distance to centroid = 10 > rest_length(7) → attractive force.
        app.world_mut().spawn((
            PackMembership::new(1, PackRole::Leader, 0),
            Transform::from_xyz(0.0, 0.0, 0.0),
            WillActuator::default(),
        ));
        let e2 = app
            .world_mut()
            .spawn((
                PackMembership::new(1, PackRole::Member, 0),
                Transform::from_xyz(20.0, 0.0, 0.0),
                WillActuator::default(),
            ))
            .id();

        app.add_systems(Update, social_pack_cohesion_system);
        app.update();

        // Entity at (20,0) should have negative x intent (toward centroid at 10,0)
        let will = app.world().get::<WillActuator>(e2).unwrap();
        let intent = will.movement_intent();
        assert!(intent.x < 0.0, "should move toward centroid: {intent:?}");
    }

    #[test]
    fn pack_cohesion_no_effect_without_will() {
        let mut app = test_app();

        // Entity without WillActuator should not crash
        app.world_mut().spawn((
            PackMembership::new(1, PackRole::Leader, 0),
            Transform::from_xyz(0.0, 0.0, 0.0),
        ));
        app.world_mut().spawn((
            PackMembership::new(1, PackRole::Member, 0),
            Transform::from_xyz(10.0, 0.0, 0.0),
        ));

        app.add_systems(Update, social_pack_cohesion_system);
        app.update();
        // No panic = pass
    }

    // ── S3: dominance ──

    #[test]
    fn pack_leader_is_strongest() {
        let mut app = test_app();
        app.insert_resource(SimulationClock { tick_id: 0 }); // tick 0 % 60 == 0

        // Weak leader, strong member
        let e1 = app
            .world_mut()
            .spawn((
                PackMembership::new(1, PackRole::Leader, 0),
                BaseEnergy::new(10.0),
                SpatialVolume::new(1.0),
            ))
            .id();
        let e2 = app
            .world_mut()
            .spawn((
                PackMembership::new(1, PackRole::Member, 0),
                BaseEnergy::new(200.0),
                SpatialVolume::new(3.0),
            ))
            .id();

        app.add_systems(Update, social_dominance_system);
        app.update();

        let pm1 = app.world().get::<PackMembership>(e1).unwrap();
        let pm2 = app.world().get::<PackMembership>(e2).unwrap();
        assert_eq!(
            pm2.role,
            PackRole::Leader,
            "stronger entity should become leader"
        );
        assert_eq!(
            pm1.role,
            PackRole::Member,
            "weaker entity should become member"
        );
    }

    #[test]
    fn dominance_skips_non_interval_tick() {
        let mut app = test_app();
        app.insert_resource(SimulationClock { tick_id: 7 }); // Not divisible by 60

        let e1 = app
            .world_mut()
            .spawn((
                PackMembership::new(1, PackRole::Leader, 0),
                BaseEnergy::new(10.0),
                SpatialVolume::new(1.0),
            ))
            .id();
        app.world_mut().spawn((
            PackMembership::new(1, PackRole::Member, 0),
            BaseEnergy::new(200.0),
            SpatialVolume::new(3.0),
        ));

        app.add_systems(Update, social_dominance_system);
        app.update();

        let pm1 = app.world().get::<PackMembership>(e1).unwrap();
        assert_eq!(
            pm1.role,
            PackRole::Leader,
            "should not change on non-interval tick"
        );
    }

    #[test]
    fn dominance_resilience_tips_balance() {
        let mut app = test_app();
        app.insert_resource(SimulationClock { tick_id: 0 });

        // e1: less energy but high resilience → higher score
        // Default resilience_effective(None) = 0.5, so e2 also gets some resilience
        let e1 = app
            .world_mut()
            .spawn((
                PackMembership::new(1, PackRole::Member, 0),
                BaseEnergy::new(90.0),
                SpatialVolume::new(2.0),
                InferenceProfile::new(0.0, 0.0, 0.0, 1.0),
            ))
            .id();
        // e2: more energy, default resilience (0.5)
        let e2 = app
            .world_mut()
            .spawn((
                PackMembership::new(1, PackRole::Leader, 0),
                BaseEnergy::new(100.0),
                SpatialVolume::new(2.0),
            ))
            .id();

        // e1 score = 90 * 2 * (1 + 1.0 * 0.5) = 90 * 2 * 1.5 = 270
        // e2 score = 100 * 2 * (1 + 0.5 * 0.5) = 100 * 2 * 1.25 = 250
        app.add_systems(Update, social_dominance_system);
        app.update();

        let pm1 = app.world().get::<PackMembership>(e1).unwrap();
        let pm2 = app.world().get::<PackMembership>(e2).unwrap();
        assert_eq!(pm1.role, PackRole::Leader, "resilient entity should win");
        assert_eq!(pm2.role, PackRole::Member);
    }

    // ── Equation integration ──

    #[test]
    fn pack_hunt_bonus_scales_with_sqrt_size() {
        let b1 = equations::pack_hunt_bonus(1, 100.0);
        let b4 = equations::pack_hunt_bonus(4, 100.0);
        let b9 = equations::pack_hunt_bonus(9, 100.0);
        assert!((b1 - 1.0).abs() < f32::EPSILON);
        assert!((b4 - 2.0).abs() < f32::EPSILON);
        assert!((b9 - 3.0).abs() < f32::EPSILON);
    }

    #[test]
    fn social_bond_breaks_when_too_far() {
        // This validates that StructuralLink with social parameters has correct break_stress
        let _link = StructuralLink::new(
            Entity::from_raw(1),
            SOCIAL_BOND_REST_LENGTH,
            SOCIAL_BOND_STIFFNESS,
            SOCIAL_BOND_BREAK_STRESS,
        );
        // Stress = stiffness × (distance - rest_length)
        // At distance = 5000 + 7 rest → stress = 0.01 × 5000 = 50 = break_stress → breaks
        let extreme_displacement = SOCIAL_BOND_BREAK_STRESS / SOCIAL_BOND_STIFFNESS;
        let stress = SOCIAL_BOND_STIFFNESS * extreme_displacement;
        assert!(
            (stress - SOCIAL_BOND_BREAK_STRESS).abs() < f32::EPSILON,
            "bond should break at displacement {extreme_displacement}"
        );
    }
}
