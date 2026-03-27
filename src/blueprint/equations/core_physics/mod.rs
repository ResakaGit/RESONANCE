use crate::math_types::Vec2;
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

/// Symplectic Euler (velocity Verlet half-step) for orbital mechanics.
///
/// Preserves angular momentum better than forward Euler for gravitational orbits.
/// `v_half = v + (a * dt/2)` then `pos_new = pos + v_half * dt` then `v_new = v_half + (a_new * dt/2)`.
/// This function computes the velocity half-step only; position update is done by the caller.
///
/// For arena-scale (low dt, short distances), identical to `integrate_velocity`.
/// For stellar-scale (large dt, orbital dynamics), preserves orbital energy.
pub fn integrate_velocity_verlet_half(velocity: Vec2, acceleration: Vec2, dt: f32) -> Vec2 {
    velocity + acceleration * (dt * 0.5)
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

#[cfg(test)]
mod tests {
    use super::*;

    const EPS: f32 = 1e-4;

    // ── sphere_volume ───────────────────────────────────────────────────────
    #[test]
    fn sphere_volume_unit_radius() {
        let expected = (4.0 / 3.0) * PI;
        assert!((sphere_volume(1.0) - expected).abs() < EPS, "got {}", sphere_volume(1.0));
    }
    #[test]
    fn sphere_volume_zero_is_zero() {
        assert_eq!(sphere_volume(0.0), 0.0);
    }
    #[test]
    fn sphere_volume_double_radius_is_eight_times() {
        let v1 = sphere_volume(1.0);
        let v2 = sphere_volume(2.0);
        assert!((v2 / v1 - 8.0).abs() < EPS);
    }

    // ── projected_circle_area ───────────────────────────────────────────────
    #[test]
    fn projected_circle_area_unit_radius() {
        assert!((projected_circle_area(1.0) - PI).abs() < EPS);
    }
    #[test]
    fn projected_circle_area_zero_is_zero() {
        assert_eq!(projected_circle_area(0.0), 0.0);
    }

    // ── sphere_surface_area ─────────────────────────────────────────────────
    #[test]
    fn sphere_surface_area_unit_radius() {
        assert!((sphere_surface_area(1.0) - 4.0 * PI).abs() < EPS);
    }
    #[test]
    fn sphere_surface_area_zero_is_zero() {
        assert_eq!(sphere_surface_area(0.0), 0.0);
    }

    // ── density ─────────────────────────────────────────────────────────────
    #[test]
    fn density_known_value() {
        let rho = density(100.0, 1.0);
        let expected = 100.0 / sphere_volume(1.0);
        assert!((rho - expected).abs() < EPS, "got {rho}");
    }
    #[test]
    fn density_zero_qe_is_zero() {
        assert_eq!(density(0.0, 1.0), 0.0);
    }
    #[test]
    fn density_zero_radius_returns_max() {
        assert_eq!(density(100.0, 0.0), f32::MAX);
    }

    // ── interference ────────────────────────────────────────────────────────
    #[test]
    fn interference_same_freq_same_phase_is_one() {
        let i = interference(440.0, 0.0, 440.0, 0.0, 0.0);
        assert!((i - 1.0).abs() < EPS, "got {i}");
    }
    #[test]
    fn interference_same_freq_opposite_phase() {
        let i = interference(440.0, 0.0, 440.0, PI, 0.0);
        assert!((i - (-1.0)).abs() < EPS, "got {i}");
    }
    #[test]
    fn interference_bounded() {
        for t in [0.0, 0.5, 1.0, 2.0, 10.0] {
            let i = interference(440.0, 0.3, 660.0, 1.7, t);
            assert!((-1.0..=1.0).contains(&i), "out of range: {i} at t={t}");
        }
    }

    // ── is_constructive / is_destructive / is_critical ──────────────────────
    #[test]
    fn is_constructive_above_threshold() {
        assert!(is_constructive(CONSTRUCTIVE_THRESHOLD + 0.01));
    }
    #[test]
    fn is_constructive_below_threshold() {
        assert!(!is_constructive(CONSTRUCTIVE_THRESHOLD - 0.01));
    }
    #[test]
    fn is_destructive_below_threshold() {
        assert!(is_destructive(DESTRUCTIVE_THRESHOLD - 0.01));
    }
    #[test]
    fn is_destructive_above_threshold() {
        assert!(!is_destructive(DESTRUCTIVE_THRESHOLD + 0.01));
    }
    #[test]
    fn is_critical_above_threshold() {
        assert!(is_critical(CRITICAL_THRESHOLD + 0.01));
    }
    #[test]
    fn is_critical_negative_above_threshold() {
        assert!(is_critical(-(CRITICAL_THRESHOLD + 0.01)));
    }
    #[test]
    fn is_critical_below_threshold() {
        assert!(!is_critical(CRITICAL_THRESHOLD - 0.01));
    }

    // ── effective_dissipation ────────────────────────────────────────────────
    #[test]
    fn effective_dissipation_zero_velocity_equals_base() {
        assert_eq!(effective_dissipation(0.05, Vec2::ZERO, 0.1), 0.05);
    }
    #[test]
    fn effective_dissipation_increases_with_speed() {
        let slow = effective_dissipation(0.05, Vec2::new(1.0, 0.0), 0.1);
        let fast = effective_dissipation(0.05, Vec2::new(3.0, 0.0), 0.1);
        assert!(fast > slow, "fast={fast} should > slow={slow}");
    }
    #[test]
    fn effective_dissipation_non_negative() {
        let d = effective_dissipation(0.0, Vec2::ZERO, 0.0);
        assert!(d >= 0.0);
    }

    // ── drag_force ──────────────────────────────────────────────────────────
    #[test]
    fn drag_zero_velocity_is_zero() {
        assert_eq!(drag_force(1.0, 10.0, Vec2::ZERO), Vec2::ZERO);
    }
    #[test]
    fn drag_opposes_velocity() {
        let v = Vec2::new(3.0, 4.0);
        let d = drag_force(1.0, 10.0, v);
        assert!(d.dot(v) < 0.0, "drag should oppose velocity: drag={d}, vel={v}");
    }
    #[test]
    fn drag_scales_with_viscosity() {
        let v = Vec2::new(2.0, 0.0);
        let d1 = drag_force(1.0, 10.0, v).length();
        let d2 = drag_force(2.0, 10.0, v).length();
        assert!((d2 / d1 - 2.0).abs() < EPS, "d1={d1}, d2={d2}");
    }

    // ── integrate_velocity ──────────────────────────────────────────────────
    #[test]
    fn integrate_velocity_zero_force_unchanged() {
        let v = Vec2::new(5.0, 3.0);
        assert_eq!(integrate_velocity(v, Vec2::ZERO, 100.0, 0.016), v);
    }
    #[test]
    fn integrate_velocity_positive_force_accelerates() {
        let v = Vec2::ZERO;
        let result = integrate_velocity(v, Vec2::new(100.0, 0.0), 10.0, 1.0);
        assert!(result.x > 0.0, "should accelerate: {result}");
    }
    #[test]
    fn integrate_velocity_zero_qe_unchanged() {
        let v = Vec2::new(1.0, 0.0);
        assert_eq!(integrate_velocity(v, Vec2::new(99.0, 0.0), 0.0, 1.0), v);
    }
    #[test]
    fn integrate_velocity_negative_qe_unchanged() {
        let v = Vec2::new(1.0, 0.0);
        assert_eq!(integrate_velocity(v, Vec2::new(99.0, 0.0), -5.0, 1.0), v);
    }

    // ── equivalent_temperature ──────────────────────────────────────────────
    #[test]
    fn equivalent_temperature_positive_density() {
        let t = equivalent_temperature(100.0);
        assert!(t > 0.0 && t.is_finite());
    }
    #[test]
    fn equivalent_temperature_zero_is_zero() {
        assert_eq!(equivalent_temperature(0.0), 0.0);
    }
    #[test]
    fn equivalent_temperature_formula() {
        let rho = 50.0;
        assert!((equivalent_temperature(rho) - rho / GAME_BOLTZMANN).abs() < EPS);
    }

    // ── state_from_temperature ──────────────────────────────────────────────
    #[test]
    fn state_solid_below_threshold() {
        let t = SOLID_TRANSITION * 10.0 - 0.01;
        assert_eq!(state_from_temperature(t, 10.0), MatterState::Solid);
    }
    #[test]
    fn state_liquid_in_range() {
        let t = (SOLID_TRANSITION * 10.0 + LIQUID_TRANSITION * 10.0) / 2.0;
        assert_eq!(state_from_temperature(t, 10.0), MatterState::Liquid);
    }
    #[test]
    fn state_gas_in_range() {
        let t = (LIQUID_TRANSITION * 10.0 + GAS_TRANSITION * 10.0) / 2.0;
        assert_eq!(state_from_temperature(t, 10.0), MatterState::Gas);
    }
    #[test]
    fn state_plasma_above_threshold() {
        let t = GAS_TRANSITION * 10.0 + 0.01;
        assert_eq!(state_from_temperature(t, 10.0), MatterState::Plasma);
    }
    #[test]
    fn state_boundary_solid_liquid_exact() {
        let t = SOLID_TRANSITION * 10.0;
        assert_eq!(state_from_temperature(t, 10.0), MatterState::Liquid);
    }
    #[test]
    fn state_boundary_liquid_gas_exact() {
        let t = LIQUID_TRANSITION * 10.0;
        assert_eq!(state_from_temperature(t, 10.0), MatterState::Gas);
    }
    #[test]
    fn state_boundary_gas_plasma_exact() {
        let t = GAS_TRANSITION * 10.0;
        assert_eq!(state_from_temperature(t, 10.0), MatterState::Plasma);
    }
}
