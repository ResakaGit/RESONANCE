//! Encola `Grimoire*Pending` desde datos puros (sin drenar buffer; resolución en PrePhysics).

use bevy::prelude::*;

use crate::blueprint::AlchemicalAlmanac;
use crate::blueprint::constants;
use crate::blueprint::equations;
use crate::entities::{InjectorConfig, PhysicsConfig};
use crate::events::{AbilityCastEvent, GrimoireProjectileCastPending, GrimoireSelfBuffCastPending};
use crate::layers::{
    AbilityOutput, AbilityTarget, AlchemicalEngine, Grimoire, SpatialVolume, WillActuator,
};
use crate::runtime_platform::compat_2d3d::SimWorldTransformParams;
use crate::runtime_platform::core_math_agnostic::sim_plane_pos;

/// Dirección en el plano sim para proyectil / vector de lanzamiento.
pub fn resolve_cast_planar_direction(
    caster_tf: &Transform,
    layout: &SimWorldTransformParams,
    actuator: &WillActuator,
    target: &AbilityTarget,
) -> Vec2 {
    let caster_plane = sim_plane_pos(caster_tf.translation, layout.use_xz_ground);
    match target {
        AbilityTarget::Point(p) => {
            let tp = sim_plane_pos(*p, layout.use_xz_ground);
            let d = tp - caster_plane;
            if d.length_squared() < constants::DIRECTION_NORMALIZE_EPS {
                fallback_dir(actuator)
            } else {
                d.normalize()
            }
        }
        AbilityTarget::Direction(d3) => {
            let d = Vec2::new(d3.x, d3.z);
            if d.length_squared() < constants::DIRECTION_NORMALIZE_EPS {
                fallback_dir(actuator)
            } else {
                d.normalize()
            }
        }
        AbilityTarget::Entity(_) | AbilityTarget::None => fallback_dir(actuator),
    }
}

fn fallback_dir(actuator: &WillActuator) -> Vec2 {
    let d = actuator.movement_intent();
    if d.length_squared() < 1e-6 {
        Vec2::X
    } else {
        d.normalize()
    }
}

/// Encola cast si hay buffer suficiente (no muta motor). Retorna `true` si envió pending.
#[allow(clippy::too_many_arguments)]
pub fn enqueue_grimoire_cast_intent(
    caster: Entity,
    slot_index: usize,
    grimoire: &Grimoire,
    engine: &AlchemicalEngine,
    transform: &Transform,
    caster_vol: &SpatialVolume,
    actuator: &WillActuator,
    target: AbilityTarget,
    layout: &SimWorldTransformParams,
    almanac: &AlchemicalAlmanac,
    cast_ev: &mut EventWriter<AbilityCastEvent>,
    pending_proj: &mut EventWriter<GrimoireProjectileCastPending>,
    pending_self: &mut EventWriter<GrimoireSelfBuffCastPending>,
) -> bool {
    let ability_count = grimoire.abilities().len();
    if slot_index >= ability_count {
        return false;
    }
    let slot = &grimoire.abilities()[slot_index];
    let cost = slot.cost_qe().max(0.0);
    if cost <= 0.0 {
        return false;
    }
    if !equations::can_cast(engine, slot) {
        return false;
    }

    let dir = resolve_cast_planar_direction(transform, layout, actuator, &target);
    let caster_plane = sim_plane_pos(transform.translation, layout.use_xz_ground);
    let base_pos = caster_plane + dir * (caster_vol.radius + constants::PROJECTILE_SPAWN_OFFSET);

    match &slot.output {
        AbilityOutput::SelfBuff { effect } => {
            pending_self.send(GrimoireSelfBuffCastPending {
                caster,
                cost_qe: cost,
                recipe: effect.clone(),
            });
            cast_ev.send(AbilityCastEvent {
                caster,
                slot_index,
                target,
            });
            true
        }
        AbilityOutput::Projectile {
            element_id,
            radius,
            speed,
            effect,
        } => {
            let Some(element_def) = almanac.get(*element_id) else {
                return false;
            };
            let spawn_radius = radius.max(0.01);
            let projectile_pos = base_pos + dir * spawn_radius;
            let velocity = dir * speed.max(0.0);
            let physics = PhysicsConfig {
                pos: projectile_pos,
                qe: cost,
                radius: spawn_radius,
                element_id: *element_id,
                velocity,
                dissipation: equations::projectile_dissipation(spawn_radius),
            };
            let injector = InjectorConfig {
                projected_qe: cost,
                forced_frequency: element_def.frequency_hz,
                influence_radius: spawn_radius,
            };
            pending_proj.send(GrimoireProjectileCastPending {
                caster,
                cost_qe: cost,
                physics,
                injector,
                effect: effect.clone(),
                despawn_on_contact: true,
            });
            cast_ev.send(AbilityCastEvent {
                caster,
                slot_index,
                target,
            });
            true
        }
        _ => false,
    }
}
