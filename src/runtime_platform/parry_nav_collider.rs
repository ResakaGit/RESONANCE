//! Collider Parry3d mínimo para `oxidized_navigation` sin motor de física (feature default `parry3d`).

use bevy::prelude::*;
use oxidized_navigation::colliders::OxidizedCollider;
use parry3d::{
    bounding_volume::Aabb,
    shape::{SharedShape, TypedShape},
};

/// Forma Parry3d compartida; escala va en `Transform` (convención oxidized).
#[derive(Component, Debug, Clone)]
pub struct ParryNavCollider {
    pub shape: SharedShape,
}

impl OxidizedCollider for ParryNavCollider {
    fn oxidized_into_typed_shape(&self) -> TypedShape<'_> {
        self.shape.as_typed_shape()
    }

    fn oxidized_compute_local_aabb(&self) -> Aabb {
        self.shape.compute_local_aabb()
    }
}
