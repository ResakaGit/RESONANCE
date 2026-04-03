//! Fenología visual (EA8): una transformación — ajusta `EnergyVisual.color` desde datos de almanaque.
//!
//! La query toca varias capas (L0, grid, `EnergyVisual`); es el coste explícito de proyectar señales
//! a color sin ramas por arquetipo. Partir en dos sistemas solo si el perfil de arquetipos crece mucho.

use bevy::prelude::*;

use crate::blueprint::AlchemicalAlmanac;
use crate::blueprint::equations;
use crate::layers::{BaseEnergy, GrowthBudget};
use crate::worldgen::constants::VISUAL_NEUTRAL_GRAY_CHANNEL;
use crate::worldgen::visual_derivation::derive_color_phenology;
use crate::worldgen::{
    BoundaryVisual, EnergyFieldGrid, EnergyVisual, Materialized, PhenologyPhaseCache,
    PhenologyVisualParams,
};

#[inline]
fn sanitize_channel(c: f32) -> f32 {
    if c.is_finite() {
        c.clamp(0.0, 1.0)
    } else {
        VISUAL_NEUTRAL_GRAY_CHANNEL
    }
}

#[inline]
fn color_from_data_rgb(rgb: (f32, f32, f32)) -> Color {
    Color::srgb(
        sanitize_channel(rgb.0),
        sanitize_channel(rgb.1),
        sanitize_channel(rgb.2),
    )
}

/// Lee señales ECS/grid, computa fase pura y reconcilia `EnergyVisual.color` con el almanaque.
/// La histeresis (`epsilon`) solo actualiza `PhenologyPhaseCache`, no bloquea el tinte frente a `derive_visual_*`.
/// Excluye fronteras ecológicas (el color allí lo fija `derive_visual_for_materialized`).
pub fn phenology_visual_apply_system(
    mut commands: Commands,
    almanac: Res<AlchemicalAlmanac>,
    grid: Option<Res<EnergyFieldGrid>>,
    mut query: Query<
        (
            Entity,
            &BaseEnergy,
            Option<&GrowthBudget>,
            &PhenologyVisualParams,
            &mut EnergyVisual,
            Option<&Materialized>,
            Option<&mut PhenologyPhaseCache>,
        ),
        (With<PhenologyVisualParams>, Without<BoundaryVisual>),
    >,
) {
    let grid_ref = grid.as_deref();
    for (entity, energy, growth, params, mut visual, materialized, phase_cache_slot) in &mut query {
        let Some(def) = almanac.get(params.element_id) else {
            continue;
        };
        let Some(ph) = def.phenology else {
            continue;
        };

        let ceiling = params.growth_norm_ceiling.max(f32::EPSILON);
        let cell_growth_purity = match (grid_ref, materialized) {
            (Some(grid), Some(mat)) => {
                let ox = mat.cell_x.max(0) as u32;
                let oy = mat.cell_y.max(0) as u32;
                grid.cell_xy(ox, oy).map(|cell| {
                    let gt = equations::normalize_range(cell.accumulated_qe, 0.0, ceiling);
                    let pt = if cell.purity.is_finite() {
                        cell.purity.clamp(0.0, 1.0)
                    } else {
                        0.0
                    };
                    (gt, pt)
                })
            }
            _ => None,
        };

        let growth_t = if let Some(g) = growth {
            equations::normalize_range(g.biomass_available, 0.0, ceiling)
        } else {
            cell_growth_purity.map(|(gt, _)| gt).unwrap_or(0.0)
        };

        let qe = energy.qe();
        let qe_t = equations::normalize_range(qe, 0.0, params.qe_reference.max(f32::EPSILON));

        let purity_t = cell_growth_purity.map(|(_, pt)| pt).unwrap_or(1.0);

        let phase =
            equations::phenology_phase(growth_t, qe_t, purity_t, ph.w_growth, ph.w_qe, ph.w_purity);

        // Siempre reconciliar con el tinte fenológico tras `derive_visual_*`: la derivación base
        // puede haber reescrito `color` en el mismo `Update` aunque la fase no cruce ε.
        let young = color_from_data_rgb(ph.young_rgb);
        let mature = color_from_data_rgb(ph.mature_rgb);
        let next_color = derive_color_phenology(young, mature, phase);

        if visual.color != next_color {
            visual.color = next_color;
        }

        let prev_phase = phase_cache_slot
            .as_ref()
            .map(|c| c.prev_phase)
            .unwrap_or(f32::NAN);
        if equations::phenology_refresh_needed(prev_phase, phase, params.epsilon) {
            if let Some(mut cache) = phase_cache_slot {
                if cache.prev_phase != phase {
                    cache.prev_phase = phase;
                }
            } else {
                commands
                    .entity(entity)
                    .insert(PhenologyPhaseCache { prev_phase: phase });
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::phenology_visual_apply_system;
    use crate::blueprint::ElementId;
    use crate::blueprint::almanac::{AlchemicalAlmanac, ElementDef, ElementPhenologyDef};
    use crate::layers::{BaseEnergy, GrowthBudget};
    use crate::worldgen::{
        EnergyCell, EnergyFieldGrid, EnergyVisual, Materialized, PhenologyVisualParams,
        WorldArchetype,
    };
    use bevy::prelude::*;

    fn mk_almanac_phenology_terra() -> AlchemicalAlmanac {
        let terra = ElementDef {
            name: "Terra".to_string(),
            symbol: "Terra".to_string(),
            atomic_number: 14,
            frequency_hz: 75.0,
            freq_band: (50.0, 84.0),
            bond_energy: 3000.0,
            conductivity: 0.4,
            visibility: 0.8,
            matter_state: crate::layers::MatterState::Solid,
            electronegativity: 0.0,
            ionization_ev: 0.0,
            color: (0.45, 0.34, 0.20),
            is_compound: false,
            phenology: Some(ElementPhenologyDef {
                young_rgb: (0.9, 0.2, 0.15),
                mature_rgb: (0.2, 0.85, 0.15),
                w_growth: 1.0,
                w_qe: 0.0,
                w_purity: 0.0,
            }),
            hz_identity_weight: crate::blueprint::FIELD_EAC4_HZ_IDENTITY_WEIGHT_RON_ONLY,
        };
        AlchemicalAlmanac::from_defs(vec![terra])
    }

    #[test]
    fn phenology_shifts_color_when_growth_crosses_threshold() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.insert_resource(mk_almanac_phenology_terra());

        let mut grid = EnergyFieldGrid::new(1, 1, 1.0, Vec2::ZERO);
        if let Some(cell) = grid.cell_xy_mut(0, 0) {
            *cell = EnergyCell {
                accumulated_qe: 10.0,
                purity: 1.0,
                ..Default::default()
            };
        }
        app.insert_resource(grid);

        app.add_systems(Update, phenology_visual_apply_system);

        let tid = ElementId::from_name("Terra");
        let entity = app
            .world_mut()
            .spawn((
                Materialized {
                    cell_x: 0,
                    cell_y: 0,
                    archetype: WorldArchetype::TerraSolid,
                },
                BaseEnergy::new(100.0),
                GrowthBudget::new(5.0, 0, 1.0),
                PhenologyVisualParams {
                    element_id: tid,
                    growth_norm_ceiling: 100.0,
                    qe_reference: 600.0,
                    epsilon: 0.001,
                },
                EnergyVisual {
                    color: Color::BLACK,
                    scale: 1.0,
                    emission: 0.0,
                    opacity: 1.0,
                },
            ))
            .id();

        app.update();
        let c_low = app
            .world()
            .entity(entity)
            .get::<EnergyVisual>()
            .expect("ev")
            .color
            .to_srgba();

        app.world_mut()
            .entity_mut(entity)
            .insert(GrowthBudget::new(95.0, 0, 1.0));
        app.update();
        let c_high = app
            .world()
            .entity(entity)
            .get::<EnergyVisual>()
            .expect("ev")
            .color
            .to_srgba();

        assert!(
            (c_low.red - c_high.red).abs() > 0.05 || (c_low.green - c_high.green).abs() > 0.05,
            "color should move young→mature: low={c_low:?} high={c_high:?}"
        );
    }

    #[test]
    fn phenology_skips_entity_without_phenology_in_almanac() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        let plain = ElementDef {
            name: "Terra".to_string(),
            symbol: "Terra".to_string(),
            atomic_number: 14,
            frequency_hz: 75.0,
            freq_band: (50.0, 84.0),
            bond_energy: 3000.0,
            conductivity: 0.4,
            visibility: 0.8,
            matter_state: crate::layers::MatterState::Solid,
            electronegativity: 0.0,
            ionization_ev: 0.0,
            color: (0.45, 0.34, 0.20),
            is_compound: false,
            phenology: None,
            hz_identity_weight: crate::blueprint::FIELD_EAC4_HZ_IDENTITY_WEIGHT_RON_ONLY,
        };
        app.insert_resource(AlchemicalAlmanac::from_defs(vec![plain]));
        app.add_systems(Update, phenology_visual_apply_system);

        let color_before = Color::srgb(0.25, 0.6, 0.1);
        let entity = app
            .world_mut()
            .spawn((
                BaseEnergy::new(50.0),
                PhenologyVisualParams {
                    element_id: ElementId::from_name("Terra"),
                    growth_norm_ceiling: 10.0,
                    qe_reference: 100.0,
                    epsilon: 0.01,
                },
                EnergyVisual {
                    color: color_before,
                    scale: 1.0,
                    emission: 0.0,
                    opacity: 1.0,
                },
            ))
            .id();

        app.update();
        let after = app
            .world()
            .entity(entity)
            .get::<EnergyVisual>()
            .unwrap()
            .color;
        assert_eq!(after, color_before);
    }
}
