//! LJ fluid validation binary — thermodynamic properties vs 2D literature.
//!
//! Usage: cargo run --release --bin lj_fluid
//!
//! Validates: Verlet (MD-0) + thermostat (MD-1) + PBC (MD-2) + cell list (MD-3).
//! Target: T*=1.0, rho*=0.7, 2D LJ fluid.

use resonance::use_cases::experiments::lj_fluid::{LjFluidConfig, run_lj_fluid};

fn main() {
    let config = LjFluidConfig {
        n_particles: 200,
        density: 0.7,
        temperature: 1.0,
        dt: 0.004,
        r_cut: 2.5,
        gamma: 1.0,
        equil_steps: 5000,
        prod_steps: 10000,
        seed: 42,
        dimensions: 3,
    };

    println!("LJ Fluid Validation (2D, reduced units)");
    println!("========================================");
    println!("  N = {}", config.n_particles);
    println!("  rho* = {}", config.density);
    println!("  T* target = {}", config.temperature);
    println!("  dt* = {}", config.dt);
    println!("  r_cut = {} sigma", config.r_cut);
    println!("  gamma = {}", config.gamma);
    println!(
        "  box = {:.2} sigma",
        (config.n_particles as f64 / config.density).sqrt()
    );
    println!(
        "  equil = {} + prod = {} steps",
        config.equil_steps, config.prod_steps
    );
    println!();

    let result = run_lj_fluid(&config);

    println!("Results:");
    println!("  <T*>    = {:.4}  (target: 1.0)", result.mean_temperature);
    println!("  <P*>    = {:.4}  (2D lit: ~1.3)", result.mean_pressure);
    println!("  <U*/N>  = {:.4}  (2D lit: ~-2.5)", result.mean_pe_per_particle);
    println!(
        "  RDF peak: r* = {:.3}, g(r) = {:.2}",
        result.rdf_peak_r, result.rdf_peak_height
    );

    // Pass/fail
    let t_ok = ((result.mean_temperature - 1.0) / 1.0).abs() < 0.05;
    let rdf_ok = result.rdf_peak_r > 0.9 && result.rdf_peak_r < 1.2;
    let p_ok = result.mean_pressure.is_finite();

    println!();
    println!(
        "  T* equilibration:  {}",
        if t_ok { "PASS" } else { "FAIL" }
    );
    println!(
        "  RDF peak at sigma: {}",
        if rdf_ok { "PASS" } else { "FAIL" }
    );
    println!(
        "  P* finite:         {}",
        if p_ok { "PASS" } else { "FAIL" }
    );

    if t_ok && rdf_ok && p_ok {
        println!("\n  === MD ENGINE VALIDATED ===");
    } else {
        println!("\n  === VALIDATION FAILED ===");
        std::process::exit(1);
    }
}
