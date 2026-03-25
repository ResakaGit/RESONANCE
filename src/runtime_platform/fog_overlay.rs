//! Overlay de niebla 3D: textura alineada al grid, alpha por celda (no visto / explorado / visible).
//! Solo se spawnea con perfil `Full3dVisual`; tests librería sin ventana omiten este paso.
//!
//! La textura solo se rellena cuando cambia [`crate::world::FogOfWarGrid::fog_stamp_generation`] o el
//! equipo observador (menos ancho de banda GPU que un upload cada frame).

use bevy::pbr::{NotShadowCaster, NotShadowReceiver, StandardMaterial};
use bevy::prelude::*;
use bevy::render::render_asset::RenderAssetUsages;
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};

use crate::layers::MobaIdentity;
use crate::runtime_platform::compat_2d3d::{RenderCompatProfile, SimWorldTransformParams};
use crate::runtime_platform::core_math_agnostic::DEFAULT_SIM_STANDING_Y;
use crate::simulation::PlayerControlled;
use crate::world::fog_of_war::{FogOfWarGrid, NUM_FOG_TEAMS, fog_team_index};

/// Por encima de meshes de unidad en `standing_y` (radio típico &lt; 1); reduce cruce con esferas.
const FOG_OVERLAY_Y_ABOVE_STANDING: f32 = 2.8;

/// Equipo cuya perspectiva de niebla usa el cliente (overlay + visibilidad de meshes).
#[derive(Resource, Debug, Clone, Copy)]
pub struct FogRenderObserver {
    pub team: u8,
}

impl Default for FogRenderObserver {
    fn default() -> Self {
        Self { team: 0 }
    }
}

/// Handles del overlay world-space (opcional si no hay perfil 3D).
#[derive(Resource, Debug)]
pub struct FogOverlayHandles {
    pub image: Handle<Image>,
    pub _material: Handle<StandardMaterial>,
    pub _mesh: Handle<Mesh>,
}

/// Marca entidad overlay para depuración / orden de render futuro.
#[derive(Component)]
pub struct FogWorldOverlay;

/// Spawnea plano texturizado sobre el mapa; requiere [`FogOfWarGrid`] ya insertado.
pub fn spawn_fog_world_overlay_startup_system(
    profile: Option<Res<RenderCompatProfile>>,
    mut commands: Commands,
    fog: Option<Res<FogOfWarGrid>>,
    layout: Option<Res<SimWorldTransformParams>>,
    mut images: ResMut<Assets<Image>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let Some(profile) = profile else {
        return;
    };
    if !profile.enables_visual_3d() {
        return;
    }
    let Some(fog) = fog else {
        return;
    };
    let layout = layout.map(|r| *r).unwrap_or(SimWorldTransformParams {
        use_xz_ground: true,
        standing_y: DEFAULT_SIM_STANDING_Y,
    });

    let w = fog.width.max(1);
    let h = fog.height.max(1);
    let size = Extent3d {
        width: w,
        height: h,
        depth_or_array_layers: 1,
    };
    let image = Image::new_fill(
        size,
        TextureDimension::D2,
        &[0u8, 0u8, 0u8, 255u8],
        TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::default(),
    );
    let image_handle = images.add(image);

    let world_w = fog.width as f32 * fog.cell_size;
    let world_h = fog.height as f32 * fog.cell_size;
    let cx = fog.origin.x + world_w * 0.5;
    let cz = fog.origin.y + world_h * 0.5;
    let y = layout.standing_y + FOG_OVERLAY_Y_ABOVE_STANDING;

    let mesh_handle = meshes.add(Plane3d::default().mesh().size(world_w, world_h));
    let material_handle = materials.add(StandardMaterial {
        base_color_texture: Some(image_handle.clone()),
        base_color: Color::WHITE,
        alpha_mode: AlphaMode::Blend,
        unlit: true,
        perceptual_roughness: 1.0,
        metallic: 0.0,
        ..default()
    });

    commands.spawn((
        FogWorldOverlay,
        Mesh3d(mesh_handle.clone()),
        MeshMaterial3d(material_handle.clone()),
        Transform::from_xyz(cx, y, cz),
        GlobalTransform::default(),
        Visibility::Visible,
        NotShadowCaster,
        NotShadowReceiver,
    ));

    commands.insert_resource(FogOverlayHandles {
        image: image_handle,
        _material: material_handle,
        _mesh: mesh_handle,
    });
}

/// Sigue al héroe controlado para elegir equipo de niebla (Red/Blue).
pub fn sync_local_fog_observer_from_player_system(
    q: Query<&MobaIdentity, With<PlayerControlled>>,
    mut observer: ResMut<FogRenderObserver>,
) {
    use std::sync::atomic::{AtomicBool, Ordering};
    static NEUTRAL_PLAYER_WARNED: AtomicBool = AtomicBool::new(false);

    for id in &q {
        if let Some(t) = fog_team_index(id.faction()) {
            if observer.team != t {
                observer.team = t;
            }
            return;
        }
        if !NEUTRAL_PLAYER_WARNED.swap(true, Ordering::Relaxed) {
            bevy::log::warn!(
                "FogRenderObserver: héroe PlayerControlled sin facción Red/Blue; se mantiene team={} para overlay",
                observer.team
            );
        }
        return;
    }
}

/// Copia el estado del grid al RGBA de la textura para el equipo observador.
pub fn fog_overlay_texture_sync_system(
    fog: Option<Res<FogOfWarGrid>>,
    handles: Option<Res<FogOverlayHandles>>,
    observer: Res<FogRenderObserver>,
    mut images: ResMut<Assets<Image>>,
    mut last_key: Local<Option<(u32, u8)>>,
) {
    let Some(fog) = fog else {
        return;
    };
    let Some(handles) = handles else {
        return;
    };
    let Some(img) = images.get_mut(&handles.image) else {
        return;
    };

    let key = (fog.fog_stamp_generation, observer.team);
    if observer.is_changed() {
        *last_key = None;
    }
    if *last_key == Some(key) {
        return;
    }
    *last_key = Some(key);

    let team = observer.team as usize;
    if team >= NUM_FOG_TEAMS {
        return;
    }
    let Some(row) = fog.team_cells_row_major(team) else {
        return;
    };

    let w = fog.width as usize;
    let h = fog.height as usize;
    let expected = w * h * 4;
    if img.data.len() != expected {
        return;
    }

    for cy in 0..h {
        for cx in 0..w {
            let idx = cy * w + cx;
            let v = row.get(idx).copied().unwrap_or(-1);
            let (r, g, b, a) = if v > 0 {
                (0u8, 0u8, 0u8, 0u8)
            } else if v == 0 {
                (55u8, 52u8, 62u8, 78u8)
            } else {
                (8u8, 8u8, 14u8, 120u8)
            };
            let o = ((h - 1 - cy) * w + cx) * 4;
            if o + 3 < img.data.len() {
                img.data[o] = r;
                img.data[o + 1] = g;
                img.data[o + 2] = b;
                img.data[o + 3] = a;
            }
        }
    }
}
