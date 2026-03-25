//! Definición serializable de un elemento (RON / Asset).
//! Dominio: datos de contenido por elemento (banda Hz, materia, fenología, EAC4).

use bevy::asset::Asset;
use bevy::prelude::ReflectDefault;
use bevy::reflect::{Reflect, TypePath};
use serde::{Deserialize, Serialize};

use crate::blueprint::constants::FIELD_EAC4_HZ_IDENTITY_WEIGHT_RON_ONLY;
use crate::layers::MatterState;

/// Default EAC4: solo identidad RON (`ElementDef.color`); sin tinte por espectro Hz.
fn default_hz_identity_weight() -> f32 {
    FIELD_EAC4_HZ_IDENTITY_WEIGHT_RON_ONLY
}

/// Pesos y anclas sRGB para fenología visual (EA8); vive en datos del almanaque.
#[derive(Clone, Copy, Debug, Default, Deserialize, Serialize, Reflect)]
#[reflect(Debug, Default)]
pub struct ElementPhenologyDef {
    pub young_rgb: (f32, f32, f32),
    pub mature_rgb: (f32, f32, f32),
    #[serde(default)]
    pub w_growth: f32,
    #[serde(default)]
    pub w_qe: f32,
    #[serde(default)]
    pub w_purity: f32,
}

#[derive(Asset, TypePath, Clone, Debug, Deserialize, Serialize)]
pub struct ElementDef {
    /// Human-readable name (contenido).
    pub name: String,
    /// Abreviatura/símbolo (contenido).
    pub symbol: String,

    /// Número atómico real (fuente: tabla periódica).
    pub atomic_number: u32,

    /// Frecuencia central [Hz].
    pub frequency_hz: f32,
    /// Banda de estabilidad [min, max].
    pub freq_band: (f32, f32),

    pub bond_energy: f32,
    pub conductivity: f32,
    pub visibility: f32,
    pub matter_state: MatterState,

    // Parámetros “periódicos” (para fórmulas emergentes futuras).
    pub electronegativity: f32,
    pub ionization_ev: f32,

    /// Color RGB en [0,1].
    pub color: (f32, f32, f32),

    /// Compat con la semántica V2 (compuestos tenían fronteras exclusivas).
    #[serde(default)]
    pub is_compound: bool,

    /// Perfil fenológico opcional (EA8). `None` → sin mezcla young/mature desde datos.
    #[serde(default)]
    pub phenology: Option<ElementPhenologyDef>,

    /// EAC4: 1.0 = solo `color` RON; 0.0 = solo matiz derivado del espectro Hz global del almanaque.
    #[serde(default = "default_hz_identity_weight")]
    pub hz_identity_weight: f32,
}

impl ElementDef {
    pub fn contains(&self, freq: f32) -> bool {
        let (min, max) = self.freq_band;
        if self.is_compound {
            freq > min && freq < max
        } else {
            freq >= min && freq <= max
        }
    }

    /// Pureza en [0,1]. Solo es >0 dentro de la banda estable.
    pub fn purity(&self, freq: f32) -> f32 {
        if !self.contains(freq) {
            return 0.0;
        }
        let range_half = self.freq_band_span() / 2.0;
        if range_half <= 0.0 {
            return 1.0;
        }
        let dist = (freq - self.frequency_hz).abs();
        (1.0 - dist / range_half).clamp(0.0, 1.0)
    }

    /// Anchura de `freq_band` (Hz). Usada por EAC2 y pureza.
    #[inline]
    pub(crate) fn freq_band_span(&self) -> f32 {
        self.freq_band.1 - self.freq_band.0
    }
}
