use bevy::prelude::*;

use crate::blueprint::ElementId;
use crate::eco::context_lookup::ContextLookup;
use crate::entities::builder::EntityBuilder;
use crate::layers::MatterState;
use crate::runtime_platform::compat_2d3d::{RenderCompatProfile, SimWorldTransformParams};
use crate::runtime_platform::core_math_agnostic::sim_plane_pos;
use crate::simulation::time_compat::simulation_delta_secs;

const CLOUD_BASE_ALTITUDE: f32 = 6.0;
const CLOUD_MIN_COUNT: usize = 3;
const CLOUD_MAX_COUNT: usize = 8;

/// Nube de demo: marcador para balanceo contextual y órbita visual.
#[derive(Component, Debug, Clone, Copy, Default)]
#[component(storage = "SparseSet")]
pub struct DemoCloudTag;

/// Ancla primaria para órbita contextual de nubes.
#[derive(Component, Debug, Clone, Copy, Default)]
pub struct DemoCloudAnchor;

/// Órbita local de una nube alrededor de la semilla.
#[derive(Component, Debug, Clone, Copy)]
pub struct DemoCloudOrbit {
    pub angle: f32,
    pub radius: f32,
    pub angular_speed: f32,
    pub altitude: f32,
}

#[derive(Resource, Debug, Default)]
pub struct DemoCloudSpawnerState {
    pub next_index: u32,
}

fn cloud_target_count(pressure: f32, reactivity_mod: f32, temperature_base: f32) -> usize {
    let pressure_term = ((pressure - 1.0) * 2.0).round() as i32;
    let react_term = ((reactivity_mod - 1.0) * 3.0).round() as i32;
    let thermal_term = (temperature_base / 1200.0).round() as i32;
    (4 + pressure_term + react_term + thermal_term)
        .clamp(CLOUD_MIN_COUNT as i32, CLOUD_MAX_COUNT as i32) as usize
}

fn cloud_spawn_transform(
    center: Vec2,
    orbit: DemoCloudOrbit,
    layout: &SimWorldTransformParams,
) -> Transform {
    let x = center.x + orbit.radius * orbit.angle.cos();
    let z = center.y + orbit.radius * orbit.angle.sin();
    if layout.use_xz_ground {
        Transform::from_xyz(x, layout.standing_y + orbit.altitude, z)
    } else {
        Transform::from_xyz(x, z, orbit.altitude)
    }
}

fn demo_seed_center(
    layout: &SimWorldTransformParams,
    seed_query: &Query<(Entity, &Transform), (With<DemoCloudAnchor>, Without<DemoCloudTag>)>,
) -> Vec2 {
    let mut best: Option<(u64, Vec2)> = None;
    for (entity, transform) in seed_query.iter() {
        let pos = sim_plane_pos(transform.translation, layout.use_xz_ground);
        let bits = entity.to_bits();
        match best {
            Some((best_bits, _)) if bits >= best_bits => {}
            _ => best = Some((bits, pos)),
        }
    }
    best.map(|(_, pos)| pos).unwrap_or(Vec2::ZERO)
}

fn spawn_demo_cloud(
    commands: &mut Commands,
    profile: RenderCompatProfile,
    state: &mut DemoCloudSpawnerState,
    center: Vec2,
    orbit: DemoCloudOrbit,
) {
    let layout = SimWorldTransformParams::from_profile(profile);
    let transform = cloud_spawn_transform(center, orbit, &layout);
    let idx = state.next_index;
    state.next_index = state.next_index.saturating_add(1);

    let cloud = EntityBuilder::new()
        .named(format!("demo_cloud_{idx}"))
        .at(Vec2::new(transform.translation.x, transform.translation.z))
        .energy(90.0)
        .volume(0.7)
        .wave(ElementId::from_name("Ventus"))
        .matter(MatterState::Gas, 250.0, 0.2)
        .sim_world_layout(&layout)
        .spawn(commands);

    commands
        .entity(cloud)
        .insert((DemoCloudTag, orbit, transform));
}

/// Startup de nubes de demo. Se refina luego por contexto.
pub fn spawn_demo_clouds_startup_system(
    mut commands: Commands,
    profile: Res<RenderCompatProfile>,
    mut state: ResMut<DemoCloudSpawnerState>,
) {
    if !profile.enables_visual_3d() {
        return;
    }
    for i in 0..4usize {
        let orbit = DemoCloudOrbit {
            angle: std::f32::consts::TAU * i as f32 / 4.0,
            radius: 2.6 + i as f32 * 0.55,
            angular_speed: 0.18 + i as f32 * 0.03,
            altitude: CLOUD_BASE_ALTITUDE + i as f32 * 0.18,
        };
        spawn_demo_cloud(&mut commands, *profile, &mut state, Vec2::ZERO, orbit);
    }
}

/// Balancea cantidad de nubes según contexto eco (presión/reactividad/temperatura).
pub fn demo_cloud_context_spawn_system(
    mut commands: Commands,
    profile: Res<RenderCompatProfile>,
    layout: Res<SimWorldTransformParams>,
    ctx_lookup: ContextLookup,
    mut state: ResMut<DemoCloudSpawnerState>,
    clouds: Query<(Entity, &DemoCloudOrbit), With<DemoCloudTag>>,
    seed_query: Query<(Entity, &Transform), (With<DemoCloudAnchor>, Without<DemoCloudTag>)>,
) {
    if !profile.enables_visual_3d() {
        return;
    }
    let center = demo_seed_center(&layout, &seed_query);
    let ctx = ctx_lookup.context_at(center);
    let target = cloud_target_count(ctx.pressure, ctx.reactivity_mod, ctx.temperature_base);

    let mut existing: Vec<(Entity, DemoCloudOrbit)> = clouds.iter().map(|(e, o)| (e, *o)).collect();
    existing.sort_by_key(|(e, _)| e.to_bits());

    if existing.len() > target {
        for (entity, _) in existing.iter().skip(target) {
            commands.entity(*entity).despawn();
        }
        return;
    }

    if existing.len() < target {
        for i in existing.len()..target {
            let orbit = DemoCloudOrbit {
                angle: std::f32::consts::TAU * i as f32 / target as f32,
                radius: 2.4 + i as f32 * 0.5,
                angular_speed: 0.14 + ctx.pressure.max(0.2) * 0.05 + i as f32 * 0.01,
                altitude: CLOUD_BASE_ALTITUDE + (ctx.reactivity_mod * 0.35) + i as f32 * 0.12,
            };
            spawn_demo_cloud(&mut commands, *profile, &mut state, center, orbit);
        }
    }
}

/// Movimiento orbital y rotación local de nubes.
pub fn demo_cloud_motion_system(
    fixed: Option<Res<Time<Fixed>>>,
    time: Res<Time>,
    layout: Res<SimWorldTransformParams>,
    seed_query: Query<(Entity, &Transform), (With<DemoCloudAnchor>, Without<DemoCloudTag>)>,
    mut clouds: Query<
        (&mut Transform, &mut DemoCloudOrbit),
        (With<DemoCloudTag>, Without<DemoCloudAnchor>),
    >,
) {
    let dt = simulation_delta_secs(fixed, &time);
    if dt <= 0.0 {
        return;
    }
    let center = demo_seed_center(&layout, &seed_query);
    for (mut transform, mut orbit) in &mut clouds {
        orbit.angle = (orbit.angle + orbit.angular_speed * dt).rem_euclid(std::f32::consts::TAU);
        let x = center.x + orbit.radius * orbit.angle.cos();
        let z = center.y + orbit.radius * orbit.angle.sin();
        if layout.use_xz_ground {
            transform.translation = Vec3::new(x, layout.standing_y + orbit.altitude, z);
        } else {
            transform.translation = Vec3::new(x, z, orbit.altitude);
        }
        transform.rotate_y(orbit.angular_speed * 0.35 * dt);
    }
}
