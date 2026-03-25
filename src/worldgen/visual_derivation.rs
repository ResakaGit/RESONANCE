use bevy::color::Color;

use crate::blueprint::almanac::AlchemicalAlmanac;
use crate::blueprint::constants::FIELD_VISUAL_OPAQUE_ALPHA;
use crate::blueprint::equations;
use crate::eco::contracts::{TransitionType, ZoneClass};
use crate::layers::{MatterState, SpatialVolume};
use crate::worldgen::archetypes::WorldArchetype;
use crate::worldgen::constants::{
    MATERIALIZED_COLLIDER_RADIUS_FACTOR, MATERIALIZED_MIN_COLLIDER_RADIUS, REFERENCE_DENSITY,
    VISUAL_EMISSION_GAS_SCALE, VISUAL_EMISSION_GAS_TEMP_DIVISOR, VISUAL_EMISSION_PLASMA_OFFSET,
    VISUAL_EMISSION_PLASMA_TEMP_DIVISOR, VISUAL_MIN_SCALE,
    VISUAL_OPACITY_GAS_BASE, VISUAL_OPACITY_GAS_RANGE, VISUAL_OPACITY_LIQUID_BASE,
    VISUAL_OPACITY_LIQUID_RANGE, VISUAL_OPACITY_PLASMA_BASE, VISUAL_OPACITY_PLASMA_RANGE,
    VISUAL_SCALE_GAS_BASE, VISUAL_SCALE_GAS_RANGE, VISUAL_SCALE_LIQUID, VISUAL_SCALE_PLASMA_BASE,
    VISUAL_SCALE_PLASMA_RANGE, VISUAL_SCALE_SOLID_BASE, VISUAL_SCALE_SOLID_RANGE,
};
use crate::worldgen::contracts::BoundaryVisual;
use crate::worldgen::contracts::{FrequencyContribution, top_two};

/// Densidad espacial del volumen de colisión estándar de un tile materializado (mismo radio que `materialization_delta_system`).
#[inline]
pub fn materialized_tile_spatial_density(qe: f32, cell_size_m: f32) -> f32 {
    let radius =
        (cell_size_m * MATERIALIZED_COLLIDER_RADIUS_FACTOR).max(MATERIALIZED_MIN_COLLIDER_RADIUS);
    SpatialVolume::new(radius).density(qe.max(0.0))
}

/// Proxy de temperatura visual: ρ / `bond_energy_eb` (coherente con `derive_emission` / spawn).
#[inline]
pub fn visual_proxy_temperature(density: f32, bond_energy_eb: f32) -> f32 {
    if bond_energy_eb > 0.0 && density.is_finite() {
        (density / bond_energy_eb).max(0.0)
    } else {
        0.0
    }
}

/// RGB lineal del gris neutro de derivación visual (canónico: `equations::neutral_field_visual_linear_rgb`).
#[inline]
pub fn neutral_visual_linear_rgb() -> [f32; 3] {
    equations::neutral_field_visual_linear_rgb()
}

/// Salida consolidada de derivación visual stateless.
///
/// Referencia canónica rápida (spec visual):
/// - Ignis puro (450Hz, pureza 1): color ~ (1.0, 0.3, 0.0)
/// - Pureza 0: color gris neutro
/// - Solid: opacidad 1.0
/// - Plasma/Gas caliente: emisión > 0
#[derive(Clone, Debug)]
pub struct VisualProperties {
    pub color: Color,
    pub scale: f32,
    pub emission: f32,
    pub opacity: f32,
}

/// Interpolación lineal entre dos colores en espacio lineal.
pub fn color_lerp(a: Color, b: Color, t: f32) -> Color {
    let t = sanitize_unit(t, 0.0);
    let a_lin = a.to_linear();
    let b_lin = b.to_linear();
    Color::linear_rgba(
        lerp(a_lin.red, b_lin.red, t),
        lerp(a_lin.green, b_lin.green, t),
        lerp(a_lin.blue, b_lin.blue, t),
        lerp(a_lin.alpha, b_lin.alpha, t),
    )
}

/// Mezcla `young` → `mature` según `phase` ∈ [0, 1] en espacio lineal (EA8).
/// RGB comparte núcleo con blueprint: [`equations::field_visual_mix_unit`] + [`equations::linear_rgb_lerp_preclamped`]
/// (misma semántica que [`equations::linear_rgb_lerp`] en arrays lineales).
#[inline]
pub fn derive_color_phenology(young: Color, mature: Color, phase: f32) -> Color {
    let t = equations::field_visual_mix_unit(phase);
    let a_lin = young.to_linear();
    let b_lin = mature.to_linear();
    let rgb = equations::linear_rgb_lerp_preclamped(
        [a_lin.red, a_lin.green, a_lin.blue],
        [b_lin.red, b_lin.green, b_lin.blue],
        t,
    );
    Color::linear_rgba(
        rgb[0],
        rgb[1],
        rgb[2],
        lerp(a_lin.alpha, b_lin.alpha, t),
    )
}

/// Deriva color desde frecuencia + pureza usando el Almanac como fuente de verdad.
/// Núcleo numérico: [`equations::field_linear_rgb_from_hz_purity`].
pub fn derive_color(frequency_hz: f32, purity: f32, almanac: &AlchemicalAlmanac) -> Color {
    let rgb = equations::field_linear_rgb_from_hz_purity(frequency_hz, purity, almanac);
    Color::linear_rgba(
        rgb[0],
        rgb[1],
        rgb[2],
        FIELD_VISUAL_OPAQUE_ALPHA,
    )
}

/// Mezcla de color para compuestos:
/// - Constructiva: sesga al primario.
/// - Destructiva: desatura hacia gris neutro.
/// Núcleo numérico: [`equations::compound_field_linear_rgba`].
pub fn compound_color_blend(
    primary_color: Color,
    secondary_color: Color,
    interference: f32,
    purity: f32,
) -> Color {
    let p_lin = primary_color.to_linear();
    let s_lin = secondary_color.to_linear();
    let out = equations::compound_field_linear_rgba(
        [p_lin.red, p_lin.green, p_lin.blue, p_lin.alpha],
        [s_lin.red, s_lin.green, s_lin.blue, s_lin.alpha],
        interference,
        purity,
    );
    Color::linear_rgba(out[0], out[1], out[2], out[3])
}

pub fn derive_color_compound(
    contributions: &[FrequencyContribution],
    purity: f32,
    t: f32,
    almanac: &AlchemicalAlmanac,
) -> Option<Color> {
    let (primary, secondary) = top_two(contributions)?;
    let primary_color = derive_color(primary.frequency_hz(), 1.0, almanac);
    let secondary_color = derive_color(secondary.frequency_hz(), 1.0, almanac);
    let interference = equations::interference(
        primary.frequency_hz(),
        0.0,
        secondary.frequency_hz(),
        0.0,
        t,
    );
    Some(compound_color_blend(
        primary_color,
        secondary_color,
        interference,
        purity,
    ))
}

/// Deriva escala visual desde densidad y estado de materia.
pub fn derive_scale(density: f32, state: MatterState) -> f32 {
    let norm = normalize_density(density);
    let scale = match state {
        MatterState::Solid => VISUAL_SCALE_SOLID_BASE + VISUAL_SCALE_SOLID_RANGE * norm,
        MatterState::Liquid => VISUAL_SCALE_LIQUID,
        MatterState::Gas => VISUAL_SCALE_GAS_BASE - VISUAL_SCALE_GAS_RANGE * norm,
        MatterState::Plasma => VISUAL_SCALE_PLASMA_BASE + VISUAL_SCALE_PLASMA_RANGE * norm,
    };
    sanitize_positive(scale, 1.0)
}

/// Deriva emisión visual desde temperatura y estado.
pub fn derive_emission(temperature: f32, state: MatterState) -> f32 {
    let temp = sanitize_non_negative(temperature, 0.0);
    let emission = match state {
        MatterState::Plasma => {
            VISUAL_EMISSION_PLASMA_OFFSET + temp / (temp + VISUAL_EMISSION_PLASMA_TEMP_DIVISOR)
        }
        MatterState::Gas if temp > 0.0 => {
            VISUAL_EMISSION_GAS_SCALE * (temp / (temp + VISUAL_EMISSION_GAS_TEMP_DIVISOR))
        }
        _ => 0.0,
    };
    sanitize_unit(emission, 0.0)
}

/// Deriva opacidad visual desde densidad y estado.
pub fn derive_opacity(density: f32, state: MatterState) -> f32 {
    let norm = normalize_density(density);
    let opacity = match state {
        MatterState::Solid => 1.0,
        MatterState::Liquid => VISUAL_OPACITY_LIQUID_BASE + VISUAL_OPACITY_LIQUID_RANGE * norm,
        MatterState::Gas => VISUAL_OPACITY_GAS_BASE + VISUAL_OPACITY_GAS_RANGE * norm,
        MatterState::Plasma => VISUAL_OPACITY_PLASMA_BASE + VISUAL_OPACITY_PLASMA_RANGE * norm,
    };
    sanitize_unit(opacity, 1.0)
}

/// Ajustes visuales específicos para variantes topográficas de materialización (T7).
pub fn apply_archetype_visual_profile(
    archetype: WorldArchetype,
    color: Color,
    scale: f32,
    emission: f32,
    opacity: f32,
) -> (Color, f32, f32, f32) {
    match archetype {
        WorldArchetype::River => (Color::srgb(0.10, 0.35, 0.82), scale * 0.92, emission, 0.88),
        WorldArchetype::Lake => (Color::srgb(0.08, 0.22, 0.58), scale * 1.05, emission, 0.95),
        WorldArchetype::GlacierPeak => (
            Color::srgb(0.80, 0.90, 0.98),
            scale * 1.08,
            emission * 0.35,
            0.98,
        ),
        WorldArchetype::LavaRiver => (
            Color::srgb(0.98, 0.38, 0.06),
            scale * 1.03,
            (emission + 0.30).clamp(0.0, 1.0),
            0.96,
        ),
        WorldArchetype::VolcanicVent => (
            Color::srgb(0.92, 0.28, 0.14),
            scale * 1.10,
            (emission + 0.35).clamp(0.0, 1.0),
            0.90,
        ),
        WorldArchetype::MistValley => (
            Color::srgb(0.58, 0.62, 0.74),
            scale * 1.12,
            (emission + 0.08).clamp(0.0, 1.0),
            0.62,
        ),
        WorldArchetype::Rockface => (Color::srgb(0.46, 0.42, 0.38), scale * 1.15, emission, 1.0),
        WorldArchetype::WindsweptPlateau => (
            Color::srgb(0.74, 0.84, 0.90),
            scale * 1.06,
            (emission + 0.06).clamp(0.0, 1.0),
            0.86,
        ),
        WorldArchetype::Hillside => (Color::srgb(0.54, 0.44, 0.28), scale * 1.04, emission, 0.96),
        WorldArchetype::Ravine => (Color::srgb(0.26, 0.22, 0.18), scale * 0.98, emission, 0.97),
        _ => (color, scale, emission, opacity),
    }
}

/// Wrapper stateless para derivar todas las propiedades visuales.
pub fn derive_all(
    frequency_hz: f32,
    purity: f32,
    density: f32,
    temperature: f32,
    state: MatterState,
    almanac: &AlchemicalAlmanac,
) -> VisualProperties {
    VisualProperties {
        color: derive_color(frequency_hz, purity, almanac),
        scale: derive_scale(density, state),
        emission: derive_emission(temperature, state),
        opacity: derive_opacity(density, state),
    }
}

/// Color representativo de `ZoneClass` para lerp en fronteras (Eco-Boundaries E6).
/// Color plano de transición entre `zone_a` y `zone_b` (única fuente para tinte de frontera en V7).
pub fn energy_visual_boundary_flat_color(bv: &BoundaryVisual) -> Color {
    let t = if bv.gradient_factor.is_finite() {
        bv.gradient_factor.clamp(0.0, 1.0)
    } else {
        0.5
    };
    color_lerp(
        zone_class_display_color(bv.zone_a),
        zone_class_display_color(bv.zone_b),
        t,
    )
}

pub fn zone_class_display_color(z: ZoneClass) -> Color {
    match z {
        ZoneClass::HighAtmosphere => Color::srgb(0.75, 0.85, 0.95),
        ZoneClass::Surface => Color::srgb(0.40, 0.55, 0.35),
        ZoneClass::Subaquatic => Color::srgb(0.12, 0.28, 0.62),
        ZoneClass::Subterranean => Color::srgb(0.28, 0.20, 0.14),
        ZoneClass::Volcanic => Color::srgb(0.95, 0.35, 0.08),
        ZoneClass::Frozen => Color::srgb(0.70, 0.82, 0.92),
        ZoneClass::Void => Color::srgb(0.10, 0.05, 0.18),
    }
}

/// Extra de emisión en fronteras dramáticas (proxy de partículas; sin sistema de VFX).
pub fn boundary_transition_emission_extra(transition: TransitionType) -> f32 {
    match transition {
        TransitionType::PhaseBoundary => 0.22,
        TransitionType::ThermalShock => 0.30,
        _ => 0.0,
    }
}

fn normalize_density(density: f32) -> f32 {
    let density = sanitize_non_negative(density, 0.0);
    let ref_density = sanitize_positive(REFERENCE_DENSITY, 1.0);
    (density / ref_density).clamp(0.0, 1.0)
}

fn sanitize_non_negative(value: f32, fallback: f32) -> f32 {
    if value.is_finite() {
        value.max(0.0)
    } else {
        fallback
    }
}

fn sanitize_positive(value: f32, fallback: f32) -> f32 {
    let base = sanitize_non_negative(value, fallback);
    base.max(VISUAL_MIN_SCALE)
}

fn sanitize_unit(value: f32, fallback: f32) -> f32 {
    if value.is_finite() {
        value.clamp(0.0, 1.0)
    } else {
        fallback
    }
}

fn lerp(a: f32, b: f32, t: f32) -> f32 {
    a + (b - a) * t
}

#[cfg(test)]
mod tests {
    use super::{
        color_lerp, compound_color_blend, derive_all, derive_color, derive_color_compound,
        derive_emission, derive_opacity, derive_scale, materialized_tile_spatial_density,
        sanitize_unit, visual_proxy_temperature,
    };
    use crate::blueprint::almanac::{AlchemicalAlmanac, ElementDef};
    use crate::layers::MatterState;
    use crate::worldgen::contracts::FrequencyContribution;
    use bevy::prelude::{Color, Entity};

    fn approx(a: f32, b: f32) {
        assert!((a - b).abs() < 1e-3, "left={a}, right={b}");
    }

    fn rgba(color: bevy::prelude::Color) -> bevy::color::Srgba {
        color.to_srgba()
    }

    fn mk_almanac_with_ignis() -> AlchemicalAlmanac {
        let ignis = ElementDef {
            name: "Ignis".to_string(),
            symbol: "Ignis".to_string(),
            atomic_number: 8,
            frequency_hz: 450.0,
            freq_band: (400.0, 500.0),
            bond_energy: 1000.0,
            conductivity: 0.5,
            visibility: 0.8,
            matter_state: MatterState::Plasma,
            electronegativity: 0.0,
            ionization_ev: 0.0,
            color: (1.0, 0.3, 0.0),
            is_compound: false,
            phenology: None,
            hz_identity_weight: crate::blueprint::FIELD_EAC4_HZ_IDENTITY_WEIGHT_RON_ONLY,
        };
        AlchemicalAlmanac::from_defs(vec![ignis])
    }

    #[test]
    fn derive_color_ignis_purity_one_matches_almanac_color() {
        let almanac = mk_almanac_with_ignis();
        let color = derive_color(450.0, 1.0, &almanac);
        let rgba = rgba(color);
        approx(rgba.red, 1.0);
        approx(rgba.green, 0.3);
        approx(rgba.blue, 0.0);
    }

    #[test]
    fn derive_color_purity_zero_returns_neutral_gray() {
        let almanac = mk_almanac_with_ignis();
        let color = derive_color(450.0, 0.0, &almanac);
        let rgba = rgba(color);
        approx(rgba.red, 0.5);
        approx(rgba.green, 0.5);
        approx(rgba.blue, 0.5);
    }

    #[test]
    fn derive_color_outside_any_band_returns_gray() {
        let almanac = mk_almanac_with_ignis();
        let color = derive_color(1500.0, 1.0, &almanac);
        let rgba = rgba(color);
        approx(rgba.red, 0.5);
        approx(rgba.green, 0.5);
        approx(rgba.blue, 0.5);
    }

    #[test]
    fn derive_scale_solid_high_density_is_greater_than_low_density() {
        let low = derive_scale(5.0, MatterState::Solid);
        let high = derive_scale(200.0, MatterState::Solid);
        assert!(high > low);
    }

    #[test]
    fn derive_scale_gas_high_density_is_lower_than_low_density() {
        let low = derive_scale(5.0, MatterState::Gas);
        let high = derive_scale(200.0, MatterState::Gas);
        assert!(high < low);
    }

    #[test]
    fn derive_emission_plasma_is_positive() {
        assert!(derive_emission(120.0, MatterState::Plasma) > 0.0);
    }

    #[test]
    fn derive_emission_solid_is_zero() {
        approx(derive_emission(999.0, MatterState::Solid), 0.0);
    }

    #[test]
    fn derive_opacity_solid_is_one() {
        approx(derive_opacity(50.0, MatterState::Solid), 1.0);
    }

    #[test]
    fn derive_opacity_gas_is_less_than_one() {
        assert!(derive_opacity(50.0, MatterState::Gas) < 1.0);
    }

    #[test]
    fn derive_all_with_invalid_inputs_clamps_ranges() {
        let almanac = mk_almanac_with_ignis();
        let visual = derive_all(
            f32::NAN,
            f32::NEG_INFINITY,
            f32::INFINITY,
            f32::NAN,
            MatterState::Plasma,
            &almanac,
        );

        assert!(visual.scale > 0.0);
        assert!((0.0..=1.0).contains(&visual.emission));
        assert!((0.0..=1.0).contains(&visual.opacity));
        let rgba = rgba(visual.color);
        assert!((0.0..=1.0).contains(&rgba.red));
        assert!((0.0..=1.0).contains(&rgba.green));
        assert!((0.0..=1.0).contains(&rgba.blue));
    }

    #[test]
    fn sanitize_unit_nan_uses_fallback() {
        approx(sanitize_unit(f32::NAN, 0.25), 0.25);
    }

    #[test]
    fn derive_color_phenology_phase_zero_matches_young() {
        let young = Color::srgb(1.0, 0.0, 0.0);
        let mature = Color::srgb(0.0, 0.0, 1.0);
        let out = super::derive_color_phenology(young, mature, 0.0).to_srgba();
        approx(out.red, 1.0);
        approx(out.green, 0.0);
        approx(out.blue, 0.0);
    }

    #[test]
    fn derive_color_phenology_phase_one_matches_mature() {
        let young = Color::srgb(1.0, 0.0, 0.0);
        let mature = Color::srgb(0.0, 0.0, 1.0);
        let out = super::derive_color_phenology(young, mature, 1.0).to_srgba();
        approx(out.red, 0.0);
        approx(out.green, 0.0);
        approx(out.blue, 1.0);
    }

    #[test]
    fn derive_color_phenology_midpoint_matches_color_lerp() {
        let young = Color::srgb(1.0, 0.0, 0.0);
        let mature = Color::srgb(0.0, 0.0, 1.0);
        let a = super::derive_color_phenology(young, mature, 0.5).to_srgba();
        let b = super::color_lerp(young, mature, 0.5).to_srgba();
        approx(a.red, b.red);
        approx(a.green, b.green);
        approx(a.blue, b.blue);
    }

    #[test]
    fn color_lerp_midpoint_is_finite_and_in_range() {
        let color = color_lerp(Color::srgb(1.0, 0.0, 0.0), Color::srgb(0.0, 0.0, 1.0), 0.5);
        let rgba = rgba(color);
        assert!(rgba.red.is_finite() && rgba.green.is_finite() && rgba.blue.is_finite());
        assert!((0.0..=1.0).contains(&rgba.red));
        assert!((0.0..=1.0).contains(&rgba.green));
        assert!((0.0..=1.0).contains(&rgba.blue));
    }

    #[test]
    fn derive_color_non_finite_frequency_returns_neutral_gray() {
        let almanac = mk_almanac_with_ignis();
        let color = derive_color(f32::NAN, 1.0, &almanac);
        let rgba = rgba(color);
        approx(rgba.red, 0.5);
        approx(rgba.green, 0.5);
        approx(rgba.blue, 0.5);
    }

    #[test]
    fn derive_scale_non_finite_density_stays_finite_positive() {
        for state in [
            MatterState::Solid,
            MatterState::Liquid,
            MatterState::Gas,
            MatterState::Plasma,
        ] {
            let value = derive_scale(f32::INFINITY, state);
            assert!(value.is_finite());
            assert!(value > 0.0);
        }
    }

    #[test]
    fn derive_emission_non_finite_temperature_stays_in_unit_interval() {
        for state in [
            MatterState::Solid,
            MatterState::Liquid,
            MatterState::Gas,
            MatterState::Plasma,
        ] {
            let value = derive_emission(f32::NAN, state);
            assert!((0.0..=1.0).contains(&value));
        }
    }

    #[test]
    fn derive_opacity_non_finite_density_stays_in_unit_interval() {
        for state in [
            MatterState::Solid,
            MatterState::Liquid,
            MatterState::Gas,
            MatterState::Plasma,
        ] {
            let value = derive_opacity(f32::NEG_INFINITY, state);
            assert!((0.0..=1.0).contains(&value));
        }
    }

    #[test]
    fn derive_all_matches_individual_derivations() {
        let almanac = mk_almanac_with_ignis();
        let frequency_hz = 450.0;
        let purity = 0.8;
        let density = 45.0;
        let temperature = 120.0;
        let state = MatterState::Plasma;

        let all = derive_all(frequency_hz, purity, density, temperature, state, &almanac);
        let color = derive_color(frequency_hz, purity, &almanac);
        let scale = derive_scale(density, state);
        let emission = derive_emission(temperature, state);
        let opacity = derive_opacity(density, state);

        let all_rgba = rgba(all.color);
        let color_rgba = rgba(color);
        approx(all_rgba.red, color_rgba.red);
        approx(all_rgba.green, color_rgba.green);
        approx(all_rgba.blue, color_rgba.blue);
        approx(all.scale, scale);
        approx(all.emission, emission);
        approx(all.opacity, opacity);
    }

    #[test]
    fn compound_color_blend_constructive_biases_primary() {
        let primary = Color::srgb(1.0, 0.2, 0.0);
        let secondary = Color::srgb(0.0, 0.0, 1.0);
        let blended = compound_color_blend(primary, secondary, 0.9, 0.2).to_srgba();
        assert!(blended.red > blended.blue);
    }

    #[test]
    fn compound_color_blend_destructive_moves_toward_neutral() {
        let primary = Color::srgb(1.0, 0.2, 0.0);
        let secondary = Color::srgb(0.0, 0.0, 1.0);
        let blended = compound_color_blend(primary, secondary, -1.0, 0.0).to_srgba();
        approx(blended.red, 0.5);
        approx(blended.green, 0.5);
        approx(blended.blue, 0.5);
    }

    #[test]
    fn derive_color_compound_uses_two_strongest_contributions() {
        let almanac = mk_almanac_with_ignis();
        let contributions = vec![
            FrequencyContribution::new(Entity::from_raw(1), 450.0, 30.0),
            FrequencyContribution::new(Entity::from_raw(2), 250.0, 20.0),
            FrequencyContribution::new(Entity::from_raw(3), 60.0, 1.0),
        ];
        let color = derive_color_compound(&contributions, 0.3, 0.1, &almanac);
        assert!(color.is_some());
    }

    #[test]
    fn materialized_tile_spatial_density_matches_collider_radius_formula() {
        use crate::layers::SpatialVolume;
        use crate::worldgen::constants::{
            MATERIALIZED_COLLIDER_RADIUS_FACTOR, MATERIALIZED_MIN_COLLIDER_RADIUS,
        };
        let cs = 2.0_f32;
        let qe = 80.0_f32;
        let r = (cs * MATERIALIZED_COLLIDER_RADIUS_FACTOR).max(MATERIALIZED_MIN_COLLIDER_RADIUS);
        approx(
            materialized_tile_spatial_density(qe, cs),
            SpatialVolume::new(r).density(qe),
        );
    }

    #[test]
    fn visual_proxy_temperature_ratio_when_bond_positive() {
        approx(visual_proxy_temperature(300.0, 1000.0), 0.3);
    }

    #[test]
    fn visual_proxy_temperature_zero_when_bond_non_positive() {
        approx(visual_proxy_temperature(500.0, 0.0), 0.0);
        approx(visual_proxy_temperature(100.0, -50.0), 0.0);
    }
}
