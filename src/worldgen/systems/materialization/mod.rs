//! Materialization module: cell spawn/sync/despawn (spawn) + season/nucleus lifecycle (season).

pub mod season;
pub mod spawn;

pub use season::{
    NucleusFreqTrack, SEASON_TRANSITION_TICKS, SeasonTransition, find_season_preset, lerp_nucleus,
    nucleus_target_from_delta, season_change_begin_system, season_transition_tick_system,
    spawn_runtime_nucleus, worldgen_nucleus_death_notify_system,
    worldgen_nucleus_freq_changed_notify_system, worldgen_nucleus_freq_seed_system,
    worldgen_runtime_nucleus_created_system,
};
pub use spawn::{clear_stale_materialized_cell_refs_system, materialization_delta_system};

#[cfg(test)]
mod tests {
    use super::super::performance::{MatBudgetCounters, MatCacheStats, WorldgenPerfSettings};
    use super::super::prephysics;
    use super::{
        SEASON_TRANSITION_TICKS, SeasonTransition, clear_stale_materialized_cell_refs_system,
        find_season_preset, lerp_nucleus, nucleus_target_from_delta, season_change_begin_system,
        season_transition_tick_system, spawn_runtime_nucleus, worldgen_nucleus_death_notify_system,
        worldgen_nucleus_freq_changed_notify_system, worldgen_nucleus_freq_seed_system,
        worldgen_runtime_nucleus_created_system,
    };
    use crate::blueprint::{AlchemicalAlmanac, ElementDef};
    use crate::eco::boundary_field::EcoBoundaryField;
    use crate::eco::contracts::{BoundaryMarker, TransitionType, ZoneClass};
    use crate::events::{DeathCause, DeathEvent, SeasonChangeEvent, WorldgenMutationEvent};
    use crate::layers::{MatterCoherence, MatterState};
    use crate::runtime_platform::compat_2d3d::SimWorldTransformParams;
    use crate::simulation::states::{GameState, PlayState};
    use crate::simulation::{Phase, PlayerControlled};
    use crate::world::Scoreboard;
    use crate::worldgen::constants::MATERIALIZED_SPAWN_BOND_ENERGY;
    use crate::worldgen::map_config::{MapConfig, NucleusDelta, SeasonPreset};
    use crate::worldgen::{
        BoundaryVisual, EnergyFieldGrid, EnergyNucleus, Materialized, PropagationDecay,
        WorldArchetype,
    };
    use bevy::ecs::event::EventCursor;
    use bevy::prelude::*;
    use bevy::state::app::StatesPlugin;
    use bevy::state::prelude::State;
    use std::time::Duration;

    fn almanac_terra_ignis() -> AlchemicalAlmanac {
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

    /// Avanza simulación vía `FixedUpdate` como `SimulationPlugin` (sin `TimePlugin` que anule `delta`).
    fn step_sim(app: &mut App, dt_secs: f32, steps: u32) {
        let step = Duration::from_secs_f32(dt_secs);
        for _ in 0..steps {
            app.world_mut().resource_mut::<Time>().advance_by(step);
            app.world_mut().run_schedule(FixedUpdate);
        }
    }

    fn count_materialized(app: &mut App) -> usize {
        let world = app.world_mut();
        let mut q = world.query::<&Materialized>();
        q.iter(world).count()
    }

    fn fill_grid_high_qe(grid: &mut EnergyFieldGrid) {
        for y in 0..grid.height {
            for x in 0..grid.width {
                let c = grid.cell_xy_mut(x, y).expect("cell");
                c.accumulated_qe = 500.0;
                c.dominant_frequency_hz = 75.0;
                c.purity = 1.0;
                c.temperature = 80.0;
                c.matter_state = MatterState::Solid;
            }
        }
    }

    #[test]
    fn materialization_respects_spawn_budget_per_tick() {
        let mut app = App::new();
        setup_worldgen_runtime_chain(&mut app);
        app.insert_resource(WorldgenPerfSettings {
            max_material_spawn_per_tick: 2,
            ..Default::default()
        });
        let mut grid = EnergyFieldGrid::new(6, 6, 1.0, Vec2::ZERO);
        fill_grid_high_qe(&mut grid);
        app.insert_resource(grid);
        step_sim(&mut app, 1.0 / 60.0, 1);
        assert_eq!(count_materialized(&mut app), 2);
        step_sim(&mut app, 1.0 / 60.0, 1);
        assert_eq!(count_materialized(&mut app), 4);
    }

    #[test]
    fn e6_todas_celdas_frontera_respeta_spawn_budget() {
        let mut app = App::new();
        setup_worldgen_runtime_chain(&mut app);
        app.insert_resource(WorldgenPerfSettings {
            max_material_spawn_per_tick: 2,
            ..Default::default()
        });
        let w = 6u32;
        let h = 6u32;
        let len = (w * h) as usize;
        let mut grid = EnergyFieldGrid::new(w, h, 1.0, Vec2::ZERO);
        fill_grid_high_qe(&mut grid);
        app.insert_resource(grid);
        let boundary_cell = BoundaryMarker::Boundary {
            zone_a: ZoneClass::Surface,
            zone_b: ZoneClass::Void,
            gradient_factor: 0.5,
            transition_type: TransitionType::PhaseBoundary,
        };
        {
            let mut field = app.world_mut().resource_mut::<EcoBoundaryField>();
            field.width = w;
            field.height = h;
            field.cell_size = 1.0;
            field.origin = Vec2::ZERO;
            field.markers = vec![boundary_cell; len];
            field.cell_zone_ids = vec![0u16; len];
        }
        step_sim(&mut app, 1.0 / 60.0, 1);
        assert_eq!(
            count_materialized(&mut app),
            2,
            "boundaries must not raise per-tick spawn cap"
        );
    }

    #[test]
    fn materialization_delta_sin_eco_boundary_resource_no_panic() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.init_resource::<Time>()
            .insert_resource(almanac_terra_ignis());
        let mut grid = EnergyFieldGrid::new(2, 2, 1.0, Vec2::ZERO);
        fill_grid_high_qe(&mut grid);
        app.insert_resource(grid)
            .insert_resource(SimWorldTransformParams::default())
            .init_resource::<super::super::performance::WorldgenLodContext>()
            .init_resource::<WorldgenPerfSettings>()
            .init_resource::<super::super::performance::MaterializationCellCache>()
            .init_resource::<MatBudgetCounters>()
            .init_resource::<MatCacheStats>();
        app.add_systems(
            Update,
            (
                super::super::performance::sync_materialization_cache_len_system,
                super::materialization_delta_system,
                super::super::visual::flush_pending_energy_visual_rebuild_system,
            )
                .chain(),
        );
        app.update();
        assert!(count_materialized(&mut app) > 0);
        let archetype = {
            let world = app.world_mut();
            let mut q = world.query::<&Materialized>();
            q.iter(world).next().map(|m| m.archetype)
        };
        assert!(
            matches!(
                archetype,
                Some(WorldArchetype::TerraSolid) | Some(WorldArchetype::Mountain)
            ),
            "without TerrainField must keep base archetype"
        );
    }

    #[test]
    fn materialization_delta_terrain_misaligned_ignores_enrichment() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.init_resource::<Time>()
            .insert_resource(almanac_terra_ignis());
        let mut grid = EnergyFieldGrid::new(2, 2, 1.0, Vec2::ZERO);
        fill_grid_high_qe(&mut grid);
        app.insert_resource(grid)
            .insert_resource(SimWorldTransformParams::default())
            .init_resource::<super::super::performance::WorldgenLodContext>()
            .init_resource::<WorldgenPerfSettings>()
            .init_resource::<super::super::performance::MaterializationCellCache>()
            .init_resource::<MatBudgetCounters>()
            .init_resource::<MatCacheStats>();
        let mut misaligned = crate::topology::TerrainField::new(2, 2, 1.0, Vec2::new(9.0, 9.0), 7);
        for terrain_type in &mut misaligned.terrain_type {
            *terrain_type = crate::topology::TerrainType::Cliff;
        }
        app.insert_resource(misaligned);
        app.add_systems(
            Update,
            (
                super::super::performance::sync_materialization_cache_len_system,
                super::materialization_delta_system,
            )
                .chain(),
        );
        app.update();
        let archetype = {
            let world = app.world_mut();
            let mut q = world.query::<&Materialized>();
            q.iter(world).next().map(|m| m.archetype)
        };
        assert!(
            matches!(
                archetype,
                Some(WorldArchetype::TerraSolid) | Some(WorldArchetype::Mountain)
            ),
            "misaligned TerrainField must behave as None"
        );
    }

    #[test]
    fn materialization_lod_cull_skips_distant_cells() {
        let mut app = App::new();
        setup_worldgen_runtime_chain(&mut app);
        app.insert_resource(WorldgenPerfSettings {
            lod_materialization_cull_distance: 4.0,
            ..Default::default()
        });
        app.world_mut()
            .spawn((PlayerControlled, Transform::from_xyz(4.0, 4.0, 0.0)));
        let mut grid = EnergyFieldGrid::new(8, 8, 1.0, Vec2::ZERO);
        fill_grid_high_qe(&mut grid);
        app.insert_resource(grid);
        step_sim(&mut app, 1.0 / 60.0, 1);
        let grid = app.world().resource::<EnergyFieldGrid>();
        assert!(
            grid.cell_xy(7, 7)
                .expect("cell")
                .materialized_entity
                .is_none(),
            "corner outside cull does not materialize"
        );
        assert!(
            grid.cell_xy(4, 4)
                .expect("cell")
                .materialized_entity
                .is_some(),
            "cell under player materializes"
        );
    }

    #[test]
    fn e6_interior_alineado_no_inserta_boundary_visual() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.init_resource::<Time>()
            .insert_resource(almanac_terra_ignis());
        let mut grid = EnergyFieldGrid::new(1, 1, 1.0, Vec2::ZERO);
        fill_grid_high_qe(&mut grid);
        let field = EcoBoundaryField {
            width: 1,
            height: 1,
            cell_size: grid.cell_size,
            origin: grid.origin,
            markers: vec![BoundaryMarker::Interior { zone_id: 0 }],
            cell_zone_ids: vec![0],
            ..Default::default()
        };
        app.insert_resource(grid)
            .insert_resource(field)
            .insert_resource(SimWorldTransformParams::default())
            .init_resource::<super::super::performance::WorldgenLodContext>()
            .init_resource::<WorldgenPerfSettings>()
            .init_resource::<super::super::performance::MaterializationCellCache>()
            .init_resource::<MatBudgetCounters>()
            .init_resource::<MatCacheStats>();
        app.add_systems(
            Update,
            (
                super::super::performance::sync_materialization_cache_len_system,
                super::materialization_delta_system,
            )
                .chain(),
        );
        app.update();
        assert_eq!(count_materialized(&mut app), 1);
        let n_bv = {
            let w = app.world_mut();
            w.query_filtered::<(), With<BoundaryVisual>>()
                .iter(w)
                .count()
        };
        assert_eq!(n_bv, 0, "Interior must not set BoundaryVisual");
    }

    #[test]
    fn e6_frontera_element_frontier_inserta_boundary_visual_con_tipo() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.init_resource::<Time>()
            .insert_resource(almanac_terra_ignis());
        let mut grid = EnergyFieldGrid::new(1, 1, 1.0, Vec2::ZERO);
        fill_grid_high_qe(&mut grid);
        let field = EcoBoundaryField {
            width: 1,
            height: 1,
            cell_size: grid.cell_size,
            origin: grid.origin,
            markers: vec![BoundaryMarker::Boundary {
                zone_a: ZoneClass::Volcanic,
                zone_b: ZoneClass::Subaquatic,
                gradient_factor: 0.5,
                transition_type: TransitionType::ElementFrontier,
            }],
            cell_zone_ids: vec![0],
            ..Default::default()
        };
        app.insert_resource(grid)
            .insert_resource(field)
            .insert_resource(SimWorldTransformParams::default())
            .init_resource::<super::super::performance::WorldgenLodContext>()
            .init_resource::<WorldgenPerfSettings>()
            .init_resource::<super::super::performance::MaterializationCellCache>()
            .init_resource::<MatBudgetCounters>()
            .init_resource::<MatCacheStats>();
        app.add_systems(
            Update,
            (
                super::super::performance::sync_materialization_cache_len_system,
                super::materialization_delta_system,
            )
                .chain(),
        );
        app.update();
        let tt = {
            let w = app.world_mut();
            let mut q = w.query::<&BoundaryVisual>();
            q.iter(w).next().map(|b| b.transition_type)
        };
        assert_eq!(tt, Some(TransitionType::ElementFrontier));
    }

    #[test]
    fn e6_interior_luego_frontera_agrega_boundary_visual_misma_entidad() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.init_resource::<Time>()
            .insert_resource(almanac_terra_ignis());
        let mut grid = EnergyFieldGrid::new(1, 1, 1.0, Vec2::ZERO);
        fill_grid_high_qe(&mut grid);
        let field = EcoBoundaryField {
            width: 1,
            height: 1,
            cell_size: grid.cell_size,
            origin: grid.origin,
            markers: vec![BoundaryMarker::Interior { zone_id: 0 }],
            cell_zone_ids: vec![0],
            ..Default::default()
        };
        app.insert_resource(grid)
            .insert_resource(field)
            .insert_resource(SimWorldTransformParams::default())
            .init_resource::<super::super::performance::WorldgenLodContext>()
            .init_resource::<WorldgenPerfSettings>()
            .init_resource::<super::super::performance::MaterializationCellCache>()
            .init_resource::<MatBudgetCounters>()
            .init_resource::<MatCacheStats>();
        app.add_systems(
            Update,
            (
                super::super::performance::sync_materialization_cache_len_system,
                super::materialization_delta_system,
            )
                .chain(),
        );
        app.update();
        let e0 = app
            .world()
            .resource::<EnergyFieldGrid>()
            .cell_xy(0, 0)
            .expect("cell")
            .materialized_entity
            .expect("materialized");
        assert!(!app.world().entity(e0).contains::<BoundaryVisual>());
        let eb0 = app
            .world()
            .entity(e0)
            .get::<MatterCoherence>()
            .expect("MatterCoherence")
            .bond_energy_eb();
        assert!(
            (eb0 - MATERIALIZED_SPAWN_BOND_ENERGY).abs() < 0.01,
            "bond spawn consistent"
        );
        app.world_mut().resource_mut::<EcoBoundaryField>().markers[0] = BoundaryMarker::Boundary {
            zone_a: ZoneClass::Surface,
            zone_b: ZoneClass::Void,
            gradient_factor: 0.4,
            transition_type: TransitionType::PhaseBoundary,
        };
        app.update();
        let e1 = app
            .world()
            .resource::<EnergyFieldGrid>()
            .cell_xy(0, 0)
            .expect("cell")
            .materialized_entity
            .expect("materialized");
        assert_eq!(e0, e1, "same materialized entity, no extra spawn");
        assert!(app.world().entity(e1).contains::<BoundaryVisual>());
        let eb1 = app
            .world()
            .entity(e1)
            .get::<MatterCoherence>()
            .expect("MatterCoherence")
            .bond_energy_eb();
        assert!(
            (eb0 - eb1).abs() < f32::EPSILON,
            "only visual boundary changed, not coherence"
        );
    }

    #[test]
    fn e6_frontera_phase_boundary_inserta_boundary_visual() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.init_resource::<Time>()
            .insert_resource(almanac_terra_ignis());
        let mut grid = EnergyFieldGrid::new(1, 1, 1.0, Vec2::ZERO);
        fill_grid_high_qe(&mut grid);
        let field = EcoBoundaryField {
            width: 1,
            height: 1,
            cell_size: grid.cell_size,
            origin: grid.origin,
            markers: vec![BoundaryMarker::Boundary {
                zone_a: ZoneClass::Surface,
                zone_b: ZoneClass::Void,
                gradient_factor: 0.25,
                transition_type: TransitionType::PhaseBoundary,
            }],
            cell_zone_ids: vec![0],
            ..Default::default()
        };
        app.insert_resource(grid)
            .insert_resource(field)
            .insert_resource(SimWorldTransformParams::default())
            .init_resource::<super::super::performance::WorldgenLodContext>()
            .init_resource::<WorldgenPerfSettings>()
            .init_resource::<super::super::performance::MaterializationCellCache>()
            .init_resource::<MatBudgetCounters>()
            .init_resource::<MatCacheStats>();
        app.add_systems(
            Update,
            (
                super::super::performance::sync_materialization_cache_len_system,
                super::materialization_delta_system,
            )
                .chain(),
        );
        app.update();
        let (n, tt) = {
            let w = app.world_mut();
            let mut q = w.query::<&BoundaryVisual>();
            let list: Vec<_> = q.iter(w).collect();
            (list.len(), list.first().map(|b| b.transition_type))
        };
        assert_eq!(n, 1);
        assert_eq!(tt, Some(TransitionType::PhaseBoundary));
    }

    #[test]
    fn e6_frontera_a_interior_remueve_boundary_visual() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.init_resource::<Time>()
            .insert_resource(almanac_terra_ignis());
        let mut grid = EnergyFieldGrid::new(1, 1, 1.0, Vec2::ZERO);
        fill_grid_high_qe(&mut grid);
        let field = EcoBoundaryField {
            width: 1,
            height: 1,
            cell_size: grid.cell_size,
            origin: grid.origin,
            markers: vec![BoundaryMarker::Boundary {
                zone_a: ZoneClass::Surface,
                zone_b: ZoneClass::Subaquatic,
                gradient_factor: 0.5,
                transition_type: TransitionType::ElementFrontier,
            }],
            cell_zone_ids: vec![0],
            ..Default::default()
        };
        app.insert_resource(grid)
            .insert_resource(field)
            .insert_resource(SimWorldTransformParams::default())
            .init_resource::<super::super::performance::WorldgenLodContext>()
            .init_resource::<WorldgenPerfSettings>()
            .init_resource::<super::super::performance::MaterializationCellCache>()
            .init_resource::<MatBudgetCounters>()
            .init_resource::<MatCacheStats>();
        app.add_systems(
            Update,
            (
                super::super::performance::sync_materialization_cache_len_system,
                super::materialization_delta_system,
            )
                .chain(),
        );
        app.update();
        let e = {
            let w = app.world_mut();
            assert_eq!(
                w.query_filtered::<(), With<BoundaryVisual>>()
                    .iter(w)
                    .count(),
                1
            );
            w.resource::<EnergyFieldGrid>()
                .cell_xy(0, 0)
                .expect("cell")
                .materialized_entity
                .expect("materialized")
        };
        app.world_mut().resource_mut::<EcoBoundaryField>().markers[0] =
            BoundaryMarker::Interior { zone_id: 0 };
        app.update();
        assert!(
            !app.world_mut().entity(e).contains::<BoundaryVisual>(),
            "Interior must remove BoundaryVisual"
        );
    }

    #[test]
    fn materialization_cache_hits_on_second_pass_stable_grid() {
        // Dos pasadas de solo materialización (sin propagación que cambie qe entre frames).
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.init_resource::<Time>()
            .insert_resource(almanac_terra_ignis());
        let mut grid = EnergyFieldGrid::new(4, 4, 1.0, Vec2::ZERO);
        fill_grid_high_qe(&mut grid);
        app.insert_resource(grid)
            .insert_resource(SimWorldTransformParams::default())
            .init_resource::<crate::eco::boundary_field::EcoBoundaryField>()
            .init_resource::<super::super::performance::WorldgenLodContext>()
            .init_resource::<WorldgenPerfSettings>()
            .init_resource::<super::super::performance::MaterializationCellCache>()
            .init_resource::<MatBudgetCounters>()
            .init_resource::<MatCacheStats>();
        app.add_systems(
            Update,
            (
                super::super::performance::sync_materialization_cache_len_system,
                super::materialization_delta_system,
            )
                .chain(),
        );
        app.update();
        let h1 = app.world().resource::<MatCacheStats>().hits;
        app.update();
        let h2 = app.world().resource::<MatCacheStats>().hits;
        assert!(
            h2 > h1,
            "second pass must register cache hits (hits {h1} -> {h2})"
        );
    }

    #[test]
    fn pure_nucleus_target_from_delta_sums_fields() {
        let base = EnergyNucleus::new(75.0, 100.0, 4.0, PropagationDecay::Flat);
        let delta = NucleusDelta {
            nucleus_name: "x".into(),
            frequency_hz_delta: Some(375.0),
            emission_rate_delta: Some(50.0),
            propagation_radius_delta: Some(1.0),
        };
        let t = nucleus_target_from_delta(base, &delta);
        assert!((t.frequency_hz() - 450.0).abs() < 0.01);
        assert!((t.emission_rate_qe_s() - 150.0).abs() < 0.01);
        assert!((t.propagation_radius() - 5.0).abs() < 0.01);
    }

    #[test]
    fn pure_lerp_nucleus_midpoint() {
        let a = EnergyNucleus::new(75.0, 100.0, 4.0, PropagationDecay::Flat);
        let b = EnergyNucleus::new(175.0, 200.0, 8.0, PropagationDecay::InverseLinear);
        let m = lerp_nucleus(a, b, 0.5);
        assert!((m.frequency_hz() - 125.0).abs() < 0.01);
        assert!((m.emission_rate_qe_s() - 150.0).abs() < 0.01);
        assert!((m.propagation_radius() - 6.0).abs() < 0.01);
        assert_eq!(m.decay(), PropagationDecay::Flat);
    }

    #[test]
    fn pure_find_season_preset_by_name() {
        let mut cfg = MapConfig::default();
        cfg.seasons.push(SeasonPreset {
            name: "winter".into(),
            nucleus_deltas: vec![],
        });
        assert!(find_season_preset(&cfg, "winter").is_some());
        assert!(find_season_preset(&cfg, "summer").is_none());
    }

    #[test]
    fn stale_materialized_entity_cleared_when_entity_despawned() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .insert_resource(EnergyFieldGrid::new(4, 4, 1.0, Vec2::ZERO))
            .add_systems(Update, clear_stale_materialized_cell_refs_system);

        let dead = app
            .world_mut()
            .spawn((
                Materialized {
                    cell_x: 1,
                    cell_y: 1,
                    archetype: WorldArchetype::TerraSolid,
                },
                Transform::default(),
            ))
            .id();
        app.world_mut()
            .resource_mut::<EnergyFieldGrid>()
            .cell_xy_mut(1, 1)
            .expect("cell")
            .materialized_entity = Some(dead);

        app.world_mut().despawn(dead);
        app.update();

        let cell = app
            .world()
            .resource::<EnergyFieldGrid>()
            .cell_xy(1, 1)
            .expect("cell");
        assert!(cell.materialized_entity.is_none());
    }

    #[test]
    fn nucleus_death_emits_worldgen_mutation_destroyed() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .add_event::<DeathEvent>()
            .add_event::<WorldgenMutationEvent>()
            .init_resource::<super::NucleusFreqTrack>()
            .insert_resource(SimWorldTransformParams::default())
            .add_systems(Update, worldgen_nucleus_death_notify_system);

        let e = app
            .world_mut()
            .spawn((
                EnergyNucleus::new(75.0, 10.0, 2.0, PropagationDecay::Flat),
                Transform::from_xyz(3.0, 4.0, 0.0),
            ))
            .id();

        app.world_mut()
            .resource_mut::<Events<DeathEvent>>()
            .send(DeathEvent {
                entity: e,
                cause: DeathCause::Destruction,
            });
        app.update();

        let mut cursor = EventCursor::<WorldgenMutationEvent>::default();
        let mut count = 0;
        {
            let mut events = app
                .world_mut()
                .resource_mut::<Events<WorldgenMutationEvent>>();
            for ev in cursor.read(&mut events) {
                if matches!(ev, WorldgenMutationEvent::NucleusDestroyed { .. }) {
                    count += 1;
                }
            }
        }
        assert_eq!(count, 1);
    }

    #[test]
    fn season_transition_not_instant_full_jump_first_tick() {
        let mut cfg = MapConfig::default();
        let n0 = cfg.nuclei[0].clone();
        cfg.seasons.push(SeasonPreset {
            name: "heat".into(),
            nucleus_deltas: vec![NucleusDelta {
                nucleus_name: n0.name.clone(),
                frequency_hz_delta: Some(375.0),
                emission_rate_delta: None,
                propagation_radius_delta: None,
            }],
        });

        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .insert_resource(almanac_terra_ignis())
            .insert_resource(cfg)
            .init_resource::<SeasonTransition>()
            .add_event::<SeasonChangeEvent>()
            .add_event::<WorldgenMutationEvent>()
            .add_systems(
                Update,
                (season_change_begin_system, season_transition_tick_system).chain(),
            );

        app.world_mut().spawn((
            Name::new(format!("nucleus::{}", n0.name)),
            EnergyNucleus::new(
                n0.frequency_hz,
                n0.emission_rate_qe_s,
                n0.propagation_radius,
                n0.decay,
            ),
            Transform::default(),
        ));

        let start_hz = {
            let world = app.world_mut();
            let mut q = world.query_filtered::<&EnergyNucleus, With<Name>>();
            q.iter(world).next().expect("nucleus").frequency_hz()
        };

        app.world_mut()
            .resource_mut::<Events<SeasonChangeEvent>>()
            .send(SeasonChangeEvent {
                preset_name: "heat".into(),
            });
        app.update();

        let hz_after_one = {
            let world = app.world_mut();
            let mut q = world.query_filtered::<&EnergyNucleus, With<Name>>();
            q.iter(world).next().expect("nucleus").frequency_hz()
        };
        assert!(
            (hz_after_one - start_hz).abs() > 1.0 && (hz_after_one - 450.0).abs() > 10.0,
            "expected partial advance, hz={hz_after_one}"
        );
    }

    #[test]
    fn season_completes_with_season_applied_event() {
        let mut cfg = MapConfig::default();
        let n0 = cfg.nuclei[0].clone();
        cfg.seasons.push(SeasonPreset {
            name: "heat".into(),
            nucleus_deltas: vec![NucleusDelta {
                nucleus_name: n0.name.clone(),
                frequency_hz_delta: Some(375.0),
                emission_rate_delta: None,
                propagation_radius_delta: None,
            }],
        });

        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .insert_resource(almanac_terra_ignis())
            .insert_resource(cfg)
            .init_resource::<SeasonTransition>()
            .add_event::<SeasonChangeEvent>()
            .add_event::<WorldgenMutationEvent>()
            .add_systems(
                Update,
                (season_change_begin_system, season_transition_tick_system).chain(),
            );

        app.world_mut().spawn((
            Name::new(format!("nucleus::{}", n0.name)),
            EnergyNucleus::new(
                n0.frequency_hz,
                n0.emission_rate_qe_s,
                n0.propagation_radius,
                n0.decay,
            ),
            Transform::default(),
        ));

        app.world_mut()
            .resource_mut::<Events<SeasonChangeEvent>>()
            .send(SeasonChangeEvent {
                preset_name: "heat".into(),
            });

        for _ in 0..=SEASON_TRANSITION_TICKS {
            app.update();
        }

        let hz = {
            let world = app.world_mut();
            let mut q = world.query_filtered::<&EnergyNucleus, With<Name>>();
            q.iter(world).next().expect("nucleus").frequency_hz()
        };
        assert!((hz - 450.0).abs() < 1.0, "final hz={hz}");

        let mut cursor = EventCursor::<WorldgenMutationEvent>::default();
        let mut found = false;
        {
            let mut events = app
                .world_mut()
                .resource_mut::<Events<WorldgenMutationEvent>>();
            for ev in cursor.read(&mut events) {
                if let WorldgenMutationEvent::SeasonApplied { preset_name } = ev {
                    if preset_name == "heat" {
                        found = true;
                    }
                }
            }
        }
        assert!(found, "SeasonApplied expected");
    }

    /// Mientras la transición está activa no debe haber spam de `NucleusModified` (solo track).
    #[test]
    fn season_active_suppresses_nucleus_modified_events() {
        let mut cfg = MapConfig::default();
        let n0 = cfg.nuclei[0].clone();
        cfg.seasons.push(SeasonPreset {
            name: "heat".into(),
            nucleus_deltas: vec![NucleusDelta {
                nucleus_name: n0.name.clone(),
                frequency_hz_delta: Some(200.0),
                emission_rate_delta: None,
                propagation_radius_delta: None,
            }],
        });

        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .insert_resource(almanac_terra_ignis())
            .insert_resource(cfg)
            .init_resource::<SeasonTransition>()
            .init_resource::<super::NucleusFreqTrack>()
            .insert_resource(SimWorldTransformParams::default())
            .add_event::<SeasonChangeEvent>()
            .add_event::<WorldgenMutationEvent>()
            .add_systems(
                Update,
                (
                    season_change_begin_system,
                    season_transition_tick_system,
                    worldgen_nucleus_freq_seed_system,
                    worldgen_runtime_nucleus_created_system,
                    worldgen_nucleus_freq_changed_notify_system,
                )
                    .chain(),
            );

        app.world_mut().spawn((
            Name::new(format!("nucleus::{}", n0.name)),
            EnergyNucleus::new(
                n0.frequency_hz,
                n0.emission_rate_qe_s,
                n0.propagation_radius,
                n0.decay,
            ),
            Transform::default(),
        ));

        app.world_mut()
            .resource_mut::<Events<SeasonChangeEvent>>()
            .send(SeasonChangeEvent {
                preset_name: "heat".into(),
            });

        let mut cursor = EventCursor::<WorldgenMutationEvent>::default();
        app.update();
        // Tras el primer `update`, `tick_index` == 1. Hacen falta 58 pasos más para llegar a 59 sin cerrar;
        // el tick 60 completa la season y puede emitir `NucleusModified` fuera de esta política.
        for _ in 0..(SEASON_TRANSITION_TICKS - 2) {
            assert!(
                app.world().resource::<SeasonTransition>().is_active(),
                "transition still active before last tick"
            );
            app.update();
            let mut events = app
                .world_mut()
                .resource_mut::<Events<WorldgenMutationEvent>>();
            for ev in cursor.read(&mut events) {
                assert!(
                    !matches!(ev, WorldgenMutationEvent::NucleusModified { .. }),
                    "no NucleusModified while SeasonTransition active"
                );
            }
        }
    }

    /// Cadena worldgen/delta + PostPhysics con mismos `run_if` que producción (`Playing` ∧ `Active`).
    fn setup_worldgen_runtime_chain(app: &mut App) {
        app.add_plugins(StatesPlugin);
        app.init_state::<GameState>().add_sub_state::<PlayState>();
        app.init_resource::<Time>()
            .init_resource::<Scoreboard>()
            .insert_resource(MapConfig::default())
            .insert_resource(almanac_terra_ignis())
            .insert_resource(EnergyFieldGrid::new(20, 20, 1.0, Vec2::ZERO))
            .init_resource::<super::NucleusFreqTrack>()
            .init_resource::<SeasonTransition>()
            .init_resource::<super::super::performance::WorldgenPerfSettings>()
            .init_resource::<super::super::performance::WorldgenLodContext>()
            .init_resource::<super::super::performance::MaterializationCellCache>()
            .init_resource::<super::super::performance::MatBudgetCounters>()
            .init_resource::<super::super::performance::MatCacheStats>()
            .init_resource::<super::super::performance::PropagationWriteBudget>()
            .insert_resource(SimWorldTransformParams::default())
            .add_event::<WorldgenMutationEvent>()
            .add_event::<SeasonChangeEvent>()
            .add_event::<DeathEvent>()
            .configure_sets(
                FixedUpdate,
                (Phase::ThermodynamicLayer, Phase::MetabolicLayer).chain(),
            );
        prephysics::register_worldgen_core_prephysics_chain(app, FixedUpdate);
        prephysics::register_postphysics_nucleus_death_before_faction(app, FixedUpdate);
        // `step_sim` solo ejecuta `FixedUpdate`; forzamos estado como haría `Main` tras startup.
        let world = app.world_mut();
        world.insert_resource(State::new(GameState::Playing));
        world.insert_resource(State::new(PlayState::Active));
    }

    #[test]
    fn runtime_new_nucleus_materializes_after_ticks() {
        let mut app = App::new();
        setup_worldgen_runtime_chain(&mut app);

        let _ = spawn_runtime_nucleus(
            &mut app.world_mut().commands(),
            "runtime_a",
            EnergyNucleus::new(75.0, 800.0, 4.0, PropagationDecay::Flat),
            Vec2::new(10.5, 10.5),
            &SimWorldTransformParams::default(),
        );

        step_sim(&mut app, 1.0, 6);
        let count = count_materialized(&mut app);
        assert!(count > 0, "must materialize near nucleus");
    }

    #[test]
    fn destroy_nucleus_despawns_materialized_zone_after_ticks() {
        let mut app = App::new();
        setup_worldgen_runtime_chain(&mut app);

        let nucleus = app
            .world_mut()
            .spawn((
                EnergyNucleus::new(75.0, 900.0, 4.0, PropagationDecay::Flat),
                Transform::from_xyz(10.5, 10.5, 0.0),
            ))
            .id();

        step_sim(&mut app, 1.0, 8);
        let before = count_materialized(&mut app);
        assert!(before > 0);

        app.world_mut().despawn(nucleus);
        step_sim(&mut app, 1.0, 120);

        let after = count_materialized(&mut app);
        assert!(
            after < before,
            "after dissipation without nucleus materialization must drop: before={before} after={after}"
        );
    }

    #[test]
    fn terra_to_ignis_frequency_updates_materialized_archetype() {
        let mut app = App::new();
        setup_worldgen_runtime_chain(&mut app);

        let nucleus = app
            .world_mut()
            .spawn((
                EnergyNucleus::new(75.0, 1200.0, 5.0, PropagationDecay::Flat),
                Transform::from_xyz(10.5, 10.5, 0.0),
            ))
            .id();

        step_sim(&mut app, 1.0, 10);

        let mat_entity = {
            let grid = app.world().resource::<EnergyFieldGrid>();
            let cell = grid.cell_xy(10, 10).expect("center cell");
            cell.materialized_entity.expect("materialized entity")
        };

        {
            let world = app.world_mut();
            let mut e = world.entity_mut(nucleus);
            let mut n = e.get_mut::<EnergyNucleus>().expect("nucleus");
            n.set_frequency_hz(450.0);
        }

        step_sim(&mut app, 1.0, 12);

        let arch = app
            .world()
            .entity(mat_entity)
            .get::<Materialized>()
            .expect("Materialized")
            .archetype;
        assert!(
            matches!(
                arch,
                WorldArchetype::IgnisSolid
                    | WorldArchetype::IgnisLiquid
                    | WorldArchetype::IgnisGas
                    | WorldArchetype::IgnisPlasma
            ),
            "expected Ignis band archetype, got {:?}",
            arch
        );
    }

    #[test]
    fn full_cycle_create_dissipate_recreate_rematerializes() {
        let mut app = App::new();
        setup_worldgen_runtime_chain(&mut app);

        let n1 = app
            .world_mut()
            .spawn((
                EnergyNucleus::new(75.0, 1000.0, 4.0, PropagationDecay::Flat),
                Transform::from_xyz(8.5, 8.5, 0.0),
            ))
            .id();
        step_sim(&mut app, 1.0, 8);
        app.world_mut().despawn(n1);
        step_sim(&mut app, 1.0, 90);

        let _ = spawn_runtime_nucleus(
            &mut app.world_mut().commands(),
            "cycle_b",
            EnergyNucleus::new(75.0, 1000.0, 4.0, PropagationDecay::Flat),
            Vec2::new(8.5, 8.5),
            &SimWorldTransformParams::default(),
        );

        step_sim(&mut app, 1.0, 8);
        let count = count_materialized(&mut app);
        assert!(count > 0);
    }

    #[test]
    fn direct_nucleus_freq_change_emits_nucleus_modified_with_old_freq() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .insert_resource(almanac_terra_ignis())
            .init_resource::<super::NucleusFreqTrack>()
            .init_resource::<SeasonTransition>()
            .add_event::<WorldgenMutationEvent>()
            .add_systems(
                Update,
                (
                    worldgen_nucleus_freq_seed_system,
                    worldgen_nucleus_freq_changed_notify_system,
                )
                    .chain(),
            );

        let e = app
            .world_mut()
            .spawn((
                EnergyNucleus::new(75.0, 10.0, 2.0, PropagationDecay::Flat),
                Transform::default(),
            ))
            .id();
        app.update();

        {
            let world = app.world_mut();
            world
                .entity_mut(e)
                .get_mut::<EnergyNucleus>()
                .expect("nucleus")
                .set_frequency_hz(450.0);
        }
        app.update();

        let mut cursor = EventCursor::<WorldgenMutationEvent>::default();
        let mut saw = false;
        {
            let mut events = app
                .world_mut()
                .resource_mut::<Events<WorldgenMutationEvent>>();
            for ev in cursor.read(&mut events) {
                if let WorldgenMutationEvent::NucleusModified {
                    old_freq, new_freq, ..
                } = ev
                {
                    if (*old_freq - 75.0).abs() < 0.1 && (*new_freq - 450.0).abs() < 1.0 {
                        saw = true;
                    }
                }
            }
        }
        assert!(saw);
    }

    /// Mismo orden PostPhysics que `register_postphysics_nucleus_death_before_faction`: notify puede leer `EnergyNucleus` antes del despawn en `faction_identity_system`.
    #[test]
    fn postphysics_death_notify_before_faction_identity_chain() {
        use crate::simulation::post::faction_identity_system;
        use crate::world::Scoreboard;

        let mut app = App::new();
        app.init_resource::<Time>()
            .init_resource::<Scoreboard>()
            .init_resource::<super::NucleusFreqTrack>()
            .insert_resource(SimWorldTransformParams::default())
            .add_event::<DeathEvent>()
            .add_event::<WorldgenMutationEvent>()
            .configure_sets(Update, (Phase::MetabolicLayer,))
            .add_systems(
                Update,
                (
                    worldgen_nucleus_death_notify_system,
                    faction_identity_system,
                )
                    .chain()
                    .in_set(Phase::MetabolicLayer),
            );

        let entity = app
            .world_mut()
            .spawn((
                EnergyNucleus::new(75.0, 10.0, 2.0, PropagationDecay::Flat),
                Transform::from_xyz(1.0, 2.0, 0.0),
            ))
            .id();

        app.world_mut()
            .resource_mut::<Events<DeathEvent>>()
            .send(DeathEvent {
                entity,
                cause: DeathCause::Destruction,
            });
        app.update();

        let mut cursor = EventCursor::<WorldgenMutationEvent>::default();
        let mut saw_destroy = false;
        {
            let mut events = app
                .world_mut()
                .resource_mut::<Events<WorldgenMutationEvent>>();
            for ev in cursor.read(&mut events) {
                if let WorldgenMutationEvent::NucleusDestroyed {
                    entity: e,
                    position,
                } = ev
                {
                    assert_eq!(*e, entity);
                    assert!((position.x - 1.0).abs() < 0.01);
                    saw_destroy = true;
                }
            }
        }
        assert!(saw_destroy);
        assert!(app.world().get_entity(entity).is_err());
    }
}
