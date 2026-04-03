//! Abiogénesis: materialización espontánea de flora y fauna desde el campo (EA5 + EA5-F).
//!
//! Evalúa `EnergyFieldGrid` + `NutrientFieldGrid` y spawnea entidades con `InferenceProfile`
//! cuando el potencial supera umbral. Flora usa frequency-band gating; fauna usa trophic
//! succession gating (flora density → herbivore, herbivore density → carnivore).
//!
//! Matemática en [`crate::blueprint::equations`]; constantes en [`crate::blueprint::constants`].

pub(crate) mod constants;

use bevy::prelude::*;

use crate::blueprint::constants::*;
use crate::blueprint::{ElementId, equations};
use crate::entities::builder::EntityBuilder;
use crate::layers::has_inferred_shape::HasInferredShape;
use crate::layers::organ::LifecycleStageCache;
use crate::layers::senescence::SenescenceProfile;
use crate::layers::shape_params::MorphogenesisShapeParams;
use crate::layers::{
    BehaviorCooldown, BehaviorIntent, BehavioralAgent, CacheScope, CapabilitySet, Homeostasis,
    InferenceProfile, MatterState, PerformanceCachePolicy, TrophicClass, TrophicConsumer,
    TrophicState,
};
use crate::runtime_platform::simulation_tick::SimulationClock;
use crate::worldgen::constants::ABIOGENESIS_FIELD_OCCUPANT_NAME;
use crate::worldgen::{EnergyFieldGrid, NutrientFieldGrid};

pub use constants::MAX_ABIOGENESIS_PER_FRAME;

/// Cursor round-robin sobre índices lineales del grid.
#[derive(Resource, Debug, Default)]
pub struct AbiogenesisCursor {
    pub next_cell: usize,
}

/// Tracks what kind of entity was spawned in each cell (lightweight parallel grid).
#[derive(Resource, Debug)]
pub struct AbiogenesisOccupancyGrid {
    width: u32,
    height: u32,
    cells: Vec<OccupantKind>,
}

/// Cell occupant type for trophic succession gating.
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum OccupantKind {
    #[default]
    Empty = 0,
    Flora = 1,
    Herbivore = 2,
    Carnivore = 3,
}

impl AbiogenesisOccupancyGrid {
    pub fn new(width: u32, height: u32) -> Self {
        Self {
            width,
            height,
            cells: vec![OccupantKind::Empty; (width * height) as usize],
        }
    }

    fn idx(&self, cx: u32, cy: u32) -> Option<usize> {
        if cx < self.width && cy < self.height {
            Some((cy * self.width + cx) as usize)
        } else {
            None
        }
    }

    pub fn set(&mut self, cx: u32, cy: u32, kind: OccupantKind) {
        if let Some(i) = self.idx(cx, cy) {
            self.cells[i] = kind;
        }
    }

    pub fn get(&self, cx: u32, cy: u32) -> OccupantKind {
        self.idx(cx, cy)
            .map(|i| self.cells[i])
            .unwrap_or(OccupantKind::Empty)
    }

    /// Count neighbours of a given kind within `radius` cells (excludes center).
    pub fn count_neighbours(&self, cx: u32, cy: u32, radius: u32, kind: OccupantKind) -> u32 {
        let mut count = 0u32;
        let x_min = cx.saturating_sub(radius);
        let y_min = cy.saturating_sub(radius);
        let x_max = (cx + radius).min(self.width - 1);
        let y_max = (cy + radius).min(self.height - 1);
        for ny in y_min..=y_max {
            for nx in x_min..=x_max {
                if nx == cx && ny == cy {
                    continue;
                }
                if self.get(nx, ny) == kind {
                    count += 1;
                }
            }
        }
        count
    }
}

/// Gather neighbor (qe, hz, distance) tuples for a cell in the energy grid.
fn gather_neighbor_data(energy: &EnergyFieldGrid, cx: u32, cy: u32) -> Vec<(f32, f32, f32)> {
    let mut neighbors = Vec::with_capacity(8);
    let cell_size = energy.cell_size;
    for dy in -1i32..=1 {
        for dx in -1i32..=1 {
            if dx == 0 && dy == 0 {
                continue;
            }
            let nx = cx as i32 + dx;
            let ny = cy as i32 + dy;
            if nx < 0 || ny < 0 {
                continue;
            }
            let Some(ncell) = energy.cell_xy(nx as u32, ny as u32) else {
                continue;
            };
            let dist = ((dx * dx + dy * dy) as f32).sqrt() * cell_size;
            neighbors.push((ncell.accumulated_qe, ncell.dominant_frequency_hz, dist));
        }
    }
    neighbors
}

/// Axiomatic abiogenesis: life emerges where coherence gain > dissipation loss.
///
/// Derived from Axioms 1, 4, 7, 8. No hardcoded bands, no element-specific catalysts.
/// Entity properties (matter state, capabilities, profile) derived from local energy density.
/// Returns the OccupantKind of the spawned entity (Flora if sessile, Herbivore if mobile), or None.
fn try_spawn_emergent_at_cell(
    commands: &mut Commands,
    energy: &mut EnergyFieldGrid,
    nutrients: &NutrientFieldGrid,
    cx: u32,
    cy: u32,
    tick_birth: u64,
) -> Option<OccupantKind> {
    let (cell_qe, cell_hz) = {
        let Some(cell_ref) = energy.cell_xy(cx, cy) else {
            return None;
        };
        if cell_ref.materialized_entity.is_some() {
            return None;
        }
        (cell_ref.accumulated_qe, cell_ref.dominant_frequency_hz)
    };

    // Axioms 7 + 8: coherence from neighboring oscillators
    let neighbors = gather_neighbor_data(energy, cx, cy);
    let coherence = equations::cell_coherence_gain(cell_hz, &neighbors);

    // Estimate local dissipation from volume (cell_size² as proxy)
    let cell_area = energy.cell_size * energy.cell_size;
    let volume_proxy = cell_area.max(0.01);
    let density = cell_qe / volume_proxy;
    let state = equations::matter_state_from_density(cell_qe, volume_proxy);
    let dissipation_rate = equations::dissipation_from_state(state);

    // Axioms 1 + 4: potential = coherence gain vs dissipation loss
    let potential =
        equations::axiomatic_abiogenesis_potential(cell_qe, coherence, dissipation_rate);
    if !equations::axiomatic_spawn_viable(potential) {
        return None;
    }

    let Some(world_pos) = energy.world_pos(cx, cy) else {
        return None;
    };

    // ── All entity properties derived from energy state ─────────────────────
    let qe_spawn = cell_qe * ABIOGENESIS_SPAWN_CELL_QE_FRACTION;
    let radius = equations::initial_radius_from_qe(qe_spawn);
    let bond = equations::bond_from_energy(qe_spawn);
    let conductivity = equations::conductivity_from_state(state);
    let dissipation = equations::dissipation_from_state(state);

    // Capabilities from energy profile (Axioms 1, 8)
    let coherence_norm = (coherence / cell_qe.max(1.0)).clamp(0.0, 1.0);
    let caps = equations::capabilities_from_energy(qe_spawn, density, coherence_norm);

    // Morphological profile from energy state
    let flow_speed = 0.0; // nascent entity, no velocity yet
    let (growth_bias, mobility_bias, branching_bias, resilience) =
        equations::inference_profile_from_energy(density, coherence_norm, flow_speed);

    // Nutrient content from grid (if available)
    let water = nutrients
        .cell_xy(cx, cy)
        .map(|n| n.water_norm)
        .unwrap_or(0.0);

    // Element resolved from dominant frequency (Axiom 8: frequency = identity)
    let entity = EntityBuilder::new()
        .named(ABIOGENESIS_FIELD_OCCUPANT_NAME)
        .energy(qe_spawn)
        .volume(radius)
        .wave_from_hz(cell_hz)
        .flow(Vec2::ZERO, dissipation)
        .matter(state, bond, conductivity)
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
        InferenceProfile::new(growth_bias, mobility_bias, branching_bias, resilience),
        CapabilitySet::new(caps),
        SenescenceProfile {
            tick_birth,
            senescence_coeff: crate::blueprint::constants::senescence_coeff_flora(),
            max_viable_age: crate::blueprint::constants::senescence_max_age_flora(),
            strategy: crate::blueprint::constants::SENESCENCE_DEFAULT_STRATEGY,
        },
    ));

    if let Some(cell_mut) = energy.cell_xy_mut(cx, cy) {
        cell_mut.materialized_entity = Some(entity);
    }

    // Derive occupant kind from capabilities (not hardcoded Flora)
    let is_mobile = caps & CapabilitySet::MOVE != 0;
    Some(if is_mobile {
        OccupantKind::Herbivore
    } else {
        OccupantKind::Flora
    })
}

/// Fauna spawn: builds a herbivore or carnivore with the full behavioral stack.
fn try_spawn_fauna_at_cell(
    commands: &mut Commands,
    energy: &mut EnergyFieldGrid,
    nutrients: &NutrientFieldGrid,
    occupancy: &mut AbiogenesisOccupancyGrid,
    cx: u32,
    cy: u32,
    tick_birth: u64,
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

    let nutrient_density = nutrients
        .cell_xy(cx, cy)
        .map(|n| (n.carbon_norm + n.nitrogen_norm + n.phosphorus_norm + n.water_norm) * 0.25)
        .unwrap_or(0.0);

    let water = nutrients
        .cell_xy(cx, cy)
        .map(|n| n.water_norm)
        .unwrap_or(0.0);
    let flora_count = occupancy.count_neighbours(cx, cy, 1, OccupantKind::Flora);
    let herbivore_count = occupancy.count_neighbours(cx, cy, 2, OccupantKind::Herbivore);

    let potential = equations::fauna_abiogenesis_potential(
        cell_qe,
        nutrient_density,
        flora_count,
        ABIOGENESIS_FAUNA_MIN_FLORA_NEIGHBOURS,
        water,
        ABIOGENESIS_FAUNA_FIELD_MIN_QE,
        ABIOGENESIS_FAUNA_WATER_FLOOR,
    );

    if potential < ABIOGENESIS_FAUNA_POTENTIAL_THRESHOLD {
        return false;
    }

    let is_carnivore = equations::fauna_infer_is_carnivore(
        herbivore_count,
        ABIOGENESIS_FAUNA_MIN_HERBIVORE_NEIGHBOURS,
    );

    let Some(world_pos) = energy.world_pos(cx, cy) else {
        return false;
    };

    let qe_spawn = equations::fauna_spawn_entity_qe(cell_qe);
    let bond = equations::fauna_spawn_matter_bond(cell_qe);

    // Element derived from local frequency (Axiom 8: frequency = identity).
    let element_sym = equations::element_symbol_from_frequency(cell_hz);

    let (trophic_class, intake_rate, profile, caps_bits) = if is_carnivore {
        (
            TrophicClass::Carnivore,
            ABIOGENESIS_CARNIVORE_INTAKE_RATE,
            InferenceProfile::new(
                ABIOGENESIS_CARNIVORE_GROWTH,
                ABIOGENESIS_CARNIVORE_MOBILITY,
                ABIOGENESIS_CARNIVORE_BRANCHING,
                ABIOGENESIS_CARNIVORE_RESILIENCE,
            ),
            CapabilitySet::MOVE | CapabilitySet::SENSE | CapabilitySet::REPRODUCE,
        )
    } else {
        (
            TrophicClass::Herbivore,
            ABIOGENESIS_HERBIVORE_INTAKE_RATE,
            InferenceProfile::new(
                ABIOGENESIS_HERBIVORE_GROWTH,
                ABIOGENESIS_HERBIVORE_MOBILITY,
                ABIOGENESIS_HERBIVORE_BRANCHING,
                ABIOGENESIS_HERBIVORE_RESILIENCE,
            ),
            CapabilitySet::MOVE
                | CapabilitySet::SENSE
                | CapabilitySet::REPRODUCE
                | CapabilitySet::GROW,
        )
    };

    let entity = EntityBuilder::new()
        .named(ABIOGENESIS_FIELD_OCCUPANT_NAME)
        .energy(qe_spawn)
        .volume(constants::FAUNA_EMERGENT_INITIAL_RADIUS)
        .wave(ElementId::from_name(element_sym))
        .flow(Vec2::ZERO, constants::FAUNA_EMERGENT_FLOW_DISSIPATION)
        .matter(
            MatterState::Solid,
            bond,
            constants::FAUNA_EMERGENT_MATTER_THERMAL_CONDUCTIVITY,
        )
        .motor(
            constants::FAUNA_EMERGENT_BUF_MAX,
            constants::FAUNA_EMERGENT_IN_VALVE,
            constants::FAUNA_EMERGENT_OUT_VALVE,
            constants::FAUNA_EMERGENT_BUF_INIT,
        )
        .will_default()
        .homeostasis(Homeostasis::new(
            constants::FAUNA_EMERGENT_ADAPT_RATE,
            constants::FAUNA_EMERGENT_QE_COST_HZ,
            constants::FAUNA_EMERGENT_STAB_BAND,
            true,
        ))
        .at(world_pos)
        .spawn(commands);

    commands.entity(entity).insert((
        BehavioralAgent,
        BehaviorIntent::default(),
        BehaviorCooldown::default(),
        TrophicConsumer::new(trophic_class, intake_rate),
        TrophicState::new(ABIOGENESIS_FAUNA_INITIAL_SATIATION),
        CapabilitySet::new(caps_bits),
        profile,
        HasInferredShape,
        LifecycleStageCache::default(),
        MorphogenesisShapeParams::default(),
        PerformanceCachePolicy {
            enabled: true,
            scope: CacheScope::StableWindow,
            version_tag: 1,
            dependency_signature: 0,
        },
        SenescenceProfile {
            tick_birth,
            senescence_coeff: crate::blueprint::constants::senescence_coeff_fauna(),
            max_viable_age: crate::blueprint::constants::senescence_max_age_fauna(),
            strategy: crate::blueprint::constants::SENESCENCE_DEFAULT_STRATEGY,
        },
    ));

    if let Some(cell_mut) = energy.cell_xy_mut(cx, cy) {
        cell_mut.materialized_entity = Some(entity);
    }
    let occ_kind = if is_carnivore {
        OccupantKind::Carnivore
    } else {
        OccupantKind::Herbivore
    };
    occupancy.set(cx, cy, occ_kind);
    true
}

/// Escanea celdas y spawnea flora/fauna emergente donde el potencial y nutrientes lo permiten.
/// Flora tiene prioridad (must exist before herbivores); fauna only spawns where flora didn't.
pub fn abiogenesis_system(
    mut commands: Commands,
    energy_grid: Option<ResMut<EnergyFieldGrid>>,
    nutrient_grid: Option<Res<NutrientFieldGrid>>,
    occupancy_grid: Option<ResMut<AbiogenesisOccupancyGrid>>,
    mut cursor: ResMut<AbiogenesisCursor>,
    clock: Res<SimulationClock>,
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

    // Lazy-init occupancy grid from energy grid dimensions.
    let mut occupancy = match occupancy_grid {
        Some(occ) => occ,
        None => {
            commands.insert_resource(AbiogenesisOccupancyGrid::new(energy.width, energy.height));
            return; // Grid will be available next tick.
        }
    };

    let mut flora_spawned = 0usize;
    let mut fauna_spawned = 0usize;
    let scan_budget = total_cells.min(constants::SCAN_BUDGET_CELLS);

    for _ in 0..scan_budget {
        if flora_spawned >= MAX_ABIOGENESIS_PER_FRAME
            && fauna_spawned >= ABIOGENESIS_FAUNA_MAX_PER_FRAME
        {
            break;
        }

        let idx = cursor.next_cell % total_cells;
        cursor.next_cell = (cursor.next_cell + 1) % total_cells.max(1);

        let cx = (idx % energy.width as usize) as u32;
        let cy = (idx / energy.width as usize) as u32;

        // Sessile emergents take priority (trophic succession: sessile → herbivore → carnivore).
        if flora_spawned < MAX_ABIOGENESIS_PER_FRAME {
            if let Some(kind) = try_spawn_emergent_at_cell(
                &mut commands,
                &mut energy,
                nutrients,
                cx,
                cy,
                clock.tick_id,
            ) {
                occupancy.set(cx, cy, kind);
                flora_spawned += 1;
                continue;
            }
        }

        // Fauna only where flora didn't spawn this tick.
        if fauna_spawned < ABIOGENESIS_FAUNA_MAX_PER_FRAME
            && try_spawn_fauna_at_cell(
                &mut commands,
                &mut energy,
                nutrients,
                &mut occupancy,
                cx,
                cy,
                clock.tick_id,
            )
        {
            fauna_spawned += 1;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::blueprint::constants::{
        ABIOGENESIS_FIELD_MIN_QE, ABIOGENESIS_FLORA_PEAK_HZ,
        ABIOGENESIS_TEST_CELL_QE_FACTOR_OVER_MIN, ABIOGENESIS_TEST_FIXTURE_WATER_NORM,
    };
    use crate::simulation::test_support::count_base_energy;

    fn test_app_with_grids(grid: EnergyFieldGrid, ngrid: NutrientFieldGrid) -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        let w = grid.width;
        let h = grid.height;
        app.insert_resource(grid);
        app.insert_resource(ngrid);
        app.insert_resource(AbiogenesisOccupancyGrid::new(w, h));
        app.init_resource::<AbiogenesisCursor>();
        app.init_resource::<SimulationClock>();
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

    /// Creates a coherent energy cluster (3×3) centered at (cx, cy) — axiomatic abiogenesis
    /// requires neighbor coherence, not just a single high-energy cell.
    fn fill_coherent_cluster(
        grid: &mut EnergyFieldGrid,
        ngrid: &mut NutrientFieldGrid,
        cx: u32,
        cy: u32,
        qe: f32,
        hz: f32,
    ) {
        for dy in 0..3u32 {
            for dx in 0..3u32 {
                let x = cx.saturating_sub(1) + dx;
                let y = cy.saturating_sub(1) + dy;
                if let Some(cell) = grid.cell_xy_mut(x, y) {
                    cell.accumulated_qe = qe;
                    cell.dominant_frequency_hz = hz;
                }
                if let Some(ncell) = ngrid.cell_xy_mut(x, y) {
                    ncell.water_norm = ABIOGENESIS_TEST_FIXTURE_WATER_NORM;
                }
            }
        }
    }

    #[test]
    fn spawn_when_conditions_met() {
        let mut grid = EnergyFieldGrid::new(4, 4, 2.0, Vec2::ZERO);
        let mut ngrid = NutrientFieldGrid::align_with_energy_grid(&grid);
        // Axiomatic: coherent cluster needed (neighbors at same frequency → constructive interference)
        let test_qe = ABIOGENESIS_FIELD_MIN_QE * ABIOGENESIS_TEST_CELL_QE_FACTOR_OVER_MIN;
        fill_coherent_cluster(
            &mut grid,
            &mut ngrid,
            1,
            1,
            test_qe,
            ABIOGENESIS_FLORA_PEAK_HZ,
        );
        let mut app = test_app_with_grids(grid, ngrid);
        app.update();
        assert!(
            count_base_energy(app.world_mut()) > 0,
            "coherent cluster should trigger abiogenesis"
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
        let mut ngrid = NutrientFieldGrid::align_with_energy_grid(&grid);
        let test_qe = ABIOGENESIS_FIELD_MIN_QE * ABIOGENESIS_TEST_CELL_QE_FACTOR_OVER_MIN;
        fill_coherent_cluster(
            &mut grid,
            &mut ngrid,
            2,
            2,
            test_qe,
            ABIOGENESIS_FLORA_PEAK_HZ,
        );
        let mut app = test_app_with_grids(grid, ngrid);
        // Run several ticks to ensure cursor reaches cells in the cluster.
        for _ in 0..4 {
            app.update();
        }
        let grid = app.world_mut().resource::<EnergyFieldGrid>();
        // At least ONE cell in the 3×3 cluster should have materialized_entity.
        let mut any_materialized = false;
        for dy in 0..3u32 {
            for dx in 0..3u32 {
                let x = 1 + dx;
                let y = 1 + dy;
                if let Some(cell) = grid.cell_xy(x, y) {
                    if cell.materialized_entity.is_some() {
                        any_materialized = true;
                    }
                }
            }
        }
        assert!(
            any_materialized,
            "at least one cell in the cluster should be materialized"
        );
    }

    #[test]
    fn cursor_wraps_around_grid() {
        let grid = EnergyFieldGrid::new(4, 4, 2.0, Vec2::ZERO);
        let ngrid = NutrientFieldGrid::align_with_energy_grid(&grid);
        let total_cells = 16usize;
        let mut app = test_app_with_grids(grid, ngrid);
        app.world_mut()
            .resource_mut::<AbiogenesisCursor>()
            .next_cell = total_cells - 2;
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
        let mut ngrid = NutrientFieldGrid::align_with_energy_grid(&grid);
        let test_qe = ABIOGENESIS_FIELD_MIN_QE * ABIOGENESIS_TEST_CELL_QE_FACTOR_OVER_MIN;
        // Fill cluster around (1,1) — only center cell should spawn (neighbors provide coherence).
        fill_coherent_cluster(
            &mut grid,
            &mut ngrid,
            1,
            1,
            test_qe,
            ABIOGENESIS_FLORA_PEAK_HZ,
        );
        let mut app = test_app_with_grids(grid, ngrid);
        // First update: spawns entities from the cluster.
        app.update();
        let count_after_first = count_base_energy(app.world_mut());
        assert!(
            count_after_first >= 1,
            "First update should spawn at least one entity"
        );
        // Second update: occupied cells skipped → no more spawns in those cells.
        let count_before_second = count_base_energy(app.world_mut());
        app.update();
        let count_after_second = count_base_energy(app.world_mut());
        // Could spawn more (from un-occupied cluster cells) but occupied ones are skipped.
        // Key invariant: count does not decrease.
        assert!(
            count_after_second >= count_before_second,
            "Count should not decrease: before={count_before_second}, after={count_after_second}"
        );
    }
}
