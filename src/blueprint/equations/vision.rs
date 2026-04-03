//! Vision — line-of-sight geometry (pure math).
//!
//! Raycast discreto sobre terreno para visibilidad.
//! Discrete raycast over terrain for line-of-sight.

use crate::math_types::Vec2;
use crate::topology::TerrainField;

/// Raycast discreto sobre celdas entre dos puntos para decidir bloqueo de visión por relieve.
/// Interpola altitud linealmente; si alguna celda intermedia supera la línea, retorna true.
///
/// Discrete raycast: checks if terrain blocks line of sight between two world positions.
pub fn terrain_blocks_vision(from: Vec2, to: Vec2, terrain: &TerrainField) -> bool {
    let Some((fx, fy)) = terrain.world_to_cell(from) else {
        return true;
    };
    let Some((tx, ty)) = terrain.world_to_cell(to) else {
        return true;
    };
    if fx == tx && fy == ty {
        return false;
    }

    let from_alt = terrain.sample_at(fx, fy).altitude;
    let to_alt = terrain.sample_at(tx, ty).altitude;
    let line = raycast_cells_exclusive((fx, fy), (tx, ty));
    if line.is_empty() {
        return false;
    }
    let total_steps = (line.len() + 1) as f32;

    for (step, (x, y)) in line.iter().enumerate() {
        let t = (step as f32 + 1.0) / total_steps;
        let expected_alt = from_alt + (to_alt - from_alt) * t;
        if terrain.sample_at(*x, *y).altitude > expected_alt {
            return true;
        }
    }

    false
}

/// Supercover line: todas las celdas entre dos puntos de grid (exclusivo endpoints).
fn raycast_cells_exclusive(from: (u32, u32), to: (u32, u32)) -> Vec<(u32, u32)> {
    let from_v = Vec2::new(from.0 as f32 + 0.5, from.1 as f32 + 0.5);
    let to_v = Vec2::new(to.0 as f32 + 0.5, to.1 as f32 + 0.5);
    let delta = to_v - from_v;
    let span = delta.x.abs().max(delta.y.abs());
    if span <= f32::EPSILON {
        return Vec::new();
    }

    let steps = (span.ceil() as usize).saturating_mul(2);
    let mut out = Vec::new();

    for i in 1..steps {
        let t = i as f32 / steps as f32;
        let p = from_v + delta * t;
        let cx = p.x.floor().max(0.0) as u32;
        let cy = p.y.floor().max(0.0) as u32;
        let cell = (cx, cy);
        if cell != from && cell != to && out.last().copied() != Some(cell) {
            out.push(cell);
        }
    }

    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn same_cell_never_blocks() {
        let terrain = TerrainField::new(10, 10, 1.0, Vec2::ZERO, 0);
        let pos = Vec2::new(0.5, 0.5);
        assert!(!terrain_blocks_vision(pos, pos, &terrain));
    }

    #[test]
    fn flat_terrain_never_blocks() {
        let terrain = TerrainField::new(20, 20, 1.0, Vec2::ZERO, 0);
        // New terrain has altitude 0.0 everywhere (flat)
        assert!(!terrain_blocks_vision(
            Vec2::new(0.5, 0.5),
            Vec2::new(19.5, 19.5),
            &terrain,
        ));
    }

    #[test]
    fn out_of_bounds_blocks() {
        let terrain = TerrainField::new(5, 5, 1.0, Vec2::ZERO, 0);
        assert!(terrain_blocks_vision(
            Vec2::new(-100.0, 0.0),
            Vec2::new(0.5, 0.5),
            &terrain,
        ));
    }

    #[test]
    fn raycast_cells_exclusive_same_cell_is_empty() {
        assert!(raycast_cells_exclusive((5, 5), (5, 5)).is_empty());
    }

    #[test]
    fn raycast_cells_exclusive_adjacent_is_empty() {
        let cells = raycast_cells_exclusive((0, 0), (1, 0));
        assert!(
            cells.is_empty(),
            "Adjacent cells have no intermediate: {cells:?}"
        );
    }
}
