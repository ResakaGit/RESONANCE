//! Inhibición de pathway — fármacos que modulan eficiencia metabólica sin matar.
//! Pathway inhibition — drugs that modulate metabolic efficiency without killing.
//!
//! Pure stateless equations. No ECS, no side effects, no Bevy dependency.
//!
//! A drug binds to a protein's active site via frequency alignment (Axiom 8),
//! reducing the corresponding metabolic node's efficiency. Three inhibition modes
//! model distinct pharmacological mechanisms (Competitive, Noncompetitive, Uncompetitive).
//!
//! All constants derived from 4 fundamentals. No hardcoded values.
//! Integration: output feeds directly into `competitive_flow_distribution` and
//! `catalytic_activation_reduction` in metabolic_genome.rs.

use crate::blueprint::constants::pathway_inhibitor as pi;
use crate::blueprint::equations::determinism::gaussian_frequency_alignment;
use crate::layers::metabolic_graph::{MetabolicGraph, METABOLIC_GRAPH_MAX_NODES};
use crate::layers::OrganRole;

// ─── Data Structures ────────────────────────────────────────────────────────

/// Modo de inhibición. Distintos mecanismos farmacológicos.
/// Inhibition mode. Distinct pharmacological mechanisms.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum InhibitionMode {
    /// Compite con sustrato por el sitio activo. Sube E_a aparente. Axioma 3.
    /// Competes with substrate for active site. Raises apparent E_a. Axiom 3.
    Competitive,
    /// Se une a sitio alostérico. Baja η máxima. Axioma 4.
    /// Binds allosteric site. Lowers max η. Axiom 4.
    Noncompetitive,
    /// Se une solo al complejo enzima-sustrato. Baja η y E_a. Axioma 4.
    /// Binds only enzyme-substrate complex. Lowers both η and E_a. Axiom 4.
    Uncompetitive,
}

/// Descriptor de un inhibidor (fármaco/sustancia). Datos puros.
/// Inhibitor descriptor (drug/substance). Pure data.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Inhibitor {
    /// Frecuencia objetivo del fármaco. Axioma 8.
    /// Drug target frequency. Axiom 8.
    pub target_frequency: f32,
    /// Concentración normalizada [0, 1]. Dosis-dependiente.
    /// Normalized concentration [0, 1]. Dose-dependent.
    pub concentration: f32,
    /// Constante de inhibición Ki. Menor = más potente.
    /// Inhibition constant Ki. Lower = more potent.
    pub ki: f32,
    /// Mecanismo de acción.
    /// Mechanism of action.
    pub mode: InhibitionMode,
}

/// Resultado de inhibición sobre un nodo metabólico.
/// Inhibition result for a single metabolic node.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct InhibitionEffect {
    /// Eficiencia efectiva tras inhibición.
    /// Effective efficiency after inhibition.
    pub effective_efficiency: f32,
    /// Energía de activación efectiva tras inhibición.
    /// Effective activation energy after inhibition.
    pub effective_activation_energy: f32,
    /// Ocupación del binding [0, 1].
    /// Binding occupancy [0, 1].
    pub occupancy: f32,
    /// True si este nodo no era el target primario.
    /// True if this node was not the primary target.
    pub is_off_target: bool,
}

/// Resultado de inhibición sobre un grafo metabólico completo.
/// Inhibition result across an entire metabolic graph.
#[derive(Clone, Debug)]
pub struct PathwayInhibitionResult {
    /// Efectos por nodo. Índice = índice del nodo en el grafo.
    /// Per-node effects. Index = node index in graph.
    pub effects: [InhibitionEffect; METABOLIC_GRAPH_MAX_NODES],
    /// Nodos afectados (ocupación > umbral off-target).
    /// Affected nodes (occupancy > off-target threshold).
    pub affected_count: u8,
    /// Pérdida total de eficiencia. Axioma 4: trabajo del fármaco.
    /// Total efficiency loss. Axiom 4: drug's work.
    pub total_efficiency_loss: f32,
    /// Carga off-target (suma de ocupación × pérdida en nodos no-target).
    /// Off-target burden (sum of occupancy × loss on non-target nodes).
    pub off_target_burden: f32,
    /// Costo energético de mantener la inhibición. Axioma 4.
    /// Energy cost of maintaining inhibition. Axiom 4.
    pub maintenance_cost: f32,
}

// ─── PI-1: Binding Affinity ─────────────────────────────────────────────────

/// Afinidad de binding fármaco-proteína. Axioma 8.
/// Drug-protein binding affinity. Axiom 8.
///
/// `affinity = gaussian_frequency_alignment(drug_freq, protein_freq, INHIBITION_BANDWIDTH)`
#[inline]
pub fn binding_affinity(drug_freq: f32, protein_freq: f32) -> f32 {
    gaussian_frequency_alignment(drug_freq, protein_freq, pi::INHIBITION_BANDWIDTH)
}

// ─── PI-2: Hill Response (canonical) ────────────────────────────────────────

/// Respuesta dosis-efecto Hill (implementación canónica). Farmacología estándar.
/// Hill dose-response (canonical implementation). Standard pharmacology.
///
/// `response = Emax × C^n / (EC50^n + C^n)`
///
/// Centralized: all pharmacological dose-response in the codebase should
/// call this function. Previously private in cancer_therapy.rs.
#[inline]
pub fn hill_response(effective_concentration: f32, ec50: f32, hill_n: f32) -> f32 {
    if effective_concentration <= 0.0 || ec50 <= 0.0 { return 0.0; }
    let c_n = effective_concentration.powf(hill_n);
    let ec_n = ec50.powf(hill_n);
    c_n / (ec_n + c_n)
}

// ─── PI-3: Inhibitor Occupancy ──────────────────────────────────────────────

/// Fracción de sitio activo ocupada por el inhibidor.
/// Fraction of active site occupied by inhibitor.
///
/// Affinity gates concentration: mismatched frequency → low effective dose.
/// `occupancy = hill(concentration × affinity, Ki, n=2)`
///
/// Hill n=2 (sigmoidal): standard for enzyme inhibitors (cooperative binding).
pub fn inhibitor_occupancy(inhibitor: &Inhibitor, protein_freq: f32) -> f32 {
    let affinity = binding_affinity(inhibitor.target_frequency, protein_freq);
    let effective_conc = inhibitor.concentration * affinity;
    hill_response(effective_conc, inhibitor.ki, 2.0)
}

// ─── PI-4: Single Node Inhibition ───────────────────────────────────────────

/// Aplica inhibición a un nodo metabólico según el modo.
/// Apply inhibition to a single metabolic node by mode.
///
/// - Competitive: raises apparent E_a, reduces effective η
/// - Noncompetitive: reduces η proportionally, E_a unchanged
/// - Uncompetitive: reduces both η and E_a by same factor
///
/// All results clamped: η ≥ MIN_RESIDUAL_EFFICIENCY (Axiom 4 floor).
pub fn inhibit_node(
    node_efficiency: f32,
    node_activation_energy: f32,
    occupancy: f32,
    mode: InhibitionMode,
    is_primary_target: bool,
) -> InhibitionEffect {
    let occ = occupancy.clamp(0.0, 1.0);

    let (eff, ea) = match mode {
        InhibitionMode::Competitive => {
            let ea_mult = 1.0 + occ * pi::COMPETITIVE_EA_MULTIPLIER;
            let eff_factor = 1.0 / ea_mult;
            (node_efficiency * eff_factor, node_activation_energy * ea_mult)
        }
        InhibitionMode::Noncompetitive => {
            let reduction = occ * pi::MAX_INHIBITION_FRACTION;
            (node_efficiency * (1.0 - reduction), node_activation_energy)
        }
        InhibitionMode::Uncompetitive => {
            let alpha = 1.0 - occ * pi::MAX_INHIBITION_FRACTION;
            (node_efficiency * alpha, node_activation_energy * alpha)
        }
    };

    InhibitionEffect {
        effective_efficiency: eff.max(pi::MIN_RESIDUAL_EFFICIENCY),
        effective_activation_energy: ea.max(0.0),
        occupancy: occ,
        is_off_target: !is_primary_target,
    }
}

// ─── PI-5: Pathway Inhibition (full graph) ──────────────────────────────────

/// Inhibición sobre grafo metabólico completo. Off-target por proximidad de frecuencia.
/// Inhibition across entire metabolic graph. Off-target via frequency proximity.
///
/// For each node: compute affinity, occupancy, determine if primary target, apply mode.
/// Returns per-node effects + summary statistics.
pub fn inhibit_pathway(
    graph: &MetabolicGraph,
    node_frequencies: &[f32],
    target_role: OrganRole,
    inhibitor: &Inhibitor,
) -> PathwayInhibitionResult {
    let null_effect = InhibitionEffect {
        effective_efficiency: 0.0,
        effective_activation_energy: 0.0,
        occupancy: 0.0,
        is_off_target: true,
    };
    let mut result = PathwayInhibitionResult {
        effects: [null_effect; METABOLIC_GRAPH_MAX_NODES],
        affected_count: 0,
        total_efficiency_loss: 0.0,
        off_target_burden: 0.0,
        maintenance_cost: 0.0,
    };

    let node_count = graph.node_count() as usize;
    let nodes = graph.nodes();

    for i in 0..node_count {
        let node = &nodes[i];
        let freq = node_frequencies.get(i).copied().unwrap_or(0.0);
        let is_primary = node.role == target_role;

        let occ = inhibitor_occupancy(inhibitor, freq);
        let effect = inhibit_node(node.efficiency, node.activation_energy, occ, inhibitor.mode, is_primary);

        let eff_loss = (node.efficiency - effect.effective_efficiency).max(0.0);
        result.total_efficiency_loss += eff_loss;

        if occ > pi::OFF_TARGET_THRESHOLD {
            result.affected_count += 1;
            if !is_primary {
                result.off_target_burden += occ * eff_loss;
            }
        }

        result.maintenance_cost += occ * pi::INHIBITION_DISSIPATION_COST;
        result.effects[i] = effect;
    }

    result
}

// ─── PI-6: Multi-Inhibitor Combination ──────────────────────────────────────

/// Combinación de ocupaciones (Bliss independence). Axioma 3: competencia entre fármacos.
/// Combined occupancy (Bliss independence). Axiom 3: competition between drugs.
///
/// `occupancy_combined = 1 - Π(1 - occ_i)`
/// Standard pharmacological model for non-interacting drugs.
#[inline]
pub fn combined_occupancy(occupancies: &[f32]) -> f32 {
    let product: f32 = occupancies.iter()
        .map(|&o| 1.0 - o.clamp(0.0, 1.0))
        .product();
    1.0 - product
}

// ─── PI-7: Selectivity Index ────────────────────────────────────────────────

/// Índice de selectividad: on-target / off-target ratio.
/// Selectivity index: ratio of intended effect to collateral damage.
///
/// Higher = more selective drug. SI >> 1 means targeted therapy.
/// SI ≈ 1 means non-selective (hits everything equally).
pub fn selectivity_index(result: &PathwayInhibitionResult, node_count: usize) -> f32 {
    if node_count == 0 { return 0.0; }

    let (mut on_sum, mut on_count, mut off_sum, mut off_count) = (0.0f32, 0u32, 0.0f32, 0u32);
    for i in 0..node_count {
        let e = &result.effects[i];
        if e.is_off_target {
            off_sum += e.occupancy;
            off_count += 1;
        } else {
            on_sum += e.occupancy;
            on_count += 1;
        }
    }

    let on_mean = if on_count > 0 { on_sum / on_count as f32 } else { 0.0 };
    let off_mean = if off_count > 0 { off_sum / off_count as f32 } else { 0.0 };
    on_mean / (off_mean + f32::EPSILON)
}

// ─── PI-8: Effective Node Parameters ────────────────────────────────────────

/// Arrays de η y E_a efectivos para cómputo de flujo post-inhibición.
/// Effective η and E_a arrays for post-inhibition flow computation.
///
/// Output plugs directly into competitive_flow_distribution and
/// catalytic_activation_reduction. Does NOT mutate the graph (Rule 15).
pub fn effective_node_params(
    graph: &MetabolicGraph,
    inhibition: &PathwayInhibitionResult,
) -> ([f32; METABOLIC_GRAPH_MAX_NODES], [f32; METABOLIC_GRAPH_MAX_NODES]) {
    let mut efficiencies = [0.0f32; METABOLIC_GRAPH_MAX_NODES];
    let mut activations = [0.0f32; METABOLIC_GRAPH_MAX_NODES];
    let nodes = graph.nodes();
    let n = graph.node_count() as usize;

    for i in 0..n {
        efficiencies[i] = inhibition.effects[i].effective_efficiency;
        activations[i] = inhibition.effects[i].effective_activation_energy;
    }
    for i in n..METABOLIC_GRAPH_MAX_NODES {
        efficiencies[i] = nodes.get(i).map_or(0.0, |n| n.efficiency);
        activations[i] = nodes.get(i).map_or(0.0, |n| n.activation_energy);
    }

    (efficiencies, activations)
}

// ─── Tests (BDD) ────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn drug(freq: f32, conc: f32, ki: f32, mode: InhibitionMode) -> Inhibitor {
        Inhibitor { target_frequency: freq, concentration: conc, ki, mode }
    }

    // ── PI-1: Binding Affinity ──────────────────────────────────────────────

    #[test]
    fn affinity_same_frequency_is_one() {
        // GIVEN drug_freq == protein_freq
        // THEN affinity == 1.0
        assert!((binding_affinity(400.0, 400.0) - 1.0).abs() < 1e-5);
    }

    #[test]
    fn affinity_distant_frequency_near_zero() {
        // GIVEN frequencies separated by 2× bandwidth
        // THEN affinity < 0.14 (Gaussian exp(-2))
        let a = binding_affinity(400.0, 500.0);
        assert!(a < 0.14, "affinity={a}");
    }

    #[test]
    fn affinity_nan_safe() {
        // GIVEN NaN frequency
        // THEN affinity == 0.0
        assert_eq!(binding_affinity(f32::NAN, 400.0), 0.0);
    }

    #[test]
    fn affinity_symmetric() {
        let ab = binding_affinity(400.0, 450.0);
        let ba = binding_affinity(450.0, 400.0);
        assert!((ab - ba).abs() < 1e-6);
    }

    // ── PI-2: Hill Response ─────────────────────────────────────────────────

    #[test]
    fn hill_zero_concentration_is_zero() {
        assert_eq!(hill_response(0.0, 1.0, 2.0), 0.0);
    }

    #[test]
    fn hill_at_ec50_is_half() {
        // GIVEN concentration == EC50
        // THEN response ≈ 0.5
        let r = hill_response(1.0, 1.0, 1.0);
        assert!((r - 0.5).abs() < 1e-5, "r={r}");
    }

    #[test]
    fn hill_saturation_approaches_one() {
        // GIVEN concentration >> EC50
        // THEN response → 1.0
        let r = hill_response(1000.0, 1.0, 2.0);
        assert!(r > 0.99, "r={r}");
    }

    #[test]
    fn hill_sigmoidal_with_n2() {
        // GIVEN n=2, C=EC50 → response = 0.5
        let r = hill_response(1.0, 1.0, 2.0);
        assert!((r - 0.5).abs() < 1e-5);
    }

    // ── PI-3: Inhibitor Occupancy ───────────────────────────────────────────

    #[test]
    fn occupancy_zero_concentration_is_zero() {
        let d = drug(400.0, 0.0, 1.0, InhibitionMode::Competitive);
        assert_eq!(inhibitor_occupancy(&d, 400.0), 0.0);
    }

    #[test]
    fn occupancy_high_concentration_perfect_affinity_near_one() {
        let d = drug(400.0, 100.0, 1.0, InhibitionMode::Competitive);
        let o = inhibitor_occupancy(&d, 400.0);
        assert!(o > 0.99, "o={o}");
    }

    #[test]
    fn occupancy_poor_affinity_reduces_effect() {
        // GIVEN high concentration but distant frequency
        // THEN occupancy << 1.0
        let d = drug(400.0, 10.0, 1.0, InhibitionMode::Competitive);
        let good = inhibitor_occupancy(&d, 400.0);
        let poor = inhibitor_occupancy(&d, 600.0);
        assert!(good > poor, "good={good}, poor={poor}");
        assert!(poor < 0.5, "poor affinity should yield low occupancy: {poor}");
    }

    // ── PI-4: Single Node Inhibition ────────────────────────────────────────

    #[test]
    fn competitive_zero_occupancy_no_change() {
        // GIVEN occupancy == 0
        // THEN η and E_a unchanged
        let e = inhibit_node(0.8, 1.0, 0.0, InhibitionMode::Competitive, true);
        assert!((e.effective_efficiency - 0.8).abs() < 1e-5);
        assert!((e.effective_activation_energy - 1.0).abs() < 1e-5);
    }

    #[test]
    fn competitive_full_occupancy_raises_ea() {
        // GIVEN Competitive + occupancy == 1.0
        // THEN E_a increases, η decreases
        let e = inhibit_node(0.8, 1.0, 1.0, InhibitionMode::Competitive, true);
        assert!(e.effective_activation_energy > 1.0, "E_a should rise: {}", e.effective_activation_energy);
        assert!(e.effective_efficiency < 0.8, "η should drop: {}", e.effective_efficiency);
    }

    #[test]
    fn noncompetitive_full_occupancy_drops_efficiency() {
        // GIVEN Noncompetitive + occupancy == 1.0
        // THEN η → MIN_RESIDUAL, E_a unchanged
        let e = inhibit_node(0.8, 1.0, 1.0, InhibitionMode::Noncompetitive, true);
        assert!((e.effective_efficiency - pi::MIN_RESIDUAL_EFFICIENCY).abs() < 0.01,
            "η should be near floor: {}", e.effective_efficiency);
        assert!((e.effective_activation_energy - 1.0).abs() < 1e-5,
            "E_a should be unchanged: {}", e.effective_activation_energy);
    }

    #[test]
    fn uncompetitive_reduces_both() {
        // GIVEN Uncompetitive + occupancy == 1.0
        // THEN both η AND E_a reduced proportionally
        let e = inhibit_node(0.8, 1.0, 1.0, InhibitionMode::Uncompetitive, true);
        assert!(e.effective_efficiency < 0.8);
        assert!(e.effective_activation_energy < 1.0);
    }

    #[test]
    fn inhibition_respects_axiom4_floor() {
        // GIVEN any mode, any occupancy
        // THEN η ≥ MIN_RESIDUAL_EFFICIENCY (Axiom 4: no process reaches zero)
        for mode in [InhibitionMode::Competitive, InhibitionMode::Noncompetitive, InhibitionMode::Uncompetitive] {
            let e = inhibit_node(0.01, 1.0, 1.0, mode, true);
            assert!(e.effective_efficiency >= pi::MIN_RESIDUAL_EFFICIENCY,
                "mode={mode:?}: η={} below floor", e.effective_efficiency);
        }
    }

    #[test]
    fn inhibition_never_creates_efficiency() {
        // GIVEN any mode, any occupancy
        // THEN η_eff ≤ η_original (Axiom 5: conservation)
        for mode in [InhibitionMode::Competitive, InhibitionMode::Noncompetitive, InhibitionMode::Uncompetitive] {
            for occ in [0.0, 0.3, 0.7, 1.0] {
                let e = inhibit_node(0.5, 1.0, occ, mode, true);
                assert!(e.effective_efficiency <= 0.5 + 1e-5,
                    "mode={mode:?}, occ={occ}: η_eff={} > original", e.effective_efficiency);
            }
        }
    }

    // ── PI-6: Combined Occupancy ────────────────────────────────────────────

    #[test]
    fn bliss_two_drugs_half_each() {
        // GIVEN two inhibitors, each with occupancy 0.5
        // THEN combined = 0.75 (Bliss: 1 - 0.5×0.5)
        let c = combined_occupancy(&[0.5, 0.5]);
        assert!((c - 0.75).abs() < 1e-5, "c={c}");
    }

    #[test]
    fn bliss_single_drug_passthrough() {
        let c = combined_occupancy(&[0.7]);
        assert!((c - 0.7).abs() < 1e-5);
    }

    #[test]
    fn bliss_all_zero_is_zero() {
        let c = combined_occupancy(&[0.0, 0.0, 0.0]);
        assert!((c).abs() < 1e-5);
    }

    #[test]
    fn bliss_empty_is_zero() {
        let c = combined_occupancy(&[]);
        assert!((c).abs() < 1e-5);
    }

    // ── PI-7: Selectivity Index ─────────────────────────────────────────────

    #[test]
    fn selectivity_all_on_target_is_high() {
        let result = PathwayInhibitionResult {
            effects: [InhibitionEffect {
                effective_efficiency: 0.1, effective_activation_energy: 1.0,
                occupancy: 0.9, is_off_target: false,
            }; METABOLIC_GRAPH_MAX_NODES],
            affected_count: 1, total_efficiency_loss: 0.7,
            off_target_burden: 0.0, maintenance_cost: 0.01,
        };
        let si = selectivity_index(&result, 1);
        assert!(si > 100.0, "SI={si}");
    }

    #[test]
    fn selectivity_equal_hits_is_one() {
        let on = InhibitionEffect {
            effective_efficiency: 0.1, effective_activation_energy: 1.0,
            occupancy: 0.5, is_off_target: false,
        };
        let off = InhibitionEffect {
            effective_efficiency: 0.1, effective_activation_energy: 1.0,
            occupancy: 0.5, is_off_target: true,
        };
        let mut result = PathwayInhibitionResult {
            effects: [off; METABOLIC_GRAPH_MAX_NODES],
            affected_count: 2, total_efficiency_loss: 0.0,
            off_target_burden: 0.0, maintenance_cost: 0.0,
        };
        result.effects[0] = on;
        let si = selectivity_index(&result, 2);
        assert!((si - 1.0).abs() < 0.01, "SI={si}");
    }

    // ── Conservation / Axiom Tests ──────────────────────────────────────────

    #[test]
    fn competitive_product_never_decreases() {
        // Thermodynamic consistency: η × E_a product should not decrease
        // (drug makes pathway harder, not easier)
        let original_product = 0.5 * 1.0; // η × E_a
        for occ in [0.1, 0.3, 0.5, 0.7, 1.0] {
            let e = inhibit_node(0.5, 1.0, occ, InhibitionMode::Competitive, true);
            let inhibited_product = e.effective_efficiency * e.effective_activation_energy;
            assert!(inhibited_product <= original_product + 1e-4,
                "occ={occ}: product {inhibited_product} > original {original_product}");
        }
    }

    #[test]
    fn maintenance_cost_nonnegative() {
        // Axiom 4: binding is never free
        let d = drug(400.0, 0.5, 1.0, InhibitionMode::Noncompetitive);
        let occ = inhibitor_occupancy(&d, 400.0);
        assert!(occ * pi::INHIBITION_DISSIPATION_COST >= 0.0);
    }
}
