//! D9: Ecological Dynamics — census, carrying capacity, succession.
//!
//! Phase: [`Phase::MetabolicLayer`], after D6 social, before faction_identity.
//! Order: census → carrying_capacity → succession.

use bevy::prelude::*;

use crate::blueprint::constants::{
    CENSUS_INTERVAL, SUCCESSION_TICK_INTERVAL, SUCCESSION_TICK_STEP,
};
use crate::blueprint::equations;
use crate::layers::BaseEnergy;
use crate::layers::inference::TrophicClass;
use crate::layers::trophic::TrophicConsumer;
use crate::runtime_platform::simulation_tick::SimulationClock;
use crate::worldgen::{EnergyFieldGrid, Materialized, NutrientFieldGrid};

const TROPHIC_CLASS_COUNT: usize = 5;
const TROPHIC_CLASSES: [TrophicClass; TROPHIC_CLASS_COUNT] = [
    TrophicClass::PrimaryProducer,
    TrophicClass::Herbivore,
    TrophicClass::Omnivore,
    TrophicClass::Carnivore,
    TrophicClass::Detritivore,
];

/// Global population census aggregated per grid cell and trophic class.
#[derive(Resource, Default)]
pub struct PopulationCensus {
    pub total_entities: u32,
    pub by_trophic_class: [u32; TROPHIC_CLASS_COUNT],
    pub by_cell: Vec<u32>,
    pub generation: u32,
}

/// Per-cell reproduction pressure and carrying capacity.
#[derive(Resource, Default)]
pub struct ReproductionPressureField {
    pub pressure: Vec<f32>,
    pub carrying_capacity: Vec<u32>,
}

/// Macro-ecological succession state.
#[derive(Resource)]
pub struct SuccessionState {
    pub stage: equations::SuccessionStage,
    pub time_since_disturbance: u32,
    pub dominant_trophic: TrophicClass,
}

impl Default for SuccessionState {
    fn default() -> Self {
        Self {
            stage: equations::SuccessionStage::Pioneer,
            time_since_disturbance: 0,
            dominant_trophic: TrophicClass::PrimaryProducer,
        }
    }
}

pub fn every_census_interval(clock: Res<SimulationClock>) -> bool {
    clock.tick_id % CENSUS_INTERVAL == 0
}

pub fn every_succession_interval(clock: Res<SimulationClock>) -> bool {
    clock.tick_id % SUCCESSION_TICK_INTERVAL == 0
}

/// S1: Count entities by trophic class and grid cell.
pub fn census_system(
    mut census: ResMut<PopulationCensus>,
    grid: Res<EnergyFieldGrid>,
    query: Query<(&Materialized, Option<&TrophicConsumer>), With<BaseEnergy>>,
) {
    let cell_count = grid.width as usize * grid.height as usize;
    census.total_entities = 0;
    census.by_trophic_class = [0; TROPHIC_CLASS_COUNT];
    census.by_cell.resize(cell_count, 0);
    census.by_cell.fill(0);
    for (mat, trophic) in &query {
        if mat.cell_x < 0 || mat.cell_y < 0 {
            continue;
        }
        let (cx, cy) = (mat.cell_x as u32, mat.cell_y as u32);
        if cx >= grid.width || cy >= grid.height {
            continue;
        }
        census.total_entities += 1;
        let class_idx = trophic
            .map(|tc| tc.class as usize)
            .unwrap_or(TrophicClass::PrimaryProducer as usize);
        if class_idx < TROPHIC_CLASS_COUNT {
            census.by_trophic_class[class_idx] += 1;
        }
        let cell_idx = cy as usize * grid.width as usize + cx as usize;
        if cell_idx < census.by_cell.len() {
            census.by_cell[cell_idx] += 1;
        }
    }
    census.generation += 1;
}

/// S2: Compute per-cell carrying capacity and reproduction pressure.
pub fn carrying_capacity_system(
    census: Res<PopulationCensus>,
    grid: Res<EnergyFieldGrid>,
    nutrient_grid: Res<NutrientFieldGrid>,
    mut field: ResMut<ReproductionPressureField>,
) {
    let cell_count = grid.width as usize * grid.height as usize;
    field.pressure.resize(cell_count, 0.0);
    field.carrying_capacity.resize(cell_count, 0);
    for y in 0..grid.height {
        for x in 0..grid.width {
            let idx = y as usize * grid.width as usize + x as usize;
            let cell_qe = grid.cell_xy(x, y).map(|c| c.accumulated_qe).unwrap_or(0.0);
            let nutrient_total = nutrient_grid
                .cell_xy(x, y)
                .map(|c| c.carbon_norm + c.nitrogen_norm + c.phosphorus_norm + c.water_norm)
                .unwrap_or(0.0);
            let k = equations::carrying_capacity(cell_qe, nutrient_total, grid.cell_size);
            let local_pop = census.by_cell.get(idx).copied().unwrap_or(0);
            field.carrying_capacity[idx] = k;
            field.pressure[idx] = equations::reproduction_pressure(local_pop, k);
        }
    }
}

/// S3: Advance succession state based on census demographics and elapsed time.
pub fn succession_system(census: Res<PopulationCensus>, mut state: ResMut<SuccessionState>) {
    state.time_since_disturbance = state
        .time_since_disturbance
        .saturating_add(SUCCESSION_TICK_STEP);
    let dominant = census
        .by_trophic_class
        .iter()
        .enumerate()
        .max_by_key(|(_, count)| *count)
        .map(|(idx, _)| TROPHIC_CLASSES[idx])
        .unwrap_or(TrophicClass::PrimaryProducer);
    state.dominant_trophic = dominant;
    let new_stage = equations::succession_stage(state.time_since_disturbance, dominant);
    if state.stage != new_stage {
        state.stage = new_stage;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::worldgen::archetypes::WorldArchetype;
    use bevy::app::App;

    fn test_app() -> App {
        let mut app = App::new();
        app.init_resource::<PopulationCensus>();
        app.init_resource::<ReproductionPressureField>();
        app.init_resource::<SuccessionState>();
        app.init_resource::<SimulationClock>();
        app
    }

    fn insert_grids(app: &mut App, width: u32, height: u32, cell_size: f32, cell_qe: f32) {
        let mut grid = EnergyFieldGrid::new(width, height, cell_size, bevy::math::Vec2::ZERO);
        for y in 0..height {
            for x in 0..width {
                if let Some(cell) = grid.cell_xy_mut(x, y) {
                    cell.accumulated_qe = cell_qe;
                }
            }
        }
        app.insert_resource(grid);
        let mut ngrid = NutrientFieldGrid::new(width, height, cell_size, bevy::math::Vec2::ZERO);
        for y in 0..height {
            for x in 0..width {
                if let Some(cell) = ngrid.cell_xy_mut(x, y) {
                    cell.carbon_norm = 0.5;
                    cell.nitrogen_norm = 0.5;
                    cell.phosphorus_norm = 0.5;
                    cell.water_norm = 0.5;
                }
            }
        }
        app.insert_resource(ngrid);
    }

    fn mat(cx: i32, cy: i32) -> Materialized {
        Materialized {
            cell_x: cx,
            cell_y: cy,
            archetype: WorldArchetype::Void,
        }
    }

    #[test]
    fn census_counts_all_entities_by_trophic_class() {
        let mut app = test_app();
        insert_grids(&mut app, 4, 4, 32.0, 100.0);
        app.world_mut().spawn((BaseEnergy::new(10.0), mat(0, 0)));
        app.world_mut().spawn((BaseEnergy::new(10.0), mat(1, 0)));
        app.world_mut().spawn((
            BaseEnergy::new(10.0),
            TrophicConsumer::new(TrophicClass::Herbivore, 1.0),
            mat(0, 0),
        ));
        app.add_systems(bevy::app::Update, census_system);
        app.update();
        let census = app.world().resource::<PopulationCensus>();
        assert_eq!(census.total_entities, 3);
        assert_eq!(
            census.by_trophic_class[TrophicClass::PrimaryProducer as usize],
            2
        );
        assert_eq!(census.by_trophic_class[TrophicClass::Herbivore as usize], 1);
    }

    #[test]
    fn census_maps_entities_to_correct_grid_cell() {
        let mut app = test_app();
        insert_grids(&mut app, 4, 4, 32.0, 100.0);
        app.world_mut().spawn((BaseEnergy::new(10.0), mat(2, 3)));
        app.add_systems(bevy::app::Update, census_system);
        app.update();
        let census = app.world().resource::<PopulationCensus>();
        let idx = 3 * 4 + 2;
        assert_eq!(census.by_cell[idx], 1);
        let other_sum: u32 = census
            .by_cell
            .iter()
            .enumerate()
            .filter(|(i, _)| *i != idx)
            .map(|(_, v)| v)
            .sum();
        assert_eq!(other_sum, 0);
    }

    #[test]
    fn carrying_capacity_computes_per_cell_pressure() {
        let mut app = test_app();
        insert_grids(&mut app, 2, 2, 1.0, 100.0);
        app.world_mut().spawn((BaseEnergy::new(10.0), mat(0, 0)));
        app.add_systems(
            bevy::app::Update,
            (census_system, carrying_capacity_system).chain(),
        );
        app.update();
        let field = app.world().resource::<ReproductionPressureField>();
        assert!(
            field.pressure[0] > 0.0,
            "cell with entity should have pressure > 0"
        );
        assert!(
            field.pressure[0] < 1.0,
            "cell with entity should have pressure < 1"
        );
        assert!(
            (field.pressure[1] - 1.0).abs() < f32::EPSILON || field.carrying_capacity[1] == 0,
            "empty cell with capacity should have full pressure",
        );
        assert!(field.carrying_capacity[0] > 0);
    }

    #[test]
    fn succession_advances_over_time() {
        let mut app = test_app();
        insert_grids(&mut app, 2, 2, 1.0, 10.0);
        app.world_mut().spawn((BaseEnergy::new(10.0), mat(0, 0)));
        app.add_systems(
            bevy::app::Update,
            (census_system, succession_system).chain(),
        );
        {
            let state = app.world().resource::<SuccessionState>();
            assert_eq!(state.stage, equations::SuccessionStage::Pioneer);
        }
        for _ in 0..6 {
            app.update();
        }
        let state = app.world().resource::<SuccessionState>();
        assert!(
            state.stage >= equations::SuccessionStage::Early,
            "expected at least Early, got {:?} at t={}",
            state.stage,
            state.time_since_disturbance,
        );
    }
}
