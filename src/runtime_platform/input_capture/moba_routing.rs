//! Enrutamiento de teclado cuando la cámara MOBA 3D consume WASD / Space.

use bevy::prelude::*;

/// Ajustes de captura para no mezclar pan de cámara con `IntentSnapshot` de locomoción.
#[derive(Resource, Debug, Clone, Copy, PartialEq, Eq)]
pub struct MobaKeyboardRouting3d {
    /// WASD y flechas no entran al buffer de movimiento (el pan lee teclas directo).
    pub suppress_wasd_in_movement_intent: bool,
    /// `BUTTON_PRIMARY_ACTION` sale de Shift izquierdo en lugar de Space (Space = toggle lock cámara).
    pub primary_action_uses_left_shift: bool,
}

impl Default for MobaKeyboardRouting3d {
    fn default() -> Self {
        Self {
            suppress_wasd_in_movement_intent: false,
            primary_action_uses_left_shift: false,
        }
    }
}
