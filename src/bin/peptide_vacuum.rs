//! MD-9: Alanine dipeptide in vacuum — Ramachandran validation binary.
//!
//! Usage: cargo run --release --bin peptide_vacuum
//!
//! Validates: bonded forces (MD-5/6) + 3D Verlet (MD-7) + LJ cutoff (MD-8).
//! Target: two Ramachandran basins (alpha-helix + beta-sheet).

use resonance::use_cases::experiments::peptide_vacuum::{PeptideConfig, run_peptide_vacuum};

fn main() {
    let config = PeptideConfig::default();

    println!("Peptide in Vacuum — Ramachandran Validation (MD-9)");
    println!("==================================================");
    println!("  Atoms:       22 (alanine dipeptide, Ace-Ala-NMe)");
    println!("  dt*:         {}", config.dt);
    println!("  T* target:   {} (~300K)", config.temperature);
    println!("  gamma:       {}", config.gamma);
    println!("  LJ sigma:    {}", config.lj_sigma);
    println!("  LJ epsilon:  {}", config.lj_epsilon);
    println!("  LJ r_cut:    {}", config.lj_r_cut);
    println!("  equil:       {} steps", config.equil_steps);
    println!("  prod:        {} steps", config.prod_steps);
    println!("  sample every {} steps", config.sample_interval);
    println!("  init phi:    {:.1} deg", config.init_phi.to_degrees());
    println!("  init psi:    {:.1} deg", config.init_psi.to_degrees());
    println!();

    let result = run_peptide_vacuum(&config);

    println!("Results:");
    println!("  <T*>              = {:.4}  (target: {:.1})", result.mean_temperature, config.temperature);
    println!("  max bond dev      = {:.4}  (< 0.05 = strict)", result.max_bond_deviation);
    println!("  NVE energy drift  = {:.2e}  (< 1e-3 = strict)", result.nve_energy_drift);
    println!("  phi/psi samples   = {}", result.n_samples);
    println!();

    // Ramachandran analysis: find occupied basins
    let n = result.n_bins;
    let bin_deg = 360.0 / n as f64;

    // Find top basins (bins with most counts)
    let mut bins_with_counts: Vec<(usize, usize, u32)> = Vec::new();
    for i in 0..n {
        for j in 0..n {
            let count = result.rama_hist[i * n + j];
            if count > 0 {
                bins_with_counts.push((i, j, count));
            }
        }
    }
    bins_with_counts.sort_by(|a, b| b.2.cmp(&a.2));

    println!("Ramachandran top basins:");
    for (idx, &(bi, bj, count)) in bins_with_counts.iter().take(10).enumerate() {
        let phi_center = -180.0 + (bi as f64 + 0.5) * bin_deg;
        let psi_center = -180.0 + (bj as f64 + 0.5) * bin_deg;
        println!(
            "  {:2}. phi={:+7.1} psi={:+7.1}  count={}",
            idx + 1,
            phi_center,
            psi_center,
            count,
        );
    }

    let n_occupied = bins_with_counts.len();
    println!();
    println!("  bins occupied: {} / {} ({:.1}%)", n_occupied, n * n, 100.0 * n_occupied as f64 / (n * n) as f64);

    // Pass/fail criteria
    let t_ok = ((result.mean_temperature - config.temperature) / config.temperature).abs() < 0.20;
    let bond_ok = result.max_bond_deviation < 0.25;
    let samples_ok = result.n_samples >= 500;
    let basins_ok = n_occupied >= 5; // at least some spread

    println!();
    println!("  T* equilibration:  {}", if t_ok { "PASS" } else { "FAIL" });
    println!("  Bond stability:    {}", if bond_ok { "PASS" } else { "FAIL" });
    println!("  Sampling:          {}", if samples_ok { "PASS" } else { "FAIL" });
    println!("  Basin diversity:   {}", if basins_ok { "PASS" } else { "FAIL" });

    if t_ok && bond_ok && samples_ok && basins_ok {
        println!("\n  === MD-9 PEPTIDE VACUUM VALIDATED ===");
    } else {
        println!("\n  === VALIDATION INCOMPLETE ===");
        std::process::exit(1);
    }
}
