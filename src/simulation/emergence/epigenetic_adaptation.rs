//! ET-6: Epigenetic adaptation — environment modulates gene expression mask.
//!
//! Uses `Converged<EpigeneticState>` to skip entities whose expression mask
//! stabilized and whose environment hasn't changed (~85% skip rate for flora). ADR-017.

use bevy::prelude::*;

use crate::blueprint::equations::emergence::epigenetics::{
    gene_expression_threshold, should_express_gene, silencing_cost,
};
use crate::layers::converged::{hash_f32, Converged};
use crate::layers::{AmbientPressure, BaseEnergy, EpigeneticState};

/// Environmental signal → gene expression mask update.
///
/// Convergence detection: skip entities where all 4 dims are within 1e-4 of target
/// AND environment (terrain_viscosity) hasn't changed since convergence.
pub fn epigenetic_adaptation_system(
    time: Res<Time>,
    mut commands: Commands,
    mut query: Query<(
        Entity,
        &mut EpigeneticState,
        &AmbientPressure,
        &mut BaseEnergy,
        Option<&Converged<EpigeneticState>>,
    )>,
) {
    let dt = time.delta_secs();
    for (entity, mut epi, pressure, mut energy, converged) in &mut query {
        let env_signal = pressure.terrain_viscosity;
        let env_hash = hash_f32(env_signal);

        // Skip converged entities whose environment hasn't changed.
        if let Some(conv) = converged {
            if conv.is_valid(env_hash) {
                continue;
            }
            commands.entity(entity).remove::<Converged<EpigeneticState>>();
        }

        let rate = epi.adaptation_speed;
        let mut all_converged = true;

        for dim in 0..4usize {
            let current = epi.expression_mask[dim];
            let threshold = gene_expression_threshold(dim as f32);
            let should_express = should_express_gene(env_signal, threshold, current);

            let target = if should_express { 1.0 } else { 0.0 };
            let new_val = (current + (target - current) * rate * dt).clamp(0.0, 1.0);

            if (current - new_val).abs() > 1e-4 {
                epi.expression_mask[dim] = new_val;
                all_converged = false;
            } else if (current - target).abs() > 1e-4 {
                all_converged = false;
            }
        }

        if all_converged {
            commands
                .entity(entity)
                .insert(Converged::<EpigeneticState>::new(env_hash));
        }

        // silencing_cost(gene_complexity, silencing_rate) — count silenced genes as complexity
        let silenced = epi.expression_mask.iter().filter(|&&v| v < 0.5).count() as f32;
        let cost = silencing_cost(silenced, epi.silencing_cost);
        if cost > 0.0 && energy.qe() > cost {
            let new_qe = energy.qe() - cost;
            if energy.qe() != new_qe {
                energy.set_qe(new_qe);
            }
        }
    }
}
