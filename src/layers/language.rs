//! ET-15: Language Capacity — LanguageCapacity component. Capa T4-2.

use bevy::prelude::*;

pub const MAX_VOCAB_SIZE: usize = 8;

/// Capa T4-2: LanguageCapacity — capacidad simbólica de una entidad.
/// `vocabulary`: hashes de símbolos conocidos (u32 — Hard Block 6 compliant).
#[derive(Component, Reflect, Debug, Clone)]
#[reflect(Component)]
pub struct LanguageCapacity {
    pub vocabulary: [u32; MAX_VOCAB_SIZE],
    pub vocab_count: u8,
    pub signal_range: f32,
    pub encoding_cost: f32,
}

impl Default for LanguageCapacity {
    fn default() -> Self {
        Self {
            vocabulary: [0u32; MAX_VOCAB_SIZE],
            vocab_count: 0,
            signal_range: 20.0,
            encoding_cost: 0.2,
        }
    }
}

impl LanguageCapacity {
    pub fn vocab_slice(&self) -> &[u32] {
        &self.vocabulary[..self.vocab_count as usize]
    }

    pub fn has_symbol(&self, symbol: u32) -> bool {
        self.vocab_slice().contains(&symbol)
    }

    /// Añade un símbolo al vocabulario. Retorna true si fue añadido.
    pub fn add_symbol(&mut self, symbol: u32) -> bool {
        if self.vocab_count as usize >= MAX_VOCAB_SIZE {
            return false;
        }
        if self.has_symbol(symbol) {
            return false;
        }
        self.vocabulary[self.vocab_count as usize] = symbol;
        self.vocab_count += 1;
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn add_symbol_increases_count() {
        let mut lc = LanguageCapacity::default();
        assert!(lc.add_symbol(42));
        assert_eq!(lc.vocab_count, 1);
    }

    #[test]
    fn add_symbol_no_duplicates() {
        let mut lc = LanguageCapacity::default();
        lc.add_symbol(42);
        assert!(!lc.add_symbol(42));
        assert_eq!(lc.vocab_count, 1);
    }

    #[test]
    fn add_symbol_respects_max_vocab() {
        let mut lc = LanguageCapacity::default();
        for i in 0..MAX_VOCAB_SIZE {
            lc.add_symbol(i as u32);
        }
        assert!(!lc.add_symbol(99));
        assert_eq!(lc.vocab_count as usize, MAX_VOCAB_SIZE);
    }
}
