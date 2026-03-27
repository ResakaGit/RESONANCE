//! Engine-agnostic math type re-exports.
//! All non-ECS code imports math types from here, not from `bevy::math`.
//! When extracting `resonance_core` as a standalone lib, change this one file.

pub use glam::{Quat, Vec2, Vec3, Vec4};
pub use std::f32::consts::PI;
