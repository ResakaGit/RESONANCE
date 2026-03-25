use std::collections::{HashMap, HashSet};

use bevy::math::primitives::Sphere;
use bevy::pbr::StandardMaterial;
use bevy::prelude::*;

use crate::blueprint::AlchemicalAlmanac;
use crate::blueprint::ElementId;
use crate::blueprint::constants::VISUAL_QE_REFERENCE;
use crate::layers::{
    AlchemicalInjector, BaseEnergy, FogHiddenMask, MatterCoherence, MatterState, SpatialVolume,
};
use crate::runtime_platform::compat_2d3d::SimWorldTransformParams;
use crate::runtime_platform::contracts::{SimStateSnapshot, VisualEntityPod};
use crate::runtime_platform::core_math_agnostic::{clamp_unit, sim_plane_pos, vec2_to_xz};
use crate::runtime_platform::fog_overlay::{
    FogRenderObserver, fog_overlay_texture_sync_system, sync_local_fog_observer_from_player_system,
};
use crate::runtime_platform::kinematics_3d_adapter::V6RuntimeEntity;
use crate::runtime_platform::simulation_tick::SimulationClock;
use crate::simulation::post::faction_identity_system;
use crate::simulation::states::{GameState, PlayState};
use crate::worldgen::constants::REFERENCE_DENSITY;

/// Vincula una entidad de simulación V6 con su nodo visual 3D.
#[derive(Component, Debug, Clone, Copy)]
pub struct V6VisualRoot {
    pub visual_entity: Entity,
}

/// Snapshot de entrada para el bridge de render (post-simulación).
#[derive(Resource, Debug, Clone, Default)]
pub struct V6RenderSnapshot(pub SimStateSnapshot);

/// Coalescer opcional de escrituras a `StandardMaterial` (por defecto desactivado).
/// Proving Grounds inserta `proving_grounds()` para ahorrar trabajo GPU.
#[derive(Resource, Clone, Copy, Debug)]
pub struct V6DemoMaterialSyncPolicy {
    pub coalesce_redundant_material_writes: bool,
    pub color_diff_eps: f32,
    pub emissive_diff_eps: f32,
    pub scalar_eps: f32,
}

impl Default for V6DemoMaterialSyncPolicy {
    fn default() -> Self {
        Self {
            coalesce_redundant_material_writes: false,
            color_diff_eps: 0.028,
            emissive_diff_eps: 0.022,
            scalar_eps: 0.038,
        }
    }
}

impl V6DemoMaterialSyncPolicy {
    pub fn proving_grounds() -> Self {
        Self {
            coalesce_redundant_material_writes: true,
            ..Default::default()
        }
    }
}

#[derive(Clone, Copy)]
pub(crate) struct CachedMaterialFinger {
    br: f32,
    bg: f32,
    bb: f32,
    er: f32,
    eg: f32,
    eb: f32,
    rough: f32,
    metal: f32,
}

impl CachedMaterialFinger {
    fn from_pod(pod: &VisualEntityPod) -> Self {
        let b = pod.base_color.to_linear();
        let e = pod.emissive.to_linear();
        Self {
            br: b.red,
            bg: b.green,
            bb: b.blue,
            er: e.red,
            eg: e.green,
            eb: e.blue,
            rough: pod.perceptual_roughness,
            metal: pod.metallic,
        }
    }

    fn matches_pod(&self, pod: &VisualEntityPod, p: &V6DemoMaterialSyncPolicy) -> bool {
        let b = pod.base_color.to_linear();
        let e = pod.emissive.to_linear();
        (self.br - b.red).abs() < p.color_diff_eps
            && (self.bg - b.green).abs() < p.color_diff_eps
            && (self.bb - b.blue).abs() < p.color_diff_eps
            && (self.er - e.red).abs() < p.emissive_diff_eps
            && (self.eg - e.green).abs() < p.emissive_diff_eps
            && (self.eb - e.blue).abs() < p.emissive_diff_eps
            && (self.rough - pod.perceptual_roughness).abs() < p.scalar_eps
            && (self.metal - pod.metallic).abs() < p.scalar_eps
    }
}

/// Plugin para puente visual 3D desacoplado de simulación (solo lectura vía snapshot).
pub struct RenderBridge3dPlugin;

impl Plugin for RenderBridge3dPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<V6RenderSnapshot>()
            .init_resource::<V6DemoMaterialSyncPolicy>()
            .init_resource::<FogRenderObserver>()
            .add_systems(
                FixedUpdate,
                capture_v6_visual_snapshot_system.after(faction_identity_system),
            )
            .add_systems(
                Update,
                (
                    sync_visual_from_sim_system,
                    sync_local_fog_observer_from_player_system,
                    sync_fog_hidden_to_visual_roots_system,
                    fog_overlay_texture_sync_system,
                )
                    .chain()
                    .run_if(in_state(GameState::Playing).and(in_state(PlayState::Active))),
            );
    }
}

/// Niebla: oculta el mesh visual si el observador local no tiene línea de “celda visible”.
fn sync_fog_hidden_to_visual_roots_system(
    observer: Res<FogRenderObserver>,
    sims: Query<(Option<&FogHiddenMask>, &V6VisualRoot), With<V6RuntimeEntity>>,
    mut vis: Query<&mut Visibility, Without<V6RuntimeEntity>>,
) {
    for (mask, root) in &sims {
        let hidden = mask
            .map(|m| m.hidden_from_team(observer.team))
            .unwrap_or(false);
        let Ok(mut v) = vis.get_mut(root.visual_entity) else {
            continue;
        };
        let want = if hidden {
            Visibility::Hidden
        } else {
            Visibility::Inherited
        };
        if *v != want {
            *v = want;
        }
    }
}

// --- Derivación visual desde capas (L0 qe, L1 radio→densidad, L2 elemento, L4 estado, L8 inyector) ---

const VISUAL_EMISSIVE_GAIN: f32 = 0.85;
const VISUAL_DENSITY_ROUGHNESS_RANGE: f32 = 0.45;

/// Copia estado visual mínimo tras PostPhysics (hex boundary: render no toca ECS de sim).
pub fn capture_v6_visual_snapshot_system(
    clock: Res<SimulationClock>,
    layout: Res<SimWorldTransformParams>,
    almanac: Res<AlchemicalAlmanac>,
    mut snap: ResMut<V6RenderSnapshot>,
    q: Query<
        (
            Entity,
            &Transform,
            Option<&BaseEnergy>,
            Option<&SpatialVolume>,
            Option<&ElementId>,
            Option<&MatterCoherence>,
            Option<&AlchemicalInjector>,
        ),
        With<V6RuntimeEntity>,
    >,
) {
    snap.0.tick_id = Some(clock.tick_id);
    snap.0.pods.clear();
    let xz_ground = layout.use_xz_ground;

    for (e, t, energy, volume, element_id, coherence, injector) in &q {
        let plane = sim_plane_pos(t.translation, xz_ground);
        let translation = if xz_ground {
            Vec3::new(plane.x, layout.standing_y, plane.y)
        } else {
            vec2_to_xz(plane)
        };

        let (scale, base_color, emissive, perceptual_roughness, metallic) =
            visual_from_layers(energy, volume, element_id, coherence, injector, &almanac);

        snap.0.pods.push(VisualEntityPod {
            sim_entity: e,
            translation,
            scale,
            base_color,
            emissive,
            perceptual_roughness,
            metallic,
        });
    }
    snap.0.pods.sort_by_key(|p| p.sim_entity.to_bits());
}

/// Deriva color PBR + escala esférica desde capas ortogonales (SSOT densidad: `SpatialVolume::density`).
fn visual_from_layers(
    energy: Option<&BaseEnergy>,
    volume: Option<&SpatialVolume>,
    element_id: Option<&ElementId>,
    coherence: Option<&MatterCoherence>,
    injector: Option<&AlchemicalInjector>,
    almanac: &AlchemicalAlmanac,
) -> (Vec3, Color, Color, f32, f32) {
    let qe = energy.map(|b| b.qe()).unwrap_or(0.0).max(0.0);
    let qe_norm = clamp_unit(qe / VISUAL_QE_REFERENCE);

    let radius = volume.map(|v| v.radius).unwrap_or(0.5).max(1e-4);
    let density = volume.map(|v| v.density(qe)).unwrap_or(0.0);
    let density_norm = clamp_unit(density / REFERENCE_DENSITY.max(1e-4));

    // Escala: esfera unidad (r=1 en mesh) × radio de colisión L1.
    let scale = Vec3::splat(radius);

    // Matiz identidad: L2 vía almanaque; fallback L4 / L8.
    let base_tint = if let Some(eid) = element_id {
        almanac
            .get(*eid)
            .map(|def| Color::srgb(def.color.0, def.color.1, def.color.2))
    } else {
        None
    };

    let (mut base_color, rough_base, metal_base, plasma_boost) = if injector.is_some() {
        (Color::srgb(0.92, 0.38, 0.95), 0.38, 0.12, 0.35_f32)
    } else {
        match coherence.map(|c| c.state()) {
            Some(MatterState::Solid) => (Color::srgb(0.58, 0.62, 0.72), 0.82, 0.04, 0.0),
            Some(MatterState::Liquid) => (Color::srgb(0.18, 0.54, 0.84), 0.45, 0.08, 0.05),
            Some(MatterState::Gas) => (Color::srgb(0.62, 0.78, 0.88), 0.62, 0.02, 0.08),
            Some(MatterState::Plasma) => (Color::srgb(0.96, 0.44, 0.19), 0.28, 0.22, 0.55),
            None => (Color::srgb(0.65, 0.65, 0.65), 0.72, 0.06, 0.0),
        }
    };

    if let Some(tint) = base_tint {
        base_color = tint;
    }

    // Oscurece ligeramente baja energía; plasma/inyector ya brillan por emissive.
    let drain = 0.15 * (1.0 - qe_norm);
    let base_linear = base_color.to_linear();
    let base_color = Color::LinearRgba(LinearRgba {
        red: (base_linear.red * (1.0 - drain)).clamp(0.0, 1.0),
        green: (base_linear.green * (1.0 - drain)).clamp(0.0, 1.0),
        blue: (base_linear.blue * (1.0 - drain)).clamp(0.0, 1.0),
        alpha: 1.0,
    });

    // Emisión ∝ qe (L0) y refuerzo por estado “caliente”.
    let emissive_strength =
        qe_norm * VISUAL_EMISSIVE_GAIN * (1.0 + plasma_boost + density_norm * 0.35);
    let e_lin = base_color.to_linear();
    let emissive = Color::LinearRgba(LinearRgba {
        red: (e_lin.red * emissive_strength).min(3.0),
        green: (e_lin.green * emissive_strength).min(3.0),
        blue: (e_lin.blue * emissive_strength).min(3.0),
        alpha: 1.0,
    });

    // “Textura” inferida sin atlas: rugosidad/metal desde densidad (L0+L1) + estado L4.
    let roughness =
        (rough_base + (1.0 - density_norm) * VISUAL_DENSITY_ROUGHNESS_RANGE).clamp(0.12, 1.0);
    let metallic = (metal_base + density_norm * 0.18).clamp(0.0, 1.0);

    (scale, base_color, emissive, roughness, metallic)
}

/// Sincroniza nodos visuales solo desde `V6RenderSnapshot` (sin queries a capas de sim).
pub(crate) fn sync_visual_from_sim_system(
    mut commands: Commands,
    snapshot: Res<V6RenderSnapshot>,
    policy: Res<V6DemoMaterialSyncPolicy>,
    mut mat_finger: Local<HashMap<Entity, CachedMaterialFinger>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    visual_roots: Query<Option<&V6VisualRoot>, With<V6RuntimeEntity>>,
    mut visual_transform_query: Query<&mut Transform, Without<V6RuntimeEntity>>,
    visual_material_query: Query<&MeshMaterial3d<StandardMaterial>, Without<V6RuntimeEntity>>,
) {
    if snapshot.0.tick_id.is_none() {
        return;
    }

    let active: HashSet<Entity> = snapshot.0.pods.iter().map(|p| p.sim_entity).collect();
    mat_finger.retain(|e, _| active.contains(e));

    for pod in &snapshot.0.pods {
        let Ok(visual_root) = visual_roots.get(pod.sim_entity) else {
            continue;
        };

        if let Some(root) = visual_root {
            if let Ok(mut tf) = visual_transform_query.get_mut(root.visual_entity) {
                tf.translation = pod.translation;
                tf.scale = pod.scale;
            }
            let push_mat = if policy.coalesce_redundant_material_writes {
                match mat_finger.get(&pod.sim_entity) {
                    Some(prev) if prev.matches_pod(pod, &policy) => false,
                    _ => true,
                }
            } else {
                true
            };
            if push_mat {
                if let Ok(mesh_mat) = visual_material_query.get(root.visual_entity) {
                    if let Some(m) = materials.get_mut(&mesh_mat.0) {
                        m.base_color = pod.base_color;
                        m.emissive = pod.emissive.into();
                        m.perceptual_roughness = pod.perceptual_roughness;
                        m.metallic = pod.metallic;
                    }
                }
                mat_finger.insert(pod.sim_entity, CachedMaterialFinger::from_pod(pod));
            }
            continue;
        }

        let material = materials.add(StandardMaterial {
            base_color: pod.base_color,
            emissive: pod.emissive.into(),
            perceptual_roughness: pod.perceptual_roughness,
            metallic: pod.metallic,
            ..default()
        });

        let mesh = meshes.add(Mesh::from(Sphere::new(1.0)));
        let visual_entity = commands
            .spawn((
                Mesh3d(mesh),
                MeshMaterial3d(material.clone()),
                Transform {
                    translation: pod.translation,
                    scale: pod.scale,
                    ..default()
                },
            ))
            .id();

        commands
            .entity(pod.sim_entity)
            .insert(V6VisualRoot { visual_entity });
        mat_finger.insert(pod.sim_entity, CachedMaterialFinger::from_pod(pod));
    }
}

// @hex_boundary:sync_visual_end (límite para verify_wave_gate)

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn visual_from_layers_zero_qe_dim_emissive() {
        let almanac = AlchemicalAlmanac::default();
        let v = SpatialVolume::new(0.5);
        let e = BaseEnergy::new(0.0);
        let (_s, _bc, em, _r, _m) =
            visual_from_layers(Some(&e), Some(&v), None, None, None, &almanac);
        let lin = em.to_linear();
        assert!(lin.red + lin.green + lin.blue < 0.2);
    }

    #[test]
    fn visual_from_layers_higher_density_lowers_roughness_vs_low_density() {
        let almanac = AlchemicalAlmanac::default();
        let v_small = SpatialVolume::new(0.4);
        let v_big = SpatialVolume::new(2.0);
        let e = BaseEnergy::new(400.0);
        let (_, _, _, r_tight, _) = visual_from_layers(
            Some(&e),
            Some(&v_small),
            None,
            Some(&MatterCoherence::new(MatterState::Solid, 5000.0, 0.2)),
            None,
            &almanac,
        );
        let (_, _, _, r_loose, _) = visual_from_layers(
            Some(&e),
            Some(&v_big),
            None,
            Some(&MatterCoherence::new(MatterState::Solid, 5000.0, 0.2)),
            None,
            &almanac,
        );
        assert!(r_tight < r_loose, "mayor densidad → menos mate");
    }
}
