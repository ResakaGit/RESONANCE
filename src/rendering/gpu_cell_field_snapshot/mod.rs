//! EPI4 — SSBO opcional del snapshot celular (CPU → GPU), feature `gpu_cell_field_snapshot`.
//! No escribe simulación; solo copia proyección ya validada en [`CellFieldSnapshotCache`].

mod plugin;

pub use plugin::GpuCellFieldSnapshotPlugin;
