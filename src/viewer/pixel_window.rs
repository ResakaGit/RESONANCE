//! Pixel window renderer — real 2D window using `minifb` crate.
//! Feature-gated: requires `pixel_viewer` feature.
//!
//! Usage: `--render window` flag in sim_viewer.
//! `cargo run --release --features pixel_viewer --bin sim_viewer -- --render window`

use super::frame_buffer::FrameBuffer;

/// Window configuration.
pub struct WindowConfig {
    pub title: String,
    pub scale: usize,
}

impl Default for WindowConfig {
    fn default() -> Self {
        Self {
            title: "Resonance — Simulation Viewer".to_string(),
            scale: 8,
        }
    }
}

/// Run the pixel window event loop.
/// Calls `tick_fn` each frame to get the next FrameBuffer.
/// Blocking: returns when window is closed or ESC pressed.
pub fn run_window<F>(config: WindowConfig, width: u32, height: u32, mut tick_fn: F)
where
    F: FnMut() -> Option<FrameBuffer>,
{
    use minifb::{Key, Window, WindowOptions};

    let w = width as usize;
    let h = height as usize;
    let scaled_w = w * config.scale;
    let scaled_h = h * config.scale;

    let mut window = Window::new(
        &config.title,
        scaled_w,
        scaled_h,
        WindowOptions::default(),
    ).expect("failed to create window");

    // ~60 fps
    window.set_target_fps(60);

    let mut buffer = vec![0u32; scaled_w * scaled_h];

    while window.is_open() && !window.is_key_down(Key::Escape) {
        if let Some(frame) = tick_fn() {
            // Upscale: frame pixels → window buffer.
            for sy in 0..scaled_h {
                for sx in 0..scaled_w {
                    let src_x = sx / config.scale;
                    let src_y = sy / config.scale;
                    let src_idx = src_y * w + src_x;
                    let [r, g, b, _] = frame.pixels[src_idx.min(frame.pixels.len() - 1)];
                    buffer[sy * scaled_w + sx] = (r as u32) << 16 | (g as u32) << 8 | b as u32;
                }
            }
            window.set_title(&format!(
                "Resonance — entities:{} beh:{} qe:{:.0}",
                frame.entity_count, frame.behavioral_count, frame.total_qe,
            ));
        }
        window.update_with_buffer(&buffer, scaled_w, scaled_h).expect("update buffer");
    }
}
