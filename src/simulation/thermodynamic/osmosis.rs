use bevy::prelude::*;

use crate::blueprint::{AlchemicalAlmanac, constants, equations};
use crate::bridge::bridged_ops::OsmosisEquationInput;
use crate::bridge::cache::BridgeCache;
use crate::bridge::config::{BridgeConfig, OsmosisBridge};
use crate::bridge::context_fill::{BridgePhase, BridgePhaseState};
use crate::bridge::decorator::{bridge_compute, bridge_warmup_record};
use crate::worldgen::EnergyFieldGrid;
use crate::worldgen::lod::{LodBand, distance_sq_cell_to_focus, lod_band_from_distance_sq};
use crate::worldgen::systems::performance::WorldgenLodContext;

#[inline]
fn apply_qe_transfer(
    qe_after: &mut [f32],
    hz_after: &mut [f32],
    touched: &mut [bool],
    src_idx: usize,
    dst_idx: usize,
    moved: f32,
) {
    if moved <= 0.0 {
        return;
    }
    let (src_hz, dst_hz) = equations::osmotic_frequency_mix(
        hz_after[src_idx],
        qe_after[src_idx],
        hz_after[dst_idx],
        qe_after[dst_idx],
        moved,
    );
    qe_after[src_idx] -= moved;
    qe_after[dst_idx] += moved;
    hz_after[src_idx] = src_hz;
    hz_after[dst_idx] = dst_hz;
    touched[src_idx] = true;
    touched[dst_idx] = true;
}

/// Difusión osmótica Capa 3: transfiere `qe` y frecuencia dominante entre celdas adyacentes.
/// Opera sobre `EnergyFieldGrid` y respeta presupuesto por frame.
pub fn osmotic_diffusion_system(
    mut grid: ResMut<EnergyFieldGrid>,
    almanac: Res<AlchemicalAlmanac>,
    phase_state: Res<BridgePhaseState>,
    bridge_config: Res<BridgeConfig<OsmosisBridge>>,
    mut bridge_cache: ResMut<BridgeCache<OsmosisBridge>>,
    lod_ctx: Option<Res<WorldgenLodContext>>,
) {
    let width = grid.width as usize;
    let height = grid.height as usize;
    let len = width * height;
    if len == 0 {
        return;
    }

    let cell_volume = grid.cell_size * grid.cell_size * grid.cell_size;
    if cell_volume <= 0.0 {
        return;
    }

    let mut qe_after = vec![0.0_f32; len];
    let mut hz_after = vec![0.0_f32; len];
    let mut electro = vec![0.0_f32; len];
    for y in 0..grid.height {
        for x in 0..grid.width {
            let idx = y as usize * width + x as usize;
            if let Some(cell) = grid.cell_xy(x, y) {
                qe_after[idx] = cell.accumulated_qe.max(0.0);
                hz_after[idx] = cell.dominant_frequency_hz.max(0.0);
                electro[idx] = almanac
                    .find_stable_band(cell.dominant_frequency_hz.max(0.0))
                    .map(|e| e.electronegativity.max(0.0))
                    .unwrap_or(0.0);
            }
        }
    }

    let mut processed_cells: u32 = 0;
    let mut touched = vec![false; len];

    for y in 0..grid.height {
        for x in 0..grid.width {
            if processed_cells >= constants::MAX_OSMOSIS_PER_FRAME {
                break;
            }
            if !grid.is_cell_dirty(x, y) {
                continue;
            }
            if let Some(ref lod) = lod_ctx
                && let Some(center) = grid.world_pos(x, y)
            {
                let dsq = distance_sq_cell_to_focus(center, lod.focus_world);
                if lod_band_from_distance_sq(dsq) == LodBand::Far {
                    continue;
                }
            }

            let idx = y as usize * width + x as usize;
            for (nx, ny) in [(x + 1, y), (x, y + 1)] {
                if nx >= grid.width || ny >= grid.height {
                    continue;
                }
                let nidx = ny as usize * width + nx as usize;
                let src_conc = equations::osmotic_concentration(qe_after[idx], cell_volume);
                let dst_conc = equations::osmotic_concentration(qe_after[nidx], cell_volume);
                let permeability = equations::osmotic_permeability(electro[idx], electro[nidx]);

                let input = OsmosisEquationInput {
                    concentration_a: src_conc,
                    concentration_b: dst_conc,
                    membrane_permeability: permeability,
                };

                let raw_delta = if phase_state.phase == BridgePhase::Warmup {
                    bridge_warmup_record(input, &bridge_config, &mut bridge_cache)
                } else {
                    bridge_compute(input, &bridge_config, &mut bridge_cache)
                };
                if !raw_delta.is_finite() {
                    continue;
                }
                let delta = raw_delta.clamp(
                    -constants::OSMOTIC_MAX_TRANSFER_PER_TICK,
                    constants::OSMOTIC_MAX_TRANSFER_PER_TICK,
                );

                if delta > 0.0 {
                    let moved = delta.min(qe_after[idx]);
                    apply_qe_transfer(&mut qe_after, &mut hz_after, &mut touched, idx, nidx, moved);
                } else if delta < 0.0 {
                    let moved = (-delta).min(qe_after[nidx]);
                    apply_qe_transfer(&mut qe_after, &mut hz_after, &mut touched, nidx, idx, moved);
                }
            }

            processed_cells = processed_cells.saturating_add(1);
        }
        if processed_cells >= constants::MAX_OSMOSIS_PER_FRAME {
            break;
        }
    }

    for y in 0..grid.height {
        for x in 0..grid.width {
            let idx = y as usize * width + x as usize;
            if !touched[idx] {
                continue;
            }
            if let Some(cell) = grid.cell_xy_mut(x, y) {
                cell.accumulated_qe = qe_after[idx].max(0.0);
                cell.dominant_frequency_hz = hz_after[idx].max(0.0);
            }
            grid.mark_cell_dirty(x, y);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bridge::cache::register_bridge_cache;
    use crate::bridge::presets::{BridgeDefaults, RigidityPreset};
    use crate::worldgen::FrequencyContribution;

    fn approx(a: f32, b: f32) {
        assert!((a - b).abs() < 1e-4, "a={a} b={b}");
    }

    fn test_app() -> App {
        let mut app = App::new();
        app.insert_resource(AlchemicalAlmanac::default())
            .insert_resource(BridgePhaseState::active_only())
            .insert_resource(OsmosisBridge::config_for_preset(RigidityPreset::Moderate));
        register_bridge_cache::<OsmosisBridge>(
            &mut app,
            OsmosisBridge::config_for_preset(RigidityPreset::Moderate).cache_capacity,
            crate::bridge::config::CachePolicy::Lru,
        );
        app
    }

    #[test]
    fn osmosis_two_cells_moves_towards_equilibrium_and_conserves_qe() {
        let mut app = test_app();
        let mut grid = EnergyFieldGrid::new(2, 1, 1.0, Vec2::ZERO);
        grid.cell_xy_mut(0, 0).expect("left").accumulated_qe = 200.0;
        grid.cell_xy_mut(1, 0).expect("right").accumulated_qe = 0.0;
        grid.mark_cell_dirty(0, 0);
        grid.mark_cell_dirty(1, 0);
        app.insert_resource(grid);
        app.add_systems(Update, osmotic_diffusion_system);

        app.update();
        let grid = app.world().resource::<EnergyFieldGrid>();
        let left = grid.cell_xy(0, 0).expect("left").accumulated_qe;
        let right = grid.cell_xy(1, 0).expect("right").accumulated_qe;
        assert!(left < 200.0, "left={left}");
        assert!(right > 0.0, "right={right}");
        approx(left + right, 200.0);
    }

    #[test]
    fn osmosis_respects_per_tick_clamp() {
        let mut app = test_app();
        let mut grid = EnergyFieldGrid::new(2, 1, 1.0, Vec2::ZERO);
        grid.cell_xy_mut(0, 0).expect("left").accumulated_qe = 10_000.0;
        grid.cell_xy_mut(1, 0).expect("right").accumulated_qe = 0.0;
        grid.mark_cell_dirty(0, 0);
        app.insert_resource(grid);
        app.add_systems(Update, osmotic_diffusion_system);

        app.update();
        let grid = app.world().resource::<EnergyFieldGrid>();
        let right = grid.cell_xy(1, 0).expect("right").accumulated_qe;
        assert!(right <= constants::OSMOTIC_MAX_TRANSFER_PER_TICK + 1e-4);
    }

    #[test]
    fn osmosis_respects_budget_cells_per_frame() {
        let mut app = test_app();
        let w = constants::MAX_OSMOSIS_PER_FRAME + 2;
        let mut grid = EnergyFieldGrid::new(w, 1, 1.0, Vec2::ZERO);
        for x in 0..w {
            let c = grid.cell_xy_mut(x, 0).expect("cell");
            c.accumulated_qe = if x % 2 == 0 { 100.0 } else { 0.0 };
            grid.mark_cell_dirty(x, 0);
        }
        app.insert_resource(grid);
        app.add_systems(Update, osmotic_diffusion_system);
        app.update();

        let grid = app.world().resource::<EnergyFieldGrid>();
        let moved_count = (0..w)
            .filter(|x| {
                let qe = grid.cell_xy(*x, 0).expect("cell").accumulated_qe;
                qe > 0.0 && qe < 100.0
            })
            .count() as u32;
        assert!(moved_count <= constants::MAX_OSMOSIS_PER_FRAME + 1);
    }

    #[test]
    fn osmosis_transfers_frequency_towards_donor_signature() {
        let mut app = test_app();
        let mut grid = EnergyFieldGrid::new(2, 1, 1.0, Vec2::ZERO);
        {
            let left = grid.cell_xy_mut(0, 0).expect("left");
            left.accumulated_qe = 100.0;
            left.dominant_frequency_hz = 1000.0;
            left.frequency_contributions
                .push(FrequencyContribution::new(
                    Entity::from_raw(1),
                    1000.0,
                    100.0,
                ));
        }
        {
            let right = grid.cell_xy_mut(1, 0).expect("right");
            right.accumulated_qe = 10.0;
            right.dominant_frequency_hz = 100.0;
            right
                .frequency_contributions
                .push(FrequencyContribution::new(Entity::from_raw(2), 100.0, 10.0));
        }
        grid.mark_cell_dirty(0, 0);
        app.insert_resource(grid);
        app.add_systems(Update, osmotic_diffusion_system);
        app.update();

        let grid = app.world().resource::<EnergyFieldGrid>();
        let right_hz = grid.cell_xy(1, 0).expect("right").dominant_frequency_hz;
        assert!(right_hz > 100.0, "right_hz={right_hz}");
    }

    #[test]
    fn osmosis_converges_with_bridge_disabled() {
        let mut app = test_app();
        app.world_mut()
            .resource_mut::<BridgeConfig<OsmosisBridge>>()
            .enabled = false;
        let mut grid = EnergyFieldGrid::new(2, 1, 1.0, Vec2::ZERO);
        grid.cell_xy_mut(0, 0).expect("left").accumulated_qe = 200.0;
        grid.cell_xy_mut(1, 0).expect("right").accumulated_qe = 0.0;
        grid.mark_cell_dirty(0, 0);
        grid.mark_cell_dirty(1, 0);
        app.insert_resource(grid);
        app.add_systems(Update, osmotic_diffusion_system);

        let mut prev_gap = 200.0_f32;
        for _ in 0..80 {
            app.update();
            let g = app.world().resource::<EnergyFieldGrid>();
            let left = g.cell_xy(0, 0).expect("left").accumulated_qe;
            let right = g.cell_xy(1, 0).expect("right").accumulated_qe;
            let gap = (left - right).abs();
            assert!(gap <= prev_gap + 1e-4, "prev_gap={prev_gap} gap={gap}");
            prev_gap = gap;
        }
        let grid = app.world().resource::<EnergyFieldGrid>();
        let left = grid.cell_xy(0, 0).expect("left").accumulated_qe;
        let right = grid.cell_xy(1, 0).expect("right").accumulated_qe;
        assert!((left - right).abs() < 10.0, "left={left} right={right}");
    }

    #[test]
    fn osmosis_budget_leaves_tail_unchanged() {
        let mut app = test_app();
        let w = constants::MAX_OSMOSIS_PER_FRAME + 4;
        let mut grid = EnergyFieldGrid::new(w, 1, 1.0, Vec2::ZERO);
        for x in 0..w {
            let c = grid.cell_xy_mut(x, 0).expect("cell");
            c.accumulated_qe = if x % 2 == 0 { 100.0 } else { 0.0 };
            grid.mark_cell_dirty(x, 0);
        }
        app.insert_resource(grid);
        app.add_systems(Update, osmotic_diffusion_system);
        app.update();

        let grid = app.world().resource::<EnergyFieldGrid>();
        let tail_idx = constants::MAX_OSMOSIS_PER_FRAME + 2;
        let tail_qe = grid.cell_xy(tail_idx, 0).expect("tail").accumulated_qe;
        assert!(
            (tail_qe - 100.0).abs() < 1e-6,
            "tail cell must remain unchanged, qe={tail_qe}"
        );
    }

    #[test]
    fn osmosis_skips_far_cells_by_lod() {
        let mut app = test_app();
        app.insert_resource(WorldgenLodContext {
            focus_world: Some(Vec2::new(0.5, 0.5)),
            sim_tick: 0,
        });
        let mut grid = EnergyFieldGrid::new(100, 1, 2.0, Vec2::ZERO);
        grid.cell_xy_mut(90, 0).expect("far donor").accumulated_qe = 100.0;
        grid.cell_xy_mut(91, 0)
            .expect("far receiver")
            .accumulated_qe = 0.0;
        grid.mark_cell_dirty(90, 0);
        app.insert_resource(grid);
        app.add_systems(Update, osmotic_diffusion_system);
        app.update();

        let grid = app.world().resource::<EnergyFieldGrid>();
        let donor = grid.cell_xy(90, 0).expect("far donor").accumulated_qe;
        let recv = grid.cell_xy(91, 0).expect("far receiver").accumulated_qe;
        assert!((donor - 100.0).abs() < 1e-6, "donor={donor}");
        assert!(recv.abs() < 1e-6, "receiver={recv}");
    }

    #[test]
    fn osmosis_runs_in_fixedupdate_reactions_set() {
        let mut app = test_app();
        app.configure_sets(
            FixedUpdate,
            (
                crate::simulation::Phase::Input,
                crate::simulation::Phase::ThermodynamicLayer,
                crate::simulation::Phase::AtomicLayer,
                crate::simulation::Phase::ChemicalLayer,
                crate::simulation::Phase::MetabolicLayer,
            )
                .chain(),
        );
        let mut grid = EnergyFieldGrid::new(2, 1, 1.0, Vec2::ZERO);
        grid.cell_xy_mut(0, 0).expect("left").accumulated_qe = 100.0;
        grid.cell_xy_mut(1, 0).expect("right").accumulated_qe = 0.0;
        grid.mark_cell_dirty(0, 0);
        app.insert_resource(grid);
        app.add_systems(
            FixedUpdate,
            osmotic_diffusion_system.in_set(crate::simulation::Phase::ChemicalLayer),
        );

        app.world_mut().run_schedule(FixedUpdate);
        let grid = app.world().resource::<EnergyFieldGrid>();
        let right = grid.cell_xy(1, 0).expect("right").accumulated_qe;
        assert!(right > 0.0, "right={right}");
    }
}
