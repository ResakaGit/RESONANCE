//! ET-2: Theory of Mind — updates OtherModelSet predictions from observed neighbors.
//!
//! Each BehavioralAgent observes nearby entities and maintains up to 4 mental models
//! (OtherModel) of their oscillatory frequency. Maintenance costs qe (Axiom 4).

use bevy::prelude::*;

use crate::blueprint::equations::emergence::other_model::{
    is_model_worth_maintaining, model_accuracy, model_maintenance_cost, update_prediction,
};
use crate::layers::other_model::{MAX_MODELS, OtherModel, OtherModelSet};
use crate::layers::{BaseEnergy, BehavioralAgent, OscillatorySignature};
use crate::world::SpatialIndex;

use crate::blueprint::constants::emergence::{
    MODEL_LEARNING_RATE, MODEL_MAX_FREQ_DEVIATION, MODEL_SCAN_RADIUS,
};

/// Updates mental models of nearby entities. Learns from observation, evicts unprofitable models.
pub fn theory_of_mind_update_system(
    mut query: Query<
        (Entity, &mut OtherModelSet, &Transform, &mut BaseEnergy),
        With<BehavioralAgent>,
    >,
    targets: Query<(&Transform, &OscillatorySignature)>,
    spatial: Res<SpatialIndex>,
) {
    for (entity, mut models, transform, mut energy) in &mut query {
        let pos = Vec2::new(transform.translation.x, transform.translation.z);
        let neighbors = spatial.query_radius(pos, MODEL_SCAN_RADIUS);

        for neighbor in &neighbors {
            if neighbor.entity == entity {
                continue;
            }
            let Ok((_, wave)) = targets.get(neighbor.entity) else {
                continue;
            };
            let actual_freq = wave.frequency_hz();
            let target_id = neighbor.entity.index();

            // Update existing model or create new one
            let mut found = false;
            for i in 0..models.model_count() {
                if models.models[i].target_id == target_id {
                    let old_pred = models.models[i].predicted_freq;
                    let new_pred = update_prediction(old_pred, actual_freq, MODEL_LEARNING_RATE);
                    if old_pred != new_pred {
                        models.models[i].predicted_freq = new_pred;
                    }
                    models.models[i].accuracy =
                        model_accuracy(new_pred, actual_freq, MODEL_MAX_FREQ_DEVIATION);
                    found = true;
                    break;
                }
            }

            if !found && models.model_count() < MAX_MODELS {
                let idx = models.model_count();
                models.models[idx] = OtherModel {
                    target_id,
                    predicted_freq: actual_freq,
                    accuracy: 0.1,
                    update_cost: 0.0,
                };
                models.model_count += 1;
            }
        }

        // Maintenance cost + eviction
        let base_cost = models.base_model_cost;
        let mut total_cost = 0.0f32;
        let mut i = 0usize;
        while i < models.model_count() {
            let acc = models.models[i].accuracy;
            let cost = model_maintenance_cost(acc, base_cost);
            total_cost += cost;
            if !is_model_worth_maintaining(acc * 2.0, cost) {
                // Evict: swap with last, decrement count
                let last = models.model_count() - 1;
                models.models[i] = models.models[last];
                models.models[last] = OtherModel::default();
                models.model_count -= 1;
                // Don't increment i — re-check swapped element
            } else {
                i += 1;
            }
        }

        if total_cost > 0.0 && energy.qe() > total_cost {
            let new_qe = energy.qe() - total_cost;
            if energy.qe() != new_qe {
                energy.set_qe(new_qe);
            }
        }
    }
}
