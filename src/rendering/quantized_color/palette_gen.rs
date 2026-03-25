//! Generación pura de paletas desde el Almanac (Sprint 05 como SSOT).

use bevy::color::Color;
use bevy::color::ColorToComponents;

use crate::blueprint::almanac::{AlchemicalAlmanac, ElementDef};
use crate::worldgen::constants::REFERENCE_DENSITY;
use crate::worldgen::visual_derivation::{derive_color, derive_emission, derive_opacity};

use super::constants::{
    PALETTE_ALPHA_EMISSION_WEIGHT, PALETTE_SAMPLE_DENSITY_MIN_FRAC,
    PALETTE_SAMPLE_DENSITY_RANGE_FRAC, PALETTE_SAMPLE_TEMP_BASE, PALETTE_SAMPLE_TEMP_SPAN,
};

/// Colores RGBA lineales listos para SSBO (`vec4` en GPU).
#[derive(Clone, Debug, PartialEq)]
pub struct PaletteBlock {
    pub n_max: u32,
    pub colors_rgba: Vec<[f32; 4]>,
}

/// Construye `n_max` muestras en el eje “energía interna” reutilizando derivación visual.
///
/// `purity` y densidad/temperatura se escalan con el índice para que el índice 0 sea baja
/// energía (apagado/gris) y el último sea alta (color almanac + emisión/opacidad coherentes).
pub fn generate_palette(
    element: &ElementDef,
    n_max: u32,
    almanac: &AlchemicalAlmanac,
) -> PaletteBlock {
    if n_max == 0 {
        return PaletteBlock {
            n_max: 0,
            colors_rgba: Vec::new(),
        };
    }

    let state = element.matter_state;
    let freq = element.frequency_hz;
    let mut colors_rgba = Vec::with_capacity(n_max as usize);

    let last = (n_max - 1).max(1) as f32;

    for i in 0..n_max {
        let t = if n_max <= 1 { 1.0 } else { i as f32 / last };
        let purity = t.clamp(0.0, 1.0);
        let color = derive_color(freq, purity, almanac);
        let density = REFERENCE_DENSITY
            * (PALETTE_SAMPLE_DENSITY_MIN_FRAC + PALETTE_SAMPLE_DENSITY_RANGE_FRAC * t);
        let temperature = PALETTE_SAMPLE_TEMP_BASE + PALETTE_SAMPLE_TEMP_SPAN * t;
        let emission = derive_emission(temperature, state);
        let opacity = derive_opacity(density, state);
        let lin = color.to_linear();
        colors_rgba.push([
            lin.red,
            lin.green,
            lin.blue,
            (opacity * (1.0 + emission * PALETTE_ALPHA_EMISSION_WEIGHT)).clamp(0.0, 1.0),
        ]);
    }

    PaletteBlock { n_max, colors_rgba }
}

/// Color de fallback visible si la paleta no está cargada (contrato blueprint).
pub fn magenta_fallback_rgba() -> [f32; 4] {
    Color::srgba(1.0, 0.0, 1.0, 1.0).to_linear().to_f32_array()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::blueprint::ElementId;
    use crate::layers::MatterState;

    fn luminance(c: &[f32; 4]) -> f32 {
        c[0] * 0.2126 + c[1] * 0.7152 + c[2] * 0.0722
    }

    fn mk_ignis() -> ElementDef {
        ElementDef {
            name: "Ignis".into(),
            symbol: "Ignis".into(),
            atomic_number: 8,
            frequency_hz: 450.0,
            freq_band: (400.0, 500.0),
            bond_energy: 1000.0,
            conductivity: 0.5,
            visibility: 0.8,
            matter_state: MatterState::Solid,
            electronegativity: 0.0,
            ionization_ev: 0.0,
            color: (1.0, 0.3, 0.0),
            is_compound: false,
            phenology: None,
            hz_identity_weight: crate::blueprint::FIELD_EAC4_HZ_IDENTITY_WEIGHT_RON_ONLY,
        }
    }

    fn mk_terra() -> ElementDef {
        ElementDef {
            name: "Terra".into(),
            symbol: "Terra".into(),
            atomic_number: 1,
            frequency_hz: 75.0,
            freq_band: (50.0, 84.0),
            bond_energy: 1000.0,
            conductivity: 0.5,
            visibility: 0.3,
            matter_state: MatterState::Solid,
            electronegativity: 0.0,
            ionization_ev: 0.0,
            color: (0.55, 0.35, 0.1),
            is_compound: false,
            phenology: None,
            hz_identity_weight: crate::blueprint::FIELD_EAC4_HZ_IDENTITY_WEIGHT_RON_ONLY,
        }
    }

    fn mk_aqua() -> ElementDef {
        ElementDef {
            name: "Aqua".into(),
            symbol: "Aqua".into(),
            atomic_number: 2,
            frequency_hz: 200.0,
            freq_band: (150.0, 250.0),
            bond_energy: 1000.0,
            conductivity: 0.5,
            visibility: 0.5,
            matter_state: MatterState::Liquid,
            electronegativity: 0.0,
            ionization_ev: 0.0,
            color: (0.1, 0.35, 0.85),
            is_compound: false,
            phenology: None,
            hz_identity_weight: crate::blueprint::FIELD_EAC4_HZ_IDENTITY_WEIGHT_RON_ONLY,
        }
    }

    #[test]
    fn generate_palette_ignis_len_matches_n_max() {
        let almanac = AlchemicalAlmanac::from_defs(vec![mk_ignis()]);
        let ignis_id = ElementId::from_name("Ignis");
        let def = almanac.get(ignis_id).expect("ignis");
        let p = generate_palette(def, 64, &almanac);
        assert_eq!(p.n_max, 64);
        assert_eq!(p.colors_rgba.len(), 64);
    }

    #[test]
    fn generate_palette_first_darker_last_brighter_ignis() {
        let almanac = AlchemicalAlmanac::from_defs(vec![mk_ignis()]);
        let def = almanac.get(ElementId::from_name("Ignis")).unwrap();
        let p = generate_palette(def, 32, &almanac);
        let l0 = luminance(&p.colors_rgba[0]);
        let l1 = luminance(&p.colors_rgba[31]);
        assert!(
            l1 > l0,
            "último índice debería ser más luminoso: {l0} vs {l1}"
        );
    }

    #[test]
    fn terra_palette_brownish_mid_band() {
        let almanac = AlchemicalAlmanac::from_defs(vec![mk_terra()]);
        let def = almanac.get(ElementId::from_name("Terra")).unwrap();
        let p = generate_palette(def, 24, &almanac);
        let c = p.colors_rgba[12];
        assert!(c[0] > c[2] && c[1] > c[2], "tonos marrones: {:?}", c);
    }

    #[test]
    fn aqua_palette_blue_dominant() {
        let almanac = AlchemicalAlmanac::from_defs(vec![mk_aqua()]);
        let def = almanac.get(ElementId::from_name("Aqua")).unwrap();
        let p = generate_palette(def, 24, &almanac);
        let c = p.colors_rgba[18];
        assert!(c[2] > c[0] && c[2] > c[1], "azul dominante: {:?}", c);
    }

    #[test]
    fn n_max_zero_and_one_no_panic() {
        let almanac = AlchemicalAlmanac::from_defs(vec![mk_ignis()]);
        let def = almanac.get(ElementId::from_name("Ignis")).unwrap();
        let z = generate_palette(def, 0, &almanac);
        assert!(z.colors_rgba.is_empty());
        let o = generate_palette(def, 1, &almanac);
        assert_eq!(o.colors_rgba.len(), 1);
    }
}
