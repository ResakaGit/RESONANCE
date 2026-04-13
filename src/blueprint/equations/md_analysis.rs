//! MD-18: Structural analysis — RMSD, Rg, contact maps, PMF.
//!
//! All functions pure, stateless, no heap in hot paths.
//! RMSD uses Kabsch alignment (SVD of 3x3 matrix, no external crate).

// ─── RMSD with Kabsch alignment ──────────────────────────────────────────

/// RMSD between two coordinate sets after optimal superposition (Kabsch algorithm).
///
/// 1. Center both structures at origin.
/// 2. Compute cross-covariance matrix H = A^T * B.
/// 3. SVD of H → optimal rotation.
/// 4. Apply rotation, compute RMSD.
///
/// Returns 0.0 for identical structures (within floating point).
pub fn rmsd_kabsch(coords_a: &[[f64; 3]], coords_b: &[[f64; 3]]) -> f64 {
    let n = coords_a.len();
    if n == 0 || n != coords_b.len() {
        return 0.0;
    }
    if n == 1 {
        return 0.0; // single point always aligns
    }

    // Center both structures
    let ca = center(coords_a);
    let cb = center(coords_b);

    // Cross-covariance matrix H (3x3)
    let h = cross_covariance(&ca, &cb);

    // SVD of H → rotation R = V * U^T (with reflection check)
    let r = kabsch_rotation(&h);

    // Apply rotation to ca, compute RMSD
    let mut sum_sq = 0.0;
    for i in 0..n {
        let rotated = mat3_vec(&r, &ca[i]);
        for k in 0..3 {
            let d = rotated[k] - cb[i][k];
            sum_sq += d * d;
        }
    }

    (sum_sq / n as f64).sqrt()
}

/// Center coordinates at origin. Returns new Vec.
fn center(coords: &[[f64; 3]]) -> Vec<[f64; 3]> {
    let n = coords.len() as f64;
    let mut com = [0.0; 3];
    for c in coords {
        for k in 0..3 { com[k] += c[k]; }
    }
    for k in 0..3 { com[k] /= n; }

    coords.iter().map(|c| [c[0] - com[0], c[1] - com[1], c[2] - com[2]]).collect()
}

/// 3x3 cross-covariance matrix: H = A^T * B.
fn cross_covariance(a: &[[f64; 3]], b: &[[f64; 3]]) -> [[f64; 3]; 3] {
    let mut h = [[0.0; 3]; 3];
    for i in 0..a.len() {
        for r in 0..3 {
            for c in 0..3 {
                h[r][c] += a[i][r] * b[i][c];
            }
        }
    }
    h
}

/// Kabsch rotation from cross-covariance via iterative polar decomposition.
///
/// Uses repeated R = 0.5*(R + R^{-T}) convergence (Newton-Schulz).
/// Robust for 3x3 matrices; converges in ~15 iterations.
fn kabsch_rotation(h: &[[f64; 3]; 3]) -> [[f64; 3]; 3] {
    let det_h = mat3_det(h);
    if det_h.abs() < 1e-30 {
        return mat3_identity(); // degenerate case
    }

    // Initialize R = H / ||H||_F (normalize to avoid overflow)
    let mut norm_sq = 0.0;
    for i in 0..3 { for j in 0..3 { norm_sq += h[i][j] * h[i][j]; } }
    let norm = norm_sq.sqrt();
    if norm < 1e-30 { return mat3_identity(); }

    let mut r = mat3_scale(h, 1.0 / norm);

    // Newton-Schulz iteration for polar decomposition: R_{k+1} = 0.5*(R_k + R_k^{-T})
    for _ in 0..30 {
        let r_inv = mat3_inv(&r);
        let r_inv_t = mat3_transpose(&r_inv);
        r = mat3_scale(&mat3_add(&r, &r_inv_t), 0.5);
    }

    // Check for reflection (det(R) should be +1)
    let det = mat3_det(&r);
    if det < 0.0 {
        for row in 0..3 {
            r[row][2] = -r[row][2];
        }
    }

    r
}

// ─── 3x3 matrix utilities ─────────────────────────────────────────────────

fn mat3_identity() -> [[f64; 3]; 3] {
    [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]]
}

fn mat3_transpose(m: &[[f64; 3]; 3]) -> [[f64; 3]; 3] {
    [[m[0][0], m[1][0], m[2][0]],
     [m[0][1], m[1][1], m[2][1]],
     [m[0][2], m[1][2], m[2][2]]]
}

fn mat3_add(a: &[[f64; 3]; 3], b: &[[f64; 3]; 3]) -> [[f64; 3]; 3] {
    let mut c = [[0.0; 3]; 3];
    for i in 0..3 {
        for j in 0..3 {
            c[i][j] = a[i][j] + b[i][j];
        }
    }
    c
}

fn mat3_scale(m: &[[f64; 3]; 3], s: f64) -> [[f64; 3]; 3] {
    let mut r = *m;
    for i in 0..3 {
        for j in 0..3 {
            r[i][j] *= s;
        }
    }
    r
}

fn mat3_det(m: &[[f64; 3]; 3]) -> f64 {
    m[0][0] * (m[1][1] * m[2][2] - m[1][2] * m[2][1])
  - m[0][1] * (m[1][0] * m[2][2] - m[1][2] * m[2][0])
  + m[0][2] * (m[1][0] * m[2][1] - m[1][1] * m[2][0])
}

fn mat3_inv(m: &[[f64; 3]; 3]) -> [[f64; 3]; 3] {
    let det = mat3_det(m);
    if det.abs() < 1e-30 {
        return mat3_identity(); // singular → return identity as fallback
    }
    let inv_det = 1.0 / det;
    [
        [(m[1][1]*m[2][2] - m[1][2]*m[2][1]) * inv_det,
         (m[0][2]*m[2][1] - m[0][1]*m[2][2]) * inv_det,
         (m[0][1]*m[1][2] - m[0][2]*m[1][1]) * inv_det],
        [(m[1][2]*m[2][0] - m[1][0]*m[2][2]) * inv_det,
         (m[0][0]*m[2][2] - m[0][2]*m[2][0]) * inv_det,
         (m[0][2]*m[1][0] - m[0][0]*m[1][2]) * inv_det],
        [(m[1][0]*m[2][1] - m[1][1]*m[2][0]) * inv_det,
         (m[0][1]*m[2][0] - m[0][0]*m[2][1]) * inv_det,
         (m[0][0]*m[1][1] - m[0][1]*m[1][0]) * inv_det],
    ]
}

fn mat3_vec(m: &[[f64; 3]; 3], v: &[f64; 3]) -> [f64; 3] {
    [m[0][0]*v[0] + m[0][1]*v[1] + m[0][2]*v[2],
     m[1][0]*v[0] + m[1][1]*v[1] + m[1][2]*v[2],
     m[2][0]*v[0] + m[2][1]*v[1] + m[2][2]*v[2]]
}

// ─── Radius of gyration ──────────────────────────────────────────────────

/// Radius of gyration: Rg = sqrt(Σ m_i |r_i - r_com|² / Σ m_i).
///
/// For equal masses, equivalent to RMS distance from centroid.
pub fn radius_of_gyration(coords: &[[f64; 3]], masses: &[f64]) -> f64 {
    let n = coords.len();
    if n == 0 { return 0.0; }

    let total_mass: f64 = masses.iter().sum();
    if total_mass <= 0.0 { return 0.0; }

    let mut com = [0.0; 3];
    for (c, &m) in coords.iter().zip(masses.iter()) {
        for k in 0..3 { com[k] += m * c[k]; }
    }
    for k in 0..3 { com[k] /= total_mass; }

    let mut sum = 0.0;
    for (c, &m) in coords.iter().zip(masses.iter()) {
        for k in 0..3 {
            let d = c[k] - com[k];
            sum += m * d * d;
        }
    }

    (sum / total_mass).sqrt()
}

// ─── Contact map ─────────────────────────────────────────────────────────

/// Pairwise distance matrix (flat, row-major, N×N).
pub fn distance_map(coords: &[[f64; 3]]) -> Vec<f64> {
    let n = coords.len();
    let mut map = vec![0.0; n * n];
    for i in 0..n {
        for j in (i + 1)..n {
            let mut d_sq = 0.0;
            for k in 0..3 {
                let dk = coords[i][k] - coords[j][k];
                d_sq += dk * dk;
            }
            let d = d_sq.sqrt();
            map[i * n + j] = d;
            map[j * n + i] = d;
        }
    }
    map
}

// ─── Native contact fraction ─────────────────────────────────────────────

/// Fraction of native contacts present: Q = (contacts with r < tolerance * sigma) / total.
pub fn native_fraction(
    coords: &[[f64; 3]],
    native_contacts: &[(u16, u16, f64)],
    tolerance: f64,
) -> f64 {
    if native_contacts.is_empty() { return 0.0; }

    let mut formed = 0u32;
    for &(i, j, sigma) in native_contacts {
        let (i, j) = (i as usize, j as usize);
        let mut d_sq = 0.0;
        for k in 0..3 {
            let dk = coords[i][k] - coords[j][k];
            d_sq += dk * dk;
        }
        if d_sq.sqrt() < tolerance * sigma {
            formed += 1;
        }
    }

    formed as f64 / native_contacts.len() as f64
}

// ─── PMF from histogram ──────────────────────────────────────────────────

/// Potential of mean force: F(x) = -k_B*T * ln(P(x)).
///
/// Bins with zero counts get f64::INFINITY. Shifted so minimum = 0.
pub fn pmf_from_histogram(bins: &[u64], k_b_t: f64) -> Vec<f64> {
    if bins.is_empty() { return Vec::new(); }

    let max_count = *bins.iter().max().unwrap_or(&1);
    if max_count == 0 { return vec![0.0; bins.len()]; }

    let norm = max_count as f64;
    let mut pmf: Vec<f64> = bins.iter().map(|&c| {
        if c == 0 {
            f64::INFINITY
        } else {
            -k_b_t * (c as f64 / norm).ln()
        }
    }).collect();

    // Shift so minimum = 0
    let min_val = pmf.iter().copied().filter(|v| v.is_finite()).fold(f64::INFINITY, f64::min);
    if min_val.is_finite() {
        for v in &mut pmf {
            if v.is_finite() { *v -= min_val; }
        }
    }

    pmf
}

// ─── Tests ────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rmsd_identical_is_zero() {
        let coords = vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]];
        assert!(rmsd_kabsch(&coords, &coords) < 1e-10);
    }

    #[test]
    fn rmsd_translated_is_zero() {
        let a = vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]];
        let b: Vec<[f64; 3]> = a.iter().map(|p| [p[0] + 5.0, p[1] - 3.0, p[2] + 10.0]).collect();
        assert!(rmsd_kabsch(&a, &b) < 1e-8, "translation should not affect RMSD");
    }

    #[test]
    fn rmsd_small_perturbation_small_value() {
        // Small perturbation → small RMSD
        let a = vec![[0.0, 0.0, 0.0], [3.0, 0.0, 0.0], [0.0, 3.0, 0.0], [3.0, 3.0, 0.0]];
        let b = vec![[0.1, 0.0, 0.0], [3.1, 0.0, 0.0], [0.1, 3.0, 0.0], [3.1, 3.0, 0.0]];
        let r = rmsd_kabsch(&a, &b);
        // After centering and alignment, the 0.1 shift should be removed → RMSD ~ 0
        assert!(r < 0.2, "small perturbation RMSD should be small, got {r}");
    }

    #[test]
    fn rmsd_different_structures() {
        let a = vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0]];
        let b = vec![[0.0, 0.0, 0.0], [0.0, 1.0, 0.0]];
        let r = rmsd_kabsch(&a, &b);
        // After centering and alignment, RMSD should be > 0
        assert!(r > 0.0);
    }

    #[test]
    fn rg_single_atom_is_zero() {
        let coords = vec![[5.0, 3.0, -1.0]];
        let masses = vec![1.0];
        assert!(radius_of_gyration(&coords, &masses) < 1e-15);
    }

    #[test]
    fn rg_two_equal_masses() {
        // Two unit masses at distance d: Rg = d/2
        let d = 4.0;
        let coords = vec![[0.0, 0.0, 0.0], [d, 0.0, 0.0]];
        let masses = vec![1.0, 1.0];
        let rg = radius_of_gyration(&coords, &masses);
        assert!((rg - d / 2.0).abs() < 1e-10, "Rg={rg}, expected {}", d / 2.0);
    }

    #[test]
    fn native_fraction_at_native() {
        // All contacts within tolerance → Q = 1.0
        let coords = vec![[0.0, 0.0, 0.0], [3.0, 0.0, 0.0], [6.0, 0.0, 0.0]];
        let contacts = vec![(0u16, 1, 3.0), (1, 2, 3.0)];
        let q = native_fraction(&coords, &contacts, 1.2);
        assert!((q - 1.0).abs() < 1e-10, "Q should be 1.0, got {q}");
    }

    #[test]
    fn native_fraction_none_formed() {
        // All contacts beyond tolerance → Q = 0.0
        let coords = vec![[0.0, 0.0, 0.0], [100.0, 0.0, 0.0]];
        let contacts = vec![(0u16, 1, 3.0)];
        let q = native_fraction(&coords, &contacts, 1.2);
        assert!(q < 1e-10, "Q should be 0.0, got {q}");
    }

    #[test]
    fn pmf_uniform_is_flat() {
        let bins = vec![100, 100, 100, 100];
        let pmf = pmf_from_histogram(&bins, 1.0);
        for v in &pmf {
            assert!((*v).abs() < 1e-10, "PMF should be 0 for uniform: {v}");
        }
    }

    #[test]
    fn pmf_zero_bins_are_infinity() {
        let bins = vec![100, 0, 50];
        let pmf = pmf_from_histogram(&bins, 1.0);
        assert!(pmf[1].is_infinite());
    }

    #[test]
    fn distance_map_symmetric() {
        let coords = vec![[0.0, 0.0, 0.0], [3.0, 4.0, 0.0], [1.0, 1.0, 1.0]];
        let dm = distance_map(&coords);
        let n = 3;
        for i in 0..n {
            for j in 0..n {
                assert!((dm[i * n + j] - dm[j * n + i]).abs() < 1e-15);
            }
            assert!(dm[i * n + i] < 1e-15); // diagonal is 0
        }
        // Check (0,1) distance: sqrt(9+16) = 5
        assert!((dm[0 * n + 1] - 5.0).abs() < 1e-10);
    }
}
