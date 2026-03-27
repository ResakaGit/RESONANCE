use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use crate::blueprint::constants::{
    BIOME_LEY_LINE_DELTA_QE, BIOME_LEY_LINE_VISCOSITY, BIOME_PLAIN_DELTA_QE, BIOME_PLAIN_VISCOSITY,
    BIOME_SWAMP_DELTA_QE, BIOME_SWAMP_VISCOSITY, BIOME_TUNDRA_DELTA_QE, BIOME_TUNDRA_VISCOSITY,
    BIOME_VOLCANO_DELTA_QE, BIOME_VOLCANO_VISCOSITY,
};

/// Capa 6: Ecosistema — Topología Macroscópica
///
/// Modificadores ambientales a gran escala que aplican presión constante
/// sobre las entidades de Capas 1 a 5.
///
/// Aplicación (en sistema presion_entorno):
///   entidad.qe += bioma.delta_qe_constante * dt
///   entidad.tasa_disipacion_efectiva *= bioma.viscosidad_terreno
#[derive(Component, Reflect, Debug, Clone, Serialize, Deserialize)]
#[reflect(Component)]
pub struct AmbientPressure {
    /// Inyecta (positivo) o roba (negativo) energía por segundo.
    pub delta_qe_constant: f32,

    /// Multiplicador de fricción/viscosidad del terreno.
    /// 1.0 = neutral, >1.0 = viscoso, <1.0 = resbaloso.
    pub terrain_viscosity: f32,
}

impl Default for AmbientPressure {
    fn default() -> Self {
        Self {
            delta_qe_constant: BIOME_PLAIN_DELTA_QE,
            terrain_viscosity: BIOME_PLAIN_VISCOSITY,
        }
    }
}

impl AmbientPressure {
    pub fn new(delta_qe: f32, viscosity: f32) -> Self {
        Self {
            delta_qe_constant: delta_qe,
            terrain_viscosity: viscosity.max(0.0),
        }
    }

    pub fn volcano() -> Self {
        Self {
            delta_qe_constant: BIOME_VOLCANO_DELTA_QE,
            terrain_viscosity: BIOME_VOLCANO_VISCOSITY,
        }
    }

    pub fn ley_line() -> Self {
        Self {
            delta_qe_constant: BIOME_LEY_LINE_DELTA_QE,
            terrain_viscosity: BIOME_LEY_LINE_VISCOSITY,
        }
    }

    pub fn swamp() -> Self {
        Self {
            delta_qe_constant: BIOME_SWAMP_DELTA_QE,
            terrain_viscosity: BIOME_SWAMP_VISCOSITY,
        }
    }

    pub fn tundra() -> Self {
        Self {
            delta_qe_constant: BIOME_TUNDRA_DELTA_QE,
            terrain_viscosity: BIOME_TUNDRA_VISCOSITY,
        }
    }

    /// Deep space vacuum: near-zero dissipation, no energy injection.
    /// Axiom 4 still holds (dissipation > 0) but at negligible rate.
    pub fn vacuum() -> Self {
        Self {
            delta_qe_constant: 0.0,
            terrain_viscosity: 0.001,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::blueprint::constants::{BIOME_PLAIN_DELTA_QE, BIOME_PLAIN_VISCOSITY};

    #[test]
    fn default_matches_plain_biome_constants() {
        let p = AmbientPressure::default();
        assert_eq!(p.delta_qe_constant, BIOME_PLAIN_DELTA_QE);
        assert_eq!(p.terrain_viscosity, BIOME_PLAIN_VISCOSITY);
    }

    #[test]
    fn volcano_preset_negative_delta_and_high_viscosity() {
        let v = AmbientPressure::volcano();
        assert!(v.delta_qe_constant < 0.0);
        assert!(v.terrain_viscosity > BIOME_PLAIN_VISCOSITY);
    }

    #[test]
    fn new_clamps_negative_viscosity_to_zero() {
        let p = AmbientPressure::new(1.0, -3.0);
        assert_eq!(p.terrain_viscosity, 0.0);
    }
}
