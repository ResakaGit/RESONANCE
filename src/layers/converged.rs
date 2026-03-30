//! Flag de convergencia genérico para procesos iterativos.
//! Generic convergence flag for iterative processes.
//!
//! Insertado cuando un proceso estabiliza (delta < ε). Removido cuando
//! inputs ambientales cambian (entity se mueve, vecino muta, etc.).
//! SparseSet: transient por definición.

use core::marker::PhantomData;

use bevy::prelude::*;

/// Marca que el proceso `T` convergió. Insertado por el sistema de `T`, removido por eventos de invalidación.
/// Marks that process `T` converged. Inserted by `T`'s system, removed by invalidation events.
#[derive(Component, Debug, Clone, Copy, PartialEq)]
#[component(storage = "SparseSet")]
pub struct Converged<T: Send + Sync + 'static> {
    env_hash: u64,
    _marker:  PhantomData<T>,
}

impl<T: Send + Sync + 'static> Converged<T> {
    /// Crea un flag de convergencia con el hash ambiental actual.
    /// Creates a convergence flag with the current environment hash.
    #[inline]
    pub fn new(env_hash: u64) -> Self {
        Self { env_hash, _marker: PhantomData }
    }

    /// Hash del entorno cuando convergió.
    /// Environment hash when converged.
    #[inline]
    pub fn env_hash(&self) -> u64 { self.env_hash }

    /// ¿Sigue válido? `true` si el hash ambiental no cambió.
    /// Still valid? `true` if environment hash unchanged.
    #[inline]
    pub fn is_valid(&self, current_env_hash: u64) -> bool {
        self.env_hash == current_env_hash
    }
}

/// Hash determinista de un f32 para usar como env_hash.
/// Deterministic f32 hash for env_hash.
///
/// NaN se normaliza a 0 para consistencia (todos los NaN → mismo hash).
#[inline]
pub fn hash_f32(v: f32) -> u64 {
    if v.is_nan() { return 0; }
    f32::to_bits(v) as u64
}

/// Hash determinista de una posición 2D para env_hash.
/// Deterministic 2D position hash for env_hash.
///
/// Knuth multiplicative hash: `2654435761 = 2^32 × φ⁻¹` (golden ratio).
#[inline]
pub fn hash_pos(x: f32, z: f32) -> u64 {
    const KNUTH_PHI: u64 = 2_654_435_761;
    let a = f32::to_bits(x) as u64;
    let b = f32::to_bits(z) as u64;
    a.wrapping_mul(KNUTH_PHI).wrapping_add(b)
}

#[cfg(test)]
mod tests {
    use super::*;

    struct MockProcess;

    #[test]
    fn converged_valid_when_hash_matches() {
        let c = Converged::<MockProcess>::new(42);
        assert!(c.is_valid(42));
    }

    #[test]
    fn converged_invalid_when_hash_differs() {
        let c = Converged::<MockProcess>::new(42);
        assert!(!c.is_valid(99));
    }

    #[test]
    fn hash_f32_deterministic() {
        assert_eq!(hash_f32(1.5), hash_f32(1.5));
    }

    #[test]
    fn hash_f32_different_values_differ() {
        assert_ne!(hash_f32(1.5), hash_f32(1.6));
    }

    #[test]
    fn hash_f32_nan_normalized_to_zero() {
        assert_eq!(hash_f32(f32::NAN), 0);
    }

    #[test]
    fn hash_pos_deterministic() {
        assert_eq!(hash_pos(1.0, 2.0), hash_pos(1.0, 2.0));
    }

    #[test]
    fn hash_pos_different_positions_differ() {
        assert_ne!(hash_pos(1.0, 2.0), hash_pos(2.0, 1.0));
    }
}
