use std::time::Duration;

use bevy::prelude::*;
use bevy::time::Virtual;

use crate::blueprint::{ElementId, IdGenerator};
use crate::entities::archetypes::{BiomeType, spawn_biome, spawn_particle};
use crate::events::DeathEvent;
use crate::layers::{BaseEnergy, ContactType, ContainedIn, MatterCoherence, MatterState};
use crate::runtime_platform::compat_2d3d::SimWorldTransformParams;
use crate::simulation::containment::{contained_thermal_transfer_system, containment_system};
use crate::simulation::structural_runtime::{
    homeostasis_system, structural_constraint_system, tension_field_system,
};
use crate::world::update_spatial_index_system;

fn setup_regression_app() -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.insert_resource(SimWorldTransformParams::default());
    app.add_event::<DeathEvent>();
    app.add_systems(
        Update,
        (containment_system, contained_thermal_transfer_system).chain(),
    );
    app
}

fn setup_structural_runtime_app() -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.insert_resource(SimWorldTransformParams::default());
    app.add_event::<DeathEvent>();
    app.add_event::<crate::events::StructuralLinkBreakEvent>();
    app.add_event::<crate::events::HomeostasisAdaptEvent>();
    app.init_resource::<crate::world::SpatialIndex>();
    app.add_systems(
        Update,
        (
            update_spatial_index_system,
            containment_system.after(update_spatial_index_system),
            structural_constraint_system.after(containment_system),
            tension_field_system.after(update_spatial_index_system),
            homeostasis_system.after(containment_system),
        ),
    );
    app
}

fn advance_simulation(app: &mut App, dt: f32, steps: usize) {
    for _ in 0..steps {
        app.world_mut()
            .resource_mut::<Time<Virtual>>()
            .advance_by(Duration::from_secs_f32(dt));
        app.update();
    }
}

fn energy_of(app: &App, entity: Entity) -> Option<f32> {
    app.world().get::<BaseEnergy>(entity).map(|e| e.qe())
}

fn loss(start: f32, end: f32) -> f32 {
    (start - end).max(0.0)
}

#[test]
fn regression_combined_thermal_channels_keep_consistency() {
    let mut app = setup_regression_app();
    let element = ElementId::from_name("Ignis");

    let (desert_center, desert_radius) = (Vec2::new(0.0, 0.0), 5.0);
    let (volcano_center, volcano_radius) = (Vec2::new(20.0, 0.0), 5.0);

    let (
        desert_radiated,
        desert_immersed,
        desert_surface,
        volcano_radiated,
        volcano_immersed,
        volcano_surface,
    ) = {
        let mut id_gen = IdGenerator::default();
        let mut commands = app.world_mut().commands();
        let lay = SimWorldTransformParams::default();
        let _desert = spawn_biome(
            &mut commands,
            &mut id_gen,
            desert_center,
            desert_radius,
            BiomeType::Desert,
            &lay,
        );
        let _volcano = spawn_biome(
            &mut commands,
            &mut id_gen,
            volcano_center,
            volcano_radius,
            BiomeType::Volcano,
            &lay,
        );

        // Mismo receptor para comparar solo por canal/host.
        let qe = 200.0;
        let radius = 0.4;
        let eb = 8000.0;
        let cond = 0.7;

        let desert_radiated = spawn_particle(
            &mut commands,
            &mut id_gen,
            Vec2::new(desert_center.x + desert_radius + radius + 0.1, 1.2),
            qe,
            radius,
            element,
            &lay,
        );
        commands
            .entity(desert_radiated)
            .insert(MatterCoherence::new(MatterState::Solid, eb, cond));

        let desert_immersed = spawn_particle(
            &mut commands,
            &mut id_gen,
            Vec2::new(desert_center.x + desert_radius - radius - 1.0, 0.0),
            qe,
            radius,
            element,
            &lay,
        );
        commands
            .entity(desert_immersed)
            .insert(MatterCoherence::new(MatterState::Solid, eb, cond));

        let desert_surface = spawn_particle(
            &mut commands,
            &mut id_gen,
            Vec2::new(desert_center.x + desert_radius + radius - 0.2, -1.2),
            qe,
            radius,
            element,
            &lay,
        );
        commands
            .entity(desert_surface)
            .insert(MatterCoherence::new(MatterState::Solid, eb, cond));

        let volcano_radiated = spawn_particle(
            &mut commands,
            &mut id_gen,
            Vec2::new(volcano_center.x + volcano_radius + radius + 0.1, 1.2),
            qe,
            radius,
            element,
            &lay,
        );
        commands
            .entity(volcano_radiated)
            .insert(MatterCoherence::new(MatterState::Solid, eb, cond));

        let volcano_immersed = spawn_particle(
            &mut commands,
            &mut id_gen,
            Vec2::new(volcano_center.x + volcano_radius - radius - 1.0, 0.0),
            qe,
            radius,
            element,
            &lay,
        );
        commands
            .entity(volcano_immersed)
            .insert(MatterCoherence::new(MatterState::Solid, eb, cond));

        let volcano_surface = spawn_particle(
            &mut commands,
            &mut id_gen,
            Vec2::new(volcano_center.x + volcano_radius + radius - 0.2, -1.2),
            qe,
            radius,
            element,
            &lay,
        );
        commands
            .entity(volcano_surface)
            .insert(MatterCoherence::new(MatterState::Solid, eb, cond));
        (
            desert_radiated,
            desert_immersed,
            desert_surface,
            volcano_radiated,
            volcano_immersed,
            volcano_surface,
        )
    };
    app.update();

    let tracked = [
        desert_radiated,
        desert_immersed,
        desert_surface,
        volcano_radiated,
        volcano_immersed,
        volcano_surface,
    ];
    let initial: Vec<(Entity, f32)> = tracked
        .iter()
        .filter_map(|entity| energy_of(&app, *entity).map(|qe| (*entity, qe)))
        .collect();

    advance_simulation(&mut app, 0.1, 120);

    let mut final_loss = [0.0; 6];
    for (index, (entity, start_qe)) in initial.iter().enumerate() {
        let end_qe = energy_of(&app, *entity).unwrap_or(0.0);
        final_loss[index] = loss(*start_qe, end_qe);
    }

    let desert_radiated_loss = final_loss[0];
    let desert_immersed_loss = final_loss[1];
    let desert_surface_loss = final_loss[2];
    let volcano_radiated_loss = final_loss[3];
    let volcano_immersed_loss = final_loss[4];
    let volcano_surface_loss = final_loss[5];

    // Los regímenes deben mantenerse clasificables por contacto dominante.
    assert_eq!(
        app.world()
            .get::<ContainedIn>(desert_radiated)
            .map(|c| c.contact),
        Some(ContactType::Radiated)
    );
    assert_eq!(
        app.world()
            .get::<ContainedIn>(desert_immersed)
            .map(|c| c.contact),
        Some(ContactType::Immersed)
    );
    assert_eq!(
        app.world()
            .get::<ContainedIn>(desert_surface)
            .map(|c| c.contact),
        Some(ContactType::Surface)
    );
    assert_eq!(
        app.world()
            .get::<ContainedIn>(volcano_radiated)
            .map(|c| c.contact),
        Some(ContactType::Radiated)
    );
    assert_eq!(
        app.world()
            .get::<ContainedIn>(volcano_immersed)
            .map(|c| c.contact),
        Some(ContactType::Immersed)
    );
    assert_eq!(
        app.world()
            .get::<ContainedIn>(volcano_surface)
            .map(|c| c.contact),
        Some(ContactType::Surface)
    );

    // Mismo canal entre biomas: volcán drena más que desierto.
    assert!(volcano_radiated_loss > desert_radiated_loss);
    assert!(volcano_immersed_loss > desert_immersed_loss);
    assert!(volcano_surface_loss > desert_surface_loss);

    // Sin ganancias energéticas inesperadas.
    assert!(desert_radiated_loss >= 0.0);
    assert!(desert_immersed_loss >= 0.0);
    assert!(desert_surface_loss >= 0.0);
    assert!(volcano_radiated_loss >= 0.0);
    assert!(volcano_immersed_loss >= 0.0);
    assert!(volcano_surface_loss >= 0.0);
    assert!(
        desert_radiated_loss + desert_immersed_loss + desert_surface_loss > 0.0,
        "At least one desert regime must transfer energy"
    );
    assert!(
        volcano_radiated_loss + volcano_immersed_loss + volcano_surface_loss > 0.0,
        "At least one volcano regime must transfer energy"
    );
}

#[test]
fn regression_position_updates_containment_predictably() {
    let mut app = setup_regression_app();
    let element = ElementId::from_name("Ignis");

    let probe = {
        let mut id_gen = IdGenerator::default();
        let mut commands = app.world_mut().commands();
        let host_center = Vec2::new(0.0, 0.0);
        let host_radius = 5.0;
        let lay = SimWorldTransformParams::default();
        let _host = spawn_biome(
            &mut commands,
            &mut id_gen,
            host_center,
            host_radius,
            BiomeType::Volcano,
            &lay,
        );
        spawn_particle(
            &mut commands,
            &mut id_gen,
            Vec2::new(8.0, 0.0),
            200.0,
            0.4,
            element,
            &lay,
        )
    };
    app.update();

    // Radiated
    advance_simulation(&mut app, 0.05, 2);
    let c0 = app.world().get::<ContainedIn>(probe).map(|c| c.contact);
    assert_eq!(c0, Some(ContactType::Radiated));

    // Surface
    if let Some(mut t) = app.world_mut().get_mut::<Transform>(probe) {
        t.translation.x = 5.2;
    }
    advance_simulation(&mut app, 0.05, 2);
    let c1 = app.world().get::<ContainedIn>(probe).map(|c| c.contact);
    assert_eq!(c1, Some(ContactType::Surface));

    // Immersed
    if let Some(mut t) = app.world_mut().get_mut::<Transform>(probe) {
        t.translation.x = 3.0;
    }
    advance_simulation(&mut app, 0.05, 2);
    let c2 = app.world().get::<ContainedIn>(probe).map(|c| c.contact);
    assert_eq!(c2, Some(ContactType::Immersed));

    // Outside range -> no containment.
    if let Some(mut t) = app.world_mut().get_mut::<Transform>(probe) {
        t.translation.x = 11.0;
    }
    advance_simulation(&mut app, 0.05, 2);
    let c3 = app.world().get::<ContainedIn>(probe).map(|c| c.contact);
    assert_eq!(c3, None);
}

#[test]
fn regression_many_entities_remain_numerically_stable() {
    let mut app = setup_regression_app();
    let element = ElementId::from_name("Terra");

    {
        let mut id_gen = IdGenerator::default();
        let mut commands = app.world_mut().commands();
        let lay = SimWorldTransformParams::default();
        let _host_a = spawn_biome(
            &mut commands,
            &mut id_gen,
            Vec2::new(0.0, 0.0),
            8.0,
            BiomeType::Volcano,
            &lay,
        );
        let _host_b = spawn_biome(
            &mut commands,
            &mut id_gen,
            Vec2::new(20.0, 0.0),
            6.0,
            BiomeType::Desert,
            &lay,
        );
        let _host_c = spawn_biome(
            &mut commands,
            &mut id_gen,
            Vec2::new(-20.0, 0.0),
            7.0,
            BiomeType::Swamp,
            &lay,
        );

        // Carga alta: muchos contenidos con variación espacial.
        for i in 0..320 {
            let x = -28.0 + (i % 40) as f32 * 1.4;
            let y = -10.0 + (i / 40) as f32 * 2.0;
            let qe = 120.0 + (i % 7) as f32 * 5.0;
            let radius = 0.25 + (i % 5) as f32 * 0.05;
            let entity = spawn_particle(
                &mut commands,
                &mut id_gen,
                Vec2::new(x, y),
                qe,
                radius,
                element,
                &lay,
            );
            commands
                .entity(entity)
                .insert(MatterCoherence::new(MatterState::Solid, 6000.0, 0.4));
        }
    }
    app.update();
    advance_simulation(&mut app, 0.05, 240);

    let mut alive = 0usize;
    {
        let world = app.world_mut();
        let mut query = world.query::<&BaseEnergy>();
        for energy in query.iter(world) {
            assert!(energy.qe().is_finite(), "qe cannot be NaN/Inf");
            assert!(energy.qe() >= 0.0, "qe cannot be negative");
            if energy.qe() > 0.0 {
                alive += 1;
            }
        }
    }

    assert!(
        alive > 40,
        "Simulation should not collapse catastrophically under load"
    );
}

#[test]
fn regression_tension_field_deflects_projectile() {
    use crate::layers::{
        FieldFalloffMode, FlowVector, OscillatorySignature, SpatialVolume, TensionField,
    };

    let mut app = setup_structural_runtime_app();
    let (source, target) = {
        let mut commands = app.world_mut().commands();
        let source = commands
            .spawn((
                Transform::from_translation(Vec3::ZERO),
                Visibility::default(),
                BaseEnergy::new(1000.0),
                SpatialVolume::new(1.0),
                OscillatorySignature::new(450.0, 0.0),
                TensionField::new(10.0, 0.02, 0.01, FieldFalloffMode::InverseSquare),
            ))
            .id();
        let target = commands
            .spawn((
                Transform::from_translation(Vec3::new(4.0, 1.0, 0.0)),
                Visibility::default(),
                BaseEnergy::new(100.0),
                SpatialVolume::new(0.4),
                OscillatorySignature::new(450.0, 0.0),
                FlowVector::new(Vec2::new(0.0, 5.0), 0.0),
            ))
            .id();
        (source, target)
    };
    app.update();
    advance_simulation(&mut app, 0.05, 20);

    let _ = source;
    let flow = app.world().get::<FlowVector>(target).expect("target flow");
    assert!(
        flow.velocity().x < 0.0,
        "Expected attraction towards source on x axis"
    );
}

#[test]
fn regression_homeostasis_adapts_with_energy_cost() {
    use crate::layers::{
        AmbientPressure, ContactType, ContainedIn, Homeostasis, OscillatorySignature, SpatialVolume,
    };

    let mut app = setup_structural_runtime_app();
    let (host, target) = {
        let mut commands = app.world_mut().commands();
        let host = commands
            .spawn((
                Transform::from_translation(Vec3::ZERO),
                Visibility::default(),
                BaseEnergy::new(500.0),
                SpatialVolume::new(5.0),
                AmbientPressure::new(-5.0, 2.0),
                OscillatorySignature::new(600.0, 0.0),
            ))
            .id();
        let target = commands
            .spawn((
                Transform::from_translation(Vec3::new(2.0, 0.0, 0.0)),
                Visibility::default(),
                BaseEnergy::new(200.0),
                SpatialVolume::new(0.5),
                OscillatorySignature::new(200.0, 0.0),
                Homeostasis::new(100.0, 1.0, 10.0, true),
                ContainedIn {
                    host,
                    contact: ContactType::Immersed,
                },
            ))
            .id();
        (host, target)
    };
    app.update();
    let qe_before = app.world().get::<BaseEnergy>(target).expect("energy").qe();
    let hz_before = app
        .world()
        .get::<OscillatorySignature>(target)
        .expect("osc")
        .frequency_hz();
    advance_simulation(&mut app, 0.1, 1);
    let qe_after = app.world().get::<BaseEnergy>(target).expect("energy").qe();
    let hz_after = app
        .world()
        .get::<OscillatorySignature>(target)
        .expect("osc")
        .frequency_hz();
    let _ = host;
    assert!(hz_after > hz_before);
    assert!(qe_after < qe_before);
}

#[test]
fn regression_structural_link_breaks_under_stress() {
    use crate::layers::{FlowVector, SpatialVolume, StructuralLink};

    let mut app = setup_structural_runtime_app();
    let (source, _target) = {
        let mut commands = app.world_mut().commands();
        let target = commands
            .spawn((
                Transform::from_translation(Vec3::new(20.0, 0.0, 0.0)),
                Visibility::default(),
                BaseEnergy::new(100.0),
                SpatialVolume::new(0.5),
                FlowVector::new(Vec2::ZERO, 0.0),
            ))
            .id();
        let source = commands
            .spawn((
                Transform::from_translation(Vec3::ZERO),
                Visibility::default(),
                BaseEnergy::new(100.0),
                SpatialVolume::new(0.5),
                FlowVector::new(Vec2::ZERO, 0.0),
                StructuralLink::new(target, 1.0, 10.0, 2.0),
            ))
            .id();
        (source, target)
    };
    app.update();
    advance_simulation(&mut app, 0.1, 1);
    assert!(
        app.world().get::<StructuralLink>(source).is_none(),
        "Link should break and be removed"
    );
}
