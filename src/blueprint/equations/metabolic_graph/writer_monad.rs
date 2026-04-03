//! MG-6 — Writer Monad: organ_transform, distribute_to_children, evaluate_metabolic_chain.
//!
//! Funciones puras que componen la cadena metabolica en orden topologico.
//! Sin dependencia ECS — sistemas orquestan queries y llaman aqui.

use crate::blueprint::constants::{CHAIN_CONSERVATION_EPSILON, DIVISION_GUARD_EPSILON};
use crate::blueprint::morphogenesis::{san_efficiency_01, san_nonneg};
use crate::layers::metabolic_graph::{
    METABOLIC_GRAPH_MAX_EDGES, METABOLIC_GRAPH_MAX_NODES, MetabolicGraph,
};

// ── Tipos (Copy, stack-only) ────────────────────────────────────────

/// Salida de un organo logico: util + desechos (Writer).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct OrganOutput {
    pub mass_out: f32,
    pub exergy_out: f32,
    pub waste_mass: f32,
    pub heat_dissipated: f32,
}

/// Resultado de evaluar todo el DAG en orden topologico.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ChainOutput {
    pub final_exergy: f32,
    pub total_heat: f32,
    pub total_waste: f32,
    pub per_node_heat: [f32; METABOLIC_GRAPH_MAX_NODES],
}

// ── organ_transform ─────────────────────────────────────────────────

/// Transforma masa y exergia de entrada en un organo logico.
/// Conservacion: mass_in = mass_out + waste_mass,
///               exergy_in >= exergy_out + heat_dissipated (igualdad cuando ea <= e*eta).
#[inline]
pub fn organ_transform(
    mass_in: f32,
    exergy_in: f32,
    efficiency: f32,
    activation_energy: f32,
) -> OrganOutput {
    let m = san_nonneg(mass_in);
    let e = san_nonneg(exergy_in);
    let eta = san_efficiency_01(efficiency);
    let ea = san_nonneg(activation_energy);

    let exergy_out = crate::blueprint::morphogenesis::exergy_balance(e, eta, ea);
    let heat_dissipated = (e - exergy_out - ea).max(0.0);
    let waste_mass = m * (1.0 - eta);
    let mass_out = m - waste_mass;

    debug_assert!(
        (m - mass_out - waste_mass).abs() < CHAIN_CONSERVATION_EPSILON + m * 1e-5,
        "mass conservation violated: in={m}, out={mass_out}, waste={waste_mass}",
    );
    debug_assert!(
        (e - exergy_out - heat_dissipated - ea).abs() < CHAIN_CONSERVATION_EPSILON + e * 1e-5
            || exergy_out == 0.0, // activation consumes everything
        "exergy conservation violated: in={e}, out={exergy_out}, heat={heat_dissipated}, ea={ea}",
    );

    OrganOutput {
        mass_out,
        exergy_out,
        waste_mass,
        heat_dissipated,
    }
}

// ── distribute_to_children ──────────────────────────────────────────

/// Reparte masa y exergia entre aristas salientes proporcional a max_capacity.
/// Retorna (edge_idx, mass, exergy) por arista.
#[inline]
pub fn distribute_to_children(
    total_mass: f32,
    total_exergy: f32,
    edge_capacities: &[(u8, f32)],
) -> ([(u8, f32, f32); METABOLIC_GRAPH_MAX_EDGES], usize) {
    let mut result = [(0u8, 0.0f32, 0.0f32); METABOLIC_GRAPH_MAX_EDGES];
    let n = edge_capacities.len().min(METABOLIC_GRAPH_MAX_EDGES);
    if n == 0 {
        return (result, 0);
    }

    let m = san_nonneg(total_mass);
    let ex = san_nonneg(total_exergy);

    let sum_cap: f32 = edge_capacities[..n]
        .iter()
        .map(|(_, cap)| san_nonneg(*cap))
        .sum();

    if sum_cap <= DIVISION_GUARD_EPSILON {
        let share_m = m / n as f32;
        let share_e = ex / n as f32;
        for (i, &(idx, _)) in edge_capacities[..n].iter().enumerate() {
            result[i] = (idx, share_m, share_e);
        }
        return (result, n);
    }

    for (i, &(idx, cap)) in edge_capacities[..n].iter().enumerate() {
        let proportion = san_nonneg(cap) / sum_cap;
        result[i] = (idx, m * proportion, ex * proportion);
    }

    (result, n)
}

// ── exergy_efficiency ───────────────────────────────────────────────

/// eta_total = final_exergy / initial_exergy. Guards division by EPSILON.
#[inline]
pub fn exergy_efficiency(final_exergy: f32, initial_exergy: f32) -> f32 {
    let num = san_nonneg(final_exergy);
    let den = san_nonneg(initial_exergy);
    if den > DIVISION_GUARD_EPSILON {
        num / den
    } else {
        0.0
    }
}

// ── evaluate_metabolic_chain ────────────────────────────────────────

/// Evalua el DAG metabolico completo en orden topologico (Kahn determinista).
/// Retorna ChainOutput con totales y heat por nodo.
pub fn evaluate_metabolic_chain(
    graph: &MetabolicGraph,
    initial_mass: f32,
    initial_exergy: f32,
) -> ChainOutput {
    let node_count = graph.node_count();
    if node_count == 0 {
        return ChainOutput {
            final_exergy: 0.0,
            total_heat: 0.0,
            total_waste: 0.0,
            per_node_heat: [0.0; METABOLIC_GRAPH_MAX_NODES],
        };
    }

    let edges = graph.edges();
    let nodes = graph.nodes();

    // In-degree + root injection
    let mut in_degree = [0u8; METABOLIC_GRAPH_MAX_NODES];
    for e in edges {
        let to = e.to as usize;
        if to < node_count {
            in_degree[to] = in_degree[to].saturating_add(1);
        }
    }

    let root_count = in_degree[..node_count].iter().filter(|&&d| d == 0).count();
    let mut mass_in = [0.0f32; METABOLIC_GRAPH_MAX_NODES];
    let mut exergy_in = [0.0f32; METABOLIC_GRAPH_MAX_NODES];
    if root_count > 0 {
        let m_share = san_nonneg(initial_mass) / root_count as f32;
        let e_share = san_nonneg(initial_exergy) / root_count as f32;
        for (i, &deg) in in_degree[..node_count].iter().enumerate() {
            if deg == 0 {
                mass_in[i] = m_share;
                exergy_in[i] = e_share;
            }
        }
    }

    // Pre-compute outgoing adjacency: O(E) once
    let (adj_indices, adj_starts, has_outgoing) = build_outgoing_adjacency(edges, node_count);

    // Kahn determinista (cola ordenada por indice ascendente)
    let order = kahn_topological_order(&in_degree, &adj_indices, &adj_starts, edges, node_count);

    // Acumuladores
    let mut per_node_heat = [0.0f32; METABOLIC_GRAPH_MAX_NODES];
    let mut exergy_out_per = [0.0f32; METABOLIC_GRAPH_MAX_NODES];
    let mut total_heat = 0.0f32;
    let mut total_waste = 0.0f32;

    for &ni in &order[..node_count] {
        let idx = ni as usize;
        let node = &nodes[idx];

        let output = organ_transform(
            mass_in[idx],
            exergy_in[idx],
            node.efficiency,
            node.activation_energy,
        );

        per_node_heat[idx] = output.heat_dissipated;
        exergy_out_per[idx] = output.exergy_out;
        total_heat += output.heat_dissipated;
        total_waste += output.waste_mass;

        // Outgoing edges via pre-computed adjacency: O(out-degree)
        let start = adj_starts[idx] as usize;
        let end = adj_starts[idx + 1] as usize;
        if start < end {
            let mut out_caps = [(0u8, 0.0f32); METABOLIC_GRAPH_MAX_EDGES];
            let out_count = end - start;
            for k in 0..out_count {
                let ei = adj_indices[start + k] as usize;
                out_caps[k] = (ei as u8, edges[ei].max_capacity);
            }

            let (distributed, dist_count) =
                distribute_to_children(output.mass_out, output.exergy_out, &out_caps[..out_count]);
            for i in 0..dist_count {
                let (ei, mass_share, exergy_share) = distributed[i];
                let to = edges[ei as usize].to as usize;
                if to < node_count {
                    mass_in[to] += mass_share;
                    exergy_in[to] += exergy_share;
                }
            }
        }
    }

    // Final exergy = sum of exergy_out from terminal nodes (no outgoing edges)
    let final_exergy: f32 = (0..node_count)
        .filter(|&i| !has_outgoing[i])
        .map(|i| exergy_out_per[i])
        .sum();

    ChainOutput {
        final_exergy,
        total_heat,
        total_waste,
        per_node_heat,
    }
}

/// Pre-computes outgoing edge adjacency in O(E). Stack-only.
/// Returns (edge_indices, starts, has_outgoing).
fn build_outgoing_adjacency(
    edges: &[crate::layers::metabolic_graph::ExergyEdge],
    node_count: usize,
) -> (
    [u8; METABOLIC_GRAPH_MAX_EDGES],
    [u8; METABOLIC_GRAPH_MAX_NODES + 1],
    [bool; METABOLIC_GRAPH_MAX_NODES],
) {
    let mut counts = [0u8; METABOLIC_GRAPH_MAX_NODES];
    let mut has_outgoing = [false; METABOLIC_GRAPH_MAX_NODES];
    for e in edges {
        let from = e.from as usize;
        if from < node_count {
            counts[from] = counts[from].saturating_add(1);
            has_outgoing[from] = true;
        }
    }

    let mut starts = [0u8; { METABOLIC_GRAPH_MAX_NODES + 1 }];
    for i in 0..node_count {
        starts[i + 1] = starts[i].saturating_add(counts[i]);
    }
    for i in (node_count + 1)..=METABOLIC_GRAPH_MAX_NODES {
        starts[i] = starts[node_count];
    }

    let mut edge_indices = [0u8; METABOLIC_GRAPH_MAX_EDGES];
    let mut offsets = starts;
    for (ei, e) in edges.iter().enumerate() {
        let from = e.from as usize;
        if from < node_count {
            let pos = offsets[from] as usize;
            if pos < METABOLIC_GRAPH_MAX_EDGES {
                edge_indices[pos] = ei as u8;
                offsets[from] = offsets[from].saturating_add(1);
            }
        }
    }

    (edge_indices, starts, has_outgoing)
}

/// Kahn topologico determinista con adjacency pre-computada: O(N+E).
fn kahn_topological_order(
    in_degree: &[u8; METABOLIC_GRAPH_MAX_NODES],
    adj_indices: &[u8; METABOLIC_GRAPH_MAX_EDGES],
    adj_starts: &[u8; METABOLIC_GRAPH_MAX_NODES + 1],
    edges: &[crate::layers::metabolic_graph::ExergyEdge],
    node_count: usize,
) -> [u8; METABOLIC_GRAPH_MAX_NODES] {
    let mut pending = *in_degree;
    let mut order = [0u8; METABOLIC_GRAPH_MAX_NODES];
    let mut order_len = 0usize;

    for i in 0..node_count {
        if pending[i] == 0 {
            order[order_len] = i as u8;
            order_len += 1;
        }
    }

    let mut head = 0usize;
    while head < order_len {
        let u = order[head] as usize;
        head += 1;

        let start = adj_starts[u] as usize;
        let end = adj_starts[u + 1] as usize;
        for k in start..end {
            let ei = adj_indices[k] as usize;
            let v = edges[ei].to as usize;
            if v >= node_count {
                continue;
            }
            pending[v] = pending[v].saturating_sub(1);
            if pending[v] == 0 && order_len < node_count {
                let mut pos = order_len;
                while pos > head && order[pos - 1] > v as u8 {
                    order[pos] = order[pos - 1];
                    pos -= 1;
                }
                order[pos] = v as u8;
                order_len += 1;
            }
        }
    }

    debug_assert!(
        order_len == node_count,
        "Kahn incomplete: {order_len} of {node_count} nodes — cycle in validated DAG?",
    );

    order
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::blueprint::OrganRole;
    use crate::blueprint::morphogenesis::exergy_balance;
    use crate::layers::MetabolicGraphBuilder;

    // ── MG-6A: Type properties ──

    #[test]
    fn organ_output_is_copy() {
        let a = OrganOutput {
            mass_out: 1.0,
            exergy_out: 2.0,
            waste_mass: 0.5,
            heat_dissipated: 0.3,
        };
        let b = a;
        assert_eq!(a, b);
    }

    #[test]
    fn chain_output_is_copy() {
        let a = ChainOutput {
            final_exergy: 1.0,
            total_heat: 2.0,
            total_waste: 3.0,
            per_node_heat: [0.0; METABOLIC_GRAPH_MAX_NODES],
        };
        let b = a;
        assert_eq!(a, b);
    }

    #[test]
    fn chain_output_size_is_stack_friendly() {
        let size = std::mem::size_of::<ChainOutput>();
        // 3 * 4 + 12 * 4 = 60 bytes
        assert_eq!(size, 60, "ChainOutput size changed unexpectedly: {size}");
    }

    // ── MG-6B: organ_transform ──

    #[test]
    fn organ_transform_standard_case() {
        let out = organ_transform(100.0, 500.0, 0.7, 10.0);
        assert!(
            (out.mass_out - 70.0).abs() < 1e-4,
            "mass_out={}",
            out.mass_out
        );
        assert!(
            (out.waste_mass - 30.0).abs() < 1e-4,
            "waste_mass={}",
            out.waste_mass
        );
        let expected_exergy = exergy_balance(500.0, 0.7, 10.0);
        assert!(
            (out.exergy_out - expected_exergy).abs() < 1e-4,
            "exergy_out={}",
            out.exergy_out
        );
        let expected_heat = 500.0 - expected_exergy - 10.0;
        assert!(
            (out.heat_dissipated - expected_heat).abs() < 1e-4,
            "heat={}",
            out.heat_dissipated
        );
    }

    #[test]
    fn organ_transform_mass_conservation() {
        let out = organ_transform(100.0, 500.0, 0.7, 10.0);
        assert!((100.0 - out.mass_out - out.waste_mass).abs() < 1e-4);
    }

    #[test]
    fn organ_transform_exergy_conservation() {
        let out = organ_transform(100.0, 500.0, 0.7, 10.0);
        let balance = out.exergy_out + out.heat_dissipated + 10.0;
        assert!((500.0 - balance).abs() < 1e-3, "balance={balance}");
    }

    #[test]
    fn organ_transform_zero_input_all_zero() {
        let out = organ_transform(0.0, 0.0, 0.7, 10.0);
        assert_eq!(out.mass_out, 0.0);
        assert_eq!(out.exergy_out, 0.0);
        assert_eq!(out.waste_mass, 0.0);
        assert_eq!(out.heat_dissipated, 0.0);
    }

    #[test]
    fn organ_transform_activation_exceeds_exergy() {
        let out = organ_transform(100.0, 5.0, 0.7, 10.0);
        assert_eq!(
            out.exergy_out, 0.0,
            "activation > exergy*eta => exergy_out=0"
        );
        assert!(out.mass_out >= 0.0);
        assert!(out.waste_mass >= 0.0);
        assert!(out.heat_dissipated >= 0.0);
    }

    #[test]
    fn organ_transform_negative_mass_clamped() {
        let out = organ_transform(-50.0, 100.0, 0.5, 5.0);
        assert_eq!(out.mass_out, 0.0);
        assert_eq!(out.waste_mass, 0.0);
    }

    #[test]
    fn organ_transform_efficiency_one_no_waste() {
        let out = organ_transform(100.0, 500.0, 1.0, 0.0);
        assert!((out.waste_mass).abs() < 1e-4);
        assert!((out.mass_out - 100.0).abs() < 1e-4);
    }

    #[test]
    fn organ_transform_efficiency_zero_all_waste() {
        let out = organ_transform(100.0, 500.0, 0.0, 0.0);
        assert!((out.mass_out).abs() < 1e-4);
        assert!((out.waste_mass - 100.0).abs() < 1e-4);
        assert!((out.exergy_out).abs() < 1e-4);
    }

    #[test]
    fn organ_transform_conservation_table_driven() {
        let cases: [(f32, f32, f32, f32); 20] = [
            (100.0, 500.0, 0.7, 10.0),
            (50.0, 200.0, 0.9, 5.0),
            (200.0, 100.0, 0.5, 20.0),
            (10.0, 1000.0, 0.3, 50.0),
            (0.0, 0.0, 0.7, 10.0),
            (100.0, 5.0, 0.7, 10.0),
            (1.0, 1.0, 0.1, 0.0),
            (1000.0, 1000.0, 0.99, 1.0),
            (0.5, 0.5, 0.5, 0.5),
            (100.0, 100.0, 0.0, 0.0),
            (100.0, 100.0, 1.0, 0.0),
            (100.0, 100.0, 1.0, 100.0),
            (1e-3, 1e-3, 0.5, 0.0),
            (1e6, 1e6, 0.8, 1e3),
            (42.0, 137.0, 0.42, 3.14),
            (99.9, 99.9, 0.999, 0.001),
            (1.0, 1000.0, 0.01, 0.0),
            (1000.0, 1.0, 0.99, 0.0),
            (50.0, 50.0, 0.5, 25.0),
            (75.0, 300.0, 0.6, 15.0),
        ];
        for (i, &(m, e, eta, ea)) in cases.iter().enumerate() {
            let out = organ_transform(m, e, eta, ea);
            let m_safe = san_nonneg(m);
            let eps_m = 1e-3 + m_safe * 1e-5;
            assert!(
                (m_safe - out.mass_out - out.waste_mass).abs() < eps_m,
                "case {i}: mass conservation failed: {m_safe} != {} + {}",
                out.mass_out,
                out.waste_mass,
            );
            let e_safe = san_nonneg(e);
            let ea_safe = san_nonneg(ea);
            let eps_e = 1e-2 + e_safe * 1e-5;
            let sum = out.exergy_out + out.heat_dissipated + ea_safe.min(e_safe);
            assert!(
                sum <= e_safe + eps_e,
                "case {i}: exergy sum {sum} > input {e_safe}",
            );
        }
    }

    // ── MG-6F: distribute_to_children ──

    #[test]
    fn distribute_proportional_split() {
        let (dist, n) = distribute_to_children(100.0, 500.0, &[(0, 60.0), (1, 40.0)]);
        assert_eq!(n, 2);
        assert!((dist[0].1 - 60.0).abs() < 1e-3, "mass 0: {}", dist[0].1);
        assert!((dist[0].2 - 300.0).abs() < 1e-3, "exergy 0: {}", dist[0].2);
        assert!((dist[1].1 - 40.0).abs() < 1e-3, "mass 1: {}", dist[1].1);
        assert!((dist[1].2 - 200.0).abs() < 1e-3, "exergy 1: {}", dist[1].2);
    }

    #[test]
    fn distribute_empty_returns_empty() {
        let (_, n) = distribute_to_children(100.0, 500.0, &[]);
        assert_eq!(n, 0);
    }

    #[test]
    fn distribute_conservation() {
        let (dist, n) = distribute_to_children(100.0, 500.0, &[(0, 60.0), (1, 40.0)]);
        let sum_m: f32 = (0..n).map(|i| dist[i].1).sum();
        let sum_e: f32 = (0..n).map(|i| dist[i].2).sum();
        assert!(
            (sum_m - 100.0).abs() < CHAIN_CONSERVATION_EPSILON,
            "mass sum={sum_m}"
        );
        assert!(
            (sum_e - 500.0).abs() < CHAIN_CONSERVATION_EPSILON,
            "exergy sum={sum_e}"
        );
    }

    #[test]
    fn distribute_zero_capacities_uniform() {
        let (dist, n) = distribute_to_children(100.0, 200.0, &[(0, 0.0), (1, 0.0), (2, 0.0)]);
        assert_eq!(n, 3);
        for i in 0..n {
            assert!((dist[i].1 - 100.0 / 3.0).abs() < 1e-3);
            assert!((dist[i].2 - 200.0 / 3.0).abs() < 1e-3);
        }
    }

    #[test]
    fn distribute_single_edge_gets_all() {
        let (dist, n) = distribute_to_children(50.0, 250.0, &[(5, 30.0)]);
        assert_eq!(n, 1);
        assert!((dist[0].1 - 50.0).abs() < 1e-4);
        assert!((dist[0].2 - 250.0).abs() < 1e-4);
        assert_eq!(dist[0].0, 5);
    }

    // ── MG-6B: evaluate_metabolic_chain ──

    fn build_linear_3_graph() -> MetabolicGraph {
        MetabolicGraphBuilder::new()
            .add_node(OrganRole::Root, 0.9, 3.0) // Captador
            .add_node(OrganRole::Core, 0.7, 8.0) // Procesador
            .add_node(OrganRole::Fin, 0.6, 5.0) // Actuador
            .add_edge(0, 1, 50.0)
            .add_edge(1, 2, 40.0)
            .build()
            .unwrap()
    }

    #[test]
    fn chain_linear_three_nodes_values() {
        let graph = build_linear_3_graph();
        let chain = evaluate_metabolic_chain(&graph, 100.0, 500.0);

        // Nodo 0: organ_transform(100, 500, 0.9, 3)
        let n0 = organ_transform(100.0, 500.0, 0.9, 3.0);
        assert!(
            (chain.per_node_heat[0] - n0.heat_dissipated).abs() < 1e-3,
            "node 0 heat: {} vs {}",
            chain.per_node_heat[0],
            n0.heat_dissipated
        );

        // Nodo 1: organ_transform(n0.mass_out, n0.exergy_out, 0.7, 8)
        let n1 = organ_transform(n0.mass_out, n0.exergy_out, 0.7, 8.0);
        assert!(
            (chain.per_node_heat[1] - n1.heat_dissipated).abs() < 1e-3,
            "node 1 heat: {} vs {}",
            chain.per_node_heat[1],
            n1.heat_dissipated
        );

        // Nodo 2: organ_transform(n1.mass_out, n1.exergy_out, 0.6, 5)
        let n2 = organ_transform(n1.mass_out, n1.exergy_out, 0.6, 5.0);
        assert!(
            (chain.per_node_heat[2] - n2.heat_dissipated).abs() < 1e-3,
            "node 2 heat: {} vs {}",
            chain.per_node_heat[2],
            n2.heat_dissipated
        );

        let expected_total_heat = n0.heat_dissipated + n1.heat_dissipated + n2.heat_dissipated;
        assert!(
            (chain.total_heat - expected_total_heat).abs() < 1e-2,
            "total heat: {} vs {}",
            chain.total_heat,
            expected_total_heat
        );

        let expected_total_waste = n0.waste_mass + n1.waste_mass + n2.waste_mass;
        assert!(
            (chain.total_waste - expected_total_waste).abs() < 1e-2,
            "total waste: {} vs {}",
            chain.total_waste,
            expected_total_waste
        );

        assert!(
            (chain.final_exergy - n2.exergy_out).abs() < 1e-2,
            "final exergy: {} vs {}",
            chain.final_exergy,
            n2.exergy_out
        );
    }

    #[test]
    fn chain_fork_proportional_distribution() {
        let graph = MetabolicGraphBuilder::new()
            .add_node(OrganRole::Root, 0.9, 3.0) // 0: captador
            .add_node(OrganRole::Stem, 0.8, 5.0) // 1: procesador A
            .add_node(OrganRole::Fin, 0.7, 4.0) // 2: procesador B
            .add_edge(0, 1, 60.0) // 60% capacity
            .add_edge(0, 2, 40.0) // 40% capacity
            .build()
            .unwrap();

        let chain = evaluate_metabolic_chain(&graph, 100.0, 500.0);

        // Node 0 processes, then distributes 60/40
        let n0 = organ_transform(100.0, 500.0, 0.9, 3.0);

        let mass_1 = n0.mass_out * 0.6;
        let exergy_1 = n0.exergy_out * 0.6;
        let mass_2 = n0.mass_out * 0.4;
        let exergy_2 = n0.exergy_out * 0.4;

        let n1 = organ_transform(mass_1, exergy_1, 0.8, 5.0);
        let n2 = organ_transform(mass_2, exergy_2, 0.7, 4.0);

        let expected_waste = n0.waste_mass + n1.waste_mass + n2.waste_mass;
        assert!(
            (chain.total_waste - expected_waste).abs() < 1e-1,
            "fork waste: {} vs {}",
            chain.total_waste,
            expected_waste
        );

        // Conservation: total inputs >= total outputs
        assert!(chain.total_heat >= 0.0);
        assert!(chain.total_waste >= 0.0);
        assert!(chain.final_exergy >= 0.0);
    }

    #[test]
    fn chain_join_sums_parent_outputs() {
        let graph = MetabolicGraphBuilder::new()
            .add_node(OrganRole::Root, 0.9, 3.0) // 0: captador A
            .add_node(OrganRole::Sensory, 0.8, 2.0) // 1: captador B
            .add_node(OrganRole::Core, 0.7, 8.0) // 2: procesador (join)
            .add_edge(0, 2, 50.0)
            .add_edge(1, 2, 50.0)
            .build()
            .unwrap();

        let chain = evaluate_metabolic_chain(&graph, 100.0, 500.0);

        // Two roots share input equally
        let n0 = organ_transform(50.0, 250.0, 0.9, 3.0);
        let n1 = organ_transform(50.0, 250.0, 0.8, 2.0);

        // Node 2 receives sum of parents
        let n2 = organ_transform(
            n0.mass_out + n1.mass_out,
            n0.exergy_out + n1.exergy_out,
            0.7,
            8.0,
        );

        assert!(
            (chain.per_node_heat[2] - n2.heat_dissipated).abs() < 1e-2,
            "join node heat: {} vs {}",
            chain.per_node_heat[2],
            n2.heat_dissipated
        );
        assert!(
            (chain.final_exergy - n2.exergy_out).abs() < 1e-2,
            "final exergy: {} vs {}",
            chain.final_exergy,
            n2.exergy_out
        );
    }

    #[test]
    fn chain_determinism_100_runs() {
        let graph = build_linear_3_graph();
        let first = evaluate_metabolic_chain(&graph, 100.0, 500.0);
        for _ in 0..100 {
            let run = evaluate_metabolic_chain(&graph, 100.0, 500.0);
            assert_eq!(
                first.final_exergy.to_bits(),
                run.final_exergy.to_bits(),
                "non-deterministic: {} vs {}",
                first.final_exergy,
                run.final_exergy,
            );
            assert_eq!(first.total_heat.to_bits(), run.total_heat.to_bits());
            assert_eq!(first.total_waste.to_bits(), run.total_waste.to_bits());
        }
    }

    #[test]
    fn chain_single_node_final_exergy_equals_output() {
        let graph = MetabolicGraphBuilder::new()
            .add_node(OrganRole::Root, 0.8, 5.0)
            .build()
            .unwrap();
        let chain = evaluate_metabolic_chain(&graph, 100.0, 500.0);
        let expected = organ_transform(100.0, 500.0, 0.8, 5.0);
        assert!((chain.final_exergy - expected.exergy_out).abs() < 1e-3);
        assert!((chain.total_heat - expected.heat_dissipated).abs() < 1e-3);
    }

    #[test]
    fn chain_empty_graph_returns_zeros() {
        let graph = MetabolicGraph::empty();
        let chain = evaluate_metabolic_chain(&graph, 100.0, 500.0);
        assert_eq!(chain.final_exergy, 0.0);
        assert_eq!(chain.total_heat, 0.0);
        assert_eq!(chain.total_waste, 0.0);
    }

    #[test]
    fn chain_zero_input_all_zero() {
        let graph = build_linear_3_graph();
        let chain = evaluate_metabolic_chain(&graph, 0.0, 0.0);
        assert_eq!(chain.total_heat, 0.0);
        assert_eq!(chain.total_waste, 0.0);
        assert_eq!(chain.final_exergy, 0.0);
    }

    #[test]
    fn chain_unused_node_slots_zero_heat() {
        let graph = build_linear_3_graph(); // 3 nodes
        let chain = evaluate_metabolic_chain(&graph, 100.0, 500.0);
        for i in 3..METABOLIC_GRAPH_MAX_NODES {
            assert_eq!(chain.per_node_heat[i], 0.0, "slot {i} should be zero");
        }
    }
}
