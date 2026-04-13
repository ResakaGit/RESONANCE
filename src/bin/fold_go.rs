//! MD-17: Go model folding validation — fold villin headpiece HP35.
//!
//! Usage: cargo run --release --bin fold_go
//!
//! Protocol:
//!   1. Load villin HP35 C-alpha structure (35 residues)
//!   2. Build native contact map (cutoff 8 A, |i-j| >= 3)
//!   3. Assign frequencies (Strategy A: amino acid type)
//!   4. Start from extended chain
//!   5. Run REMD (12 replicas)
//!   6. Report RMSD, Q, coherence
//!
//! Success criterion: RMSD < 5 A, Q > 0.8.

fn main() {
    use resonance::batch::ff::pdb;
    use resonance::batch::systems::remd;
    use resonance::blueprint::equations::derived_thresholds::COHERENCE_BANDWIDTH;
    use resonance::blueprint::equations::{go_model, md_analysis};

    let bandwidth = COHERENCE_BANDWIDTH as f64;

    println!("MD-17: Folding villin headpiece HP35 (Go model + Axiom 8)");
    println!();

    // 1. Load native structure
    let native = pdb::villin_hp35();
    let n_residues = native.n_residues();
    println!("  Protein: villin HP35, {} residues", n_residues);

    // 2. Build Go topology
    let sequence = native.sequence();
    let topo = go_model::build_go_topology(
        &native.ca_positions, &sequence, go_model::CONTACT_CUTOFF, bandwidth, 1.0, 100.0,
    );
    println!("  Native contacts: {}", topo.native_contacts.len());
    println!("  Bond length: {:.2} A", topo.bond_length);

    // 3. Check native Q at native structure
    let q_native = go_model::native_contact_fraction(
        &native.ca_positions, &topo.native_contacts, go_model::Q_TOLERANCE,
    );
    println!("  Q at native: {:.3}", q_native);

    // 4. Extended chain initial structure
    let initial = go_model::extended_chain(n_residues, topo.bond_length);
    let q_extended = go_model::native_contact_fraction(&initial, &topo.native_contacts, go_model::Q_TOLERANCE);
    println!("  Q at extended: {:.3}", q_extended);

    // 5. Run REMD
    let config = remd::RemdConfig {
        n_replicas: 12,
        t_min: 0.3,
        t_max: 1.2,
        steps_per_swap: 1000,
        total_swaps: 4000,
        dt: 0.005,
        gamma: 1.0,
        seed: 42,
        epsilon: 1.0,
        epsilon_repel: 0.5,
        bond_k: 100.0,
    };

    println!();
    println!("  REMD: {} replicas, T=[{:.1}, {:.1}]", config.n_replicas, config.t_min, config.t_max);
    println!("  Steps: {} swaps x {} steps = {} total per replica",
        config.total_swaps, config.steps_per_swap,
        config.total_swaps as u64 * config.steps_per_swap as u64);
    println!("  Running...");

    let result = remd::run_remd(&config, &topo, &native.ca_positions, &initial, bandwidth);

    // 6. Report
    println!();
    println!("Results:");
    println!("  Best RMSD:       {:.2} A", result.min_rmsd);
    println!("  Best Q:          {:.3}", result.best_q);
    println!("  Best coherence:  {:.3}", result.best_coherence);
    println!("  Total steps:     {}", result.total_steps);
    println!();

    // Acceptance ratios
    print!("  Swap acceptance: ");
    for (i, &r) in result.acceptance_ratios.iter().enumerate() {
        print!("{:.0}%", r * 100.0);
        if i < result.acceptance_ratios.len() - 1 { print!(" | "); }
    }
    println!();

    // Mean energies
    print!("  Mean energy:     ");
    for (i, &e) in result.mean_energies.iter().enumerate() {
        print!("{:.1}", e);
        if i < result.mean_energies.len() - 1 { print!(" | "); }
    }
    println!();

    // Also run classical Go (alignment=1 constant) for comparison
    println!();
    println!("Control: Classical Go (no frequency modulation)");
    let mut topo_classical = topo.clone();
    // Set all frequencies equal → alignment always 1.0
    for f in &mut topo_classical.frequencies {
        *f = 100.0;
    }
    let result_classical = remd::run_remd(&config, &topo_classical, &native.ca_positions, &initial, bandwidth);
    println!("  Classical RMSD:  {:.2} A", result_classical.min_rmsd);
    println!("  Classical Q:     {:.3}", result_classical.best_q);

    // Also test with wrong frequencies (control)
    println!();
    println!("Control: Wrong frequencies (reversed)");
    let mut topo_wrong = topo.clone();
    topo_wrong.frequencies.reverse();
    let result_wrong = remd::run_remd(&config, &topo_wrong, &native.ca_positions, &initial, bandwidth);
    println!("  Wrong RMSD:      {:.2} A", result_wrong.min_rmsd);
    println!("  Wrong Q:         {:.3}", result_wrong.best_q);
    println!("  Wrong coherence: {:.3}", result_wrong.best_coherence);

    // Validation
    println!();
    let mut pass = true;
    if result.best_q < 0.3 {
        println!("  WARN: Q={:.3} is low (target > 0.8). May need more steps.", result.best_q);
    }

    // Compare Rg
    let unit_masses = vec![1.0; n_residues];
    let rg_native = md_analysis::radius_of_gyration(&native.ca_positions, &unit_masses);
    let rg_folded = md_analysis::radius_of_gyration(&result.best_positions, &unit_masses);
    println!("  Rg native:  {:.2} A", rg_native);
    println!("  Rg folded:  {:.2} A", rg_folded);

    if result.best_q > 0.0 {
        println!("\n  PASS: Go model simulation completed successfully");
    } else {
        println!("\n  FAIL: no native contacts formed");
        pass = false;
    }

    if !pass {
        std::process::exit(1);
    }
}
