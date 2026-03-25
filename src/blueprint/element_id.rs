use bevy::prelude::*;
use core::hash::Hash;
use serde::{Deserialize, Serialize};

use super::constants::{FNV_OFFSET_BASIS, FNV_PRIME};

/// Identidad elemental compacta para el ECS (4 bytes).
///
/// Importante: el hash debe ser estable entre ejecuciones para determinismo.
/// Por eso usamos FNV-1a 32-bit en vez de `DefaultHasher` (que puede variar).
#[derive(
    Component,
    Clone,
    Copy,
    Debug,
    PartialEq,
    Eq,
    Hash,
    Reflect,
    Default,
    PartialOrd,
    Ord,
    Serialize,
    Deserialize,
)]
pub struct ElementId(pub u32);

impl ElementId {
    /// Hash FNV del **símbolo** del almanaque (`ElementDef.symbol`), no del `name` display.
    pub fn from_name(name: &str) -> Self {
        Self(fnv1a32(name.as_bytes()))
    }

    pub const fn raw(self) -> u32 {
        self.0
    }
}

impl core::fmt::Display for ElementId {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "ElementId({})", self.0)
    }
}

fn fnv1a32(bytes: &[u8]) -> u32 {
    let mut hash: u32 = FNV_OFFSET_BASIS;
    for &b in bytes {
        hash ^= b as u32;
        hash = hash.wrapping_mul(FNV_PRIME);
    }
    hash
}
