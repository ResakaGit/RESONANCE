use crate::math_types::Vec2;
use bevy::prelude::{Entity, Reflect, Resource};
use serde::{Deserialize, Serialize};

use crate::worldgen::contracts::Materialized;
use crate::worldgen::EnergyCell;

pub use crate::worldgen::constants::FIELD_GRID_CHUNK_SIZE;

/// Grid denso de energía del mundo (estado global, no componente).
#[derive(Clone, Debug, Resource, Reflect, Serialize, Deserialize)]
pub struct EnergyFieldGrid {
    pub width: u32,
    pub height: u32,
    pub cell_size: f32,
    pub origin: Vec2,
    /// Se incrementa cuando `derive_cell_state_system` observa cambios derivados (temperatura, fase, Hz).
    pub generation: u32,
    cells: Vec<EnergyCell>,
    /// Bit por celda: necesita revisión de materialización / trabajo incremental.
    #[serde(skip)]
    dirty_words: Vec<u64>,
    /// 1 si alguna celda del chunk 16×16 está dirty (acelera skip).
    #[serde(skip)]
    chunk_dirty: Vec<u8>,
}

impl EnergyFieldGrid {
    pub fn new(width: u32, height: u32, cell_size: f32, origin: Vec2) -> Self {
        let width = width.max(1);
        let height = height.max(1);
        let cell_size = if cell_size.is_finite() {
            cell_size.max(0.001)
        } else {
            1.0
        };
        let len = width as usize * height as usize;
        let cells = vec![EnergyCell::default(); len];
        let dirty_words = vec![0u64; dirty_word_count(len)];
        let chunk_dirty = vec![0u8; chunk_linear_len(width, height)];
        Self {
            width,
            height,
            cell_size,
            origin,
            generation: 0,
            cells,
            dirty_words,
            chunk_dirty,
        }
    }

    /// Bytes aproximados del resource (grid + bitsets), sin heap interno de `Vec` en cada `EnergyCell`.
    pub fn approx_footprint_bytes(&self) -> usize {
        let cells_heap: usize = self
            .cells
            .iter()
            .map(|c| {
                c.frequency_contributions.capacity()
                    * std::mem::size_of::<crate::worldgen::FrequencyContribution>()
            })
            .sum();
        std::mem::size_of::<Self>()
            + self.cells.len() * std::mem::size_of::<EnergyCell>()
            + self.dirty_words.len() * std::mem::size_of::<u64>()
            + self.chunk_dirty.len()
            + cells_heap
    }

    #[inline]
    pub fn clear_dirty(&mut self) {
        self.dirty_words.fill(0);
        self.chunk_dirty.fill(0);
    }

    /// Marca celda dirty y el chunk que la contiene.
    pub fn mark_cell_dirty(&mut self, x: u32, y: u32) {
        let Some(idx) = self.index_of(x, y) else {
            return;
        };
        let word = idx / 64;
        let bit = idx % 64;
        if let Some(w) = self.dirty_words.get_mut(word) {
            *w |= 1u64 << bit;
        }
        let cx = x / FIELD_GRID_CHUNK_SIZE;
        let cy = y / FIELD_GRID_CHUNK_SIZE;
        let chunks_w = chunk_w(self.width);
        let cidx = cy as usize * chunks_w as usize + cx as usize;
        if let Some(c) = self.chunk_dirty.get_mut(cidx) {
            *c = 1;
        }
    }

    #[inline]
    pub fn is_cell_dirty(&self, x: u32, y: u32) -> bool {
        let Some(idx) = self.index_of(x, y) else {
            return false;
        };
        let word = idx / 64;
        let bit = idx % 64;
        self.dirty_words
            .get(word)
            .is_some_and(|w| (*w & (1u64 << bit)) != 0)
    }

    #[inline]
    pub fn any_dirty(&self) -> bool {
        self.dirty_words.iter().any(|w| *w != 0)
    }

    /// Drena hasta `budget` índices dirty, limpia sus bits y retorna los índices.
    pub fn drain_dirty_budgeted(&mut self, budget: usize) -> impl Iterator<Item = usize> {
        let mut result = Vec::with_capacity(budget);
        let total_cells = (self.width as usize) * (self.height as usize);
        'outer: for (wi, word) in self.dirty_words.iter_mut().enumerate() {
            if *word == 0 { continue; }
            let mut w = *word;
            while w != 0 {
                let bit = w.trailing_zeros() as usize;
                let idx = wi * 64 + bit;
                if idx >= total_cells { break; }
                result.push(idx);
                w &= !(1u64 << bit);
                if result.len() >= budget {
                    *word = w;
                    break 'outer;
                }
            }
            *word = w;
        }
        result.into_iter()
    }

    /// Si el chunk no está dirty, se puede saltar trabajo O(chunk) en iteradores.
    #[inline]
    pub fn is_chunk_dirty(&self, chunk_x: u32, chunk_y: u32) -> bool {
        let chunks_w = chunk_w(self.width);
        let idx = chunk_y as usize * chunks_w as usize + chunk_x as usize;
        self.chunk_dirty.get(idx).is_some_and(|v| *v != 0)
    }

    pub fn iter_cells(&self) -> impl Iterator<Item = &EnergyCell> {
        self.cells.iter()
    }

    pub fn iter_cells_mut(&mut self) -> impl Iterator<Item = &mut EnergyCell> {
        self.cells.iter_mut()
    }

    pub fn cell_coords(&self, world_pos: Vec2) -> Option<(u32, u32)> {
        if !world_pos.is_finite() {
            return None;
        }
        let rel = world_pos - self.origin;
        if rel.x < 0.0 || rel.y < 0.0 {
            return None;
        }
        let x = (rel.x / self.cell_size).floor() as i32;
        let y = (rel.y / self.cell_size).floor() as i32;
        if x < 0 || y < 0 || x >= self.width as i32 || y >= self.height as i32 {
            return None;
        }
        Some((x as u32, y as u32))
    }

    pub fn world_pos(&self, cell_x: u32, cell_y: u32) -> Option<Vec2> {
        if cell_x >= self.width || cell_y >= self.height {
            return None;
        }
        let x = self.origin.x + (cell_x as f32 + 0.5) * self.cell_size;
        let y = self.origin.y + (cell_y as f32 + 0.5) * self.cell_size;
        Some(Vec2::new(x, y))
    }

    pub fn cell_at(&self, world_pos: Vec2) -> Option<&EnergyCell> {
        let (x, y) = self.cell_coords(world_pos)?;
        self.cell_xy(x, y)
    }

    pub fn cell_at_mut(&mut self, world_pos: Vec2) -> Option<&mut EnergyCell> {
        let (x, y) = self.cell_coords(world_pos)?;
        self.cell_xy_mut(x, y)
    }

    /// Índice lineal row-major (`y * width + x`) si [`Materialized`] cae dentro del grid.
    #[inline]
    pub fn linear_index_for_materialized(&self, mat: &Materialized) -> Option<usize> {
        if mat.cell_x < 0 || mat.cell_y < 0 {
            return None;
        }
        self.index_of(mat.cell_x as u32, mat.cell_y as u32)
    }

    pub fn cell_xy(&self, x: u32, y: u32) -> Option<&EnergyCell> {
        let idx = self.index_of(x, y)?;
        self.cells.get(idx)
    }

    pub fn cell_xy_mut(&mut self, x: u32, y: u32) -> Option<&mut EnergyCell> {
        let idx = self.index_of(x, y)?;
        self.cells.get_mut(idx)
    }

    /// Celda por índice lineal **row-major** `idx = y * width + x` (mismo orden que `index_of`).
    #[inline]
    pub fn cell_linear(&self, idx: usize) -> Option<&EnergyCell> {
        self.cells.get(idx)
    }

    /// 4-neighbors with toroidal wrapping (planetary surface has no edges).
    pub fn neighbors4(&self, x: u32, y: u32) -> [Option<(u32, u32)>; 4] {
        let w = self.width;
        let h = self.height;
        [
            Some(((x + w - 1) % w, y)),
            Some(((x + 1) % w, y)),
            Some((x, (y + h - 1) % h)),
            Some((x, (y + 1) % h)),
        ]
    }

    /// Seeds all cells with uniform energy and a frequency contribution.
    /// Used for Big Bang scenarios: energy uniformly distributed, nuclei emerge later.
    /// The frequency determines the initial elemental identity of the field.
    pub fn seed_uniform(&mut self, qe: f32, frequency_hz: f32) {
        let qe = if qe.is_finite() { qe.max(0.0) } else { return };
        let freq = if frequency_hz.is_finite() { frequency_hz.max(0.0) } else { return };
        let seed_entity = Entity::from_raw(u32::MAX); // placeholder, not a real entity
        for y in 0..self.height {
            for x in 0..self.width {
                if let Some(cell) = self.cell_xy_mut(x, y) {
                    cell.accumulated_qe = qe;
                    cell.push_contribution_bounded(
                        crate::worldgen::FrequencyContribution::new(seed_entity, freq, qe),
                    );
                }
                self.mark_cell_dirty(x, y);
            }
        }
    }

    pub fn clear_frequency_contributions(&mut self) {
        for cell in &mut self.cells {
            cell.frequency_contributions.clear();
        }
    }

    pub fn total_qe(&self) -> f32 {
        self.cells
            .iter()
            .map(|cell| cell.accumulated_qe.max(0.0))
            .sum::<f32>()
    }

    /// `accumulated_qe` de la celda en posición mundo `(x, z)`. 0.0 si fuera del grid.
    /// Convenio de eje: `x` → columna, `z` → fila (coordenadas 3D world-space).
    pub fn cell_qe_at_world(&self, x: f32, z: f32) -> f32 {
        self.cell_at(Vec2::new(x, z))
            .map(|c| c.accumulated_qe.max(0.0))
            .unwrap_or(0.0)
    }

    /// Índice lineal row-major de la celda bajo la posición mundo `(x, z)`.
    /// Retorna `u32::MAX` si fuera del grid (valor centinela — comprobar antes de usar).
    pub fn world_to_cell_idx(&self, x: f32, z: f32) -> u32 {
        let Some((cx, cy)) = self.cell_coords(Vec2::new(x, z)) else {
            return u32::MAX;
        };
        cy * self.width + cx
    }

    /// `accumulated_qe` por índice lineal. 0.0 si fuera de rango.
    #[inline]
    pub fn cell_qe(&self, idx: usize) -> f32 {
        self.cells.get(idx).map(|c| c.accumulated_qe.max(0.0)).unwrap_or(0.0)
    }

    /// Aplica `delta` de drenaje sobre la celda `idx` (positivo = drenar, negativo = añadir).
    /// Clampea `accumulated_qe` a `[0, ∞)` y marca la celda dirty.
    pub fn drain_cell(&mut self, idx: u32, delta: f32) {
        let i = idx as usize;
        if let Some(cell) = self.cells.get_mut(i) {
            cell.accumulated_qe = (cell.accumulated_qe - delta).max(0.0);
            let x = (i % self.width as usize) as u32;
            let y = (i / self.width as usize) as u32;
            self.mark_cell_dirty(x, y);
        }
    }

    fn index_of(&self, x: u32, y: u32) -> Option<usize> {
        if x >= self.width || y >= self.height {
            return None;
        }
        Some(y as usize * self.width as usize + x as usize)
    }
}

fn dirty_word_count(cell_len: usize) -> usize {
    cell_len.div_ceil(64)
}

fn chunk_w(width: u32) -> u32 {
    width.div_ceil(FIELD_GRID_CHUNK_SIZE)
}

fn chunk_h(height: u32) -> u32 {
    height.div_ceil(FIELD_GRID_CHUNK_SIZE)
}

fn chunk_linear_len(width: u32, height: u32) -> usize {
    (chunk_w(width) * chunk_h(height)) as usize
}

#[cfg(test)]
mod tests {
    use super::EnergyFieldGrid;
    use bevy::math::Vec2;

    #[test]
    fn field_grid_cell_at_outside_returns_none() {
        let grid = EnergyFieldGrid::new(10, 10, 1.0, Vec2::ZERO);
        assert!(grid.cell_at(Vec2::new(-1.0, 1.0)).is_none());
        assert!(grid.cell_at(Vec2::new(10.1, 1.0)).is_none());
        assert!(grid.cell_at(Vec2::new(1.0, 10.1)).is_none());
    }

    #[test]
    fn field_grid_world_pos_roundtrip_inside_bounds() {
        let grid = EnergyFieldGrid::new(8, 8, 2.0, Vec2::new(-8.0, -8.0));
        let pos = grid.world_pos(3, 4).expect("valid cell");
        let coords = grid.cell_coords(pos).expect("valid position");
        assert_eq!(coords, (3, 4));
    }

    #[test]
    fn field_grid_neighbors4_only_returns_valid_coords() {
        let grid = EnergyFieldGrid::new(3, 3, 1.0, Vec2::ZERO);
        let center = grid.neighbors4(1, 1);
        assert_eq!(center.iter().flatten().count(), 4);
        let corner = grid.neighbors4(0, 0);
        assert_eq!(corner.iter().flatten().count(), 2);
    }

    #[test]
    fn field_grid_clear_dirty_clears_all_flags() {
        let mut grid = EnergyFieldGrid::new(4, 4, 1.0, Vec2::ZERO);
        grid.mark_cell_dirty(1, 2);
        assert!(grid.is_cell_dirty(1, 2));
        grid.clear_dirty();
        assert!(!grid.any_dirty());
        assert!(!grid.is_cell_dirty(1, 2));
    }

    #[test]
    fn field_grid_200x200_footprint_under_10mb() {
        let grid = EnergyFieldGrid::new(200, 200, 1.0, Vec2::ZERO);
        let bytes = grid.approx_footprint_bytes();
        const LIMIT: usize = 10 * 1024 * 1024;
        assert!(
            bytes < LIMIT,
            "approximate footprint {bytes} bytes exceeds {LIMIT}"
        );
    }

    #[test]
    fn linear_index_for_materialized_row_major() {
        use crate::worldgen::contracts::Materialized;
        use crate::worldgen::WorldArchetype;

        let grid = EnergyFieldGrid::new(4, 4, 2.0, Vec2::ZERO);
        let ok = Materialized {
            cell_x: 2,
            cell_y: 1,
            archetype: WorldArchetype::TerraSolid,
        };
        assert_eq!(grid.linear_index_for_materialized(&ok), Some(6));

        let bad = Materialized {
            cell_x: 4,
            cell_y: 0,
            archetype: WorldArchetype::TerraSolid,
        };
        assert_eq!(grid.linear_index_for_materialized(&bad), None);
    }
}
