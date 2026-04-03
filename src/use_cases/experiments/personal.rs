//! D1: Personal Universe — your birthday = your ecosystem.

use crate::blueprint::equations::determinism;
use crate::use_cases::presets::EARTH;
use crate::use_cases::{ExperimentReport, evolve_with};

/// Evolve a universe from a personal string (name, birthday, etc.).
///
/// Deterministic: same string → same universe → same creatures.
pub fn run(personal_string: &str) -> ExperimentReport {
    let seed = hash_string(personal_string);
    evolve_with(&EARTH, seed, 100, 50, 2000, 12)
}

/// Hash a string to a deterministic u64 seed.
fn hash_string(s: &str) -> u64 {
    let bytes: Vec<f32> = s.bytes().map(|b| b as f32).collect();
    determinism::hash_f32_slice(&bytes)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn same_string_same_seed() {
        assert_eq!(hash_string("hello"), hash_string("hello"));
    }

    #[test]
    fn different_strings_different_seeds() {
        assert_ne!(hash_string("hello"), hash_string("world"));
    }
}
