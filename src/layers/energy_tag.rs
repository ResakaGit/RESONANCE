//! Tag de energía para transferencia cruzada — transportado por entidad móvil.
//! Energy tag for cross-transfer — carried by mobile entity.
//!
//! Deposited on collision with sessile entity, transferred on collision with
//! compatible target. Decays over time (Axiom 4). Compatibility via frequency
//! alignment (Axiom 8).

use bevy::prelude::*;

/// Transient energy tag carried by a mobile entity for cross-reproduction.
/// Tag de energía transitorio transportado por entidad móvil.
///
/// SparseSet: only entities currently carrying a tag have this.
/// Max 4 fields: qe, source_freq, profile_hash, age_ticks.
#[derive(Component, Reflect, Debug, Clone, Copy)]
#[reflect(Component)]
#[component(storage = "SparseSet")]
pub struct EnergyTag {
    /// Energy of the tag (decays over time).
    pub qe: f32,
    /// Frequency of the source entity.
    pub source_freq: f32,
    /// Hash of source InferenceProfile for compatibility check.
    pub profile_hash: u32,
    /// Age in ticks (for decay/lifetime).
    pub age_ticks: u32,
}

impl Default for EnergyTag {
    fn default() -> Self {
        Self {
            qe: 0.0,
            source_freq: 0.0,
            profile_hash: 0,
            age_ticks: 0,
        }
    }
}
