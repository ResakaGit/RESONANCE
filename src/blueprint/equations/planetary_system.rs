//! Sistema planetario — Titius-Bode, equilibrio térmico, zona habitable.
//! Planetary system — Titius-Bode, thermal equilibrium, habitable zone.
//!
//! Pure math. CT-5 / ADR-036 §D4 (S1→S2).
//!
//! **Titius-Bode:** radios orbitales en progresión geométrica emergente.
//! **Equilibrio térmico:** Stefan-Boltzmann `T^4 ∝ L_star / r²` → `T ∝ (L/r²)^0.25`.
//! **MatterState:** derivado de T comparado con tasas de dissipation.

use crate::blueprint::equations::derived_thresholds::{
    DISSIPATION_GAS, DISSIPATION_LIQUID, DISSIPATION_SOLID,
};
use crate::blueprint::domain_enums::MatterState;

// ─── Titius-Bode radii ──────────────────────────────────────────────────────

/// Razón geométrica canónica de Titius-Bode modificado: `r_n = a0 · ratio^n`.
/// Canonical Titius-Bode-like ratio.
pub const TITIUS_BODE_RATIO: f64 = 1.7;

/// Genera `n` radios orbitales en progresión geométrica a partir de `r_inner`.
/// Generates `n` orbital radii in geometric progression from `r_inner`.
pub fn titius_bode_radii(n: usize, r_inner: f64) -> Vec<f64> {
    (0..n).map(|i| r_inner * TITIUS_BODE_RATIO.powi(i as i32)).collect()
}

// ─── Thermal equilibrium ────────────────────────────────────────────────────

/// Factor de escala calibrado para que la zona habitable caiga en radios típicos
/// (Titius-Bode a partir de radio estelar). Deriva del acoplamiento `dissipation·DENSITY_SCALE`
/// — mantiene T adimensional en el mismo orden que las tasas de dissipation.
/// Scale factor so habitable zone aligns with typical orbital radii.
pub const PLANET_TEMPERATURE_SCALE: f64 = 0.01;

/// Temperatura de equilibrio normalizada: `T = SCALE · (star_qe / r²)^0.25`.
/// Equilibrium temperature (normalized): `T = SCALE · (star_qe / r²)^0.25`.
#[inline]
pub fn planet_temperature(star_qe: f64, orbital_radius: f64) -> f64 {
    if star_qe <= 0.0 || orbital_radius <= 0.0 { return 0.0; }
    PLANET_TEMPERATURE_SCALE * (star_qe / (orbital_radius * orbital_radius)).powf(0.25)
}

/// Clasifica un planeta según temperatura comparada con tasas de dissipation.
/// Classifies a planet by temperature vs dissipation rates.
///
/// Umbrales: Solid < DISSIPATION_SOLID; Liquid < DISSIPATION_LIQUID; Gas < DISSIPATION_GAS; Plasma ≥ DISSIPATION_GAS.
pub fn matter_state_from_temperature(temperature: f64) -> MatterState {
    let s = DISSIPATION_SOLID as f64;
    let l = DISSIPATION_LIQUID as f64;
    let g = DISSIPATION_GAS as f64;
    if temperature < s { MatterState::Solid }
    else if temperature < l { MatterState::Liquid }
    else if temperature < g { MatterState::Gas }
    else { MatterState::Plasma }
}

// ─── Habitable zone ─────────────────────────────────────────────────────────

/// Rango orbital donde T ∈ [T_solid, T_liquid] → agua líquida posible.
/// Orbital range where T ∈ [T_solid, T_liquid] → liquid water possible.
///
/// Retorna `(r_outer, r_inner)` porque `r_outer` (T_solid lower bound)
/// corresponde a órbita más lejana y `r_inner` a órbita más cercana.
/// Valores en unidades normalizadas consistentes con `planet_temperature`.
pub fn habitable_zone_bounds(star_qe: f64) -> (f64, f64) {
    if star_qe <= 0.0 { return (0.0, 0.0); }
    // Resolver T = SCALE · (star_qe/r²)^0.25 para r → r = sqrt(star_qe / (T/SCALE)^4).
    let t_to_r2 = |t: f64| star_qe / (t / PLANET_TEMPERATURE_SCALE).powi(4);
    let r_outer = t_to_r2(DISSIPATION_SOLID as f64).sqrt();
    let r_inner = t_to_r2(DISSIPATION_LIQUID as f64).sqrt();
    (r_outer, r_inner)
}

/// `true` si el planeta está en zona líquida.
/// Returns `true` if the planet sits in the liquid zone.
#[inline]
pub fn is_habitable(temperature: f64) -> bool {
    matches!(matter_state_from_temperature(temperature), MatterState::Liquid)
}

// ─── Tests ──────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::blueprint::equations::derived_thresholds::DISSIPATION_PLASMA;

    #[test]
    fn titius_bode_empty_for_zero_planets() {
        assert!(titius_bode_radii(0, 1.0).is_empty());
    }

    #[test]
    fn titius_bode_geometric_sequence() {
        let radii = titius_bode_radii(5, 1.0);
        for i in 0..4 {
            let ratio = radii[i + 1] / radii[i];
            assert!((ratio - TITIUS_BODE_RATIO).abs() < 1e-9);
        }
    }

    #[test]
    fn titius_bode_monotone_increasing() {
        let radii = titius_bode_radii(7, 0.5);
        for i in 0..6 { assert!(radii[i + 1] > radii[i]); }
    }

    #[test]
    fn temperature_decreases_with_distance() {
        let t_near = planet_temperature(1000.0, 1.0);
        let t_mid = planet_temperature(1000.0, 5.0);
        let t_far = planet_temperature(1000.0, 20.0);
        assert!(t_near > t_mid);
        assert!(t_mid > t_far);
    }

    #[test]
    fn temperature_scales_as_star_qe_quarter() {
        let t1 = planet_temperature(1.0, 1.0);
        let t16 = planet_temperature(16.0, 1.0);
        // 16^0.25 = 2
        assert!((t16 / t1 - 2.0).abs() < 1e-9);
    }

    #[test]
    fn temperature_zero_input_guards() {
        assert_eq!(planet_temperature(0.0, 1.0), 0.0);
        assert_eq!(planet_temperature(1.0, 0.0), 0.0);
    }

    #[test]
    fn matter_state_exhaustive_boundaries() {
        assert_eq!(matter_state_from_temperature(0.0), MatterState::Solid);
        assert_eq!(matter_state_from_temperature(DISSIPATION_SOLID as f64), MatterState::Liquid);
        assert_eq!(matter_state_from_temperature(DISSIPATION_LIQUID as f64), MatterState::Gas);
        assert_eq!(matter_state_from_temperature(DISSIPATION_GAS as f64), MatterState::Plasma);
        assert_eq!(matter_state_from_temperature(DISSIPATION_PLASMA as f64), MatterState::Plasma);
    }

    #[test]
    fn habitable_zone_bounded_by_temperature_band() {
        let (outer, inner) = habitable_zone_bounds(1000.0);
        assert!(inner > 0.0 && outer > inner);
        // At r=inner the temperature should equal T_LIQUID boundary.
        let t_inner = planet_temperature(1000.0, inner);
        assert!((t_inner - DISSIPATION_LIQUID as f64).abs() < 1e-9);
        // At r=outer the temperature should equal T_SOLID boundary.
        let t_outer = planet_temperature(1000.0, outer);
        assert!((t_outer - DISSIPATION_SOLID as f64).abs() < 1e-9);
    }

    #[test]
    fn is_habitable_true_inside_liquid_band() {
        let t = (DISSIPATION_SOLID as f64 + DISSIPATION_LIQUID as f64) * 0.5;
        assert!(is_habitable(t));
    }

    #[test]
    fn is_habitable_false_outside_liquid_band() {
        assert!(!is_habitable(0.0));
        assert!(!is_habitable(DISSIPATION_PLASMA as f64));
    }

    #[test]
    fn habitable_zone_zero_star_empty() {
        assert_eq!(habitable_zone_bounds(0.0), (0.0, 0.0));
    }
}
