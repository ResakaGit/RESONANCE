use bevy::prelude::*;

use crate::blueprint::equations;
use crate::blueprint::equations::derived_thresholds as dt;
use crate::events::DeathEvent;
use crate::layers::{BaseEnergy, NutrientProfile};
use crate::worldgen::{Materialized, NUTRIENT_WRITE_EPS, NutrientCell, NutrientFieldGrid};

pub const MAX_NUTRIENT_UPTAKE_PER_FRAME: u32 = 128;

#[derive(Resource, Debug, Default)]
pub struct NutrientUptakeCursor {
    missing_offset: usize,
    existing_offset: usize,
}

type UptakeCoords = (Entity, u32, u32);

#[inline]
fn profile_differs(a: NutrientProfile, b: NutrientProfile) -> bool {
    (a.carbon_norm - b.carbon_norm).abs() > NUTRIENT_WRITE_EPS
        || (a.nitrogen_norm - b.nitrogen_norm).abs() > NUTRIENT_WRITE_EPS
        || (a.phosphorus_norm - b.phosphorus_norm).abs() > NUTRIENT_WRITE_EPS
        || (a.water_norm - b.water_norm).abs() > NUTRIENT_WRITE_EPS
}

#[inline]
fn cell_to_profile(cell: &NutrientCell) -> NutrientProfile {
    NutrientProfile::new(
        cell.carbon_norm,
        cell.nitrogen_norm,
        cell.phosphorus_norm,
        cell.water_norm,
    )
}

#[inline]
fn valid_materialized_coords(mat: &Materialized) -> Option<(u32, u32)> {
    if mat.cell_x < 0 || mat.cell_y < 0 {
        return None;
    }
    Some((mat.cell_x as u32, mat.cell_y as u32))
}

#[inline]
fn apply_profile_delta(cell: &mut NutrientCell, profile: NutrientProfile, factor: f32, sign: f32) {
    let signed = factor * sign;
    cell.carbon_norm = (cell.carbon_norm + profile.carbon_norm * signed).clamp(0.0, 1.0);
    cell.nitrogen_norm = (cell.nitrogen_norm + profile.nitrogen_norm * signed).clamp(0.0, 1.0);
    cell.phosphorus_norm =
        (cell.phosphorus_norm + profile.phosphorus_norm * signed).clamp(0.0, 1.0);
    cell.water_norm = (cell.water_norm + profile.water_norm * signed).clamp(0.0, 1.0);
}

fn collect_sorted_uptake_coords<'w, 's, I>(iter: I) -> Vec<UptakeCoords>
where
    I: Iterator<Item = (Entity, &'w Materialized)> + 's,
{
    let mut out: Vec<UptakeCoords> = iter
        .filter_map(|(entity, mat)| {
            let (x, y) = valid_materialized_coords(mat)?;
            Some((entity, x, y))
        })
        .collect();
    out.sort_by_key(|(entity, _, _)| entity.to_bits());
    out
}

pub fn nutrient_uptake_system(
    mut commands: Commands,
    nutrient_grid: Option<Res<NutrientFieldGrid>>,
    mut cursor: ResMut<NutrientUptakeCursor>,
    missing_profile: Query<(Entity, &Materialized), (With<BaseEnergy>, Without<NutrientProfile>)>,
    existing_profile_candidates: Query<
        (Entity, &Materialized),
        (
            With<BaseEnergy>,
            Or<(Changed<BaseEnergy>, Changed<Materialized>)>,
        ),
    >,
    mut existing_profile_mut: Query<&mut NutrientProfile>,
) {
    let Some(nutrient_grid) = nutrient_grid else {
        return;
    };
    let mut processed = 0u32;
    let missing = collect_sorted_uptake_coords(missing_profile.iter());

    let missing_len = missing.len();
    for i in 0..missing_len {
        if processed >= MAX_NUTRIENT_UPTAKE_PER_FRAME {
            break;
        }
        let idx = (cursor.missing_offset + i) % missing_len;
        let (entity, x, y) = missing[idx];
        let Some(cell) = nutrient_grid.cell_xy(x, y) else {
            continue;
        };
        commands.entity(entity).insert(cell_to_profile(cell));
        processed += 1;
    }
    if missing_len > 0 {
        cursor.missing_offset = (cursor.missing_offset + processed as usize) % missing_len;
    }

    if processed >= MAX_NUTRIENT_UPTAKE_PER_FRAME {
        return;
    }

    let existing = collect_sorted_uptake_coords(existing_profile_candidates.iter());

    let existing_len = existing.len();
    let mut processed_existing = 0u32;
    for i in 0..existing_len {
        if processed >= MAX_NUTRIENT_UPTAKE_PER_FRAME {
            break;
        }
        let idx = (cursor.existing_offset + i) % existing_len;
        let (entity, x, y) = existing[idx];
        let Some(cell) = nutrient_grid.cell_xy(x, y) else {
            continue;
        };
        let next = cell_to_profile(cell);
        if let Ok(mut profile) = existing_profile_mut.get_mut(entity) {
            if profile_differs(*profile, next) {
                *profile = next;
            }
        }
        processed += 1;
        processed_existing += 1;
    }
    if existing_len > 0 {
        cursor.existing_offset =
            (cursor.existing_offset + processed_existing as usize) % existing_len;
    }
}

pub fn nutrient_regen_system(nutrient_grid: Option<ResMut<NutrientFieldGrid>>) {
    let Some(mut nutrient_grid) = nutrient_grid else {
        return;
    };
    for cell in nutrient_grid.iter_cells_mut() {
        cell.regenerate(dt::nutrient_regen_per_tick());
    }
}

pub fn nutrient_depletion_system(
    nutrient_grid: Option<ResMut<NutrientFieldGrid>>,
    mut entities: Query<(&Materialized, &NutrientProfile, &BaseEnergy), Changed<NutrientProfile>>,
) {
    let Some(mut nutrient_grid) = nutrient_grid else {
        return;
    };
    for (mat, profile, energy) in &mut entities {
        let scale = equations::nutrient_depletion_scale(energy.qe(), dt::nutrient_depletion_rate());
        let Some((x, y)) = valid_materialized_coords(mat) else {
            continue;
        };
        let Some(cell) = nutrient_grid.cell_xy_mut(x, y) else {
            continue;
        };
        apply_profile_delta(cell, *profile, scale, -1.0);
    }
}

pub fn nutrient_return_on_death_system(
    mut deaths: EventReader<DeathEvent>,
    nutrient_grid: Option<ResMut<NutrientFieldGrid>>,
    entities: Query<(&Materialized, &NutrientProfile, &BaseEnergy)>,
) {
    let Some(mut nutrient_grid) = nutrient_grid else {
        return;
    };
    for death in deaths.read() {
        let Ok((mat, profile, energy)) = entities.get(death.entity) else {
            continue;
        };
        let returned = equations::nutrient_return_scale(energy.qe(), dt::nutrient_return_rate());
        let Some((x, y)) = valid_materialized_coords(mat) else {
            continue;
        };
        let Some(cell) = nutrient_grid.cell_xy_mut(x, y) else {
            continue;
        };
        apply_profile_delta(cell, *profile, returned, 1.0);
    }
}

#[cfg(test)]
mod tests {
    use super::{
        MAX_NUTRIENT_UPTAKE_PER_FRAME, nutrient_return_on_death_system, nutrient_uptake_system,
    };
    use crate::events::{DeathCause, DeathEvent};
    use crate::layers::BaseEnergy;
    use crate::simulation::nutrient_uptake::{nutrient_depletion_system, nutrient_regen_system};
    use crate::worldgen::{Materialized, NutrientCell, NutrientFieldGrid};
    use bevy::math::Vec2;
    use bevy::prelude::*;

    #[test]
    fn materialized_entity_receives_profile_from_cell() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.insert_resource(NutrientFieldGrid::new(2, 2, 1.0, Vec2::ZERO));
        app.init_resource::<super::NutrientUptakeCursor>();
        app.add_systems(Update, nutrient_uptake_system);
        {
            let mut grid = app.world_mut().resource_mut::<NutrientFieldGrid>();
            if let Some(cell) = grid.cell_xy_mut(1, 1) {
                *cell = NutrientCell::new(0.9, 0.8, 0.7, 0.6);
            }
        }
        let entity = app
            .world_mut()
            .spawn((
                Materialized {
                    cell_x: 1,
                    cell_y: 1,
                    archetype: crate::worldgen::WorldArchetype::TerraSolid,
                },
                BaseEnergy::new(10.0),
            ))
            .id();
        app.update();
        let profile = app
            .world()
            .entity(entity)
            .get::<crate::layers::NutrientProfile>()
            .copied()
            .expect("profile inserted");
        assert!((profile.carbon_norm - 0.9).abs() < 0.01);
        assert!((profile.water_norm - 0.6).abs() < 0.01);
    }

    #[test]
    fn depletion_then_death_return_modifies_grid() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_event::<DeathEvent>();
        app.insert_resource(NutrientFieldGrid::new(1, 1, 1.0, Vec2::ZERO));
        app.add_systems(
            Update,
            (
                nutrient_regen_system,
                nutrient_depletion_system,
                nutrient_return_on_death_system,
            )
                .chain(),
        );

        {
            let mut grid = app.world_mut().resource_mut::<NutrientFieldGrid>();
            if let Some(cell) = grid.cell_xy_mut(0, 0) {
                *cell = NutrientCell::new(1.0, 1.0, 1.0, 1.0);
            }
        }

        let e = app
            .world_mut()
            .spawn((
                Materialized {
                    cell_x: 0,
                    cell_y: 0,
                    archetype: crate::worldgen::WorldArchetype::TerraSolid,
                },
                BaseEnergy::new(100.0),
                crate::layers::NutrientProfile::new(1.0, 1.0, 1.0, 1.0),
            ))
            .id();

        app.update();
        let after_depletion = app
            .world()
            .resource::<NutrientFieldGrid>()
            .cell_xy(0, 0)
            .copied()
            .expect("cell");
        assert!(after_depletion.nitrogen_norm < 1.0);

        app.world_mut()
            .resource_mut::<Events<DeathEvent>>()
            .send(DeathEvent {
                entity: e,
                cause: DeathCause::Dissipation,
            });
        app.update();
        let after_return = app
            .world()
            .resource::<NutrientFieldGrid>()
            .cell_xy(0, 0)
            .copied()
            .expect("cell");
        assert!(after_return.nitrogen_norm > after_depletion.nitrogen_norm);
    }

    #[test]
    fn uptake_ignores_negative_materialized_coords() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.insert_resource(NutrientFieldGrid::new(2, 2, 1.0, Vec2::ZERO));
        app.init_resource::<super::NutrientUptakeCursor>();
        app.add_systems(Update, nutrient_uptake_system);
        let entity = app
            .world_mut()
            .spawn((
                Materialized {
                    cell_x: -1,
                    cell_y: -1,
                    archetype: crate::worldgen::WorldArchetype::TerraSolid,
                },
                BaseEnergy::new(10.0),
            ))
            .id();
        app.update();
        assert!(
            app.world()
                .entity(entity)
                .get::<crate::layers::NutrientProfile>()
                .is_none()
        );
    }

    #[test]
    fn uptake_respects_budget_per_frame() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.insert_resource(NutrientFieldGrid::new(1, 1, 1.0, Vec2::ZERO));
        app.init_resource::<super::NutrientUptakeCursor>();
        app.add_systems(Update, nutrient_uptake_system);
        for _ in 0..(MAX_NUTRIENT_UPTAKE_PER_FRAME + 10) {
            app.world_mut().spawn((
                Materialized {
                    cell_x: 0,
                    cell_y: 0,
                    archetype: crate::worldgen::WorldArchetype::TerraSolid,
                },
                BaseEnergy::new(10.0),
            ));
        }
        app.update();
        let updated = app
            .world_mut()
            .query::<&crate::layers::NutrientProfile>()
            .iter(app.world())
            .count() as u32;
        assert_eq!(updated, MAX_NUTRIENT_UPTAKE_PER_FRAME);
    }
}
