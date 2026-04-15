//! AP-6a/b/d: ejecución headless del `autopoietic_lab` (ADR-042).
//! AP-6a/b/d: headless execution of `autopoietic_lab` (ADR-042).
//!
//! Recibe una `Cli` ya parseada (la parsing vive en `main.rs`) y ejecuta la
//! simulación determinística, escribiendo `SoupReport` a JSON + DOT opcional
//! + PPM snapshot(s) opcional.  Sin Bevy, sin GPU — CI/validación/benchmarks.
//!
//! Dos caminos de ejecución (elegidos según flags):
//!   - Fast path (`run_soup*`): sin snapshots espaciales, cómputo puro.
//!   - Streaming (`SoupSim::new → loop { step }`): requerido cuando
//!     `--ppm` o `--ppm-every` están presentes (necesita acceso al grid
//!     por tick).  ADR-040 §5 garantiza byte-equivalence con fast path.

use std::fs;
use std::path::{Path, PathBuf};

use resonance::layers::reaction_network::ReactionNetwork;
use resonance::layers::species_grid::SpeciesGrid;
use resonance::use_cases::experiments::autopoiesis::{
    SoupSim, run_soup, run_soup_with_network,
};

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

    let report = if cli.out_ppm.is_some() {
        execute_streaming(cli)?
    } else {
        execute_fast(cli)?
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
        eprintln!("  -> wrote {:?}{}{}", cli.out_json,
            cli.out_dot.as_ref().map(|p| format!(" + {p:?}")).unwrap_or_default(),
            cli.out_ppm.as_ref().map(|p| format!(" + {p:?}")).unwrap_or_default(),
        );
    }
    Ok(())
}

fn execute_fast(cli: &Cli) -> Result<resonance::use_cases::experiments::autopoiesis::SoupReport, String> {
    Ok(match &cli.network {
        Some(path) => {
            let net = load_network(path)?;
            run_soup_with_network(&cli.config, net)
        }
        None => run_soup(&cli.config),
    })
}

fn execute_streaming(cli: &Cli) -> Result<resonance::use_cases::experiments::autopoiesis::SoupReport, String> {
    let net = match &cli.network {
        Some(path) => load_network(path)?,
        None => {
            use resonance::blueprint::constants::chemistry::{
                MAX_PRODUCTS_PER_REACTION, MAX_REACTANTS_PER_REACTION,
            };
            use resonance::use_cases::experiments::autopoiesis::random_reaction_network;
            random_reaction_network(
                cli.config.seed, cli.config.n_species, cli.config.n_reactions,
                MAX_REACTANTS_PER_REACTION as u8, MAX_PRODUCTS_PER_REACTION as u8,
            )
        }
    };
    let ppm_path = cli.out_ppm.as_ref().expect("streaming path requires --ppm");
    let mut sim = SoupSim::new(cli.config.clone(), net);
    while !sim.is_done() {
        sim.step();
        if let Some(every) = cli.ppm_every {
            if every > 0 && sim.tick() % every == 0 {
                let frame = frame_path(ppm_path, sim.tick());
                write_ppm(&frame, sim.grid(), cli.ppm_scale, cli.config.initial_food_qe)?;
            }
        }
    }
    // Snapshot final (siempre, aunque haya frames intermedios).
    write_ppm(ppm_path, sim.grid(), cli.ppm_scale, cli.config.initial_food_qe)?;
    Ok(sim.finish())
}

fn load_network(path: &Path) -> Result<ReactionNetwork, String> {
    let text = fs::read_to_string(path)
        .map_err(|e| format!("read {path:?}: {e}"))?;
    ReactionNetwork::from_ron_str(&text)
        .map_err(|e| format!("parse {path:?}: {e:?}"))
}

/// Inserta `_tNNNNNN` antes de la extensión: `sim.ppm` + tick=1500 ⇒ `sim_t001500.ppm`.
fn frame_path(base: &Path, tick: u64) -> PathBuf {
    let stem = base.file_stem().and_then(|s| s.to_str()).unwrap_or("frame");
    let ext = base.extension().and_then(|s| s.to_str()).unwrap_or("ppm");
    let parent = base.parent().unwrap_or_else(|| Path::new("."));
    parent.join(format!("{stem}_t{tick:06}.{ext}"))
}

/// Renderiza el grid a PPM P6.  Mapeo: species 1→R, 2→G, 3→B (top-3 productos);
/// species 0 (food) se superpone como brillo blanco aditivo para visibilizar
/// el spot de HCHO.  Escala cada canal sobre `max_conc_ref` (clamp 0..=255).
/// Upscale nearest-neighbor por `scale`.
fn write_ppm(
    path: &Path,
    grid: &SpeciesGrid,
    scale: usize,
    max_conc_ref: f32,
) -> Result<(), String> {
    let (w, h) = (grid.width(), grid.height());
    let out_w = w * scale;
    let out_h = h * scale;
    let denom = max_conc_ref.max(1e-6);

    // Pre-compute 3-byte pixel por celda lógica.
    let mut cells: Vec<(u8, u8, u8)> = Vec::with_capacity(w * h);
    for y in 0..h {
        for x in 0..w {
            let cell = grid.cell(x, y);
            let r = channel(cell.species[1], denom);
            let g = channel(cell.species[2], denom);
            let b = channel(cell.species[3], denom);
            // Food (species 0) como brillo blanco aditivo, capped.
            let food = (cell.species[0] / denom).clamp(0.0, 1.0);
            let boost = (food * 96.0) as u16;
            cells.push((
                (r as u16 + boost).min(255) as u8,
                (g as u16 + boost).min(255) as u8,
                (b as u16 + boost).min(255) as u8,
            ));
        }
    }

    // Upscale nearest-neighbor.
    let mut pixels = Vec::with_capacity(out_w * out_h * 3);
    for oy in 0..out_h {
        for ox in 0..out_w {
            let (r, g, b) = cells[(oy / scale) * w + (ox / scale)];
            pixels.push(r); pixels.push(g); pixels.push(b);
        }
    }
    let header = format!("P6\n{out_w} {out_h}\n255\n");
    let mut buf = header.into_bytes();
    buf.extend_from_slice(&pixels);
    fs::write(path, &buf).map_err(|e| format!("write {path:?}: {e}"))?;
    Ok(())
}

#[inline]
fn channel(conc: f32, denom: f32) -> u8 {
    if !conc.is_finite() || conc <= 0.0 { return 0; }
    ((conc / denom).clamp(0.0, 1.0) * 255.0) as u8
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn frame_path_interpolates_tick_suffix() {
        let p = Path::new("/tmp/sim.ppm");
        assert_eq!(frame_path(p, 1500), PathBuf::from("/tmp/sim_t001500.ppm"));
        assert_eq!(frame_path(p, 0), PathBuf::from("/tmp/sim_t000000.ppm"));
    }

    #[test]
    fn channel_clamps_and_scales() {
        assert_eq!(channel(0.0, 10.0), 0);
        assert_eq!(channel(10.0, 10.0), 255);
        assert_eq!(channel(5.0, 10.0), 127);
        assert_eq!(channel(-1.0, 10.0), 0);
        assert_eq!(channel(f32::NAN, 10.0), 0);
    }

    #[test]
    fn write_ppm_produces_valid_p6_header() {
        use resonance::layers::reaction::SpeciesId;
        let mut grid = SpeciesGrid::new(4, 3, 50.0);
        grid.seed(1, 1, SpeciesId::new(1).unwrap(), 5.0);
        let tmp = std::env::temp_dir().join("ap6d_write_ppm_test.ppm");
        let _ = fs::remove_file(&tmp);
        write_ppm(&tmp, &grid, 2, 10.0).unwrap();
        let data = fs::read(&tmp).unwrap();
        assert!(data.starts_with(b"P6\n8 6\n255\n"));
        // Pixel count = 8×6 × 3 bytes = 144 after header.
        let header_end = data.iter().position(|&b| b == b'\n').unwrap();
        let header_end = data[header_end + 1..].iter().position(|&b| b == b'\n').unwrap() + header_end + 1;
        let header_end = data[header_end + 1..].iter().position(|&b| b == b'\n').unwrap() + header_end + 1;
        assert_eq!(data.len() - (header_end + 1), 8 * 6 * 3);
    }
}
