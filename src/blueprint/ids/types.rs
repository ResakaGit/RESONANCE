//! Newtypes de ID persistente + generador determinista.

use bevy::prelude::*;

// ─── Newtypes (identificadores estables) ───────────────────────────────────

/// ID persistente para héroes / campeones. Estable si el orden de spawn es determinista.
#[derive(Component, Copy, Clone, Reflect, Hash, Eq, PartialEq, Debug)]
#[reflect(Component)]
pub struct ChampionId(pub u32);

/// ID persistente para entidades de mundo (cristales, biomas, estructuras, etc.).
#[derive(Component, Copy, Clone, Reflect, Hash, Eq, PartialEq, Debug)]
#[reflect(Component)]
pub struct WorldEntityId(pub u32);

/// ID persistente para proyectiles y entidades-efecto (L10, hechizos).
#[derive(Component, Copy, Clone, Reflect, Hash, Eq, PartialEq, Debug)]
#[reflect(Component)]
pub struct EffectId(pub u32);

// ─── Generador determinista (única fuente de contadores) ────────────────────

/// Contadores secuenciales. Si el orden de `next_*` es el mismo entre runs, los IDs coinciden.
#[derive(Resource, Default, Debug, Clone, PartialEq, Eq)]
pub struct IdGenerator {
    next_champion: u32,
    next_world: u32,
    next_effect: u32,
}

impl IdGenerator {
    pub fn next_champion(&mut self) -> ChampionId {
        let id = ChampionId(self.next_champion);
        self.next_champion = self.next_champion.wrapping_add(1);
        id
    }

    pub fn next_world(&mut self) -> WorldEntityId {
        let id = WorldEntityId(self.next_world);
        self.next_world = self.next_world.wrapping_add(1);
        id
    }

    pub fn next_effect(&mut self) -> EffectId {
        let id = EffectId(self.next_effect);
        self.next_effect = self.next_effect.wrapping_add(1);
        id
    }
}
