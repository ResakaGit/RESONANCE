use std::collections::HashMap;

use bevy::prelude::*;

use crate::layers::Faction;

/// Recurso: qué entidades son visibles para cada facción.
#[derive(Resource, Default, Debug)]
pub struct PerceptionCache {
    visible_by_faction: HashMap<Faction, Vec<Entity>>,
}

impl PerceptionCache {
    pub fn clear(&mut self) {
        self.visible_by_faction.clear();
    }

    pub fn mark_visible(&mut self, faction: Faction, entity: Entity) {
        self.visible_by_faction
            .entry(faction)
            .or_default()
            .push(entity);
    }

    pub fn is_visible_to(&self, faction: Faction, entity: Entity) -> bool {
        self.visible_by_faction
            .get(&faction)
            .map(|list| list.contains(&entity))
            .unwrap_or(false)
    }
}
