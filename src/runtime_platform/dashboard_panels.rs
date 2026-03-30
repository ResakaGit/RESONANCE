//! Paneles de dashboard científico usando egui. Stateless: leen Resources, renderizan.
//! Scientific dashboard panels using egui. Stateless: read Resources, render.
//!
//! Cada función de panel es pura sobre `&egui::Context` + `Res<T>`.
//! Zero mutación de la simulación. Zero queries ECS.
//! El plugin se añade SOLO en binarios que tienen ventana gráfica.

use bevy::prelude::*;
use bevy_egui::{EguiContexts, EguiPlugin, egui};
use egui_plot::{Line, Plot, PlotPoints};

use crate::runtime_platform::dashboard_bridge::{
    CameraMode, ColorMode, RingBuffer, SimSpeedConfig, SimTickSummary, SimTimeSeries, ViewConfig,
};

// ─── Layout constants (visual calibration, no physics) ──────────────────────

const SPEED_MIN: f32           = 0.1;
const SPEED_MAX: f32           = 10.0;
const CONTROL_PANEL_WIDTH: f32 = 200.0;
const CHART_HEIGHT_RATIO: f32  = 0.45;
const CHART_MIN_HEIGHT: f32    = 100.0;
const CHART_SPACING: f32       = 8.0;

// ─── Chart colors (visual identity, no physics) ─────────────────────────────

const COLOR_POPULATION: egui::Color32 = egui::Color32::GREEN;
const COLOR_ENERGY: egui::Color32     = egui::Color32::from_rgb(100, 150, 255);
const COLOR_SPECIES: egui::Color32    = egui::Color32::from_rgb(255, 180, 50);
const COLOR_CORRELATION: egui::Color32 = egui::Color32::from_rgb(200, 100, 200);

// ─── Tab state ──────────────────────────────────────────────────────────────

/// Pestaña activa del dashboard.
/// Active dashboard tab.
#[derive(Resource, Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum DashboardTab {
    #[default]
    Simulation,
    Parameters,
    Analysis,
}

// ─── Panel systems (Update schedule, stateless) ─────────────────────────────

/// Top bar: tabs + status. Lightweight — solo lee summary para el status label.
/// Top bar: tabs + status. Lightweight — only reads summary for status label.
pub fn dashboard_top_bar_system(
    mut contexts: EguiContexts,
    mut tab:      ResMut<DashboardTab>,
    summary:      Res<SimTickSummary>,
) {
    let Some(ctx) = contexts.try_ctx_mut() else { return };
    egui::TopBottomPanel::top("dashboard_tabs").show(ctx, |ui| {
        ui.horizontal(|ui| {
            ui.selectable_value(&mut *tab, DashboardTab::Simulation, "Simulation");
            ui.selectable_value(&mut *tab, DashboardTab::Parameters, "Parameters");
            ui.selectable_value(&mut *tab, DashboardTab::Analysis, "Analysis");
            ui.separator();
            ui.label(format!(
                "tick {} | pop {} | qe {:.0}",
                summary.tick, summary.alive_count, summary.total_qe,
            ));
        });
    });
}

/// Left panel: controles según tab activo.
/// Left panel: controls based on active tab.
pub fn dashboard_controls_system(
    mut contexts: EguiContexts,
    tab:          Res<DashboardTab>,
    summary:      Res<SimTickSummary>,
    mut speed:    ResMut<SimSpeedConfig>,
    mut view:     ResMut<ViewConfig>,
) {
    let Some(ctx) = contexts.try_ctx_mut() else { return };
    egui::SidePanel::left("controls").default_width(CONTROL_PANEL_WIDTH).show(ctx, |ui| {
        match *tab {
            DashboardTab::Simulation => render_simulation_controls(ui, &mut speed),
            DashboardTab::Parameters => render_parameter_controls(ui, &mut view),
            DashboardTab::Analysis   => render_analysis_controls(ui, &summary),
        }
    });
}

/// Central panel: charts/views según tab activo.
/// Central panel: charts/views based on active tab.
pub fn dashboard_charts_system(
    mut contexts: EguiContexts,
    tab:          Res<DashboardTab>,
    summary:      Res<SimTickSummary>,
    series:       Res<SimTimeSeries>,
    view:         Res<ViewConfig>,
) {
    let Some(ctx) = contexts.try_ctx_mut() else { return };
    egui::CentralPanel::default().show(ctx, |ui| {
        match *tab {
            DashboardTab::Simulation => render_simulation_charts(ui, &series),
            DashboardTab::Parameters => render_parameter_view(ui, &view),
            DashboardTab::Analysis   => render_analysis_charts(ui, &series),
        }
    });
}

// ─── Simulation tab ─────────────────────────────────────────────────────────

/// Controles de simulación: velocidad, pausa.
fn render_simulation_controls(ui: &mut egui::Ui, speed: &mut SimSpeedConfig) {
    ui.heading("Simulation");
    ui.separator();

    ui.label("Speed");
    ui.add(egui::Slider::new(&mut speed.time_scale, SPEED_MIN..=SPEED_MAX).text("×"));

    ui.checkbox(&mut speed.paused, "Paused");

    ui.separator();
    ui.label(if speed.paused { "PAUSED" } else { "RUNNING" });
}

/// Gráficos de simulación: population + energy time series.
fn render_simulation_charts(ui: &mut egui::Ui, series: &SimTimeSeries) {
    let available = ui.available_size();
    let chart_height = (available.y * CHART_HEIGHT_RATIO).max(CHART_MIN_HEIGHT);

    // ── Population chart ──
    ui.label("Population");
    let pop_data = ring_to_plot_points(&series.pop_history);
    Plot::new("pop_chart")
        .height(chart_height)
        .allow_drag(false)
        .show(ui, |plot_ui| {
            plot_ui.line(Line::new(pop_data).name("alive").color(COLOR_POPULATION));
        });

    ui.add_space(CHART_SPACING);

    // ── Energy chart ──
    ui.label("Total Energy (qe)");
    let qe_data = ring_to_plot_points(&series.qe_history);
    Plot::new("qe_chart")
        .height(chart_height)
        .allow_drag(false)
        .show(ui, |plot_ui| {
            plot_ui.line(Line::new(qe_data).name("total qe").color(COLOR_ENERGY));
        });
}

// ─── Parameters tab ─────────────────────────────────────────────────────────

/// Controles de visualización: color mode, camera, grid.
fn render_parameter_controls(ui: &mut egui::Ui, view: &mut ViewConfig) {
    ui.heading("View");
    ui.separator();

    ui.checkbox(&mut view.show_grid, "Show grid");
    ui.checkbox(&mut view.show_trajectories, "Show trajectories");

    ui.separator();
    ui.label("Color mode");
    egui::ComboBox::from_id_salt("color_mode")
        .selected_text(color_mode_label(view.color_mode))
        .show_ui(ui, |ui| {
            ui.selectable_value(&mut view.color_mode, ColorMode::Frequency, "Frequency");
            ui.selectable_value(&mut view.color_mode, ColorMode::Energy, "Energy");
            ui.selectable_value(&mut view.color_mode, ColorMode::Trophic, "Trophic");
            ui.selectable_value(&mut view.color_mode, ColorMode::Age, "Age");
        });

    ui.separator();
    ui.label("Camera");
    egui::ComboBox::from_id_salt("camera_mode")
        .selected_text(camera_mode_label(view.camera_mode))
        .show_ui(ui, |ui| {
            ui.selectable_value(&mut view.camera_mode, CameraMode::Orbital, "Orbital");
            ui.selectable_value(&mut view.camera_mode, CameraMode::FollowPlayer, "Follow Player");
            ui.selectable_value(&mut view.camera_mode, CameraMode::TopDown, "Top Down");
        });
}

/// Vista de parámetros actuales (read-only summary).
fn render_parameter_view(ui: &mut egui::Ui, view: &ViewConfig) {
    ui.heading("Current View Config");
    ui.separator();
    egui::Grid::new("param_grid").show(ui, |ui| {
        ui.label("Grid:");      ui.label(if view.show_grid { "ON" } else { "OFF" }); ui.end_row();
        ui.label("Trajectories:"); ui.label(if view.show_trajectories { "ON" } else { "OFF" }); ui.end_row();
        ui.label("Color:");     ui.label(color_mode_label(view.color_mode)); ui.end_row();
        ui.label("Camera:");    ui.label(camera_mode_label(view.camera_mode)); ui.end_row();
    });
}

// ─── Analysis tab ───────────────────────────────────────────────────────────

/// Controles de análisis: statistics summary.
fn render_analysis_controls(ui: &mut egui::Ui, summary: &SimTickSummary) {
    ui.heading("Analysis");
    ui.separator();
    egui::Grid::new("analysis_grid").show(ui, |ui| {
        ui.label("Tick:");      ui.label(format!("{}", summary.tick));       ui.end_row();
        ui.label("Population:"); ui.label(format!("{}", summary.alive_count)); ui.end_row();
        ui.label("Total qe:");  ui.label(format!("{:.1}", summary.total_qe)); ui.end_row();
        ui.label("Species:");   ui.label(format!("{}", summary.species_count)); ui.end_row();
    });
}

/// Gráficos de análisis: species time series + population vs energy scatter.
fn render_analysis_charts(ui: &mut egui::Ui, series: &SimTimeSeries) {
    let available = ui.available_size();
    let chart_height = (available.y * CHART_HEIGHT_RATIO).max(CHART_MIN_HEIGHT);

    ui.label("Species Count");
    let species_data = ring_to_plot_points(&series.species_history);
    Plot::new("species_chart")
        .height(chart_height)
        .allow_drag(false)
        .show(ui, |plot_ui| {
            plot_ui.line(Line::new(species_data).name("species").color(COLOR_SPECIES));
        });

    ui.add_space(CHART_SPACING);

    // Population vs Energy correlation (zero-alloc: zip iterators)
    ui.label("Population vs Energy");
    let scatter: PlotPoints = series.pop_history.iter()
        .zip(series.qe_history.iter())
        .map(|(p, q)| [p as f64, q as f64])
        .collect();
    Plot::new("pop_qe_scatter")
        .height(chart_height)
        .allow_drag(false)
        .show(ui, |plot_ui| {
            plot_ui.line(Line::new(scatter).name("pop×qe").color(COLOR_CORRELATION));
        });
}

// ─── Helpers (stateless, pure) ──────────────────────────────────────────────

/// Convierte RingBuffer a PlotPoints para egui_plot.
fn ring_to_plot_points(ring: &RingBuffer) -> PlotPoints {
    ring.iter()
        .enumerate()
        .map(|(i, v)| [i as f64, v as f64])
        .collect()
}

/// Label para ColorMode.
fn color_mode_label(mode: ColorMode) -> &'static str {
    match mode {
        ColorMode::Frequency => "Frequency",
        ColorMode::Energy    => "Energy",
        ColorMode::Trophic   => "Trophic",
        ColorMode::Age       => "Age",
    }
}

/// Label para CameraMode.
fn camera_mode_label(mode: CameraMode) -> &'static str {
    match mode {
        CameraMode::Orbital      => "Orbital",
        CameraMode::FollowPlayer => "Follow Player",
        CameraMode::TopDown      => "Top Down",
    }
}

// ─── Plugin ─────────────────────────────────────────────────────────────────

/// Plugin de dashboard visual. Requiere `DashboardBridgePlugin` + ventana gráfica.
/// Visual dashboard plugin. Requires `DashboardBridgePlugin` + graphical window.
pub struct DashboardPanelsPlugin;

impl Plugin for DashboardPanelsPlugin {
    fn build(&self, app: &mut App) {
        if !app.is_plugin_added::<EguiPlugin>() {
            app.add_plugins(EguiPlugin);
        }
        app.init_resource::<DashboardTab>()
           .add_systems(Update, (
               dashboard_top_bar_system,
               dashboard_controls_system,
               dashboard_charts_system,
           ).chain());
    }
}
