use bevy::prelude::*;

use crate::blueprint::constants::{
    BIOME_DESERT_DELTA_QE, BIOME_DESERT_VISCOSITY, BIOME_LEY_LINE_DELTA_QE,
    BIOME_LEY_LINE_VISCOSITY, BIOME_PLAIN_DELTA_QE, BIOME_PLAIN_VISCOSITY, BIOME_SWAMP_DELTA_QE,
    BIOME_SWAMP_VISCOSITY, BIOME_TUNDRA_DELTA_QE, BIOME_TUNDRA_VISCOSITY, BIOME_VOLCANO_DELTA_QE,
    BIOME_VOLCANO_VISCOSITY,
};
use crate::blueprint::recipes::EffectRecipe;
use crate::blueprint::{ElementId, IdGenerator};
use crate::entities::builder::EntityBuilder;
use crate::entities::composition::{EffectConfig, InjectorConfig, PhysicsConfig};
use crate::layers::{Homeostasis, MatterState, TensionField};
use crate::runtime_platform::compat_2d3d::SimWorldTransformParams;
use crate::runtime_platform::hud::MinimapIcon;
use crate::simulation::SpellMarker;
use crate::simulation::states::GameState;

/// Tipos de bioma predefinidos.
#[derive(Debug, Clone, Copy)]
pub enum BiomeType {
    Plain,
    Volcano,
    LeyLine,
    Swamp,
    Tundra,
    Desert,
}

/// Preset de spawn para `spawn_biome`: presión ambiental + energía + coherencia.
#[derive(Clone, Copy, Debug)]
struct BiomeSpawnPreset {
    /// Inyección/robo ambiental (Capa 6), alineado con `AmbientPressure`.
    delta_qe: f32,
    terrain_viscosity: f32,
    /// qe inicial de la entidad bioma.
    energy: f32,
    matter_state: MatterState,
    thermal_conductivity: f32,
    bond_energy_eb: f32,
}

pub(super) fn minimap_icon_for_biome(biome: BiomeType) -> MinimapIcon {
    let c = match biome {
        BiomeType::Volcano => Color::srgb(0.55, 0.18, 0.12),
        BiomeType::Swamp => Color::srgb(0.2, 0.38, 0.22),
        BiomeType::LeyLine => Color::srgb(0.45, 0.35, 0.75),
        BiomeType::Tundra => Color::srgb(0.75, 0.82, 0.92),
        BiomeType::Desert => Color::srgb(0.78, 0.65, 0.35),
        BiomeType::Plain => Color::srgb(0.35, 0.5, 0.28),
    };
    MinimapIcon::new(14.0, c)
}

impl BiomeType {
    fn spawn_preset(self) -> BiomeSpawnPreset {
        match self {
            BiomeType::Plain => BiomeSpawnPreset {
                delta_qe: BIOME_PLAIN_DELTA_QE,
                terrain_viscosity: BIOME_PLAIN_VISCOSITY,
                energy: 50.0,
                matter_state: MatterState::Solid,
                thermal_conductivity: 0.2,
                bond_energy_eb: 5000.0,
            },
            BiomeType::Volcano => BiomeSpawnPreset {
                delta_qe: BIOME_VOLCANO_DELTA_QE,
                terrain_viscosity: BIOME_VOLCANO_VISCOSITY,
                energy: 500.0,
                matter_state: MatterState::Plasma,
                thermal_conductivity: 0.9,
                bond_energy_eb: 6000.0,
            },
            BiomeType::LeyLine => BiomeSpawnPreset {
                delta_qe: BIOME_LEY_LINE_DELTA_QE,
                terrain_viscosity: BIOME_LEY_LINE_VISCOSITY,
                energy: 200.0,
                matter_state: MatterState::Gas,
                thermal_conductivity: 0.25,
                bond_energy_eb: 2000.0,
            },
            BiomeType::Swamp => BiomeSpawnPreset {
                delta_qe: BIOME_SWAMP_DELTA_QE,
                terrain_viscosity: BIOME_SWAMP_VISCOSITY,
                energy: 100.0,
                matter_state: MatterState::Liquid,
                thermal_conductivity: 0.6,
                bond_energy_eb: 4000.0,
            },
            BiomeType::Tundra => BiomeSpawnPreset {
                delta_qe: BIOME_TUNDRA_DELTA_QE,
                terrain_viscosity: BIOME_TUNDRA_VISCOSITY,
                energy: 80.0,
                matter_state: MatterState::Solid,
                thermal_conductivity: 0.4,
                bond_energy_eb: 8000.0,
            },
            BiomeType::Desert => BiomeSpawnPreset {
                delta_qe: BIOME_DESERT_DELTA_QE,
                terrain_viscosity: BIOME_DESERT_VISCOSITY,
                energy: 30.0,
                matter_state: MatterState::Solid,
                thermal_conductivity: 0.7,
                bond_energy_eb: 8000.0,
            },
        }
    }
}

/// Entidad-efecto (Capa 10): modificador temporal enlazado a un target.
pub fn spawn_effect(
    commands: &mut Commands,
    id_gen: &mut IdGenerator,
    effect: EffectConfig,
) -> Entity {
    let eid = id_gen.next_effect();
    let (e, f, r) = effect.spawn_components();
    commands.spawn((eid, e, f, r)).id()
}

/// Entidad-efecto L10 con `Transform` en el plano de sim (V6 XZ o XY legacy).
pub fn spawn_resonance_effect(
    commands: &mut Commands,
    id_gen: &mut IdGenerator,
    layout: &SimWorldTransformParams,
    at: Vec2,
    effect: EffectConfig,
) -> Entity {
    let eid = id_gen.next_effect();
    let translation = if layout.use_xz_ground {
        Vec3::new(at.x, layout.standing_y, at.y)
    } else {
        Vec3::new(at.x, at.y, 0.0)
    };
    let (e, f, r) = effect.spawn_components();
    commands
        .spawn((
            Transform::from_translation(translation),
            Visibility::default(),
            Name::new("resonance_effect"),
            eid,
            e,
            f,
            r,
        ))
        .id()
}

/// Ancla estática L11 (pozo de tensión, trampa, vórtice).
pub fn spawn_tension_entity(
    commands: &mut Commands,
    id_gen: &mut IdGenerator,
    pos: Vec2,
    qe: f32,
    radius: f32,
    element_id: ElementId,
    field: TensionField,
    layout: &SimWorldTransformParams,
    name: &str,
) -> Entity {
    let wid = id_gen.next_world();
    let e = EntityBuilder::new()
        .named(format!("tension_{name}"))
        .at(pos)
        .energy(qe)
        .volume(radius)
        .wave(element_id)
        .flow(Vec2::ZERO, 0.0)
        .tension_field(field)
        .sim_world_layout(layout)
        .spawn(commands);
    commands.entity(e).insert(wid);
    e
}

/// Entidad estática con homeostasis (L12) + materia mínima para labels/debug.
pub fn spawn_adaptive_entity(
    commands: &mut Commands,
    id_gen: &mut IdGenerator,
    pos: Vec2,
    qe: f32,
    radius: f32,
    element_id: ElementId,
    homeostasis: Homeostasis,
    layout: &SimWorldTransformParams,
    name: &str,
) -> Entity {
    let wid = id_gen.next_world();
    let e = EntityBuilder::new()
        .named(format!("adaptive_{name}"))
        .at(pos)
        .energy(qe)
        .volume(radius)
        .wave(element_id)
        .flow(Vec2::ZERO, 0.5)
        .matter(MatterState::Solid, 5000.0, 0.2)
        .homeostasis(homeostasis)
        .sim_world_layout(layout)
        .spawn(commands);
    commands.entity(e).insert(wid);
    e
}

pub fn spawn_projectile(
    commands: &mut Commands,
    id_gen: &mut IdGenerator,
    caster: Option<Entity>,
    physics: PhysicsConfig,
    injector: InjectorConfig,
    on_contact_effect: Option<EffectRecipe>,
    despawn_on_contact: bool,
    layout: &SimWorldTransformParams,
) -> Entity {
    let eid = id_gen.next_effect();
    let entity = EntityBuilder::new()
        .named("projectile")
        .at(physics.pos)
        .energy(physics.qe)
        .volume(physics.radius)
        .wave(physics.element_id)
        .flow(physics.velocity, physics.dissipation)
        .injector(
            injector.projected_qe,
            injector.forced_frequency,
            injector.influence_radius,
        )
        .sim_world_layout(layout)
        .spawn(commands);

    // `GameState::Paused` del sprint G2 despawnea scoped al salir de `Playing`; el héroe no va aquí.
    commands
        .entity(entity)
        .insert((eid, SpellMarker { caster }, StateScoped(GameState::Playing)));
    if despawn_on_contact {
        commands
            .entity(entity)
            .insert(crate::layers::DespawnOnContact);
    }
    if let Some(recipe) = on_contact_effect {
        commands
            .entity(entity)
            .insert(crate::layers::OnContactEffect { recipe });
    }
    entity
}

pub fn spawn_crystal(
    commands: &mut Commands,
    id_gen: &mut IdGenerator,
    pos: Vec2,
    qe: f32,
    element_id: ElementId,
    layout: &SimWorldTransformParams,
) -> Entity {
    let wid = id_gen.next_world();
    let e = EntityBuilder::new()
        .named("crystal")
        .at(pos)
        .energy(qe)
        .volume(0.6)
        .wave(element_id)
        .flow(Vec2::ZERO, 0.0)
        .matter(MatterState::Solid, 8000.0, 0.1)
        .motor(qe * 2.0, 2.0, 0.0, 0.0)
        .sim_world_layout(layout)
        .spawn(commands);
    commands
        .entity(e)
        .insert((wid, MinimapIcon::crystal(element_id)));
    e
}

pub fn spawn_biome(
    commands: &mut Commands,
    id_gen: &mut IdGenerator,
    pos: Vec2,
    radius: f32,
    biome: BiomeType,
    layout: &SimWorldTransformParams,
) -> Entity {
    let p = biome.spawn_preset();
    let wid = id_gen.next_world();

    // El bioma participa como host por Capa 6 + geometría, sin branching por "tipo".
    let e = EntityBuilder::new()
        .named(format!("biome_{biome:?}"))
        .at(pos)
        .energy(p.energy)
        .volume(radius)
        .wave(ElementId::from_name("Terra"))
        .flow(Vec2::ZERO, 0.0)
        .ambient(p.delta_qe, p.terrain_viscosity)
        .matter(p.matter_state, p.bond_energy_eb, p.thermal_conductivity)
        .sim_world_layout(layout)
        .spawn(commands);
    commands
        .entity(e)
        .insert((wid, minimap_icon_for_biome(biome)));
    e
}

/// Entidad "partícula" (Sprint 01): minimiza capas
/// y deja que los sistemas actúen por presencia de componentes.
pub fn spawn_particle(
    commands: &mut Commands,
    id_gen: &mut IdGenerator,
    pos: Vec2,
    qe: f32,
    radius: f32,
    element_id: ElementId,
    layout: &SimWorldTransformParams,
) -> Entity {
    let wid = id_gen.next_world();
    let e = EntityBuilder::new()
        .named("particle")
        .at(pos)
        .energy(qe)
        .volume(radius)
        .wave(element_id)
        .flow(Vec2::ZERO, 0.0)
        .sim_world_layout(layout)
        .spawn(commands);
    commands.entity(e).insert(wid);
    e
}

/// Entidad "piedra" (Sprint 01): misma base que la partícula,
/// pero con Capa 4 (Materia Sólida) para que existan efectos que
/// solo pueden ocurrir si la capa está presente.
pub fn spawn_stone(
    commands: &mut Commands,
    id_gen: &mut IdGenerator,
    pos: Vec2,
    qe: f32,
    radius: f32,
    element_id: ElementId,
    layout: &SimWorldTransformParams,
) -> Entity {
    let wid = id_gen.next_world();
    let e = EntityBuilder::new()
        .named("stone")
        .at(pos)
        .energy(qe)
        .volume(radius)
        .wave(element_id)
        .flow(Vec2::ZERO, 0.0)
        .matter(MatterState::Solid, 8000.0, 0.1)
        .sim_world_layout(layout)
        .spawn(commands);
    commands.entity(e).insert(wid);
    e
}

/// Entidad "caballero de lava" (Sprint 01): mismas interacciones emergen por
/// composición de capas (capas extra habilitan motor, movimiento, ambient y catálisis).
pub fn spawn_lava_knight(
    commands: &mut Commands,
    id_gen: &mut IdGenerator,
    pos: Vec2,
    element_id: ElementId,
    forced_frequency: f32,
    layout: &SimWorldTransformParams,
) -> Entity {
    let wid = id_gen.next_world();
    // `forced_frequency` se usa para lock de frecuencia en catálisis constructiva.
    let entity = EntityBuilder::new()
        .named("lava_knight")
        .at(pos)
        .energy(800.0)
        .volume(1.2)
        .wave(element_id)
        .flow(Vec2::ZERO, 3.0)
        .matter(MatterState::Plasma, 6000.0, 0.9)
        .motor(600.0, 20.0, 50.0, 300.0)
        .ambient(-5.0, 2.5)
        .will_default()
        .injector(20.0, forced_frequency.max(0.0), 3.0)
        .sim_world_layout(layout)
        .spawn(commands);

    // Habilita catálisis_resolution_system.
    commands
        .entity(entity)
        .insert((wid, SpellMarker { caster: None }));
    entity
}
