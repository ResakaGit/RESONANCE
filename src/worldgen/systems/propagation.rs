use bevy::prelude::*;

use super::performance::PropagationWriteBudget;
use crate::blueprint::AlchemicalAlmanac;
use crate::blueprint::constants::{NUCLEUS_DEPLETION_FACTOR, NUCLEUS_EMISSION_CUTOFF_QE};
use crate::layers::MatterState;
use crate::runtime_platform::compat_2d3d::SimWorldTransformParams;
use crate::runtime_platform::core_math_agnostic::sim_plane_pos;
use crate::simulation::time_compat::simulation_delta_secs;
use crate::topology::{
    ModulationParams, TerrainConfigRuntime, TerrainField, modulate_decay_with_params,
    modulate_diffusion_with_params, modulate_emission_with_params,
};
use crate::worldgen::constants::FIELD_CONDUCTIVITY_SPREAD;
use crate::worldgen::propagation::{
    cell_density, cell_matter_state, cell_temperature, diffusion_transfer, field_dissipation,
    nucleus_intensity_at, resolve_dominant_frequency,
};
use crate::worldgen::{EnergyFieldGrid, EnergyNucleus, FrequencyContribution, NucleusReservoir};

fn bbox_for_radius(
    grid: &EnergyFieldGrid,
    center: Vec2,
    radius: f32,
) -> Option<(u32, u32, u32, u32)> {
    if !center.is_finite() || !radius.is_finite() || radius <= 0.0 {
        return None;
    }

    let min_world = center - Vec2::splat(radius);
    let max_world = center + Vec2::splat(radius);

    let min_x = ((min_world.x - grid.origin.x) / grid.cell_size).floor() as i32;
    let min_y = ((min_world.y - grid.origin.y) / grid.cell_size).floor() as i32;
    let max_x = ((max_world.x - grid.origin.x) / grid.cell_size).floor() as i32;
    let max_y = ((max_world.y - grid.origin.y) / grid.cell_size).floor() as i32;

    let clamped_min_x = min_x.clamp(0, grid.width as i32 - 1);
    let clamped_min_y = min_y.clamp(0, grid.height as i32 - 1);
    let clamped_max_x = max_x.clamp(0, grid.width as i32 - 1);
    let clamped_max_y = max_y.clamp(0, grid.height as i32 - 1);

    if clamped_min_x > clamped_max_x || clamped_min_y > clamped_max_y {
        return None;
    }

    Some((
        clamped_min_x as u32,
        clamped_max_x as u32,
        clamped_min_y as u32,
        clamped_max_y as u32,
    ))
}

fn resolve_modulation_params(cfg: Option<&Res<TerrainConfigRuntime>>) -> ModulationParams {
    let mut params = cfg
        .and_then(|runtime| runtime.effective.as_ref().map(|v| v.modulation))
        .unwrap_or_default();
    let defaults = ModulationParams::default();
    if !params.altitude_emission_scale.is_finite() || params.altitude_emission_scale < 0.0 {
        params.altitude_emission_scale = defaults.altitude_emission_scale;
    }
    if !params.slope_diffusion_scale.is_finite() || params.slope_diffusion_scale < 0.0 {
        params.slope_diffusion_scale = defaults.slope_diffusion_scale;
    }
    if !params.reference_altitude.is_finite() {
        params.reference_altitude = defaults.reference_altitude;
    }
    if !params.decay_peak_factor.is_finite() || params.decay_peak_factor < 0.0 {
        params.decay_peak_factor = defaults.decay_peak_factor;
    }
    if !params.decay_valley_factor.is_finite() || params.decay_valley_factor < 0.0 {
        params.decay_valley_factor = defaults.decay_valley_factor;
    }
    if !params.decay_riverbed_factor.is_finite() || params.decay_riverbed_factor < 0.0 {
        params.decay_riverbed_factor = defaults.decay_riverbed_factor;
    }
    params
}

/// Emite núcleos al grid y acumula contribuciones por celda.
pub fn propagate_nuclei_system(
    fixed: Option<Res<Time<Fixed>>>,
    time: Res<Time>,
    layout: Res<SimWorldTransformParams>,
    mut nuclei: Query<(
        Entity,
        &EnergyNucleus,
        &Transform,
        Option<&mut NucleusReservoir>,
    )>,
    mut grid: ResMut<EnergyFieldGrid>,
    mut prop_budget: ResMut<PropagationWriteBudget>,
    terrain: Option<Res<TerrainField>>,
    terrain_cfg: Option<Res<TerrainConfigRuntime>>,
) {
    let dt = simulation_delta_secs(fixed, &time).max(0.0);
    if dt <= 0.0 {
        return;
    }

    grid.clear_frequency_contributions();
    let modulation = resolve_modulation_params(terrain_cfg.as_ref());

    let mut ordered_nuclei = nuclei.iter_mut().collect::<Vec<_>>();
    ordered_nuclei.sort_by_key(|(entity, _, _, _)| entity.index());

    let xz = layout.use_xz_ground;
    for (entity, nucleus, transform, mut reservoir) in ordered_nuclei {
        // Finite reservoir: skip depleted nuclei.
        if let Some(ref res) = reservoir {
            if res.qe < NUCLEUS_EMISSION_CUTOFF_QE {
                continue;
            }
        }
        let center = sim_plane_pos(transform.translation, xz);
        let emission_rate_qe_s = terrain
            .as_ref()
            .and_then(|t| t.sample_at_world(center))
            .map(|sample| {
                modulate_emission_with_params(
                    nucleus.emission_rate_qe_s(),
                    sample.altitude,
                    &modulation,
                )
            })
            .unwrap_or_else(|| nucleus.emission_rate_qe_s());
        let Some((min_x, max_x, min_y, max_y)) =
            bbox_for_radius(&grid, center, nucleus.propagation_radius())
        else {
            continue;
        };

        let mut weights = Vec::new();
        let mut total_weight = 0.0_f32;

        for y in min_y..=max_y {
            for x in min_x..=max_x {
                let Some(cell_center) = grid.world_pos(x, y) else {
                    continue;
                };
                let weight = nucleus_intensity_at(
                    center,
                    cell_center,
                    emission_rate_qe_s,
                    nucleus.propagation_radius(),
                    nucleus.decay(),
                );
                if weight <= 0.0 || !weight.is_finite() {
                    continue;
                }
                total_weight += weight;
                weights.push((x, y, weight));
            }
        }

        if total_weight <= 0.0 || !total_weight.is_finite() {
            continue;
        }

        let raw_budget = emission_rate_qe_s * dt;
        // Clamp emission to remaining reservoir (if finite).
        let nucleus_budget_qe = if let Some(ref res) = reservoir {
            raw_budget.min(res.qe * NUCLEUS_DEPLETION_FACTOR)
        } else {
            raw_budget
        };
        for (x, y, weight) in weights {
            if prop_budget.remaining == 0 {
                break;
            }
            let delta_qe = nucleus_budget_qe * (weight / total_weight);
            if !delta_qe.is_finite() || delta_qe <= 0.0 {
                continue;
            }
            {
                if let Some(cell) = grid.cell_xy_mut(x, y) {
                    cell.accumulated_qe += delta_qe;
                    cell.push_contribution_bounded(FrequencyContribution::new(
                        entity,
                        nucleus.frequency_hz(),
                        delta_qe,
                    ));
                }
            }
            prop_budget.remaining -= 1;
            grid.mark_cell_dirty(x, y);
        }
        // Drain reservoir by total emitted this tick.
        if let Some(ref mut res) = reservoir {
            res.qe = (res.qe - nucleus_budget_qe).max(0.0);
        }
    }
}

/// Aplica disipación global y difusión lateral 4-neighbors.
pub fn dissipate_field_system(
    fixed: Option<Res<Time<Fixed>>>,
    time: Res<Time>,
    mut grid: ResMut<EnergyFieldGrid>,
    terrain: Option<Res<TerrainField>>,
    terrain_cfg: Option<Res<TerrainConfigRuntime>>,
) {
    let dt = simulation_delta_secs(fixed, &time).max(0.0);
    if dt <= 0.0 {
        return;
    }

    let width = grid.width as usize;
    let height = grid.height as usize;
    let modulation = resolve_modulation_params(terrain_cfg.as_ref());
    let len = width * height;
    let mut after_dissipation = vec![0.0_f32; len];

    for y in 0..grid.height {
        for x in 0..grid.width {
            let idx = y as usize * width + x as usize;
            let current = grid
                .cell_xy(x, y)
                .map(|cell| cell.accumulated_qe)
                .unwrap_or(0.0);
            let decay_rate = terrain
                .as_ref()
                .and_then(|t| {
                    if t.is_valid(x, y) {
                        Some(modulate_decay_with_params(
                            crate::worldgen::FIELD_DECAY_RATE,
                            t.sample_at(x, y).terrain_type,
                            &modulation,
                        ))
                    } else {
                        None
                    }
                })
                .unwrap_or(crate::worldgen::FIELD_DECAY_RATE);
            after_dissipation[idx] = field_dissipation(current, decay_rate, dt);
        }
    }

    let mut deltas = vec![0.0_f32; len];
    let mut available = after_dissipation.clone();
    for y in 0..height {
        for x in 0..width {
            let idx = y * width + x;
            let current = after_dissipation[idx];

            {
                let right_idx = y * width + ((x + 1) % width);
                let flow_dir = if current >= after_dissipation[right_idx] {
                    Vec2::X
                } else {
                    -Vec2::X
                };
                let conductivity = terrain
                    .as_ref()
                    .and_then(|t| {
                        let x0 = x as u32;
                        let y0 = y as u32;
                        let x1 = ((x + 1) % width) as u32;
                        if t.is_valid(x0, y0) && t.is_valid(x1, y0) {
                            let a = t.sample_at(x0, y0);
                            let b = t.sample_at(x1, y0);
                            let ka = modulate_diffusion_with_params(
                                FIELD_CONDUCTIVITY_SPREAD,
                                a.slope,
                                flow_dir,
                                a.aspect,
                                &modulation,
                            );
                            let kb = modulate_diffusion_with_params(
                                FIELD_CONDUCTIVITY_SPREAD,
                                b.slope,
                                flow_dir,
                                b.aspect,
                                &modulation,
                            );
                            Some((ka + kb) * 0.5)
                        } else {
                            None
                        }
                    })
                    .unwrap_or(FIELD_CONDUCTIVITY_SPREAD);
                let transfer =
                    diffusion_transfer(current, after_dissipation[right_idx], conductivity, dt);
                if transfer > 0.0 {
                    let moved = transfer.min(available[idx]);
                    available[idx] -= moved;
                    available[right_idx] += moved;
                    deltas[idx] -= moved;
                    deltas[right_idx] += moved;
                } else if transfer < 0.0 {
                    let moved = (-transfer).min(available[right_idx]);
                    available[right_idx] -= moved;
                    available[idx] += moved;
                    deltas[idx] += moved;
                    deltas[right_idx] -= moved;
                }
            }

            {
                let up_idx = ((y + 1) % height) * width + x;
                let flow_dir = if current >= after_dissipation[up_idx] {
                    Vec2::Y
                } else {
                    -Vec2::Y
                };
                let conductivity = terrain
                    .as_ref()
                    .and_then(|t| {
                        let x0 = x as u32;
                        let y0 = y as u32;
                        let y1 = ((y + 1) % height) as u32;
                        if t.is_valid(x0, y0) && t.is_valid(x0, y1) {
                            let a = t.sample_at(x0, y0);
                            let b = t.sample_at(x0, y1);
                            let ka = modulate_diffusion_with_params(
                                FIELD_CONDUCTIVITY_SPREAD,
                                a.slope,
                                flow_dir,
                                a.aspect,
                                &modulation,
                            );
                            let kb = modulate_diffusion_with_params(
                                FIELD_CONDUCTIVITY_SPREAD,
                                b.slope,
                                flow_dir,
                                b.aspect,
                                &modulation,
                            );
                            Some((ka + kb) * 0.5)
                        } else {
                            None
                        }
                    })
                    .unwrap_or(FIELD_CONDUCTIVITY_SPREAD);
                let transfer =
                    diffusion_transfer(current, after_dissipation[up_idx], conductivity, dt);
                if transfer > 0.0 {
                    let moved = transfer.min(available[idx]);
                    available[idx] -= moved;
                    available[up_idx] += moved;
                    deltas[idx] -= moved;
                    deltas[up_idx] += moved;
                } else if transfer < 0.0 {
                    let moved = (-transfer).min(available[up_idx]);
                    available[up_idx] -= moved;
                    available[idx] += moved;
                    deltas[idx] += moved;
                    deltas[up_idx] -= moved;
                }
            }
        }
    }

    for y in 0..grid.height {
        for x in 0..grid.width {
            let idx = y as usize * width + x as usize;
            let old_qe = grid.cell_xy(x, y).map(|c| c.accumulated_qe).unwrap_or(0.0);
            let new_qe = (after_dissipation[idx] + deltas[idx]).max(0.0);
            if let Some(cell) = grid.cell_xy_mut(x, y) {
                cell.accumulated_qe = new_qe;
            }
            if (new_qe - old_qe).abs() > 1e-6 {
                grid.mark_cell_dirty(x, y);
            }
        }
    }
}

/// Deriva frecuencia dominante, pureza, temperatura y estado por celda.
pub fn derive_cell_state_system(
    almanac: Res<AlchemicalAlmanac>,
    mut grid: ResMut<EnergyFieldGrid>,
) {
    // Propagación/disipación marcaron dirty este tick; invalida derivados (p. ej. eco) aunque ρ→T no cruce ε.
    let from_field_writes = grid.any_dirty();
    let cell_size = grid.cell_size;
    let height = grid.height;
    let width = grid.width;
    let mut any_derived_change = false;
    for y in 0..height {
        for x in 0..width {
            let changed = {
                let Some(cell) = grid.cell_xy_mut(x, y) else {
                    continue;
                };
                let old_dom = cell.dominant_frequency_hz;
                let old_purity = cell.purity;
                let old_temp = cell.temperature;
                let old_state = cell.matter_state;

                if cell.accumulated_qe <= 0.0 {
                    cell.dominant_frequency_hz = 0.0;
                    cell.purity = 0.0;
                    cell.temperature = 0.0;
                    cell.matter_state = MatterState::Solid;
                } else {
                    let (dominant_hz, purity) =
                        resolve_dominant_frequency(cell.frequency_contributions());
                    let density = cell_density(cell.accumulated_qe, cell_size);
                    let temperature = cell_temperature(density);
                    let bond_energy = almanac
                        .find_stable_band(dominant_hz)
                        .map(|def| def.bond_energy.max(0.0))
                        .unwrap_or(1.0);

                    cell.dominant_frequency_hz = dominant_hz.max(0.0);
                    cell.purity = purity.clamp(0.0, 1.0);
                    cell.temperature = temperature.max(0.0);
                    cell.matter_state = cell_matter_state(cell.temperature, bond_energy);
                }

                (cell.dominant_frequency_hz - old_dom).abs() > 1e-5
                    || (cell.purity - old_purity).abs() > 1e-5
                    || (cell.temperature - old_temp).abs() > 1e-5
                    || cell.matter_state != old_state
            };
            if changed {
                any_derived_change = true;
                grid.mark_cell_dirty(x, y);
            }
        }
    }
    if any_derived_change || from_field_writes {
        grid.generation = grid.generation.wrapping_add(1);
    }
}

#[cfg(test)]
mod tests {
    use super::{derive_cell_state_system, dissipate_field_system, propagate_nuclei_system};
    use crate::blueprint::{AlchemicalAlmanac, ElementDef};
    use crate::layers::MatterState;
    use crate::simulation::Phase;
    use crate::topology::{TerrainField, TerrainType};
    use crate::worldgen::propagation::{cell_density, cell_matter_state, cell_temperature};
    use crate::worldgen::{
        EnergyFieldGrid, EnergyNucleus, FrequencyContribution, PropagationDecay,
    };
    use bevy::prelude::*;
    use std::time::Duration;

    fn make_almanac_terra() -> AlchemicalAlmanac {
        AlchemicalAlmanac::from_defs(vec![ElementDef {
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
        }])
    }

    fn tick_one_second(app: &mut App) {
        app.world_mut()
            .resource_mut::<Time>()
            .advance_by(Duration::from_secs_f32(1.0));
        app.update();
    }

    fn setup_app() -> App {
        let mut app = App::new();
        app.init_resource::<Time>()
            .init_resource::<super::super::performance::PropagationWriteBudget>()
            .insert_resource(
                crate::runtime_platform::compat_2d3d::SimWorldTransformParams::default(),
            );
        app.configure_sets(
            FixedUpdate,
            (
                Phase::Input,
                Phase::ThermodynamicLayer,
                Phase::AtomicLayer,
                Phase::ChemicalLayer,
                Phase::MetabolicLayer,
            )
                .chain(),
        );
        app
    }

    fn make_flat_terrain_for(grid: &EnergyFieldGrid) -> TerrainField {
        let mut terrain =
            TerrainField::new(grid.width, grid.height, grid.cell_size, grid.origin, 123);
        for i in 0..terrain.total_cells() {
            terrain.altitude[i] = crate::topology::REFERENCE_ALTITUDE;
            terrain.slope[i] = 0.0;
            terrain.aspect[i] = 0.0;
            terrain.terrain_type[i] = TerrainType::Plain;
        }
        terrain
    }

    fn spawn_test_nucleus(world: &mut World, pos: Vec2, hz: f32, radius: f32) -> Entity {
        world
            .spawn((
                EnergyNucleus::new(hz, 100.0, radius, PropagationDecay::Flat),
                Transform::from_xyz(pos.x, pos.y, 0.0),
            ))
            .id()
    }

    #[test]
    fn propagation_marks_cells_inside_radius_only() {
        let mut app = setup_app();
        app.insert_resource(make_almanac_terra())
            .insert_resource(EnergyFieldGrid::new(10, 10, 1.0, Vec2::ZERO))
            .add_systems(Update, (propagate_nuclei_system,));

        let center = Vec2::new(5.5, 5.5);
        spawn_test_nucleus(app.world_mut(), center, 75.0, 2.0);
        tick_one_second(&mut app);

        let grid = app.world().resource::<EnergyFieldGrid>();
        let inside = grid.cell_xy(5, 5).expect("center cell");
        let outside = grid.cell_xy(0, 0).expect("edge cell");
        assert!(inside.accumulated_qe > 0.0);
        assert_eq!(outside.accumulated_qe, 0.0);
    }

    #[test]
    fn field_dissipates_to_zero_without_nuclei() {
        let mut app = setup_app();
        let mut grid = EnergyFieldGrid::new(3, 3, 1.0, Vec2::ZERO);
        grid.cell_xy_mut(1, 1).expect("cell").accumulated_qe = 3.0;

        app.insert_resource(make_almanac_terra())
            .insert_resource(grid)
            .add_systems(Update, (dissipate_field_system,));

        for _ in 0..5 {
            tick_one_second(&mut app);
        }
        let grid = app.world().resource::<EnergyFieldGrid>();
        assert_eq!(grid.total_qe(), 0.0);
    }

    #[test]
    fn terra_nucleus_derives_dominant_frequency_close_to_75() {
        let mut app = setup_app();
        app.insert_resource(make_almanac_terra())
            .insert_resource(EnergyFieldGrid::new(10, 10, 1.0, Vec2::ZERO))
            .add_systems(
                Update,
                (propagate_nuclei_system, derive_cell_state_system).chain(),
            );

        spawn_test_nucleus(app.world_mut(), Vec2::new(5.5, 5.5), 75.0, 3.0);
        tick_one_second(&mut app);

        let grid = app.world().resource::<EnergyFieldGrid>();
        let center = grid.cell_xy(5, 5).expect("center cell");
        assert!((center.dominant_frequency_hz - 75.0).abs() < 0.1);
    }

    #[test]
    fn propagation_grid_cell_at_outside_returns_none() {
        let grid = EnergyFieldGrid::new(4, 4, 2.0, Vec2::ZERO);
        assert!(grid.cell_at(Vec2::new(-1.0, 0.0)).is_none());
        assert!(grid.cell_at(Vec2::new(9.0, 0.0)).is_none());
    }

    #[test]
    fn derive_cell_state_matches_equation_output() {
        let mut app = setup_app();
        let mut grid = EnergyFieldGrid::new(3, 3, 1.0, Vec2::ZERO);
        let cell = grid.cell_xy_mut(1, 1).expect("cell");
        cell.accumulated_qe = 200.0;
        cell.frequency_contributions
            .push(crate::worldgen::FrequencyContribution::new(
                Entity::from_raw(1),
                75.0,
                200.0,
            ));

        app.insert_resource(make_almanac_terra())
            .insert_resource(grid)
            .add_systems(Update, (derive_cell_state_system,));

        tick_one_second(&mut app);

        let grid = app.world().resource::<EnergyFieldGrid>();
        let cell = grid.cell_xy(1, 1).expect("cell");
        let density = cell_density(200.0, 1.0);
        let temp = cell_temperature(density);
        let expected = cell_matter_state(temp, 3000.0);
        assert_eq!(cell.matter_state, expected);
    }

    #[test]
    fn diffusion_never_increases_total_energy_without_nucleus() {
        let mut app = setup_app();
        let mut grid = EnergyFieldGrid::new(2, 1, 1.0, Vec2::ZERO);
        grid.cell_xy_mut(0, 0).expect("left").accumulated_qe = 10.0;
        grid.cell_xy_mut(1, 0).expect("right").accumulated_qe = 0.0;

        app.insert_resource(make_almanac_terra())
            .insert_resource(grid)
            .add_systems(Update, (dissipate_field_system,));

        let before = app.world().resource::<EnergyFieldGrid>().total_qe();
        tick_one_second(&mut app);
        let after = app.world().resource::<EnergyFieldGrid>().total_qe();
        assert!(after <= before + 1e-6);
    }

    #[test]
    fn propagation_respects_nucleus_emission_budget() {
        let mut app = setup_app();
        app.insert_resource(make_almanac_terra())
            .insert_resource(EnergyFieldGrid::new(12, 12, 1.0, Vec2::ZERO))
            .add_systems(Update, (propagate_nuclei_system,));

        app.world_mut().spawn((
            EnergyNucleus::new(75.0, 120.0, 3.0, PropagationDecay::InverseSquare),
            Transform::from_xyz(6.5, 6.5, 0.0),
        ));

        tick_one_second(&mut app);
        let total = app.world().resource::<EnergyFieldGrid>().total_qe();
        assert!((total - 120.0).abs() < 1e-3, "total_qe={total}");
    }

    #[derive(Resource, Default)]
    struct OrderLog(Vec<&'static str>);

    fn mark_after_propagate(mut log: ResMut<OrderLog>) {
        log.0.push("propagate");
    }
    fn mark_after_dissipate(mut log: ResMut<OrderLog>) {
        log.0.push("dissipate");
    }
    fn mark_after_derive(mut log: ResMut<OrderLog>) {
        log.0.push("derive");
    }

    #[test]
    fn prephysics_worldgen_order_is_deterministic() {
        let mut app = setup_app();
        app.insert_resource(OrderLog::default())
            .insert_resource(make_almanac_terra())
            .insert_resource(EnergyFieldGrid::new(8, 8, 1.0, Vec2::ZERO))
            .add_systems(
                FixedUpdate,
                (
                    propagate_nuclei_system,
                    mark_after_propagate.after(propagate_nuclei_system),
                    dissipate_field_system.after(mark_after_propagate),
                    mark_after_dissipate.after(dissipate_field_system),
                    derive_cell_state_system.after(mark_after_dissipate),
                    mark_after_derive.after(derive_cell_state_system),
                )
                    .chain()
                    .in_set(Phase::ThermodynamicLayer),
            );

        spawn_test_nucleus(app.world_mut(), Vec2::new(3.5, 3.5), 75.0, 2.0);

        app.world_mut()
            .resource_mut::<Time>()
            .advance_by(Duration::from_secs_f32(1.0));
        app.world_mut().run_schedule(FixedUpdate);

        let log = &app.world().resource::<OrderLog>().0;
        assert_eq!(log.as_slice(), &["propagate", "dissipate", "derive"]);
    }

    #[test]
    fn derive_cell_state_does_not_mark_dirty_when_already_converged() {
        let mut app = setup_app();
        let mut grid = EnergyFieldGrid::new(2, 2, 1.0, Vec2::ZERO);
        {
            let c = grid.cell_xy_mut(0, 0).expect("cell");
            c.accumulated_qe = 200.0;
            c.frequency_contributions.push(FrequencyContribution::new(
                Entity::from_raw(1),
                75.0,
                200.0,
            ));
            let density = cell_density(200.0, 1.0);
            let temp = cell_temperature(density);
            c.dominant_frequency_hz = 75.0;
            c.purity = 1.0;
            c.temperature = temp;
            c.matter_state = cell_matter_state(temp, 3000.0);
        }
        grid.clear_dirty();
        app.insert_resource(make_almanac_terra())
            .insert_resource(grid)
            .add_systems(Update, (derive_cell_state_system,));
        tick_one_second(&mut app);
        let grid = app.world().resource::<EnergyFieldGrid>();
        assert!(
            !grid.is_cell_dirty(0, 0),
            "no derivation changes must not mark dirty"
        );
    }

    #[test]
    fn propagation_respects_cell_write_budget() {
        let mut app = setup_app();
        app.insert_resource(super::super::performance::PropagationWriteBudget { remaining: 12 })
            .insert_resource(make_almanac_terra())
            .insert_resource(EnergyFieldGrid::new(24, 24, 1.0, Vec2::ZERO))
            .add_systems(Update, (propagate_nuclei_system,));
        app.world_mut().spawn((
            EnergyNucleus::new(75.0, 5000.0, 20.0, PropagationDecay::Flat),
            Transform::from_xyz(12.0, 12.0, 0.0),
        ));
        tick_one_second(&mut app);
        let grid = app.world().resource::<EnergyFieldGrid>();
        let mut with_qe = 0u32;
        for y in 0..grid.height {
            for x in 0..grid.width {
                if grid.cell_xy(x, y).is_some_and(|c| c.accumulated_qe > 0.0) {
                    with_qe += 1;
                }
            }
        }
        assert!(
            with_qe <= 12,
            "at most 12 propagation writes: cells with qe={with_qe}"
        );
    }

    #[test]
    fn with_flat_terrain_matches_without_terrain_in_single_tick() {
        let mut app_without = setup_app();
        app_without
            .insert_resource(make_almanac_terra())
            .insert_resource(EnergyFieldGrid::new(10, 10, 1.0, Vec2::ZERO))
            .add_systems(
                Update,
                (
                    propagate_nuclei_system,
                    dissipate_field_system,
                    derive_cell_state_system,
                )
                    .chain(),
            );
        spawn_test_nucleus(app_without.world_mut(), Vec2::new(5.0, 5.0), 75.0, 3.0);
        tick_one_second(&mut app_without);

        let mut app_flat = setup_app();
        app_flat
            .insert_resource(make_almanac_terra())
            .insert_resource(EnergyFieldGrid::new(10, 10, 1.0, Vec2::ZERO));
        let terrain = {
            let grid = app_flat.world().resource::<EnergyFieldGrid>().clone();
            make_flat_terrain_for(&grid)
        };
        app_flat.insert_resource(terrain);
        app_flat.add_systems(
            Update,
            (
                propagate_nuclei_system,
                dissipate_field_system,
                derive_cell_state_system,
            )
                .chain(),
        );
        spawn_test_nucleus(app_flat.world_mut(), Vec2::new(5.0, 5.0), 75.0, 3.0);
        tick_one_second(&mut app_flat);

        let a = app_without.world().resource::<EnergyFieldGrid>();
        let b = app_flat.world().resource::<EnergyFieldGrid>();
        for y in 0..a.height {
            for x in 0..a.width {
                let qa = a.cell_xy(x, y).expect("cell").accumulated_qe;
                let qb = b.cell_xy(x, y).expect("cell").accumulated_qe;
                assert!((qa - qb).abs() < 1e-5, "x={x}, y={y}, qa={qa}, qb={qb}");
                let fa = a.cell_xy(x, y).expect("cell").dominant_frequency_hz;
                let fb = b.cell_xy(x, y).expect("cell").dominant_frequency_hz;
                let pa = a.cell_xy(x, y).expect("cell").purity;
                let pb = b.cell_xy(x, y).expect("cell").purity;
                assert!(
                    (fa - fb).abs() < 1e-5,
                    "freq x={x}, y={y}, fa={fa}, fb={fb}"
                );
                assert!(
                    (pa - pb).abs() < 1e-5,
                    "purity x={x}, y={y}, pa={pa}, pb={pb}"
                );
            }
        }
    }

    #[test]
    fn valley_accumulates_more_than_peak_after_propagation() {
        let mut app = setup_app();
        let grid = EnergyFieldGrid::new(3, 1, 1.0, Vec2::ZERO);
        let mut terrain = make_flat_terrain_for(&grid);
        // topología inclinada de derecha a izquierda: flujo favorece hacia x=0 (valle).
        terrain.slope.fill(8.0);
        terrain.aspect.fill(180.0);
        terrain.altitude[0] = crate::topology::REFERENCE_ALTITUDE - 20.0;
        terrain.altitude[2] = crate::topology::REFERENCE_ALTITUDE + 20.0;
        terrain.terrain_type[0] = TerrainType::Valley;
        terrain.terrain_type[2] = TerrainType::Peak;

        app.insert_resource(make_almanac_terra())
            .insert_resource(grid)
            .insert_resource(terrain)
            .add_systems(
                Update,
                (propagate_nuclei_system, dissipate_field_system).chain(),
            );

        app.world_mut().spawn((
            EnergyNucleus::new(75.0, 120.0, 6.0, PropagationDecay::Flat),
            Transform::from_xyz(1.5, 0.5, 0.0),
        ));
        tick_one_second(&mut app);

        let g = app.world().resource::<EnergyFieldGrid>();
        let valley_qe = g.cell_xy(0, 0).expect("valley").accumulated_qe;
        let peak_qe = g.cell_xy(2, 0).expect("peak").accumulated_qe;
        assert!(
            valley_qe > peak_qe,
            "expect higher accumulation in valley: valley={valley_qe}, peak={peak_qe}"
        );
    }
}
