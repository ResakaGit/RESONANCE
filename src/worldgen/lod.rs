//! LOD y presupuestos de materialización: funciones puras (sin ECS), testeables.

use crate::math_types::Vec2;

use crate::blueprint::AlchemicalAlmanac;
use crate::layers::MatterState;
use crate::topology::TerrainType;
use crate::worldgen::contracts::EnergyCell;
use crate::worldgen::materialization_rules::compound_path_active;

pub use crate::worldgen::constants::{LOD_MID_MAX, LOD_NEAR_MAX};

/// Banda LOD por distancia al foco (jugador o cámara).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LodBand {
    Near,
    Mid,
    Far,
}

/// Distancia al cuadrado del centro de celda al foco. `focus == None` ⇒ Near en todo el grid (tests sin héroe).
#[inline]
pub fn distance_sq_cell_to_focus(cell_center: Vec2, focus: Option<Vec2>) -> f32 {
    let Some(f) = focus else {
        return 0.0;
    };
    if !cell_center.is_finite() || !f.is_finite() {
        return f32::MAX;
    }
    cell_center.distance_squared(f)
}

/// Clasificación por distancia euclídea (no cuadrada).
#[inline]
pub fn lod_band_from_distance_sq(distance_sq: f32) -> LodBand {
    if !distance_sq.is_finite() {
        return LodBand::Far;
    }
    let d = distance_sq.sqrt();
    if d <= LOD_NEAR_MAX {
        LodBand::Near
    } else if d <= LOD_MID_MAX {
        LodBand::Mid
    } else {
        LodBand::Far
    }
}

/// ¿Este tick de simulación debe intentar materializar según LOD? Far usa `far_period`, Mid usa `mid_period`.
#[inline]
pub fn materialization_tick_active_for_band(
    band: LodBand,
    sim_tick: u64,
    mid_period: u64,
    far_period: u64,
) -> bool {
    let mid_period = mid_period.max(1);
    let far_period = far_period.max(1);
    match band {
        LodBand::Near => true,
        LodBand::Mid => sim_tick % mid_period == 0,
        LodBand::Far => sim_tick % far_period == 0,
    }
}

/// Corta materialización/visual más allá de esta distancia (anillo “fuera de visión”).
#[inline]
pub fn materialization_culled(distance_sq: f32, cull_distance: f32) -> bool {
    if !distance_sq.is_finite() || !cull_distance.is_finite() || cull_distance <= 0.0 {
        return false;
    }
    distance_sq > cull_distance * cull_distance
}

/// Regla completa: no cull + banda activa en este tick.
pub fn materialization_allowed(
    cell_center: Vec2,
    focus: Option<Vec2>,
    sim_tick: u64,
    cull_distance: f32,
    mid_period: u64,
    far_period: u64,
) -> bool {
    let dsq = distance_sq_cell_to_focus(cell_center, focus);
    if materialization_culled(dsq, cull_distance) {
        return false;
    }
    let band = lod_band_from_distance_sq(dsq);
    materialization_tick_active_for_band(band, sim_tick, mid_period, far_period)
}

/// Firma para cache de materialización: mezcla compuesta incluye `t` (animación); ruta pura no.
pub fn materialize_input_signature(cell: &EnergyCell, t: f32, almanac: &AlchemicalAlmanac) -> u64 {
    let mut h: u64 = 0x9E37_79B9_7F4A_7C15;
    h ^= u64::from(cell.accumulated_qe.to_bits());
    h = h.rotate_left(11) ^ u64::from(cell.dominant_frequency_hz.to_bits());
    h = h.rotate_left(7) ^ u64::from(cell.purity.to_bits());
    h = h.rotate_left(13) ^ u64::from(cell.temperature.to_bits());
    h ^= match cell.matter_state {
        MatterState::Solid => 1,
        MatterState::Liquid => 2,
        MatterState::Gas => 3,
        MatterState::Plasma => 4,
    };
    let mut contrib: Vec<_> = cell
        .frequency_contributions()
        .iter()
        .take(8)
        .copied()
        .collect();
    // Orden estable: misma multicontribución con distinto orden de inserción → misma firma.
    contrib.sort_unstable_by(|a, b| {
        a.frequency_hz()
            .total_cmp(&b.frequency_hz())
            .then_with(|| a.source_entity().index().cmp(&b.source_entity().index()))
            .then_with(|| a.intensity_qe().total_cmp(&b.intensity_qe()))
    });
    for c in contrib.iter() {
        h = h
            .wrapping_add(u64::from(c.frequency_hz().to_bits()))
            .rotate_left(3)
            ^ u64::from(c.intensity_qe().to_bits());
    }
    if compound_path_active(cell, almanac) {
        h = h.rotate_left(5) ^ u64::from(t.to_bits());
    }
    h
}

/// Tag estable para mezclar relieve (`TerrainType`) en la firma de cache de materialización.
/// Debe coincidir con lo que consume `enrich_archetype` vía `materialize_cell_at_time`.
#[inline]
pub fn terrain_type_cache_tag(terrain_type: Option<TerrainType>) -> u64 {
    let v = match terrain_type {
        None => 0u8,
        Some(TerrainType::Peak) => 1,
        Some(TerrainType::Ridge) => 2,
        Some(TerrainType::Slope) => 3,
        Some(TerrainType::Valley) => 4,
        Some(TerrainType::Plain) => 5,
        Some(TerrainType::Riverbed) => 6,
        Some(TerrainType::Basin) => 7,
        Some(TerrainType::Cliff) => 8,
        Some(TerrainType::Plateau) => 9,
    };
    u64::from(v)
}

/// Decide si reutilizar resultado cacheado (misma firma).
#[inline]
pub fn materialization_cache_hit(
    cached_sig: u64,
    cell: &EnergyCell,
    t: f32,
    almanac: &AlchemicalAlmanac,
) -> bool {
    cached_sig != 0 && cached_sig == materialize_input_signature(cell, t, almanac)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::blueprint::AlchemicalAlmanac;
    use crate::topology::TerrainType;
    use crate::worldgen::FrequencyContribution;
    use bevy::prelude::Entity;

    #[test]
    fn materialize_signature_stable_under_contribution_order() {
        let almanac = AlchemicalAlmanac::default();
        let e1 = Entity::from_raw(1);
        let e2 = Entity::from_raw(2);
        let mut a = EnergyCell {
            accumulated_qe: 40.0,
            dominant_frequency_hz: 60.0,
            purity: 0.9,
            ..Default::default()
        };
        a.frequency_contributions
            .push(FrequencyContribution::new(e1, 100.0, 5.0));
        a.frequency_contributions
            .push(FrequencyContribution::new(e2, 200.0, 3.0));
        let mut b = EnergyCell {
            accumulated_qe: a.accumulated_qe,
            dominant_frequency_hz: a.dominant_frequency_hz,
            purity: a.purity,
            ..Default::default()
        };
        b.frequency_contributions
            .push(FrequencyContribution::new(e2, 200.0, 3.0));
        b.frequency_contributions
            .push(FrequencyContribution::new(e1, 100.0, 5.0));
        assert_eq!(
            materialize_input_signature(&a, 0.0, &almanac),
            materialize_input_signature(&b, 0.0, &almanac)
        );
    }

    #[test]
    fn lod_far_band_skips_most_ticks() {
        let dsq = 85.0_f32 * 85.0;
        assert_eq!(lod_band_from_distance_sq(dsq), LodBand::Far);
        assert!(!materialization_tick_active_for_band(
            LodBand::Far,
            1,
            4,
            16
        ));
        assert!(materialization_tick_active_for_band(LodBand::Far, 0, 4, 16));
    }

    #[test]
    fn lod_cull_disables_materialization_beyond_distance() {
        let focus = Vec2::ZERO;
        let far_cell = Vec2::new(200.0, 0.0);
        assert!(!materialization_allowed(
            far_cell,
            Some(focus),
            0,
            150.0,
            4,
            16
        ));
    }

    #[test]
    fn no_focus_implies_full_near_lod() {
        let cell = Vec2::new(1000.0, 1000.0);
        assert!(materialization_allowed(cell, None, 3, 50.0, 4, 16));
    }

    #[test]
    fn materialize_signature_changes_when_qe_changes() {
        let almanac = AlchemicalAlmanac::default();
        let a = EnergyCell {
            accumulated_qe: 20.0,
            dominant_frequency_hz: 60.0,
            ..Default::default()
        };
        let mut b = a.clone();
        b.accumulated_qe = 21.0;
        assert_ne!(
            materialize_input_signature(&a, 0.0, &almanac),
            materialize_input_signature(&b, 0.0, &almanac)
        );
    }

    #[test]
    fn terrain_type_cache_tag_variants_differ() {
        assert_ne!(
            terrain_type_cache_tag(Some(TerrainType::Plain)),
            terrain_type_cache_tag(Some(TerrainType::Riverbed))
        );
        assert_eq!(terrain_type_cache_tag(None), 0);
    }

    #[test]
    fn pure_path_cache_ignores_time_in_signature() {
        let almanac = AlchemicalAlmanac::default();
        let mut cell = EnergyCell {
            accumulated_qe: 50.0,
            dominant_frequency_hz: 75.0,
            purity: 0.9,
            ..Default::default()
        };
        cell.frequency_contributions
            .push(crate::worldgen::FrequencyContribution::new(
                Entity::from_raw(1),
                75.0,
                10.0,
            ));
        let s = materialize_input_signature(&cell, 0.2, &almanac);
        assert!(materialization_cache_hit(s, &cell, 0.2, &almanac));
        assert!(materialization_cache_hit(s, &cell, 0.3, &almanac));
    }
}
