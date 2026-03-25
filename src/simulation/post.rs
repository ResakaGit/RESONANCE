use bevy::prelude::*;
use bevy::utils::HashSet;

use crate::events::DeathEvent;
use crate::layers::{Faction, MobaIdentity};
use crate::world::Scoreboard;

/// Sistema: Lógica de facciones y puntuación.
/// Fase: Phase::MetabolicLayer
pub fn faction_identity_system(
    mut commands: Commands,
    mut scoreboard: ResMut<Scoreboard>,
    mut ev_death: EventReader<DeathEvent>,
    query: Query<Option<&MobaIdentity>>,
    mut dedupe: Local<HashSet<Entity>>,
) {
    dedupe.clear();
    for event in ev_death.read() {
        if !dedupe.insert(event.entity) {
            continue;
        }
        if let Ok(identity_opt) = query.get(event.entity) {
            if let Some(identity) = identity_opt {
                match identity.faction() {
                    Faction::Red => {
                        scoreboard.blue_points += 1;
                        scoreboard.blue_kills += 1;
                    }
                    Faction::Blue => {
                        scoreboard.red_points += 1;
                        scoreboard.red_kills += 1;
                    }
                    _ => {}
                }
            }

            commands.entity(event.entity).despawn();
        }
    }
}
