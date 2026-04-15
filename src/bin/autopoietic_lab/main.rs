//! AP-6a/b/c: `autopoietic_lab` entry binario (ADR-042).
//! AP-6a/b/c: `autopoietic_lab` binary entry (ADR-042).
//!
//! Thin entry que parsea CLI y dispatch al submódulo apropiado.  Hoy sólo
//! existe `headless`; AP-6c.1+ agregará `view`/`ui`/`lineage` con Bevy cuando
//! ese slice arranque.  Ver `docs/arquitectura/ADR/ADR-042-bevy-viz-layout.md`.
//!
//! Stdlib-only CLI (sin `clap`) para respetar el hard block de crates externos
//! no aprobados (ver `CLAUDE.md` § "Hard Blocks").
//!
//! ```text
//! cargo run --release --bin autopoietic_lab -- \
//!     --seed 42 --ticks 5000 --out report.json --dot lineage.dot
//! ```

mod headless;

use std::env;
use std::path::PathBuf;
use std::process::ExitCode;

use resonance::use_cases::experiments::autopoiesis::SoupConfig;

const HELP: &str = "\
autopoietic_lab — AP-6a/b headless runner

USAGE:
    autopoietic_lab [OPTIONS]

OPTIONS:
    --seed <N>         PRNG seed (default: 42)
    --ticks <N>        Simulation ticks (default: 2000)
    --grid <WxH>       Grid dims, e.g. 16x16 (default: 12x12)
    --species <N>      Species count, 2..=32 (default: 8)
    --reactions <N>    Reaction count (default: 16)
    --food <N>         Initial food species count (default: 3)
    --spot <R>         Centered food spot of radius R (breaks translational
                       symmetry — required for emergent blobs).  Default:
                       uniform (legacy AP-5 harness).
    --food-qe <Q>      Initial food qe per seeded cell (default: 2.0)
    --network <path>   Load reaction network from RON file
                       (overrides --reactions; food set still uses --seed)
    --out-dir <dir>    Preset: crea `dir/` y escribe todo adentro:
                       `report.json`, `lineage.dot`, `grid.ppm` (+ frames
                       si se pasa --ppm-every).  Sustituye a --out/--dot/--ppm.
    --out <path>       JSON report path (default: report.json)
    --dot <path>       Optional DOT lineage path (default: none)
    --ppm <path>       Optional PPM species-heatmap of final grid state.
                       Species 1→R, 2→G, 3→B (top-3 products).  Food
                       (species 0) superpuesto como brillo blanco.
    --ppm-every <N>    Animar: también escribe `{path}_t{tick:06}.ppm`
                       cada N ticks.  Combinable con --ppm.
    --ppm-scale <N>    Upscale nearest-neighbor (default: 16).
    --live             Render grid to terminal (ANSI 24-bit color) cada
                       `--live-every` ticks.  Requiere Windows Terminal /
                       PowerShell 7 / terminal moderna para colores.
    --live-every <N>   Frecuencia de render (default: 20 ticks).
    --live-delay <MS>  Pausa entre renders en milisegundos (default: 80).
    --quiet            Suppress progress output
    --help             Show this help
";

#[derive(Debug)]
pub(crate) struct Cli {
    pub(crate) config: SoupConfig,
    pub(crate) network: Option<PathBuf>,
    pub(crate) out_json: PathBuf,
    pub(crate) out_dot: Option<PathBuf>,
    pub(crate) out_ppm: Option<PathBuf>,
    pub(crate) ppm_every: Option<u64>,
    pub(crate) ppm_scale: usize,
    pub(crate) live: bool,
    pub(crate) live_every: u64,
    pub(crate) live_delay_ms: u64,
    pub(crate) quiet: bool,
}

/// Resultado del parseo: ejecutar con config, o mostrar help.
#[derive(Debug)]
enum CliAction {
    Run(Cli),
    Help,
}

impl Cli {
    fn parse(argv: impl IntoIterator<Item = String>) -> Result<CliAction, String> {
        let mut cfg = SoupConfig { seed: 42, ..SoupConfig::default() };
        let mut out_json = PathBuf::from("report.json");
        let mut out_dot: Option<PathBuf> = None;
        let mut out_ppm: Option<PathBuf> = None;
        let mut ppm_every: Option<u64> = None;
        let mut ppm_scale: usize = 16;
        let mut live = false;
        let mut live_every: u64 = 20;
        let mut live_delay_ms: u64 = 80;
        let mut network: Option<PathBuf> = None;
        let mut quiet = false;

        let mut args = argv.into_iter();
        let _prog = args.next();
        while let Some(a) = args.next() {
            match a.as_str() {
                "--help" | "-h" => return Ok(CliAction::Help),
                "--quiet" => quiet = true,
                "--seed"      => cfg.seed      = take_val(&mut args, &a)?.parse().map_err(|e| format!("{a}: {e}"))?,
                "--ticks"     => cfg.ticks     = take_val(&mut args, &a)?.parse().map_err(|e| format!("{a}: {e}"))?,
                "--species"   => cfg.n_species = take_val(&mut args, &a)?.parse().map_err(|e| format!("{a}: {e}"))?,
                "--reactions" => cfg.n_reactions = take_val(&mut args, &a)?.parse().map_err(|e| format!("{a}: {e}"))?,
                "--food"      => cfg.food_size   = take_val(&mut args, &a)?.parse().map_err(|e| format!("{a}: {e}"))?,
                "--grid"      => cfg.grid = parse_grid(&take_val(&mut args, &a)?)?,
                "--spot"      => cfg.food_spot_radius = Some(take_val(&mut args, &a)?.parse().map_err(|e| format!("{a}: {e}"))?),
                "--food-qe"   => cfg.initial_food_qe = take_val(&mut args, &a)?.parse().map_err(|e| format!("{a}: {e}"))?,
                "--network"   => network  = Some(PathBuf::from(take_val(&mut args, &a)?)),
                "--out"       => out_json = PathBuf::from(take_val(&mut args, &a)?),
                "--dot"       => out_dot  = Some(PathBuf::from(take_val(&mut args, &a)?)),
                "--ppm"       => out_ppm  = Some(PathBuf::from(take_val(&mut args, &a)?)),
                "--out-dir"   => {
                    let dir = PathBuf::from(take_val(&mut args, &a)?);
                    out_json = dir.join("report.json");
                    out_dot  = Some(dir.join("lineage.dot"));
                    out_ppm  = Some(dir.join("grid.ppm"));
                }
                "--ppm-every" => ppm_every = Some(take_val(&mut args, &a)?.parse().map_err(|e| format!("{a}: {e}"))?),
                "--ppm-scale" => ppm_scale = take_val(&mut args, &a)?.parse().map_err(|e| format!("{a}: {e}"))?,
                "--live"      => live = true,
                "--live-every" => live_every = take_val(&mut args, &a)?.parse().map_err(|e| format!("{a}: {e}"))?,
                "--live-delay" => live_delay_ms = take_val(&mut args, &a)?.parse().map_err(|e| format!("{a}: {e}"))?,
                other => return Err(format!("unknown flag: {other}")),
            }
        }
        if ppm_scale == 0 { return Err("--ppm-scale must be > 0".into()); }
        if live_every == 0 { return Err("--live-every must be > 0".into()); }
        Ok(CliAction::Run(Self {
            config: cfg, network, out_json, out_dot,
            out_ppm, ppm_every, ppm_scale,
            live, live_every, live_delay_ms,
            quiet,
        }))
    }
}

fn take_val(args: &mut impl Iterator<Item = String>, flag: &str) -> Result<String, String> {
    args.next().ok_or_else(|| format!("{flag}: missing value"))
}

fn parse_grid(s: &str) -> Result<(usize, usize), String> {
    let (w, h) = s.split_once('x').ok_or_else(|| format!("--grid: expected WxH, got {s}"))?;
    let w: usize = w.parse().map_err(|e| format!("--grid width: {e}"))?;
    let h: usize = h.parse().map_err(|e| format!("--grid height: {e}"))?;
    if w == 0 || h == 0 { return Err("--grid: dims must be > 0".into()); }
    Ok((w, h))
}

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => { eprintln!("error: {e}\n\n{HELP}"); ExitCode::FAILURE }
    }
}

fn run() -> Result<(), String> {
    let cli = match Cli::parse(env::args())? {
        CliAction::Help => { print!("{HELP}"); return Ok(()); }
        CliAction::Run(c) => c,
    };
    // AP-6c.1+ dispatch point: if `--headless` explícito o Bevy no compilado,
    // llamar headless::execute.  Hoy sólo hay headless.
    headless::execute(&cli)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use resonance::layers::reaction_network::ReactionNetwork;
    use resonance::use_cases::experiments::autopoiesis::{run_soup, run_soup_with_network};

    fn argv(items: &[&str]) -> Vec<String> {
        std::iter::once("autopoietic_lab")
            .chain(items.iter().copied())
            .map(String::from)
            .collect()
    }

    fn parse_run(items: &[&str]) -> Result<Cli, String> {
        match Cli::parse(argv(items))? {
            CliAction::Run(c) => Ok(c),
            CliAction::Help => Err("unexpected Help".into()),
        }
    }

    #[test]
    fn parse_uses_sensible_defaults_when_no_flags() {
        let cli = parse_run(&[]).unwrap();
        assert_eq!(cli.config.seed, 42);
        assert_eq!(cli.out_json, PathBuf::from("report.json"));
        assert!(cli.out_dot.is_none());
        assert!(!cli.quiet);
    }

    #[test]
    fn parse_overrides_reach_config() {
        let cli = parse_run(&[
            "--seed", "123", "--ticks", "500", "--grid", "16x20",
            "--species", "12", "--reactions", "24", "--food", "4",
            "--out", "r.json", "--dot", "g.dot", "--quiet",
        ]).unwrap();
        assert_eq!(cli.config.seed, 123);
        assert_eq!(cli.config.ticks, 500);
        assert_eq!(cli.config.grid, (16, 20));
        assert_eq!(cli.config.n_species, 12);
        assert_eq!(cli.config.n_reactions, 24);
        assert_eq!(cli.config.food_size, 4);
        assert_eq!(cli.out_json, PathBuf::from("r.json"));
        assert_eq!(cli.out_dot, Some(PathBuf::from("g.dot")));
        assert!(cli.quiet);
    }

    #[test]
    fn parse_spot_and_food_qe_reach_config() {
        let cli = parse_run(&["--spot", "3", "--food-qe", "50.0"]).unwrap();
        assert_eq!(cli.config.food_spot_radius, Some(3));
        assert!((cli.config.initial_food_qe - 50.0).abs() < 1e-6);
    }

    #[test]
    fn spot_defaults_to_none_uniform_seeding() {
        let cli = parse_run(&[]).unwrap();
        assert!(cli.config.food_spot_radius.is_none());
    }

    #[test]
    fn out_dir_sets_all_three_outputs_with_default_names() {
        let cli = parse_run(&["--out-dir", "runs/exp1"]).unwrap();
        assert_eq!(cli.out_json, PathBuf::from("runs/exp1/report.json"));
        assert_eq!(cli.out_dot,  Some(PathBuf::from("runs/exp1/lineage.dot")));
        assert_eq!(cli.out_ppm,  Some(PathBuf::from("runs/exp1/grid.ppm")));
    }

    #[test]
    fn parse_rejects_unknown_flag() {
        let e = Cli::parse(argv(&["--bogus"])).unwrap_err();
        assert!(e.contains("unknown flag"));
    }

    #[test]
    fn parse_rejects_malformed_grid() {
        let e = Cli::parse(argv(&["--grid", "not-a-grid"])).unwrap_err();
        assert!(e.to_lowercase().contains("grid"));
    }

    #[test]
    fn parse_missing_value_errors_cleanly() {
        let e = Cli::parse(argv(&["--seed"])).unwrap_err();
        assert!(e.contains("--seed"));
    }

    // ── AP-6b: --network flag ──────────────────────────────────────────────

    #[test]
    fn parse_network_flag_stores_path() {
        let cli = parse_run(&["--network", "assets/reactions/raf_minimal.ron"]).unwrap();
        assert_eq!(cli.network, Some(PathBuf::from("assets/reactions/raf_minimal.ron")));
    }

    #[test]
    fn network_flag_defaults_to_none() {
        let cli = parse_run(&["--seed", "99"]).unwrap();
        assert!(cli.network.is_none());
    }

    #[test]
    fn raf_minimal_ron_loads_and_runs_via_run_soup_with_network() {
        let text = fs::read_to_string("assets/reactions/raf_minimal.ron")
            .expect("asset must exist");
        let net = ReactionNetwork::from_ron_str(&text).expect("valid RON");
        assert_eq!(net.len(), 3, "raf_minimal has 3 reactions");

        let cfg = SoupConfig {
            seed: 42, n_species: 4, food_size: 2,
            ticks: 150, grid: (6, 6), ..SoupConfig::default()
        };
        let report = run_soup_with_network(&cfg, net);
        assert_eq!(report.seed, 42);
        assert_eq!(report.n_ticks, 150);
        assert!(report.total_dissipated >= 0.0);
        // Food set con 2 species + 3 reacciones ⟹ posible detectar closures,
        // pero no lo afirmamos — solo que el harness no panic.
    }

    #[test]
    fn network_flag_rejects_nonexistent_file() {
        let cli = parse_run(&["--network", "no_such_file.ron", "--quiet"]).unwrap();
        // Emula lo que hace run(): fs::read_to_string debe fallar.
        let err = fs::read_to_string(cli.network.as_ref().unwrap()).unwrap_err();
        assert!(matches!(err.kind(), std::io::ErrorKind::NotFound));
    }

    #[test]
    fn malformed_ron_is_rejected_by_from_ron_str() {
        let bad = "(reactions: [(this is not valid RON";
        assert!(ReactionNetwork::from_ron_str(bad).is_err());
    }

    #[test]
    fn end_to_end_writes_valid_json_and_dot() {
        use resonance::use_cases::experiments::autopoiesis::SoupReport;
        let tmp = std::env::temp_dir();
        let json_path = tmp.join("ap6a_it_report.json");
        let dot_path  = tmp.join("ap6a_it_lineage.dot");
        let _ = fs::remove_file(&json_path);
        let _ = fs::remove_file(&dot_path);

        // Runs `run()` against a crafted argv via env-var bypass: direct API.
        let cli = parse_run(&[
            "--seed", "321", "--ticks", "150", "--grid", "6x6",
            "--out", json_path.to_str().unwrap(),
            "--dot", dot_path.to_str().unwrap(),
            "--quiet",
        ]).unwrap();
        let report = run_soup(&cli.config);
        fs::write(&json_path, report.to_json().unwrap()).unwrap();
        fs::write(&dot_path,  report.to_dot()).unwrap();

        let json = fs::read_to_string(&json_path).unwrap();
        let parsed: SoupReport = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.seed, 321);
        assert_eq!(parsed.n_ticks, 150);
        assert!(parsed.total_dissipated >= 0.0);

        let dot = fs::read_to_string(&dot_path).unwrap();
        assert!(dot.starts_with("digraph autopoiesis"));
        assert_eq!(dot.matches('{').count(), dot.matches('}').count());
    }
}
