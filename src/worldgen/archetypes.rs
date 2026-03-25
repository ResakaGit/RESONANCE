use bevy::prelude::*;

use crate::layers::MatterState;

/// Clasificación discreta de densidad para reglas de materialización/visual.
#[derive(Clone, Copy, Debug, Reflect, PartialEq, Eq)]
pub enum DensityClass {
    Low,
    Medium,
    High,
}

/// Arquetipos visuales/materiales emergentes del campo energético.
///
/// Enum plano sin data asociada para mantener serialización y matching simples.
#[derive(Clone, Copy, Debug, Reflect, PartialEq, Eq, Hash)]
pub enum WorldArchetype {
    Void,
    UmbraSolid,
    UmbraLiquid,
    UmbraGas,
    UmbraPlasma,
    TerraSolid,
    TerraLiquid,
    TerraGas,
    TerraPlasma,
    AquaSolid,
    AquaLiquid,
    AquaGas,
    AquaPlasma,
    IgnisSolid,
    IgnisLiquid,
    IgnisGas,
    IgnisPlasma,
    VentusSolid,
    VentusLiquid,
    VentusGas,
    VentusPlasma,
    LuxSolid,
    LuxLiquid,
    LuxGas,
    LuxPlasma,
    DeepWater,
    LavaFlow,
    Mountain,
    Swamp,
    MistField,
    ShadowFog,
    Steam,
    StormZone,
    VolcanicBeach,
    Oasis,
    Tundra,
    DualityField,
    /// Frontera fase sólido–líquido (Eco-Boundaries / V7).
    Shoreline,
    /// Frontera líquido–gas (vapor / transición).
    SteamVent,
    /// Frontera elemento Ignis–Aqua (zona volcánica + subacuática).
    ObsidianRift,
    /// Frontera Terra–Umbra (superficie/subterráneo vs vacío).
    CorruptedEarth,
    /// Shock térmico entre zonas.
    HeatDistortion,
    /// Gradiente de densidad alto–bajo entre zonas.
    Cliff,
    River,
    Lake,
    GlacierPeak,
    LavaRiver,
    VolcanicVent,
    MistValley,
    Rockface,
    WindsweptPlateau,
    Hillside,
    Ravine,
}

/// Cobertura mínima obligatoria: 6 elementos puros x 4 estados = 24.
pub const PURE_ELEMENT_STATE_ARCHETYPES: [WorldArchetype; 24] = [
    WorldArchetype::UmbraSolid,
    WorldArchetype::UmbraLiquid,
    WorldArchetype::UmbraGas,
    WorldArchetype::UmbraPlasma,
    WorldArchetype::TerraSolid,
    WorldArchetype::TerraLiquid,
    WorldArchetype::TerraGas,
    WorldArchetype::TerraPlasma,
    WorldArchetype::AquaSolid,
    WorldArchetype::AquaLiquid,
    WorldArchetype::AquaGas,
    WorldArchetype::AquaPlasma,
    WorldArchetype::IgnisSolid,
    WorldArchetype::IgnisLiquid,
    WorldArchetype::IgnisGas,
    WorldArchetype::IgnisPlasma,
    WorldArchetype::VentusSolid,
    WorldArchetype::VentusLiquid,
    WorldArchetype::VentusGas,
    WorldArchetype::VentusPlasma,
    WorldArchetype::LuxSolid,
    WorldArchetype::LuxLiquid,
    WorldArchetype::LuxGas,
    WorldArchetype::LuxPlasma,
];

#[derive(Clone, Copy, Debug, Reflect, PartialEq, Eq, Hash)]
pub enum ElementBand {
    Umbra,
    Terra,
    Aqua,
    Ignis,
    Ventus,
    Lux,
    Unknown,
}

fn normalized_pair(a: ElementBand, b: ElementBand) -> (ElementBand, ElementBand) {
    use ElementBand::{Aqua, Ignis, Lux, Terra, Umbra, Unknown, Ventus};
    fn rank(band: ElementBand) -> u8 {
        match band {
            Umbra => 0,
            Terra => 1,
            Aqua => 2,
            Ignis => 3,
            Ventus => 4,
            Lux => 5,
            Unknown => 6,
        }
    }

    if rank(a) <= rank(b) { (a, b) } else { (b, a) }
}

pub fn archetype_from_signature(
    band: ElementBand,
    state: MatterState,
    density_class: DensityClass,
) -> WorldArchetype {
    match (band, state, density_class) {
        (ElementBand::Umbra, MatterState::Solid, DensityClass::Low) => WorldArchetype::ShadowFog,
        (ElementBand::Umbra, MatterState::Solid, _) => WorldArchetype::UmbraSolid,
        (ElementBand::Umbra, MatterState::Liquid, _) => WorldArchetype::UmbraLiquid,
        (ElementBand::Umbra, MatterState::Gas, _) => WorldArchetype::UmbraGas,
        (ElementBand::Umbra, MatterState::Plasma, _) => WorldArchetype::UmbraPlasma,

        (ElementBand::Terra, MatterState::Solid, DensityClass::High) => WorldArchetype::Mountain,
        (ElementBand::Terra, MatterState::Solid, _) => WorldArchetype::TerraSolid,
        (ElementBand::Terra, MatterState::Liquid, DensityClass::Low) => WorldArchetype::Swamp,
        (ElementBand::Terra, MatterState::Liquid, _) => WorldArchetype::TerraLiquid,
        (ElementBand::Terra, MatterState::Gas, _) => WorldArchetype::TerraGas,
        (ElementBand::Terra, MatterState::Plasma, _) => WorldArchetype::TerraPlasma,

        (ElementBand::Aqua, MatterState::Solid, _) => WorldArchetype::AquaSolid,
        (ElementBand::Aqua, MatterState::Liquid, DensityClass::High) => WorldArchetype::DeepWater,
        (ElementBand::Aqua, MatterState::Liquid, _) => WorldArchetype::AquaLiquid,
        (ElementBand::Aqua, MatterState::Gas, _) => WorldArchetype::MistField,
        (ElementBand::Aqua, MatterState::Plasma, _) => WorldArchetype::AquaPlasma,

        (ElementBand::Ignis, MatterState::Solid, _) => WorldArchetype::IgnisSolid,
        (ElementBand::Ignis, MatterState::Liquid, DensityClass::High) => WorldArchetype::LavaFlow,
        (ElementBand::Ignis, MatterState::Liquid, _) => WorldArchetype::IgnisLiquid,
        (ElementBand::Ignis, MatterState::Gas, _) => WorldArchetype::IgnisGas,
        (ElementBand::Ignis, MatterState::Plasma, _) => WorldArchetype::IgnisPlasma,

        (ElementBand::Ventus, MatterState::Solid, _) => WorldArchetype::VentusSolid,
        (ElementBand::Ventus, MatterState::Liquid, _) => WorldArchetype::VentusLiquid,
        (ElementBand::Ventus, MatterState::Gas, _) => WorldArchetype::VentusGas,
        (ElementBand::Ventus, MatterState::Plasma, _) => WorldArchetype::VentusPlasma,

        (ElementBand::Lux, MatterState::Solid, _) => WorldArchetype::LuxSolid,
        (ElementBand::Lux, MatterState::Liquid, _) => WorldArchetype::LuxLiquid,
        (ElementBand::Lux, MatterState::Gas, _) => WorldArchetype::LuxGas,
        (ElementBand::Lux, MatterState::Plasma, _) => WorldArchetype::LuxPlasma,

        (ElementBand::Unknown, _, _) => WorldArchetype::Void,
    }
}

/// Resuelve arquetipos compuestos para pares de bandas dominantes.
///
/// - Si no hay regla explícita, vuelve al arquetipo del dominante.
/// - Interferencia destructiva fuerte (< -0.5) prioriza variantes inestables.
pub fn compound_archetype_for_pair(
    primary: ElementBand,
    secondary: ElementBand,
    interference: f32,
    dominant_fallback: WorldArchetype,
) -> WorldArchetype {
    use ElementBand::{Aqua, Ignis, Lux, Terra, Umbra, Ventus};
    use WorldArchetype::{
        DualityField, MistField, Oasis, Steam, StormZone, Swamp, Tundra, VolcanicBeach,
    };

    match normalized_pair(primary, secondary) {
        (Terra, Aqua) => Swamp,
        (Aqua, Ignis) => {
            if interference < -0.5 {
                Steam
            } else {
                Oasis
            }
        }
        (Terra, Ignis) => VolcanicBeach,
        (Aqua, Ventus) => {
            if interference < -0.5 {
                StormZone
            } else {
                MistField
            }
        }
        (Umbra, Lux) => {
            if interference < -0.5 {
                Tundra
            } else {
                DualityField
            }
        }
        _ => dominant_fallback,
    }
}

#[cfg(test)]
mod tests {
    use super::{
        DensityClass, ElementBand, PURE_ELEMENT_STATE_ARCHETYPES, WorldArchetype,
        archetype_from_signature, compound_archetype_for_pair,
    };
    use crate::layers::MatterState;
    use std::collections::HashSet;

    #[test]
    fn world_archetype_covers_minimum_pure_element_states() {
        let unique = PURE_ELEMENT_STATE_ARCHETYPES
            .iter()
            .copied()
            .collect::<HashSet<WorldArchetype>>();
        let expected = HashSet::from([
            WorldArchetype::UmbraSolid,
            WorldArchetype::UmbraLiquid,
            WorldArchetype::UmbraGas,
            WorldArchetype::UmbraPlasma,
            WorldArchetype::TerraSolid,
            WorldArchetype::TerraLiquid,
            WorldArchetype::TerraGas,
            WorldArchetype::TerraPlasma,
            WorldArchetype::AquaSolid,
            WorldArchetype::AquaLiquid,
            WorldArchetype::AquaGas,
            WorldArchetype::AquaPlasma,
            WorldArchetype::IgnisSolid,
            WorldArchetype::IgnisLiquid,
            WorldArchetype::IgnisGas,
            WorldArchetype::IgnisPlasma,
            WorldArchetype::VentusSolid,
            WorldArchetype::VentusLiquid,
            WorldArchetype::VentusGas,
            WorldArchetype::VentusPlasma,
            WorldArchetype::LuxSolid,
            WorldArchetype::LuxLiquid,
            WorldArchetype::LuxGas,
            WorldArchetype::LuxPlasma,
        ]);

        assert_eq!(
            unique, expected,
            "must cover exactly 6 pure elements x 4 states"
        );
    }

    #[test]
    fn archetype_from_signature_maps_special_cases_correctly() {
        assert_eq!(
            archetype_from_signature(ElementBand::Terra, MatterState::Solid, DensityClass::High),
            WorldArchetype::Mountain
        );
        assert_eq!(
            archetype_from_signature(ElementBand::Aqua, MatterState::Liquid, DensityClass::High),
            WorldArchetype::DeepWater
        );
        assert_eq!(
            archetype_from_signature(ElementBand::Ignis, MatterState::Liquid, DensityClass::High),
            WorldArchetype::LavaFlow
        );
        assert_eq!(
            archetype_from_signature(ElementBand::Unknown, MatterState::Gas, DensityClass::Medium),
            WorldArchetype::Void
        );
    }

    #[test]
    fn compound_table_covers_five_key_pairs() {
        assert_eq!(
            compound_archetype_for_pair(
                ElementBand::Terra,
                ElementBand::Aqua,
                0.2,
                WorldArchetype::TerraLiquid
            ),
            WorldArchetype::Swamp
        );
        assert_eq!(
            compound_archetype_for_pair(
                ElementBand::Ignis,
                ElementBand::Aqua,
                -1.0,
                WorldArchetype::IgnisGas
            ),
            WorldArchetype::Steam
        );
        assert_eq!(
            compound_archetype_for_pair(
                ElementBand::Ignis,
                ElementBand::Terra,
                0.6,
                WorldArchetype::IgnisSolid
            ),
            WorldArchetype::VolcanicBeach
        );
        assert_eq!(
            compound_archetype_for_pair(
                ElementBand::Ventus,
                ElementBand::Aqua,
                -0.7,
                WorldArchetype::VentusGas
            ),
            WorldArchetype::StormZone
        );
        assert_eq!(
            compound_archetype_for_pair(
                ElementBand::Lux,
                ElementBand::Umbra,
                0.1,
                WorldArchetype::LuxPlasma
            ),
            WorldArchetype::DualityField
        );
    }
}
