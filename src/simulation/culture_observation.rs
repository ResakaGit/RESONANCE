//! Culture observation — CE track.
//!
//! Cultura(G) = coherencia × síntesis × resiliencia × longevidad.
//! Stateless: pure equations called per observation interval.
//! Phase: [`Phase::MetabolicLayer`], after `metrics_snapshot_system`,
//! before `faction_identity_system`.
//!
//! Run condition: every [`CULTURE_OBSERVATION_INTERVAL_TICKS`] ticks (30 ≈ 0.5 s @ 60 TPS).
//! O(n²) per faction for synthesis — throttle mandatory.

use bevy::prelude::*;

use crate::blueprint::constants::{
    CULTURE_GROUP_MIN_SIZE, CULTURE_OBSERVATION_INTERVAL_TICKS, CULTURE_PERCOLATION_CONNECTIVITY,
};
use crate::blueprint::equations::{
    conflict_active, culture_emergent, culture_index, group_frequency_coherence,
    group_longevity_norm, inter_group_conflict_potential, internal_synthesis_rate,
    pattern_resilience,
};
use crate::events::{CultureConflictEvent, CultureEmergenceEvent};
use crate::layers::{Faction, MobaIdentity, OscillatorySignature};
use crate::runtime_platform::simulation_tick::SimulationClock;

// ─── Constants ────────────────────────────────────────────────────────────────

const FACTION_COUNT: usize = 4;
const FACTIONS: [Faction; FACTION_COUNT] =
    [Faction::Neutral, Faction::Red, Faction::Blue, Faction::Wild];

#[inline]
fn faction_idx(f: Faction) -> usize {
    match f {
        Faction::Neutral => 0,
        Faction::Red     => 1,
        Faction::Blue    => 2,
        Faction::Wild    => 3,
    }
}

// ─── Run condition ─────────────────────────────────────────────────────────────

/// Throttle: run every [`CULTURE_OBSERVATION_INTERVAL_TICKS`] ticks.
/// Prevents O(n²) synthesis from running every tick.
pub fn every_culture_observation_interval(clock: Res<SimulationClock>) -> bool {
    clock.tick_id % CULTURE_OBSERVATION_INTERVAL_TICKS == 0
}

// ─── Local state ──────────────────────────────────────────────────────────────

/// Cross-tick observation state per faction (stored in `Local<T>` — NOT a component).
/// Max 4 fields (DOD).
pub struct CultureObservationState {
    /// Coherence at the previous observation tick — baseline for resilience.
    /// 0.0 means "never observed" (sentinel).
    prev_coherence:  [f32; FACTION_COUNT],
    /// Tick at which this faction first had ≥ N_min members.
    /// 0 means "never observed".
    first_seen_tick: [u64; FACTION_COUNT],
    /// Whether culture was emergent at the previous observation.
    was_emergent:    [bool; FACTION_COUNT],
}

impl Default for CultureObservationState {
    fn default() -> Self {
        Self {
            prev_coherence:  [0.0; FACTION_COUNT],
            first_seen_tick: [0; FACTION_COUNT],
            was_emergent:    [false; FACTION_COUNT],
        }
    }
}

/// Per-faction frequency buffers — reused across ticks to avoid allocation.
pub struct FactionFreqBuffers {
    bins: [Vec<f32>; FACTION_COUNT],
}

impl Default for FactionFreqBuffers {
    fn default() -> Self {
        Self { bins: [Vec::new(), Vec::new(), Vec::new(), Vec::new()] }
    }
}

impl FactionFreqBuffers {
    fn clear(&mut self) {
        for bin in &mut self.bins { bin.clear(); }
    }

    fn push(&mut self, faction: Faction, freq: f32) {
        self.bins[faction_idx(faction)].push(freq);
    }

    fn freqs(&self, faction: Faction) -> &[f32] {
        &self.bins[faction_idx(faction)]
    }
}

// ─── System ───────────────────────────────────────────────────────────────────

/// Computes cultural emergence and inter-group conflict for each faction.
///
/// Reads `OscillatorySignature` + `MobaIdentity`, groups by faction, computes
/// per-group metrics via pure equations, emits `CultureEmergenceEvent` on
/// phase transitions and `CultureConflictEvent` when destructive interference
/// is active between two factions.
pub fn culture_observation_system(
    query:             Query<(&OscillatorySignature, &MobaIdentity)>,
    clock:             Res<SimulationClock>,
    mut obs_state:     Local<CultureObservationState>,
    mut buffers:       Local<FactionFreqBuffers>,
    mut emergence_w:   EventWriter<CultureEmergenceEvent>,
    mut conflict_w:    EventWriter<CultureConflictEvent>,
) {
    let tick = clock.tick_id;

    // 1. Collect frequencies by faction into pre-allocated buffers.
    buffers.clear();
    for (osc, identity) in &query {
        buffers.push(identity.faction(), osc.frequency_hz);
    }

    // 2. Per-faction metrics and emergence events.
    for &faction in &FACTIONS {
        let idx    = faction_idx(faction);
        let freqs  = buffers.freqs(faction);

        if freqs.len() < CULTURE_GROUP_MIN_SIZE { continue; }

        let coherence = group_frequency_coherence(freqs);
        let synthesis = internal_synthesis_rate(freqs);

        // Resilience: ratio of current coherence vs previous observation.
        // First observation (prev == 0.0) → 1.0 (no degradation baseline yet).
        let resilience = if obs_state.prev_coherence[idx] == 0.0 {
            1.0
        } else {
            pattern_resilience(obs_state.prev_coherence[idx], coherence)
        };

        // Longevity: ticks since first N_min-qualified observation.
        if obs_state.first_seen_tick[idx] == 0 {
            obs_state.first_seen_tick[idx] = tick;
        }
        let age_ticks = tick.saturating_sub(obs_state.first_seen_tick[idx]) as f32;
        let longevity = group_longevity_norm(age_ticks);

        let ci = culture_index(coherence, synthesis, resilience, longevity);
        // Spatial connectivity: CULTURE_PERCOLATION_CONNECTIVITY as threshold proxy
        // (full grid-graph connectivity is a future MG-track enhancement).
        let emergent = culture_emergent(
            coherence,
            synthesis,
            resilience,
            freqs.len(),
            CULTURE_PERCOLATION_CONNECTIVITY,
        );

        // Emit only on rising edge (false → true transition).
        if emergent && !obs_state.was_emergent[idx] {
            emergence_w.send(CultureEmergenceEvent { faction, culture_index: ci, coherence });
        }

        obs_state.prev_coherence[idx] = coherence;
        obs_state.was_emergent[idx]   = emergent;
    }

    // 3. Cross-faction conflict (C(4,2) = 6 pairs — constant cost).
    for i in 0..FACTION_COUNT {
        for j in (i + 1)..FACTION_COUNT {
            let fa = FACTIONS[i];
            let fb = FACTIONS[j];
            let fa_freqs = buffers.freqs(fa);
            let fb_freqs = buffers.freqs(fb);

            if fa_freqs.len() < CULTURE_GROUP_MIN_SIZE
                || fb_freqs.len() < CULTURE_GROUP_MIN_SIZE
            {
                continue;
            }

            let potential = inter_group_conflict_potential(fa_freqs, fb_freqs);
            if conflict_active(potential) {
                conflict_w.send(CultureConflictEvent {
                    faction_a: fa,
                    faction_b: fb,
                    conflict_potential: potential,
                });
            }
        }
    }
}

// ─── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    // ── Run condition ─────────────────────────────────────────────────────────

    #[test]
    fn every_culture_observation_interval_fires_at_multiples() {
        // Test the logic directly without Bevy App.
        for tick in 0..300u64 {
            let should = tick % CULTURE_OBSERVATION_INTERVAL_TICKS == 0;
            // Spot-check specific ticks.
            if tick == 0   { assert!(should,  "tick 0 should fire"); }
            if tick == 30  { assert!(should,  "tick 30 should fire"); }
            if tick == 60  { assert!(should,  "tick 60 should fire"); }
            if tick == 15  { assert!(!should, "tick 15 should not fire"); }
            if tick == 31  { assert!(!should, "tick 31 should not fire"); }
        }
    }

    // ── faction_idx ───────────────────────────────────────────────────────────

    #[test]
    fn faction_idx_all_unique_and_in_bounds() {
        let indices: Vec<usize> = FACTIONS.iter().map(|&f| faction_idx(f)).collect();
        for &i in &indices {
            assert!(i < FACTION_COUNT);
        }
        // All unique.
        let mut sorted = indices.clone();
        sorted.sort_unstable();
        sorted.dedup();
        assert_eq!(sorted.len(), FACTION_COUNT);
    }

    // ── FactionFreqBuffers ────────────────────────────────────────────────────

    #[test]
    fn faction_freq_buffers_clear_resets_all_bins() {
        let mut buf = FactionFreqBuffers::default();
        buf.push(Faction::Red, 100.0);
        buf.push(Faction::Blue, 200.0);
        buf.clear();
        for &f in &FACTIONS {
            assert!(buf.freqs(f).is_empty(), "bin for {f:?} should be empty after clear");
        }
    }

    #[test]
    fn faction_freq_buffers_push_routes_to_correct_bin() {
        let mut buf = FactionFreqBuffers::default();
        buf.push(Faction::Red, 440.0);
        buf.push(Faction::Red, 450.0);
        buf.push(Faction::Blue, 250.0);
        assert_eq!(buf.freqs(Faction::Red).len(),     2);
        assert_eq!(buf.freqs(Faction::Blue).len(),    1);
        assert_eq!(buf.freqs(Faction::Neutral).len(), 0);
        assert_eq!(buf.freqs(Faction::Wild).len(),    0);
    }

    // ── CultureObservationState ───────────────────────────────────────────────

    #[test]
    fn culture_observation_state_default_zero_sentinel() {
        let s = CultureObservationState::default();
        for i in 0..FACTION_COUNT {
            assert_eq!(s.prev_coherence[i],  0.0);
            assert_eq!(s.first_seen_tick[i], 0);
            assert!(!s.was_emergent[i]);
        }
    }

    // ── Resilience on first observation ───────────────────────────────────────

    #[test]
    fn resilience_first_observation_returns_one() {
        // Guard: prev_coherence == 0.0 → resilience = 1.0
        let prev = 0.0f32;
        let resilience = if prev == 0.0 { 1.0 } else { pattern_resilience(prev, 0.7) };
        assert!((resilience - 1.0).abs() < 1e-6);
    }

    // ── Longevity age tracking ─────────────────────────────────────────────────

    #[test]
    fn age_ticks_saturating_sub_never_underflows() {
        let first_seen: u64 = 50;
        let tick: u64 = 30;
        let age = tick.saturating_sub(first_seen) as f32;
        assert_eq!(age, 0.0);  // saturating: no wrap-around
    }
}
