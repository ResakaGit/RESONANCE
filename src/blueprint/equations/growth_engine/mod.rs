use crate::blueprint::constants::*;

// ═══════════════════════════════════════════════
// Capa 5: Motor Alquímico
// ═══════════════════════════════════════════════

/// Intake alométrico: escala con superficie efectiva (`volume^(2/3)`).
#[inline]
pub fn allometric_intake(base_intake: f32, radius: f32) -> f32 {
    let intake = if base_intake.is_finite() {
        base_intake.max(0.0)
    } else {
        0.0
    };
    if intake <= 0.0 {
        return 0.0;
    }
    let r = if radius.is_finite() {
        radius.max(0.0)
    } else {
        0.0
    };
    // `sphere_volume(r)^(2/3)` con esfera => `ALLOMETRIC_SURFACE_FACTOR * r^2`.
    let surface_proxy = ALLOMETRIC_SURFACE_FACTOR * r * r;
    (intake * surface_proxy).max(ALLOMETRIC_INTAKE_FLOOR)
}

/// Output/consumo alométrico: misma ley de escala que intake.
#[inline]
pub fn allometric_consumption(base_output: f32, radius: f32) -> f32 {
    let output = if base_output.is_finite() {
        base_output.max(0.0)
    } else {
        0.0
    };
    if output <= 0.0 {
        return 0.0;
    }
    let r = if radius.is_finite() {
        radius.max(0.0)
    } else {
        0.0
    };
    let surface_proxy = ALLOMETRIC_SURFACE_FACTOR * r * r;
    let out = output * surface_proxy;
    if out.is_finite() { out } else { 0.0 }
}

/// Feedback logístico de crecimiento radial.
#[inline]
pub fn growth_size_feedback(growth_budget: f32, current_radius: f32, max_radius: f32) -> f32 {
    if !growth_budget.is_finite() || growth_budget <= 0.0 {
        return 0.0;
    }
    let current = if current_radius.is_finite() {
        current_radius.max(0.0)
    } else {
        0.0
    };
    let max_r = if max_radius.is_finite() {
        max_radius.max(0.0)
    } else {
        0.0
    };
    if max_r <= 0.0 || current >= max_r {
        return 0.0;
    }
    let logistic = (1.0 - current / max_r).clamp(0.0, 1.0);
    growth_budget.max(0.0) * ALLOMETRIC_GROWTH_RATE * logistic
}

/// Modula delta de crecimiento con sesgo de inferencia y energía disponible.
#[inline]
pub fn inferred_growth_delta(
    base_delta: f32,
    growth_bias: f32,
    resilience: f32,
    qe_norm: f32,
) -> f32 {
    if !base_delta.is_finite() || base_delta <= 0.0 {
        return 0.0;
    }
    let g = if growth_bias.is_finite() {
        growth_bias.clamp(0.0, 1.0)
    } else {
        0.0
    };
    let r = if resilience.is_finite() {
        resilience.clamp(0.0, 1.0)
    } else {
        0.0
    };
    let q = if qe_norm.is_finite() {
        qe_norm.clamp(0.0, 1.0)
    } else {
        0.0
    };
    let modifier = (0.35 + 0.35 * g + 0.30 * r + 0.35 * q).clamp(0.0, 1.0);
    (base_delta * modifier).max(0.0)
}

/// Maximum allowed radius for allometric growth: `base * factor`, floored at `VOLUME_MIN_RADIUS`.
#[inline]
pub fn allometric_max_radius(base_radius: f32, max_factor: f32) -> f32 {
    (base_radius.max(0.0) * max_factor.max(0.0)).max(VOLUME_MIN_RADIUS)
}

/// Normalizes qe against a reference value, clamped to [0, 1].
#[inline]
pub fn normalized_qe(qe: f32, reference: f32) -> f32 {
    (qe / reference.max(1.0)).clamp(0.0, 1.0)
}

/// Cantidad de intake que el motor puede absorber este tick.
/// intake = min(valvula_entrada * dt, qe_disponible, espacio_libre)
pub fn engine_intake(
    input_valve: f32,
    dt: f32,
    qe_available: f32,
    current_buffer: f32,
    max_buffer: f32,
) -> f32 {
    let max_tick = input_valve * dt;
    let space = (max_buffer - current_buffer).max(0.0);
    max_tick.min(qe_available).min(space)
}

/// Variante de intake que aplica escalamiento alométrico sobre la válvula de entrada.
#[inline]
pub fn engine_intake_allometric(
    input_valve: f32,
    dt: f32,
    qe_available: f32,
    current_buffer: f32,
    max_buffer: f32,
    radius: f32,
) -> f32 {
    let effective_input = allometric_intake(input_valve, radius);
    engine_intake(
        effective_input,
        dt,
        qe_available,
        current_buffer,
        max_buffer,
    )
}
