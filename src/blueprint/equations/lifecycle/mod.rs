use super::finite_helpers::{finite_non_negative, finite_unit};
use crate::blueprint::constants::*;
use crate::layers::LifecycleStage;

#[inline]
fn can_enter_reproductive_stage(viability: f32, biomass: f32, can_reproduce: bool) -> bool {
    can_reproduce
        && biomass > LIFECYCLE_REPRODUCTIVE_BIOMASS
        && viability > LIFECYCLE_REPRODUCTIVE_VIABILITY_MIN
}

/// Progreso de crecimiento normalizado en `[0, 1]`.
#[inline]
pub fn growth_progress(current_radius: f32, base_radius: f32, max_factor: f32) -> f32 {
    let radius = finite_non_negative(current_radius);
    let base = finite_non_negative(base_radius);
    let factor = finite_non_negative(max_factor);
    let max_radius = base * factor;
    if max_radius <= 0.0 {
        return 0.0;
    }
    finite_unit(radius / max_radius)
}

/// Infiere la fase de ciclo de vida desde viabilidad, progreso de crecimiento y biomasa.
#[inline]
pub fn infer_lifecycle_stage(
    viability: f32,
    growth_progress: f32,
    biomass: f32,
    can_reproduce: bool,
) -> LifecycleStage {
    let v = finite_non_negative(viability);
    let growth = finite_unit(growth_progress);
    let bio = finite_non_negative(biomass);

    if v < LIFECYCLE_DECLINING_VIABILITY {
        return LifecycleStage::Declining;
    }
    if v < LIFECYCLE_DORMANT_VIABILITY {
        return LifecycleStage::Dormant;
    }
    if growth < LIFECYCLE_EMERGING_GROWTH {
        return LifecycleStage::Emerging;
    }
    if growth < LIFECYCLE_MATURE_GROWTH {
        return LifecycleStage::Growing;
    }
    if can_enter_reproductive_stage(v, bio, can_reproduce) {
        return LifecycleStage::Reproductive;
    }
    LifecycleStage::Mature
}

/// Histeresis para transición de fase; deterioro (`Declining`) transiciona inmediato.
#[inline]
pub fn lifecycle_stage_with_hysteresis(
    current: LifecycleStage,
    inferred: LifecycleStage,
    candidate_stage: Option<LifecycleStage>,
    candidate_ticks: u16,
    min_ticks_for_transition: u16,
) -> (LifecycleStage, Option<LifecycleStage>, u16) {
    if inferred == LifecycleStage::Declining && current != LifecycleStage::Declining {
        return (LifecycleStage::Declining, None, 0);
    }
    if inferred == current {
        return (current, None, 0);
    }
    if min_ticks_for_transition == 0 {
        return (inferred, None, 0);
    }
    if candidate_stage == Some(inferred) {
        let next_ticks = candidate_ticks.saturating_add(1);
        if next_ticks >= min_ticks_for_transition {
            return (inferred, None, 0);
        }
        return (current, Some(inferred), next_ticks);
    }
    (current, Some(inferred), 1)
}
