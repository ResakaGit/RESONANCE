//! Gap 3 — MG-10: body plan layout inference for MOVE-capable entities.
//!
//! Computes bilateral quadruped attachment positions from entity radius and
//! mobility_bias. Populates `BodyPlanLayout` so the organ mesh pipeline can
//! place legs, head, and tail without hard-coded offsets.

use bevy::prelude::*;

use crate::blueprint::equations::bilateral_quadruped_attachments;
use crate::layers::{AmbientPressure, BodyPlanLayout, CapabilitySet, HasInferredShape, InferenceProfile, SpatialVolume};

/// Fallback: populates `BodyPlanLayout` for MOVE entities WITHOUT AmbientPressure (L6).
///
/// Entities WITH L6 are handled by `constructal_body_plan_system` (thermodynamic inference).
/// Re-runs whenever `SpatialVolume` or `InferenceProfile` changes.
pub fn body_plan_layout_inference_system(
    mut commands: Commands,
    query: Query<
        (Entity, &SpatialVolume, Option<&InferenceProfile>),
        (
            With<HasInferredShape>,
            With<CapabilitySet>,
            Without<AmbientPressure>,
            Or<(Changed<SpatialVolume>, Changed<InferenceProfile>, Without<BodyPlanLayout>)>,
        ),
    >,
    cap_query: Query<&CapabilitySet>,
) {
    for (entity, volume, profile_opt) in &query {
        let Ok(caps) = cap_query.get(entity) else { continue; };
        if !caps.has(CapabilitySet::MOVE) { continue; }

        let mobility = profile_opt.map(|p| p.mobility_bias).unwrap_or(0.5);
        let (positions, directions, symmetry, count) =
            bilateral_quadruped_attachments(volume.radius, mobility);

        let layout = BodyPlanLayout::new(positions, directions, symmetry, count);
        commands.entity(entity).insert(layout);
    }
}

#[cfg(test)]
mod tests {
    use bevy::prelude::*;

    use super::body_plan_layout_inference_system;
    use crate::layers::{BodyPlanLayout, CapabilitySet, HasInferredShape, InferenceProfile, SpatialVolume};

    fn test_app() -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app
    }

    #[test]
    fn inserts_body_plan_for_mobile_entity() {
        let mut app = test_app();
        app.add_systems(Update, body_plan_layout_inference_system);

        let entity = app.world_mut().spawn((
            SpatialVolume::new(0.5),
            InferenceProfile::new(0.8, 0.8, 0.3, 0.6),
            CapabilitySet::new(CapabilitySet::MOVE | CapabilitySet::GROW),
            HasInferredShape,
        )).id();
        app.update();

        assert!(
            app.world().entity(entity).contains::<BodyPlanLayout>(),
            "BodyPlanLayout should be inserted for MOVE-capable entity"
        );
    }

    #[test]
    fn does_not_insert_for_non_mobile_entity() {
        let mut app = test_app();
        app.add_systems(Update, body_plan_layout_inference_system);

        let entity = app.world_mut().spawn((
            SpatialVolume::new(0.5),
            CapabilitySet::new(CapabilitySet::GROW | CapabilitySet::BRANCH),
            HasInferredShape,
        )).id();
        app.update();

        assert!(
            !app.world().entity(entity).contains::<BodyPlanLayout>(),
            "BodyPlanLayout must not be inserted for non-MOVE entity"
        );
    }

    #[test]
    fn body_plan_active_count_within_max_organs() {
        use crate::layers::organ::MAX_ORGANS_PER_ENTITY;

        let mut app = test_app();
        app.add_systems(Update, body_plan_layout_inference_system);

        let entity = app.world_mut().spawn((
            SpatialVolume::new(1.0),
            CapabilitySet::new(CapabilitySet::MOVE),
            HasInferredShape,
        )).id();
        app.update();

        let layout = app.world().entity(entity).get::<BodyPlanLayout>().unwrap();
        assert!(
            layout.active_count() as usize <= MAX_ORGANS_PER_ENTITY,
            "active_count {} must not exceed MAX_ORGANS_PER_ENTITY {}",
            layout.active_count(),
            MAX_ORGANS_PER_ENTITY
        );
    }
}
