use bevy::prelude::*;

use crate::blueprint::constants::{
    BIOME_DESERT_DELTA_QE, BIOME_DESERT_VISCOSITY, BIOME_LEY_LINE_DELTA_QE,
    BIOME_LEY_LINE_VISCOSITY, BIOME_PLAIN_DELTA_QE, BIOME_PLAIN_VISCOSITY, BIOME_SWAMP_DELTA_QE,
    BIOME_SWAMP_VISCOSITY, BIOME_TUNDRA_DELTA_QE, BIOME_TUNDRA_VISCOSITY, BIOME_VOLCANO_DELTA_QE,
    BIOME_VOLCANO_VISCOSITY, FOG_DEFAULT_PROVIDER_RADIUS, FOG_DEFAULT_SENSITIVITY,
};
use crate::blueprint::recipes::EffectRecipe;
use crate::blueprint::{ElementId, IdGenerator};
use crate::entities::builder::EntityBuilder;
use crate::entities::constants::{
    flora_ea2, morphogenesis_mg8, FloraSpawnPreset, MorphogenesisSpawnPreset,
    FLORA_ELEMENT_SYMBOL, FLORA_GROWTH_LIMITER, FLORA_MINIMAP_ICON_RADIUS,
};
use crate::entities::composition::{
    EffectConfig, EngineConfig, InjectorConfig, MatterConfig, PhysicsConfig,
};
use crate::layers::{
    CapabilitySet, Faction, Homeostasis, InferenceProfile, LifecycleStage, MatterState,
    OrganManifest, OrganSpec, RelationalTag, StructuralLink, TensionField,
    VisionFogAnchor, VisionProvider,
};
use crate::runtime_platform::compat_2d3d::SimWorldTransformParams;
use crate::runtime_platform::hud::MinimapIcon;
use crate::simulation::PlayerControlled;
use crate::simulation::SpellMarker;
use crate::simulation::pathfinding::{NavAgent, NavPath};
use crate::simulation::states::GameState;
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

fn minimap_icon_for_biome(biome: BiomeType) -> MinimapIcon {
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
        .identity(faction, vec![RelationalTag::Hero], crit)
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

/// Instancia una semilla botánica (Single-Plant Sandbox) acoplada a la tubería metabólica.
pub fn spawn_botanical_seed(
    commands: &mut Commands,
    id_gen: &mut IdGenerator,
    pos: Vec2,
    layout: &SimWorldTransformParams,
) -> Entity {
    let wid = id_gen.next_world();
    let qe = 150.0;
    let radius = 0.5;
    
    // Plantas: Dominio Umbra (20.0 Hz) absorbiendo luz (Lux/Ignis) y agua.
    let element_id = ElementId::from_name("Umbra");
    
    let e = EntityBuilder::new()
        .named("botanical_seed")
        .at(pos)
        .energy(qe)
        .volume(radius)
        .wave(element_id)
        .flow(Vec2::ZERO, 0.5)
        .matter(MatterState::Solid, 3000.0, 0.8)
        .nutrient(50.0, 50.0, 50.0, 50.0) // Carbon, Nitrogen, Phosphor, Water crudos
        .growth_budget(0.0, 0, 0.8) // 0 Biomasa inicial, eficiencia 80%
        .sim_world_layout(layout)
        .spawn(commands);
        
    commands.entity(e).insert((
        wid,
        MinimapIcon::new(12.0, Color::srgb(0.2, 0.8, 0.2)),
    ));
    e
}

// ── MG-8: Arquetipos de morfogénesis inferida ────────────────────────────────
// Organismos cuya forma, color y textura emergen de la termodinámica.
// Presets numéricos: `entities/constants.rs` → `morphogenesis_mg8`.

fn spawn_morphogenesis_from_preset(
    commands: &mut Commands,
    position: Vec2,
    preset: MorphogenesisSpawnPreset,
) -> Entity {
    let mut manifest = OrganManifest::new(LifecycleStage::Mature);
    for &role in preset.roles {
        manifest.push(OrganSpec::new(role, 1, 1.0));
    }
    let velocity = if preset.velocity_x != 0.0 {
        Vec2::new(preset.velocity_x, 0.0)
    } else {
        Vec2::ZERO
    };
    EntityBuilder::new()
        .named(preset.entity_name)
        .at(position)
        .energy(preset.qe)
        .volume(preset.radius)
        .flow(velocity, preset.dissipation)
        .ambient(preset.delta_qe, preset.viscosity)
        .irradiance(preset.photon_density, preset.absorbed_fraction)
        .with_organ_manifest(manifest)
        .with_metabolic_graph_inferred(preset.t_core_build, preset.t_env_build)
        .spawn(commands)
}

/// Organismo acuático: fusiforme, oscuro, liso.
pub fn spawn_aquatic_organism(commands: &mut Commands, position: Vec2) -> Entity {
    spawn_morphogenesis_from_preset(commands, position, morphogenesis_mg8::AQUATIC_ORGANISM)
}

/// Planta desértica: compacta, clara, radiadores.
pub fn spawn_desert_plant(commands: &mut Commands, position: Vec2) -> Entity {
    spawn_morphogenesis_from_preset(commands, position, morphogenesis_mg8::DESERT_PLANT)
}

/// Criatura desértica: ligeramente alargada, clara, crestas.
pub fn spawn_desert_creature(commands: &mut Commands, position: Vec2) -> Entity {
    spawn_morphogenesis_from_preset(commands, position, morphogenesis_mg8::DESERT_CREATURE)
}

/// Planta de bosque: forma intermedia, color medio.
pub fn spawn_forest_plant(commands: &mut Commands, position: Vec2) -> Entity {
    spawn_morphogenesis_from_preset(commands, position, morphogenesis_mg8::FOREST_PLANT)
}

// ── Flora: variantes por composición de capas (EA2) ─────────────────────────
// Presets numéricos: `entities/constants.rs` → `flora_ea2`.

fn spawn_flora_from_preset(
    commands: &mut Commands,
    id_gen: &mut IdGenerator,
    pos: Vec2,
    layout: &SimWorldTransformParams,
    preset: FloraSpawnPreset,
) -> Entity {
    let wid = id_gen.next_world();
    let entity = EntityBuilder::new()
        .named(preset.entity_name)
        .at(pos)
        .energy(preset.qe)
        .volume(preset.radius)
        .wave(ElementId::from_name(FLORA_ELEMENT_SYMBOL))
        .flow(Vec2::ZERO, preset.flow_dissipation)
        .matter(
            MatterState::Solid,
            preset.bond_energy_eb,
            preset.thermal_conductivity,
        )
        .nutrient(
            preset.nutrient_c,
            preset.nutrient_n,
            preset.nutrient_p,
            preset.nutrient_w,
        )
        .growth_budget(
            preset.growth_biomass,
            FLORA_GROWTH_LIMITER,
            preset.growth_efficiency,
        )
        .sim_world_layout(layout)
        .spawn(commands);

    let tint = Color::srgb(
        preset.minimap_rgb[0],
        preset.minimap_rgb[1],
        preset.minimap_rgb[2],
    );
    commands.entity(entity).insert((
        wid,
        MinimapIcon::new(FLORA_MINIMAP_ICON_RADIUS, tint),
        InferenceProfile::new(
            preset.growth_bias,
            preset.mobility_bias,
            preset.branching_bias,
            preset.resilience,
        ),
        CapabilitySet::new(preset.capability_flags),
    ));
    entity
}

/// Rosa: flexible, crece rápido, ramifica mucho, no se mueve.
pub fn spawn_rosa(
    commands: &mut Commands,
    id_gen: &mut IdGenerator,
    pos: Vec2,
    layout: &SimWorldTransformParams,
) -> Entity {
    spawn_flora_from_preset(commands, id_gen, pos, layout, flora_ea2::ROSA)
}

/// Roble: rígido, crece lento, muy resiliente.
pub fn spawn_oak(
    commands: &mut Commands,
    id_gen: &mut IdGenerator,
    pos: Vec2,
    layout: &SimWorldTransformParams,
) -> Entity {
    spawn_flora_from_preset(commands, id_gen, pos, layout, flora_ea2::OAK)
}

/// Musgo: muy flexible, crece rápido, frágil, sin raíces.
pub fn spawn_moss(
    commands: &mut Commands,
    id_gen: &mut IdGenerator,
    pos: Vec2,
    layout: &SimWorldTransformParams,
) -> Entity {
    spawn_flora_from_preset(commands, id_gen, pos, layout, flora_ea2::MOSS)
}

#[cfg(test)]
mod flora_spawn_tests {
    use super::{spawn_moss, spawn_oak, spawn_rosa};
    use crate::blueprint::{IdGenerator, WorldEntityId};
    use crate::entities::constants::flora_ea2;
    use crate::layers::{
        BaseEnergy, CapabilitySet, FlowVector, GrowthBudget, InferenceProfile, MatterCoherence,
        NutrientProfile, OscillatorySignature, SpatialVolume,
    };
    use crate::runtime_platform::compat_2d3d::SimWorldTransformParams;
    use crate::runtime_platform::hud::MinimapIcon;
    use bevy::prelude::*;

    fn test_app() -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app
    }

    /// Nueve capas metabólicas del contrato EA2 + paridad con `spawn_botanical_seed` (ID fuerte + minimapa).
    fn assert_flora_spawn_stack(world: &World, entity: Entity) {
        let e = world.entity(entity);
        assert!(e.contains::<BaseEnergy>());
        assert!(e.contains::<SpatialVolume>());
        assert!(e.contains::<OscillatorySignature>());
        assert!(e.contains::<FlowVector>());
        assert!(e.contains::<MatterCoherence>());
        assert!(e.contains::<NutrientProfile>());
        assert!(e.contains::<GrowthBudget>());
        assert!(e.contains::<InferenceProfile>());
        assert!(e.contains::<CapabilitySet>());
        assert!(e.contains::<WorldEntityId>());
        assert!(e.contains::<MinimapIcon>());
    }

    #[test]
    fn spawn_all_flora_variants_have_metabolic_stack() {
        let mut app = test_app();
        let layout = SimWorldTransformParams::default();
        let mut id_gen = IdGenerator::default();
        let mut commands = app.world_mut().commands();
        let rosa = spawn_rosa(&mut commands, &mut id_gen, Vec2::ZERO, &layout);
        let oak = spawn_oak(&mut commands, &mut id_gen, Vec2::new(5.0, 0.0), &layout);
        let moss = spawn_moss(&mut commands, &mut id_gen, Vec2::new(10.0, 0.0), &layout);
        drop(commands);
        app.update();
        let world = app.world();
        assert_flora_spawn_stack(world, rosa);
        assert_flora_spawn_stack(world, oak);
        assert_flora_spawn_stack(world, moss);
    }

    #[test]
    fn spawn_rosa_capability_can_grow_and_branch() {
        let mut app = test_app();
        let layout = SimWorldTransformParams::default();
        let mut id_gen = IdGenerator::default();
        let mut commands = app.world_mut().commands();
        let entity = spawn_rosa(&mut commands, &mut id_gen, Vec2::ZERO, &layout);
        drop(commands);
        app.update();
        let cap = app.world().entity(entity).get::<CapabilitySet>().unwrap();
        assert_eq!(cap.flags, flora_ea2::ROSA.capability_flags);
        assert!(cap.can_grow());
        assert!(cap.flags & CapabilitySet::MOVE == 0, "Rosa no se mueve");
    }

    #[test]
    fn spawn_moss_has_no_root_capability() {
        let mut app = test_app();
        let layout = SimWorldTransformParams::default();
        let mut id_gen = IdGenerator::default();
        let mut commands = app.world_mut().commands();
        let entity = spawn_moss(&mut commands, &mut id_gen, Vec2::ZERO, &layout);
        drop(commands);
        app.update();
        let cap = app.world().entity(entity).get::<CapabilitySet>().unwrap();
        assert_eq!(cap.flags, flora_ea2::MOSS.capability_flags);
        assert!(cap.flags & CapabilitySet::ROOT == 0, "Musgo sin raíces");
    }

    #[test]
    fn spawn_oak_has_higher_bond_energy_than_rosa_and_moss() {
        let mut app = test_app();
        let layout = SimWorldTransformParams::default();
        let mut id_gen = IdGenerator::default();
        let mut commands = app.world_mut().commands();
        let rosa = spawn_rosa(&mut commands, &mut id_gen, Vec2::ZERO, &layout);
        let oak = spawn_oak(&mut commands, &mut id_gen, Vec2::new(5.0, 0.0), &layout);
        let moss = spawn_moss(&mut commands, &mut id_gen, Vec2::new(10.0, 0.0), &layout);
        drop(commands);
        app.update();
        let w = app.world();
        let rosa_be = w.entity(rosa).get::<MatterCoherence>().unwrap().bond_energy_eb;
        let oak_be = w.entity(oak).get::<MatterCoherence>().unwrap().bond_energy_eb;
        let moss_be = w.entity(moss).get::<MatterCoherence>().unwrap().bond_energy_eb;
        assert_eq!(rosa_be, flora_ea2::ROSA.bond_energy_eb);
        assert_eq!(oak_be, flora_ea2::OAK.bond_energy_eb);
        assert_eq!(moss_be, flora_ea2::MOSS.bond_energy_eb);
        assert!(oak_be > rosa_be && oak_be > moss_be && rosa_be > moss_be);
    }

    #[test]
    fn spawn_moss_is_fragile_and_oak_is_resilient() {
        let mut app = test_app();
        let layout = SimWorldTransformParams::default();
        let mut id_gen = IdGenerator::default();
        let mut commands = app.world_mut().commands();
        let oak_e = spawn_oak(&mut commands, &mut id_gen, Vec2::ZERO, &layout);
        let moss_e = spawn_moss(&mut commands, &mut id_gen, Vec2::new(3.0, 0.0), &layout);
        drop(commands);
        app.update();
        let w = app.world();
        let oak_r = w.entity(oak_e).get::<InferenceProfile>().unwrap().resilience;
        let moss_r = w.entity(moss_e).get::<InferenceProfile>().unwrap().resilience;
        assert_eq!(moss_r, flora_ea2::MOSS.resilience);
        assert_eq!(oak_r, flora_ea2::OAK.resilience);
        assert!(oak_r > moss_r);
    }

    #[test]
    fn all_flora_spawns_return_valid_entity() {
        let mut app = test_app();
        let layout = SimWorldTransformParams::default();
        let mut id_gen = IdGenerator::default();
        let mut commands = app.world_mut().commands();
        let r = spawn_rosa(&mut commands, &mut id_gen, Vec2::ZERO, &layout);
        let o = spawn_oak(&mut commands, &mut id_gen, Vec2::new(3.0, 0.0), &layout);
        let m = spawn_moss(&mut commands, &mut id_gen, Vec2::new(6.0, 0.0), &layout);
        drop(commands);
        app.update();
        assert!(app.world().get_entity(r).is_ok());
        assert!(app.world().get_entity(o).is_ok());
        assert!(app.world().get_entity(m).is_ok());
    }
}

#[cfg(test)]
mod morphogenesis_spawn_tests {
    use super::*;
    use crate::layers::{
        AmbientPressure, BaseEnergy, FlowVector, IrradianceReceiver, MetabolicGraph,
        MorphogenesisShapeParams, SpatialVolume,
    };

    fn test_app() -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app
    }

    #[test]
    fn spawn_aquatic_organism_has_metabolic_graph() {
        let mut app = test_app();
        let mut commands = app.world_mut().commands();
        let entity = spawn_aquatic_organism(&mut commands, Vec2::ZERO);
        drop(commands);
        app.update();
        assert!(app.world().entity(entity).contains::<MetabolicGraph>());
    }

    #[test]
    fn spawn_aquatic_organism_graph_has_five_nodes() {
        let mut app = test_app();
        let mut commands = app.world_mut().commands();
        let entity = spawn_aquatic_organism(&mut commands, Vec2::ZERO);
        drop(commands);
        app.update();
        let graph = app.world().entity(entity).get::<MetabolicGraph>().unwrap();
        assert!(graph.node_count() >= 5, "aquatic graph has {} nodes, expected >= 5", graph.node_count());
    }

    #[test]
    fn spawn_desert_plant_graph_has_five_nodes() {
        let mut app = test_app();
        let mut commands = app.world_mut().commands();
        let entity = spawn_desert_plant(&mut commands, Vec2::ZERO);
        drop(commands);
        app.update();
        let graph = app.world().entity(entity).get::<MetabolicGraph>().unwrap();
        assert!(graph.node_count() >= 5, "desert plant graph has {} nodes, expected >= 5", graph.node_count());
    }

    #[test]
    fn spawn_forest_plant_graph_has_six_nodes() {
        let mut app = test_app();
        let mut commands = app.world_mut().commands();
        let entity = spawn_forest_plant(&mut commands, Vec2::ZERO);
        drop(commands);
        app.update();
        let graph = app.world().entity(entity).get::<MetabolicGraph>().unwrap();
        assert!(graph.node_count() >= 6, "forest plant graph has {} nodes, expected >= 6", graph.node_count());
    }

    #[test]
    fn spawn_desert_creature_has_metabolic_graph() {
        let mut app = test_app();
        let mut commands = app.world_mut().commands();
        let entity = spawn_desert_creature(&mut commands, Vec2::ZERO);
        drop(commands);
        app.update();
        assert!(app.world().entity(entity).contains::<MetabolicGraph>());
    }

    #[test]
    fn all_morphogenesis_archetypes_have_full_layer_stack() {
        let mut app = test_app();
        let mut commands = app.world_mut().commands();
        let entities = [
            spawn_aquatic_organism(&mut commands, Vec2::ZERO),
            spawn_desert_plant(&mut commands, Vec2::new(10.0, 0.0)),
            spawn_desert_creature(&mut commands, Vec2::new(20.0, 0.0)),
            spawn_forest_plant(&mut commands, Vec2::new(30.0, 0.0)),
        ];
        drop(commands);
        app.update();
        for entity in entities {
            let e = app.world().entity(entity);
            assert!(e.contains::<BaseEnergy>(), "missing BaseEnergy");
            assert!(e.contains::<SpatialVolume>(), "missing SpatialVolume");
            assert!(e.contains::<FlowVector>(), "missing FlowVector");
            assert!(e.contains::<AmbientPressure>(), "missing AmbientPressure");
            assert!(e.contains::<MetabolicGraph>(), "missing MetabolicGraph");
            assert!(e.contains::<MorphogenesisShapeParams>(), "missing ShapeParams");
            assert!(e.contains::<IrradianceReceiver>(), "missing IrradianceReceiver");
        }
    }

    #[test]
    fn no_morphogenesis_archetype_panics() {
        let mut app = test_app();
        let mut commands = app.world_mut().commands();
        let _ = spawn_aquatic_organism(&mut commands, Vec2::ZERO);
        let _ = spawn_desert_plant(&mut commands, Vec2::new(10.0, 0.0));
        let _ = spawn_desert_creature(&mut commands, Vec2::new(20.0, 0.0));
        let _ = spawn_forest_plant(&mut commands, Vec2::new(30.0, 0.0));
        drop(commands);
        app.update(); // no panics
    }
}

#[cfg(test)]
mod morphogenesis_phenotype_tests {
    use super::*;
    use crate::layers::{InferredAlbedo, MorphogenesisShapeParams, MorphogenesisSurface};
    use crate::simulation::metabolic::morphogenesis::{
        albedo_inference_system, entropy_constraint_system, entropy_ledger_system,
        metabolic_graph_step_system, shape_optimization_system, surface_rugosity_system,
    };

    fn phenotype_app() -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_systems(
            Update,
            (
                metabolic_graph_step_system,
                entropy_constraint_system,
                entropy_ledger_system,
                bevy::ecs::schedule::apply_deferred,
                shape_optimization_system,
                surface_rugosity_system,
                albedo_inference_system,
            )
                .chain(),
        );
        app
    }

    #[test]
    fn aquatic_organism_converges_to_high_fineness() {
        let mut app = phenotype_app();
        let mut commands = app.world_mut().commands();
        let entity = spawn_aquatic_organism(&mut commands, Vec2::ZERO);
        drop(commands);
        for _ in 0..10 {
            app.update();
        }
        let shape = app
            .world()
            .entity(entity)
            .get::<MorphogenesisShapeParams>()
            .expect("MorphogenesisShapeParams missing after 10 ticks");
        assert!(
            shape.fineness_ratio() > 3.0,
            "aquatic fineness {} should be > 3.0",
            shape.fineness_ratio()
        );
    }

    #[test]
    fn aquatic_organism_has_dark_albedo() {
        let mut app = phenotype_app();
        let mut commands = app.world_mut().commands();
        let entity = spawn_aquatic_organism(&mut commands, Vec2::ZERO);
        drop(commands);
        for _ in 0..10 {
            app.update();
        }
        let albedo = app
            .world()
            .entity(entity)
            .get::<InferredAlbedo>()
            .expect("InferredAlbedo missing after 10 ticks");
        assert!(
            albedo.albedo() < 0.3,
            "aquatic albedo {} should be < 0.3",
            albedo.albedo()
        );
    }

    #[test]
    fn aquatic_organism_has_smooth_surface() {
        let mut app = phenotype_app();
        let mut commands = app.world_mut().commands();
        let entity = spawn_aquatic_organism(&mut commands, Vec2::ZERO);
        drop(commands);
        for _ in 0..10 {
            app.update();
        }
        let surface = app
            .world()
            .entity(entity)
            .get::<MorphogenesisSurface>()
            .expect("MorphogenesisSurface missing after 10 ticks");
        assert!(
            surface.rugosity() < 1.3,
            "aquatic rugosity {} should be < 1.3",
            surface.rugosity()
        );
    }

    #[test]
    fn desert_plant_has_high_albedo() {
        let mut app = phenotype_app();
        let mut commands = app.world_mut().commands();
        let entity = spawn_desert_plant(&mut commands, Vec2::ZERO);
        drop(commands);
        for _ in 0..10 {
            app.update();
        }
        let albedo = app
            .world()
            .entity(entity)
            .get::<InferredAlbedo>()
            .expect("InferredAlbedo missing after 10 ticks");
        assert!(
            albedo.albedo() > 0.7,
            "desert plant albedo {} should be > 0.7",
            albedo.albedo()
        );
    }

    #[test]
    fn desert_plant_has_rough_surface() {
        let mut app = phenotype_app();
        let mut commands = app.world_mut().commands();
        let entity = spawn_desert_plant(&mut commands, Vec2::ZERO);
        drop(commands);
        for _ in 0..10 {
            app.update();
        }
        let surface = app
            .world()
            .entity(entity)
            .get::<MorphogenesisSurface>()
            .expect("MorphogenesisSurface missing after 10 ticks");
        assert!(
            surface.rugosity() > 2.0,
            "desert plant rugosity {} should be > 2.0",
            surface.rugosity()
        );
    }

    #[test]
    fn forest_plant_has_medium_albedo() {
        let mut app = phenotype_app();
        let mut commands = app.world_mut().commands();
        let entity = spawn_forest_plant(&mut commands, Vec2::ZERO);
        drop(commands);
        for _ in 0..10 {
            app.update();
        }
        let albedo = app
            .world()
            .entity(entity)
            .get::<InferredAlbedo>()
            .expect("InferredAlbedo missing after 10 ticks");
        assert!(
            albedo.albedo() >= 0.25 && albedo.albedo() <= 0.55,
            "forest plant albedo {} should be in [0.25, 0.55]",
            albedo.albedo()
        );
    }

    #[test]
    fn forest_plant_has_moderate_rugosity() {
        let mut app = phenotype_app();
        let mut commands = app.world_mut().commands();
        let entity = spawn_forest_plant(&mut commands, Vec2::ZERO);
        drop(commands);
        for _ in 0..10 {
            app.update();
        }
        let surface = app
            .world()
            .entity(entity)
            .get::<MorphogenesisSurface>()
            .expect("MorphogenesisSurface missing after 10 ticks");
        assert!(
            surface.rugosity() >= 1.0 && surface.rugosity() <= 2.0,
            "forest plant rugosity {} should be in [1.0, 2.0]",
            surface.rugosity()
        );
    }

    #[test]
    fn desert_creature_has_bright_albedo() {
        let mut app = phenotype_app();
        let mut commands = app.world_mut().commands();
        let entity = spawn_desert_creature(&mut commands, Vec2::ZERO);
        drop(commands);
        for _ in 0..10 {
            app.update();
        }
        let albedo = app
            .world()
            .entity(entity)
            .get::<InferredAlbedo>()
            .expect("InferredAlbedo missing after 10 ticks");
        assert!(
            albedo.albedo() > 0.6,
            "desert creature albedo {} should be > 0.6",
            albedo.albedo()
        );
    }

    #[test]
    fn desert_creature_has_rough_surface() {
        let mut app = phenotype_app();
        let mut commands = app.world_mut().commands();
        let entity = spawn_desert_creature(&mut commands, Vec2::ZERO);
        drop(commands);
        for _ in 0..10 {
            app.update();
        }
        let surface = app
            .world()
            .entity(entity)
            .get::<MorphogenesisSurface>()
            .expect("MorphogenesisSurface missing after 10 ticks");
        assert!(
            surface.rugosity() > 2.0,
            "desert creature rugosity {} should be > 2.0",
            surface.rugosity()
        );
    }

    #[test]
    fn legacy_entity_without_metabolic_graph_unaffected_by_pipeline() {
        let mut app = phenotype_app();
        let mut commands = app.world_mut().commands();
        // MG entity — will get derived components
        let _mg = spawn_aquatic_organism(&mut commands, Vec2::ZERO);
        // Legacy entity — NO MetabolicGraph, should be untouched
        let legacy = commands
            .spawn((
                Transform::default(),
                Visibility::default(),
                crate::layers::BaseEnergy::new(100.0),
                crate::layers::SpatialVolume::new(1.0),
                crate::layers::FlowVector::new(Vec2::ZERO, 0.1),
                crate::layers::AmbientPressure::new(0.0, 1.0),
            ))
            .id();
        drop(commands);
        for _ in 0..10 {
            app.update();
        }
        let e = app.world().entity(legacy);
        assert!(!e.contains::<crate::layers::EntropyLedger>(), "legacy should not gain EntropyLedger");
        assert!(!e.contains::<InferredAlbedo>(), "legacy should not gain InferredAlbedo");
        assert!(!e.contains::<MorphogenesisSurface>(), "legacy should not gain MorphogenesisSurface");
        assert!(!e.contains::<MorphogenesisShapeParams>(), "legacy should not gain ShapeParams");
    }
}
