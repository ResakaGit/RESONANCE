//! MD-13: Force field parameter loading and assignment.
//!
//! Parses standard force field files (AMBER `.dat` format) and assigns
//! parameters to a `Topology`. Setup-time only — never called during
//! force computation loops.
//!
//! Pipeline: load_amber_params(text) -> ForceFieldParams -> assign_parameters(topology, ff)

pub mod amber;
pub mod pdb;
pub mod water;

use super::topology::{AngleParams, BondParams, DihedralParams, Topology};

// ─── Types ────────────────────────────────────────────────────────────────

/// Atom type metadata from MASS section.
#[derive(Clone, Debug, PartialEq)]
pub struct AtomTypeInfo {
    pub name: String,
    pub mass: f64,
}

/// LJ (Lennard-Jones) parameters per atom type.
#[derive(Clone, Debug, PartialEq)]
pub struct LjParams {
    pub sigma: f64,
    pub epsilon: f64,
}

/// Complete force field parameter set.
///
/// String keys match atom type names from AMBER `.dat` files.
/// Only used at setup time — not stored in components (Hard Block #7).
#[derive(Clone, Debug, Default)]
pub struct ForceFieldParams {
    pub atom_types: Vec<AtomTypeInfo>,
    pub bond_params: Vec<(String, String, BondParams)>,
    pub angle_params: Vec<(String, String, String, AngleParams)>,
    pub dihedral_params: Vec<(String, String, String, String, DihedralParams)>,
    pub lj_params: Vec<(String, LjParams)>,
}

// ─── Lookup helpers ───────────────────────────────────────────────────────

/// Canonical key for a pair: sort alphabetically.
fn canonical_pair<'a>(a: &'a str, b: &'a str) -> (&'a str, &'a str) {
    if a <= b { (a, b) } else { (b, a) }
}

/// Canonical key for a triplet: compare endpoints, keep vertex in center.
fn canonical_triplet<'a>(a: &'a str, b: &'a str, c: &'a str) -> (&'a str, &'a str, &'a str) {
    if a <= c { (a, b, c) } else { (c, b, a) }
}

/// Canonical key for a quartet: compare endpoints (a vs d), keep center order.
fn canonical_quartet<'a>(
    a: &'a str, b: &'a str, c: &'a str, d: &'a str,
) -> (&'a str, &'a str, &'a str, &'a str) {
    if a <= d { (a, b, c, d) } else { (d, c, b, a) }
}

impl ForceFieldParams {
    /// Find bond params for atom type pair. Tries canonical ordering.
    pub fn find_bond(&self, type_a: &str, type_b: &str) -> Option<BondParams> {
        let (ca, cb) = canonical_pair(type_a, type_b);
        self.bond_params
            .iter()
            .find(|(a, b, _)| {
                let (fa, fb) = canonical_pair(a, b);
                fa == ca && fb == cb
            })
            .map(|(_, _, p)| *p)
    }

    /// Find angle params for atom type triplet. Tries canonical ordering.
    pub fn find_angle(&self, type_a: &str, type_b: &str, type_c: &str) -> Option<AngleParams> {
        let (ca, cb, cc) = canonical_triplet(type_a, type_b, type_c);
        self.angle_params
            .iter()
            .find(|(a, b, c, _)| {
                let (fa, fb, fc) = canonical_triplet(a, b, c);
                fa == ca && fb == cb && fc == cc
            })
            .map(|(_, _, _, p)| *p)
    }

    /// Find ALL dihedral params for atom type quartet.
    /// AMBER allows multiple terms per quartet (Fourier series).
    /// Wildcard "X" matches any type.
    pub fn find_dihedrals(
        &self,
        type_a: &str, type_b: &str, type_c: &str, type_d: &str,
    ) -> Vec<DihedralParams> {
        let (ca, cb, cc, cd) = canonical_quartet(type_a, type_b, type_c, type_d);
        let mut results = Vec::new();

        // Exact match first
        for (a, b, c, d, p) in &self.dihedral_params {
            let (fa, fb, fc, fd) = canonical_quartet(a, b, c, d);
            if fa == ca && fb == cb && fc == cc && fd == cd {
                results.push(*p);
            }
        }

        // Wildcard match (X-B-C-X) if no exact match
        if results.is_empty() {
            for (a, b, c, d, p) in &self.dihedral_params {
                let (fa, fb, fc, fd) = canonical_quartet(a, b, c, d);
                let a_match = fa == "X" || fa == ca;
                let b_match = fb == cb;
                let c_match = fc == cc;
                let d_match = fd == "X" || fd == cd;
                if a_match && b_match && c_match && d_match {
                    results.push(*p);
                }
            }
        }

        results
    }

    /// Find LJ params for an atom type.
    pub fn find_lj(&self, type_name: &str) -> Option<LjParams> {
        self.lj_params
            .iter()
            .find(|(name, _)| name == type_name)
            .map(|(_, p)| p.clone())
    }

    /// Find mass for an atom type.
    pub fn find_mass(&self, type_name: &str) -> Option<f64> {
        self.atom_types
            .iter()
            .find(|at| at.name == type_name)
            .map(|at| at.mass)
    }
}

// ─── Assignment ───────────────────────────────────────────────────────────

/// Assign force field parameters to an existing topology.
///
/// Requires `type_names` to map each atom index to its string type name.
/// Updates bond/angle/dihedral params in-place. Returns list of warnings
/// for missing parameters (non-fatal for partial assignment).
///
/// # Errors
///
/// Returns `Err` if a bond/angle has no matching FF entry and no fallback.
pub fn assign_parameters(
    topology: &mut Topology,
    ff: &ForceFieldParams,
    type_names: &[&str],
) -> Result<Vec<String>, String> {
    if type_names.len() != topology.n_atoms {
        return Err(format!(
            "type_names length {} != topology.n_atoms {}",
            type_names.len(), topology.n_atoms,
        ));
    }

    let mut warnings = Vec::new();

    // Assign bond params
    for bond in &mut topology.bonds {
        let (i, j) = (bond.0 as usize, bond.1 as usize);
        let ti = type_names[i];
        let tj = type_names[j];
        if let Some(p) = ff.find_bond(ti, tj) {
            bond.2 = p;
        } else {
            warnings.push(format!("no bond params for {ti}-{tj} (atoms {i}-{j})"));
        }
    }

    // Assign angle params
    for angle in &mut topology.angles {
        let (i, j, k) = (angle.0 as usize, angle.1 as usize, angle.2 as usize);
        let (ti, tj, tk) = (type_names[i], type_names[j], type_names[k]);
        if let Some(p) = ff.find_angle(ti, tj, tk) {
            angle.3 = p;
        } else {
            warnings.push(format!("no angle params for {ti}-{tj}-{tk} (atoms {i}-{j}-{k})"));
        }
    }

    // Assign dihedral params — replace existing with FF terms
    let old_dihedrals = std::mem::take(&mut topology.dihedrals);
    for (i, j, k, l, existing) in old_dihedrals {
        let (ti, tj, tk, tl) = (
            type_names[i as usize], type_names[j as usize],
            type_names[k as usize], type_names[l as usize],
        );
        let ff_params = ff.find_dihedrals(ti, tj, tk, tl);
        if ff_params.is_empty() {
            // Keep existing param as fallback
            topology.dihedrals.push((i, j, k, l, existing));
            warnings.push(format!("no dihedral params for {ti}-{tj}-{tk}-{tl}, keeping default"));
        } else {
            for p in ff_params {
                topology.dihedrals.push((i, j, k, l, p));
            }
        }
    }

    Ok(warnings)
}

// ─── Tests ────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_ff() -> ForceFieldParams {
        ForceFieldParams {
            atom_types: vec![
                AtomTypeInfo { name: "CT".into(), mass: 12.01 },
                AtomTypeInfo { name: "N".into(), mass: 14.01 },
                AtomTypeInfo { name: "HC".into(), mass: 1.008 },
            ],
            bond_params: vec![
                ("CT".into(), "CT".into(), BondParams { r0: 1.526, k: 310.0 }),
                ("CT".into(), "N".into(), BondParams { r0: 1.449, k: 337.0 }),
            ],
            angle_params: vec![
                ("CT".into(), "CT".into(), "N".into(), AngleParams { theta0: 1.9199, k: 80.0 }),
            ],
            dihedral_params: vec![
                ("X".into(), "CT".into(), "CT".into(), "X".into(), DihedralParams { k: 0.156, n: 3, delta: 0.0 }),
            ],
            lj_params: vec![
                ("CT".into(), LjParams { sigma: 1.908, epsilon: 0.1094 }),
                ("N".into(), LjParams { sigma: 1.824, epsilon: 0.17 }),
            ],
        }
    }

    #[test]
    fn find_bond_canonical_order() {
        let ff = sample_ff();
        let p = ff.find_bond("N", "CT").unwrap();
        assert!((p.r0 - 1.449).abs() < 1e-6);
        assert!((p.k - 337.0).abs() < 1e-6);
    }

    #[test]
    fn find_bond_missing_returns_none() {
        let ff = sample_ff();
        assert!(ff.find_bond("CT", "O").is_none());
    }

    #[test]
    fn find_angle_canonical() {
        let ff = sample_ff();
        // N-CT-CT should match CT-CT-N via canonical triplet
        let p = ff.find_angle("N", "CT", "CT").unwrap();
        assert!((p.theta0 - 1.9199).abs() < 1e-6);
    }

    #[test]
    fn find_dihedral_wildcard() {
        let ff = sample_ff();
        let params = ff.find_dihedrals("HC", "CT", "CT", "N");
        assert_eq!(params.len(), 1);
        assert_eq!(params[0].n, 3);
    }

    #[test]
    fn assign_parameters_updates_topology() {
        let ff = sample_ff();
        let mut topo = Topology::new(3);
        topo.add_bond(0, 1, BondParams { r0: 0.0, k: 0.0 });
        topo.add_bond(1, 2, BondParams { r0: 0.0, k: 0.0 });
        let type_names = ["CT", "CT", "N"];
        let warnings = assign_parameters(&mut topo, &ff, &type_names).unwrap();
        // Bond 0-1 (CT-CT) should be updated
        assert!((topo.bonds[0].2.r0 - 1.526).abs() < 1e-6);
        // Bond 1-2 (CT-N) should be updated
        assert!((topo.bonds[1].2.r0 - 1.449).abs() < 1e-6);
        assert!(warnings.is_empty(), "unexpected warnings: {warnings:?}");
    }

    #[test]
    fn assign_parameters_wrong_length_errors() {
        let ff = sample_ff();
        let mut topo = Topology::new(3);
        let type_names: &[&str] = &["CT", "CT"]; // wrong length
        let result = assign_parameters(&mut topo, &ff, type_names);
        assert!(result.is_err());
    }

    #[test]
    fn assign_parameters_missing_warns() {
        let ff = sample_ff();
        let mut topo = Topology::new(2);
        topo.add_bond(0, 1, BondParams { r0: 0.0, k: 0.0 });
        let type_names = ["CT", "O"]; // O not in FF
        let warnings = assign_parameters(&mut topo, &ff, &type_names).unwrap();
        assert!(!warnings.is_empty());
    }

    #[test]
    fn find_lj_by_type() {
        let ff = sample_ff();
        let lj = ff.find_lj("CT").unwrap();
        assert!((lj.sigma - 1.908).abs() < 1e-6);
    }

    #[test]
    fn find_mass_by_type() {
        let ff = sample_ff();
        assert!((ff.find_mass("N").unwrap() - 14.01).abs() < 1e-6);
        assert!(ff.find_mass("ZZ").is_none());
    }
}
