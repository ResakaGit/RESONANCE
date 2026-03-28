//! A2: Laboratory of Universes — evolve under any preset.

use crate::use_cases::{evolve_with, ExperimentReport};
use crate::use_cases::presets::UniversePreset;

/// Evolve life in a universe defined by a preset.
///
/// The simplest use case: one preset, one seed, one report.
pub fn run(preset: &UniversePreset, seed: u64, worlds: usize, gens: u32, ticks: u32) -> ExperimentReport {
    evolve_with(preset, seed, worlds, gens, ticks, 16)
}
