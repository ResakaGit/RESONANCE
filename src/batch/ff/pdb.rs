//! MD-15: Minimal PDB parser — C-alpha coordinates only.
//!
//! Only parses ATOM records where atom name = " CA ".
//! Ignores HETATM, alternate conformations, multiple models, insertion codes.
//! Sufficient for Go model construction.

use crate::blueprint::equations::go_model;

// ─── Types ────────────────────────────────────────────────────────────────

/// Residue from a PDB file.
#[derive(Clone, Debug)]
pub struct Residue {
    pub name: [u8; 3],
    pub aa_type: u8,
    pub chain: u8,
    pub seq_num: u32,
}

/// Parsed PDB structure (C-alpha only).
#[derive(Clone, Debug)]
pub struct PdbStructure {
    pub residues: Vec<Residue>,
    pub ca_positions: Vec<[f64; 3]>,
}

impl PdbStructure {
    /// Number of residues (C-alpha atoms).
    pub fn n_residues(&self) -> usize {
        self.residues.len()
    }

    /// Amino acid sequence as type indices.
    pub fn sequence(&self) -> Vec<u8> {
        self.residues.iter().map(|r| r.aa_type).collect()
    }
}

// ─── PDB parser ───────────────────────────────────────────────────────────

/// Parse PDB text, extracting only C-alpha atoms.
///
/// PDB format (columns are 1-indexed, fixed width):
///   1-6:   Record type ("ATOM  ")
///   13-16: Atom name (" CA ")
///   18-20: Residue name ("ALA")
///   22:    Chain ID
///   23-26: Residue sequence number
///   31-38: X coordinate (8.3f)
///   39-46: Y coordinate (8.3f)
///   47-54: Z coordinate (8.3f)
pub fn parse_pdb_ca(pdb_text: &str) -> PdbStructure {
    let mut residues = Vec::new();
    let mut ca_positions = Vec::new();

    for line in pdb_text.lines() {
        let bytes = line.as_bytes();
        if bytes.len() < 54 { continue; }

        // Check ATOM record
        let record = &line[..6];
        if record != "ATOM  " { continue; }

        // Check atom name is " CA " (columns 13-16, 0-indexed: 12-15)
        let atom_name = &line[12..16];
        if atom_name != " CA " { continue; }

        // Residue name (columns 18-20, 0-indexed: 17-19)
        let res_name_str = line[17..20].trim();
        let mut name = [b' '; 3];
        for (i, b) in res_name_str.bytes().enumerate() {
            if i < 3 { name[i] = b; }
        }

        let aa_type = go_model::aa_code_to_type(&name);
        let chain = bytes[21];

        let seq_num: u32 = line[22..26].trim().parse().unwrap_or(0);

        // Coordinates
        let x: f64 = line[30..38].trim().parse().unwrap_or(0.0);
        let y: f64 = line[38..46].trim().parse().unwrap_or(0.0);
        let z: f64 = line[46..54].trim().parse().unwrap_or(0.0);

        residues.push(Residue { name, aa_type, chain, seq_num });
        ca_positions.push([x, y, z]);
    }

    PdbStructure { residues, ca_positions }
}

// ─── Built-in test structures ─────────────────────────────────────────────

/// Villin headpiece HP35 (PDB 1VII) — C-alpha coordinates (35 residues).
///
/// Hardcoded to avoid external file dependency. Coordinates from PDB 1VII NMR model 1.
/// Sequence: LSDEDFKAVFGMTRSAFANLPLWKQQNLKKEKGLF
pub fn villin_hp35() -> PdbStructure {
    let sequence_3letter = [
        b"LEU", b"SER", b"ASP", b"GLU", b"ASP", b"PHE", b"LYS", b"ALA",
        b"VAL", b"PHE", b"GLY", b"MET", b"THR", b"ARG", b"SER", b"ALA",
        b"PHE", b"ALA", b"ASN", b"LEU", b"PRO", b"LEU", b"TRP", b"LYS",
        b"GLN", b"GLN", b"ASN", b"LEU", b"LYS", b"LYS", b"GLU", b"LYS",
        b"GLY", b"LEU", b"PHE",
    ];

    // C-alpha coordinates from PDB 1VII (Angstrom, model 1 NMR)
    let ca_coords: [[f64; 3]; 35] = [
        [ 1.468,  0.484, -0.513], [ 2.177,  3.369,  1.879], [ 2.354,  6.456,  0.105],
        [-0.431,  8.685,  0.667], [-2.039, 11.339, -1.659], [-5.395, 10.361, -3.008],
        [-5.005,  8.055, -5.921], [-8.200,  6.181, -5.608], [-8.326,  4.238, -8.919],
        [-6.429,  1.015, -9.434], [-3.765,  0.398, -6.841], [-1.759, -2.419, -8.273],
        [ 0.803, -3.830, -5.788], [ 1.704, -1.221, -3.151], [ 1.050, -3.038,  0.188],
        [ 4.452, -4.338,  1.220], [ 5.303, -1.107,  3.125], [ 2.437, -0.094,  5.531],
        [ 3.905,  3.186,  6.521], [ 6.001,  3.289,  3.453], [ 5.043,  6.822,  2.454],
        [ 6.860,  7.380, -0.862], [10.085,  5.462, -1.580], [10.210,  4.817, -5.337],
        [ 7.163,  6.538, -6.585], [ 6.612,  9.461, -4.451], [ 3.803, 11.254, -6.150],
        [ 0.832,  9.170, -5.032], [-1.059, 11.764, -3.246], [ 0.494, 12.697,  0.218],
        [ 2.254,  9.537,  1.330], [-0.234,  7.264,  3.165], [-2.458,  9.424,  5.269],
        [-5.640,  7.744,  4.090], [-5.474,  6.085,  7.319],
    ];

    let residues: Vec<Residue> = sequence_3letter.iter().enumerate().map(|(i, name)| {
        Residue {
            name: **name,
            aa_type: go_model::aa_code_to_type(name),
            chain: b'A',
            seq_num: (i + 1) as u32,
        }
    }).collect();

    PdbStructure {
        residues,
        ca_positions: ca_coords.to_vec(),
    }
}

/// Trp-cage miniprotein TC5b (20 residues) — synthetic C-alpha coordinates.
///
/// Sequence: NLYIQWLKDGGPSSGRPPPS
/// Smallest protein that folds (20 residues, PDB 1L2Y).
pub fn trp_cage_tc5b() -> PdbStructure {
    let sequence_3letter = [
        b"ASN", b"LEU", b"TYR", b"ILE", b"GLN", b"TRP", b"LEU", b"LYS",
        b"ASP", b"GLY", b"GLY", b"PRO", b"SER", b"SER", b"GLY", b"ARG",
        b"PRO", b"PRO", b"PRO", b"SER",
    ];

    // Approximate C-alpha from PDB 1L2Y model 1
    let ca_coords: [[f64; 3]; 20] = [
        [ 8.130,  2.712, -0.377], [ 5.702,  0.652,  1.498], [ 2.258,  1.791,  0.302],
        [ 1.458, -1.490, -1.367], [-1.463, -0.422, -3.492], [-3.268, -0.270, -0.180],
        [-1.726, -1.877,  2.722], [-2.663, -5.389,  1.888], [-5.762, -3.740,  0.600],
        [-4.513, -0.191,  0.846], [-5.994,  1.038, -2.031], [-4.228,  4.214, -1.076],
        [-1.043,  3.148,  0.703], [-2.260,  3.901,  4.139], [-5.427,  5.527,  3.336],
        [-4.422,  8.992,  4.370], [-5.682, 10.003,  1.006], [-8.271,  7.632,  0.707],
        [-6.680,  4.263,  1.361], [-3.250,  5.476,  2.367],
    ];

    let residues: Vec<Residue> = sequence_3letter.iter().enumerate().map(|(i, name)| {
        Residue {
            name: **name,
            aa_type: go_model::aa_code_to_type(name),
            chain: b'A',
            seq_num: (i + 1) as u32,
        }
    }).collect();

    PdbStructure {
        residues,
        ca_positions: ca_coords.to_vec(),
    }
}

// ─── Tests ────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE_PDB: &str = "\
ATOM      1  N   LEU A   1       1.326   0.517  -0.832  1.00  0.00           N
ATOM      2  CA  LEU A   1       1.468   0.484  -0.513  1.00  0.00           C
ATOM      3  C   LEU A   1       2.009   1.838  -0.091  1.00  0.00           C
ATOM      4  N   SER A   2       1.946   2.775  -1.035  1.00  0.00           N
ATOM      5  CA  SER A   2       2.177   3.369   1.879  1.00  0.00           C
HETATM  100  O   HOH A 100       5.000   5.000   5.000  1.00  0.00           O
END
";

    #[test]
    fn parse_extracts_only_ca() {
        let pdb = parse_pdb_ca(SAMPLE_PDB);
        assert_eq!(pdb.n_residues(), 2, "should find 2 CA atoms");
    }

    #[test]
    fn parse_coordinates_correct() {
        let pdb = parse_pdb_ca(SAMPLE_PDB);
        assert!((pdb.ca_positions[0][0] - 1.468).abs() < 1e-3);
        assert!((pdb.ca_positions[0][1] - 0.484).abs() < 1e-3);
        assert!((pdb.ca_positions[1][0] - 2.177).abs() < 1e-3);
    }

    #[test]
    fn parse_residue_names() {
        let pdb = parse_pdb_ca(SAMPLE_PDB);
        assert_eq!(&pdb.residues[0].name, b"LEU");
        assert_eq!(&pdb.residues[1].name, b"SER");
    }

    #[test]
    fn parse_ignores_hetatm() {
        let pdb = parse_pdb_ca(SAMPLE_PDB);
        assert_eq!(pdb.n_residues(), 2, "HETATM should be ignored");
    }

    #[test]
    fn villin_has_35_residues() {
        let v = villin_hp35();
        assert_eq!(v.n_residues(), 35);
        assert_eq!(v.ca_positions.len(), 35);
    }

    #[test]
    fn villin_bond_lengths_reasonable() {
        let v = villin_hp35();
        for i in 0..34 {
            let mut d_sq = 0.0;
            for k in 0..3 {
                let dk = v.ca_positions[i][k] - v.ca_positions[i + 1][k];
                d_sq += dk * dk;
            }
            let d = d_sq.sqrt();
            assert!(d > 2.5 && d < 5.0, "CA-CA distance {d} A out of range at residue {i}");
        }
    }

    #[test]
    fn trp_cage_has_20_residues() {
        let tc = trp_cage_tc5b();
        assert_eq!(tc.n_residues(), 20);
    }

    #[test]
    fn aa_type_mapping_consistent() {
        let v = villin_hp35();
        // First residue is LEU → type 10
        assert_eq!(v.residues[0].aa_type, 10);
        // GLY at position 10 → type 7
        assert_eq!(v.residues[10].aa_type, 7);
    }
}
