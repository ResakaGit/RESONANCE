//! Emergencia de vida desde surplus energético — condición de viabilidad axiomática.
//! Life emergence from energy surplus — axiomatic viability condition.
//!
//! No hardcoded bands, no element-specific catalysts.
//! Life emerges wherever constructive interference gain exceeds dissipation loss.
//!
//! **Model:** Sigmoid viability potential `net/(net+qe)` — ratio of free surplus to total
//! energy. NOT a Prigogine dissipative-structure model (no far-from-equilibrium dynamics,
//! no nonlinear feedback). Simpler: if coherence gain > dissipation cost, the local energy
//! field can sustain a self-maintaining pattern.
//!
//! **Axiom grounding:**
//! - **Axiom 1:** Everything is energy (qe). Existence = qe > 0.
//! - **Axiom 4:** All processes dissipate energy. loss ≥ qe × rate.
//! - **Axiom 7:** Interaction decays with distance. attenuation = 1 / (1 + d²).
//! - **Axiom 8:** Every concentration oscillates at frequency f. Interference = cos(2π Δf t + Δφ).
//!
//! Derived: spawn_condition = coherence_gain(neighbors) > dissipation_cost(local).
//! Entity properties (matter_state, capabilities, morph profile) derived from energy density.

use crate::blueprint::equations::derived_thresholds as dt;
use crate::layers::MatterState;

// ── All thresholds derived from 4 fundamentals via derived_thresholds.rs ────
// No hardcoded constants. See docs/sprints/AXIOMATIC_INFERENCE/ for derivations.

/// Frequency alignment bandwidth (Hz). Axiom 8: observation window.
/// Centralized in derived_thresholds.rs (4th fundamental constant).
use crate::blueprint::equations::derived_thresholds::COHERENCE_BANDWIDTH;

// ── Profile derivation scales (derived from density thresholds) ─────────────

/// Reference density for profile scaling = gas threshold × 2 (high-energy baseline).
fn profile_density_reference() -> f32 { dt::gas_density_threshold() * 2.0 }

/// Reference velocity for mobility_bias = 1.0.
/// Derived: sqrt(gas_density_threshold) — flow speed at gas transition energy.
fn profile_velocity_reference() -> f32 { dt::gas_density_threshold().sqrt() }

/// Frequency alignment factor (Axiom 8, time-averaged). Delegates to centralized impl.
#[inline]
pub fn frequency_alignment(freq_a: f32, freq_b: f32) -> f32 {
    super::super::determinism::gaussian_frequency_alignment(freq_a, freq_b, COHERENCE_BANDWIDTH)
}

/// Coherence gain from neighboring cells (Axioms 7 + 8).
///
/// Sums constructive interference contributions from neighbors, each attenuated by distance.
/// `neighbors`: `(qe, frequency_hz, distance)` per neighbor.
///
/// `coherence = Σ qe_i × alignment(f_center, f_i) × 1/(1 + d_i²)`
#[inline]
pub fn cell_coherence_gain(
    center_hz: f32,
    neighbors: &[(f32, f32, f32)],
) -> f32 {
    if !center_hz.is_finite() { return 0.0; }
    let mut sum = 0.0f32;
    for &(qe, hz, dist) in neighbors {
        if !qe.is_finite() || !hz.is_finite() || !dist.is_finite() { continue; }
        let alignment = frequency_alignment(center_hz, hz);
        let attenuation = 1.0 / (1.0 + dist * dist); // Axiom 7
        sum += qe.max(0.0) * alignment * attenuation;
    }
    sum
}

/// Axiomatic abiogenesis potential (Axioms 1, 4, 7, 8).
///
/// `potential = (coherence_gain - dissipation_cost) / normalizer`
///
/// Returns `[0, 1]`: above `spawn_potential_threshold()` → spawn possible.
/// Frequency-agnostic: works for any band. The dominant frequency of the cell
/// determines WHAT emerges, not WHETHER it emerges.
pub fn axiomatic_abiogenesis_potential(
    cell_qe: f32,
    coherence_gain: f32,
    dissipation_rate: f32,
) -> f32 {
    let qe = if cell_qe.is_finite() { cell_qe.max(0.0) } else { return 0.0 };
    if qe < dt::self_sustaining_qe_min() { return 0.0; }
    let gain = if coherence_gain.is_finite() { coherence_gain.max(0.0) } else { 0.0 };
    let loss = qe * dissipation_rate.max(0.0);
    let net = gain - loss;
    if net <= 0.0 { return 0.0; }
    // Sigmoid normalization to [0, 1]
    (net / (net + qe)).clamp(0.0, 1.0)
}

/// Whether axiomatic potential exceeds spawn threshold.
#[inline]
pub fn axiomatic_spawn_viable(potential: f32) -> bool {
    potential >= dt::spawn_potential_threshold()
}

/// Matter state derived from energy density (Axiom 1).
///
/// High density → Plasma (unbound, energetic).
/// Low density → Solid (crystallized, stable).
/// No element-specific mapping — state is a CONSEQUENCE of energy.
pub fn matter_state_from_density(qe: f32, volume: f32) -> MatterState {
    let density = qe.max(0.0) / volume.max(f32::EPSILON);
    if density >= dt::plasma_density_threshold() { MatterState::Plasma }
    else if density >= dt::gas_density_threshold() { MatterState::Gas }
    else if density >= dt::liquid_density_threshold() { MatterState::Liquid }
    else { MatterState::Solid }
}

/// Capabilities derived from energy profile (Axioms 1, 8).
///
/// - MOVE: moderate density (not crystallized, not diffuse)
/// - SENSE: high coherence (can detect frequency patterns)
/// - BRANCH: enough energy + not gaseous
/// - GROW: always (qe accumulation is universal)
pub fn capabilities_from_energy(
    qe: f32,
    density: f32,
    coherence: f32,
) -> u8 {
    use crate::layers::CapabilitySet;
    let mut caps = CapabilitySet::GROW;
    if density >= dt::move_density_min() && density <= dt::move_density_max() {
        caps |= CapabilitySet::MOVE;
    }
    if coherence >= dt::sense_coherence_min() {
        caps |= CapabilitySet::SENSE;
    }
    if qe >= dt::branch_qe_min() && density < dt::gas_density_threshold() {
        caps |= CapabilitySet::BRANCH;
    }
    caps
}

/// Inference profile derived from energy state (Axioms 1, 3, 8).
///
/// Returns `(growth_bias, mobility_bias, branching_bias, resilience)`.
/// All derived from density, coherence, and flow demand — no arbitrary assignment.
pub fn inference_profile_from_energy(
    density: f32,
    coherence: f32,
    flow_speed: f32,
) -> (f32, f32, f32, f32) {
    let d = density.max(0.0);
    let c = coherence.clamp(0.0, 1.0);
    let growth    = (1.0 - d / profile_density_reference()).clamp(0.1, 0.95);
    let mobility  = (flow_speed / profile_velocity_reference().max(1.0)).clamp(0.0, 0.95);
    let branching = growth * (1.0 - mobility).max(0.1); // mobile entities don't branch
    let resilience = (0.5 * d / profile_density_reference() + 0.5 * c).clamp(0.1, 0.95); // density + coherence → structural organization
    (growth, mobility, branching, resilience)
}

/// Bond energy from local qe (Axiom 1 + 4).
///
/// `bond = qe / DISSIPATION_SOLID` — stronger bonds where dissipation is lowest.
/// Clamped to [liquid_threshold, plasma_threshold × 10] (physical bounds).
#[inline]
pub fn bond_from_energy(qe: f32) -> f32 {
    let scale = 1.0 / dt::DISSIPATION_SOLID; // = 200
    (qe.max(0.0) * scale).clamp(dt::liquid_density_threshold(), dt::plasma_density_threshold() * 10.0)
}

/// Thermal conductivity from matter state (Axiom 4).
///
/// `conductivity = dissipation × DENSITY_SCALE` — proportional to particle freedom.
/// Derived: conductivity scales with dissipation rate (Axiom 4), normalized by spatial scale.
pub fn conductivity_from_state(state: MatterState) -> f32 {
    dissipation_from_state(state) * dt::DENSITY_SCALE
}

/// Dissipation rate from matter state (Axiom 4).
///
/// Plasma dissipates fastest (highest entropy production), Solid slowest.
pub fn dissipation_from_state(state: MatterState) -> f32 {
    match state {
        MatterState::Plasma => dt::DISSIPATION_PLASMA,
        MatterState::Gas    => dt::DISSIPATION_GAS,
        MatterState::Liquid => dt::DISSIPATION_LIQUID,
        MatterState::Solid  => dt::DISSIPATION_SOLID,
    }
}

/// Element symbol from dominant frequency (Axiom 8: frequency = identity).
///
/// Buckets by COHERENCE_BANDWIDTH (50 Hz) bands. Returns a static str
/// suitable for `ElementId::from_name`. Used by both abiogenesis and EntityBuilder.
#[inline]
pub fn element_symbol_from_frequency(frequency_hz: f32) -> &'static str {
    let band = (frequency_hz / COHERENCE_BANDWIDTH) as u32;
    match band {
        0       => "Um",  // Umbra    ~0–50 Hz
        1       => "Te",  // Terra    ~50–100 Hz
        2..=4   => "Fl",  // Flora    ~100–250 Hz
        5..=6   => "Aq",  // Aqua     ~250–350 Hz
        7..=10  => "Ig",  // Ignis    ~350–550 Hz
        11..=15 => "Ve",  // Ventus   ~550–800 Hz
        _       => "Lx",  // Lux      ~800+ Hz
    }
}

/// Initial radius from energy budget (Axiom 1 + 4).
///
/// `radius = sqrt(qe) × DISSIPATION_SOLID` — higher energy = larger extent, scaled by base dissipation.
/// Clamped: min = DISSIPATION_SOLID (smallest stable structure), max = 1.0 (grid-scale limit).
#[inline]
pub fn initial_radius_from_qe(qe: f32) -> f32 {
    (qe.max(0.0).sqrt() * dt::DISSIPATION_SOLID).clamp(dt::DISSIPATION_SOLID, 1.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn alignment_same_frequency_is_one() {
        let a = frequency_alignment(100.0, 100.0);
        assert!((a - 1.0).abs() < 1e-6, "same freq alignment={a}");
    }

    #[test]
    fn alignment_far_frequency_near_zero() {
        let a = frequency_alignment(100.0, 500.0);
        assert!(a < 0.01, "distant freq alignment={a}");
    }

    #[test]
    fn alignment_symmetric() {
        let ab = frequency_alignment(100.0, 150.0);
        let ba = frequency_alignment(150.0, 100.0);
        assert!((ab - ba).abs() < 1e-6);
    }

    #[test]
    fn coherence_gain_empty_neighbors_is_zero() {
        assert_eq!(cell_coherence_gain(100.0, &[]), 0.0);
    }

    #[test]
    fn coherence_gain_same_freq_close_neighbor_high() {
        let gain = cell_coherence_gain(100.0, &[(50.0, 100.0, 1.0)]);
        assert!(gain > 10.0, "close same-freq neighbor should give high gain: {gain}");
    }

    #[test]
    fn coherence_gain_far_neighbor_attenuated() {
        let close = cell_coherence_gain(100.0, &[(50.0, 100.0, 1.0)]);
        let far   = cell_coherence_gain(100.0, &[(50.0, 100.0, 10.0)]);
        assert!(close > far, "close > far: {close} vs {far}");
    }

    #[test]
    fn potential_below_min_qe_is_zero() {
        let p = axiomatic_abiogenesis_potential(5.0, 100.0, 0.01);
        assert_eq!(p, 0.0);
    }

    #[test]
    fn potential_no_coherence_is_zero() {
        let p = axiomatic_abiogenesis_potential(50.0, 0.0, 0.01);
        assert_eq!(p, 0.0);
    }

    #[test]
    fn potential_high_coherence_low_dissipation_is_positive() {
        let p = axiomatic_abiogenesis_potential(50.0, 30.0, 0.01);
        assert!(p > 0.0, "should be positive: {p}");
    }

    #[test]
    fn potential_in_unit_range() {
        for qe in [20.0, 100.0, 500.0, 2000.0] {
            let p = axiomatic_abiogenesis_potential(qe, qe * 2.0, 0.01);
            assert!((0.0..=1.0).contains(&p), "out of range: {p} for qe={qe}");
        }
    }

    #[test]
    fn matter_state_low_density_is_solid() {
        assert_eq!(matter_state_from_density(1.0, 1.0), MatterState::Solid);
    }

    #[test]
    fn matter_state_high_density_is_plasma() {
        let plasma_d = dt::plasma_density_threshold();
        assert_eq!(matter_state_from_density(plasma_d * 2.0, 1.0), MatterState::Plasma);
    }

    #[test]
    fn matter_state_transitions_monotonic() {
        let s = matter_state_from_density(1.0, 1.0);
        let l = matter_state_from_density(dt::liquid_density_threshold() + 1.0, 1.0);
        let g = matter_state_from_density(dt::gas_density_threshold() + 1.0, 1.0);
        let p = matter_state_from_density(dt::plasma_density_threshold() + 1.0, 1.0);
        assert_eq!(s, MatterState::Solid);
        assert_eq!(l, MatterState::Liquid);
        assert_eq!(g, MatterState::Gas);
        assert_eq!(p, MatterState::Plasma);
    }

    #[test]
    fn capabilities_below_move_min_no_move() {
        let caps = capabilities_from_energy(10.0, 1.0, 0.0);
        assert_eq!(caps & crate::layers::CapabilitySet::MOVE, 0);
        assert_ne!(caps & crate::layers::CapabilitySet::GROW, 0);
    }

    #[test]
    fn capabilities_in_move_range_gets_move() {
        let mid = (dt::move_density_min() + dt::move_density_max()) * 0.5;
        let caps = capabilities_from_energy(100.0, mid, 0.8);
        assert_ne!(caps & crate::layers::CapabilitySet::MOVE, 0);
        assert_ne!(caps & crate::layers::CapabilitySet::SENSE, 0);
    }

    #[test]
    fn profile_high_density_and_coherence_high_resilience() {
        let ref_d = profile_density_reference();
        let (_, _, _, resilience) = inference_profile_from_energy(ref_d, 0.9, 0.0);
        assert!(resilience > 0.7, "high density + coherence → high resilience: {resilience}");
    }

    #[test]
    fn profile_high_flow_high_mobility() {
        let ref_v = profile_velocity_reference();
        let (_, mobility, _, _) = inference_profile_from_energy(200.0, 0.5, ref_v * 0.9);
        assert!(mobility > 0.7, "flow near reference → high mobility: {mobility}");
    }
}
