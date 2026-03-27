//! Catálogo de entidades por complejidad ascendente: célula → virus → planta → animal.
//! Cada spawn function demuestra una composición de capas distinta y observable.

use bevy::prelude::*;

use crate::blueprint::{ElementId, IdGenerator};
use crate::entities::builder::EntityBuilder;
use crate::layers::{
    BehaviorCooldown, BehaviorIntent, BehavioralAgent, CacheScope, CapabilitySet, Faction,
    GrowthBudget, HasInferredShape, Homeostasis, InferenceProfile, MatterState,
    MorphogenesisShapeParams, NutrientProfile, PerformanceCachePolicy, RelationalTag,
    TrophicClass, TrophicConsumer, TrophicState,
};
use crate::layers::organ::LifecycleStageCache;
use crate::layers::{LifecycleStage, OrganManifest, OrganRole, OrganSpec};
use crate::runtime_platform::compat_2d3d::SimWorldTransformParams;

// ── Element symbols ──────────────────────────────────────────────────────────
const AQUA_ELEMENT:   &str = "Aqua";
const IGNIS_ELEMENT:  &str = "Ignis";
const TERRA_ELEMENT:  &str = "Terra";

// ── Célula ───────────────────────────────────────────────────────────────────
const CELULA_QE:            f32 = 150.0;
const CELULA_RADIUS:        f32 = 0.12;
const CELULA_DISSIPATION:   f32 = 0.08;
const CELULA_BOND:          f32 = 600.0;
const CELULA_CONDUCTIVITY:  f32 = 2.5;
const CELULA_BUF_MAX:       f32 = 180.0;
const CELULA_IN_VALVE:      f32 = 0.8;
const CELULA_OUT_VALVE:     f32 = 0.4;
const CELULA_BUF_INIT:      f32 = 60.0;
const CELULA_NUTRIENT_C:    f32 = 65.0;
const CELULA_NUTRIENT_N:    f32 = 55.0;
const CELULA_NUTRIENT_P:    f32 = 25.0;
const CELULA_NUTRIENT_W:    f32 = 95.0;
const CELULA_BIOMASS:       f32 = 12.0;
const CELULA_LIMITER:       u8  = 1;
const CELULA_EFFICIENCY:    f32 = 0.75;
const CELULA_ADAPT_RATE:    f32 = 5.0;
const CELULA_QE_COST_HZ:    f32 = 0.2;
const CELULA_STAB_BAND:     f32 = 8.0;
const CELULA_INTAKE_RATE:   f32 = 5.0;
const CELULA_SATIATION:     f32 = 0.5;  // detritivore starts mid-hungry
const CELULA_CAPS: u8 = CapabilitySet::GROW | CapabilitySet::REPRODUCE;

// ── Virus ────────────────────────────────────────────────────────────────────
const VIRUS_QE:             f32 = 25.0;
const VIRUS_RADIUS:         f32 = 0.04;
const VIRUS_DISSIPATION:    f32 = 1.5;
const VIRUS_BOND:           f32 = 3000.0;
const VIRUS_CONDUCTIVITY:   f32 = 0.1;
const VIRUS_INJECTOR_QE:    f32 = 30.0;
const VIRUS_INJECTOR_FREQ:  f32 = 450.0;
const VIRUS_INJECTOR_R:     f32 = 0.2;
const VIRUS_INTAKE_RATE:    f32 = 8.0;
const VIRUS_SATIATION:      f32 = 0.2;  // carnivore/parasite starts very hungry
const VIRUS_CAPS: u8 = CapabilitySet::REPRODUCE;

// ── Planta ───────────────────────────────────────────────────────────────────
const PLANTA_QE:            f32 = 200.0;
const PLANTA_RADIUS:        f32 = 0.25;
const PLANTA_DISSIPATION:   f32 = 0.05;
const PLANTA_BOND:          f32 = 2000.0;
const PLANTA_CONDUCTIVITY:  f32 = 1.2;
const PLANTA_BUF_MAX:       f32 = 350.0;
const PLANTA_IN_VALVE:      f32 = 0.6;
const PLANTA_OUT_VALVE:     f32 = 0.3;
const PLANTA_BUF_INIT:      f32 = 80.0;
const PLANTA_NUTRIENT_C:    f32 = 80.0;
const PLANTA_NUTRIENT_N:    f32 = 60.0;
const PLANTA_NUTRIENT_P:    f32 = 40.0;
const PLANTA_NUTRIENT_W:    f32 = 70.0;
const PLANTA_BIOMASS:       f32 = 30.0;
const PLANTA_LIMITER:       u8  = 0;
const PLANTA_EFFICIENCY:    f32 = 0.85;
const PLANTA_CAPS: u8 =
    CapabilitySet::GROW | CapabilitySet::BRANCH | CapabilitySet::ROOT | CapabilitySet::PHOTOSYNTH;
const PLANTA_DELTA_QE:  f32 = 1.5;   // ambient pressure from terrain/soil
const PLANTA_VISCOSITY: f32 = 1.2;   // rooted — high ground resistance
const PLANTA_T_CORE:    f32 = 573.0; // internal metabolic temperature (K)
const PLANTA_T_ENV:     f32 = 284.0; // ambient environment temperature (K)

// ── Animal ───────────────────────────────────────────────────────────────────
const ANIMAL_QE:            f32 = 450.0;
const ANIMAL_RADIUS:        f32 = 0.55;
const ANIMAL_DISSIPATION:   f32 = 0.12;
const ANIMAL_BOND:          f32 = 1200.0;
const ANIMAL_CONDUCTIVITY:  f32 = 1.8;
const ANIMAL_BUF_MAX:       f32 = 500.0;
const ANIMAL_IN_VALVE:      f32 = 0.7;
const ANIMAL_OUT_VALVE:     f32 = 0.6;
const ANIMAL_BUF_INIT:      f32 = 150.0;
const ANIMAL_ADAPT_RATE:    f32 = 3.0;
const ANIMAL_QE_COST_HZ:    f32 = 0.1;
const ANIMAL_STAB_BAND:     f32 = 5.0;
const ANIMAL_INTAKE_RATE:   f32 = 15.0;
const ANIMAL_SATIATION:     f32 = 0.3;
const ANIMAL_NUTRIENT_C:    f32 = 50.0;
const ANIMAL_NUTRIENT_N:    f32 = 70.0;
const ANIMAL_NUTRIENT_P:    f32 = 30.0;
const ANIMAL_NUTRIENT_W:    f32 = 80.0;
const ANIMAL_BIOMASS:       f32 = 45.0;
const ANIMAL_LIMITER:       u8  = 0;
const ANIMAL_EFFICIENCY:    f32 = 0.70;
const ANIMAL_CAPS: u8 =
    CapabilitySet::GROW | CapabilitySet::MOVE | CapabilitySet::SENSE | CapabilitySet::REPRODUCE;

// ── Public spawn functions ────────────────────────────────────────────────────

/// Célula: L0+L1+L2+L3+L4+L5+L12 + metabolismo Aqua.
/// Detritivora, puede crecer y reproducirse. Observable: ciclo metabólico + homeostasis.
pub fn spawn_celula(
    commands: &mut Commands,
    id_gen: &mut IdGenerator,
    pos: Vec2,
    layout: &SimWorldTransformParams,
) -> Entity {
    let wid = id_gen.next_world();
    let entity = EntityBuilder::new()
        .named("demo_celula")
        .at(pos)
        .energy(CELULA_QE)
        .volume(CELULA_RADIUS)
        .wave(ElementId::from_name(AQUA_ELEMENT))
        .flow(Vec2::ZERO, CELULA_DISSIPATION)
        .matter(MatterState::Liquid, CELULA_BOND, CELULA_CONDUCTIVITY)
        .motor(CELULA_BUF_MAX, CELULA_IN_VALVE, CELULA_OUT_VALVE, CELULA_BUF_INIT)
        .homeostasis(Homeostasis::new(CELULA_ADAPT_RATE, CELULA_QE_COST_HZ, CELULA_STAB_BAND, true))
        .nutrient(CELULA_NUTRIENT_C, CELULA_NUTRIENT_N, CELULA_NUTRIENT_P, CELULA_NUTRIENT_W)
        .growth_budget(CELULA_BIOMASS, CELULA_LIMITER, CELULA_EFFICIENCY)
        .sim_world_layout(layout)
        .spawn(commands);

    commands.entity(entity).insert((
        wid,
        TrophicConsumer::new(TrophicClass::Detritivore, CELULA_INTAKE_RATE),
        TrophicState::new(CELULA_SATIATION),
        CapabilitySet::new(CELULA_CAPS),
        HasInferredShape,
        LifecycleStageCache::default(),
        MorphogenesisShapeParams::default(),
        PerformanceCachePolicy { enabled: true, scope: CacheScope::StableWindow, version_tag: 1, dependency_signature: 0 },
    ));
    entity
}

/// Virus: L0+L1+L2+L3+L4+L8 — cápside rígido, sin motor propio.
/// Inyector Ignis: perturba la frecuencia del huésped. Observable: parasitismo energético.
pub fn spawn_virus(
    commands: &mut Commands,
    id_gen: &mut IdGenerator,
    pos: Vec2,
    layout: &SimWorldTransformParams,
) -> Entity {
    let wid = id_gen.next_world();
    let entity = EntityBuilder::new()
        .named("demo_virus")
        .at(pos)
        .energy(VIRUS_QE)
        .volume(VIRUS_RADIUS)
        .wave(ElementId::from_name(IGNIS_ELEMENT))
        .flow(Vec2::ZERO, VIRUS_DISSIPATION)
        .matter(MatterState::Solid, VIRUS_BOND, VIRUS_CONDUCTIVITY)
        .injector(VIRUS_INJECTOR_QE, VIRUS_INJECTOR_FREQ, VIRUS_INJECTOR_R)
        .sim_world_layout(layout)
        .spawn(commands);

    commands.entity(entity).insert((
        wid,
        TrophicConsumer::new(TrophicClass::Carnivore, VIRUS_INTAKE_RATE),
        TrophicState::new(VIRUS_SATIATION),
        CapabilitySet::new(VIRUS_CAPS),
        HasInferredShape,
        LifecycleStageCache::default(),
        MorphogenesisShapeParams::default(),
        PerformanceCachePolicy { enabled: true, scope: CacheScope::StableWindow, version_tag: 1, dependency_signature: 0 },
    ));
    entity
}

/// Planta: L0+L1+L2+L3+L4+L5 + IrradianceReceiver + NutrientProfile + GrowthBudget.
/// Fotosíntesis, raíces, ramificación. Observable: crecimiento morfogenético + fotosíntesis.
pub fn spawn_planta_demo(
    commands: &mut Commands,
    id_gen: &mut IdGenerator,
    pos: Vec2,
    layout: &SimWorldTransformParams,
) -> Entity {
    let wid = id_gen.next_world();
    let mut manifest = OrganManifest::new(LifecycleStage::Growing);
    for role in [OrganRole::Root, OrganRole::Stem, OrganRole::Leaf, OrganRole::Leaf] {
        manifest.push(OrganSpec::new(role, 1, 1.0));
    }
    let entity = EntityBuilder::new()
        .named("demo_planta")
        .at(pos)
        .energy(PLANTA_QE)
        .volume(PLANTA_RADIUS)
        .wave(ElementId::from_name(TERRA_ELEMENT))
        .flow(Vec2::ZERO, PLANTA_DISSIPATION)
        .matter(MatterState::Solid, PLANTA_BOND, PLANTA_CONDUCTIVITY)
        .motor(PLANTA_BUF_MAX, PLANTA_IN_VALVE, PLANTA_OUT_VALVE, PLANTA_BUF_INIT)
        .nutrient(PLANTA_NUTRIENT_C, PLANTA_NUTRIENT_N, PLANTA_NUTRIENT_P, PLANTA_NUTRIENT_W)
        .growth_budget(PLANTA_BIOMASS, PLANTA_LIMITER, PLANTA_EFFICIENCY)
        .ambient(PLANTA_DELTA_QE, PLANTA_VISCOSITY)
        .with_organ_manifest(manifest)
        .with_metabolic_graph_inferred(PLANTA_T_CORE, PLANTA_T_ENV)
        .sim_world_layout(layout)
        .spawn(commands);

    commands.entity(entity).insert((
        wid,
        crate::layers::IrradianceReceiver::new(0.0, 0.75),
        InferenceProfile::new(0.9, 0.0, 0.7, 0.7),
        CapabilitySet::new(PLANTA_CAPS),
        HasInferredShape,
        LifecycleStageCache::default(),
        MorphogenesisShapeParams::default(),
        PerformanceCachePolicy { enabled: true, scope: CacheScope::StableWindow, version_tag: 1, dependency_signature: 0 },
    ));
    entity
}

/// Animal: L0+L1+L2+L3+L4+L5+L7+L9+L12 + BehavioralAgent + TrophicConsumer(Herbivore).
/// Voluntad, identidad, homeostasis, comportamiento autónomo. Observable: comportamiento trófico.
pub fn spawn_animal_demo(
    commands: &mut Commands,
    id_gen: &mut IdGenerator,
    pos: Vec2,
    layout: &SimWorldTransformParams,
) -> Entity {
    let wid = id_gen.next_world();
    let entity = EntityBuilder::new()
        .named("demo_animal")
        .at(pos)
        .energy(ANIMAL_QE)
        .volume(ANIMAL_RADIUS)
        .wave(ElementId::from_name(TERRA_ELEMENT))
        .flow(Vec2::ZERO, ANIMAL_DISSIPATION)
        .matter(MatterState::Solid, ANIMAL_BOND, ANIMAL_CONDUCTIVITY)
        .motor(ANIMAL_BUF_MAX, ANIMAL_IN_VALVE, ANIMAL_OUT_VALVE, ANIMAL_BUF_INIT)
        .will_default()
        .identity(Faction::Neutral, vec![RelationalTag::Jungle], 1.0)
        .homeostasis(Homeostasis::new(ANIMAL_ADAPT_RATE, ANIMAL_QE_COST_HZ, ANIMAL_STAB_BAND, true))
        .ambient(0.0, 1.0) // L6: neutral biome — enables constructal body plan inference
        .sim_world_layout(layout)
        .spawn(commands);

    commands.entity(entity).insert((
        wid,
        BehavioralAgent,
        BehaviorIntent::default(),
        BehaviorCooldown::default(),
        TrophicConsumer::new(TrophicClass::Herbivore, ANIMAL_INTAKE_RATE),
        TrophicState::new(ANIMAL_SATIATION),
        NutrientProfile::new(ANIMAL_NUTRIENT_C, ANIMAL_NUTRIENT_N, ANIMAL_NUTRIENT_P, ANIMAL_NUTRIENT_W),
        GrowthBudget::new(ANIMAL_BIOMASS, ANIMAL_LIMITER, ANIMAL_EFFICIENCY),
        CapabilitySet::new(ANIMAL_CAPS),
        HasInferredShape,
        LifecycleStageCache::default(),
        MorphogenesisShapeParams::default(),
        InferenceProfile::new(0.5, 0.8, 0.2, 0.6), // high mobility → primate-like proportions
        PerformanceCachePolicy { enabled: true, scope: CacheScope::StableWindow, version_tag: 1, dependency_signature: 0 },
    ));
    entity
}

// ── Stellar archetypes ────────────────────────────────────────────────────────

use crate::blueprint::constants::stellar;
use crate::layers::{FieldFalloffMode, TensionField};
use crate::worldgen::{EnergyNucleus, PropagationDecay};

/// Star: massive energy source with 1/r² gravity and radiation.
///
/// L0 (high qe) + L1 (small visual radius) + L2 (Lux frequency) + L4 (Plasma) +
/// L11 (InverseSquare gravity, system-wide radius) + EnergyNucleus (radiation).
pub fn spawn_star(
    commands: &mut Commands,
    id_gen: &mut IdGenerator,
    pos: Vec2,
    layout: &SimWorldTransformParams,
) -> Entity {
    let wid = id_gen.next_world();
    let entity = EntityBuilder::new()
        .named("star")
        .at(pos)
        .energy(stellar::STAR_DEFAULT_QE)
        .volume(stellar::STAR_DEFAULT_RADIUS)
        .wave_from_hz(stellar::STAR_FREQUENCY_HZ)
        .flow(Vec2::ZERO, 0.0) // stars don't move (or very slowly)
        .matter(MatterState::Plasma, 100.0, 10.0)
        .ambient(0.0, 0.001) // vacuum around star
        .sim_world_layout(layout)
        .spawn(commands);

    commands.entity(entity).insert((
        wid,
        TensionField::new(
            stellar::STAR_FIELD_RADIUS,
            stellar::STELLAR_GRAVITY_GAIN,
            stellar::STELLAR_MAGNETIC_GAIN,
            FieldFalloffMode::InverseSquare,
        ),
        EnergyNucleus {
            frequency_hz: stellar::STAR_FREQUENCY_HZ,
            emission_rate_qe_s: stellar::STAR_EMISSION_RATE,
            propagation_radius: stellar::STAR_FIELD_RADIUS,
            decay: PropagationDecay::InverseSquare,
        },
    ));
    entity
}

/// Planet: orbiting body with local gravity well.
///
/// L0 + L1 + L2 (band from distance to star) + L3 (orbital velocity) + L4 (Solid) +
/// L6 (surface conditions) + L11 (local gravity). Receives stellar radiation via field grid.
/// Life may emerge on surface via axiomatic abiogenesis if conditions are right.
pub fn spawn_planet(
    commands: &mut Commands,
    id_gen: &mut IdGenerator,
    pos: Vec2,
    orbital_velocity: Vec2,
    frequency_hz: f32,
    surface_delta_qe: f32,
    layout: &SimWorldTransformParams,
) -> Entity {
    let wid = id_gen.next_world();
    let entity = EntityBuilder::new()
        .named("planet")
        .at(pos)
        .energy(stellar::PLANET_DEFAULT_QE)
        .volume(stellar::PLANET_DEFAULT_RADIUS)
        .wave_from_hz(frequency_hz)
        .flow(orbital_velocity, 0.001) // near-zero dissipation in vacuum (Axiom 4)
        .matter(MatterState::Solid, stellar::PLANET_DEFAULT_BOND, stellar::PLANET_DEFAULT_CONDUCTIVITY)
        .ambient(surface_delta_qe, 1.0) // surface: moderate viscosity, energy injection from star
        .sim_world_layout(layout)
        .spawn(commands);

    commands.entity(entity).insert((
        wid,
        TensionField::new(
            stellar::PLANET_FIELD_RADIUS,
            stellar::STELLAR_GRAVITY_GAIN * 0.01, // planets have much less gravity than stars
            0.0,
            FieldFalloffMode::InverseSquare,
        ),
    ));
    entity
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod catalog_spawn_tests {
    use super::*;
    use crate::blueprint::{IdGenerator, WorldEntityId};
    use crate::layers::{
        AlchemicalEngine, AlchemicalInjector, BaseEnergy, BehavioralAgent, CapabilitySet,
        FlowVector, GrowthBudget, Homeostasis, IrradianceReceiver, MatterCoherence,
        MobaIdentity, NutrientProfile, OscillatorySignature, SpatialVolume, TrophicConsumer,
        TrophicState, WillActuator,
    };
    use crate::runtime_platform::compat_2d3d::SimWorldTransformParams;
    use bevy::prelude::*;

    fn test_app() -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app
    }

    fn layout() -> SimWorldTransformParams {
        SimWorldTransformParams::default()
    }

    // ── Célula ────────────────────────────────────────────────────────────────

    #[test]
    fn spawn_celula_has_core_layers() {
        let mut app = test_app();
        let mut id_gen = IdGenerator::default();
        let mut commands = app.world_mut().commands();
        let e = spawn_celula(&mut commands, &mut id_gen, Vec2::ZERO, &layout());
        drop(commands);
        app.update();
        let w = app.world();
        let ent = w.entity(e);
        assert!(ent.contains::<BaseEnergy>());
        assert!(ent.contains::<SpatialVolume>());
        assert!(ent.contains::<OscillatorySignature>());
        assert!(ent.contains::<FlowVector>());
        assert!(ent.contains::<MatterCoherence>());
        assert!(ent.contains::<AlchemicalEngine>());
        assert!(ent.contains::<Homeostasis>());
        assert!(ent.contains::<NutrientProfile>());
        assert!(ent.contains::<GrowthBudget>());
        assert!(ent.contains::<TrophicConsumer>());
        assert!(ent.contains::<CapabilitySet>());
        assert!(ent.contains::<WorldEntityId>());
    }

    #[test]
    fn spawn_celula_has_no_will_or_injector() {
        let mut app = test_app();
        let mut id_gen = IdGenerator::default();
        let mut commands = app.world_mut().commands();
        let e = spawn_celula(&mut commands, &mut id_gen, Vec2::ZERO, &layout());
        drop(commands);
        app.update();
        let ent = app.world().entity(e);
        assert!(!ent.contains::<WillActuator>(), "célula no tiene voluntad");
        assert!(!ent.contains::<AlchemicalInjector>(), "célula no inyecta");
        assert!(!ent.contains::<MobaIdentity>(), "célula no tiene facción");
    }

    #[test]
    fn spawn_celula_capabilities_are_grow_and_reproduce_only() {
        let mut app = test_app();
        let mut id_gen = IdGenerator::default();
        let mut commands = app.world_mut().commands();
        let e = spawn_celula(&mut commands, &mut id_gen, Vec2::ZERO, &layout());
        drop(commands);
        app.update();
        let cap = app.world().entity(e).get::<CapabilitySet>().unwrap();
        assert!(cap.can_grow());
        assert!(cap.flags & CapabilitySet::MOVE == 0);
        assert!(cap.flags & CapabilitySet::PHOTOSYNTH == 0);
    }

    // ── Virus ─────────────────────────────────────────────────────────────────

    #[test]
    fn spawn_virus_has_injector_no_engine() {
        let mut app = test_app();
        let mut id_gen = IdGenerator::default();
        let mut commands = app.world_mut().commands();
        let e = spawn_virus(&mut commands, &mut id_gen, Vec2::ZERO, &layout());
        drop(commands);
        app.update();
        let ent = app.world().entity(e);
        assert!(ent.contains::<AlchemicalInjector>(), "virus tiene inyector");
        assert!(!ent.contains::<AlchemicalEngine>(), "virus no tiene motor propio");
        assert!(!ent.contains::<Homeostasis>(), "virus no se adapta");
    }

    #[test]
    fn spawn_virus_only_capability_is_reproduce() {
        let mut app = test_app();
        let mut id_gen = IdGenerator::default();
        let mut commands = app.world_mut().commands();
        let e = spawn_virus(&mut commands, &mut id_gen, Vec2::ZERO, &layout());
        drop(commands);
        app.update();
        let cap = app.world().entity(e).get::<CapabilitySet>().unwrap();
        assert_eq!(cap.flags, CapabilitySet::REPRODUCE);
        assert!(cap.flags & CapabilitySet::GROW == 0);
        assert!(cap.flags & CapabilitySet::MOVE == 0);
    }

    // ── Planta ────────────────────────────────────────────────────────────────

    #[test]
    fn spawn_planta_has_irradiance_and_inference() {
        let mut app = test_app();
        let mut id_gen = IdGenerator::default();
        let mut commands = app.world_mut().commands();
        let e = spawn_planta_demo(&mut commands, &mut id_gen, Vec2::ZERO, &layout());
        drop(commands);
        app.update();
        let ent = app.world().entity(e);
        assert!(ent.contains::<IrradianceReceiver>());
        assert!(ent.contains::<InferenceProfile>());
        assert!(ent.contains::<GrowthBudget>());
        assert!(ent.contains::<NutrientProfile>());
    }

    #[test]
    fn spawn_planta_can_photosynth_and_branch() {
        let mut app = test_app();
        let mut id_gen = IdGenerator::default();
        let mut commands = app.world_mut().commands();
        let e = spawn_planta_demo(&mut commands, &mut id_gen, Vec2::ZERO, &layout());
        drop(commands);
        app.update();
        let cap = app.world().entity(e).get::<CapabilitySet>().unwrap();
        assert!(cap.flags & CapabilitySet::PHOTOSYNTH != 0);
        assert!(cap.flags & CapabilitySet::BRANCH != 0);
        assert!(cap.flags & CapabilitySet::ROOT != 0);
        assert!(cap.flags & CapabilitySet::MOVE == 0, "planta no se mueve");
    }

    // ── Animal ────────────────────────────────────────────────────────────────

    #[test]
    fn spawn_animal_has_will_identity_behavior() {
        let mut app = test_app();
        let mut id_gen = IdGenerator::default();
        let mut commands = app.world_mut().commands();
        let e = spawn_animal_demo(&mut commands, &mut id_gen, Vec2::ZERO, &layout());
        drop(commands);
        app.update();
        let ent = app.world().entity(e);
        assert!(ent.contains::<WillActuator>());
        assert!(ent.contains::<MobaIdentity>());
        assert!(ent.contains::<Homeostasis>());
        assert!(ent.contains::<BehavioralAgent>());
        assert!(ent.contains::<TrophicConsumer>());
        assert!(ent.contains::<TrophicState>());
    }

    #[test]
    fn spawn_animal_caps_include_move_and_sense() {
        let mut app = test_app();
        let mut id_gen = IdGenerator::default();
        let mut commands = app.world_mut().commands();
        let e = spawn_animal_demo(&mut commands, &mut id_gen, Vec2::ZERO, &layout());
        drop(commands);
        app.update();
        let cap = app.world().entity(e).get::<CapabilitySet>().unwrap();
        assert!(cap.flags & CapabilitySet::MOVE != 0);
        assert!(cap.flags & CapabilitySet::SENSE != 0);
        assert!(cap.flags & CapabilitySet::PHOTOSYNTH == 0, "animal no fotosintiza");
    }
}
