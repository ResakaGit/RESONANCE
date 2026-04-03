//! GS-1: Lockstep types, resources, and systems.

use bevy::prelude::*;

use crate::blueprint::constants::CHECKSUM_LOG_PRUNE_WINDOW;
use crate::blueprint::equations::tick_checksum;
use crate::blueprint::ids::WorldEntityId;
use crate::layers::BaseEnergy;
use crate::runtime_platform::simulation_tick::SimulationClock;
use crate::sim_world::InputCommand;

// ─── Components ───────────────────────────────────────────────────────────────

/// Input de un jugador para tick T. Transmitido sobre red. SparseSet (por-tick efímero).
#[derive(Component, Reflect, Debug, Clone, Copy, PartialEq, Eq)]
#[reflect(Component)]
#[component(storage = "SparseSet")]
pub struct InputPacket {
    pub tick_id: u64,
    pub player_id: u8,
    pub command_count: u8,
    pub checksum: u64,
}

// ─── Events ───────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DesyncResult {
    Synchronized,
    Desynced {
        player_a: u8,
        player_b: u8,
        tick_id: u64,
    },
}

#[derive(Event, Debug, Clone)]
pub struct DesyncEvent {
    pub result: DesyncResult,
    pub tick_id: u64,
}

// ─── Resources ────────────────────────────────────────────────────────────────

/// Lockstep runtime configuration.
#[derive(Resource, Reflect, Debug, Clone)]
#[reflect(Resource)]
pub struct LockstepConfig {
    pub player_count: u8,
    pub input_delay_ticks: u32,
    pub tick_rate_hz: f32,
}

impl Default for LockstepConfig {
    fn default() -> Self {
        Self {
            player_count: 2,
            input_delay_ticks: 3,
            tick_rate_hz: 20.0,
        }
    }
}

/// One player's buffered commands for a specific tick.
#[derive(Debug, Clone)]
pub struct BufferedInput {
    pub tick_id: u64,
    pub player_id: u8,
    pub commands: Vec<InputCommand>,
}

/// Buffer de inputs pendientes. Vec ordenado por `(tick_id, player_id)` (NO HashMap).
#[derive(Resource, Default, Debug)]
pub struct InputBuffer {
    pub entries: Vec<BufferedInput>,
}

impl InputBuffer {
    /// Count entries for a specific tick.
    #[inline]
    pub fn count_for_tick(&self, tick_id: u64) -> usize {
        self.entries.iter().filter(|e| e.tick_id == tick_id).count()
    }
}

/// Checksums por tick para detección de desync.
/// Vec ordenado por `(tick_id, player_id)` (NO HashMap).
#[derive(Resource, Default, Debug)]
pub struct ChecksumLog {
    pub entries: Vec<(u64, u8, u64)>, // (tick_id, player_id, checksum)
}

/// Flag que controla si el tick puede avanzar (todos los inputs recibidos).
#[derive(Resource, Default, Debug, Reflect)]
#[reflect(Resource)]
pub struct LockstepRunCondition {
    pub can_advance: bool,
}

// ─── Systems ─────────────────────────────────────────────────────────────────

/// Verifica que todos los inputs del tick T están disponibles antes de avanzar.
/// Debe ejecutarse en `Phase::Input`, before `InputChannelSet::PlatformWill`.
pub fn lockstep_input_gate_system(
    config: Res<LockstepConfig>,
    buffer: Res<InputBuffer>,
    clock: Res<SimulationClock>,
    mut run_condition: ResMut<LockstepRunCondition>,
) {
    let target_tick = clock.tick_id + config.input_delay_ticks as u64;
    let received = buffer.count_for_tick(target_tick);
    let can = received >= config.player_count as usize;
    if run_condition.can_advance != can {
        run_condition.can_advance = can;
    }
}

/// Registra checksum del estado de energía actual. Ejecutado en `PostUpdate`.
/// Pruna entradas más antiguas que `CHECKSUM_LOG_PRUNE_WINDOW` ticks.
pub fn lockstep_checksum_record_system(
    query: Query<(&WorldEntityId, &BaseEnergy)>,
    clock: Res<SimulationClock>,
    mut log: ResMut<ChecksumLog>,
) {
    let mut energies: Vec<(u32, f32)> = query.iter().map(|(id, e)| (id.0, e.qe())).collect();
    energies.sort_unstable_by_key(|(id, _)| *id);
    let values: Vec<f32> = energies.into_iter().map(|(_, qe)| qe).collect();
    let checksum = tick_checksum(&values);
    // 255 = local sentinel player_id (this node's own state).
    log.entries.push((clock.tick_id, 255, checksum));
    let cutoff = clock.tick_id.saturating_sub(CHECKSUM_LOG_PRUNE_WINDOW);
    log.entries.retain(|(tick, _, _)| *tick >= cutoff);
}

// ─── Tests ───────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::blueprint::constants::{CHECKSUM_LOG_PRUNE_WINDOW, MAX_COMMANDS_PER_TICK};

    #[test]
    fn input_buffer_count_for_tick_empty() {
        let buf = InputBuffer::default();
        assert_eq!(buf.count_for_tick(0), 0);
    }

    #[test]
    fn input_buffer_count_for_tick_matches_only_target() {
        let buf = InputBuffer {
            entries: vec![
                BufferedInput {
                    tick_id: 5,
                    player_id: 0,
                    commands: vec![],
                },
                BufferedInput {
                    tick_id: 5,
                    player_id: 1,
                    commands: vec![],
                },
                BufferedInput {
                    tick_id: 6,
                    player_id: 0,
                    commands: vec![],
                },
            ],
        };
        assert_eq!(buf.count_for_tick(5), 2);
        assert_eq!(buf.count_for_tick(6), 1);
        assert_eq!(buf.count_for_tick(7), 0);
    }

    #[test]
    fn lockstep_config_default_values() {
        let cfg = LockstepConfig::default();
        assert_eq!(cfg.player_count, 2);
        assert_eq!(cfg.input_delay_ticks, 3);
        assert!((cfg.tick_rate_hz - 20.0).abs() < f32::EPSILON);
    }

    #[test]
    fn max_commands_per_tick_constant_fits_in_command_count_field() {
        // command_count is u8; MAX_COMMANDS_PER_TICK must not overflow u8.
        assert!(MAX_COMMANDS_PER_TICK <= u8::MAX);
    }

    #[test]
    fn checksum_log_prune_window_nonzero() {
        assert!(CHECKSUM_LOG_PRUNE_WINDOW > 0);
    }
}
