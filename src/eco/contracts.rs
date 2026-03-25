//! Contratos de Eco-Boundaries. Consumidos por grid, lookup y simulación.
//!
//! **Reflect en enums:** acoplamiento con Bevy a propósito del monolito Resonance (inspector /
//! registro de tipos). Extraer crate `eco` puro implicaría duplicar tipos o un bridge de mapeo.

use bevy::prelude::Reflect;
use serde::{Deserialize, Serialize};

/// Clasificación emergente de zona por celda del campo energético.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize, Reflect)]
#[reflect(Debug, PartialEq, Hash)]
#[repr(u8)]
pub enum ZoneClass {
    HighAtmosphere = 0,
    Surface = 1,
    Subaquatic = 2,
    Subterranean = 3,
    Volcanic = 4,
    Frozen = 5,
    Void = 6,
}

/// Tipo de frontera entre dos zonas adyacentes.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize, Reflect)]
#[reflect(Debug, PartialEq, Hash)]
#[repr(u8)]
pub enum TransitionType {
    PhaseBoundary = 0,
    DensityGradient = 1,
    ElementFrontier = 2,
    ThermalShock = 3,
}

/// Marca por celda: interior estable de zona o frontera con interpolación.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize, Reflect)]
#[reflect(Debug, PartialEq)]
pub enum BoundaryMarker {
    /// Centro de una zona homogénea; `zone_id` indexa en la tabla de contextos cacheados.
    Interior { zone_id: u16 },
    /// Celda en transición entre dos clases de zona.
    Boundary {
        zone_a: ZoneClass,
        zone_b: ZoneClass,
        gradient_factor: f32,
        transition_type: TransitionType,
    },
}

/// Valores base cacheados por `zone_id` (sin flags de posición).
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct ZoneContext {
    pub pressure: f32,
    pub viscosity: f32,
    pub temperature_base: f32,
    pub dissipation_mod: f32,
    pub reactivity_mod: f32,
}

impl Default for ZoneContext {
    /// Caso más frecuente: superficie habitable (presión/viscosidad “llanura”, sin mod extra).
    fn default() -> Self {
        Self {
            pressure: 1.0,
            viscosity: crate::blueprint::constants::BIOME_PLAIN_VISCOSITY,
            temperature_base: 0.0,
            dissipation_mod: 1.0,
            reactivity_mod: 1.0,
        }
    }
}

/// Contexto que consumen sistemas de simulación en el hot path.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct ContextResponse {
    pub pressure: f32,
    pub viscosity: f32,
    pub temperature_base: f32,
    pub dissipation_mod: f32,
    pub reactivity_mod: f32,
    pub is_boundary: bool,
    pub zone: ZoneClass,
}

impl Default for ContextResponse {
    fn default() -> Self {
        let z = ZoneContext::default();
        Self {
            pressure: z.pressure,
            viscosity: z.viscosity,
            temperature_base: z.temperature_base,
            dissipation_mod: z.dissipation_mod,
            reactivity_mod: z.reactivity_mod,
            is_boundary: false,
            zone: ZoneClass::Surface,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn assert_send_sync<T: Send + Sync>() {}

    fn assert_copy<T: Copy>() {}

    #[test]
    fn tipos_copy_send_sync() {
        assert_copy::<ZoneClass>();
        assert_copy::<TransitionType>();
        assert_copy::<BoundaryMarker>();
        assert_copy::<ZoneContext>();
        assert_copy::<ContextResponse>();
        assert_send_sync::<ZoneClass>();
        assert_send_sync::<TransitionType>();
        assert_send_sync::<BoundaryMarker>();
        assert_send_sync::<ZoneContext>();
        assert_send_sync::<ContextResponse>();

        let ctx_sz = std::mem::size_of::<ContextResponse>();
        assert!(
            ctx_sz <= 32,
            "ContextResponse grew ({ctx_sz} B); review padding on hot path"
        );
    }

    #[test]
    fn zone_class_ron_roundtrip() {
        for z in [
            ZoneClass::HighAtmosphere,
            ZoneClass::Surface,
            ZoneClass::Subaquatic,
            ZoneClass::Subterranean,
            ZoneClass::Volcanic,
            ZoneClass::Frozen,
            ZoneClass::Void,
        ] {
            let s = ron::ser::to_string(&z).expect("serialize");
            let back: ZoneClass = ron::de::from_str(&s).expect("deserialize");
            assert_eq!(back, z, "RON: {s}");
        }
    }

    #[test]
    fn boundary_marker_constructores() {
        let i = BoundaryMarker::Interior { zone_id: 42 };
        assert!(matches!(i, BoundaryMarker::Interior { zone_id: 42 }));

        let b = BoundaryMarker::Boundary {
            zone_a: ZoneClass::Surface,
            zone_b: ZoneClass::Void,
            gradient_factor: 0.35,
            transition_type: TransitionType::DensityGradient,
        };
        assert!(matches!(
            b,
            BoundaryMarker::Boundary {
                zone_a: ZoneClass::Surface,
                zone_b: ZoneClass::Void,
                gradient_factor: _,
                transition_type: TransitionType::DensityGradient,
            }
        ));
    }

    #[test]
    fn context_response_default_es_surface() {
        let d = ContextResponse::default();
        assert_eq!(d.zone, ZoneClass::Surface);
        assert!(!d.is_boundary);
        assert_eq!(d.pressure, 1.0);
        assert_eq!(
            d.viscosity,
            crate::blueprint::constants::BIOME_PLAIN_VISCOSITY
        );
        assert_eq!(d.temperature_base, 0.0);
        assert_eq!(d.dissipation_mod, 1.0);
        assert_eq!(d.reactivity_mod, 1.0);
    }

    #[test]
    fn zone_context_default_coincide_con_surface() {
        let z = ZoneContext::default();
        let c = ContextResponse::default();
        assert_eq!(z.pressure, c.pressure);
        assert_eq!(z.viscosity, c.viscosity);
        assert_eq!(z.temperature_base, c.temperature_base);
        assert_eq!(z.dissipation_mod, c.dissipation_mod);
        assert_eq!(z.reactivity_mod, c.reactivity_mod);
    }
}
