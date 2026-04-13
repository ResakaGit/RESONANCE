//! MD-4: Molecular dynamics observables — pure math.
//!
//! Virial pressure, radial distribution function (RDF), potential energy,
//! lattice initialization, Maxwell-Boltzmann velocities.
//! All stateless. All in reduced LJ units (sigma=1, epsilon=1, m=1, k_B=1).

use super::determinism;

// ─── Virial pressure ───────────────────────────────────────────────────────

/// Virial pressure contribution from one pair.
///
/// P_virial = (1 / (D * V)) * Σ_{i<j} (r_ij · f_ij)
/// where r_ij · f_ij = dx*fx + dy*fy.
/// Caller sums over pairs; divides by (D * V) and adds ideal gas term rho*T.
#[inline]
pub fn virial_pair(dx: f32, dy: f32, fx: f32, fy: f32) -> f64 {
    (dx as f64) * (fx as f64) + (dy as f64) * (fy as f64)
}

/// Full virial pressure: P = rho*T + virial_sum / (D * V).
///
/// `virial_sum`: accumulated Σ (r·f) from all pairs.
/// `n`: particle count. `volume`: box area (2D). `temperature`: instantaneous T.
/// `d`: spatial dimension (2 for 2D).
#[inline]
pub fn virial_pressure(virial_sum: f64, n: usize, volume: f64, temperature: f64, d: usize) -> f64 {
    let rho = n as f64 / volume;
    rho * temperature + virial_sum / (d as f64 * volume)
}

// ─── Radial distribution function ──────────────────────────────────────────

/// RDF histogram accumulator. Stateful but no external dependencies.
pub struct RdfAccumulator {
    bins: Vec<u64>,
    dr: f64,
    n_frames: u64,
    n_particles: usize,
    box_area: f64,
}

impl RdfAccumulator {
    /// Create with `n_bins` bins covering [0, r_max].
    pub fn new(r_max: f64, n_bins: usize, n_particles: usize, box_area: f64) -> Self {
        Self {
            bins: vec![0; n_bins],
            dr: r_max / n_bins as f64,
            n_frames: 0,
            n_particles,
            box_area,
        }
    }

    /// Add one pair distance to the histogram.
    #[inline]
    pub fn add_pair(&mut self, r: f64) {
        let bin = (r / self.dr) as usize;
        if bin < self.bins.len() {
            self.bins[bin] += 1;
        }
    }

    /// Mark end of one frame (call after iterating all pairs in one snapshot).
    pub fn end_frame(&mut self) {
        self.n_frames += 1;
    }

    /// Normalize to g(r). Returns (r_center, g(r)) pairs.
    ///
    /// 2D: g(r) = 2 * count / (n_frames * N * rho * 2π * r * dr).
    /// Factor of 2: pairs are counted once (i<j), but g(r) sums over all i≠j.
    pub fn normalize(&self) -> Vec<(f64, f64)> {
        let n = self.n_particles as f64;
        let rho = n / self.box_area;
        let frames = self.n_frames.max(1) as f64;
        let two_pi = 2.0 * core::f64::consts::PI;

        self.bins
            .iter()
            .enumerate()
            .map(|(i, &count)| {
                let r = (i as f64 + 0.5) * self.dr;
                let shell_area = two_pi * r * self.dr;
                let expected = frames * n * rho * shell_area;
                let g = if expected > 0.0 {
                    2.0 * count as f64 / expected
                } else {
                    0.0
                };
                (r, g)
            })
            .collect()
    }
}

// ─── 3D Radial distribution function (MD-10) ──────────────────────────────

/// 3D RDF histogram accumulator. Normalizes with spherical shell volume 4πr²dr.
pub struct RdfAccumulator3D {
    bins: Vec<u64>,
    dr: f64,
    n_frames: u64,
    n_particles: usize,
    box_volume: f64,
}

impl RdfAccumulator3D {
    /// Create with `n_bins` bins covering [0, r_max].
    pub fn new(r_max: f64, n_bins: usize, n_particles: usize, box_volume: f64) -> Self {
        Self {
            bins: vec![0; n_bins],
            dr: r_max / n_bins as f64,
            n_frames: 0,
            n_particles,
            box_volume,
        }
    }

    /// Add one pair distance to the histogram.
    #[inline]
    pub fn add_pair(&mut self, r: f64) {
        let bin = (r / self.dr) as usize;
        if bin < self.bins.len() {
            self.bins[bin] += 1;
        }
    }

    /// Mark end of one frame.
    pub fn end_frame(&mut self) {
        self.n_frames += 1;
    }

    /// Normalize to g(r). Returns (r_center, g(r)) pairs.
    ///
    /// 3D: g(r) = 2 * count / (n_frames * N * rho * 4π * r² * dr).
    pub fn normalize(&self) -> Vec<(f64, f64)> {
        let n = self.n_particles as f64;
        let rho = n / self.box_volume;
        let frames = self.n_frames.max(1) as f64;
        let four_pi = 4.0 * core::f64::consts::PI;

        self.bins
            .iter()
            .enumerate()
            .map(|(i, &count)| {
                let r = (i as f64 + 0.5) * self.dr;
                let shell_vol = four_pi * r * r * self.dr;
                let expected = frames * n * rho * shell_vol;
                let g = if expected > 0.0 {
                    2.0 * count as f64 / expected
                } else {
                    0.0
                };
                (r, g)
            })
            .collect()
    }
}

// ─── LJ potential ──────────────────────────────────────────────────────────

/// LJ potential in reduced units: V(r) = 4 * [(1/r)^12 - (1/r)^6].
/// Shifted so V(r_cut) = 0 (standard MD practice).
#[inline]
pub fn lj_potential_reduced(r: f32, r_cut: f32) -> f64 {
    if r >= r_cut || r <= 0.0 {
        return 0.0;
    }
    let r = r as f64;
    let r_cut = r_cut as f64;
    let sr6 = 1.0 / (r * r * r * r * r * r);
    let sr12 = sr6 * sr6;
    let sr6_c = 1.0 / (r_cut * r_cut * r_cut * r_cut * r_cut * r_cut);
    let sr12_c = sr6_c * sr6_c;
    4.0 * (sr12 - sr6) - 4.0 * (sr12_c - sr6_c)
}

/// LJ force on particle i from particle j, in reduced units.
///
/// Convention: positive component in +dx direction = attractive (toward j).
/// At close range (r < 2^(1/6)σ): force is NEGATIVE along dx (repulsive, away from j).
/// At medium range: force is POSITIVE along dx (attractive, toward j).
///
/// Softened at r < 0.5σ and capped to prevent explosion.
pub fn lj_force_reduced(dx: f32, dy: f32, r_cut: f32) -> [f32; 2] {
    let r_sq = dx as f64 * dx as f64 + dy as f64 * dy as f64;
    let r_cut_sq = r_cut as f64 * r_cut as f64;
    if r_sq >= r_cut_sq || r_sq < 1e-20 {
        return [0.0, 0.0];
    }
    // Softening: clamp minimum r to 0.5σ to prevent singularity
    let r_sq_safe = r_sq.max(0.25); // 0.5² = 0.25
    let r = r_sq.sqrt();
    // Radial LJ: F_radial = 24*[2*(1/r)^12 - (1/r)^6] / r. Positive = radially outward.
    // Convention: force on i = -F_radial * (dx/r, dy/r) [negative because dx points TOWARD j]
    let sr2 = 1.0 / r_sq_safe;
    let sr6 = sr2 * sr2 * sr2;
    let sr12 = sr6 * sr6;
    let f_radial = 24.0 * (2.0 * sr12 - sr6) / r_sq_safe.sqrt();
    let f_on_i = -f_radial;
    let ux = dx as f64 / r;
    let uy = dy as f64 / r;
    [(f_on_i * ux) as f32, (f_on_i * uy) as f32]
}

// ─── 3D f64 LJ force (MD-7) ────────────────────────────────────────────────

/// LJ force in 3D reduced units (f64). Returns force on particle i from j.
///
/// Same convention as 2D: negative along displacement = repulsive at close range.
/// Softened at r < 0.5σ, capped at ±1000.
pub fn lj_force_reduced_3d(d: [f64; 3], r_cut: f64) -> [f64; 3] {
    let r_sq = d[0] * d[0] + d[1] * d[1] + d[2] * d[2];
    let r_cut_sq = r_cut * r_cut;
    if r_sq >= r_cut_sq || r_sq < 1e-30 {
        return [0.0; 3];
    }
    let r_sq_safe = r_sq.max(0.25);
    let r = r_sq.sqrt();
    let sr2 = 1.0 / r_sq_safe;
    let sr6 = sr2 * sr2 * sr2;
    let sr12 = sr6 * sr6;
    let f_radial = 24.0 * (2.0 * sr12 - sr6) / r_sq_safe.sqrt();
    let f_on_i = -f_radial;
    [f_on_i * d[0] / r, f_on_i * d[1] / r, f_on_i * d[2] / r]
}

/// LJ potential in reduced units (f64 input). Shifted so V(r_cut) = 0.
pub fn lj_potential_reduced_f64(r: f64, r_cut: f64) -> f64 {
    if r >= r_cut || r <= 0.0 {
        return 0.0;
    }
    let sr6 = 1.0 / (r * r * r * r * r * r);
    let sr12 = sr6 * sr6;
    let sr6_c = 1.0 / (r_cut * r_cut * r_cut * r_cut * r_cut * r_cut);
    let sr12_c = sr6_c * sr6_c;
    4.0 * (sr12 - sr6) - 4.0 * (sr12_c - sr6_c)
}

// ─── Initialization ────────────────────────────────────────────────────────

/// Place N particles on a 2D square lattice within [0, L)².
///
/// Square lattice (simpler than triangular, sufficient for initialization
/// since thermostat will equilibrate the structure).
pub fn square_lattice_2d(n: usize, box_length: f64) -> Vec<[f32; 2]> {
    let side = (n as f64).sqrt().ceil() as usize;
    let spacing = box_length / side as f64;
    let mut positions = Vec::with_capacity(n);
    for i in 0..n {
        let row = i / side;
        let col = i % side;
        let x = (col as f64 + 0.5) * spacing;
        let y = (row as f64 + 0.5) * spacing;
        positions.push([x as f32, y as f32]);
    }
    positions
}

/// Assign Maxwell-Boltzmann velocities in 2D, then remove COM drift.
///
/// In reduced units: v_component ~ N(0, sqrt(T/m)) = N(0, sqrt(T)).
/// Deterministic via `determinism::gaussian_f32`.
pub fn init_velocities_2d(n: usize, temperature: f64, seed: u64) -> Vec<[f32; 2]> {
    let sigma = (temperature).sqrt() as f32;
    let mut velocities = Vec::with_capacity(n);
    let mut rng = seed;
    for _ in 0..n {
        rng = determinism::next_u64(rng);
        let vx = determinism::gaussian_f32(rng, sigma);
        rng = determinism::next_u64(determinism::next_u64(rng));
        let vy = determinism::gaussian_f32(rng, sigma);
        rng = determinism::next_u64(determinism::next_u64(rng));
        velocities.push([vx, vy]);
    }
    // Remove COM drift: Σv = 0
    let n_f = n as f32;
    let com_vx: f32 = velocities.iter().map(|v| v[0]).sum::<f32>() / n_f;
    let com_vy: f32 = velocities.iter().map(|v| v[1]).sum::<f32>() / n_f;
    for v in &mut velocities {
        v[0] -= com_vx;
        v[1] -= com_vy;
    }
    velocities
}

// ─── 3D initialization (MD-7) ──────────────────────────────────────────────

/// Place N particles on a 3D cubic lattice within [0, L)^3.
pub fn cubic_lattice_3d(n: usize, box_length: f64) -> Vec<[f64; 3]> {
    let side = (n as f64).cbrt().ceil() as usize;
    let spacing = box_length / side as f64;
    let mut positions = Vec::with_capacity(n);
    for i in 0..n {
        let z = i / (side * side);
        let rem = i % (side * side);
        let y = rem / side;
        let x = rem % side;
        positions.push([
            (x as f64 + 0.5) * spacing,
            (y as f64 + 0.5) * spacing,
            (z as f64 + 0.5) * spacing,
        ]);
    }
    positions
}

/// Assign Maxwell-Boltzmann velocities in 3D (f64), zero COM drift.
pub fn init_velocities_3d(n: usize, temperature: f64, seed: u64) -> Vec<[f64; 3]> {
    let sigma = temperature.sqrt() as f32;
    let mut velocities = Vec::with_capacity(n);
    let mut rng = seed;
    for _ in 0..n {
        let mut v = [0.0f64; 3];
        for dim in 0..3 {
            rng = determinism::next_u64(rng);
            v[dim] = determinism::gaussian_f32(rng, sigma) as f64;
            rng = determinism::next_u64(determinism::next_u64(rng));
        }
        velocities.push(v);
    }
    let n_f = n as f64;
    let com: [f64; 3] = [
        velocities.iter().map(|v| v[0]).sum::<f64>() / n_f,
        velocities.iter().map(|v| v[1]).sum::<f64>() / n_f,
        velocities.iter().map(|v| v[2]).sum::<f64>() / n_f,
    ];
    for v in &mut velocities {
        v[0] -= com[0];
        v[1] -= com[1];
        v[2] -= com[2];
    }
    velocities
}

// ─── MD-8: Force-shifted potential + tail corrections ──────────────────────

/// Raw LJ radial force: F(r) = 24 * [2/r^13 - 1/r^7]. Positive = outward (repulsive).
/// Helper for force-shifted potential.
fn lj_radial_force_raw(r: f64) -> f64 {
    let sr2 = 1.0 / (r * r);
    let sr6 = sr2 * sr2 * sr2;
    let sr12 = sr6 * sr6;
    24.0 * (2.0 * sr12 - sr6) / r
}

/// Force-shifted LJ potential: both V AND dV/dr are continuous at r_cut.
///
/// V_fs(r) = V(r) - V(r_cut) - (r - r_cut) * dV/dr(r_cut).
/// Eliminates energy jumps AND force jumps at cutoff → better energy conservation.
/// Reduced units: sigma=1, epsilon=1.
pub fn lj_potential_force_shifted(r: f64, r_cut: f64) -> f64 {
    if r >= r_cut || r <= 0.0 {
        return 0.0;
    }
    let v_r = lj_raw_potential(r);
    let v_rc = lj_raw_potential(r_cut);
    let dv_rc = -lj_radial_force_raw(r_cut); // dV/dr = -F(r)
    v_r - v_rc - (r - r_cut) * dv_rc
}

/// Raw LJ potential (unshifted): V(r) = 4*[(1/r)^12 - (1/r)^6].
fn lj_raw_potential(r: f64) -> f64 {
    let sr6 = 1.0 / (r * r * r * r * r * r);
    4.0 * (sr6 * sr6 - sr6)
}

/// LJ tail correction for energy per particle (3D, reduced units).
///
/// U_tail/N = (8/3) * pi * rho * [1/(3*r_cut^9) - 1/r_cut^3].
/// Compensates for truncation of attractive tail beyond r_cut.
/// Always negative (attractive contribution missed by cutoff).
pub fn lj_tail_correction_energy_3d(n: usize, density: f64, r_cut: f64) -> f64 {
    let rc3 = r_cut * r_cut * r_cut;
    let rc9 = rc3 * rc3 * rc3;
    let per_particle = (8.0 / 3.0) * core::f64::consts::PI * density
        * (1.0 / (3.0 * rc9) - 1.0 / rc3);
    per_particle * n as f64
}

/// LJ tail correction for pressure (3D, reduced units).
///
/// P_tail = (16/3) * pi * rho^2 * [2/(3*r_cut^9) - 1/r_cut^3].
/// Scales as rho^2. Always negative for standard r_cut.
pub fn lj_tail_correction_pressure_3d(density: f64, r_cut: f64) -> f64 {
    let rc3 = r_cut * r_cut * r_cut;
    let rc9 = rc3 * rc3 * rc3;
    (16.0 / 3.0) * core::f64::consts::PI * density * density
        * (2.0 / (3.0 * rc9) - 1.0 / rc3)
}

/// LJ tail correction for energy per particle (2D, reduced units).
///
/// U_tail/N = 2 * pi * rho * [1/(5*r_cut^10) - 1/(2*r_cut^4)].
pub fn lj_tail_correction_energy_2d(n: usize, density: f64, r_cut: f64) -> f64 {
    let rc4 = r_cut.powi(4);
    let rc10 = r_cut.powi(10);
    let per_particle = 2.0 * core::f64::consts::PI * density
        * (1.0 / (5.0 * rc10) - 1.0 / (2.0 * rc4));
    per_particle * n as f64
}

// ─── MD-9: Ramachandran observable ─────────────────────────────────────────

/// Bin a (phi, psi) pair into a 2D histogram for the Ramachandran plot.
///
/// Both angles in radians [-pi, pi]. Returns (row, col) bin indices in [0, n_bins).
#[inline]
pub fn ramachandran_bin(phi: f32, psi: f32, n_bins: usize) -> (usize, usize) {
    let pi = core::f32::consts::PI;
    let two_pi = 2.0 * pi;
    let phi_norm = (phi + pi) / two_pi;
    let psi_norm = (psi + pi) / two_pi;
    let i = ((phi_norm * n_bins as f32) as usize).min(n_bins - 1);
    let j = ((psi_norm * n_bins as f32) as usize).min(n_bins - 1);
    (i, j)
}

/// LJ force in 3D with explicit sigma and epsilon parameters.
///
/// d: displacement from i to j. Returns force ON i.
/// Softened at r < 0.5*sigma, capped at ±1000*epsilon.
pub fn lj_force_3d_params(d: [f64; 3], sigma: f64, epsilon: f64, r_cut: f64) -> [f64; 3] {
    let r_sq = d[0] * d[0] + d[1] * d[1] + d[2] * d[2];
    let r_cut_sq = r_cut * r_cut;
    if r_sq >= r_cut_sq || r_sq < 1e-30 {
        return [0.0; 3];
    }
    let sig_sq = sigma * sigma;
    let r_sq_safe = r_sq.max(0.25 * sig_sq);
    let r = r_sq.sqrt();
    let sr2 = sig_sq / r_sq_safe;
    let sr6 = sr2 * sr2 * sr2;
    let sr12 = sr6 * sr6;
    let f_radial = 24.0 * epsilon * (2.0 * sr12 - sr6) / r_sq_safe.sqrt();
    let f_on_i = -f_radial;
    [f_on_i * d[0] / r, f_on_i * d[1] / r, f_on_i * d[2] / r]
}

/// LJ potential with explicit sigma and epsilon. Shifted so V(r_cut)=0.
pub fn lj_potential_3d_params(r: f64, sigma: f64, epsilon: f64, r_cut: f64) -> f64 {
    if r >= r_cut || r <= 0.0 {
        return 0.0;
    }
    let sr = sigma / r;
    let sr6 = sr.powi(6);
    let sr12 = sr6 * sr6;
    let sc = sigma / r_cut;
    let sc6 = sc.powi(6);
    let sc12 = sc6 * sc6;
    4.0 * epsilon * ((sr12 - sr6) - (sc12 - sc6))
}

// ─── Tests ──────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn virial_pressure_ideal_gas() {
        // No forces → virial_sum = 0 → P = rho * T
        let p = virial_pressure(0.0, 100, 100.0, 2.0, 2);
        let expected = 1.0 * 2.0; // rho=1, T=2
        assert!((p - expected).abs() < 1e-10, "P={p}, expected={expected}");
    }

    #[test]
    fn virial_pair_symmetric() {
        let v1 = virial_pair(1.0, 0.0, -2.0, 0.0);
        let v2 = virial_pair(-1.0, 0.0, 2.0, 0.0);
        assert!((v1 - v2).abs() < 1e-10, "Newton 3: same virial");
    }

    #[test]
    fn lj_potential_zero_at_cutoff() {
        let v = lj_potential_reduced(2.5, 2.5);
        assert!(v.abs() < 1e-10, "V(r_cut) should be 0: {v}");
    }

    #[test]
    fn lj_potential_minimum_near_sigma() {
        // LJ minimum at r = 2^(1/6) ≈ 1.1225
        let r_min = 2.0f32.powf(1.0 / 6.0);
        let v_min = lj_potential_reduced(r_min, 2.5);
        // At minimum, V = -1.0 (reduced) + shift
        assert!(v_min < 0.0, "potential at minimum should be negative: {v_min}");
        // Check nearby values are higher
        let v_inner = lj_potential_reduced(r_min * 0.95, 2.5);
        let v_outer = lj_potential_reduced(r_min * 1.05, 2.5);
        assert!(v_inner > v_min, "inner is less favorable");
        assert!(v_outer > v_min, "outer is less favorable");
    }

    #[test]
    fn lj_force_repulsive_close() {
        // r=0.9 < sigma=1: repulsive → force on i pushes AWAY from j (negative dx)
        let f = lj_force_reduced(0.9, 0.0, 2.5);
        assert!(f[0] < 0.0, "repulsive at r < sigma: fx={}", f[0]);
    }

    #[test]
    fn lj_force_attractive_medium() {
        // r=1.5 > 2^(1/6): attractive → force on i pulls TOWARD j (positive dx)
        let f = lj_force_reduced(1.5, 0.0, 2.5);
        assert!(f[0] > 0.0, "attractive at r > 2^(1/6): fx={}", f[0]);
    }

    #[test]
    fn lj_force_zero_beyond_cutoff() {
        let f = lj_force_reduced(3.0, 0.0, 2.5);
        assert_eq!(f, [0.0, 0.0], "zero beyond cutoff");
    }

    #[test]
    fn rdf_normalization_approaches_one() {
        // Uniform random particles → g(r) ≈ 1 at all r
        let n = 500;
        let box_len = 20.0;
        let area = box_len * box_len;
        let mut rdf = RdfAccumulator::new(box_len / 2.0, 50, n, area);

        // Generate "uniform" pair distances
        let positions = square_lattice_2d(n, box_len);
        for i in 0..n {
            for j in (i + 1)..n {
                let d = super::super::pbc::minimum_image_2d(
                    positions[i],
                    positions[j],
                    [box_len as f32, box_len as f32],
                );
                let r = ((d[0] * d[0] + d[1] * d[1]) as f64).sqrt();
                rdf.add_pair(r);
            }
        }
        rdf.end_frame();
        let gr = rdf.normalize();
        // Check bins far from origin (avoid lattice artifacts)
        let far_bins: Vec<f64> = gr.iter().filter(|(r, _)| *r > 3.0 && *r < 8.0).map(|(_, g)| *g).collect();
        let mean_g: f64 = far_bins.iter().sum::<f64>() / far_bins.len() as f64;
        assert!(
            (mean_g - 1.0).abs() < 0.3,
            "g(r) far from origin should be ~1: {mean_g:.3}",
        );
    }

    #[test]
    fn lattice_correct_density() {
        let n = 100;
        let box_len = 10.0;
        let pos = square_lattice_2d(n, box_len);
        assert_eq!(pos.len(), n);
        // All within box
        for p in &pos {
            assert!(p[0] >= 0.0 && p[0] < box_len as f32, "x={}", p[0]);
            assert!(p[1] >= 0.0 && p[1] < box_len as f32, "y={}", p[1]);
        }
        // Density = N / V
        let rho = n as f64 / (box_len * box_len);
        assert!((rho - 1.0).abs() < 1e-10);
    }

    #[test]
    fn velocities_zero_com_drift() {
        let vels = init_velocities_2d(200, 1.0, 42);
        let com_vx: f32 = vels.iter().map(|v| v[0]).sum::<f32>();
        let com_vy: f32 = vels.iter().map(|v| v[1]).sum::<f32>();
        assert!(com_vx.abs() < 1e-4, "COM vx = {com_vx}");
        assert!(com_vy.abs() < 1e-4, "COM vy = {com_vy}");
    }

    #[test]
    fn velocities_correct_temperature() {
        let n = 1000;
        let target_t = 2.0;
        let vels = init_velocities_2d(n, target_t, 42);
        // T = <m v²> / (D * k_B) = <v²> / 2  (reduced units, m=1, k_B=1, D=2)
        let sum_v2: f64 = vels.iter().map(|v| v[0] as f64 * v[0] as f64 + v[1] as f64 * v[1] as f64).sum();
        let t_measured = sum_v2 / (2.0 * n as f64);
        assert!(
            ((t_measured - target_t) / target_t).abs() < 0.1,
            "T={t_measured:.3}, expected={target_t}",
        );
    }

    // ─── MD-8: Force-shifted potential + tail corrections ──────────────────

    #[test]
    fn force_shifted_continuous_at_cutoff() {
        // V_fs(r_cut) = 0 AND dV_fs/dr(r_cut) = 0 (both continuous)
        let r_cut = 2.5;
        // Potential at cutoff must be zero
        let v_at_cut = lj_potential_force_shifted(r_cut - 1e-10, r_cut);
        assert!(v_at_cut.abs() < 1e-6, "V_fs(r_cut) should → 0: {v_at_cut}");
        // Potential just beyond cutoff is exactly zero
        let v_beyond = lj_potential_force_shifted(r_cut + 1e-10, r_cut);
        assert_eq!(v_beyond, 0.0, "V_fs beyond r_cut must be 0");
        // Numerical derivative at r_cut should → 0 (force continuity)
        let eps = 1e-7;
        let r_near = r_cut - eps;
        let r_far = r_cut - 2.0 * eps;
        let dv_dr = (lj_potential_force_shifted(r_near, r_cut)
            - lj_potential_force_shifted(r_far, r_cut))
            / eps;
        assert!(
            dv_dr.abs() < 1e-3,
            "dV_fs/dr near r_cut should → 0: {dv_dr}",
        );
    }

    #[test]
    fn force_shifted_zero_or_negative_r() {
        assert_eq!(lj_potential_force_shifted(0.0, 2.5), 0.0);
        assert_eq!(lj_potential_force_shifted(-1.0, 2.5), 0.0);
    }

    #[test]
    fn force_shifted_has_minimum() {
        // Force-shifted LJ should still have a minimum near r = 2^(1/6)
        let r_cut = 2.5;
        let r_min = 2.0f64.powf(1.0 / 6.0);
        let v_min = lj_potential_force_shifted(r_min, r_cut);
        let v_inner = lj_potential_force_shifted(r_min * 0.95, r_cut);
        let v_outer = lj_potential_force_shifted(r_min * 1.05, r_cut);
        assert!(v_min < v_inner, "minimum lower than inner: {v_min} vs {v_inner}");
        assert!(v_min < v_outer, "minimum lower than outer: {v_min} vs {v_outer}");
    }

    #[test]
    fn tail_correction_sign() {
        // Both U_tail and P_tail are negative (attractive contribution missed by cutoff)
        let r_cut = 2.5;
        let density = 0.8;
        let n = 100;
        let u_tail = lj_tail_correction_energy_3d(n, density, r_cut);
        let p_tail = lj_tail_correction_pressure_3d(density, r_cut);
        assert!(u_tail < 0.0, "U_tail should be < 0: {u_tail}");
        assert!(p_tail < 0.0, "P_tail should be < 0: {p_tail}");
    }

    #[test]
    fn tail_correction_scales_with_density() {
        // P_tail ∝ rho^2 → P_tail(2*rho) / P_tail(rho) = 4
        let r_cut = 2.5;
        let rho = 0.5;
        let p1 = lj_tail_correction_pressure_3d(rho, r_cut);
        let p2 = lj_tail_correction_pressure_3d(2.0 * rho, r_cut);
        let ratio = p2 / p1;
        assert!(
            (ratio - 4.0).abs() < 1e-10,
            "P_tail(2ρ)/P_tail(ρ) = {ratio}, expected 4.0",
        );
    }

    #[test]
    fn tail_correction_energy_scales_with_n_and_density() {
        // U_tail ∝ N * rho → doubling N doubles U_tail, doubling rho doubles U_tail
        let r_cut = 2.5;
        let rho = 0.8;
        let u_100 = lj_tail_correction_energy_3d(100, rho, r_cut);
        let u_200 = lj_tail_correction_energy_3d(200, rho, r_cut);
        assert!(
            (u_200 / u_100 - 2.0).abs() < 1e-10,
            "U_tail should scale linearly with N",
        );
        let u_rho2 = lj_tail_correction_energy_3d(100, 2.0 * rho, r_cut);
        assert!(
            (u_rho2 / u_100 - 2.0).abs() < 1e-10,
            "U_tail should scale linearly with density",
        );
    }

    // ─── MD-9: Ramachandran + parameterized LJ ──────────────────────────

    #[test]
    fn ramachandran_bin_center() {
        // phi=0, psi=0 → center of the plot (n_bins/2, n_bins/2)
        let (i, j) = ramachandran_bin(0.0, 0.0, 36);
        assert_eq!(i, 18, "phi=0 → bin 18");
        assert_eq!(j, 18, "psi=0 → bin 18");
    }

    #[test]
    fn ramachandran_bin_edges() {
        let pi = std::f32::consts::PI;
        let (i, j) = ramachandran_bin(-pi, -pi, 36);
        assert_eq!(i, 0, "phi=-pi → bin 0");
        assert_eq!(j, 0, "psi=-pi → bin 0");
        let (i, j) = ramachandran_bin(pi - 0.01, pi - 0.01, 36);
        assert_eq!(i, 35, "phi≈pi → last bin");
        assert_eq!(j, 35, "psi≈pi → last bin");
    }

    #[test]
    fn lj_params_zero_beyond_cutoff() {
        let f = lj_force_3d_params([5.0, 0.0, 0.0], 1.0, 1.0, 2.5);
        assert_eq!(f, [0.0; 3]);
    }

    #[test]
    fn lj_params_repulsive_close() {
        let f = lj_force_3d_params([0.9, 0.0, 0.0], 1.0, 1.0, 2.5);
        assert!(f[0] < 0.0, "repulsive at r < sigma: fx={}", f[0]);
    }

    #[test]
    fn lj_params_sigma_scaling() {
        // Force at r=sigma should be same as reduced-unit force at r=1
        let f_reduced = lj_force_reduced_3d([1.0, 0.0, 0.0], 2.5);
        let f_params = lj_force_3d_params([2.0, 0.0, 0.0], 2.0, 1.0, 5.0);
        // At r=sigma the reduced-unit value is at r=1. Forces scale as epsilon/sigma.
        let ratio = f_params[0] / (f_reduced[0] / 2.0);
        assert!(
            (ratio - 1.0).abs() < 0.01,
            "sigma scaling: ratio={ratio:.4}",
        );
    }

    #[test]
    fn lj_potential_params_zero_at_cutoff() {
        let v = lj_potential_3d_params(2.5, 1.0, 1.0, 2.5);
        assert!(v.abs() < 1e-10, "V(r_cut)=0: {v}");
    }

    #[test]
    fn lj_potential_params_minimum() {
        let r_min = 2.0_f64.powf(1.0 / 6.0) * 1.5; // sigma=1.5
        let v_min = lj_potential_3d_params(r_min, 1.5, 2.0, 5.0);
        let v_inner = lj_potential_3d_params(r_min * 0.95, 1.5, 2.0, 5.0);
        let v_outer = lj_potential_3d_params(r_min * 1.05, 1.5, 2.0, 5.0);
        assert!(v_min < v_inner, "minimum lower than inner");
        assert!(v_min < v_outer, "minimum lower than outer");
    }

    #[test]
    fn tail_correction_2d_sign() {
        let u = lj_tail_correction_energy_2d(100, 0.8, 2.5);
        assert!(u < 0.0, "2D U_tail should be < 0: {u}");
    }

    #[test]
    fn tail_correction_decreases_with_larger_cutoff() {
        // Larger r_cut → less tail truncated → |correction| smaller
        let rho = 0.8;
        let u_25 = lj_tail_correction_energy_3d(100, rho, 2.5).abs();
        let u_40 = lj_tail_correction_energy_3d(100, rho, 4.0).abs();
        assert!(
            u_40 < u_25,
            "|U_tail(r_cut=4)| < |U_tail(r_cut=2.5)|: {u_40} vs {u_25}",
        );
    }
}
