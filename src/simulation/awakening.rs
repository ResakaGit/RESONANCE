//! Entity awakening: inert materialized tiles gain behavioral capabilities
//! when energy conditions cross the coherence threshold.
//!
//! Axiom 1: everything is energy — the distinction between "terrain" and "organism"
//! is not structural, it's energetic. An entity with sufficient coherent energy IS alive.
//!
//! Phase: [`Phase::MorphologicalLayer`], after abiogenesis.

use bevy::prelude::*;

use crate::blueprint::constants::*;
use crate::blueprint::equations;
use crate::simulation::abiogenesis::constants::{
    FAUNA_EMERGENT_ADAPT_RATE, FAUNA_EMERGENT_QE_COST_HZ, FAUNA_EMERGENT_STAB_BAND,
};
use crate::blueprint::equations::awakening::{AWAKENING_BUDGET_PER_TICK, AWAKENING_SCAN_INTERVAL};
use crate::blueprint::equations::derived_thresholds as dt;
use crate::layers::{
    BaseEnergy, BehaviorCooldown, BehaviorIntent, BehavioralAgent, CapabilitySet, Homeostasis,
    InferenceProfile, MatterCoherence, OscillatorySignature, SpatialVolume, TrophicClass,
    TrophicConsumer, TrophicState,
};
use crate::layers::has_inferred_shape::HasInferredShape;
use crate::layers::organ::LifecycleStageCache;
use crate::layers::senescence::SenescenceProfile;
use crate::layers::shape_params::MorphogenesisShapeParams;
use crate::runtime_platform::simulation_tick::SimulationClock;
use crate::worldgen::EnergyFieldGrid;

/// Promotes inert entities to living organisms when coherence conditions are met.
/// Stateless: reads energy state, computes potential, inserts components if threshold passed.
pub fn awakening_system(
    mut commands: Commands,
    clock: Res<SimulationClock>,
    grid: Option<Res<EnergyFieldGrid>>,
    candidates: Query<
        (Entity, &BaseEnergy, &OscillatorySignature, &SpatialVolume, &MatterCoherence, &Transform),
        Without<BehavioralAgent>,
    >,
) {
    // Scan only every N ticks to reduce cost.
    if clock.tick_id % AWAKENING_SCAN_INTERVAL as u64 != 0 {
        return;
    }
    let Some(grid) = grid else { return };

    let cell_size = grid.cell_size;
    let mut awakened = 0_usize;

    for (entity, energy, osc, volume, matter, transform) in &candidates {
        if awakened >= AWAKENING_BUDGET_PER_TICK {
            break;
        }
        if energy.qe() < dt::self_sustaining_qe_min() {
            continue;
        }

        // Compute coherence from field neighbors (same as abiogenesis).
        let pos = crate::math_types::Vec2::new(transform.translation.x, transform.translation.z);
        let Some((cx, cy)) = grid.cell_coords(pos) else { continue };
        let neighbors = gather_neighbor_coherence(&grid, cx, cy, cell_size);
        let coherence = equations::cell_coherence_gain(osc.frequency_hz(), &neighbors);
        let dissipation = equations::dissipation_from_state(matter.state());
        let potential = equations::awakening::awakening_potential(energy.qe(), coherence, dissipation);

        if potential < dt::spawn_potential_threshold() {
            continue;
        }

        // Derive capabilities from current energy state (Axiom 1).
        let vol = volume.radius.max(dt::DISSIPATION_SOLID);
        let density = energy.qe() / (vol * vol);
        let coherence_norm = (coherence / (coherence + energy.qe())).clamp(0.0, 1.0);
        let caps = equations::capabilities_from_energy(energy.qe(), density, coherence_norm);

        if caps & CapabilitySet::MOVE == 0 && caps & CapabilitySet::SENSE == 0 {
            continue; // Not enough energy for behavioral capabilities.
        }

        // Derive inference profile from energy (Axiom 1).
        let (growth, mobility, branching, resilience) =
            equations::inference_profile_from_energy(density, coherence_norm, 0.0);

        // Determine trophic class from capabilities.
        let is_mobile = caps & CapabilitySet::MOVE != 0;
        let trophic_class = if is_mobile { TrophicClass::Herbivore } else { TrophicClass::PrimaryProducer };
        let intake_rate = if is_mobile {
            ABIOGENESIS_HERBIVORE_INTAKE_RATE
        } else {
            0.0
        };

        // Insert behavioral + lifecycle stacks via component group factories.
        use crate::entities::component_groups as cg;
        let profile = InferenceProfile::new(growth, mobility, branching, resilience);
        commands.entity(entity).insert((
            cg::behavior_components(caps, profile),
            cg::lifecycle_components(clock.tick_id, is_mobile),
            crate::layers::InternalEnergyField::uniform(energy.qe()),
        ));

        // Mobile entities get full motor + trophic stack.
        if is_mobile {
            commands.entity(entity).insert((
                cg::trophic_components(trophic_class, intake_rate),
                Homeostasis::new(
                    FAUNA_EMERGENT_ADAPT_RATE,
                    FAUNA_EMERGENT_QE_COST_HZ,
                    FAUNA_EMERGENT_STAB_BAND,
                    true,
                ),
            ));
        }

        awakened += 1;
    }
}

/// Gather neighbor qe/frequency/distance for coherence calculation.
fn gather_neighbor_coherence(
    grid: &EnergyFieldGrid,
    cx: u32,
    cy: u32,
    cell_size: f32,
) -> Vec<(f32, f32, f32)> {
    let mut neighbors = Vec::with_capacity(8);
    for dy in -1_i32..=1 {
        for dx in -1_i32..=1 {
            if dx == 0 && dy == 0 { continue; }
            let nx = cx as i32 + dx;
            let ny = cy as i32 + dy;
            if nx < 0 || ny < 0 { continue; }
            let Some(ncell) = grid.cell_xy(nx as u32, ny as u32) else { continue };
            let dist = ((dx * dx + dy * dy) as f32).sqrt() * cell_size;
            neighbors.push((ncell.accumulated_qe, ncell.dominant_frequency_hz, dist));
        }
    }
    neighbors
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn gather_neighbor_coherence_center_has_8() {
        let grid = EnergyFieldGrid::new(8, 8, 2.0, crate::math_types::Vec2::ZERO);
        let neighbors = gather_neighbor_coherence(&grid, 4, 4, 2.0);
        assert_eq!(neighbors.len(), 8);
    }

    #[test]
    fn gather_neighbor_coherence_corner_has_3() {
        let grid = EnergyFieldGrid::new(8, 8, 2.0, crate::math_types::Vec2::ZERO);
        let neighbors = gather_neighbor_coherence(&grid, 0, 0, 2.0);
        assert_eq!(neighbors.len(), 3);
    }
}
