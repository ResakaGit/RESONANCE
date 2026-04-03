//! Override efectivo para captura MOBA (evita importar `Camera3dEnabled` aquí → ciclo con cámara).

use bevy::prelude::*;

/// Valores que `capture_input_system` aplica tras sincronizar con `Camera3dEnabled`.
#[derive(Resource, Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct MobaIntentCaptureOverride {
    pub suppress_wasd_in_movement_intent: bool,
    pub primary_action_uses_left_shift: bool,
}
