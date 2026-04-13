//! MD-14: Solvated alanine dipeptide — integration validation binary.
//!
//! Usage: cargo run --release --bin peptide_solvated
//!
//! Runs alanine dipeptide in explicit TIP3P water with SHAKE + Ewald.
//! Reports: density, hydration shell RDF, phi/psi angles, bond stability.

fn main() {
    let config = resonance::use_cases::experiments::peptide_solvated::SolvatedConfig {
        dt: 0.001,
        gamma: 1.0,
        temperature: 1.0,
        equil_steps: 2_000,
        prod_steps: 5_000,
        sample_interval: 10,
        seed: 42,
        n_waters: 64,
        box_length: 20.0,
        lj_sigma: 1.0,
        lj_epsilon: 1.0,
        r_cut: 6.0,
        ewald_k_max: 3,
        k_coulomb: 0.5,
        init_phi: -1.0,
        init_psi: -0.8,
    };

    println!("MD-14: Solvated alanine dipeptide");
    println!("  Peptide: 22 atoms");
    println!("  Water: {} molecules ({} atoms)", config.n_waters, config.n_waters * 3);
    println!("  Box: {:.1} A", config.box_length);
    println!("  Steps: {} equil + {} prod", config.equil_steps, config.prod_steps);
    println!("  Ewald: alpha={:.3}, k_max={}", 5.0 / config.box_length, config.ewald_k_max);
    println!();

    let result = resonance::use_cases::experiments::peptide_solvated::run_peptide_solvated(&config);

    println!("Results:");
    println!("  Temperature:       {:.3}", result.mean_temperature);
    println!("  Water density:     {:.4} g/cm3", result.water_density);
    println!("  Peptide stable:    {}", result.peptide_stable);
    println!("  Max bond deviation:{:.4}", result.max_bond_deviation);
    println!("  Mean phi:          {:.2} deg", result.mean_phi.to_degrees());
    println!("  Mean psi:          {:.2} deg", result.mean_psi.to_degrees());
    println!();

    // O-O RDF peak
    if let Some((r_peak, g_peak)) = result.oo_rdf.iter()
        .filter(|(r, _)| *r > 1.0)
        .max_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal))
    {
        println!("  O-O RDF peak:      r={:.2} A, g(r)={:.2}", r_peak, g_peak);
    }

    // Validation
    let mut pass = true;
    if result.max_bond_deviation > 1.0 {
        println!("  FAIL: bonds broke (deviation {:.3})", result.max_bond_deviation);
        pass = false;
    }
    if !result.peptide_stable {
        println!("  FAIL: peptide drifted out of box");
        pass = false;
    }
    if result.mean_temperature <= 0.0 {
        println!("  FAIL: temperature collapsed");
        pass = false;
    }

    if pass {
        println!("\n  PASS: solvated peptide stable");
    }
}
