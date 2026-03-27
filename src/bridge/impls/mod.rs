//! Bridge implementations — `Bridgeable` impls + `SystemParam` wrappers.

#[cfg(feature = "bridge_optimizer")]
pub mod physics;
pub mod ops;

#[cfg(feature = "bridge_optimizer")]
pub use physics::*;
pub use ops::*;
