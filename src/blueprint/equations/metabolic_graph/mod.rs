mod temporal_step;
mod writer_monad;

pub use temporal_step::{
    ambient_equivalent_temperature, propagate_edge_flows, redistribute_node_violation,
};
pub use writer_monad::{
    ChainOutput, OrganOutput, distribute_to_children, evaluate_metabolic_chain, exergy_efficiency,
    organ_transform,
};

use crate::blueprint::constants::*;
use crate::blueprint::morphogenesis::carnot_efficiency;
use crate::blueprint::{MAX_ORGANS_PER_ENTITY, OrganRole};
use crate::layers::OrganManifest;
use crate::layers::metabolic_graph::{MetabolicGraph, MetabolicGraphBuilder};

// Compile-time: las tablas de constantes deben alinearse con OrganRole.
const _: () = assert!(ROLE_EFFICIENCY_FACTOR.len() == OrganRole::COUNT);
const _: () = assert!(ROLE_ACTIVATION_ENERGY.len() == OrganRole::COUNT);

/// Tier topológico default: captadores → núcleo → tallo → actuadores → terminales.
#[inline]
pub(crate) fn metabolic_topology_tier(role: OrganRole) -> u8 {
    match role {
        OrganRole::Root | OrganRole::Leaf | OrganRole::Sensory => 0,
        OrganRole::Core => 1,
        OrganRole::Stem => 2,
        OrganRole::Fin | OrganRole::Limb => 3,
        OrganRole::Petal
        | OrganRole::Thorn
        | OrganRole::Shell
        | OrganRole::Fruit
        | OrganRole::Bud => 4,
    }
}

/// Infiere un `MetabolicGraph` desde `OrganManifest` (un nodo por `OrganSpec`, aristas en cadena
/// según tier metabólico). Manifest vacío → grafo vacío.
pub fn metabolic_graph_from_manifest(
    manifest: &OrganManifest,
    t_core: f32,
    t_env: f32,
) -> MetabolicGraph {
    if manifest.is_empty() {
        return MetabolicGraph::empty();
    }
    let slice = manifest.as_slice();
    let eta_c = carnot_efficiency(t_core, t_env);

    let mut builder = MetabolicGraphBuilder::new();
    for spec in slice.iter() {
        let role = spec.role();
        let i = role as usize;
        let eff = (eta_c * ROLE_EFFICIENCY_FACTOR[i]).clamp(0.0, 1.0);
        let ea = ROLE_ACTIVATION_ENERGY[i];
        builder = builder.add_node(role, eff, ea);
    }

    let n = slice.len();
    let mut idx_perm = [0u8; MAX_ORGANS_PER_ENTITY];
    for i in 0..n {
        idx_perm[i] = i as u8;
    }
    idx_perm[..n].sort_by_key(|&j| (metabolic_topology_tier(slice[j as usize].role()), j));

    for k in 0..n.saturating_sub(1) {
        let a = idx_perm[k];
        let b = idx_perm[k + 1];
        let cap = METABOLIC_EDGE_CAPACITY_BASE
            * slice[a as usize]
                .scale_factor()
                .max(slice[b as usize].scale_factor())
                .max(1e-4);
        builder = builder.add_edge(a, b, cap);
    }

    builder.build().unwrap_or_else(|_| MetabolicGraph::empty())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::blueprint::{LifecycleStage, OrganRole};
    use crate::layers::{OrganManifest, OrganSpec};

    #[test]
    fn empty_manifest_returns_empty_graph() {
        let m = OrganManifest::new(LifecycleStage::Growing);
        let g = metabolic_graph_from_manifest(&m, 300.0, 280.0);
        assert_eq!(g.node_count(), 0);
        assert_eq!(g.edge_count(), 0);
    }

    #[test]
    fn rosa_manifest_produces_four_nodes() {
        let mut m = OrganManifest::new(LifecycleStage::Mature);
        m.push(OrganSpec::new(OrganRole::Stem, 1, 1.0));
        m.push(OrganSpec::new(OrganRole::Leaf, 1, 0.8));
        m.push(OrganSpec::new(OrganRole::Thorn, 1, 0.3));
        m.push(OrganSpec::new(OrganRole::Petal, 1, 0.5));
        let g = metabolic_graph_from_manifest(&m, 400.0, 280.0);
        assert_eq!(g.node_count(), 4);
        assert_eq!(g.edge_count(), 3);
    }

    #[test]
    fn efficiency_bounded_by_carnot() {
        let mut m = OrganManifest::new(LifecycleStage::Mature);
        m.push(OrganSpec::new(OrganRole::Root, 1, 1.0));
        m.push(OrganSpec::new(OrganRole::Core, 1, 1.0));
        m.push(OrganSpec::new(OrganRole::Leaf, 1, 1.0));
        let g = metabolic_graph_from_manifest(&m, 400.0, 280.0);
        let eta_c = carnot_efficiency(400.0, 280.0);
        for node in g.nodes() {
            assert!(
                node.efficiency <= eta_c,
                "{:?} efficiency {} > carnot {}",
                node.role,
                node.efficiency,
                eta_c,
            );
        }
    }

    #[test]
    fn edges_flow_from_lower_to_higher_tier() {
        let mut m = OrganManifest::new(LifecycleStage::Mature);
        m.push(OrganSpec::new(OrganRole::Fin, 1, 1.0));
        m.push(OrganSpec::new(OrganRole::Root, 1, 1.0));
        m.push(OrganSpec::new(OrganRole::Core, 1, 1.0));
        m.push(OrganSpec::new(OrganRole::Stem, 1, 1.0));
        let g = metabolic_graph_from_manifest(&m, 400.0, 280.0);
        for edge in g.edges() {
            let tier_from = metabolic_topology_tier(g.nodes()[edge.from as usize].role);
            let tier_to = metabolic_topology_tier(g.nodes()[edge.to as usize].role);
            assert!(
                tier_from <= tier_to,
                "edge {}→{}: tier {} > tier {}",
                edge.from,
                edge.to,
                tier_from,
                tier_to,
            );
        }
    }

    #[test]
    fn scale_factor_influences_max_capacity() {
        let mut m_small = OrganManifest::new(LifecycleStage::Mature);
        m_small.push(OrganSpec::new(OrganRole::Root, 1, 0.5));
        m_small.push(OrganSpec::new(OrganRole::Stem, 1, 0.5));
        let g_small = metabolic_graph_from_manifest(&m_small, 400.0, 280.0);

        let mut m_large = OrganManifest::new(LifecycleStage::Mature);
        m_large.push(OrganSpec::new(OrganRole::Root, 1, 2.0));
        m_large.push(OrganSpec::new(OrganRole::Stem, 1, 2.0));
        let g_large = metabolic_graph_from_manifest(&m_large, 400.0, 280.0);

        assert!(g_large.edges()[0].max_capacity > g_small.edges()[0].max_capacity);
    }

    #[test]
    fn equal_temps_produce_zero_efficiency() {
        let mut m = OrganManifest::new(LifecycleStage::Mature);
        m.push(OrganSpec::new(OrganRole::Leaf, 1, 1.0));
        let g = metabolic_graph_from_manifest(&m, 300.0, 300.0);
        assert_eq!(g.nodes()[0].efficiency, 0.0);
    }

    #[test]
    fn nan_temps_produce_zero_efficiency() {
        let mut m = OrganManifest::new(LifecycleStage::Mature);
        m.push(OrganSpec::new(OrganRole::Root, 1, 1.0));
        let g = metabolic_graph_from_manifest(&m, f32::NAN, 280.0);
        assert_eq!(g.nodes()[0].efficiency, 0.0);
    }

    #[test]
    fn negative_temps_produce_zero_efficiency() {
        let mut m = OrganManifest::new(LifecycleStage::Mature);
        m.push(OrganSpec::new(OrganRole::Root, 1, 1.0));
        let g = metabolic_graph_from_manifest(&m, 400.0, -10.0);
        assert_eq!(g.nodes()[0].efficiency, 0.0);
    }
}
