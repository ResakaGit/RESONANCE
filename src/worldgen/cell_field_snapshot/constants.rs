//! Constantes del módulo snapshot: EPI1 (huella) + EPI4 (contrato GPU/WGSL).
//! No reutilizar FNV como hash criptográfico; es huella determinista para cache e inferencia.

// ═══ EPI4 — contrato WGSL (`assets/shaders/cell_field_snapshot.wgsl`) / SSBO ═══
// Orden fijado por `layers::coherence::MatterState` (variantes en orden de declaración).

/// Discriminante `matter_state` en GPU para [`MatterState::Solid`].
pub const GPU_MATTER_STATE_SOLID: u32 = 0;
/// Discriminante para [`MatterState::Liquid`].
pub const GPU_MATTER_STATE_LIQUID: u32 = 1;
/// Discriminante para [`MatterState::Gas`].
pub const GPU_MATTER_STATE_GAS: u32 = 2;
/// Discriminante para [`MatterState::Plasma`].
pub const GPU_MATTER_STATE_PLASMA: u32 = 3;

/// `materialized_present` cuando hay entidad materializada.
pub const GPU_MATERIALIZED_PRESENT: u32 = 1;
/// Sin entidad (`Option::None` en CPU).
pub const GPU_MATERIALIZED_ABSENT: u32 = 0;

// ═══ EPI1 — FNV-1a 32-bit ═══

/// Bias inicial FNV-1a 32-bit.
pub const FNV1A_U32_OFFSET_BASIS: u32 = 0x811c_9dc5;
/// Primo FNV-1a 32-bit (distinto de hashes u64 en otros módulos).
pub const FNV1A_U32_PRIME: u32 = 0x0100_0193;

/// Tope de `size_of::<CellFieldSnapshot>()` en CI (layout típico x86_64 ~40 B).
#[cfg(test)]
pub const SNAPSHOT_STRUCT_MAX_BYTES: usize = 48;
/// Tope razonable para `Option<CellFieldSnapshot>` en el vector denso de cache.
#[cfg(test)]
pub const CACHE_OPTION_ENTRY_MAX_BYTES: usize = 56;

/// Un paso FNV-1a 32-bit (word ya es entero, p. ej. `f32::to_bits()`).
#[inline]
pub(crate) fn fnv1a_u32_mix(mut hash: u32, word: u32) -> u32 {
    hash ^= word;
    hash.wrapping_mul(FNV1A_U32_PRIME)
}
