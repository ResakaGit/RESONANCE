//! Batch simulator — millions of worlds without Bevy.
//!
//! Reuses `blueprint::equations` (100%) and `blueprint::constants` (100%).
//! Zero Bevy dependency. Fixed-size layouts. Cache-friendly iteration.
//!
//! See `docs/arquitectura/blueprint_batch_simulator.md`.

pub mod arena;
pub mod batch;
pub mod bridge;
pub mod constants;
pub mod events;
pub mod genome;
pub mod harness;
pub mod pipeline;
pub mod scratch;
pub mod systems;

pub use arena::{EntitySlot, SimWorldFlat};
pub use batch::{BatchConfig, WorldBatch};
pub use constants::{GRID_CELLS, MAX_ENTITIES};
pub use genome::GenomeBlob;
pub use harness::{FitnessReport, GenerationStats, GeneticHarness};
pub use scratch::ScratchPad;
