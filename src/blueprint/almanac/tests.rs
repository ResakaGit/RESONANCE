use ron::de::from_str;

use crate::blueprint::MatterState;
use crate::blueprint::constants::FIELD_EAC4_HZ_IDENTITY_WEIGHT_RON_ONLY;
use crate::blueprint::element_id::ElementId;

use super::catalog::AlchemicalAlmanac;
use super::element_def::{ElementDef, ElementPhenologyDef};
use super::test_assets_elements_almanac;

/// Defaults compartidos por fixtures de `ElementDef` en este módulo.
mod test_defaults {
    pub(super) const BOND_ENERGY: f32 = 1000.0;
    pub(super) const CONDUCTIVITY: f32 = 0.5;
    pub(super) const ELECTRONEGATIVITY: f32 = 0.0;
    pub(super) const IONIZATION_EV: f32 = 0.0;
}

/// Hz usados en `eac2_find_stable_band_id_matches_find_stable_band` (orden irrelevante).
const EAC2_INVARIANT_PROBE_HZ: [f32; 11] = [
    15.0, 30.0, 40.0, 50.0, 84.0, 85.0, 105.0, 150.0, 300.0, 350.0, 500.0,
];

/// Vecindad de 300 Hz para Aqua (cerrado) vs Vapor compuesto (abierto en 300).
const EAC2_AQUA_VAPOR_PROBE_EPS_HZ: f32 = 1e-3;

/// Builder mínimo para tests: evita struct literals de 14 campos repetidos.
#[derive(Clone)]
struct TestElement {
    name: String,
    symbol: String,
    atomic_number: u32,
    frequency_hz: f32,
    freq_band: (f32, f32),
    bond_energy: f32,
    conductivity: f32,
    visibility: f32,
    matter_state: MatterState,
    electronegativity: f32,
    ionization_ev: f32,
    color: (f32, f32, f32),
    is_compound: bool,
    phenology: Option<ElementPhenologyDef>,
    hz_identity_weight: f32,
}

impl TestElement {
    fn new(symbol: &str, freq_band: (f32, f32), frequency_hz: f32) -> Self {
        Self {
            name: symbol.to_string(),
            symbol: symbol.to_string(),
            atomic_number: 0,
            frequency_hz,
            freq_band,
            bond_energy: test_defaults::BOND_ENERGY,
            conductivity: test_defaults::CONDUCTIVITY,
            visibility: 0.5,
            matter_state: MatterState::Solid,
            electronegativity: test_defaults::ELECTRONEGATIVITY,
            ionization_ev: test_defaults::IONIZATION_EV,
            color: (0.0, 0.0, 0.0),
            is_compound: false,
            phenology: None,
            hz_identity_weight: FIELD_EAC4_HZ_IDENTITY_WEIGHT_RON_ONLY,
        }
    }

    fn display_name(mut self, name: &str) -> Self {
        self.name = name.to_string();
        self
    }

    fn compound(mut self, v: bool) -> Self {
        self.is_compound = v;
        self
    }

    fn visibility(mut self, v: f32) -> Self {
        self.visibility = v;
        self
    }

    fn color_rgb(mut self, c: (f32, f32, f32)) -> Self {
        self.color = c;
        self
    }

    fn matter(mut self, m: MatterState) -> Self {
        self.matter_state = m;
        self
    }

    fn atomic_number(mut self, n: u32) -> Self {
        self.atomic_number = n;
        self
    }

    fn bond_energy(mut self, v: f32) -> Self {
        self.bond_energy = v;
        self
    }

    fn hz_identity_weight(mut self, w: f32) -> Self {
        self.hz_identity_weight = w;
        self
    }

    fn build(self) -> ElementDef {
        ElementDef {
            name: self.name,
            symbol: self.symbol,
            atomic_number: self.atomic_number,
            frequency_hz: self.frequency_hz,
            freq_band: self.freq_band,
            bond_energy: self.bond_energy,
            conductivity: self.conductivity,
            visibility: self.visibility,
            matter_state: self.matter_state,
            electronegativity: self.electronegativity,
            ionization_ev: self.ionization_ev,
            color: self.color,
            is_compound: self.is_compound,
            phenology: self.phenology,
            hz_identity_weight: self.hz_identity_weight,
        }
    }
}

fn mk_id(name: &str) -> ElementId {
    ElementId::from_name(name)
}

#[test]
fn ron_parsing_element_def() {
    let input = r#"
            ElementDef(
                name: "Umbra",
                symbol: "Umbra",
                atomic_number: 0,
                frequency_hz: 20.0,
                freq_band: (10.0, 30.0),
                bond_energy: 1000.0,
                conductivity: 0.5,
                visibility: 0.1,
                matter_state: Solid,
                electronegativity: 0.0,
                ionization_ev: 0.0,
                color: (0.15, 0.0, 0.3),
                is_compound: false,
            )
        "#;

    let def: ElementDef = from_str(input).expect("RON -> ElementDef");
    assert_eq!(def.name, "Umbra");
    assert_eq!(def.symbol, "Umbra");
    assert!((def.hz_identity_weight - FIELD_EAC4_HZ_IDENTITY_WEIGHT_RON_ONLY).abs() < f32::EPSILON);
    assert!(def.contains(20.0));

    // Pure: bordes inclusivos; purity en el centro ~1 y en el borde ~0.
    assert!((def.purity(20.0) - 1.0).abs() < 1e-6);
    assert!(def.purity(10.0) < 1e-6);
}

#[test]
fn find_stable_band_id_compound_exclusive_boundaries() {
    let pure_id = mk_id("Umbra");
    let comp_id = mk_id("Ceniza");

    let pure_def = TestElement::new("Umbra", (10.0, 30.0), 20.0)
        .visibility(0.1)
        .color_rgb((0.15, 0.0, 0.3))
        .build();
    let comp_def = TestElement::new("Ceniza", (30.0, 50.0), 40.0)
        .compound(true)
        .visibility(0.05)
        .color_rgb((0.35, 0.18, 0.2))
        .build();

    let almanac = AlchemicalAlmanac::from_defs(vec![pure_def, comp_def]);

    // Boundary en min del compuesto: debe caer en el puro adyacente (porque compuesto excluye min).
    assert_eq!(
        almanac.find_stable_band_id(30.0),
        Some(pure_id),
        "Boundary 30.0 must be assigned to the pure element"
    );

    // Dentro del compuesto: retorna compuesto.
    assert_eq!(almanac.find_stable_band_id(35.0), Some(comp_id));
}

#[test]
fn from_defs_smoke_two_non_overlapping_bands_lookup() {
    let id_a = mk_id("A");
    let id_b = mk_id("B");

    let def_a = TestElement::new("A", (10.0, 20.0), 15.0)
        .visibility(0.1)
        .build();
    let def_b = TestElement::new("B", (50.0, 60.0), 55.0)
        .visibility(0.2)
        .build();

    let almanac = AlchemicalAlmanac::from_defs(vec![def_a, def_b]);

    // Smoke: lookup en cada banda.
    assert_eq!(almanac.find_stable_band_id(15.0), Some(id_a));
    assert_eq!(almanac.find_stable_band_id(55.0), Some(id_b));
    assert_eq!(almanac.find_stable_band_id(999.0), None);
}

#[test]
fn from_defs_discards_non_finite_bands_and_normalizes_inverted_ranges() {
    let valid = TestElement::new("Valid", (60.0, 40.0), 50.0)
        .atomic_number(1)
        .visibility(0.3)
        .color_rgb((0.2, 0.2, 0.2))
        .build();
    let invalid = TestElement::new("Invalid", (f32::NAN, 80.0), 70.0)
        .atomic_number(2)
        .visibility(0.3)
        .color_rgb((0.2, 0.2, 0.2))
        .build();

    let almanac = AlchemicalAlmanac::from_defs(vec![valid, invalid]);
    assert!(almanac.find_stable_band(50.0).is_some());
    assert!(almanac.find_stable_band(70.0).is_none());
}

#[test]
fn from_defs_deduplicates_symbol_and_uses_latest_definition() {
    let first = TestElement::new("Ignis", (400.0, 450.0), 430.0)
        .display_name("Ignis A")
        .atomic_number(1)
        .matter(MatterState::Plasma)
        .visibility(0.8)
        .color_rgb((1.0, 0.2, 0.0))
        .build();
    let second = TestElement::new("Ignis", (460.0, 500.0), 470.0)
        .display_name("Ignis B")
        .atomic_number(2)
        .matter(MatterState::Plasma)
        .visibility(0.8)
        .color_rgb((1.0, 0.3, 0.0))
        .build();

    let almanac = AlchemicalAlmanac::from_defs(vec![first, second]);
    let stable = almanac.find_stable_band(470.0).expect("Ignis B must exist");
    assert_eq!(stable.name, "Ignis B");
    assert!(almanac.find_stable_band(430.0).is_none());
}

#[test]
fn content_fingerprint_changes_with_matter_state() {
    let mut a = TestElement::new("X", (90.0, 110.0), 100.0)
        .bond_energy(1.0)
        .visibility(0.5)
        .color_rgb((0.5, 0.5, 0.5))
        .build();
    let al1 = AlchemicalAlmanac::from_defs(vec![a.clone()]);
    a.matter_state = MatterState::Plasma;
    let al2 = AlchemicalAlmanac::from_defs(vec![a]);
    assert_ne!(al1.content_fingerprint(), al2.content_fingerprint());
}

#[test]
fn flora_element_loads_and_has_correct_band() {
    let almanac = test_assets_elements_almanac();
    let flora = almanac
        .find_stable_band(85.0)
        .expect("Flora debe resolverse en 85 Hz sin solapar Terra");
    assert_eq!(flora.name, "Flora");
    assert!(
        flora.freq_band.0 <= 85.0 && flora.freq_band.1 >= 85.0,
        "banda debe contener 85 Hz"
    );
    assert!(
        flora.electronegativity > 2.0,
        "Flora necesita alta avidez por nutrientes"
    );
    assert!(flora.bond_energy < 2000.0, "Flora es flexible, no roca");
    assert!(
        flora.phenology.is_some(),
        "EA8: flora.ron debe declarar perfil fenológico opcional"
    );
}

#[test]
fn terra_flora_band_boundaries_resolve_without_overlap() {
    let almanac = test_assets_elements_almanac();
    let t = almanac
        .find_stable_band(84.0)
        .expect("84 Hz debe seguir en Terra");
    assert_eq!(t.symbol, "Terra");
    let f = almanac
        .find_stable_band(85.0)
        .expect("85 Hz debe resolver Flora");
    assert_eq!(f.symbol, "Fl");
}

#[test]
fn flora_element_id_uses_symbol_fl_not_display_name() {
    let almanac = test_assets_elements_almanac();
    assert!(almanac.get(ElementId::from_name("Fl")).is_some());
    assert!(almanac.get(ElementId::from_name("Flora")).is_none());
}

/// EAC2: antes del índice filtrado, Ceniza quedaba fuera (min 30 == max Umbra).
#[test]
fn eac2_assets_ceniza_resolves_inside_compound_band() {
    let almanac = test_assets_elements_almanac();
    let d = almanac
        .find_stable_band(40.0)
        .expect("40 Hz debe caer en Ceniza (compuesto exclusivo entre 30 y 50)");
    assert_eq!(d.symbol, "Ceniza");
}

/// Solape Flora [85,110] × Lodo (100,200) exclusivo: banda más estrecha gana.
#[test]
fn eac2_assets_flora_wins_over_lodo_at_overlap() {
    let almanac = test_assets_elements_almanac();
    let d = almanac
        .find_stable_band(105.0)
        .expect("105 Hz en solape Flora/Lodo");
    assert_eq!(d.symbol, "Fl");
}

/// Humus más estrecho que Lodo en la región 110–200.
#[test]
fn eac2_assets_humus_wins_over_lodo_mid_band() {
    let almanac = test_assets_elements_almanac();
    let d = almanac
        .find_stable_band(150.0)
        .expect("150 Hz humus vs lodo");
    assert_eq!(d.symbol, "Humus");
}

/// Misma anchura y ambos puros: en 500 Hz entran Ignis [400,500] y Rayo [500,600]; gana mayor `min`.
#[test]
fn eac2_assets_equal_width_boundary_prefers_later_min() {
    let almanac = test_assets_elements_almanac();
    let d = almanac.find_stable_band(500.0).expect("borde Ignis/Rayo");
    assert_eq!(d.symbol, "Rayo");
}

/// Vapor es compuesto exclusivo: 300 Hz solo en Aqua (no en Vapor); regresión EAC2 vs suposición “ambos en borde”.
#[test]
fn eac2_assets_aqua_at_300_vapor_compound_excludes_endpoint() {
    let almanac = test_assets_elements_almanac();
    let d = almanac.find_stable_band(300.0).expect("300 Hz");
    assert_eq!(d.symbol, "Aqua");
}

#[test]
fn eac2_assets_umbra_at_shared_boundary_30() {
    let almanac = test_assets_elements_almanac();
    let d = almanac
        .find_stable_band(30.0)
        .expect("30 Hz: Umbra cerrado, Ceniza excluye min");
    assert_eq!(d.symbol, "Umbra");
}

#[test]
fn eac2_assets_terra_at_shared_boundary_50() {
    let almanac = test_assets_elements_almanac();
    let d = almanac
        .find_stable_band(50.0)
        .expect("50 Hz: Ceniza excluye max, Terra cierra min");
    assert_eq!(d.symbol, "Terra");
}

#[test]
fn eac2_near_300_hz_float_robustness_aqua_vs_vapor() {
    let almanac = test_assets_elements_almanac();
    let eps = EAC2_AQUA_VAPOR_PROBE_EPS_HZ;
    assert_eq!(
        almanac
            .find_stable_band(300.0 - eps)
            .map(|d| d.symbol.as_str()),
        Some("Aqua")
    );
    assert_eq!(
        almanac
            .find_stable_band(300.0 + eps)
            .map(|d| d.symbol.as_str()),
        Some("Vapor")
    );
}

#[test]
fn eac2_find_stable_band_rejects_non_finite_freq() {
    let almanac = test_assets_elements_almanac();
    for bad in [f32::NAN, f32::INFINITY, f32::NEG_INFINITY] {
        assert!(
            almanac.find_stable_band(bad).is_none(),
            "find_stable_band({bad:?})"
        );
        assert!(
            almanac.find_stable_band_id(bad).is_none(),
            "find_stable_band_id({bad:?})"
        );
    }
}

/// Invariante API: mismo `ElementId` vía id o vía def (`symbol` canónico).
#[test]
fn eac2_find_stable_band_id_matches_find_stable_band() {
    let almanac = test_assets_elements_almanac();
    for f in EAC2_INVARIANT_PROBE_HZ {
        let id = almanac.find_stable_band_id(f);
        let from_def = almanac
            .find_stable_band(f)
            .map(|d| ElementId::from_name(d.symbol.as_str()));
        assert_eq!(id, from_def, "freq={f}");
    }
}

/// Empate ancho + min: gana el `ElementId::raw()` menor (determinismo).
#[test]
fn eac2_synthetic_tiebreak_prefers_lower_element_id_raw() {
    let lo = TestElement::new("LoRaw", (10.0, 20.0), 15.0)
        .bond_energy(1.0)
        .build();
    let hi = TestElement::new("HiRaw", (10.0, 20.0), 15.0)
        .bond_energy(1.0)
        .build();
    let id_lo = ElementId::from_name("LoRaw");
    let id_hi = ElementId::from_name("HiRaw");
    let almanac_a = AlchemicalAlmanac::from_defs(vec![lo.clone(), hi.clone()]);
    let almanac_b = AlchemicalAlmanac::from_defs(vec![hi, lo]);
    let pick = [id_lo, id_hi]
        .into_iter()
        .min_by_key(|id| id.raw())
        .expect("dos candidatos");
    assert_eq!(almanac_a.find_stable_band_id(15.0), Some(pick));
    assert_eq!(almanac_b.find_stable_band_id(15.0), Some(pick));
}

#[test]
fn eac4_from_defs_computes_global_hz_bounds() {
    let lo = TestElement::new("E4Lo", (400.0, 500.0), 450.0)
        .color_rgb((1.0, 0.2, 0.0))
        .build();
    let hi = TestElement::new("E4Hi", (200.0, 300.0), 250.0)
        .color_rgb((0.0, 0.1, 0.9))
        .build();
    let almanac = AlchemicalAlmanac::from_defs(vec![lo, hi]);
    assert_eq!(almanac.game_frequency_hz_bounds(), Some((200.0, 500.0)));
}

#[test]
fn eac4_builder_hz_identity_weight_applied() {
    let def = TestElement::new("E4W", (100.0, 200.0), 150.0)
        .hz_identity_weight(0.0)
        .build();
    assert_eq!(def.hz_identity_weight, 0.0);
}

#[test]
fn eac4_content_fingerprint_depends_on_hz_identity_weight() {
    let al_a = AlchemicalAlmanac::from_defs(vec![
        TestElement::new("Fp1", (100.0, 200.0), 150.0)
            .hz_identity_weight(FIELD_EAC4_HZ_IDENTITY_WEIGHT_RON_ONLY)
            .build(),
    ]);
    let al_b = AlchemicalAlmanac::from_defs(vec![
        TestElement::new("Fp1", (100.0, 200.0), 150.0)
            .hz_identity_weight(0.0)
            .build(),
    ]);
    assert_ne!(
        al_a.content_fingerprint(),
        al_b.content_fingerprint(),
        "huella debe invalidar paleta si solo cambia hz_identity_weight"
    );
}
