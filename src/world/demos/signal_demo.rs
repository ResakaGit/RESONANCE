//! SF-7E/F — Signal latency demo.
//! Un solo núcleo en el centro; PropagationMode::WaveFront activo.
//! `RESONANCE_MAP=signal_latency_demo cargo run` muestra la onda expandiéndose.

use bevy::prelude::*;

use crate::worldgen::PropagationMode;

/// Slug de mapa para dispatch condicional en plugins.
pub const SIGNAL_DEMO_SLUG: &str = "signal_latency_demo";

/// Habilita WaveFront y registra el inicio del demo.
/// Sin fauna — observación pura de la onda de campo.
pub fn spawn_signal_demo_startup_system(mut commands: Commands) {
    commands.insert_resource(PropagationMode::WaveFront);
    info!("SF-7 signal demo: single nucleus, WaveFront propagation active — watching the wave");
}
