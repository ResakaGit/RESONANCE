use bevy::prelude::*;

use super::performance::PropagationWriteBudget;
use super::propagation::{
    derive_cell_state_system, dissipate_field_system, propagate_nuclei_system,
};
use crate::blueprint::AlchemicalAlmanac;
use crate::eco::context_lookup::EcoPlayfieldMargin;
use crate::layers::{
    AmbientPressure, BaseEnergy, MatterCoherence, OscillatorySignature, SpatialVolume,
};
use crate::runtime_platform::compat_2d3d::SimWorldTransformParams;
use crate::topology::TerrainField;
use crate::worldgen::materialization_rules::materialize_cell_at_time;
use crate::worldgen::{
    ActiveMapName, EnergyFieldGrid, EnergyNucleus, MapConfig, Materialized, NucleusReservoir, NutrientFieldGrid,
    WARMUP_TICKS, active_map_slug_from_env, load_default_map_asset,
    load_map_config_from_env_result, resolve_nuclei_for_spawn, validate_map_config,
};
use std::time::Duration;

#[derive(Resource, Debug, Clone, Copy)]
pub struct WorldgenWarmupConfig {
    pub ticks: u32,
}

impl Default for WorldgenWarmupConfig {
    fn default() -> Self {
        Self {
            ticks: WARMUP_TICKS,
        }
    }
}

#[inline]
fn terrain_field_aligned_with_grid(grid: &EnergyFieldGrid, terrain: &TerrainField) -> bool {
    terrain.width == grid.width
        && terrain.height == grid.height
        && (terrain.cell_size - grid.cell_size).abs() <= f32::EPSILON
        && terrain.origin == grid.origin
}

#[derive(Component)]
pub struct StartupNucleus;

pub fn load_map_config_startup_system(
    mut commands: Commands,
    mut warmup: ResMut<WorldgenWarmupConfig>,
) {
    commands.insert_resource(ActiveMapName(active_map_slug_from_env()));

    let config = match load_map_config_from_env_result() {
        Ok(cfg) => cfg,
        Err(err) => {
            warn!("Failed to load selected map config: {err}");
            match load_default_map_asset() {
                Ok(default_asset) => default_asset,
                Err(default_err) => {
                    warn!(
                        "Failed to load assets/maps/default.ron: {default_err}. Using MapConfig::default()."
                    );
                    MapConfig::default()
                }
            }
        }
    };
    if let Err(errors) = validate_map_config(&config) {
        warn!("Invalid MapConfig, using default. errors={:?}", errors);
        let fallback = load_default_map_asset().unwrap_or_else(|_| MapConfig::default());
        warmup.ticks = fallback.warmup_ticks.unwrap_or(WARMUP_TICKS);
        commands.insert_resource(EnergyFieldGrid::new(
            fallback.width_cells,
            fallback.height_cells,
            fallback.cell_size,
            fallback.origin_vec2(),
        ));
        commands.insert_resource(NutrientFieldGrid::new(
            fallback.width_cells,
            fallback.height_cells,
            fallback.cell_size,
            fallback.origin_vec2(),
        ));
        commands.insert_resource(EcoPlayfieldMargin {
            cells: fallback.playfield_margin_cells.unwrap_or(0),
        });
        commands.insert_resource(fallback);
        return;
    }

    warmup.ticks = config.warmup_ticks.unwrap_or(WARMUP_TICKS);
    commands.insert_resource(EnergyFieldGrid::new(
        config.width_cells,
        config.height_cells,
        config.cell_size,
        config.origin_vec2(),
    ));
    commands.insert_resource(NutrientFieldGrid::new(
        config.width_cells,
        config.height_cells,
        config.cell_size,
        config.origin_vec2(),
    ));
    commands.insert_resource(EcoPlayfieldMargin {
        cells: config.playfield_margin_cells.unwrap_or(0),
    });
    commands.insert_resource(config);
}

/// Big Bang: seeds the energy field and nutrient grid with uniform values if configured.
/// Runs after grid creation, before nucleus spawn and warmup.
pub fn seed_initial_field_system(
    config: Option<Res<MapConfig>>,
    mut grid: Option<ResMut<EnergyFieldGrid>>,
    mut nutrients: Option<ResMut<NutrientFieldGrid>>,
) {
    let Some(config) = config else { return };
    let Some(ref mut grid) = grid else { return };

    if let Some(qe) = config.initial_field_qe {
        let freq = config.initial_field_freq.unwrap_or(85.0);
        grid.seed_uniform(qe, freq);
    }

    if let Some(water) = config.initial_nutrient_water {
        let Some(ref mut nutrients) = nutrients else { return };
        nutrients.seed_uniform(water * 0.3, water * 0.2, water * 0.15, water);
    }
}

/// Inserts DayNightConfig resource if map has day_period_ticks.
/// Runs after grid creation, before simulation loop.
pub fn init_day_night_config_system(
    mut commands: Commands,
    config: Option<Res<MapConfig>>,
    grid: Option<Res<EnergyFieldGrid>>,
) {
    let Some(config) = config else { return };
    // Cosmological anchor override from map config.
    if let Some(qe) = config.self_sustaining_qe {
        commands.insert_resource(
            crate::blueprint::equations::derived_thresholds::SelfSustainingQeMin(qe),
        );
    }
    let Some(period) = config.day_period_ticks else { return };
    let Some(grid) = grid else { return };
    let grid_width_world = grid.width as f32 * grid.cell_size;
    commands.insert_resource(
        crate::worldgen::systems::day_night::DayNightConfig::new(period, grid_width_world),
    );
}

pub fn spawn_nuclei_from_map_config_system(
    mut commands: Commands,
    layout: Res<SimWorldTransformParams>,
    config: Option<Res<MapConfig>>,
    nuclei: Query<Entity, With<EnergyNucleus>>,
) {
    if !nuclei.is_empty() {
        return;
    }
    let Some(config) = config else {
        return;
    };
    for spawn in resolve_nuclei_for_spawn(config.as_ref()) {
        let transform = if layout.use_xz_ground {
            Transform::from_xyz(spawn.position.x, layout.standing_y, spawn.position.y)
        } else {
            Transform::from_xyz(spawn.position.x, spawn.position.y, 0.0)
        };
        let mut ec = commands.spawn((
            StartupNucleus,
            Name::new(format!("nucleus::{}", spawn.name)),
            spawn.nucleus,
            transform,
            GlobalTransform::default(),
            Visibility::default(),
            InheritedVisibility::default(),
        ));
        // Finite reservoir only when map config specifies one.
        // No component = infinite emission (SparseSet opt-in).
        if let Some(qe) = spawn.reservoir {
            ec.insert(NucleusReservoir { qe });
        }
        if let Some(cfg) = spawn.ambient_pressure.as_ref() {
            // Capa 1 + 6: burbuja alineada al radio; `collision_interference` ignora `AmbientPressure`.
            ec.insert((
                SpatialVolume::new(spawn.nucleus.propagation_radius()),
                AmbientPressure::new(cfg.delta_qe, cfg.viscosity),
            ));
        }
    }
}

/// Entra en `GameState::Playing`; `PlayState` queda en `Warmup` (default del sub-estado).
pub fn enter_game_state_playing_system(mut next: ResMut<NextState<crate::simulation::GameState>>) {
    next.set(crate::simulation::GameState::Playing);
}

pub fn worldgen_warmup_system(world: &mut World) {
    let ticks = world.resource::<WorldgenWarmupConfig>().ticks;
    let step = Duration::from_secs_f32(1.0 / 60.0);
    if !world.contains_resource::<PropagationWriteBudget>() {
        world.insert_resource(PropagationWriteBudget::default());
    }
    for _ in 0..ticks {
        // Cada paso de warmup debe ver un dt > 0 (mismo contrato que un tick de simulación).
        if let Some(mut time) = world.get_resource_mut::<Time>() {
            time.advance_by(step);
        }
        if let Some(mut budget) = world.get_resource_mut::<PropagationWriteBudget>() {
            budget.remaining = u32::MAX;
        }
        if let Err(err) = world.run_system_cached(propagate_nuclei_system) {
            warn!("warmup: propagate_nuclei_system failed: {err:?}");
        }
        if let Err(err) = world.run_system_cached(dissipate_field_system) {
            warn!("warmup: dissipate_field_system failed: {err:?}");
        }
        if let Err(err) = world.run_system_cached(derive_cell_state_system) {
            warn!("warmup: derive_cell_state_system failed: {err:?}");
        }
    }

    materialization_full_world(world);
}

fn materialization_full_world(world: &mut World) {
    let layout = world
        .get_resource::<SimWorldTransformParams>()
        .copied()
        .unwrap_or_default();
    let Some(almanac) = world.get_resource::<AlchemicalAlmanac>().cloned() else {
        return;
    };
    let interference_t = world
        .get_resource::<Time>()
        .map(|time| time.elapsed_secs())
        .unwrap_or(0.0);
    let grid_snapshot = world.get_resource::<EnergyFieldGrid>().cloned();
    let Some(grid_snapshot) = grid_snapshot else {
        return;
    };
    let terrain_snapshot = world
        .get_resource::<TerrainField>()
        .filter(|terrain| terrain_field_aligned_with_grid(&grid_snapshot, terrain))
        .cloned();
    let mut to_spawn: Vec<(u32, u32, _)> = Vec::new();

    for y in 0..grid_snapshot.height {
        for x in 0..grid_snapshot.width {
            let Some(cell) = grid_snapshot.cell_xy(x, y) else {
                continue;
            };
            let terrain_type = terrain_snapshot.as_ref().map(|terrain| {
                let i = terrain.cell_index(x, y);
                terrain.terrain_type[i]
            });
            let Some(result) = materialize_cell_at_time(
                cell,
                &almanac,
                interference_t,
                grid_snapshot.cell_size,
                terrain_type,
            ) else {
                continue;
            };
            let Some(world_pos) = grid_snapshot.world_pos(x, y) else {
                continue;
            };
            to_spawn.push((
                x,
                y,
                (
                    Materialized {
                        cell_x: x as i32,
                        cell_y: y as i32,
                        archetype: result.archetype,
                    },
                    BaseEnergy::new(cell.accumulated_qe.max(0.0)),
                    OscillatorySignature::new(cell.dominant_frequency_hz, 0.0),
                    SpatialVolume::new(
                        (grid_snapshot.cell_size * crate::worldgen::constants::MATERIALIZED_COLLIDER_RADIUS_FACTOR)
                            .max(crate::worldgen::constants::MATERIALIZED_MIN_COLLIDER_RADIUS),
                    ),
                    MatterCoherence::new(
                        cell.matter_state,
                        crate::worldgen::constants::MATERIALIZED_SPAWN_BOND_ENERGY,
                        crate::worldgen::constants::MATERIALIZED_SPAWN_THERMAL_CONDUCTIVITY,
                    ),
                    crate::entities::component_groups::terrain_senescence(0),
                    layout.materialized_tile_transform(world_pos),
                    GlobalTransform::default(),
                    Sprite::default(),
                ),
            ));
        }
    }
    let mut spawned: Vec<(u32, u32, Entity)> = Vec::with_capacity(to_spawn.len());
    for (x, y, bundle) in to_spawn {
        let id = world.spawn(bundle).id();
        spawned.push((x, y, id));
    }
    let mut grid = world.resource_mut::<EnergyFieldGrid>();
    for (x, y, id) in spawned {
        if let Some(cell) = grid.cell_xy_mut(x, y) {
            cell.materialized_entity = Some(id);
        }
    }
}

/// Tras warmup síncrono: gameplay en `PlayState::Active`.
pub fn mark_play_state_active_system(mut next: ResMut<NextState<crate::simulation::PlayState>>) {
    next.set(crate::simulation::PlayState::Active);
}

#[cfg(test)]
mod tests {
    use super::{
        SimWorldTransformParams, WorldgenWarmupConfig, enter_game_state_playing_system,
        mark_play_state_active_system, worldgen_warmup_system,
    };
    use crate::blueprint::{AlchemicalAlmanac, ElementDef};
    use crate::layers::{MatterState, OscillatorySignature};
    use crate::simulation::{GameState, PlayState};
    use crate::worldgen::{EnergyFieldGrid, EnergyNucleus, Materialized, PropagationDecay};
    use bevy::prelude::*;
    use std::time::Instant;

    fn make_almanac_terra_ignis() -> AlchemicalAlmanac {
        AlchemicalAlmanac::from_defs(vec![
            ElementDef {
                name: "Terra".to_string(),
                symbol: "Terra".to_string(),
                atomic_number: 14,
                frequency_hz: 75.0,
                freq_band: (50.0, 84.0),
                bond_energy: 3000.0,
                conductivity: 0.4,
                visibility: 0.8,
                matter_state: MatterState::Solid,
                electronegativity: 0.0,
                ionization_ev: 0.0,
                color: (0.45, 0.34, 0.20),
                is_compound: false,
                phenology: None,
                hz_identity_weight: crate::blueprint::FIELD_EAC4_HZ_IDENTITY_WEIGHT_RON_ONLY,
            },
            ElementDef {
                name: "Ignis".to_string(),
                symbol: "Ignis".to_string(),
                atomic_number: 8,
                frequency_hz: 450.0,
                freq_band: (400.0, 500.0),
                bond_energy: 1000.0,
                conductivity: 0.5,
                visibility: 0.8,
                matter_state: MatterState::Plasma,
                electronegativity: 0.0,
                ionization_ev: 0.0,
                color: (1.0, 0.30, 0.0),
                is_compound: false,
                phenology: None,
                hz_identity_weight: crate::blueprint::FIELD_EAC4_HZ_IDENTITY_WEIGHT_RON_ONLY,
            },
        ])
    }

    fn setup_warmup_app(ticks: u32) -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_plugins(bevy::state::app::StatesPlugin);
        app.init_state::<GameState>();
        app.add_sub_state::<PlayState>();
        app.insert_resource(WorldgenWarmupConfig { ticks });
        app.insert_resource(make_almanac_terra_ignis());
        app.insert_resource(EnergyFieldGrid::new(20, 20, 1.0, Vec2::ZERO));
        app.insert_resource(SimWorldTransformParams::default());
        app.add_systems(
            Startup,
            (
                enter_game_state_playing_system,
                worldgen_warmup_system,
                mark_play_state_active_system,
            )
                .chain(),
        );
        app
    }

    #[test]
    fn startup_warmup_leaves_game_playing_and_play_active() {
        let mut app = setup_warmup_app(4);
        app.world_mut().spawn((
            EnergyNucleus::new(75.0, 120.0, 4.0, PropagationDecay::Flat),
            Transform::from_xyz(10.5, 10.5, 0.0),
        ));
        app.update();
        assert_eq!(
            app.world().resource::<State<GameState>>().get(),
            &GameState::Playing
        );
        assert_eq!(
            app.world().resource::<State<PlayState>>().get(),
            &PlayState::Active
        );
    }

    #[test]
    fn startup_warmup_with_zero_nuclei_is_noop() {
        let mut app = setup_warmup_app(8);
        app.update();
        let count = {
            let world = app.world_mut();
            let mut query = world.query::<&Materialized>();
            query.iter(world).count()
        };
        assert_eq!(count, 0);
    }

    #[test]
    fn startup_warmup_materializes_near_terra_and_ignis_nuclei() {
        // Emisión alta + radio acotado: cada celda del núcleo supera MIN_MATERIALIZATION_QE tras varios ticks.
        let mut app = setup_warmup_app(40);
        app.world_mut().spawn((
            EnergyNucleus::new(75.0, 4000.0, 3.5, PropagationDecay::Flat),
            Transform::from_xyz(4.5, 4.5, 0.0),
        ));
        app.world_mut().spawn((
            EnergyNucleus::new(450.0, 4000.0, 3.5, PropagationDecay::Flat),
            Transform::from_xyz(16.5, 16.5, 0.0),
        ));
        app.update();

        let mut near_terra = false;
        let mut near_ignis = false;
        {
            let world = app.world_mut();
            let mut query = world.query::<(&Materialized, &OscillatorySignature)>();
            for (mat, sig) in query.iter(world) {
                let dx_t = mat.cell_x - 4;
                let dy_t = mat.cell_y - 4;
                if dx_t * dx_t + dy_t * dy_t <= 9 && (sig.frequency_hz() - 75.0).abs() < 20.0 {
                    near_terra = true;
                }
                let dx_i = mat.cell_x - 16;
                let dy_i = mat.cell_y - 16;
                if dx_i * dx_i + dy_i * dy_i <= 16 && (sig.frequency_hz() - 450.0).abs() < 40.0 {
                    near_ignis = true;
                }
            }
        }
        assert!(
            near_terra && near_ignis,
            "expected materialization near Terra and Ignis"
        );
    }

    #[test]
    fn startup_warmup_accumulates_field_energy_with_single_nucleus() {
        let mut app = setup_warmup_app(12);
        app.world_mut().spawn((
            EnergyNucleus::new(75.0, 320.0, 8.0, PropagationDecay::Flat),
            Transform::from_xyz(10.0, 10.0, 0.0),
        ));
        app.update();

        let total_qe = app.world().resource::<EnergyFieldGrid>().total_qe();
        assert!(total_qe > 0.0);
    }

    #[test]
    fn startup_warmup_60_ticks_20x20_is_reasonable() {
        let mut app = setup_warmup_app(60);
        app.world_mut().spawn((
            EnergyNucleus::new(75.0, 120.0, 4.0, PropagationDecay::Flat),
            Transform::from_xyz(10.0, 10.0, 0.0),
        ));

        let start = Instant::now();
        app.update();
        let elapsed_ms = start.elapsed().as_millis();
        assert!(elapsed_ms < 1000, "warmup took too long: {elapsed_ms}ms");
    }
}
