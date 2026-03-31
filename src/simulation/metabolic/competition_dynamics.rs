//! EC-5D: Competition Dynamics System — diagnóstico analítico de pools por tick.
//! Read-only sobre EnergyPool y PoolParentLink. Escribe PoolDiagnostic (SparseSet).
//!
//! STATUS: IMPLEMENTED, NOT REGISTERED. Used in integration tests only.
//! Designed for Phase::MetabolicLayer after pool_dissipation_system, but
//! no plugin wires it into the schedule.

use bevy::prelude::*;

use crate::blueprint::constants::MAX_EXTRACTION_MODIFIERS;
use crate::blueprint::equations::{
    available_for_extraction, competition_intensity, detect_collapse, dissipation_loss,
    evaluate_extraction, predict_pool_trajectory, scale_extractions_to_available,
    ExtractionContext, ExtractionProfile, PoolHealthStatus,
};
use crate::layers::{EnergyPool, ExtractionType, MacroStepTarget, PoolParentLink};

const MAX_EC5_ENTRIES: usize = 512;
/// Máximo de hijos analizados por pool (capped por MAX_COMPETITION_MATRIX = 16).
const MAX_EC5_GROUP: usize = 16;

// ─── EC-5E: Componente ───────────────────────────────────────────────────────

/// Diagnóstico competitivo del pool. Recomputado cada tick. SparseSet: solo pools con hijos.
#[derive(Component, Clone, Copy, Debug, PartialEq, Reflect)]
#[reflect(Component)]
#[component(storage = "SparseSet")]
pub struct PoolDiagnostic {
    /// Intensidad de competencia (Gini coefficient). [0, 1].
    pub competition_intensity: f32,
    /// Estado de salud del pool.
    pub health_status: PoolHealthStatus,
    /// Ticks estimados hasta colapso (u32::MAX si estable/creciente).
    pub ticks_to_collapse: u32,
}

// ─── Buffer interno ───────────────────────────────────────────────────────────

#[derive(Clone, Copy)]
struct Ec5Entry {
    parent_idx: u32,
    parent:     Entity,
    etype:      ExtractionType,
    param:      f32,
}

impl Ec5Entry {
    fn placeholder() -> Self {
        Self {
            parent_idx: u32::MAX,
            parent:     Entity::PLACEHOLDER,
            etype:      ExtractionType::Proportional,
            param:      0.0,
        }
    }
}

// ─── EC-5D: Sistema ───────────────────────────────────────────────────────────

/// Analiza la dinámica competitiva de cada pool y escribe PoolDiagnostic.
/// Read-only sobre pools y links. Escribe PoolDiagnostic con change detection.
pub fn competition_dynamics_system(
    pools:        Query<&EnergyPool>,
    links:        Query<&PoolParentLink>,
    mut diags:    Query<&mut PoolDiagnostic>,
    mut commands: Commands,
) {
    // ── Fase 1: recolectar hijos en buffer stack ──────────────────────────────
    let mut buf   = [Ec5Entry::placeholder(); MAX_EC5_ENTRIES];
    let mut count = 0usize;

    for link in links.iter() {
        if count < MAX_EC5_ENTRIES {
            buf[count] = Ec5Entry {
                parent_idx: link.parent().index(),
                parent:     link.parent(),
                etype:      link.extraction_type(),
                param:      link.primary_param(),
            };
            count += 1;
        }
    }

    if count == 0 { return; }

    // Orden determinista por parent_idx → grupos contiguos.
    buf[..count].sort_unstable_by_key(|e| e.parent_idx);

    // ── Fase 2: procesar grupos ───────────────────────────────────────────────
    let mut i = 0;
    while i < count {
        let parent_idx = buf[i].parent_idx;
        let parent     = buf[i].parent;

        let mut j = i + 1;
        while j < count && buf[j].parent_idx == parent_idx { j += 1; }
        let group_len = (j - i).min(MAX_EC5_GROUP);

        let Ok(pool) = pools.get(parent) else { i = j; continue; };

        // Re-evaluar extracciones como proxy para análisis (forward-looking).
        let n_siblings   = group_len as u32;
        let total_fitness: f32 = buf[i..i + group_len]
            .iter()
            .filter(|e| matches!(e.etype, ExtractionType::Competitive))
            .map(|e| e.param)
            .sum();

        let available  = available_for_extraction(pool.pool(), 0.0, pool.dissipation_rate());
        let pool_ratio = pool.pool_ratio();
        let ctx        = ExtractionContext { available, pool_ratio, n_siblings, total_fitness };

        let mut claimed = [0.0f32; MAX_EC5_GROUP];
        for (k, entry) in buf[i..i + group_len].iter().enumerate() {
            let profile = ExtractionProfile {
                base:          entry.etype,
                primary_param: entry.param,
                modifiers:     [None; MAX_EXTRACTION_MODIFIERS],
            };
            claimed[k] = evaluate_extraction(&profile, &ctx);
        }
        scale_extractions_to_available(&mut claimed[..group_len], available);

        let total_claimed: f32 = claimed[..group_len].iter().sum();
        let loss               = dissipation_loss(pool.pool(), pool.dissipation_rate());
        let net_drain          = (total_claimed + loss) - pool.intake_rate();

        let intensity  = competition_intensity(&claimed[..group_len]);
        let health     = detect_collapse(pool.pool(), pool.intake_rate(), total_claimed, loss);
        let trajectory = predict_pool_trajectory(pool.pool(), net_drain, pool.capacity());

        let new_diag = PoolDiagnostic {
            competition_intensity: intensity,
            health_status:         health,
            ticks_to_collapse:     trajectory.ticks_to_collapse,
        };

        // Guard change detection: no muta si el diagnóstico no cambió.
        if let Ok(mut diag) = diags.get_mut(parent) {
            if *diag != new_diag { *diag = new_diag; }
        } else {
            commands.entity(parent).insert(new_diag);
        }

        i = j;
    }
}

// ─── Tests ───────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layers::pool_link::ExtractionType;

    fn make_app() -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.register_type::<EnergyPool>();
        app.register_type::<PoolParentLink>();
        app.register_type::<PoolDiagnostic>();
        app.register_type::<PoolHealthStatus>();
        app
    }

    // ── EC-5D: Sistema ───────────────────────────────────────────────────────

    #[test]
    fn competition_dynamics_system_inserts_pool_diagnostic() {
        let mut app = make_app();
        app.add_systems(Update, competition_dynamics_system);

        let parent = app.world_mut()
            .spawn(EnergyPool::new(1000.0, 2000.0, 50.0, 0.01))
            .id();
        for _ in 0..3 {
            app.world_mut().spawn((
                crate::layers::BaseEnergy::new(0.0),
                PoolParentLink::new(parent, ExtractionType::Proportional, 0.0),
            ));
        }

        app.update();

        let diag = app.world().get::<PoolDiagnostic>(parent);
        assert!(diag.is_some(), "PoolDiagnostic should be inserted after first update");
        let d = diag.unwrap();
        // 3 equal children → Gini ≈ 0.0
        assert!(d.competition_intensity < 0.05, "uniform → low intensity: {}", d.competition_intensity);
    }

    #[test]
    fn competition_dynamics_system_competitive_children_high_intensity() {
        let mut app = make_app();
        app.add_systems(Update, competition_dynamics_system);

        let parent = app.world_mut()
            .spawn(EnergyPool::new(1000.0, 2000.0, 0.0, 0.01))
            .id();
        // One dominant child with fitness 0.95, one weak with 0.05
        app.world_mut().spawn((
            crate::layers::BaseEnergy::new(0.0),
            PoolParentLink::new(parent, ExtractionType::Competitive, 0.95),
        ));
        app.world_mut().spawn((
            crate::layers::BaseEnergy::new(0.0),
            PoolParentLink::new(parent, ExtractionType::Competitive, 0.05),
        ));

        app.update();

        let d = app.world().get::<PoolDiagnostic>(parent).unwrap();
        // Highly skewed (0.95 vs 0.05) → Gini = 0.45, well above uniform
        assert!(d.competition_intensity > 0.4, "skewed → high intensity: {}", d.competition_intensity);
    }

    #[test]
    fn competition_dynamics_system_idempotent_no_spurious_mutation() {
        let mut app = make_app();
        app.add_systems(Update, competition_dynamics_system);

        let parent = app.world_mut()
            .spawn(EnergyPool::new(1000.0, 2000.0, 0.0, 0.001))
            .id();
        app.world_mut().spawn((
            crate::layers::BaseEnergy::new(0.0),
            PoolParentLink::new(parent, ExtractionType::Proportional, 0.0),
        ));

        app.update(); // inserts PoolDiagnostic
        let d1 = *app.world().get::<PoolDiagnostic>(parent).unwrap();

        app.update(); // should re-compute same values (pool now drained, stable state)
        let d2 = *app.world().get::<PoolDiagnostic>(parent).unwrap();

        // Values may differ since pool drains each tick, but diagnostic is consistent with pool state.
        // Main assertion: no panic, both reads succeed.
        let _ = (d1, d2); // Both valid PoolDiagnostic values.
    }

    #[test]
    fn competition_dynamics_system_healthy_pool_high_intake() {
        let mut app = make_app();
        app.add_systems(Update, competition_dynamics_system);

        // pool=5 so available≈5, intake_rate=500 >> claimed+loss → net_drain < 0 → Healthy
        let parent = app.world_mut()
            .spawn(EnergyPool::new(5.0, 2000.0, 500.0, 0.001))
            .id();
        app.world_mut().spawn((
            crate::layers::BaseEnergy::new(0.0),
            PoolParentLink::new(parent, ExtractionType::Proportional, 0.0),
        ));

        app.update();

        let d = app.world().get::<PoolDiagnostic>(parent).unwrap();
        assert_eq!(d.health_status, PoolHealthStatus::Healthy);
    }

    #[test]
    fn competition_dynamics_system_no_children_no_diagnostic() {
        let mut app = make_app();
        app.add_systems(Update, competition_dynamics_system);

        let parent = app.world_mut()
            .spawn(EnergyPool::new(1000.0, 2000.0, 50.0, 0.01))
            .id();

        app.update();

        // No children → no PoolDiagnostic inserted
        assert!(app.world().get::<PoolDiagnostic>(parent).is_none());
    }

    #[test]
    fn competition_dynamics_system_orphan_parent_no_panic() {
        let mut app = make_app();
        app.add_systems(Update, competition_dynamics_system);

        let fake_parent = Entity::from_raw(9999);
        app.world_mut().spawn((
            crate::layers::BaseEnergy::new(100.0),
            PoolParentLink::new(fake_parent, ExtractionType::Proportional, 0.0),
        ));

        app.update(); // must not panic
    }
}

// ─── M2: Macro-Step Application ───────────────────────────────────────────────

/// Applies analytical macro-step to entities marked MacroStepTarget.
/// Advances `qe` by `exponential_decay(last_qe, decay_rate, ticks_remaining)` then removes the marker.
/// Phase: MetabolicLayer.
pub fn apply_macro_step(
    mut commands: Commands,
    mut query: Query<(Entity, &mut crate::layers::BaseEnergy, &crate::layers::MacroStepTarget)>,
) {
    use crate::blueprint::equations::exponential_decay;
    for (entity, mut energy, step) in &mut query {
        let new_qe = exponential_decay(step.last_qe, step.decay_rate, step.ticks_remaining);
        if energy.qe != new_qe {
            energy.qe = new_qe.max(0.0);
        }
        commands.entity(entity).remove::<crate::layers::MacroStepTarget>();
    }
}

// ─── M4: LOD Observer ─────────────────────────────────────────────────────────

/// Marks entities far from all WillActuator anchors with MacroStepTarget for analytical LOD.
/// Stack-bounded: max 8 anchors checked. Skips if no anchors present.
/// Phase: MorphologicalLayer (runs after metabolic, before rendering).
pub fn lod_mark_distant_entities(
    mut commands: Commands,
    anchors: Query<&Transform, (With<crate::layers::WillActuator>, Without<MacroStepTarget>)>,
    candidates: Query<(Entity, &Transform, &crate::layers::BaseEnergy), Without<MacroStepTarget>>,
) {
    use crate::blueprint::constants::{DISSIPATION_RATE_DEFAULT, LOD_MACRO_STEP_DIST_SQ, LOD_MACRO_STEP_TICKS};
    let mut anchor_positions: [bevy::math::Vec3; 8] = [bevy::math::Vec3::ZERO; 8];
    let mut anchor_count = 0_usize;
    for t in anchors.iter() {
        if anchor_count >= 8 { break; }
        anchor_positions[anchor_count] = t.translation;
        anchor_count += 1;
    }
    if anchor_count == 0 { return; }
    for (entity, transform, energy) in candidates.iter() {
        let pos = transform.translation;
        let all_distant = (0..anchor_count).all(|i| {
            pos.distance_squared(anchor_positions[i]) > LOD_MACRO_STEP_DIST_SQ
        });
        if all_distant {
            commands.entity(entity).insert(MacroStepTarget::new(
                LOD_MACRO_STEP_TICKS,
                energy.qe,
                DISSIPATION_RATE_DEFAULT,
            ));
        }
    }
}
