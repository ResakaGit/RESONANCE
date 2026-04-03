//! EPI4 — Layout binario GPU ↔ CPU del snapshot celular (SSBO).
//!
//! **Fuente de verdad:** [`super::CellFieldSnapshot`] en CPU; estos tipos son copia empaquetada para WGSL.
//! **Versión:** incrementar [`CELL_FIELD_SNAPSHOT_GPU_SCHEMA_VERSION`] ante cualquier cambio de campo u orden.
//!
//! Alineación: fila por celda = [`GPU_CELL_FIELD_ROW_BYTES`] bytes (múltiplo de 16 para arrays WGSL).
//! Ver `assets/shaders/cell_field_snapshot.wgsl` — debe mantenerse 1:1 con los `struct` de este módulo.

use bevy::prelude::Entity;

use crate::layers::MatterState;

use super::CellFieldSnapshot;
use super::constants::{
    GPU_MATERIALIZED_ABSENT, GPU_MATERIALIZED_PRESENT, GPU_MATTER_STATE_GAS,
    GPU_MATTER_STATE_LIQUID, GPU_MATTER_STATE_PLASMA, GPU_MATTER_STATE_SOLID,
};

/// Versión del esquema SSBO; subir en cada breaking change del layout (WGSL + Rust).
pub const CELL_FIELD_SNAPSHOT_GPU_SCHEMA_VERSION: u32 = 1;

/// Ruta del WGSL desde la raíz del crate (tests / documentación; debe coincidir con el archivo real).
pub const CELL_FIELD_SNAPSHOT_WGSL_PATH: &str = "assets/shaders/cell_field_snapshot.wgsl";

/// Cabecera fija al inicio del buffer de storage (16 B).
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct GpuCellFieldSnapshotHeader {
    pub snapshot_schema_version: u32,
    pub grid_width: u32,
    pub grid_height: u32,
    pub grid_generation: u32,
}

/// Celda empaquetada para GPU (48 B stride). Orden y tamaño = `cell_field_snapshot.wgsl` (`GpuCellFieldPacked`).
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Default)]
pub struct GpuCellFieldPacked {
    pub accumulated_qe: f32,
    pub dominant_frequency_hz: f32,
    pub purity: f32,
    pub temperature: f32,
    /// Discriminante estable: 0 = Solid, 1 = Liquid, 2 = Gas, 3 = Plasma (orden del enum Rust).
    pub matter_state: u32,
    pub materialized_present: u32,
    pub materialized_index: u32,
    pub materialized_generation: u32,
    pub contributions_fingerprint: u32,
    pub _pad: [u32; 3],
}

/// Tamaño en bytes de una fila/celda en el SSBO (incluye padding final).
pub const GPU_CELL_FIELD_ROW_BYTES: usize = size_of::<GpuCellFieldPacked>();

/// Tamaño de la cabecera [`GpuCellFieldSnapshotHeader`].
pub const GPU_SNAPSHOT_HEADER_BYTES: usize = size_of::<GpuCellFieldSnapshotHeader>();

const _: () = assert!(GPU_SNAPSHOT_HEADER_BYTES == 16);
const _: () = assert!(GPU_CELL_FIELD_ROW_BYTES == 48);
const _: () = assert!(GPU_CELL_FIELD_ROW_BYTES % 16 == 0);

/// Cabecera inicial del SSBO (grid vacío hasta el primer upload válido).
#[inline]
pub fn initial_gpu_cell_field_snapshot_header() -> GpuCellFieldSnapshotHeader {
    GpuCellFieldSnapshotHeader {
        snapshot_schema_version: CELL_FIELD_SNAPSHOT_GPU_SCHEMA_VERSION,
        grid_width: 0,
        grid_height: 0,
        grid_generation: 0,
    }
}

/// Convierte la cache lineal EPI1 en filas GPU.
///
/// Con cache sincronizada, todas las entradas son `Some`; `None` → fila cero (release) y `debug_assert` en dev.
#[inline]
pub fn gpu_packed_rows_from_cache_entries(
    entries: &[Option<CellFieldSnapshot>],
) -> Vec<GpuCellFieldPacked> {
    debug_assert!(
        !entries.iter().any(|s| s.is_none()),
        "EPI1: cache alineada a `grid.generation` debe tener todas las celdas rellenas"
    );
    entries
        .iter()
        .map(|slot| {
            slot.as_ref()
                .map(cell_field_snapshot_to_gpu_packed)
                .unwrap_or_default()
        })
        .collect()
}

#[inline]
fn matter_state_discriminant(state: MatterState) -> u32 {
    match state {
        MatterState::Solid => GPU_MATTER_STATE_SOLID,
        MatterState::Liquid => GPU_MATTER_STATE_LIQUID,
        MatterState::Gas => GPU_MATTER_STATE_GAS,
        MatterState::Plasma => GPU_MATTER_STATE_PLASMA,
    }
}

#[inline]
fn materialized_entity_gpu(e: Option<Entity>) -> (u32, u32, u32) {
    match e {
        Some(ent) => (GPU_MATERIALIZED_PRESENT, ent.index(), ent.generation()),
        None => (GPU_MATERIALIZED_ABSENT, 0, 0),
    }
}

/// Proyección pura: snapshot lógico → fila GPU (sin I/O).
#[inline]
pub fn cell_field_snapshot_to_gpu_packed(s: &CellFieldSnapshot) -> GpuCellFieldPacked {
    let (present, idx, ent_gen) = materialized_entity_gpu(s.materialized_entity);
    GpuCellFieldPacked {
        accumulated_qe: s.accumulated_qe,
        dominant_frequency_hz: s.dominant_frequency_hz,
        purity: s.purity,
        temperature: s.temperature,
        matter_state: matter_state_discriminant(s.matter_state),
        materialized_present: present,
        materialized_index: idx,
        materialized_generation: ent_gen,
        contributions_fingerprint: s.contributions_fingerprint,
        _pad: [0; 3],
    }
}

/// Serializa cabecera + celdas en orden lineal `idx = y * width + x` (igual que [`super::CellFieldSnapshotCache`]).
pub fn gpu_cell_field_snapshot_bytes(
    header: GpuCellFieldSnapshotHeader,
    cells: &[GpuCellFieldPacked],
) -> Vec<u8> {
    let mut out =
        Vec::with_capacity(GPU_SNAPSHOT_HEADER_BYTES + GPU_CELL_FIELD_ROW_BYTES * cells.len());
    out.extend_from_slice(bytemuck::bytes_of(&header));
    out.extend_from_slice(bytemuck::cast_slice(cells));
    out
}

// --- bytemuck: solo campos Pod (sin padding no inicializado problemático; _pad = 0).
unsafe impl bytemuck::Pod for GpuCellFieldSnapshotHeader {}
unsafe impl bytemuck::Zeroable for GpuCellFieldSnapshotHeader {}
unsafe impl bytemuck::Pod for GpuCellFieldPacked {}
unsafe impl bytemuck::Zeroable for GpuCellFieldPacked {}

#[cfg(test)]
mod tests {
    use super::super::constants::{
        GPU_MATTER_STATE_GAS, GPU_MATTER_STATE_LIQUID, GPU_MATTER_STATE_PLASMA,
        GPU_MATTER_STATE_SOLID,
    };
    use super::*;
    use bevy::prelude::Entity;

    #[test]
    fn layout_sizes_match_compile_time_contract() {
        assert_eq!(GPU_SNAPSHOT_HEADER_BYTES, 16);
        assert_eq!(GPU_CELL_FIELD_ROW_BYTES, 48);
    }

    #[test]
    fn initial_header_carries_schema_version() {
        let h = initial_gpu_cell_field_snapshot_header();
        assert_eq!(
            h.snapshot_schema_version,
            CELL_FIELD_SNAPSHOT_GPU_SCHEMA_VERSION
        );
        assert_eq!(h.grid_width, 0);
        assert_eq!(h.grid_generation, 0);
    }

    #[test]
    fn matter_state_discriminants_match_wgsl_contract() {
        use MatterState::*;
        let s = CellFieldSnapshot {
            accumulated_qe: 0.0,
            dominant_frequency_hz: 0.0,
            purity: 0.0,
            temperature: 0.0,
            matter_state: Solid,
            materialized_entity: None,
            contributions_fingerprint: 0,
        };
        assert_eq!(
            cell_field_snapshot_to_gpu_packed(&s).matter_state,
            GPU_MATTER_STATE_SOLID
        );
        let mut s2 = s;
        s2.matter_state = Liquid;
        assert_eq!(
            cell_field_snapshot_to_gpu_packed(&s2).matter_state,
            GPU_MATTER_STATE_LIQUID
        );
        s2.matter_state = Gas;
        assert_eq!(
            cell_field_snapshot_to_gpu_packed(&s2).matter_state,
            GPU_MATTER_STATE_GAS
        );
        s2.matter_state = Plasma;
        assert_eq!(
            cell_field_snapshot_to_gpu_packed(&s2).matter_state,
            GPU_MATTER_STATE_PLASMA
        );
    }

    #[test]
    fn packed_roundtrip_matches_snapshot_fields() {
        let s = CellFieldSnapshot {
            accumulated_qe: 1.25,
            dominant_frequency_hz: 75.0,
            purity: 0.9,
            temperature: 300.0,
            matter_state: MatterState::Liquid,
            materialized_entity: Some(Entity::from_raw(42)),
            contributions_fingerprint: 0xdead_beef,
        };
        let p = cell_field_snapshot_to_gpu_packed(&s);
        assert_eq!(p.accumulated_qe, 1.25);
        assert_eq!(p.dominant_frequency_hz, 75.0);
        assert_eq!(p.purity, 0.9);
        assert_eq!(p.temperature, 300.0);
        assert_eq!(p.matter_state, 1);
        assert_eq!(p.materialized_present, 1);
        assert_eq!(p.materialized_index, Entity::from_raw(42).index());
        assert_eq!(p.materialized_generation, Entity::from_raw(42).generation());
        assert_eq!(p.contributions_fingerprint, 0xdead_beef);
    }

    #[test]
    fn none_materialized_zeroes_entity_slots() {
        let s = CellFieldSnapshot {
            accumulated_qe: 0.0,
            dominant_frequency_hz: 0.0,
            purity: 0.0,
            temperature: 0.0,
            matter_state: MatterState::Solid,
            materialized_entity: None,
            contributions_fingerprint: 0,
        };
        let p = cell_field_snapshot_to_gpu_packed(&s);
        assert_eq!(p.materialized_present, 0);
        assert_eq!(p.materialized_index, 0);
        assert_eq!(p.materialized_generation, 0);
    }

    #[test]
    fn gpu_bytes_len_matches_grid() {
        let h = GpuCellFieldSnapshotHeader {
            snapshot_schema_version: CELL_FIELD_SNAPSHOT_GPU_SCHEMA_VERSION,
            grid_width: 2,
            grid_height: 2,
            grid_generation: 7,
        };
        let cells = [GpuCellFieldPacked::default(); 4];
        let v = gpu_cell_field_snapshot_bytes(h, &cells);
        assert_eq!(
            v.len(),
            GPU_SNAPSHOT_HEADER_BYTES + 4 * GPU_CELL_FIELD_ROW_BYTES
        );
    }
}
