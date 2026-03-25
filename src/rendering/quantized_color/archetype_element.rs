//! Mapeo determinista `WorldArchetype` → `ElementId` de paleta (elemento dominante).

use crate::blueprint::ElementId;
use crate::worldgen::archetypes::WorldArchetype;
use WorldArchetype::*;

/// Elemento cuya paleta SSOT alimenta el material (no confundir con forma/topología).
#[inline]
pub fn element_id_for_world_archetype(archetype: WorldArchetype) -> ElementId {
    match archetype {
        Void | ShadowFog | CorruptedEarth => ElementId::from_name("Umbra"),

        UmbraSolid | UmbraLiquid | UmbraGas | UmbraPlasma => ElementId::from_name("Umbra"),

        TerraSolid | TerraLiquid | TerraGas | TerraPlasma | Mountain | Swamp | VolcanicBeach
        | Rockface | Hillside | Ravine | Cliff | Shoreline => ElementId::from_name("Terra"),

        AquaSolid | AquaLiquid | AquaGas | AquaPlasma | DeepWater | River | Lake | GlacierPeak
        | Steam | SteamVent | MistField | MistValley | Oasis | ObsidianRift => {
            ElementId::from_name("Aqua")
        }

        IgnisSolid | IgnisLiquid | IgnisGas | IgnisPlasma | LavaFlow | LavaRiver | VolcanicVent
        | HeatDistortion => ElementId::from_name("Ignis"),

        VentusSolid | VentusLiquid | VentusGas | VentusPlasma | StormZone | WindsweptPlateau => {
            ElementId::from_name("Ventus")
        }

        LuxSolid | LuxLiquid | LuxGas | LuxPlasma | DualityField | Tundra => {
            ElementId::from_name("Lux")
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ignis_solid_maps_ignis() {
        assert_eq!(
            element_id_for_world_archetype(IgnisSolid).raw(),
            ElementId::from_name("Ignis").raw()
        );
    }
}
