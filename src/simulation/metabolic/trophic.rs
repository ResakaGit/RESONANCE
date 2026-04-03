//! D2: Trophic & Predation — 4 sistemas de cadena trófica.
//!
//! Fase: [`Phase::MetabolicLayer`], después de `growth_budget_system`.
//! Orden: satiation_decay → herbivore_forage → predation_attempt → decomposer.

use bevy::prelude::*;

use crate::blueprint::constants::*;
use crate::blueprint::equations;
use crate::blueprint::equations::{apply_metabolic_interference, metabolic_interference_factor};
use crate::events::{DeathCause, DeathEvent, HungerEvent, PreyConsumedEvent};
use crate::layers::energy::EnergyOps;
use crate::layers::{
    BaseEnergy, MatterCoherence, OscillatorySignature, TrophicConsumer, TrophicState,
};
use crate::runtime_platform::compat_2d3d::SimWorldTransformParams;
use crate::runtime_platform::core_math_agnostic::sim_plane_pos;
use crate::runtime_platform::simulation_tick::SimulationElapsed;
use crate::world::SpatialIndex;
use crate::worldgen::NutrientFieldGrid;

/// Límite de profundidad al recorrer cadenas de StructuralLink para tamaño de manada.
/// Max depth when traversing StructuralLink chains for pack size.
const PACK_TRAVERSE_MAX_SIZE: u32 = 8;

/// Cursor de throttle para queries espaciales tróficas por frame.
#[derive(Resource, Default)]
pub struct TrophicScanCursor {
    pub scans_this_frame: usize,
}

/// S1: Decay de saciedad + emisión de HungerEvent bajo umbral.
/// Per-tick (FixedUpdate = fixed timestep, dt=1 implícito).
pub fn trophic_satiation_decay_system(
    mut query: Query<(Entity, &mut TrophicState, &BaseEnergy)>,
    mut hunger_events: EventWriter<HungerEvent>,
) {
    for (entity, mut state, energy) in &mut query {
        let new_satiation = equations::satiation_decay(state.satiation, 1.0);
        if state.satiation != new_satiation {
            state.satiation = new_satiation;
        }
        if state.satiation < HUNGER_THRESHOLD {
            hunger_events.send(HungerEvent {
                entity,
                deficit_qe: energy.qe() * (HUNGER_THRESHOLD - state.satiation),
            });
        }
    }
}

/// S2: Herbívoros/omnívoros extraen qe de NutrientFieldGrid.
/// Per-tick (FixedUpdate — constant timestep, dt=1 implícito).
pub fn trophic_herbivore_forage_system(
    layout: Res<SimWorldTransformParams>,
    mut nutrient_grid: ResMut<NutrientFieldGrid>,
    mut query: Query<(
        &TrophicConsumer,
        &mut TrophicState,
        &mut BaseEnergy,
        &Transform,
    )>,
    mut cursor: ResMut<TrophicScanCursor>,
) {
    cursor.scans_this_frame = 0;

    for (consumer, mut state, mut energy, transform) in &mut query {
        if !consumer.is_herbivore() {
            continue;
        }
        if cursor.scans_this_frame >= TROPHIC_SCAN_BUDGET {
            break;
        }
        cursor.scans_this_frame += 1;

        let pos = sim_plane_pos(transform.translation, layout.use_xz_ground);
        let Some((cx, cy)) = nutrient_grid.cell_coords(pos) else {
            continue;
        };
        let Some(cell) = nutrient_grid.cell_xy(cx, cy) else {
            continue;
        };

        let cell_qe = equations::nutrient_cell_average(
            cell.carbon_norm,
            cell.nitrogen_norm,
            cell.phosphorus_norm,
            cell.water_norm,
        );
        let intake = equations::foraging_intake_from_field(cell_qe, consumer.intake_rate, 1.0);
        if intake <= 0.0 {
            continue;
        }

        let assimilated =
            equations::trophic_assimilation(intake, HERBIVORE_ASSIMILATION, TEMPERATURE_NEUTRAL);
        if assimilated > 0.0 {
            energy.inject(assimilated);
            let gain = equations::satiation_gain_from_meal(assimilated);
            let new_satiation = (state.satiation + gain).min(1.0);
            if state.satiation != new_satiation {
                state.satiation = new_satiation;
            }
        }

        let drain_fraction = equations::nutrient_drain_fraction(intake, cell_qe);
        if let Some(cell_mut) = nutrient_grid.cell_xy_mut(cx, cy) {
            cell_mut.carbon_norm = (cell_mut.carbon_norm * (1.0 - drain_fraction)).max(0.0);
            cell_mut.nitrogen_norm = (cell_mut.nitrogen_norm * (1.0 - drain_fraction)).max(0.0);
            cell_mut.phosphorus_norm = (cell_mut.phosphorus_norm * (1.0 - drain_fraction)).max(0.0);
            cell_mut.water_norm = (cell_mut.water_norm * (1.0 - drain_fraction)).max(0.0);
        }
    }
}

/// S3: Carnívoros/omnívoros intentan capturar presas cercanas.
/// Energy model: prey loses raw_drain, predator receives drained × assimilation × interference.
/// AC-1: assimilation is modulated by oscillatory alignment between predator and prey (Axiom 3×8).
pub fn trophic_predation_attempt_system(
    spatial_index: Res<SpatialIndex>,
    layout: Res<SimWorldTransformParams>,
    elapsed: Option<Res<SimulationElapsed>>,
    mut energy_ops: EnergyOps,
    mut predator_query: Query<(
        Entity,
        &TrophicConsumer,
        &mut TrophicState,
        &Transform,
        Option<&OscillatorySignature>,
        Option<&crate::layers::StructuralLink>,
    )>,
    link_count_query: Query<(), With<crate::layers::StructuralLink>>,
    prey_query: Query<(Option<&MatterCoherence>, Option<&OscillatorySignature>), With<BaseEnergy>>,
    mut prey_consumed: EventWriter<PreyConsumedEvent>,
    mut cursor: ResMut<TrophicScanCursor>,
) {
    let t = elapsed.map(|e| e.secs).unwrap_or(0.0);

    // Collect eligible predators bounded by scan budget to avoid borrow conflicts
    let remaining_budget = TROPHIC_SCAN_BUDGET.saturating_sub(cursor.scans_this_frame);
    let predators: Vec<_> = predator_query
        .iter()
        .filter(|(_, consumer, state, _, _, _)| {
            consumer.is_predator() && state.satiation <= PREDATION_WELL_FED_THRESHOLD
        })
        .take(remaining_budget)
        .map(|(e, consumer, _state, transform, osc, link)| {
            let (freq, phase) = osc
                .map(|o| (o.frequency_hz(), o.phase()))
                .unwrap_or((0.0, 0.0));
            // Pack size: traverse StructuralLink chain (cap 8, track prev to avoid backtrack).
            let pack_size = {
                let mut size = 1u32;
                let mut prev = e;
                let mut current = link.map(|l| l.target);
                while let Some(target) = current {
                    if target == prev || target == e || size >= PACK_TRAVERSE_MAX_SIZE {
                        break;
                    }
                    if link_count_query.get(target).is_err() {
                        break;
                    }
                    size += 1;
                    let next = predator_query
                        .get(target)
                        .ok()
                        .and_then(|(_, _, _, _, _, nl)| nl.map(|l| l.target))
                        .filter(|&t| t != prev && t != e);
                    prev = target;
                    current = next;
                }
                size
            };
            (
                e,
                consumer.intake_rate,
                transform.translation,
                freq,
                phase,
                pack_size,
            )
        })
        .collect();

    for (pred_entity, intake_rate, pred_pos_3d, pred_freq, pred_phase, pack_size) in &predators {
        if cursor.scans_this_frame >= TROPHIC_SCAN_BUDGET {
            break;
        }
        cursor.scans_this_frame += 1;

        let pred_pos = sim_plane_pos(*pred_pos_3d, layout.use_xz_ground);
        let nearby = spatial_index.query_radius(pred_pos, PREDATION_CAPTURE_RADIUS);

        for entry in &nearby {
            if entry.entity == *pred_entity {
                continue;
            }
            let Some(prey_qe) = energy_ops.qe(entry.entity) else {
                continue;
            };
            if prey_qe <= QE_MIN_EXISTENCE {
                continue;
            }

            let (bond_energy, prey_freq, prey_phase) = prey_query
                .get(entry.entity)
                .map(|(coherence, osc)| {
                    let be = coherence.map(|c| c.bond_energy_eb()).unwrap_or(0.0);
                    let (f, p) = osc
                        .map(|o| (o.frequency_hz(), o.phase()))
                        .unwrap_or((0.0, 0.0));
                    (be, f, p)
                })
                .unwrap_or((0.0, 0.0, 0.0));

            let distance = pred_pos.distance(entry.position);
            let success = equations::predation_success_probability(
                *intake_rate * PREDATION_INTAKE_TO_SPEED_FACTOR,
                PREY_BASE_SPEED,
                distance,
                TERRAIN_FACTOR_NEUTRAL,
            );

            // Deterministic threshold (rule 4: no RNG)
            if success < PREDATION_BASE_SUCCESS {
                continue;
            }

            // Energy model: drain raw amount, predator receives fraction scaled by
            // oscillatory alignment (AC-1: Axiom 3 × Axiom 8). Waste heat = drained - assimilated.
            // Pack hunt bonus: linked clusters drain more (cooperative predation).
            let hunt_bonus = equations::pack_hunt_bonus(*pack_size, prey_qe);
            let raw_drain = equations::predation_raw_drain(prey_qe, bond_energy) * hunt_bonus;
            if raw_drain <= 0.0 {
                continue;
            }

            let interference =
                metabolic_interference_factor(*pred_freq, *pred_phase, prey_freq, prey_phase, t);
            let drained = energy_ops.drain(entry.entity, raw_drain, DeathCause::Predation);
            let base_assimilated =
                equations::predation_assimilated(drained, CARNIVORE_ASSIMILATION);
            let assimilated = apply_metabolic_interference(base_assimilated, interference);
            energy_ops.inject(*pred_entity, assimilated);

            prey_consumed.send(PreyConsumedEvent {
                predator: *pred_entity,
                prey: entry.entity,
                qe_transferred: assimilated,
            });

            if let Ok((_, _, mut state, _, _, _)) = predator_query.get_mut(*pred_entity) {
                let new_satiation = (state.satiation + MEAL_SATIATION_GAIN).min(1.0);
                if state.satiation != new_satiation {
                    state.satiation = new_satiation;
                }
            }

            break; // One prey per predator per tick
        }
    }
}

/// S4: Descomponedores procesan cadáveres, devuelven nutrientes al grid.
/// Lee BaseEnergy del cadáver cuando disponible; fallback a constante si ya despawneado.
pub fn trophic_decomposer_system(
    layout: Res<SimWorldTransformParams>,
    mut death_events: EventReader<DeathEvent>,
    mut nutrient_grid: ResMut<NutrientFieldGrid>,
    corpse_query: Query<(&Transform, &BaseEnergy), Without<TrophicConsumer>>,
    transform_only: Query<&Transform, (Without<BaseEnergy>, Without<TrophicConsumer>)>,
    mut decomposer_query: Query<(&TrophicConsumer, &mut TrophicState, &mut BaseEnergy)>,
    mut cursor: ResMut<TrophicScanCursor>,
) {
    let corpses: Vec<_> = death_events.read().map(|ev| ev.entity).collect();

    if corpses.is_empty() {
        return;
    }

    for corpse_entity in &corpses {
        if cursor.scans_this_frame >= TROPHIC_SCAN_BUDGET {
            break;
        }

        // Read corpse data: position + remaining qe (may be 0 if fully drained)
        let (corpse_pos, corpse_qe) = if let Ok((t, e)) = corpse_query.get(*corpse_entity) {
            (
                sim_plane_pos(t.translation, layout.use_xz_ground),
                e.qe().max(DECOMPOSITION_DEFAULT_CORPSE_QE),
            )
        } else if let Ok(t) = transform_only.get(*corpse_entity) {
            (
                sim_plane_pos(t.translation, layout.use_xz_ground),
                DECOMPOSITION_DEFAULT_CORPSE_QE,
            )
        } else {
            continue;
        };
        cursor.scans_this_frame += 1;

        // Return nutrients to soil proportional to corpse energy
        let nutrient_return =
            equations::decomposition_nutrient_return(corpse_qe, DECOMPOSER_ASSIMILATION);
        if let Some((cx, cy)) = nutrient_grid.cell_coords(corpse_pos) {
            if let Some(cell) = nutrient_grid.cell_xy_mut(cx, cy) {
                let delta = equations::decomposition_grid_delta(nutrient_return);
                cell.regenerate(delta);
            }
        }

        // Feed nearest decomposer (first in iteration — deterministic for same archetype set)
        for (consumer, mut state, mut energy) in &mut decomposer_query {
            if !consumer.is_decomposer() {
                continue;
            }
            let gain = equations::decomposition_nutrient_return(corpse_qe, DECOMPOSER_ASSIMILATION);
            if gain > 0.0 {
                energy.inject(gain);
                let new_satiation =
                    (state.satiation + MEAL_SATIATION_GAIN * DECOMPOSER_SATIATION_FACTOR).min(1.0);
                if state.satiation != new_satiation {
                    state.satiation = new_satiation;
                }
            }
            break; // One decomposer per corpse per tick
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layers::TrophicClass;
    use crate::layers::trophic::{TrophicConsumer, TrophicState};

    fn test_app() -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_event::<DeathEvent>();
        app.add_event::<HungerEvent>();
        app.add_event::<PreyConsumedEvent>();
        app.init_resource::<TrophicScanCursor>();
        app
    }

    fn drain_hunger_events(app: &mut App) -> Vec<HungerEvent> {
        app.world_mut()
            .resource_mut::<Events<HungerEvent>>()
            .drain()
            .collect()
    }

    // ── S1: satiation decay ──

    #[test]
    fn satiation_decays_over_time() {
        let mut app = test_app();
        let e = app
            .world_mut()
            .spawn((TrophicState::new(0.8), BaseEnergy::new(100.0)))
            .id();
        app.add_systems(Update, trophic_satiation_decay_system);
        app.update();
        let state = app.world().get::<TrophicState>(e).unwrap();
        assert!(
            state.satiation < 0.8,
            "satiation should decay: {}",
            state.satiation
        );
    }

    #[test]
    fn hunger_event_emitted_below_threshold() {
        let mut app = test_app();
        app.world_mut()
            .spawn((TrophicState::new(0.1), BaseEnergy::new(100.0)));
        app.add_systems(Update, trophic_satiation_decay_system);
        app.update();
        let events = drain_hunger_events(&mut app);
        assert!(
            !events.is_empty(),
            "should emit HungerEvent when satiation < threshold"
        );
    }

    #[test]
    fn no_hunger_event_above_threshold() {
        let mut app = test_app();
        app.world_mut()
            .spawn((TrophicState::new(0.9), BaseEnergy::new(100.0)));
        app.add_systems(Update, trophic_satiation_decay_system);
        app.update();
        let events = drain_hunger_events(&mut app);
        assert!(
            events.is_empty(),
            "should not emit HungerEvent when well-fed"
        );
    }

    // ── S2: herbivore forage ──

    #[test]
    fn herbivore_gains_qe_from_nutrient_cell() {
        let mut app = test_app();
        app.insert_resource(SimWorldTransformParams::default());
        let mut grid = NutrientFieldGrid::new(4, 4, 10.0, bevy::math::Vec2::ZERO);
        if let Some(cell) = grid.cell_xy_mut(0, 0) {
            *cell = crate::worldgen::nutrient_field::NutrientCell::new(0.8, 0.8, 0.8, 0.8);
        }
        app.insert_resource(grid);

        let e = app
            .world_mut()
            .spawn((
                TrophicConsumer::new(TrophicClass::Herbivore, 1.0),
                TrophicState::new(0.3),
                BaseEnergy::new(50.0),
                Transform::from_xyz(5.0, 0.0, 0.0),
            ))
            .id();
        app.init_resource::<TrophicScanCursor>();
        app.add_systems(Update, trophic_herbivore_forage_system);
        app.update();

        let energy = app.world().get::<BaseEnergy>(e).unwrap();
        assert!(
            energy.qe() > 50.0,
            "herbivore should gain energy: {}",
            energy.qe()
        );
    }

    #[test]
    fn carnivore_does_not_forage() {
        let mut app = test_app();
        app.insert_resource(SimWorldTransformParams::default());
        let mut grid = NutrientFieldGrid::new(4, 4, 10.0, bevy::math::Vec2::ZERO);
        if let Some(cell) = grid.cell_xy_mut(0, 0) {
            *cell = crate::worldgen::nutrient_field::NutrientCell::new(0.8, 0.8, 0.8, 0.8);
        }
        app.insert_resource(grid);

        let e = app
            .world_mut()
            .spawn((
                TrophicConsumer::new(TrophicClass::Carnivore, 1.0),
                TrophicState::new(0.3),
                BaseEnergy::new(50.0),
                Transform::from_xyz(5.0, 0.0, 0.0),
            ))
            .id();
        app.init_resource::<TrophicScanCursor>();
        app.add_systems(Update, trophic_herbivore_forage_system);
        app.update();

        let energy = app.world().get::<BaseEnergy>(e).unwrap();
        assert!(
            (energy.qe() - 50.0).abs() < f32::EPSILON,
            "carnivore should not forage"
        );
    }

    // ── S3: predation attempt ──

    #[test]
    fn carnivore_drains_prey_on_capture() {
        let mut app = test_app();
        app.insert_resource(SimWorldTransformParams::default());

        let prey_id = app
            .world_mut()
            .spawn((
                BaseEnergy::new(100.0),
                Transform::from_xyz(1.0, 0.0, 0.0),
                crate::layers::SpatialVolume::new(1.0),
            ))
            .id();
        let pred_id = app
            .world_mut()
            .spawn((
                TrophicConsumer::new(TrophicClass::Carnivore, 5.0),
                TrophicState::new(0.2),
                BaseEnergy::new(80.0),
                Transform::from_xyz(0.0, 0.0, 0.0),
                crate::layers::SpatialVolume::new(1.0),
            ))
            .id();

        let mut index = SpatialIndex::new(5.0);
        index.insert(crate::world::space::SpatialEntry {
            entity: prey_id,
            position: bevy::math::Vec2::new(1.0, 0.0),
            radius: 1.0,
        });
        index.insert(crate::world::space::SpatialEntry {
            entity: pred_id,
            position: bevy::math::Vec2::ZERO,
            radius: 1.0,
        });
        app.insert_resource(index);
        app.init_resource::<TrophicScanCursor>();

        app.add_systems(Update, trophic_predation_attempt_system);
        app.update();

        let prey_energy = app.world().get::<BaseEnergy>(prey_id).unwrap();
        let pred_energy = app.world().get::<BaseEnergy>(pred_id).unwrap();
        // Prey loses raw_drain, predator gains drained × CARNIVORE_ASSIMILATION
        assert!(
            prey_energy.qe() < 100.0,
            "prey should lose energy: {}",
            prey_energy.qe()
        );
        assert!(
            pred_energy.qe() > 80.0,
            "predator should gain energy: {}",
            pred_energy.qe()
        );
        // Predator gains less than prey lost (waste heat)
        let prey_lost = 100.0 - prey_energy.qe();
        let pred_gained = pred_energy.qe() - 80.0;
        assert!(
            pred_gained < prey_lost,
            "predator should gain less than prey lost (assimilation loss)"
        );
    }

    #[test]
    fn prey_dies_when_qe_depleted() {
        use crate::simulation::test_support::drain_death_events;

        let mut app = test_app();
        app.insert_resource(SimWorldTransformParams::default());

        // predation_raw_drain(0.012, 0.0) = 0.012; drain 0.012 → qe=0 < QE_MIN → death
        let prey_id = app
            .world_mut()
            .spawn((
                BaseEnergy::new(0.012),
                Transform::from_xyz(1.0, 0.0, 0.0),
                crate::layers::SpatialVolume::new(1.0),
            ))
            .id();
        let _pred_id = app
            .world_mut()
            .spawn((
                TrophicConsumer::new(TrophicClass::Carnivore, 50.0),
                TrophicState::new(0.1),
                BaseEnergy::new(80.0),
                Transform::from_xyz(0.0, 0.0, 0.0),
                crate::layers::SpatialVolume::new(1.0),
            ))
            .id();

        let mut index = SpatialIndex::new(5.0);
        index.insert(crate::world::space::SpatialEntry {
            entity: prey_id,
            position: bevy::math::Vec2::new(1.0, 0.0),
            radius: 1.0,
        });
        index.insert(crate::world::space::SpatialEntry {
            entity: _pred_id,
            position: bevy::math::Vec2::ZERO,
            radius: 1.0,
        });
        app.insert_resource(index);
        app.init_resource::<TrophicScanCursor>();

        app.add_systems(Update, trophic_predation_attempt_system);
        app.update();

        let deaths = drain_death_events(&mut app);
        assert!(!deaths.is_empty(), "prey should die when qe depleted");
        assert_eq!(deaths[0].cause, DeathCause::Predation);
    }

    #[test]
    fn well_fed_predator_does_not_hunt() {
        let mut app = test_app();
        app.insert_resource(SimWorldTransformParams::default());

        let prey_id = app
            .world_mut()
            .spawn((
                BaseEnergy::new(100.0),
                Transform::from_xyz(1.0, 0.0, 0.0),
                crate::layers::SpatialVolume::new(1.0),
            ))
            .id();
        app.world_mut().spawn((
            TrophicConsumer::new(TrophicClass::Carnivore, 5.0),
            TrophicState::new(0.95), // Well-fed (> PREDATION_WELL_FED_THRESHOLD)
            BaseEnergy::new(80.0),
            Transform::from_xyz(0.0, 0.0, 0.0),
            crate::layers::SpatialVolume::new(1.0),
        ));

        let mut index = SpatialIndex::new(5.0);
        index.insert(crate::world::space::SpatialEntry {
            entity: prey_id,
            position: bevy::math::Vec2::new(1.0, 0.0),
            radius: 1.0,
        });
        app.insert_resource(index);
        app.init_resource::<TrophicScanCursor>();

        app.add_systems(Update, trophic_predation_attempt_system);
        app.update();

        let prey_energy = app.world().get::<BaseEnergy>(prey_id).unwrap();
        assert!(
            (prey_energy.qe() - 100.0).abs() < f32::EPSILON,
            "well-fed predator should not hunt"
        );
    }

    // ── S4: decomposer ──

    #[test]
    fn decomposer_returns_nutrients_to_grid() {
        let mut app = test_app();
        app.insert_resource(SimWorldTransformParams::default());
        let grid = NutrientFieldGrid::new(4, 4, 10.0, bevy::math::Vec2::ZERO);
        app.insert_resource(grid);
        app.init_resource::<TrophicScanCursor>();

        let corpse = app
            .world_mut()
            .spawn((BaseEnergy::new(100.0), Transform::from_xyz(5.0, 0.0, 0.0)))
            .id();

        let decomposer = app
            .world_mut()
            .spawn((
                TrophicConsumer::new(TrophicClass::Detritivore, 1.0),
                TrophicState::new(0.3),
                BaseEnergy::new(20.0),
            ))
            .id();

        app.world_mut()
            .resource_mut::<Events<DeathEvent>>()
            .send(DeathEvent {
                entity: corpse,
                cause: DeathCause::Dissipation,
            });

        app.add_systems(Update, trophic_decomposer_system);
        app.update();

        let nutrient = app.world().resource::<NutrientFieldGrid>();
        let cell = nutrient.cell_xy(0, 0).unwrap();
        assert!(
            cell.carbon_norm > 0.0,
            "nutrients should return to grid after decomposition"
        );

        // Decomposer should gain energy from corpse
        let decomp_energy = app.world().get::<BaseEnergy>(decomposer).unwrap();
        assert!(
            decomp_energy.qe() > 20.0,
            "decomposer should gain energy: {}",
            decomp_energy.qe()
        );
    }

    #[test]
    fn predation_waste_heat_conserves_energy_model() {
        // Verify that predator receives less than prey loses (assimilation loss = waste heat)
        let raw = equations::predation_raw_drain(100.0, 0.0);
        let assimilated = equations::predation_assimilated(raw, CARNIVORE_ASSIMILATION);
        assert!(assimilated < raw, "assimilated={assimilated} < raw={raw}");
        let waste = raw - assimilated;
        assert!(waste > 0.0, "waste heat should be positive");
    }

    // ── AC-1: interference × predation ──────────────────────────────────────────

    #[test]
    fn same_frequency_predator_gets_full_assimilation() {
        // factor = 1.0 when freq/phase identical → apply_metabolic_interference(x, 1.0) = x
        let raw = equations::predation_raw_drain(100.0, 0.0);
        let base = equations::predation_assimilated(raw, CARNIVORE_ASSIMILATION);
        let factor = metabolic_interference_factor(75.0, 0.0, 75.0, 0.0, 0.0);
        let result = apply_metabolic_interference(base, factor);
        assert!(
            (result - base).abs() < 1e-5,
            "same-freq: expected {base} got {result}"
        );
    }

    #[test]
    fn cross_band_predator_gets_reduced_assimilation() {
        // factor < 1.0 when freqs differ significantly
        use std::f32::consts::PI;
        let raw = equations::predation_raw_drain(100.0, 0.0);
        let base = equations::predation_assimilated(raw, CARNIVORE_ASSIMILATION);
        // Opposite phase → factor = FLOOR < 1.0
        let factor = metabolic_interference_factor(75.0, 0.0, 75.0, PI, 0.0);
        let result = apply_metabolic_interference(base, factor);
        assert!(
            result < base,
            "destructive: expected < {base}, got {result}"
        );
        assert!(result >= 0.0, "must be non-negative");
    }

    #[test]
    fn interference_factor_never_amplifies_assimilation() {
        use std::f32::consts::PI;
        let raw = equations::predation_raw_drain(50.0, 0.0);
        let base = equations::predation_assimilated(raw, CARNIVORE_ASSIMILATION);
        for (f_pred, p_pred, f_prey, p_prey, t) in [
            (75.0_f32, 0.0_f32, 75.0, 0.0, 0.0_f32),
            (75.0, 0.0, 75.0, PI, 0.0),
            (75.0, 0.0, 1000.0, 0.0, 1.0),
        ] {
            let factor = metabolic_interference_factor(f_pred, p_pred, f_prey, p_prey, t);
            let result = apply_metabolic_interference(base, factor);
            assert!(
                result <= base + 1e-5,
                "factor={factor} result={result} base={base}"
            );
        }
    }
}
