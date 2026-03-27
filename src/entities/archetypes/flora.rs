use bevy::prelude::*;

use crate::blueprint::{ElementId, IdGenerator};
use crate::entities::builder::EntityBuilder;
use crate::entities::constants::{
    flora_ea2, FloraSpawnPreset, FLORA_ELEMENT_SYMBOL, FLORA_GROWTH_LIMITER,
    FLORA_MINIMAP_ICON_RADIUS,
};
use crate::layers::{CapabilitySet, InferenceProfile, MatterState};
use crate::runtime_platform::compat_2d3d::SimWorldTransformParams;
use crate::runtime_platform::hud::MinimapIcon;

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
