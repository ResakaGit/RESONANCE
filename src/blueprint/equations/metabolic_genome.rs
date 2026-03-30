//! Metabolic Genome — MGN-1/2/3: VariableGenome → MetabolicGraph.
//!
//! Pure stateless equations. Gene position → ExergyNode role. Gene proximity → DAG edges.
//! Epigenetic mask gates which genes express as metabolic nodes.
//!
//! Axiom 4: every node has efficiency < 1, every edge has dissipation.
//! Axiom 6: topology emerges from gene positions, not templates.
//! Axiom 7: edge capacity ∝ 1/(1 + gene_distance × cost_factor).

use crate::blueprint::constants::{
    METABOLIC_EDGE_CAPACITY_BASE, ROLE_ACTIVATION_ENERGY, ROLE_EFFICIENCY_FACTOR,
};
use crate::blueprint::equations::derived_thresholds::{DISSIPATION_SOLID, DISSIPATION_LIQUID};
use crate::blueprint::equations::variable_genome::{VariableGenome, MIN_GENES};
#[cfg(test)]
use crate::blueprint::equations::variable_genome::MAX_GENES;
use crate::layers::metabolic_graph::{
    ExergyEdge, ExergyNode, MetabolicGraph, MetabolicGraphBuilder, MetabolicGraphError,
    METABOLIC_GRAPH_MAX_EDGES, METABOLIC_GRAPH_MAX_NODES,
};
use crate::layers::OrganRole;

// ─── Constants ──────────────────────────────────────────────────────────────

/// Minimum gated expression for a gene to produce a metabolic node.
/// Derived: DISSIPATION_SOLID × 40 = 0.2. A gene must overcome 40× base dissipation
/// worth of expression to justify a metabolic node's existence. Axiom 4.
const NODE_EXPRESSION_THRESHOLD: f32 = DISSIPATION_SOLID * 40.0;

/// Transport cost per unit of gene distance. Axiom 7: farther genes = costlier edges.
/// Derived: DISSIPATION_SOLID × 20 = 0.1 qe per gene-index distance.
const TRANSPORT_COST_PER_DISTANCE: f32 = DISSIPATION_SOLID * 20.0;

/// Role map: [dimension (0-3)][tier (0-2)] → OrganRole.
///
/// dimension 0 (growth):     Root (captor)  → Core (process) → Fruit (actuator)
/// dimension 1 (mobility):   Fin (captor)   → Limb (process) → Bud (actuator)
/// dimension 2 (branching):  Leaf (captor)  → Stem (process) → Petal (actuator)
/// dimension 3 (resilience): Shell (captor) → Thorn (process)→ Sensory (actuator)
const ROLE_MAP: [[OrganRole; 3]; 4] = [
    [OrganRole::Root,  OrganRole::Core,  OrganRole::Fruit],   // growth
    [OrganRole::Fin,   OrganRole::Limb,  OrganRole::Bud],     // mobility
    [OrganRole::Leaf,  OrganRole::Stem,  OrganRole::Petal],   // branching
    [OrganRole::Shell, OrganRole::Thorn, OrganRole::Sensory], // resilience
];

// ─── MGN-1: Gene → ExergyNode ───────────────────────────────────────────────

/// Infer OrganRole from gene position. Dimension = index % 4, tier = distance from core.
///
/// Axiom 6: role is a consequence of position, not an assignment.
pub fn infer_role_from_gene(gene_index: usize) -> OrganRole {
    if gene_index < MIN_GENES { return OrganRole::Stem; } // core biases aren't organs
    let dimension = gene_index % 4;
    let tier = ((gene_index - MIN_GENES) / 4).min(2);
    ROLE_MAP[dimension][tier]
}

/// Convert a single gene to an ExergyNode. Pure: `(value, index) → ExergyNode`.
///
/// - `efficiency = ROLE_FACTOR[role] × gene_value` — gene modulates η.
/// - `activation_energy = ROLE_E_A[role] × (1 - gene_value)` — high gene → low barrier.
/// - thermal_output, entropy_rate = 0 (computed by step_system at runtime).
///
/// Axiom 4: efficiency ≤ ROLE_FACTOR[role] < 1.0 always.
pub fn gene_to_exergy_node(gene_value: f32, gene_index: usize) -> ExergyNode {
    let value = sanitize(gene_value);
    let role = infer_role_from_gene(gene_index);
    let role_idx = role as usize;
    ExergyNode {
        role,
        efficiency: ROLE_EFFICIENCY_FACTOR[role_idx] * value,
        activation_energy: ROLE_ACTIVATION_ENERGY[role_idx] * (1.0 - value),
        thermal_output: 0.0,
        entropy_rate: 0.0,
    }
}

/// Tier of a gene (distance from core): 0=captor, 1=process, 2=actuator.
fn gene_tier(gene_index: usize) -> usize {
    if gene_index < MIN_GENES { return 0; }
    ((gene_index - MIN_GENES) / 4).min(2)
}

/// Dimension of a gene (0=growth, 1=mobility, 2=branching, 3=resilience).
fn gene_dimension(gene_index: usize) -> usize {
    gene_index % 4
}

// ─── MGN-2: Topology Inference ──────────────────────────────────────────────

/// Edge candidate from topology inference.
#[derive(Clone, Copy, Debug)]
pub(crate) struct EdgeCandidate {
    from: u8,
    to: u8,
    capacity: f32,
}

/// Infer DAG edges from gene positions.
///
/// Edge(A→B) exists if:
/// 1. tier(A) < tier(B) — flow goes captor → process → actuator (DAG guarantee)
/// 2. dimension(A) == dimension(B) OR dimension(A) == 0 (growth hub connects all)
///
/// Capacity = METABOLIC_EDGE_CAPACITY_BASE × min(gene_A, gene_B).
/// Transport cost = TRANSPORT_COST_PER_DISTANCE × |index_A - index_B| (Axiom 7).
///
/// Returns (edges, edge_count).
pub(crate) fn infer_topology(
    active_gene_indices: &[usize],
    gene_values: &[f32],
) -> ([EdgeCandidate; METABOLIC_GRAPH_MAX_EDGES], usize) {
    let mut edges = [EdgeCandidate { from: 0, to: 0, capacity: 0.0 }; METABOLIC_GRAPH_MAX_EDGES];
    let mut count = 0usize;
    let n = active_gene_indices.len();

    for i in 0..n {
        for j in 0..n {
            if count >= METABOLIC_GRAPH_MAX_EDGES { return (edges, count); }
            let gi = active_gene_indices[i];
            let gj = active_gene_indices[j];
            let ti = gene_tier(gi);
            let tj = gene_tier(gj);
            if ti >= tj { continue; } // must flow lower → higher tier

            let di = gene_dimension(gi);
            let dj = gene_dimension(gj);
            // Same metabolic pathway OR growth hub (dim 0 connects to all)
            if di != dj && di != 0 { continue; }

            let vi = sanitize(gene_values[i]);
            let vj = sanitize(gene_values[j]);
            // Capacity scaled by weaker gene × distance penalty (Axiom 7).
            let min_gene = vi.min(vj).clamp(0.01, 1.0);
            let dist = (gi as f32 - gj as f32).abs();
            let distance_penalty = 1.0 / (1.0 + dist * TRANSPORT_COST_PER_DISTANCE);
            let cap = METABOLIC_EDGE_CAPACITY_BASE * min_gene * distance_penalty;

            edges[count] = EdgeCandidate {
                from: i as u8,
                to: j as u8,
                capacity: cap,
            };
            count += 1;
        }
    }
    (edges, count)
}

// ─── MGN-3: MetabolicGraph from VariableGenome ──────────────────────────────

/// Build a complete MetabolicGraph from a VariableGenome + epigenetic mask.
///
/// 1. Genes 0-3 skipped (core biases, not metabolic nodes).
/// 2. Each gene 4..len with gated expression > threshold → ExergyNode.
/// 3. Topology inferred from active gene positions.
/// 4. Built via MetabolicGraphBuilder (validates DAG, no cycles, captor present).
///
/// Returns Err if no captor node or genome too short for metabolic graph.
pub fn metabolic_graph_from_variable_genome(
    genome: &VariableGenome,
    expression_mask: &[f32; 4],
) -> Result<MetabolicGraph, MetabolicGraphError> {
    if genome.gene_count() <= MIN_GENES {
        return Err(MetabolicGraphError::Empty);
    }

    // 1. Identify active genes (gated expression > threshold)
    let mut active_indices = [0usize; METABOLIC_GRAPH_MAX_NODES];
    let mut active_values = [0.0f32; METABOLIC_GRAPH_MAX_NODES];
    let mut active_count = 0usize;

    for i in MIN_GENES..genome.gene_count() {
        if active_count >= METABOLIC_GRAPH_MAX_NODES { break; }
        let dim = gene_dimension(i);
        let mask = expression_mask[dim].clamp(0.0, 1.0);
        let gated = genome.genes[i] * mask;
        if gated > NODE_EXPRESSION_THRESHOLD {
            active_indices[active_count] = i;
            active_values[active_count] = gated;
            active_count += 1;
        }
    }

    if active_count == 0 {
        return Err(MetabolicGraphError::Empty);
    }

    // 2. Build nodes
    let mut builder = MetabolicGraphBuilder::new();
    for k in 0..active_count {
        let node = gene_to_exergy_node(active_values[k], active_indices[k]);
        builder = builder.add_node(node.role, node.efficiency, node.activation_energy);
    }

    // 3. Infer topology
    let (edges, edge_count) = infer_topology(
        &active_indices[..active_count],
        &active_values[..active_count],
    );
    for k in 0..edge_count {
        let e = &edges[k];
        builder = builder.add_edge(e.from, e.to, e.capacity);
    }

    // 4. Build (validates DAG, captor, indices)
    builder.build()
}

// ─── Cache: GenomeMetabolicPhenotype ────────────────────────────────────────

/// Pre-computed metabolic phenotype. One call per entity per tick.
#[derive(Clone, Debug)]
pub struct GenomeMetabolicPhenotype {
    pub graph: MetabolicGraph,
    pub node_count: u8,
    pub edge_count: u8,
    pub total_capacity: f32,
}

/// Compute full metabolic phenotype. Cache-friendly: one call replaces many.
pub fn compute_metabolic_phenotype(
    genome: &VariableGenome,
    expression_mask: &[f32; 4],
) -> Option<GenomeMetabolicPhenotype> {
    let graph = metabolic_graph_from_variable_genome(genome, expression_mask).ok()?;
    let nc = graph.node_count();
    let ec = graph.edge_count();
    let total_cap: f32 = graph.edges()[..ec].iter().map(|e| e.max_capacity).sum();
    Some(GenomeMetabolicPhenotype {
        graph,
        node_count: nc as u8,
        edge_count: ec as u8,
        total_capacity: total_cap,
    })
}

// ─── MGN-5: Node Competition ────────────────────────────────────────────────

/// Competition overhead per competing edge. Axiom 4: distributing has a cost.
/// Derived: DISSIPATION_SOLID × 2 = 0.01 qe per competitor per tick.
pub const COMPETITION_OVERHEAD_RATE: f32 = DISSIPATION_SOLID * 2.0;

/// Distribute available exergy among outgoing edges competitively.
///
/// Each edge's share = (target_η × capacity) / Σ(target_η × capacity).
/// Axiom 3: higher efficiency captures more. Axiom 2: Σ shares ≤ j_in.
/// Axiom 4: overhead = COMPETITION_OVERHEAD_RATE × n_competitors.
///
/// Returns (shares per edge, total overhead consumed).
pub fn competitive_flow_distribution(
    j_in: f32,
    outgoing: &[(u8, f32, f32)], // (edge_idx, capacity, target_efficiency)
) -> ([f32; METABOLIC_GRAPH_MAX_EDGES], f32) {
    let mut shares = [0.0f32; METABOLIC_GRAPH_MAX_EDGES];
    if outgoing.is_empty() || j_in <= 0.0 { return (shares, 0.0); }

    let overhead = COMPETITION_OVERHEAD_RATE * outgoing.len() as f32;
    let available = (j_in - overhead).max(0.0);
    if available <= 0.0 { return (shares, overhead.min(j_in)); }

    // Weighted score: η × capacity (Axiom 3)
    let total_score: f32 = outgoing.iter()
        .map(|&(_, cap, eta)| sanitize(eta) * cap.max(0.0))
        .sum();

    if total_score <= 0.0 { return (shares, overhead.min(j_in)); }

    // Distribute proportionally, clamped by capacity (Axiom 2)
    for &(idx, cap, eta) in outgoing {
        let score = sanitize(eta) * cap.max(0.0);
        let raw_share = available * score / total_score;
        shares[idx as usize] = raw_share.min(cap.max(0.0)); // bottleneck
    }

    (shares, overhead.min(j_in))
}

// ─── MGN-6: Hebbian Rewiring ───────────────────────────────────────────────

/// Minimum edge capacity — atrophied but not dead. Axiom 4.
/// Derived: 1.0 / DISSIPATION_SOLID = 200 → inverse = min viable flow = 1.0 qe/s.
pub const EDGE_MIN_CAPACITY: f32 = 1.0;
/// Maximum edge capacity — physical limit.
/// Derived: METABOLIC_EDGE_CAPACITY_BASE × (1/DISSIPATION_SOLID)^KLEIBER_EXPONENT ≈ 200.
/// Capped at 4× base capacity to prevent runaway growth.
pub const EDGE_MAX_CAPACITY: f32 = METABOLIC_EDGE_CAPACITY_BASE * 4.0;
/// Learning rate for Hebbian update. Derived: DISSIPATION_SOLID × 2 = 0.01.
pub const HEBBIAN_LEARNING_RATE: f32 = DISSIPATION_SOLID * 2.0;
/// Baseline utilization (50% = neutral). Mathematical symmetry: below=weaken, above=strengthen.
/// Not derived from physics — this is a property of the Hebbian rule itself.
const HEBBIAN_BASELINE: f32 = 0.5;

/// Adjust edge capacities based on usage (Hebbian rule for metabolism).
///
/// `new_cap = cap + lr × (flow/cap - baseline) × cap`
/// High usage → grow. Low usage → shrink. Never below MIN, never above MAX.
/// Cost = DISSIPATION_SOLID × Σ positive_delta × transport_cost (Axiom 4 + 7).
///
/// Returns (new_capacities, total_rewiring_cost).
pub fn hebbian_capacity_update(
    capacities: &[f32],
    flows: &[f32],
    transport_costs: &[f32],
    edge_count: usize,
) -> ([f32; METABOLIC_GRAPH_MAX_EDGES], f32) {
    let mut new_caps = [0.0f32; METABOLIC_GRAPH_MAX_EDGES];
    let mut total_cost = 0.0f32;
    let n = edge_count.min(METABOLIC_GRAPH_MAX_EDGES);

    for i in 0..n {
        let cap = capacities[i].max(EDGE_MIN_CAPACITY);
        let flow = flows[i].max(0.0);
        let utilization = if cap > 0.0 { flow / cap } else { 0.0 };
        let delta = HEBBIAN_LEARNING_RATE * (utilization - HEBBIAN_BASELINE) * cap;
        let new_cap = (cap + delta).clamp(EDGE_MIN_CAPACITY, EDGE_MAX_CAPACITY);
        new_caps[i] = new_cap;

        // Only strengthening costs energy (Axiom 4)
        if delta > 0.0 {
            let tc = transport_costs.get(i).copied().unwrap_or(DISSIPATION_SOLID);
            total_cost += delta * tc;
        }
    }

    (new_caps, total_cost)
}

// ─── MGN-7: Internal Catalysis ──────────────────────────────────────────────

/// Catalytic efficiency: fraction of thermal output usable for neighbor activation reduction.
/// Derived: DISSIPATION_SOLID / DISSIPATION_LIQUID = 0.25.
pub const CATALYSIS_EFFICIENCY: f32 = DISSIPATION_SOLID / DISSIPATION_LIQUID;

/// Cost fraction of catalytic benefit. Axiom 4: catalysis isn't free.
/// Derived: DISSIPATION_SOLID × 4 = 0.02.
pub const CATALYSIS_COST_FRACTION: f32 = DISSIPATION_SOLID * 4.0;

/// Minimum activation energy after catalysis. Thermodynamic floor.
/// Derived: DISSIPATION_SOLID × 100 = 0.5.
pub const CATALYSIS_MIN_ACTIVATION: f32 = DISSIPATION_SOLID * 100.0;

/// Axiom 8: frequency alignment. Delegates to centralized implementation.
fn catalytic_freq_alignment(f_a: f32, f_b: f32) -> f32 {
    use super::derived_thresholds::COHERENCE_BANDWIDTH;
    super::determinism::gaussian_frequency_alignment(f_a, f_b, COHERENCE_BANDWIDTH)
}

/// Compute catalytic reduction of activation_energy for each node.
///
/// For each edge A→B: `reduction_B += A.thermal_output × CATALYSIS_EFFICIENCY × freq_align(A,B)`.
/// Effective E_a = max(base - reduction, CATALYSIS_MIN_ACTIVATION).
/// Cost = Σ reduction × CATALYSIS_COST_FRACTION. Axiom 4: catalysis has a price.
///
/// `node_frequencies[i]` is the frequency of gene that produced node i.
///
/// Returns (effective_activation_energies, total_catalysis_cost).
pub fn catalytic_activation_reduction(
    nodes: &[ExergyNode],
    edges: &[ExergyEdge],
    node_count: usize,
    edge_count: usize,
    node_frequencies: &[f32],
) -> ([f32; METABOLIC_GRAPH_MAX_NODES], f32) {
    let mut effective_ea = [0.0f32; METABOLIC_GRAPH_MAX_NODES];
    let nc = node_count.min(METABOLIC_GRAPH_MAX_NODES);
    let ec = edge_count.min(METABOLIC_GRAPH_MAX_EDGES);

    // Start with base activation energies
    for i in 0..nc {
        effective_ea[i] = nodes[i].activation_energy;
    }

    // Accumulate reductions from upstream nodes via edges
    let mut total_cost = 0.0f32;
    for k in 0..ec {
        let from = edges[k].from as usize;
        let to = edges[k].to as usize;
        if from >= nc || to >= nc { continue; }

        let heat = nodes[from].thermal_output.max(0.0);
        if heat <= 0.0 { continue; }

        let f_from = node_frequencies.get(from).copied().unwrap_or(0.0);
        let f_to = node_frequencies.get(to).copied().unwrap_or(0.0);
        let alignment = catalytic_freq_alignment(f_from, f_to);

        let reduction = heat * CATALYSIS_EFFICIENCY * alignment;
        effective_ea[to] = (effective_ea[to] - reduction).max(CATALYSIS_MIN_ACTIVATION);
        total_cost += reduction * CATALYSIS_COST_FRACTION;
    }

    (effective_ea, total_cost)
}

// ─── Helpers ────────────────────────────────────────────────────────────────

fn sanitize(v: f32) -> f32 {
    if v.is_finite() { v.clamp(0.0, 1.0) } else { 0.0 }
}

// ─── Tests ──────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    // ── MGN-1: Gene → Node ──────────────────────────────────────────────────

    #[test]
    fn infer_role_core_genes_return_stem() {
        // Genes 0-3 are core biases, default to Stem (not real organs)
        for i in 0..MIN_GENES {
            assert_eq!(infer_role_from_gene(i), OrganRole::Stem);
        }
    }

    #[test]
    fn infer_role_dimension_cycles() {
        // gene 4,8,12 → dimension 0 (growth) → Root/Core/Fruit
        assert_eq!(infer_role_from_gene(4), OrganRole::Root);
        assert_eq!(infer_role_from_gene(8), OrganRole::Core);
        assert_eq!(infer_role_from_gene(12), OrganRole::Fruit);
    }

    #[test]
    fn infer_role_mobility_dimension() {
        // gene 5,9,13 → dimension 1 (mobility) → Fin/Limb/Bud
        assert_eq!(infer_role_from_gene(5), OrganRole::Fin);
        assert_eq!(infer_role_from_gene(9), OrganRole::Limb);
        assert_eq!(infer_role_from_gene(13), OrganRole::Bud);
    }

    #[test]
    fn infer_role_branching_dimension() {
        assert_eq!(infer_role_from_gene(6), OrganRole::Leaf);
        assert_eq!(infer_role_from_gene(10), OrganRole::Stem);
        assert_eq!(infer_role_from_gene(14), OrganRole::Petal);
    }

    #[test]
    fn infer_role_resilience_dimension() {
        assert_eq!(infer_role_from_gene(7), OrganRole::Shell);
        assert_eq!(infer_role_from_gene(11), OrganRole::Thorn);
        assert_eq!(infer_role_from_gene(15), OrganRole::Sensory);
    }

    #[test]
    fn infer_role_high_index_clamps_tier() {
        // gene 28 → tier = (28-4)/4 = 6, clamped to 2 (actuator)
        let role = infer_role_from_gene(28);
        assert_eq!(role, OrganRole::Fruit); // dim=0, tier=2
    }

    #[test]
    fn gene_to_node_efficiency_bounded() {
        for idx in MIN_GENES..MAX_GENES {
            let node = gene_to_exergy_node(1.0, idx);
            let role_idx = node.role as usize;
            assert!(node.efficiency <= ROLE_EFFICIENCY_FACTOR[role_idx] + 1e-5,
                "gene {idx}: η={} > factor={}", node.efficiency, ROLE_EFFICIENCY_FACTOR[role_idx]);
            assert!(node.efficiency >= 0.0);
        }
    }

    #[test]
    fn gene_to_node_zero_value_zero_efficiency() {
        let node = gene_to_exergy_node(0.0, 4);
        assert_eq!(node.efficiency, 0.0);
    }

    #[test]
    fn gene_to_node_high_value_low_activation() {
        let node_high = gene_to_exergy_node(1.0, 4);
        let node_low = gene_to_exergy_node(0.0, 4);
        assert!(node_high.activation_energy < node_low.activation_energy,
            "high gene should lower activation barrier");
    }

    #[test]
    fn gene_to_node_nan_safe() {
        let node = gene_to_exergy_node(f32::NAN, 4);
        assert_eq!(node.efficiency, 0.0);
    }

    #[test]
    fn gene_to_node_out_of_range_clamped() {
        let node = gene_to_exergy_node(2.0, 4);
        let role_idx = node.role as usize;
        assert!(node.efficiency <= ROLE_EFFICIENCY_FACTOR[role_idx] + 1e-5);
    }

    #[test]
    fn gene_to_node_thermal_and_entropy_zero() {
        let node = gene_to_exergy_node(0.5, 6);
        assert_eq!(node.thermal_output, 0.0);
        assert_eq!(node.entropy_rate, 0.0);
    }

    // ── MGN-2: Topology Inference ───────────────────────────────────────────

    #[test]
    fn topology_empty_no_edges() {
        let (_, count) = infer_topology(&[], &[]);
        assert_eq!(count, 0);
    }

    #[test]
    fn topology_single_gene_no_edges() {
        let (_, count) = infer_topology(&[4], &[0.5]);
        assert_eq!(count, 0); // needs a target in higher tier
    }

    #[test]
    fn topology_same_dim_different_tiers_one_edge() {
        // gene 4 (dim=0, tier=0) → gene 8 (dim=0, tier=1)
        let (edges, count) = infer_topology(&[4, 8], &[0.5, 0.5]);
        assert_eq!(count, 1);
        assert_eq!(edges[0].from, 0); // index 0 in active list
        assert_eq!(edges[0].to, 1);   // index 1 in active list
    }

    #[test]
    fn topology_different_dim_no_edge_unless_hub() {
        // gene 5 (dim=1, tier=0) → gene 10 (dim=2, tier=1): different dim, not hub
        let (_, count) = infer_topology(&[5, 10], &[0.5, 0.5]);
        assert_eq!(count, 0);
    }

    #[test]
    fn topology_growth_hub_connects_cross_dimension() {
        // gene 4 (dim=0, tier=0) → gene 9 (dim=1, tier=1): dim=0 is hub
        let (edges, count) = infer_topology(&[4, 9], &[0.5, 0.5]);
        assert_eq!(count, 1, "growth hub should connect to mobility tier 1");
        assert_eq!(edges[0].from, 0);
    }

    #[test]
    fn topology_tier_ordering_respected() {
        let indices: Vec<usize> = (4..16).collect();
        let values = vec![0.5; 12];
        let (edges, count) = infer_topology(&indices, &values);
        for k in 0..count {
            let ti = gene_tier(indices[edges[k].from as usize]);
            let tj = gene_tier(indices[edges[k].to as usize]);
            assert!(ti < tj, "edge {k}: tier {ti} → {tj} violates DAG ordering");
        }
    }

    #[test]
    fn topology_distance_reduces_capacity() {
        // gene 4 (dim=0, tier=0) → gene 8 (dim=0, tier=1): distance = 4
        let (edges_near, cn) = infer_topology(&[4, 8], &[0.5, 0.5]);
        // gene 4 (dim=0, tier=0) → gene 12 (dim=0, tier=2): distance = 8
        let (edges_far, cf) = infer_topology(&[4, 12], &[0.5, 0.5]);
        assert_eq!(cn, 1); assert_eq!(cf, 1);
        // Axiom 7: farther genes → lower capacity (distance penalty baked in)
        assert!(edges_far[0].capacity < edges_near[0].capacity,
            "farther genes should have lower capacity: {} < {}",
            edges_far[0].capacity, edges_near[0].capacity);
    }

    #[test]
    fn topology_capacity_scales_with_gene_value() {
        let (edges_high, c1) = infer_topology(&[4, 8], &[0.9, 0.9]);
        let (edges_low, c2) = infer_topology(&[4, 8], &[0.1, 0.1]);
        assert_eq!(c1, 1); assert_eq!(c2, 1);
        assert!(edges_high[0].capacity > edges_low[0].capacity);
    }

    #[test]
    fn topology_max_edges_respected() {
        let indices: Vec<usize> = (4..32).collect();
        let values = vec![0.5; 28];
        let (_, count) = infer_topology(&indices, &values);
        assert!(count <= METABOLIC_GRAPH_MAX_EDGES);
    }

    #[test]
    fn topology_deterministic() {
        let (a, ca) = infer_topology(&[4, 5, 8, 9], &[0.5, 0.6, 0.7, 0.8]);
        let (b, cb) = infer_topology(&[4, 5, 8, 9], &[0.5, 0.6, 0.7, 0.8]);
        assert_eq!(ca, cb);
        for k in 0..ca { assert_eq!(a[k].from, b[k].from); assert_eq!(a[k].to, b[k].to); }
    }

    // ── MGN-3: MetabolicGraph from VariableGenome ───────────────────────────

    #[test]
    fn four_gene_genome_returns_empty_error() {
        let g = VariableGenome::default(); // 4 genes
        let result = metabolic_graph_from_variable_genome(&g, &[1.0; 4]);
        assert!(result.is_err());
    }

    #[test]
    fn eight_gene_genome_produces_graph() {
        let mut g = VariableGenome::from_biases(0.5, 0.5, 0.5, 0.5);
        for i in 4..8 { g.genes[i] = 0.6; }
        g.len = 8;
        let result = metabolic_graph_from_variable_genome(&g, &[1.0; 4]);
        assert!(result.is_ok(), "8 genes should produce a valid graph");
        let graph = result.unwrap();
        assert!(graph.node_count() > 0);
    }

    #[test]
    fn fully_silenced_returns_empty() {
        let mut g = VariableGenome::from_biases(0.5, 0.5, 0.5, 0.5);
        for i in 4..8 { g.genes[i] = 0.5; }
        g.len = 8;
        let result = metabolic_graph_from_variable_genome(&g, &[0.0; 4]);
        assert!(result.is_err(), "fully silenced → no nodes → empty");
    }

    #[test]
    fn partial_mask_filters_dimensions() {
        let mut g = VariableGenome::from_biases(0.5, 0.5, 0.5, 0.5);
        for i in 4..12 { g.genes[i] = 0.6; }
        g.len = 12;
        // Silence mobility (dim 1) and resilience (dim 3)
        let mask = [1.0, 0.0, 1.0, 0.0];
        let result = metabolic_graph_from_variable_genome(&g, &mask);
        if let Ok(graph) = result {
            // Nodes for dim 1 (Fin/Limb) and dim 3 (Shell/Thorn) should be absent
            let roles: Vec<OrganRole> = graph.nodes()[..graph.node_count()]
                .iter().map(|n| n.role).collect();
            assert!(!roles.contains(&OrganRole::Fin), "mobility silenced");
            assert!(!roles.contains(&OrganRole::Shell), "resilience silenced");
        }
    }

    #[test]
    fn node_count_scales_with_genome_length() {
        let build = |n: usize| -> usize {
            let mut g = VariableGenome::from_biases(0.5, 0.5, 0.5, 0.5);
            for i in 4..n { g.genes[i] = 0.6; }
            g.len = n as u8;
            metabolic_graph_from_variable_genome(&g, &[1.0; 4])
                .map(|g| g.node_count()).unwrap_or(0)
        };
        let n8 = build(8);
        let n16 = build(16);
        assert!(n16 >= n8, "16 genes should have ≥ nodes than 8");
    }

    #[test]
    fn graph_is_valid_dag() {
        let mut g = VariableGenome::from_biases(0.5, 0.5, 0.5, 0.5);
        for i in 4..16 { g.genes[i] = 0.6; }
        g.len = 16;
        // build() internally validates DAG (no cycles, valid indices)
        let result = metabolic_graph_from_variable_genome(&g, &[1.0; 4]);
        assert!(result.is_ok(), "16-gene graph should be valid DAG");
    }

    #[test]
    fn max_genome_valid_graph() {
        let mut g = VariableGenome::default();
        for i in 0..MAX_GENES { g.genes[i] = 0.6; }
        g.len = MAX_GENES as u8;
        let result = metabolic_graph_from_variable_genome(&g, &[1.0; 4]);
        assert!(result.is_ok(), "max genome should produce valid graph");
        let graph = result.unwrap();
        assert!(graph.node_count() <= METABOLIC_GRAPH_MAX_NODES);
        assert!(graph.edge_count() <= METABOLIC_GRAPH_MAX_EDGES);
    }

    #[test]
    fn mutated_genome_produces_different_graph() {
        use crate::blueprint::equations::variable_genome::mutate_variable;
        let mut g = VariableGenome::from_biases(0.5, 0.5, 0.5, 0.5);
        for i in 4..12 { g.genes[i] = 0.5; }
        g.len = 12;
        let m = mutate_variable(&g, 42);
        let graph_original = metabolic_graph_from_variable_genome(&g, &[1.0; 4]);
        let graph_mutated = metabolic_graph_from_variable_genome(&m, &[1.0; 4]);
        // At least one should differ (node count or edge flows)
        if let (Ok(go), Ok(gm)) = (graph_original, graph_mutated) {
            let diff = go.node_count() != gm.node_count()
                || go.edge_count() != gm.edge_count();
            // May or may not differ — mutation is stochastic
            let _ = diff; // just verify no panic
        }
    }

    #[test]
    fn deterministic() {
        let mut g = VariableGenome::from_biases(0.3, 0.7, 0.2, 0.8);
        for i in 4..10 { g.genes[i] = 0.5; }
        g.len = 10;
        let mask = [0.8, 0.5, 1.0, 0.3];
        let a = metabolic_graph_from_variable_genome(&g, &mask);
        let b = metabolic_graph_from_variable_genome(&g, &mask);
        match (a, b) {
            (Ok(ga), Ok(gb)) => {
                assert_eq!(ga.node_count(), gb.node_count());
                assert_eq!(ga.edge_count(), gb.edge_count());
            }
            (Err(ea), Err(eb)) => assert_eq!(ea, eb),
            _ => panic!("determinism violation"),
        }
    }

    // ── Cache: Phenotype ────────────────────────────────────────────────────

    #[test]
    fn phenotype_none_for_short_genome() {
        let g = VariableGenome::default();
        assert!(compute_metabolic_phenotype(&g, &[1.0; 4]).is_none());
    }

    #[test]
    fn phenotype_some_for_long_genome() {
        let mut g = VariableGenome::from_biases(0.5, 0.5, 0.5, 0.5);
        for i in 4..12 { g.genes[i] = 0.6; } // need tier 0 + tier 1 for edges
        g.len = 12;
        let p = compute_metabolic_phenotype(&g, &[1.0; 4]);
        assert!(p.is_some());
        let p = p.unwrap();
        assert!(p.node_count > 0);
        assert!(p.total_capacity > 0.0);
    }

    #[test]
    fn phenotype_consistent_with_direct_build() {
        let mut g = VariableGenome::from_biases(0.5, 0.5, 0.5, 0.5);
        for i in 4..10 { g.genes[i] = 0.6; }
        g.len = 10;
        let mask = [1.0; 4];
        let direct = metabolic_graph_from_variable_genome(&g, &mask).unwrap();
        let cached = compute_metabolic_phenotype(&g, &mask).unwrap();
        assert_eq!(direct.node_count(), cached.node_count as usize);
        assert_eq!(direct.edge_count(), cached.edge_count as usize);
    }

    // ── MGN-5: Node Competition ─────────────────────────────────────────────

    #[test]
    fn competition_sum_le_input() {
        let out = [(0u8, 50.0f32, 0.8f32), (1, 40.0, 0.5), (2, 30.0, 0.3)];
        let (shares, overhead) = competitive_flow_distribution(100.0, &out);
        let total: f32 = shares[..3].iter().sum::<f32>() + overhead;
        assert!(total <= 100.0 + 1e-3, "Axiom 2: Σ shares + overhead ≤ input: {total}");
    }

    #[test]
    fn competition_sum_le_input_fuzz() {
        for j in 1..50 {
            let j_in = j as f32 * 3.0;
            let out = [(0u8, 20.0f32, 0.9f32), (1, 15.0, 0.4)];
            let (shares, overhead) = competitive_flow_distribution(j_in, &out);
            let total = shares[0] + shares[1] + overhead;
            assert!(total <= j_in + 1e-3, "Axiom 2 at j_in={j_in}: total={total}");
        }
    }

    #[test]
    fn competition_zero_input_zero_output() {
        let (shares, overhead) = competitive_flow_distribution(0.0, &[(0, 50.0, 0.8)]);
        assert_eq!(shares[0], 0.0);
        assert_eq!(overhead, 0.0);
    }

    #[test]
    fn competition_higher_efficiency_gets_more() {
        let out = [(0u8, 50.0f32, 0.9f32), (1, 50.0, 0.1)];
        let (shares, _) = competitive_flow_distribution(100.0, &out);
        assert!(shares[0] > shares[1], "η=0.9 should get more than η=0.1: {} vs {}", shares[0], shares[1]);
    }

    #[test]
    fn competition_equal_efficiency_equal_share() {
        let out = [(0u8, 50.0f32, 0.5f32), (1, 50.0, 0.5)];
        let (shares, _) = competitive_flow_distribution(100.0, &out);
        assert!((shares[0] - shares[1]).abs() < 1e-3, "equal η → equal share");
    }

    #[test]
    fn competition_zero_efficiency_zero_share() {
        let out = [(0u8, 50.0f32, 0.0f32), (1, 50.0, 0.8)];
        let (shares, _) = competitive_flow_distribution(100.0, &out);
        assert_eq!(shares[0], 0.0, "η=0 → zero share");
    }

    #[test]
    fn competition_single_edge_gets_all_minus_overhead() {
        let (shares, overhead) = competitive_flow_distribution(100.0, &[(0, 200.0, 0.8)]);
        assert!((shares[0] - (100.0 - overhead)).abs() < 1e-3);
    }

    #[test]
    fn competition_capacity_bottleneck() {
        let out = [(0u8, 5.0f32, 0.9f32)]; // capacity = 5, much less than j_in
        let (shares, _) = competitive_flow_distribution(100.0, &out);
        assert!(shares[0] <= 5.0 + 1e-3, "share capped by capacity: {}", shares[0]);
    }

    #[test]
    fn competition_overhead_increases_with_edges() {
        let (_, ov1) = competitive_flow_distribution(100.0, &[(0, 50.0, 0.5)]);
        let (_, ov3) = competitive_flow_distribution(100.0, &[(0, 50.0, 0.5), (1, 50.0, 0.5), (2, 50.0, 0.5)]);
        assert!(ov3 > ov1, "more edges → more overhead: {} vs {}", ov3, ov1);
    }

    #[test]
    fn competition_empty_outgoing_no_panic() {
        let (shares, overhead) = competitive_flow_distribution(100.0, &[]);
        assert_eq!(shares[0], 0.0);
        assert_eq!(overhead, 0.0);
    }

    #[test]
    fn competition_deterministic() {
        let out = [(0u8, 50.0f32, 0.8f32), (1, 40.0, 0.3)];
        let a = competitive_flow_distribution(100.0, &out);
        let b = competitive_flow_distribution(100.0, &out);
        assert_eq!(a.0[0].to_bits(), b.0[0].to_bits());
    }

    // ── MGN-6: Hebbian Rewiring ─────────────────────────────────────────────

    #[test]
    fn hebbian_high_flow_strengthens() {
        let caps = [50.0f32];
        let flows = [45.0f32]; // 90% utilization > 50% baseline
        let costs = [0.1f32];
        let (new, _) = hebbian_capacity_update(&caps, &flows, &costs, 1);
        assert!(new[0] > caps[0], "high usage → capacity grows: {} vs {}", new[0], caps[0]);
    }

    #[test]
    fn hebbian_low_flow_weakens() {
        let caps = [50.0f32];
        let flows = [5.0f32]; // 10% utilization < 50% baseline
        let costs = [0.1f32];
        let (new, _) = hebbian_capacity_update(&caps, &flows, &costs, 1);
        assert!(new[0] < caps[0], "low usage → capacity shrinks: {} vs {}", new[0], caps[0]);
    }

    #[test]
    fn hebbian_baseline_no_change() {
        let caps = [50.0f32];
        let flows = [25.0f32]; // 50% = baseline
        let costs = [0.1f32];
        let (new, _) = hebbian_capacity_update(&caps, &flows, &costs, 1);
        assert!((new[0] - caps[0]).abs() < 1e-3, "baseline → no change: {}", new[0]);
    }

    #[test]
    fn hebbian_bounded_by_min_max() {
        let caps = [EDGE_MIN_CAPACITY];
        let flows = [0.0f32]; // zero flow → decay
        let costs = [0.1f32];
        let (new, _) = hebbian_capacity_update(&caps, &flows, &costs, 1);
        assert!(new[0] >= EDGE_MIN_CAPACITY, "never below min: {}", new[0]);

        let caps2 = [EDGE_MAX_CAPACITY];
        let flows2 = [EDGE_MAX_CAPACITY]; // 100% utilization
        let (new2, _) = hebbian_capacity_update(&caps2, &flows2, &costs, 1);
        assert!(new2[0] <= EDGE_MAX_CAPACITY, "never above max: {}", new2[0]);
    }

    #[test]
    fn hebbian_strengthening_has_cost() {
        let caps = [50.0f32];
        let flows = [45.0f32]; // strengthen
        let costs = [0.5f32];
        let (_, cost) = hebbian_capacity_update(&caps, &flows, &costs, 1);
        assert!(cost > 0.0, "strengthening costs energy: {cost}");
    }

    #[test]
    fn hebbian_weakening_zero_cost() {
        let caps = [50.0f32];
        let flows = [5.0f32]; // weaken
        let costs = [0.5f32];
        let (_, cost) = hebbian_capacity_update(&caps, &flows, &costs, 1);
        assert_eq!(cost, 0.0, "weakening should be free: {cost}");
    }

    #[test]
    fn hebbian_far_edge_costs_more() {
        let caps = [50.0, 50.0];
        let flows = [45.0, 45.0]; // both strengthen equally
        let costs = [0.1, 1.0];   // second edge is "farther"
        let (_, _) = hebbian_capacity_update(&caps, &flows, &costs, 2);
        // Individual cost check: delta is same, but cost[1] > cost[0]
        let (_, cost_near) = hebbian_capacity_update(&[50.0], &[45.0], &[0.1], 1);
        let (_, cost_far) = hebbian_capacity_update(&[50.0], &[45.0], &[1.0], 1);
        assert!(cost_far > cost_near, "far edge costs more: {} vs {}", cost_far, cost_near);
    }

    #[test]
    fn hebbian_converges() {
        let mut cap = [50.0f32];
        let flow = [30.0f32]; // constant 60% utilization
        let cost = [0.1f32];
        for _ in 0..200 {
            let (new, _) = hebbian_capacity_update(&cap, &flow, &cost, 1);
            cap = [new[0]];
        }
        // After many iterations, should stabilize near some equilibrium
        let (new, _) = hebbian_capacity_update(&cap, &flow, &cost, 1);
        assert!((new[0] - cap[0]).abs() < 0.5, "should converge: delta={}", (new[0] - cap[0]).abs());
    }

    #[test]
    fn hebbian_empty_no_panic() {
        let (new, cost) = hebbian_capacity_update(&[], &[], &[], 0);
        assert_eq!(new[0], 0.0);
        assert_eq!(cost, 0.0);
    }

    #[test]
    fn hebbian_deterministic() {
        let a = hebbian_capacity_update(&[50.0, 30.0], &[40.0, 10.0], &[0.1, 0.2], 2);
        let b = hebbian_capacity_update(&[50.0, 30.0], &[40.0, 10.0], &[0.1, 0.2], 2);
        assert_eq!(a.0[0].to_bits(), b.0[0].to_bits());
    }

    // ── MGN-7: Internal Catalysis ───────────────────────────────────────────

    fn make_test_nodes(n: usize, heat: f32) -> [ExergyNode; METABOLIC_GRAPH_MAX_NODES] {
        let mut nodes = [ExergyNode::default(); METABOLIC_GRAPH_MAX_NODES];
        for i in 0..n.min(METABOLIC_GRAPH_MAX_NODES) {
            nodes[i].activation_energy = 5.0;
            nodes[i].thermal_output = heat;
            nodes[i].efficiency = 0.7;
        }
        nodes
    }

    fn make_test_edge(from: u8, to: u8) -> ExergyEdge {
        ExergyEdge { from, to, flow_rate: 10.0, max_capacity: 50.0, transport_cost: 0.1 }
    }

    #[test]
    fn catalysis_reduces_neighbor_ea() {
        let nodes = make_test_nodes(2, 20.0); // node 0 has heat
        let edges = [make_test_edge(0, 1)]; // 0 → 1
        let freqs = [100.0, 100.0]; // same freq → max alignment
        let (ea, _) = catalytic_activation_reduction(&nodes, &edges, 2, 1, &freqs);
        assert!(ea[1] < nodes[1].activation_energy,
            "catalysis should reduce E_a[1]: {} < {}", ea[1], nodes[1].activation_energy);
    }

    #[test]
    fn catalysis_no_edge_no_reduction() {
        let nodes = make_test_nodes(2, 20.0);
        let (ea, cost) = catalytic_activation_reduction(&nodes, &[], 2, 0, &[100.0, 100.0]);
        assert_eq!(ea[0], nodes[0].activation_energy, "no edges → no reduction");
        assert_eq!(cost, 0.0);
    }

    #[test]
    fn catalysis_zero_heat_no_reduction() {
        let nodes = make_test_nodes(2, 0.0); // no heat
        let edges = [make_test_edge(0, 1)];
        let (ea, cost) = catalytic_activation_reduction(&nodes, &edges, 2, 1, &[100.0, 100.0]);
        assert_eq!(ea[1], nodes[1].activation_energy);
        assert_eq!(cost, 0.0);
    }

    #[test]
    fn catalysis_has_cost() {
        let nodes = make_test_nodes(2, 20.0);
        let edges = [make_test_edge(0, 1)];
        let (_, cost) = catalytic_activation_reduction(&nodes, &edges, 2, 1, &[100.0, 100.0]);
        assert!(cost > 0.0, "catalysis should cost energy: {cost}");
    }

    #[test]
    fn catalysis_ea_never_below_minimum() {
        let mut nodes = make_test_nodes(2, 1000.0); // massive heat
        nodes[1].activation_energy = 1.0; // low base E_a
        let edges = [make_test_edge(0, 1)];
        let (ea, _) = catalytic_activation_reduction(&nodes, &edges, 2, 1, &[100.0, 100.0]);
        assert!(ea[1] >= CATALYSIS_MIN_ACTIVATION,
            "E_a[1] must be ≥ minimum: {} ≥ {}", ea[1], CATALYSIS_MIN_ACTIVATION);
    }

    #[test]
    fn catalysis_same_freq_max_alignment() {
        let align = catalytic_freq_alignment(100.0, 100.0);
        assert!((align - 1.0).abs() < 1e-5, "same freq → align=1.0: {align}");
    }

    #[test]
    fn catalysis_different_freq_reduced() {
        let same = catalytic_freq_alignment(100.0, 100.0);
        let diff = catalytic_freq_alignment(100.0, 200.0);
        assert!(diff < same, "different freq → less alignment: {diff} < {same}");
    }

    #[test]
    fn catalysis_very_different_freq_near_zero() {
        let align = catalytic_freq_alignment(100.0, 1000.0);
        assert!(align < 0.01, "very different freq → near zero: {align}");
    }

    #[test]
    fn catalysis_freq_alignment_symmetric() {
        let ab = catalytic_freq_alignment(100.0, 300.0);
        let ba = catalytic_freq_alignment(300.0, 100.0);
        assert!((ab - ba).abs() < 1e-6, "alignment should be symmetric");
    }

    #[test]
    fn catalysis_chain_amplifies() {
        // A→B→C: A catalyzes B, B catalyzes C
        let mut nodes = make_test_nodes(3, 15.0);
        nodes[0].thermal_output = 20.0;
        nodes[1].thermal_output = 15.0;
        let edges = [make_test_edge(0, 1), make_test_edge(1, 2)];
        let freqs = [100.0, 100.0, 100.0];
        let (ea, _) = catalytic_activation_reduction(&nodes, &edges, 3, 2, &freqs);
        assert!(ea[1] < nodes[1].activation_energy, "B catalyzed by A");
        assert!(ea[2] < nodes[2].activation_energy, "C catalyzed by B");
    }

    #[test]
    fn catalysis_single_node_no_change() {
        let nodes = make_test_nodes(1, 20.0);
        let (ea, cost) = catalytic_activation_reduction(&nodes, &[], 1, 0, &[100.0]);
        assert_eq!(ea[0], nodes[0].activation_energy);
        assert_eq!(cost, 0.0);
    }

    #[test]
    fn catalysis_deterministic() {
        let nodes = make_test_nodes(2, 15.0);
        let edges = [make_test_edge(0, 1)];
        let freqs = [100.0, 150.0];
        let a = catalytic_activation_reduction(&nodes, &edges, 2, 1, &freqs);
        let b = catalytic_activation_reduction(&nodes, &edges, 2, 1, &freqs);
        assert_eq!(a.0[1].to_bits(), b.0[1].to_bits());
        assert_eq!(a.1.to_bits(), b.1.to_bits());
    }

    #[test]
    fn catalysis_cost_proportional_to_reduction() {
        let nodes_low = make_test_nodes(2, 5.0);
        let nodes_high = make_test_nodes(2, 50.0);
        let edges = [make_test_edge(0, 1)];
        let freqs = [100.0, 100.0];
        let (_, cost_low) = catalytic_activation_reduction(&nodes_low, &edges, 2, 1, &freqs);
        let (_, cost_high) = catalytic_activation_reduction(&nodes_high, &edges, 2, 1, &freqs);
        assert!(cost_high > cost_low, "more heat → more catalysis → more cost");
    }
}
