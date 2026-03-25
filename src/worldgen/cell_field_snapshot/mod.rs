//! EPI1 — Snapshot de campo por celda (inferencia de partes).
//! Proyección invalidable alineada a `EnergyFieldGrid.generation` (opción A: rebuild completo al bump).
//! En `prephysics` el sync corre **después** de `materialization_delta_system` / `flush_pending_energy_visual_rebuild_system`
//! para que `materialized_entity` y derivados coincidan con el grid en el mismo tick.

mod constants;
pub mod gpu_layout;

use bevy::prelude::*;

use crate::layers::MatterState;
use crate::worldgen::constants::MAX_FREQUENCY_CONTRIBUTIONS;
use crate::worldgen::{EnergyCell, EnergyFieldGrid, FrequencyContribution};

use constants::{fnv1a_u32_mix, FNV1A_U32_OFFSET_BASIS};

/// Tupla ordenable para huella de contribuciones (índice entidad, generación Bevy, bits f32).
type ContribFingerprintKey = (u32, u32, u32, u32);

/// Vista compacta y `Copy` de derivados ya resueltos en [`EnergyCell`].
/// El espectro completo no se clona: solo huella estable de `frequency_contributions`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CellFieldSnapshot {
    pub accumulated_qe: f32,
    pub dominant_frequency_hz: f32,
    pub purity: f32,
    pub temperature: f32,
    pub matter_state: MatterState,
    pub materialized_entity: Option<Entity>,
    /// Huella ordenada de contribuciones (no es inyectiva; ver tests).
    pub contributions_fingerprint: u32,
}

/// Huella determinista del multiset de contribuciones (orden de inserción irrelevante).
#[inline]
pub fn frequency_contributions_fingerprint(contributions: &[FrequencyContribution]) -> u32 {
    let mut buf = [ContribFingerprintKey::default(); MAX_FREQUENCY_CONTRIBUTIONS];
    for (n, c) in contributions
        .iter()
        .take(MAX_FREQUENCY_CONTRIBUTIONS)
        .enumerate()
    {
        let e = c.source_entity();
        buf[n] = (
            e.index(),
            e.generation(),
            sanitize_f32_bits(c.frequency_hz()),
            sanitize_f32_bits(c.intensity_qe()),
        );
    }
    let n = contributions.len().min(MAX_FREQUENCY_CONTRIBUTIONS);
    buf[..n].sort_unstable();
    fold_fingerprint(&buf[..n])
}

#[inline]
fn sanitize_f32_bits(x: f32) -> u32 {
    if x.is_finite() {
        x.to_bits()
    } else {
        0
    }
}

fn fold_fingerprint(sorted: &[ContribFingerprintKey]) -> u32 {
    let mut h = FNV1A_U32_OFFSET_BASIS;
    for &(idx, entity_gen, fb, ib) in sorted {
        h = fnv1a_u32_mix(h, idx);
        h = fnv1a_u32_mix(h, entity_gen);
        h = fnv1a_u32_mix(h, fb);
        h = fnv1a_u32_mix(h, ib);
    }
    h
}

/// Construye snapshot desde una celda ya derivada (sin I/O).
#[inline]
pub fn cell_field_snapshot_from_energy_cell(cell: &EnergyCell) -> CellFieldSnapshot {
    CellFieldSnapshot {
        accumulated_qe: snapshot_scalar_nonneg(cell.accumulated_qe),
        dominant_frequency_hz: snapshot_scalar_nonneg(cell.dominant_frequency_hz),
        purity: snapshot_scalar_unit(cell.purity),
        temperature: snapshot_scalar_nonneg(cell.temperature),
        matter_state: cell.matter_state,
        materialized_entity: cell.materialized_entity,
        contributions_fingerprint: frequency_contributions_fingerprint(cell.frequency_contributions()),
    }
}

/// Escalares de snapshot: mismos contratos que lectores del grid (no finito → 0).
#[inline]
fn snapshot_scalar_nonneg(x: f32) -> f32 {
    if x.is_finite() {
        x.max(0.0)
    } else {
        0.0
    }
}

#[inline]
fn snapshot_scalar_unit(x: f32) -> f32 {
    if x.is_finite() {
        x.clamp(0.0, 1.0)
    } else {
        0.0
    }
}

/// Cache lineal `idx = y * width + x` (mismo orden que [`EnergyFieldGrid`]).
/// Válido solo si `synced_generation == grid.generation` (opción A: rebuild completo al bump).
#[derive(Resource, Debug, Default)]
pub struct CellFieldSnapshotCache {
    pub entries: Vec<Option<CellFieldSnapshot>>,
    /// `Some(g)` cuando `entries` refleja el grid tras derive + materialización delta del tick con `generation == g`.
    pub synced_generation: Option<u32>,
}

/// Sincroniza la cache cuando cambia `grid.generation` (tras derive y mutaciones de celdas del mismo tick).
/// Opción A: si cambió `grid.generation`, rebuild completo O(celdas).
pub fn cell_field_snapshot_sync_system(
    grid: Option<Res<EnergyFieldGrid>>,
    mut cache: ResMut<CellFieldSnapshotCache>,
) {
    let Some(grid) = grid else {
        return;
    };
    let len = grid.width as usize * grid.height as usize;
    debug_assert!(len > 0, "EnergyFieldGrid fuerza width,height >= 1");
    if cache.entries.len() != len {
        cache.entries.resize(len, None);
        cache.synced_generation = None;
    }
    if cache.synced_generation == Some(grid.generation) {
        return;
    }
    for (idx, cell) in grid.iter_cells().enumerate() {
        cache.entries[idx] = Some(cell_field_snapshot_from_energy_cell(cell));
    }
    cache.synced_generation = Some(grid.generation);
}

/// Lectura O(1); falla si la cache no está alineada a `grid.generation`.
#[inline]
pub fn cell_field_snapshot_read(
    cache: &CellFieldSnapshotCache,
    grid: &EnergyFieldGrid,
    linear_idx: usize,
) -> Option<CellFieldSnapshot> {
    if cache.synced_generation != Some(grid.generation) {
        return None;
    }
    cache.entries.get(linear_idx).and_then(|slot| *slot)
}

#[cfg(test)]
mod tests {
    use super::constants::{CACHE_OPTION_ENTRY_MAX_BYTES, SNAPSHOT_STRUCT_MAX_BYTES};
    use super::*;
    use bevy::math::Vec2;
    use crate::blueprint::{AlchemicalAlmanac, ElementDef};
    use crate::worldgen::systems::propagation::derive_cell_state_system;

    fn terra_almanac() -> AlchemicalAlmanac {
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

    /// App mínima: almanaque + grid + cache (el test registra la cadena de sistemas).
    fn snapshot_test_app(grid: EnergyFieldGrid) -> App {
        let mut app = App::new();
        app.insert_resource(terra_almanac());
        app.insert_resource(grid);
        app.insert_resource(CellFieldSnapshotCache::default());
        app
    }

    #[test]
    fn fingerprint_order_invariant() {
        let e1 = Entity::from_raw(1);
        let e2 = Entity::from_raw(2);
        let a = [
            FrequencyContribution::new(e1, 10.0, 2.0),
            FrequencyContribution::new(e2, 20.0, 5.0),
        ];
        let b = [
            FrequencyContribution::new(e2, 20.0, 5.0),
            FrequencyContribution::new(e1, 10.0, 2.0),
        ];
        assert_eq!(
            frequency_contributions_fingerprint(&a),
            frequency_contributions_fingerprint(&b)
        );
    }

    #[test]
    fn snapshot_matches_direct_from_cell_after_derive() {
        let mut grid = EnergyFieldGrid::new(2, 2, 1.0, Vec2::ZERO);
        let cell = grid.cell_xy_mut(0, 0).expect("cell");
        cell.accumulated_qe = 12.0;
        cell.push_contribution_bounded(FrequencyContribution::new(
            Entity::from_raw(7),
            75.0,
            8.0,
        ));
        let mut app = snapshot_test_app(grid);
        app.add_systems(
            Update,
            (derive_cell_state_system, cell_field_snapshot_sync_system).chain(),
        );
        app.update();
        let grid = app.world().resource::<EnergyFieldGrid>();
        let cache = app.world().resource::<CellFieldSnapshotCache>();
        assert_eq!(cache.synced_generation, Some(grid.generation));
        let cell = grid.cell_linear(0).expect("c0");
        let expected = cell_field_snapshot_from_energy_cell(cell);
        let got = cell_field_snapshot_read(cache, grid, 0).expect("cached");
        assert_eq!(got, expected);
    }

    #[test]
    fn sync_before_derive_mismatches_derived_cell() {
        let mut grid = EnergyFieldGrid::new(1, 1, 1.0, Vec2::ZERO);
        let cell = grid.cell_xy_mut(0, 0).expect("cell");
        cell.accumulated_qe = 12.0;
        cell.push_contribution_bounded(FrequencyContribution::new(
            Entity::from_raw(3),
            75.0,
            8.0,
        ));
        let mut app = snapshot_test_app(grid);
        app.add_systems(
            Update,
            (cell_field_snapshot_sync_system, derive_cell_state_system).chain(),
        );
        app.update();
        let grid = app.world().resource::<EnergyFieldGrid>();
        let cache = app.world().resource::<CellFieldSnapshotCache>();
        let cell = grid.cell_linear(0).expect("c0");
        let expected_dom = cell.dominant_frequency_hz;
        assert!(expected_dom > 1.0, "derive debe fijar Hz dominante");
        assert!(
            cell_field_snapshot_read(cache, grid, 0).is_none(),
            "generación desalineada: lectura segura debe fallar si sync corrió antes que derive"
        );
        let stale = cache.entries[0].expect("sync escribió slot 0");
        assert!(
            stale.dominant_frequency_hz < 1.0,
            "snapshot previo a derive debe conservar Hz no derivado (got {})",
            stale.dominant_frequency_hz
        );
    }

    #[test]
    fn no_grid_resource_no_panic() {
        let mut app = App::new();
        app.insert_resource(CellFieldSnapshotCache::default());
        app.add_systems(Update, cell_field_snapshot_sync_system);
        app.update();
    }

    #[test]
    fn cache_cleared_recomputes_identical() {
        let mut grid = EnergyFieldGrid::new(1, 1, 1.0, Vec2::ZERO);
        let cell = grid.cell_xy_mut(0, 0).expect("cell");
        cell.accumulated_qe = 5.0;
        cell.push_contribution_bounded(FrequencyContribution::new(
            Entity::from_raw(9),
            75.0,
            4.0,
        ));
        let mut app = snapshot_test_app(grid);
        app.add_systems(
            Update,
            (derive_cell_state_system, cell_field_snapshot_sync_system).chain(),
        );
        app.update();
        let first = {
            let grid = app.world().resource::<EnergyFieldGrid>();
            let cache = app.world().resource::<CellFieldSnapshotCache>();
            cell_field_snapshot_read(cache, grid, 0).expect("snap")
        };
        app.insert_resource(CellFieldSnapshotCache::default());
        app.update();
        let second = {
            let grid = app.world().resource::<EnergyFieldGrid>();
            let cache = app.world().resource::<CellFieldSnapshotCache>();
            cell_field_snapshot_read(cache, grid, 0).expect("snap2")
        };
        assert_eq!(first, second);
    }

    #[test]
    fn snapshot_cache_layout_documented() {
        use std::mem::size_of;
        assert!(
            size_of::<CellFieldSnapshot>() <= SNAPSHOT_STRUCT_MAX_BYTES,
            "CellFieldSnapshot debe mantenerse compacto; revisar EPI2/EPI3"
        );
        assert!(
            size_of::<Option<CellFieldSnapshot>>() <= CACHE_OPTION_ENTRY_MAX_BYTES,
            "Option<CellFieldSnapshot> en cache denso"
        );
    }

    #[test]
    fn grid_resize_resyncs_cache_length() {
        let mut app = snapshot_test_app(EnergyFieldGrid::new(2, 2, 1.0, Vec2::ZERO));
        app.add_systems(
            Update,
            (derive_cell_state_system, cell_field_snapshot_sync_system).chain(),
        );
        app.update();
        {
            let grid = app.world().resource::<EnergyFieldGrid>();
            let cache = app.world().resource::<CellFieldSnapshotCache>();
            assert_eq!(cache.entries.len(), 4);
            assert_eq!(cache.synced_generation, Some(grid.generation));
        }
        app.insert_resource(EnergyFieldGrid::new(3, 2, 1.0, Vec2::ZERO));
        app.update();
        let grid = app.world().resource::<EnergyFieldGrid>();
        let cache = app.world().resource::<CellFieldSnapshotCache>();
        assert_eq!(cache.entries.len(), 6);
        assert_eq!(cache.synced_generation, Some(grid.generation));
        assert!(cell_field_snapshot_read(cache, grid, 5).is_some());
    }

    #[test]
    fn second_tick_refreshes_when_generation_bumps() {
        let mut grid = EnergyFieldGrid::new(1, 1, 1.0, Vec2::ZERO);
        {
            let c = grid.cell_xy_mut(0, 0).expect("c");
            c.accumulated_qe = 4.0;
            c.push_contribution_bounded(FrequencyContribution::new(
                Entity::from_raw(1),
                75.0,
                3.0,
            ));
        }
        let mut app = snapshot_test_app(grid);
        app.add_systems(
            Update,
            (derive_cell_state_system, cell_field_snapshot_sync_system).chain(),
        );
        app.update();
        let q1 = {
            let grid = app.world().resource::<EnergyFieldGrid>();
            let cache = app.world().resource::<CellFieldSnapshotCache>();
            assert_eq!(cache.synced_generation, Some(grid.generation));
            cell_field_snapshot_read(cache, grid, 0)
                .expect("read1")
                .accumulated_qe
        };
        {
            let mut grid = app.world_mut().resource_mut::<EnergyFieldGrid>();
            let c = grid.cell_xy_mut(0, 0).expect("c");
            c.accumulated_qe = 40.0;
            grid.mark_cell_dirty(0, 0);
        }
        app.update();
        let q2 = {
            let grid = app.world().resource::<EnergyFieldGrid>();
            let cache = app.world().resource::<CellFieldSnapshotCache>();
            assert_eq!(cache.synced_generation, Some(grid.generation));
            cell_field_snapshot_read(cache, grid, 0)
                .expect("read2")
                .accumulated_qe
        };
        assert!(q2 > q1);
    }

    #[test]
    fn fingerprint_distinguishes_entity_generation() {
        let mut world = World::new();
        let e0 = world.spawn_empty().id();
        world.despawn(e0);
        let e1 = world.spawn_empty().id();
        assert_eq!(e0.index(), e1.index(), "setup: reuso de índice tras despawn");
        assert_ne!(
            e0.generation(),
            e1.generation(),
            "setup: generación distinta en mismo slot"
        );
        let fa =
            frequency_contributions_fingerprint(&[FrequencyContribution::new(e0, 50.0, 2.0)]);
        let fb =
            frequency_contributions_fingerprint(&[FrequencyContribution::new(e1, 50.0, 2.0)]);
        assert_ne!(fa, fb);
    }

    #[test]
    fn non_finite_scalars_sanitized_in_snapshot() {
        let mut cell = EnergyCell::default();
        cell.accumulated_qe = f32::NAN;
        cell.dominant_frequency_hz = f32::INFINITY;
        cell.purity = f32::NEG_INFINITY;
        let s = cell_field_snapshot_from_energy_cell(&cell);
        assert_eq!(s.accumulated_qe, 0.0);
        assert_eq!(s.dominant_frequency_hz, 0.0);
        assert_eq!(s.purity, 0.0);
    }

    /// EPI4: misma semántica que `gf1_field_linear_rgb_qe_at_position` para `qe_norm` (celda fija).
    #[test]
    fn fixed_cell_palette_index_from_snapshot_matches_enorm_formula() {
        use crate::blueprint::constants::VISUAL_QE_REFERENCE;
        use crate::blueprint::equations::quantized_palette_index;

        let mut grid = EnergyFieldGrid::new(1, 1, 1.0, Vec2::ZERO);
        {
            let c = grid.cell_xy_mut(0, 0).expect("c");
            c.accumulated_qe = 52.0;
            c.push_contribution_bounded(FrequencyContribution::new(
                Entity::from_raw(1),
                75.0,
                5.0,
            ));
        }
        let mut app = snapshot_test_app(grid);
        app.add_systems(
            Update,
            (derive_cell_state_system, cell_field_snapshot_sync_system).chain(),
        );
        app.update();
        let grid = app.world().resource::<EnergyFieldGrid>();
        let cache = app.world().resource::<CellFieldSnapshotCache>();
        let snap = cell_field_snapshot_read(cache, grid, 0).expect("snap");
        let qe_ref = VISUAL_QE_REFERENCE.max(1.0);
        let enorm = (snap.accumulated_qe / qe_ref).clamp(0.0, 1.0);
        let rho = 0.41_f32;
        let n_max = 88_u32;
        let idx = quantized_palette_index(enorm, rho, n_max);
        let idx_direct = quantized_palette_index(
            (52.0_f32 / qe_ref).clamp(0.0, 1.0),
            rho,
            n_max,
        );
        assert_eq!(idx, idx_direct);
    }
}
