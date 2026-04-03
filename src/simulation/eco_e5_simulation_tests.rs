//! Tests sprint E5 — integración `ContextLookup` en physics/reactions.
//! Ver `docs/sprints/ECO_BOUNDARIES/README.md` (E5 cerrado).

use std::time::Duration;

use bevy::prelude::*;

use crate::blueprint::recipes::EffectRecipe;
use crate::eco::boundary_field::{EcoBoundaryField, aggregate_zone_class_contexts};
use crate::eco::constants::SUBAQUATIC_DENSITY_THRESHOLD;
use crate::eco::context_lookup::context_response_legacy_baseline;
use crate::eco::contracts::{BoundaryMarker, ContextResponse, ZoneClass, ZoneContext};
use crate::events::{
    CatalysisEvent, CatalysisRequest, DeathEvent, DeltaEnergyCommit, PhaseTransitionEvent,
};
use crate::layers::{
    AlchemicalInjector, BaseEnergy, DespawnOnContact, FlowVector, MatterCoherence, MatterState,
    ModifiedField, OnContactEffect, OscillatorySignature, SpatialVolume,
};
use crate::runtime_platform::compat_2d3d::SimWorldTransformParams;
use crate::runtime_platform::simulation_tick::SimulationElapsed;
use crate::simulation::physics::{dissipation_system, movement_will_drag_system};
use crate::simulation::reactions::{
    SpellMarker, catalysis_energy_reducer_system, catalysis_math_strategy_system,
    catalysis_side_effects_system, catalysis_spatial_filter_system, state_transitions_system,
};
use crate::world::{SpatialEntry, SpatialIndex};
use crate::worldgen::EnergyFieldGrid;
use crate::worldgen::propagation::{cell_density, cell_temperature};

fn minimal_time_app() -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_event::<DeathEvent>();
    app.add_event::<PhaseTransitionEvent>();
    app.add_event::<CatalysisRequest>();
    app.add_event::<DeltaEnergyCommit>();
    app.add_event::<CatalysisEvent>();
    app.init_resource::<Time>();
    app.insert_resource(SimWorldTransformParams::default());
    app.insert_resource(EnergyFieldGrid::new(1, 1, 1.0, Vec2::ZERO));
    app
}

fn advance_secs(app: &mut App, dt: f32) {
    app.world_mut()
        .resource_mut::<Time>()
        .advance_by(Duration::from_secs_f32(dt));
}

fn drain_catalysis_requests(app: &mut App) -> usize {
    app.world_mut()
        .resource_mut::<Events<CatalysisRequest>>()
        .drain()
        .count()
}

fn drain_catalysis_events(app: &mut App) -> Vec<CatalysisEvent> {
    app.world_mut()
        .resource_mut::<Events<CatalysisEvent>>()
        .drain()
        .collect()
}

/// Campo 1×1 alineado, interior zona 0, contexto controlado (E5 equivalencia / mod).
fn eco_field_1x1(zone: ZoneContext) -> EcoBoundaryField {
    let grid = EnergyFieldGrid::new(1, 1, 1.0, Vec2::ZERO);
    let markers = vec![BoundaryMarker::Interior { zone_id: 0 }];
    let zone_contexts = std::collections::HashMap::from([(0, zone)]);
    let zone_class_context =
        aggregate_zone_class_contexts(&markers, &grid, &zone_contexts, grid.cell_size);
    EcoBoundaryField {
        width: 1,
        height: 1,
        cell_size: 1.0,
        origin: Vec2::ZERO,
        markers,
        cell_zone_ids: vec![0],
        zone_class_context,
        zone_contexts,
        last_seen_grid_generation: 0,
        last_recompute_sim_tick: 0,
    }
}

#[test]
fn e5_dissipation_sin_eco_igual_multiplicador_uno() {
    let mut app = minimal_time_app();
    app.add_systems(Update, dissipation_system);
    let e = app
        .world_mut()
        .spawn((
            BaseEnergy::new(100.0),
            FlowVector::new(Vec2::new(3.0, 4.0), 0.5),
            Transform::from_xyz(0.5, 0.5, 0.0),
        ))
        .id();
    advance_secs(&mut app, 1.0);
    app.update();
    let qe_sin = app.world().get::<BaseEnergy>(e).unwrap().qe();

    let mut app2 = minimal_time_app();
    app2.insert_resource(eco_field_1x1(ZoneContext {
        pressure: 1.0,
        viscosity: 1.0,
        temperature_base: 0.0,
        dissipation_mod: 1.0,
        reactivity_mod: 1.0,
    }));
    app2.add_systems(Update, dissipation_system);
    let e2 = app2
        .world_mut()
        .spawn((
            BaseEnergy::new(100.0),
            FlowVector::new(Vec2::new(3.0, 4.0), 0.5),
            Transform::from_xyz(0.5, 0.5, 0.0),
        ))
        .id();
    advance_secs(&mut app2, 1.0);
    app2.update();
    let qe_con = app2.world().get::<BaseEnergy>(e2).unwrap().qe();

    assert!(
        (qe_sin - qe_con).abs() < 1e-3,
        "Surface neutro vs sin resource: qe {qe_sin} vs {qe_con}"
    );
}

#[test]
fn e5_dissipation_con_eco_respeta_dissipation_mod() {
    let mut app = minimal_time_app();
    app.insert_resource(eco_field_1x1(ZoneContext {
        pressure: 1.0,
        viscosity: 1.0,
        temperature_base: 0.0,
        dissipation_mod: 0.0,
        reactivity_mod: 1.0,
    }));
    app.add_systems(Update, dissipation_system);
    let e = app
        .world_mut()
        .spawn((
            BaseEnergy::new(100.0),
            FlowVector::new(Vec2::ZERO, 2.0),
            Transform::from_xyz(0.5, 0.5, 0.0),
        ))
        .id();
    advance_secs(&mut app, 1.0);
    app.update();
    assert!(
        (app.world().get::<BaseEnergy>(e).unwrap().qe() - 100.0).abs() < 1e-4,
        "dissipation_mod 0 ⇒ sin pérdida por flujo (solo rate base * 0)"
    );
}

#[test]
fn e5_movement_sin_eco_igual_campo_neutro() {
    let spawn = |app: &mut App| {
        app.world_mut()
            .spawn((
                BaseEnergy::new(50.0),
                SpatialVolume::new(1.0),
                FlowVector::new(Vec2::new(8.0, 0.0), 0.0),
                Transform::from_xyz(0.5, 0.5, 0.0),
                MatterCoherence::new(MatterState::Gas, 1000.0, 0.5),
            ))
            .id()
    };

    let mut app_a = minimal_time_app();
    app_a.add_systems(Update, movement_will_drag_system);
    let ea = spawn(&mut app_a);

    let mut app_b = minimal_time_app();
    app_b.insert_resource(eco_field_1x1(ZoneContext::default()));
    app_b.add_systems(Update, movement_will_drag_system);
    let eb = spawn(&mut app_b);

    for app in [&mut app_a, &mut app_b] {
        advance_secs(app, 1.0 / 60.0);
        app.update();
        advance_secs(app, 1.0 / 60.0);
        app.update();
    }

    let va = app_a.world().get::<FlowVector>(ea).unwrap().velocity();
    let vb = app_b.world().get::<FlowVector>(eb).unwrap().velocity();
    // Puede haber diferencia numérica mínima entre baseline legado y ruta Interior+`ZoneContext::default`.
    assert!(
        (va - vb).length() < 0.05,
        "sin resource vs eco neutro (ZoneContext::default): v {:?} vs {:?}",
        va,
        vb
    );
}

#[test]
fn e5_movement_usa_viscosidad_contexto() {
    let mut app_high = minimal_time_app();
    app_high.insert_resource(eco_field_1x1(ZoneContext {
        pressure: 1.0,
        viscosity: 8.0,
        temperature_base: 0.0,
        dissipation_mod: 1.0,
        reactivity_mod: 1.0,
    }));
    app_high.add_systems(Update, movement_will_drag_system);
    let eh = app_high
        .world_mut()
        .spawn((
            BaseEnergy::new(50.0),
            SpatialVolume::new(1.0),
            FlowVector::new(Vec2::new(12.0, 0.0), 0.0),
            Transform::from_xyz(0.5, 0.5, 0.0),
            // Gas: sin tope de materia (líquido caparía en 5 y enmascararía el arrastre en un tick).
            MatterCoherence::new(MatterState::Gas, 1000.0, 0.5),
        ))
        .id();

    let mut app_low = minimal_time_app();
    app_low.insert_resource(eco_field_1x1(ZoneContext {
        pressure: 1.0,
        viscosity: 0.01,
        temperature_base: 0.0,
        dissipation_mod: 1.0,
        reactivity_mod: 1.0,
    }));
    app_low.add_systems(Update, movement_will_drag_system);
    let el = app_low
        .world_mut()
        .spawn((
            BaseEnergy::new(50.0),
            SpatialVolume::new(1.0),
            FlowVector::new(Vec2::new(12.0, 0.0), 0.0),
            Transform::from_xyz(0.5, 0.5, 0.0),
            MatterCoherence::new(MatterState::Gas, 1000.0, 0.5),
        ))
        .id();

    // Un solo tick puede dar delta 0 en el primer `update` según orden de plugins; dos avances aseguran dt > 0.
    advance_secs(&mut app_high, 1.0 / 60.0);
    app_high.update();
    advance_secs(&mut app_high, 1.0 / 60.0);
    advance_secs(&mut app_low, 1.0 / 60.0);
    app_low.update();
    advance_secs(&mut app_low, 1.0 / 60.0);
    app_high.update();
    app_low.update();

    let v_high = app_high
        .world()
        .get::<FlowVector>(eh)
        .unwrap()
        .velocity()
        .length();
    let v_low = app_low
        .world()
        .get::<FlowVector>(el)
        .unwrap()
        .velocity()
        .length();
    assert!(
        v_high < v_low,
        "higher context viscosity ⇒ stronger braking: v_high={v_high} v_low={v_low}"
    );
}

#[test]
fn e5_state_transitions_neutro_sin_o_con_eco() {
    let mut app_a = minimal_time_app();
    app_a.add_systems(Update, state_transitions_system);
    let ea = app_a
        .world_mut()
        .spawn((
            BaseEnergy::new(10.0),
            SpatialVolume::new(1.0),
            Transform::from_xyz(0.5, 0.5, 0.0),
            MatterCoherence::new(MatterState::Liquid, 5000.0, 0.5),
        ))
        .id();

    let mut app_b = minimal_time_app();
    app_b.insert_resource(eco_field_1x1(ZoneContext {
        pressure: 1.0,
        viscosity: 1.0,
        temperature_base: 0.0,
        dissipation_mod: 1.0,
        reactivity_mod: 1.0,
    }));
    app_b.add_systems(Update, state_transitions_system);
    let eb = app_b
        .world_mut()
        .spawn((
            BaseEnergy::new(10.0),
            SpatialVolume::new(1.0),
            Transform::from_xyz(0.5, 0.5, 0.0),
            MatterCoherence::new(MatterState::Liquid, 5000.0, 0.5),
        ))
        .id();

    advance_secs(&mut app_a, 1.0);
    advance_secs(&mut app_b, 1.0);
    app_a.update();
    app_b.update();

    assert_eq!(
        app_a.world().get::<MatterCoherence>(ea).unwrap().state(),
        app_b.world().get::<MatterCoherence>(eb).unwrap().state()
    );
}

#[test]
fn e5_state_transitions_suma_temperature_base() {
    let mut app = minimal_time_app();
    app.insert_resource(eco_field_1x1(ZoneContext {
        pressure: 1.0,
        viscosity: 1.0,
        temperature_base: -5000.0,
        dissipation_mod: 1.0,
        reactivity_mod: 1.0,
    }));
    app.add_systems(Update, state_transitions_system);
    let e = app
        .world_mut()
        .spawn((
            BaseEnergy::new(10.0),
            SpatialVolume::new(1.0),
            Transform::from_xyz(0.5, 0.5, 0.0),
            MatterCoherence::new(MatterState::Liquid, 50.0, 0.5),
        ))
        .id();
    advance_secs(&mut app, 1.0);
    app.update();
    assert_eq!(
        app.world().get::<MatterCoherence>(e).unwrap().state(),
        MatterState::Solid,
        "strong negative thermal offset must force solid"
    );
}

#[test]
fn e5_catalysis_reactivity_cero_skip_con_eco() {
    let mut app = minimal_time_app();
    app.insert_resource(eco_field_1x1(ZoneContext {
        pressure: 1.0,
        viscosity: 1.0,
        temperature_base: 0.0,
        dissipation_mod: 1.0,
        reactivity_mod: 0.0,
    }));
    app.insert_resource(SimulationElapsed { secs: 0.5 });
    let mut index = SpatialIndex::new(5.0);
    let _spell = app
        .world_mut()
        .spawn((
            Transform::from_xyz(0.5, 0.5, 0.0),
            AlchemicalInjector::new(80.0, 450.0, 20.0),
            OscillatorySignature::new(450.0, 0.1),
            SpellMarker { caster: None },
        ))
        .id();
    let target = app
        .world_mut()
        .spawn((
            Transform::from_xyz(0.5, 0.5, 0.0),
            SpatialVolume::new(0.5),
            OscillatorySignature::new(700.0, 0.2),
        ))
        .id();
    index.insert(SpatialEntry {
        entity: target,
        position: Vec2::new(0.5, 0.5),
        radius: 0.5,
    });
    app.insert_resource(index);
    app.add_systems(Update, catalysis_spatial_filter_system);
    advance_secs(&mut app, 1.0 / 60.0);
    app.update();
    assert!(
        drain_catalysis_requests(&mut app) == 0,
        "reactivity_mod 0 ⇒ sin catálisis"
    );
}

#[test]
fn e5_catalysis_skip_void_con_eco() {
    let mut app = minimal_time_app();
    // Campo alineado (eco activo) pero posición fuera del grid ⇒ `void_context_response`, no baseline desalineado.
    app.insert_resource(eco_field_1x1(ZoneContext::default()));
    app.insert_resource(SimulationElapsed { secs: 0.5 });
    let mut index = SpatialIndex::new(5.0);
    let _spell = app
        .world_mut()
        .spawn((
            Transform::from_xyz(50.0, 50.0, 0.0),
            AlchemicalInjector::new(80.0, 450.0, 20.0),
            OscillatorySignature::new(450.0, 0.1),
            SpellMarker { caster: None },
        ))
        .id();
    let target = app
        .world_mut()
        .spawn((
            Transform::from_xyz(50.0, 50.0, 0.0),
            SpatialVolume::new(0.5),
            OscillatorySignature::new(700.0, 0.2),
        ))
        .id();
    index.insert(SpatialEntry {
        entity: target,
        position: Vec2::new(50.0, 50.0),
        radius: 0.5,
    });
    app.insert_resource(index);
    app.add_systems(Update, catalysis_spatial_filter_system);
    advance_secs(&mut app, 1.0 / 60.0);
    app.update();
    assert!(
        drain_catalysis_requests(&mut app) == 0,
        "Void (off grid) must not generate strikes"
    );
}

#[test]
fn e5_catalysis_sin_eco_no_skip_aunque_fuera_de_grid() {
    let mut app = minimal_time_app();
    // Sin `EcoBoundaryField`: `ContextLookup` usa baseline y no skipea.
    app.insert_resource(SimulationElapsed { secs: 0.5 });
    let mut index = SpatialIndex::new(5.0);
    let _spell = app
        .world_mut()
        .spawn((
            Transform::from_xyz(50.0, 50.0, 0.0),
            AlchemicalInjector::new(80.0, 450.0, 20.0),
            OscillatorySignature::new(450.0, 0.1),
            SpellMarker { caster: None },
        ))
        .id();
    let target = app
        .world_mut()
        .spawn((
            Transform::from_xyz(50.0, 50.0, 0.0),
            SpatialVolume::new(0.5),
            OscillatorySignature::new(700.0, 0.2),
        ))
        .id();
    index.insert(SpatialEntry {
        entity: target,
        position: Vec2::new(50.0, 50.0),
        radius: 0.5,
    });
    app.insert_resource(index);
    app.add_systems(Update, catalysis_spatial_filter_system);
    advance_secs(&mut app, 1.0 / 60.0);
    app.update();
    assert!(
        drain_catalysis_requests(&mut app) > 0,
        "sin eco, catálisis sigue evaluándose fuera del grid ambiental"
    );
}

#[test]
fn e5_catalysis_filter_emite_payload_correcto() {
    let mut app = minimal_time_app();
    let mut index = SpatialIndex::new(5.0);

    let spell = app
        .world_mut()
        .spawn((
            Transform::from_xyz(0.5, 0.0, 0.5),
            AlchemicalInjector::new(80.0, 450.0, 20.0),
            OscillatorySignature::new(450.0, 0.1),
            OnContactEffect {
                recipe: EffectRecipe {
                    field: ModifiedField::VelocityMultiplier,
                    magnitude: 1.2,
                    fuel_qe: 5.0,
                    dissipation: 1.0,
                },
            },
            DespawnOnContact,
            SpellMarker { caster: None },
        ))
        .id();
    let target = app
        .world_mut()
        .spawn((
            Transform::from_xyz(0.5, 0.0, 0.5),
            SpatialVolume::new(0.5),
            OscillatorySignature::new(450.0, 0.1),
        ))
        .id();
    index.insert(SpatialEntry {
        entity: target,
        position: Vec2::new(0.5, 0.5),
        radius: 0.5,
    });
    app.insert_resource(index);
    app.add_systems(Update, catalysis_spatial_filter_system);
    advance_secs(&mut app, 1.0 / 60.0);
    app.update();

    let requests: Vec<CatalysisRequest> = app
        .world_mut()
        .resource_mut::<Events<CatalysisRequest>>()
        .drain()
        .collect();
    assert_eq!(requests.len(), 1);
    assert_eq!(requests[0].spell, spell);
    assert_eq!(requests[0].target, target);
    assert_eq!(requests[0].caster, None);
    assert!(requests[0].on_contact_effect.is_some());
    assert!(requests[0].despawn_on_contact);
}

#[test]
fn e5_catalysis_pipeline_strategy_y_reducer_aplican_qe() {
    let mut app = minimal_time_app();
    app.insert_resource(SimulationElapsed { secs: 0.5 });
    app.init_resource::<crate::blueprint::IdGenerator>();
    let mut index = SpatialIndex::new(5.0);

    let spell = app
        .world_mut()
        .spawn((
            Transform::from_xyz(0.5, 0.0, 0.5),
            AlchemicalInjector::new(80.0, 450.0, 20.0),
            OscillatorySignature::new(450.0, 0.1),
            BaseEnergy::new(10.0),
            SpellMarker { caster: None },
        ))
        .id();
    let target = app
        .world_mut()
        .spawn((
            Transform::from_xyz(0.5, 0.0, 0.5),
            SpatialVolume::new(0.5),
            BaseEnergy::new(100.0),
            OscillatorySignature::new(450.0, 0.1),
            MatterCoherence::new(MatterState::Liquid, 1200.0, 0.6),
        ))
        .id();
    index.insert(SpatialEntry {
        entity: target,
        position: Vec2::new(0.5, 0.5),
        radius: 0.5,
    });
    app.insert_resource(index);
    app.add_systems(
        Update,
        (
            catalysis_spatial_filter_system,
            catalysis_math_strategy_system,
            catalysis_energy_reducer_system,
            catalysis_side_effects_system,
        )
            .chain(),
    );
    advance_secs(&mut app, 1.0 / 60.0);
    app.update();

    let qe_after = app.world().get::<BaseEnergy>(target).unwrap().qe();
    assert_ne!(
        qe_after, 100.0,
        "la cadena completa debe mutar SSOT de energía"
    );
    let events = drain_catalysis_events(&mut app);
    assert_eq!(events.len(), 1);
    assert_eq!(events[0].spell, spell);
    assert_eq!(events[0].target, target);
}

#[test]
fn e5_catalysis_filter_preserva_orden_determinista_por_spell() {
    let mut app = minimal_time_app();
    let mut index = SpatialIndex::new(5.0);

    let spell_b = app
        .world_mut()
        .spawn((
            Transform::from_xyz(0.5, 0.0, 0.5),
            AlchemicalInjector::new(50.0, 450.0, 20.0),
            OscillatorySignature::new(450.0, 0.0),
            SpellMarker { caster: None },
        ))
        .id();
    let spell_a = app
        .world_mut()
        .spawn((
            Transform::from_xyz(0.5, 0.0, 0.5),
            AlchemicalInjector::new(50.0, 450.0, 20.0),
            OscillatorySignature::new(450.0, 0.0),
            SpellMarker { caster: None },
        ))
        .id();
    let target = app
        .world_mut()
        .spawn((
            Transform::from_xyz(0.5, 0.0, 0.5),
            SpatialVolume::new(0.5),
            OscillatorySignature::new(450.0, 0.0),
        ))
        .id();
    index.insert(SpatialEntry {
        entity: target,
        position: Vec2::new(0.5, 0.5),
        radius: 0.5,
    });
    app.insert_resource(index);
    app.add_systems(Update, catalysis_spatial_filter_system);
    advance_secs(&mut app, 1.0 / 60.0);
    app.update();

    let requests: Vec<CatalysisRequest> = app
        .world_mut()
        .resource_mut::<Events<CatalysisRequest>>()
        .drain()
        .collect();
    assert_eq!(requests.len(), 2);
    let expected_first = if spell_a.to_bits() < spell_b.to_bits() {
        spell_a
    } else {
        spell_b
    };
    assert_eq!(requests[0].spell, expected_first);
}

#[test]
fn e5_legacy_baseline_coincide_con_contrato_publico() {
    let c: ContextResponse = context_response_legacy_baseline();
    assert!((c.dissipation_mod - 1.0).abs() < 1e-5 && (c.reactivity_mod - 1.0).abs() < 1e-5);
}

#[test]
fn e5_subaquatic_lookup_viscosidad_swamp() {
    let mut grid = EnergyFieldGrid::new(1, 1, 1.0, Vec2::ZERO);
    let cell_size = 1.0_f32;
    let qe = (SUBAQUATIC_DENSITY_THRESHOLD * 2.0 + 0.5) * cell_size.powi(3);
    let c = grid.cell_xy_mut(0, 0).unwrap();
    c.accumulated_qe = qe;
    c.temperature = cell_temperature(cell_density(qe, cell_size));
    c.matter_state = MatterState::Liquid;
    c.dominant_frequency_hz = 250.0;
    let mut field = EcoBoundaryField::default();
    assert!(field.recompute_if_needed(&grid, 0));
    let ctx =
        crate::eco::context_lookup::context_at_inner(&grid, &field, None, Vec2::new(0.5, 0.5), 0);
    assert_eq!(ctx.zone, ZoneClass::Subaquatic);
    assert!(
        (ctx.viscosity - crate::blueprint::constants::BIOME_SWAMP_VISCOSITY).abs() < 0.05,
        "líquido denso ⇒ viscosidad pantano en agregado de zona"
    );
}
