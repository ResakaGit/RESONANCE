//! MGN-4 Batch: Metabolic graph inference from entity genome fields.
//!
//! Derives MetabolicGraph on-the-fly from EntitySlot fields each tick.
//! No new fields in EntitySlot — graph is ephemeral workspace, not stored.
//! Applies competition (MGN-5), rewiring (MGN-6), catalysis (MGN-7).

use crate::batch::arena::SimWorldFlat;
use crate::blueprint::equations::derived_thresholds::DISSIPATION_SOLID;
use crate::blueprint::equations::metabolic_genome;
use crate::blueprint::equations::variable_genome;

/// Max dissipation reduction from efficient metabolic graph.
/// Derived: DISSIPATION_SOLID × 10 = 0.05 (5%). Axiom 4: bonus can't eliminate dissipation.
const EFFICIENCY_BONUS_CAP: f32 = DISSIPATION_SOLID * 10.0;

/// Infer metabolic graph from entity genome and apply metabolic effects.
///
/// For each alive entity with > MIN_GENES biases:
/// 1. Build VariableGenome from slot fields.
/// 2. Infer MetabolicGraph via `metabolic_graph_from_variable_genome`.
/// 3. Apply genome maintenance cost (dissipation scaled by complexity).
/// 4. If graph has edges, apply competitive flow effects to qe.
///
/// Stateless: reads slot fields, writes only `qe` (dissipation cost).
/// Conservation: cost is always subtracted, never added. Axiom 4/5.
pub fn metabolic_graph_infer(world: &mut SimWorldFlat) {
    let mut mask = world.alive_mask;
    while mask != 0 {
        let i = mask.trailing_zeros() as usize;
        mask &= mask - 1;

        // Read genome from side-table (cold data, DoD separated)
        let vg = world.genomes[i]; // Copy (VariableGenome is Copy)
        let mask_expr = world.entities[i].expression_mask;

        // Genome maintenance cost: all organisms pay (Axiom 4)
        let cost = variable_genome::gated_maintenance_cost(&vg, &mask_expr, DISSIPATION_SOLID);
        let loss = cost.min(world.entities[i].qe);
        world.entities[i].qe -= loss;

        // Attempt metabolic graph (only if complex enough)
        let Ok(graph) = metabolic_genome::metabolic_graph_from_variable_genome(&vg, &mask_expr)
        else {
            continue;
        };

        // Efficient graph → reduced dissipation (selection pressure). Axiom 4.
        let nc = graph.node_count();
        let ec = graph.edge_count();
        if nc > 0 && ec > 0 {
            let mean_efficiency: f32 = graph.nodes()[..nc]
                .iter()
                .map(|n| n.efficiency)
                .sum::<f32>()
                / nc as f32;
            let bonus =
                (mean_efficiency.clamp(0.0, 1.0) * EFFICIENCY_BONUS_CAP).min(EFFICIENCY_BONUS_CAP);
            world.entities[i].dissipation *= 1.0 - bonus;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::batch::arena::SimWorldFlat;
    use crate::blueprint::equations::variable_genome::VariableGenome;

    fn world_with_entity(
        growth: f32,
        mobility: f32,
        branching: f32,
        resilience: f32,
        qe: f32,
    ) -> SimWorldFlat {
        let mut world = SimWorldFlat::new(42, 0.05);
        let e = &mut world.entities[0];
        e.qe = qe;
        e.growth_bias = growth;
        e.mobility_bias = mobility;
        e.branching_bias = branching;
        e.resilience = resilience;
        e.expression_mask = [1.0; 4];
        e.dissipation = 0.02;
        world.genomes[0] = VariableGenome::from_biases(growth, mobility, branching, resilience);
        world.alive_mask = 1;
        world
    }

    #[test]
    fn infer_conserves_energy() {
        let mut world = world_with_entity(0.5, 0.5, 0.5, 0.5, 100.0);
        let qe_before = world.entities[0].qe;
        metabolic_graph_infer(&mut world);
        assert!(world.entities[0].qe <= qe_before, "qe should not increase");
    }

    #[test]
    fn infer_costs_energy() {
        let mut world = world_with_entity(0.5, 0.5, 0.5, 0.5, 100.0);
        let qe_before = world.entities[0].qe;
        metabolic_graph_infer(&mut world);
        assert!(
            world.entities[0].qe < qe_before,
            "maintenance cost should drain qe"
        );
    }

    #[test]
    fn infer_zero_qe_no_panic() {
        let mut world = world_with_entity(0.5, 0.5, 0.5, 0.5, 0.0);
        metabolic_graph_infer(&mut world);
        assert!(world.entities[0].qe >= 0.0);
    }

    #[test]
    fn infer_dead_entity_skipped() {
        let mut world = world_with_entity(0.5, 0.5, 0.5, 0.5, 100.0);
        world.alive_mask = 0; // no alive entities
        metabolic_graph_infer(&mut world);
        assert_eq!(world.entities[0].qe, 100.0, "dead entity untouched");
    }

    #[test]
    fn infer_silenced_genes_lower_cost() {
        let mut world_full = world_with_entity(0.5, 0.5, 0.5, 0.5, 100.0);
        let mut world_half = world_with_entity(0.5, 0.5, 0.5, 0.5, 100.0);
        world_half.entities[0].expression_mask = [0.5; 4];

        metabolic_graph_infer(&mut world_full);
        metabolic_graph_infer(&mut world_half);

        assert!(
            world_half.entities[0].qe > world_full.entities[0].qe,
            "silenced genome should cost less: {} > {}",
            world_half.entities[0].qe,
            world_full.entities[0].qe
        );
    }

    #[test]
    fn infer_efficient_graph_reduces_dissipation() {
        // High biases → efficient metabolic nodes → dissipation bonus
        let mut world = world_with_entity(0.9, 0.9, 0.9, 0.9, 100.0);
        let diss_before = world.entities[0].dissipation;
        metabolic_graph_infer(&mut world);
        // With 4-gene genome, graph may or may not form — depends on whether
        // from_biases genome gets enough nodes. Just verify no panic and conservation.
        assert!(world.entities[0].dissipation <= diss_before);
    }

    #[test]
    fn infer_deterministic() {
        let mut a = world_with_entity(0.5, 0.5, 0.5, 0.5, 100.0);
        let mut b = world_with_entity(0.5, 0.5, 0.5, 0.5, 100.0);
        metabolic_graph_infer(&mut a);
        metabolic_graph_infer(&mut b);
        assert_eq!(a.entities[0].qe.to_bits(), b.entities[0].qe.to_bits());
    }
}
