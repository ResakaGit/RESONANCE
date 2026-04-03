//! 2D radial energy field equations — emergent bilateral morphology.
//!
//! 8 axial stations × 4 radial sectors = 32 nodes per entity.
//! Peaks = appendages. Valleys = joints. Symmetry from isotropic init.
//! Zero labels. Zero templates. 100% axiom-derived.
//!
//! Axiom 1: all nodes are qe. Axiom 4: diffusion = entropy.
//! Axiom 6: peaks emerge, not programmed. Axiom 7: adjacent only.

/// Axial stations along body axis (16 for complex morphology).
pub const AXIAL: usize = 16;
/// Radial sectors around body axis (8 for fine bilateral resolution).
pub const RADIAL: usize = 8;
/// Maximum detectable peaks.
pub const MAX_PEAKS: usize = 16;

/// 2D radial field: `[axial][radial]`.
pub type RadialField = [[f32; RADIAL]; AXIAL];

// ─── Totals ─────────────────────────────────────────────────────────────────

/// Sum all 32 nodes.
#[inline]
pub fn radial_total(field: &RadialField) -> f32 {
    field.iter().flat_map(|row| row.iter()).sum()
}

/// Maximum absolute change between two field states.
#[inline]
pub fn radial_max_delta(a: &RadialField, b: &RadialField) -> f32 {
    let mut max = 0.0_f32;
    for ax in 0..AXIAL {
        for rad in 0..RADIAL {
            let d = (a[ax][rad] - b[ax][rad]).abs();
            if d > max {
                max = d;
            }
        }
    }
    max
}

/// Check if field has converged: max delta < epsilon.
#[inline]
pub fn radial_converged(before: &RadialField, after: &RadialField, epsilon: f32) -> bool {
    radial_max_delta(before, after) < epsilon
}

// ─── Diffusion ──────────────────────────────────────────────────────────────

/// 2D diffusion: axial (i±1, same sector) + radial (same i, sector±1 mod 4).
///
/// Conservation guaranteed: each transfer subtracts from source, adds to target.
/// Axiom 7: adjacent only. Axiom 4: energy spreads (entropy increases).
pub fn radial_diffuse(field: &RadialField, conductivity: f32, dt: f32) -> RadialField {
    let mut out = *field;
    let k = conductivity.clamp(0.0, 1.0) * dt;

    // Axial diffusion (same sector, adjacent stations)
    for r in 0..RADIAL {
        for a in 0..(AXIAL - 1) {
            let delta = (out[a][r] - out[a + 1][r]) * k * 0.25;
            let safe = delta.clamp(-out[a + 1][r] * 0.25, out[a][r] * 0.25);
            out[a][r] -= safe;
            out[a + 1][r] += safe;
        }
    }

    // Radial diffusion (same station, adjacent sectors, wrapping)
    for a in 0..AXIAL {
        for r in 0..RADIAL {
            let rn = (r + 1) % RADIAL;
            let delta = (out[a][r] - out[a][rn]) * k * 0.25;
            let safe = delta.clamp(-out[a][rn] * 0.25, out[a][r] * 0.25);
            out[a][r] -= safe;
            out[a][rn] += safe;
        }
    }

    out
}

// ─── Rescale ────────────────────────────────────────────────────────────────

/// Rescale field so total matches target_qe. Conservation repair.
pub fn radial_rescale(field: &mut RadialField, target_qe: f32) {
    let sum = radial_total(field);
    if sum < 1e-10 {
        let per = target_qe.max(0.0) / (AXIAL * RADIAL) as f32;
        for row in field.iter_mut() {
            row.fill(per);
        }
        return;
    }
    let factor = target_qe / sum;
    for row in field.iter_mut() {
        for v in row.iter_mut() {
            *v *= factor;
        }
    }
}

// ─── Distribution from genome ───────────────────────────────────────────────

/// Genome biases → 2D radial field. Initialized from center (isotropic).
///
/// growth → axial tips (stations 0, 7). resilience → axial center (3, 4).
/// branching → lateral sectors (1, 3). All sectors start equal → bilateral emerges.
/// Axiom 6: symmetry from isotropy, not from mirroring.
pub fn distribute_to_radial(
    total_qe: f32,
    growth: f32,
    resilience: f32,
    branching: f32,
) -> RadialField {
    let mut profile = [[1.0_f32; RADIAL]; AXIAL];

    // Axial emphasis (same as 1D)
    let g = growth;
    profile[0] = profile[0].map(|v| v + g * 2.0);
    profile[7] = profile[7].map(|v| v + g * 2.0);
    profile[1] = profile[1].map(|v| v + g * 1.0);
    profile[6] = profile[6].map(|v| v + g * 1.0);

    let r = resilience;
    profile[3] = profile[3].map(|v| v + r * 3.0);
    profile[4] = profile[4].map(|v| v + r * 3.0);
    profile[2] = profile[2].map(|v| v + r * 1.5);
    profile[5] = profile[5].map(|v| v + r * 1.5);

    // Lateral emphasis: sectors 1 and 3 (NOT mirroring — equal addition)
    let b = branching;
    for a in 0..AXIAL {
        profile[a][1] += b * 2.0;
        profile[a][3] += b * 2.0;
        // Slight dorsal/ventral variation from branching
        profile[a][0] += b * 0.5;
        profile[a][2] += b * 0.5;
    }

    // Normalize to total_qe
    let sum: f32 = profile.iter().flat_map(|r| r.iter()).sum();
    if sum > 1e-10 && total_qe > 0.0 {
        let factor = total_qe / sum;
        for row in profile.iter_mut() {
            for v in row.iter_mut() {
                *v *= factor;
            }
        }
    }
    profile
}

// ─── Peak detection ─────────────────────────────────────────────────────────

/// Detect peaks: nodes higher than all 4 neighbors AND above mean × threshold_factor.
///
/// Returns (axial, radial, qe) for up to MAX_PEAKS peaks, sorted by qe descending.
/// Axiom 6: peaks emerge from diffusion dynamics, not programmed.
pub fn detect_peaks(field: &RadialField, threshold_factor: f32) -> [(u8, u8, f32); MAX_PEAKS] {
    let mean = radial_total(field) / (AXIAL * RADIAL) as f32;
    let threshold = mean * threshold_factor;
    let mut peaks = [(0u8, 0u8, 0.0f32); MAX_PEAKS];
    let mut count = 0usize;

    for a in 0..AXIAL {
        for r in 0..RADIAL {
            let v = field[a][r];
            if v < threshold {
                continue;
            }

            // Check 4 neighbors (axial±1, radial±1 wrapping)
            let higher_than_all = [
                if a > 0 { field[a - 1][r] } else { 0.0 },
                if a < AXIAL - 1 { field[a + 1][r] } else { 0.0 },
                field[a][(r + RADIAL - 1) % RADIAL],
                field[a][(r + 1) % RADIAL],
            ]
            .iter()
            .all(|&n| v > n);

            if higher_than_all && count < MAX_PEAKS {
                peaks[count] = (a as u8, r as u8, v);
                count += 1;
            }
        }
    }

    // Sort by qe descending
    peaks[..count]
        .sort_unstable_by(|a, b| b.2.partial_cmp(&a.2).unwrap_or(std::cmp::Ordering::Equal));
    peaks
}

/// Count valid peaks (qe > 0) in detect_peaks result.
pub fn peak_count(peaks: &[(u8, u8, f32); MAX_PEAKS]) -> usize {
    peaks.iter().take_while(|p| p.2 > 0.0).count()
}

// ─── Gradient ───────────────────────────────────────────────────────────────

/// Gradient at node: (axial_component, radial_component).
///
/// Points from low to high qe. Axiom 7: adjacent neighbors only.
pub fn gradient_at(field: &RadialField, ax: usize, rad: usize) -> (f32, f32) {
    let ax_grad = if ax == 0 {
        field[1][rad] - field[0][rad]
    } else if ax == AXIAL - 1 {
        field[AXIAL - 1][rad] - field[AXIAL - 2][rad]
    } else {
        (field[ax + 1][rad] - field[ax - 1][rad]) * 0.5
    };

    let rn = (rad + 1) % RADIAL;
    let rp = (rad + RADIAL - 1) % RADIAL;
    let rad_grad = (field[ax][rn] - field[ax][rp]) * 0.5;

    (ax_grad, rad_grad)
}

// ─── Aspect ratio ───────────────────────────────────────────────────────────

/// Aspect ratio of peak region: axial extent / radial extent.
///
/// High → elongated (tube/limb). Low → compact (bulb).
/// Axiom 6: shape from physics, not labels.
pub fn peak_aspect_ratio(field: &RadialField, ax: u8, rad: u8) -> f32 {
    let a = ax as usize;
    let r = rad as usize;
    let center_qe = field[a][r];
    if center_qe < 1e-6 {
        return 1.0;
    }
    let half = center_qe * 0.5;

    // Axial extent: count stations where qe > half of peak
    let mut ax_extent = 1u8;
    for da in 1..AXIAL {
        if a + da < AXIAL && field[a + da][r] > half {
            ax_extent += 1;
        } else {
            break;
        }
    }
    for da in 1..AXIAL {
        if a >= da && field[a - da][r] > half {
            ax_extent += 1;
        } else {
            break;
        }
    }

    // Radial extent: count sectors where qe > half of peak
    let mut rad_extent = 1u8;
    for dr in 1..RADIAL {
        if field[a][(r + dr) % RADIAL] > half {
            rad_extent += 1;
        } else {
            break;
        }
    }

    let ratio = ax_extent as f32 / rad_extent.max(1) as f32;
    ratio.clamp(0.1, 10.0)
}

// ─── Axial radii ────────────────────────────────────────────────────────────

/// Average across radial sectors → per-station radius for trunk mesh.
///
/// Axiom 1: radius ∝ sqrt(local_qe / mean_qe).
pub fn radial_to_axial_radii(
    field: &RadialField,
    base_radius: f32,
    min_ratio: f32,
    max_ratio: f32,
) -> [f32; AXIAL] {
    let total = radial_total(field);
    let mean = if total > 1e-6 {
        total / (AXIAL * RADIAL) as f32
    } else {
        1.0
    };
    let mut radii = [0.0f32; AXIAL];
    for a in 0..AXIAL {
        let station_mean: f32 = field[a].iter().sum::<f32>() / RADIAL as f32;
        let ratio = if mean > 1e-6 {
            station_mean / mean
        } else {
            1.0
        };
        radii[a] = base_radius * ratio.sqrt().clamp(min_ratio, max_ratio);
    }
    radii
}

// ─── Joint detection ────────────────────────────────────────────────────────

/// Detect joints: axial stations where ALL sectors have qe below threshold.
///
/// A cross-sectional valley = natural segmentation point.
/// Returns (axial_idx, min_qe_at_station) for stations that qualify.
/// Axiom 6: joints are energy valleys, never hardcoded.
pub fn detect_joints(field: &RadialField, threshold: f32) -> [(u8, f32); AXIAL] {
    let mut joints = [(0u8, 0.0f32); AXIAL];
    let mut count = 0usize;
    for a in 1..(AXIAL - 1) {
        // skip tips
        let station_min = field[a].iter().copied().fold(f32::MAX, f32::min);
        let station_max = field[a].iter().copied().fold(0.0f32, f32::max);
        // Valley: all sectors low AND lower than neighbors
        if station_max < threshold {
            let prev_max = field[a - 1].iter().copied().fold(0.0f32, f32::max);
            let next_max = field[a + 1].iter().copied().fold(0.0f32, f32::max);
            if station_max < prev_max && station_max < next_max && count < AXIAL {
                joints[count] = (a as u8, station_min);
                count += 1;
            }
        }
    }
    joints
}

/// Count valid joints (min_qe > 0 or station_idx > 0 for first).
pub fn joint_count(joints: &[(u8, f32); AXIAL]) -> usize {
    joints.iter().take_while(|j| j.0 > 0 || j.1 > 0.0).count()
}

// ─── EM-4: Appendage joint articulation ──────────────────────────────────────

/// Extract 1D energy profile along an appendage from its peak outward.
///
/// Walks from `peak_ax` in `direction` along the axial axis, collecting
/// the qe values at `peak_rad` sector. Returns up to AXIAL values.
/// Axiom 6: profile shape emerges from field, not programmed.
pub fn extract_appendage_profile(
    field: &RadialField,
    peak_ax: u8,
    peak_rad: u8,
    direction: i8,
) -> ([f32; AXIAL], usize) {
    let mut profile = [0.0f32; AXIAL];
    let mut len = 0usize;
    let mut ax = peak_ax as i32;
    while ax >= 0 && (ax as usize) < AXIAL && len < AXIAL {
        profile[len] = field[ax as usize][peak_rad as usize];
        len += 1;
        ax += direction as i32;
    }
    (profile, len)
}

/// Find valleys (local minima) in a 1D appendage profile.
///
/// A valley where `profile[i] < mean × threshold_ratio` → joint point.
/// Returns `(position_t ∈ [0,1], flexibility ∈ [0,1])` pairs.
/// `flexibility = 1.0 - (valley_qe / peak_qe).sqrt()` — lower qe = more flexible.
/// Axiom 1: joint = low energy region. Axiom 4: dissipation thins the connection.
pub fn detect_appendage_joints(profile: &[f32], len: usize) -> [(f32, f32); AXIAL] {
    let mut joints = [(0.0f32, 0.0f32); AXIAL];
    if len < 3 {
        return joints;
    }
    let active = &profile[..len];
    let peak_qe = active.iter().copied().fold(0.0f32, f32::max);
    if peak_qe <= 0.0 {
        return joints;
    }

    let mut count = 0usize;
    for i in 1..(len - 1) {
        if active[i] < active[i - 1] && active[i] < active[i + 1] {
            let t = i as f32 / (len - 1).max(1) as f32;
            let flexibility = 1.0 - (active[i] / peak_qe).sqrt().min(1.0);
            if count < AXIAL {
                joints[count] = (t, flexibility);
                count += 1;
            }
        }
    }
    joints
}

/// Count valid appendage joints (position > 0).
pub fn appendage_joint_count(joints: &[(f32, f32); AXIAL]) -> usize {
    joints.iter().take_while(|j| j.0 > 0.0 || j.1 > 0.0).count()
}

/// Compute per-segment radii for a segmented appendage.
///
/// At joint positions, radius thins proportional to flexibility.
/// Between joints, radius interpolates smoothly.
/// `base_radius`: the appendage's nominal radius.
/// Returns array of radii along the appendage (one per segment station).
pub fn segmented_radii(
    base_radius: f32,
    joints: &[(f32, f32)],
    joint_count: usize,
    stations: usize,
) -> [f32; AXIAL] {
    let mut radii = [base_radius; AXIAL];
    if stations == 0 {
        return radii;
    }
    for j in 0..joint_count {
        let (t, flexibility) = joints[j];
        let idx = (t * (stations - 1).max(1) as f32) as usize;
        let idx = idx.min(stations - 1);
        // Joint radius: thinner where flexibility is high (Axiom 4: dissipation thins)
        radii[idx] = base_radius * (1.0 - flexibility * 0.8).max(0.05);
    }
    radii
}

// ─── Frequency field ────────────────────────────────────────────────────────

/// Frequency entrainment across 2D neighbors.
pub fn radial_freq_entrain(freq: &RadialField, coupling: f32, dt: f32) -> RadialField {
    let mut out = *freq;
    let k = coupling.clamp(0.0, 1.0) * dt;

    for r in 0..RADIAL {
        for a in 0..(AXIAL - 1) {
            let delta = (out[a + 1][r] - out[a][r]) * k * 0.25;
            out[a][r] += delta;
            out[a + 1][r] -= delta;
        }
    }
    for a in 0..AXIAL {
        for r in 0..RADIAL {
            let rn = (r + 1) % RADIAL;
            let delta = (out[a][rn] - out[a][r]) * k * 0.25;
            out[a][r] += delta;
            out[a][rn] -= delta;
        }
    }
    out
}

// ─── Appendage geometry mapping (EM-3) ──────────────────────────────────────

/// Maps a 2D field peak to a 3D attachment position + direction on the trunk spine.
///
/// Returns `(attach_position, branch_direction)`.
/// `spine_positions`: trunk spine node positions (world space).
/// Peak's axial index maps to a trunk station. Radial sector maps to a direction.
pub fn peak_to_3d_offset(
    peak_ax: u8,
    peak_rad: u8,
    spine_positions: &[crate::math_types::Vec3],
) -> (crate::math_types::Vec3, crate::math_types::Vec3) {
    let spine_len = spine_positions.len();
    if spine_len == 0 {
        return (crate::math_types::Vec3::ZERO, crate::math_types::Vec3::Y);
    }
    let ax_t = peak_ax as f32 / (AXIAL - 1).max(1) as f32;
    let attach_idx = (ax_t * (spine_len - 1) as f32) as usize;
    let attach_idx = attach_idx.clamp(0, spine_len - 1);
    let attach_pos = spine_positions[attach_idx];

    let sector_angle = peak_rad as f32 * std::f32::consts::FRAC_PI_2;
    let branch_dir =
        crate::math_types::Vec3::new(sector_angle.sin(), sector_angle.cos() * 0.3, 0.0)
            .normalize_or_zero();

    (attach_pos, branch_dir)
}

/// Derives appendage spine parameters from peak properties.
///
/// Returns `(length, radius, detail_factor)`.
/// `aspect_ratio` from `peak_aspect_ratio()`: high = long tube, low = compact bulb.
/// All values derived from the peak's energy share — no hardcoded sizes.
pub fn peak_to_spine_params(
    peak_qe: f32,
    aspect_ratio: f32,
    base_length: f32,
    base_radius: f32,
    total_field_qe: f32,
) -> (f32, f32, f32) {
    let ar = aspect_ratio.min(3.0);
    let qe_share = if total_field_qe > 1e-6 {
        (peak_qe / total_field_qe).sqrt()
    } else {
        0.0
    };
    let app_length = base_length * 0.3 * ar;
    let app_radius = (base_radius * qe_share * 0.6).max(base_radius * 0.15);
    let detail = 0.7; // relative to trunk detail
    (app_length, app_radius, detail)
}

// ─── Viewer calibration (NOT simulation physics) ────────────────────────────
//
// These parameters tune mesh rendering visualization only. They do NOT affect
// simulation spawning, energy conservation, or axiom-derived thresholds.
// Equivalent to camera FOV or render resolution — presentation, not physics.

/// Diffusion iterations for viewer/export field maturation.
const VIEWER_DIFFUSION_STEPS: usize = 30;
/// Diffusion rate for viewer field maturation (α parameter).
const VIEWER_DIFFUSION_ALPHA: f32 = 0.1;
/// Diffusion coupling for viewer field maturation (β parameter).
const VIEWER_DIFFUSION_BETA: f32 = 0.05;
/// Axial frequency gradient multiplier (Hz per station from center).
const VIEWER_FREQ_AXIAL_GRAD: f32 = 20.0;
/// Radial frequency gradient multiplier (Hz per sector from center).
const VIEWER_FREQ_RADIAL_GRAD: f32 = 10.0;

/// Build a mature radial field from genome parameters.
///
/// Applies isotropic distribution + diffusion steps to develop emergent
/// bilateral peaks. Used by viewer/export binaries for mesh construction.
pub fn build_viewer_field(
    growth: f32,
    resilience: f32,
    branching: f32,
    base_qe: f32,
) -> RadialField {
    let mut field = distribute_to_radial(base_qe, growth, resilience, branching);
    for _ in 0..VIEWER_DIFFUSION_STEPS {
        field = radial_diffuse(&field, VIEWER_DIFFUSION_ALPHA, VIEWER_DIFFUSION_BETA);
    }
    field
}

/// Build a frequency field centered on `center_freq` with axial+radial gradients.
pub fn build_viewer_freq_field(center_freq: f32) -> RadialField {
    let mut freq_field = [[0.0f32; RADIAL]; AXIAL];
    let ax_center = (AXIAL as f32 - 1.0) / 2.0;
    let rad_center = (RADIAL as f32 - 1.0) / 2.0;
    for a in 0..AXIAL {
        for r in 0..RADIAL {
            freq_field[a][r] = center_freq
                + (a as f32 - ax_center) * VIEWER_FREQ_AXIAL_GRAD
                + (r as f32 - rad_center) * VIEWER_FREQ_RADIAL_GRAD;
        }
    }
    freq_field
}

// ─── Tests ──────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn uniform(qe: f32) -> RadialField {
        [[qe / (AXIAL * RADIAL) as f32; RADIAL]; AXIAL]
    }

    fn spike_at(ax: usize, rad: usize, total: f32) -> RadialField {
        let mut f = uniform(total * 0.5);
        f[ax][rad] += total * 0.5;
        f
    }

    // ── radial_total ────────────────────────────────────────────────────────

    #[test]
    fn total_uniform() {
        let f = uniform(100.0);
        assert!((radial_total(&f) - 100.0).abs() < 1e-3);
    }

    #[test]
    fn total_zeros() {
        assert_eq!(radial_total(&[[0.0; RADIAL]; AXIAL]), 0.0);
    }

    // ── radial_diffuse ──────────────────────────────────────────────────────

    #[test]
    fn diffuse_conserves() {
        let f = spike_at(3, 1, 100.0);
        let before = radial_total(&f);
        let after = radial_diffuse(&f, 0.5, 0.05);
        assert!((radial_total(&after) - before).abs() < 1e-3, "conservation");
    }

    #[test]
    fn diffuse_smooths_spike() {
        let f = spike_at(3, 1, 100.0);
        let smoothed = radial_diffuse(&f, 1.0, 1.0);
        assert!(smoothed[3][1] < f[3][1], "spike should decrease");
        assert!(smoothed[3][2] > f[3][2], "neighbor should increase");
    }

    #[test]
    fn diffuse_zero_conductivity_noop() {
        let f = spike_at(4, 0, 80.0);
        let result = radial_diffuse(&f, 0.0, 1.0);
        for a in 0..AXIAL {
            for r in 0..RADIAL {
                assert_eq!(f[a][r], result[a][r]);
            }
        }
    }

    #[test]
    fn diffuse_never_negative() {
        let mut f = [[0.001; RADIAL]; AXIAL];
        f[0][0] = 100.0;
        for _ in 0..100 {
            f = radial_diffuse(&f, 1.0, 0.1);
            for a in 0..AXIAL {
                for r in 0..RADIAL {
                    assert!(f[a][r] >= 0.0, "node [{a}][{r}] = {}", f[a][r]);
                }
            }
        }
    }

    // ── distribute_to_radial ────────────────────────────────────────────────

    #[test]
    fn distribute_conserves() {
        let f = distribute_to_radial(200.0, 0.8, 0.5, 0.6);
        assert!((radial_total(&f) - 200.0).abs() < 1e-3);
    }

    #[test]
    fn distribute_isotropic_sectors_equal() {
        // With zero branching, all sectors should be equal per station
        let f = distribute_to_radial(100.0, 0.5, 0.5, 0.0);
        for a in 0..AXIAL {
            let first = f[a][0];
            for r in 1..RADIAL {
                assert!(
                    (f[a][r] - first).abs() < 1e-4,
                    "station {a}: sector {r}={} vs sector 0={first}",
                    f[a][r]
                );
            }
        }
    }

    #[test]
    fn distribute_branching_emphasizes_laterals() {
        let f = distribute_to_radial(100.0, 0.0, 0.0, 1.0);
        // Sectors 1 and 3 (lateral) should have more than sectors 0 and 2
        let lateral: f32 = (0..AXIAL).map(|a| f[a][1] + f[a][3]).sum();
        let dorsal_ventral: f32 = (0..AXIAL).map(|a| f[a][0] + f[a][2]).sum();
        assert!(
            lateral > dorsal_ventral,
            "lateral={lateral} > dv={dorsal_ventral}"
        );
    }

    #[test]
    fn distribute_bilateral_symmetry() {
        let f = distribute_to_radial(100.0, 0.5, 0.5, 0.8);
        // Sectors 1 (right) and 3 (left) should be equal (isotropic init)
        for a in 0..AXIAL {
            assert!(
                (f[a][1] - f[a][3]).abs() < 1e-4,
                "station {a}: right={} left={}",
                f[a][1],
                f[a][3]
            );
        }
    }

    // ── detect_peaks ────────────────────────────────────────────────────────

    #[test]
    fn no_peaks_in_uniform() {
        let f = uniform(100.0);
        let peaks = detect_peaks(&f, 1.5);
        assert_eq!(peak_count(&peaks), 0);
    }

    #[test]
    fn single_spike_detected() {
        let f = spike_at(4, 2, 100.0);
        let peaks = detect_peaks(&f, 1.5);
        assert_eq!(peak_count(&peaks), 1);
        assert_eq!(peaks[0].0, 4); // axial
        assert_eq!(peaks[0].1, 2); // radial
    }

    #[test]
    fn bilateral_peaks_detected() {
        let mut f = uniform(50.0);
        f[3][1] += 30.0; // right
        f[3][3] += 30.0; // left (bilateral)
        let peaks = detect_peaks(&f, 1.5);
        assert!(peak_count(&peaks) >= 2, "should detect bilateral pair");
    }

    // ── gradient_at ─────────────────────────────────────────────────────────

    #[test]
    fn gradient_flat_is_zero() {
        let f = uniform(100.0);
        let (ax, rad) = gradient_at(&f, 4, 2);
        assert!(ax.abs() < 1e-4 && rad.abs() < 1e-4);
    }

    #[test]
    fn gradient_points_uphill() {
        let mut f = uniform(10.0);
        f[5][2] = 50.0; // high at [5][2]
        let (ax, _) = gradient_at(&f, 4, 2);
        assert!(ax > 0.0, "gradient should point toward higher station");
    }

    // ── peak_aspect_ratio ───────────────────────────────────────────────────

    #[test]
    fn aspect_ratio_compact_peak() {
        let f = spike_at(4, 1, 100.0);
        let ar = peak_aspect_ratio(&f, 4, 1);
        assert!(ar <= 2.0, "single node spike → compact: ar={ar}");
    }

    #[test]
    fn aspect_ratio_elongated() {
        let mut f = uniform(20.0);
        // Elongated peak across 3 axial stations
        f[3][1] = 20.0;
        f[4][1] = 25.0;
        f[5][1] = 20.0;
        let ar = peak_aspect_ratio(&f, 4, 1);
        assert!(ar > 1.5, "multi-station peak → elongated: ar={ar}");
    }

    // ── radial_to_axial_radii ───────────────────────────────────────────────

    #[test]
    fn uniform_field_uniform_radii() {
        let f = uniform(100.0);
        let radii = radial_to_axial_radii(&f, 1.0, 0.3, 2.5);
        for r in &radii {
            assert!((*r - 1.0).abs() < 0.1, "uniform → uniform: r={r}");
        }
    }

    #[test]
    fn spike_station_wider() {
        let mut f = uniform(50.0);
        for r in 0..RADIAL {
            f[3][r] += 20.0;
        }
        let radii = radial_to_axial_radii(&f, 1.0, 0.3, 2.5);
        assert!(
            radii[3] > radii[0],
            "spike station wider: {}>{}",
            radii[3],
            radii[0]
        );
    }

    // ── detect_joints ───────────────────────────────────────────────────────

    #[test]
    fn no_joints_in_uniform() {
        let f = uniform(100.0);
        let joints = detect_joints(&f, 5.0);
        assert_eq!(joint_count(&joints), 0);
    }

    #[test]
    fn valley_detected_as_joint() {
        let mut f = [[10.0; RADIAL]; AXIAL];
        f[4] = [1.0; RADIAL]; // valley at station 4
        let joints = detect_joints(&f, 5.0);
        assert!(joint_count(&joints) >= 1);
        assert_eq!(joints[0].0, 4);
    }

    // ── radial_freq_entrain ─────────────────────────────────────────────────

    #[test]
    fn freq_entrain_converges() {
        let mut freq = [[0.0; RADIAL]; AXIAL];
        for a in 0..AXIAL {
            for r in 0..RADIAL {
                freq[a][r] = (a * 100 + r * 50) as f32;
            }
        }
        let entrained = radial_freq_entrain(&freq, 1.0, 1.0);
        let gap_before = (freq[0][0] - freq[1][0]).abs();
        let gap_after = (entrained[0][0] - entrained[1][0]).abs();
        assert!(gap_after < gap_before, "should converge");
    }

    // ── rescale ─────────────────────────────────────────────────────────────

    // ── EM-4: appendage joints ─────────────────────────────────────────────

    #[test]
    fn extract_profile_from_peak_outward() {
        let mut f = uniform(10.0);
        f[3][1] = 50.0; // peak at ax=3, rad=1
        let (profile, len) = extract_appendage_profile(&f, 3, 1, 1); // walk forward
        assert!(len > 0);
        assert!((profile[0] - 50.0).abs() < 1e-5, "starts at peak");
    }

    #[test]
    fn extract_profile_backward() {
        let f = uniform(10.0);
        let (_, len) = extract_appendage_profile(&f, 5, 0, -1);
        assert_eq!(len, 6, "from ax=5 walking back: 5,4,3,2,1,0");
    }

    #[test]
    fn detect_joints_uniform_returns_empty() {
        let profile = [10.0; AXIAL];
        let joints = detect_appendage_joints(&profile, 8);
        assert_eq!(appendage_joint_count(&joints), 0);
    }

    #[test]
    fn detect_joints_single_valley() {
        let profile = [10.0, 15.0, 2.0, 15.0, 10.0, 0.0, 0.0, 0.0];
        let joints = detect_appendage_joints(&profile, 5);
        assert_eq!(appendage_joint_count(&joints), 1);
        let (t, flex) = joints[0];
        assert!(t > 0.3 && t < 0.7, "joint at middle: t={t}");
        assert!(flex > 0.5, "low valley → high flexibility: {flex}");
    }

    #[test]
    fn detect_joints_two_valleys() {
        let profile = [10.0, 1.0, 10.0, 1.0, 10.0, 0.0, 0.0, 0.0];
        let joints = detect_appendage_joints(&profile, 5);
        assert_eq!(appendage_joint_count(&joints), 2);
    }

    #[test]
    fn segmented_radii_no_joints_uniform() {
        let joints = [(0.0, 0.0); AXIAL];
        let radii = segmented_radii(1.0, &joints, 0, 8);
        for r in &radii[..8] {
            assert!((*r - 1.0).abs() < 1e-5);
        }
    }

    #[test]
    fn segmented_radii_joint_thins() {
        let mut joints = [(0.0, 0.0); AXIAL];
        joints[0] = (0.5, 0.9); // joint at middle, high flexibility
        let radii = segmented_radii(1.0, &joints, 1, 8);
        let mid = radii[3]; // idx = 0.5 * 7 ≈ 3
        assert!(mid < 0.5, "joint should thin radius: {mid}");
    }

    #[test]
    fn rescale_matches_target() {
        let mut f = spike_at(2, 1, 80.0);
        radial_rescale(&mut f, 100.0);
        assert!((radial_total(&f) - 100.0).abs() < 1e-3);
    }

    // ── peak_to_3d_offset ────────────────────────────────────────────────────

    #[test]
    fn peak_3d_offset_mid_attaches_mid_spine() {
        let spine: Vec<crate::math_types::Vec3> = (0..10)
            .map(|i| crate::math_types::Vec3::new(0.0, i as f32, 0.0))
            .collect();
        let mid_ax = (AXIAL / 2) as u8;
        let (pos, _dir) = peak_to_3d_offset(mid_ax, 1, &spine);
        assert!(pos.y > 2.0, "mid axial should attach mid spine: {pos:?}");
    }

    #[test]
    fn peak_3d_offset_empty_spine_returns_zero() {
        let (pos, dir) = peak_to_3d_offset(0, 0, &[]);
        assert_eq!(pos, crate::math_types::Vec3::ZERO);
        assert!(dir.length() > 0.0);
    }

    #[test]
    fn peak_3d_offset_sector_direction_varies() {
        let spine = vec![crate::math_types::Vec3::ZERO, crate::math_types::Vec3::Y];
        let (_, dir_0) = peak_to_3d_offset(0, 0, &spine);
        let (_, dir_1) = peak_to_3d_offset(0, 1, &spine);
        assert!(
            (dir_0 - dir_1).length() > 0.01,
            "different sectors → different directions"
        );
    }

    // ── peak_to_spine_params ─────────────────────────────────────────────────

    #[test]
    fn spine_params_high_ar_longer() {
        let (len_low, _, _) = peak_to_spine_params(10.0, 0.5, 5.0, 1.0, 100.0);
        let (len_high, _, _) = peak_to_spine_params(10.0, 2.5, 5.0, 1.0, 100.0);
        assert!(
            len_high > len_low,
            "high AR → longer: {len_high} vs {len_low}"
        );
    }

    #[test]
    fn spine_params_more_qe_wider() {
        let (_, rad_low, _) = peak_to_spine_params(5.0, 1.0, 5.0, 1.0, 100.0);
        let (_, rad_high, _) = peak_to_spine_params(50.0, 1.0, 5.0, 1.0, 100.0);
        assert!(
            rad_high > rad_low,
            "more qe → wider: {rad_high} vs {rad_low}"
        );
    }

    #[test]
    fn spine_params_zero_total_no_panic() {
        let (len, rad, _) = peak_to_spine_params(10.0, 1.0, 5.0, 1.0, 0.0);
        assert!(len >= 0.0);
        assert!(rad >= 0.0);
    }
}
