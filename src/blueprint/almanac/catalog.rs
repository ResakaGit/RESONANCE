//! Recurso runtime de solo lectura: mapa `ElementId` → `ElementDef`.
//! Dominio: almacenamiento y consultas del almanaque (incl. huella de contenido).

use std::collections::HashMap;

use bevy::prelude::*;

use crate::blueprint::element_id::ElementId;
use crate::blueprint::constants::{FNV_OFFSET_BASIS, FNV_PRIME};
use crate::layers::MatterState;

use super::eac::{
    cmp_stable_band_candidates_eac2, compute_game_frequency_hz_bounds,
    ALMANAC_DEFAULT_PEAK_FROM_BAND_MIDPOINT,
};
use super::element_def::ElementDef;

/// Almanac read-only (en runtime) para que el motor sea agnóstico a contenido.
///
/// Resolución Hz → elemento estable: ver `cmp_stable_band_candidates_eac2` y
/// `docs/sprints/ELEMENT_ALMANAC_CANON/README.md` (EAC2 cerrado).
#[derive(Resource, Clone, Debug, Default)]
pub struct AlchemicalAlmanac {
    elements: HashMap<ElementId, ElementDef>,
    /// EAC4: envolvente global `(min(freq_band.0), max(freq_band.1))` en Hz de juego; `None` si no hay rango útil.
    game_frequency_hz_bounds: Option<(f32, f32)>,
}

impl AlchemicalAlmanac {
    /// Constructor data-driven para tests y utilidades puras.
    pub fn from_defs(defs: Vec<ElementDef>) -> Self {
        let mut elements: HashMap<ElementId, ElementDef> = HashMap::new();

        for mut def in defs {
            let (mut min, mut max) = def.freq_band;
            if !min.is_finite() || !max.is_finite() {
                continue;
            }
            if min > max {
                std::mem::swap(&mut min, &mut max);
            }
            def.freq_band = (min, max);
            if !def.frequency_hz.is_finite() {
                def.frequency_hz = (min + max) * ALMANAC_DEFAULT_PEAK_FROM_BAND_MIDPOINT;
            }

            let id = ElementId::from_name(&def.symbol);
            elements.insert(id, def);
        }

        let game_frequency_hz_bounds = compute_game_frequency_hz_bounds(&elements);

        Self {
            elements,
            game_frequency_hz_bounds,
        }
    }

    /// Límites del espectro Hz de juego sobre todas las bandas del almanaque (EAC4).
    #[inline]
    pub fn game_frequency_hz_bounds(&self) -> Option<(f32, f32)> {
        self.game_frequency_hz_bounds
    }

    pub fn get(&self, id: ElementId) -> Option<&ElementDef> {
        self.elements.get(&id)
    }

    /// Iteración determinista (orden `ElementId::raw` ascendente).
    pub fn all_element_ids_sorted(&self) -> Vec<ElementId> {
        let mut ids: Vec<ElementId> = self.elements.keys().copied().collect();
        ids.sort_by_key(|id| id.raw());
        ids
    }

    /// Resuelve el elemento estable cuyo `ElementDef::contains(freq)` es verdadero (EAC2).
    fn resolve_stable_band_element_id(&self, freq: f32) -> Option<ElementId> {
        if !freq.is_finite() || self.elements.is_empty() {
            return None;
        }

        self.elements
            .iter()
            .filter(|(_, def)| def.contains(freq))
            .min_by(|(a_id, a_def), (b_id, b_def)| {
                cmp_stable_band_candidates_eac2(**a_id, *a_def, **b_id, *b_def)
            })
            .map(|(id, _)| *id)
    }

    /// Resolución Hz → elemento (EAC2). Coste **O(|elements|)** por llamada.
    pub fn find_stable_band_id(&self, freq: f32) -> Option<ElementId> {
        self.resolve_stable_band_element_id(freq)
    }

    /// Igual que [`Self::find_stable_band_id`] pero devuelve la definición.
    pub fn find_stable_band(&self, freq: f32) -> Option<&ElementDef> {
        let id = self.resolve_stable_band_element_id(freq)?;
        self.elements.get(&id)
    }

    /// Huella determinista del contenido (invalidar cachés derivadas: paletas GPU, etc.).
    ///
    /// Hashea banda Hz, RGB, `hz_identity_weight` (EAC4), `visibility`, `is_compound`, fenología, etc.
    pub fn content_fingerprint(&self) -> u64 {
        let mut ids: Vec<ElementId> = self.elements.keys().copied().collect();
        ids.sort_by_key(|id| id.raw());
        let mut h: u64 = u64::from(FNV_OFFSET_BASIS);
        for id in ids {
            let Some(def) = self.elements.get(&id) else {
                continue;
            };
            h ^= u64::from(id.raw());
            h = h.wrapping_mul(u64::from(FNV_PRIME));
            h ^= u64::from(def.frequency_hz.to_bits());
            h = h.rotate_left(7) ^ u64::from(def.freq_band.0.to_bits());
            h = h.rotate_left(7) ^ u64::from(def.freq_band.1.to_bits());
            h = h.rotate_left(5) ^ u64::from(matter_state_tag(def.matter_state));
            h = h.rotate_left(7) ^ u64::from(def.color.0.to_bits());
            h = h.rotate_left(7) ^ u64::from(def.color.1.to_bits());
            h = h.rotate_left(7) ^ u64::from(def.color.2.to_bits());
            h = h.rotate_left(5) ^ u64::from(def.hz_identity_weight.to_bits());
            h = h.rotate_left(11) ^ u64::from(def.symbol.len() as u32);
            h ^= u64::from(def.visibility.to_bits());
            h = h.rotate_left(3) ^ u64::from(def.is_compound as u8);
            if let Some(ph) = def.phenology {
                h = h.rotate_left(5) ^ u64::from(ph.young_rgb.0.to_bits());
                h = h.rotate_left(5) ^ u64::from(ph.young_rgb.1.to_bits());
                h = h.rotate_left(5) ^ u64::from(ph.young_rgb.2.to_bits());
                h = h.rotate_left(5) ^ u64::from(ph.mature_rgb.0.to_bits());
                h = h.rotate_left(5) ^ u64::from(ph.mature_rgb.1.to_bits());
                h = h.rotate_left(5) ^ u64::from(ph.mature_rgb.2.to_bits());
                h = h.rotate_left(3) ^ u64::from(ph.w_growth.to_bits());
                h = h.rotate_left(3) ^ u64::from(ph.w_qe.to_bits());
                h = h.rotate_left(3) ^ u64::from(ph.w_purity.to_bits());
            }
        }
        h
    }
}

#[inline]
fn matter_state_tag(state: MatterState) -> u8 {
    match state {
        MatterState::Solid => 1,
        MatterState::Liquid => 2,
        MatterState::Gas => 3,
        MatterState::Plasma => 4,
    }
}
