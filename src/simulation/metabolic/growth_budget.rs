use bevy::ecs::system::SystemParam;
use bevy::prelude::*;
use std::collections::BTreeSet;

use crate::blueprint::{AlchemicalAlmanac, constants, equations};
use crate::layers::{
    BaseEnergy, GrowthBudget, IrradianceReceiver, MatterCoherence, NutrientProfile,
    OscillatorySignature,
};
use crate::worldgen::Materialized;

type GrowthCandidate = (
    Entity,
    f32,
    f32,
    f32,
    f32,
    f32,
    f32,
    f32,
    Option<IrradianceReceiver>,
    Option<GrowthBudget>,
);

type GrowthChangedQuery<'w, 's> = Query<
    'w,
    's,
    (
        Entity,
        &'static NutrientProfile,
        &'static OscillatorySignature,
        Option<&'static MatterCoherence>,
        &'static BaseEnergy,
        Option<&'static IrradianceReceiver>,
        Option<&'static GrowthBudget>,
    ),
    (
        With<Materialized>,
        Or<(
            Changed<NutrientProfile>,
            Changed<IrradianceReceiver>,
            Changed<BaseEnergy>,
            Changed<OscillatorySignature>,
            Changed<MatterCoherence>,
        )>,
    ),
>;

type GrowthRemovedLookup<'w, 's> = Query<
    'w,
    's,
    (
        &'static NutrientProfile,
        &'static OscillatorySignature,
        Option<&'static MatterCoherence>,
        &'static BaseEnergy,
        Option<&'static IrradianceReceiver>,
        Option<&'static GrowthBudget>,
    ),
    With<Materialized>,
>;

#[derive(SystemParam)]
pub struct GrowthBudgetParams<'w, 's> {
    pending_query: Query<'w, 's, Entity, With<PendingGrowthBudgetUpdate>>,
    all_query: Query<'w, 's, Entity, (With<Materialized>, With<NutrientProfile>)>,
    changed_query: GrowthChangedQuery<'w, 's>,
    removed_lookup: GrowthRemovedLookup<'w, 's>,
    removed_irradiance: RemovedComponents<'w, 's, IrradianceReceiver>,
}

#[derive(Resource, Debug, Default)]
pub struct GrowthBudgetCursor {
    offset: usize,
}

/// Marca de recomputo pendiente para no perder updates por throttling.
#[derive(Component, Clone, Copy, Debug, Default)]
#[component(storage = "SparseSet")]
pub struct PendingGrowthBudgetUpdate;

#[inline]
fn growth_budget_differs(a: GrowthBudget, b: GrowthBudget) -> bool {
    (a.biomass_available - b.biomass_available).abs() > constants::GROWTH_WRITE_EPS
        || a.limiting_factor != b.limiting_factor
        || (a.efficiency - b.efficiency).abs() > constants::GROWTH_WRITE_EPS
}

#[inline]
fn candidate_from_components(
    entity: Entity,
    nutrients: &NutrientProfile,
    signature: &OscillatorySignature,
    coherence: Option<&MatterCoherence>,
    energy: &BaseEnergy,
    irradiance: Option<&IrradianceReceiver>,
    current: Option<&GrowthBudget>,
) -> GrowthCandidate {
    let bond_energy = coherence
        .map(MatterCoherence::bond_energy_eb)
        .unwrap_or(constants::DEFAULT_BOND_ENERGY);
    (
        entity,
        nutrients.carbon_norm,
        nutrients.nitrogen_norm,
        nutrients.phosphorus_norm,
        nutrients.water_norm,
        signature.frequency_hz(),
        bond_energy,
        energy.qe(),
        irradiance.copied(),
        current.copied(),
    )
}

/// Capa 4: sintetiza nutrientes en un presupuesto de biomasa (Liebig).
pub fn growth_budget_system(
    mut commands: Commands,
    almanac: Res<AlchemicalAlmanac>,
    mut cursor: ResMut<GrowthBudgetCursor>,
    mut params: GrowthBudgetParams,
) {
    let mut dirty_entities: BTreeSet<Entity> = BTreeSet::new();
    for entity in &params.pending_query {
        dirty_entities.insert(entity);
    }
    for (entity, ..) in &params.changed_query {
        dirty_entities.insert(entity);
    }
    for entity in params.removed_irradiance.read() {
        if params.removed_lookup.get(entity).is_ok() {
            dirty_entities.insert(entity);
        }
    }
    if almanac.is_changed() {
        for entity in &params.all_query {
            dirty_entities.insert(entity);
        }
    }
    if dirty_entities.is_empty() {
        return;
    }

    let mut candidates: Vec<GrowthCandidate> = Vec::with_capacity(dirty_entities.len());
    for entity in &dirty_entities {
        let Ok((nutrients, signature, coherence, energy, irradiance, current)) =
            params.removed_lookup.get(*entity)
        else {
            commands
                .entity(*entity)
                .remove::<PendingGrowthBudgetUpdate>();
            continue;
        };
        candidates.push(candidate_from_components(
            *entity, nutrients, signature, coherence, energy, irradiance, current,
        ));
    }
    if candidates.is_empty() {
        return;
    }
    candidates.sort_by_key(|(entity, ..)| entity.to_bits());

    let total = candidates.len();
    let budget = constants::MAX_GROWTH_BUDGET_PER_FRAME.min(total as u32);
    for i in 0..budget as usize {
        let idx = (cursor.offset + i) % total;
        let (entity, c, n, p, w, freq, bond_energy, qe, irradiance, current) = candidates[idx];
        let electronegativity = almanac
            .find_stable_band(freq.max(0.0))
            .map(|def| def.electronegativity)
            .unwrap_or(0.0);
        let efficiency = equations::genetic_efficiency_for_element(bond_energy, electronegativity);
        let (liebig_biomass, limiter) = equations::liebig_growth_budget(c, n, p, w, efficiency);
        let photo_bonus = irradiance
            .map(|r| equations::photosynthetic_growth_bonus(r.photon_density, r.absorbed_fraction))
            .unwrap_or(0.0);
        let biomass = liebig_biomass + photo_bonus;

        if biomass < constants::GROWTH_BUDGET_MIN_THRESHOLD || qe <= constants::QE_MIN_EXISTENCE {
            if current.is_some() {
                commands.entity(entity).remove::<GrowthBudget>();
            }
            commands
                .entity(entity)
                .remove::<PendingGrowthBudgetUpdate>();
            continue;
        }

        let next = GrowthBudget::new(biomass, limiter, efficiency);
        if let Some(prev) = current {
            if growth_budget_differs(prev, next) {
                commands.entity(entity).insert(next);
            }
        } else {
            commands.entity(entity).insert(next);
        }
        commands
            .entity(entity)
            .remove::<PendingGrowthBudgetUpdate>();
    }

    if budget < total as u32 {
        for i in budget as usize..total {
            let idx = (cursor.offset + i) % total;
            let (entity, ..) = candidates[idx];
            commands.entity(entity).insert(PendingGrowthBudgetUpdate);
        }
    }

    cursor.offset = (cursor.offset + budget as usize) % total;
}
#[cfg(test)]
mod tests {
    use super::growth_budget_system;
    use crate::blueprint::ElementDef;
    use crate::layers::{
        BaseEnergy, GrowthBudget, IrradianceReceiver, MatterCoherence, MatterState,
        NutrientProfile, OscillatorySignature,
    };
    use crate::simulation::growth_budget::GrowthBudgetCursor;
    use crate::worldgen::{Materialized, WorldArchetype};
    use bevy::prelude::*;

    fn test_almanac() -> crate::blueprint::AlchemicalAlmanac {
        crate::blueprint::AlchemicalAlmanac::from_defs(vec![ElementDef {
            name: "Terra".to_string(),
            symbol: "Terra".to_string(),
            atomic_number: 14,
            frequency_hz: 75.0,
            freq_band: (50.0, 84.0),
            bond_energy: 3000.0,
            conductivity: 0.4,
            visibility: 0.8,
            matter_state: MatterState::Solid,
            electronegativity: 0.5,
            ionization_ev: 0.0,
            color: (0.45, 0.34, 0.20),
            is_compound: false,
            phenology: None,
            hz_identity_weight: crate::blueprint::FIELD_EAC4_HZ_IDENTITY_WEIGHT_RON_ONLY,
        }])
    }

    fn base_app() -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.insert_resource(test_almanac());
        app.init_resource::<GrowthBudgetCursor>();
        app.add_systems(Update, growth_budget_system);
        app
    }

    #[test]
    fn rich_nutrient_profile_gets_growth_budget() {
        let mut app = base_app();
        let entity = app
            .world_mut()
            .spawn((
                Materialized {
                    cell_x: 0,
                    cell_y: 0,
                    archetype: WorldArchetype::TerraSolid,
                },
                BaseEnergy::new(100.0),
                NutrientProfile::new(0.9, 0.8, 0.7, 0.95),
                OscillatorySignature::new(75.0, 0.0),
                MatterCoherence::new(MatterState::Solid, 2000.0, 0.4),
            ))
            .id();

        app.update();
        let budget = app
            .world()
            .entity(entity)
            .get::<GrowthBudget>()
            .copied()
            .expect("growth budget inserted");
        assert!(budget.biomass_available > 0.0);
    }

    #[test]
    fn zero_limiter_nutrient_removes_growth_budget() {
        let mut app = base_app();
        let entity = app
            .world_mut()
            .spawn((
                Materialized {
                    cell_x: 0,
                    cell_y: 0,
                    archetype: WorldArchetype::TerraSolid,
                },
                BaseEnergy::new(100.0),
                NutrientProfile::new(0.8, 0.0, 0.8, 0.8),
                OscillatorySignature::new(75.0, 0.0),
            ))
            .id();

        app.update();
        assert!(app.world().entity(entity).get::<GrowthBudget>().is_none());
    }

    #[test]
    fn changed_nutrient_profile_updates_growth_budget() {
        let mut app = base_app();
        let entity = app
            .world_mut()
            .spawn((
                Materialized {
                    cell_x: 0,
                    cell_y: 0,
                    archetype: WorldArchetype::TerraSolid,
                },
                BaseEnergy::new(100.0),
                NutrientProfile::new(0.9, 0.9, 0.9, 0.9),
                OscillatorySignature::new(75.0, 0.0),
            ))
            .id();
        app.update();
        let before = app
            .world()
            .entity(entity)
            .get::<GrowthBudget>()
            .expect("initial growth")
            .biomass_available;

        app.world_mut()
            .entity_mut(entity)
            .insert(NutrientProfile::new(0.1, 0.1, 0.1, 0.1));
        app.update();
        let after = app
            .world()
            .entity(entity)
            .get::<GrowthBudget>()
            .expect("updated growth")
            .biomass_available;
        assert!(after < before);
    }

    #[test]
    fn growth_budget_respects_frame_budget() {
        let mut app = base_app();
        for _ in 0..(crate::blueprint::constants::MAX_GROWTH_BUDGET_PER_FRAME + 10) {
            app.world_mut().spawn((
                Materialized {
                    cell_x: 0,
                    cell_y: 0,
                    archetype: WorldArchetype::TerraSolid,
                },
                BaseEnergy::new(100.0),
                NutrientProfile::new(0.9, 0.9, 0.9, 0.9),
                OscillatorySignature::new(75.0, 0.0),
            ));
        }
        app.update();

        let updated = app
            .world_mut()
            .query::<&GrowthBudget>()
            .iter(app.world())
            .count() as u32;
        assert_eq!(
            updated,
            crate::blueprint::constants::MAX_GROWTH_BUDGET_PER_FRAME
        );
    }

    #[test]
    fn pending_entities_eventually_processed_across_frames() {
        let mut app = base_app();
        let total = crate::blueprint::constants::MAX_GROWTH_BUDGET_PER_FRAME + 12;
        for _ in 0..total {
            app.world_mut().spawn((
                Materialized {
                    cell_x: 0,
                    cell_y: 0,
                    archetype: WorldArchetype::TerraSolid,
                },
                BaseEnergy::new(100.0),
                NutrientProfile::new(0.9, 0.9, 0.9, 0.9),
                OscillatorySignature::new(75.0, 0.0),
            ));
        }

        app.update();
        app.update();
        let updated = app
            .world_mut()
            .query::<&GrowthBudget>()
            .iter(app.world())
            .count() as u32;
        assert_eq!(updated, total);
    }

    #[test]
    fn irradiance_bonus_increases_growth_budget() {
        let mut app = base_app();
        let entity = app
            .world_mut()
            .spawn((
                Materialized {
                    cell_x: 0,
                    cell_y: 0,
                    archetype: WorldArchetype::TerraSolid,
                },
                BaseEnergy::new(100.0),
                NutrientProfile::new(0.6, 0.6, 0.6, 0.6),
                OscillatorySignature::new(75.0, 0.0),
            ))
            .id();
        app.update();
        let no_light = app
            .world()
            .entity(entity)
            .get::<GrowthBudget>()
            .expect("growth budget without light")
            .biomass_available;

        app.world_mut()
            .entity_mut(entity)
            .insert(IrradianceReceiver::new(10.0, 1.0));
        app.update();
        let with_light = app
            .world()
            .entity(entity)
            .get::<GrowthBudget>()
            .expect("growth budget with light")
            .biomass_available;
        assert!(
            with_light > no_light,
            "with={with_light} without={no_light}"
        );
    }

    #[test]
    fn growth_budget_removed_when_qe_drops_without_nutrient_changes() {
        let mut app = base_app();
        let entity = app
            .world_mut()
            .spawn((
                Materialized {
                    cell_x: 0,
                    cell_y: 0,
                    archetype: WorldArchetype::TerraSolid,
                },
                BaseEnergy::new(100.0),
                NutrientProfile::new(0.7, 0.7, 0.7, 0.7),
                OscillatorySignature::new(75.0, 0.0),
            ))
            .id();
        app.update();
        assert!(app.world().entity(entity).get::<GrowthBudget>().is_some());

        app.world_mut()
            .entity_mut(entity)
            .insert(BaseEnergy::new(0.0));
        app.update();
        assert!(app.world().entity(entity).get::<GrowthBudget>().is_none());
    }

    #[test]
    fn growth_budget_recomputes_when_irradiance_is_removed() {
        let mut app = base_app();
        let entity = app
            .world_mut()
            .spawn((
                Materialized {
                    cell_x: 0,
                    cell_y: 0,
                    archetype: WorldArchetype::TerraSolid,
                },
                BaseEnergy::new(100.0),
                NutrientProfile::new(0.6, 0.6, 0.6, 0.6),
                OscillatorySignature::new(75.0, 0.0),
                IrradianceReceiver::new(10.0, 1.0),
            ))
            .id();
        app.update();
        let with_light = app
            .world()
            .entity(entity)
            .get::<GrowthBudget>()
            .expect("with light")
            .biomass_available;

        app.world_mut()
            .entity_mut(entity)
            .remove::<IrradianceReceiver>();
        app.update();
        let without_light = app
            .world()
            .entity(entity)
            .get::<GrowthBudget>()
            .expect("without light")
            .biomass_available;
        assert!(
            without_light < with_light,
            "without={without_light} with={with_light}"
        );
    }
}
