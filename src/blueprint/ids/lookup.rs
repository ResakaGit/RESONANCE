//! Resolución ID → Entity vía observers (sincronización automática).

use std::collections::HashMap;

use bevy::prelude::*;
use tracing::warn;

use super::types::{ChampionId, EffectId, WorldEntityId};

// ─── Lookup ID → Entity (sincronizado vía observers) ────────────────────────

/// Resolución de IDs fuertes a `Entity` viva en el mundo actual.
#[derive(Resource, Default, Debug)]
pub struct EntityLookup {
    champions:         HashMap<ChampionId, Entity>,
    champion_by_entity: HashMap<Entity, ChampionId>,
    world_entities:    HashMap<WorldEntityId, Entity>,
    world_by_entity:   HashMap<Entity, WorldEntityId>,
    effects:           HashMap<EffectId, Entity>,
    effect_by_entity:  HashMap<Entity, EffectId>,
}

impl EntityLookup {
    pub fn champion_entity(&self, id: ChampionId) -> Option<Entity> {
        self.champions.get(&id).copied()
    }

    pub fn world_entity(&self, id: WorldEntityId) -> Option<Entity> {
        self.world_entities.get(&id).copied()
    }

    pub fn effect_entity(&self, id: EffectId) -> Option<Entity> {
        self.effects.get(&id).copied()
    }

    pub(crate) fn on_champion_added(&mut self, id: ChampionId, entity: Entity) {
        if let Some(prev) = self.champions.insert(id, entity) {
            if prev != entity {
                self.champion_by_entity.remove(&prev);
            }
        }
        self.champion_by_entity.insert(entity, id);
    }

    pub(crate) fn on_champion_removed(&mut self, entity: Entity) {
        if let Some(id) = self.champion_by_entity.remove(&entity) {
            if self.champions.get(&id) == Some(&entity) {
                self.champions.remove(&id);
            }
        }
    }

    pub(crate) fn on_world_added(&mut self, id: WorldEntityId, entity: Entity) {
        if let Some(prev) = self.world_entities.insert(id, entity) {
            if prev != entity {
                self.world_by_entity.remove(&prev);
            }
        }
        self.world_by_entity.insert(entity, id);
    }

    pub(crate) fn on_world_removed(&mut self, entity: Entity) {
        if let Some(id) = self.world_by_entity.remove(&entity) {
            if self.world_entities.get(&id) == Some(&entity) {
                self.world_entities.remove(&id);
            }
        }
    }

    pub(crate) fn on_effect_added(&mut self, id: EffectId, entity: Entity) {
        if let Some(prev) = self.effects.insert(id, entity) {
            if prev != entity {
                self.effect_by_entity.remove(&prev);
            }
        }
        self.effect_by_entity.insert(entity, id);
    }

    pub(crate) fn on_effect_removed(&mut self, entity: Entity) {
        if let Some(id) = self.effect_by_entity.remove(&entity) {
            if self.effects.get(&id) == Some(&entity) {
                self.effects.remove(&id);
            }
        }
    }
}

// ─── Observers ──────────────────────────────────────────────────────────────

/// Registra observers globales que mantienen [`EntityLookup`] ante add/remove de IDs.
pub fn setup_entity_id_observers(app: &mut App) {
    app.add_observer(on_champion_id_added);
    app.add_observer(on_champion_id_removed);
    app.add_observer(on_world_entity_id_added);
    app.add_observer(on_world_entity_id_removed);
    app.add_observer(on_effect_id_added);
    app.add_observer(on_effect_id_removed);
}

fn on_champion_id_added(
    trigger: Trigger<OnAdd, ChampionId>,
    ids: Query<&ChampionId>,
    mut lookup: ResMut<EntityLookup>,
) {
    let entity = trigger.entity();
    let Ok(id) = ids.get(entity) else {
        warn!(
            "OnAdd<ChampionId>: sin componente en {:?} tras trigger",
            entity
        );
        return;
    };
    lookup.on_champion_added(*id, entity);
}

fn on_champion_id_removed(
    trigger: Trigger<OnRemove, ChampionId>,
    mut lookup: ResMut<EntityLookup>,
) {
    lookup.on_champion_removed(trigger.entity());
}

fn on_world_entity_id_added(
    trigger: Trigger<OnAdd, WorldEntityId>,
    ids: Query<&WorldEntityId>,
    mut lookup: ResMut<EntityLookup>,
) {
    let entity = trigger.entity();
    let Ok(id) = ids.get(entity) else {
        warn!(
            "OnAdd<WorldEntityId>: sin componente en {:?} tras trigger",
            entity
        );
        return;
    };
    lookup.on_world_added(*id, entity);
}

fn on_world_entity_id_removed(
    trigger: Trigger<OnRemove, WorldEntityId>,
    mut lookup: ResMut<EntityLookup>,
) {
    lookup.on_world_removed(trigger.entity());
}

fn on_effect_id_added(
    trigger: Trigger<OnAdd, EffectId>,
    ids: Query<&EffectId>,
    mut lookup: ResMut<EntityLookup>,
) {
    let entity = trigger.entity();
    let Ok(id) = ids.get(entity) else {
        warn!(
            "OnAdd<EffectId>: sin componente en {:?} tras trigger",
            entity
        );
        return;
    };
    lookup.on_effect_added(*id, entity);
}

fn on_effect_id_removed(trigger: Trigger<OnRemove, EffectId>, mut lookup: ResMut<EntityLookup>) {
    lookup.on_effect_removed(trigger.entity());
}
