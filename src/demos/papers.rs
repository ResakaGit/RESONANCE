//! Demo 2 · Paper validation summary — six published predictions in one HOF.
//!
//! Each entry in `PaperRunner::all()` is a closure returning a `PaperOutcome`
//! (PASS / FAIL + wall time + numeric verdict).  `run` iterates them via
//! `super::run_demo`, treating each paper as one "tick" of the orchestrator.
//!
//! What this proves: the same axiomatic simulator (4 fundamental constants,
//! 8 axioms) reproduces six published cancer-therapy predictions with NO
//! per-paper tuning.  Honest reporting: PV-6 returns the actual sub-test
//! count (currently 4/6) rather than a single boolean.

use std::time::Instant;

use crate::use_cases::experiments::{
    paper_foo_michor2009::{self, FooMichorConfig},
    paper_hill_ccle,
    paper_michor2005::{self, MichorConfig},
    paper_sharma2010::{self, SharmaConfig},
    paper_unified_axioms,
    paper_zhang2022::{self, ZhangConfig},
};

/// Demo input — controls the seed that propagates to all stochastic papers
/// + which papers to include (defaults to all six).
#[derive(Clone, Debug)]
pub struct PapersDemoConfig {
    pub seed: u64,
    pub include: PaperSet,
}

impl Default for PapersDemoConfig {
    fn default() -> Self { Self { seed: 0, include: PaperSet::All } }
}

#[derive(Clone, Copy, Debug)]
pub enum PaperSet { All, Subset(&'static [&'static str]) }

/// Outcome of one paper validation.  `verdict` mirrors the PV-N original
/// boolean OR the "K/N sub-tests" form for PV-6 (unified axioms).
#[derive(Clone, Debug, PartialEq)]
pub struct PaperOutcome {
    pub id: &'static str,    // "PV-1", "PV-2", …
    pub citation: &'static str,
    pub verdict: PaperVerdict,
    pub wall_ms: u128,
}

#[derive(Clone, Debug, PartialEq)]
pub enum PaperVerdict {
    /// Boolean predicate from the paper's `Report` struct.
    Boolean(bool),
    /// `passed` out of `total` sub-tests (used by PV-6 unified axioms).
    Fraction { passed: u32, total: u32 },
}

impl PaperVerdict {
    /// `true` only if Boolean(true) or Fraction with all sub-tests passed.
    pub fn is_pass(&self) -> bool {
        match *self {
            PaperVerdict::Boolean(b) => b,
            PaperVerdict::Fraction { passed, total } => passed == total,
        }
    }
}

/// Final summary — list of outcomes + aggregate count.
#[derive(Clone, Debug, PartialEq)]
pub struct PapersReport {
    pub outcomes: Vec<PaperOutcome>,
    pub total_wall_ms: u128,
}

impl PapersReport {
    pub fn passed_count(&self) -> usize {
        self.outcomes.iter().filter(|o| o.verdict.is_pass()).count()
    }
    pub fn total(&self) -> usize { self.outcomes.len() }
}

// ── Phase 1 · setup ────────────────────────────────────────────────────────

/// Build the runner queue.  Each closure is a thunk that, when called,
/// runs ONE paper validation and returns a `PaperOutcome`.  Stored as
/// boxed closures so the orchestrator iterates them uniformly.
type Runner = Box<dyn FnOnce(u64) -> PaperOutcome>;

fn runners(set: PaperSet) -> Vec<(&'static str, Runner)> {
    let all: Vec<(&'static str, Runner)> = vec![
        ("PV-1", Box::new(run_zhang2022)),
        ("PV-2", Box::new(run_sharma2010)),
        ("PV-3", Box::new(run_hill_ccle)),
        ("PV-4", Box::new(run_foo_michor2009)),
        ("PV-5", Box::new(run_michor2005)),
        ("PV-6", Box::new(run_unified_axioms)),
    ];
    match set {
        PaperSet::All => all,
        PaperSet::Subset(ids) => {
            all.into_iter().filter(|(id, _)| ids.contains(id)).collect()
        }
    }
}

fn timed<F: FnOnce() -> PaperVerdict>(
    id: &'static str, citation: &'static str, f: F,
) -> PaperOutcome {
    let t0 = Instant::now();
    let verdict = f();
    PaperOutcome { id, citation, verdict, wall_ms: t0.elapsed().as_millis() }
}

fn run_zhang2022(seed: u64) -> PaperOutcome {
    timed("PV-1", "Zhang 2022 eLife (adaptive prostate)", || {
        let cfg = ZhangConfig { seed, ..ZhangConfig::default() };
        PaperVerdict::Boolean(paper_zhang2022::run(&cfg).prediction_met)
    })
}

fn run_sharma2010(seed: u64) -> PaperOutcome {
    timed("PV-2", "Sharma 2010 Cell (drug-tolerant persisters)", || {
        let cfg = SharmaConfig { seed, ..SharmaConfig::default() };
        PaperVerdict::Boolean(paper_sharma2010::run(&cfg).recovery_detected)
    })
}

fn run_hill_ccle(_seed: u64) -> PaperOutcome {
    // PV-3 is deterministic — no seed dependence.
    timed("PV-3", "Garnett+Barretina 2012 (Hill n=2 vs GDSC/CCLE)", || {
        PaperVerdict::Boolean(paper_hill_ccle::validate_against_published().resonance_assumption_valid)
    })
}

fn run_foo_michor2009(seed: u64) -> PaperOutcome {
    timed("PV-4", "Foo & Michor 2009 PLoS (pulsed vs continuous)", || {
        let cfg = FooMichorConfig { seed, ..FooMichorConfig::default() };
        PaperVerdict::Boolean(paper_foo_michor2009::run(&cfg).pulsed_beats_continuous)
    })
}

fn run_michor2005(seed: u64) -> PaperOutcome {
    timed("PV-5", "Michor 2005 Nature (biphasic CML imatinib)", || {
        let cfg = MichorConfig { seed, ..MichorConfig::default() };
        PaperVerdict::Boolean(paper_michor2005::run(&cfg).biphasic_detected)
    })
}

fn run_unified_axioms(seed: u64) -> PaperOutcome {
    timed("PV-6", "Internal: all six derivable from 4 fundamentals", || {
        let report = paper_unified_axioms::run(seed);
        PaperVerdict::Fraction { passed: report.passed_count, total: 6 }
    })
}

// ── Phases 2 + 3 · step + summarize via run_demo ───────────────────────────

/// Stateful queue of runners — pops one per step, runs it, emits the outcome.
/// `state` is just `(remaining_runners, seed)`; no globals.
struct PaperState {
    remaining: Vec<(&'static str, Runner)>,
    seed: u64,
}

fn step_paper(state: &mut PaperState) -> Option<PaperOutcome> {
    if state.remaining.is_empty() { return None; }
    let (_id, runner) = state.remaining.remove(0);
    Some(runner(state.seed))
}

fn summarize_papers(_state: &PaperState, frames: Vec<PaperOutcome>) -> PapersReport {
    let total_wall_ms: u128 = frames.iter().map(|o| o.wall_ms).sum();
    PapersReport { outcomes: frames, total_wall_ms }
}

/// Compose `setup → run_demo(step, summarize)`.
pub fn run(cfg: &PapersDemoConfig) -> PapersReport {
    let state = PaperState { remaining: runners(cfg.include), seed: cfg.seed };
    super::run_demo(state, step_paper, summarize_papers)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn verdict_is_pass_boolean() {
        assert!(PaperVerdict::Boolean(true).is_pass());
        assert!(!PaperVerdict::Boolean(false).is_pass());
    }

    #[test]
    fn verdict_is_pass_fraction() {
        assert!(PaperVerdict::Fraction { passed: 6, total: 6 }.is_pass());
        assert!(!PaperVerdict::Fraction { passed: 4, total: 6 }.is_pass());
    }

    #[test]
    fn runners_subset_filters() {
        let r = runners(PaperSet::Subset(&["PV-1", "PV-3"]));
        let ids: Vec<_> = r.iter().map(|(id, _)| *id).collect();
        assert_eq!(ids, vec!["PV-1", "PV-3"]);
    }

    #[test]
    fn runners_all_returns_six() {
        assert_eq!(runners(PaperSet::All).len(), 6);
    }

    #[test]
    fn report_passed_count_aggregates() {
        let r = PapersReport {
            total_wall_ms: 0,
            outcomes: vec![
                PaperOutcome { id: "A", citation: "", verdict: PaperVerdict::Boolean(true), wall_ms: 1 },
                PaperOutcome { id: "B", citation: "", verdict: PaperVerdict::Boolean(false), wall_ms: 2 },
                PaperOutcome { id: "C", citation: "", verdict: PaperVerdict::Fraction { passed: 4, total: 6 }, wall_ms: 3 },
            ],
        };
        assert_eq!(r.passed_count(), 1);
        assert_eq!(r.total(), 3);
    }
}
