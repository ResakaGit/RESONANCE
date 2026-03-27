//! Intra-tick event buffers for batch simulation.
//!
//! Fixed-size. Cleared at the start of each tick. Zero heap allocation.

use super::constants::MAX_ENTITIES;

/// Per-world event buffers. Integrated into `SimWorldFlat`.
#[derive(Clone)]
pub struct EventBuffer {
    pub deaths:       [u8; MAX_ENTITIES],
    pub deaths_len:   usize,
    pub reproductions: [(u8, u8); 32],
    pub repro_len:    usize,
    pub hunger:       [u8; MAX_ENTITIES],
    pub hunger_len:   usize,
}

impl EventBuffer {
    pub fn new() -> Self {
        Self {
            deaths:       [0; MAX_ENTITIES],
            deaths_len:   0,
            reproductions: [(0, 0); 32],
            repro_len:    0,
            hunger:       [0; MAX_ENTITIES],
            hunger_len:   0,
        }
    }

    #[inline]
    pub fn clear(&mut self) {
        self.deaths_len = 0;
        self.repro_len  = 0;
        self.hunger_len = 0;
    }

    pub fn record_death(&mut self, idx: u8) {
        if self.deaths_len < self.deaths.len() {
            self.deaths[self.deaths_len] = idx;
            self.deaths_len += 1;
        }
    }

    pub fn record_reproduction(&mut self, parent: u8, child: u8) {
        if self.repro_len < self.reproductions.len() {
            self.reproductions[self.repro_len] = (parent, child);
            self.repro_len += 1;
        }
    }

    pub fn record_hunger(&mut self, idx: u8) {
        if self.hunger_len < self.hunger.len() {
            self.hunger[self.hunger_len] = idx;
            self.hunger_len += 1;
        }
    }
}

impl Default for EventBuffer {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_event_buffer_is_empty() {
        let eb = EventBuffer::new();
        assert_eq!(eb.deaths_len, 0);
        assert_eq!(eb.repro_len, 0);
        assert_eq!(eb.hunger_len, 0);
    }

    #[test]
    fn record_and_clear() {
        let mut eb = EventBuffer::new();
        eb.record_death(3);
        eb.record_reproduction(1, 5);
        eb.record_hunger(7);
        assert_eq!(eb.deaths_len, 1);
        assert_eq!(eb.repro_len, 1);
        assert_eq!(eb.hunger_len, 1);
        eb.clear();
        assert_eq!(eb.deaths_len, 0);
        assert_eq!(eb.repro_len, 0);
        assert_eq!(eb.hunger_len, 0);
    }

    #[test]
    fn record_death_saturates() {
        let mut eb = EventBuffer::new();
        for i in 0..MAX_ENTITIES + 10 {
            eb.record_death(i as u8);
        }
        assert_eq!(eb.deaths_len, MAX_ENTITIES);
    }
}
