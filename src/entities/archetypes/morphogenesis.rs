use bevy::prelude::*;

use crate::entities::builder::EntityBuilder;
use crate::entities::constants::{MorphogenesisSpawnPreset, morphogenesis_mg8};
use crate::layers::{LifecycleStage, OrganManifest, OrganSpec};

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
        assert!(
            graph.node_count() >= 5,
            "aquatic graph has {} nodes, expected >= 5",
            graph.node_count()
        );
    }

    #[test]
    fn spawn_desert_plant_graph_has_five_nodes() {
        let mut app = test_app();
        let mut commands = app.world_mut().commands();
        let entity = spawn_desert_plant(&mut commands, Vec2::ZERO);
        drop(commands);
        app.update();
        let graph = app.world().entity(entity).get::<MetabolicGraph>().unwrap();
        assert!(
            graph.node_count() >= 5,
            "desert plant graph has {} nodes, expected >= 5",
            graph.node_count()
        );
    }

    #[test]
    fn spawn_forest_plant_graph_has_six_nodes() {
        let mut app = test_app();
        let mut commands = app.world_mut().commands();
        let entity = spawn_forest_plant(&mut commands, Vec2::ZERO);
        drop(commands);
        app.update();
        let graph = app.world().entity(entity).get::<MetabolicGraph>().unwrap();
        assert!(
            graph.node_count() >= 6,
            "forest plant graph has {} nodes, expected >= 6",
            graph.node_count()
        );
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
            assert!(
                e.contains::<MorphogenesisShapeParams>(),
                "missing ShapeParams"
            );
            assert!(
                e.contains::<IrradianceReceiver>(),
                "missing IrradianceReceiver"
            );
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
        assert!(
            !e.contains::<crate::layers::EntropyLedger>(),
            "legacy should not gain EntropyLedger"
        );
        assert!(
            !e.contains::<InferredAlbedo>(),
            "legacy should not gain InferredAlbedo"
        );
        assert!(
            !e.contains::<MorphogenesisSurface>(),
            "legacy should not gain MorphogenesisSurface"
        );
        assert!(
            !e.contains::<MorphogenesisShapeParams>(),
            "legacy should not gain ShapeParams"
        );
    }
}
