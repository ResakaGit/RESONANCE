//! MD-10: TIP3P water model — constants, topology builder, box placement.
//!
//! TIP3P (Transferable Intermolecular Potential, 3-Point): simplest rigid water.
//! 3 sites (O + 2H), fixed geometry, partial charges.
//! Rigid geometry enforced by high-k harmonic bonds (true SHAKE in MD-11).
//!
//! Axiom 1: water is energy (qe) with specific dissipation properties.
//! Axiom 4: dissipation via thermostat coupling, not internal water dynamics.

use crate::batch::topology::{AngleParams, BondParams, ResidueInfo, Topology};

// ─── TIP3P Constants ──────────────────────────────────────────────────────

/// Oxygen partial charge (e).
pub const TIP3P_CHARGE_O: f64 = -0.834;
/// Hydrogen partial charge (e).
pub const TIP3P_CHARGE_H: f64 = 0.417;
/// O-H bond length (Angstrom).
pub const TIP3P_R_OH: f64 = 0.9572;
/// H-O-H angle (degrees).
pub const TIP3P_ANGLE_HOH_DEG: f64 = 104.52;
/// H-O-H angle (radians).
pub const TIP3P_ANGLE_HOH: f64 = TIP3P_ANGLE_HOH_DEG * std::f64::consts::PI / 180.0;
/// Oxygen LJ sigma (Angstrom).
pub const TIP3P_SIGMA_O: f64 = 3.1507;
/// Oxygen LJ epsilon (kcal/mol).
pub const TIP3P_EPSILON_O: f64 = 0.1521;
/// Oxygen mass (amu).
pub const TIP3P_MASS_O: f64 = 15.9994;
/// Hydrogen mass (amu).
pub const TIP3P_MASS_H: f64 = 1.008;

/// High spring constant for rigid O-H bond approximation (before SHAKE).
/// Large enough to keep geometry near-rigid at 2 fs timestep.
const RIGID_BOND_K: f64 = 10_000.0;
/// High spring constant for H-O-H angle approximation.
const RIGID_ANGLE_K: f64 = 2_000.0;

// ─── Topology builder ─────────────────────────────────────────────────────

/// Build topology for `n_waters` TIP3P water molecules.
///
/// Atom ordering per molecule: O, H1, H2 (3 atoms each).
/// Total atoms = 3 * n_waters.
pub fn create_water_topology(n_waters: usize) -> Topology {
    let n_atoms = 3 * n_waters;
    let mut topo = Topology::new(n_atoms);

    let bond_oh = BondParams {
        r0: TIP3P_R_OH,
        k: RIGID_BOND_K,
    };
    let angle_hoh = AngleParams {
        theta0: TIP3P_ANGLE_HOH,
        k: RIGID_ANGLE_K,
    };

    for w in 0..n_waters {
        let o = (3 * w) as u16;
        let h1 = o + 1;
        let h2 = o + 2;

        // Bonds: O-H1, O-H2
        topo.add_bond(o, h1, bond_oh);
        topo.add_bond(o, h2, bond_oh);

        // Angle: H1-O-H2
        topo.add_angle(h1, o, h2, angle_hoh);

        // Residue
        topo.add_residue(ResidueInfo {
            name: *b"HOH\0",
            first_atom: o,
            atom_count: 3,
        });

        // Atom types: 0 = O, 1 = H
        topo.atom_types[o as usize] = 0;
        topo.atom_types[h1 as usize] = 1;
        topo.atom_types[h2 as usize] = 1;
    }

    topo
}

/// Return per-atom masses for `n_waters` TIP3P molecules.
///
/// Ordering: [O, H, H, O, H, H, ...].
pub fn water_masses(n_waters: usize) -> Vec<f64> {
    let mut masses = Vec::with_capacity(3 * n_waters);
    for _ in 0..n_waters {
        masses.push(TIP3P_MASS_O);
        masses.push(TIP3P_MASS_H);
        masses.push(TIP3P_MASS_H);
    }
    masses
}

/// Return per-atom charges for `n_waters` TIP3P molecules.
///
/// Ordering: [O, H, H, O, H, H, ...].
pub fn water_charges(n_waters: usize) -> Vec<f64> {
    let mut charges = Vec::with_capacity(3 * n_waters);
    for _ in 0..n_waters {
        charges.push(TIP3P_CHARGE_O);
        charges.push(TIP3P_CHARGE_H);
        charges.push(TIP3P_CHARGE_H);
    }
    charges
}

// ─── Water box placement ──────────────────────────────────────────────────

/// Place `n_waters` TIP3P molecules on a cubic grid in a box of side `box_length`.
///
/// Returns 3*n_waters positions in Angstrom. Atom ordering: O, H1, H2 per molecule.
/// Molecules are placed on a cubic lattice; H atoms are offset from O using
/// the TIP3P geometry (bond length + angle).
pub fn place_water_box(n_waters: usize, box_length: f64) -> Vec<[f64; 3]> {
    let n_side = (n_waters as f64).cbrt().ceil() as usize;
    let spacing = box_length / n_side as f64;

    // H offsets from O in local frame (O at origin, bisector along +y)
    let half_angle = TIP3P_ANGLE_HOH / 2.0;
    let h_dx = TIP3P_R_OH * half_angle.sin();
    let h_dy = TIP3P_R_OH * half_angle.cos();

    let mut positions = Vec::with_capacity(3 * n_waters);
    let mut placed = 0;

    for ix in 0..n_side {
        for iy in 0..n_side {
            for iz in 0..n_side {
                if placed >= n_waters {
                    break;
                }
                let ox = (ix as f64 + 0.5) * spacing;
                let oy = (iy as f64 + 0.5) * spacing;
                let oz = (iz as f64 + 0.5) * spacing;

                // Oxygen
                positions.push([ox, oy, oz]);
                // H1: +dx, +dy from O
                positions.push([ox + h_dx, oy + h_dy, oz]);
                // H2: -dx, +dy from O
                positions.push([ox - h_dx, oy + h_dy, oz]);

                placed += 1;
            }
        }
    }

    positions
}

/// Expected H-H distance from TIP3P geometry.
pub fn tip3p_hh_distance() -> f64 {
    let half_angle = TIP3P_ANGLE_HOH / 2.0;
    2.0 * TIP3P_R_OH * half_angle.sin()
}

// ─── Tests ────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn charge_neutrality() {
        let total = TIP3P_CHARGE_O + 2.0 * TIP3P_CHARGE_H;
        assert!(total.abs() < 1e-10, "water must be neutral: {total}");
    }

    #[test]
    fn topology_counts() {
        let n = 10;
        let topo = create_water_topology(n);
        assert_eq!(topo.n_atoms, 30);
        assert_eq!(topo.bonds.len(), 20, "2 bonds per water");
        assert_eq!(topo.angles.len(), 10, "1 angle per water");
        assert_eq!(topo.residues.len(), 10);
    }

    #[test]
    fn topology_atom_ordering() {
        let topo = create_water_topology(2);
        // First water: O=0, H1=1, H2=2
        assert_eq!(topo.bonds[0].0, 0); // O
        assert_eq!(topo.bonds[0].1, 1); // H1
        assert_eq!(topo.bonds[1].0, 0); // O
        assert_eq!(topo.bonds[1].1, 2); // H2
        // Second water: O=3, H1=4, H2=5
        assert_eq!(topo.bonds[2].0, 3);
        assert_eq!(topo.bonds[2].1, 4);
    }

    #[test]
    fn topology_bond_params() {
        let topo = create_water_topology(1);
        assert!((topo.bonds[0].2.r0 - TIP3P_R_OH).abs() < 1e-4);
        assert!((topo.bonds[0].2.k - RIGID_BOND_K).abs() < 1e-4);
    }

    #[test]
    fn topology_angle_params() {
        let topo = create_water_topology(1);
        assert!((topo.angles[0].3.theta0 - TIP3P_ANGLE_HOH).abs() < 1e-3);
    }

    #[test]
    fn water_box_positions_in_bounds() {
        let n = 27;
        let box_l = 30.0;
        let pos = place_water_box(n, box_l);
        assert_eq!(pos.len(), 81, "27 waters * 3 atoms");
        for p in &pos {
            for &c in p {
                assert!(c >= 0.0 && c <= box_l + 2.0, "position out of box: {c}");
            }
        }
    }

    #[test]
    fn water_box_oh_distance() {
        let pos = place_water_box(1, 10.0);
        let o = pos[0];
        let h1 = pos[1];
        let d = ((h1[0] - o[0]).powi(2) + (h1[1] - o[1]).powi(2) + (h1[2] - o[2]).powi(2)).sqrt();
        assert!(
            (d - TIP3P_R_OH).abs() < 1e-10,
            "O-H distance {d} != expected {}", TIP3P_R_OH,
        );
    }

    #[test]
    fn water_box_hoh_angle() {
        let pos = place_water_box(1, 10.0);
        let o = pos[0];
        let h1 = pos[1];
        let h2 = pos[2];
        // Vectors O->H1 and O->H2
        let v1 = [h1[0] - o[0], h1[1] - o[1], h1[2] - o[2]];
        let v2 = [h2[0] - o[0], h2[1] - o[1], h2[2] - o[2]];
        let dot: f64 = v1.iter().zip(v2.iter()).map(|(a, b)| a * b).sum();
        let mag1: f64 = v1.iter().map(|x| x * x).sum::<f64>().sqrt();
        let mag2: f64 = v2.iter().map(|x| x * x).sum::<f64>().sqrt();
        let angle = (dot / (mag1 * mag2)).acos();
        assert!(
            (angle - TIP3P_ANGLE_HOH).abs() < 1e-10,
            "H-O-H angle {:.2}° != expected {:.2}°",
            angle.to_degrees(), TIP3P_ANGLE_HOH_DEG,
        );
    }

    #[test]
    fn water_hh_distance_consistent() {
        let pos = place_water_box(1, 10.0);
        let h1 = pos[1];
        let h2 = pos[2];
        let d = ((h2[0] - h1[0]).powi(2) + (h2[1] - h1[1]).powi(2) + (h2[2] - h1[2]).powi(2)).sqrt();
        let expected = tip3p_hh_distance();
        assert!((d - expected).abs() < 1e-10, "H-H distance {d} != {expected}");
    }

    #[test]
    fn water_masses_length() {
        let m = water_masses(5);
        assert_eq!(m.len(), 15);
        assert!((m[0] - TIP3P_MASS_O).abs() < 1e-6);
        assert!((m[1] - TIP3P_MASS_H).abs() < 1e-6);
    }

    #[test]
    fn water_charges_neutral_per_molecule() {
        let c = water_charges(3);
        assert_eq!(c.len(), 9);
        for w in 0..3 {
            let sum = c[3 * w] + c[3 * w + 1] + c[3 * w + 2];
            assert!(sum.abs() < 1e-10, "molecule {w} not neutral: {sum}");
        }
    }

    #[test]
    fn exclusion_matrix_water() {
        let topo = create_water_topology(1);
        let excl = topo.build_exclusion_matrix();
        let n = 3;
        // O-H1 bonded
        assert!(excl[0 * n + 1]);
        assert!(excl[1 * n + 0]);
        // O-H2 bonded
        assert!(excl[0 * n + 2]);
        assert!(excl[2 * n + 0]);
        // H1-H2: 1-3 via angle
        assert!(excl[1 * n + 2]);
        assert!(excl[2 * n + 1]);
    }
}
