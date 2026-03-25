//! Abiogénesis: materialización espontánea de flora desde el campo (EA5).
//!
//! Evalúa `EnergyFieldGrid` + `NutrientFieldGrid` y spawnea entidades con `InferenceProfile`
//! cuando el potencial ([`equations::abiogenesis_potential`]) supera umbral. Matemática en
//! [`crate::blueprint::equations`]; constantes compartidas en [`crate::blueprint::constants`].

mod constants;

use bevy::prelude::*;

use crate::blueprint::constants::{
    ABIOGENESIS_FIELD_MIN_QE, ABIOGENESIS_FLORA_BAND_HZ_HIGH, ABIOGENESIS_FLORA_BAND_HZ_LOW,
    ABIOGENESIS_FLORA_ELEMENT_SYMBOL, ABIOGENESIS_FLORA_PEAK_HZ,
    ABIOGENESIS_POTENTIAL_SCORE_THRESHOLD,
};
use crate::blueprint::{equations, ElementId};
use crate::entities::builder::EntityBuilder;
use crate::layers::{CapabilitySet, InferenceProfile, MatterState};
use crate::worldgen::constants::ABIOGENESIS_FIELD_OCCUPANT_NAME;
use crate::worldgen::{EnergyFieldGrid, NutrientFieldGrid};

pub use constants::MAX_ABIOGENESIS_PER_FRAME;

/// Cursor round-robin sobre índices lineales del grid.
#[derive(Resource, Debug, Default)]
pub struct AbiogenesisCursor {
    pub next_cell: usize,
}

/// Una iteración: evalúa celda y spawnea si corresponde. Mantiene el sistema como orquestador.
fn try_spawn_emergent_at_cell(
    commands: &mut Commands,
    energy: &mut EnergyFieldGrid,
    nutrients: &NutrientFieldGrid,
    cx: u32,
    cy: u32,
) -> bool {
    let (cell_qe, cell_hz) = {
        let Some(cell_ref) = energy.cell_xy(cx, cy) else {
            return false;
        };
        if cell_ref.materialized_entity.is_some() {
            return false;
        }
        (cell_ref.accumulated_qe, cell_ref.dominant_frequency_hz)
    };

    let water = nutrients
        .cell_xy(cx, cy)
        .map(|n| n.water_norm)
        .unwrap_or(0.0);

    let potential = equations::abiogenesis_potential(
        cell_qe,
        cell_hz,
        ABIOGENESIS_FLORA_BAND_HZ_LOW,
        ABIOGENESIS_FLORA_BAND_HZ_HIGH,
        ABIOGENESIS_FLORA_PEAK_HZ,
        water,
        ABIOGENESIS_FIELD_MIN_QE,
    );

    if potential < ABIOGENESIS_POTENTIAL_SCORE_THRESHOLD {
        return false;
    }

    let water_sat = water.clamp(0.0, 1.0);
    let bond_local = equations::abiogenesis_bond_heuristic_from_cell_qe(cell_qe);
    let (growth_bias, branching_bias, resilience) =
        equations::abiogenesis_profile_from_conditions(bond_local, water_sat);

    let Some(world_pos) = energy.world_pos(cx, cy) else {
        return false;
    };

    let qe_spawn = equations::abiogenesis_spawn_entity_qe(cell_qe);
    let bond_matter = equations::abiogenesis_spawn_matter_bond(cell_qe);

    let entity = EntityBuilder::new()
        .named(ABIOGENESIS_FIELD_OCCUPANT_NAME)
        .energy(qe_spawn)
        .volume(constants::EMERGENT_INITIAL_RADIUS)
        .wave(ElementId::from_name(ABIOGENESIS_FLORA_ELEMENT_SYMBOL))
        .flow(Vec2::ZERO, constants::EMERGENT_FLOW_DISSIPATION)
        .matter(
            MatterState::Solid,
            bond_matter,
            constants::EMERGENT_MATTER_THERMAL_CONDUCTIVITY,
        )
        .nutrient(
            water * constants::EMERGENT_NUTRIENT_CARBON_SCALE,
            water * constants::EMERGENT_NUTRIENT_NITROGEN_SCALE,
            water * constants::EMERGENT_NUTRIENT_PHOSPHORUS_SCALE,
            water * constants::EMERGENT_NUTRIENT_WATER_SCALE,
        )
        .growth_budget(
            constants::EMERGENT_GROWTH_BIOMASS,
            constants::EMERGENT_GROWTH_LIMITER,
            constants::EMERGENT_GROWTH_EFFICIENCY,
        )
        .at(world_pos)
        .spawn(commands);

    commands.entity(entity).insert((
        InferenceProfile::new(growth_bias, 0.0, branching_bias, resilience),
        CapabilitySet::new(CapabilitySet::GROW | CapabilitySet::BRANCH | CapabilitySet::ROOT),
    ));

    if let Some(cell_mut) = energy.cell_xy_mut(cx, cy) {
        cell_mut.materialized_entity = Some(entity);
    }

    true
}

/// Escanea celdas y spawnea flora emergente donde el potencial y nutrientes lo permiten.
pub fn abiogenesis_system(
    mut commands: Commands,
    energy_grid: Option<ResMut<EnergyFieldGrid>>,
    nutrient_grid: Option<Res<NutrientFieldGrid>>,
    mut cursor: ResMut<AbiogenesisCursor>,
) {
    let Some(mut energy) = energy_grid else {
        return;
    };
    let Some(nutrients) = nutrient_grid.as_ref() else {
        return;
    };

    let total_cells = (energy.width * energy.height) as usize;
    if total_cells == 0 {
        return;
    }

    let mut spawned = 0usize;
    let scan_budget = total_cells.min(constants::SCAN_BUDGET_CELLS);

    for _ in 0..scan_budget {
        if spawned >= MAX_ABIOGENESIS_PER_FRAME {
            break;
        }

        let idx = cursor.next_cell % total_cells;
        cursor.next_cell = (cursor.next_cell + 1) % total_cells.max(1);

        let cx = (idx % energy.width as usize) as u32;
        let cy = (idx / energy.width as usize) as u32;

        if try_spawn_emergent_at_cell(&mut commands, &mut energy, nutrients, cx, cy) {
            spawned += 1;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::blueprint::constants::{
        ABIOGENESIS_FIELD_MIN_QE, ABIOGENESIS_FLORA_BAND_HZ_HIGH, ABIOGENESIS_FLORA_BAND_HZ_LOW,
        ABIOGENESIS_FLORA_PEAK_HZ, ABIOGENESIS_POTENTIAL_SCORE_THRESHOLD,
        ABIOGENESIS_TEST_CELL_QE_FACTOR_OVER_MIN, ABIOGENESIS_TEST_FIXTURE_WATER_NORM,
    };
    use crate::blueprint::equations;
    use crate::simulation::test_support::count_base_energy;

    fn test_app_with_grids(grid: EnergyFieldGrid, ngrid: NutrientFieldGrid) -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.insert_resource(grid);
        app.insert_resource(ngrid);
        app.init_resource::<AbiogenesisCursor>();
        app.add_systems(Update, abiogenesis_system);
        app
    }

    #[test]
    fn no_spawn_when_grid_empty() {
        let grid = EnergyFieldGrid::new(4, 4, 2.0, Vec2::ZERO);
        let ngrid = NutrientFieldGrid::align_with_energy_grid(&grid);
        let mut app = test_app_with_grids(grid, ngrid);
        app.update();
        assert_eq!(count_base_energy(app.world_mut()), 0);
    }

    #[test]
    fn spawn_when_conditions_met() {
        let mut grid = EnergyFieldGrid::new(4, 4, 2.0, Vec2::ZERO);
        if let Some(cell) = grid.cell_xy_mut(1, 1) {
            cell.accumulated_qe =
                ABIOGENESIS_FIELD_MIN_QE * ABIOGENESIS_TEST_CELL_QE_FACTOR_OVER_MIN;
            cell.dominant_frequency_hz = ABIOGENESIS_FLORA_PEAK_HZ;
        }
        let mut ngrid = NutrientFieldGrid::align_with_energy_grid(&grid);
        if let Some(ncell) = ngrid.cell_xy_mut(1, 1) {
            ncell.water_norm = ABIOGENESIS_TEST_FIXTURE_WATER_NORM;
        }
        let mut app = test_app_with_grids(grid, ngrid);
        app.update();
        assert!(count_base_energy(app.world_mut()) > 0);
        let pot = equations::abiogenesis_potential(
            ABIOGENESIS_FIELD_MIN_QE * ABIOGENESIS_TEST_CELL_QE_FACTOR_OVER_MIN,
            ABIOGENESIS_FLORA_PEAK_HZ,
            ABIOGENESIS_FLORA_BAND_HZ_LOW,
            ABIOGENESIS_FLORA_BAND_HZ_HIGH,
            ABIOGENESIS_FLORA_PEAK_HZ,
            ABIOGENESIS_TEST_FIXTURE_WATER_NORM,
            ABIOGENESIS_FIELD_MIN_QE,
        );
        assert!(
            pot >= ABIOGENESIS_POTENTIAL_SCORE_THRESHOLD,
            "fixture debe quedar por encima del umbral de spawn (pot={pot})"
        );
    }

    #[test]
    fn respects_max_spawns_per_frame() {
        let mut grid = EnergyFieldGrid::new(8, 8, 2.0, Vec2::ZERO);
        for y in 0..8 {
            for x in 0..8 {
                if let Some(cell) = grid.cell_xy_mut(x, y) {
                    cell.accumulated_qe =
                        ABIOGENESIS_FIELD_MIN_QE * ABIOGENESIS_TEST_CELL_QE_FACTOR_OVER_MIN;
                    cell.dominant_frequency_hz = ABIOGENESIS_FLORA_PEAK_HZ;
                }
            }
        }
        let mut ngrid = NutrientFieldGrid::align_with_energy_grid(&grid);
        for y in 0..8 {
            for x in 0..8 {
                if let Some(ncell) = ngrid.cell_xy_mut(x, y) {
                    ncell.water_norm = ABIOGENESIS_TEST_FIXTURE_WATER_NORM;
                }
            }
        }
        let mut app = test_app_with_grids(grid, ngrid);
        app.update();
        let count = count_base_energy(app.world_mut());
        assert!(
            count <= MAX_ABIOGENESIS_PER_FRAME,
            "Should not exceed budget: got {count}"
        );
    }

    #[test]
    fn sets_materialized_entity_on_cell() {
        let mut grid = EnergyFieldGrid::new(4, 4, 2.0, Vec2::ZERO);
        if let Some(cell) = grid.cell_xy_mut(2, 2) {
            cell.accumulated_qe =
                ABIOGENESIS_FIELD_MIN_QE * ABIOGENESIS_TEST_CELL_QE_FACTOR_OVER_MIN;
            cell.dominant_frequency_hz = ABIOGENESIS_FLORA_PEAK_HZ;
        }
        let mut ngrid = NutrientFieldGrid::align_with_energy_grid(&grid);
        if let Some(ncell) = ngrid.cell_xy_mut(2, 2) {
            ncell.water_norm = ABIOGENESIS_TEST_FIXTURE_WATER_NORM;
        }
        let mut app = test_app_with_grids(grid, ngrid);
        app.update();
        let grid = app.world_mut().resource::<EnergyFieldGrid>();
        let cell = grid.cell_xy(2, 2).expect("cell");
        assert!(cell.materialized_entity.is_some());
    }

    #[test]
    fn cursor_wraps_around_grid() {
        let grid = EnergyFieldGrid::new(4, 4, 2.0, Vec2::ZERO);
        let ngrid = NutrientFieldGrid::align_with_energy_grid(&grid);
        let total_cells = 16usize;
        let mut app = test_app_with_grids(grid, ngrid);
        app.world_mut().resource_mut::<AbiogenesisCursor>().next_cell = total_cells - 2;
        app.update();
        let cursor_val = app.world_mut().resource::<AbiogenesisCursor>().next_cell;
        assert!(
            cursor_val < total_cells,
            "Cursor should wrap within grid bounds: got {cursor_val}, total_cells={total_cells}"
        );
        // Run several more updates to confirm cursor keeps cycling without going out of bounds.
        for _ in 0..5 {
            app.update();
            let c = app.world_mut().resource::<AbiogenesisCursor>().next_cell;
            assert!(
                c < total_cells,
                "Cursor out of bounds after multiple updates: got {c}"
            );
        }
    }

    #[test]
    fn occupied_cell_skipped_no_double_spawn() {
        let mut grid = EnergyFieldGrid::new(4, 4, 2.0, Vec2::ZERO);
        if let Some(cell) = grid.cell_xy_mut(0, 0) {
            cell.accumulated_qe =
                ABIOGENESIS_FIELD_MIN_QE * ABIOGENESIS_TEST_CELL_QE_FACTOR_OVER_MIN;
            cell.dominant_frequency_hz = ABIOGENESIS_FLORA_PEAK_HZ;
        }
        let mut ngrid = NutrientFieldGrid::align_with_energy_grid(&grid);
        if let Some(ncell) = ngrid.cell_xy_mut(0, 0) {
            ncell.water_norm = ABIOGENESIS_TEST_FIXTURE_WATER_NORM;
        }
        let mut app = test_app_with_grids(grid, ngrid);
        // First update: spawns entity at cell (0,0).
        app.update();
        let count_after_first = count_base_energy(app.world_mut());
        assert_eq!(count_after_first, 1, "First update should spawn exactly one entity");
        // Second update: cursor wraps back; cell (0,0) is now occupied → skipped.
        app.update();
        let count_after_second = count_base_energy(app.world_mut());
        assert_eq!(
            count_after_second, 1,
            "Occupied cell should be skipped; no double spawn"
        );
    }
}
