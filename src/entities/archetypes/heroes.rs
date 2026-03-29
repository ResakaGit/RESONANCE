use bevy::prelude::*;

use crate::blueprint::constants::{FOG_DEFAULT_PROVIDER_RADIUS, FOG_DEFAULT_SENSITIVITY};
use crate::blueprint::{ElementId, IdGenerator};
use crate::entities::builder::EntityBuilder;
use crate::entities::composition::{EngineConfig, MatterConfig, PhysicsConfig};
use crate::layers::{
    Faction, Homeostasis, MatterState, RelationalTag, StructuralLink, TensionField,
    VisionFogAnchor, VisionProvider,
};
use crate::runtime_platform::compat_2d3d::SimWorldTransformParams;
use crate::runtime_platform::hud::MinimapIcon;
use crate::simulation::PlayerControlled;
use crate::simulation::pathfinding::{NavAgent, NavPath};
use crate::world::fog_team_index;

/// Clases de héroe con valores base pre-configurados.
#[derive(Debug, Clone, Copy)]
pub enum HeroClass {
    /// Mago de fuego: alta energía, afinidad Ignis, buffer grande
    FireMage,
    /// Guerrero de tierra: alta coherencia, afinidad Terra, resistente
    EarthWarrior,
    /// Asesino de sombra: baja energía, alta velocidad, afinidad Umbra
    PlantAssassin,
    /// Sanador de luz: buffer grande de salida, afinidad Lux
    LightHealer,
    /// Tirador de viento: largo alcance, afinidad Ventus
    WindShooter,
    /// Tanque de agua: máxima coherencia, afinidad Aqua
    WaterTank,
}

/// Stats numéricas de héroe antes de ensamblar `PhysicsConfig` / `MatterConfig` / `EngineConfig`.
#[derive(Clone, Copy, Debug)]
struct HeroSpawnPreset {
    qe: f32,
    radius: f32,
    bond_energy: f32,
    conductivity: f32,
    max_buffer: f32,
    input_valve: f32,
    output_valve: f32,
    dissipation: f32,
    critical_multiplier: f32,
}

/// Capas opcionales L11/L12/L13 sobre un héroe (sin tocar identidad base).
#[derive(Clone, Debug, Default)]
pub struct HeroLayerAddons {
    pub tension_field: Option<TensionField>,
    pub homeostasis: Option<Homeostasis>,
    pub structural_link: Option<StructuralLink>,
}

/// Héroe con composición extendida; `player_controlled` + `grimoire` solo para el jugador.
pub fn spawn_hero_layers(
    commands: &mut Commands,
    id_gen: &mut IdGenerator,
    class: HeroClass,
    faction: Faction,
    pos: Vec2,
    layout: &SimWorldTransformParams,
    addons: HeroLayerAddons,
    player_controlled: bool,
    grimoire: Option<crate::layers::will::Grimoire>,
) -> Entity {
    let champion_id = id_gen.next_champion();
    let (mut physics, matter, engine, crit) = hero_preset(class);
    physics.pos = pos;

    let mut b = EntityBuilder::new()
        .named("hero")
        .observe_hero_base_energy_spawn()
        .at(physics.pos)
        .energy(physics.qe)
        .volume(physics.radius)
        .wave(physics.element_id)
        .flow(physics.velocity, physics.dissipation)
        .matter(matter.state, matter.bond_energy, matter.conductivity)
        .motor(
            engine.max_buffer,
            engine.input_valve,
            engine.output_valve,
            engine.initial_buffer,
        )
        .will_default()
        .identity(faction, RelationalTag::Hero.bit(), crit)
        .sim_world_layout(layout);

    if let Some(tf) = addons.tension_field {
        b = b.tension_field(tf);
    }
    if let Some(h) = addons.homeostasis {
        b = b.homeostasis(h);
    }
    if let Some(s) = addons.structural_link {
        b = b.structural_link(s);
    }

    let entity = b.spawn(commands);

    commands
        .entity(entity)
        .insert((champion_id, MinimapIcon::hero_faction(faction)));

    if let Some(team) = fog_team_index(faction) {
        commands.entity(entity).insert((
            VisionProvider::new(FOG_DEFAULT_PROVIDER_RADIUS, FOG_DEFAULT_SENSITIVITY, team),
            VisionFogAnchor::default(),
        ));
    }

    if player_controlled {
        let g = grimoire.unwrap_or_default();
        let r = physics.radius;
        commands
            .entity(entity)
            .insert((PlayerControlled, g, NavAgent::new(r), NavPath::default()));
    }

    entity
}

pub fn spawn_hero(
    commands: &mut Commands,
    id_gen: &mut IdGenerator,
    class: HeroClass,
    faction: Faction,
    pos: Vec2,
    layout: &SimWorldTransformParams,
) -> Entity {
    spawn_hero_layers(
        commands,
        id_gen,
        class,
        faction,
        pos,
        layout,
        HeroLayerAddons::default(),
        true,
        Some(crate::layers::will::Grimoire::default()),
    )
}

/// Dummy pasivo: catálisis / coherencia sin motor ni voluntad (L0–L4).
pub fn spawn_dummy(
    commands: &mut Commands,
    id_gen: &mut IdGenerator,
    pos: Vec2,
    qe: f32,
    radius: f32,
    element_id: ElementId,
    layout: &SimWorldTransformParams,
    label: &str,
) -> Entity {
    let wid = id_gen.next_world();
    let e = EntityBuilder::new()
        .named(format!("dummy_{label}"))
        .at(pos)
        .energy(qe)
        .volume(radius)
        .wave(element_id)
        .flow(Vec2::ZERO, 1.0)
        .matter(MatterState::Solid, 12_000.0, 0.35)
        .sim_world_layout(layout)
        .spawn(commands);
    commands.entity(e).insert(wid);
    e
}

fn hero_element_id(class: HeroClass) -> ElementId {
    match class {
        HeroClass::FireMage => ElementId::from_name("Ignis"),
        HeroClass::EarthWarrior => ElementId::from_name("Terra"),
        HeroClass::PlantAssassin => ElementId::from_name("Umbra"),
        HeroClass::LightHealer => ElementId::from_name("Lux"),
        HeroClass::WindShooter => ElementId::from_name("Ventus"),
        HeroClass::WaterTank => ElementId::from_name("Aqua"),
    }
}

fn hero_spawn_preset(class: HeroClass) -> HeroSpawnPreset {
    match class {
        HeroClass::FireMage => HeroSpawnPreset {
            qe: 500.0,
            radius: 0.8,
            bond_energy: 2000.0,
            conductivity: 0.6,
            max_buffer: 1500.0,
            input_valve: 8.0,
            output_valve: 80.0,
            dissipation: 3.0,
            critical_multiplier: 1.8,
        },
        HeroClass::EarthWarrior => HeroSpawnPreset {
            qe: 800.0,
            radius: 1.2,
            bond_energy: 8000.0,
            conductivity: 0.1,
            max_buffer: 500.0,
            input_valve: 15.0,
            output_valve: 30.0,
            dissipation: 2.0,
            critical_multiplier: 1.5,
        },
        HeroClass::PlantAssassin => HeroSpawnPreset {
            qe: 300.0,
            radius: 0.5,
            bond_energy: 1500.0,
            conductivity: 0.3,
            max_buffer: 800.0,
            input_valve: 20.0,
            output_valve: 100.0,
            dissipation: 8.0,
            critical_multiplier: 2.5,
        },
        HeroClass::LightHealer => HeroSpawnPreset {
            qe: 400.0,
            radius: 0.9,
            bond_energy: 3000.0,
            conductivity: 0.4,
            max_buffer: 2000.0,
            input_valve: 12.0,
            output_valve: 60.0,
            dissipation: 2.0,
            critical_multiplier: 1.5,
        },
        HeroClass::WindShooter => HeroSpawnPreset {
            qe: 350.0,
            radius: 0.7,
            bond_energy: 2500.0,
            conductivity: 0.3,
            max_buffer: 1000.0,
            input_valve: 10.0,
            output_valve: 70.0,
            dissipation: 4.0,
            critical_multiplier: 1.5,
        },
        HeroClass::WaterTank => HeroSpawnPreset {
            qe: 1000.0,
            radius: 1.5,
            bond_energy: 10000.0,
            conductivity: 0.05,
            max_buffer: 300.0,
            input_valve: 5.0,
            output_valve: 20.0,
            dissipation: 1.0,
            critical_multiplier: 1.5,
        },
    }
}

fn hero_preset(class: HeroClass) -> (PhysicsConfig, MatterConfig, EngineConfig, f32) {
    let h = hero_spawn_preset(class);

    let physics = PhysicsConfig {
        pos: Vec2::ZERO,
        qe: h.qe,
        radius: h.radius,
        element_id: hero_element_id(class),
        velocity: Vec2::ZERO,
        dissipation: h.dissipation,
    };

    let matter = MatterConfig {
        state: MatterState::Solid,
        bond_energy: h.bond_energy,
        conductivity: h.conductivity,
    };

    let engine = EngineConfig {
        max_buffer: h.max_buffer,
        input_valve: h.input_valve,
        output_valve: h.output_valve,
        initial_buffer: h.max_buffer * 0.5,
    };

    (physics, matter, engine, h.critical_multiplier)
}
