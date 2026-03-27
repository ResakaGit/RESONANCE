//! GS-1: Netcode lockstep constants.

/// Maximum acceptable input delay for competitive play (ticks).
/// At 20Hz, 6 ticks = 300ms one-way; ≤300ms RTT is LAN/regional quality.
pub const LOCKSTEP_MAX_ACCEPTABLE_DELAY_TICKS: u32 = 6;

/// Imperceptible delay threshold (ticks).
/// At 20Hz, 3 ticks = 150ms RTT — subconsciously unnoticeable.
pub const LOCKSTEP_IMPERCEPTIBLE_DELAY_TICKS: u32 = 3;

/// Maximum entries kept in ChecksumLog before pruning.
/// 240 ticks = 12 seconds at 20Hz.
pub const CHECKSUM_LOG_MAX_ENTRIES: usize = 240;

/// Maximum input commands a player may issue per tick.
pub const MAX_COMMANDS_PER_TICK: u8 = 8;

/// Pruning window for ChecksumLog — ticks older than this are discarded.
pub const CHECKSUM_LOG_PRUNE_WINDOW: u64 = 120;
