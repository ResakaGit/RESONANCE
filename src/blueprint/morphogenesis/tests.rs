use super::*;
use crate::blueprint::constants::morphogenesis as mg;
use crate::blueprint::constants::DIVISION_GUARD_EPSILON;

// Compile-time: flujo casi nulo usa eps más estricto que el piso genérico.
const _: () = assert!(mg::ALBEDO_IRRADIANCE_FLUX_EPS < DIVISION_GUARD_EPSILON);

// ── MG-1A: Carnot ────────────────────────────────────────────────────

#[test]
fn carnot_efficiency_standard_gradient_approx_04() {
    assert!((carnot_efficiency(500.0, 300.0) - 0.4).abs() < 1e-5);
}

#[test]
fn carnot_efficiency_no_gradient_returns_zero() {
    assert_eq!(carnot_efficiency(300.0, 300.0), 0.0);
}

#[test]
fn carnot_efficiency_inverted_gradient_returns_zero() {
    assert_eq!(carnot_efficiency(300.0, 500.0), 0.0);
}

#[test]
fn carnot_efficiency_nan_returns_zero() {
    assert_eq!(carnot_efficiency(f32::NAN, 300.0), 0.0);
    assert_eq!(carnot_efficiency(400.0, f32::NAN), 0.0);
}

#[test]
fn carnot_efficiency_negative_temps_return_zero() {
    assert_eq!(carnot_efficiency(-10.0, 200.0), 0.0);
    assert_eq!(carnot_efficiency(400.0, -1.0), 0.0);
}

#[test]
fn carnot_efficiency_output_strictly_below_one() {
    let eta = carnot_efficiency(1_000_000.0, 1.0);
    assert!(eta < 1.0, "η must be strictly < 1.0: {eta}");
}

// ── MG-1A: Entropía ──────────────────────────────────────────────────

#[test]
fn entropy_production_standard_value() {
    assert!((entropy_production(100.0, 500.0) - 0.2).abs() < 1e-5);
}

#[test]
fn entropy_production_negative_q_clamped_to_zero() {
    assert_eq!(entropy_production(-5.0, 500.0), 0.0);
}

#[test]
fn entropy_production_nan_t_returns_zero() {
    assert_eq!(entropy_production(10.0, f32::NAN), 0.0);
}

#[test]
fn entropy_production_negative_t_returns_zero() {
    assert_eq!(entropy_production(10.0, -50.0), 0.0);
}

// ── MG-1A: Exergía ───────────────────────────────────────────────────

#[test]
fn exergy_balance_standard_value() {
    assert!((exergy_balance(100.0, 0.5, 10.0) - 40.0).abs() < 1e-5);
}

#[test]
fn exergy_balance_insufficient_input_returns_zero() {
    assert_eq!(exergy_balance(10.0, 0.5, 10.0), 0.0);
}

#[test]
fn exergy_balance_nan_input_returns_zero() {
    assert_eq!(exergy_balance(f32::NAN, 0.5, 10.0), 0.0);
}

// ── MG-1A: Capacidad calorífica ──────────────────────────────────────

#[test]
fn heat_capacity_standard_value() {
    assert!((heat_capacity(100.0, mg::SPECIFIC_HEAT_FACTOR) - 1.0).abs() < 1e-6);
}

#[test]
fn heat_capacity_zero_qe_returns_zero() {
    assert_eq!(heat_capacity(0.0, mg::SPECIFIC_HEAT_FACTOR), 0.0);
}

// ── MG-1B: Shape cost ────────────────────────────────────────────────

#[test]
fn shape_cost_zero_velocity_equals_vascular_only() {
    let cv = 12.0_f32;
    assert!((shape_cost(1000.0, 0.0, 0.04, 3.14, cv) - cv).abs() < 1e-5);
}

#[test]
fn shape_cost_grows_quadratically_with_velocity() {
    let c1 = shape_cost(2.0, 1.0, 1.0, 1.0, 0.0);
    let c2 = shape_cost(2.0, 2.0, 1.0, 1.0, 0.0);
    let ratio = c2 / c1;
    assert!((ratio - 4.0).abs() < 0.02, "drag ~v²: ratio={ratio}");
}

#[test]
fn shape_cost_nan_inputs_return_finite() {
    let c = shape_cost(f32::NAN, 1.0, 0.5, 1.0, 0.0);
    assert!(c.is_finite(), "NaN density should sanitize to 0");
}

// ── MG-1B: Vascular transport ────────────────────────────────────────

#[test]
fn vascular_transport_cost_grows_with_length_cubed() {
    let short = vascular_transport_cost(1.0, 1.0, 1.0);
    let long  = vascular_transport_cost(1.0, 2.0, 1.0);
    assert!((long / short - 8.0).abs() < 1e-4);
}

#[test]
fn vascular_transport_cost_decreases_with_radius_fourth_power() {
    let narrow = vascular_transport_cost(1.0, 1.0, 0.5);
    let wide   = vascular_transport_cost(1.0, 1.0, 1.0);
    assert!(narrow > wide);
    let ratio = narrow / wide;
    assert!((ratio - 16.0).abs() < 1e-3, "r⁴ scaling: ratio={ratio}");
}

#[test]
fn vascular_transport_cost_overflow_returns_finite() {
    let c = vascular_transport_cost(1.0, 1e13, 0.001);
    assert!(c.is_finite(), "L³ overflow should be guarded: {c}");
}

#[test]
fn vascular_transport_cost_nan_returns_finite() {
    let c = vascular_transport_cost(f32::NAN, 1.0, 1.0);
    assert!(c.is_finite());
}

// ── MG-1B: Drag coefficient ─────────────────────────────────────────

#[test]
fn drag_coefficient_fusiform_less_than_chunky() {
    let fusiforme = inferred_drag_coefficient(10.0, 2.0);
    let chunky    = inferred_drag_coefficient(2.0, 2.0);
    assert!(fusiforme < chunky);
}

#[test]
fn drag_coefficient_always_in_valid_range() {
    let lens  = [0.05_f32, 0.2, 0.8, 1.0, 3.0, 12.0, 80.0];
    let diams = [0.05_f32, 0.2, 0.5, 1.0, 2.0, 6.0];
    for &len in &lens {
        for &diam in &diams {
            let cd = inferred_drag_coefficient(len, diam);
            assert!(
                cd >= mg::DRAG_COEFF_MIN && cd <= mg::DRAG_COEFF_BASE,
                "cd={cd} len={len} diam={diam}",
            );
        }
    }
}

#[test]
fn drag_coefficient_nan_returns_finite_in_range() {
    let cd = inferred_drag_coefficient(f32::NAN, 2.0);
    assert!(cd >= mg::DRAG_COEFF_MIN && cd <= mg::DRAG_COEFF_BASE);
}

// ── MG-1C: Albedo ────────────────────────────────────────────────────

#[test]
fn albedo_hot_creature_reflects_more_than_cool() {
    let em = mg::DEFAULT_EMISSIVITY;
    let h  = mg::DEFAULT_CONVECTION_COEFF;
    let hot  = inferred_albedo(85_000.0, 1400.0, 8.0, em, 400.0, 280.0, 12.0, h);
    let cool = inferred_albedo(2_000.0,  1400.0, 8.0, em, 400.0, 280.0, 12.0, h);
    assert!(hot > cool, "hot={hot} cool={cool}");
    assert!((hot - mg::ALBEDO_MAX).abs() < 0.08, "hot={hot}");
}

#[test]
fn albedo_cave_creature_near_minimum() {
    let em = mg::DEFAULT_EMISSIVITY;
    let h  = mg::DEFAULT_CONVECTION_COEFF;
    let cave = inferred_albedo(0.0, 12.0, 2.0, em, 500.0, 200.0, 28.0, h);
    assert!((cave - mg::ALBEDO_MIN).abs() < 0.02, "cave={cave}");
}

#[test]
fn albedo_no_solar_returns_fallback() {
    let em = mg::DEFAULT_EMISSIVITY;
    let h  = mg::DEFAULT_CONVECTION_COEFF;
    assert_eq!(inferred_albedo(100.0, 0.0, 8.0, em, 400.0, 280.0, 12.0, h), mg::ALBEDO_FALLBACK);
    assert_eq!(inferred_albedo(50.0, 800.0, 0.0, em, 400.0, 280.0, 12.0, h), mg::ALBEDO_FALLBACK);
}

#[test]
fn albedo_radiative_balance_self_consistency() {
    let em = mg::DEFAULT_EMISSIVITY;
    let h  = mg::DEFAULT_CONVECTION_COEFF;
    let (tc, te, a_s, a_p, i_b) = (310.0_f32, 305.0, 8.0, 4.0, 50.0);
    let q_d  = surface_dissipation_power(em, tc, te, a_s, h);
    let flux = i_b * a_p;
    let q_m  = q_d - 0.5 * flux;
    let alpha = inferred_albedo(q_m, i_b, a_p, em, tc, te, a_s, h);
    let lhs = q_m + (1.0 - alpha) * flux;
    assert!(
        (lhs - q_d).abs() < 2.0_f32.max(1e-3 * q_d.abs()),
        "balance: lhs={lhs} q_d={q_d} alpha={alpha}",
    );
}

#[test]
fn albedo_always_in_valid_range() {
    let em = mg::DEFAULT_EMISSIVITY;
    let h  = mg::DEFAULT_CONVECTION_COEFF;
    for i in 0..25 {
        let a = inferred_albedo(
            300.0 + i as f32 * 400.0,
            20.0 + i as f32 * 30.0,
            1.5 + 0.1 * i as f32,
            em, 330.0, 295.0,
            6.0 + i as f32,
            h,
        );
        assert!(a >= mg::ALBEDO_MIN && a <= mg::ALBEDO_MAX, "a={a} i={i}");
    }
}

// ── MG-1C: Surface dissipation ───────────────────────────────────────

#[test]
fn surface_dissipation_extreme_temp_stays_finite_and_positive() {
    // T^4 overflow a ~136 000 sin clamp; con clamp debe ser finito y positivo.
    let p = surface_dissipation_power(0.9, 200_000.0, 280.0, 10.0, 10.0);
    assert!(p.is_finite(), "extreme T must not overflow: {p}");
    assert!(p > 0.0, "hot body must dissipate: {p}");
}

#[test]
fn surface_dissipation_nan_t_core_returns_finite() {
    let p = surface_dissipation_power(0.9, f32::NAN, 280.0, 10.0, 10.0);
    assert!(p.is_finite(), "NaN t_core sanitized to 0 → finite result: {p}");
}

#[test]
fn surface_dissipation_zero_area_returns_zero() {
    let p = surface_dissipation_power(0.9, 400.0, 280.0, 0.0, 10.0);
    assert_eq!(p, 0.0);
}

// ── MG-1D: Rugosidad ────────────────────────────────────────────────

#[test]
fn rugosity_low_q_near_minimum() {
    let r = inferred_surface_rugosity(8.0, 10.0, 315.0, 300.0, mg::DEFAULT_CONVECTION_COEFF);
    assert!((r - mg::RUGOSITY_MIN).abs() < 0.12, "low Q → smooth: {r}");
}

#[test]
fn rugosity_high_q_small_volume_above_two() {
    let r = inferred_surface_rugosity(120_000.0, 1.5, 360.0, 300.0, mg::DEFAULT_CONVECTION_COEFF);
    assert!(r > 2.0, "high Q, small V → finned: {r}");
}

#[test]
fn rugosity_zero_delta_t_hits_maximum() {
    let r = inferred_surface_rugosity(50.0, 6.0, 300.0, 300.0, mg::DEFAULT_CONVECTION_COEFF);
    assert!((r - mg::RUGOSITY_MAX).abs() < 1e-3, "ΔT=0 → max: {r}");
}

#[test]
fn rugosity_always_in_valid_range() {
    for q in [0.0_f32, 30.0, 5000.0, 50_000.0] {
        let r = inferred_surface_rugosity(q, 4.0, 320.0, 300.0, 12.0);
        assert!(r >= mg::RUGOSITY_MIN && r <= mg::RUGOSITY_MAX, "r={r} q={q}");
    }
}

#[test]
fn rugosity_nan_q_returns_minimum() {
    let r = inferred_surface_rugosity(f32::NAN, 4.0, 320.0, 300.0, 10.0);
    assert!((r - mg::RUGOSITY_MIN).abs() < 0.1, "NaN Q → low rugosity: {r}");
}

// ── MG-4B: bounded_fineness_descent ─────────────────────────────────

#[test]
fn descent_water_dense_pushes_fusiform() {
    let f = bounded_fineness_descent(1.5, 1000.0, 4.0, 3.14, 12.0, 0.3, 3);
    assert!(f > 1.5, "ρ=1000, v=4 → fusiform: f={f}");
}

#[test]
fn descent_air_light_minimal_change() {
    let f = bounded_fineness_descent(1.5, 1.2, 0.5, 3.14, 12.0, 0.3, 3);
    assert!(
        (f - 1.5).abs() < 0.3,
        "ρ=1.2, v=0.5 → minimal pressure: f={f}",
    );
}

#[test]
fn descent_ceiling_stays_clamped() {
    let f = bounded_fineness_descent(8.0, 1000.0, 4.0, 3.14, 12.0, 0.3, 3);
    assert!(
        (f - mg::FINENESS_MAX).abs() < 1e-3,
        "already at max: f={f}",
    );
}

#[test]
fn descent_floor_pushed_away_under_pressure() {
    let f = bounded_fineness_descent(1.0, 1000.0, 4.0, 3.14, 12.0, 0.3, 3);
    assert!(f > 1.0, "pressure pushes away from sphere: f={f}");
}

#[test]
fn descent_deterministic() {
    let a = bounded_fineness_descent(1.5, 1000.0, 4.0, 3.14, 12.0, 0.3, 3);
    let b = bounded_fineness_descent(1.5, 1000.0, 4.0, 3.14, 12.0, 0.3, 3);
    assert_eq!(a, b, "same inputs → same output");
}

#[test]
fn descent_zero_velocity_no_significant_change() {
    let f = bounded_fineness_descent(1.5, 1000.0, 0.0, 3.14, 12.0, 0.3, 3);
    assert!(
        (f - 1.5).abs() < 0.3,
        "v=0 → no drag gradient: f={f}",
    );
}

#[test]
fn descent_nan_inputs_return_finite() {
    let f = bounded_fineness_descent(f32::NAN, 1000.0, 4.0, 3.14, 12.0, 0.3, 3);
    assert!(f.is_finite() && f >= mg::FINENESS_MIN && f <= mg::FINENESS_MAX);
}

#[test]
fn descent_max_iter_zero_returns_clamped_input() {
    let f = bounded_fineness_descent(3.0, 1000.0, 4.0, 3.14, 12.0, 0.3, 0);
    assert!((f - 3.0).abs() < 1e-6, "0 iterations → no change: f={f}");
}

// ── MG-5B: irradiance_effective_for_albedo ──────────────────────────

#[test]
fn irradiance_effective_standard_product() {
    assert!((irradiance_effective_for_albedo(50.0, 0.8) - 40.0).abs() < 1e-6);
}

#[test]
fn irradiance_effective_zero_photon_returns_zero() {
    assert_eq!(irradiance_effective_for_albedo(0.0, 0.8), 0.0);
}

#[test]
fn irradiance_effective_zero_absorbed_returns_zero() {
    assert_eq!(irradiance_effective_for_albedo(50.0, 0.0), 0.0);
}

#[test]
fn irradiance_effective_negative_photon_returns_zero() {
    assert_eq!(irradiance_effective_for_albedo(-5.0, 0.8), 0.0);
}

#[test]
fn irradiance_effective_nan_returns_zero() {
    assert_eq!(irradiance_effective_for_albedo(f32::NAN, 0.8), 0.0);
    assert_eq!(irradiance_effective_for_albedo(50.0, f32::NAN), 0.0);
}

// ── MG-5E: albedo_luminosity_blend ──────────────────────────────────

#[test]
fn luminosity_blend_low_albedo_darkens() {
    let result = albedo_luminosity_blend(1.0, 0.05);
    assert!((result - 0.335).abs() < 1e-3, "result={result}");
}

#[test]
fn luminosity_blend_high_albedo_brightens() {
    let result = albedo_luminosity_blend(1.0, 0.95);
    assert!((result - 0.965).abs() < 1e-3, "result={result}");
}

#[test]
fn luminosity_blend_no_albedo_component_preserves_base() {
    // Fallback albedo (0.5) → factor = 0.3 + 0.35 = 0.65
    let result = albedo_luminosity_blend(1.0, mg::ALBEDO_FALLBACK);
    assert!((result - 0.65).abs() < 1e-3, "result={result}");
}

#[test]
fn luminosity_blend_clamps_out_of_range_albedo() {
    // Negative → clamped to ALBEDO_MIN (0.05)
    let lo = albedo_luminosity_blend(1.0, -1.0);
    assert!((lo - 0.335).abs() < 1e-3, "lo={lo}");
    // Over 1.0 → clamped to ALBEDO_MAX (0.95)
    let hi = albedo_luminosity_blend(1.0, 2.0);
    assert!((hi - 0.965).abs() < 1e-3, "hi={hi}");
}

// ── MG-7C: rugosity_to_detail_multiplier ──────────────────────────

#[test]
fn detail_multiplier_at_minimum_is_one() {
    assert!((rugosity_to_detail_multiplier(1.0) - 1.0).abs() < 1e-6);
}

#[test]
fn detail_multiplier_at_1_5_boundary_is_one() {
    assert!((rugosity_to_detail_multiplier(1.5) - 1.0).abs() < 1e-6);
}

#[test]
fn detail_multiplier_at_2_0_is_1_25() {
    assert!((rugosity_to_detail_multiplier(2.0) - 1.25).abs() < 1e-6);
}

#[test]
fn detail_multiplier_at_2_5_is_1_5() {
    assert!((rugosity_to_detail_multiplier(2.5) - 1.5).abs() < 1e-6);
}

#[test]
fn detail_multiplier_at_maximum_approx_2() {
    let m = rugosity_to_detail_multiplier(4.0);
    // 1.5 + (4.0 - 2.5) * 0.33 = 1.5 + 0.495 = 1.995
    assert!((m - 1.995).abs() < 1e-3, "m={m}");
}

#[test]
fn detail_multiplier_monotonic_increasing() {
    let samples: Vec<f32> = (0..=300).map(|i| 1.0 + i as f32 * 0.01).collect();
    for w in samples.windows(2) {
        let lo = rugosity_to_detail_multiplier(w[0]);
        let hi = rugosity_to_detail_multiplier(w[1]);
        assert!(hi >= lo, "monotonicity: r={} → {}, r={} → {}", w[0], lo, w[1], hi);
    }
}

#[test]
fn detail_multiplier_clamps_below_min() {
    assert!((rugosity_to_detail_multiplier(0.0) - 1.0).abs() < 1e-6);
}

#[test]
fn detail_multiplier_clamps_above_max() {
    let at_max = rugosity_to_detail_multiplier(mg::RUGOSITY_MAX);
    let above  = rugosity_to_detail_multiplier(6.0);
    assert!((at_max - above).abs() < 1e-6, "clamped above: at_max={at_max}, above={above}");
}
