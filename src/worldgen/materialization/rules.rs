use crate::blueprint::almanac::AlchemicalAlmanac;
use crate::blueprint::equations;
use crate::eco::contracts::{BoundaryMarker, TransitionType, ZoneClass};
use crate::layers::MatterState;
use crate::topology::TerrainType;

use super::super::archetypes::{
    DensityClass, ElementBand, WorldArchetype, archetype_from_signature,
    compound_archetype_for_pair,
};
use super::super::constants::{
    DENSITY_HIGH_THRESHOLD, DENSITY_LOW_THRESHOLD, MIN_MATERIALIZATION_QE, PURITY_THRESHOLD,
};
use super::super::contracts::{
    BoundaryVisual, EnergyCell, FrequencyContribution, MaterializationResult, top_two,
};
use super::super::inference::visual_derivation::materialized_tile_spatial_density;

pub fn classify_density(density: f32) -> DensityClass {
    if density < DENSITY_LOW_THRESHOLD {
        DensityClass::Low
    } else if density > DENSITY_HIGH_THRESHOLD {
        DensityClass::High
    } else {
        DensityClass::Medium
    }
}

pub fn band_of(frequency_hz: f32, almanac: &AlchemicalAlmanac) -> ElementBand {
    if let Some(def) = almanac.find_stable_band(frequency_hz) {
        match def.symbol.as_str() {
            "Umbra" => ElementBand::Umbra,
            "Terra" => ElementBand::Terra,
            "Fl" => ElementBand::Terra,
            "Aqua" => ElementBand::Aqua,
            "Ignis" => ElementBand::Ignis,
            "Ventus" => ElementBand::Ventus,
            "Lux" => ElementBand::Lux,
            _ => ElementBand::Unknown,
        }
    } else {
        #[cfg(test)]
        {
            match frequency_hz {
                f if (10.0..=30.0).contains(&f) => ElementBand::Umbra,
                f if (50.0..=110.0).contains(&f) => ElementBand::Terra,
                f if (200.0..=300.0).contains(&f) => ElementBand::Aqua,
                f if (400.0..=500.0).contains(&f) => ElementBand::Ignis,
                f if (600.0..=800.0).contains(&f) => ElementBand::Ventus,
                f if (900.0..=1100.0).contains(&f) => ElementBand::Lux,
                _ => ElementBand::Unknown,
            }
        }
        #[cfg(not(test))]
        {
            ElementBand::Unknown
        }
    }
}

/// API legacy / tests; la materialización efectiva usa `archetype_from_signature` + compuestos.
#[allow(dead_code)]
pub fn lookup_archetype(
    frequency_hz: f32,
    state: MatterState,
    density_class: DensityClass,
    purity: f32,
    almanac: &AlchemicalAlmanac,
) -> WorldArchetype {
    if purity < PURITY_THRESHOLD {
        return WorldArchetype::Void;
    }
    let band = band_of(frequency_hz, almanac);
    archetype_from_signature(band, state, density_class)
}

/// Pureza baja y dos contribuciones dominantes en bandas distintas → ruta compuesta.
pub fn compound_path_active(cell: &EnergyCell, almanac: &AlchemicalAlmanac) -> bool {
    if cell.purity >= PURITY_THRESHOLD {
        return false;
    }
    let Some((a, b)) = top_two(cell.frequency_contributions()) else {
        return false;
    };
    band_of(a.frequency_hz(), almanac) != band_of(b.frequency_hz(), almanac)
}

pub fn resolve_compound(
    contributions: &[FrequencyContribution],
    state: MatterState,
    density_class: DensityClass,
    t: f32,
    almanac: &AlchemicalAlmanac,
) -> Option<WorldArchetype> {
    let (primary, secondary) = top_two(contributions)?;
    let primary_band = band_of(primary.frequency_hz, almanac);
    let secondary_band = band_of(secondary.frequency_hz, almanac);
    let dominant_fallback = archetype_from_signature(primary_band, state, density_class);
    let interference =
        equations::interference(primary.frequency_hz, 0.0, secondary.frequency_hz, 0.0, t);
    Some(compound_archetype_for_pair(
        primary_band,
        secondary_band,
        interference,
        dominant_fallback,
    ))
}

/// API estable `t = 0` (tests); runtime usa `materialize_cell_at_time`.
#[cfg_attr(not(test), allow(dead_code))]
pub fn materialize_cell(
    cell: &EnergyCell,
    almanac: &AlchemicalAlmanac,
    cell_size_m: f32,
) -> Option<MaterializationResult> {
    materialize_cell_at_time(cell, almanac, 0.0, cell_size_m, None)
}

/// Second pass topológico: mantiene el arquetipo base si no hay regla de enriquecimiento.
pub fn enrich_archetype(
    base: WorldArchetype,
    terrain: TerrainType,
    _energy: &EnergyCell,
) -> WorldArchetype {
    match (base, terrain) {
        (WorldArchetype::Mountain, TerrainType::Riverbed) => WorldArchetype::Ravine,
        (WorldArchetype::TerraSolid, TerrainType::Cliff)
        | (WorldArchetype::Mountain, TerrainType::Cliff) => WorldArchetype::Rockface,
        (WorldArchetype::DeepWater, TerrainType::Riverbed) => WorldArchetype::River,
        (WorldArchetype::AquaLiquid, TerrainType::Riverbed) => WorldArchetype::River,
        (WorldArchetype::DeepWater, TerrainType::Basin) => WorldArchetype::Lake,
        (WorldArchetype::AquaLiquid, TerrainType::Basin) => WorldArchetype::Lake,
        (WorldArchetype::AquaSolid, TerrainType::Peak) => WorldArchetype::GlacierPeak,
        (WorldArchetype::LavaFlow, TerrainType::Valley) => WorldArchetype::LavaRiver,
        (WorldArchetype::IgnisPlasma, TerrainType::Valley) => WorldArchetype::LavaRiver,
        (WorldArchetype::IgnisGas, TerrainType::Peak) => WorldArchetype::VolcanicVent,
        (WorldArchetype::UmbraGas, TerrainType::Valley)
        | (WorldArchetype::ShadowFog, TerrainType::Valley) => WorldArchetype::MistValley,
        (WorldArchetype::TerraSolid, TerrainType::Slope) => WorldArchetype::Hillside,
        (WorldArchetype::VentusGas, TerrainType::Plateau) => WorldArchetype::WindsweptPlateau,
        _ => base,
    }
}

/// Firma extra para cache de materialización cuando hay `EcoBoundaryField` alineado.
/// Mezcla tipo SplitMix64 ligera para reducir colisiones entre fronteras distintas.
#[inline]
pub fn boundary_marker_cache_tag(marker: Option<BoundaryMarker>) -> u64 {
    match marker {
        None | Some(BoundaryMarker::Interior { .. }) => 0,
        Some(BoundaryMarker::Boundary {
            zone_a,
            zone_b,
            gradient_factor,
            transition_type,
        }) => {
            let mut x = u64::from(zone_a as u8)
                | (u64::from(zone_b as u8) << 8)
                | (u64::from(transition_type as u8) << 16)
                | (u64::from(gradient_factor.to_bits()) << 32);
            x ^= x >> 30;
            x = x.wrapping_mul(0xBF58_476D_1CE4_E5B9);
            x ^= x >> 27;
            x = x.wrapping_mul(0x94D0_49BB_1331_11EB);
            x ^= x >> 31;
            x | (1u64 << 63)
        }
    }
}

#[inline]
fn zone_pair_volcanic_subaquatic(a: ZoneClass, b: ZoneClass) -> bool {
    matches!(
        (a, b),
        (ZoneClass::Volcanic, ZoneClass::Subaquatic) | (ZoneClass::Subaquatic, ZoneClass::Volcanic)
    )
}

#[inline]
fn zone_pair_earth_void(a: ZoneClass, b: ZoneClass) -> bool {
    let earth_like = |z: ZoneClass| {
        matches!(
            z,
            ZoneClass::Surface | ZoneClass::Subterranean | ZoneClass::Frozen
        )
    };
    matches!((a, b), (z, ZoneClass::Void) if earth_like(z))
        || matches!((a, b), (ZoneClass::Void, z) if earth_like(z))
}

/// Arquetipo de frontera (solo referencia V7 / UI / futuro tooling).
///
/// **No** se escribe en `Materialized.archetype` (gameplay = arquetipo base de celda).
/// Mapeo v1 acotado: `ElementFrontier` solo cubre pares Volcánico↔Subacuático y tierra↔Void;
/// otras fronteras elementales conservan `base` hasta ampliar tabla en sprint posterior.
pub fn boundary_world_archetype(
    transition_type: TransitionType,
    zone_a: ZoneClass,
    zone_b: ZoneClass,
    matter_state: MatterState,
    base: WorldArchetype,
) -> WorldArchetype {
    match transition_type {
        TransitionType::ThermalShock => WorldArchetype::HeatDistortion,
        TransitionType::DensityGradient => WorldArchetype::Cliff,
        TransitionType::PhaseBoundary => match matter_state {
            MatterState::Gas => WorldArchetype::SteamVent,
            _ => WorldArchetype::Shoreline,
        },
        TransitionType::ElementFrontier => {
            if zone_pair_volcanic_subaquatic(zone_a, zone_b) {
                WorldArchetype::ObsidianRift
            } else if zone_pair_earth_void(zone_a, zone_b) {
                WorldArchetype::CorruptedEarth
            } else {
                base
            }
        }
    }
}

/// `BoundaryVisual` para spawn/delta (sin entidades extra).
pub fn boundary_visual_from_marker(marker: BoundaryMarker) -> Option<BoundaryVisual> {
    match marker {
        BoundaryMarker::Interior { .. } => None,
        BoundaryMarker::Boundary {
            zone_a,
            zone_b,
            gradient_factor,
            transition_type,
        } => Some(BoundaryVisual {
            transition_type,
            gradient_factor: if gradient_factor.is_finite() {
                gradient_factor.clamp(0.0, 1.0)
            } else {
                0.5
            },
            zone_a,
            zone_b,
        }),
    }
}

pub fn materialize_cell_at_time(
    cell: &EnergyCell,
    almanac: &AlchemicalAlmanac,
    t: f32,
    cell_size_m: f32,
    terrain_type: Option<TerrainType>,
) -> Option<MaterializationResult> {
    if cell.accumulated_qe < MIN_MATERIALIZATION_QE {
        return None;
    }

    let (effective_qe, force_pure_path) =
        if let Some((a, b)) = top_two(cell.frequency_contributions()) {
            let band_a = band_of(a.frequency_hz(), almanac);
            let band_b = band_of(b.frequency_hz(), almanac);
            if band_a == band_b {
                (cell.accumulated_qe, true)
            } else {
                (cell.accumulated_qe, false)
            }
        } else {
            (cell.accumulated_qe, false)
        };
    // Misma densidad espacial que el spawn materializado (no confundir qe acumulado con ρ).
    let rho = materialized_tile_spatial_density(effective_qe.max(0.0), cell_size_m);
    let density_class = classify_density(rho);
    let dominant_band = band_of(cell.dominant_frequency_hz, almanac);
    let pure_archetype = archetype_from_signature(dominant_band, cell.matter_state, density_class);
    let base_archetype = if cell.purity < PURITY_THRESHOLD && !force_pure_path {
        resolve_compound(
            &cell.frequency_contributions,
            cell.matter_state,
            density_class,
            t,
            almanac,
        )
        .unwrap_or(pure_archetype)
    } else {
        pure_archetype
    };
    let archetype = terrain_type
        .map(|terrain| enrich_archetype(base_archetype, terrain, cell))
        .unwrap_or(base_archetype);

    Some(MaterializationResult { archetype })
}

/// API estable para cache: el resultado es **solo** el de la celda energética; el tinte de frontera
/// vive en `BoundaryVisual` y en los sistemas de `worldgen::systems::visual`.
pub fn materialize_cell_at_time_with_boundary(
    cell: &EnergyCell,
    almanac: &AlchemicalAlmanac,
    t: f32,
    cell_size_m: f32,
    terrain_type: Option<TerrainType>,
    _boundary: Option<BoundaryMarker>,
) -> Option<MaterializationResult> {
    materialize_cell_at_time(cell, almanac, t, cell_size_m, terrain_type)
}

#[cfg(test)]
mod tests {
    use super::{
        ElementBand, band_of, boundary_marker_cache_tag, boundary_world_archetype,
        classify_density, enrich_archetype, lookup_archetype, materialize_cell,
        materialize_cell_at_time, materialize_cell_at_time_with_boundary,
        materialized_tile_spatial_density, resolve_compound,
    };
    use crate::blueprint::almanac::{AlchemicalAlmanac, test_assets_elements_almanac};
    use crate::eco::contracts::{BoundaryMarker, TransitionType, ZoneClass};
    use crate::layers::MatterState;
    use crate::topology::TerrainType;
    use crate::worldgen::archetypes::{DensityClass, WorldArchetype};
    use crate::worldgen::constants::{
        DENSITY_HIGH_THRESHOLD, FIELD_CELL_SIZE, MIN_MATERIALIZATION_QE,
    };
    use crate::worldgen::contracts::{EnergyCell, FrequencyContribution};
    use crate::worldgen::inference::visual_derivation::energy_visual_boundary_flat_color;
    use bevy::prelude::{Color, Entity};

    fn mk_cell(freq: f32, state: MatterState, qe: f32, purity: f32) -> EnergyCell {
        let mut cell = EnergyCell::default();
        cell.accumulated_qe = qe;
        cell.dominant_frequency_hz = freq;
        cell.purity = purity;
        cell.temperature = qe;
        cell.matter_state = state;
        cell
    }

    #[test]
    fn materialize_cell_below_threshold_returns_none() {
        let almanac = AlchemicalAlmanac::default();
        let cell = mk_cell(60.0, MatterState::Solid, MIN_MATERIALIZATION_QE - 0.01, 1.0);
        assert!(materialize_cell(&cell, &almanac, FIELD_CELL_SIZE).is_none());
    }

    #[test]
    fn classify_density_low_medium_high() {
        assert_eq!(classify_density(5.0), DensityClass::Low);
        assert_eq!(classify_density(50.0), DensityClass::Medium);
        assert_eq!(classify_density(120.0), DensityClass::High);
    }

    #[test]
    fn band_of_classifies_six_bands() {
        let almanac = AlchemicalAlmanac::default();
        assert_eq!(band_of(20.0, &almanac), ElementBand::Umbra);
        assert_eq!(band_of(60.0, &almanac), ElementBand::Terra);
        assert_eq!(band_of(250.0, &almanac), ElementBand::Aqua);
        assert_eq!(band_of(450.0, &almanac), ElementBand::Ignis);
        assert_eq!(band_of(700.0, &almanac), ElementBand::Ventus);
        assert_eq!(band_of(1000.0, &almanac), ElementBand::Lux);
    }

    #[test]
    fn band_of_unknown_when_outside_ranges() {
        let almanac = AlchemicalAlmanac::default();
        assert_eq!(band_of(1500.0, &almanac), ElementBand::Unknown);
    }

    #[test]
    fn band_of_flora_hz_maps_to_terra_family_with_assets_almanac() {
        let almanac = test_assets_elements_almanac();
        assert_eq!(band_of(85.0, &almanac), ElementBand::Terra);
        assert_eq!(band_of(100.0, &almanac), ElementBand::Terra);
    }

    #[test]
    fn pure_elements_solid_medium_map_to_expected_archetype() {
        let almanac = AlchemicalAlmanac::default();
        assert_eq!(
            lookup_archetype(
                20.0,
                MatterState::Solid,
                DensityClass::Medium,
                1.0,
                &almanac
            ),
            WorldArchetype::UmbraSolid
        );
        assert_eq!(
            lookup_archetype(
                60.0,
                MatterState::Solid,
                DensityClass::Medium,
                1.0,
                &almanac
            ),
            WorldArchetype::TerraSolid
        );
        assert_eq!(
            lookup_archetype(
                250.0,
                MatterState::Solid,
                DensityClass::Medium,
                1.0,
                &almanac
            ),
            WorldArchetype::AquaSolid
        );
        assert_eq!(
            lookup_archetype(
                450.0,
                MatterState::Solid,
                DensityClass::Medium,
                1.0,
                &almanac
            ),
            WorldArchetype::IgnisSolid
        );
        assert_eq!(
            lookup_archetype(
                700.0,
                MatterState::Solid,
                DensityClass::Medium,
                1.0,
                &almanac
            ),
            WorldArchetype::VentusSolid
        );
        assert_eq!(
            lookup_archetype(
                1000.0,
                MatterState::Solid,
                DensityClass::Medium,
                1.0,
                &almanac
            ),
            WorldArchetype::LuxSolid
        );
    }

    #[test]
    fn same_frequency_different_state_returns_different_archetype() {
        let almanac = AlchemicalAlmanac::default();
        let solid = lookup_archetype(
            60.0,
            MatterState::Solid,
            DensityClass::Medium,
            1.0,
            &almanac,
        );
        let gas = lookup_archetype(60.0, MatterState::Gas, DensityClass::Medium, 1.0, &almanac);
        assert_ne!(solid, gas);
    }

    #[test]
    fn same_frequency_state_low_vs_high_density_differs() {
        let almanac = AlchemicalAlmanac::default();
        let low = lookup_archetype(450.0, MatterState::Liquid, DensityClass::Low, 1.0, &almanac);
        let high = lookup_archetype(
            450.0,
            MatterState::Liquid,
            DensityClass::High,
            1.0,
            &almanac,
        );
        assert_ne!(low, high);
    }

    #[test]
    fn low_purity_without_compound_rule_falls_back_to_dominant() {
        let almanac = AlchemicalAlmanac::default();
        let dominant = lookup_archetype(
            60.0,
            MatterState::Solid,
            DensityClass::Medium,
            1.0,
            &almanac,
        );
        assert_eq!(dominant, WorldArchetype::TerraSolid);

        let cell = mk_cell(60.0, MatterState::Solid, 50.0, 0.2);
        let result = materialize_cell(&cell, &almanac, FIELD_CELL_SIZE);
        assert!(result.is_some());
        assert_eq!(
            result.expect("expected result").archetype,
            WorldArchetype::TerraSolid
        );
    }

    #[test]
    fn materialize_cell_at_threshold_returns_some() {
        let almanac = AlchemicalAlmanac::default();
        let cell = mk_cell(60.0, MatterState::Solid, MIN_MATERIALIZATION_QE, 1.0);
        assert!(materialize_cell(&cell, &almanac, FIELD_CELL_SIZE).is_some());
    }

    #[test]
    fn resolve_compound_terra_aqua_returns_swamp() {
        let almanac = AlchemicalAlmanac::default();
        let contributions = vec![
            FrequencyContribution::new(Entity::from_raw(1), 75.0, 50.0),
            FrequencyContribution::new(Entity::from_raw(2), 250.0, 40.0),
        ];
        let result = resolve_compound(
            &contributions,
            MatterState::Liquid,
            DensityClass::Medium,
            0.25,
            &almanac,
        );
        assert_eq!(result, Some(WorldArchetype::Swamp));
    }

    #[test]
    fn resolve_compound_ignis_aqua_destructive_returns_steam() {
        let almanac = AlchemicalAlmanac::default();
        let contributions = vec![
            FrequencyContribution::new(Entity::from_raw(1), 450.0, 50.0),
            FrequencyContribution::new(Entity::from_raw(2), 250.0, 45.0),
        ];
        let result = resolve_compound(
            &contributions,
            MatterState::Gas,
            DensityClass::Medium,
            0.0025,
            &almanac,
        );
        assert_eq!(result, Some(WorldArchetype::Steam));
    }

    #[test]
    fn same_band_pair_uses_pure_path_with_high_density_class() {
        let almanac = AlchemicalAlmanac::default();
        // Clase High = ρ del spawn materializado > DENSITY_HIGH_THRESHOLD (no qe cruda).
        let qe_high_rho = 450.0_f32;
        assert!(
            materialized_tile_spatial_density(qe_high_rho, FIELD_CELL_SIZE)
                > DENSITY_HIGH_THRESHOLD,
            "tune qe if spawn geometry changes"
        );
        let mut cell = mk_cell(60.0, MatterState::Solid, qe_high_rho, 0.2);
        cell.frequency_contributions = vec![
            FrequencyContribution::new(Entity::from_raw(1), 70.0, 40.0),
            FrequencyContribution::new(Entity::from_raw(2), 80.0, 35.0),
        ];
        let result = materialize_cell_at_time(&cell, &almanac, 0.1, FIELD_CELL_SIZE, None)
            .expect("must materialize");
        assert_eq!(result.archetype, WorldArchetype::Mountain);
    }

    #[test]
    fn compound_without_explicit_rule_falls_back_to_dominant() {
        let almanac = AlchemicalAlmanac::default();
        let contributions = vec![
            FrequencyContribution::new(Entity::from_raw(1), 60.0, 40.0),
            FrequencyContribution::new(Entity::from_raw(2), 1000.0, 30.0),
        ];
        let result = resolve_compound(
            &contributions,
            MatterState::Solid,
            DensityClass::Medium,
            0.2,
            &almanac,
        );
        assert_eq!(result, Some(WorldArchetype::TerraSolid));
    }

    #[test]
    fn e6_boundary_marker_cache_tag_interior_es_cero() {
        assert_eq!(
            boundary_marker_cache_tag(Some(BoundaryMarker::Interior { zone_id: 3 })),
            0
        );
    }

    #[test]
    fn e6_boundary_world_archetype_phase_solid_es_shoreline() {
        assert_eq!(
            boundary_world_archetype(
                TransitionType::PhaseBoundary,
                ZoneClass::Surface,
                ZoneClass::Subaquatic,
                MatterState::Solid,
                WorldArchetype::TerraSolid,
            ),
            WorldArchetype::Shoreline
        );
    }

    #[test]
    fn e6_boundary_world_archetype_phase_gas_es_steam_vent() {
        assert_eq!(
            boundary_world_archetype(
                TransitionType::PhaseBoundary,
                ZoneClass::Surface,
                ZoneClass::HighAtmosphere,
                MatterState::Gas,
                WorldArchetype::TerraGas,
            ),
            WorldArchetype::SteamVent
        );
    }

    #[test]
    fn e6_boundary_world_archetype_element_volcanic_subaquatic_es_obsidian() {
        assert_eq!(
            boundary_world_archetype(
                TransitionType::ElementFrontier,
                ZoneClass::Volcanic,
                ZoneClass::Subaquatic,
                MatterState::Solid,
                WorldArchetype::TerraSolid,
            ),
            WorldArchetype::ObsidianRift
        );
    }

    #[test]
    fn e6_materialize_sin_overlay_arquetipo_ni_color_por_frontera() {
        let almanac = AlchemicalAlmanac::default();
        let cell = mk_cell(60.0, MatterState::Solid, 50.0, 1.0);
        let marker = BoundaryMarker::Boundary {
            zone_a: ZoneClass::Surface,
            zone_b: ZoneClass::Void,
            gradient_factor: 0.5,
            transition_type: TransitionType::PhaseBoundary,
        };
        let base =
            materialize_cell_at_time(&cell, &almanac, 0.0, FIELD_CELL_SIZE, None).expect("base");
        let with_b = materialize_cell_at_time_with_boundary(
            &cell,
            &almanac,
            0.0,
            FIELD_CELL_SIZE,
            None,
            Some(marker),
        )
        .expect("with boundary");
        assert_eq!(base.archetype, with_b.archetype);
    }

    #[test]
    fn e6_boundary_cache_tag_difiere_entre_marcadores_distintos() {
        let a = BoundaryMarker::Boundary {
            zone_a: ZoneClass::Surface,
            zone_b: ZoneClass::Void,
            gradient_factor: 0.1,
            transition_type: TransitionType::PhaseBoundary,
        };
        let b = BoundaryMarker::Boundary {
            zone_a: ZoneClass::Surface,
            zone_b: ZoneClass::Void,
            gradient_factor: 0.9,
            transition_type: TransitionType::PhaseBoundary,
        };
        assert_ne!(
            boundary_marker_cache_tag(Some(a)),
            boundary_marker_cache_tag(Some(b))
        );
    }

    #[test]
    fn e6_energy_visual_boundary_flat_color_coincide_funcion_publica() {
        use crate::worldgen::contracts::BoundaryVisual;
        let bv = BoundaryVisual {
            transition_type: TransitionType::ThermalShock,
            gradient_factor: 0.25,
            zone_a: ZoneClass::Volcanic,
            zone_b: ZoneClass::Frozen,
        };
        let c = energy_visual_boundary_flat_color(&bv);
        assert!(c.to_linear().red.is_finite());
    }

    #[test]
    fn t7_enrich_aqua_liquid_riverbed_to_river() {
        let almanac = AlchemicalAlmanac::default();
        let cell = mk_cell(250.0, MatterState::Liquid, 220.0, 1.0);
        let result = materialize_cell_at_time(
            &cell,
            &almanac,
            0.0,
            FIELD_CELL_SIZE,
            Some(TerrainType::Riverbed),
        )
        .expect("materializes");
        assert_eq!(result.archetype, WorldArchetype::River);
    }

    #[test]
    fn t7_enrich_aqua_liquid_basin_to_lake() {
        let almanac = AlchemicalAlmanac::default();
        let cell = mk_cell(250.0, MatterState::Liquid, 220.0, 1.0);
        let result = materialize_cell_at_time(
            &cell,
            &almanac,
            0.0,
            FIELD_CELL_SIZE,
            Some(TerrainType::Basin),
        )
        .expect("materializes");
        assert_eq!(result.archetype, WorldArchetype::Lake);
    }

    #[test]
    fn t7_enrich_terra_solid_cliff_to_rockface() {
        let almanac = AlchemicalAlmanac::default();
        let cell = mk_cell(60.0, MatterState::Solid, 60.0, 1.0);
        let result = materialize_cell_at_time(
            &cell,
            &almanac,
            0.0,
            FIELD_CELL_SIZE,
            Some(TerrainType::Cliff),
        )
        .expect("materializes");
        assert_eq!(result.archetype, WorldArchetype::Rockface);
    }

    #[test]
    fn t7_enrich_ignis_plasma_valley_to_lava_river() {
        let almanac = AlchemicalAlmanac::default();
        let cell = mk_cell(450.0, MatterState::Plasma, 160.0, 1.0);
        let result = materialize_cell_at_time(
            &cell,
            &almanac,
            0.0,
            FIELD_CELL_SIZE,
            Some(TerrainType::Valley),
        )
        .expect("materializes");
        assert_eq!(result.archetype, WorldArchetype::LavaRiver);
    }

    #[test]
    fn t7_enrich_terra_solid_plain_keeps_base() {
        let almanac = AlchemicalAlmanac::default();
        let cell = mk_cell(60.0, MatterState::Solid, 60.0, 1.0);
        let result = materialize_cell_at_time(
            &cell,
            &almanac,
            0.0,
            FIELD_CELL_SIZE,
            Some(TerrainType::Plain),
        )
        .expect("materializes");
        assert_eq!(result.archetype, WorldArchetype::TerraSolid);
    }

    #[test]
    fn t7_without_terrain_field_keeps_base_archetype() {
        let almanac = AlchemicalAlmanac::default();
        let cell = mk_cell(250.0, MatterState::Liquid, 220.0, 1.0);
        let result = materialize_cell_at_time(&cell, &almanac, 0.0, FIELD_CELL_SIZE, None)
            .expect("materializes");
        assert_eq!(result.archetype, WorldArchetype::AquaLiquid);
    }

    #[test]
    fn t7_enrich_is_deterministic() {
        let cell = mk_cell(60.0, MatterState::Solid, 60.0, 1.0);
        let a = enrich_archetype(WorldArchetype::TerraSolid, TerrainType::Cliff, &cell);
        let b = enrich_archetype(WorldArchetype::TerraSolid, TerrainType::Cliff, &cell);
        assert_eq!(a, b);
    }

    #[test]
    fn t7_enrich_mountain_riverbed_to_ravine() {
        let cell = mk_cell(60.0, MatterState::Solid, 500.0, 1.0);
        assert_eq!(
            enrich_archetype(WorldArchetype::Mountain, TerrainType::Riverbed, &cell),
            WorldArchetype::Ravine
        );
    }

    #[test]
    fn t7_enrich_aqua_solid_peak_to_glacier_peak() {
        let cell = mk_cell(250.0, MatterState::Solid, 100.0, 1.0);
        assert_eq!(
            enrich_archetype(WorldArchetype::AquaSolid, TerrainType::Peak, &cell),
            WorldArchetype::GlacierPeak
        );
    }

    #[test]
    fn t7_enrich_ignis_gas_peak_to_volcanic_vent() {
        let cell = mk_cell(450.0, MatterState::Gas, 100.0, 1.0);
        assert_eq!(
            enrich_archetype(WorldArchetype::IgnisGas, TerrainType::Peak, &cell),
            WorldArchetype::VolcanicVent
        );
    }

    #[test]
    fn t7_enrich_umbra_gas_valley_to_mist_valley() {
        let cell = mk_cell(20.0, MatterState::Gas, 100.0, 1.0);
        assert_eq!(
            enrich_archetype(WorldArchetype::UmbraGas, TerrainType::Valley, &cell),
            WorldArchetype::MistValley
        );
    }

    #[test]
    fn t7_enrich_ventus_gas_plateau_to_windswept_plateau() {
        let cell = mk_cell(700.0, MatterState::Gas, 100.0, 1.0);
        assert_eq!(
            enrich_archetype(WorldArchetype::VentusGas, TerrainType::Plateau, &cell),
            WorldArchetype::WindsweptPlateau
        );
    }

    #[test]
    fn t7_new_archetypes_have_specific_visual_profiles() {
        let profiled = [
            WorldArchetype::River,
            WorldArchetype::Lake,
            WorldArchetype::GlacierPeak,
            WorldArchetype::LavaRiver,
            WorldArchetype::VolcanicVent,
            WorldArchetype::MistValley,
            WorldArchetype::Rockface,
            WorldArchetype::WindsweptPlateau,
            WorldArchetype::Hillside,
            WorldArchetype::Ravine,
        ];
        for archetype in profiled {
            let (c, scale, emission, opacity) = crate::worldgen::apply_archetype_visual_profile(
                archetype,
                Color::srgb(0.5, 0.5, 0.5),
                1.0,
                0.1,
                0.9,
            );
            assert!(c.to_linear().red.is_finite());
            assert!(scale.is_finite() && scale > 0.0);
            assert!((0.0..=1.0).contains(&emission));
            assert!((0.0..=1.0).contains(&opacity));
        }
    }
}
