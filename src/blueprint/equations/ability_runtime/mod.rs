use crate::math_types::Vec2;
use crate::layers::{AbilitySlot, AlchemicalEngine};

// ═══════════════════════════════════════════════
// Grimoire × Motor: “cooldown” = buffer / intake (sin timers en slots)
// ═══════════════════════════════════════════════

/// ¿El motor tiene buffer suficiente para costear el cast? Única condición tipo cooldown.
#[inline]
pub fn can_cast(engine: &AlchemicalEngine, slot: &AbilitySlot) -> bool {
    let cost = slot.cost_qe().max(0.0);
    cost > 0.0 && engine.buffer_level() >= cost
}

/// Fracción de barra MOBA: 1.0 = listo para pagar `cost_qe`, 0.0 = vacío.
#[inline]
pub fn cooldown_fraction(engine: &AlchemicalEngine, slot: &AbilitySlot) -> f32 {
    let cost = slot.cost_qe();
    if cost <= 0.0 {
        return 1.0;
    }
    (engine.buffer_level() / cost).clamp(0.0, 1.0)
}

/// Tiempo estimado hasta poder pagar el costo (derivado; no se almacena).
#[inline]
pub fn estimated_recharge_secs(engine: &AlchemicalEngine, slot: &AbilitySlot) -> f32 {
    let cost = slot.cost_qe();
    let deficit = cost - engine.buffer_level();
    if deficit <= 0.0 {
        return 0.0;
    }
    deficit / engine.valve_in_rate().max(0.001)
}

/// ¿El punto en el plano sim está dentro del alcance plano del caster?
#[inline]
pub fn ability_point_in_cast_range(caster_plane: Vec2, target_plane: Vec2, range: f32) -> bool {
    let r = range.max(0.0);
    (caster_plane - target_plane).length_squared() <= r * r
}

/// Dissipation rate for spawned projectiles: base decay + radius-scaled component.
/// `d = 0.5 + 0.02 * spawn_radius`
#[inline]
pub fn projectile_dissipation(spawn_radius: f32) -> f32 {
    0.5 + 0.02 * spawn_radius.max(0.0)
}

