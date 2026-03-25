//! Albedo inferido por balance radiativo (MG-5A).
//! Solo entidades con MetabolicGraph lo reciben.

use bevy::prelude::*;

use crate::blueprint::constants::morphogenesis as mg;

/// Albedo inferido por balance radiativo. α ∈ [ALBEDO_MIN, ALBEDO_MAX].
#[derive(Component, Reflect, Debug, Clone, Copy, PartialEq)]
#[reflect(Component)]
#[component(storage = "SparseSet")]
pub struct InferredAlbedo {
    albedo: f32,
}

impl InferredAlbedo {
    /// Construye con clamp a [ALBEDO_MIN, ALBEDO_MAX].
    pub fn new(albedo: f32) -> Self {
        Self {
            albedo: albedo.clamp(mg::ALBEDO_MIN, mg::ALBEDO_MAX),
        }
    }

    #[inline]
    pub fn albedo(&self) -> f32 {
        self.albedo
    }

    /// Setter con clamp — para uso desde sistemas con guard externo.
    pub fn set_albedo(&mut self, val: f32) {
        self.albedo = val.clamp(mg::ALBEDO_MIN, mg::ALBEDO_MAX);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::ecs::component::StorageType;
    use crate::blueprint::constants::morphogenesis as mg;

    #[test]
    fn new_standard_value_preserved() {
        let a = InferredAlbedo::new(0.7);
        assert!((a.albedo() - 0.7).abs() < 1e-6);
    }

    #[test]
    fn new_negative_clamps_to_min() {
        let a = InferredAlbedo::new(-0.5);
        assert!((a.albedo() - mg::ALBEDO_MIN).abs() < 1e-6);
    }

    #[test]
    fn new_over_one_clamps_to_max() {
        let a = InferredAlbedo::new(1.5);
        assert!((a.albedo() - mg::ALBEDO_MAX).abs() < 1e-6);
    }

    #[test]
    fn is_copy() {
        let a = InferredAlbedo::new(0.5);
        let b = a;
        assert_eq!(a.albedo(), b.albedo());
    }

    #[test]
    fn is_sparse_set() {
        assert_eq!(InferredAlbedo::STORAGE_TYPE, StorageType::SparseSet);
    }

    #[test]
    fn set_albedo_clamps() {
        let mut a = InferredAlbedo::new(0.5);
        a.set_albedo(2.0);
        assert!((a.albedo() - mg::ALBEDO_MAX).abs() < 1e-6);
        a.set_albedo(-1.0);
        assert!((a.albedo() - mg::ALBEDO_MIN).abs() < 1e-6);
    }

    #[test]
    fn reflect_registered_in_layers_plugin() {
        use crate::plugins::layers_plugin::LayersPlugin;
        let mut app = bevy::prelude::App::new();
        app.add_plugins(LayersPlugin);
        let registry = app.world().resource::<bevy::prelude::AppTypeRegistry>().read();
        assert!(
            registry.get(std::any::TypeId::of::<InferredAlbedo>()).is_some(),
            "InferredAlbedo must be registered for Reflect",
        );
    }
}
