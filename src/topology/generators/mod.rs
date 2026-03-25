//! Generadores stateless del heightmap (T2+). Sin ECS; solo funciones puras.

pub mod classifier;
pub mod drainage;
pub mod hydraulics;
pub mod noise;
pub mod slope;

pub use classifier::{ClassificationThresholds, classify_all, classify_terrain};
pub use drainage::{compute_flow_accumulation, compute_flow_direction, fill_pits};
pub use hydraulics::{ErosionParams, erode_hydraulic};
pub use noise::{NoiseParams, generate_heightmap, normalize_heightmap};
pub use slope::{derive_aspect, derive_slope, derive_slope_aspect};
