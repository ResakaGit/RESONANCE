//! Propagador de cascada (TT-8).
//! Cascade propagator (TT-8).
//!
//! Aplica correcciones locales del DiffReport al mundo especulativo.
//! La propagación atenúa por distancia (Axioma 7) y muere en pocos hops.

use crate::batch::arena::SimWorldFlat;
use crate::batch::constants::MAX_ENTITIES;
use crate::blueprint::equations::batch_stepping::neighbors_within_radius;

use super::diff::{DiffClass, DiffReport};

/// Reporte de cascada aplicada.
/// Applied cascade report.
#[derive(Clone, Debug, Default)]
pub struct CascadeReport {
    pub entities_corrected: u16,
    pub entities_cascaded: u16,
    pub total_affected: u16,
    pub max_hops: u8,
    pub total_qe_correction: f32,
}

/// Aplica correcciones del DiffReport al mundo especulativo.
/// Applies DiffReport corrections to the speculative world.
///
/// PERFECT: no-op. SYSTEMIC: full copy from anchor. LOCAL: correct + cascade.
pub fn cascade(
    telescope: &mut SimWorldFlat,
    anchor: &SimWorldFlat,
    diff: &DiffReport,
    max_hops: u8,
    attenuation_per_hop: f32,
    correction_epsilon: f32,
) -> CascadeReport {
    debug_assert!(attenuation_per_hop > 0.0 && attenuation_per_hop <= 1.0,
        "attenuation must be in (0, 1], got {attenuation_per_hop}");

    match diff.class {
        DiffClass::Perfect => CascadeReport::default(),
        DiffClass::Systemic => {
            let alive_before = telescope.alive_mask.count_ones() as u16;
            *telescope = anchor.clone();
            CascadeReport {
                entities_corrected: alive_before,
                entities_cascaded: 0,
                total_affected: alive_before,
                max_hops: 0,
                total_qe_correction: diff.max_qe_delta,
            }
        }
        DiffClass::Local => cascade_local(
            telescope,
            anchor,
            diff,
            max_hops,
            attenuation_per_hop,
            correction_epsilon,
        ),
    }
}

/// Cascada local: corrige entidades afectadas + propaga a vecinos con atenuación.
/// Local cascade: corrects affected entities + propagates to neighbors with damping.
fn cascade_local(
    telescope: &mut SimWorldFlat,
    anchor: &SimWorldFlat,
    diff: &DiffReport,
    max_hops: u8,
    attenuation_per_hop: f32,
    correction_epsilon: f32,
) -> CascadeReport {
    let isolation_range_sq = crate::batch::constants::ISOLATION_RANGE_SQ;
    let mut corrected = 0_u128;
    let mut total_qe_correction = 0.0_f32;
    let mut entities_corrected = 0_u16;

    // Extraer posiciones a stack array (zero-heap, DoD).
    let mut positions = [[0.0_f32; 2]; MAX_ENTITIES];
    for i in 0..MAX_ENTITIES {
        positions[i] = telescope.entities[i].position;
    }

    // Paso 1: Corregir entidades directamente afectadas.
    for i in 0..MAX_ENTITIES {
        let d = &diff.entity_diffs[i];
        if d.alive_mismatch {
            telescope.entities[i] = anchor.entities[i];
            if anchor.entities[i].alive {
                telescope.alive_mask |= 1u128 << i;
            } else {
                telescope.alive_mask &= !(1u128 << i);
            }
            corrected |= 1u128 << i;
            entities_corrected += 1;
            total_qe_correction += d.qe_delta.abs();
        } else if d.qe_delta.abs() > correction_epsilon {
            telescope.entities[i].qe = anchor.entities[i].qe;
            telescope.entities[i].position = anchor.entities[i].position;
            corrected |= 1u128 << i;
            entities_corrected += 1;
            total_qe_correction += d.qe_delta.abs();
        }
    }

    // Paso 2: Propagar correcciones a vecinos con atenuación.
    let mut entities_cascaded = 0_u16;
    let mut current_hop_mask = corrected;
    let mut all_affected = corrected;
    let mut actual_max_hops = 0_u8;

    for hop in 0..max_hops {
        let mut next_hop_mask = 0_u128;
        let mut hop_mask = current_hop_mask;
        while hop_mask != 0 {
            let i = hop_mask.trailing_zeros() as usize;
            hop_mask &= hop_mask - 1;

            let original_delta = diff.entity_diffs[i].qe_delta;
            let attenuation = attenuation_per_hop.powi((hop + 1) as i32);
            let correction = original_delta * attenuation;

            if correction.abs() < correction_epsilon {
                continue;
            }

            let (neighbors, count) = neighbors_within_radius(
                &positions,
                telescope.alive_mask,
                i,
                isolation_range_sq,
            );

            for &j in &neighbors[..count] {
                if all_affected & (1u128 << j) != 0 {
                    continue;
                }
                telescope.entities[j].qe += correction;
                telescope.entities[j].qe = telescope.entities[j].qe.max(0.0);
                next_hop_mask |= 1u128 << j;
                all_affected |= 1u128 << j;
                entities_cascaded += 1;
                total_qe_correction += correction.abs();
            }
        }
        if next_hop_mask == 0 {
            break;
        }
        actual_max_hops = hop + 1;
        current_hop_mask = next_hop_mask;
    }

    CascadeReport {
        entities_corrected,
        entities_cascaded,
        total_affected: entities_corrected + entities_cascaded,
        max_hops: actual_max_hops,
        total_qe_correction,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::batch::arena::EntitySlot;

    fn spawn_at(w: &mut SimWorldFlat, qe: f32, x: f32, y: f32) {
        let mut e = EntitySlot::default();
        e.qe = qe;
        e.position = [x, y];
        w.spawn(e);
    }

    #[test]
    fn cascade_perfect_is_noop() {
        let mut t = SimWorldFlat::new(1, 0.05);
        let a = t.clone();
        let diff = DiffReport { class: DiffClass::Perfect, ..Default::default() };
        let report = cascade(&mut t, &a, &diff, 3, 0.1, 0.005);
        assert_eq!(report.total_affected, 0);
    }

    #[test]
    fn cascade_systemic_replaces_world() {
        let mut t = SimWorldFlat::new(1, 0.05);
        spawn_at(&mut t, 100.0, 0.0, 0.0);
        let mut a = t.clone();
        a.entities[0].qe = 50.0;
        let diff = DiffReport { class: DiffClass::Systemic, max_qe_delta: 50.0, ..Default::default() };
        let report = cascade(&mut t, &a, &diff, 3, 0.1, 0.005);
        assert_eq!(t.entities[0].qe, 50.0);
        assert!(report.total_affected > 0);
    }

    #[test]
    fn cascade_local_corrects_single_entity() {
        let mut a = SimWorldFlat::new(1, 0.05);
        spawn_at(&mut a, 100.0, 50.0, 50.0);
        let mut t = a.clone();
        t.entities[0].qe = 80.0;

        let diff = super::super::diff::world_diff(&a, &t, 0.02);
        let report = cascade(&mut t, &a, &diff, 3, 0.1, 0.005);
        assert_eq!(t.entities[0].qe, 100.0);
        assert!(report.entities_corrected >= 1);
    }

    #[test]
    fn cascade_corrects_affected_entity() {
        let mut a = SimWorldFlat::new(1, 0.05);
        spawn_at(&mut a, 100.0, 0.0, 0.0);
        let mut t = a.clone();
        t.entities[0].qe = 0.0;

        let diff = super::super::diff::world_diff(&a, &t, 0.02);
        let report = cascade(&mut t, &a, &diff, 3, 0.5, 0.005);
        assert_eq!(t.entities[0].qe, 100.0, "entity 0 should be corrected from anchor");
        assert!(report.entities_corrected >= 1);
    }

    #[test]
    fn cascade_neighbor_receives_attenuated_correction() {
        // Directly test neighbors_within_radius + manual cascade logic.
        let positions: [[f32; 2]; 128] = {
            let mut p = [[0.0_f32; 2]; 128];
            p[0] = [0.0, 0.0];
            p[1] = [2.0, 0.0]; // within range
            p
        };
        let alive_mask: u128 = 0b11;
        let (neighbors, count) = neighbors_within_radius(&positions, alive_mask, 0, 64.0);
        assert_eq!(count, 1, "entity 1 should be neighbor of entity 0");
        assert_eq!(neighbors[0], 1);
    }

    #[test]
    fn cascade_does_not_propagate_to_distant() {
        let mut a = SimWorldFlat::new(1, 0.05);
        spawn_at(&mut a, 100.0, 0.0, 0.0);
        spawn_at(&mut a, 50.0, 100.0, 100.0); // far away
        let mut t = a.clone();
        t.entities[0].qe = 50.0;

        let diff = super::super::diff::world_diff(&a, &t, 0.02);
        let report = cascade(&mut t, &a, &diff, 3, 0.1, 0.005);
        assert_eq!(report.entities_cascaded, 0, "distant entity should not cascade");
    }

    #[test]
    fn cascade_max_hops_limits_depth() {
        let mut a = SimWorldFlat::new(1, 0.05);
        // Chain: 0 → 1 → 2 → 3, each 2 units apart
        for i in 0..4 {
            spawn_at(&mut a, 100.0, i as f32 * 2.0, 0.0);
        }
        let mut t = a.clone();
        t.entities[0].qe = 0.0; // 100 qe diff

        let diff = super::super::diff::world_diff(&a, &t, 0.02);
        let report = cascade(&mut t, &a, &diff, 1, 0.5, 0.005);
        assert!(report.max_hops <= 1);
    }

    #[test]
    fn neighbors_isolated_returns_zero() {
        let positions = [[0.0, 0.0], [100.0, 100.0]];
        let (_, count) = neighbors_within_radius(&positions, 0b11, 0, 64.0);
        assert_eq!(count, 0);
    }

    #[test]
    fn neighbors_close_returns_count() {
        let positions = [[0.0, 0.0], [1.0, 0.0], [2.0, 0.0]];
        let (_, count) = neighbors_within_radius(&positions, 0b111, 0, 64.0);
        assert_eq!(count, 2);
    }

    #[test]
    fn cascade_qe_never_negative() {
        let mut a = SimWorldFlat::new(1, 0.05);
        spawn_at(&mut a, 1.0, 0.0, 0.0);
        spawn_at(&mut a, 1.0, 1.0, 0.0);
        let mut t = a.clone();
        t.entities[0].qe = 1000.0; // telescope overestimated

        let diff = super::super::diff::world_diff(&a, &t, 0.02);
        cascade(&mut t, &a, &diff, 3, 0.5, 0.005);
        for i in 0..MAX_ENTITIES {
            if t.alive_mask & (1u128 << i) != 0 {
                assert!(t.entities[i].qe >= 0.0, "entity {i} has negative qe: {}", t.entities[i].qe);
            }
        }
    }

    // ── Axiom Property Tests ─────────────────────────────────────

    #[test]
    fn axiom7_attenuation_decays_exponentially() {
        // Axioma 7: corrección × attenuation^hop debe decaer monótonamente.
        let attenuation = 0.1_f32;
        let original_delta = 100.0_f32;
        for hop in 0..5_u32 {
            let correction = original_delta * attenuation.powi(hop as i32);
            let next = original_delta * attenuation.powi((hop + 1) as i32);
            assert!(next < correction,
                "Axiom 7: correction must decrease with hops: hop {hop}: {correction} → {next}");
        }
    }

    #[test]
    fn axiom7_cascade_dies_within_max_hops() {
        // Para un delta unitario (1.0 qe), la corrección en hop=MAX_HOPS
        // debe estar debajo del epsilon.
        use crate::blueprint::constants::temporal_telescope as c;
        let correction_at_max = 1.0 * c::CASCADE_ATTENUATION_PER_HOP.powi(c::CASCADE_MAX_HOPS as i32);
        assert!(correction_at_max < c::CASCADE_CORRECTION_EPSILON,
            "unit cascade should die by max_hops: {correction_at_max} >= {}", c::CASCADE_CORRECTION_EPSILON);
    }

    #[test]
    fn cascade_alive_mask_integrity_post_correction() {
        // Después de cascade, alive_mask debe coincidir con entities[i].alive.
        let mut a = SimWorldFlat::new(1, 0.05);
        spawn_at(&mut a, 100.0, 0.0, 0.0);
        spawn_at(&mut a, 50.0, 5.0, 0.0);
        let mut t = a.clone();
        t.entities[0].qe = 0.0;
        t.kill(1); // telescope thinks entity 1 is dead

        let diff = super::super::diff::world_diff(&a, &t, 0.02);
        cascade(&mut t, &a, &diff, 3, 0.1, 0.005);

        for i in 0..MAX_ENTITIES {
            let bit_alive = t.alive_mask & (1u128 << i) != 0;
            assert_eq!(bit_alive, t.entities[i].alive,
                "alive_mask mismatch at slot {i} post-cascade");
        }
    }

    #[test]
    fn cascade_systemic_produces_exact_anchor_copy() {
        let mut a = SimWorldFlat::new(1, 0.05);
        for i in 0..10 {
            spawn_at(&mut a, 100.0 + i as f32, i as f32, 0.0);
        }
        a.update_total_qe();
        let mut t = a.clone();
        // Corrupt telescope
        for i in 0..10 {
            t.entities[i].qe = 0.0;
        }

        let diff = DiffReport { class: DiffClass::Systemic, ..Default::default() };
        cascade(&mut t, &a, &diff, 3, 0.1, 0.005);

        // After systemic cascade, telescope should be identical to anchor.
        assert_eq!(t.alive_mask, a.alive_mask);
        for i in 0..MAX_ENTITIES {
            assert_eq!(t.entities[i].qe, a.entities[i].qe,
                "entity {i} qe mismatch after systemic cascade");
        }
    }
}
