//! Reproducción por biomasa: dispersión de semillas cuando el radio supera umbral (EA6).
//!
//! No modifica `GrowthBudget` ni `allometric_growth`; observa `SpatialVolume`, `AllometricRadiusAnchor`,
//! `InferenceProfile` y `CapabilitySet`. Matemática en [`crate::blueprint::equations`].
//! Drenaje parental vía [`EnergyOps::drain`](crate::layers::energy::EnergyOps::drain) para contrato L0.

mod constants;

use bevy::prelude::*;

use crate::blueprint::{equations, ElementId};
use crate::entities::builder::EntityBuilder;
use crate::events::DeathCause;
use crate::layers::{
    AllometricRadiusAnchor, BaseEnergy, CapabilitySet, EnergyOps, InferenceProfile, MatterState,
    SpatialVolume,
};
use crate::worldgen::{EnergyFieldGrid, Materialized, WorldArchetype};

pub use constants::{
    MAX_REPRODUCTIONS_PER_FRAME, REPRODUCTION_RADIUS_FACTOR, SEED_ENERGY_FRACTION,
};

/// Cooldown: evita reproducción cada frame en la misma entidad (`SparseSet` = transitorio).
#[derive(Component, Debug, Clone, Copy)]
#[component(storage = "SparseSet")]
pub struct ReproductionCooldown {
    pub remaining_ticks: u32,
}

type ReproductionSpawnQuery<'w, 's> = Query<
    'w,
    's,
    (
        Entity,
        &'static Transform,
        &'static SpatialVolume,
        &'static AllometricRadiusAnchor,
        &'static InferenceProfile,
        &'static CapabilitySet,
    ),
    (With<BaseEnergy>, Without<ReproductionCooldown>),
>;

/// Tick de cooldown: una sola transformación sobre el marcador transitorio.
pub fn reproduction_cooldown_tick_system(
    mut commands: Commands,
    mut cooldown_query: Query<(Entity, &mut ReproductionCooldown)>,
) {
    for (entity, mut cd) in &mut cooldown_query {
        if cd.remaining_ticks > 0 {
            cd.remaining_ticks -= 1;
        } else {
            commands.entity(entity).remove::<ReproductionCooldown>();
        }
    }
}

#[inline]
fn dispersal_offset_xy(entity: Entity) -> Vec2 {
    let idx = entity.index() as f32;
    let angle = idx * constants::SEED_DISPERSAL_ANGLE_STEP;
    Vec2::new(angle.cos(), angle.sin()) * constants::SEED_DISPERSAL_DISTANCE
}

#[inline]
fn sim_plane_xz(transform: &Transform) -> Vec2 {
    Vec2::new(transform.translation.x, transform.translation.z)
}

/// Deriva drift principal y deltas acoplados para `InferenceProfile` (determinista por `Entity`).
#[inline]
fn profile_mutation_drifts(entity: Entity) -> (f32, f32, f32) {
    let idx = entity.index() as f32;
    let base =
        (idx * constants::REPRODUCTION_MUTATION_INDEX_SCALE).sin() * constants::MUTATION_MAX_DRIFT;
    (
        base,
        -base * constants::REPRODUCTION_MUTATION_BRANCHING_SCALE,
        base * constants::REPRODUCTION_MUTATION_RESILIENCE_SCALE,
    )
}

/// Candidatos elegibles por biomasa y capacidad; orden estable por `Entity` antes del cupo global.
pub fn reproduction_spawn_system(
    mut commands: Commands,
    mut energy_ops: EnergyOps,
    grid: Option<Res<EnergyFieldGrid>>,
    query: ReproductionSpawnQuery,
) {
    let mut eligible: Vec<Entity> = Vec::new();
    for (entity, _, volume, anchor, profile, caps) in &query {
        if !caps.can_branch() {
            continue;
        }
        if !equations::can_reproduce(
            volume.radius,
            anchor.base_radius,
            profile.branching_bias,
            constants::REPRODUCTION_RADIUS_FACTOR,
        ) {
            continue;
        }
        eligible.push(entity);
    }
    eligible.sort_unstable();

    let mut spawned = 0usize;
    for entity in eligible {
        if spawned >= constants::MAX_REPRODUCTIONS_PER_FRAME {
            break;
        }
        let Some(qe) = energy_ops.qe(entity) else {
            continue;
        };
        let seed_want = qe * constants::SEED_ENERGY_FRACTION;
        if seed_want <= 0.0 {
            continue;
        }

        let Ok((_, transform, _, _anchor, profile, caps)) = query.get(entity) else {
            continue;
        };

        let drained = energy_ops.drain(entity, seed_want, DeathCause::Dissipation);
        if drained <= 0.0 {
            continue;
        }

        let seed_pos = sim_plane_xz(transform) + dispersal_offset_xy(entity);
        let (d_growth, d_branch, d_res) = profile_mutation_drifts(entity);
        let child_profile = InferenceProfile::new(
            equations::mutate_bias(profile.growth_bias, d_growth, constants::MUTATION_MAX_DRIFT),
            profile.mobility_bias,
            equations::mutate_bias(
                profile.branching_bias,
                d_branch,
                constants::MUTATION_MAX_DRIFT,
            ),
            equations::mutate_bias(profile.resilience, d_res, constants::MUTATION_MAX_DRIFT),
        );

        let child = EntityBuilder::new()
            .named(constants::SEED_ENTITY_NAME)
            .energy(drained)
            .volume(constants::SEED_INITIAL_RADIUS)
            .wave(ElementId::from_name(constants::FLORA_ELEMENT_SYMBOL))
            .flow(Vec2::ZERO, constants::SEED_FLOW_DISSIPATION)
            .matter(
                MatterState::Solid,
                constants::SEED_MATTER_BOND_EB,
                constants::SEED_MATTER_THERMAL_CONDUCTIVITY,
            )
            .nutrient(
                constants::SEED_NUTRIENT_CARBON,
                constants::SEED_NUTRIENT_NITROGEN,
                constants::SEED_NUTRIENT_PHOSPHORUS,
                constants::SEED_NUTRIENT_WATER,
            )
            .growth_budget(
                constants::SEED_GROWTH_BIOMASS,
                constants::SEED_GROWTH_LIMITER,
                constants::SEED_GROWTH_EFFICIENCY,
            )
            .at(seed_pos)
            .spawn(&mut commands);

        commands.entity(child).insert((child_profile, *caps));
        if let Some(grid) = grid.as_deref()
            && let Some((cx, cy)) = grid.cell_coords(seed_pos)
        {
            commands.entity(child).insert(Materialized {
                cell_x: cx as i32,
                cell_y: cy as i32,
                archetype: WorldArchetype::TerraSolid,
            });
        }

        commands.entity(entity).insert(ReproductionCooldown {
            remaining_ticks: constants::REPRODUCTION_COOLDOWN_TICKS,
        });

        spawned += 1;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::blueprint::constants::QE_MIN_EXISTENCE;
    use crate::events::DeathEvent;
    use crate::layers::BaseEnergy;
    use crate::simulation::test_support::{count_base_energy, drain_death_events};
    use bevy::prelude::Name;

    /// qe parental: tras donar `SEED_ENERGY_FRACTION`, el remanente queda bajo `QE_MIN_EXISTENCE`.
    const QE_PARENT_DEATH_AFTER_SEED: f32 = 0.012;

    fn test_app() -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_event::<DeathEvent>();
        app.add_systems(
            Update,
            (reproduction_cooldown_tick_system, reproduction_spawn_system).chain(),
        );
        app
    }

    fn spawn_flora_parent(
        app: &mut App,
        volume_r: f32,
        qe: f32,
        profile: InferenceProfile,
        caps: CapabilitySet,
    ) {
        app.world_mut().spawn((
            Transform::from_xyz(0.0, 0.0, 0.0),
            SpatialVolume::new(volume_r),
            AllometricRadiusAnchor::new(0.08),
            profile,
            caps,
            BaseEnergy::new(qe),
        ));
    }

    #[test]
    fn small_plant_does_not_reproduce() {
        let mut app = test_app();
        spawn_flora_parent(
            &mut app,
            0.1,
            200.0,
            InferenceProfile::new(0.9, 0.0, 0.8, 0.5),
            CapabilitySet::new(CapabilitySet::GROW | CapabilitySet::BRANCH),
        );
        app.update();
        let world = app.world_mut();
        assert_eq!(
            count_base_energy(world),
            1,
            "Small plant should not reproduce"
        );
    }

    #[test]
    fn large_plant_produces_seed() {
        let mut app = test_app();
        let parent_profile = InferenceProfile::new(0.9, 0.35, 0.8, 0.5);
        let parent_caps = CapabilitySet::new(CapabilitySet::GROW | CapabilitySet::BRANCH);
        spawn_flora_parent(&mut app, 2.0, 200.0, parent_profile, parent_caps);
        app.update();
        let world = app.world_mut();
        assert_eq!(
            count_base_energy(world),
            2,
            "Large plant should produce one seed"
        );

        let mut child_profile: Option<InferenceProfile> = None;
        let mut child_caps: Option<CapabilitySet> = None;
        let world = app.world_mut();
        for (name, prof, caps) in world
            .query::<(&Name, &InferenceProfile, &CapabilitySet)>()
            .iter(world)
        {
            if name.as_str() == constants::SEED_ENTITY_NAME {
                child_profile = Some(*prof);
                child_caps = Some(*caps);
            }
        }
        let cp = child_profile.expect("seed InferenceProfile");
        assert_eq!(
            cp.mobility_bias, parent_profile.mobility_bias,
            "mobility_bias se hereda sin mutar (EA6)"
        );
        assert_eq!(
            child_caps.expect("seed caps").flags,
            parent_caps.flags,
            "CapabilitySet heredado del padre"
        );
    }

    #[test]
    fn cooldown_prevents_immediate_re_reproduction() {
        let mut app = test_app();
        spawn_flora_parent(
            &mut app,
            2.0,
            200.0,
            InferenceProfile::new(0.9, 0.0, 0.8, 0.5),
            CapabilitySet::new(CapabilitySet::GROW | CapabilitySet::BRANCH),
        );
        app.update();
        app.update();
        let world = app.world_mut();
        assert_eq!(
            count_base_energy(world),
            2,
            "Cooldown should prevent second reproduction"
        );
    }

    #[test]
    fn no_branch_capability_no_reproduction() {
        let mut app = test_app();
        spawn_flora_parent(
            &mut app,
            2.0,
            200.0,
            InferenceProfile::new(0.3, 0.0, 0.1, 0.95),
            CapabilitySet::new(CapabilitySet::GROW | CapabilitySet::ROOT),
        );
        app.update();
        let world = app.world_mut();
        assert_eq!(
            count_base_energy(world),
            1,
            "Without BRANCH capability, no reproduction"
        );
    }

    #[test]
    fn parent_energy_drained_by_seed_fraction() {
        let mut app = test_app();
        spawn_flora_parent(
            &mut app,
            2.0,
            100.0,
            InferenceProfile::new(0.9, 0.0, 0.8, 0.5),
            CapabilitySet::new(CapabilitySet::GROW | CapabilitySet::BRANCH),
        );
        app.update();
        let world = app.world_mut();
        let mut q = world.query::<&BaseEnergy>();
        let mut qs: Vec<f32> = q.iter(world).map(|e| e.qe()).collect();
        qs.sort_by(|a, b| a.total_cmp(b));
        assert_eq!(qs.len(), 2);
        let sum: f32 = qs.iter().sum();
        assert!((sum - 100.0).abs() < 0.01, "Total qe should be conserved");
        let frac = constants::SEED_ENERGY_FRACTION;
        assert!((qs[0] - 100.0 * frac).abs() < 0.01, "Seed fraction");
        assert!(
            (qs[1] - 100.0 * (1.0 - frac)).abs() < 0.01,
            "Parent remainder"
        );
    }

    #[test]
    fn zero_qe_parent_does_not_spawn_seed() {
        let mut app = test_app();
        spawn_flora_parent(
            &mut app,
            2.0,
            0.0,
            InferenceProfile::new(0.9, 0.0, 0.8, 0.5),
            CapabilitySet::new(CapabilitySet::GROW | CapabilitySet::BRANCH),
        );
        app.update();
        let world = app.world_mut();
        assert_eq!(count_base_energy(world), 1);
    }

    #[test]
    fn max_reproductions_per_frame_global_cap() {
        let mut app = test_app();
        let profile = InferenceProfile::new(0.9, 0.0, 0.9, 0.5);
        let caps = CapabilitySet::new(CapabilitySet::GROW | CapabilitySet::BRANCH);
        spawn_flora_parent(&mut app, 2.0, 500.0, profile, caps);
        spawn_flora_parent(&mut app, 2.0, 500.0, profile, caps);
        spawn_flora_parent(&mut app, 2.0, 500.0, profile, caps);
        app.update();
        let world = app.world_mut();
        let n = count_base_energy(world);
        assert_eq!(n, 5, "3 parents + 2 seeds (cupo global = 2)");
    }

    #[test]
    fn drain_below_existence_threshold_emits_death_event() {
        let mut app = test_app();
        spawn_flora_parent(
            &mut app,
            2.0,
            QE_PARENT_DEATH_AFTER_SEED,
            InferenceProfile::new(0.9, 0.0, 0.9, 0.5),
            CapabilitySet::new(CapabilitySet::GROW | CapabilitySet::BRANCH),
        );
        app.update();
        let deaths = drain_death_events(&mut app);
        assert_eq!(
            deaths.len(),
            1,
            "EnergyOps debe emitir DeathEvent al cruzar umbral L0 (QE_MIN_EXISTENCE={})",
            QE_MIN_EXISTENCE
        );
    }

    #[test]
    fn mutation_drifts_stay_within_unit_interval() {
        let extreme_profiles = [
            InferenceProfile::new(0.99, 0.0, 0.01, 0.99),
            InferenceProfile::new(0.01, 0.0, 0.99, 0.01),
            InferenceProfile::new(1.0, 0.0, 0.0, 1.0),
            InferenceProfile::new(0.0, 0.0, 1.0, 0.0),
        ];
        let caps = CapabilitySet::new(CapabilitySet::GROW | CapabilitySet::BRANCH);
        for parent_profile in extreme_profiles {
            let mut app = test_app();
            spawn_flora_parent(&mut app, 2.0, 200.0, parent_profile, caps);
            app.update();
            let world = app.world_mut();
            for (name, prof) in world
                .query::<(&Name, &InferenceProfile)>()
                .iter(world)
            {
                if name.as_str() == constants::SEED_ENTITY_NAME {
                    assert!(
                        prof.growth_bias >= 0.0 && prof.growth_bias <= 1.0,
                        "growth_bias {:.4} out of [0,1] for parent {:?}",
                        prof.growth_bias, parent_profile
                    );
                    assert!(
                        prof.mobility_bias >= 0.0 && prof.mobility_bias <= 1.0,
                        "mobility_bias {:.4} out of [0,1] for parent {:?}",
                        prof.mobility_bias, parent_profile
                    );
                    assert!(
                        prof.branching_bias >= 0.0 && prof.branching_bias <= 1.0,
                        "branching_bias {:.4} out of [0,1] for parent {:?}",
                        prof.branching_bias, parent_profile
                    );
                    assert!(
                        prof.resilience >= 0.0 && prof.resilience <= 1.0,
                        "resilience {:.4} out of [0,1] for parent {:?}",
                        prof.resilience, parent_profile
                    );
                }
            }
        }
    }

    #[test]
    fn seed_placement_offset_nonzero() {
        let mut app = test_app();
        let parent_pos = Vec3::new(10.0, 0.0, 15.0);
        let profile = InferenceProfile::new(0.9, 0.0, 0.8, 0.5);
        let caps = CapabilitySet::new(CapabilitySet::GROW | CapabilitySet::BRANCH);
        app.world_mut().spawn((
            Transform::from_translation(parent_pos),
            SpatialVolume::new(2.0),
            AllometricRadiusAnchor::new(0.08),
            profile,
            caps,
            BaseEnergy::new(200.0),
        ));
        app.update();
        let world = app.world_mut();
        let mut seed_found = false;
        for (name, transform) in world
            .query::<(&Name, &Transform)>()
            .iter(world)
        {
            if name.as_str() == constants::SEED_ENTITY_NAME {
                seed_found = true;
                let dist = transform.translation.distance(parent_pos);
                assert!(
                    dist > 0.01,
                    "Seed should be offset from parent; dist={dist}"
                );
            }
        }
        assert!(seed_found, "Expected at least one seed to be spawned");
    }
}
