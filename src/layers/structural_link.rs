use bevy::prelude::*;

/// Capa 13: Vínculo físico tipo resorte entre nodos.
#[derive(Component, Reflect, Debug, Clone)]
#[reflect(Component)]
pub struct StructuralLink {
    pub target: Entity,
    pub rest_length: f32,
    pub stiffness: f32,
    pub break_stress: f32,
}

impl StructuralLink {
    pub fn new(target: Entity, rest_length: f32, stiffness: f32, break_stress: f32) -> Self {
        Self {
            target,
            rest_length: rest_length.max(0.0),
            stiffness: stiffness.max(0.0),
            break_stress: break_stress.max(0.0),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::Entity;

    #[test]
    fn new_clamps_negative_parameters() {
        let t = Entity::from_raw(99);
        let s = StructuralLink::new(t, -1.0, -2.0, -3.0);
        assert_eq!(s.target, t);
        assert_eq!(s.rest_length, 0.0);
        assert_eq!(s.stiffness, 0.0);
        assert_eq!(s.break_stress, 0.0);
    }

    #[test]
    fn new_preserves_positive_geometry() {
        let t = Entity::from_raw(3);
        let s = StructuralLink::new(t, 2.0, 100.0, 50.0);
        assert!((s.rest_length - 2.0).abs() < 1e-5);
        assert!((s.stiffness - 100.0).abs() < 1e-5);
    }

    #[test]
    fn structural_link_clone_preserves_target() {
        let t = Entity::from_raw(1);
        let a = StructuralLink::new(t, 1.0, 1.0, 1.0);
        let b = a.clone();
        assert_eq!(a.target, b.target);
    }
}
