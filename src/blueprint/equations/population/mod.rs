use crate::blueprint::constants::*;

// ═══════════════════════════════════════════════
// Ecosistema: exclusión competitiva por densidad (EA7)
// ═══════════════════════════════════════════════

/// Penalización de energía por competencia: más vecinos en la misma celda = más drain.
/// Retorna el qe a drenar por tick.
/// `competitors`: número de entidades en la misma celda (incluida la propia).
/// `base_drain`: drain base por competidor extra.
/// `resilience`: \[0, 1\] — alta resiliencia reduce el drain.
#[inline]
pub fn competition_energy_drain(competitors: u32, base_drain: f32, resilience: f32) -> f32 {
    if competitors <= 1 {
        return 0.0;
    }
    let extra = (competitors - 1) as f32;
    let drain = if base_drain.is_finite() {
        base_drain.max(0.0)
    } else {
        0.0
    };
    let r = if resilience.is_finite() {
        resilience.clamp(0.0, 1.0)
    } else {
        0.0
    };
    let raw = extra * drain * (1.0 - r * COMPETITION_RESILIENCE_DRAIN_ATTENUATION);
    if raw.is_finite() {
        raw.max(0.0)
    } else {
        0.0
    }
}

// ═══════════════════════════════════════════════
// Ecosistema: reproducción por biomasa (EA6)
// ═══════════════════════════════════════════════

/// Evalúa si una entidad tiene suficiente biomasa para reproducirse.
/// `true` si `current_radius` supera el umbral derivado de `base_radius`, `branching_bias` y `reproduction_radius_factor`.
#[inline]
pub fn can_reproduce(
    current_radius: f32,
    base_radius: f32,
    branching_bias: f32,
    reproduction_radius_factor: f32,
) -> bool {
    if current_radius <= 0.0 || base_radius <= 0.0 {
        return false;
    }
    let factor = reproduction_radius_factor * (1.0 - branching_bias * REPRODUCTION_BRANCHING_BETA);
    let threshold = base_radius * factor.max(REPRODUCTION_EFFECTIVE_FACTOR_FLOOR);
    current_radius >= threshold
}

/// Muta un sesgo de `InferenceProfile` con perturbación acotada.
/// `drift` se clampea a `[-max_drift, +max_drift]`; resultado en \[0, 1\].
#[inline]
pub fn mutate_bias(value: f32, drift: f32, max_drift: f32) -> f32 {
    let max_d = if max_drift.is_finite() {
        max_drift.max(0.0)
    } else {
        0.0
    };
    let d = if drift.is_finite() {
        drift.clamp(-max_d, max_d)
    } else {
        0.0
    };
    let v = if value.is_finite() {
        value
    } else {
        MUTATE_BIAS_NONFINITE_VALUE_FALLBACK
    };
    (v + d).clamp(0.0, 1.0)
}
