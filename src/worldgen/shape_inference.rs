//! Shape & color inference: derives [`GeometryInfluence`] from materialized energy
//! and quantized palette colors, bridging worldgen V7 → GF1 + Sprint 14.

use bevy::ecs::system::SystemParam;
use bevy::math::Affine3A;
use bevy::prelude::*;
use bevy::render::alpha::AlphaMode;
use bevy::render::mesh::VertexAttributeValues;

use crate::blueprint::almanac::AlchemicalAlmanac;
use crate::blueprint::constants::{
    BRANCH_MIN_BIOMASS, MAX_GROWTH_MORPH_PER_FRAME, SHAPE_INF_BASE_LENGTH,
    SHAPE_INF_BOND_RESISTANCE_SCALE, SHAPE_INF_DEFAULT_RESISTANCE, SHAPE_INF_DETAIL,
    SHAPE_INF_GRADIENT_BLEND, SHAPE_INF_MAX_PER_FRAME, SHAPE_INF_MAX_SEGMENTS,
    SHAPE_INF_QE_LENGTH_SCALE, SHAPE_INF_RADIUS_FACTOR, VISUAL_QE_REFERENCE,
};
use crate::blueprint::equations::{
    BranchRole, energy_gradient_2d, infer_organ_manifest, organ_manifest_inputs_from_state,
    quantized_palette_index, shape_inferred_direction, shape_inferred_length,
    shape_inferred_resistance,
};
use crate::geometry_flow::branching::{
    build_branched_tree_dyn, estimate_branch_cost, flatten_tree_to_mesh,
};
use crate::geometry_flow::{
    GeometryInfluence, build_flow_mesh, build_flow_spine_painted, spine_paint_vertex_from_raw_field,
};
use crate::layers::{
    BaseEnergy, CapabilitySet, GrowthBudget, InferenceProfile, LifecycleStageCache, MatterCoherence,
};
use crate::rendering::quantized_color::element_id_for_world_archetype;
use crate::rendering::quantized_color::{PaletteRegistry, QuantizedPrecision};
use crate::runtime_platform::compat_2d3d::RenderCompatProfile;
use crate::simulation::env_scenario::EffectiveOrganViability;
use crate::worldgen::contracts::Materialized;
use crate::worldgen::field_visual_sample::gf1_field_linear_rgb_qe_at_position;
use crate::worldgen::lod::{
    distance_sq_cell_to_focus, lod_band_from_distance_sq, materialization_tick_active_for_band,
};
use crate::worldgen::organ_inference::build_organ_mesh;
use crate::worldgen::{EnergyFieldGrid, EnergyVisual};
use crate::worldgen::{WorldgenLodContext, WorldgenPerfSettings};

/// Marker: entity already has an inferred GF1 mesh (SparseSet, no archetype thrash).
#[derive(Component, Clone, Copy, Debug, Default)]
#[component(storage = "SparseSet")]
pub struct ShapeInferred;

/// Per-frame budget counter for shape inference.
#[derive(Resource, Default, Debug)]
pub struct ShapeInferenceFrameState {
    pub processed_this_frame: u32,
    pub growth_morph_processed_this_frame: u32,
}

/// Marca de regeneración morfológica pendiente cuando el budget del frame no alcanza.
#[derive(Component, Clone, Copy, Debug, Default)]
#[component(storage = "SparseSet")]
pub struct PendingGrowthMorphRebuild;

#[derive(Clone, Copy, Debug)]
pub struct GeometryInferenceInput {
    pub cell_x: i32,
    pub cell_y: i32,
    pub world_pos: Vec3,
    pub qe: f32,
    pub bond_energy: Option<f32>,
    pub tint_rgb: [f32; 3],
    pub rho: f32,
}

type ShapeInferenceTuple<'a> = (
    Entity,
    &'a Materialized,
    &'a BaseEnergy,
    &'a EnergyVisual,
    &'a Transform,
    Option<&'a MatterCoherence>,
    Option<&'a QuantizedPrecision>,
    Option<&'a GrowthBudget>,
    Option<&'a LifecycleStageCache>,
    Option<&'a InferenceProfile>,
    Option<&'a CapabilitySet>,
    Option<&'a EffectiveOrganViability>,
    Option<&'a crate::layers::InferredAlbedo>,
);

type GrowthMorphTuple<'a> = (
    Entity,
    &'a Materialized,
    &'a BaseEnergy,
    &'a EnergyVisual,
    &'a Transform,
    Option<&'a MatterCoherence>,
    Option<&'a QuantizedPrecision>,
    &'a GrowthBudget,
    Option<&'a LifecycleStageCache>,
    Option<&'a InferenceProfile>,
    Option<&'a CapabilitySet>,
    Option<&'a EffectiveOrganViability>,
);

type ShapeInferenceQuery<'w, 's> =
    Query<'w, 's, ShapeInferenceTuple<'static>, Without<ShapeInferred>>;
type GrowthMorphQuery<'w, 's> = Query<
    'w,
    's,
    GrowthMorphTuple<'static>,
    (
        With<ShapeInferred>,
        Or<(
            Changed<GrowthBudget>,
            Changed<InferenceProfile>,
            Changed<CapabilitySet>,
            Changed<EffectiveOrganViability>,
            With<PendingGrowthMorphRebuild>,
        )>,
    ),
>;

#[derive(SystemParam)]
pub struct ShapeInferenceParams<'w, 's> {
    commands: Commands<'w, 's>,
    meshes: ResMut<'w, Assets<Mesh>>,
    materials: ResMut<'w, Assets<StandardMaterial>>,
    frame: ResMut<'w, ShapeInferenceFrameState>,
    perf: Res<'w, WorldgenPerfSettings>,
    lod: Res<'w, WorldgenLodContext>,
    profile: Option<Res<'w, RenderCompatProfile>>,
    grid: Option<Res<'w, EnergyFieldGrid>>,
    palette_reg: Res<'w, PaletteRegistry>,
    almanac: Res<'w, AlchemicalAlmanac>,
    query: ShapeInferenceQuery<'w, 's>,
}

#[derive(SystemParam)]
pub struct GrowthMorphParams<'w, 's> {
    commands: Commands<'w, 's>,
    meshes: ResMut<'w, Assets<Mesh>>,
    frame: ResMut<'w, ShapeInferenceFrameState>,
    perf: Res<'w, WorldgenPerfSettings>,
    lod: Res<'w, WorldgenLodContext>,
    profile: Option<Res<'w, RenderCompatProfile>>,
    grid: Option<Res<'w, EnergyFieldGrid>>,
    palette_reg: Res<'w, PaletteRegistry>,
    almanac: Res<'w, AlchemicalAlmanac>,
    query: GrowthMorphQuery<'w, 's>,
}

/// Resets the per-frame budget counter (runs before the inference system each frame).
pub fn reset_shape_inference_frame_system(mut state: ResMut<ShapeInferenceFrameState>) {
    state.processed_this_frame = 0;
    state.growth_morph_processed_this_frame = 0;
}

/// Look up the quantized palette color for an entity's archetype and energy state.
/// Returns `[r, g, b]` in linear space, or falls back to `EnergyVisual.color`.
fn palette_tint_rgb(
    archetype: crate::worldgen::archetypes::WorldArchetype,
    enorm: f32,
    rho: f32,
    registry: &PaletteRegistry,
) -> Option<[f32; 3]> {
    if registry.flat_rgba.is_empty() {
        return None;
    }
    let eid = element_id_for_world_archetype(archetype);
    let (offset, n_max) = registry.palette_meta_for_element_raw(eid.raw())?;
    if n_max == 0 {
        return None;
    }
    let idx = quantized_palette_index(enorm, rho, n_max);
    let flat_idx = offset as usize + idx as usize;
    let rgba = registry.flat_rgba.get(flat_idx)?;
    Some([rgba[0], rgba[1], rgba[2]])
}

/// Fallback: extract tint from `EnergyVisual.color`.
fn tint_from_visual(visual: &EnergyVisual) -> [f32; 3] {
    let lin = visual.color.to_linear();
    [lin.red, lin.green, lin.blue]
}

#[inline]
fn shape_inference_3d_enabled(profile: Option<&RenderCompatProfile>) -> bool {
    profile.map(|p| p.enables_visual_3d()).unwrap_or(false)
}

/// Entrada común a inferencia GF1 + EPI3 (un solo sitio para los dos sistemas que regeneran malla).
struct ResolvedMaterializedGf1 {
    influence: GeometryInfluence,
    tint_linear: [f32; 3],
    qe_norm: f32,
}

#[derive(Clone, Copy)]
struct OrganInferenceContext {
    stage: LifecycleStageCache,
    profile: InferenceProfile,
    capabilities: CapabilitySet,
    growth: GrowthBudget,
    effective_viability: Option<f32>,
    qe_norm: f32,
}

#[inline]
fn resolve_organ_inference_context(
    stage_cache: Option<&LifecycleStageCache>,
    profile: Option<&InferenceProfile>,
    capabilities: Option<&CapabilitySet>,
    growth: Option<GrowthBudget>,
    effective_viability: Option<&EffectiveOrganViability>,
    qe_norm: f32,
) -> Option<OrganInferenceContext> {
    let stage = (*stage_cache?).to_owned();
    let profile = profile.copied().unwrap_or_default();
    let capabilities = capabilities.copied().unwrap_or_default();
    let growth = growth.unwrap_or(GrowthBudget {
        biomass_available: 0.0,
        limiting_factor: 0,
        efficiency: 0.0,
    });
    Some(OrganInferenceContext {
        stage,
        profile,
        capabilities,
        growth,
        effective_viability: effective_viability.map(|v| v.value),
        qe_norm,
    })
}

#[inline]
fn infer_manifest_from_context(ctx: OrganInferenceContext) -> crate::layers::OrganManifest {
    let (growth_progress, fallback_viability) = organ_manifest_inputs_from_state(
        ctx.qe_norm,
        ctx.growth.efficiency,
        ctx.growth.biomass_available,
    );
    let viability = ctx.effective_viability.unwrap_or(fallback_viability);
    infer_organ_manifest(
        ctx.stage.stage,
        ctx.capabilities.flags,
        ctx.profile.growth_bias,
        ctx.profile.mobility_bias,
        ctx.profile.branching_bias,
        ctx.profile.resilience,
        ctx.growth.biomass_available,
        growth_progress,
        viability,
    )
}

#[inline]
fn resolve_organ_manifest(
    stage_cache: Option<&LifecycleStageCache>,
    profile: Option<&InferenceProfile>,
    capabilities: Option<&CapabilitySet>,
    growth: Option<GrowthBudget>,
    effective_viability: Option<&EffectiveOrganViability>,
    qe_norm: f32,
) -> Option<(crate::layers::OrganManifest, GrowthBudget)> {
    let ctx = resolve_organ_inference_context(
        stage_cache,
        profile,
        capabilities,
        growth,
        effective_viability,
        qe_norm,
    )?;
    Some((infer_manifest_from_context(ctx), ctx.growth))
}

fn resolve_materialized_gf1(
    materialized: &Materialized,
    energy: &BaseEnergy,
    visual: &EnergyVisual,
    transform: &Transform,
    coherence: Option<&MatterCoherence>,
    precision: Option<&QuantizedPrecision>,
    grid: &EnergyFieldGrid,
    palette_reg: &PaletteRegistry,
) -> ResolvedMaterializedGf1 {
    let qe = energy.qe().max(0.0);
    let qe_ref = VISUAL_QE_REFERENCE.max(1.0);
    let enorm = (qe / qe_ref).clamp(0.0, 1.0);
    let rho = precision.map(|p| p.0).unwrap_or(1.0);
    let bond = coherence.map(|c| c.bond_energy_eb());
    let tint = palette_tint_rgb(materialized.archetype, enorm, rho, palette_reg)
        .unwrap_or_else(|| tint_from_visual(visual));
    let input = GeometryInferenceInput {
        cell_x: materialized.cell_x,
        cell_y: materialized.cell_y,
        world_pos: transform.translation,
        qe,
        bond_energy: bond,
        tint_rgb: tint,
        rho,
    };
    let influence = derive_geometry_influence(&input, grid);
    ResolvedMaterializedGf1 {
        influence,
        tint_linear: tint,
        qe_norm: enorm,
    }
}

/// GF1 + EPI3: muestreo de campo por nodo/rama (`grid` + almanaque); fallback = tinte agregado de entidad.
#[inline]
pub(crate) fn build_shape_mesh(
    influence: &GeometryInfluence,
    growth: Option<GrowthBudget>,
    grid: &EnergyFieldGrid,
    almanac: &AlchemicalAlmanac,
    fallback_rgb: [f32; 3],
    fallback_qe_norm: f32,
) -> Mesh {
    let sample_field: &dyn Fn(Vec3) -> ([f32; 3], f32) = &|pos: Vec3| {
        gf1_field_linear_rgb_qe_at_position(
            grid,
            pos,
            almanac,
            VISUAL_QE_REFERENCE,
            // Sin reloj global aquí: compuestos quedan con fase de interferencia estática (ver doc en `field_visual_sample`).
            0.0,
            fallback_rgb,
            fallback_qe_norm,
        )
    };
    if let Some(g) = growth
        && g.biomass_available > BRANCH_MIN_BIOMASS
    {
        let root = build_branched_tree_dyn(influence, g.biomass_available, Some(sample_field));
        return flatten_tree_to_mesh(&root);
    }
    let spine = build_flow_spine_painted(influence, |pos, inf| {
        spine_paint_vertex_from_raw_field(pos, inf, sample_field)
    });
    build_flow_mesh(&spine, influence)
}

#[inline]
fn build_shape_or_organ_mesh(
    influence: &GeometryInfluence,
    growth: Option<GrowthBudget>,
    organ_manifest: Option<(&crate::layers::OrganManifest, GrowthBudget)>,
    grid: &EnergyFieldGrid,
    almanac: &AlchemicalAlmanac,
    fallback_rgb: [f32; 3],
    fallback_qe_norm: f32,
) -> Mesh {
    let Some((manifest, manifest_growth)) = organ_manifest else {
        return build_shape_mesh(
            influence,
            growth,
            grid,
            almanac,
            fallback_rgb,
            fallback_qe_norm,
        );
    };
    let spine = build_flow_spine_painted(influence, |pos, inf| {
        spine_paint_vertex_from_raw_field(pos, inf, &|p| {
            gf1_field_linear_rgb_qe_at_position(
                grid,
                p,
                almanac,
                VISUAL_QE_REFERENCE,
                0.0,
                fallback_rgb,
                fallback_qe_norm,
            )
        })
    });
    build_organ_mesh(
        manifest,
        &spine,
        influence,
        Some(manifest_growth),
        grid,
        almanac,
        fallback_rgb,
        fallback_qe_norm,
    )
}

#[inline]
fn estimate_organ_cost(
    influence: &GeometryInfluence,
    manifest: Option<&crate::layers::OrganManifest>,
) -> u32 {
    let Some(manifest) = manifest else {
        return 0;
    };
    let segments = influence.segment_count().max(1);
    manifest
        .iter()
        .map(|spec| {
            let prim_w = match spec.primitive() {
                crate::layers::GeometryPrimitive::Tube => 2u32,
                crate::layers::GeometryPrimitive::FlatSurface => 3u32,
                crate::layers::GeometryPrimitive::PetalFan => 4u32,
                crate::layers::GeometryPrimitive::Bulb => 5u32,
            };
            (spec.count() as u32)
                .max(1)
                .saturating_mul(prim_w)
                .saturating_mul(segments)
        })
        .sum::<u32>()
}

/// Coste de presupuesto alineado a muestreos EPI3: nodos del spine × `BranchNode` + re-muestreo en pivotes hijo.
#[inline]
fn shape_mesh_cost(
    influence: &GeometryInfluence,
    growth: Option<GrowthBudget>,
    manifest: Option<&crate::layers::OrganManifest>,
) -> u32 {
    let spine_samples = influence.segment_count().saturating_add(1).max(1);
    let base = if let Some(g) = growth
        && g.biomass_available > BRANCH_MIN_BIOMASS
    {
        let b = estimate_branch_cost(g.biomass_available);
        b.saturating_mul(spine_samples)
            .saturating_add(b.saturating_sub(1))
            .max(1)
    } else {
        spine_samples.max(1)
    };
    base.saturating_add(estimate_organ_cost(influence, manifest))
}

#[inline]
fn shape_rebuild_tick_active(
    materialized: &Materialized,
    transform: &Transform,
    grid: &EnergyFieldGrid,
    lod: &WorldgenLodContext,
    perf: &WorldgenPerfSettings,
) -> bool {
    let center = grid
        .world_pos(
            materialized.cell_x.max(0) as u32,
            materialized.cell_y.max(0) as u32,
        )
        .unwrap_or(bevy::math::Vec2::new(
            transform.translation.x,
            transform.translation.z,
        ));
    let dsq = distance_sq_cell_to_focus(center, lod.focus_world);
    let band = lod_band_from_distance_sq(dsq);
    materialization_tick_active_for_band(
        band,
        lod.sim_tick,
        perf.shape_rebuild_mid_period,
        perf.shape_rebuild_far_period,
    )
}

/// `build_shape_mesh` / GF1 emiten geometría en **espacio mundo**. Las celdas full3d usan
/// `Transform` con rotación −90° en X para el `Sprite` plano; si aplicamos esa rotación al tubo,
/// los vértices quedan basculados y parecen “tubos al cielo”. Reexpresamos posiciones y normales
/// al espacio **local** de la entidad (`inv(affine)` para puntos; normales vía transpuesta lineal).
fn bake_world_mesh_to_entity_local(mesh: &mut Mesh, transform: &Transform) {
    let affine = Affine3A::from_scale_rotation_translation(
        transform.scale,
        transform.rotation,
        transform.translation,
    );
    let inv = affine.inverse();
    if let Some(attr) = mesh.attribute_mut(Mesh::ATTRIBUTE_POSITION) {
        if let VertexAttributeValues::Float32x3(v) = attr {
            for p in v.iter_mut() {
                let w = Vec3::from_array(*p);
                *p = inv.transform_point3(w).to_array();
            }
        }
    }
    if let Some(attr) = mesh.attribute_mut(Mesh::ATTRIBUTE_NORMAL) {
        if let VertexAttributeValues::Float32x3(v) = attr {
            let m3 = affine.matrix3;
            for n in v.iter_mut() {
                let nw = Vec3::from_array(*n);
                *n = (m3.transpose() * nw).normalize_or_zero().to_array();
            }
        }
    }
}

/// Derive a `GeometryInfluence` from a materialized entity's energy context.
///
/// Pure-ish: reads grid neighbors for gradient, entity components for shape params.
pub fn derive_geometry_influence(
    input: &GeometryInferenceInput,
    grid: &EnergyFieldGrid,
) -> GeometryInfluence {
    let cx = input.cell_x.max(0) as u32;
    let cy = input.cell_y.max(0) as u32;

    // Neighbor qe for gradient computation.
    let qe_left = grid
        .cell_xy(cx.wrapping_sub(1), cy)
        .map(|c| c.accumulated_qe)
        .unwrap_or(0.0);
    let qe_right = grid
        .cell_xy(cx + 1, cy)
        .map(|c| c.accumulated_qe)
        .unwrap_or(0.0);
    let qe_down = grid
        .cell_xy(cx, cy.wrapping_sub(1))
        .map(|c| c.accumulated_qe)
        .unwrap_or(0.0);
    let qe_up = grid
        .cell_xy(cx, cy + 1)
        .map(|c| c.accumulated_qe)
        .unwrap_or(0.0);

    let gradient = energy_gradient_2d(qe_left, qe_right, qe_down, qe_up);
    let energy_direction = shape_inferred_direction(gradient, SHAPE_INF_GRADIENT_BLEND);

    let qe_ref = VISUAL_QE_REFERENCE.max(1.0);
    let qe_norm = (input.qe / qe_ref).clamp(0.0, 1.0);

    let resistance = shape_inferred_resistance(
        input.bond_energy.unwrap_or(0.0),
        SHAPE_INF_DEFAULT_RESISTANCE,
        SHAPE_INF_BOND_RESISTANCE_SCALE,
    );
    let length = shape_inferred_length(qe_norm, SHAPE_INF_BASE_LENGTH, SHAPE_INF_QE_LENGTH_SCALE);
    let radius = grid.cell_size * SHAPE_INF_RADIUS_FACTOR;

    // Least resistance: perpendicular to gradient in XZ, biased toward the gradient if weak.
    let least = if gradient.length_squared() > 1e-6 {
        let perp = Vec2::new(-gradient.y, gradient.x).normalize_or_zero();
        Vec3::new(perp.x, 0.0, perp.y)
    } else {
        Vec3::X
    };

    GeometryInfluence {
        detail: input.rho.clamp(0.0, 1.0) * SHAPE_INF_DETAIL,
        energy_direction,
        energy_strength: qe_norm * 3.0,
        resistance,
        least_resistance_direction: least,
        length_budget: length,
        max_segments: SHAPE_INF_MAX_SEGMENTS,
        radius_base: radius,
        start_position: input.world_pos,
        qe_norm,
        tint_rgb: input.tint_rgb,
        branch_role: BranchRole::default(),
    }
}

/// Infers GF1 mesh shape + quantized palette color for materialized entities (3D mode only).
///
/// Presupuesto: suma de coste por entidad (`shape_mesh_cost`, ~muestreos EPI3) hasta `SHAPE_INF_MAX_PER_FRAME`.
/// Replaces `Sprite` with `Mesh3d` + `MeshMaterial3d` and inserts `ShapeInferred` marker.
pub fn shape_color_inference_system(mut p: ShapeInferenceParams) {
    if !shape_inference_3d_enabled(p.profile.as_deref()) {
        return;
    }

    let Some(grid) = p.grid.as_ref() else {
        return;
    };

    let mut entities: Vec<Entity> = p.query.iter().map(|(entity, ..)| entity).collect();
    entities.sort_by_key(|entity| entity.to_bits());
    for entity in entities {
        let Ok((
            entity,
            materialized,
            energy,
            visual,
            transform,
            coherence,
            precision,
            growth,
            stage_cache,
            profile,
            capabilities,
            effective_viability,
            inferred_albedo,
        )) = p.query.get(entity)
        else {
            continue;
        };
        if !shape_rebuild_tick_active(materialized, transform, grid, &p.lod, &p.perf) {
            continue;
        }

        let r = resolve_materialized_gf1(
            materialized,
            energy,
            visual,
            transform,
            coherence,
            precision,
            grid,
            &p.palette_reg,
        );
        let organ_manifest = resolve_organ_manifest(
            stage_cache,
            profile,
            capabilities,
            growth.copied(),
            effective_viability,
            r.qe_norm,
        );
        let cost = shape_mesh_cost(
            &r.influence,
            growth.copied(),
            organ_manifest.as_ref().map(|(m, _)| m),
        );
        if p.frame.processed_this_frame.saturating_add(cost) > SHAPE_INF_MAX_PER_FRAME
            && p.frame.processed_this_frame > 0
        {
            break;
        }
        p.frame.processed_this_frame += cost;

        // MG-5E: modular tint por albedo inferido (luminosidad). Sin InferredAlbedo → sin cambio.
        let tint = if let Some(albedo) = inferred_albedo {
            let factor = crate::blueprint::equations::albedo_luminosity_blend(1.0, albedo.albedo());
            [
                r.tint_linear[0] * factor,
                r.tint_linear[1] * factor,
                r.tint_linear[2] * factor,
            ]
        } else {
            r.tint_linear
        };

        let mut mesh = build_shape_or_organ_mesh(
            &r.influence,
            growth.copied(),
            organ_manifest.as_ref().map(|(m, g)| (m, *g)),
            grid,
            &p.almanac,
            tint,
            r.qe_norm,
        );
        bake_world_mesh_to_entity_local(&mut mesh, transform);
        let mesh_handle = p.meshes.add(mesh);
        let mat = p.materials.add(StandardMaterial {
            base_color: Color::WHITE,
            // LI6: vertex alpha (opacidad por OrganRole) requiere material en blend.
            alpha_mode: AlphaMode::Blend,
            perceptual_roughness: 0.55,
            metallic: 0.05,
            ..default()
        });

        p.commands
            .entity(entity)
            .insert((ShapeInferred, Mesh3d(mesh_handle), MeshMaterial3d(mat)));
        // Remove Sprite so the flat tile no longer renders.
        p.commands.entity(entity).remove::<Sprite>();
    }
}

/// Re-genera la malla si cambia `GrowthBudget` (marchitamiento / rebrote).
pub fn growth_morphology_system(mut p: GrowthMorphParams) {
    if !shape_inference_3d_enabled(p.profile.as_deref()) {
        return;
    }
    let Some(grid) = p.grid.as_ref() else {
        return;
    };

    let mut entities: Vec<Entity> = p.query.iter().map(|(entity, ..)| entity).collect();
    entities.sort_by_key(|entity| entity.to_bits());
    for entity in entities {
        let Ok((
            entity,
            materialized,
            energy,
            visual,
            transform,
            coherence,
            precision,
            growth,
            stage_cache,
            profile,
            capabilities,
            effective_viability,
        )) = p.query.get(entity)
        else {
            continue;
        };
        if !shape_rebuild_tick_active(materialized, transform, grid, &p.lod, &p.perf) {
            p.commands.entity(entity).insert(PendingGrowthMorphRebuild);
            continue;
        }

        let r = resolve_materialized_gf1(
            materialized,
            energy,
            visual,
            transform,
            coherence,
            precision,
            grid,
            &p.palette_reg,
        );
        let organ_manifest = resolve_organ_manifest(
            stage_cache,
            profile,
            capabilities,
            Some(*growth),
            effective_viability,
            r.qe_norm,
        );
        let cost = shape_mesh_cost(
            &r.influence,
            Some(*growth),
            organ_manifest.as_ref().map(|(m, _)| m),
        );
        let used = p.frame.growth_morph_processed_this_frame;
        if used.saturating_add(cost) > MAX_GROWTH_MORPH_PER_FRAME && used > 0 {
            p.commands.entity(entity).insert(PendingGrowthMorphRebuild);
            continue;
        }
        p.frame.growth_morph_processed_this_frame += cost;

        let mut mesh = build_shape_or_organ_mesh(
            &r.influence,
            Some(*growth),
            organ_manifest.as_ref().map(|(m, g)| (m, *g)),
            grid,
            &p.almanac,
            r.tint_linear,
            r.qe_norm,
        );
        bake_world_mesh_to_entity_local(&mut mesh, transform);
        let mesh_handle = p.meshes.add(mesh);
        p.commands
            .entity(entity)
            .insert(Mesh3d(mesh_handle))
            .remove::<PendingGrowthMorphRebuild>();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::worldgen::archetypes::WorldArchetype;

    #[test]
    fn derive_geometry_influence_returns_valid_influence() {
        let mut grid = EnergyFieldGrid::new(4, 4, 1.0, bevy::math::Vec2::ZERO);
        // Set center cell and neighbors with some energy.
        if let Some(c) = grid.cell_xy_mut(1, 1) {
            c.accumulated_qe = 200.0;
        }
        if let Some(c) = grid.cell_xy_mut(2, 1) {
            c.accumulated_qe = 400.0;
        }
        if let Some(c) = grid.cell_xy_mut(0, 1) {
            c.accumulated_qe = 100.0;
        }

        let inf = derive_geometry_influence(
            &GeometryInferenceInput {
                cell_x: 1,
                cell_y: 1,
                world_pos: Vec3::new(1.5, 0.0, 1.5),
                qe: 200.0,
                bond_energy: Some(1500.0),
                tint_rgb: [0.8, 0.2, 0.1],
                rho: 1.0,
            },
            &grid,
        );

        assert!(inf.length_budget > 0.0);
        assert!(inf.energy_direction.length() > 0.9);
        assert!(inf.radius_base > 0.0);
        assert!(inf.qe_norm >= 0.0 && inf.qe_norm <= 1.0);
    }

    #[test]
    fn palette_tint_returns_none_when_registry_empty() {
        let reg = PaletteRegistry::default();
        assert!(palette_tint_rgb(WorldArchetype::IgnisSolid, 0.5, 1.0, &reg).is_none());
    }

    #[test]
    fn derive_influence_zero_energy_still_valid() {
        let grid = EnergyFieldGrid::new(4, 4, 1.0, bevy::math::Vec2::ZERO);
        let inf = derive_geometry_influence(
            &GeometryInferenceInput {
                cell_x: 1,
                cell_y: 1,
                world_pos: Vec3::ZERO,
                qe: 0.0,
                bond_energy: None,
                tint_rgb: [0.5, 0.5, 0.5],
                rho: 1.0,
            },
            &grid,
        );
        assert!(inf.length_budget >= 0.1);
        assert!(inf.energy_direction.is_normalized());
    }

    #[test]
    fn build_shape_mesh_epi3_deterministic_same_inputs() {
        use crate::geometry_flow::flow_mesh_triangle_count;

        let almanac = AlchemicalAlmanac::default();
        let mut grid = EnergyFieldGrid::new(4, 4, 1.0, bevy::math::Vec2::ZERO);
        if let Some(c) = grid.cell_xy_mut(1, 1) {
            c.dominant_frequency_hz = 200.0;
            c.purity = 1.0;
            c.accumulated_qe = 150.0;
        }
        let input = GeometryInferenceInput {
            cell_x: 1,
            cell_y: 1,
            world_pos: Vec3::new(1.5, 0.0, 1.5),
            qe: 150.0,
            bond_energy: None,
            tint_rgb: [0.2, 0.5, 0.3],
            rho: 1.0,
        };
        let inf = derive_geometry_influence(&input, &grid);
        let m1 = super::build_shape_mesh(&inf, None, &grid, &almanac, input.tint_rgb, inf.qe_norm);
        let m2 = super::build_shape_mesh(&inf, None, &grid, &almanac, input.tint_rgb, inf.qe_norm);
        assert_eq!(flow_mesh_triangle_count(&m1), flow_mesh_triangle_count(&m2));
    }

    #[test]
    fn shape_mesh_cost_increases_with_detail_rho() {
        let grid = EnergyFieldGrid::new(4, 4, 1.0, bevy::math::Vec2::ZERO);
        let base = GeometryInferenceInput {
            cell_x: 1,
            cell_y: 1,
            world_pos: Vec3::ZERO,
            qe: 100.0,
            bond_energy: None,
            tint_rgb: [0.5, 0.5, 0.5],
            rho: 1.0,
        };
        let inf_hi = derive_geometry_influence(&base, &grid);
        let inf_lo = derive_geometry_influence(&GeometryInferenceInput { rho: 0.0, ..base }, &grid);
        let c_hi = super::shape_mesh_cost(&inf_hi, None, None);
        let c_lo = super::shape_mesh_cost(&inf_lo, None, None);
        assert!(
            c_hi >= c_lo,
            "más ρ (LOD) → más segmentos y mayor coste de muestreo EPI3"
        );
    }

    #[test]
    fn build_shape_or_organ_mesh_without_lifecycle_cache_falls_back_to_shape_mesh() {
        use crate::geometry_flow::flow_mesh_triangle_count;

        let almanac = AlchemicalAlmanac::default();
        let mut grid = EnergyFieldGrid::new(4, 4, 1.0, bevy::math::Vec2::ZERO);
        if let Some(c) = grid.cell_xy_mut(1, 1) {
            c.dominant_frequency_hz = 200.0;
            c.purity = 1.0;
            c.accumulated_qe = 150.0;
        }
        let input = GeometryInferenceInput {
            cell_x: 1,
            cell_y: 1,
            world_pos: Vec3::new(1.5, 0.0, 1.5),
            qe: 150.0,
            bond_energy: None,
            tint_rgb: [0.2, 0.5, 0.3],
            rho: 1.0,
        };
        let inf = derive_geometry_influence(&input, &grid);
        let base =
            super::build_shape_mesh(&inf, None, &grid, &almanac, input.tint_rgb, inf.qe_norm);
        let fallback = super::build_shape_or_organ_mesh(
            &inf,
            None,
            None,
            &grid,
            &almanac,
            input.tint_rgb,
            inf.qe_norm,
        );
        assert_eq!(
            flow_mesh_triangle_count(&base),
            flow_mesh_triangle_count(&fallback)
        );
    }
}
