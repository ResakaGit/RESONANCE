// Sprint G11 — Strong IDs integration tests.
// IdGenerator tests are pure struct (no Bevy). EntityLookup tests use MinimalPlugins + observers.

use resonance::blueprint::ids::{ChampionId, EffectId, EntityLookup, IdGenerator, WorldEntityId};
use resonance::blueprint::setup_entity_id_observers;

// ─── IdGenerator — pure struct, no Bevy App ─────────────────────────────────

#[test]
fn id_generator_sequential_champion_ids() {
    let mut g = IdGenerator::default();
    assert_eq!(g.next_champion(), ChampionId(0));
    assert_eq!(g.next_champion(), ChampionId(1));
    assert_eq!(g.next_champion(), ChampionId(2));
}

#[test]
fn id_generator_independent_counters() {
    let mut g = IdGenerator::default();
    assert_eq!(g.next_champion().0, 0);
    assert_eq!(g.next_world().0, 0);
    assert_eq!(g.next_effect().0, 0);
    assert_eq!(g.next_champion().0, 1);
    assert_eq!(g.next_world().0, 1);
    assert_eq!(g.next_effect().0, 1);
}

#[test]
fn id_generator_count_accessors_reflect_issued_ids() {
    let mut g = IdGenerator::default();
    assert_eq!(g.champion_count(), 0);
    g.next_champion();
    g.next_champion();
    assert_eq!(g.champion_count(), 2);
    g.next_world();
    assert_eq!(g.world_count(), 1);
    assert_eq!(g.effect_count(), 0);
    g.next_effect();
    assert_eq!(g.effect_count(), 1);
}

#[test]
fn id_generator_same_spawn_order_yields_same_ids() {
    let mut a = IdGenerator::default();
    let mut b = IdGenerator::default();
    assert_eq!(a.next_champion(), b.next_champion());
    assert_eq!(a.next_world(), b.next_world());
    assert_eq!(a.next_effect(), b.next_effect());
}

// ─── EntityLookup — observer-based via MinimalPlugins ───────────────────────

fn minimal_app() -> bevy::prelude::App {
    let mut app = bevy::prelude::App::new();
    app.add_plugins(bevy::prelude::MinimalPlugins);
    app.init_resource::<EntityLookup>();
    setup_entity_id_observers(&mut app);
    app
}

#[test]
fn entity_lookup_insert_find_champion() {
    let mut app = minimal_app();
    let id = ChampionId(5);
    let entity = app.world_mut().spawn(id).id();
    app.world_mut().flush();
    assert_eq!(
        app.world().resource::<EntityLookup>().champion_entity(id),
        Some(entity)
    );
}

#[test]
fn entity_lookup_remove_champion_returns_none() {
    let mut app = minimal_app();
    let id = ChampionId(5);
    let entity = app.world_mut().spawn(id).id();
    app.world_mut().flush();
    app.world_mut().entity_mut(entity).remove::<ChampionId>();
    app.world_mut().flush();
    assert_eq!(
        app.world().resource::<EntityLookup>().champion_entity(id),
        None
    );
}

#[test]
fn entity_lookup_multiple_champions_find_all() {
    let mut app = minimal_app();
    let e0 = app.world_mut().spawn(ChampionId(7)).id();
    let e1 = app.world_mut().spawn(ChampionId(1)).id();
    let e2 = app.world_mut().spawn(ChampionId(3)).id();
    app.world_mut().flush();
    let lookup = app.world().resource::<EntityLookup>();
    assert_eq!(lookup.champion_entity(ChampionId(7)), Some(e0));
    assert_eq!(lookup.champion_entity(ChampionId(1)), Some(e1));
    assert_eq!(lookup.champion_entity(ChampionId(3)), Some(e2));
}

#[test]
fn entity_lookup_unknown_id_returns_none() {
    let app = minimal_app();
    let lookup = app.world().resource::<EntityLookup>();
    assert_eq!(lookup.champion_entity(ChampionId(99)), None);
    assert_eq!(lookup.world_entity(WorldEntityId(99)), None);
    assert_eq!(lookup.effect_entity(EffectId(99)), None);
}

#[test]
fn entity_lookup_world_and_effect_insert_find() {
    let mut app = minimal_app();
    let ew = app.world_mut().spawn(WorldEntityId(2)).id();
    let ee = app.world_mut().spawn(EffectId(4)).id();
    app.world_mut().flush();
    let lookup = app.world().resource::<EntityLookup>();
    assert_eq!(lookup.world_entity(WorldEntityId(2)), Some(ew));
    assert_eq!(lookup.effect_entity(EffectId(4)), Some(ee));
}

#[test]
fn entity_lookup_rebound_id_keeps_latest_entity() {
    let mut app = minimal_app();
    let e_old = app.world_mut().spawn(ChampionId(9)).id();
    app.world_mut().flush();
    let e_new = app.world_mut().spawn(ChampionId(9)).id();
    app.world_mut().flush();
    assert_eq!(
        app.world()
            .resource::<EntityLookup>()
            .champion_entity(ChampionId(9)),
        Some(e_new)
    );
    app.world_mut().entity_mut(e_old).despawn();
    app.world_mut().flush();
    assert_eq!(
        app.world()
            .resource::<EntityLookup>()
            .champion_entity(ChampionId(9)),
        Some(e_new)
    );
}
