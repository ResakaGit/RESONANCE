//! ET-6: Epigenetic adaptation — environment modulates gene expression mask.

use bevy::prelude::*;

use crate::blueprint::equations::emergence::epigenetics::{
    gene_expression_threshold, should_express_gene, silencing_cost,
};
use crate::layers::{AmbientPressure, BaseEnergy, EpigeneticState};

/// Environmental signal → gene expression mask update.
pub fn epigenetic_adaptation_system(
    time: Res<Time>,
    mut query: Query<(&mut EpigeneticState, &AmbientPressure, &mut BaseEnergy)>,
) {
    let dt = time.delta_secs();
    for (mut epi, pressure, mut energy) in &mut query {
        let env_signal = pressure.terrain_viscosity;
        let rate = epi.adaptation_speed;

        for dim in 0..4usize {
            let current = epi.expression_mask[dim];
            let threshold = gene_expression_threshold(dim as f32);
            let should_express = should_express_gene(env_signal, threshold, current);

            let target = if should_express { 1.0 } else { 0.0 };
            let new_val = (current + (target - current) * rate * dt).clamp(0.0, 1.0);

            if (current - new_val).abs() > 1e-4 {
                epi.expression_mask[dim] = new_val;
            }
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
