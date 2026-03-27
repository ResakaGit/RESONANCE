use bevy::prelude::*;

use crate::blueprint::constants::{
    ALLOMETRIC_MAX_RADIUS_FACTOR, LIFECYCLE_HYSTERESIS_TICKS, METABOLIC_STARVATION_BASE_THRESHOLD_QE,
};
use crate::blueprint::equations::{
    growth_progress, infer_lifecycle_stage, lifecycle_stage_with_hysteresis, metabolic_viability,
    starvation_threshold,
};
use crate::layers::{
    AllometricRadiusAnchor, BaseEnergy, CapabilitySet, GrowthBudget, InferenceProfile, LifecycleStageCache,
    SpatialVolume,
};
use crate::worldgen::shape_inference::PendingGrowthMorphRebuild;

#[derive(Clone, Copy)]
struct LifecycleTransition {
    next_stage: crate::layers::LifecycleStage,
    next_candidate: Option<crate::layers::LifecycleStage>,
    next_candidate_ticks: u16,
    next_ticks_in_stage: u16,
    stage_changed: bool,
}

#[inline]
fn infer_lifecycle_transition(
    energy: &BaseEnergy,
    volume: &SpatialVolume,
    anchor: Option<&AllometricRadiusAnchor>,
    growth: &GrowthBudget,
    caps: &CapabilitySet,
    profile: Option<&InferenceProfile>,
    cache: &LifecycleStageCache,
) -> LifecycleTransition {
    let resilience = InferenceProfile::resilience_effective(profile);
    let threshold = starvation_threshold(METABOLIC_STARVATION_BASE_THRESHOLD_QE, resilience);
    let viability = metabolic_viability(energy.qe(), threshold);
    let base_radius = anchor.map(|a| a.base_radius).unwrap_or(volume.radius);
    let progress = growth_progress(volume.radius, base_radius, ALLOMETRIC_MAX_RADIUS_FACTOR);
    let inferred = infer_lifecycle_stage(
        viability,
        progress,
        growth.biomass_available,
        caps.can_reproduce(),
    );
    let (next_stage, next_candidate, next_candidate_ticks) = lifecycle_stage_with_hysteresis(
        cache.stage,
        inferred,
        cache.candidate_stage,
        cache.candidate_ticks,
        LIFECYCLE_HYSTERESIS_TICKS,
    );
    let stage_changed = cache.stage != next_stage;
    let next_ticks_in_stage = if stage_changed {
        0
    } else {
        cache.ticks_in_stage.saturating_add(1)
    };
    LifecycleTransition {
        next_stage,
        next_candidate,
        next_candidate_ticks,
        next_ticks_in_stage,
        stage_changed,
    }
}

/// Inserta cache de ciclo de vida en entidades vivas con crecimiento/capacidades.
/// Corre en MorphologicalLayer antes de inferencia de ciclo.
pub fn lifecycle_stage_init_system(
    mut commands: Commands,
    query: Query<Entity, (With<GrowthBudget>, With<CapabilitySet>, Without<LifecycleStageCache>)>,
) {
    for entity in &query {
        commands.entity(entity).insert(LifecycleStageCache::default());
    }
}

// NOTE: 8 component types justified — lifecycle reads full entity state for stage transition inference.
// Splitting would create ordering hazards; all fields are read-only except LifecycleStageCache (&mut) and PendingGrowthMorphRebuild (inserted via Commands).
/// Infiere etapa funcional de ciclo de vida y actualiza `LifecycleStageCache` con histéresis.
/// Una transformación: estado energético/morfológico -> cache de etapa.
pub fn lifecycle_stage_inference_system(
    mut commands: Commands,
    mut query: Query<
        (
            Entity,
            &BaseEnergy,
            &SpatialVolume,
            Option<&AllometricRadiusAnchor>,
            &GrowthBudget,
            &CapabilitySet,
            Option<&InferenceProfile>,
            &mut LifecycleStageCache,
        ),
        With<LifecycleStageCache>,
    >,
) {
    for (entity, energy, volume, anchor, growth, caps, profile, mut cache) in &mut query {
        let transition =
            infer_lifecycle_transition(energy, volume, anchor, growth, caps, profile, &cache);
        if transition.stage_changed
            || cache.candidate_stage != transition.next_candidate
            || cache.candidate_ticks != transition.next_candidate_ticks
            || cache.ticks_in_stage != transition.next_ticks_in_stage
        {
            cache.stage = transition.next_stage;
            cache.candidate_stage = transition.next_candidate;
            cache.candidate_ticks = transition.next_candidate_ticks;
            cache.ticks_in_stage = transition.next_ticks_in_stage;
            if transition.stage_changed {
                commands
                    .entity(entity)
                    .insert(PendingGrowthMorphRebuild);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{lifecycle_stage_inference_system, lifecycle_stage_init_system};
    use crate::layers::{
        AllometricRadiusAnchor, BaseEnergy, CapabilitySet, GrowthBudget, LifecycleStage, LifecycleStageCache,
        SpatialVolume,
    };
    use crate::worldgen::shape_inference::PendingGrowthMorphRebuild;
    use bevy::prelude::*;

    #[test]
    fn lifecycle_stage_init_inserts_cache_when_missing() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_systems(Update, lifecycle_stage_init_system);
        let entity = app
            .world_mut()
            .spawn((GrowthBudget::new(1.0, 0, 1.0), CapabilitySet::default()))
            .id();

        app.update();

        assert!(
            app.world().entity(entity).contains::<LifecycleStageCache>(),
            "debe insertar cache de lifecycle"
        );
    }

    #[test]
    fn lifecycle_stage_init_is_idempotent_when_cache_exists() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_systems(Update, lifecycle_stage_init_system);
        let entity = app
            .world_mut()
            .spawn((
                GrowthBudget::new(1.0, 0, 1.0),
                CapabilitySet::default(),
                LifecycleStageCache {
                    stage: LifecycleStage::Growing,
                    ticks_in_stage: 42,
                    candidate_stage: Some(LifecycleStage::Mature),
                    candidate_ticks: 3,
                },
            ))
            .id();

        app.update();

        let cache = app
            .world()
            .entity(entity)
            .get::<LifecycleStageCache>()
            .copied()
            .expect("cache debe existir");
        assert_eq!(cache.stage, LifecycleStage::Growing);
        assert_eq!(cache.ticks_in_stage, 42);
        assert_eq!(cache.candidate_stage, Some(LifecycleStage::Mature));
        assert_eq!(cache.candidate_ticks, 3);
    }

    #[test]
    fn lifecycle_stage_inference_reaches_reproductive_after_hysteresis() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_systems(Update, lifecycle_stage_inference_system);
        let entity = app
            .world_mut()
            .spawn((
                BaseEnergy::new(20.0),
                SpatialVolume::new(1.0),
                AllometricRadiusAnchor::new(0.33),
                GrowthBudget::new(3.0, 0, 1.0),
                CapabilitySet::new(CapabilitySet::GROW | CapabilitySet::REPRODUCE),
                LifecycleStageCache::default(),
            ))
            .id();

        for _ in 0..12 {
            app.update();
        }

        let cache = app
            .world()
            .entity(entity)
            .get::<LifecycleStageCache>()
            .copied()
            .expect("cache debe existir");
        assert_eq!(cache.stage, LifecycleStage::Reproductive);
        assert!(cache.candidate_stage.is_none());
    }

    #[test]
    fn lifecycle_stage_inference_marks_pending_morph_rebuild_on_stage_change() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_systems(Update, lifecycle_stage_inference_system);
        let entity = app
            .world_mut()
            .spawn((
                BaseEnergy::new(20.0),
                SpatialVolume::new(1.0),
                AllometricRadiusAnchor::new(0.33),
                GrowthBudget::new(3.0, 0, 1.0),
                CapabilitySet::new(CapabilitySet::GROW | CapabilitySet::REPRODUCE),
                LifecycleStageCache::default(),
            ))
            .id();

        for _ in 0..12 {
            app.update();
        }

        assert!(
            app.world()
                .entity(entity)
                .contains::<PendingGrowthMorphRebuild>(),
            "cuando cambia stage debe marcar rebuild morfológico"
        );
    }
}
