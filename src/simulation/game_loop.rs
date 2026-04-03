use bevy::prelude::*;

use crate::blueprint::constants::QE_NUCLEUS_VIABILITY_THRESHOLD;
use crate::blueprint::equations::{is_nucleus_viable, nucleus_effective_intake};
use crate::layers::{AlchemicalEngine, BaseEnergy, MatterCoherence, MobaIdentity};
use crate::runtime_platform::simulation_tick::SimulationClock;
use crate::simulation::states::PlayState;

/// Marca un EnergyNucleus como objetivo de victoria de su facción.
/// SparseSet: máx 1-2 entidades en partida.
#[derive(Component, Reflect, Debug, Clone, Copy)]
#[reflect(Component)]
#[component(storage = "SparseSet")]
pub struct VictoryNucleus {
    pub is_final_target: bool,
    pub base_intake_qe: f32,
}

/// Evento de victoria. Emitido una vez cuando el núcleo colapsa.
#[derive(Event, Debug, Clone)]
pub struct VictoryEvent {
    pub winner_faction: crate::layers::Faction,
    pub loser_nucleus: Entity,
    pub tick_id: u64,
}

/// Fuente de verdad para el resultado de la partida.
#[derive(Resource, Debug, Default, Reflect)]
#[reflect(Resource)]
pub struct GameOutcome {
    pub winner: Option<crate::layers::Faction>,
    pub winning_tick: Option<u64>,
}

/// Reduce el intake del núcleo según su daño estructural.
/// Phase::ThermodynamicLayer — antes de cálculos de energía.
pub fn nucleus_intake_decay_system(
    mut query: Query<(&mut AlchemicalEngine, &MatterCoherence), With<VictoryNucleus>>,
) {
    for (mut engine, coherence) in &mut query {
        let effective =
            nucleus_effective_intake(engine.base_intake(), coherence.structural_damage());
        if (engine.intake() - effective).abs() > f32::EPSILON {
            engine.set_intake(effective);
        }
    }
}

/// Verifica condición de victoria. Phase::MetabolicLayer, after metabolic_stress systems.
pub fn victory_check_system(
    nuclei: Query<(Entity, &BaseEnergy, &MobaIdentity, &VictoryNucleus)>,
    clock: Res<SimulationClock>,
    mut outcome: ResMut<GameOutcome>,
    mut events: EventWriter<VictoryEvent>,
    mut next_state: ResMut<NextState<PlayState>>,
) {
    if outcome.winner.is_some() {
        return;
    }
    for (entity, energy, identity, nucleus) in &nuclei {
        if !nucleus.is_final_target {
            continue;
        }
        if !is_nucleus_viable(energy.qe(), QE_NUCLEUS_VIABILITY_THRESHOLD) {
            let winner = identity.faction().opponent();
            outcome.winner = Some(winner);
            outcome.winning_tick = Some(clock.tick_id);
            events.send(VictoryEvent {
                winner_faction: winner,
                loser_nucleus: entity,
                tick_id: clock.tick_id,
            });
            next_state.set(PlayState::Victory);
        }
    }
}
