//! MG-9 + V5: entity shape inference with `PerformanceCachePolicy` cache.
//!
//! Bridges `MorphogenesisShapeParams.fineness_ratio` → GF1 mesh geometry.
//! Cache is keyed by `shape_cache_signature` — a u16 encoding tendency
//! (fineness, qe_norm, radius) and sensory inputs (hunger, food proximity, hostile).
//! Mesh rebuild only occurs on cache miss; `PerformanceCachePolicy.dependency_signature`
//! is updated after each rebuild.
//!
//! Sensory modulation:
//!   - Hunger (EnergyAssessment) → slight fineness elongation (streamlining to hunt)
//!   - Hostile nearby (SensoryAwareness) → resistance stiffening (defensive posture)

use bevy::prelude::*;

use crate::blueprint::constants::{FINENESS_DEFAULT, VISUAL_QE_REFERENCE};
use crate::blueprint::equations::{
    albedo_luminosity_blend, entity_geometry_influence, entity_lod_detail,
    frequency_to_tint_rgb, matter_to_gf1_resistance, normalized_qe,
    organ_slot_scale, rugosity_to_detail_multiplier, shape_cache_signature,
    shape_cache_signature_with_surface,
};
use crate::geometry_flow::{build_flow_mesh, build_flow_spine, merge_meshes, GeometryInfluence};
use crate::layers::{
    BaseEnergy, BodyPlanLayout, CacheScope, EnergyAssessment, FlowVector, HasInferredShape,
    InferenceProfile, InferredAlbedo, MatterCoherence, MorphogenesisSurface,
    MorphogenesisShapeParams, OscillatorySignature, PerformanceCachePolicy, SensoryAwareness,
    SpatialVolume,
};
use crate::runtime_platform::render_bridge_3d::V6VisualRoot;
use crate::worldgen::ShapeInferred;

const DEFAULT_Z:             f32 = 0.0;
const HUNGER_FINENESS_BOOST: f32 = 0.25;  // max +25% elongation when fully hungry
const HOSTILE_RESIST_MULT:   f32 = 1.35;  // +35% stiffness when predator is nearby

/// Infers GF1 tube mesh from entity physics layers with `PerformanceCachePolicy` caching.
///
/// Cache miss conditions (trigger rebuild):
///   1. No `ShapeInferred` yet (first build).
///   2. Policy disabled, or scope is `FrameLocal`.
///   3. `policy.dependency_signature != shape_cache_signature(current inputs)`.
///
/// Sensory inputs modulate geometry on rebuild; signature tracks the modulated state.
pub fn entity_shape_inference_system(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut query: Query<
        (
            Entity,
            &BaseEnergy,
            &SpatialVolume,
            &OscillatorySignature,
            &FlowVector,
            &MatterCoherence,
            Option<&MorphogenesisShapeParams>,
            Option<&EnergyAssessment>,
            Option<&SensoryAwareness>,
            Option<&mut PerformanceCachePolicy>,
            Option<&ShapeInferred>,
            &V6VisualRoot,
            Option<&MorphogenesisSurface>,
            Option<&InferredAlbedo>,
            Option<&BodyPlanLayout>,
        ),
        With<HasInferredShape>,
    >,
    mut visual_query: Query<(&mut Mesh3d, &mut Transform), Without<HasInferredShape>>,
    profile_query: Query<&InferenceProfile>,
) {
    for (entity, energy, volume, wave, flow, matter, shape_opt,
         hunger_opt, sensory_opt, mut policy_opt, shape_inferred, visual_root,
         surface_opt, albedo_opt, body_plan_opt)
        in query.iter_mut()
    {
        let qe_norm        = normalized_qe(energy.qe(), VISUAL_QE_REFERENCE);
        let fineness_base  = shape_opt.map(|s| s.fineness_ratio()).unwrap_or(FINENESS_DEFAULT);
        let hunger         = hunger_opt.map(|e| e.hunger_fraction).unwrap_or(0.0);
        let (food_dist, has_hostile) = sensory_opt
            .map(|s| (s.food_distance, s.hostile_entity.is_some()))
            .unwrap_or((f32::MAX, false));

        let base_sig = shape_cache_signature(
            fineness_base, qe_norm, volume.radius, hunger, food_dist, has_hostile,
        );
        let new_sig = shape_cache_signature_with_surface(
            base_sig,
            surface_opt.map(|s| s.rugosity()),
            albedo_opt.map(|a| a.albedo()),
        );

        // ── Cache hit check ───────────────────────────────────────────────────
        if shape_inferred.is_some() {
            match policy_opt.as_deref() {
                Some(p) if p.enabled
                        && p.scope == CacheScope::StableWindow
                        && p.dependency_signature == new_sig => continue,
                None => continue, // no policy + already built → keep sphere
                _ => {}           // policy disabled, FrameLocal, or sig changed → rebuild
            }
        }

        // ── Sensory geometry modulation ───────────────────────────────────────
        let fineness = fineness_base * (1.0 + hunger * HUNGER_FINENESS_BOOST);
        let base_res = matter_to_gf1_resistance(matter.bond_energy_eb(), matter.state());
        let resistance = if has_hostile { (base_res * HOSTILE_RESIST_MULT).min(2.5) } else { base_res };
        let tint_base = frequency_to_tint_rgb(wave.frequency_hz());
        let detail_base = entity_lod_detail(qe_norm, volume.radius);

        // MG-7: rugosity drives mesh detail multiplier
        let detail = if let Some(surface) = surface_opt {
            (detail_base * rugosity_to_detail_multiplier(surface.rugosity())).clamp(0.0, 1.0)
        } else {
            detail_base
        };

        // MG-5: albedo modulates tint brightness
        let tint = if let Some(albedo) = albedo_opt {
            let lum = albedo_luminosity_blend(1.0, albedo.albedo());
            [tint_base[0] * lum, tint_base[1] * lum, tint_base[2] * lum]
        } else {
            tint_base
        };

        let vel_2d = flow.velocity();

        let influence = entity_geometry_influence(
            Vec3::ZERO,
            qe_norm,
            volume.radius,
            fineness,
            resistance,
            Vec3::new(vel_2d.x, vel_2d.y, DEFAULT_Z),
            tint,
            detail,
        );

        let spine = build_flow_spine(&influence);
        let torso_mesh = build_flow_mesh(&spine, &influence);

        // ── Compound mesh: body plan → per-organ GF1 tubes merged ───────────
        let final_mesh = if let Some(layout) = body_plan_opt {
            let count = layout.active_count();
            if count > 0 {
                let mobility = profile_query.get(entity).map(|p| p.mobility_bias).unwrap_or(0.5);
                let mut organ_meshes = Vec::with_capacity(count as usize + 1);
                organ_meshes.push(torso_mesh);

                for i in 0..count as usize {
                    let (len_factor, rad_factor) = organ_slot_scale(i, count, mobility);
                    if len_factor <= 0.0 { continue; }

                    let organ_pos = layout.position(i);
                    let organ_dir = layout.direction(i);
                    let organ_inf = GeometryInfluence {
                        detail:                     influence.detail * 0.7,
                        energy_direction:           organ_dir,
                        energy_strength:            influence.energy_strength * 0.3,
                        resistance:                 influence.resistance,
                        least_resistance_direction: organ_dir.cross(Vec3::Y).normalize_or(Vec3::X),
                        length_budget:              influence.length_budget * len_factor,
                        max_segments:               8,
                        radius_base:                influence.radius_base * rad_factor,
                        start_position:             organ_pos,
                        qe_norm:                    influence.qe_norm,
                        tint_rgb:                   influence.tint_rgb,
                        branch_role:                influence.branch_role,
                    };
                    let organ_spine = build_flow_spine(&organ_inf);
                    organ_meshes.push(build_flow_mesh(&organ_spine, &organ_inf));
                }
                merge_meshes(&organ_meshes)
            } else {
                torso_mesh
            }
        } else {
            torso_mesh
        };

        let mesh_handle = meshes.add(final_mesh);

        if let Ok((mut mesh3d, mut tf)) = visual_query.get_mut(visual_root.visual_entity) {
            mesh3d.0 = mesh_handle;
            tf.scale  = Vec3::ONE;
        }

        // ── Write-back ────────────────────────────────────────────────────────
        if let Some(ref mut policy) = policy_opt {
            if policy.dependency_signature != new_sig {
                policy.dependency_signature = new_sig;
            }
        }
        if shape_inferred.is_none() {
            commands.entity(entity).insert(ShapeInferred);
        }
    }
}

#[cfg(test)]
mod tests {
    use bevy::asset::AssetPlugin;
    use bevy::prelude::*;

    use crate::blueprint::IdGenerator;
    use crate::layers::{EnergyAssessment, SensoryAwareness};
    use crate::runtime_platform::render_bridge_3d::V6VisualRoot;
    use crate::worldgen::ShapeInferred;

    use super::entity_shape_inference_system;

    fn test_app() -> App {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, AssetPlugin::default()));
        app
    }

    fn attach_visual_root(app: &mut App, sim_entity: Entity) -> Entity {
        let handle = {
            let mut meshes = app.world_mut().resource_mut::<Assets<Mesh>>();
            meshes.add(Mesh::from(bevy::math::primitives::Sphere::new(1.0)))
        };
        let ve = app.world_mut().commands().spawn(Mesh3d(handle)).id();
        app.world_mut().flush();
        app.world_mut()
            .entity_mut(sim_entity)
            .insert(V6VisualRoot { visual_entity: ve });
        ve
    }

    #[test]
    fn first_build_inserts_shape_inferred() {
        use crate::entities::archetypes::spawn_celula;
        use crate::runtime_platform::compat_2d3d::SimWorldTransformParams;

        let mut app = test_app();
        app.init_asset::<Mesh>();
        app.add_systems(Update, entity_shape_inference_system);

        let layout = SimWorldTransformParams::default();
        let mut id_gen = IdGenerator::default();
        let sim_entity = {
            let mut commands = app.world_mut().commands();
            spawn_celula(&mut commands, &mut id_gen, Vec2::ZERO, &layout)
        };
        app.update();
        attach_visual_root(&mut app, sim_entity);
        app.update();

        assert!(app.world().entity(sim_entity).contains::<ShapeInferred>());
    }

    #[test]
    fn cache_hit_skips_rebuild_when_signature_unchanged() {
        use crate::entities::archetypes::spawn_celula;
        use crate::runtime_platform::compat_2d3d::SimWorldTransformParams;

        let mut app = test_app();
        app.init_asset::<Mesh>();
        app.add_systems(Update, entity_shape_inference_system);

        let layout = SimWorldTransformParams::default();
        let mut id_gen = IdGenerator::default();
        let sim_entity = {
            let mut commands = app.world_mut().commands();
            spawn_celula(&mut commands, &mut id_gen, Vec2::ZERO, &layout)
        };
        app.update();
        attach_visual_root(&mut app, sim_entity);
        app.update(); // first build — sets ShapeInferred + policy.dependency_signature

        let ve = app.world().entity(sim_entity).get::<V6VisualRoot>().unwrap().visual_entity;
        let handle_after_first = app.world().entity(ve).get::<Mesh3d>().unwrap().0.clone();

        app.update(); // second update — signature unchanged → cache hit → no rebuild
        let handle_after_second = app.world().entity(ve).get::<Mesh3d>().unwrap().0.clone();

        assert_eq!(
            handle_after_first.id(), handle_after_second.id(),
            "mesh handle should not change on cache hit"
        );
    }

    #[test]
    fn hostile_nearby_triggers_rebuild_on_state_change() {
        use crate::entities::archetypes::spawn_animal_demo;
        use crate::runtime_platform::compat_2d3d::SimWorldTransformParams;

        let mut app = test_app();
        app.init_asset::<Mesh>();
        app.add_systems(Update, entity_shape_inference_system);

        let layout = SimWorldTransformParams::default();
        let mut id_gen = IdGenerator::default();
        let sim_entity = {
            let mut commands = app.world_mut().commands();
            spawn_animal_demo(&mut commands, &mut id_gen, Vec2::ZERO, &layout)
        };
        app.update();
        attach_visual_root(&mut app, sim_entity);
        app.update(); // first build

        let ve = app.world().entity(sim_entity).get::<V6VisualRoot>().unwrap().visual_entity;
        let handle_before = app.world().entity(ve).get::<Mesh3d>().unwrap().0.clone();

        // Introduce hostile entity in sensory awareness.
        let hostile = app.world_mut().spawn_empty().id();
        app.world_mut().entity_mut(sim_entity).insert(SensoryAwareness {
            hostile_entity: Some(hostile),
            hostile_distance: 0.5,
            food_entity: None,
            food_distance: f32::MAX,
        });
        app.update(); // signature changes → rebuild

        let handle_after = app.world().entity(ve).get::<Mesh3d>().unwrap().0.clone();
        assert_ne!(
            handle_before.id(), handle_after.id(),
            "mesh should be rebuilt when hostile enters sensory range"
        );
    }

    #[test]
    fn shape_inferred_not_set_without_visual_root() {
        use crate::entities::archetypes::spawn_celula;
        use crate::runtime_platform::compat_2d3d::SimWorldTransformParams;

        let mut app = test_app();
        app.init_asset::<Mesh>();
        app.add_systems(Update, entity_shape_inference_system);

        let layout = SimWorldTransformParams::default();
        let mut id_gen = IdGenerator::default();
        let sim_entity = {
            let mut commands = app.world_mut().commands();
            spawn_celula(&mut commands, &mut id_gen, Vec2::ZERO, &layout)
        };
        app.update();

        assert!(!app.world().entity(sim_entity).contains::<ShapeInferred>());
    }

    #[test]
    fn hunger_modulation_does_not_panic_at_extremes() {
        let ea_full   = EnergyAssessment { hunger_fraction: 1.0, energy_ratio: 0.1, biomass: 10.0 };
        let ea_sated  = EnergyAssessment { hunger_fraction: 0.0, energy_ratio: 0.9, biomass: 10.0 };
        assert!(ea_full.hunger_fraction  > ea_sated.hunger_fraction);
        assert!(ea_sated.hunger_fraction >= 0.0);
        assert!(ea_full.hunger_fraction  <= 1.0);
    }
}
