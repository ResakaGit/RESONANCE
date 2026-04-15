//! AP-6a/b: ejecución headless del `autopoietic_lab` (ADR-042).
//! AP-6a/b: headless execution of `autopoietic_lab` (ADR-042).
//!
//! Recibe una `Cli` ya parseada (la parsing vive en `main.rs`) y ejecuta la
//! simulación determinística, escribiendo `SoupReport` a JSON + DOT opcional.
//! Sin Bevy, sin GPU — se usa en CI, validación regulatoria y benchmarks.

use std::fs;

use resonance::layers::reaction_network::ReactionNetwork;
use resonance::use_cases::experiments::autopoiesis::{run_soup, run_soup_with_network};

use super::Cli;

pub(crate) fn execute(cli: &Cli) -> Result<(), String> {
    if !cli.quiet {
        let net_label = cli.network.as_ref()
            .map(|p| format!("network={p:?}"))
            .unwrap_or_else(|| format!("reactions={}", cli.config.n_reactions));
        eprintln!(
            "autopoietic_lab: seed={} ticks={} grid={}x{} species={} {net_label} food={}",
            cli.config.seed, cli.config.ticks, cli.config.grid.0, cli.config.grid.1,
            cli.config.n_species, cli.config.food_size,
        );
    }

    let report = match &cli.network {
        Some(path) => {
            let text = fs::read_to_string(path)
                .map_err(|e| format!("read {path:?}: {e}"))?;
            let net = ReactionNetwork::from_ron_str(&text)
                .map_err(|e| format!("parse {path:?}: {e:?}"))?;
            run_soup_with_network(&cli.config, net)
        }
        None => run_soup(&cli.config),
    };
    let json = report.to_json().map_err(|e| format!("json serialize: {e}"))?;
    fs::write(&cli.out_json, json).map_err(|e| format!("write {:?}: {e}", cli.out_json))?;

    if let Some(dot_path) = &cli.out_dot {
        fs::write(dot_path, report.to_dot()).map_err(|e| format!("write {dot_path:?}: {e}"))?;
    }

    if !cli.quiet {
        eprintln!(
            "  -> closures: initial={} final={} dissipated={:.4} fissions={}",
            report.n_closures_initial, report.n_closures_final,
            report.total_dissipated, report.fission_events.len(),
        );
        eprintln!("  -> wrote {:?}{}", cli.out_json,
            cli.out_dot.as_ref().map(|p| format!(" + {p:?}")).unwrap_or_default(),
        );
    }
    Ok(())
}
