//! MG-3A — Funciones puras del paso temporal del DAG metabólico.
//!
//! Aritmética de reparto de flujo y redistribución por constraint de Carnot.
//! Sin dependencia ECS — los sistemas orquestan queries y llaman aquí.
//!
//! **Mapeo T_core / T_env (convención del track MG):**
//! - T_core = `equivalent_temperature(density(qe, radius))` (Capa 0 × Capa 1).
//! - T_env  = `ambient_equivalent_temperature(terrain_viscosity)` (Capa 6).

use crate::blueprint::constants::{
    AMBIENT_BASE_TEMPERATURE, AMBIENT_TEMP_VISCOSITY_SCALE, DIVISION_GUARD_EPSILON,
};
use crate::layers::METABOLIC_GRAPH_MAX_EDGES;

/// Reparte `available_exergy` proporcionalmente a `max_capacity` de cada arista saliente.
/// - Vacío → retorna (array vacío, 0) — nodo terminal, la exergía se disipa completa.
/// - Suma de capacidades ≈ 0 → reparto uniforme.
/// - Σ J_out ≤ available_exergy (cada arista capeada a su max_capacity).
#[inline]
pub fn propagate_edge_flows(
    available_exergy: f32,
    edge_capacities: &[(u8, f32)],
) -> ([(u8, f32); METABOLIC_GRAPH_MAX_EDGES], usize) {
    let mut result = [(0u8, 0.0f32); METABOLIC_GRAPH_MAX_EDGES];
    let n = edge_capacities.len().min(METABOLIC_GRAPH_MAX_EDGES);
    if n == 0 || !available_exergy.is_finite() || available_exergy <= 0.0 {
        return (result, 0);
    }

    let total_capacity: f32 = edge_capacities[..n]
        .iter()
        .map(|(_, cap)| san_nonneg(*cap))
        .sum();

    if total_capacity <= DIVISION_GUARD_EPSILON {
        let share = available_exergy / n as f32;
        for (i, &(idx, _)) in edge_capacities[..n].iter().enumerate() {
            result[i] = (idx, share);
        }
        return (result, n);
    }

    let mut remaining = available_exergy;
    for (i, &(idx, cap)) in edge_capacities[..n].iter().enumerate() {
        let cap_safe = san_nonneg(cap);
        let proportion = cap_safe / total_capacity;
        let ideal = available_exergy * proportion;
        let assigned = ideal.min(cap_safe).min(remaining).max(0.0);
        result[i] = (idx, assigned);
        remaining -= assigned;
    }

    (result, n)
}

/// f32 finito y ≥ 0; NaN/Inf → 0.
#[inline]
fn san_nonneg(x: f32) -> f32 {
    if x.is_finite() { x.max(0.0) } else { 0.0 }
}

/// f32 finito clampeado a [0, 1]; NaN/Inf → 0.
#[inline]
fn san_efficiency(x: f32) -> f32 {
    if x.is_finite() {
        x.clamp(0.0, 1.0)
    } else {
        0.0
    }
}

/// Redistribuye violación de eficiencia tras constraint de Carnot.
/// Si `current_efficiency > carnot_limit`, reduce a `carnot_limit` y genera calor adicional.
/// Retorna `(new_efficiency, additional_heat)`.
///
/// - `current_efficiency ≤ carnot_limit` → sin cambio: `(current_efficiency, 0.0)`.
/// - `carnot_limit` fuera de `[0, 1)` se clampea.
/// - Calor adicional = (η_old - η_new) × (exergy_in - activation_energy).max(0).
#[inline]
pub fn redistribute_node_violation(
    current_efficiency: f32,
    carnot_limit: f32,
    exergy_in: f32,
    activation_energy: f32,
) -> (f32, f32) {
    let eta_curr = san_efficiency(current_efficiency);
    let eta_max = if carnot_limit.is_finite() {
        carnot_limit.clamp(0.0, 1.0 - f32::EPSILON)
    } else {
        0.0
    };

    if eta_curr <= eta_max {
        return (eta_curr, 0.0);
    }

    let effective_input = (san_nonneg(exergy_in) - san_nonneg(activation_energy)).max(0.0);
    let extra_heat = (eta_curr - eta_max) * effective_input;

    (eta_max, extra_heat)
}

/// Temperatura ambiental equivalente desde viscosidad del terreno (Capa 6).
/// T_env = T_base + (viscosity - 1) × scale.
/// Bioma plain (viscosity=1) → T_base. Agua (viscosity≫1) → más cálido.
#[inline]
pub fn ambient_equivalent_temperature(terrain_viscosity: f32) -> f32 {
    let v = if terrain_viscosity.is_finite() {
        terrain_viscosity.max(0.0)
    } else {
        1.0
    };
    (AMBIENT_BASE_TEMPERATURE + (v - 1.0) * AMBIENT_TEMP_VISCOSITY_SCALE).max(1.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── propagate_edge_flows ──

    #[test]
    fn propagate_proportional_exact_split() {
        let (flows, n) = propagate_edge_flows(100.0, &[(0, 60.0), (1, 40.0)]);
        assert_eq!(n, 2);
        assert!((flows[0].1 - 60.0).abs() < 1e-3, "edge 0: {}", flows[0].1);
        assert!((flows[1].1 - 40.0).abs() < 1e-3, "edge 1: {}", flows[1].1);
    }

    #[test]
    fn propagate_empty_edges_returns_empty() {
        let (_, n) = propagate_edge_flows(100.0, &[]);
        assert_eq!(n, 0);
    }

    #[test]
    fn propagate_zero_capacities_uniform_split() {
        let (flows, n) = propagate_edge_flows(100.0, &[(0, 0.0), (1, 0.0)]);
        assert_eq!(n, 2);
        assert!((flows[0].1 - 50.0).abs() < 1e-3);
        assert!((flows[1].1 - 50.0).abs() < 1e-3);
    }

    #[test]
    fn propagate_zero_exergy_returns_zero_flows() {
        let (_flows, n) = propagate_edge_flows(0.0, &[(0, 60.0), (1, 40.0)]);
        assert_eq!(n, 0);
    }

    #[test]
    fn propagate_negative_exergy_returns_empty() {
        let (_, n) = propagate_edge_flows(-10.0, &[(0, 50.0)]);
        assert_eq!(n, 0);
    }

    #[test]
    fn propagate_respects_individual_cap() {
        // available=200, but caps are 60+40=100 → total assigned ≤ 100
        let (flows, n) = propagate_edge_flows(200.0, &[(0, 60.0), (1, 40.0)]);
        assert_eq!(n, 2);
        assert!(flows[0].1 <= 60.0 + 1e-3);
        assert!(flows[1].1 <= 40.0 + 1e-3);
    }

    #[test]
    fn propagate_preserves_edge_indices() {
        let (flows, n) = propagate_edge_flows(50.0, &[(3, 25.0), (7, 25.0)]);
        assert_eq!(n, 2);
        assert_eq!(flows[0].0, 3);
        assert_eq!(flows[1].0, 7);
    }

    #[test]
    fn propagate_nan_exergy_returns_empty() {
        let (_, n) = propagate_edge_flows(f32::NAN, &[(0, 50.0)]);
        assert_eq!(n, 0);
    }

    #[test]
    fn propagate_nan_capacity_treated_as_zero() {
        let (flows, n) = propagate_edge_flows(100.0, &[(0, f32::NAN), (1, 40.0)]);
        assert_eq!(n, 2);
        // NaN cap → 0; only edge 1 has valid capacity → gets all available.
        assert!(flows[0].1.is_finite());
        assert!(flows[1].1.is_finite());
        assert!(
            flows[1].1 > flows[0].1,
            "valid cap edge should get more flow"
        );
    }

    // ── redistribute_node_violation ──

    #[test]
    fn redistribute_violating_node_clamped() {
        let (eff, heat) = redistribute_node_violation(0.8, 0.6, 100.0, 10.0);
        assert!((eff - 0.6).abs() < 1e-5);
        assert!(heat > 0.0, "extra heat must be positive");
        // extra = (0.8 - 0.6) * (100 - 10) = 0.2 * 90 = 18
        assert!((heat - 18.0).abs() < 1e-3);
    }

    #[test]
    fn redistribute_non_violating_unchanged() {
        let (eff, heat) = redistribute_node_violation(0.5, 0.6, 100.0, 10.0);
        assert!((eff - 0.5).abs() < 1e-5);
        assert_eq!(heat, 0.0);
    }

    #[test]
    fn redistribute_exactly_at_limit_unchanged() {
        let (eff, heat) = redistribute_node_violation(0.6, 0.6, 100.0, 10.0);
        assert!((eff - 0.6).abs() < 1e-5);
        assert_eq!(heat, 0.0);
    }

    #[test]
    fn redistribute_zero_carnot_collapses() {
        let (eff, heat) = redistribute_node_violation(0.8, 0.0, 100.0, 5.0);
        assert!(eff < 1e-5);
        assert!(heat > 0.0);
    }

    #[test]
    fn redistribute_nan_inputs_safe() {
        let (eff, heat) = redistribute_node_violation(f32::NAN, 0.5, 100.0, 10.0);
        assert!(eff.is_finite());
        assert!(heat.is_finite());
    }

    // ── ambient_equivalent_temperature ──

    #[test]
    fn ambient_temp_plain_biome_equals_base() {
        let t = ambient_equivalent_temperature(1.0);
        assert!((t - AMBIENT_BASE_TEMPERATURE).abs() < 1e-3);
    }

    #[test]
    fn ambient_temp_higher_viscosity_higher_temp() {
        let t_plain = ambient_equivalent_temperature(1.0);
        let t_water = ambient_equivalent_temperature(3.0);
        assert!(t_water > t_plain);
    }

    #[test]
    fn ambient_temp_zero_viscosity_clamped() {
        let t = ambient_equivalent_temperature(0.0);
        assert!(t >= 1.0);
    }

    #[test]
    fn ambient_temp_nan_returns_base() {
        let t = ambient_equivalent_temperature(f32::NAN);
        assert!((t - AMBIENT_BASE_TEMPERATURE).abs() < 1e-3);
    }
}
