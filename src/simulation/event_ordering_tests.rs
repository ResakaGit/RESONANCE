//! G9 — tests de **subconjuntos** de fases con el mismo mecanismo `.chain()` que el pipeline.
//!
//! Garantizan: (1) consumo en el mismo `run_schedule(FixedUpdate)` si el productor está en una fase
//! anterior de la cadena local; (2) sin nuevos `send`, el mismo `EventReader` no incrementa conteos
//! en una segunda pasada (cursor). **No** sustituyen un harness con `register_simulation_pipeline`
//! completo ni `App::update()` entero.

use bevy::prelude::*;

use crate::events::{DeathCause, DeathEvent};
use crate::simulation::Phase;

#[derive(Event, Debug, Clone, Copy)]
struct G9CrossChannelPending {
    value: u32,
}

#[derive(Resource, Default)]
struct G9DeathSeenThisTick(bool);

#[derive(Resource, Default)]
struct G9DeathTotal(u32);

#[derive(Resource)]
struct G9DeathTarget(Entity);

/// Emite un único `DeathEvent` en el primer tick de `Phase::AtomicLayer`.
fn g9_death_emit_once_physics(
    mut done: Local<bool>,
    mut writer: EventWriter<DeathEvent>,
    target: Res<G9DeathTarget>,
) {
    if *done {
        return;
    }
    *done = true;
    writer.send(DeathEvent {
        entity: target.0,
        cause: DeathCause::Dissipation,
    });
}

fn g9_death_consume_postphysics(
    mut reader: EventReader<DeathEvent>,
    mut seen: ResMut<G9DeathSeenThisTick>,
    mut total: ResMut<G9DeathTotal>,
) {
    for _ in reader.read() {
        seen.0 = true;
        total.0 += 1;
    }
}

fn g9_pending_emit_input(mut once: Local<bool>, mut writer: EventWriter<G9CrossChannelPending>) {
    if *once {
        return;
    }
    *once = true;
    writer.send(G9CrossChannelPending { value: 7 });
}

fn g9_pending_consume_prephysics(
    mut reader: EventReader<G9CrossChannelPending>,
    mut out: ResMut<G9PendingConsumed>,
    mut deliveries: ResMut<G9PendingDeliveryCount>,
) {
    for ev in reader.read() {
        out.0 = Some(ev.value);
        deliveries.0 += 1;
    }
}

#[derive(Resource, Default)]
struct G9PendingConsumed(Option<u32>);

#[derive(Resource, Default)]
struct G9PendingDeliveryCount(u32);

fn configure_g9_two_phase_chain(app: &mut App) {
    app.add_event::<DeathEvent>();
    app.configure_sets(FixedUpdate, (Phase::AtomicLayer, Phase::MetabolicLayer).chain());
    app.add_systems(
        FixedUpdate,
        g9_death_emit_once_physics.in_set(Phase::AtomicLayer),
    );
    app.add_systems(
        FixedUpdate,
        g9_death_consume_postphysics.in_set(Phase::MetabolicLayer),
    );
}

fn configure_g9_input_prephysics_chain(app: &mut App) {
    app.add_event::<G9CrossChannelPending>();
    app.configure_sets(FixedUpdate, (Phase::Input, Phase::ThermodynamicLayer).chain());
    app.add_systems(FixedUpdate, g9_pending_emit_input.in_set(Phase::Input));
    app.add_systems(
        FixedUpdate,
        g9_pending_consume_prephysics.in_set(Phase::ThermodynamicLayer),
    );
}

#[test]
fn g9_death_cross_phase_consumed_same_fixed_tick() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    configure_g9_two_phase_chain(&mut app);

    let entity = app
        .world_mut()
        .spawn(crate::layers::BaseEnergy::new(1.0))
        .id();
    app.insert_resource(G9DeathTarget(entity));
    app.init_resource::<G9DeathSeenThisTick>();
    app.init_resource::<G9DeathTotal>();

    app.world_mut().run_schedule(FixedUpdate);

    assert!(
        app.world().resource::<G9DeathSeenThisTick>().0,
        "PostPhysics debe ver el DeathEvent del mismo tick"
    );
    assert_eq!(app.world().resource::<G9DeathTotal>().0, 1);

    app.world_mut().run_schedule(FixedUpdate);
    assert_eq!(
        app.world().resource::<G9DeathTotal>().0,
        1,
        "sin nuevos send no debe incrementar (no leak al siguiente tick)"
    );
}

#[test]
fn g9_pending_input_before_prephysics_same_fixed_tick() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    configure_g9_input_prephysics_chain(&mut app);
    app.init_resource::<G9PendingConsumed>();
    app.init_resource::<G9PendingDeliveryCount>();

    app.world_mut().run_schedule(FixedUpdate);

    assert_eq!(
        app.world().resource::<G9PendingConsumed>().0,
        Some(7),
        "consumidor PrePhysics debe leer lo emitido en Input en el mismo FixedUpdate"
    );
    assert_eq!(app.world().resource::<G9PendingDeliveryCount>().0, 1);

    app.world_mut().run_schedule(FixedUpdate);
    assert_eq!(
        app.world().resource::<G9PendingDeliveryCount>().0,
        1,
        "sin re-envío: segunda vuelta no entrega el mismo pending otra vez"
    );
}
