//! Shared frame buffer — pure function that converts EnergyFieldGrid → pixel data.
//! Used by terminal, pixel_window, and planet_viewer renderers.

use crate::worldgen::EnergyFieldGrid;

// ── Visual calibration (rendering, not physics) ─────────────────────────────
/// Log normalization reference: ln(1 + REF) is the 100% white point.
const LOG_REFERENCE_QE: f32 = 50.0;
/// Minimum brightness so dark terrain is visible against space background.
const SURFACE_MIN_BRIGHTNESS: f32 = 0.06;

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
///
/// Color ramp: thermal (black → blue → cyan → green → yellow → white).
/// Hue from dominant frequency. Brightness from log(qe).
pub fn render_frame(
    grid: &EnergyFieldGrid,
    entity_positions: &[(u32, u32, f32)], // (cell_x, cell_y, frequency_hz)
    behavioral_positions: &[(u32, u32)],
) -> FrameBuffer {
    let w = grid.width as usize;
    let h = grid.height as usize;
    let mut pixels = vec![[0u8, 0, 0, 255]; w * h];
    let log_ref = (1.0 + LOG_REFERENCE_QE).ln();

    for y in 0..grid.height {
        for x in 0..grid.width {
            if let Some(cell) = grid.cell_xy(x, y) {
                let t = ((1.0 + cell.accumulated_qe).ln() / log_ref).clamp(0.0, 1.0);
                let freq_hue = cell.dominant_frequency_hz / 800.0; // 0→0 Hz, 1→800 Hz
                let (r, g, b) = thermal_ramp(t, freq_hue);
                let idx = (grid.height - 1 - y) as usize * w + x as usize;
                pixels[idx] = [r, g, b, 255];
            }
        }
    }

    // Entity dots: bright white.
    for &(cx, cy, _freq) in entity_positions {
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

/// Thermal color ramp modulated by frequency hue.
/// `t` = energy intensity [0, 1]. `freq_hue` = frequency band [0, 1].
/// Low energy → dark blue/purple. High energy → bright green/yellow/white.
fn thermal_ramp(t: f32, freq_hue: f32) -> (u8, u8, u8) {
    let t = t.max(SURFACE_MIN_BRIGHTNESS);

    // Base thermal: black → blue → cyan → green → yellow → white.
    let (r, g, b) = if t < 0.2 {
        let s = t / 0.2;
        (0.0, 0.0, s * 0.5)              // black → dark blue
    } else if t < 0.4 {
        let s = (t - 0.2) / 0.2;
        (0.0, s * 0.5, 0.5)              // dark blue → cyan
    } else if t < 0.6 {
        let s = (t - 0.4) / 0.2;
        (0.0, 0.5 + s * 0.5, 0.5 - s * 0.3) // cyan → green
    } else if t < 0.8 {
        let s = (t - 0.6) / 0.2;
        (s * 0.8, 1.0, 0.2 - s * 0.2)   // green → yellow
    } else {
        let s = (t - 0.8) / 0.2;
        (0.8 + s * 0.2, 1.0 - s * 0.3, s * 0.5) // yellow → warm white
    };

    // Frequency tint: shift hue slightly based on dominant frequency.
    let tint_r = r + freq_hue * 0.15;
    let tint_g = g;
    let tint_b = b + (1.0 - freq_hue) * 0.1;

    (
        (tint_r.clamp(0.0, 1.0) * 255.0) as u8,
        (tint_g.clamp(0.0, 1.0) * 255.0) as u8,
        (tint_b.clamp(0.0, 1.0) * 255.0) as u8,
    )
}
