use bevy::prelude::*;

/// Gap 1 marker: entity mesh should be built by `entity_shape_inference_system`
/// from layer composition, not the default sphere from `sync_visual_from_sim_system`.
///
/// Add to any archetype whose shape should visually emerge from physics layers.
/// Consumed once `ShapeInferred` is also present on the entity.
#[derive(Component, Reflect, Debug, Clone, Copy, Default)]
#[reflect(Component)]
#[component(storage = "SparseSet")]
pub struct HasInferredShape;
