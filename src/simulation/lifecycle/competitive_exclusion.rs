//! Exclusión competitiva por densidad en celda del campo (EA7).
//! Usa la misma celda que nutrientes / `growth_budget`: [`Materialized`] + `EnergyFieldGrid`.
//! Drena vía [`EnergyOps::drain`](crate::layers::energy::EnergyOps::drain) para contrato L0 y `DeathEvent`.

use bevy::prelude::*;

use crate::blueprint::equations;
use crate::events::DeathCause;
use crate::layers::{BaseEnergy, EnergyOps, InferenceProfile, NutrientProfile};
use crate::worldgen::{
    EnergyFieldGrid, Materialized, COMPETITION_BASE_DRAIN_PER_EXTRA_COMPETITOR_QE,
};

/// Penaliza energía de **celdas del campo** (`Materialized` sin metabolismo de flora) que comparten
/// índice de grid. Los mobs con [`NutrientProfile`] pueden llevar `Materialized` solo como ancla de
/// inferencia visual — no deben contar como segundo “ocupante” ni recibir drain (EA7 ≠ proxy GF1).
/// Fase: `Phase::ChemicalLayer`.
pub fn competitive_exclusion_system(
    energy_grid: Option<Res<EnergyFieldGrid>>,
    mut energy_ops: EnergyOps,
    mut scratch: Local<Vec<u32>>,
    query: Query<
        (Entity, &Materialized, Option<&InferenceProfile>),
        (With<BaseEnergy>, Without<NutrientProfile>),
    >,
) {
    let Some(grid) = energy_grid else {
        return;
    };

    let total_cells = (grid.width * grid.height) as usize;
    if scratch.len() != total_cells {
        scratch.resize(total_cells, 0);
    } else {
        scratch.fill(0);
    }

    for (_, mat, _) in &query {
        if let Some(idx) = grid.linear_index_for_materialized(mat) {
            scratch[idx] += 1;
        }
    }

    for (entity, mat, profile_opt) in &query {
        let Some(idx) = grid.linear_index_for_materialized(mat) else {
            continue;
        };

        let competitors = scratch[idx];
        let resilience = InferenceProfile::resilience_effective(profile_opt);
        let drain = equations::competition_energy_drain(
            competitors,
            COMPETITION_BASE_DRAIN_PER_EXTRA_COMPETITOR_QE,
            resilience,
        );

        if drain > 0.0 {
            energy_ops.drain(entity, drain, DeathCause::Dissipation);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::events::{DeathCause, DeathEvent};
    use crate::layers::BaseEnergy;
    use crate::simulation::test_support::drain_death_events;
    use crate::worldgen::WorldArchetype;
    use bevy::math::Vec2;
    use bevy::prelude::{App, MinimalPlugins};

    fn mat(x: i32, y: i32) -> Materialized {
        Materialized {
            cell_x: x,
            cell_y: y,
            archetype: WorldArchetype::TerraSolid,
        }
    }

    fn test_app_with_grid() -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_event::<DeathEvent>();
        let grid = EnergyFieldGrid::new(4, 4, 2.0, Vec2::new(0.0, 0.0));
        app.insert_resource(grid);
        app.add_systems(Update, competitive_exclusion_system);
        app
    }

    #[test]
    fn single_entity_no_drain() {
        let mut app = test_app_with_grid();
        app.world_mut().spawn((
            mat(0, 0),
            BaseEnergy::new(100.0),
            InferenceProfile::new(0.5, 0.0, 0.5, 0.5),
        ));
        app.update();
        let w = app.world_mut();
        let qe = w.query::<&BaseEnergy>().single(w).qe();
        assert_eq!(qe, 100.0, "Single entity should not lose energy");
    }

    #[test]
    fn two_entities_same_cell_both_drained() {
        let mut app = test_app_with_grid();
        app.world_mut().spawn((
            mat(0, 0),
            BaseEnergy::new(100.0),
            InferenceProfile::new(0.5, 0.0, 0.5, 0.5),
        ));
        app.world_mut().spawn((
            mat(0, 0),
            BaseEnergy::new(100.0),
            InferenceProfile::new(0.5, 0.0, 0.5, 0.5),
        ));
        app.update();

        let w = app.world_mut();
        for energy in w.query::<&BaseEnergy>().iter(w) {
            assert!(energy.qe() < 100.0, "Competing entities should lose energy");
        }
    }

    #[test]
    fn resilient_entity_loses_less() {
        let mut app = test_app_with_grid();
        let fragile = app
            .world_mut()
            .spawn((
                mat(0, 0),
                BaseEnergy::new(100.0),
                InferenceProfile::new(0.5, 0.0, 0.5, 0.1),
            ))
            .id();
        let tough = app
            .world_mut()
            .spawn((
                mat(0, 0),
                BaseEnergy::new(100.0),
                InferenceProfile::new(0.5, 0.0, 0.5, 0.95),
            ))
            .id();
        app.update();

        let fragile_qe = app
            .world()
            .entity(fragile)
            .get::<BaseEnergy>()
            .unwrap()
            .qe();
        let tough_qe = app.world().entity(tough).get::<BaseEnergy>().unwrap().qe();
        assert!(
            tough_qe > fragile_qe,
            "Resilient entity should lose less energy"
        );
    }

    #[test]
    fn different_cells_no_interference() {
        let mut app = test_app_with_grid();
        app.world_mut().spawn((
            mat(0, 0),
            BaseEnergy::new(100.0),
            InferenceProfile::new(0.5, 0.0, 0.5, 0.5),
        ));
        app.world_mut().spawn((
            mat(2, 2),
            BaseEnergy::new(100.0),
            InferenceProfile::new(0.5, 0.0, 0.5, 0.5),
        ));
        app.update();

        let w = app.world_mut();
        for energy in w.query::<&BaseEnergy>().iter(w) {
            assert_eq!(
                energy.qe(),
                100.0,
                "Entities in different cells should not compete"
            );
        }
    }

    #[test]
    fn without_grid_no_effect() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_event::<DeathEvent>();
        app.add_systems(Update, competitive_exclusion_system);
        app.world_mut().spawn((
            mat(0, 0),
            BaseEnergy::new(100.0),
            InferenceProfile::new(0.5, 0.0, 0.5, 0.5),
        ));
        app.update();
        let w = app.world_mut();
        assert_eq!(w.query::<&BaseEnergy>().single(w).qe(), 100.0);
    }

    #[test]
    fn without_inference_profile_uses_default_resilience_for_drain() {
        let mut app = test_app_with_grid();
        app.world_mut().spawn((mat(0, 0), BaseEnergy::new(100.0)));
        app.world_mut().spawn((mat(0, 0), BaseEnergy::new(100.0)));
        app.update();
        let w = app.world_mut();
        let with_prof = w
            .query::<(&BaseEnergy, &InferenceProfile)>()
            .iter(w)
            .count();
        assert_eq!(with_prof, 0);
        let mut sum = 0.0f32;
        for e in w.query::<&BaseEnergy>().iter(w) {
            sum += e.qe();
        }
        assert!(sum < 200.0, "default resilience 0.5 should still compete");
    }

    #[test]
    fn drain_to_zero_emits_death_via_energy_ops() {
        let mut app = test_app_with_grid();
        app.world_mut().spawn((
            mat(0, 0),
            BaseEnergy::new(0.15),
            InferenceProfile::new(0.5, 0.0, 0.5, 0.5),
        ));
        app.world_mut().spawn((
            mat(0, 0),
            BaseEnergy::new(0.15),
            InferenceProfile::new(0.5, 0.0, 0.5, 0.5),
        ));
        app.update();
        let deaths = drain_death_events(&mut app);
        assert!(
            deaths.iter().any(|d| d.cause == DeathCause::Dissipation),
            "L0 drain to zero must emit DeathEvent"
        );
    }

    #[test]
    fn out_of_grid_materialized_skipped() {
        let mut app = test_app_with_grid();
        app.world_mut().spawn((
            mat(99, 99),
            BaseEnergy::new(100.0),
            InferenceProfile::new(0.5, 0.0, 0.5, 0.5),
        ));
        app.world_mut().spawn((
            mat(0, 0),
            BaseEnergy::new(100.0),
            InferenceProfile::new(0.5, 0.0, 0.5, 0.5),
        ));
        app.update();
        let w = app.world_mut();
        let outlier = w
            .query::<(&BaseEnergy, &Materialized)>()
            .iter(w)
            .find(|(_, m)| m.cell_x == 99)
            .map(|(e, _)| e.qe())
            .expect("outlier entity");
        assert_eq!(outlier, 100.0);
    }
}
