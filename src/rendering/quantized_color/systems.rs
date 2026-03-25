//! Sistemas ECS: ρ por distancia y sincronización de paletas CPU.

use bevy::math::Vec2;
use bevy::prelude::*;

use crate::blueprint::AlchemicalAlmanac;
use crate::blueprint::equations::precision_rho_from_lod_distance;
use crate::runtime_platform::compat_2d3d::SimWorldTransformParams;
use crate::runtime_platform::core_math_agnostic::sim_plane_pos;
use crate::simulation::sensory::AttentionGrid;
use crate::worldgen::constants::{LOD_MID_MAX, LOD_NEAR_MAX};
use crate::worldgen::contracts::Materialized;

use super::camera_plane::active_camera_sim_plane;
use super::constants::{
    DEFAULT_PALETTE_N_MAX, QUANTIZED_COLOR_RHO_MIN, QUANTIZED_RHO_WRITE_EPS,
};
use super::registry::PaletteRegistry;
use super::QuantizedPrecision;

/// Asegura `QuantizedPrecision` en entidades materializadas (sin tocar materialización).
pub fn quantized_precision_ensure_system(
    mut commands: Commands,
    q: Query<Entity, (With<Materialized>, Without<QuantizedPrecision>)>,
) {
    for e in &q {
        commands.entity(e).insert(QuantizedPrecision(1.0));
    }
}

/// Cámara o Grid de Atención → ρ usando bandas Near/Mid/Far (Modulando Visuals).
pub fn factor_precision_system(
    cameras: Query<(&Camera, &GlobalTransform)>,
    layout: Res<SimWorldTransformParams>,
    attention: Option<Res<AttentionGrid>>,
    mut q: Query<(&Transform, &mut QuantizedPrecision), With<Materialized>>,
) {
    let cam_plane = active_camera_sim_plane(&cameras, layout.use_xz_ground);
    let xz = layout.use_xz_ground;

    for (tf, mut rho_c) in &mut q {
        let ep = sim_plane_pos(tf.translation, xz);

        let rho = if let Some(att) = &attention {
            let a = att.get_attention(ep);
            QUANTIZED_COLOR_RHO_MIN + (1.0 - QUANTIZED_COLOR_RHO_MIN) * a
        } else if let Some(cp) = cam_plane {
            let d = planar_distance_or_far(cp, ep);
            precision_rho_from_lod_distance(d, LOD_NEAR_MAX, LOD_MID_MAX, QUANTIZED_COLOR_RHO_MIN)
        } else {
            1.0
        };

        if (rho_c.0 - rho).abs() > QUANTIZED_RHO_WRITE_EPS {
            rho_c.0 = rho;
        }
    }
}

#[inline]
fn planar_distance_or_far(a: Vec2, b: Vec2) -> f32 {
    if a.is_finite() && b.is_finite() {
        a.distance(b)
    } else {
        f32::MAX
    }
}

/// Reconstruye paletas CPU cuando cambia el almanac.
pub fn palette_registry_cpu_sync_system(
    almanac: Res<AlchemicalAlmanac>,
    mut reg: ResMut<PaletteRegistry>,
) {
    let changed = almanac.is_changed();
    reg.rebuild_if_needed(&almanac, changed, DEFAULT_PALETTE_N_MAX);
}
