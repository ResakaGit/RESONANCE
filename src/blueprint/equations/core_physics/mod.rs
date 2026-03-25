use bevy::math::Vec2;
use std::f32::consts::PI;
use crate::blueprint::constants::*;
use crate::layers::MatterState;

// ══════════════════════════════════════════════════════════════
// Funciones puras que implementan toda la matemática del motor.
// Sin dependencia de ECS — los sistemas llaman a estas funciones.
// ══════════════════════════════════════════════════════════════

// ═══════════════════════════════════════════════
// Capa 0 × Capa 1: Densidad
// ═══════════════════════════════════════════════

/// Volumen de una esfera: V = (4/3) * π * r³ (mismas constantes que `SpatialVolume` / blueprint).
pub fn sphere_volume(radius: f32) -> f32 {
    (SPHERE_VOLUME_NUMERATOR / SPHERE_VOLUME_DENOMINATOR) * PI * radius.powi(3)
}

/// Área proyectada (sección circular): A_proj = π r².
#[inline]
pub fn projected_circle_area(radius: f32) -> f32 {
    PI * radius * radius
}

/// Superficie de esfera: A_surf = 4 π r².
#[inline]
pub fn sphere_surface_area(radius: f32) -> f32 {
    4.0 * PI * radius * radius
}

/// Densidad: ρ = qe / V
pub fn density(qe: f32, radius: f32) -> f32 {
    let v = sphere_volume(radius);
    if v > 0.0 {
        qe / v
    } else {
        f32::MAX
    }
}

// ═══════════════════════════════════════════════
// Capa 2: Interferencia de Ondas
// ═══════════════════════════════════════════════

/// Interferencia entre dos firmas oscilatorias en el instante `t`.
/// I = cos(2π * |f₁ - f₂| * t + (φ₁ - φ₂))
/// Retorna valor en [-1.0, 1.0].
pub fn interference(f1: f32, phase1: f32, f2: f32, phase2: f32, t: f32) -> f32 {
    let delta_f = (f1 - f2).abs();
    let delta_phase = phase1 - phase2;
    (2.0 * PI * delta_f * t + delta_phase).cos()
}

/// ¿La interferencia es constructiva?
pub fn is_constructive(interference: f32) -> bool {
    interference > CONSTRUCTIVE_THRESHOLD
}

/// ¿La interferencia es destructiva?
pub fn is_destructive(interference: f32) -> bool {
    interference < DESTRUCTIVE_THRESHOLD
}

/// ¿Es un golpe crítico?
pub fn is_critical(interference: f32) -> bool {
    interference.abs() > CRITICAL_THRESHOLD
}

// ═══════════════════════════════════════════════
// Capa 3: Disipación y Arrastre
// ═══════════════════════════════════════════════

/// Tasa de disipación efectiva con fricción cinética.
/// d_eff = d_base + coef_friccion * |v|²
pub fn effective_dissipation(base_rate: f32, velocity: Vec2, friction_coef: f32) -> f32 {
    base_rate + friction_coef * velocity.length_squared()
}

/// Fuerza de arrastre del terreno.
/// F_drag = -0.5 * viscosidad * ρ * |v| * v
pub fn drag_force(viscosity: f32, density: f32, velocity: Vec2) -> Vec2 {
    let speed = velocity.length();
    if speed < DRAG_SPEED_EPSILON {
        return Vec2::ZERO;
    }
    -0.5 * viscosity * density * speed * velocity
}

/// Integración de velocidad con fuerza de voluntad.
/// v_new = v_old + (fuerza / masa) * dt
/// masa_efectiva = qe (la energía actúa como inercia)
pub fn integrate_velocity(velocity: Vec2, force: Vec2, qe: f32, dt: f32) -> Vec2 {
    if qe <= 0.0 {
        return velocity;
    }
    velocity + (force / qe) * dt
}

// ═══════════════════════════════════════════════
// Capa 4: Transiciones de Estado
// ═══════════════════════════════════════════════

/// Temperatura equivalente desde densidad.
/// T = ρ / k_boltzmann_juego
pub fn equivalent_temperature(density: f32) -> f32 {
    density / GAME_BOLTZMANN
}

/// Determina el estado de la materia según la temperatura relativa a la energía de enlace.
/// T < 0.3 * eb → Sólido
/// T < 1.0 * eb → Líquido
/// T < 3.0 * eb → Gas
/// T ≥ 3.0 * eb → Plasma
pub fn state_from_temperature(temp: f32, bond_energy: f32) -> MatterState {
    if temp < SOLID_TRANSITION * bond_energy {
        MatterState::Solid
    } else if temp < LIQUID_TRANSITION * bond_energy {
        MatterState::Liquid
    } else if temp < GAS_TRANSITION * bond_energy {
        MatterState::Gas
    } else {
        MatterState::Plasma
    }
}
