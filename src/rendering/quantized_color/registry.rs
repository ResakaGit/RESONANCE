//! Registro de paletas planas + metadatos para SSBO.

use bevy::prelude::*;
use fxhash::FxHashMap;

use crate::blueprint::AlchemicalAlmanac;
use crate::rendering::quantized_color::constants::DEFAULT_PALETTE_N_MAX;
use crate::rendering::quantized_color::palette_gen::{generate_palette, magenta_fallback_rgba};

/// Paletas linealizadas y tabla de offsets por `ElementId` (orden estable).
#[derive(Resource, Debug, Default)]
pub struct PaletteRegistry {
    /// `palette_offset` → `ElementId::raw` para depuración.
    pub slot_element_raw: Vec<u32>,
    pub offsets: Vec<u32>,
    pub n_max_per_slot: Vec<u32>,
    /// RGBA lineal contigua (última entrada = magenta fallback).
    pub flat_rgba: Vec<[f32; 4]>,
    /// Lookup O(1) por `ElementId::raw` (reconstruido en cada rebuild).
    meta_by_element_raw: FxHashMap<u32, (u32, u32)>,
    last_fingerprint: u64,
    last_palette_n_max: u32,
}

impl PaletteRegistry {
    pub fn total_vram_bytes(&self) -> usize {
        self.flat_rgba.len() * core::mem::size_of::<[f32; 4]>()
    }

    /// Reconstruye si el almanac cambió, `n_max` difiere o aún no hay datos.
    pub fn rebuild_if_needed(
        &mut self,
        almanac: &AlchemicalAlmanac,
        almanac_changed: bool,
        n_max_per_element: u32,
    ) -> bool {
        let need_probe = almanac_changed
            || self.flat_rgba.is_empty()
            || self.last_palette_n_max != n_max_per_element;
        if !need_probe {
            return false;
        }

        let fp = almanac.content_fingerprint();
        let must_rebuild = self.flat_rgba.is_empty()
            || fp != self.last_fingerprint
            || self.last_palette_n_max != n_max_per_element;
        if !must_rebuild {
            return false;
        }

        self.rebuild(almanac, n_max_per_element);
        self.rebuild_meta_map();
        self.last_fingerprint = fp;
        self.last_palette_n_max = n_max_per_element;
        true
    }

    fn rebuild(&mut self, almanac: &AlchemicalAlmanac, n_max_per_element: u32) {
        self.slot_element_raw.clear();
        self.offsets.clear();
        self.n_max_per_slot.clear();
        self.flat_rgba.clear();
        self.meta_by_element_raw.clear();

        let n_palette = n_max_per_element.max(1);
        let ids = almanac.all_element_ids_sorted();
        let mut cursor: u32 = 0;

        for id in ids {
            let Some(def) = almanac.get(id) else {
                continue;
            };
            let block = generate_palette(def, n_palette, almanac);
            if block.colors_rgba.is_empty() {
                continue;
            }
            self.slot_element_raw.push(id.raw());
            self.offsets.push(cursor);
            self.n_max_per_slot.push(block.n_max);
            cursor += block.n_max;
            self.flat_rgba.extend_from_slice(&block.colors_rgba);
        }

        self.flat_rgba.push(magenta_fallback_rgba());
    }

    fn rebuild_meta_map(&mut self) {
        self.meta_by_element_raw.clear();
        for i in 0..self.slot_element_raw.len() {
            let raw = self.slot_element_raw[i];
            let off = self.offsets[i];
            let n = self.n_max_per_slot[i];
            self.meta_by_element_raw.insert(raw, (off, n));
        }
    }

    #[inline]
    pub fn palette_meta_for_element_raw(&self, element_raw: u32) -> Option<(u32, u32)> {
        self.meta_by_element_raw.get(&element_raw).copied()
    }

    /// Utilidad tests / demos: fuerza rebuild ignorando huella.
    pub fn force_rebuild_for_tests(almanac: &AlchemicalAlmanac) -> Self {
        let mut r = PaletteRegistry::default();
        r.rebuild(almanac, DEFAULT_PALETTE_N_MAX);
        r.rebuild_meta_map();
        r.last_fingerprint = almanac.content_fingerprint();
        r.last_palette_n_max = DEFAULT_PALETTE_N_MAX;
        r
    }
}

#[cfg(test)]
mod tests {
    use super::super::archetype_element::element_id_for_world_archetype;
    use super::*;
    use crate::blueprint::almanac::ElementDef;
    use crate::layers::MatterState;
    use crate::worldgen::archetypes::WorldArchetype;

    fn mini_almanac() -> AlchemicalAlmanac {
        let ignis = ElementDef {
            name: "Ignis".into(),
            symbol: "Ignis".into(),
            atomic_number: 0,
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
        };
        AlchemicalAlmanac::from_defs(vec![ignis])
    }

    #[test]
    fn vram_footprint_under_cap_for_small_almanac() {
        let almanac = mini_almanac();
        let reg = PaletteRegistry::force_rebuild_for_tests(&almanac);
        assert!(reg.total_vram_bytes() < 200 * 1024);
    }

    #[test]
    fn n_max_id_valid_for_ignis_archetype() {
        let almanac = mini_almanac();
        let reg = PaletteRegistry::force_rebuild_for_tests(&almanac);
        let eid = element_id_for_world_archetype(WorldArchetype::IgnisSolid);
        let (off, n) = reg
            .palette_meta_for_element_raw(eid.raw())
            .expect("slot ignis");
        assert!(n >= 1);
        assert!((off as usize) < reg.flat_rgba.len());
    }

    #[test]
    fn rebuild_tracks_n_max_change() {
        let almanac = mini_almanac();
        let mut reg = PaletteRegistry::default();
        assert!(reg.rebuild_if_needed(&almanac, true, 32));
        let b1 = reg.flat_rgba.len();
        assert!(!reg.rebuild_if_needed(&almanac, false, 32));
        assert!(reg.rebuild_if_needed(&almanac, false, 48));
        let b2 = reg.flat_rgba.len();
        assert_ne!(b1, b2);
    }
}
