//! Bridge implementations — `Bridgeable` impls + `SystemParam` wrappers.

pub mod ops;
#[cfg(feature = "bridge_optimizer")]
pub mod physics;

pub use ops::*;
#[cfg(feature = "bridge_optimizer")]
pub use physics::*;
