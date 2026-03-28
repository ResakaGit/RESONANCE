//! AC-2: Kuramoto Entrainment System — Axiom 8 consequence.
//!
//! Entities with `OscillatorySignature` within `ENTRAINMENT_SCAN_RADIUS` gradually
//! align their frequencies toward their neighbours (Kuramoto model).
//! Coupling decays with distance via AC-4 purity function.
//!
//! Phase: `Phase::AtomicLayer` — after oscillatory state is stable but before
//! MetabolicLayer reads frequencies for interference calculations.

use bevy::prelude::*;

use crate::blueprint::constants::*;
use crate::blueprint::equations::emergence::entrainment::{
    entrainment_lock_achieved, kuramoto_entrainment_step, ENTRAINMENT_MAX_NEIGHBOURS,
};
use crate::layers::OscillatorySignature;
use crate::runtime_platform::compat_2d3d::SimWorldTransformParams;
use crate::runtime_platform::core_math_agnostic::sim_plane_pos;
use crate::world::SpatialIndex;

/// Applies one Kuramoto entrainment step to all oscillatory entities.
///
/// Algorithm:
/// 1. Collect (entity, freq, pos) for all oscillatory entities — avoids borrow conflict.
/// 2. For each entity find spatial neighbours within `ENTRAINMENT_SCAN_RADIUS`.
/// 3. Compute new frequency via `kuramoto_entrainment_step` with distance purity.
/// 4. Apply with change-detection guard.
pub fn entrainment_system(
    spatial_index: Res<SpatialIndex>,
    layout: Res<SimWorldTransformParams>,
    mut query: Query<(Entity, &mut OscillatorySignature, &Transform)>,
) {
    // Phase 1: snapshot — read-only, no mutable borrows yet.
    let snapshot: Vec<(Entity, f32, Vec2)> = query
        .iter()
        .map(|(e, osc, t)| {
            let pos = sim_plane_pos(t.translation, layout.use_xz_ground);
            (e, osc.frequency_hz(), pos)
        })
        .collect();

    if snapshot.is_empty() {
        return;
    }

    // Phase 2: compute deltas from neighbours.
    // Sort snapshot by entity index for O(log n) lookup per neighbour.
    let mut sorted_snapshot = snapshot.clone();
    sorted_snapshot.sort_unstable_by_key(|(e, _, _)| e.index());

    let mut neighbours_scratch: Vec<(f32, f32)> =
        Vec::with_capacity(ENTRAINMENT_MAX_NEIGHBOURS * snapshot.len());
    let mut offsets: Vec<usize> = Vec::with_capacity(snapshot.len() + 1);
    offsets.push(0);

    for &(entity_i, _freq_i, pos_i) in &snapshot {
        let nearby = spatial_index.query_radius(pos_i, ENTRAINMENT_SCAN_RADIUS);
        let count_before = neighbours_scratch.len();
        for entry in nearby.iter().filter(|e| e.entity != entity_i) {
            // O(log n) binary search on sorted snapshot
            let Ok(idx_j) = sorted_snapshot.binary_search_by_key(&entry.entity.index(), |(e, _, _)| e.index()) else {
                continue;
            };
            let (_, freq_j, pos_j) = sorted_snapshot[idx_j];
            let dist = pos_i.distance(pos_j);
            neighbours_scratch.push((freq_j, dist));
            if neighbours_scratch.len() - count_before >= ENTRAINMENT_MAX_NEIGHBOURS {
                break;
            }
        }
        offsets.push(neighbours_scratch.len());
    }

    // Phase 3: apply updates with change-detection guard.
    for (idx, (entity_i, freq_i, _)) in snapshot.iter().enumerate() {
        let slice = &neighbours_scratch[offsets[idx]..offsets[idx + 1]];

        // Already locked? Skip (saves writes and reduces unnecessary change events).
        if slice.iter().all(|&(fj, _)| entrainment_lock_achieved(*freq_i, fj, KURAMOTO_LOCK_THRESHOLD_HZ)) {
            continue;
        }

        let new_freq = kuramoto_entrainment_step(
            *freq_i,
            slice,
            KURAMOTO_BASE_COUPLING,
            ENTRAINMENT_COHERENCE_LAMBDA,
            1.0, // dt = 1 tick (FixedUpdate)
        );

        if let Ok((_, mut osc, _)) = query.get_mut(*entity_i) {
            // 1e-5 tolerance: avoids spurious writes from accumulated f32 rounding error
            if (osc.frequency_hz() - new_freq).abs() > 1e-5 {
                osc.set_frequency_hz(new_freq);
            }
        }
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::world::space::SpatialEntry;

    fn test_app() -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.insert_resource(SimWorldTransformParams::default());
        app
    }

    fn build_index(entities: &[(Entity, Vec2, f32)]) -> SpatialIndex {
        let mut idx = SpatialIndex::new(ENTRAINMENT_SCAN_RADIUS);
        for &(e, pos, radius) in entities {
            idx.insert(SpatialEntry { entity: e, position: pos, radius });
        }
        idx
    }

    #[test]
    fn isolated_oscillator_keeps_frequency() {
        let mut app = test_app();
        let e = app.world_mut().spawn((
            OscillatorySignature::new(75.0, 0.0),
            Transform::from_xyz(0.0, 0.0, 0.0),
        )).id();

        let idx = build_index(&[(e, Vec2::ZERO, 1.0)]);
        app.insert_resource(idx);
        app.add_systems(Update, entrainment_system);
        app.update();

        let osc = app.world().get::<OscillatorySignature>(e).unwrap();
        assert!((osc.frequency_hz() - 75.0).abs() < 1e-5, "isolated: {}", osc.frequency_hz());
    }

    #[test]
    fn two_close_oscillators_converge() {
        let mut app = test_app();
        let e1 = app.world_mut().spawn((
            OscillatorySignature::new(70.0, 0.0),
            Transform::from_xyz(0.0, 0.0, 0.0),
        )).id();
        let e2 = app.world_mut().spawn((
            OscillatorySignature::new(80.0, 0.0),
            Transform::from_xyz(1.0, 0.0, 0.0),
        )).id();

        let idx = build_index(&[
            (e1, Vec2::new(0.0, 0.0), 1.0),
            (e2, Vec2::new(1.0, 0.0), 1.0),
        ]);
        app.insert_resource(idx);
        app.add_systems(Update, entrainment_system);
        app.update();

        let f1 = app.world().get::<OscillatorySignature>(e1).unwrap().frequency_hz();
        let f2 = app.world().get::<OscillatorySignature>(e2).unwrap().frequency_hz();
        let gap_after = (f1 - f2).abs();
        assert!(gap_after < 10.0, "frequencies should converge: f1={f1} f2={f2} gap={gap_after}");
        assert!(f1 > 70.0, "lower oscillator should be pulled up: {f1}");
        assert!(f2 < 80.0, "higher oscillator should be pulled down: {f2}");
    }

    #[test]
    fn far_neighbour_has_less_pull() {
        let mut app_near = test_app();
        let e_target = app_near.world_mut().spawn((
            OscillatorySignature::new(70.0, 0.0),
            Transform::from_xyz(0.0, 0.0, 0.0),
        )).id();
        let e_near = app_near.world_mut().spawn((
            OscillatorySignature::new(80.0, 0.0),
            Transform::from_xyz(1.0, 0.0, 0.0),
        )).id();
        let idx_near = build_index(&[
            (e_target, Vec2::ZERO, 1.0),
            (e_near, Vec2::new(1.0, 0.0), 1.0),
        ]);
        app_near.insert_resource(idx_near);
        app_near.add_systems(Update, entrainment_system);
        app_near.update();
        let freq_after_near = app_near.world().get::<OscillatorySignature>(e_target).unwrap().frequency_hz();

        let mut app_far = test_app();
        let e_target2 = app_far.world_mut().spawn((
            OscillatorySignature::new(70.0, 0.0),
            Transform::from_xyz(0.0, 0.0, 0.0),
        )).id();
        let e_far = app_far.world_mut().spawn((
            OscillatorySignature::new(80.0, 0.0),
            Transform::from_xyz(10.0, 0.0, 0.0),
        )).id();
        let idx_far = build_index(&[
            (e_target2, Vec2::ZERO, 1.0),
            (e_far, Vec2::new(10.0, 0.0), 1.0),
        ]);
        app_far.insert_resource(idx_far);
        app_far.add_systems(Update, entrainment_system);
        app_far.update();
        let freq_after_far = app_far.world().get::<OscillatorySignature>(e_target2).unwrap().frequency_hz();

        assert!(freq_after_near > freq_after_far, "near={freq_after_near} should move more than far={freq_after_far}");
    }

    #[test]
    fn out_of_range_neighbour_has_no_effect() {
        let mut app = test_app();
        let e1 = app.world_mut().spawn((
            OscillatorySignature::new(70.0, 0.0),
            Transform::from_xyz(0.0, 0.0, 0.0),
        )).id();
        let _e2 = app.world_mut().spawn((
            OscillatorySignature::new(80.0, 0.0),
            Transform::from_xyz(100.0, 0.0, 0.0), // way outside ENTRAINMENT_SCAN_RADIUS
        )).id();

        // Only e1 in the spatial index query result for e1 (e2 is too far)
        let idx = build_index(&[(e1, Vec2::ZERO, 1.0)]); // e2 not inserted near e1
        app.insert_resource(idx);
        app.add_systems(Update, entrainment_system);
        app.update();

        let f1 = app.world().get::<OscillatorySignature>(e1).unwrap().frequency_hz();
        assert!((f1 - 70.0).abs() < 1e-5, "out-of-range: should not move: {f1}");
    }
}
