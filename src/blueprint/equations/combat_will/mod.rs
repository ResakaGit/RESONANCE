use crate::blueprint::constants::*;
use crate::blueprint::equations::core_physics::{is_constructive, is_critical, is_destructive};
use crate::math_types::Vec2;

// ═══════════════════════════════════════════════
// Capa 8: Catálisis
// ═══════════════════════════════════════════════

/// Calcula el daño/curación final de un evento de catálisis.
/// Positivo = curación (constructivo), Negativo = daño (destructivo).
pub fn catalysis_result(projected_qe: f32, interference: f32, critical_multiplier: f32) -> f32 {
    let base = if is_constructive(interference) {
        projected_qe * interference
    } else if is_destructive(interference) {
        -projected_qe * interference.abs()
    } else {
        // Ortogonal: efecto mínimo
        return 0.0;
    };

    if is_critical(interference) {
        base * critical_multiplier
    } else {
        base
    }
}

/// Factor de resonance lock: cuánto se acerca la frecuencia del objetivo
/// a la del hechizo durante interferencia constructiva.
pub fn frequency_lock_delta(spell_freq: f32, target_freq: f32) -> f32 {
    (spell_freq - target_freq) * RESONANCE_LOCK_FACTOR
}

/// Factor de debilitamiento de enlace durante interferencia destructiva.
pub fn weakening_factor(interference: f32) -> f32 {
    1.0 + interference * BOND_WEAKENING_FACTOR
}

// ═══════════════════════════════════════════════
// Capa 7: Fuerza de Voluntad
// ═══════════════════════════════════════════════

/// Fuerza generada por el actuador de voluntad.
/// F = dirección_normalizada * potencia_motor * (buffer / buffer_max)
pub fn will_force(intent: Vec2, current_buffer: f32, max_buffer: f32) -> Vec2 {
    let efficiency = if max_buffer > 0.0 {
        (current_buffer / max_buffer).clamp(0.0, 1.0)
    } else {
        0.0
    };
    intent * BASE_MOTOR_POWER * efficiency
}
