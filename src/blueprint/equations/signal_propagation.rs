/// Pure math for finite-speed wave front propagation and diffusion budget.

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

/// Max cells the propagation front advances per tick.
pub const PROPAGATION_SPEED_CELLS_PER_TICK: f32 = 2.0;

/// Default diffusion conductivity between adjacent cells.
pub const DIFFUSION_CONDUCTIVITY_DEFAULT: f32 = 0.1;

/// Max cells the diffusion pass may process per tick.
pub const DIFFUSION_BUDGET_MAX: usize = 256;

/// Amplitude decay factor applied to the wave front each tick.
pub const FRONT_DAMPING_DEFAULT: f32 = 0.98;

// ---------------------------------------------------------------------------
// Pure functions
// ---------------------------------------------------------------------------

/// Maximum radius the propagation front can reach at a given tick.
pub const fn propagation_front_radius(speed: f32, elapsed_ticks: u32) -> f32 {
    if speed <= 0.0 {
        return 0.0;
    }
    speed * elapsed_ticks as f32
}

/// Intensity at a point considering wave front radius constraint.
///
/// Returns 0 if `distance > front_radius` (signal hasn't arrived yet).
/// Otherwise applies exponential spatial decay and temporal front damping.
///
/// `damping_per_tick = front_damping.pow(elapsed_ticks)` attenuates the
/// amplitude over time, while `exp(-decay_rate * distance)` attenuates
/// over space.
pub fn propagation_intensity_at_tick(
    source_qe: f32,
    distance: f32,
    decay_rate: f32,
    front_radius: f32,
    front_damping: f32,
    elapsed_ticks: u32,
) -> f32 {
    // Guard: outside the front, signal hasn't arrived.
    if distance > front_radius || front_radius <= 0.0 {
        return 0.0;
    }
    if source_qe <= 0.0 {
        return 0.0;
    }

    let damping = front_damping.clamp(0.0, 1.0);
    let decay = decay_rate.max(0.0);
    let dist = distance.max(0.0);

    // Spatial attenuation: exponential decay with distance.
    let spatial = (-decay * dist).exp();

    // Temporal attenuation: damping compounded over elapsed ticks.
    let temporal = damping.powi(elapsed_ticks as i32);

    let result = source_qe * spatial * temporal;
    result.max(0.0)
}

/// Energy delta for diffusion between two adjacent cells.
///
/// `delta_qe = conductivity * (source_qe - target_qe) * dt`
///
/// Positive result means source loses energy (flows toward target).
pub fn diffusion_delta(source_qe: f32, target_qe: f32, conductivity: f32, dt: f32) -> f32 {
    let k = conductivity.clamp(0.0, 1.0);
    let t = dt.max(0.0);
    let raw = k * (source_qe - target_qe) * t;

    // Clamp: cannot drain more than the source has, nor extract more than
    // the target has (when flow is negative, source is the receiver).
    let src = source_qe.max(0.0);
    let tgt = target_qe.max(0.0);
    raw.clamp(-tgt, src)
}

/// How many cells can be processed this tick given a budget.
pub const fn diffusion_budget(candidate_count: usize, budget_max: usize) -> usize {
    if candidate_count < budget_max {
        candidate_count
    } else {
        budget_max
    }
}

/// Decay constant for frequency purity loss over distance (λ in exp(-d/λ)).
/// Coherence falls to 1/e ≈ 37% at this distance. Shorter than amplitude λ
/// — identity degrades before the signal disappears entirely.
pub const FREQ_COHERENCE_DECAY_LAMBDA: f32 = 12.0;

/// Minimum frequency purity below which a receiver cannot identify the source frequency.
/// Used as a threshold in perception and entrainment coupling.
pub const FREQ_PURITY_PERCEPTION_THRESHOLD: f32 = 0.1;

/// Frequency purity received at `distance` from the source.
///
/// Equation: `exp(-distance / lambda_coherence)`
/// - `d = 0`:  purity = 1.0  (exact frequency, contact)
/// - `d = λ`:  purity ≈ 0.37 (degraded — band still identifiable)
/// - `d ≫ λ`:  purity → 0   (frequency unrecognisable)
///
/// Invariant: monotonically decreasing in `distance`. Always in `[0.0, 1.0]`.
pub fn frequency_purity_at_distance(distance: f32, lambda_coherence: f32) -> f32 {
    let lambda = lambda_coherence.max(0.001);
    let dist = distance.max(0.0);
    (-dist / lambda).exp()
}

/// Entrainment coupling strength modulated by frequency purity at distance.
///
/// The Kuramoto model requires knowing the neighbour's frequency precisely.
/// At low purity the coupling collapses, giving the system a natural radius.
pub fn entrainment_coupling_at_distance(base_coupling: f32, distance: f32, lambda_coherence: f32) -> f32 {
    base_coupling.max(0.0) * frequency_purity_at_distance(distance, lambda_coherence)
}

/// Whether a cell sits inside the active corona (the ring where new propagation happens).
///
/// The corona spans from `front_radius - speed` (inner edge, previous tick's front)
/// to `front_radius` (current outer edge). Cells in this ring are the ones
/// freshly reached by the wave front this tick.
pub fn is_in_propagation_corona(distance: f32, front_radius: f32, speed: f32) -> bool {
    if front_radius <= 0.0 || speed <= 0.0 {
        return false;
    }
    let inner = (front_radius - speed).max(0.0);
    distance >= inner && distance <= front_radius
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn approx_eq(a: f32, b: f32) {
        assert!(
            (a - b).abs() < 1e-5,
            "expected approx equal: left={a}, right={b}"
        );
    }

    // -- propagation_front_radius --

    #[test]
    fn propagation_front_radius_grows_linearly() {
        approx_eq(propagation_front_radius(2.0, 0), 0.0);
        approx_eq(propagation_front_radius(2.0, 1), 2.0);
        approx_eq(propagation_front_radius(2.0, 5), 10.0);
        approx_eq(propagation_front_radius(3.5, 4), 14.0);
    }

    #[test]
    fn propagation_front_radius_zero_speed_returns_zero() {
        approx_eq(propagation_front_radius(0.0, 10), 0.0);
    }

    #[test]
    fn propagation_front_radius_negative_speed_returns_zero() {
        approx_eq(propagation_front_radius(-1.0, 5), 0.0);
    }

    // -- propagation_intensity_at_tick --

    #[test]
    fn propagation_intensity_zero_before_front_arrives() {
        // Front radius 4.0, but cell is at distance 6.0 — not reached yet.
        let intensity = propagation_intensity_at_tick(100.0, 6.0, 0.1, 4.0, 0.98, 2);
        approx_eq(intensity, 0.0);
    }

    #[test]
    fn propagation_intensity_decays_with_distance() {
        let near = propagation_intensity_at_tick(100.0, 1.0, 0.1, 10.0, 1.0, 1);
        let far = propagation_intensity_at_tick(100.0, 5.0, 0.1, 10.0, 1.0, 1);
        assert!(near > far, "near={near} should be > far={far}");
        assert!(far > 0.0);
    }

    #[test]
    fn propagation_intensity_decays_with_ticks() {
        // Same distance, same front radius, but more elapsed ticks means more damping.
        let early = propagation_intensity_at_tick(100.0, 2.0, 0.1, 10.0, 0.9, 1);
        let late = propagation_intensity_at_tick(100.0, 2.0, 0.1, 10.0, 0.9, 10);
        assert!(early > late, "early={early} should be > late={late}");
        assert!(late > 0.0);
    }

    #[test]
    fn propagation_intensity_full_at_origin_no_damping() {
        // distance=0, damping=1.0 (no decay), decay_rate=0 (no spatial decay).
        let intensity = propagation_intensity_at_tick(50.0, 0.0, 0.0, 10.0, 1.0, 5);
        approx_eq(intensity, 50.0);
    }

    #[test]
    fn propagation_intensity_zero_source_returns_zero() {
        let intensity = propagation_intensity_at_tick(0.0, 1.0, 0.1, 10.0, 0.98, 3);
        approx_eq(intensity, 0.0);
    }

    #[test]
    fn propagation_intensity_zero_front_radius_returns_zero() {
        let intensity = propagation_intensity_at_tick(100.0, 0.0, 0.1, 0.0, 0.98, 0);
        approx_eq(intensity, 0.0);
    }

    // -- diffusion_delta --

    #[test]
    fn diffusion_delta_flows_from_high_to_low() {
        let delta = diffusion_delta(100.0, 50.0, 0.1, 1.0);
        approx_eq(delta, 5.0);
        assert!(delta > 0.0);
    }

    #[test]
    fn diffusion_delta_zero_when_equal() {
        let delta = diffusion_delta(42.0, 42.0, 0.1, 1.0);
        approx_eq(delta, 0.0);
    }

    #[test]
    fn diffusion_delta_negative_when_target_higher() {
        let delta = diffusion_delta(20.0, 80.0, 0.1, 1.0);
        assert!(delta < 0.0);
        approx_eq(delta, -6.0); // 0.1 * (20 - 80) * 1.0 = -6.0
    }

    #[test]
    fn diffusion_delta_clamps_cannot_drain_source() {
        // source has 2 qe, raw delta would be 0.5 * (2 - 100) * 1.0 = -49.
        // Clamped to -target (negative direction) or +source (positive direction).
        // Here raw = -49, clamp(-100, 2) = -49... but also cannot extract more
        // than target has. Actually raw=-49, clamp(-100,2) = -49. Let's test
        // the positive direction clamp: source=1, target=0, conductivity=1, dt=10
        // raw = 1*1*10 = 10, but clamped to source=1.
        let delta = diffusion_delta(1.0, 0.0, 1.0, 10.0);
        approx_eq(delta, 1.0);
    }

    #[test]
    fn diffusion_delta_clamps_cannot_drain_target() {
        // Negative flow: source=0, target=2, conductivity=1, dt=10.
        // raw = 1.0 * (0 - 2) * 10 = -20, clamped to -target = -2.
        let delta = diffusion_delta(0.0, 2.0, 1.0, 10.0);
        approx_eq(delta, -2.0);
    }

    #[test]
    fn diffusion_delta_is_antisymmetric() {
        let ab = diffusion_delta(80.0, 30.0, 0.1, 0.5);
        let ba = diffusion_delta(30.0, 80.0, 0.1, 0.5);
        approx_eq(ab, -ba);
    }

    // -- diffusion_budget --

    #[test]
    fn diffusion_budget_clamps_to_max() {
        assert_eq!(diffusion_budget(1000, 256), 256);
    }

    #[test]
    fn diffusion_budget_passes_when_under_max() {
        assert_eq!(diffusion_budget(100, 256), 100);
    }

    #[test]
    fn diffusion_budget_exact_match() {
        assert_eq!(diffusion_budget(256, 256), 256);
    }

    // -- is_in_propagation_corona --

    #[test]
    fn is_in_propagation_corona_true_at_edge() {
        // front_radius=10, speed=2 => corona is [8, 10].
        assert!(is_in_propagation_corona(9.0, 10.0, 2.0));
        assert!(is_in_propagation_corona(10.0, 10.0, 2.0));
        assert!(is_in_propagation_corona(8.0, 10.0, 2.0));
    }

    #[test]
    fn is_in_propagation_corona_false_inside() {
        // distance=3 is well inside the front (inner edge at 8).
        assert!(!is_in_propagation_corona(3.0, 10.0, 2.0));
    }

    #[test]
    fn is_in_propagation_corona_false_outside() {
        // distance=12 is beyond the front.
        assert!(!is_in_propagation_corona(12.0, 10.0, 2.0));
    }

    #[test]
    fn is_in_propagation_corona_first_tick() {
        // front_radius=2, speed=2 => corona is [0, 2]. Everything within is corona.
        assert!(is_in_propagation_corona(0.0, 2.0, 2.0));
        assert!(is_in_propagation_corona(1.0, 2.0, 2.0));
        assert!(is_in_propagation_corona(2.0, 2.0, 2.0));
        assert!(!is_in_propagation_corona(2.5, 2.0, 2.0));
    }

    #[test]
    fn is_in_propagation_corona_zero_radius_returns_false() {
        assert!(!is_in_propagation_corona(0.0, 0.0, 2.0));
    }

    #[test]
    fn is_in_propagation_corona_zero_speed_returns_false() {
        assert!(!is_in_propagation_corona(5.0, 10.0, 0.0));
    }

    // -- frequency_purity_at_distance --

    #[test]
    fn freq_purity_at_zero_distance_is_one() {
        approx_eq(frequency_purity_at_distance(0.0, 12.0), 1.0);
    }

    #[test]
    fn freq_purity_at_one_lambda_is_inv_e() {
        let expected = (-1.0_f32).exp();
        approx_eq(frequency_purity_at_distance(12.0, 12.0), expected);
    }

    #[test]
    fn freq_purity_decreases_monotonically() {
        let near = frequency_purity_at_distance(5.0, 12.0);
        let far  = frequency_purity_at_distance(20.0, 12.0);
        assert!(near > far, "near={near} far={far}");
    }

    #[test]
    fn freq_purity_far_approaches_zero() {
        assert!(frequency_purity_at_distance(200.0, 12.0) < 0.001);
    }

    #[test]
    fn freq_purity_zero_lambda_protected() {
        // lambda clamped to 0.001 — must not panic or return NaN
        let p = frequency_purity_at_distance(0.0, 0.0);
        assert!(p.is_finite() && p >= 0.0);
    }

    #[test]
    fn freq_purity_always_in_unit_range() {
        for d in [0.0_f32, 1.0, 5.0, 12.0, 50.0, 1000.0] {
            let p = frequency_purity_at_distance(d, 12.0);
            assert!((0.0..=1.0).contains(&p), "d={d} p={p}");
        }
    }

    // -- entrainment_coupling_at_distance --

    #[test]
    fn entrainment_coupling_at_distance_zero_is_base() {
        approx_eq(entrainment_coupling_at_distance(0.5, 0.0, 12.0), 0.5);
    }

    #[test]
    fn entrainment_coupling_decays_with_distance() {
        let close = entrainment_coupling_at_distance(1.0, 2.0,  12.0);
        let far   = entrainment_coupling_at_distance(1.0, 30.0, 12.0);
        assert!(close > far, "close={close} far={far}");
    }

    #[test]
    fn entrainment_coupling_negative_base_clamped_to_zero() {
        assert_eq!(entrainment_coupling_at_distance(-1.0, 0.0, 12.0), 0.0);
    }
}
