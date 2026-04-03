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

/// Strong ID for EnergyPool entities — EC track. Not reusable, survives serialization.
#[derive(Component, Copy, Clone, Reflect, Hash, Eq, PartialEq, Debug)]
#[reflect(Component)]
pub struct PoolId(pub u32);

impl PoolId {
    pub fn new(id: u32) -> Self {
        Self(id)
    }
    pub fn raw(self) -> u32 {
        self.0
    }
}

/// Strong ID for organ entities — morphogenesis track. Not reusable, survives serialization.
#[derive(Component, Copy, Clone, Reflect, Hash, Eq, PartialEq, Debug)]
#[reflect(Component)]
pub struct OrganId(pub u32);

impl OrganId {
    pub fn new(id: u32) -> Self {
        Self(id)
    }
    pub fn raw(self) -> u32 {
        self.0
    }
}

/// Strong ID for MOBA agent entities (MobaIdentity). Not reusable, survives serialization.
#[derive(Component, Copy, Clone, Reflect, Hash, Eq, PartialEq, Debug)]
#[reflect(Component)]
pub struct AgentId(pub u32);

impl AgentId {
    pub fn new(id: u32) -> Self {
        Self(id)
    }
    pub fn raw(self) -> u32 {
        self.0
    }
}

// ─── Generador determinista (única fuente de contadores) ────────────────────

/// Contadores secuenciales. Si el orden de `next_*` es el mismo entre runs, los IDs coinciden.
#[derive(Resource, Default, Reflect, Debug, Clone, PartialEq, Eq)]
#[reflect(Resource)]
pub struct IdGenerator {
    next_champion: u32,
    next_world: u32,
    next_effect: u32,
    next_pool: u32,
    next_organ: u32,
    next_agent: u32,
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

    pub fn next_pool(&mut self) -> PoolId {
        let id = PoolId(self.next_pool);
        self.next_pool = self.next_pool.wrapping_add(1);
        id
    }

    pub fn next_organ(&mut self) -> OrganId {
        let id = OrganId(self.next_organ);
        self.next_organ = self.next_organ.wrapping_add(1);
        id
    }

    pub fn next_agent(&mut self) -> AgentId {
        let id = AgentId(self.next_agent);
        self.next_agent = self.next_agent.wrapping_add(1);
        id
    }

    /// Number of champion IDs issued so far.
    pub fn champion_count(&self) -> u32 {
        self.next_champion
    }
    /// Number of world-entity IDs issued so far.
    pub fn world_count(&self) -> u32 {
        self.next_world
    }
    /// Number of effect IDs issued so far.
    pub fn effect_count(&self) -> u32 {
        self.next_effect
    }
    /// Number of pool IDs issued so far.
    pub fn pool_count(&self) -> u32 {
        self.next_pool
    }
    /// Number of organ IDs issued so far.
    pub fn organ_count(&self) -> u32 {
        self.next_organ
    }
    /// Number of agent IDs issued so far.
    pub fn agent_count(&self) -> u32 {
        self.next_agent
    }
}
