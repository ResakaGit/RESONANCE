//! Terminal renderer — ASCII art with ANSI colors in real-time.
//! Zero external deps. Works in any terminal that supports 256-color.
//!
//! Usage: `--render terminal` flag in headless_sim.

use super::frame_buffer::FrameBuffer;

/// ANSI 256-color block characters for terminal output.
const BLOCK_CHARS: [char; 5] = [' ', '░', '▒', '▓', '█'];

/// Render a frame buffer to the terminal using ANSI escape codes.
/// Pure function: FrameBuffer → String. No I/O.
pub fn render_to_string(frame: &FrameBuffer, tick: u64) -> String {
    let mut out = String::with_capacity(frame.width * frame.height * 20);

    // Clear screen + move cursor to top.
    out.push_str("\x1b[2J\x1b[H");

    // Header.
    out.push_str(&format!(
        "\x1b[1;37m tick:{tick:<6} entities:{:<4} beh:{:<3} qe:{:.0}\x1b[0m\n",
        frame.entity_count, frame.behavioral_count, frame.total_qe
    ));

    // Render grid: 2 cells per character (half-height for aspect ratio).
    let step_y = if frame.height > 40 { 2 } else { 1 };
    let step_x = if frame.width > 80 { 2 } else { 1 };

    for y in (0..frame.height).step_by(step_y) {
        for x in (0..frame.width).step_by(step_x) {
            let idx = y * frame.width + x;
            let [r, g, b, _] = frame.pixels[idx];

            if r == 0 && g == 0 && b == 0 {
                out.push(' ');
                continue;
            }

            // Entity dots: white or cyan.
            if r == 255 && g == 255 && b == 255 {
                out.push_str("\x1b[97m●\x1b[0m");
                continue;
            }
            if r == 0 && g == 255 && b == 255 {
                out.push_str("\x1b[96m○\x1b[0m");
                continue;
            }

            // Field: intensity → block character, color from RGB.
            let intensity = (r as f32 + g as f32 + b as f32) / (3.0 * 255.0);
            let block_idx = (intensity * 4.0).min(4.0) as usize;
            let ansi_color = rgb_to_ansi256(r, g, b);
            out.push_str(&format!("\x1b[38;5;{ansi_color}m{}\x1b[0m", BLOCK_CHARS[block_idx]));
        }
        out.push('\n');
    }

    out
}

/// Print frame directly to stderr (avoids buffering issues with stdout).
pub fn display_frame(frame: &FrameBuffer, tick: u64) {
    let rendered = render_to_string(frame, tick);
    eprint!("{rendered}");
}

/// Map RGB to ANSI 256-color index. Pure.
fn rgb_to_ansi256(r: u8, g: u8, b: u8) -> u8 {
    if r == g && g == b {
        // Grayscale ramp (232-255).
        let gray = r as u16;
        if gray < 8 { return 16; }
        if gray > 248 { return 231; }
        return (((gray - 8) * 24 / 240) + 232) as u8;
    }
    // 6×6×6 color cube (16-231).
    let ri = (r as u16 * 5 / 255) as u8;
    let gi = (g as u16 * 5 / 255) as u8;
    let bi = (b as u16 * 5 / 255) as u8;
    16 + 36 * ri + 6 * gi + bi
}
