//! Plugin: sube bytes SSBO cuando `CellFieldSnapshotCache.synced_generation == EnergyFieldGrid.generation`.

use bevy::prelude::*;
use bevy::render::render_asset::RenderAssetUsages;
use bevy::render::storage::ShaderStorageBuffer;

use crate::worldgen::cell_field_snapshot::gpu_layout::{
    CELL_FIELD_SNAPSHOT_GPU_SCHEMA_VERSION, GpuCellFieldSnapshotHeader,
    gpu_cell_field_snapshot_bytes, gpu_packed_rows_from_cache_entries,
    initial_gpu_cell_field_snapshot_header,
};
use crate::worldgen::{CellFieldSnapshotCache, EnergyFieldGrid};

/// Handle al [`ShaderStorageBuffer`] con layout documentado en `gpu_layout` + `cell_field_snapshot.wgsl`.
#[derive(Resource, Debug, Clone)]
pub struct GpuCellFieldSnapshotBuffer(pub Handle<ShaderStorageBuffer>);

#[derive(Resource, Debug, Default)]
struct GpuCellFieldSnapshotUploadMeta {
    last_uploaded_generation: Option<u32>,
    last_grid_width: u32,
    last_grid_height: u32,
}

impl GpuCellFieldSnapshotUploadMeta {
    fn matches_current_grid(&self, grid: &EnergyFieldGrid) -> bool {
        self.last_uploaded_generation == Some(grid.generation)
            && self.last_grid_width == grid.width
            && self.last_grid_height == grid.height
    }

    fn record_upload(&mut self, grid: &EnergyFieldGrid) {
        self.last_uploaded_generation = Some(grid.generation);
        self.last_grid_width = grid.width;
        self.last_grid_height = grid.height;
    }
}

fn init_gpu_cell_field_snapshot_buffer(
    mut commands: Commands,
    mut assets: ResMut<Assets<ShaderStorageBuffer>>,
) {
    let initial = gpu_cell_field_snapshot_bytes(initial_gpu_cell_field_snapshot_header(), &[]);
    let handle = assets.add(ShaderStorageBuffer::new(
        &initial,
        RenderAssetUsages::default(),
    ));
    commands.insert_resource(GpuCellFieldSnapshotBuffer(handle));
    commands.init_resource::<GpuCellFieldSnapshotUploadMeta>();
}

fn build_header_for_grid(grid: &EnergyFieldGrid) -> GpuCellFieldSnapshotHeader {
    GpuCellFieldSnapshotHeader {
        snapshot_schema_version: CELL_FIELD_SNAPSHOT_GPU_SCHEMA_VERSION,
        grid_width: grid.width,
        grid_height: grid.height,
        grid_generation: grid.generation,
    }
}

/// Copia snapshot al SSBO solo si la cache EPI1 está alineada al grid (evita divergencia CPU/GPU).
fn gpu_cell_field_snapshot_upload_system(
    grid: Option<Res<EnergyFieldGrid>>,
    cache: Option<Res<CellFieldSnapshotCache>>,
    buffer: Option<Res<GpuCellFieldSnapshotBuffer>>,
    mut meta: ResMut<GpuCellFieldSnapshotUploadMeta>,
    mut assets: ResMut<Assets<ShaderStorageBuffer>>,
) {
    let (Some(grid), Some(cache), Some(buf)) = (grid, cache, buffer) else {
        return;
    };
    if cache.synced_generation != Some(grid.generation) {
        return;
    }
    if meta.matches_current_grid(&grid) {
        return;
    }

    let cells = gpu_packed_rows_from_cache_entries(&cache.entries);
    let bytes = gpu_cell_field_snapshot_bytes(build_header_for_grid(&grid), &cells);
    let Some(ssbo) = assets.get_mut(&buf.0) else {
        return;
    };
    *ssbo = ShaderStorageBuffer::new(&bytes, ssbo.asset_usage);
    meta.record_upload(&grid);
}

pub struct GpuCellFieldSnapshotPlugin;

impl Plugin for GpuCellFieldSnapshotPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, init_gpu_cell_field_snapshot_buffer)
            .add_systems(Update, gpu_cell_field_snapshot_upload_system);
    }
}
