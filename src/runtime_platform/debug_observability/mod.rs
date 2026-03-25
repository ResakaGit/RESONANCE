use bevy::prelude::*;
use bevy::text::{TextColor, TextFont};
use bevy::ui::{BackgroundColor, Node, PositionType, Val};

use crate::layers::BaseEnergy;
use crate::runtime_platform::camera_controller_3d::Camera3dEnabled;
use crate::runtime_platform::input_capture::IntentBuffer;
use crate::runtime_platform::kinematics_3d_adapter::V6RuntimeEntity;
use crate::runtime_platform::render_bridge_3d::V6RenderSnapshot;
use crate::runtime_platform::simulation_tick::{SimulationClock, V6RuntimeConfig};

/// Flag único para activar/desactivar observabilidad de Sprint 10.
#[derive(Resource, Debug, Clone, Copy, PartialEq, Eq)]
pub struct ObservabilityEnabled(pub bool);

impl Default for ObservabilityEnabled {
    fn default() -> Self {
        Self(true)
    }
}

/// Métricas por frame para HUD/logs sin tocar gameplay.
#[derive(Resource, Debug, Clone, Copy, Default)]
pub struct FrameMetrics {
    pub tick_id: u64,
    pub fixed_dt_secs: f32,
    pub total_entities: u32,
    pub runtime_entities: u32,
    pub negative_qe_entities: u32,
    pub wrote_input_this_frame: bool,
    pub v6_fixed_tick_enabled: bool,
    pub v6_camera_enabled: bool,
    pub v6_render_snapshot_ready: bool,
    pub v5_future_hit_rate: Option<f32>,
}

#[derive(Component, Debug, Clone, Copy, Default)]
struct ObservabilityHud;

/// Plugin de observabilidad/debug (Sprint 10).
pub struct ObservabilityPlugin;

impl Plugin for ObservabilityPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ObservabilityEnabled>()
            .init_resource::<FrameMetrics>()
            .add_systems(
                Update,
                (
                    ensure_v6_observability_hud_system,
                    update_v6_frame_metrics_system,
                    sync_v6_observability_hud_text_system,
                    emit_runtime_metrics_log_system,
                )
                    .chain()
                    .run_if(v6_observability_enabled),
            );
    }
}

fn v6_observability_enabled(enabled: Res<ObservabilityEnabled>) -> bool {
    enabled.0
}

/// Crea un overlay mínimo para visualizar métricas de runtime.
fn ensure_v6_observability_hud_system(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    hud_query: Query<Entity, With<ObservabilityHud>>,
) {
    if hud_query.iter().next().is_some() {
        return;
    }

    commands.spawn((
        ObservabilityHud,
        Node {
            position_type: PositionType::Absolute,
            left: Val::Px(10.0),
            top: Val::Px(10.0),
            padding: UiRect::all(Val::Px(8.0)),
            ..default()
        },
        BackgroundColor(Color::srgba(0.04, 0.04, 0.04, 0.72)),
        Text::new("V6 Observability"),
        TextFont {
            font: asset_server.load("fonts/FiraSans-Bold.ttf"),
            font_size: 13.0,
            ..default()
        },
        TextColor(Color::srgba(0.92, 0.95, 1.0, 1.0)),
    ));
}

/// Actualiza métricas por frame desde lecturas ECS (sin writes de simulación).
pub fn update_v6_frame_metrics_system(
    time: Res<Time>,
    simulation_clock: Option<Res<SimulationClock>>,
    runtime_config: Option<Res<V6RuntimeConfig>>,
    camera_enabled: Option<Res<Camera3dEnabled>>,
    intent_buffer: Option<Res<IntentBuffer>>,
    render_snapshot: Option<Res<V6RenderSnapshot>>,
    mut metrics: ResMut<FrameMetrics>,
    all_entities: Query<Entity>,
    runtime_entities: Query<Entity, With<V6RuntimeEntity>>,
    energy_query: Query<&BaseEnergy>,
) {
    let negative_qe_count = energy_query.iter().filter(|layer| layer.qe() < 0.0).count();
    let fixed_dt_secs = runtime_config
        .as_ref()
        .map(|cfg| {
            if cfg.use_fixed_tick {
                if cfg.fixed_hz > 0.0 {
                    (1.0 / cfg.fixed_hz) as f32
                } else {
                    0.0
                }
            } else {
                time.delta_secs()
            }
        })
        .unwrap_or_else(|| time.delta_secs());

    let wrote_input_this_frame = intent_buffer
        .as_ref()
        .map(|buffer| buffer.wrote_this_frame)
        .unwrap_or(false);
    metrics.tick_id = simulation_clock
        .as_ref()
        .map(|clock| clock.tick_id)
        .unwrap_or(0);
    metrics.fixed_dt_secs = fixed_dt_secs;
    metrics.total_entities = all_entities.iter().count() as u32;
    metrics.runtime_entities = runtime_entities.iter().count() as u32;
    metrics.negative_qe_entities = negative_qe_count as u32;
    metrics.wrote_input_this_frame = wrote_input_this_frame;
    metrics.v6_fixed_tick_enabled = runtime_config
        .as_ref()
        .map(|cfg| cfg.use_fixed_tick)
        .unwrap_or(false);
    metrics.v6_camera_enabled = camera_enabled
        .as_ref()
        .map(|enabled| enabled.0)
        .unwrap_or(false);
    metrics.v6_render_snapshot_ready = render_snapshot
        .as_ref()
        .map(|snapshot| snapshot.0.tick_id.is_some())
        .unwrap_or(false);
}

/// Sincroniza texto del HUD con el recurso de métricas.
fn sync_v6_observability_hud_text_system(
    metrics: Res<FrameMetrics>,
    mut hud_query: Query<&mut Text, With<ObservabilityHud>>,
) {
    if !metrics.is_changed() {
        return;
    }

    let mut text_value = String::from("V6 OBS\n");
    text_value.push_str(&format!(
        "tick={} dt={:.4}\n",
        metrics.tick_id, metrics.fixed_dt_secs
    ));
    text_value.push_str(&format!(
        "entities={} runtime={}\n",
        metrics.total_entities, metrics.runtime_entities
    ));
    text_value.push_str(&format!("inv_qe_neg={}\n", metrics.negative_qe_entities));
    text_value.push_str(&format!(
        "flags fixed={} camera={} render={}",
        metrics.v6_fixed_tick_enabled, metrics.v6_camera_enabled, metrics.v6_render_snapshot_ready
    ));
    text_value.push_str(&format!("\ninput_wrote={}", metrics.wrote_input_this_frame));

    if let Some(hit_rate) = metrics.v5_future_hit_rate {
        text_value.push_str(&format!("\nv5_hit_rate={hit_rate:.3}"));
    } else {
        text_value.push_str("\nv5_hit_rate=n/a");
    }

    for mut text in &mut hud_query {
        if text.0 != text_value {
            text.0 = text_value.clone();
        }
    }
}

/// Emite logs acotados (1Hz) para tracing target `runtime_platform::debug_observability`.
fn emit_runtime_metrics_log_system(
    time: Res<Time>,
    metrics: Res<FrameMetrics>,
    mut last_log_secs: Local<f32>,
) {
    let now = time.elapsed_secs();
    if now - *last_log_secs < 1.0 {
        return;
    }
    *last_log_secs = now;

    info!(
        target: "runtime_platform::debug_observability",
        "tick={} dt={:.4} entities={} runtime={} inv_qe_neg={} flags(fixed={},camera={},render={})",
        metrics.tick_id,
        metrics.fixed_dt_secs,
        metrics.total_entities,
        metrics.runtime_entities,
        metrics.negative_qe_entities,
        metrics.v6_fixed_tick_enabled,
        metrics.v6_camera_enabled,
        metrics.v6_render_snapshot_ready
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn disabled_flag_can_turn_off_observability() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .add_plugins(ObservabilityPlugin)
            .insert_resource(ObservabilityEnabled(false));

        app.update();
        let world = app.world_mut();
        let mut hud_query = world.query::<&ObservabilityHud>();
        let hud_count = hud_query.iter(world).count();
        assert_eq!(hud_count, 0);
    }
}
