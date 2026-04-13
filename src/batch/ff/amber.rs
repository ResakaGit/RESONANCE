//! AMBER `.dat` format parser — minimal subset for MD-13.
//!
//! Parses: MASS, BOND, ANGLE, DIHE, NONBON sections.
//! Not a full AMBER suite — only what's needed for peptide simulations.
//!
//! Format reference: AMBER parm.dat (fixed-width fields, section headers).

use super::{AtomTypeInfo, ForceFieldParams, LjParams};
use crate::batch::topology::{AngleParams, BondParams, DihedralParams};

/// Current parser section.
#[derive(Clone, Copy, PartialEq)]
enum Section {
    None,
    Mass,
    Bond,
    Angle,
    Dihe,
    Nonbon,
}

/// Parse AMBER `.dat` format text into `ForceFieldParams`.
///
/// Section headers are case-insensitive: `MASS`, `BOND`, `ANGLE`, `DIHE`, `NONBON`.
/// Blank lines and lines starting with `#` are skipped.
/// A line starting with a known section keyword switches the parser state.
///
/// # Format
///
/// ```text
/// MASS
/// CT  12.01
/// N   14.01
///
/// BOND
/// CT-CT   310.0    1.526
/// CT-N    337.0    1.449
///
/// ANGLE
/// CT-CT-N    80.0    110.1
///
/// DIHE
/// X -CT-CT-X    1    0.156    0.0    3.0
///
/// NONBON
/// CT    1.908    0.1094
/// ```
pub fn load_amber_params(data: &str) -> Result<ForceFieldParams, String> {
    let mut ff = ForceFieldParams::default();
    let mut section = Section::None;

    for (line_num, raw_line) in data.lines().enumerate() {
        let line = raw_line.trim();

        // Skip empty lines and comments
        if line.is_empty() || line.starts_with('#') {
            continue;
        }

        // Check for section header
        let upper = line.to_uppercase();
        if let Some(new_section) = match_section_header(&upper) {
            section = new_section;
            continue;
        }

        // Parse based on current section
        match section {
            Section::None => {
                // Ignore lines before any section header (title/comments)
            }
            Section::Mass => parse_mass_line(line, line_num, &mut ff)?,
            Section::Bond => parse_bond_line(line, line_num, &mut ff)?,
            Section::Angle => parse_angle_line(line, line_num, &mut ff)?,
            Section::Dihe => parse_dihe_line(line, line_num, &mut ff)?,
            Section::Nonbon => parse_nonbon_line(line, line_num, &mut ff)?,
        }
    }

    Ok(ff)
}

fn match_section_header(upper: &str) -> Option<Section> {
    // Match section header: line is exactly or starts with the keyword
    if upper == "MASS" || upper.starts_with("MASS ") {
        Some(Section::Mass)
    } else if upper == "BOND" || upper.starts_with("BOND ") {
        Some(Section::Bond)
    } else if upper == "ANGLE" || upper.starts_with("ANGLE") {
        Some(Section::Angle)
    } else if upper == "DIHE" || upper.starts_with("DIHE") {
        Some(Section::Dihe)
    } else if upper == "NONBON" || upper.starts_with("NONBON") {
        Some(Section::Nonbon)
    } else {
        None
    }
}

/// MASS line: `TypeName  mass`
fn parse_mass_line(
    line: &str, line_num: usize, ff: &mut ForceFieldParams,
) -> Result<(), String> {
    let parts: Vec<&str> = line.split_whitespace().collect();
    if parts.len() < 2 {
        return Err(format!("line {}: MASS expects 'type mass', got: {line}", line_num + 1));
    }
    let name = parts[0].to_string();
    let mass: f64 = parts[1]
        .parse()
        .map_err(|_| format!("line {}: bad mass value: {}", line_num + 1, parts[1]))?;
    ff.atom_types.push(AtomTypeInfo { name, mass });
    Ok(())
}

/// BOND line: `Type1-Type2  k  r_eq`
fn parse_bond_line(
    line: &str, line_num: usize, ff: &mut ForceFieldParams,
) -> Result<(), String> {
    let (types_str, rest) = split_types_and_values(line, 2, line_num, "BOND")?;
    let types = parse_type_pair(&types_str)?;
    let k: f64 = rest[0]
        .parse()
        .map_err(|_| format!("line {}: bad bond k: {}", line_num + 1, rest[0]))?;
    let r0: f64 = rest[1]
        .parse()
        .map_err(|_| format!("line {}: bad bond r_eq: {}", line_num + 1, rest[1]))?;
    ff.bond_params.push((types.0, types.1, BondParams { r0, k }));
    Ok(())
}

/// ANGLE line: `Type1-Type2-Type3  k  theta_eq(degrees)`
fn parse_angle_line(
    line: &str, line_num: usize, ff: &mut ForceFieldParams,
) -> Result<(), String> {
    let (types_str, rest) = split_types_and_values(line, 2, line_num, "ANGLE")?;
    let types = parse_type_triplet(&types_str)?;
    let k: f64 = rest[0]
        .parse()
        .map_err(|_| format!("line {}: bad angle k: {}", line_num + 1, rest[0]))?;
    let theta_deg: f64 = rest[1]
        .parse()
        .map_err(|_| format!("line {}: bad angle theta: {}", line_num + 1, rest[1]))?;
    let theta0 = theta_deg * std::f64::consts::PI / 180.0;
    ff.angle_params.push((types.0, types.1, types.2, AngleParams { theta0, k }));
    Ok(())
}

/// DIHE line: `Type1-Type2-Type3-Type4  divider  barrier  phase(deg)  periodicity`
fn parse_dihe_line(
    line: &str, line_num: usize, ff: &mut ForceFieldParams,
) -> Result<(), String> {
    let (types_str, rest) = split_types_and_values(line, 4, line_num, "DIHE")?;
    let types = parse_type_quartet(&types_str)?;
    let divider: f64 = rest[0]
        .parse()
        .map_err(|_| format!("line {}: bad dihedral divider: {}", line_num + 1, rest[0]))?;
    let barrier: f64 = rest[1]
        .parse()
        .map_err(|_| format!("line {}: bad dihedral barrier: {}", line_num + 1, rest[1]))?;
    let phase_deg: f64 = rest[2]
        .parse()
        .map_err(|_| format!("line {}: bad dihedral phase: {}", line_num + 1, rest[2]))?;
    let periodicity: f64 = rest[3]
        .parse()
        .map_err(|_| format!("line {}: bad dihedral periodicity: {}", line_num + 1, rest[3]))?;

    let k = if divider.abs() > 1e-12 { barrier / divider } else { barrier };
    let delta = phase_deg * std::f64::consts::PI / 180.0;
    let n = periodicity.abs() as u8;

    ff.dihedral_params.push((types.0, types.1, types.2, types.3, DihedralParams { k, n, delta }));
    Ok(())
}

/// NONBON line: `TypeName  sigma(or R*)  epsilon`
fn parse_nonbon_line(
    line: &str, line_num: usize, ff: &mut ForceFieldParams,
) -> Result<(), String> {
    let parts: Vec<&str> = line.split_whitespace().collect();
    if parts.len() < 3 {
        return Err(format!("line {}: NONBON expects 'type sigma epsilon', got: {line}", line_num + 1));
    }
    let name = parts[0].to_string();
    let sigma: f64 = parts[1]
        .parse()
        .map_err(|_| format!("line {}: bad sigma: {}", line_num + 1, parts[1]))?;
    let epsilon: f64 = parts[2]
        .parse()
        .map_err(|_| format!("line {}: bad epsilon: {}", line_num + 1, parts[2]))?;
    ff.lj_params.push((name, LjParams { sigma, epsilon }));
    Ok(())
}

// ─── Helpers ──────────────────────────────────────────────────────────────

/// Split a line into type-spec (dash-separated) and numeric values.
///
/// The type spec is everything before the first whitespace-separated numeric token.
/// Handles AMBER's flexible formatting: `CT-CT  310.0  1.526` or `CT -CT   310.0  1.526`.
fn split_types_and_values(
    line: &str, expected_values: usize, line_num: usize, section: &str,
) -> Result<(String, Vec<String>), String> {
    // Strategy: collect all whitespace-separated tokens. The last `expected_values`
    // tokens are numeric values. Everything before is the type specification.
    let tokens: Vec<&str> = line.split_whitespace().collect();
    if tokens.len() < expected_values + 1 {
        return Err(format!(
            "line {}: {section} needs type-spec + {expected_values} values, got: {line}",
            line_num + 1,
        ));
    }

    let split_point = tokens.len() - expected_values;
    let type_str = tokens[..split_point].join("");
    let values: Vec<String> = tokens[split_point..].iter().map(|s| s.to_string()).collect();

    // Clean up type string: remove spaces that AMBER sometimes inserts around dashes
    let clean_types = type_str.replace(' ', "");

    Ok((clean_types, values))
}

/// Parse "A-B" into (A, B).
fn parse_type_pair(s: &str) -> Result<(String, String), String> {
    let parts: Vec<&str> = s.split('-').collect();
    if parts.len() != 2 {
        return Err(format!("expected type1-type2, got: {s}"));
    }
    Ok((parts[0].trim().to_string(), parts[1].trim().to_string()))
}

/// Parse "A-B-C" into (A, B, C).
fn parse_type_triplet(s: &str) -> Result<(String, String, String), String> {
    let parts: Vec<&str> = s.split('-').collect();
    if parts.len() != 3 {
        return Err(format!("expected type1-type2-type3, got: {s}"));
    }
    Ok((parts[0].trim().to_string(), parts[1].trim().to_string(), parts[2].trim().to_string()))
}

/// Parse "A-B-C-D" into (A, B, C, D).
fn parse_type_quartet(s: &str) -> Result<(String, String, String, String), String> {
    let parts: Vec<&str> = s.split('-').collect();
    if parts.len() != 4 {
        return Err(format!("expected type1-type2-type3-type4, got: {s}"));
    }
    Ok((
        parts[0].trim().to_string(), parts[1].trim().to_string(),
        parts[2].trim().to_string(), parts[3].trim().to_string(),
    ))
}

// ─── Tests ────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE_AMBER: &str = "\
# Minimal AMBER-style force field for testing
MASS
CT  12.01
N   14.01
HC  1.008
C   12.01
O   16.00

BOND
CT-CT   310.0    1.526
CT-N    337.0    1.449
CT-HC   340.0    1.090
C -O    570.0    1.229

ANGLE
CT-CT-N    80.0    109.7
CT-CT-HC   50.0    109.5
HC-CT-HC   35.0    109.5

DIHE
X -CT-CT-X    1    0.156    0.0    3.0
HC-CT-CT-N    1    0.100  180.0    1.0

NONBON
CT    1.908    0.1094
N     1.824    0.1700
HC    1.487    0.0157
C     1.908    0.0860
O     1.661    0.2100
";

    #[test]
    fn parse_mass_section() {
        let ff = load_amber_params(SAMPLE_AMBER).unwrap();
        assert_eq!(ff.atom_types.len(), 5);
        let ct = ff.atom_types.iter().find(|a| a.name == "CT").unwrap();
        assert!((ct.mass - 12.01).abs() < 1e-6);
        let hc = ff.atom_types.iter().find(|a| a.name == "HC").unwrap();
        assert!((hc.mass - 1.008).abs() < 1e-6);
    }

    #[test]
    fn parse_bond_section() {
        let ff = load_amber_params(SAMPLE_AMBER).unwrap();
        assert_eq!(ff.bond_params.len(), 4);
        let ct_ct = ff.find_bond("CT", "CT").unwrap();
        assert!((ct_ct.k - 310.0).abs() < 1e-6);
        assert!((ct_ct.r0 - 1.526).abs() < 1e-3);
    }

    #[test]
    fn parse_bond_with_spaces_in_types() {
        // AMBER format: "C -O" with space before dash
        let ff = load_amber_params(SAMPLE_AMBER).unwrap();
        let c_o = ff.find_bond("C", "O").unwrap();
        assert!((c_o.k - 570.0).abs() < 1e-6);
        assert!((c_o.r0 - 1.229).abs() < 1e-3);
    }

    #[test]
    fn parse_angle_section() {
        let ff = load_amber_params(SAMPLE_AMBER).unwrap();
        assert_eq!(ff.angle_params.len(), 3);
        let ct_ct_n = ff.find_angle("CT", "CT", "N").unwrap();
        assert!((ct_ct_n.k - 80.0).abs() < 1e-6);
        // 109.7 degrees in radians
        let expected = 109.7_f64 * std::f64::consts::PI / 180.0;
        assert!((ct_ct_n.theta0 - expected).abs() < 1e-3);
    }

    #[test]
    fn parse_dihedral_section() {
        let ff = load_amber_params(SAMPLE_AMBER).unwrap();
        assert_eq!(ff.dihedral_params.len(), 2);
        // Wildcard: X-CT-CT-X
        let params = ff.find_dihedrals("HC", "CT", "CT", "HC");
        assert!(!params.is_empty());
        assert_eq!(params[0].n, 3);
    }

    #[test]
    fn parse_dihedral_exact_over_wildcard() {
        let ff = load_amber_params(SAMPLE_AMBER).unwrap();
        // Exact match HC-CT-CT-N should take precedence over X-CT-CT-X
        let params = ff.find_dihedrals("HC", "CT", "CT", "N");
        assert_eq!(params.len(), 1);
        assert_eq!(params[0].n, 1, "exact match should win over wildcard");
    }

    #[test]
    fn parse_nonbon_section() {
        let ff = load_amber_params(SAMPLE_AMBER).unwrap();
        assert_eq!(ff.lj_params.len(), 5);
        let ct_lj = ff.find_lj("CT").unwrap();
        assert!((ct_lj.sigma - 1.908).abs() < 1e-6);
        assert!((ct_lj.epsilon - 0.1094).abs() < 1e-6);
    }

    #[test]
    fn empty_input_produces_empty_ff() {
        let ff = load_amber_params("").unwrap();
        assert!(ff.atom_types.is_empty());
        assert!(ff.bond_params.is_empty());
    }

    #[test]
    fn comments_and_blank_lines_skipped() {
        let input = "# comment\n\nMASS\n# another comment\nCT  12.01\n\n";
        let ff = load_amber_params(input).unwrap();
        assert_eq!(ff.atom_types.len(), 1);
    }

    #[test]
    fn bad_mass_value_errors() {
        let input = "MASS\nCT  notanumber\n";
        let result = load_amber_params(input);
        assert!(result.is_err());
    }

    #[test]
    fn roundtrip_assign_with_parsed_ff() {
        let ff = load_amber_params(SAMPLE_AMBER).unwrap();
        let mut topo = crate::batch::topology::Topology::new(3);
        // CT(0) - CT(1) - N(2)
        topo.add_bond(0, 1, BondParams { r0: 0.0, k: 0.0 });
        topo.add_bond(1, 2, BondParams { r0: 0.0, k: 0.0 });
        topo.infer_angles_from_bonds(AngleParams { theta0: 0.0, k: 0.0 });

        let type_names = ["CT", "CT", "N"];
        let warnings = super::super::assign_parameters(&mut topo, &ff, &type_names).unwrap();
        assert!(warnings.is_empty(), "warnings: {warnings:?}");

        // Bond params updated
        assert!((topo.bonds[0].2.k - 310.0).abs() < 1e-6, "CT-CT k");
        assert!((topo.bonds[1].2.k - 337.0).abs() < 1e-6, "CT-N k");

        // Angle params updated (CT-CT-N)
        assert_eq!(topo.angles.len(), 1);
        assert!((topo.angles[0].3.k - 80.0).abs() < 1e-6, "angle k");
    }
}
