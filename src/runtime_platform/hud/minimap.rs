//! Minimap estilo MOBA: panel UI, iconos ligados a entidades sim, click → teletransporte de foco.
//!
//! - Mapeo world XZ ↔ UV: [`crate::blueprint::equations`].
//! - Extensión world: [`crate::runtime_platform::camera_controller_3d::MobaCameraBounds`] (sin duplicar bounds).
//! - Alpha por frecuencia L2: [`crate::blueprint::equations::minimap_perception_alpha`].

use bevy::prelude::*;
use bevy::ui::{
    AlignItems, BackgroundColor, BorderRadius, Display, GlobalZIndex, JustifyContent, Node,
    Overflow, PositionType, Val,
};

use crate::blueprint::ElementId;
use crate::blueprint::equations;
use crate::layers::{Faction, OscillatorySignature};
use crate::runtime_platform::camera_controller_3d::{
    CameraMode, MobaCameraBounds, MobaCameraConfig, MobaCameraState,
};
use crate::simulation::states::{GameState, PlayState};

use super::minimap_constants::{
    MINIMAP_INNER_INSET_PX, MINIMAP_MARGIN_PX, MINIMAP_SIZE_PX, MINIMAP_UPDATE_EVERY_FRAMES,
    MINIMAP_VIEWPORT_FILL_ALPHA,
};

/// Marca entidades que deben mostrarse en el minimap (presentación; no es capa L0–L13).
#[derive(Component, Debug, Clone, Copy)]
pub struct MinimapIcon {
    /// Diámetro del marcador en píxeles del panel.
    pub diameter_px: f32,
    /// Color base (RGB); alpha viene de percepción por frecuencia.
    pub base_color: Color,
}

impl MinimapIcon {
    pub fn new(diameter_px: f32, base_color: Color) -> Self {
        Self {
            diameter_px,
            base_color,
        }
    }

    pub fn hero_faction(faction: Faction) -> Self {
        Self {
            diameter_px: 9.0,
            base_color: minimap_faction_color(faction),
        }
    }

    pub fn crystal(element: ElementId) -> Self {
        Self {
            diameter_px: 6.0,
            base_color: minimap_crystal_color(element),
        }
    }
}

#[inline]
fn minimap_faction_color(faction: Faction) -> Color {
    match faction {
        Faction::Red => Color::srgb(0.92, 0.22, 0.18),
        Faction::Blue => Color::srgb(0.22, 0.48, 0.95),
        Faction::Wild => Color::srgb(0.78, 0.55, 0.18),
        Faction::Neutral => Color::srgb(0.72, 0.74, 0.78),
    }
}

#[inline]
fn minimap_crystal_color(element: ElementId) -> Color {
    if element == ElementId::from_name("Ignis") {
        Color::srgb(0.98, 0.45, 0.12)
    } else if element == ElementId::from_name("Aqua") {
        Color::srgb(0.2, 0.55, 0.95)
    } else if element == ElementId::from_name("Lux") {
        Color::srgb(0.95, 0.92, 0.55)
    } else if element == ElementId::from_name("Ventus") {
        Color::srgb(0.55, 0.9, 0.75)
    } else if element == ElementId::from_name("Terra") {
        Color::srgb(0.55, 0.42, 0.28)
    } else {
        Color::srgb(0.65, 0.65, 0.7)
    }
}

/// Entidad mundo rastreada por un nodo UI de icono.
#[derive(Component, Debug, Clone, Copy)]
pub struct MinimapTracksWorld(pub Entity);

#[derive(Component)]
struct MinimapRoot;

#[derive(Component)]
struct MinimapPanel;

#[derive(Component)]
struct MinimapIconsLayer;

#[derive(Component)]
struct MinimapViewportTag;

/// Rectángulo del panel en coordenadas de cursor de ventana (origen arriba-izquierda).
#[derive(Resource, Debug, Clone, Copy, Default)]
pub struct MinimapScreenRect {
    pub min: Vec2,
    pub max: Vec2,
    pub valid: bool,
}

/// Contador de frames HUD (throttle iconos); avanza cada frame con el bloque del minimap.
#[derive(Resource, Default, Debug, Clone, Copy)]
struct MinimapHudFrame(pub u32);

pub struct MinimapPlugin;

impl Plugin for MinimapPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<MinimapScreenRect>()
            .init_resource::<MinimapHudFrame>()
            .add_systems(
                Update,
                (
                    minimap_refresh_screen_rect_system,
                    minimap_ensure_ui_system,
                    minimap_registry_system,
                    minimap_throttled_icon_positions_system,
                    minimap_throttled_icon_alphas_system,
                    minimap_viewport_system,
                    minimap_click_system,
                    minimap_hud_frame_tick,
                )
                    .chain()
                    .run_if(in_state(GameState::Playing).and(in_state(PlayState::Active))),
            );
    }
}

/// True si el cursor está sobre el rect del minimap (para no disparar raycast mundo / targeting).
#[inline]
pub fn minimap_cursor_blocks_primary_pick(
    cursor: Option<Vec2>,
    screen: &MinimapScreenRect,
) -> bool {
    let Some(c) = cursor else {
        return false;
    };
    screen.valid
        && c.x >= screen.min.x
        && c.x <= screen.max.x
        && c.y >= screen.min.y
        && c.y <= screen.max.y
}

fn minimap_hud_frame_tick(mut f: ResMut<MinimapHudFrame>) {
    f.0 = f.0.wrapping_add(1);
}

/// Área útil del panel (ancho × alto) alineada al rect en pantalla (ventanas chicas / clip).
fn minimap_inner_dims_from_screen(screen: &MinimapScreenRect) -> (f32, f32) {
    let fallback = (MINIMAP_SIZE_PX - 2.0 * MINIMAP_INNER_INSET_PX).max(1.0);
    if !screen.valid {
        return (fallback, fallback);
    }
    let w = screen.max.x - screen.min.x - 2.0 * MINIMAP_INNER_INSET_PX;
    let h = screen.max.y - screen.min.y - 2.0 * MINIMAP_INNER_INSET_PX;
    (w.max(1.0), h.max(1.0))
}

fn minimap_refresh_screen_rect_system(
    windows: Query<&Window, With<bevy::window::PrimaryWindow>>,
    mut rect: ResMut<MinimapScreenRect>,
) {
    let Ok(w) = windows.get_single() else {
        rect.valid = false;
        return;
    };
    let ww = w.width();
    let wh = w.height();
    if ww < 1.0 || wh < 1.0 {
        rect.valid = false;
        return;
    }
    let left = ww - MINIMAP_MARGIN_PX - MINIMAP_SIZE_PX;
    let top = wh - MINIMAP_MARGIN_PX - MINIMAP_SIZE_PX;
    rect.min = Vec2::new(left.max(0.0), top.max(0.0));
    rect.max = Vec2::new(
        (left + MINIMAP_SIZE_PX).min(ww),
        (top + MINIMAP_SIZE_PX).min(wh),
    );
    rect.valid = true;
}

fn minimap_ensure_ui_system(mut commands: Commands, q_root: Query<Entity, With<MinimapRoot>>) {
    if q_root.iter().next().is_some() {
        return;
    }

    commands
        .spawn((
            MinimapRoot,
            Node {
                position_type: PositionType::Absolute,
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                ..default()
            },
            GlobalZIndex(4096),
        ))
        .with_children(|root| {
            root.spawn((
                MinimapPanel,
                Node {
                    position_type: PositionType::Absolute,
                    right: Val::Px(MINIMAP_MARGIN_PX),
                    bottom: Val::Px(MINIMAP_MARGIN_PX),
                    width: Val::Px(MINIMAP_SIZE_PX),
                    height: Val::Px(MINIMAP_SIZE_PX),
                    overflow: Overflow::clip(),
                    display: Display::Flex,
                    align_items: AlignItems::FlexStart,
                    justify_content: JustifyContent::FlexStart,
                    ..default()
                },
                BackgroundColor(Color::srgba(0.04, 0.06, 0.08, 0.82)),
                BorderRadius::all(Val::Px(6.0)),
            ))
            .with_children(|panel| {
                panel.spawn((
                    MinimapViewportTag,
                    Node {
                        position_type: PositionType::Absolute,
                        left: Val::Px(0.0),
                        top: Val::Px(0.0),
                        width: Val::Px(8.0),
                        height: Val::Px(8.0),
                        ..default()
                    },
                    BackgroundColor(Color::srgba(1.0, 1.0, 1.0, MINIMAP_VIEWPORT_FILL_ALPHA)),
                ));
                panel.spawn((
                    MinimapIconsLayer,
                    Node {
                        position_type: PositionType::Absolute,
                        left: Val::Px(0.0),
                        top: Val::Px(0.0),
                        width: Val::Percent(100.0),
                        height: Val::Percent(100.0),
                        ..default()
                    },
                ));
            });
        });
}

fn minimap_registry_system(
    mut commands: Commands,
    layer: Query<Entity, With<MinimapIconsLayer>>,
    q_tracks: Query<(Entity, &MinimapTracksWorld)>,
    q_world: Query<(Entity, &MinimapIcon), With<GlobalTransform>>,
) {
    let Ok(layer_e) = layer.get_single() else {
        return;
    };

    let mut tracked_world = bevy::utils::HashSet::<Entity>::default();
    for (ui_e, track) in &q_tracks {
        let world_alive = q_world.contains(track.0);
        if !world_alive {
            commands.entity(ui_e).despawn();
        } else {
            tracked_world.insert(track.0);
        }
    }

    for (e, icon) in &q_world {
        if tracked_world.contains(&e) {
            continue;
        }
        let d = icon.diameter_px.max(2.0);
        let icon_ui = commands
            .spawn((
                MinimapTracksWorld(e),
                Node {
                    position_type: PositionType::Absolute,
                    left: Val::Px(0.0),
                    top: Val::Px(0.0),
                    width: Val::Px(d),
                    height: Val::Px(d),
                    ..default()
                },
                BackgroundColor(icon.base_color),
                BorderRadius::all(Val::Px(d * 0.5)),
            ))
            .id();
        commands.entity(icon_ui).set_parent(layer_e);
    }
}

fn minimap_throttled_icon_positions_system(
    bounds: Res<MobaCameraBounds>,
    screen: Res<MinimapScreenRect>,
    frame: Res<MinimapHudFrame>,
    world_q: Query<(&GlobalTransform, &MinimapIcon), With<MinimapIcon>>,
    mut ui_q: Query<(&MinimapTracksWorld, &mut Node)>,
) {
    if frame.0 % MINIMAP_UPDATE_EVERY_FRAMES != 0 {
        return;
    }
    let (inner_w, inner_h) = minimap_inner_dims_from_screen(&screen);

    for (track, mut node) in &mut ui_q {
        let Ok((gt, icon)) = world_q.get(track.0) else {
            continue;
        };
        let t = gt.translation();
        let xz = Vec2::new(t.x, t.z);
        let uv = equations::minimap_world_xz_to_uv(xz, bounds.min_xz, bounds.max_xz);
        let px = MINIMAP_INNER_INSET_PX + uv.x * inner_w - icon.diameter_px * 0.5;
        let py = MINIMAP_INNER_INSET_PX + uv.y * inner_h - icon.diameter_px * 0.5;
        let left = Val::Px(px);
        let top = Val::Px(py);
        if node.left != left {
            node.left = left;
        }
        if node.top != top {
            node.top = top;
        }
    }
}

fn minimap_throttled_icon_alphas_system(
    frame: Res<MinimapHudFrame>,
    world_q: Query<(&MinimapIcon, Option<&OscillatorySignature>), With<MinimapIcon>>,
    mut ui_q: Query<(&MinimapTracksWorld, &mut BackgroundColor)>,
) {
    if frame.0 % MINIMAP_UPDATE_EVERY_FRAMES != 0 {
        return;
    }

    for (track, mut bg) in &mut ui_q {
        let Ok((icon, osc_opt)) = world_q.get(track.0) else {
            continue;
        };
        let hz = osc_opt.map(|o| o.frequency_hz()).unwrap_or(400.0);
        let a = equations::minimap_perception_alpha(hz);
        let next_color = icon.base_color.with_alpha(a);
        if bg.0 != next_color {
            bg.0 = next_color;
        }
    }
}

fn minimap_viewport_system(
    bounds: Res<MobaCameraBounds>,
    state: Res<MobaCameraState>,
    screen: Res<MinimapScreenRect>,
    windows: Query<&Window, With<bevy::window::PrimaryWindow>>,
    mut q_vp: Query<&mut Node, With<MinimapViewportTag>>,
) {
    let Ok(w) = windows.get_single() else {
        return;
    };
    let aspect = (w.width() / w.height().max(1.0)).max(0.2);
    let half_xz = equations::moba_minimap_viewport_half_extents_xz(state.zoom_horizontal, aspect);

    let span_x = (bounds.max_xz.x - bounds.min_xz.x).max(1.0);
    let span_z = (bounds.max_xz.y - bounds.min_xz.y).max(1.0);
    let (inner_w, inner_h) = minimap_inner_dims_from_screen(&screen);

    let uv_c = equations::minimap_world_xz_to_uv(state.focus_xz, bounds.min_xz, bounds.max_xz);
    let w_uv = (2.0 * half_xz.x / span_x).clamp(0.04, 0.98);
    let h_uv = (2.0 * half_xz.y / span_z).clamp(0.04, 0.98);
    let w_px = (w_uv * inner_w).max(6.0);
    let h_px = (h_uv * inner_h).max(6.0);
    let cx = MINIMAP_INNER_INSET_PX + uv_c.x * inner_w;
    let cy = MINIMAP_INNER_INSET_PX + uv_c.y * inner_h;
    let left = cx - w_px * 0.5;
    let top = cy - h_px * 0.5;

    for mut node in &mut q_vp {
        let l = Val::Px(left.max(0.0));
        let t = Val::Px(top.max(0.0));
        let ww = Val::Px(w_px);
        let hh = Val::Px(h_px);
        if node.left != l {
            node.left = l;
        }
        if node.top != t {
            node.top = t;
        }
        if node.width != ww {
            node.width = ww;
        }
        if node.height != hh {
            node.height = hh;
        }
    }
}

fn minimap_click_system(
    mouse: Res<ButtonInput<MouseButton>>,
    screen: Res<MinimapScreenRect>,
    bounds: Res<MobaCameraBounds>,
    mut cam_cfg: ResMut<MobaCameraConfig>,
    mut cam_state: ResMut<MobaCameraState>,
    windows: Query<&Window, With<bevy::window::PrimaryWindow>>,
) {
    if !mouse.just_pressed(MouseButton::Left) || !screen.valid {
        return;
    }
    let Ok(w) = windows.get_single() else {
        return;
    };
    let Some(cursor) = w.cursor_position() else {
        return;
    };
    if cursor.x < screen.min.x
        || cursor.x > screen.max.x
        || cursor.y < screen.min.y
        || cursor.y > screen.max.y
    {
        return;
    }

    let (inner_w, inner_h) = minimap_inner_dims_from_screen(&screen);
    let u = ((cursor.x - screen.min.x - MINIMAP_INNER_INSET_PX) / inner_w).clamp(0.0, 1.0);
    let v = ((cursor.y - screen.min.y - MINIMAP_INNER_INSET_PX) / inner_h).clamp(0.0, 1.0);
    let world_xz = equations::minimap_uv_to_world_xz(Vec2::new(u, v), bounds.min_xz, bounds.max_xz);

    cam_cfg.mode = CameraMode::Free;
    if cam_state.focus_xz != world_xz {
        cam_state.focus_xz = world_xz;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cursor_block_matches_screen_rect() {
        let s = MinimapScreenRect {
            valid: true,
            min: Vec2::new(100.0, 50.0),
            max: Vec2::new(200.0, 150.0),
        };
        assert!(minimap_cursor_blocks_primary_pick(
            Some(Vec2::new(150.0, 100.0)),
            &s
        ));
        assert!(!minimap_cursor_blocks_primary_pick(
            Some(Vec2::new(99.0, 100.0)),
            &s
        ));
        assert!(!minimap_cursor_blocks_primary_pick(None, &s));
    }
}
