use bevy::prelude::*;
use super::*;

#[test]
fn id_generator_same_order_yields_same_sequence() {
    let mut a = IdGenerator::default();
    let mut b = IdGenerator::default();
    assert_eq!(a.next_champion(), b.next_champion());
    assert_eq!(a.next_world(), b.next_world());
    assert_eq!(a.next_effect(), b.next_effect());
    assert_eq!(a.next_pool(), b.next_pool());
    assert_eq!(a.next_organ(), b.next_organ());
    assert_eq!(a.next_agent(), b.next_agent());
    assert_eq!(a, b);
}

#[test]
fn id_generator_counters_are_independent() {
    let mut g = IdGenerator::default();
    let c0 = g.next_champion();
    let w0 = g.next_world();
    let e0 = g.next_effect();
    assert_eq!(c0.0, 0);
    assert_eq!(w0.0, 0);
    assert_eq!(e0.0, 0);
    assert_eq!(g.next_champion().0, 1);
    assert_eq!(g.next_world().0, 1);
    assert_eq!(g.next_effect().0, 1);
}

#[test]
fn pool_id_roundtrip() {
    let id = PoolId::new(42);
    assert_eq!(id.raw(), 42);
}

#[test]
fn organ_id_roundtrip() {
    let id = OrganId::new(7);
    assert_eq!(id.raw(), 7);
}

#[test]
fn agent_id_roundtrip() {
    let id = AgentId::new(99);
    assert_eq!(id.raw(), 99);
}

#[test]
fn pool_id_generator_increments_independently() {
    let mut g = IdGenerator::default();
    let p0 = g.next_pool();
    let o0 = g.next_organ();
    let a0 = g.next_agent();
    assert_eq!(p0.raw(), 0);
    assert_eq!(o0.raw(), 0);
    assert_eq!(a0.raw(), 0);
    assert_eq!(g.next_pool().raw(), 1);
    assert_eq!(g.next_organ().raw(), 1);
    assert_eq!(g.next_agent().raw(), 1);
    assert_eq!(g.pool_count(), 2);
    assert_eq!(g.organ_count(), 2);
    assert_eq!(g.agent_count(), 2);
}

#[test]
fn pool_id_eq_hash() {
    let a = PoolId::new(5);
    let b = PoolId::new(5);
    let c = PoolId::new(6);
    assert_eq!(a, b);
    assert_ne!(a, c);
}

#[test]
fn organ_id_eq_hash() {
    let a = OrganId::new(3);
    let b = OrganId::new(3);
    let c = OrganId::new(4);
    assert_eq!(a, b);
    assert_ne!(a, c);
}

#[test]
fn agent_id_eq_hash() {
    let a = AgentId::new(10);
    let b = AgentId::new(10);
    let c = AgentId::new(11);
    assert_eq!(a, b);
    assert_ne!(a, c);
}

#[test]
fn new_ids_are_copy() {
    let p = PoolId::new(1);
    let o = OrganId::new(2);
    let a = AgentId::new(3);
    let _p2 = p;
    let _o2 = o;
    let _a2 = a;
    // Copy: originals still usable
    assert_eq!(p.raw(), 1);
    assert_eq!(o.raw(), 2);
    assert_eq!(a.raw(), 3);
}

fn minimal_app_with_id_observers() -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.init_resource::<EntityLookup>();
    setup_entity_id_observers(&mut app);
    app
}

#[test]
fn entity_lookup_tracks_champion_until_despawn() {
    let mut app = minimal_app_with_id_observers();
    let id = ChampionId(7);
    let e = app.world_mut().spawn(id).id();
    app.world_mut().flush();

    assert_eq!(
        app.world().resource::<EntityLookup>().champion_entity(id),
        Some(e)
    );

    app.world_mut().entity_mut(e).despawn();
    app.world_mut().flush();

    assert_eq!(
        app.world().resource::<EntityLookup>().champion_entity(id),
        None
    );
}

#[test]
fn entity_lookup_remove_component_unregisters() {
    let mut app = minimal_app_with_id_observers();
    let id = ChampionId(3);
    let e = app.world_mut().spawn(id).id();
    app.world_mut().flush();
    assert_eq!(
        app.world().resource::<EntityLookup>().champion_entity(id),
        Some(e)
    );

    app.world_mut().entity_mut(e).remove::<ChampionId>();
    app.world_mut().flush();
    assert_eq!(
        app.world().resource::<EntityLookup>().champion_entity(id),
        None
    );
}

#[test]
fn entity_lookup_tracks_world_until_despawn() {
    let mut app = minimal_app_with_id_observers();
    let id = WorldEntityId(11);
    let e = app.world_mut().spawn(id).id();
    app.world_mut().flush();
    assert_eq!(
        app.world().resource::<EntityLookup>().world_entity(id),
        Some(e)
    );
    app.world_mut().entity_mut(e).despawn();
    app.world_mut().flush();
    assert_eq!(
        app.world().resource::<EntityLookup>().world_entity(id),
        None
    );
}

#[test]
fn entity_lookup_tracks_effect_until_remove() {
    let mut app = minimal_app_with_id_observers();
    let id = EffectId(5);
    let e = app.world_mut().spawn(id).id();
    app.world_mut().flush();
    assert_eq!(
        app.world().resource::<EntityLookup>().effect_entity(id),
        Some(e)
    );
    app.world_mut().entity_mut(e).remove::<EffectId>();
    app.world_mut().flush();
    assert_eq!(
        app.world().resource::<EntityLookup>().effect_entity(id),
        None
    );
}

#[test]
fn entity_lookup_remove_stale_forward_if_id_rebound() {
    let mut app = minimal_app_with_id_observers();
    let id = ChampionId(9);
    let e_old = app.world_mut().spawn(id).id();
    app.world_mut().flush();
    let e_new = app.world_mut().spawn(id).id();
    app.world_mut().flush();
    assert_eq!(
        app.world().resource::<EntityLookup>().champion_entity(id),
        Some(e_new)
    );
    app.world_mut().entity_mut(e_old).despawn();
    app.world_mut().flush();
    assert_eq!(
        app.world().resource::<EntityLookup>().champion_entity(id),
        Some(e_new)
    );
}
