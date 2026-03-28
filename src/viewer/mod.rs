//! Viewer module — stateless rendering functions for simulation output.
//!
//! Two backends:
//! - `terminal`: ASCII art in terminal (zero deps, real-time)
//! - `pixel_window`: 2D pixel window (requires `pixel_viewer` feature)
//!
//! Both share `frame_buffer` for grid → pixel conversion.

pub mod frame_buffer;
pub mod terminal;
#[cfg(feature = "pixel_viewer")]
pub mod pixel_window;
