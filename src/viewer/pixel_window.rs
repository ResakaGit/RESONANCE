//! Pixel window renderer — real 2D window using `pixels` crate.
//! Feature-gated: requires `pixel_viewer` feature.
//!
//! Usage: `--render window` flag in headless_sim.
//! `cargo run --release --features pixel_viewer --bin headless_sim -- --render window`

use super::frame_buffer::FrameBuffer;

/// Window configuration.
pub struct WindowConfig {
    pub title: String,
    pub scale: u32,
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
    use pixels::Pixels;
    use winit::application::ApplicationHandler;
    use winit::dpi::LogicalSize;
    use winit::event::WindowEvent;
    use winit::event_loop::{ActiveEventLoop, EventLoop};
    use winit::keyboard::{Key, NamedKey};
    use winit::window::{Window, WindowId};

    struct App<'a, F2: FnMut() -> Option<FrameBuffer>> {
        window: Option<Window>,
        pixels: Option<Pixels<'a>>,
        width: u32,
        height: u32,
        scale: u32,
        title: String,
        tick_fn: F2,
    }

    impl<F2: FnMut() -> Option<FrameBuffer>> ApplicationHandler for App<'_, F2> {
        fn resumed(&mut self, event_loop: &ActiveEventLoop) {
            let size = LogicalSize::new(
                (self.width * self.scale) as f64,
                (self.height * self.scale) as f64,
            );
            let attrs = Window::default_attributes()
                .with_title(&self.title)
                .with_inner_size(size)
                .with_min_inner_size(size);
            let window = event_loop.create_window(attrs).expect("create window");

            let surface_size = window.inner_size();
            let pixels = Pixels::new(self.width, self.height, pixels::SurfaceTexture::new(
                surface_size.width, surface_size.height, &window,
            )).expect("create pixels");

            self.window = Some(window);
            self.pixels = Some(pixels);
        }

        fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
            match event {
                WindowEvent::CloseRequested => event_loop.exit(),
                WindowEvent::KeyboardInput { event, .. } => {
                    if event.logical_key == Key::Named(NamedKey::Escape) {
                        event_loop.exit();
                    }
                }
                WindowEvent::RedrawRequested => {
                    let Some(ref mut px) = self.pixels else { return };
                    if let Some(frame) = (self.tick_fn)() {
                        let fb = px.frame_mut();
                        for (i, [r, g, b, a]) in frame.pixels.iter().enumerate() {
                            let offset = i * 4;
                            if offset + 3 < fb.len() {
                                fb[offset] = *r;
                                fb[offset + 1] = *g;
                                fb[offset + 2] = *b;
                                fb[offset + 3] = *a;
                            }
                        }
                        let _ = px.render();
                    }
                    if let Some(ref w) = self.window {
                        w.request_redraw();
                    }
                }
                _ => {}
            }
        }
    }

    let event_loop = EventLoop::new().expect("event loop");
    let mut app = App {
        window: None,
        pixels: None,
        width,
        height,
        scale: config.scale,
        title: config.title,
        tick_fn,
    };
    let _ = event_loop.run_app(&mut app);
}
