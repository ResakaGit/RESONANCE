//! MD-6: Molecular topology — connectivity graph for bonded interactions.
//!
//! Describes which atoms are bonded, angles, dihedrals, and residue grouping.
//! Immutable during force computation. Pre-built lists, no heap in force loops.
//!
//! ADR-021: topology separate from EntitySlot (cold data, DoD separation).

// ─── Parameter types ───────────────────────────────────────────────────────

/// Harmonic bond parameters: V = 0.5 * k * (r - r0)^2.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BondParams {
    pub r0: f64,
    pub k: f64,
}

/// Harmonic angle parameters: V = 0.5 * k * (theta - theta0)^2.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct AngleParams {
    pub theta0: f64,
    pub k: f64,
}

/// Proper dihedral parameters: V = k * (1 + cos(n*phi - delta)).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct DihedralParams {
    pub k: f64,
    pub n: u8,
    pub delta: f64,
}

/// Residue information (amino acid, nucleotide, etc.).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ResidueInfo {
    /// 3-letter code + null, e.g. *b"ALA\0".
    pub name: [u8; 4],
    /// Index of first atom in this residue.
    pub first_atom: u16,
    /// Number of atoms in this residue.
    pub atom_count: u16,
}

// ─── Topology ──────────────────────────────────────────────────────────────

/// Molecular connectivity graph. Read-only during force computation.
#[derive(Clone, Debug)]
pub struct Topology {
    pub n_atoms: usize,
    pub bonds: Vec<(u16, u16, BondParams)>,
    pub angles: Vec<(u16, u16, u16, AngleParams)>,
    pub dihedrals: Vec<(u16, u16, u16, u16, DihedralParams)>,
    pub residues: Vec<ResidueInfo>,
    pub atom_types: Vec<u8>,
}

impl Topology {
    /// Empty topology for `n` atoms.
    pub fn new(n_atoms: usize) -> Self {
        Self {
            n_atoms,
            bonds: Vec::new(),
            angles: Vec::new(),
            dihedrals: Vec::new(),
            residues: Vec::new(),
            atom_types: vec![0; n_atoms],
        }
    }

    /// Add a bond between atoms i and j. Stores with i < j for canonical ordering.
    pub fn add_bond(&mut self, i: u16, j: u16, params: BondParams) {
        let (a, b) = if i < j { (i, j) } else { (j, i) };
        self.bonds.push((a, b, params));
    }

    /// Add an angle i-j-k (j is vertex).
    pub fn add_angle(&mut self, i: u16, j: u16, k: u16, params: AngleParams) {
        self.angles.push((i, j, k, params));
    }

    /// Add a dihedral i-j-k-l.
    pub fn add_dihedral(&mut self, i: u16, j: u16, k: u16, l: u16, params: DihedralParams) {
        self.dihedrals.push((i, j, k, l, params));
    }

    /// Add a residue definition.
    pub fn add_residue(&mut self, info: ResidueInfo) {
        self.residues.push(info);
    }

    /// Infer angles from bond graph: if bonds (a-b) and (b-c) exist, add angle a-b-c.
    ///
    /// `default_params`: applied to all inferred angles.
    /// Skips angles that already exist.
    pub fn infer_angles_from_bonds(&mut self, default_params: AngleParams) {
        // Build adjacency: for each atom, list its bonded neighbors.
        let mut neighbors: Vec<Vec<u16>> = vec![Vec::new(); self.n_atoms];
        for &(a, b, _) in &self.bonds {
            neighbors[a as usize].push(b);
            neighbors[b as usize].push(a);
        }

        let mut new_angles = Vec::new();
        // For each atom b (vertex), find all pairs of neighbors (a, c).
        for b in 0..self.n_atoms {
            let nbrs = &neighbors[b];
            for ni in 0..nbrs.len() {
                for nj in (ni + 1)..nbrs.len() {
                    let a = nbrs[ni];
                    let c = nbrs[nj];
                    // Canonical: a < c
                    let (a, c) = if a < c { (a, c) } else { (c, a) };
                    new_angles.push((a, b as u16, c, default_params));
                }
            }
        }

        // Deduplicate: skip angles that already exist (same triplet regardless of order)
        for (a, b, c, params) in new_angles {
            let exists = self.angles.iter().any(|&(ea, eb, ec, _)| {
                eb == b && ((ea == a && ec == c) || (ea == c && ec == a))
            });
            if !exists {
                self.angles.push((a, b, c, params));
            }
        }
    }

    /// Infer dihedrals from bond graph: if bonds (a-b), (b-c), (c-d) exist, add dihedral a-b-c-d.
    ///
    /// `default_params`: applied to all inferred dihedrals.
    /// Skips dihedrals that already exist.
    pub fn infer_dihedrals_from_bonds(&mut self, default_params: DihedralParams) {
        let mut neighbors: Vec<Vec<u16>> = vec![Vec::new(); self.n_atoms];
        for &(a, b, _) in &self.bonds {
            neighbors[a as usize].push(b);
            neighbors[b as usize].push(a);
        }

        let mut new_dihedrals = Vec::new();
        // For each bond (b, c), find all a bonded to b (a != c) and d bonded to c (d != b).
        for &(b, c, _) in &self.bonds {
            for &a in &neighbors[b as usize] {
                if a == c {
                    continue;
                }
                for &d in &neighbors[c as usize] {
                    if d == b || d == a {
                        continue;
                    }
                    new_dihedrals.push((a, b, c, d, default_params));
                }
            }
        }

        // Deduplicate: a-b-c-d is same as d-c-b-a
        for (a, b, c, d, params) in new_dihedrals {
            let exists = self.dihedrals.iter().any(|&(ea, eb, ec, ed, _)| {
                (ea == a && eb == b && ec == c && ed == d)
                    || (ea == d && eb == c && ec == b && ed == a)
            });
            if !exists {
                self.dihedrals.push((a, b, c, d, params));
            }
        }
    }

    /// Build a linear chain topology: 0-1-2-..-(n-1).
    ///
    /// Convenience for testing and polymer simulations.
    pub fn linear_chain(n: usize, bond_params: BondParams) -> Self {
        let mut topo = Self::new(n);
        for i in 0..(n - 1) {
            topo.add_bond(i as u16, (i + 1) as u16, bond_params);
        }
        topo
    }

    /// Build exclusion matrix: flat bool array of size n_atoms * n_atoms.
    ///
    /// `excluded[i * n + j] = true` means pair (i, j) is excluded from non-bonded interactions.
    /// Excludes 1-2 (bonded) and 1-3 (share an angle vertex) pairs.
    pub fn build_exclusion_matrix(&self) -> Vec<bool> {
        let n = self.n_atoms;
        let mut excl = vec![false; n * n];

        // Self-exclusion
        for i in 0..n {
            excl[i * n + i] = true;
        }

        // 1-2 pairs (bonded)
        for &(a, b, _) in &self.bonds {
            let (a, b) = (a as usize, b as usize);
            excl[a * n + b] = true;
            excl[b * n + a] = true;
        }

        // 1-3 pairs (share angle vertex)
        for &(a, _b, c, _) in &self.angles {
            let (a, c) = (a as usize, c as usize);
            excl[a * n + c] = true;
            excl[c * n + a] = true;
        }

        excl
    }
}

// ─── Tests ──────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn default_bond() -> BondParams {
        BondParams { r0: 1.0, k: 100.0 }
    }

    fn default_angle() -> AngleParams {
        AngleParams {
            theta0: std::f64::consts::PI,
            k: 50.0,
        }
    }

    fn default_dihedral() -> DihedralParams {
        DihedralParams {
            k: 1.0,
            n: 3,
            delta: 0.0,
        }
    }

    #[test]
    fn add_bond_stores_canonical() {
        let mut t = Topology::new(3);
        t.add_bond(2, 0, default_bond());
        assert_eq!(t.bonds[0].0, 0, "canonical: smaller index first");
        assert_eq!(t.bonds[0].1, 2);
    }

    #[test]
    fn add_bond_preserves_params() {
        let mut t = Topology::new(2);
        let p = BondParams { r0: 1.5, k: 200.0 };
        t.add_bond(0, 1, p);
        assert_eq!(t.bonds[0].2, p);
    }

    #[test]
    fn infer_angles_from_linear_chain() {
        // 0-1-2-3: bonds (0,1), (1,2), (2,3) → angles 0-1-2, 1-2-3
        let mut t = Topology::linear_chain(4, default_bond());
        t.infer_angles_from_bonds(default_angle());
        assert_eq!(t.angles.len(), 2, "4-atom chain has 2 angles");
        // Verify vertex atoms
        let vertices: Vec<u16> = t.angles.iter().map(|a| a.1).collect();
        assert!(vertices.contains(&1), "angle at atom 1");
        assert!(vertices.contains(&2), "angle at atom 2");
    }

    #[test]
    fn infer_angles_no_duplicates() {
        let mut t = Topology::linear_chain(3, default_bond());
        t.infer_angles_from_bonds(default_angle());
        t.infer_angles_from_bonds(default_angle()); // call twice
        assert_eq!(t.angles.len(), 1, "no duplicate angles");
    }

    #[test]
    fn infer_dihedrals_from_linear_chain() {
        // 0-1-2-3-4: bonds (0,1), (1,2), (2,3), (3,4) → dihedrals 0-1-2-3, 1-2-3-4
        let mut t = Topology::linear_chain(5, default_bond());
        t.infer_dihedrals_from_bonds(default_dihedral());
        assert_eq!(t.dihedrals.len(), 2, "5-atom chain has 2 dihedrals");
    }

    #[test]
    fn infer_dihedrals_no_duplicates() {
        let mut t = Topology::linear_chain(4, default_bond());
        t.infer_dihedrals_from_bonds(default_dihedral());
        t.infer_dihedrals_from_bonds(default_dihedral());
        assert_eq!(t.dihedrals.len(), 1, "no duplicate dihedrals");
    }

    #[test]
    fn infer_angles_branched() {
        // Star: center=0, branches=1,2,3
        let mut t = Topology::new(4);
        t.add_bond(0, 1, default_bond());
        t.add_bond(0, 2, default_bond());
        t.add_bond(0, 3, default_bond());
        t.infer_angles_from_bonds(default_angle());
        // 3 branches from center → C(3,2) = 3 angles: 1-0-2, 1-0-3, 2-0-3
        assert_eq!(t.angles.len(), 3, "star with 3 branches → 3 angles");
    }

    #[test]
    fn linear_chain_counts() {
        let n = 10;
        let mut t = Topology::linear_chain(n, default_bond());
        t.infer_angles_from_bonds(default_angle());
        t.infer_dihedrals_from_bonds(default_dihedral());
        assert_eq!(t.bonds.len(), n - 1, "{n}-chain: {}-1 bonds", n);
        assert_eq!(t.angles.len(), n - 2, "{n}-chain: {}-2 angles", n);
        assert_eq!(t.dihedrals.len(), n - 3, "{n}-chain: {}-3 dihedrals", n);
    }

    #[test]
    fn exclusion_matrix_bonds_and_angles() {
        let mut t = Topology::linear_chain(4, default_bond());
        t.infer_angles_from_bonds(default_angle());
        let excl = t.build_exclusion_matrix();
        let n = 4;
        // Self-exclusion
        assert!(excl[0 * n + 0]);
        // 1-2 bonded: (0,1), (1,2), (2,3)
        assert!(excl[0 * n + 1]);
        assert!(excl[1 * n + 0]);
        assert!(excl[1 * n + 2]);
        // 1-3 via angles: (0,2), (1,3)
        assert!(excl[0 * n + 2]);
        assert!(excl[2 * n + 0]);
        assert!(excl[1 * n + 3]);
        // 1-4: NOT excluded (0,3)
        assert!(!excl[0 * n + 3]);
        assert!(!excl[3 * n + 0]);
    }

    #[test]
    fn residue_boundaries() {
        let mut t = Topology::new(6);
        t.add_residue(ResidueInfo {
            name: *b"ALA\0",
            first_atom: 0,
            atom_count: 3,
        });
        t.add_residue(ResidueInfo {
            name: *b"GLY\0",
            first_atom: 3,
            atom_count: 3,
        });
        assert_eq!(t.residues.len(), 2);
        assert_eq!(t.residues[0].first_atom, 0);
        assert_eq!(t.residues[0].atom_count, 3);
        assert_eq!(t.residues[1].first_atom, 3);
    }
}
