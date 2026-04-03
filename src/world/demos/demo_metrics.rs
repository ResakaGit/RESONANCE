//! HUD de métricas para las demos del catálogo.
//! Muestra en pantalla (esquina inferior-izquierda) métricas globales + estado por entidad.
//! Se activa con cualquiera de los 4 slugs: demo_celula / demo_virus / demo_planta / demo_animal.

use bevy::prelude::*;
use bevy::text::{TextColor, TextFont};
use bevy::ui::{BackgroundColor, Node, PositionType, Val};

use crate::layers::{
    BaseEnergy, BehaviorIntent, BehaviorMode, MorphogenesisShapeParams, OscillatorySignature,
    TrophicState,
};
use crate::simulation::observability::{
    SimulationEcologySnapshot, SimulationHealthDashboard, SimulationMetricsSnapshot,
};
use crate::worldgen::ShapeInferred;

const HUD_FONT_SIZE: f32 = 11.0;
const HUD_MAX_ENTITIES: usize = 12;

/// Marca el nodo de texto del HUD de métricas demo.
#[derive(Component, Debug, Clone, Copy)]
pub struct DemoMetricsHud;

/// Crea el overlay de texto (una sola vez).
pub fn ensure_demo_metrics_hud_system(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    hud_query: Query<Entity, With<DemoMetricsHud>>,
) {
    if hud_query.iter().next().is_some() {
        return;
    }
    commands.spawn((
        DemoMetricsHud,
        Node {
            position_type: PositionType::Absolute,
            left: Val::Px(10.0),
            bottom: Val::Px(10.0),
            padding: UiRect::all(Val::Px(8.0)),
            max_width: Val::Px(380.0),
            ..default()
        },
        BackgroundColor(Color::srgba(0.02, 0.04, 0.06, 0.82)),
        Text::new("DEMO METRICS\n..."),
        TextFont {
            font: asset_server.load("fonts/FiraSans-Bold.ttf"),
            font_size: HUD_FONT_SIZE,
            ..default()
        },
        TextColor(Color::srgba(0.85, 0.95, 0.75, 1.0)),
    ));
}

/// Actualiza el HUD de métricas cada frame (guarded by change detection).
pub fn sync_demo_metrics_hud_system(
    snap: Res<SimulationMetricsSnapshot>,
    eco: Res<SimulationEcologySnapshot>,
    health: Res<SimulationHealthDashboard>,
    entity_query: Query<(
        &Name,
        &BaseEnergy,
        &OscillatorySignature,
        Option<&TrophicState>,
        Option<&BehaviorIntent>,
        Option<&MorphogenesisShapeParams>,
        Option<&ShapeInferred>,
    )>,
    mut hud_query: Query<&mut Text, With<DemoMetricsHud>>,
) {
    if !snap.is_changed() && !eco.is_changed() && !health.is_changed() {
        return;
    }

    let conservation_ok = health.conservation_error.abs() < 0.05;
    let cons_sym = if conservation_ok { "✓" } else { "!" };

    let mut text = format!(
        "── DEMO METRICS ──\n\
         tick:{:>7}   qe_total:{:>9.1}\n\
         entities:{:>3}   deaths:{:>3}   growth:{:>+.3}\n\
         field_occ:{:.2}  diversity:{:.2}\n\
         drift:{:.4}  conservation:{cons_sym}\n\
         ── ENTITIES ──\n",
        snap.tick,
        snap.total_qe,
        eco.entity_count,
        eco.deaths_this_tick,
        eco.growth_rate,
        snap.field_occupancy,
        eco.frequency_diversity,
        health.drift_rate,
    );

    let mut count = 0;
    for (name, energy, osc, trophic, behavior, shape_params, shape_inferred) in &entity_query {
        if count >= HUD_MAX_ENTITIES {
            text.push_str("  ...\n");
            break;
        }
        let qe = energy.qe();
        let hz = osc.frequency_hz();
        let sati = trophic
            .map(|t| format!(" sati:{:.2}", t.satiation))
            .unwrap_or_default();
        let mode = behavior
            .map(|b| format!(" {}", format_mode(&b.mode)))
            .unwrap_or_default();
        let shape = shape_params
            .map(|s| {
                let sym = if shape_inferred.is_some() {
                    "✓"
                } else {
                    "…"
                };
                format!(" f:{:.1}{sym}", s.fineness_ratio())
            })
            .unwrap_or_default();
        text.push_str(&format!(
            "  {:<14} qe:{:>7.1} hz:{:>6.1}{sati}{mode}{shape}\n",
            name.as_str(),
            qe,
            hz
        ));
        count += 1;
    }

    for mut t in &mut hud_query {
        if t.0 != text {
            t.0 = text.clone();
        }
    }
}

fn format_mode(mode: &BehaviorMode) -> &'static str {
    match mode {
        BehaviorMode::Idle => "Idle",
        BehaviorMode::Forage { .. } => "Forage",
        BehaviorMode::Hunt { .. } => "Hunt",
        BehaviorMode::Flee { .. } => "Flee",
        BehaviorMode::Reproduce => "Reproduce",
        BehaviorMode::Migrate { .. } => "Migrate",
        BehaviorMode::FocusFire { .. } => "FocusFire",
        BehaviorMode::Regroup { .. } => "Regroup",
    }
}
