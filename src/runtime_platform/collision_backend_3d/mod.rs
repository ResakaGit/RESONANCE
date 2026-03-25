use std::collections::HashMap;

use bevy::prelude::{Entity, Query, Res, ResMut, Resource, Transform, Vec3};

use crate::layers::SpatialVolume;
use crate::runtime_platform::contracts::{CollisionContact, Pose3, SpatialCandidatePair};
use crate::runtime_platform::spatial_index_backend::SpatialBroadphase;

/// Config de bridge 3D/2D para narrowphase V6.
#[derive(Resource, Debug, Clone, Copy)]
pub struct V6CollisionBackendConfig {
    /// Si es `true`, usa narrowphase 3D esfera-esfera.
    /// Si es `false`, delega al snapshot legacy 2D.
    pub use_3d_backend: bool,
}

impl Default for V6CollisionBackendConfig {
    fn default() -> Self {
        Self {
            use_3d_backend: cfg!(feature = "v6_collision_backend_3d"),
        }
    }
}

/// Snapshot puente para mantener compatibilidad cuando el backend 3D está apagado.
#[derive(Resource, Debug, Clone, Default)]
pub struct LegacyCollisionContactSet {
    pub contacts: Vec<CollisionContact>,
}

/// Salida ordenada de narrowphase para consumo del pipeline.
#[derive(Resource, Debug, Clone, Default)]
pub struct CollisionContactSet {
    pub contacts: Vec<CollisionContact>,
}

/// Contacto esfera-esfera puro en 3D.
pub fn sphere_sphere_contact(
    a_entity: Entity,
    a: Pose3,
    b_entity: Entity,
    b: Pose3,
) -> Option<CollisionContact> {
    let pair = SpatialCandidatePair::new(a_entity, b_entity);
    let (left_pose, right_pose) = if pair.a == a_entity { (a, b) } else { (b, a) };

    let delta = right_pose.position - left_pose.position;
    let distance_sq = delta.length_squared();
    let radius_sum = left_pose.radius + right_pose.radius;
    if distance_sq > radius_sum * radius_sum {
        return None;
    }

    let distance = distance_sq.sqrt();
    // Si centros coinciden, elegimos un normal estable para determinismo.
    let normal = if distance > f32::EPSILON {
        delta / distance
    } else {
        Vec3::X
    };
    let penetration = (radius_sum - distance).max(0.0);

    Some(CollisionContact::new(
        pair.a,
        pair.b,
        normal,
        Some(penetration),
    ))
}

/// Narrowphase 3D puro: consume candidatos broadphase y poses.
pub fn build_narrowphase_3d_contacts<B: SpatialBroadphase>(
    broadphase: &B,
    poses: &HashMap<Entity, Pose3>,
) -> Vec<CollisionContact> {
    let mut contacts = Vec::new();

    for pair in broadphase.candidate_pairs() {
        let Some(a_pose) = poses.get(&pair.a).copied() else {
            continue;
        };
        let Some(b_pose) = poses.get(&pair.b).copied() else {
            continue;
        };

        if let Some(contact) = sphere_sphere_contact(pair.a, a_pose, pair.b, b_pose) {
            contacts.push(contact);
        }
    }

    contacts.sort_by_key(|c| (c.a.to_bits(), c.b.to_bits()));
    contacts
}

/// Sistema narrowphase 3D con bridge de compatibilidad 2D cuando el flag está apagado.
pub fn narrowphase_3d_system<B: SpatialBroadphase + Resource>(
    config: Res<V6CollisionBackendConfig>,
    backend: Option<Res<B>>,
    volume_query: Query<(Entity, &Transform, &SpatialVolume)>,
    legacy_contacts: Res<LegacyCollisionContactSet>,
    mut out_contacts: ResMut<CollisionContactSet>,
) {
    if !config.use_3d_backend {
        let mut contacts = legacy_contacts.contacts.clone();
        contacts.sort_by_key(|c| (c.a.to_bits(), c.b.to_bits()));
        out_contacts.contacts = contacts;
        return;
    }

    let Some(backend) = backend else {
        out_contacts.contacts.clear();
        return;
    };

    let mut poses = HashMap::with_capacity(volume_query.iter().len());
    for (entity, transform, volume) in &volume_query {
        poses.insert(entity, Pose3::new(transform.translation, volume.radius));
    }

    out_contacts.contacts = build_narrowphase_3d_contacts(backend.as_ref(), &poses);
}

#[cfg(test)]
mod tests {
    use super::*;

    struct TestBroadphase {
        pairs: Vec<SpatialCandidatePair>,
    }

    impl SpatialBroadphase for TestBroadphase {
        fn clear(&mut self) {
            self.pairs.clear();
        }

        fn insert(
            &mut self,
            _pose: crate::runtime_platform::spatial_index_backend::SpatialPose,
            _entity: Entity,
            _radius: f32,
        ) {
        }

        fn candidate_pairs(&self) -> Vec<SpatialCandidatePair> {
            self.pairs.clone()
        }
    }

    #[test]
    fn sphere_sphere_contact_tangent_returns_zero_penetration() {
        let a = Entity::from_raw(1);
        let b = Entity::from_raw(2);
        let contact = sphere_sphere_contact(
            a,
            Pose3::new(Vec3::ZERO, 1.0),
            b,
            Pose3::new(Vec3::new(2.0, 0.0, 0.0), 1.0),
        )
        .expect("tangent contact must exist");

        assert_eq!(contact.a, a);
        assert_eq!(contact.b, b);
        assert_eq!(contact.normal, Vec3::X);
        assert_eq!(contact.penetration, Some(0.0));
    }

    #[test]
    fn sphere_sphere_contact_separated_returns_none() {
        let a = Entity::from_raw(1);
        let b = Entity::from_raw(2);
        let contact = sphere_sphere_contact(
            a,
            Pose3::new(Vec3::ZERO, 1.0),
            b,
            Pose3::new(Vec3::new(3.0, 0.0, 0.0), 1.0),
        );

        assert!(contact.is_none());
    }

    #[test]
    fn sphere_sphere_contact_contained_returns_penetration() {
        let a = Entity::from_raw(1);
        let b = Entity::from_raw(2);
        let contact = sphere_sphere_contact(
            a,
            Pose3::new(Vec3::ZERO, 3.0),
            b,
            Pose3::new(Vec3::new(1.0, 0.0, 0.0), 1.0),
        )
        .expect("containment contact must exist");

        assert_eq!(contact.penetration, Some(3.0));
        assert_eq!(contact.normal, Vec3::X);
    }

    #[test]
    fn build_narrowphase_3d_contacts_is_deterministic() {
        let e1 = Entity::from_raw(11);
        let e2 = Entity::from_raw(7);
        let e3 = Entity::from_raw(29);

        // Forzamos candidatos como lo haría broadphase real.
        let mut poses = HashMap::new();
        poses.insert(e1, Pose3::new(Vec3::new(0.0, 0.0, 0.0), 1.0));
        poses.insert(e2, Pose3::new(Vec3::new(1.5, 0.0, 0.0), 1.0));
        poses.insert(e3, Pose3::new(Vec3::new(10.0, 0.0, 0.0), 1.0));
        let backend = TestBroadphase {
            pairs: vec![
                SpatialCandidatePair::new(e3, e1),
                SpatialCandidatePair::new(e2, e1),
                SpatialCandidatePair::new(e3, e2),
            ],
        };

        let first = build_narrowphase_3d_contacts(&backend, &poses);
        let second = build_narrowphase_3d_contacts(&backend, &poses);

        assert_eq!(first, second);
        assert_eq!(first.len(), 1);
        assert_eq!(first[0].a, Entity::from_raw(7));
        assert_eq!(first[0].b, Entity::from_raw(11));
    }
}
