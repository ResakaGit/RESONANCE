//! Helpers de test compartidos (ecosistema EA4–EA7, contrato L0).
//! Reduce duplicación de `drain` de `DeathEvent` y conteos `BaseEnergy` entre módulos.

use bevy::prelude::*;

use crate::events::DeathEvent;
use crate::layers::BaseEnergy;

pub fn drain_death_events(app: &mut App) -> Vec<DeathEvent> {
    app.world_mut()
        .resource_mut::<Events<DeathEvent>>()
        .drain()
        .collect()
}

pub fn count_base_energy(world: &mut World) -> usize {
    world.query::<&BaseEnergy>().iter(world).count()
}
