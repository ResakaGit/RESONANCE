//! MG-3B/C/6D/7B — DAG metabólico, EntropyLedger, shape, albedo, surface rugosity.
//!
//! Sistemas, una transformación cada uno:
//! - `metabolic_graph_step_system`: propaga flujos J por aristas, computa Q_diss y S_gen por nodo.
//! - `entropy_constraint_system`: reclampea eficiencias al techo de Carnot actual.
//! - `entropy_ledger_system`: materializa EntropyLedger desde evaluate_metabolic_chain (MG-6D).
//! - `shape_optimization_system`: ajusta fineness_ratio por descenso acotado (MG-4).
//! - `albedo_inference_system`: infiere albedo desde balance radiativo (MG-5).
//! - `surface_rugosity_system`: infiere rugosidad desde Q/V ratio (MG-7).
//!
//! **Mapeo ECS → escalar (convención MG-3):**
//! - T_core = `equivalent_temperature(density(qe, radius))` — Capa 0 × Capa 1.
//! - T_env  = `ambient_equivalent_temperature(terrain_viscosity)` — Capa 6.
//! - Ambos via funciones puras en `blueprint::equations`.

use bevy::prelude::*;

use crate::blueprint::constants::{METABOLIC_MIN_FLOW, METABOLIC_STARVATION_THRESHOLD, METABOLIC_STEP_EPSILON};
use crate::blueprint::constants::morphogenesis as mg;
use crate::blueprint::equations;
use crate::layers::{
    AmbientPressure, BaseEnergy, EntropyLedger, FlowVector, InferredAlbedo, IrradianceReceiver,
    MetabolicGraph, MorphogenesisSurface, MorphogenesisShapeParams, SpatialVolume,
    METABOLIC_GRAPH_MAX_EDGES, METABOLIC_GRAPH_MAX_NODES,
};

// ── MG-3B: Propagación de flujos por tick ───────────────────────────

/// Propaga flujos J por aristas y computa Q_diss, S_gen por nodo.
/// Conservación por nodo: J_in = Σ J_out + Q_diss (+ E_a absorbida).
pub fn metabolic_graph_step_system(
    mut query: Query<
        (&mut MetabolicGraph, &BaseEnergy, &AmbientPressure, &SpatialVolume),
    >,
) {
    for (mut graph, energy, _pressure, volume) in &mut query {
        let qe = energy.qe();
        if graph.node_count() == 0 {
            continue;
        }

        if qe < METABOLIC_STARVATION_THRESHOLD {
            collapse_flows(&mut graph);
            continue;
        }

        let t_core = equations::equivalent_temperature(volume.density(qe));
        step_dag(&mut graph, qe, t_core);
    }
}

// ── MG-3C: Constraint de Carnot ─────────────────────────────────────

/// Reclampea eficiencias al techo de Carnot por condiciones ambientales actuales.
pub fn entropy_constraint_system(
    mut query: Query<
        (&mut MetabolicGraph, &BaseEnergy, &AmbientPressure, &SpatialVolume),
    >,
) {
    for (mut graph, energy, pressure, volume) in &mut query {
        if graph.node_count() == 0 {
            continue;
        }

        let qe = energy.qe();
        let t_core = equations::equivalent_temperature(volume.density(qe));
        let t_env = equations::ambient_equivalent_temperature(pressure.terrain_viscosity);
        let eta_carnot = equations::carnot_efficiency(t_core, t_env);

        let mut changed = false;
        for node in graph.nodes_mut() {
            // extra_heat = (η_old - η_new) × J_in_estimado.
            // J_in no se almacena; estimación conservadora: thermal_output es la disipación
            // original, y el J_in que lo generó fue ≥ thermal_output.
            let estimated_j_in = node.thermal_output + node.activation_energy;
            let (new_eff, extra_heat) = equations::redistribute_node_violation(
                node.efficiency,
                eta_carnot,
                estimated_j_in,
                0.0, // E_a ya descontada en estimated_j_in → effective_input = estimated_j_in.
            );
            if (node.efficiency - new_eff).abs() > METABOLIC_STEP_EPSILON {
                node.efficiency = new_eff;
                node.thermal_output += extra_heat;
                changed = true;
            }
        }

        if changed {
            let total_s: f32 = graph
                .nodes()
                .iter()
                .map(|n| equations::entropy_production(n.thermal_output, t_core))
                .sum();
            if (graph.total_entropy_rate() - total_s).abs() > METABOLIC_STEP_EPSILON {
                graph.set_total_entropy_rate(total_s);
            }
        }
    }
}

// ── Tipos internos ──────────────────────────────────────────────────

/// Snapshot de arista para lectura desacoplada del grafo (evita borrow conflict).
#[derive(Clone, Copy)]
struct EdgeSnap {
    from:           u8,
    to:             u8,
    max_capacity:   f32,
    transport_cost: f32,
}

/// Adjacency list pre-computada: outgoing edges por nodo.
/// `starts[i]..starts[i+1]` = rango de edge indices en `edge_indices` para el nodo i.
struct OutgoingEdges {
    edge_indices: [u8; METABOLIC_GRAPH_MAX_EDGES],
    starts:       [u8; METABOLIC_GRAPH_MAX_NODES + 1],
}

impl OutgoingEdges {
    /// Pre-computa adjacency en O(E) desde el snapshot de aristas.
    fn build(edges: &[EdgeSnap], edge_count: usize, node_count: usize) -> Self {
        let mut counts = [0u8; METABOLIC_GRAPH_MAX_NODES];
        for ei in 0..edge_count {
            let from = edges[ei].from as usize;
            if from < node_count {
                counts[from] = counts[from].saturating_add(1);
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
        for ei in 0..edge_count {
            let from = edges[ei].from as usize;
            if from < node_count {
                let pos = offsets[from] as usize;
                if pos < METABOLIC_GRAPH_MAX_EDGES {
                    edge_indices[pos] = ei as u8;
                    offsets[from] = offsets[from].saturating_add(1);
                }
            }
        }

        Self { edge_indices, starts }
    }

    /// Itera los indices de aristas salientes del nodo `node_idx`.
    #[inline]
    fn outgoing(&self, node_idx: usize) -> &[u8] {
        let start = self.starts[node_idx] as usize;
        let end = self.starts[node_idx + 1] as usize;
        &self.edge_indices[start..end]
    }
}

// ── Lógica interna (funciones libres, stateless) ────────────────────

/// Colapsa todos los flujos y outputs a cero (inanición).
fn collapse_flows(graph: &mut MetabolicGraph) {
    let has_nonzero = graph.edges().iter().any(|e| e.flow_rate.abs() > METABOLIC_STEP_EPSILON)
        || graph.nodes().iter().any(|n| {
            n.thermal_output.abs() > METABOLIC_STEP_EPSILON
                || n.entropy_rate.abs() > METABOLIC_STEP_EPSILON
        })
        || graph.total_entropy_rate().abs() > METABOLIC_STEP_EPSILON;

    if !has_nonzero {
        return;
    }

    for edge in graph.edges_mut() {
        edge.flow_rate = 0.0;
    }
    for node in graph.nodes_mut() {
        node.thermal_output = 0.0;
        node.entropy_rate = 0.0;
    }
    graph.set_total_entropy_rate(0.0);
}

/// Propaga flujos a través del DAG en orden topológico (índice ascendente).
/// Nodos raíz (in-degree 0) reciben porción de qe como J_in.
fn step_dag(graph: &mut MetabolicGraph, qe: f32, t_core: f32) {
    let node_count = graph.node_count();
    let edge_count = graph.edge_count();

    let (in_degree, mut j_in) = compute_in_degree_and_root_injection(graph, qe);
    let (order, order_len) = topological_order(&in_degree, graph.edges(), node_count, edge_count);

    // Snapshot de edges (evita borrow conflict con nodes_mut).
    let mut edge_snap = [EdgeSnap { from: 0, to: 0, max_capacity: 0.0, transport_cost: 0.0 }; METABOLIC_GRAPH_MAX_EDGES];
    for (i, e) in graph.edges().iter().enumerate() {
        edge_snap[i] = EdgeSnap {
            from:           e.from,
            to:             e.to,
            max_capacity:   e.max_capacity,
            transport_cost: e.transport_cost,
        };
    }

    // Adjacency pre-computada: O(E) en vez de O(N*E) por nodo.
    let adj = OutgoingEdges::build(&edge_snap, edge_count, node_count);

    let (node_thermal, node_entropy, edge_flows) =
        propagate_flows(graph, &order, order_len, &mut j_in, &edge_snap, &adj, t_core);

    write_results(graph, &node_thermal, &node_entropy, &edge_flows, node_count, edge_count);
}

/// Calcula in-degree por nodo e inyecta J_in en nodos raíz (in-degree 0).
fn compute_in_degree_and_root_injection(
    graph: &MetabolicGraph,
    qe: f32,
) -> ([u8; METABOLIC_GRAPH_MAX_NODES], [f32; METABOLIC_GRAPH_MAX_NODES]) {
    let node_count = graph.node_count();
    let mut in_degree = [0u8; METABOLIC_GRAPH_MAX_NODES];
    let mut j_in = [0.0f32; METABOLIC_GRAPH_MAX_NODES];

    for e in graph.edges() {
        let to = e.to as usize;
        if to < node_count {
            in_degree[to] = in_degree[to].saturating_add(1);
        }
    }

    let root_count = in_degree[..node_count].iter().filter(|&&d| d == 0).count();
    if root_count > 0 {
        let share = qe / root_count as f32;
        for (i, deg) in in_degree[..node_count].iter().enumerate() {
            if *deg == 0 {
                j_in[i] = share;
            }
        }
    }

    (in_degree, j_in)
}

/// Kahn con inserción ordenada (determinismo sin RNG).
fn topological_order(
    in_degree: &[u8; METABOLIC_GRAPH_MAX_NODES],
    edges: &[crate::layers::ExergyEdge],
    node_count: usize,
    edge_count: usize,
) -> ([u8; METABOLIC_GRAPH_MAX_NODES], usize) {
    let mut pending = *in_degree;
    let mut order = [0u8; METABOLIC_GRAPH_MAX_NODES];
    let mut order_len = 0usize;
    let mut processed = [false; METABOLIC_GRAPH_MAX_NODES];

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
        processed[u] = true;

        for ei in 0..edge_count {
            if edges[ei].from as usize != u {
                continue;
            }
            let v = edges[ei].to as usize;
            if v >= node_count || processed[v] {
                continue;
            }
            pending[v] = pending[v].saturating_sub(1);
            if pending[v] == 0 && order_len < node_count {
                sorted_insert(&mut order, head, &mut order_len, v as u8);
            }
        }
    }

    (order, order_len)
}

/// Inserta `val` en `order[head..len]` manteniendo orden ascendente.
fn sorted_insert(order: &mut [u8; METABOLIC_GRAPH_MAX_NODES], head: usize, len: &mut usize, val: u8) {
    let mut pos = *len;
    while pos > head && order[pos - 1] > val {
        order[pos] = order[pos - 1];
        pos -= 1;
    }
    order[pos] = val;
    *len += 1;
}

/// Procesa nodos en orden topológico: balance de exergía + distribución de flujo.
fn propagate_flows(
    graph: &MetabolicGraph,
    order: &[u8; METABOLIC_GRAPH_MAX_NODES],
    order_len: usize,
    j_in: &mut [f32; METABOLIC_GRAPH_MAX_NODES],
    edge_snap: &[EdgeSnap; METABOLIC_GRAPH_MAX_EDGES],
    adj: &OutgoingEdges,
    t_core: f32,
) -> ([f32; METABOLIC_GRAPH_MAX_NODES], [f32; METABOLIC_GRAPH_MAX_NODES], [f32; METABOLIC_GRAPH_MAX_EDGES]) {
    let node_count = graph.node_count();
    let mut node_thermal = [0.0f32; METABOLIC_GRAPH_MAX_NODES];
    let mut node_entropy = [0.0f32; METABOLIC_GRAPH_MAX_NODES];
    let mut edge_flows = [0.0f32; METABOLIC_GRAPH_MAX_EDGES];

    for &node_idx in &order[..order_len] {
        let ni = node_idx as usize;
        let node = &graph.nodes()[ni];
        let available = j_in[ni];

        let useful = equations::exergy_balance(available, node.efficiency, node.activation_energy);
        let q_diss = (available - useful).max(0.0);

        node_thermal[ni] = q_diss;
        node_entropy[ni] = equations::entropy_production(q_diss, t_core);

        // Aristas salientes via adjacency pre-computada (O(out-degree), no O(E)).
        let out = adj.outgoing(ni);
        let mut out_caps = [(0u8, 0.0f32); METABOLIC_GRAPH_MAX_EDGES];
        for (k, &ei) in out.iter().enumerate() {
            out_caps[k] = (ei, edge_snap[ei as usize].max_capacity);
        }

        let (flows, flow_count) = equations::propagate_edge_flows(useful, &out_caps[..out.len()]);
        for i in 0..flow_count {
            let (ei_local, flow) = flows[i];
            let ei = ei_local as usize;
            let flow_clamped = if flow >= METABOLIC_MIN_FLOW { flow } else { 0.0 };
            edge_flows[ei] = flow_clamped;

            let snap = &edge_snap[ei];
            let transport_loss = snap.transport_cost.min(flow_clamped);
            let delivered = (flow_clamped - transport_loss).max(0.0);
            node_thermal[ni] += transport_loss;
            let ti = snap.to as usize;
            if ti < node_count {
                j_in[ti] += delivered;
            }
        }
    }

    (node_thermal, node_entropy, edge_flows)
}

/// Escribe resultados al grafo con guard change detection.
fn write_results(
    graph: &mut MetabolicGraph,
    node_thermal: &[f32; METABOLIC_GRAPH_MAX_NODES],
    node_entropy: &[f32; METABOLIC_GRAPH_MAX_NODES],
    edge_flows: &[f32; METABOLIC_GRAPH_MAX_EDGES],
    node_count: usize,
    edge_count: usize,
) {
    let mut total_entropy = 0.0f32;
    for ni in 0..node_count {
        let node = &mut graph.nodes_mut()[ni];
        if (node.thermal_output - node_thermal[ni]).abs() > METABOLIC_STEP_EPSILON {
            node.thermal_output = node_thermal[ni];
        }
        if (node.entropy_rate - node_entropy[ni]).abs() > METABOLIC_STEP_EPSILON {
            node.entropy_rate = node_entropy[ni];
        }
        total_entropy += node_entropy[ni];
    }
    for ei in 0..edge_count {
        let edge = &mut graph.edges_mut()[ei];
        if (edge.flow_rate - edge_flows[ei]).abs() > METABOLIC_STEP_EPSILON {
            edge.flow_rate = edge_flows[ei];
        }
    }
    if (graph.total_entropy_rate() - total_entropy).abs() > METABOLIC_STEP_EPSILON {
        graph.set_total_entropy_rate(total_entropy);
    }
}

// ── MG-6D: EntropyLedger materialización ────────────────────────

/// Materializa EntropyLedger desde evaluate_metabolic_chain cada tick.
/// Phase: MetabolicLayer, after entropy_constraint_system.
pub fn entropy_ledger_system(
    mut commands: Commands,
    query: Query<(Entity, &MetabolicGraph, &BaseEnergy, &SpatialVolume)>,
    mut ledger_query: Query<&mut EntropyLedger>,
) {
    for (entity, graph, energy, volume) in &query {
        if graph.node_count() == 0 {
            continue;
        }

        let qe = energy.qe();
        let t_core = equations::equivalent_temperature(volume.density(qe));
        let initial_exergy = qe;
        let initial_mass = qe; // proxy: mass = qe in model

        let chain = equations::evaluate_metabolic_chain(graph, initial_mass, initial_exergy);

        let s_gen = equations::entropy_production(chain.total_heat, t_core);
        let eta = equations::exergy_efficiency(chain.final_exergy, initial_exergy);

        let new_ledger = EntropyLedger {
            total_heat_generated:  chain.total_heat,
            total_waste_generated: chain.total_waste,
            entropy_rate:          s_gen,
            exergy_efficiency:     eta,
        };

        if let Ok(mut existing) = ledger_query.get_mut(entity) {
            if *existing != new_ledger {
                *existing = new_ledger;
            }
        } else {
            commands.entity(entity).insert(new_ledger);
        }
    }
}

// ── MG-4: Shape Optimization ─────────────────────────────────────

/// Σ transport_cost de aristas del DAG.
fn graph_vascular_cost(graph: &MetabolicGraph) -> f32 {
    graph.edges().iter().map(|e| e.transport_cost).sum()
}

/// Ajusta fineness_ratio minimizando shape_cost por descenso acotado (MG-4).
pub fn shape_optimization_system(
    mut query: Query<
        (&MetabolicGraph, &FlowVector, &AmbientPressure, &SpatialVolume,
         &mut MorphogenesisShapeParams),
    >,
) {
    for (graph, flow, pressure, volume, mut shape) in &mut query {
        let velocity  = flow.speed();
        let density   = pressure.terrain_viscosity;
        let radius    = volume.radius;
        let proj_area = equations::projected_circle_area(radius);
        let vasc_cost = graph_vascular_cost(graph);

        let new_fineness = equations::bounded_fineness_descent(
            shape.fineness_ratio(),
            density,
            velocity,
            proj_area,
            vasc_cost,
            mg::SHAPE_OPTIMIZER_DAMPING,
            mg::SHAPE_OPTIMIZER_MAX_ITER,
        );

        let diameter = radius * 2.0;
        let new_cost = equations::shape_cost(
            density,
            velocity,
            equations::inferred_drag_coefficient(new_fineness * diameter, diameter),
            proj_area,
            vasc_cost,
        );

        shape.update(new_fineness, diameter, new_cost);
    }
}

// ── MG-5: Albedo Inference ──────────────────────────────────────────

/// Infiere albedo desde balance radiativo: Q_met, irradiancia, geometría, convección.
/// Phase: MorphologicalLayer, after shape_optimization_system.
///
/// Query >5 tipos justificado: la ecuación `inferred_albedo` requiere 8 parámetros
/// provenientes de capas ortogonales (L0 energy, L1 volume, L6 pressure, aux irradiance,
/// MG-6 ledger). Los 2 `Option` son fallback-safe y no amplían el archetype filter.
pub fn albedo_inference_system(
    mut commands: Commands,
    query: Query<
        (Entity, &MetabolicGraph, &BaseEnergy, &SpatialVolume, &AmbientPressure,
         Option<&IrradianceReceiver>, Option<&EntropyLedger>),
    >,
    mut albedo_query: Query<&mut InferredAlbedo>,
) {
    for (entity, graph, energy, volume, pressure, irradiance, ledger) in &query {
        if graph.node_count() == 0 {
            continue;
        }

        let qe = energy.qe();
        let t_core = equations::equivalent_temperature(volume.density(qe));
        let t_env = equations::ambient_equivalent_temperature(pressure.terrain_viscosity);

        // Q metabólico desde EntropyLedger (fuente canónica, MG-6).
        // Fallback: S_dot * T ≈ Q_dot (proxy termodinámico). Sobreestima Q si hay
        // entropía por activación. Eliminar cuando MG-6 sea obligatorio en el archetype.
        let q_met = ledger.map_or_else(
            || graph.total_entropy_rate() * t_core,
            |l| l.total_heat_generated,
        );

        let i_solar = irradiance
            .map(|ir| equations::irradiance_effective_for_albedo(ir.photon_density, ir.absorbed_fraction))
            .unwrap_or(0.0);

        let r = volume.radius;
        let proj_area = equations::projected_circle_area(r);
        let surf_area = equations::sphere_surface_area(r);

        let alpha = equations::inferred_albedo(
            q_met, i_solar, proj_area,
            mg::DEFAULT_EMISSIVITY, t_core, t_env,
            surf_area, mg::DEFAULT_CONVECTION_COEFF,
        );

        if let Ok(mut existing) = albedo_query.get_mut(entity) {
            if (existing.albedo() - alpha).abs() > mg::ALBEDO_EPSILON {
                existing.set_albedo(alpha);
            }
        } else {
            commands.entity(entity).insert(InferredAlbedo::new(alpha));
        }
    }
}

// ── MG-7B: Surface Rugosity ─────────────────────────────────────────

/// Infiere rugosidad de superficie desde balance térmico Q/V.
/// Phase: MorphologicalLayer, after shape_optimization_system, before albedo_inference_system.
pub fn surface_rugosity_system(
    mut commands: Commands,
    query: Query<(Entity, &EntropyLedger, &SpatialVolume, &AmbientPressure, &BaseEnergy)>,
    mut surface_query: Query<&mut MorphogenesisSurface>,
) {
    for (entity, ledger, volume, pressure, energy) in &query {
        let q_total = ledger.total_heat_generated;
        let vol = volume.volume();

        let qe = energy.qe();
        let t_core = equations::equivalent_temperature(volume.density(qe));
        let t_env = equations::ambient_equivalent_temperature(pressure.terrain_viscosity);
        let h = mg::DEFAULT_CONVECTION_COEFF;

        let rug = equations::inferred_surface_rugosity(q_total, vol, t_core, t_env, h);
        let qv = if vol > crate::blueprint::constants::DIVISION_GUARD_EPSILON {
            q_total / vol
        } else {
            0.0
        };

        let new_surface = MorphogenesisSurface::new(rug, qv);

        if let Ok(mut existing) = surface_query.get_mut(entity) {
            if (existing.rugosity() - new_surface.rugosity()).abs() > mg::RUGOSITY_EPSILON {
                *existing = new_surface;
            }
        } else {
            commands.entity(entity).insert(new_surface);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layers::{MetabolicGraphBuilder, OrganRole};

    // ── Helpers ──

    fn build_chain_graph() -> MetabolicGraph {
        MetabolicGraphBuilder::new()
            .add_node(OrganRole::Root, 0.9, 3.0)
            .add_node(OrganRole::Core, 0.7, 8.0)
            .add_node(OrganRole::Fin,  0.6, 5.0)
            .add_edge(0, 1, 50.0)
            .add_edge(1, 2, 40.0)
            .build()
            .unwrap()
    }

    fn build_fork_graph() -> MetabolicGraph {
        MetabolicGraphBuilder::new()
            .add_node(OrganRole::Root, 0.9, 3.0)
            .add_node(OrganRole::Stem, 0.8, 5.0)
            .add_node(OrganRole::Fin,  0.7, 4.0)
            .add_edge(0, 1, 60.0)
            .add_edge(0, 2, 40.0)
            .build()
            .unwrap()
    }

    fn spawn_metabolic_entity(
        world: &mut World,
        graph: MetabolicGraph,
        qe: f32,
        radius: f32,
        viscosity: f32,
    ) -> Entity {
        world.spawn((
            graph,
            BaseEnergy::new(qe),
            SpatialVolume::new(radius),
            AmbientPressure::new(0.0, viscosity),
        )).id()
    }

    fn run_step_system(world: &mut World) {
        let mut schedule = Schedule::default();
        schedule.add_systems(metabolic_graph_step_system);
        schedule.run(world);
    }

    fn run_constraint_system(world: &mut World) {
        let mut schedule = Schedule::default();
        schedule.add_systems(entropy_constraint_system);
        schedule.run(world);
    }

    // ── MG-3B: Step system ──

    #[test]
    fn step_chain_three_nodes_produces_positive_thermal_output() {
        let mut world = World::new();
        let e = spawn_metabolic_entity(&mut world, build_chain_graph(), 500.0, 2.0, 1.0);
        run_step_system(&mut world);

        let graph = world.entity(e).get::<MetabolicGraph>().unwrap();
        for node in graph.nodes() {
            assert!(
                node.thermal_output >= 0.0,
                "{:?} thermal_output = {} (expected >= 0)",
                node.role, node.thermal_output,
            );
        }
        assert!(graph.total_entropy_rate() >= 0.0);
    }

    #[test]
    fn step_chain_conservation_holds() {
        let mut world = World::new();
        let qe = 500.0;
        let e = spawn_metabolic_entity(&mut world, build_chain_graph(), qe, 2.0, 1.0);
        run_step_system(&mut world);

        let graph = world.entity(e).get::<MetabolicGraph>().unwrap();
        let total_thermal: f32 = graph.nodes().iter().map(|n| n.thermal_output).sum();
        let last_node = &graph.nodes()[graph.node_count() - 1];
        let last_useful = equations::exergy_balance(
            graph.edges().last().map(|e| e.flow_rate).unwrap_or(0.0),
            last_node.efficiency,
            last_node.activation_energy,
        );
        let total_ea: f32 = graph.nodes().iter().map(|n| n.activation_energy).sum();
        let balance = total_thermal + last_useful + total_ea;
        assert!(
            balance <= qe + 1.0,
            "balance {} exceeds input {} by more than tolerance",
            balance, qe,
        );
    }

    #[test]
    fn step_entropy_rate_is_sum_of_nodes() {
        let mut world = World::new();
        let e = spawn_metabolic_entity(&mut world, build_chain_graph(), 500.0, 2.0, 1.0);
        run_step_system(&mut world);

        let graph = world.entity(e).get::<MetabolicGraph>().unwrap();
        let sum_s: f32 = graph.nodes().iter().map(|n| n.entropy_rate).sum();
        assert!(
            (graph.total_entropy_rate() - sum_s).abs() < 1e-3,
            "total {} != sum {}",
            graph.total_entropy_rate(), sum_s,
        );
    }

    #[test]
    fn step_no_graph_entities_no_panic() {
        let mut world = World::new();
        world.spawn((BaseEnergy::new(100.0), SpatialVolume::new(1.0), AmbientPressure::default()));
        run_step_system(&mut world);
    }

    #[test]
    fn step_starvation_collapses_flows() {
        let mut world = World::new();
        let e = spawn_metabolic_entity(&mut world, build_chain_graph(), 0.5, 2.0, 1.0);
        run_step_system(&mut world);

        let graph = world.entity(e).get::<MetabolicGraph>().unwrap();
        for edge in graph.edges() {
            assert_eq!(edge.flow_rate, 0.0, "edge {}→{} flow should be 0", edge.from, edge.to);
        }
        for node in graph.nodes() {
            assert_eq!(node.thermal_output, 0.0);
            assert_eq!(node.entropy_rate, 0.0);
        }
    }

    #[test]
    fn step_fork_distributes_proportionally() {
        let mut world = World::new();
        let e = spawn_metabolic_entity(&mut world, build_fork_graph(), 500.0, 2.0, 1.0);
        run_step_system(&mut world);

        let graph = world.entity(e).get::<MetabolicGraph>().unwrap();
        let edges = graph.edges();
        let f0 = edges[0].flow_rate;
        let f1 = edges[1].flow_rate;
        if f0 > METABOLIC_MIN_FLOW && f1 > METABOLIC_MIN_FLOW {
            let ratio = f0 / (f0 + f1);
            assert!(
                (ratio - 0.6).abs() < 0.05,
                "expected ~60% to edge 0, got {:.1}%",
                ratio * 100.0,
            );
        }
    }

    #[test]
    fn step_edges_respect_max_capacity() {
        let mut world = World::new();
        let e = spawn_metabolic_entity(&mut world, build_chain_graph(), 500.0, 2.0, 1.0);
        run_step_system(&mut world);

        let graph = world.entity(e).get::<MetabolicGraph>().unwrap();
        for edge in graph.edges() {
            assert!(
                edge.flow_rate <= edge.max_capacity + 1e-3,
                "edge {}→{}: flow {} > cap {}",
                edge.from, edge.to, edge.flow_rate, edge.max_capacity,
            );
        }
    }

    // ── MG-3C: Constraint system ──

    #[test]
    fn constraint_equal_temps_collapses_efficiency() {
        let mut world = World::new();
        let mut graph = build_chain_graph();
        for node in graph.nodes_mut() {
            node.efficiency = 0.8;
        }
        let qe_for_280 = 280.0 * equations::sphere_volume(1.0);
        let e = spawn_metabolic_entity(&mut world, graph, qe_for_280, 1.0, 1.0);
        run_constraint_system(&mut world);

        let g = world.entity(e).get::<MetabolicGraph>().unwrap();
        let t_core = equations::equivalent_temperature(equations::density(qe_for_280, 1.0));
        let t_env = equations::ambient_equivalent_temperature(1.0);
        let eta_c = equations::carnot_efficiency(t_core, t_env);
        for node in g.nodes() {
            assert!(
                node.efficiency <= eta_c + 1e-4,
                "{:?}: efficiency {} > carnot {}",
                node.role, node.efficiency, eta_c,
            );
        }
    }

    #[test]
    fn constraint_high_t_core_preserves_low_efficiency() {
        let mut world = World::new();
        let mut graph = build_chain_graph();
        for node in graph.nodes_mut() {
            node.efficiency = 0.1;
        }
        let e = spawn_metabolic_entity(&mut world, graph, 5000.0, 0.5, 1.0);
        run_constraint_system(&mut world);

        let g = world.entity(e).get::<MetabolicGraph>().unwrap();
        for node in g.nodes() {
            assert!(
                (node.efficiency - 0.1).abs() < 1e-4,
                "{:?}: efficiency changed to {}",
                node.role, node.efficiency,
            );
        }
    }

    #[test]
    fn constraint_mixed_efficiency_selective_clamp() {
        // Nodo 0: η=0.8 (viola), nodo 1: η=0.3 (no viola), nodo 2: η=0.6 (puede violar).
        let mut world = World::new();
        let mut graph = build_chain_graph();
        graph.nodes_mut()[0].efficiency = 0.8;
        graph.nodes_mut()[1].efficiency = 0.3;
        graph.nodes_mut()[2].efficiency = 0.6;

        // T_core alto → Carnot ~0.4.
        // density(500, 2.0) = 500 / sphere_volume(2.0). T_core = density.
        // T_env(viscosity=1) = 280. eta_c = 1 - 280/T_core.
        // Queremos carnot ~0.4 → T_core = 280/0.6 ≈ 466.7.
        // density = 466.7 → qe = 466.7 * sphere_volume(1.0) ≈ 1955.
        let target_t_core = 466.7;
        let qe = target_t_core * equations::sphere_volume(1.0);
        let e = spawn_metabolic_entity(&mut world, graph, qe, 1.0, 1.0);
        run_constraint_system(&mut world);

        let g = world.entity(e).get::<MetabolicGraph>().unwrap();
        let t_core = equations::equivalent_temperature(equations::density(qe, 1.0));
        let t_env = equations::ambient_equivalent_temperature(1.0);
        let eta_c = equations::carnot_efficiency(t_core, t_env);

        // Nodo 0: η=0.8 > carnot → reclamped.
        assert!(
            g.nodes()[0].efficiency <= eta_c + 1e-4,
            "node 0 should be clamped: {} > {}",
            g.nodes()[0].efficiency, eta_c,
        );
        // Nodo 1: η=0.3 < carnot → sin cambio.
        assert!(
            (g.nodes()[1].efficiency - 0.3).abs() < 1e-4,
            "node 1 should be unchanged: {}",
            g.nodes()[1].efficiency,
        );
    }

    #[test]
    fn constraint_idempotent() {
        let mut world = World::new();
        let e = spawn_metabolic_entity(&mut world, build_chain_graph(), 500.0, 2.0, 1.0);

        run_constraint_system(&mut world);
        let snap1: Vec<f32> = world
            .entity(e)
            .get::<MetabolicGraph>()
            .unwrap()
            .nodes()
            .iter()
            .map(|n| n.efficiency)
            .collect();

        run_constraint_system(&mut world);
        let snap2: Vec<f32> = world
            .entity(e)
            .get::<MetabolicGraph>()
            .unwrap()
            .nodes()
            .iter()
            .map(|n| n.efficiency)
            .collect();

        assert_eq!(snap1, snap2, "constraint must be idempotent");
    }

    #[test]
    fn constraint_after_step_all_efficiencies_within_carnot() {
        let mut world = World::new();
        let e = spawn_metabolic_entity(&mut world, build_chain_graph(), 500.0, 2.0, 1.0);
        run_step_system(&mut world);
        run_constraint_system(&mut world);

        let g = world.entity(e).get::<MetabolicGraph>().unwrap();
        let qe = 500.0;
        let t_core = equations::equivalent_temperature(equations::density(qe, 2.0));
        let t_env = equations::ambient_equivalent_temperature(1.0);
        let eta_c = equations::carnot_efficiency(t_core, t_env);
        for node in g.nodes() {
            assert!(
                node.efficiency <= eta_c + 1e-4,
                "{:?}: efficiency {} > carnot {}",
                node.role, node.efficiency, eta_c,
            );
        }
    }

    // ── MG-4: Shape Optimization ──

    fn spawn_shape_entity(
        world: &mut World,
        graph: MetabolicGraph,
        velocity: Vec2,
        viscosity: f32,
        radius: f32,
    ) -> Entity {
        world.spawn((
            graph,
            BaseEnergy::new(500.0),
            SpatialVolume::new(radius),
            AmbientPressure::new(0.0, viscosity),
            FlowVector::new(velocity, 0.05),
            MorphogenesisShapeParams::default(),
        )).id()
    }

    fn run_shape_system(world: &mut World) {
        let mut schedule = Schedule::default();
        schedule.add_systems(shape_optimization_system);
        schedule.run(world);
    }

    #[test]
    fn shape_water_dense_increases_fineness() {
        let mut world = World::new();
        let e = spawn_shape_entity(
            &mut world, build_chain_graph(),
            Vec2::new(4.0, 0.0), 1000.0, 2.0,
        );
        for _ in 0..10 {
            run_shape_system(&mut world);
        }
        let s = world.entity(e).get::<MorphogenesisShapeParams>().unwrap();
        assert!(
            s.fineness_ratio() > 3.0,
            "dense water + high speed → fusiform; got {}",
            s.fineness_ratio(),
        );
    }

    #[test]
    fn shape_air_light_stays_compact() {
        // Low ρ*v²*A → tiny drag gradient → fineness barely moves.
        let mut world = World::new();
        let e = spawn_shape_entity(
            &mut world, build_chain_graph(),
            Vec2::new(0.5, 0.0), 1.2, 0.5,
        );
        for _ in 0..10 {
            run_shape_system(&mut world);
        }
        let s = world.entity(e).get::<MorphogenesisShapeParams>().unwrap();
        assert!(
            s.fineness_ratio() < 2.5,
            "light air + low speed → compact; got {}",
            s.fineness_ratio(),
        );
    }

    #[test]
    fn shape_cost_decreases_over_ticks() {
        let mut world = World::new();
        let e = spawn_shape_entity(
            &mut world, build_chain_graph(),
            Vec2::new(4.0, 0.0), 1000.0, 2.0,
        );
        run_shape_system(&mut world);
        let cost_1 = world.entity(e).get::<MorphogenesisShapeParams>().unwrap().current_shape_cost();
        for _ in 0..4 {
            run_shape_system(&mut world);
        }
        let cost_5 = world.entity(e).get::<MorphogenesisShapeParams>().unwrap().current_shape_cost();
        assert!(
            cost_5 <= cost_1 + 0.01,
            "optimizer should reduce cost: tick1={}, tick5={}",
            cost_1, cost_5,
        );
    }

    #[test]
    fn shape_no_metabolic_graph_not_affected() {
        let mut world = World::new();
        // Entity sin MetabolicGraph pero con ShapeParams → no matchea la query
        world.spawn((
            BaseEnergy::new(500.0),
            SpatialVolume::new(2.0),
            AmbientPressure::new(0.0, 1000.0),
            FlowVector::new(Vec2::new(4.0, 0.0), 0.05),
            MorphogenesisShapeParams::default(),
        ));
        run_shape_system(&mut world); // no panic, no change
    }

    #[test]
    fn shape_ceiling_clamped_at_max() {
        let mut world = World::new();
        let e = world.spawn((
            build_chain_graph(),
            BaseEnergy::new(500.0),
            SpatialVolume::new(2.0),
            AmbientPressure::new(0.0, 1000.0),
            FlowVector::new(Vec2::new(4.0, 0.0), 0.05),
            MorphogenesisShapeParams::new(mg::FINENESS_MAX),
        )).id();
        for _ in 0..10 {
            run_shape_system(&mut world);
        }
        let s = world.entity(e).get::<MorphogenesisShapeParams>().unwrap();
        assert!(s.fineness_ratio() <= mg::FINENESS_MAX + 1e-4);
    }

    #[test]
    fn shape_floor_pushes_away_from_sphere() {
        let mut world = World::new();
        let e = world.spawn((
            build_chain_graph(),
            BaseEnergy::new(500.0),
            SpatialVolume::new(2.0),
            AmbientPressure::new(0.0, 1000.0),
            FlowVector::new(Vec2::new(4.0, 0.0), 0.05),
            MorphogenesisShapeParams::new(mg::FINENESS_MIN),
        )).id();
        for _ in 0..10 {
            run_shape_system(&mut world);
        }
        let s = world.entity(e).get::<MorphogenesisShapeParams>().unwrap();
        assert!(
            s.fineness_ratio() > mg::FINENESS_MIN,
            "high pressure should push away from sphere: {}",
            s.fineness_ratio(),
        );
    }

    #[test]
    fn shape_zero_velocity_minimal_change() {
        let mut world = World::new();
        let e = spawn_shape_entity(
            &mut world, build_chain_graph(),
            Vec2::ZERO, 1000.0, 2.0,
        );
        let initial = world.entity(e).get::<MorphogenesisShapeParams>().unwrap().fineness_ratio();
        for _ in 0..10 {
            run_shape_system(&mut world);
        }
        let final_f = world.entity(e).get::<MorphogenesisShapeParams>().unwrap().fineness_ratio();
        assert!(
            (final_f - initial).abs() < 0.5,
            "v=0 → minimal change: {} → {}",
            initial, final_f,
        );
    }

    #[test]
    fn shape_deterministic_1000_calls() {
        let mut world = World::new();
        let e = spawn_shape_entity(
            &mut world, build_chain_graph(),
            Vec2::new(4.0, 0.0), 1000.0, 2.0,
        );
        for _ in 0..50 {
            run_shape_system(&mut world);
        }
        let ref_f = world.entity(e).get::<MorphogenesisShapeParams>().unwrap().fineness_ratio();

        // Replay idéntico
        let mut world2 = World::new();
        let e2 = spawn_shape_entity(
            &mut world2, build_chain_graph(),
            Vec2::new(4.0, 0.0), 1000.0, 2.0,
        );
        for _ in 0..50 {
            run_shape_system(&mut world2);
        }
        let ref_f2 = world2.entity(e2).get::<MorphogenesisShapeParams>().unwrap().fineness_ratio();
        assert_eq!(ref_f, ref_f2, "deterministic: same inputs → same output");
    }

    #[test]
    fn shape_oscillating_input_damped() {
        let mut world = World::new();
        let e = spawn_shape_entity(
            &mut world, build_chain_graph(),
            Vec2::new(4.0, 0.0), 1000.0, 2.0,
        );
        let mut prev = world.entity(e).get::<MorphogenesisShapeParams>().unwrap().fineness_ratio();
        // Max per-tick delta = SHAPE_FD_DELTA * max_iter = 0.1 * 3 = 0.3.
        let max_delta_per_tick = mg::SHAPE_FD_DELTA * mg::SHAPE_OPTIMIZER_MAX_ITER as f32 + 0.01;
        for i in 0..20 {
            let visc = if i % 2 == 0 { 1000.0 } else { 1.2 };
            world.entity_mut(e).get_mut::<AmbientPressure>().unwrap().terrain_viscosity = visc;
            run_shape_system(&mut world);
            let cur = world.entity(e).get::<MorphogenesisShapeParams>().unwrap().fineness_ratio();
            assert!(
                (cur - prev).abs() <= max_delta_per_tick,
                "tick {}: damping violated: {} → {} (Δ={:.3}, max={:.3})",
                i, prev, cur, (cur - prev).abs(), max_delta_per_tick,
            );
            prev = cur;
        }
    }

    #[test]
    fn shape_converges_stable_input() {
        let mut world = World::new();
        let e = spawn_shape_entity(
            &mut world, build_chain_graph(),
            Vec2::new(4.0, 0.0), 1000.0, 2.0,
        );
        for _ in 0..50 {
            run_shape_system(&mut world);
        }
        let f50 = world.entity(e).get::<MorphogenesisShapeParams>().unwrap().fineness_ratio();
        for _ in 0..10 {
            run_shape_system(&mut world);
        }
        let f60 = world.entity(e).get::<MorphogenesisShapeParams>().unwrap().fineness_ratio();
        assert!(
            (f60 - f50).abs() < mg::SHAPE_OPTIMIZER_EPSILON * 2.0,
            "should converge: f50={}, f60={}",
            f50, f60,
        );
    }

    #[test]
    fn shape_optimizer_uses_constants_from_config() {
        assert_eq!(mg::SHAPE_OPTIMIZER_MAX_ITER, 3);
        assert!((mg::SHAPE_OPTIMIZER_DAMPING - 0.3).abs() < 1e-6);
        assert!((mg::SHAPE_FD_DELTA - 0.1).abs() < 1e-6);
    }

    #[test]
    fn graph_vascular_cost_sums_transport() {
        let mut graph = build_chain_graph();
        graph.edges_mut()[0].transport_cost = 3.0;
        graph.edges_mut()[1].transport_cost = 7.0;
        assert!((graph_vascular_cost(&graph) - 10.0).abs() < 1e-6);
    }

    #[test]
    fn graph_vascular_cost_empty_is_zero() {
        let graph = MetabolicGraphBuilder::new()
            .add_node(OrganRole::Root, 0.9, 3.0)
            .build()
            .unwrap();
        assert_eq!(graph_vascular_cost(&graph), 0.0);
    }

    // ── MG-6D: EntropyLedger system ──

    fn run_ledger_system(world: &mut World) {
        let mut schedule = Schedule::default();
        schedule.add_systems(entropy_ledger_system);
        schedule.run(world);
    }

    #[test]
    fn ledger_inserted_for_metabolic_entity() {
        let mut world = World::new();
        let e = spawn_metabolic_entity(&mut world, build_chain_graph(), 500.0, 2.0, 1.0);
        run_ledger_system(&mut world);

        let ledger = world.entity(e).get::<EntropyLedger>();
        assert!(ledger.is_some(), "EntropyLedger must be inserted");
        let ledger = ledger.unwrap();
        assert!(ledger.total_heat_generated >= 0.0);
        assert!(ledger.total_waste_generated >= 0.0);
        assert!(ledger.entropy_rate >= 0.0);
        assert!(ledger.exergy_efficiency >= 0.0);
        assert!(ledger.exergy_efficiency <= 1.0 + 1e-4);
    }

    #[test]
    fn ledger_consistent_with_manual_chain() {
        let mut world = World::new();
        let qe = 500.0;
        let radius = 2.0;
        let graph = build_chain_graph();

        // Manual computation
        let chain = equations::evaluate_metabolic_chain(&graph, qe, qe);
        let t_core = equations::equivalent_temperature(equations::density(qe, radius));
        let expected_s = equations::entropy_production(chain.total_heat, t_core);
        let expected_eta = chain.final_exergy / qe;

        let e = spawn_metabolic_entity(&mut world, graph, qe, radius, 1.0);
        run_ledger_system(&mut world);

        let ledger = world.entity(e).get::<EntropyLedger>().unwrap();
        assert!(
            (ledger.total_heat_generated - chain.total_heat).abs() < 1e-2,
            "heat: {} vs {}", ledger.total_heat_generated, chain.total_heat,
        );
        assert!(
            (ledger.total_waste_generated - chain.total_waste).abs() < 1e-2,
            "waste: {} vs {}", ledger.total_waste_generated, chain.total_waste,
        );
        assert!(
            (ledger.entropy_rate - expected_s).abs() < 1e-3,
            "entropy: {} vs {}", ledger.entropy_rate, expected_s,
        );
        assert!(
            (ledger.exergy_efficiency - expected_eta).abs() < 1e-3,
            "eta: {} vs {}", ledger.exergy_efficiency, expected_eta,
        );
    }

    #[test]
    fn ledger_idempotent_when_graph_unchanged() {
        let mut world = World::new();
        let e = spawn_metabolic_entity(&mut world, build_chain_graph(), 500.0, 2.0, 1.0);
        run_ledger_system(&mut world);
        let snap1 = *world.entity(e).get::<EntropyLedger>().unwrap();

        run_ledger_system(&mut world);
        let snap2 = *world.entity(e).get::<EntropyLedger>().unwrap();

        assert_eq!(snap1, snap2, "ledger must be idempotent");
    }

    #[test]
    fn ledger_not_inserted_without_metabolic_graph() {
        let mut world = World::new();
        let e = world.spawn((
            BaseEnergy::new(500.0),
            SpatialVolume::new(2.0),
            AmbientPressure::new(0.0, 1.0),
        )).id();
        run_ledger_system(&mut world);

        assert!(
            world.entity(e).get::<EntropyLedger>().is_none(),
            "entity without MetabolicGraph should not get EntropyLedger",
        );
    }

    #[test]
    fn ledger_zero_exergy_efficiency_when_zero_qe() {
        let mut world = World::new();
        let e = spawn_metabolic_entity(&mut world, build_chain_graph(), 0.0, 2.0, 1.0);
        run_ledger_system(&mut world);

        // Empty graph (0 nodes after starvation check) or zero input
        // Either no ledger (skip) or efficiency = 0
        if let Some(ledger) = world.entity(e).get::<EntropyLedger>() {
            assert_eq!(ledger.exergy_efficiency, 0.0, "zero qe => zero efficiency");
            assert!(!ledger.exergy_efficiency.is_nan(), "no NaN allowed");
        }
    }

    #[test]
    fn ledger_full_pipeline_step_constraint_ledger() {
        let mut world = World::new();
        // High qe + small radius → high T_core, large Carnot margin.
        let e = spawn_metabolic_entity(&mut world, build_chain_graph(), 5000.0, 0.5, 1.0);
        run_step_system(&mut world);
        run_constraint_system(&mut world);
        run_ledger_system(&mut world);

        let ledger = world.entity(e).get::<EntropyLedger>().unwrap();
        assert!(ledger.total_heat_generated > 0.0, "should have heat after full pipeline");
        assert!(ledger.entropy_rate > 0.0, "should have entropy after full pipeline");
        assert!(ledger.exergy_efficiency >= 0.0, "efficiency must be non-negative");
        assert!(ledger.exergy_efficiency <= 1.0, "efficiency must be <= 1");
    }

    // ── MG-5: Albedo Inference ──

    fn spawn_albedo_entity(
        world: &mut World,
        graph: MetabolicGraph,
        qe: f32,
        radius: f32,
        viscosity: f32,
        irradiance: Option<IrradianceReceiver>,
        ledger: Option<EntropyLedger>,
    ) -> Entity {
        let mut e = world.spawn((
            graph,
            BaseEnergy::new(qe),
            SpatialVolume::new(radius),
            AmbientPressure::new(0.0, viscosity),
        ));
        if let Some(ir) = irradiance {
            e.insert(ir);
        }
        if let Some(l) = ledger {
            e.insert(l);
        }
        e.id()
    }

    fn run_albedo_system(world: &mut World) {
        let mut schedule = Schedule::default();
        schedule.add_systems(albedo_inference_system);
        schedule.run(world);
    }

    #[test]
    fn albedo_hot_high_solar_reflects() {
        let mut world = World::new();
        let ledger = EntropyLedger {
            total_heat_generated: 300.0,
            total_waste_generated: 10.0,
            entropy_rate: 0.6,
            exergy_efficiency: 0.5,
        };
        let e = spawn_albedo_entity(
            &mut world, build_chain_graph(), 500.0, 2.0, 1.0,
            Some(IrradianceReceiver::new(100.0, 0.8)),
            Some(ledger),
        );
        run_albedo_system(&mut world);

        let albedo = world.entity(e).get::<InferredAlbedo>();
        assert!(albedo.is_some(), "InferredAlbedo should be inserted");
        let a = albedo.unwrap().albedo();
        assert!(a > 0.7, "hot + high solar → high α; got {a}");
    }

    #[test]
    fn albedo_low_q_high_dissipation_absorbs() {
        // T_core >> T_env → large dissipation capacity, low Q_met → creature absorbs more.
        // T_core = density(qe=1466, r=1) ≈ 350, T_env = ambient(visc=2) = 300.
        let mut world = World::new();
        let ledger = EntropyLedger {
            total_heat_generated: 20.0,
            total_waste_generated: 2.0,
            entropy_rate: 0.05,
            exergy_efficiency: 0.8,
        };
        let e = spawn_albedo_entity(
            &mut world, build_chain_graph(), 1466.0, 1.0, 2.0,
            Some(IrradianceReceiver::new(5.0, 0.5)),
            Some(ledger),
        );
        run_albedo_system(&mut world);

        let a = world.entity(e).get::<InferredAlbedo>().unwrap().albedo();
        assert!(a < 0.3, "high dissipation + low Q_met → low α (absorbs); got {a}");
    }

    #[test]
    fn albedo_no_irradiance_returns_fallback() {
        let mut world = World::new();
        let e = spawn_albedo_entity(
            &mut world, build_chain_graph(), 500.0, 2.0, 1.0,
            None, None,
        );
        run_albedo_system(&mut world);

        let a = world.entity(e).get::<InferredAlbedo>().unwrap().albedo();
        assert!(
            (a - mg::ALBEDO_FALLBACK).abs() < 0.01,
            "no irradiance → fallback {}, got {a}",
            mg::ALBEDO_FALLBACK,
        );
    }

    #[test]
    fn albedo_not_inserted_without_metabolic_graph() {
        let mut world = World::new();
        let e = world.spawn((
            BaseEnergy::new(500.0),
            SpatialVolume::new(2.0),
            AmbientPressure::new(0.0, 1.0),
        )).id();
        run_albedo_system(&mut world); // no panic

        assert!(
            world.entity(e).get::<InferredAlbedo>().is_none(),
            "entity without MetabolicGraph should not get InferredAlbedo",
        );
    }

    #[test]
    fn albedo_without_irradiance_no_panic() {
        let mut world = World::new();
        let e = spawn_albedo_entity(
            &mut world, build_chain_graph(), 500.0, 2.0, 1.0,
            None,
            Some(EntropyLedger {
                total_heat_generated: 100.0,
                total_waste_generated: 5.0,
                entropy_rate: 0.2,
                exergy_efficiency: 0.7,
            }),
        );
        run_albedo_system(&mut world);

        let a = world.entity(e).get::<InferredAlbedo>().unwrap().albedo();
        assert!(a >= mg::ALBEDO_MIN && a <= mg::ALBEDO_MAX);
    }

    #[test]
    fn albedo_always_in_valid_range_extreme_inputs() {
        let mut world = World::new();
        let extremes: [(f32, f32, f32); 4] = [
            (0.0,     0.0,   0.0),
            (10000.0, 1000.0, 0.9),
            (0.1,     1000.0, 0.9),
            (10000.0, 0.0,   0.0),
        ];
        for (q, pd, af) in extremes {
            let ledger = EntropyLedger {
                total_heat_generated: q,
                total_waste_generated: 0.0,
                entropy_rate: 0.0,
                exergy_efficiency: 0.5,
            };
            let ir = if pd > 0.0 { Some(IrradianceReceiver::new(pd, af)) } else { None };
            let e = spawn_albedo_entity(
                &mut world, build_chain_graph(), 500.0, 2.0, 1.0,
                ir, Some(ledger),
            );
            run_albedo_system(&mut world);
            let a = world.entity(e).get::<InferredAlbedo>().unwrap().albedo();
            assert!(
                a >= mg::ALBEDO_MIN && a <= mg::ALBEDO_MAX,
                "Q={q}, pd={pd}, af={af} → α={a} out of range",
            );
        }
    }

    #[test]
    fn albedo_guard_change_detection() {
        let mut world = World::new();
        let e = spawn_albedo_entity(
            &mut world, build_chain_graph(), 500.0, 2.0, 1.0,
            None, None,
        );
        run_albedo_system(&mut world);
        let a1 = world.entity(e).get::<InferredAlbedo>().unwrap().albedo();

        // Run again — same inputs → same albedo, no mutation.
        run_albedo_system(&mut world);
        let a2 = world.entity(e).get::<InferredAlbedo>().unwrap().albedo();
        assert_eq!(a1, a2, "idempotent: same inputs → same albedo");
    }

    #[test]
    fn albedo_uses_ledger_over_proxy_when_available() {
        let mut world = World::new();
        // Con ledger: Q=300.
        let ledger = EntropyLedger {
            total_heat_generated: 300.0,
            total_waste_generated: 10.0,
            entropy_rate: 0.6,
            exergy_efficiency: 0.5,
        };
        let e_with_ledger = spawn_albedo_entity(
            &mut world, build_chain_graph(), 500.0, 2.0, 1.0,
            Some(IrradianceReceiver::new(100.0, 0.8)),
            Some(ledger),
        );
        run_albedo_system(&mut world);
        let a_ledger = world.entity(e_with_ledger).get::<InferredAlbedo>().unwrap().albedo();

        // Sin ledger: proxy = total_entropy_rate * T_core (diferente de 300).
        let e_proxy = spawn_albedo_entity(
            &mut world, build_chain_graph(), 500.0, 2.0, 1.0,
            Some(IrradianceReceiver::new(100.0, 0.8)),
            None,
        );
        run_albedo_system(&mut world);
        let a_proxy = world.entity(e_proxy).get::<InferredAlbedo>().unwrap().albedo();

        // Ambos deben ser válidos; pueden diferir.
        assert!(a_ledger >= mg::ALBEDO_MIN && a_ledger <= mg::ALBEDO_MAX);
        assert!(a_proxy >= mg::ALBEDO_MIN && a_proxy <= mg::ALBEDO_MAX);
    }

    // ── MG-7B: Surface Rugosity System ──

    fn spawn_rugosity_entity(
        world: &mut World,
        qe: f32,
        radius: f32,
        viscosity: f32,
        ledger: EntropyLedger,
    ) -> Entity {
        world.spawn((
            BaseEnergy::new(qe),
            SpatialVolume::new(radius),
            AmbientPressure::new(0.0, viscosity),
            ledger,
        )).id()
    }

    fn run_rugosity_system(world: &mut World) {
        let mut schedule = Schedule::default();
        schedule.add_systems(surface_rugosity_system);
        schedule.run(world);
    }

    #[test]
    fn rugosity_low_q_near_minimum() {
        let mut world = World::new();
        // radius=0.5 → vol≈0.524, qe=500 → density≈954 → T_core≈954.
        // viscosity=1.0 → T_env=280 → ΔT=674. Low Q → A_needed tiny → rug≈1.0.
        let ledger = EntropyLedger {
            total_heat_generated: 10.0,
            total_waste_generated: 1.0,
            entropy_rate: 0.03,
            exergy_efficiency: 0.8,
        };
        let e = spawn_rugosity_entity(&mut world, 500.0, 0.5, 1.0, ledger);
        run_rugosity_system(&mut world);

        let surface = world.entity(e).get::<MorphogenesisSurface>().unwrap();
        assert!(
            (surface.rugosity() - mg::RUGOSITY_MIN).abs() < 0.2,
            "low Q → smooth: rug={}",
            surface.rugosity(),
        );
    }

    #[test]
    fn rugosity_extreme_q_tiny_delta_t_hits_maximum() {
        let mut world = World::new();
        // radius=2.0 → vol≈33.5, qe=500 → density≈14.9 → T_core≈14.9.
        // viscosity=1.0 → T_env=280. T_core < T_env → ΔT < 0 → h*ΔT clamped to ε.
        // Q=3000 → A_needed = 3000/ε → huge → rugosity = RUGOSITY_MAX.
        let ledger = EntropyLedger {
            total_heat_generated: 3000.0,
            total_waste_generated: 10.0,
            entropy_rate: 1.0,
            exergy_efficiency: 0.3,
        };
        let e = spawn_rugosity_entity(&mut world, 500.0, 2.0, 1.0, ledger);
        run_rugosity_system(&mut world);

        let surface = world.entity(e).get::<MorphogenesisSurface>().unwrap();
        assert!(
            (surface.rugosity() - mg::RUGOSITY_MAX).abs() < 1e-3,
            "extreme Q + ΔT<0 → max rugosity: rug={}",
            surface.rugosity(),
        );
    }

    #[test]
    fn rugosity_entity_without_ledger_no_surface() {
        let mut world = World::new();
        // Entity sin EntropyLedger — no debe recibir MorphogenesisSurface.
        let e = world.spawn((
            BaseEnergy::new(500.0),
            SpatialVolume::new(2.0),
            AmbientPressure::new(0.0, 1.0),
        )).id();
        run_rugosity_system(&mut world);

        assert!(
            world.entity(e).get::<MorphogenesisSurface>().is_none(),
            "no ledger → no surface component",
        );
    }

    #[test]
    fn rugosity_guard_change_detection() {
        let mut world = World::new();
        let ledger = EntropyLedger {
            total_heat_generated: 100.0,
            total_waste_generated: 5.0,
            entropy_rate: 0.25,
            exergy_efficiency: 0.6,
        };
        // radius=0.5 → T_core≈954 > T_env=280. Meaningful ΔT.
        let e = spawn_rugosity_entity(&mut world, 500.0, 0.5, 1.0, ledger);
        run_rugosity_system(&mut world);
        let r1 = world.entity(e).get::<MorphogenesisSurface>().unwrap().rugosity();

        // Run again — same inputs → same rugosity.
        run_rugosity_system(&mut world);
        let r2 = world.entity(e).get::<MorphogenesisSurface>().unwrap().rugosity();
        assert_eq!(r1, r2, "idempotent: same inputs → same rugosity");
    }

    #[test]
    fn rugosity_always_in_valid_range() {
        let mut world = World::new();
        for q in [0.0_f32, 10.0, 500.0, 5000.0] {
            let ledger = EntropyLedger {
                total_heat_generated: q,
                total_waste_generated: 1.0,
                entropy_rate: 0.1,
                exergy_efficiency: 0.5,
            };
            // radius=0.5 → T_core > T_env with qe=500.
            let e = spawn_rugosity_entity(&mut world, 500.0, 0.5, 1.0, ledger);
            run_rugosity_system(&mut world);
            let rug = world.entity(e).get::<MorphogenesisSurface>().unwrap().rugosity();
            assert!(
                rug >= mg::RUGOSITY_MIN && rug <= mg::RUGOSITY_MAX,
                "Q={q} → rug={rug} out of range",
            );
        }
    }

    #[test]
    fn rugosity_heat_volume_ratio_positive() {
        let mut world = World::new();
        let ledger = EntropyLedger {
            total_heat_generated: 200.0,
            total_waste_generated: 5.0,
            entropy_rate: 0.5,
            exergy_efficiency: 0.5,
        };
        let e = spawn_rugosity_entity(&mut world, 500.0, 0.5, 1.0, ledger);
        run_rugosity_system(&mut world);
        let qv = world.entity(e).get::<MorphogenesisSurface>().unwrap().heat_volume_ratio();
        assert!(qv >= 0.0, "Q/V ratio must be non-negative: qv={qv}");
    }

    #[test]
    fn rugosity_mid_range_positive_delta_t() {
        let mut world = World::new();
        // qe=500, radius=0.5 → T_core≈954, T_env=280, ΔT=674.
        // Q=500 → mid-range heat → rugosity between min and max.
        let ledger = EntropyLedger {
            total_heat_generated: 500.0,
            total_waste_generated: 5.0,
            entropy_rate: 0.5,
            exergy_efficiency: 0.5,
        };
        let e = spawn_rugosity_entity(&mut world, 500.0, 0.5, 1.0, ledger);
        run_rugosity_system(&mut world);

        let rug = world.entity(e).get::<MorphogenesisSurface>().unwrap().rugosity();
        assert!(
            rug >= mg::RUGOSITY_MIN && rug <= mg::RUGOSITY_MAX,
            "mid-range Q + positive ΔT → valid rugosity: rug={rug}",
        );
    }

    // ── MG-7E: Presupuesto geométrico ──

    #[test]
    fn geometry_budget_max_rugosity_within_segment_limit() {
        // base_detail=32, multiplier at RUGOSITY_MAX → ≤ MAX_SEGMENTS_PER_ENTITY.
        let base_detail: u32 = 32;
        let multiplier = equations::rugosity_to_detail_multiplier(mg::RUGOSITY_MAX);
        let total = (base_detail as f32 * multiplier) as u32;
        assert!(
            total <= mg::MAX_SEGMENTS_PER_ENTITY,
            "32 * {multiplier} = {total} > {}",
            mg::MAX_SEGMENTS_PER_ENTITY,
        );
    }

    #[test]
    fn geometry_budget_high_base_detail_clamped() {
        // base_detail=40, multiplier=2.0 → 80 → clamp to MAX_SEGMENTS_PER_ENTITY=64.
        let base_detail: u32 = 40;
        let multiplier = mg::RUGOSITY_MAX_DETAIL_MULTIPLIER;
        let raw = (base_detail as f32 * multiplier) as u32;
        let clamped = raw.min(mg::MAX_SEGMENTS_PER_ENTITY);
        assert_eq!(clamped, mg::MAX_SEGMENTS_PER_ENTITY, "clamped: {raw} → {clamped}");
    }

    // ── MG-5: Albedo fallback paths ──

    #[test]
    fn albedo_inference_with_ledger_produces_valid_albedo() {
        let mut world = World::new();
        let ledger = EntropyLedger {
            total_heat_generated:  150.0,
            total_waste_generated: 5.0,
            entropy_rate:          0.3,
            exergy_efficiency:     0.6,
        };
        let e = spawn_albedo_entity(
            &mut world, build_chain_graph(), 500.0, 2.0, 1.0,
            None, Some(ledger),
        );
        run_albedo_system(&mut world);

        let albedo = world.entity(e).get::<InferredAlbedo>();
        assert!(albedo.is_some(), "InferredAlbedo must be inserted");
        let a = albedo.unwrap().albedo();
        assert!(
            a >= mg::ALBEDO_MIN && a <= mg::ALBEDO_MAX,
            "albedo {a} outside [{}, {}]",
            mg::ALBEDO_MIN, mg::ALBEDO_MAX,
        );
    }

    #[test]
    fn albedo_inference_without_ledger_uses_fallback() {
        let mut world = World::new();
        let e = spawn_albedo_entity(
            &mut world, build_chain_graph(), 500.0, 2.0, 1.0,
            None, None,
        );
        run_albedo_system(&mut world);

        let albedo = world.entity(e).get::<InferredAlbedo>();
        assert!(
            albedo.is_some(),
            "InferredAlbedo must be inserted even without EntropyLedger (fallback path)",
        );
        let a = albedo.unwrap().albedo();
        assert!(
            a >= mg::ALBEDO_MIN && a <= mg::ALBEDO_MAX,
            "fallback albedo {a} outside valid range",
        );
    }

    #[test]
    fn albedo_inference_without_irradiance_defaults_to_zero_solar() {
        let mut world = World::new();
        let ledger = EntropyLedger {
            total_heat_generated:  200.0,
            total_waste_generated: 5.0,
            entropy_rate:          0.4,
            exergy_efficiency:     0.6,
        };
        // No IrradianceReceiver → i_solar = 0 path.
        let e = spawn_albedo_entity(
            &mut world, build_chain_graph(), 500.0, 2.0, 1.0,
            None, Some(ledger),
        );
        run_albedo_system(&mut world);

        let albedo = world.entity(e).get::<InferredAlbedo>();
        assert!(
            albedo.is_some(),
            "InferredAlbedo must be inserted when IrradianceReceiver absent (i_solar=0 path)",
        );
        let a = albedo.unwrap().albedo();
        assert!(
            a >= mg::ALBEDO_MIN && a <= mg::ALBEDO_MAX,
            "zero-solar albedo {a} outside valid range",
        );
    }

    #[test]
    fn albedo_inference_skips_empty_graph() {
        let mut world = World::new();
        let empty_graph = MetabolicGraph::empty();
        let e = spawn_albedo_entity(
            &mut world, empty_graph, 500.0, 2.0, 1.0,
            Some(IrradianceReceiver::new(50.0, 0.5)),
            Some(EntropyLedger {
                total_heat_generated:  100.0,
                total_waste_generated: 5.0,
                entropy_rate:          0.2,
                exergy_efficiency:     0.7,
            }),
        );
        run_albedo_system(&mut world);

        assert!(
            world.entity(e).get::<InferredAlbedo>().is_none(),
            "empty MetabolicGraph (0 nodes) must NOT receive InferredAlbedo",
        );
    }
}
