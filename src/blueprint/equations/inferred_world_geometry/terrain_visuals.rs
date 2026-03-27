//! IWG-3 — Terrain mesh visual equations: cell color from energy field + terrain slope.

use crate::blueprint::constants::inferred_world_geometry::{
    QE_BRIGHTNESS_MAX, QE_BRIGHTNESS_MIN, SLOPE_SHADOW_FACTOR, SLOPE_SHADOW_THRESHOLD,
    STATE_SATURATION, TERRAIN_BAND_COLOR,
};
use crate::layers::MatterState;
use crate::topology::{TerrainField, TerrainVisuals};
use crate::worldgen::EnergyFieldGrid;

/// Compute RGBA vertex color for a single terrain cell.
///
/// `rgb = lerp(gray, base, saturation) * brightness`, with slope shadow applied.
pub fn terrain_cell_color(
    element_band: u8,
    qe_norm: f32,
    matter_state: MatterState,
    slope: f32,
) -> [f32; 4] {
    let band_idx = (element_band as usize).min(7);
    let [br, bg, bb] = TERRAIN_BAND_COLOR[band_idx];

    let qe_t = qe_norm.clamp(0.0, 1.0);
    let brightness = QE_BRIGHTNESS_MIN + (QE_BRIGHTNESS_MAX - QE_BRIGHTNESS_MIN) * qe_t;

    let sat_idx = match matter_state {
        MatterState::Solid  => 0,
        MatterState::Liquid => 1,
        MatterState::Gas    => 2,
        MatterState::Plasma => 3,
    };
    let saturation = STATE_SATURATION[sat_idx];

    // Grayscale luminance (Rec. 709).
    let gray = 0.2126 * br + 0.7152 * bg + 0.0722 * bb;

    let mut r = gray + (br - gray) * saturation;
    let mut g = gray + (bg - gray) * saturation;
    let mut b = gray + (bb - gray) * saturation;

    r *= brightness;
    g *= brightness;
    b *= brightness;

    if slope > SLOPE_SHADOW_THRESHOLD {
        r *= SLOPE_SHADOW_FACTOR;
        g *= SLOPE_SHADOW_FACTOR;
        b *= SLOPE_SHADOW_FACTOR;
    }

    [r.clamp(0.0, 1.0), g.clamp(0.0, 1.0), b.clamp(0.0, 1.0), 1.0]
}

/// Build per-cell vertex colors by crossing energy field data with terrain slope.
///
/// Returns `TerrainVisuals::neutral_flat` on dimension mismatch.
pub fn build_terrain_visuals(grid: &EnergyFieldGrid, terrain: &TerrainField) -> TerrainVisuals {
    let neutral_rgba = [0.4, 0.4, 0.4, 1.0];
    if grid.width != terrain.width || grid.height != terrain.height {
        return TerrainVisuals::neutral_flat(terrain, neutral_rgba);
    }

    let total = terrain.total_cells();
    let max_qe = grid
        .iter_cells()
        .map(|c| c.accumulated_qe)
        .fold(0.0f32, f32::max)
        .max(1.0);

    let mut vertex_colors = Vec::with_capacity(total);
    for idx in 0..total {
        let Some(cell) = grid.cell_linear(idx) else {
            vertex_colors.push(neutral_rgba);
            continue;
        };

        let qe_norm = (cell.accumulated_qe / max_qe).clamp(0.0, 1.0);
        let element_band = ((cell.dominant_frequency_hz / 100.0) as u8).min(7);
        let slope = terrain.slope.get(idx).copied().unwrap_or(0.0);

        vertex_colors.push(terrain_cell_color(element_band, qe_norm, cell.matter_state, slope));
    }

    TerrainVisuals { vertex_colors }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::math::Vec2;

    fn make_grid(w: u32, h: u32) -> EnergyFieldGrid {
        EnergyFieldGrid::new(w, h, 1.0, Vec2::ZERO)
    }

    fn make_terrain(w: u32, h: u32) -> TerrainField {
        TerrainField::new(w, h, 1.0, Vec2::ZERO, 0)
    }

    #[test]
    fn terra_band_solid_flat_slope_brown() {
        let c = terrain_cell_color(0, 0.8, MatterState::Solid, 0.0);
        assert!(c[0] > c[1], "r > g for Terra");
        assert!(c[0] > c[2], "r > b for Terra");
    }

    #[test]
    fn aqua_band_liquid_blue_dominant() {
        let c = terrain_cell_color(1, 0.8, MatterState::Liquid, 0.0);
        assert!(c[2] > c[0], "b > r for Aqua Liquid");
    }

    #[test]
    fn high_qe_brighter_than_low() {
        let bright = terrain_cell_color(7, 1.0, MatterState::Solid, 0.0);
        let dim    = terrain_cell_color(7, 0.0, MatterState::Solid, 0.0);
        let lum_bright = 0.2126 * bright[0] + 0.7152 * bright[1] + 0.0722 * bright[2];
        let lum_dim    = 0.2126 * dim[0]    + 0.7152 * dim[1]    + 0.0722 * dim[2];
        assert!(lum_bright > lum_dim, "high qe should be brighter");
    }

    #[test]
    fn slope_shadow_darkens() {
        let flat  = terrain_cell_color(3, 0.5, MatterState::Solid, 0.0);
        let steep = terrain_cell_color(3, 0.5, MatterState::Solid, 0.5);
        assert!(steep[0] < flat[0], "slope shadow should darken r");
        assert!(steep[1] < flat[1], "slope shadow should darken g");
        assert!(steep[2] < flat[2], "slope shadow should darken b");
    }

    #[test]
    fn channels_clamped_alpha_one() {
        for band in 0..8u8 {
            for &state in &[MatterState::Solid, MatterState::Liquid, MatterState::Gas, MatterState::Plasma] {
                let c = terrain_cell_color(band, 1.5, state, 10.0);
                assert!(c[0] >= 0.0 && c[0] <= 1.0);
                assert!(c[1] >= 0.0 && c[1] <= 1.0);
                assert!(c[2] >= 0.0 && c[2] <= 1.0);
                assert_eq!(c[3], 1.0);
            }
        }
    }

    #[test]
    fn terrain_cell_color_determinism() {
        let a = terrain_cell_color(4, 0.6, MatterState::Gas, 0.2);
        let b = terrain_cell_color(4, 0.6, MatterState::Gas, 0.2);
        assert_eq!(a, b, "same inputs must yield identical outputs");
    }

    #[test]
    fn build_terrain_visuals_dimension_mismatch_neutral() {
        let grid = make_grid(4, 4);
        let terrain = make_terrain(3, 5);
        let vis = build_terrain_visuals(&grid, &terrain);
        assert_eq!(vis.vertex_colors.len(), terrain.total_cells());
        // All should be the neutral RGBA.
        for c in &vis.vertex_colors {
            assert_eq!(*c, [0.4, 0.4, 0.4, 1.0]);
        }
    }

    #[test]
    fn build_terrain_visuals_4x4_16_colors() {
        let grid = make_grid(4, 4);
        let terrain = make_terrain(4, 4);
        let vis = build_terrain_visuals(&grid, &terrain);
        assert_eq!(vis.vertex_colors.len(), 16);
        for c in &vis.vertex_colors {
            assert!(c[0] >= 0.0 && c[0] <= 1.0);
            assert!(c[1] >= 0.0 && c[1] <= 1.0);
            assert!(c[2] >= 0.0 && c[2] <= 1.0);
            assert_eq!(c[3], 1.0);
        }
    }
}
