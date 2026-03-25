use bevy::prelude::*;

use crate::runtime_platform::contracts::{BUTTON_PRIMARY_ACTION, IntentSnapshot};

pub mod moba_intent_override;
pub mod moba_routing;

pub use moba_intent_override::MobaIntentCaptureOverride;
pub use moba_routing::MobaKeyboardRouting3d;

/// Buffer mínimo legacy para consumidores del Sprint 03.
#[derive(Resource, Debug, Clone, Copy, Default)]
pub struct LastIntentSnapshot(pub IntentSnapshot);

/// Buffer extendido por frame para desacople con FixedUpdate.
#[derive(Resource, Debug, Clone, Copy, Default)]
pub struct IntentBuffer {
    pub last_snapshot: IntentSnapshot,
    pub frame_id: u64,
    pub wrote_this_frame: bool,
}

/// Evento opcional para consumidores event-driven.
#[derive(Event, Debug, Clone, Copy)]
pub struct IntentCommitted {
    pub snapshot: IntentSnapshot,
}

/// Set dedicado para ordenar captura antes de otros sistemas.
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct V6InputCaptureSet;

/// Plugin de captura de input.
pub struct InputCapturePlugin;

impl Plugin for InputCapturePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<LastIntentSnapshot>()
            .init_resource::<IntentBuffer>()
            .init_resource::<MobaIntentCaptureOverride>()
            .add_event::<IntentCommitted>()
            .add_systems(
                PreUpdate,
                (clear_intent_buffer_system, capture_input_system)
                    .chain()
                    .in_set(V6InputCaptureSet),
            );
    }
}

/// Limpia metadata por frame sin borrar el último snapshot válido.
pub fn clear_intent_buffer_system(mut buffer: ResMut<IntentBuffer>) {
    buffer.frame_id = buffer.frame_id.saturating_add(1);
    buffer.wrote_this_frame = false;
}

/// Captura WASD/flechas/space/mouse en PreUpdate.
pub fn capture_input_system(
    input: Res<ButtonInput<KeyCode>>,
    moba_override: Res<MobaIntentCaptureOverride>,
    mut last_snapshot: ResMut<LastIntentSnapshot>,
    mut buffer: ResMut<IntentBuffer>,
    mut intent_events: EventWriter<IntentCommitted>,
) {
    let suppress_wasd = moba_override.suppress_wasd_in_movement_intent;
    let primary_from_shift = moba_override.primary_action_uses_left_shift;

    let mut movement = Vec2::ZERO;

    if !suppress_wasd {
        if input.pressed(KeyCode::KeyW) || input.pressed(KeyCode::ArrowUp) {
            movement.y += 1.0;
        }
        if input.pressed(KeyCode::KeyS) || input.pressed(KeyCode::ArrowDown) {
            movement.y -= 1.0;
        }
        if input.pressed(KeyCode::KeyA) || input.pressed(KeyCode::ArrowLeft) {
            movement.x -= 1.0;
        }
        if input.pressed(KeyCode::KeyD) || input.pressed(KeyCode::ArrowRight) {
            movement.x += 1.0;
        }
    }

    let mut button_mask = 0_u16;
    if primary_from_shift {
        if input.pressed(KeyCode::ShiftLeft) {
            button_mask |= BUTTON_PRIMARY_ACTION;
        }
    } else if input.pressed(KeyCode::Space) {
        button_mask |= BUTTON_PRIMARY_ACTION;
    }

    let snapshot = IntentSnapshot::new(movement, button_mask, Some(buffer.frame_id));

    last_snapshot.0 = snapshot;
    buffer.last_snapshot = snapshot;
    buffer.wrote_this_frame = true;
    intent_events.send(IntentCommitted { snapshot });
}

#[cfg(test)]
mod tests {
    use super::*;

    fn new_test_app() -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .init_resource::<ButtonInput<KeyCode>>()
            .add_plugins(InputCapturePlugin);
        app
    }

    #[test]
    fn held_key_produces_consistent_snapshot_across_frames() {
        let mut app = new_test_app();
        {
            let mut input = app.world_mut().resource_mut::<ButtonInput<KeyCode>>();
            input.press(KeyCode::KeyW);
        }

        app.update();
        let first = app.world().resource::<IntentBuffer>().last_snapshot;
        app.update();
        let second = app.world().resource::<IntentBuffer>().last_snapshot;

        assert_eq!(first.movement_xy, Vec2::Y);
        assert_eq!(second.movement_xy, Vec2::Y);
        assert_eq!(first.button_mask, 0);
        assert_eq!(second.button_mask, 0);
    }

    #[test]
    fn snapshot_is_stable_with_no_input() {
        let mut app = new_test_app();
        app.update();
        let first = app.world().resource::<IntentBuffer>().last_snapshot;
        app.update();
        let second = app.world().resource::<IntentBuffer>().last_snapshot;

        assert_eq!(first.movement_xy, Vec2::ZERO);
        assert_eq!(second.movement_xy, Vec2::ZERO);
        assert_eq!(first.button_mask, 0);
        assert_eq!(second.button_mask, 0);
        assert!(first.tick_id.is_some());
        assert!(second.tick_id.is_some());
    }
}
