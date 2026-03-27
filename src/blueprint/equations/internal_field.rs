//! Internal energy field equations — diffusion along 8 body-axis nodes.
//!
//! Enables emergent organ-like structures: energy concentrations drive
//! variable cross-section geometry. No organs are programmed.
//!
//! Axiom 1: everything is qe. Axiom 4: dissipation. Axiom 6: emergence.
//! Axiom 7: distance attenuation (adjacent nodes only).

/// Number of internal energy nodes along the body axis.
pub const NODE_COUNT: usize = 8;

/// Sum all nodes. Invariant: result >= 0.0.
#[inline]
pub fn field_total(field: &[f32; NODE_COUNT]) -> f32 {
    field.iter().sum()
}

/// Diffuse energy between adjacent nodes. Conservation guaranteed.
///
/// `conductivity ∈ [0,1]`: 0 = no diffusion (persistent gradients),
/// 1 = instant equalization. Each adjacent pair exchanges
/// `delta = (source - target) × conductivity × dt`, clamped to avoid overshoot.
pub fn field_diffuse(field: &[f32; NODE_COUNT], conductivity: f32, dt: f32) -> [f32; NODE_COUNT] {
    let mut out = *field;
    let k = conductivity.clamp(0.0, 1.0) * dt;
    for i in 0..(NODE_COUNT - 1) {
        let delta = (out[i] - out[i + 1]) * k * 0.5;
        // Clamp to prevent negative nodes
        let safe = delta.clamp(-out[i + 1] * 0.5, out[i] * 0.5);
        out[i]     -= safe;
        out[i + 1] += safe;
    }
    out
}

/// Frequency entrainment between adjacent internal nodes.
///
/// Adjacent nodes with different frequencies pull toward each other.
/// Axiom 8: oscillatory nature modulates everything.
pub fn freq_field_entrain(freq: &[f32; NODE_COUNT], coupling: f32, dt: f32) -> [f32; NODE_COUNT] {
    let mut out = *freq;
    let k = coupling.clamp(0.0, 1.0) * dt;
    for i in 0..(NODE_COUNT - 1) {
        let delta = (out[i + 1] - out[i]) * k * 0.5;
        out[i]     += delta;
        out[i + 1] -= delta;
    }
    out
}

/// Map internal qe field to per-node radii for variable-thickness geometry.
///
/// `radius_i = base_radius × (qe_i / mean_qe).sqrt()`, clamped to [min_ratio, max_ratio].
/// Sqrt produces natural scaling: 4× energy → 2× radius (area-proportional).
pub fn field_to_radii(
    qe_field: &[f32; NODE_COUNT],
    base_radius: f32,
    min_ratio: f32,
    max_ratio: f32,
) -> [f32; NODE_COUNT] {
    let total = field_total(qe_field);
    let mean = if total > 1e-6 { total / NODE_COUNT as f32 } else { 1.0 };
    let mut radii = [0.0; NODE_COUNT];
    for i in 0..NODE_COUNT {
        let ratio = if mean > 1e-6 { qe_field[i] / mean } else { 1.0 };
        let scaled = ratio.sqrt().clamp(min_ratio, max_ratio);
        radii[i] = base_radius * scaled;
    }
    radii
}

/// Distribute a scalar qe into 8 nodes weighted by profile.
///
/// Profile is normalized internally. Returns field where `sum == total_qe`.
pub fn distribute_to_field(total_qe: f32, profile: &[f32; NODE_COUNT]) -> [f32; NODE_COUNT] {
    let sum: f32 = profile.iter().sum();
    let mut field = [0.0; NODE_COUNT];
    if sum < 1e-10 || total_qe <= 0.0 {
        // Uniform fallback
        let per = total_qe / NODE_COUNT as f32;
        field.fill(per.max(0.0));
        return field;
    }
    for i in 0..NODE_COUNT {
        field[i] = total_qe * (profile[i] / sum);
    }
    field
}

/// Genome biases → initial distribution profile.
///
/// Axiom 6: no organs programmed — just energy distribution patterns.
/// - growth high → tips (nodes 0, 7) get more energy (elongation)
/// - resilience high → center (nodes 3, 4) concentrates (robust core)
/// - branching high → even spread with local peaks (multi-lobe)
pub fn genome_to_profile(
    growth_bias: f32,
    resilience: f32,
    branching_bias: f32,
) -> [f32; NODE_COUNT] {
    let mut profile = [1.0_f32; NODE_COUNT];

    // Growth → tip emphasis
    let g = growth_bias;
    profile[0] += g * 2.0;
    profile[7] += g * 2.0;
    profile[1] += g * 1.0;
    profile[6] += g * 1.0;

    // Resilience → center emphasis
    let r = resilience;
    profile[3] += r * 3.0;
    profile[4] += r * 3.0;
    profile[2] += r * 1.5;
    profile[5] += r * 1.5;

    // Branching → periodic peaks (lobes)
    let b = branching_bias;
    profile[1] += b * 2.0;
    profile[3] += b * 2.0;
    profile[5] += b * 2.0;
    profile[7] += b * 1.0;

    profile
}

/// Rescale field so sum matches target_qe. Conservation repair.
pub fn rescale_field(field: &mut [f32; NODE_COUNT], target_qe: f32) {
    let sum = field_total(field);
    if sum < 1e-10 {
        let per = target_qe.max(0.0) / NODE_COUNT as f32;
        field.fill(per);
        return;
    }
    let factor = target_qe / sum;
    for v in field.iter_mut() {
        *v *= factor;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── field_total ─────────────────────────────────────────────────────────

    #[test]
    fn total_of_zeros_is_zero() {
        assert_eq!(field_total(&[0.0; 8]), 0.0);
    }

    #[test]
    fn total_sums_correctly() {
        assert!((field_total(&[1.0; 8]) - 8.0).abs() < 1e-5);
    }

    // ── field_diffuse ───────────────────────────────────────────────────────

    #[test]
    fn diffuse_conserves_energy() {
        let field = [10.0, 0.0, 5.0, 0.0, 8.0, 0.0, 3.0, 0.0];
        let before = field_total(&field);
        let after_field = field_diffuse(&field, 0.5, 0.05);
        let after = field_total(&after_field);
        assert!((after - before).abs() < 1e-4, "before={before} after={after}");
    }

    #[test]
    fn diffuse_smooths_gradient() {
        let field = [10.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0];
        let smoothed = field_diffuse(&field, 1.0, 1.0);
        assert!(smoothed[0] < 10.0, "peak should decrease");
        assert!(smoothed[1] > 0.0, "neighbor should increase");
    }

    #[test]
    fn diffuse_zero_conductivity_no_change() {
        let field = [5.0, 0.0, 3.0, 0.0, 7.0, 0.0, 1.0, 0.0];
        let result = field_diffuse(&field, 0.0, 1.0);
        for i in 0..8 {
            assert_eq!(field[i], result[i]);
        }
    }

    #[test]
    fn diffuse_never_negative() {
        let field = [100.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.001];
        for step in 0..100 {
            let result = field_diffuse(&field, 1.0, 0.1);
            for (i, &v) in result.iter().enumerate() {
                assert!(v >= 0.0, "step {step} node {i}: {v}");
            }
        }
    }

    // ── freq_field_entrain ──────────────────────────────────────────────────

    #[test]
    fn freq_entrain_converges() {
        let freq = [100.0, 200.0, 300.0, 400.0, 500.0, 600.0, 700.0, 800.0];
        let entrained = freq_field_entrain(&freq, 1.0, 1.0);
        let gap_before = (freq[0] - freq[1]).abs();
        let gap_after = (entrained[0] - entrained[1]).abs();
        assert!(gap_after < gap_before, "should converge");
    }

    // ── field_to_radii ──────────────────────────────────────────────────────

    #[test]
    fn uniform_field_uniform_radii() {
        let field = [5.0; 8];
        let radii = field_to_radii(&field, 1.0, 0.3, 2.5);
        for r in &radii {
            assert!((*r - 1.0).abs() < 1e-4, "uniform field → uniform radii: {r}");
        }
    }

    #[test]
    fn concentrated_field_variable_radii() {
        let mut field = [1.0; 8];
        field[3] = 10.0; // big lump at node 3
        let radii = field_to_radii(&field, 1.0, 0.3, 2.5);
        assert!(radii[3] > radii[0], "node 3 should be wider: {} vs {}", radii[3], radii[0]);
    }

    #[test]
    fn radii_clamped_to_bounds() {
        let field = [0.001, 0.001, 0.001, 100.0, 0.001, 0.001, 0.001, 0.001];
        let radii = field_to_radii(&field, 1.0, 0.3, 2.5);
        for r in &radii {
            assert!(*r >= 0.3 - 1e-5 && *r <= 2.5 + 1e-5, "r={r} out of bounds");
        }
    }

    // ── distribute_to_field ─────────────────────────────────────────────────

    #[test]
    fn distribute_conserves() {
        let profile = [1.0, 2.0, 1.0, 3.0, 1.0, 2.0, 1.0, 1.0];
        let field = distribute_to_field(100.0, &profile);
        assert!((field_total(&field) - 100.0).abs() < 1e-4);
    }

    #[test]
    fn distribute_zero_profile_is_uniform() {
        let field = distribute_to_field(80.0, &[0.0; 8]);
        for v in &field {
            assert!((*v - 10.0).abs() < 1e-4);
        }
    }

    // ── genome_to_profile ───────────────────────────────────────────────────

    #[test]
    fn high_growth_emphasizes_tips() {
        let p = genome_to_profile(1.0, 0.0, 0.0);
        assert!(p[0] > p[3], "tip > center with high growth");
        assert!(p[7] > p[4], "tip > center with high growth");
    }

    #[test]
    fn high_resilience_emphasizes_center() {
        let p = genome_to_profile(0.0, 1.0, 0.0);
        assert!(p[3] > p[0], "center > tip with high resilience");
        assert!(p[4] > p[7], "center > tip with high resilience");
    }

    // ── rescale_field ───────────────────────────────────────────────────────

    #[test]
    fn rescale_matches_target() {
        let mut field = [1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0];
        rescale_field(&mut field, 100.0);
        assert!((field_total(&field) - 100.0).abs() < 1e-3);
    }

    #[test]
    fn rescale_zero_field_distributes_uniformly() {
        let mut field = [0.0; 8];
        rescale_field(&mut field, 40.0);
        for v in &field {
            assert!((*v - 5.0).abs() < 1e-4);
        }
    }
}
