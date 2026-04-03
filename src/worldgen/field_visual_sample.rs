//! EPI2 — API **celda → RGB lineal** para inferencia de partes y consumidores futuros (GF1, shape).
//!
//! Orquestación celda/compuesto aquí; núcleo Hz/pureza/mecla en [`crate::blueprint::equations`].
//! `visual_derivation::derive_color` delega a ese núcleo. Proyección mundo→celda:
//! [`EnergyFieldGrid::world_pos`] / coords XZ (EA3).

use crate::math_types::{Vec2, Vec3};
use bevy::color::Color;

use crate::blueprint::almanac::AlchemicalAlmanac;
use crate::blueprint::constants::FIELD_VISUAL_OPAQUE_ALPHA;
use crate::blueprint::equations;
use crate::worldgen::EnergyFieldGrid;
use crate::worldgen::contracts::{EnergyCell, FrequencyContribution, top_two};
use crate::worldgen::materialization_rules::compound_path_active;

/// Orquestación EAC3/EPI2: `compound_path_active` viene de `materialization_rules`; el núcleo numérico es `blueprint::equations`.
#[inline]
pub fn field_linear_rgb_from_cell_inputs(
    dominant_frequency_hz: f32,
    purity: f32,
    contributions: &[FrequencyContribution],
    compound_path_active: bool,
    interference_t: f32,
    almanac: &AlchemicalAlmanac,
) -> [f32; 3] {
    let a = FIELD_VISUAL_OPAQUE_ALPHA;
    match (compound_path_active, top_two(contributions)) {
        (true, Some((p, s))) => {
            let p_rgb = equations::field_linear_rgb_from_hz_purity(p.frequency_hz(), 1.0, almanac);
            let s_rgb = equations::field_linear_rgb_from_hz_purity(s.frequency_hz(), 1.0, almanac);
            let interf = equations::interference(
                p.frequency_hz(),
                0.0,
                s.frequency_hz(),
                0.0,
                interference_t,
            );
            let rgba = equations::compound_field_linear_rgba(
                [p_rgb[0], p_rgb[1], p_rgb[2], a],
                [s_rgb[0], s_rgb[1], s_rgb[2], a],
                interf,
                purity,
            );
            [rgba[0], rgba[1], rgba[2]]
        }
        _ => equations::field_linear_rgb_from_hz_purity(dominant_frequency_hz, purity, almanac),
    }
}

/// Convierte un `Color` de derivación visual a RGB lineal; canales no finitos → gris neutro (misma función pura que el núcleo EAC3).
#[inline]
pub fn linear_rgb_from_derive_color(color: Color) -> [f32; 3] {
    let l = color.to_linear();
    equations::field_linear_rgb_sanitize_finite([l.red, l.green, l.blue])
}

/// Tinte lineal desde una celda ya derivada: ruta simple o compuesta según `compound_path_active`.
///
/// `purity_t` y `interference_t` son las mismas entradas que usa la derivación visual materializada
/// (pureza de mezcla; tiempo para interferencia en compuestos).
#[inline]
pub fn field_linear_rgb_from_cell(
    cell: &EnergyCell,
    purity_t: f32,
    interference_t: f32,
    almanac: &AlchemicalAlmanac,
) -> [f32; 3] {
    field_linear_rgb_from_cell_inputs(
        cell.dominant_frequency_hz,
        purity_t,
        cell.frequency_contributions(),
        compound_path_active(cell, almanac),
        interference_t,
        almanac,
    )
}

/// Muestreo EPI3 para GF1: mundo XZ → celda → RGB lineal (EPI2) + `qe_norm` local.
///
/// `interference_t` fijo (p. ej. `0.0` en inferencia de forma) evita acoplar `Update` al reloj global;
/// puede desalinear compuestos vs `visual.rs` si el materializado usa interferencia animada.
#[inline]
pub fn gf1_field_linear_rgb_qe_at_position(
    grid: &EnergyFieldGrid,
    world_position: Vec3,
    almanac: &AlchemicalAlmanac,
    qe_reference: f32,
    interference_t: f32,
    fallback_linear_rgb: [f32; 3],
    fallback_qe_norm: f32,
) -> ([f32; 3], f32) {
    let xz = Vec2::new(world_position.x, world_position.z);
    let qe_ref = qe_reference.max(1.0);
    let Some(cell) = grid.cell_at(xz) else {
        return (fallback_linear_rgb, fallback_qe_norm);
    };
    let rgb = field_linear_rgb_from_cell(cell, cell.purity, interference_t, almanac);
    let qn = (cell.accumulated_qe / qe_ref).clamp(0.0, 1.0);
    (rgb, qn)
}

#[cfg(test)]
mod tests {
    use super::field_linear_rgb_from_cell;
    use crate::blueprint::almanac::{AlchemicalAlmanac, ElementDef};
    use crate::blueprint::equations::linear_rgb_lerp;
    use crate::layers::MatterState;
    use crate::worldgen::EnergyFieldGrid;
    use crate::worldgen::constants::field_sample_test_thresholds as thr;
    use crate::worldgen::contracts::{EnergyCell, FrequencyContribution};
    use crate::worldgen::neutral_visual_linear_rgb;
    use bevy::math::{Vec2, Vec3};
    use bevy::prelude::{Color, Entity};

    fn approx3(a: [f32; 3], b: [f32; 3]) {
        for i in 0..3 {
            assert!(
                (a[i] - b[i]).abs() < thr::RGB_APPROX_EPS,
                "i={i} left={} right={}",
                a[i],
                b[i]
            );
        }
    }

    fn mk_ignis_almanac() -> AlchemicalAlmanac {
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

    fn mk_dual_band_almanac() -> AlchemicalAlmanac {
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
            color: (1.0, 0.2, 0.0),
            is_compound: false,
            phenology: None,
            hz_identity_weight: crate::blueprint::FIELD_EAC4_HZ_IDENTITY_WEIGHT_RON_ONLY,
        };
        let aqua = ElementDef {
            name: "Aqua".to_string(),
            symbol: "Aqua".to_string(),
            atomic_number: 5,
            frequency_hz: 250.0,
            freq_band: (200.0, 300.0),
            bond_energy: 800.0,
            conductivity: 0.6,
            visibility: 0.85,
            matter_state: MatterState::Liquid,
            electronegativity: 0.0,
            ionization_ev: 0.0,
            color: (0.1, 0.4, 0.95),
            is_compound: false,
            phenology: None,
            hz_identity_weight: crate::blueprint::FIELD_EAC4_HZ_IDENTITY_WEIGHT_RON_ONLY,
        };
        AlchemicalAlmanac::from_defs(vec![ignis, aqua])
    }

    #[test]
    fn gf1_field_sample_uses_fallback_outside_grid() {
        let almanac = mk_ignis_almanac();
        let grid = EnergyFieldGrid::new(2, 2, 1.0, Vec2::ZERO);
        let fb = [0.11f32, 0.22, 0.33];
        let fq = 0.44f32;
        let (rgb, qn) = super::gf1_field_linear_rgb_qe_at_position(
            &grid,
            Vec3::new(99.0, 0.0, 99.0),
            &almanac,
            100.0,
            0.0,
            fb,
            fq,
        );
        approx3(rgb, fb);
        assert!((qn - fq).abs() < thr::RGB_APPROX_EPS);
    }

    #[test]
    fn gf1_field_sample_differs_across_cells_on_xz() {
        let almanac = mk_ignis_almanac();
        let mut grid = EnergyFieldGrid::new(4, 4, 1.0, Vec2::ZERO);
        if let Some(c) = grid.cell_xy_mut(0, 0) {
            c.dominant_frequency_hz = 450.0;
            c.purity = 1.0;
            c.accumulated_qe = 100.0;
        }
        if let Some(c) = grid.cell_xy_mut(2, 0) {
            c.dominant_frequency_hz = 50.0;
            c.purity = 1.0;
            c.accumulated_qe = 10.0;
        }
        let fb = [0.05f32, 0.05, 0.05];
        let (rgb_a, q_a) = super::gf1_field_linear_rgb_qe_at_position(
            &grid,
            Vec3::new(0.5, 0.0, 0.5),
            &almanac,
            200.0,
            0.0,
            fb,
            0.25,
        );
        let (rgb_b, q_b) = super::gf1_field_linear_rgb_qe_at_position(
            &grid,
            Vec3::new(2.5, 0.0, 0.5),
            &almanac,
            200.0,
            0.0,
            fb,
            0.25,
        );
        let d =
            (rgb_a[0] - rgb_b[0]).abs() + (rgb_a[1] - rgb_b[1]).abs() + (rgb_a[2] - rgb_b[2]).abs();
        assert!(
            d > thr::MIN_BAND_L1,
            "expected distinct RGB per cell: a={rgb_a:?} b={rgb_b:?}"
        );
        assert!(
            q_a > q_b,
            "qe_norm debe seguir qe acumulado: {q_a} vs {q_b}"
        );
    }

    #[test]
    fn ignis_band_vs_outside_band_differs() {
        let almanac = mk_ignis_almanac();
        let mut in_band = EnergyCell::default();
        in_band.dominant_frequency_hz = 450.0;
        in_band.purity = 1.0;
        let mut out_band = EnergyCell::default();
        out_band.dominant_frequency_hz = 1500.0;
        out_band.purity = 1.0;
        let a = field_linear_rgb_from_cell(&in_band, 1.0, 0.0, &almanac);
        let b = field_linear_rgb_from_cell(&out_band, 1.0, 0.0, &almanac);
        let d = (a[0] - b[0]).abs() + (a[1] - b[1]).abs() + (a[2] - b[2]).abs();
        assert!(
            d > thr::MIN_BAND_L1,
            "expected distinct linear RGB: a={a:?} b={b:?}"
        );
    }

    #[test]
    fn compound_path_changes_rgb_vs_pure_dominant() {
        let almanac = mk_dual_band_almanac();
        let e1 = Entity::from_raw(1);
        let e2 = Entity::from_raw(2);
        let mut cell = EnergyCell::default();
        cell.purity = 0.5;
        cell.dominant_frequency_hz = 450.0;
        cell.frequency_contributions
            .push(FrequencyContribution::new(e1, 450.0, 2.0));
        cell.frequency_contributions
            .push(FrequencyContribution::new(e2, 250.0, 1.8));

        let compound_rgb = field_linear_rgb_from_cell(&cell, cell.purity, 0.0, &almanac);

        let mut pure = EnergyCell::default();
        pure.purity = 0.5;
        pure.dominant_frequency_hz = 450.0;
        pure.frequency_contributions
            .push(FrequencyContribution::new(e1, 450.0, 2.0));
        let simple_rgb = field_linear_rgb_from_cell(&pure, pure.purity, 0.0, &almanac);

        let d = (compound_rgb[0] - simple_rgb[0]).abs()
            + (compound_rgb[1] - simple_rgb[1]).abs()
            + (compound_rgb[2] - simple_rgb[2]).abs();
        assert!(
            d > thr::MIN_COMPOUND_L1,
            "compound blend should diverge from dominant-only: c={compound_rgb:?} s={simple_rgb:?}"
        );
    }

    #[test]
    fn non_finite_dominant_frequency_fallback_neutral_linear() {
        let almanac = mk_ignis_almanac();
        let mut cell = EnergyCell::default();
        cell.dominant_frequency_hz = f32::NAN;
        cell.purity = 1.0;
        let rgb = field_linear_rgb_from_cell(&cell, 1.0, 0.0, &almanac);
        approx3(rgb, neutral_visual_linear_rgb());
    }

    #[test]
    fn linear_rgb_lerp_matches_derive_color_phenology_in_linear_space() {
        let young = Color::srgb(0.9, 0.2, 0.15);
        let mature = Color::srgb(0.2, 0.85, 0.15);
        let phase = 0.42;
        let yl = young.to_linear();
        let ml = mature.to_linear();
        let from_lerp = linear_rgb_lerp(
            [yl.red, yl.green, yl.blue],
            [ml.red, ml.green, ml.blue],
            phase,
        );
        let from_color =
            crate::worldgen::visual_derivation::derive_color_phenology(young, mature, phase)
                .to_linear();
        approx3(
            from_lerp,
            [from_color.red, from_color.green, from_color.blue],
        );
    }

    #[test]
    fn compound_path_forced_true_with_lt_two_contribs_falls_back_to_dominant() {
        let almanac = mk_dual_band_almanac();
        let e1 = Entity::from_raw(1);
        let one = [FrequencyContribution::new(e1, 450.0, 2.0)];
        let fb = super::field_linear_rgb_from_cell_inputs(450.0, 0.6, &one, true, 0.0, &almanac);
        let simple =
            super::field_linear_rgb_from_cell_inputs(450.0, 0.6, &one, false, 0.0, &almanac);
        approx3(fb, simple);
    }

    #[test]
    fn derive_color_compound_matches_field_linear_rgb_from_cell_inputs() {
        use crate::worldgen::visual_derivation::derive_color_compound;

        let almanac = mk_dual_band_almanac();
        let e1 = Entity::from_raw(1);
        let e2 = Entity::from_raw(2);
        let contribs = vec![
            FrequencyContribution::new(e1, 450.0, 2.0),
            FrequencyContribution::new(e2, 250.0, 1.8),
        ];
        let t = 0.37_f32;
        let purity = 0.45_f32;
        let color = derive_color_compound(&contribs, purity, t, &almanac).expect("compound");
        let from_derive = super::linear_rgb_from_derive_color(color);
        let from_cell =
            super::field_linear_rgb_from_cell_inputs(450.0, purity, &contribs, true, t, &almanac);
        approx3(from_derive, from_cell);
    }

    #[test]
    fn interference_t_changes_compound_rgb() {
        let almanac = mk_dual_band_almanac();
        let e1 = Entity::from_raw(1);
        let e2 = Entity::from_raw(2);
        let contribs = vec![
            FrequencyContribution::new(e1, 450.0, 2.0),
            FrequencyContribution::new(e2, 250.0, 1.8),
        ];
        let a =
            super::field_linear_rgb_from_cell_inputs(450.0, 0.5, &contribs, true, 0.0, &almanac);
        let b = super::field_linear_rgb_from_cell_inputs(
            450.0,
            0.5,
            &contribs,
            true,
            thr::INTERFERENCE_TEST_PHASE_STEP_T,
            &almanac,
        );
        let d = (a[0] - b[0]).abs() + (a[1] - b[1]).abs() + (a[2] - b[2]).abs();
        assert!(
            d > thr::RGB_APPROX_EPS * 2.0,
            "interference phase should affect blend: a={a:?} b={b:?}"
        );
    }

    #[test]
    fn field_linear_rgb_cell_purity_zero_neutral() {
        let almanac = mk_ignis_almanac();
        let mut cell = EnergyCell::default();
        cell.dominant_frequency_hz = 450.0;
        cell.purity = 1.0;
        let rgb = field_linear_rgb_from_cell(&cell, 0.0, 0.0, &almanac);
        approx3(rgb, neutral_visual_linear_rgb());
    }
}
