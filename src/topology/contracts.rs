//! Contratos públicos del subsistema topológico: muestras, tipos y drenaje.

use crate::math_types::Vec2;
use bevy::prelude::Reflect;
use serde::{Deserialize, Serialize};

use crate::topology::constants::{DRAINAGE_DRY, DRAINAGE_MOIST, DRAINAGE_WET};

/// Clasificación geométrica derivada (sin energía): T5 escribe, el resto lee.
#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug, Default, Reflect, Serialize, Deserialize)]
pub enum TerrainType {
    Peak,
    Ridge,
    Slope,
    Valley,
    #[default]
    Plain,
    Riverbed,
    Basin,
    Cliff,
    Plateau,
}

/// Clasificación de caudal por acumulación de drenaje.
#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug, Reflect, Serialize, Deserialize)]
pub enum DrainageClass {
    Dry,
    Moist,
    Wet,
    River,
}

impl DrainageClass {
    /// Umbrales desde `constants` — determinista y sin estado.
    #[inline]
    pub fn from_accumulation(acc: f32) -> Self {
        if !acc.is_finite() || acc < DRAINAGE_DRY {
            Self::Dry
        } else if acc < DRAINAGE_MOIST {
            Self::Moist
        } else if acc <= DRAINAGE_WET {
            Self::Wet
        } else {
            Self::River
        }
    }
}

/// Snapshot de una celda para lectura O(1) sin borrow del resource completo.
#[derive(Copy, Clone, PartialEq, Debug, Reflect, Serialize, Deserialize)]
pub struct TerrainSample {
    pub altitude: f32,
    pub slope: f32,
    pub aspect: f32,
    pub drainage: Vec2,
    pub drainage_accumulation: f32,
    pub terrain_type: TerrainType,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::topology::constants;

    fn assert_copy<T: Copy>() {}

    #[test]
    fn terrain_type_default_is_plain() {
        assert_eq!(TerrainType::default(), TerrainType::Plain);
    }

    #[test]
    fn drainage_class_from_accumulation_dry_and_river() {
        assert_eq!(DrainageClass::from_accumulation(5.0), DrainageClass::Dry);
        assert_eq!(
            DrainageClass::from_accumulation(150.0),
            DrainageClass::River
        );
    }

    #[test]
    fn terrain_sample_is_copy() {
        assert_copy::<TerrainSample>();
    }

    #[test]
    fn terrain_type_serde_roundtrip() {
        let v = TerrainType::Cliff;
        let json = serde_json::to_string(&v).expect("serialize");
        let back: TerrainType = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(v, back);
    }

    #[test]
    fn drainage_at_threshold_boundary_is_wet_not_river() {
        assert_eq!(
            DrainageClass::from_accumulation(constants::DRAINAGE_WET),
            DrainageClass::Wet
        );
    }

    #[test]
    fn drainage_moist_mid_band() {
        assert_eq!(DrainageClass::from_accumulation(25.0), DrainageClass::Moist);
    }

    #[test]
    fn river_threshold_matches_wet_upper_bound() {
        assert_eq!(
            constants::RIVER_THRESHOLD,
            constants::DRAINAGE_WET,
            "BLUEPRINT: River is accumulation > RIVER_THRESHOLD; Wet includes up to DRAINAGE_WET"
        );
    }
}
