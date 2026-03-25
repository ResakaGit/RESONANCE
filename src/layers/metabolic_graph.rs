//! DAG metabólico (MG-2): nodos de exergía, aristas dirigidas, topología.
//! Almacenamiento en stack (sin alloc); campos derivados (MG-3) inicializados a cero.

use bevy::prelude::*;

use crate::layers::OrganRole;

/// Techo de nodos por grafo metabólico (1 por órgano, max 12 roles).
pub const METABOLIC_GRAPH_MAX_NODES: usize = 12;
/// Techo de aristas por grafo metabólico.
pub const METABOLIC_GRAPH_MAX_EDGES: usize = 16;

// ── Nodo ──────────────────────────────────────────────────────────────

/// Nodo funcional del DAG (órgano lógico).
#[derive(Clone, Copy, Debug, PartialEq, Reflect)]
pub struct ExergyNode {
    pub role:              OrganRole,
    pub efficiency:        f32,
    pub activation_energy: f32,
    pub thermal_output:    f32,
    pub entropy_rate:      f32,
}

impl Default for ExergyNode {
    fn default() -> Self {
        Self {
            role:              OrganRole::Stem,
            efficiency:        0.0,
            activation_energy: 0.0,
            thermal_output:    0.0,
            entropy_rate:      0.0,
        }
    }
}

// ── Arista ────────────────────────────────────────────────────────────

/// Arista dirigida: topología + datos de flujo entre nodos.
#[derive(Clone, Copy, Debug, PartialEq, Reflect)]
pub struct ExergyEdge {
    pub from:           u8,
    pub to:             u8,
    pub flow_rate:      f32,
    pub max_capacity:   f32,
    pub transport_cost: f32,
}

impl Default for ExergyEdge {
    fn default() -> Self {
        Self { from: 0, to: 0, flow_rate: 0.0, max_capacity: 0.0, transport_cost: 0.0 }
    }
}

// ── Componente ────────────────────────────────────────────────────────

/// DAG metabólico — SparseSet; solo entidades vivas complejas.
#[derive(Component, Clone, Copy, Debug, PartialEq, Reflect)]
#[reflect(Component)]
#[component(storage = "SparseSet")]
pub struct MetabolicGraph {
    nodes:              [ExergyNode; METABOLIC_GRAPH_MAX_NODES],
    nodes_len:          u8,
    edges:              [ExergyEdge; METABOLIC_GRAPH_MAX_EDGES],
    edges_len:          u8,
    total_entropy_rate: f32,
}

impl Default for MetabolicGraph {
    fn default() -> Self { Self::empty() }
}

impl MetabolicGraph {
    pub fn empty() -> Self {
        Self {
            nodes:              [ExergyNode::default(); METABOLIC_GRAPH_MAX_NODES],
            nodes_len:          0,
            edges:              [ExergyEdge::default(); METABOLIC_GRAPH_MAX_EDGES],
            edges_len:          0,
            total_entropy_rate: 0.0,
        }
    }

    #[inline] pub fn nodes(&self) -> &[ExergyNode]            { &self.nodes[..self.nodes_len as usize] }
    #[inline] pub fn nodes_mut(&mut self) -> &mut [ExergyNode] { &mut self.nodes[..self.nodes_len as usize] }
    #[inline] pub fn edges(&self) -> &[ExergyEdge]            { &self.edges[..self.edges_len as usize] }
    #[inline] pub fn edges_mut(&mut self) -> &mut [ExergyEdge] { &mut self.edges[..self.edges_len as usize] }
    #[inline] pub fn node_count(&self) -> usize                { self.nodes_len as usize }
    #[inline] pub fn edge_count(&self) -> usize                { self.edges_len as usize }
    #[inline] pub fn total_entropy_rate(&self) -> f32          { self.total_entropy_rate }

    #[inline]
    pub fn set_total_entropy_rate(&mut self, val: f32) {
        if self.total_entropy_rate != val {
            self.total_entropy_rate = val;
        }
    }
}

// ── Error ─────────────────────────────────────────────────────────────

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MetabolicGraphError {
    Empty,
    NoCaptorNode,
    InvalidIndex,
    DuplicateEdge,
    Cycle,
}

// ── Builder ───────────────────────────────────────────────────────────

/// Builder fluido; valida aciclicidad, índices, y mínimo funcional.
#[derive(Clone, Debug)]
pub struct MetabolicGraphBuilder {
    nodes:     [ExergyNode; METABOLIC_GRAPH_MAX_NODES],
    nodes_len: u8,
    edges:     [ExergyEdge; METABOLIC_GRAPH_MAX_EDGES],
    edges_len: u8,
}

impl Default for MetabolicGraphBuilder {
    fn default() -> Self { Self::new() }
}

impl MetabolicGraphBuilder {
    pub fn new() -> Self {
        Self {
            nodes:     [ExergyNode::default(); METABOLIC_GRAPH_MAX_NODES],
            nodes_len: 0,
            edges:     [ExergyEdge::default(); METABOLIC_GRAPH_MAX_EDGES],
            edges_len: 0,
        }
    }

    /// Agrega un nodo. Capped silenciosamente en `METABOLIC_GRAPH_MAX_NODES`.
    pub fn add_node(mut self, role: OrganRole, efficiency: f32, activation_energy: f32) -> Self {
        let n = self.nodes_len as usize;
        if n < METABOLIC_GRAPH_MAX_NODES {
            self.nodes[n] = ExergyNode {
                role,
                efficiency,
                activation_energy,
                thermal_output: 0.0,
                entropy_rate:   0.0,
            };
            self.nodes_len += 1;
        }
        self
    }

    /// Agrega una arista dirigida. Capped silenciosamente en `METABOLIC_GRAPH_MAX_EDGES`.
    pub fn add_edge(mut self, from: u8, to: u8, max_capacity: f32) -> Self {
        let e = self.edges_len as usize;
        if e < METABOLIC_GRAPH_MAX_EDGES {
            self.edges[e] = ExergyEdge {
                from,
                to,
                flow_rate: 0.0,
                max_capacity,
                transport_cost: 0.0,
            };
            self.edges_len += 1;
        }
        self
    }

    /// Valida y construye el DAG metabólico.
    pub fn build(self) -> Result<MetabolicGraph, MetabolicGraphError> {
        let n = self.nodes_len as usize;
        if n == 0 {
            return Err(MetabolicGraphError::Empty);
        }

        let has_captor = self.nodes[..n].iter().any(|node| {
            matches!(node.role, OrganRole::Root | OrganRole::Leaf | OrganRole::Sensory)
        });
        if !has_captor {
            return Err(MetabolicGraphError::NoCaptorNode);
        }

        let e = self.edges_len as usize;
        for i in 0..e {
            let edge = self.edges[i];
            if edge.from == edge.to || edge.from as usize >= n || edge.to as usize >= n {
                return Err(MetabolicGraphError::InvalidIndex);
            }
        }

        for i in 0..e {
            for j in (i + 1)..e {
                if self.edges[i].from == self.edges[j].from
                    && self.edges[i].to == self.edges[j].to
                {
                    return Err(MetabolicGraphError::DuplicateEdge);
                }
            }
        }

        if has_cycle(n, &self.edges[..e]) {
            return Err(MetabolicGraphError::Cycle);
        }

        Ok(MetabolicGraph {
            nodes:              self.nodes,
            nodes_len:          self.nodes_len,
            edges:              self.edges,
            edges_len:          self.edges_len,
            total_entropy_rate: 0.0,
        })
    }
}

// ── Topological validation ────────────────────────────────────────────

/// Kahn: si no se procesan todos los nodos, hay ciclo.
fn has_cycle(n: usize, edges: &[ExergyEdge]) -> bool {
    if n <= 1 {
        return false;
    }
    let mut in_degree = [0u8; METABOLIC_GRAPH_MAX_NODES];
    for edge in edges {
        let to = edge.to as usize;
        if to < n {
            in_degree[to] = in_degree[to].saturating_add(1);
        }
    }
    let mut queue = [0u8; METABOLIC_GRAPH_MAX_NODES];
    let mut head  = 0usize;
    let mut tail  = 0usize;
    for i in 0..n {
        if in_degree[i] == 0 {
            queue[tail] = i as u8;
            tail += 1;
        }
    }
    let mut processed = 0usize;
    while head < tail {
        let u = queue[head] as usize;
        head += 1;
        processed += 1;
        for edge in edges {
            if edge.from as usize == u {
                let v = edge.to as usize;
                if v < n && in_degree[v] > 0 {
                    in_degree[v] -= 1;
                    if in_degree[v] == 0 {
                        queue[tail] = v as u8;
                        tail += 1;
                    }
                }
            }
        }
    }
    processed != n
}

// ── Tests ─────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    // ── MG-2A: tipos core ──

    #[test]
    fn exergy_node_is_copy_and_clone() {
        let a = ExergyNode { role: OrganRole::Root, ..Default::default() };
        let b = a;
        let c = a.clone();
        assert_eq!(a, b);
        assert_eq!(a, c);
    }

    #[test]
    fn empty_graph_has_zero_counts() {
        let g = MetabolicGraph::empty();
        assert_eq!(g.node_count(), 0);
        assert_eq!(g.edge_count(), 0);
        assert_eq!(g.total_entropy_rate(), 0.0);
    }

    #[test]
    fn build_empty_returns_err() {
        assert_eq!(MetabolicGraphBuilder::new().build(), Err(MetabolicGraphError::Empty));
    }

    #[test]
    fn build_max_nodes_and_edges_ok() {
        let mut b = MetabolicGraphBuilder::new()
            .add_node(OrganRole::Root, 0.5, 1.0);
        for _ in 1..METABOLIC_GRAPH_MAX_NODES {
            b = b.add_node(OrganRole::Core, 0.5, 1.0);
        }
        for i in 0..(METABOLIC_GRAPH_MAX_NODES - 1) {
            b = b.add_edge(i as u8, (i + 1) as u8, 10.0);
        }
        let g = b.build().unwrap();
        assert_eq!(g.node_count(), METABOLIC_GRAPH_MAX_NODES);
        assert_eq!(g.edge_count(), METABOLIC_GRAPH_MAX_NODES - 1);
    }

    #[test]
    fn overflow_nodes_capped_at_max() {
        let mut b = MetabolicGraphBuilder::new();
        for _ in 0..METABOLIC_GRAPH_MAX_NODES + 5 {
            b = b.add_node(OrganRole::Root, 0.5, 1.0);
        }
        let g = b.build().unwrap();
        assert_eq!(g.node_count(), METABOLIC_GRAPH_MAX_NODES);
    }

    // ── MG-2B: validación del builder ──

    #[test]
    fn cycle_a_b_a_rejected() {
        let r = MetabolicGraphBuilder::new()
            .add_node(OrganRole::Root, 0.5, 1.0)
            .add_node(OrganRole::Stem, 0.5, 1.0)
            .add_edge(0, 1, 10.0)
            .add_edge(1, 0, 10.0)
            .build();
        assert_eq!(r, Err(MetabolicGraphError::Cycle));
    }

    #[test]
    fn no_captor_rejected() {
        let r = MetabolicGraphBuilder::new()
            .add_node(OrganRole::Stem, 0.5, 1.0)
            .add_node(OrganRole::Core, 0.5, 1.0)
            .add_edge(0, 1, 10.0)
            .build();
        assert_eq!(r, Err(MetabolicGraphError::NoCaptorNode));
    }

    #[test]
    fn self_loop_rejected() {
        let r = MetabolicGraphBuilder::new()
            .add_node(OrganRole::Root, 0.5, 1.0)
            .add_edge(0, 0, 10.0)
            .build();
        assert_eq!(r, Err(MetabolicGraphError::InvalidIndex));
    }

    #[test]
    fn out_of_bounds_edge_rejected() {
        let r = MetabolicGraphBuilder::new()
            .add_node(OrganRole::Root, 0.5, 1.0)
            .add_edge(0, 5, 10.0)
            .build();
        assert_eq!(r, Err(MetabolicGraphError::InvalidIndex));
    }

    #[test]
    fn duplicate_edge_rejected() {
        let r = MetabolicGraphBuilder::new()
            .add_node(OrganRole::Root, 0.5, 1.0)
            .add_node(OrganRole::Stem, 0.5, 1.0)
            .add_edge(0, 1, 10.0)
            .add_edge(0, 1, 20.0)
            .build();
        assert_eq!(r, Err(MetabolicGraphError::DuplicateEdge));
    }

    #[test]
    fn valid_chain_ok() {
        let g = MetabolicGraphBuilder::new()
            .add_node(OrganRole::Root, 0.9, 3.0)
            .add_node(OrganRole::Core, 0.7, 8.0)
            .add_node(OrganRole::Stem, 0.8, 5.0)
            .add_edge(0, 1, 50.0)
            .add_edge(1, 2, 40.0)
            .build()
            .unwrap();
        assert_eq!(g.node_count(), 3);
        assert_eq!(g.edge_count(), 2);
    }

    #[test]
    fn valid_tree_one_captor_three_terminals() {
        let g = MetabolicGraphBuilder::new()
            .add_node(OrganRole::Root, 0.9, 3.0)
            .add_node(OrganRole::Stem, 0.8, 5.0)
            .add_node(OrganRole::Core, 0.7, 8.0)
            .add_node(OrganRole::Fin,  0.8, 5.0)
            .add_edge(0, 1, 50.0)
            .add_edge(0, 2, 50.0)
            .add_edge(0, 3, 50.0)
            .build()
            .unwrap();
        assert_eq!(g.node_count(), 4);
        assert_eq!(g.edge_count(), 3);
    }

    // ── Accessors ──

    #[test]
    fn accessors_return_correct_slices() {
        let g = MetabolicGraphBuilder::new()
            .add_node(OrganRole::Root, 0.9, 3.0)
            .add_node(OrganRole::Stem, 0.8, 5.0)
            .add_edge(0, 1, 50.0)
            .build()
            .unwrap();
        assert_eq!(g.nodes().len(), 2);
        assert_eq!(g.edges().len(), 1);
        assert_eq!(g.nodes()[0].role, OrganRole::Root);
        assert_eq!(g.edges()[0].max_capacity, 50.0);
        assert_eq!(g.edges()[0].from, 0);
        assert_eq!(g.edges()[0].to, 1);
    }

    #[test]
    fn nodes_mut_allows_derived_update() {
        let mut g = MetabolicGraphBuilder::new()
            .add_node(OrganRole::Root, 0.9, 3.0)
            .build()
            .unwrap();
        g.nodes_mut()[0].thermal_output = 42.0;
        assert_eq!(g.nodes()[0].thermal_output, 42.0);
    }

    #[test]
    fn set_total_entropy_rate_persists() {
        let mut g = MetabolicGraphBuilder::new()
            .add_node(OrganRole::Leaf, 0.9, 2.0)
            .build()
            .unwrap();
        g.set_total_entropy_rate(1.5);
        assert_eq!(g.total_entropy_rate(), 1.5);
    }

    #[test]
    fn set_total_entropy_rate_guards_equal_value() {
        let mut g = MetabolicGraphBuilder::new()
            .add_node(OrganRole::Leaf, 0.9, 2.0)
            .build()
            .unwrap();
        g.set_total_entropy_rate(1.5);
        g.set_total_entropy_rate(1.5); // no-op, guard prevents unnecessary write
        assert_eq!(g.total_entropy_rate(), 1.5);
    }

    // ── Verifier-requested: missing coverage ──

    #[test]
    fn three_node_cycle_rejected() {
        let r = MetabolicGraphBuilder::new()
            .add_node(OrganRole::Root,   0.5, 1.0)
            .add_node(OrganRole::Stem,   0.5, 1.0)
            .add_node(OrganRole::Core,   0.5, 1.0)
            .add_edge(0, 1, 10.0)
            .add_edge(1, 2, 10.0)
            .add_edge(2, 0, 10.0)
            .build();
        assert_eq!(r, Err(MetabolicGraphError::Cycle));
    }

    #[test]
    fn diamond_dag_accepted() {
        let g = MetabolicGraphBuilder::new()
            .add_node(OrganRole::Root, 0.9, 3.0) // 0
            .add_node(OrganRole::Stem, 0.8, 5.0) // 1
            .add_node(OrganRole::Core, 0.7, 8.0) // 2
            .add_node(OrganRole::Fin,  0.8, 5.0) // 3
            .add_edge(0, 1, 50.0)
            .add_edge(0, 2, 50.0)
            .add_edge(1, 3, 40.0)
            .add_edge(2, 3, 40.0) // fan-in convergent
            .build()
            .unwrap();
        assert_eq!(g.node_count(), 4);
        assert_eq!(g.edge_count(), 4);
    }

    #[test]
    fn single_captor_node_no_edges_ok() {
        let g = MetabolicGraphBuilder::new()
            .add_node(OrganRole::Root, 0.9, 3.0)
            .build()
            .unwrap();
        assert_eq!(g.node_count(), 1);
        assert_eq!(g.edge_count(), 0);
    }

    #[test]
    fn sensory_as_sole_captor_accepted() {
        let g = MetabolicGraphBuilder::new()
            .add_node(OrganRole::Sensory, 0.5, 4.0)
            .add_node(OrganRole::Core,    0.7, 8.0)
            .add_edge(0, 1, 30.0)
            .build()
            .unwrap();
        assert_eq!(g.nodes()[0].role, OrganRole::Sensory);
    }

    #[test]
    fn overflow_edges_capped_at_max() {
        // 12 nodos → star topology permite hasta 11 aristas únicas desde nodo 0.
        // Agregamos 5 más duplicadas → builder capped, build detecta duplicados.
        let mut b = MetabolicGraphBuilder::new()
            .add_node(OrganRole::Root, 0.5, 1.0)
            .add_node(OrganRole::Stem, 0.5, 1.0);
        for _ in 0..METABOLIC_GRAPH_MAX_EDGES + 5 {
            b = b.add_edge(0, 1, 10.0);
        }
        // Prueba que no crashea por overflow y que cap funciona
        let r = b.build();
        assert_eq!(r, Err(MetabolicGraphError::DuplicateEdge));
    }
}
