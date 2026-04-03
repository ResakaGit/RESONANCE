//! Enums de dominio puro — tipos canónicos del modelo físico.
//! Pure domain enums — canonical types of the physics model.
//!
//! Estos enums representan conceptos del modelo (estados de materia,
//! roles de órganos, clases tróficas, etapas de ciclo de vida).
//! Los componentes ECS en layers/ re-exportan estos tipos y añaden
//! derives de Component donde es necesario.
//! Las ecuaciones en blueprint/equations/ los importan desde aquí.

use bevy::prelude::Reflect;
use serde::{Deserialize, Serialize};

// ---------------------------------------------------------------------------
// MatterState
// ---------------------------------------------------------------------------

/// Estados de la materia con implicaciones de gameplay:
///   Solido:  sin velocidad (fijado), alto daño colisión, baja disipación
///   Liquido: velocidad limitada, conductividad moderada, fluye alrededor de obstáculos
///   Gas:     sin límite de velocidad, alta disipación, atraviesa sólidos
///   Plasma:  máximo daño, máxima disipación, emite radiación (Capa 8)
///
/// Matter states with gameplay implications (Axiom 1 + Axiom 4).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Reflect, Default, Deserialize, Serialize)]
pub enum MatterState {
    #[default]
    Solid,
    Liquid,
    Gas,
    Plasma,
}

// ---------------------------------------------------------------------------
// OrganRole
// ---------------------------------------------------------------------------

/// Rol funcional de un órgano inferido; no es un componente ECS.
/// Functional role of an inferred organ; not an ECS component.
#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Default, Reflect)]
pub enum OrganRole {
    #[default]
    Stem = 0,
    Root = 1,
    Core = 2,
    Leaf = 3,
    Petal = 4,
    Sensory = 5,
    Thorn = 6,
    Shell = 7,
    Fruit = 8,
    Bud = 9,
    Limb = 10,
    Fin = 11,
}

impl OrganRole {
    pub const COUNT: usize = OrganRole::Fin as usize + 1;
}

// ---------------------------------------------------------------------------
// GeometryPrimitive
// ---------------------------------------------------------------------------

/// Clase geométrica base para sintetizar órganos.
/// Base geometry class for organ synthesis.
#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Default, Reflect)]
pub enum GeometryPrimitive {
    #[default]
    Tube = 0,
    FlatSurface = 1,
    PetalFan = 2,
    Bulb = 3,
}

/// Tabla de mapeo `OrganRole -> GeometryPrimitive` sin branching en hot path.
/// Lookup table: `OrganRole -> GeometryPrimitive`, no branching in hot path.
pub const ORGAN_ROLE_PRIMITIVE: [GeometryPrimitive; OrganRole::COUNT] = [
    GeometryPrimitive::Tube,        // Stem
    GeometryPrimitive::Tube,        // Root
    GeometryPrimitive::Tube,        // Core
    GeometryPrimitive::FlatSurface, // Leaf
    GeometryPrimitive::PetalFan,    // Petal
    GeometryPrimitive::Bulb,        // Sensory
    GeometryPrimitive::Tube,        // Thorn
    GeometryPrimitive::FlatSurface, // Shell
    GeometryPrimitive::Bulb,        // Fruit
    GeometryPrimitive::Bulb,        // Bud
    GeometryPrimitive::Tube,        // Limb
    GeometryPrimitive::FlatSurface, // Fin
];

const _: () = assert!(ORGAN_ROLE_PRIMITIVE.len() == OrganRole::COUNT);

impl OrganRole {
    /// Primitiva geométrica asociada a este rol de órgano.
    /// Geometry primitive associated with this organ role.
    #[inline]
    pub const fn primitive(self) -> GeometryPrimitive {
        ORGAN_ROLE_PRIMITIVE[self as usize]
    }
}

// ---------------------------------------------------------------------------
// LifecycleStage
// ---------------------------------------------------------------------------

/// Fase funcional del ciclo de vida inferido.
/// Functional phase of the inferred lifecycle.
#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Default, Reflect)]
pub enum LifecycleStage {
    #[default]
    Dormant = 0,
    Emerging = 1,
    Growing = 2,
    Mature = 3,
    Reproductive = 4,
    Declining = 5,
}

// ---------------------------------------------------------------------------
// TrophicClass
// ---------------------------------------------------------------------------

/// Clase trófica para transformaciones energéticas data-driven.
/// Trophic class for data-driven energy transformations.
#[repr(u8)]
#[derive(Reflect, Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum TrophicClass {
    #[default]
    PrimaryProducer = 0,
    Herbivore = 1,
    Omnivore = 2,
    Carnivore = 3,
    Detritivore = 4,
}

// ---------------------------------------------------------------------------
// MAX_ORGANS_PER_ENTITY
// ---------------------------------------------------------------------------

/// Cantidad máxima de órganos inferidos por entidad en un tick.
/// Maximum inferred organs per entity per tick.
pub const MAX_ORGANS_PER_ENTITY: usize = 16;

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn matter_state_default_is_solid() {
        assert_eq!(MatterState::default(), MatterState::Solid);
    }

    #[test]
    fn organ_role_primitive_is_const() {
        const P: GeometryPrimitive = OrganRole::Stem.primitive();
        assert_eq!(P, GeometryPrimitive::Tube);
    }

    #[test]
    fn organ_role_count_matches_variants() {
        assert_eq!(OrganRole::COUNT, 12);
    }

    #[test]
    fn trophic_class_exhaustive() {
        let classes = [
            TrophicClass::PrimaryProducer,
            TrophicClass::Herbivore,
            TrophicClass::Omnivore,
            TrophicClass::Carnivore,
            TrophicClass::Detritivore,
        ];
        assert_eq!(classes.len(), 5);
    }

    #[test]
    fn lifecycle_stage_ordering_matches_repr() {
        assert!((LifecycleStage::Dormant as u8) < (LifecycleStage::Mature as u8));
        assert!((LifecycleStage::Mature as u8) < (LifecycleStage::Declining as u8));
    }

    #[test]
    fn max_organs_consistent_with_organ_role_count() {
        assert!(MAX_ORGANS_PER_ENTITY >= OrganRole::COUNT);
    }
}
