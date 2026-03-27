//! AC-5: Cooperation Emergence — Nash-stable alliance detection.
//!
//! For each entity pair within `COOPERATION_SCAN_RADIUS`, evaluates whether
//! forming/maintaining a cooperation satisfies the Nash equilibrium condition:
//! both parties gain more together than alone, accounting for AC-1 interference cost.
//!
//! Phase: `Phase::MetabolicLayer` — after trophic (extraction rates are current),
//! before faction_identity (alliances can affect faction alignment).

use bevy::prelude::*;

use crate::blueprint::constants::*;
use crate::blueprint::equations::emergence::symbiosis::{
    cooperation_is_beneficial, defection_temptation, extraction_estimate_in_group,
    extraction_estimate_solo,
};
use crate::blueprint::equations::{apply_metabolic_interference, metabolic_interference_factor};
use crate::events::{AllianceDefectEvent, AllianceProposedEvent};
use crate::layers::{BaseEnergy, OscillatorySignature, TrophicConsumer};
use crate::runtime_platform::compat_2d3d::SimWorldTransformParams;
use crate::runtime_platform::core_math_agnostic::sim_plane_pos;
use crate::runtime_platform::simulation_tick::SimulationElapsed;
use crate::world::SpatialIndex;

/// Evaluates Nash-stable cooperation for all entity pairs within scan radius.
///
/// Emits `AllianceProposedEvent` when cooperation is beneficial,
/// `AllianceDefectEvent` when an existing solo stance is better.
///
/// Budget guard: only processes the first `TROPHIC_SCAN_BUDGET` entities to
/// bound O(n²) worst case — same pattern as trophic systems.
pub fn cooperation_evaluation_system(
    spatial_index: Res<SpatialIndex>,
    layout: Res<SimWorldTransformParams>,
    elapsed: Option<Res<SimulationElapsed>>,
    query: Query<(Entity, &BaseEnergy, &TrophicConsumer, &Transform, Option<&OscillatorySignature>)>,
    mut alliance_events: EventWriter<AllianceProposedEvent>,
    mut defect_events: EventWriter<AllianceDefectEvent>,
) {
    let t = elapsed.map(|e| e.secs).unwrap_or(0.0);

    // Snapshot: (entity, qe, intake_rate, pos, freq, phase)
    let snapshot: Vec<(Entity, f32, f32, bevy::math::Vec2, f32, f32)> = query
        .iter()
        .map(|(e, energy, consumer, transform, osc)| {
            let pos = sim_plane_pos(transform.translation, layout.use_xz_ground);
            let (freq, phase) = osc.map(|o| (o.frequency_hz(), o.phase())).unwrap_or((0.0, 0.0));
            (e, energy.qe(), consumer.intake_rate, pos, freq, phase)
        })
        .collect();

    // Sort by entity index for O(log n) neighbour lookup.
    let mut sorted_snapshot = snapshot.clone();
    sorted_snapshot.sort_unstable_by_key(|(e, _, _, _, _, _)| e.index());

    let budget = TROPHIC_SCAN_BUDGET.min(snapshot.len());

    for &(entity_a, qe_a, rate_a, pos_a, freq_a, phase_a) in snapshot.iter().take(budget) {
        if qe_a < QE_MIN_EXISTENCE || rate_a < COOPERATION_MIN_VIABLE_RATE {
            continue;
        }

        let nearby = spatial_index.query_radius(pos_a, COOPERATION_SCAN_RADIUS);

        for entry in nearby.iter().filter(|e| e.entity != entity_a) {
            let Ok(idx_b) = sorted_snapshot.binary_search_by_key(&entry.entity.index(), |(e, _, _, _, _, _)| e.index()) else { continue };
            let (_, qe_b, rate_b, _, freq_b, phase_b) = sorted_snapshot[idx_b];

            if qe_b < QE_MIN_EXISTENCE || rate_b < COOPERATION_MIN_VIABLE_RATE {
                continue;
            }

            // AC-1: interference cost from oscillatory mismatch.
            let interference = metabolic_interference_factor(freq_a, phase_a, freq_b, phase_b, t);
            let interference_cost = apply_metabolic_interference(rate_a * COOPERATION_INTERFERENCE_RATE_SCALING, 1.0 - interference);

            let group_size = 2.0;
            let a_in_group = extraction_estimate_in_group(rate_a, group_size, COOPERATION_GROUP_BONUS);
            let b_in_group = extraction_estimate_in_group(rate_b, group_size, COOPERATION_GROUP_BONUS);
            let a_solo = extraction_estimate_solo(rate_a);
            let b_solo = extraction_estimate_solo(rate_b);

            if cooperation_is_beneficial(a_solo, a_in_group, b_solo, b_in_group, interference_cost) {
                alliance_events.send(AllianceProposedEvent {
                    initiator: entity_a,
                    partner: entry.entity,
                    expected_gain: a_in_group - a_solo,
                });
            } else {
                let temptation = defection_temptation(a_solo, a_in_group);
                if temptation > COOPERATION_DEFECT_THRESHOLD {
                    defect_events.send(AllianceDefectEvent {
                        defector: entity_a,
                        abandoned: entry.entity,
                        defection_temptation: temptation,
                    });
                }
            }
        }
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layers::{TrophicClass, TrophicConsumer};
    use crate::world::space::SpatialEntry;

    fn test_app() -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_event::<AllianceProposedEvent>();
        app.add_event::<AllianceDefectEvent>();
        app.insert_resource(SimWorldTransformParams::default());
        app
    }

    fn build_index(entries: &[(Entity, bevy::math::Vec2)]) -> SpatialIndex {
        let mut idx = SpatialIndex::new(COOPERATION_SCAN_RADIUS);
        for &(e, pos) in entries {
            idx.insert(SpatialEntry { entity: e, position: pos, radius: 1.0 });
        }
        idx
    }

    #[test]
    fn same_band_entities_propose_alliance() {
        let mut app = test_app();

        let e1 = app.world_mut().spawn((
            BaseEnergy::new(100.0),
            TrophicConsumer::new(TrophicClass::Carnivore, 10.0),
            OscillatorySignature::new(75.0, 0.0), // same band
            Transform::from_xyz(0.0, 0.0, 0.0),
        )).id();
        let e2 = app.world_mut().spawn((
            BaseEnergy::new(100.0),
            TrophicConsumer::new(TrophicClass::Carnivore, 10.0),
            OscillatorySignature::new(75.0, 0.0), // same band
            Transform::from_xyz(2.0, 0.0, 0.0),
        )).id();

        let idx = build_index(&[(e1, bevy::math::Vec2::ZERO), (e2, bevy::math::Vec2::new(2.0, 0.0))]);
        app.insert_resource(idx);
        app.add_systems(Update, cooperation_evaluation_system);
        app.update();

        let events: Vec<AllianceProposedEvent> = app.world_mut()
            .resource_mut::<Events<AllianceProposedEvent>>()
            .drain()
            .collect();
        assert!(!events.is_empty(), "same-band entities should propose alliance");
    }

    #[test]
    fn solo_entity_emits_no_events() {
        let mut app = test_app();

        let e1 = app.world_mut().spawn((
            BaseEnergy::new(100.0),
            TrophicConsumer::new(TrophicClass::Carnivore, 10.0),
            Transform::from_xyz(0.0, 0.0, 0.0),
        )).id();

        let idx = build_index(&[(e1, bevy::math::Vec2::ZERO)]);
        app.insert_resource(idx);
        app.add_systems(Update, cooperation_evaluation_system);
        app.update();

        let alliances: Vec<_> = app.world_mut()
            .resource_mut::<Events<AllianceProposedEvent>>()
            .drain()
            .collect();
        let defects: Vec<_> = app.world_mut()
            .resource_mut::<Events<AllianceDefectEvent>>()
            .drain()
            .collect();
        assert!(alliances.is_empty() && defects.is_empty(), "solo entity: no events");
    }

    #[test]
    fn cooperation_gain_is_positive_in_event() {
        let mut app = test_app();

        let e1 = app.world_mut().spawn((
            BaseEnergy::new(100.0),
            TrophicConsumer::new(TrophicClass::Carnivore, 10.0),
            OscillatorySignature::new(75.0, 0.0),
            Transform::from_xyz(0.0, 0.0, 0.0),
        )).id();
        let e2 = app.world_mut().spawn((
            BaseEnergy::new(100.0),
            TrophicConsumer::new(TrophicClass::Carnivore, 10.0),
            OscillatorySignature::new(75.0, 0.0),
            Transform::from_xyz(1.0, 0.0, 0.0),
        )).id();

        let idx = build_index(&[(e1, bevy::math::Vec2::ZERO), (e2, bevy::math::Vec2::new(1.0, 0.0))]);
        app.insert_resource(idx);
        app.add_systems(Update, cooperation_evaluation_system);
        app.update();

        let events: Vec<AllianceProposedEvent> = app.world_mut()
            .resource_mut::<Events<AllianceProposedEvent>>()
            .drain()
            .collect();
        if let Some(event) = events.iter().find(|e| e.initiator == e1) {
            assert!(event.expected_gain > 0.0, "expected_gain should be positive: {}", event.expected_gain);
        }
    }
}
