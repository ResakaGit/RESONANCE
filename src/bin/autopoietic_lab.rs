//! AP-6a: `autopoietic_lab` headless CLI.
//!
//! Corre una sopa determinística y exporta `SoupReport` en JSON + DOT opcional.
//! Sin Bevy, sin GPU.  Stdlib-only CLI (sin `clap`) para respetar el hard block
//! de crates externos no aprobados (ver `CLAUDE.md`).
//!
//! Runs a deterministic soup and exports `SoupReport` as JSON + optional DOT.
//! Stdlib-only CLI to honour the "no unapproved external crates" hard block.
//!
//! ```text
//! cargo run --release --bin autopoietic_lab -- \
//!     --seed 42 --ticks 5000 --out report.json --dot lineage.dot
//! ```

use std::env;
use std::fs;
use std::path::PathBuf;
use std::process::ExitCode;

use resonance::use_cases::experiments::autopoiesis::{SoupConfig, run_soup};

const HELP: &str = "\
autopoietic_lab — AP-6a headless runner

USAGE:
    autopoietic_lab [OPTIONS]

OPTIONS:
    --seed <N>         PRNG seed (default: 42)
    --ticks <N>        Simulation ticks (default: 2000)
    --grid <WxH>       Grid dims, e.g. 16x16 (default: 12x12)
    --species <N>      Species count, 2..=32 (default: 8)
    --reactions <N>    Reaction count (default: 16)
    --food <N>         Initial food species count (default: 3)
    --out <path>       JSON report path (default: report.json)
    --dot <path>       Optional DOT lineage path (default: none)
    --quiet            Suppress progress output
    --help             Show this help
";

#[derive(Debug)]
struct Cli {
    config: SoupConfig,
    out_json: PathBuf,
    out_dot: Option<PathBuf>,
    quiet: bool,
}

impl Cli {
    fn parse(argv: impl IntoIterator<Item = String>) -> Result<Self, String> {
        let mut cfg = SoupConfig { seed: 42, ..SoupConfig::default() };
        let mut out_json = PathBuf::from("report.json");
        let mut out_dot: Option<PathBuf> = None;
        let mut quiet = false;

        let mut args = argv.into_iter();
        let _prog = args.next();
        while let Some(a) = args.next() {
            match a.as_str() {
                "--help" | "-h" => return Err("__HELP__".into()),
                "--quiet" => quiet = true,
                "--seed"      => cfg.seed      = take_val(&mut args, &a)?.parse().map_err(|e| format!("{a}: {e}"))?,
                "--ticks"     => cfg.ticks     = take_val(&mut args, &a)?.parse().map_err(|e| format!("{a}: {e}"))?,
                "--species"   => cfg.n_species = take_val(&mut args, &a)?.parse().map_err(|e| format!("{a}: {e}"))?,
                "--reactions" => cfg.n_reactions = take_val(&mut args, &a)?.parse().map_err(|e| format!("{a}: {e}"))?,
                "--food"      => cfg.food_size   = take_val(&mut args, &a)?.parse().map_err(|e| format!("{a}: {e}"))?,
                "--grid"      => cfg.grid = parse_grid(&take_val(&mut args, &a)?)?,
                "--out"       => out_json = PathBuf::from(take_val(&mut args, &a)?),
                "--dot"       => out_dot  = Some(PathBuf::from(take_val(&mut args, &a)?)),
                other => return Err(format!("unknown flag: {other}")),
            }
        }
        Ok(Self { config: cfg, out_json, out_dot, quiet })
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
        Err(e) if e == "__HELP__" => { print!("{HELP}"); ExitCode::SUCCESS }
        Err(e) => { eprintln!("error: {e}\n\n{HELP}"); ExitCode::FAILURE }
    }
}

fn run() -> Result<(), String> {
    let cli = Cli::parse(env::args())?;

    if !cli.quiet {
        eprintln!(
            "autopoietic_lab: seed={} ticks={} grid={}x{} species={} reactions={} food={}",
            cli.config.seed, cli.config.ticks, cli.config.grid.0, cli.config.grid.1,
            cli.config.n_species, cli.config.n_reactions, cli.config.food_size,
        );
    }

    let report = run_soup(&cli.config);
    let json = report.to_json().map_err(|e| format!("json serialize: {e}"))?;
    fs::write(&cli.out_json, json).map_err(|e| format!("write {:?}: {e}", cli.out_json))?;

    if let Some(dot_path) = &cli.out_dot {
        fs::write(dot_path, report.to_dot()).map_err(|e| format!("write {dot_path:?}: {e}"))?;
    }

    if !cli.quiet {
        eprintln!(
            "  -> closures: initial={} final={} dissipated={:.4}",
            report.n_closures_initial, report.n_closures_final, report.total_dissipated,
        );
        eprintln!("  -> wrote {:?}{}", cli.out_json,
            cli.out_dot.as_ref().map(|p| format!(" + {p:?}")).unwrap_or_default(),
        );
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn argv(items: &[&str]) -> Vec<String> {
        std::iter::once("autopoietic_lab")
            .chain(items.iter().copied())
            .map(String::from)
            .collect()
    }

    #[test]
    fn parse_uses_sensible_defaults_when_no_flags() {
        let cli = Cli::parse(argv(&[])).unwrap();
        assert_eq!(cli.config.seed, 42);
        assert_eq!(cli.out_json, PathBuf::from("report.json"));
        assert!(cli.out_dot.is_none());
        assert!(!cli.quiet);
    }

    #[test]
    fn parse_overrides_reach_config() {
        let cli = Cli::parse(argv(&[
            "--seed", "123", "--ticks", "500", "--grid", "16x20",
            "--species", "12", "--reactions", "24", "--food", "4",
            "--out", "r.json", "--dot", "g.dot", "--quiet",
        ])).unwrap();
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

    #[test]
    fn end_to_end_writes_valid_json_and_dot() {
        use resonance::use_cases::experiments::autopoiesis::SoupReport;
        let tmp = std::env::temp_dir();
        let json_path = tmp.join("ap6a_it_report.json");
        let dot_path  = tmp.join("ap6a_it_lineage.dot");
        let _ = fs::remove_file(&json_path);
        let _ = fs::remove_file(&dot_path);

        // Runs `run()` against a crafted argv via env-var bypass: direct API.
        let cli = Cli::parse(argv(&[
            "--seed", "321", "--ticks", "150", "--grid", "6x6",
            "--out", json_path.to_str().unwrap(),
            "--dot", dot_path.to_str().unwrap(),
            "--quiet",
        ])).unwrap();
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
