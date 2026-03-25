use bevy::prelude::*;

use crate::{
    blueprint::{AlchemicalAlmanac, IdGenerator, constants},
    entities::EffectConfig,
    entities::archetypes::{spawn_projectile, spawn_resonance_effect},
    events::{
        AbilityCastEvent, AbilitySelectionEvent, GrimoireProjectileCastPending,
        GrimoireSelfBuffCastPending,
    },
    layers::{
        AbilityTarget, AlchemicalEngine, Channeling, Grimoire, ProjectedQeFromEnergy,
        SpatialVolume, TargetingMode, WillActuator,
    },
    runtime_platform::compat_2d3d::SimWorldTransformParams,
    runtime_platform::core_math_agnostic::sim_plane_pos,
    simulation::PlayerControlled,
    simulation::ability_targeting::{ActiveTargeting, TargetingState},
    simulation::grimoire_enqueue::enqueue_grimoire_cast_intent,
};

/// Sistema: Lee input del teclado y lo traduce a intención de movimiento.
/// Fase: Phase::Input
pub fn will_input_system(
    input: Res<ButtonInput<KeyCode>>,
    mut query: Query<&mut WillActuator, With<PlayerControlled>>,
) {
    for mut actuator in &mut query {
        let mut direction = Vec2::ZERO;

        if input.pressed(KeyCode::KeyW) || input.pressed(KeyCode::ArrowUp) {
            direction.y += 1.0;
        }
        if input.pressed(KeyCode::KeyS) || input.pressed(KeyCode::ArrowDown) {
            direction.y -= 1.0;
        }
        if input.pressed(KeyCode::KeyA) || input.pressed(KeyCode::ArrowLeft) {
            direction.x -= 1.0;
        }
        if input.pressed(KeyCode::KeyD) || input.pressed(KeyCode::ArrowRight) {
            direction.x += 1.0;
        }

        let movement = if direction.length_squared() > 0.0 {
            direction.normalize()
        } else {
            Vec2::ZERO
        };
        actuator.set_movement_intent(movement);
    }
}

/// Q / F / E / R + targeting → pending grimorio (sin spawn ni drenaje L5). Resolución: `grimoire_cast_resolve_system`.
/// Fase: Phase::Input
pub fn grimoire_cast_intent_system(
    input: Res<ButtonInput<KeyCode>>,
    mut commands: Commands,
    mut targeting: ResMut<TargetingState>,
    mut select_ev: EventWriter<AbilitySelectionEvent>,
    mut cast_ev: EventWriter<AbilityCastEvent>,
    mut pending_proj: EventWriter<GrimoireProjectileCastPending>,
    mut pending_self: EventWriter<GrimoireSelfBuffCastPending>,
    mut query: Query<
        (
            Entity,
            &mut WillActuator,
            &Transform,
            &SpatialVolume,
            &Grimoire,
            &AlchemicalEngine,
        ),
        With<PlayerControlled>,
    >,
    layout: Res<SimWorldTransformParams>,
    almanac: Res<AlchemicalAlmanac>,
) {
    // Q / F / E / R: en Full3d el WASD no entra al IntentBuffer (pan MOBA); W libre para pan.
    const KEYS: [(KeyCode, usize); 4] = [
        (KeyCode::KeyQ, 0),
        (KeyCode::KeyF, 1),
        (KeyCode::KeyE, 2),
        (KeyCode::KeyR, 3),
    ];

    for (caster, mut will, tf, vol, grim, eng) in &mut query {
        let n = grim.abilities().len();
        if n == 0 {
            continue;
        }

        let mut picked: Option<usize> = None;
        for &(key, idx) in &KEYS {
            if input.just_pressed(key) && idx < n {
                picked = Some(idx);
                break;
            }
        }
        let Some(idx) = picked else {
            continue;
        };

        if will.active_slot() != Some(idx) {
            will.set_active_slot(Some(idx));
        }
        let slot = &grim.abilities()[idx];
        let mode = slot.targeting().clone();

        select_ev.send(AbilitySelectionEvent {
            caster,
            slot_index: idx,
        });

        match mode {
            TargetingMode::NoTarget => {
                if slot.min_channeling_secs() > 0.0 {
                    commands.entity(caster).insert(Channeling {
                        remaining_secs: slot.min_channeling_secs(),
                        slot_index: idx,
                        target: AbilityTarget::None,
                    });
                } else {
                    let _ = enqueue_grimoire_cast_intent(
                        caster,
                        idx,
                        grim,
                        eng,
                        tf,
                        vol,
                        &*will,
                        AbilityTarget::None,
                        &layout,
                        &almanac,
                        &mut cast_ev,
                        &mut pending_proj,
                        &mut pending_self,
                    );
                }
            }
            TargetingMode::PointTarget { .. }
            | TargetingMode::AreaTarget { .. }
            | TargetingMode::UnitTarget { .. } => {
                targeting.active = Some(ActiveTargeting {
                    caster,
                    slot_index: idx,
                    mode,
                });
            }
            TargetingMode::DirectionTarget { .. } => {
                let d = will.movement_intent();
                let dir3 = if layout.use_xz_ground {
                    Vec3::new(d.x, 0.0, d.y)
                } else {
                    Vec3::new(d.x, d.y, 0.0)
                };
                let at = if dir3.length_squared() < constants::DIRECTION_NORMALIZE_EPS {
                    AbilityTarget::None
                } else {
                    AbilityTarget::Direction(dir3.normalize())
                };
                if slot.min_channeling_secs() > 0.0 {
                    commands.entity(caster).insert(Channeling {
                        remaining_secs: slot.min_channeling_secs(),
                        slot_index: idx,
                        target: at,
                    });
                } else {
                    let _ = enqueue_grimoire_cast_intent(
                        caster,
                        idx,
                        grim,
                        eng,
                        tf,
                        vol,
                        &*will,
                        at,
                        &layout,
                        &almanac,
                        &mut cast_ev,
                        &mut pending_proj,
                        &mut pending_self,
                    );
                }
            }
        }
    }
}

/// Materializa proyectiles pendientes y consume buffer del motor (PrePhysics).
pub fn grimoire_cast_resolve_system(
    mut commands: Commands,
    mut id_gen: ResMut<IdGenerator>,
    layout: Res<SimWorldTransformParams>,
    mut pending_proj: EventReader<GrimoireProjectileCastPending>,
    mut pending_self: EventReader<GrimoireSelfBuffCastPending>,
    mut engines: Query<&mut AlchemicalEngine>,
    transforms: Query<&Transform>,
) {
    for ev in pending_proj.read() {
        let Ok(mut engine) = engines.get_mut(ev.caster) else {
            continue;
        };

        if !engine.drain_buffer(ev.cost_qe) {
            continue;
        }

        let entity = spawn_projectile(
            &mut commands,
            &mut id_gen,
            Some(ev.caster),
            ev.physics.clone(),
            ev.injector.clone(),
            ev.effect.clone(),
            ev.despawn_on_contact,
            &layout,
        );
        commands.entity(entity).insert(ProjectedQeFromEnergy);
    }

    for ev in pending_self.read() {
        let Ok(mut engine) = engines.get_mut(ev.caster) else {
            continue;
        };

        if !engine.drain_buffer(ev.cost_qe) {
            continue;
        }

        let Ok(tf) = transforms.get(ev.caster) else {
            continue;
        };
        let at = sim_plane_pos(tf.translation, layout.use_xz_ground);
        let cfg = EffectConfig {
            target: ev.caster,
            modified_field: ev.recipe.field,
            magnitude: ev.recipe.magnitude,
            fuel_qe: ev.recipe.fuel_qe,
            dissipation_rate: ev.recipe.dissipation,
        };
        spawn_resonance_effect(&mut commands, &mut id_gen, &layout, at, cfg);
    }
}
