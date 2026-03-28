//! Shared frame buffer — pure function that converts EnergyFieldGrid → pixel data.
//! Used by both terminal and pixel_window renderers.

use crate::worldgen::EnergyFieldGrid;

/// RGBA pixel data for one frame.
pub struct FrameBuffer {
    pub width: usize,
    pub height: usize,
    pub pixels: Vec<[u8; 4]>, // RGBA
    pub entity_count: usize,
    pub behavioral_count: usize,
    pub total_qe: f32,
}

/// Render the energy field grid to a pixel buffer.
/// Pure function: grid in → pixels out. No state. No side effects.
pub fn render_frame(
    grid: &EnergyFieldGrid,
    entity_positions: &[(u32, u32, f32)], // (cell_x, cell_y, frequency_hz)
    behavioral_positions: &[(u32, u32)],
) -> FrameBuffer {
    let w = grid.width as usize;
    let h = grid.height as usize;
    let mut pixels = vec![[0u8, 0, 0, 255]; w * h];

    // Find max for normalization.
    let mut max_qe: f32 = 1.0;
    let mut max_freq: f32 = 1.0;
    for y in 0..grid.height {
        for x in 0..grid.width {
            if let Some(cell) = grid.cell_xy(x, y) {
                max_qe = max_qe.max(cell.accumulated_qe);
                max_freq = max_freq.max(cell.dominant_frequency_hz);
            }
        }
    }

    // Field cells → pixels.
    for y in 0..grid.height {
        for x in 0..grid.width {
            if let Some(cell) = grid.cell_xy(x, y) {
                let intensity = (cell.accumulated_qe / max_qe).sqrt();
                let hue = if max_freq > 0.0 { cell.dominant_frequency_hz / max_freq } else { 0.0 };
                let sat = cell.purity.max(0.3);
                let boosted = (intensity * 1.5).min(1.0);
                let (r, g, b) = hsv_to_rgb(hue, sat, boosted);
                let idx = (grid.height - 1 - y) as usize * w + x as usize;
                pixels[idx] = [r, g, b, 255];
            }
        }
    }

    // Entity dots.
    for &(cx, cy, freq) in entity_positions {
        let idx = (grid.height - 1 - cy) as usize * w + cx as usize;
        if idx < pixels.len() {
            pixels[idx] = [255, 255, 255, 255];
        }
    }

    // Behavioral agents: cyan ring.
    for &(cx, cy) in behavioral_positions {
        for (dx, dy) in [(-1i32, 0), (1, 0), (0, -1), (0, 1)] {
            let nx = cx as i32 + dx;
            let ny = cy as i32 + dy;
            if nx >= 0 && ny >= 0 && (nx as u32) < grid.width && (ny as u32) < grid.height {
                let ri = (grid.height as i32 - 1 - ny) as usize * w + nx as usize;
                if ri < pixels.len() {
                    pixels[ri] = [0, 255, 255, 255];
                }
            }
        }
    }

    FrameBuffer {
        width: w,
        height: h,
        pixels,
        entity_count: entity_positions.len(),
        behavioral_count: behavioral_positions.len(),
        total_qe: grid.total_qe(),
    }
}

/// HSV → RGB conversion. Pure.
fn hsv_to_rgb(h: f32, s: f32, v: f32) -> (u8, u8, u8) {
    let h = h.clamp(0.0, 1.0) * 6.0;
    let s = s.clamp(0.0, 1.0);
    let v = v.clamp(0.0, 1.0);
    let c = v * s;
    let x = c * (1.0 - (h % 2.0 - 1.0).abs());
    let m = v - c;
    let (r, g, b) = match h as u32 {
        0 => (c, x, 0.0),
        1 => (x, c, 0.0),
        2 => (0.0, c, x),
        3 => (0.0, x, c),
        4 => (x, 0.0, c),
        _ => (c, 0.0, x),
    };
    (((r + m) * 255.0) as u8, ((g + m) * 255.0) as u8, ((b + m) * 255.0) as u8)
}
