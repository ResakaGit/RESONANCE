//! AD-4: Axiomatic cell division — entity splits when internal field disconnects.
//!
//! Split condition: valley.qe ≤ 0 (Axiom 1 pure).
//! Conservation: sum(children) ≤ sum(parent) (Axiom 2).
//! No thresholds. No cooldowns. No reproduction flags.
//! Division is physics, not a decision.
//!
//! Phase: MorphologicalLayer (after internal_field_diffusion, before abiogenesis).

use bevy::prelude::*;

use crate::blueprint::equations::derived_thresholds as dt;
use crate::blueprint::equations::field_division::{
    child_viable, find_valleys, is_split_viable, split_field_at, valley_count,
};
use crate::entities::component_groups as cg;
use crate::layers::{
    BaseEnergy, InferenceProfile, InternalEnergyField, MatterCoherence, OscillatorySignature,
    SpatialVolume, StructuralLink,
};
use crate::runtime_platform::simulation_tick::SimulationClock;
use crate::simulation::emergence::culture::CulturalMemory;

/// Max splits per tick (structural: prevents frame spikes).
const SPLIT_BUDGET_PER_TICK: usize = 4;

/// Detects valleys ≤ 0 in internal fields and splits entities.
/// Cache: uses `Changed<InternalEnergyField>` — only checks entities whose field changed.
pub fn axiomatic_split_system(
    mut commands: Commands,
    query: Query<
        (
            Entity,
            &InternalEnergyField,
            &BaseEnergy,
            &Transform,
            &OscillatorySignature,
            &SpatialVolume,
            &MatterCoherence,
            Option<&CulturalMemory>,
        ),
        Changed<InternalEnergyField>,
    >,
    clock: Res<SimulationClock>,
) {
    let mut splits = 0usize;

    for (entity, field, energy, transform, osc, volume, _matter, culture) in &query {
        if splits >= SPLIT_BUDGET_PER_TICK {
            break;
        }
        if energy.is_dead() {
            continue;
        }

        let valleys = find_valleys(&field.nodes);
        let n_valleys = valley_count(&valleys);
        if n_valleys == 0 {
            continue;
        }

        // Find first viable split point (valley ≤ 0).
        let Some(&(valley_idx, _)) = valleys[..n_valleys]
            .iter()
            .find(|(idx, _)| is_split_viable(&field.nodes, *idx))
        else {
            continue;
        };

        let (left_nodes, right_nodes) = split_field_at(&field.nodes, valley_idx);

        // Both children must be viable (Axiom 1: enough energy to exist).
        if !child_viable(&left_nodes) || !child_viable(&right_nodes) {
            continue;
        }

        let left_qe: f32 = left_nodes.iter().sum();
        let right_qe: f32 = right_nodes.iter().sum();

        // Position offset: children separate along the body axis.
        let split_t = valley_idx as f32 / 7.0;
        let offset = Vec3::new(
            volume.radius * split_t,
            0.0,
            volume.radius * (1.0 - split_t),
        );
        let pos_left = transform.translation - offset * 0.5;
        let pos_right = transform.translation + offset * 0.5;

        // Frequency: slight drift proportional to field asymmetry (Axiom 8).
        let freq = osc.frequency_hz();
        let asymmetry = (left_qe - right_qe) / (left_qe + right_qe).max(1e-6);
        let freq_left = freq * (1.0 + asymmetry * 0.05);
        let freq_right = freq * (1.0 - asymmetry * 0.05);

        let radius_left = volume.radius * (left_qe / energy.qe().max(1e-6)).sqrt();
        let radius_right = volume.radius * (right_qe / energy.qe().max(1e-6)).sqrt();

        let is_mobile = left_qe > dt::self_sustaining_qe_min() * 1.5;

        // Specialization: larger child → growth (interior), smaller → mobility (exterior).
        // Bias = energy fraction: dominant child grows, minor child moves (Axiom 1 + 4).
        let total = (left_qe + right_qe).max(1e-6);
        let left_frac = left_qe / total;
        let right_frac = right_qe / total;
        let profile_left = InferenceProfile::new(left_frac, 1.0 - left_frac, left_frac, left_frac);
        let profile_right =
            InferenceProfile::new(right_frac, 1.0 - right_frac, right_frac, right_frac);

        // Spawn child A (left nodes).
        let child_a = commands
            .spawn((
                cg::physical_components(
                    left_qe,
                    radius_left,
                    freq_left,
                    crate::math_types::Vec2::new(pos_left.x, pos_left.z),
                ),
                InternalEnergyField { nodes: left_nodes },
                cg::lifecycle_components(clock.tick_id, is_mobile),
                profile_left,
            ))
            .id();

        // Spawn child B (right nodes).
        let child_b = commands
            .spawn((
                cg::physical_components(
                    right_qe,
                    radius_right,
                    freq_right,
                    crate::math_types::Vec2::new(pos_right.x, pos_right.z),
                ),
                InternalEnergyField { nodes: right_nodes },
                cg::lifecycle_components(clock.tick_id, is_mobile),
                profile_right,
            ))
            .id();

        // Multicelularity: structural bond between siblings (Axiom 7: spring at distance).
        // rest_length = sum of radii, stiffness = thermal conductivity, break = bond energy.
        let rest_len = radius_left + radius_right;
        let stiffness = dt::materialized_thermal_conductivity();
        let break_stress = dt::materialized_bond_energy();
        commands.entity(child_a).insert(StructuralLink::new(
            child_b,
            rest_len,
            stiffness,
            break_stress,
        ));
        commands.entity(child_b).insert(StructuralLink::new(
            child_a,
            rest_len,
            stiffness,
            break_stress,
        ));

        // Cultural inheritance: larger child gets parent's memes.
        if let Some(culture) = culture {
            let heir = if left_qe >= right_qe {
                child_a
            } else {
                child_b
            };
            commands.entity(heir).insert(culture.clone());
        }

        // Despawn parent (Axiom 2: parent ceases to exist, children persist as linked pair).
        commands.entity(entity).despawn();
        splits += 1;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn split_budget_is_positive() {
        assert!(SPLIT_BUDGET_PER_TICK > 0);
    }
}
