use bevy::prelude::*;

use crate::{
    blueprint::{AlchemicalAlmanac, constants},
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

// ─── Internal signal ─────────────────────────────────────────────────────────

/// Player pressed an ability key this tick.
/// Produced by `grimoire_slot_selection_system`, consumed by `grimoire_targeting_system`
/// and `grimoire_channeling_start_system` in the same frame.
#[derive(Event, Debug, Clone, Copy)]
pub struct SlotActivatedEvent {
    pub caster: Entity,
    pub slot_index: usize,
}

// ─── Systems ──────────────────────────────────────────────────────────────────

/// Translates keyboard input to movement intent.
/// Phase: Phase::Input
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
        if actuator.movement_intent() != movement {
            actuator.set_movement_intent(movement);
        }
    }
}

/// Q / F / E / R key press → writes active slot + fires `AbilitySelectionEvent` + `SlotActivatedEvent`.
/// Phase: Phase::Input, InputChannelSet::SimulationRest
pub fn grimoire_slot_selection_system(
    input: Res<ButtonInput<KeyCode>>,
    mut query: Query<(Entity, &Grimoire, &mut WillActuator), With<PlayerControlled>>,
    mut slot_ev: EventWriter<SlotActivatedEvent>,
    mut select_ev: EventWriter<AbilitySelectionEvent>,
) {
    const KEYS: [(KeyCode, usize); 4] = [
        (KeyCode::KeyQ, 0),
        (KeyCode::KeyF, 1),
        (KeyCode::KeyE, 2),
        (KeyCode::KeyR, 3),
    ];

    for (caster, grim, mut will) in &mut query {
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
        let Some(idx) = picked else { continue; };

        if will.active_slot() != Some(idx) {
            will.set_active_slot(Some(idx));
        }
        select_ev.send(AbilitySelectionEvent { caster, slot_index: idx });
        slot_ev.send(SlotActivatedEvent { caster, slot_index: idx });
    }
}

/// Reads `SlotActivatedEvent` → sets `TargetingState.active` for abilities requiring player targeting.
/// Phase: Phase::Input, after `grimoire_slot_selection_system`
pub fn grimoire_targeting_system(
    mut slot_ev: EventReader<SlotActivatedEvent>,
    grimoire_q: Query<&Grimoire>,
    mut targeting: ResMut<TargetingState>,
) {
    for ev in slot_ev.read() {
        let Ok(grim) = grimoire_q.get(ev.caster) else { continue; };
        if ev.slot_index >= grim.abilities().len() {
            continue;
        }
        let mode = grim.abilities()[ev.slot_index].targeting().clone();
        match mode {
            TargetingMode::PointTarget { .. }
            | TargetingMode::AreaTarget { .. }
            | TargetingMode::UnitTarget { .. } => {
                targeting.active = Some(ActiveTargeting {
                    caster: ev.caster,
                    slot_index: ev.slot_index,
                    mode,
                });
            }
            TargetingMode::NoTarget | TargetingMode::DirectionTarget { .. } => {}
        }
    }
}

/// Reads `SlotActivatedEvent` → inserts `Channeling` or dispatches immediate cast
/// for `NoTarget` and `DirectionTarget` abilities.
/// Phase: Phase::Input, after `grimoire_targeting_system`
pub fn grimoire_channeling_start_system(
    mut slot_ev: EventReader<SlotActivatedEvent>,
    mut commands: Commands,
    query: Query<
        (&Transform, &SpatialVolume, &Grimoire, &AlchemicalEngine, &WillActuator),
        With<PlayerControlled>,
    >,
    layout: Res<SimWorldTransformParams>,
    almanac: Res<AlchemicalAlmanac>,
    mut cast_ev: EventWriter<AbilityCastEvent>,
    mut pending_proj: EventWriter<GrimoireProjectileCastPending>,
    mut pending_self: EventWriter<GrimoireSelfBuffCastPending>,
) {
    for ev in slot_ev.read() {
        let Ok((tf, vol, grim, eng, will)) = query.get(ev.caster) else { continue; };
        if ev.slot_index >= grim.abilities().len() {
            continue;
        }
        let slot = &grim.abilities()[ev.slot_index];
        match slot.targeting().clone() {
            TargetingMode::NoTarget => {
                if slot.min_channeling_secs() > 0.0 {
                    commands.entity(ev.caster).insert(Channeling {
                        remaining_secs: slot.min_channeling_secs(),
                        slot_index: ev.slot_index,
                        target: AbilityTarget::None,
                    });
                } else {
                    let _ = enqueue_grimoire_cast_intent(
                        ev.caster,
                        ev.slot_index,
                        grim,
                        eng,
                        tf,
                        vol,
                        will,
                        AbilityTarget::None,
                        &layout,
                        &almanac,
                        &mut cast_ev,
                        &mut pending_proj,
                        &mut pending_self,
                    );
                }
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
                    commands.entity(ev.caster).insert(Channeling {
                        remaining_secs: slot.min_channeling_secs(),
                        slot_index: ev.slot_index,
                        target: at,
                    });
                } else {
                    let _ = enqueue_grimoire_cast_intent(
                        ev.caster,
                        ev.slot_index,
                        grim,
                        eng,
                        tf,
                        vol,
                        will,
                        at,
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
                // Handled by `grimoire_targeting_system`.
            }
        }
    }
}

/// Materializes pending projectiles and drains engine buffer (ThermodynamicLayer).
pub fn grimoire_cast_resolve_system(
    mut commands: Commands,
    mut id_gen: ResMut<crate::blueprint::IdGenerator>,
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
