use bevy::math::primitives::{Cuboid, Sphere};
use bevy::pbr::{NotShadowCaster, NotShadowReceiver, StandardMaterial};
use bevy::prelude::*;
use bevy::render::alpha::AlphaMode;

use crate::runtime_platform::compat_2d3d::RenderCompatProfile;
use crate::runtime_platform::core_math_agnostic::DEFAULT_SIM_STANDING_Y;
use crate::simulation::states::{GameState, PlayState};
use crate::worldgen::map_config::ROUND_WORLD_ROSA_MAP_SLUG;
use crate::worldgen::systems::startup::StartupNucleus;
use crate::worldgen::{ActiveMapName, EnergyFieldGrid, EnergyNucleus};

/// Escenarios V6 reproducibles para aislar pruebas de runtime.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum V6Scenario {
    /// Escenario mínimo para caminar en 3D sin demo monolítico.
    #[default]
    Minimal3dWalk,
}

impl V6Scenario {
    pub const fn next(self) -> Self {
        match self {
            Self::Minimal3dWalk => Self::Minimal3dWalk,
        }
    }
}

/// Ownership explícito de entidades creadas por `v6_scenario_isolation`.
#[derive(Component, Debug, Clone, Copy, Default)]
pub struct V6ScenarioOwned;

#[derive(Component, Debug, Clone, Copy, Default)]
struct V6GroundPlane;

#[derive(Component, Debug, Clone, Copy)]
struct V6AtmosphereSlab;

/// Hijo visual: orbe en núcleo de mapa (solo full3d).
#[derive(Component, Debug, Clone, Copy)]
struct V6NucleusOrb;

/// Evita duplicar orbes por núcleo.
#[derive(Component, Debug, Clone, Copy)]
struct V6NucleusOrbSpawned;

/// Extensión horizontal (X y Z) del único bloque de piso cuando no hay grid o perfil 2D.
const V6_DEMO_FLOOR_EXTENT_XZ: f32 = 6.0;
/// Grosor vertical (Y) del bloque.
const V6_DEMO_FLOOR_THICKNESS: f32 = 0.2;
/// Centro Y del bloque: alineado a [`DEFAULT_SIM_STANDING_Y`] (plano sim XZ en full3d).
const V6_DEMO_FLOOR_CENTER_Y: f32 = DEFAULT_SIM_STANDING_Y - V6_DEMO_FLOOR_THICKNESS * 0.5;

/// Alpha de la capa “techo” atmosférica (`V6AtmosphereSlab`). Más bajo = más transparente.
const V6_ATMOSPHERE_SLAB_ALPHA: f32 = 0.003;

/// Estado runtime del escenario activo.
#[derive(Resource, Debug, Clone, Copy, Default)]
pub struct ActiveV6Scenario {
    pub id: V6Scenario,
}

/// Plugin de aislamiento de escenarios (Sprint 11).
pub struct ScenarioIsolationPlugin;

impl Plugin for ScenarioIsolationPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ActiveV6Scenario>()
            .add_systems(Startup, spawn_active_v6_scenario_startup_system)
            .add_systems(
                Update,
                (
                    dev_cycle_v6_scenario_hotkey_system,
                    ensure_v6_ground_plane_system,
                    spawn_v6_nucleus_orb_markers_system,
                    ensure_v6_atmosphere_slab_system,
                ),
            );
    }
}

/// Spawn de escenario V6 con ownership explícito.
///
/// Contrato Sprint 11:
/// - Inputs: `V6Scenario`, `Commands`
/// - Outputs: entidades válidas por capas
/// - Writes: solo entidades marcadas con `V6ScenarioOwned`
/// - NoWrite: recursos runtime/almanac
pub fn spawn_v6_scenario(commands: &mut Commands, scenario: V6Scenario) {
    match scenario {
        V6Scenario::Minimal3dWalk => spawn_minimal_3d_walk(commands),
    }
}

/// Limpia únicamente entidades pertenecientes al escenario V6.
pub fn cleanup_v6_scenario(commands: &mut Commands, entities: &[Entity]) {
    for &entity in entities {
        commands.entity(entity).despawn_recursive();
    }
}

fn spawn_active_v6_scenario_startup_system(mut commands: Commands, active: Res<ActiveV6Scenario>) {
    spawn_v6_demo_environment(&mut commands);
    spawn_v6_scenario(&mut commands, active.id);
}

/// Hotkey dev opcional: F6 cicla escenario y re-instancia entidades V6.
fn dev_cycle_v6_scenario_hotkey_system(
    input: Res<ButtonInput<KeyCode>>,
    mut commands: Commands,
    mut active: ResMut<ActiveV6Scenario>,
    owned_query: Query<Entity, With<V6ScenarioOwned>>,
) {
    if !input.just_pressed(KeyCode::F6) {
        return;
    }

    let to_cleanup: Vec<Entity> = owned_query.iter().collect();
    cleanup_v6_scenario(&mut commands, &to_cleanup);
    active.id = active.id.next();
    spawn_v6_demo_environment(&mut commands);
    spawn_v6_scenario(&mut commands, active.id);
    info!("V6 scenario switched to {:?}", active.id);
}

fn spawn_minimal_3d_walk(_commands: &mut Commands) {
    // Vacío a propósito: el héroe lo spawnea `demo_level`; bioma/partículas duplicaban ruido visual.
    // Luz + piso siguen en `spawn_v6_demo_environment` / `ensure_v6_ground_plane_system`.
}

fn spawn_v6_demo_environment(commands: &mut Commands) {
    commands.spawn((
        DirectionalLight {
            shadows_enabled: true,
            illuminance: 20_000.0,
            ..default()
        },
        Transform::from_xyz(8.0, 16.0, 10.0).looking_at(Vec3::ZERO, Vec3::Y),
        V6ScenarioOwned,
    ));
}

fn ground_extent_xz(profile: Option<&RenderCompatProfile>, grid: Option<&EnergyFieldGrid>) -> f32 {
    let full3d = profile.map(|p| p.enables_visual_3d()).unwrap_or(false);
    if full3d {
        if let Some(g) = grid {
            return (g.width.max(g.height) as f32 * g.cell_size + 8.0).max(V6_DEMO_FLOOR_EXTENT_XZ);
        }
    }
    V6_DEMO_FLOOR_EXTENT_XZ
}

fn nucleus_orb_color(frequency_hz: f32) -> Color {
    if frequency_hz < 120.0 {
        Color::srgb(0.42, 0.55, 0.22)
    } else if frequency_hz < 350.0 {
        Color::srgb(0.15, 0.45, 0.85)
    } else if frequency_hz < 550.0 {
        Color::srgb(0.95, 0.35, 0.08)
    } else if frequency_hz < 850.0 {
        Color::srgb(0.55, 0.92, 0.65)
    } else {
        Color::srgb(0.95, 0.95, 0.75)
    }
}

/// Spawn visual: losa al tamaño del `EnergyFieldGrid` en full3d; cubo chico en 2D/tests.
/// El color del mosaico viene de celdas materializadas; ver `docs/DEMO_FLOW.md`.
fn ensure_v6_ground_plane_system(
    mut commands: Commands,
    mut meshes: Option<ResMut<Assets<Mesh>>>,
    mut materials: Option<ResMut<Assets<StandardMaterial>>>,
    profile: Option<Res<RenderCompatProfile>>,
    grid: Option<Res<EnergyFieldGrid>>,
    active_map: Option<Res<ActiveMapName>>,
    owned: Query<Entity, With<V6ScenarioOwned>>,
    ground: Query<Entity, With<V6GroundPlane>>,
) {
    if active_map.as_ref().is_some_and(|m| m.0 == ROUND_WORLD_ROSA_MAP_SLUG) {
        return;
    }
    if owned.iter().next().is_none() || ground.iter().next().is_some() {
        return;
    }
    let Some(meshes) = meshes.as_deref_mut() else {
        return;
    };
    let Some(materials) = materials.as_deref_mut() else {
        return;
    };

    let extent = ground_extent_xz(profile.as_deref(), grid.as_deref());
    let mesh = meshes.add(Mesh::from(Cuboid::from_size(Vec3::new(
        extent,
        V6_DEMO_FLOOR_THICKNESS,
        extent,
    ))));
    let material = materials.add(StandardMaterial {
        base_color: Color::srgb(0.12, 0.16, 0.14),
        perceptual_roughness: 0.95,
        metallic: 0.0,
        ..default()
    });
    commands.spawn((
        V6GroundPlane,
        Mesh3d(mesh),
        MeshMaterial3d(material),
        Transform::from_xyz(0.0, V6_DEMO_FLOOR_CENTER_Y, 0.0),
        V6ScenarioOwned,
    ));
}

/// Orbes esféricos en núcleos `StartupNucleus` (Terra ≈ verde oliva, Ventus ≈ menta, etc.).
fn spawn_v6_nucleus_orb_markers_system(
    mut commands: Commands,
    mut meshes: Option<ResMut<Assets<Mesh>>>,
    mut materials: Option<ResMut<Assets<StandardMaterial>>>,
    profile: Option<Res<RenderCompatProfile>>,
    game: Option<Res<State<GameState>>>,
    play: Option<Res<State<PlayState>>>,
    mut q: Query<
        (Entity, &EnergyNucleus, &Transform),
        (With<StartupNucleus>, Without<V6NucleusOrbSpawned>),
    >,
) {
    let Some(meshes) = meshes.as_deref_mut() else {
        return;
    };
    let Some(materials) = materials.as_deref_mut() else {
        return;
    };
    if !profile
        .as_ref()
        .map(|p| p.enables_visual_3d())
        .unwrap_or(false)
    {
        return;
    }
    let gameplay_active = game.is_some_and(|s| s.get() == &GameState::Playing)
        && play.is_some_and(|s| s.get() == &PlayState::Active);
    if !gameplay_active {
        return;
    }
    for (entity, nucleus, _tf) in &mut q {
        let r = (nucleus.propagation_radius() * 0.065).clamp(0.35, 2.4);
        let mesh = meshes.add(Mesh::from(Sphere::new(r)));
        let mat = materials.add(StandardMaterial {
            base_color: nucleus_orb_color(nucleus.frequency_hz()),
            perceptual_roughness: 0.45,
            metallic: 0.15,
            ..default()
        });
        commands.entity(entity).with_children(|parent| {
            parent.spawn((
                V6NucleusOrb,
                Mesh3d(mesh),
                MeshMaterial3d(mat),
                Transform::from_translation(Vec3::Y * (r * 1.1)),
            ));
        });
        commands.entity(entity).insert(V6NucleusOrbSpawned);
    }
}

/// Losa fina semitransparente: metáfora visual de “atmósfera” sobre el campo (no otro grid).
fn ensure_v6_atmosphere_slab_system(
    mut commands: Commands,
    mut meshes: Option<ResMut<Assets<Mesh>>>,
    mut materials: Option<ResMut<Assets<StandardMaterial>>>,
    profile: Option<Res<RenderCompatProfile>>,
    grid: Option<Res<EnergyFieldGrid>>,
    active_map: Option<Res<ActiveMapName>>,
    game: Option<Res<State<GameState>>>,
    play: Option<Res<State<PlayState>>>,
    existing: Query<Entity, With<V6AtmosphereSlab>>,
) {
    if active_map.as_ref().is_some_and(|m| m.0 == ROUND_WORLD_ROSA_MAP_SLUG) {
        return;
    }
    let Some(meshes) = meshes.as_deref_mut() else {
        return;
    };
    let Some(materials) = materials.as_deref_mut() else {
        return;
    };
    if !profile
        .as_ref()
        .map(|p| p.enables_visual_3d())
        .unwrap_or(false)
    {
        return;
    }
    let gameplay_active = game.is_some_and(|s| s.get() == &GameState::Playing)
        && play.is_some_and(|s| s.get() == &PlayState::Active);
    if !gameplay_active {
        return;
    }
    if existing.iter().next().is_some() {
        return;
    }
    let Some(grid) = grid else {
        return;
    };
    let extent = ground_extent_xz(profile.as_deref(), Some(grid.as_ref()));
    let thickness = 0.14_f32;
    let y = DEFAULT_SIM_STANDING_Y + 9.0;
    let mesh = meshes.add(Mesh::from(Cuboid::from_size(Vec3::new(
        extent * 0.98,
        thickness,
        extent * 0.98,
    ))));
    // Capa tipo atmósfera: baja opacidad + unlit; sin sombras (transparente ≠ proyectar/recibir sombra dura).
    let material = materials.add(StandardMaterial {
        base_color: Color::srgba(0.78, 0.9, 1.0, V6_ATMOSPHERE_SLAB_ALPHA),
        alpha_mode: AlphaMode::Blend,
        unlit: true,
        double_sided: true,
        // Sin mezcla con distance fog del PBR: evita teñido global tipo “todo negro”.
        fog_enabled: false,
        perceptual_roughness: 1.0,
        metallic: 0.0,
        ..default()
    });
    commands.spawn((
        V6AtmosphereSlab,
        Mesh3d(mesh),
        MeshMaterial3d(material),
        Transform::from_xyz(0.0, y, 0.0),
        V6ScenarioOwned,
        NotShadowCaster,
        NotShadowReceiver,
    ));
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_app() -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .init_resource::<ButtonInput<KeyCode>>()
            .add_plugins(ScenarioIsolationPlugin);
        app
    }

    #[test]
    fn startup_spawns_owned_entities() {
        let mut app = test_app();
        app.update();

        let world = app.world_mut();
        let owned_q = world.query::<&V6ScenarioOwned>().iter(world).count();
        assert!(owned_q >= 1);
    }

    #[test]
    fn hotkey_rebuild_keeps_owned_population() {
        let mut app = test_app();
        app.update();

        {
            let mut input = app.world_mut().resource_mut::<ButtonInput<KeyCode>>();
            input.press(KeyCode::F6);
        }
        app.update();
        {
            let mut input = app.world_mut().resource_mut::<ButtonInput<KeyCode>>();
            input.release(KeyCode::F6);
        }
        app.update();

        let world = app.world_mut();
        let mut owned_q = world.query::<&V6ScenarioOwned>();
        assert!(owned_q.iter(world).count() >= 1);
    }
}
