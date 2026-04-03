use bevy::ecs::schedule::ScheduleLabel;
use bevy::prelude::*;

use crate::blueprint::constants::{
    ACTUATOR_FALLBACK_BUFFER_MAX, ACTUATOR_MATTER_LOW_VELOCITY_CAP, ACTUATOR_VELOCITY_LIMIT,
    ACTUATOR_VELOCITY_SQ_TRACE_EPSILON, COLLISION_CONDUCTIVITY_BLEND, FRICTION_COEF,
    MAX_GLOBAL_VELOCITY, THERMAL_CONDUCTIVITY_FALLBACK,
};
use crate::blueprint::equations;
use crate::eco::EcoPlayfieldMargin;
use crate::eco::context_lookup::ContextLookup;
use crate::events::{CollisionEvent, DeathCause};
use crate::layers::{
    AlchemicalEngine, AmbientPressure, BaseEnergy, EnergyOps, FlowVector, InterferenceOps,
    MatterCoherence, MatterState, ResonanceFlowOverlay, ResonanceThermalOverlay, SpatialVolume,
    WillActuator,
};
use crate::runtime_platform::compat_2d3d::SimWorldTransformParams;
use crate::runtime_platform::core_math_agnostic::{sim_plane_pos, vec2_to_xz};
use crate::simulation::Phase;
use crate::simulation::PlayerControlled;
use crate::simulation::structural_runtime;
use crate::simulation::time_compat::simulation_delta_secs;
use crate::topology::{TerrainField, TerrainType};
use crate::world::{SpatialEntry, SpatialIndex, update_spatial_index_after_move_system};
use crate::worldgen::EnergyFieldGrid;

// ── Modificadores de travesía por terreno (T8 gameplay) ──
// ── Terrain traverse cost modifiers (T8 gameplay) ──
// Positivo = más lento, negativo = más rápido.
// Positive = slower, negative = faster.

/// Bonus de velocidad para líquidos en cauces y cuencas.
/// Speed bonus for liquids in riverbeds and basins.
const LIQUID_WATER_CHANNEL_BONUS: f32 = -0.3;

/// Penalización para líquidos en pendientes y acantilados.
/// Penalty for liquids on slopes and cliffs.
const LIQUID_SLOPE_PENALTY: f32 = 0.2;

/// Penalización para sólidos en picos y acantilados.
/// Penalty for solids on peaks and cliffs.
const SOLID_CLIFF_PENALTY: f32 = 0.5;

/// Penalización para sólidos en pendientes.
/// Penalty for solids on slopes.
const SOLID_SLOPE_PENALTY: f32 = 0.2;

/// Bonus de velocidad para sólidos en valles y cauces.
/// Speed bonus for solids in valleys and riverbeds.
const SOLID_VALLEY_BONUS: f32 = -0.1;

/// Costo de travesía por topología (T8 gameplay). Positivo = más lento, negativo = más rápido.
/// Traverse cost by topology (T8 gameplay). Positive = slower, negative = faster.
pub fn traverse_cost_modifier(terrain_type: TerrainType, entity_state: MatterState) -> f32 {
    match entity_state {
        MatterState::Gas | MatterState::Plasma => 0.0,
        MatterState::Liquid => match terrain_type {
            TerrainType::Riverbed | TerrainType::Basin => LIQUID_WATER_CHANNEL_BONUS,
            TerrainType::Slope | TerrainType::Cliff => LIQUID_SLOPE_PENALTY,
            _ => 0.0,
        },
        MatterState::Solid => match terrain_type {
            TerrainType::Peak | TerrainType::Cliff => SOLID_CLIFF_PENALTY,
            TerrainType::Slope => SOLID_SLOPE_PENALTY,
            TerrainType::Valley | TerrainType::Riverbed => SOLID_VALLEY_BONUS,
            _ => 0.0,
        },
    }
}

/// Fricción adicional por pendiente.
///
/// Convención: `aspect` marca dirección de descenso; moverse en sentido contrario implica subida.
pub fn slope_friction(slope: f32, movement_direction: Vec2, aspect: f32) -> f32 {
    if !slope.is_finite() || slope <= 0.0 {
        return 0.0;
    }
    let dir = movement_direction.normalize_or_zero();
    if dir == Vec2::ZERO {
        return 0.0;
    }
    let radians = aspect.to_radians();
    let downhill = Vec2::new(radians.cos(), radians.sin()).normalize_or_zero();
    if downhill == Vec2::ZERO {
        return 0.0;
    }

    // alignment > 0: bajando; alignment < 0: subiendo.
    let alignment = dir.dot(downhill).clamp(-1.0, 1.0);
    let slope01 = (slope / 90.0).clamp(0.0, 1.0);
    -alignment * slope01
}

// terrain_blocks_vision moved to blueprint/equations/vision.rs (DC-4)

/// Sistema: Disipación entrópica (Segunda Ley del juego).
/// Fase: Phase::AtomicLayer
pub fn dissipation_system(
    fixed: Option<Res<Time<Fixed>>>,
    time: Res<Time>,
    layout: Res<SimWorldTransformParams>,
    ctx_lookup: ContextLookup,
    mut energy_ops: EnergyOps,
    query: Query<(
        Entity,
        &FlowVector,
        &Transform,
        Option<&MatterCoherence>,
        Option<&ResonanceFlowOverlay>,
    )>,
) {
    let dt = simulation_delta_secs(fixed, &time);
    let xz = layout.use_xz_ground;

    for (entity, flow_vec, transform, matter_opt, overlay_opt) in query.iter() {
        let pos = sim_plane_pos(transform.translation, xz);
        let ctx = ctx_lookup.context_at(pos);
        let mut rate = flow_vec.effective_dissipation(FRICTION_COEF);
        rate *= ctx.dissipation_mod.max(0.0);

        if let Some(matter) = matter_opt {
            rate *= matter.dissipation_multiplier();
        }
        if let Some(overlay) = overlay_opt {
            rate *= overlay.dissipation_multiplier.max(0.0);
        }

        let amount = rate * dt;
        if amount > 0.0 {
            energy_ops.drain(entity, amount, DeathCause::Dissipation);
        }
    }
}

/// Integra fuerzas de voluntad + arrastre en velocidad (sin desplazar Transform).
/// Fase: Phase::AtomicLayer
#[allow(dead_code)]
pub fn movement_will_drag_system(
    fixed: Option<Res<Time<Fixed>>>,
    time: Res<Time>,
    layout: Res<SimWorldTransformParams>,
    ctx_lookup: ContextLookup,
    mut query: Query<(
        Entity,
        &mut FlowVector,
        &BaseEnergy,
        &SpatialVolume,
        &Transform,
        Option<&WillActuator>,
        Option<&AlchemicalEngine>,
        Option<&MatterCoherence>,
        Option<&ResonanceFlowOverlay>,
    )>,
) {
    let dt = simulation_delta_secs(fixed, &time);
    if dt <= 0.0 {
        return;
    }
    let xz = layout.use_xz_ground;

    for (
        entity,
        mut flow,
        energy,
        volume,
        transform,
        actuator_opt,
        engine_opt,
        matter_opt,
        overlay_opt,
    ) in query.iter_mut()
    {
        if let Some(matter) = matter_opt {
            if matter.state() == MatterState::Solid && actuator_opt.is_none() {
                flow.set_velocity(Vec2::ZERO, None);
                continue;
            }
        }

        let will_force_vec = if let Some(actuator) = actuator_opt {
            if actuator.can_move() {
                let buffer = engine_opt.map(|e| e.buffer_level()).unwrap_or(energy.qe());
                let buffer_max = engine_opt
                    .map(|e| e.buffer_cap())
                    .unwrap_or(ACTUATOR_FALLBACK_BUFFER_MAX);
                equations::will_force(actuator.movement_intent(), buffer, buffer_max)
            } else {
                Vec2::ZERO
            }
        } else {
            Vec2::ZERO
        };

        let pos = sim_plane_pos(transform.translation, xz);
        // Viscosidad de eco (grid V7). Coexiste con `contained_thermal_transfer_system` (Capa 6
        // `terrain_viscosity`); el contrato dual está en `docs/design/ECO_BOUNDARIES.md` §2 / §8.
        let visc = ctx_lookup.context_at(pos).viscosity.max(0.0);
        let drag = equations::drag_force(visc, volume.density(energy.qe()), flow.velocity());
        let net_force = will_force_vec + drag;

        let v_integrated =
            equations::integrate_velocity(flow.velocity(), net_force, energy.qe(), dt);
        flow.set_velocity(v_integrated, None);

        if actuator_opt.is_some()
            && flow.velocity().length_squared() > ACTUATOR_VELOCITY_SQ_TRACE_EPSILON
        {
            trace!("Héroe {:?} velocidad: {:?}", entity, flow.velocity());
        }

        let mut limit = matter_opt
            .and_then(|m| m.velocity_limit())
            .unwrap_or(MAX_GLOBAL_VELOCITY);

        if actuator_opt.is_some() && limit < ACTUATOR_MATTER_LOW_VELOCITY_CAP {
            limit = ACTUATOR_VELOCITY_LIMIT;
        }

        limit = limit.min(MAX_GLOBAL_VELOCITY);

        let mut v = flow.velocity();
        if let Some(overlay) = overlay_opt {
            v *= overlay.velocity_multiplier.max(0.0);
        }
        let speed = v.length();
        v = v.clamp_length_max(limit);
        if speed > limit && speed > 0.0 {
            v = v.normalize() * limit;
        }
        flow.set_velocity(v, None);
    }
}

/// Aplica fuerzas de voluntad + arrastre eco → velocidad integrada. Sin SolidLock si tiene actuador.
/// Fase: Phase::AtomicLayer
pub fn will_to_velocity_system(
    fixed: Option<Res<Time<Fixed>>>,
    time: Res<Time>,
    layout: Res<SimWorldTransformParams>,
    ctx_lookup: ContextLookup,
    mut query: Query<(
        Entity,
        &mut FlowVector,
        &BaseEnergy,
        &SpatialVolume,
        &Transform,
        Option<&WillActuator>,
        Option<&AlchemicalEngine>,
        Option<&MatterCoherence>,
    )>,
) {
    let dt = simulation_delta_secs(fixed, &time);
    if dt <= 0.0 {
        return;
    }
    let xz = layout.use_xz_ground;

    for (entity, mut flow, energy, volume, transform, actuator_opt, engine_opt, matter_opt) in
        query.iter_mut()
    {
        if let Some(matter) = matter_opt {
            if matter.state() == MatterState::Solid && actuator_opt.is_none() {
                flow.set_velocity(Vec2::ZERO, None);
                continue;
            }
        }

        let will_force_vec = if let Some(actuator) = actuator_opt {
            if actuator.can_move() {
                let buffer = engine_opt.map(|e| e.buffer_level()).unwrap_or(energy.qe());
                let buffer_max = engine_opt
                    .map(|e| e.buffer_cap())
                    .unwrap_or(ACTUATOR_FALLBACK_BUFFER_MAX);
                equations::will_force(actuator.movement_intent(), buffer, buffer_max)
            } else {
                Vec2::ZERO
            }
        } else {
            Vec2::ZERO
        };

        let pos = sim_plane_pos(transform.translation, xz);
        // Viscosidad de eco (grid V7). Coexiste con containment drag (Capa 6 terrain_viscosity);
        // contrato dual en `docs/design/ECO_BOUNDARIES.md` §2 / §8.
        let visc = ctx_lookup.context_at(pos).viscosity.max(0.0);
        let drag = equations::drag_force(visc, volume.density(energy.qe()), flow.velocity());
        let net_force = will_force_vec + drag;

        let v_integrated =
            equations::integrate_velocity(flow.velocity(), net_force, energy.qe(), dt);
        if flow.velocity() != v_integrated {
            flow.set_velocity(v_integrated, None);
        }

        if actuator_opt.is_some()
            && flow.velocity().length_squared() > ACTUATOR_VELOCITY_SQ_TRACE_EPSILON
        {
            trace!("Héroe {:?} velocidad: {:?}", entity, flow.velocity());
        }
    }
}

/// Aplica overlay de velocidad + clamp global al límite de materia/actuador.
/// Fase: Phase::AtomicLayer — después de `will_to_velocity_system`.
pub fn velocity_cap_system(
    mut query: Query<(
        &mut FlowVector,
        Option<&WillActuator>,
        Option<&MatterCoherence>,
        Option<&ResonanceFlowOverlay>,
    )>,
) {
    for (mut flow, actuator_opt, matter_opt, overlay_opt) in query.iter_mut() {
        let mut limit = matter_opt
            .and_then(|m| m.velocity_limit())
            .unwrap_or(MAX_GLOBAL_VELOCITY);

        if actuator_opt.is_some() && limit < ACTUATOR_MATTER_LOW_VELOCITY_CAP {
            limit = ACTUATOR_VELOCITY_LIMIT;
        }
        limit = limit.min(MAX_GLOBAL_VELOCITY);

        let mut v = flow.velocity();
        if let Some(overlay) = overlay_opt {
            v *= overlay.velocity_multiplier.max(0.0);
        }
        let speed = v.length();
        v = v.clamp_length_max(limit);
        if speed > limit && speed > 0.0 {
            v = v.normalize() * limit;
        }
        if flow.velocity() != v {
            flow.set_velocity(v, None);
        }
    }
}

/// Aplica velocidad integrada al Transform (separado de fuerzas).
/// Fase: Phase::AtomicLayer
pub fn movement_integrate_transform_system(
    fixed: Option<Res<Time<Fixed>>>,
    time: Res<Time>,
    layout: Res<SimWorldTransformParams>,
    mut query: Query<(&FlowVector, &mut Transform)>,
) {
    let dt = simulation_delta_secs(fixed, &time);
    if dt <= 0.0 {
        return;
    }

    for (flow, mut transform) in &mut query {
        let delta = if layout.use_xz_ground {
            vec2_to_xz(flow.velocity()) * dt
        } else {
            flow.velocity().extend(0.0) * dt
        };
        if delta != Vec3::ZERO {
            transform.translation += delta;
        }
    }
}

/// Clampea posición 2D del plano sim al rectángulo **jugable** (grid menos `margin_cells` por borde).
fn clamp_sim_pos_to_grid_bounds(
    pos: Vec2,
    grid: &EnergyFieldGrid,
    radius: f32,
    margin_cells: u32,
) -> Vec2 {
    let inset = margin_cells as f32 * grid.cell_size;
    let full_w = grid.width as f32 * grid.cell_size;
    let full_h = grid.height as f32 * grid.cell_size;
    let playable_w = full_w - 2.0 * inset;
    let playable_h = full_h - 2.0 * inset;
    if playable_w <= 0.0 || playable_h <= 0.0 {
        return pos;
    }
    let r = radius.max(0.0);

    let mut min_x = grid.origin.x + inset + r;
    let mut max_x = grid.origin.x + inset + playable_w - r;
    if min_x > max_x {
        let c = grid.origin.x + inset + playable_w * 0.5;
        min_x = c;
        max_x = c;
    }

    let mut min_y = grid.origin.y + inset + r;
    let mut max_y = grid.origin.y + inset + playable_h - r;
    if min_y > max_y {
        let c = grid.origin.y + inset + playable_h * 0.5;
        min_y = c;
        max_y = c;
    }

    Vec2::new(pos.x.clamp(min_x, max_x), pos.y.clamp(min_y, max_y))
}

/// Demo / full3d: sin collider de malla contra `V6GroundPlane`, el héroe es cinemático en XZ.
/// Ancla Y a `standing_y` y evita salirse del área del campo (misma envolvente que el mosaico).
///
/// Fase: `Phase::AtomicLayer`, **después** de integrar velocidad y **antes** del índice espacial.
pub fn player_demo_walk_constraint_system(
    layout: Res<SimWorldTransformParams>,
    grid: Option<Res<EnergyFieldGrid>>,
    margin: Option<Res<EcoPlayfieldMargin>>,
    mut query: Query<(&mut Transform, Option<&SpatialVolume>), With<PlayerControlled>>,
) {
    if !layout.use_xz_ground {
        return;
    }
    let Some(grid) = grid else {
        return;
    };
    let m = margin.map(|r| r.cells).unwrap_or(0);

    for (mut transform, vol_opt) in &mut query {
        let r = vol_opt.map(|v| v.radius).unwrap_or(0.5);
        let plane = sim_plane_pos(transform.translation, true);
        let clamped = clamp_sim_pos_to_grid_bounds(plane, grid.as_ref(), r, m);

        let nx = clamped.x;
        let ny = layout.standing_y;
        let nz = clamped.y;
        if transform.translation.x != nx
            || transform.translation.y != ny
            || transform.translation.z != nz
        {
            transform.translation = Vec3::new(nx, ny, nz);
        }
    }
}

/// Modulación de movimiento por relieve (T8 gameplay).
/// Se ejecuta después de integrar voluntad+drag y antes de aplicar desplazamiento.
pub fn terrain_effects_system(
    layout: Res<SimWorldTransformParams>,
    terrain: Option<Res<TerrainField>>,
    mut query: Query<(&mut FlowVector, &Transform, Option<&MatterCoherence>)>,
) {
    let Some(terrain) = terrain else {
        return;
    };
    let xz = layout.use_xz_ground;

    for (mut flow, transform, matter_opt) in &mut query {
        let Some(matter) = matter_opt else {
            continue;
        };
        if matches!(matter.state(), MatterState::Gas | MatterState::Plasma) {
            continue;
        }
        let pos = sim_plane_pos(transform.translation, xz);
        let Some(sample) = terrain.sample_at_world(pos) else {
            continue;
        };

        let traverse = traverse_cost_modifier(sample.terrain_type, matter.state());
        let slope = slope_friction(sample.slope, flow.velocity(), sample.aspect);
        let total_friction = (traverse + slope).clamp(-0.9, 0.95);
        let velocity_factor = (1.0 - total_friction).max(0.05);
        let current_velocity = flow.velocity();
        flow.set_velocity(current_velocity * velocity_factor, None);
    }
}

/// Conductividad térmica efectiva (Capa 4 + overlay térmico Capa 10).
fn effective_thermal_conductivity(
    row: Option<(&MatterCoherence, Option<&ResonanceThermalOverlay>)>,
) -> f32 {
    row.map(|(c, overlay_opt)| {
        let mult = overlay_opt
            .map(|overlay| overlay.conductivity_multiplier)
            .unwrap_or(1.0)
            .max(0.0);
        c.thermal_conductivity() * mult
    })
    .unwrap_or(THERMAL_CONDUCTIVITY_FALLBACK)
}

/// Sistema: Detección de colisiones y resolución de interferencia de ondas.
/// Fase: Phase::AtomicLayer
pub fn collision_interference_system(
    fixed: Option<Res<Time<Fixed>>>,
    time: Res<Time>,
    mut energy_ops: EnergyOps,
    interference_ops: InterferenceOps,
    volumes: Query<&SpatialVolume>,
    coherences: Query<(&MatterCoherence, Option<&ResonanceThermalOverlay>)>,
    index: Res<SpatialIndex>,
    mut ev_collision: EventWriter<CollisionEvent>,
    query: Query<Entity, Without<AmbientPressure>>,
) {
    let dt = simulation_delta_secs(fixed, &time);
    let pairs: Vec<(SpatialEntry, SpatialEntry)> = index.overlapping_pairs();

    for (a, b) in pairs {
        if query.get(a.entity).is_err() || query.get(b.entity).is_err() {
            continue;
        }

        let Some(interf) = interference_ops.between(a.entity, b.entity) else {
            continue;
        };

        let qe_a = energy_ops.qe(a.entity).unwrap_or(0.0);
        let qe_b = energy_ops.qe(b.entity).unwrap_or(0.0);
        if qe_a <= 0.0 && qe_b <= 0.0 {
            continue;
        }

        let density_a = volumes
            .get(a.entity)
            .ok()
            .map(|v| v.density(qe_a))
            .unwrap_or(0.0);
        let density_b = volumes
            .get(b.entity)
            .ok()
            .map(|v| v.density(qe_b))
            .unwrap_or(0.0);

        let conductivity_a = effective_thermal_conductivity(coherences.get(a.entity).ok());
        let conductivity_b = effective_thermal_conductivity(coherences.get(b.entity).ok());
        let conductivity = (conductivity_a + conductivity_b) * COLLISION_CONDUCTIVITY_BLEND;

        let transfer = equations::collision_transfer(qe_a, qe_b, interf, conductivity, dt);
        if transfer <= 0.0 {
            continue;
        }

        if interf > 0.0 {
            if density_a > density_b {
                let drained = energy_ops.drain(a.entity, transfer, DeathCause::Destruction);
                if drained > 0.0 {
                    energy_ops.inject(b.entity, drained);
                }
            } else {
                let drained = energy_ops.drain(b.entity, transfer, DeathCause::Destruction);
                if drained > 0.0 {
                    energy_ops.inject(a.entity, drained);
                }
            }
        } else {
            energy_ops.drain(a.entity, transfer, DeathCause::Destruction);
            energy_ops.drain(b.entity, transfer, DeathCause::Destruction);
        }

        ev_collision.send(CollisionEvent {
            entity_a: a.entity,
            entity_b: b.entity,
            interference: interf,
            transferred_qe: transfer,
        });
    }
}

/// Registra la cadena `Phase::AtomicLayer` (orden fijo; sprint Q5 + SM-8D split).
pub fn register_physics_phase_systems<S: ScheduleLabel + Clone>(app: &mut App, schedule: S) {
    app.add_systems(
        schedule,
        (
            dissipation_system,
            will_to_velocity_system,
            velocity_cap_system,
            terrain_effects_system,
            super::locomotion::locomotion_energy_drain_system,
            super::locomotion::locomotion_exhaustion_system,
            movement_integrate_transform_system,
            player_demo_walk_constraint_system,
            update_spatial_index_after_move_system,
            structural_runtime::tension_field_system,
            collision_interference_system,
        )
            .chain()
            .in_set(Phase::AtomicLayer),
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::blueprint::equations::terrain_blocks_vision;
    use crate::layers::{FlowVector, MatterCoherence, MatterState};
    use crate::worldgen::EnergyFieldGrid;
    use bevy::math::Vec2;

    #[test]
    fn traverse_cost_acceptance_cases() {
        assert_eq!(
            traverse_cost_modifier(TerrainType::Peak, MatterState::Solid),
            0.5
        );
        assert_eq!(
            traverse_cost_modifier(TerrainType::Valley, MatterState::Solid),
            -0.1
        );
        assert_eq!(
            traverse_cost_modifier(TerrainType::Peak, MatterState::Gas),
            0.0
        );
        assert_eq!(
            traverse_cost_modifier(TerrainType::Riverbed, MatterState::Liquid),
            -0.3
        );
    }

    #[test]
    fn terrain_blocks_vision_clear_line_same_height() {
        let mut terrain = TerrainField::new(3, 1, 1.0, Vec2::ZERO, 0);
        terrain.altitude = vec![0.0, 0.0, 0.0];
        let from = terrain.cell_to_world(0, 0);
        let to = terrain.cell_to_world(2, 0);
        assert!(!terrain_blocks_vision(from, to, &terrain));
    }

    #[test]
    fn terrain_blocks_vision_when_middle_cell_is_higher() {
        let mut terrain = TerrainField::new(3, 1, 1.0, Vec2::ZERO, 0);
        terrain.altitude = vec![0.0, 2.0, 0.0];
        let from = terrain.cell_to_world(0, 0);
        let to = terrain.cell_to_world(2, 0);
        assert!(terrain_blocks_vision(from, to, &terrain));
    }

    #[test]
    fn terrain_blocks_vision_adjacent_cells_do_not_block() {
        let mut terrain = TerrainField::new(2, 1, 1.0, Vec2::ZERO, 0);
        terrain.altitude = vec![0.0, 100.0];
        let from = terrain.cell_to_world(0, 0);
        let to = terrain.cell_to_world(1, 0);
        assert!(!terrain_blocks_vision(from, to, &terrain));
    }

    #[test]
    fn terrain_blocks_vision_when_endpoint_is_outside_grid() {
        let terrain = TerrainField::new(2, 2, 1.0, Vec2::ZERO, 0);
        let from = Vec2::new(-1.0, 0.0);
        let to = terrain.cell_to_world(1, 1);
        assert!(terrain_blocks_vision(from, to, &terrain));
    }

    #[test]
    fn slope_friction_is_positive_when_climbing() {
        let f = slope_friction(45.0, Vec2::new(-1.0, 0.0), 0.0);
        assert!(f > 0.0);
    }

    #[test]
    fn slope_friction_is_negative_when_descending() {
        let f = slope_friction(45.0, Vec2::new(1.0, 0.0), 0.0);
        assert!(f < 0.0);
    }

    #[test]
    fn clamp_sim_pos_to_grid_bounds_pulls_back_from_outside() {
        let grid = EnergyFieldGrid::new(10, 10, 2.0, Vec2::new(-10.0, -10.0));
        let c = super::clamp_sim_pos_to_grid_bounds(Vec2::new(50.0, -50.0), &grid, 0.0, 0);
        assert!((c.x - 10.0).abs() < 1e-3);
        assert!((c.y - (-10.0)).abs() < 1e-3);
    }

    #[test]
    fn clamp_sim_pos_respeta_playfield_margin() {
        let grid = EnergyFieldGrid::new(10, 10, 2.0, Vec2::new(-10.0, -10.0));
        let c = super::clamp_sim_pos_to_grid_bounds(Vec2::new(10.0, 0.0), &grid, 0.0, 1);
        assert!((c.x - 8.0).abs() < 1e-3);
    }

    #[test]
    fn terrain_effects_without_terrain_resource_keeps_velocity() {
        let mut app = App::new();
        app.insert_resource(SimWorldTransformParams::default());
        let e = app
            .world_mut()
            .spawn((
                FlowVector::new(Vec2::new(2.0, 0.0), 0.01),
                Transform::default(),
                MatterCoherence::new(MatterState::Solid, 1.0, 0.5),
            ))
            .id();

        app.add_systems(Update, terrain_effects_system);
        app.update();

        let flow = app.world().get::<FlowVector>(e).expect("flow exists");
        assert_eq!(flow.velocity(), Vec2::new(2.0, 0.0));
    }

    #[test]
    fn terrain_effects_keeps_gas_velocity_unchanged() {
        let mut app = App::new();
        app.insert_resource(SimWorldTransformParams::default());
        let mut terrain = TerrainField::new(2, 1, 1.0, Vec2::ZERO, 7);
        terrain.slope[0] = 60.0;
        terrain.aspect[0] = 180.0;
        terrain.terrain_type[0] = TerrainType::Cliff;
        app.insert_resource(terrain);

        let e = app
            .world_mut()
            .spawn((
                FlowVector::new(Vec2::new(2.0, 0.0), 0.01),
                Transform::from_xyz(0.25, 0.25, 0.0),
                MatterCoherence::new(MatterState::Gas, 1.0, 0.5),
            ))
            .id();

        app.add_systems(Update, terrain_effects_system);
        app.update();

        let flow = app.world().get::<FlowVector>(e).expect("flow exists");
        assert_eq!(flow.velocity(), Vec2::new(2.0, 0.0));
    }

    #[test]
    fn terrain_effects_keeps_plasma_velocity_unchanged() {
        let mut app = App::new();
        app.insert_resource(SimWorldTransformParams::default());
        let mut terrain = TerrainField::new(2, 1, 1.0, Vec2::ZERO, 7);
        terrain.slope[0] = 60.0;
        terrain.aspect[0] = 180.0;
        terrain.terrain_type[0] = TerrainType::Cliff;
        app.insert_resource(terrain);

        let e = app
            .world_mut()
            .spawn((
                FlowVector::new(Vec2::new(2.0, 0.0), 0.01),
                Transform::from_xyz(0.25, 0.25, 0.0),
                MatterCoherence::new(MatterState::Plasma, 1.0, 0.5),
            ))
            .id();

        app.add_systems(Update, terrain_effects_system);
        app.update();

        let flow = app.world().get::<FlowVector>(e).expect("flow exists");
        assert_eq!(flow.velocity(), Vec2::new(2.0, 0.0));
    }
}
