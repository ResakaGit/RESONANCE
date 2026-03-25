use bevy::prelude::*;

use crate::blueprint::{constants, equations};
use crate::layers::{
    AllometricRadiusAnchor, BaseEnergy, CapabilitySet, GrowthBudget, GrowthIntent, InferenceProfile,
    SpatialVolume,
};

/// Infiere intención de crecimiento a partir de estímulos/rasgos (stateless).
pub fn growth_intent_inference_system(
    mut commands: Commands,
    query: Query<
        (
            Entity,
            &GrowthBudget,
            &BaseEnergy,
            &SpatialVolume,
            &CapabilitySet,
            Option<&AllometricRadiusAnchor>,
            Option<&InferenceProfile>,
            Option<&GrowthIntent>,
        ),
    >,
) {
    for (entity, budget, energy, volume, capabilities, anchor_opt, profile_opt, current_intent_opt) in &query {
        if !capabilities.can_grow() {
            if current_intent_opt.is_some() {
                commands.entity(entity).remove::<GrowthIntent>();
            }
            continue;
        }
        if budget.biomass_available <= 0.0 {
            if current_intent_opt.is_some() {
                commands.entity(entity).remove::<GrowthIntent>();
            }
            continue;
        }

        let profile = profile_opt.copied().unwrap_or_default();
        let base_radius = anchor_opt.map(|a| a.base_radius).unwrap_or(volume.radius);
        let max_radius = equations::allometric_max_radius(base_radius, constants::ALLOMETRIC_MAX_RADIUS_FACTOR);
        let base_delta = equations::growth_size_feedback(budget.biomass_available, volume.radius, max_radius);
        let qe_norm = equations::normalized_qe(energy.qe(), constants::VISUAL_QE_REFERENCE);
        let delta_radius = equations::inferred_growth_delta(
            base_delta,
            profile.growth_bias,
            profile.resilience,
            qe_norm,
        );
        if delta_radius <= constants::VOLUME_WRITE_EPS {
            if current_intent_opt.is_some() {
                commands.entity(entity).remove::<GrowthIntent>();
            }
            continue;
        }
        let confidence = (budget.efficiency * profile.growth_bias).clamp(0.0, 1.0);
        let intent = GrowthIntent::new(delta_radius, confidence, profile.resilience);
        let must_write = current_intent_opt
            .map(|curr| {
                (curr.delta_radius - intent.delta_radius).abs() > constants::VOLUME_WRITE_EPS
                    || (curr.confidence - intent.confidence).abs() > constants::GROWTH_INTENT_FIELD_EPS
                    || (curr.structural_stability - intent.structural_stability).abs() > constants::GROWTH_INTENT_FIELD_EPS
            })
            .unwrap_or(true);
        if must_write {
            commands.entity(entity).insert(intent);
        }
    }
}

/// Limpia intents huérfanos cuando ya no hay presupuesto de crecimiento.
pub fn cleanup_orphan_growth_intent_system(
    mut commands: Commands,
    query: Query<Entity, (With<GrowthIntent>, Without<GrowthBudget>)>,
) {
    for entity in &query {
        commands.entity(entity).remove::<GrowthIntent>();
    }
}

#[cfg(test)]
mod tests {
    use crate::simulation::allometric_growth::allometric_growth_system;
    use super::{cleanup_orphan_growth_intent_system, growth_intent_inference_system};
    use crate::layers::{
        BaseEnergy, CapabilitySet, GrowthBudget, GrowthIntent, InferenceProfile, SpatialVolume,
    };
    use bevy::prelude::*;

    #[test]
    fn growth_intent_inference_emits_intent_when_growth_is_possible() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_systems(Update, growth_intent_inference_system);
        let entity = app
            .world_mut()
            .spawn((
                GrowthBudget::new(1.0, 0, 0.9),
                BaseEnergy::new(100.0),
                SpatialVolume::new(0.5),
                CapabilitySet::new(CapabilitySet::GROW),
                InferenceProfile::new(0.8, 0.2, 0.4, 0.7),
            ))
            .id();
        app.update();
        assert!(app.world().entity(entity).contains::<GrowthIntent>());
    }

    #[test]
    fn growth_intent_inference_removes_intent_when_growth_disabled() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_systems(Update, growth_intent_inference_system);
        let entity = app
            .world_mut()
            .spawn((
                GrowthBudget::new(1.0, 0, 0.9),
                BaseEnergy::new(100.0),
                SpatialVolume::new(0.5),
                CapabilitySet::new(CapabilitySet::MOVE),
                GrowthIntent::new(0.5, 1.0, 1.0),
            ))
            .id();
        app.update();
        assert!(!app.world().entity(entity).contains::<GrowthIntent>());
    }

    #[test]
    fn cleanup_orphan_growth_intent_removes_intent_without_budget() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_systems(Update, cleanup_orphan_growth_intent_system);
        let entity = app
            .world_mut()
            .spawn((SpatialVolume::new(0.5), GrowthIntent::new(0.2, 1.0, 1.0)))
            .id();
        app.update();
        assert!(!app.world().entity(entity).contains::<GrowthIntent>());
    }

    #[test]
    fn capability_without_grow_prevents_radius_change_in_pipeline() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_systems(
            Update,
            (growth_intent_inference_system, allometric_growth_system).chain(),
        );
        let entity = app
            .world_mut()
            .spawn((
                GrowthBudget::new(1.0, 0, 0.9),
                BaseEnergy::new(100.0),
                SpatialVolume::new(0.5),
                CapabilitySet::new(CapabilitySet::MOVE),
            ))
            .id();
        let before = app.world().entity(entity).get::<SpatialVolume>().unwrap().radius;
        app.update();
        let after = app.world().entity(entity).get::<SpatialVolume>().unwrap().radius;
        assert!((after - before).abs() < 1e-6, "before={before} after={after}");
    }
}
