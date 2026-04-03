//! AD-2: Internal energy field diffusion — stateless per-tick system.
//!
//! Diffuses energy between adjacent nodes of InternalEnergyField.
//! Uses existing equations (diffusion_delta, dissipation_from_state).
//! Cache optimization: only processes entities where BaseEnergy changed.
//!
//! Phase: MorphologicalLayer (before split detection).

use bevy::prelude::*;

use crate::blueprint::equations::{diffusion_delta, dissipation_from_state};
use crate::layers::{BaseEnergy, InternalEnergyField, MatterCoherence};

/// Diffuses energy between adjacent internal field nodes.
/// Uses `Changed<BaseEnergy>` as cache filter — skip stable entities.
pub fn internal_field_diffusion_system(
    mut query: Query<
        (&mut InternalEnergyField, &BaseEnergy, &MatterCoherence),
        Changed<BaseEnergy>,
    >,
) {
    for (mut field, energy, matter) in &mut query {
        if energy.is_dead() {
            continue;
        }

        let k = dissipation_from_state(matter.state());
        let dt = 1.0;

        // Sync total: field should match entity's qe (intake may have changed it).
        let field_total = field.total();
        let entity_qe = energy.qe();
        if field_total > 0.0 && (field_total - entity_qe).abs() > 1e-3 {
            let scale = entity_qe / field_total;
            for node in &mut field.nodes {
                *node *= scale;
            }
        }

        // Diffusion between adjacent nodes (Axiom 4).
        for i in 0..7 {
            let delta = diffusion_delta(field.nodes[i], field.nodes[i + 1], k, dt);
            field.nodes[i] -= delta;
            field.nodes[i + 1] += delta;
        }

        // Per-node dissipation (Axiom 4: entropy increases).
        for node in &mut field.nodes {
            let loss = *node * k * 0.1; // fraction of dissipation applied internally
            *node = (*node - loss).max(0.0);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layers::MatterState;

    fn make_field(nodes: [f32; 8]) -> InternalEnergyField {
        InternalEnergyField { nodes }
    }

    #[test]
    fn diffusion_equalizes_gradient() {
        let mut field = make_field([20.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0]);
        let k = dissipation_from_state(MatterState::Solid);
        let before_max = field.nodes[0];
        for _ in 0..50 {
            for i in 0..7 {
                let delta = diffusion_delta(field.nodes[i], field.nodes[i + 1], k, 1.0);
                field.nodes[i] -= delta;
                field.nodes[i + 1] += delta;
            }
        }
        let after_max = field.nodes.iter().copied().fold(0.0f32, f32::max);
        assert!(
            after_max < before_max,
            "diffusion should reduce peak: {after_max} < {before_max}"
        );
    }

    #[test]
    fn uniform_stays_uniform() {
        let field = make_field([10.0; 8]);
        let k = dissipation_from_state(MatterState::Solid);
        let mut nodes = field.nodes;
        for i in 0..7 {
            let delta = diffusion_delta(nodes[i], nodes[i + 1], k, 1.0);
            nodes[i] -= delta;
            nodes[i + 1] += delta;
        }
        for n in &nodes {
            assert!((*n - 10.0).abs() < 1e-3, "uniform should stay: {n}");
        }
    }
}
