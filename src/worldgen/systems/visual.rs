use bevy::prelude::*;

use super::performance::{VisualDerivationFrameState, WorldgenPerfSettings};
use crate::blueprint::AlchemicalAlmanac;
use crate::layers::{
    BaseEnergy, MatterCoherence, MatterState, OscillatorySignature, SpatialVolume,
};
use crate::runtime_platform::compat_2d3d::RenderCompatProfile;
use crate::worldgen::constants::VISUAL_MIN_SCALE;
use crate::worldgen::materialization_rules::compound_path_active;
use crate::worldgen::visual_derivation::{
    apply_archetype_visual_profile, boundary_transition_emission_extra,
    energy_visual_boundary_flat_color, visual_proxy_temperature,
};
use crate::worldgen::{
    BoundaryVisual, EnergyFieldGrid, EnergyVisual, Materialized, PendingEnergyVisualRebuild,
    derive_all, derive_color_compound,
};

const FALLBACK_DENSITY: f32 = 0.0;
const DEFAULT_VISUAL_PURITY: f32 = 1.0;

#[derive(Clone, Copy, Debug, Default, Resource)]
pub struct VisualDerivationStats {
    pub recompute_count: u64,
}

#[derive(Clone, Copy, Debug)]
struct VisualInput {
    qe: f32,
    frequency_hz: f32,
    matter_state: MatterState,
    density: f32,
    temperature: f32,
}

type PendingVisualRebuildQueryItem<'w> = (
    Entity,
    &'w BaseEnergy,
    &'w OscillatorySignature,
    Option<&'w SpatialVolume>,
    Option<&'w MatterCoherence>,
    &'w Materialized,
    Option<&'w BoundaryVisual>,
);

type MissingVisualQueryItem<'w> = (
    Entity,
    &'w BaseEnergy,
    &'w OscillatorySignature,
    Option<&'w SpatialVolume>,
    Option<&'w MatterCoherence>,
    &'w Materialized,
    Option<&'w BoundaryVisual>,
);

type ChangedVisualQueryItem<'w> = (
    Entity,
    &'w BaseEnergy,
    &'w OscillatorySignature,
    Option<&'w SpatialVolume>,
    Option<&'w MatterCoherence>,
    &'w Materialized,
    Option<&'w BoundaryVisual>,
    &'w mut EnergyVisual,
);

type SyncRenderQueryItem<'w> = (&'w EnergyVisual, &'w mut Transform, Option<&'w mut Sprite>);

fn visual_input_from_components(
    energy: &BaseEnergy,
    signature: &OscillatorySignature,
    volume: Option<&SpatialVolume>,
    coherence: Option<&MatterCoherence>,
) -> VisualInput {
    let qe = if energy.qe().is_finite() {
        energy.qe().max(0.0)
    } else {
        0.0
    };
    let frequency_hz = if signature.frequency_hz().is_finite() {
        signature.frequency_hz().max(0.0)
    } else {
        0.0
    };
    let density = volume
        .map(|vol| vol.density(qe))
        .filter(|value| value.is_finite())
        .map(|value| value.max(0.0))
        .unwrap_or(FALLBACK_DENSITY);
    let matter_state = coherence
        .map(|coh| coh.state())
        .unwrap_or(MatterState::Solid);
    let temperature = coherence
        .map(|coh| visual_proxy_temperature(density, coh.bond_energy_eb()))
        .filter(|value| value.is_finite())
        .unwrap_or(0.0);

    VisualInput {
        qe,
        frequency_hz,
        matter_state,
        density,
        temperature,
    }
}

/// Pureza y mezcla compuesta desde `EnergyFieldGrid` si existe (misma lógica que materialización).
fn derive_visual_for_materialized(
    input: VisualInput,
    materialized: &Materialized,
    grid: Option<&EnergyFieldGrid>,
    interference_t: f32,
    almanac: &AlchemicalAlmanac,
    boundary: Option<&BoundaryVisual>,
) -> EnergyVisual {
    let default_purity = if input.qe > 0.0 {
        DEFAULT_VISUAL_PURITY
    } else {
        0.0
    };

    let (purity_for_base, cell_ref) = if let Some(grid) = grid {
        let ox = materialized.cell_x.max(0) as u32;
        let oy = materialized.cell_y.max(0) as u32;
        match grid.cell_xy(ox, oy) {
            Some(cell) => {
                let p = if cell.purity.is_finite() {
                    cell.purity.clamp(0.0, 1.0)
                } else {
                    0.0
                };
                (p, Some(cell))
            }
            None => (default_purity, None),
        }
    } else {
        (default_purity, None)
    };

    let mut derived = derive_all(
        input.frequency_hz,
        purity_for_base,
        input.density,
        input.temperature,
        input.matter_state,
        almanac,
    );

    if let Some(cell) = cell_ref {
        if compound_path_active(cell, almanac) {
            if let Some(c) = derive_color_compound(
                cell.frequency_contributions(),
                purity_for_base,
                interference_t,
                almanac,
            ) {
                derived.color = c;
            }
        }
    }

    if let Some(bv) = boundary {
        derived.color = energy_visual_boundary_flat_color(bv);
        let extra = boundary_transition_emission_extra(bv.transition_type);
        derived.emission = (derived.emission + extra).clamp(0.0, 1.0);
    }

    let (color, scale, emission, opacity) = apply_archetype_visual_profile(
        materialized.archetype,
        derived.color,
        derived.scale,
        derived.emission,
        derived.opacity,
    );

    EnergyVisual {
        color,
        scale: scale.max(VISUAL_MIN_SCALE),
        emission: emission.clamp(0.0, 1.0),
        opacity: opacity.clamp(0.0, 1.0),
    }
}

/// Rebuild inmediato tras invalidar `EnergyVisual` en delta (sin tope `max_visual_derivation_per_frame`).
pub fn flush_pending_energy_visual_rebuild_system(
    mut commands: Commands,
    almanac: Res<AlchemicalAlmanac>,
    time: Res<Time>,
    grid: Option<Res<EnergyFieldGrid>>,
    q: Query<PendingVisualRebuildQueryItem<'_>, With<crate::worldgen::PendingEnergyVisualRebuild>>,
    mut stats: Option<ResMut<VisualDerivationStats>>,
) {
    let grid_ref = grid.as_deref();
    let interference_t = time.elapsed_secs();
    for (entity, energy, signature, volume, coherence, materialized, boundary) in &q {
        let input = visual_input_from_components(energy, signature, volume, coherence);
        let visual = derive_visual_for_materialized(
            input,
            materialized,
            grid_ref,
            interference_t,
            &almanac,
            boundary,
        );
        commands
            .entity(entity)
            .insert(visual)
            .remove::<crate::worldgen::PendingEnergyVisualRebuild>();
        if let Some(ref mut s) = stats {
            s.recompute_count += 1;
        }
    }
}

/// Inserta `EnergyVisual` en materializados que aún no tienen componente visual
/// (presupuesto por frame vía `WorldgenPerfSettings`).
#[allow(clippy::too_many_arguments)]
#[allow(clippy::type_complexity)]
pub fn visual_derivation_insert_missing_system(
    mut commands: Commands,
    almanac: Res<AlchemicalAlmanac>,
    time: Res<Time>,
    settings: Res<WorldgenPerfSettings>,
    mut frame: ResMut<VisualDerivationFrameState>,
    grid: Option<Res<EnergyFieldGrid>>,
    query: Query<
        MissingVisualQueryItem<'_>,
        (
            With<Materialized>,
            Without<EnergyVisual>,
            Without<crate::worldgen::PendingEnergyVisualRebuild>,
        ),
    >,
    mut stats: ResMut<VisualDerivationStats>,
) {
    let grid_ref = grid.as_deref();
    let interference_t = time.elapsed_secs();
    let mut order: Vec<(i32, i32, Entity)> = Vec::new();
    for (entity, _, _, _, _, materialized, _) in query.iter() {
        order.push((materialized.cell_y, materialized.cell_x, entity));
    }
    order.sort_by_key(|(cy, cx, _)| (*cy, *cx));
    for (_, _, entity) in order {
        if frame.processed_this_frame >= settings.max_visual_derivation_per_frame {
            break;
        }
        let Ok((_, energy, signature, volume, coherence, materialized, boundary)) =
            query.get(entity)
        else {
            continue;
        };
        frame.processed_this_frame += 1;
        let input = visual_input_from_components(energy, signature, volume, coherence);
        let visual = derive_visual_for_materialized(
            input,
            materialized,
            grid_ref,
            interference_t,
            &almanac,
            boundary,
        );
        commands.entity(entity).insert(visual);
        stats.recompute_count += 1;
    }
}

/// Actualiza `EnergyVisual` sólo cuando cambia la energía.
#[allow(clippy::too_many_arguments)]
#[allow(clippy::type_complexity)]
pub fn visual_derivation_update_changed_system(
    mut commands: Commands,
    almanac: Res<AlchemicalAlmanac>,
    time: Res<Time>,
    settings: Res<WorldgenPerfSettings>,
    mut frame: ResMut<VisualDerivationFrameState>,
    grid: Option<Res<EnergyFieldGrid>>,
    mut query: Query<
        ChangedVisualQueryItem<'_>,
        (
            With<Materialized>,
            With<EnergyVisual>,
            Or<(
                Changed<BaseEnergy>,
                Changed<OscillatorySignature>,
                Changed<SpatialVolume>,
                Changed<MatterCoherence>,
                Changed<BoundaryVisual>,
                Changed<Materialized>,
            )>,
        ),
    >,
    mut stats: ResMut<VisualDerivationStats>,
) {
    let grid_ref = grid.as_deref();
    let interference_t = time.elapsed_secs();
    let mut order: Vec<(i32, i32, Entity)> = Vec::new();
    for (entity, _, _, _, _, materialized, _, _) in query.iter() {
        order.push((materialized.cell_y, materialized.cell_x, entity));
    }
    order.sort_by_key(|(cy, cx, _)| (*cy, *cx));
    for (i, (_, _, entity)) in order.iter().enumerate() {
        if frame.processed_this_frame >= settings.max_visual_derivation_per_frame {
            // Presupuesto agotado: `Changed<*>` no vuelve a disparar el próximo frame.
            for (_, _, e) in order.iter().skip(i) {
                commands.entity(*e).insert(PendingEnergyVisualRebuild);
            }
            break;
        }
        let Ok((_, energy, signature, volume, coherence, materialized, boundary, mut visual)) =
            query.get_mut(*entity)
        else {
            continue;
        };
        frame.processed_this_frame += 1;
        let input = visual_input_from_components(energy, signature, volume, coherence);
        let derived = derive_visual_for_materialized(
            input,
            materialized,
            grid_ref,
            interference_t,
            &almanac,
            boundary,
        );
        if *visual != derived {
            *visual = derived;
            stats.recompute_count += 1;
        }
    }
}

/// Traduce `EnergyVisual` a componentes de render 2D compatibles.
/// Entities with `ShapeInferred` already have GF1 mesh geometry — skip scale/sprite sync.
pub fn visual_sync_to_render_system(
    profile: Option<Res<RenderCompatProfile>>,
    mut query: Query<
        SyncRenderQueryItem<'_>,
        (
            Changed<EnergyVisual>,
            Without<crate::worldgen::shape_inference::ShapeInferred>,
        ),
    >,
) {
    let full3d = profile
        .as_ref()
        .map(|p| p.enables_visual_3d())
        .unwrap_or(false);

    for (visual, mut transform, maybe_sprite) in &mut query {
        let clamped_scale = visual.scale.max(VISUAL_MIN_SCALE);
        if full3d {
            // Sprites en XZ (rot -90° X): escala uniforme para mosaico legible en 3D.
            let s = Vec3::splat(clamped_scale);
            if transform.scale != s {
                transform.scale = s;
            }
        } else if transform.scale.x != clamped_scale || transform.scale.y != clamped_scale {
            transform.scale.x = clamped_scale;
            transform.scale.y = clamped_scale;
        }

        if let Some(mut sprite) = maybe_sprite {
            let opacity = if visual.opacity.is_finite() {
                visual.opacity.clamp(0.0, 1.0)
            } else {
                1.0
            };
            let emissive_boost = if visual.emission.is_finite() {
                1.0 + 0.35 * visual.emission.clamp(0.0, 1.0)
            } else {
                1.0
            };
            let color = visual.color.to_srgba();
            let next_sprite_color = Color::srgba(
                (color.red * emissive_boost).clamp(0.0, 1.0),
                (color.green * emissive_boost).clamp(0.0, 1.0),
                (color.blue * emissive_boost).clamp(0.0, 1.0),
                opacity,
            );
            if sprite.color != next_sprite_color {
                sprite.color = next_sprite_color;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{
        VisualDerivationStats, visual_derivation_insert_missing_system,
        visual_derivation_update_changed_system,
    };
    use crate::blueprint::{AlchemicalAlmanac, ElementDef};
    use crate::layers::{
        BaseEnergy, MatterCoherence, MatterState, OscillatorySignature, SpatialVolume,
    };
    use crate::worldgen::{EnergyVisual, Materialized, PendingEnergyVisualRebuild, WorldArchetype};
    use bevy::prelude::*;

    fn make_almanac_ignis_terra() -> AlchemicalAlmanac {
        let ignis = ElementDef {
            name: "Ignis".to_string(),
            symbol: "Ignis".to_string(),
            atomic_number: 8,
            frequency_hz: 450.0,
            freq_band: (400.0, 500.0),
            bond_energy: 1000.0,
            conductivity: 0.5,
            visibility: 0.8,
            matter_state: MatterState::Plasma,
            electronegativity: 0.0,
            ionization_ev: 0.0,
            color: (1.0, 0.30, 0.0),
            is_compound: false,
            phenology: None,
            hz_identity_weight: crate::blueprint::FIELD_EAC4_HZ_IDENTITY_WEIGHT_RON_ONLY,
        };
        let terra = ElementDef {
            name: "Terra".to_string(),
            symbol: "Terra".to_string(),
            atomic_number: 14,
            frequency_hz: 75.0,
            freq_band: (50.0, 84.0),
            bond_energy: 3000.0,
            conductivity: 0.4,
            visibility: 0.8,
            matter_state: MatterState::Solid,
            electronegativity: 0.0,
            ionization_ev: 0.0,
            color: (0.45, 0.34, 0.20),
            is_compound: false,
            phenology: None,
            hz_identity_weight: crate::blueprint::FIELD_EAC4_HZ_IDENTITY_WEIGHT_RON_ONLY,
        };
        AlchemicalAlmanac::from_defs(vec![ignis, terra])
    }

    fn setup_app() -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.insert_resource(make_almanac_ignis_terra())
            .init_resource::<VisualDerivationStats>()
            .init_resource::<super::super::performance::WorldgenPerfSettings>()
            .init_resource::<super::super::performance::VisualDerivationFrameState>()
            .add_systems(
                Update,
                (
                    super::super::performance::reset_visual_derivation_frame_system,
                    visual_derivation_update_changed_system,
                    visual_derivation_insert_missing_system,
                )
                    .chain(),
            );
        app
    }

    #[test]
    fn visual_derivation_ignis_generates_orange_like_color() {
        let mut app = setup_app();
        let entity = app
            .world_mut()
            .spawn((
                Materialized {
                    cell_x: 0,
                    cell_y: 0,
                    archetype: WorldArchetype::IgnisSolid,
                },
                BaseEnergy::new(100.0),
                OscillatorySignature::new(450.0, 0.0),
                SpatialVolume::new(1.0),
            ))
            .id();

        app.update();

        let visual = app
            .world()
            .entity(entity)
            .get::<EnergyVisual>()
            .expect("EnergyVisual must be inserted");
        let color = visual.color.to_srgba();
        assert!(
            color.red > color.green && color.green > color.blue,
            "expected Ignis orange, got r={} g={} b={}",
            color.red,
            color.green,
            color.blue
        );
    }

    #[test]
    fn visual_derivation_river_archetype_biases_vertex_blue_channel() {
        let mut app = setup_app();
        let entity = app
            .world_mut()
            .spawn((
                Materialized {
                    cell_x: 0,
                    cell_y: 0,
                    archetype: WorldArchetype::River,
                },
                BaseEnergy::new(100.0),
                OscillatorySignature::new(250.0, 0.0),
                SpatialVolume::new(1.0),
                MatterCoherence::new(MatterState::Liquid, 1000.0, 0.3),
            ))
            .id();

        app.update();

        let visual = app
            .world()
            .entity(entity)
            .get::<EnergyVisual>()
            .expect("EnergyVisual must be inserted");
        let rgba = visual.color.to_srgba();
        assert!(
            rgba.blue > rgba.red && rgba.blue > rgba.green,
            "perfil River debe empujar azul: r={} g={} b={}",
            rgba.red,
            rgba.green,
            rgba.blue
        );
    }

    #[test]
    fn visual_derivation_terra_generates_brown_like_color() {
        let mut app = setup_app();
        let entity = app
            .world_mut()
            .spawn((
                Materialized {
                    cell_x: 0,
                    cell_y: 1,
                    archetype: WorldArchetype::TerraSolid,
                },
                BaseEnergy::new(100.0),
                OscillatorySignature::new(75.0, 0.0),
                SpatialVolume::new(1.0),
            ))
            .id();

        app.update();

        let visual = app
            .world()
            .entity(entity)
            .get::<EnergyVisual>()
            .expect("EnergyVisual must be inserted");
        let color = visual.color.to_srgba();
        assert!(
            color.red > color.blue && color.green > color.blue,
            "expected Terra brown, got r={} g={} b={}",
            color.red,
            color.green,
            color.blue
        );
    }

    #[test]
    fn visual_derivation_inserts_energy_visual_when_missing() {
        let mut app = setup_app();
        let entity = app
            .world_mut()
            .spawn((
                Materialized {
                    cell_x: 2,
                    cell_y: 2,
                    archetype: WorldArchetype::AquaSolid,
                },
                BaseEnergy::new(80.0),
                OscillatorySignature::new(250.0, 0.0),
            ))
            .id();

        app.update();

        assert!(app.world().entity(entity).contains::<EnergyVisual>());
    }

    #[test]
    fn visual_derivation_budget_queues_pending_for_remaining_dirty_entities() {
        let mut app = setup_app();
        let entity_a = app
            .world_mut()
            .spawn((
                Materialized {
                    cell_x: 0,
                    cell_y: 0,
                    archetype: WorldArchetype::LuxSolid,
                },
                BaseEnergy::new(100.0),
                OscillatorySignature::new(950.0, 0.0),
                SpatialVolume::new(1.0),
            ))
            .id();
        let entity_b = app
            .world_mut()
            .spawn((
                Materialized {
                    cell_x: 1,
                    cell_y: 0,
                    archetype: WorldArchetype::LuxSolid,
                },
                BaseEnergy::new(100.0),
                OscillatorySignature::new(950.0, 0.0),
                SpatialVolume::new(1.0),
            ))
            .id();

        // Presupuesto amplio: ambas entidades necesitan `EnergyVisual` antes de probar el tope en `update_changed`.
        app.update();
        app.world_mut()
            .resource_mut::<super::super::performance::WorldgenPerfSettings>()
            .max_visual_derivation_per_frame = 1;

        {
            let world = app.world_mut();
            world.entity_mut(entity_a).insert(BaseEnergy::new(101.0));
            world.entity_mut(entity_b).insert(BaseEnergy::new(101.0));
        }

        app.update();

        assert!(
            !app.world()
                .entity(entity_a)
                .contains::<PendingEnergyVisualRebuild>(),
            "first in (cell_y, cell_x) order should process in-frame"
        );
        assert!(
            app.world()
                .entity(entity_b)
                .contains::<PendingEnergyVisualRebuild>(),
            "remaining dirty entities need pending rebuild when budget exhausts"
        );
    }

    #[test]
    fn visual_derivation_skips_recompute_when_energy_unchanged() {
        let mut app = setup_app();
        app.world_mut().spawn((
            Materialized {
                cell_x: 3,
                cell_y: 3,
                archetype: WorldArchetype::LuxSolid,
            },
            BaseEnergy::new(42.0),
            OscillatorySignature::new(950.0, 0.0),
        ));

        app.update();
        let first_count = app
            .world()
            .resource::<VisualDerivationStats>()
            .recompute_count;
        app.update();
        let second_count = app
            .world()
            .resource::<VisualDerivationStats>()
            .recompute_count;

        assert_eq!(first_count, second_count);
    }

    #[test]
    fn visual_derivation_recomputes_when_frequency_changes_without_energy_change() {
        let mut app = setup_app();
        let entity = app
            .world_mut()
            .spawn((
                Materialized {
                    cell_x: 7,
                    cell_y: 7,
                    archetype: WorldArchetype::IgnisSolid,
                },
                BaseEnergy::new(120.0),
                OscillatorySignature::new(450.0, 0.0),
                SpatialVolume::new(1.0),
            ))
            .id();

        app.update();
        let color_before = app
            .world()
            .entity(entity)
            .get::<EnergyVisual>()
            .expect("EnergyVisual expected")
            .color
            .to_srgba();
        let count_before = app
            .world()
            .resource::<VisualDerivationStats>()
            .recompute_count;

        {
            let world = app.world_mut();
            let mut entity_mut = world.entity_mut(entity);
            let mut signature = entity_mut
                .get_mut::<OscillatorySignature>()
                .expect("OscillatorySignature expected");
            signature.set_frequency_hz(75.0);
        }

        app.update();

        let color_after = app
            .world()
            .entity(entity)
            .get::<EnergyVisual>()
            .expect("EnergyVisual expected")
            .color
            .to_srgba();
        let count_after = app
            .world()
            .resource::<VisualDerivationStats>()
            .recompute_count;

        assert!(count_after > count_before);
        assert!(
            (color_after.red - color_before.red).abs() > 1e-3
                || (color_after.green - color_before.green).abs() > 1e-3
                || (color_after.blue - color_before.blue).abs() > 1e-3
        );
    }

    #[test]
    fn visual_sync_applies_scale_opacity_and_emission_to_sprite() {
        let mut app = App::new();
        app.add_systems(Update, super::visual_sync_to_render_system);

        let entity = app
            .world_mut()
            .spawn((
                EnergyVisual {
                    color: Color::srgb(0.3, 0.2, 0.1),
                    scale: 2.0,
                    emission: 1.0,
                    opacity: 0.4,
                },
                Transform::default(),
                Sprite::default(),
            ))
            .id();

        app.update();

        let e = app.world().entity(entity);
        let transform = e.get::<Transform>().expect("Transform expected");
        let sprite = e.get::<Sprite>().expect("Sprite expected");
        let rgba = sprite.color.to_srgba();

        assert!((transform.scale.x - 2.0).abs() < 1e-4);
        assert!((transform.scale.y - 2.0).abs() < 1e-4);
        assert!((rgba.alpha - 0.4).abs() < 1e-4);
        assert!(rgba.red >= 0.3);
    }
}
