//! Pre-allocated scratch buffers for per-tick temporary data.
//!
//! One `ScratchPad` per thread (thread-local in batch mode).
//! Cleared at the start of each tick. Zero heap allocation.

use super::constants::MAX_ENTITIES;

/// Reusable buffers for intra-tick computation.
///
/// All `_len` fields are logical lengths into the fixed arrays.
/// `clear()` resets all lengths to zero — the array contents are stale but ignored.
pub struct ScratchPad {
    /// Collision / interaction pairs: max C(64,2) = 2016.
    pub pairs:        [(u8, u8); 2048],
    pub pairs_len:    usize,

    /// Spatial neighbor indices for a single query.
    pub neighbors:    [u8; MAX_ENTITIES],
    pub neighbors_len: usize,

    /// Death indices recorded during a tick.
    pub deaths:       [u8; MAX_ENTITIES],
    pub deaths_len:   usize,
}

impl ScratchPad {
    pub fn new() -> Self {
        Self {
            pairs:        [(0, 0); 2048],
            pairs_len:    0,
            neighbors:    [0; MAX_ENTITIES],
            neighbors_len: 0,
            deaths:       [0; MAX_ENTITIES],
            deaths_len:   0,
        }
    }

    /// Reset all logical lengths. O(1) — no array zeroing.
    #[inline]
    pub fn clear(&mut self) {
        self.pairs_len    = 0;
        self.neighbors_len = 0;
        self.deaths_len   = 0;
    }
}

impl Default for ScratchPad {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_scratch_has_zero_lengths() {
        let s = ScratchPad::new();
        assert_eq!(s.pairs_len, 0);
        assert_eq!(s.neighbors_len, 0);
        assert_eq!(s.deaths_len, 0);
    }

    #[test]
    fn clear_resets_lengths() {
        let mut s = ScratchPad::new();
        s.pairs_len = 42;
        s.neighbors_len = 7;
        s.deaths_len = 3;
        s.clear();
        assert_eq!(s.pairs_len, 0);
        assert_eq!(s.neighbors_len, 0);
        assert_eq!(s.deaths_len, 0);
    }
}
