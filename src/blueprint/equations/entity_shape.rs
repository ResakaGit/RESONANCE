//! Pure equations for entity shape inference (Gaps 2–3 — MG-9/MG-10).
//!
//! Converts entity physics layers into a [`GeometryInfluence`] packet
//! consumed by the GF1 mesh pipeline in `entity_shape_inference_system`,
//! and computes bilateral quadruped body plan layouts for MOVE-capable entities.
//! No ECS reads; all inputs are scalars already extracted from components.

use crate::math_types::Vec3;

use crate::geometry_flow::GeometryInfluence;
use crate::blueprint::equations::{BranchRole, SymmetryMode};
use crate::layers::organ::MAX_ORGANS_PER_ENTITY;
use crate::layers::MatterState;

// ── Element frequency bands (Hz) ─────────────────────────────────────────────
const BAND_UMBRA:   f32 =   20.0;
const BAND_TERRA:   f32 =   75.0;
const BAND_AQUA:    f32 =  250.0;
const BAND_IGNIS:   f32 =  450.0;
const BAND_VENTUS:  f32 =  700.0;
const BAND_LUX:     f32 = 1000.0;

// ── Tint anchors: linear RGB per band ────────────────────────────────────────
const TINT_UMBRA:   [f32; 3] = [0.15, 0.00, 0.25];
const TINT_TERRA:   [f32; 3] = [0.42, 0.22, 0.05];
const TINT_AQUA:    [f32; 3] = [0.05, 0.25, 0.75];
const TINT_IGNIS:   [f32; 3] = [0.85, 0.28, 0.02];
const TINT_VENTUS:  [f32; 3] = [0.18, 0.65, 0.22];
const TINT_LUX:     [f32; 3] = [0.95, 0.80, 0.15];

// ── Resistance base values per MatterState ────────────────────────────────────
const RESIST_SOLID:  f32 = 1.20;
const RESIST_LIQUID: f32 = 0.60;
const RESIST_GAS:    f32 = 0.20;
const RESIST_PLASMA: f32 = 0.05;

const BOND_RESIST_SCALE: f32 = 0.08;

// ── Max segments for entity meshes ───────────────────────────────────────────
const ENTITY_MAX_SEGMENTS: u32 = 12;

// ── Minimum spine / radius guard values ──────────────────────────────────────
const MIN_LENGTH_BUDGET:   f32 = 0.05;
const MIN_RADIUS_BASE_FRAC: f32 = 0.04;

/// Maps element frequency Hz to linear RGB tint by interpolating 6 anchor bands.
///
/// Anchors: Umbra 20Hz, Terra 75Hz, Aqua 250Hz, Ignis 450Hz, Ventus 700Hz, Lux 1000Hz.
pub fn frequency_to_tint_rgb(hz: f32) -> [f32; 3] {
    let bands: [(f32, [f32; 3]); 6] = [
        (BAND_UMBRA,  TINT_UMBRA),
        (BAND_TERRA,  TINT_TERRA),
        (BAND_AQUA,   TINT_AQUA),
        (BAND_IGNIS,  TINT_IGNIS),
        (BAND_VENTUS, TINT_VENTUS),
        (BAND_LUX,    TINT_LUX),
    ];

    if hz <= bands[0].0 { return bands[0].1; }
    if hz >= bands[5].0 { return bands[5].1; }

    for i in 0..5 {
        let (lo_hz, lo_tint) = bands[i];
        let (hi_hz, hi_tint) = bands[i + 1];
        if hz <= hi_hz {
            let t = (hz - lo_hz) / (hi_hz - lo_hz).max(1e-6);
            return lerp_rgb(lo_tint, hi_tint, t);
        }
    }
    bands[5].1
}

/// Converts fineness_ratio + entity_radius into GF1 spine parameters.
///
/// Returns `(length_budget, radius_base)`.
/// Invariant: `length_budget / radius_base ≈ fineness²` (matches MG-4 shape optimizer).
pub fn fineness_to_spine_params(fineness_ratio: f32, entity_radius: f32) -> (f32, f32) {
    let f = fineness_ratio.max(0.1);
    let diameter = entity_radius * 2.0;
    let length  = (diameter * f).max(MIN_LENGTH_BUDGET);
    let r_base  = (diameter / f).max(entity_radius * MIN_RADIUS_BASE_FRAC);
    (length, r_base)
}

/// MatterState + bond_energy → GF1 resistance value in `[0.02, 2.5]`.
///
/// Solid is stiffest; Plasma is almost frictionless.
/// Bond energy contributes a log-scaled additive term within each band.
pub fn matter_to_gf1_resistance(bond_energy: f32, state: MatterState) -> f32 {
    let base = match state {
        MatterState::Solid  => RESIST_SOLID,
        MatterState::Liquid => RESIST_LIQUID,
        MatterState::Gas    => RESIST_GAS,
        MatterState::Plasma => RESIST_PLASMA,
    };
    let bond_add = (bond_energy.max(0.0).ln_1p()) * BOND_RESIST_SCALE;
    (base + bond_add).clamp(0.02, 2.5)
}

/// LOD detail `[0, 1]` from qe_norm and entity radius.
///
/// Larger, more energetic entities receive higher detail budgets.
pub fn entity_lod_detail(qe_norm: f32, radius: f32) -> f32 {
    let radius_factor = (radius / 2.0).clamp(0.0, 1.0);
    (qe_norm * 0.6 + radius_factor * 0.4).clamp(0.0, 1.0)
}

/// Assembles a [`GeometryInfluence`] from entity physics layer scalars.
///
/// `velocity_3d` drives energy_direction; `fineness_ratio` shapes the tube aspect ratio.
/// All heavy derivation is done by the caller (already extracted from ECS components).
pub fn entity_geometry_influence(
    world_pos: Vec3,
    qe_norm: f32,
    radius: f32,
    fineness_ratio: f32,
    resistance: f32,
    velocity_3d: Vec3,
    tint_rgb: [f32; 3],
    detail: f32,
) -> GeometryInfluence {
    let (length_budget, radius_base) = fineness_to_spine_params(fineness_ratio, radius);

    let speed = velocity_3d.length();
    let energy_direction = if speed > 1e-6 {
        velocity_3d / speed
    } else {
        Vec3::Y
    };

    // Lateral axis orthogonal to energy_direction — least resistance bends toward it.
    let side_hint = if energy_direction.dot(Vec3::X).abs() < 0.9 {
        Vec3::X
    } else {
        Vec3::Z
    };
    let least_resistance_direction = energy_direction.cross(side_hint).normalize_or(Vec3::X);

    GeometryInfluence {
        detail,
        energy_direction,
        energy_strength: speed,
        resistance,
        least_resistance_direction,
        length_budget,
        max_segments: ENTITY_MAX_SEGMENTS,
        radius_base,
        start_position: world_pos,
        qe_norm,
        tint_rgb,
        branch_role: BranchRole::Stem,
    }
}

// ── Shape cache signature constants ──────────────────────────────────────────
const FINENESS_SIG_MIN: f32 =  0.5;
const FINENESS_SIG_MAX: f32 =  8.0;
const RADIUS_SIG_MAX:   f32 =  4.0;
const FOOD_DIST_IMM:    f32 =  1.0;
const FOOD_DIST_NEAR:   f32 =  4.0;
const FOOD_DIST_FAR:    f32 = 16.0;

/// Compact `u16` encoding shape inference inputs for [`PerformanceCachePolicy`] invalidation.
///
/// Bit layout (16 bits):
///   `[15:12]` fineness_ratio — 4 bits, 16 buckets over `[0.5, 8.0]`
///   `[11:9]`  qe_norm        — 3 bits, 8 buckets over `[0, 1]`
///   `[8:6]`   radius         — 3 bits, 8 buckets over `[0, 4]`
///   `[5:4]`   hunger         — 2 bits, 4 buckets over `[0, 1]` (EnergyAssessment)
///   `[3:2]`   food proximity — 2 bits: 0=immediate<1, 1=near<4, 2=far<16, 3=none (SensoryAwareness)
///   `[1]`     has_hostile    — 1 bit (SensoryAwareness)
///   `[0]`     reserved
pub fn shape_cache_signature(
    fineness_ratio: f32,
    qe_norm: f32,
    radius: f32,
    hunger: f32,
    food_distance: f32,
    has_hostile: bool,
) -> u16 {
    let f = fineness_ratio.clamp(FINENESS_SIG_MIN, FINENESS_SIG_MAX);
    let fineness_b = ((f - FINENESS_SIG_MIN) / (FINENESS_SIG_MAX - FINENESS_SIG_MIN) * 15.0) as u16 & 0xF;
    let qe_b       = (qe_norm.clamp(0.0, 1.0) * 7.0) as u16 & 0x7;
    let r          = radius.clamp(0.0, RADIUS_SIG_MAX);
    let radius_b   = (r / RADIUS_SIG_MAX * 7.0) as u16 & 0x7;
    let hunger_b   = (hunger.clamp(0.0, 1.0) * 3.0) as u16 & 0x3;
    let food_b: u16 = if food_distance < FOOD_DIST_IMM  { 0 }
                      else if food_distance < FOOD_DIST_NEAR { 1 }
                      else if food_distance < FOOD_DIST_FAR  { 2 }
                      else                                    { 3 };
    let hostile_b  = has_hostile as u16;

    (fineness_b << 12) | (qe_b << 9) | (radius_b << 6) | (hunger_b << 4) | (food_b << 2) | (hostile_b << 1)
}

/// Computes bilateral quadruped attachment positions (head, tail, 4 legs) for MOVE entities.
///
/// Returns `(positions, directions, symmetry, active_count)` ready for `BodyPlanLayout::new`.
/// `mobility_bias` scales leg spread: low → compact stance, high → wide/dynamic stance.
pub fn bilateral_quadruped_attachments(
    radius: f32,
    mobility_bias: f32,
) -> ([Vec3; MAX_ORGANS_PER_ENTITY], [Vec3; MAX_ORGANS_PER_ENTITY], SymmetryMode, u8) {
    let r = radius.max(0.01);
    let spread = r * (0.4 + mobility_bias * 0.4).clamp(0.2, 0.8);
    let leg_y = -r * 0.7;
    let front_z = r * 0.3;
    let rear_z  = -r * 0.3;

    let mut positions  = [Vec3::ZERO; MAX_ORGANS_PER_ENTITY];
    let mut directions = [Vec3::Y;    MAX_ORGANS_PER_ENTITY];

    // Head (0) — forward
    positions[0]  = Vec3::new(0.0, 0.0, r * 0.65);
    directions[0] = Vec3::Z;
    // Tail (1) — backward
    positions[1]  = Vec3::new(0.0, 0.0, -r * 0.55);
    directions[1] = -Vec3::Z;
    // Front-left leg (2)
    positions[2]  = Vec3::new( spread, leg_y, front_z);
    directions[2] = Vec3::new( 0.3, -0.9, 0.0).normalize_or(Vec3::NEG_Y);
    // Front-right leg (3)
    positions[3]  = Vec3::new(-spread, leg_y, front_z);
    directions[3] = Vec3::new(-0.3, -0.9, 0.0).normalize_or(Vec3::NEG_Y);
    // Rear-left leg (4)
    positions[4]  = Vec3::new( spread, leg_y, rear_z);
    directions[4] = directions[2];
    // Rear-right leg (5)
    positions[5]  = Vec3::new(-spread, leg_y, rear_z);
    directions[5] = directions[3];

    (positions, directions, SymmetryMode::Bilateral, 6)
}

// ── Constructal shape inference ──────────────────────────────────────────────

/// Projected frontal area with N cylindrical appendages.
///
/// `A_proj = π r² + N × limb_length × 2 × r_limb`.
pub fn projected_area_with_limbs(
    body_radius: f32,
    limb_count: u8,
    limb_length: f32,
    limb_radius: f32,
) -> f32 {
    use std::f32::consts::PI;
    let body_area = PI * body_radius.max(0.0).powi(2);
    let limb_area = limb_count as f32 * limb_length.max(0.0) * 2.0 * limb_radius.max(0.0);
    body_area + limb_area
}

/// Scale factors for organ sub-meshes, parameterized by `mobility_bias`.
///
/// Returns `(length_factor, radius_factor)` per slot type.
/// High mobility → longer front limbs (arms), bigger head, longer tail.
/// Low mobility → uniform limbs, smaller head, shorter tail.
/// Front/rear asymmetry emerges from mobility: climbers get long arms, runners get equal legs.
pub fn organ_slot_scale(slot_index: usize, active_count: u8, mobility_bias: f32) -> (f32, f32) {
    let m = mobility_bias.clamp(0.0, 1.0);
    match slot_index {
        0 => (0.30 + m * 0.15, 0.55 + m * 0.25),                // head: bigger with mobility (proxy for neural cost)
        1 => (0.45 + m * 0.35, 0.12 + m * 0.12),                // tail: longer with mobility (balance organ)
        _ if slot_index >= active_count as usize => (0.0, 0.0),
        _ => {
            let limb_slots = active_count.saturating_sub(2) as usize;
            let half = limb_slots / 2;
            let is_front = slot_index < 2 + half;
            if is_front {
                (0.40 + m * 0.35, 0.22 + m * 0.12)              // front limbs: longer with mobility (arms)
            } else {
                (0.40 + (1.0 - m) * 0.15, 0.30 + (1.0 - m) * 0.15) // rear limbs: sturdier with low mobility (legs)
            }
        }
    }
}

/// Per-limb thrust factor: reduces effective drag.
/// `efficiency = 1 / (1 + LIMB_THRUST_FACTOR × N)`.
const LIMB_THRUST_FACTOR: f32 = 0.6;

/// Per-limb metabolic maintenance cost (qe/tick equivalent).
/// Counterbalances thrust benefit — keeps optimizer from always choosing max limbs.
const LIMB_MAINTENANCE_COST: f32 = 0.5;

/// Optimal appendage count by constructal cost minimization.
///
/// `cost(N) = drag(N) × thrust_efficiency(N) + maintenance(N)`
///   - drag(N) = ½ ρ v² C_D A_proj(N) — increases with N (more frontal area)
///   - thrust_efficiency(N) = 1 / (1 + 0.6 × N) — limbs distribute propulsive force
///   - maintenance(N) = LIMB_MAINTENANCE_COST × N — metabolic cost per limb
///
/// Returns N minimizing total cost; ties favor fewer limbs.
/// Zero velocity → 0 limbs (no locomotion demand).
pub fn optimal_appendage_count(
    body_radius: f32,
    fineness_ratio: f32,
    medium_density: f32,
    velocity: f32,
    _viscosity: f32,
    limb_length: f32,
    limb_radius: f32,
    max_limbs: u8,
) -> u8 {
    use crate::blueprint::constants::DRAG_SPEED_EPSILON;
    use crate::blueprint::morphogenesis::inferred_drag_coefficient;

    if velocity.abs() <= DRAG_SPEED_EPSILON { return 0; }

    let diameter = body_radius.max(0.01) * 2.0;
    let body_length = fineness_ratio.max(0.1) * diameter;
    let cd = inferred_drag_coefficient(body_length, diameter);
    let v = velocity.abs();

    let mut best_n: u8 = 0;
    let mut best_cost = f32::MAX;

    for n in 0..=max_limbs {
        let area = projected_area_with_limbs(body_radius, n, limb_length, limb_radius);
        let drag = 0.5 * medium_density.max(0.0) * v * v * cd * area;
        let efficiency = 1.0 / (1.0 + LIMB_THRUST_FACTOR * n as f32);
        let maintenance = LIMB_MAINTENANCE_COST * n as f32;
        let cost = drag * efficiency + maintenance;
        if cost < best_cost {
            best_cost = cost;
            best_n = n;
        }
    }
    best_n
}

#[inline]
fn lerp_rgb(a: [f32; 3], b: [f32; 3], t: f32) -> [f32; 3] {
    [
        a[0] + (b[0] - a[0]) * t,
        a[1] + (b[1] - a[1]) * t,
        a[2] + (b[2] - a[2]) * t,
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layers::MatterState;

    // ── frequency_to_tint_rgb ─────────────────────────────────────────────────

    #[test]
    fn tint_below_umbra_returns_umbra_anchor() {
        let t = frequency_to_tint_rgb(0.0);
        assert!((t[0] - TINT_UMBRA[0]).abs() < 1e-6);
        assert!((t[2] - TINT_UMBRA[2]).abs() < 1e-6);
    }

    #[test]
    fn tint_above_lux_returns_lux_anchor() {
        let t = frequency_to_tint_rgb(9999.0);
        assert!((t[0] - TINT_LUX[0]).abs() < 1e-6);
    }

    #[test]
    fn tint_at_exact_band_anchors_matches() {
        let terra = frequency_to_tint_rgb(BAND_TERRA);
        assert!((terra[0] - TINT_TERRA[0]).abs() < 1e-5, "terra R");
        assert!((terra[1] - TINT_TERRA[1]).abs() < 1e-5, "terra G");
        assert!((terra[2] - TINT_TERRA[2]).abs() < 1e-5, "terra B");

        let aqua = frequency_to_tint_rgb(BAND_AQUA);
        assert!((aqua[2] - TINT_AQUA[2]).abs() < 1e-5, "aqua B");
    }

    #[test]
    fn tint_midpoint_is_interpolated() {
        let mid_hz = (BAND_TERRA + BAND_AQUA) * 0.5;
        let t = frequency_to_tint_rgb(mid_hz);
        let expected_r = (TINT_TERRA[0] + TINT_AQUA[0]) * 0.5;
        assert!((t[0] - expected_r).abs() < 1e-5, "midpoint R = {}", t[0]);
    }

    #[test]
    fn tint_all_bands_produce_unit_interval_rgb() {
        let bands = [BAND_UMBRA, BAND_TERRA, BAND_AQUA, BAND_IGNIS, BAND_VENTUS, BAND_LUX];
        for hz in bands {
            let t = frequency_to_tint_rgb(hz);
            for ch in t {
                assert!((0.0..=1.0).contains(&ch), "channel {ch} out of [0,1] for hz={hz}");
            }
        }
    }

    // ── fineness_to_spine_params ──────────────────────────────────────────────

    #[test]
    fn fineness_invariant_length_over_radius_approx_fineness_squared() {
        let f = 4.0_f32;
        let r = 1.0_f32;
        let (length, radius_base) = fineness_to_spine_params(f, r);
        let ratio = length / radius_base;
        assert!((ratio - f * f).abs() < 1e-3, "ratio={ratio} fineness²={}", f * f);
    }

    #[test]
    fn fineness_sphere_gives_equal_length_and_diameter() {
        // fineness=1 → length ≈ diameter, radius_base ≈ radius
        let (length, rb) = fineness_to_spine_params(1.0, 0.5);
        assert!((length - 1.0).abs() < 1e-6, "length={length}");
        assert!((rb - 1.0).abs() < 1e-6, "radius_base={rb}");
    }

    #[test]
    fn fineness_near_zero_does_not_panic() {
        let (l, r) = fineness_to_spine_params(0.0, 1.0);
        assert!(l >= MIN_LENGTH_BUDGET);
        assert!(r > 0.0);
    }

    // ── matter_to_gf1_resistance ──────────────────────────────────────────────

    #[test]
    fn resistance_solid_gt_liquid_gt_gas_gt_plasma() {
        let bond = 500.0;
        let r_solid  = matter_to_gf1_resistance(bond, MatterState::Solid);
        let r_liquid = matter_to_gf1_resistance(bond, MatterState::Liquid);
        let r_gas    = matter_to_gf1_resistance(bond, MatterState::Gas);
        let r_plasma = matter_to_gf1_resistance(bond, MatterState::Plasma);
        assert!(r_solid > r_liquid, "solid > liquid");
        assert!(r_liquid > r_gas,   "liquid > gas");
        assert!(r_gas > r_plasma,   "gas > plasma");
    }

    #[test]
    fn resistance_clamped_to_bounds() {
        let lo = matter_to_gf1_resistance(0.0, MatterState::Plasma);
        let hi = matter_to_gf1_resistance(f32::MAX, MatterState::Solid);
        assert!(lo >= 0.02);
        assert!(hi <= 2.5);
    }

    #[test]
    fn resistance_higher_bond_increases_value() {
        let r_lo = matter_to_gf1_resistance(100.0, MatterState::Liquid);
        let r_hi = matter_to_gf1_resistance(5000.0, MatterState::Liquid);
        assert!(r_hi >= r_lo, "higher bond should not decrease resistance");
    }

    // ── entity_lod_detail ─────────────────────────────────────────────────────

    #[test]
    fn lod_detail_returns_unit_interval() {
        for &qe in &[0.0_f32, 0.5, 1.0, 2.0] {
            for &r in &[0.0_f32, 0.5, 1.0, 5.0] {
                let d = entity_lod_detail(qe, r);
                assert!((0.0..=1.0).contains(&d), "d={d} qe={qe} r={r}");
            }
        }
    }

    #[test]
    fn lod_detail_higher_qe_not_lower_detail() {
        let d_lo = entity_lod_detail(0.1, 0.5);
        let d_hi = entity_lod_detail(0.9, 0.5);
        assert!(d_hi >= d_lo);
    }

    // ── entity_geometry_influence ─────────────────────────────────────────────

    #[test]
    fn influence_zero_velocity_uses_y_direction() {
        let inf = entity_geometry_influence(
            Vec3::ZERO, 0.5, 0.5, 2.0, 0.8, Vec3::ZERO, [0.5; 3], 0.5,
        );
        assert!((inf.energy_direction - Vec3::Y).length() < 1e-6);
    }

    #[test]
    fn influence_nonzero_velocity_normalizes_direction() {
        let vel = Vec3::new(3.0, 0.0, 0.0);
        let inf = entity_geometry_influence(
            Vec3::ZERO, 0.5, 0.5, 2.0, 0.8, vel, [0.5; 3], 0.5,
        );
        assert!((inf.energy_direction.length() - 1.0).abs() < 1e-6);
        assert!((inf.energy_direction - Vec3::X).length() < 1e-6);
    }

    #[test]
    fn influence_fields_match_inputs() {
        let pos = Vec3::new(1.0, 2.0, 3.0);
        let tint = [0.3, 0.6, 0.9];
        let inf = entity_geometry_influence(pos, 0.7, 1.0, 3.0, 1.1, Vec3::ZERO, tint, 0.8);
        assert_eq!(inf.start_position, pos);
        assert_eq!(inf.tint_rgb, tint);
        assert!((inf.qe_norm - 0.7).abs() < 1e-6);
        assert!((inf.resistance - 1.1).abs() < 1e-6);
        assert!((inf.detail - 0.8).abs() < 1e-6);
    }

    #[test]
    fn influence_least_resistance_orthogonal_to_energy_direction() {
        let vel = Vec3::new(0.0, 1.0, 0.0);
        let inf = entity_geometry_influence(
            Vec3::ZERO, 0.5, 0.5, 2.0, 0.5, vel, [0.5; 3], 0.5,
        );
        let dot = inf.energy_direction.dot(inf.least_resistance_direction).abs();
        assert!(dot < 1e-5, "not orthogonal: dot={dot}");
    }

    // ── shape_cache_signature ─────────────────────────────────────────────────

    #[test]
    fn signature_same_inputs_gives_same_output() {
        let a = shape_cache_signature(2.0, 0.5, 1.0, 0.3, 8.0, false);
        let b = shape_cache_signature(2.0, 0.5, 1.0, 0.3, 8.0, false);
        assert_eq!(a, b);
    }

    #[test]
    fn signature_hostile_bit_changes_output() {
        let no_hostile = shape_cache_signature(2.0, 0.5, 1.0, 0.0, f32::MAX, false);
        let hostile    = shape_cache_signature(2.0, 0.5, 1.0, 0.0, f32::MAX, true);
        assert_ne!(no_hostile, hostile);
    }

    #[test]
    fn signature_fineness_bucket_change_changes_output() {
        // 0.5 and 8.0 map to bucket 0 and 15 respectively
        let lo = shape_cache_signature(0.5, 0.5, 1.0, 0.0, f32::MAX, false);
        let hi = shape_cache_signature(8.0, 0.5, 1.0, 0.0, f32::MAX, false);
        assert_ne!(lo, hi);
    }

    #[test]
    fn signature_hunger_bucket_change_changes_output() {
        let sated  = shape_cache_signature(2.0, 0.5, 1.0, 0.0,  f32::MAX, false);
        let hungry = shape_cache_signature(2.0, 0.5, 1.0, 0.99, f32::MAX, false);
        assert_ne!(sated, hungry);
    }

    #[test]
    fn signature_food_proximity_buckets_differ() {
        let imm  = shape_cache_signature(2.0, 0.5, 1.0, 0.0, 0.5,      false); // <1
        let near = shape_cache_signature(2.0, 0.5, 1.0, 0.0, 2.0,      false); // <4
        let far  = shape_cache_signature(2.0, 0.5, 1.0, 0.0, 8.0,      false); // <16
        let none = shape_cache_signature(2.0, 0.5, 1.0, 0.0, f32::MAX, false); // >=16
        assert_ne!(imm, near);
        assert_ne!(near, far);
        assert_ne!(far, none);
    }

    #[test]
    fn signature_extreme_inputs_do_not_panic() {
        let _ = shape_cache_signature(f32::MAX, f32::MAX, f32::MAX, f32::MAX, f32::MAX, true);
        let _ = shape_cache_signature(-f32::MAX, -1.0, -1.0, -1.0, 0.0, false);
    }

    #[test]
    fn signature_fits_in_u16() {
        // All 16 fineness buckets × hostile on/off — just verify no overflow
        for i in 0u16..16 {
            let f = FINENESS_SIG_MIN + (FINENESS_SIG_MAX - FINENESS_SIG_MIN) * (i as f32 / 15.0);
            let sig = shape_cache_signature(f, 0.5, 1.0, 0.5, 5.0, i % 2 == 0);
            let _ = sig; // type is u16, overflow is a compile-time concern
        }
    }

    #[test]
    fn full_gf1_pipeline_produces_mesh_with_triangles() {
        use crate::geometry_flow::{build_flow_mesh, build_flow_spine};
        let inf = entity_geometry_influence(
            Vec3::ZERO, 0.6, 0.5, 2.5, 0.8, Vec3::new(0.1, 1.0, 0.0), TINT_TERRA, 0.5,
        );
        let spine = build_flow_spine(&inf);
        assert!(spine.len() >= 2, "spine needs at least 2 nodes");
        let mesh = build_flow_mesh(&spine, &inf);
        use crate::geometry_flow::flow_mesh_triangle_count;
        let tris = flow_mesh_triangle_count(&mesh);
        assert!(tris > 0, "mesh should have triangles");
    }

    // ── projected_area_with_limbs ───────────────────────────────────────────

    #[test]
    fn projected_area_zero_limbs_equals_circle() {
        use std::f32::consts::PI;
        let r = 1.0;
        let area = projected_area_with_limbs(r, 0, 0.8, 0.15);
        assert!((area - PI * r * r).abs() < 1e-6);
    }

    #[test]
    fn projected_area_increases_with_limbs() {
        let a0 = projected_area_with_limbs(1.0, 0, 0.8, 0.15);
        let a2 = projected_area_with_limbs(1.0, 2, 0.8, 0.15);
        let a4 = projected_area_with_limbs(1.0, 4, 0.8, 0.15);
        assert!(a2 > a0, "2 limbs > 0 limbs");
        assert!(a4 > a2, "4 limbs > 2 limbs");
    }

    #[test]
    fn projected_area_negative_inputs_do_not_panic() {
        let a = projected_area_with_limbs(-1.0, 4, -0.5, -0.1);
        assert!(a >= 0.0);
    }

    // ── optimal_appendage_count ─────────────────────────────────────────────

    #[test]
    fn optimal_appendage_zero_velocity_returns_zero() {
        let n = optimal_appendage_count(0.5, 2.0, 1.0, 0.0, 0.1, 0.4, 0.08, 8);
        assert_eq!(n, 0, "at rest, vascular cost dominates → zero limbs");
    }

    #[test]
    fn optimal_appendage_high_velocity_returns_nonzero() {
        let n = optimal_appendage_count(0.5, 2.0, 1.0, 10.0, 0.1, 0.4, 0.08, 8);
        assert!(n > 0, "high velocity should favor limbs for thrust, got {n}");
    }

    #[test]
    fn optimal_appendage_never_exceeds_max() {
        for v in [0.0, 1.0, 5.0, 50.0, 500.0] {
            let n = optimal_appendage_count(1.0, 3.0, 1.0, v, 0.1, 0.8, 0.15, 6);
            assert!(n <= 6, "exceeded max_limbs=6 at v={v}, got {n}");
        }
    }

    #[test]
    fn optimal_appendage_monotonic_over_velocity_sweep() {
        let mut prev = 0u8;
        for v_int in 0..=20 {
            let v = v_int as f32 * 2.5;
            let n = optimal_appendage_count(0.5, 2.0, 1.0, v, 0.1, 0.4, 0.08, 8);
            assert!(n >= prev, "limb count should not decrease as velocity grows: v={v} n={n} prev={prev}");
            prev = n;
        }
    }

    #[test]
    fn optimal_appendage_extreme_inputs_do_not_panic() {
        let _ = optimal_appendage_count(f32::MAX, f32::MAX, f32::MAX, f32::MAX, 0.1, 0.4, 0.08, 8);
        let _ = optimal_appendage_count(0.001, 0.1, 0.001, 0.001, 0.001, 0.001, 0.001, 0);
    }
}
