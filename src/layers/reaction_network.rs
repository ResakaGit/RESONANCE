//! AP-0: ReactionNetwork — colección de reacciones, Resource.
//! AP-0: ReactionNetwork — collection of reactions, Resource.
//!
//! El formato RON "spec" es human-friendly; se compila a la representación
//! interna (arrays `Copy` en `Reaction`) al cargar.  Esto mantiene el hot-path
//! plano y cache-friendly sin forzar al editor a contar sentinels.
//!
//! RON spec format (ver `assets/reactions/raf_minimal.ron`):
//!
//! ```ron
//! (
//!     reactions: [
//!         (reactants: [(0,1),(1,1)], products: [(2,1)], k: 1.0, freq: 50.0),
//!         ...
//!     ]
//! )
//! ```
//!
//! Cada tupla `(u8, u8)` es `(species_id, stoich_count)`.

use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use crate::blueprint::constants::chemistry::{
    MAX_PRODUCTS_PER_REACTION, MAX_REACTANTS_PER_REACTION, MAX_REACTIONS_PER_NETWORK,
};
use crate::layers::reaction::{Reaction, StoichEntry};

// ── Internal, optimized representation ──────────────────────────────────────

/// Colección de reacciones. `reactions[i]` es estable — `ReactionId(i)` es un
/// índice válido hasta que la red se re-cargue.
#[derive(Resource, Clone, Debug, Default)]
pub struct ReactionNetwork {
    reactions: Vec<Reaction>,
}

/// Índice estable dentro de una `ReactionNetwork` concreta.
#[repr(transparent)]
#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug, Ord, PartialOrd)]
pub struct ReactionId(pub u32);

impl ReactionId {
    #[inline] pub const fn index(self) -> usize { self.0 as usize }
}

impl ReactionNetwork {
    pub fn empty() -> Self { Self::default() }

    /// Construye desde un vector validado. Filtra reacciones mal formadas.
    pub fn from_reactions(reactions: impl IntoIterator<Item = Reaction>) -> Self {
        let reactions: Vec<Reaction> = reactions.into_iter()
            .filter(Reaction::is_well_formed)
            .take(MAX_REACTIONS_PER_NETWORK)
            .collect();
        Self { reactions }
    }

    #[inline] pub fn len(&self) -> usize { self.reactions.len() }
    #[inline] pub fn is_empty(&self) -> bool { self.reactions.is_empty() }
    #[inline] pub fn reactions(&self) -> &[Reaction] { &self.reactions }

    #[inline]
    pub fn get(&self, id: ReactionId) -> Option<&Reaction> {
        self.reactions.get(id.index())
    }

    /// Iterador `(ReactionId, &Reaction)` para algoritmos estilo RAF.
    #[inline]
    pub fn iter_indexed(&self) -> impl Iterator<Item = (ReactionId, &Reaction)> + '_ {
        self.reactions.iter().enumerate().map(|(i, r)| (ReactionId(i as u32), r))
    }
}

// ── RON spec (human-friendly) ───────────────────────────────────────────────

/// Par `(species_id, stoich_count)` legible en RON como `(0, 1)`.
#[derive(Copy, Clone, Debug, Serialize, Deserialize)]
pub struct StoichSpec(pub u8, pub u8);

/// Versión RON-amigable de `Reaction`: `Vec` de tamaño variable, sin sentinels.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ReactionSpec {
    pub reactants: Vec<StoichSpec>,
    pub products: Vec<StoichSpec>,
    pub k: f32,
    pub freq: f32,
}

/// Top-level del archivo RON.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct ReactionNetworkSpec {
    pub reactions: Vec<ReactionSpec>,
}

impl ReactionNetwork {
    /// Parse desde string RON. Devuelve error de serde si el archivo es inválido
    /// o `Err` con mensaje si una reacción excede `MAX_REACTANTS`/`MAX_PRODUCTS`.
    pub fn from_ron_str(text: &str) -> Result<Self, ReactionNetworkError> {
        let spec: ReactionNetworkSpec = ron::de::from_str(text)?;
        Self::from_spec(spec)
    }

    /// Compila un `ReactionNetworkSpec` a la representación interna.
    pub fn from_spec(spec: ReactionNetworkSpec) -> Result<Self, ReactionNetworkError> {
        let mut out = Vec::with_capacity(spec.reactions.len().min(MAX_REACTIONS_PER_NETWORK));
        for (i, r) in spec.reactions.into_iter().enumerate() {
            if r.reactants.len() > MAX_REACTANTS_PER_REACTION {
                return Err(ReactionNetworkError::TooManyReactants {
                    reaction: i,
                    got: r.reactants.len(),
                    max: MAX_REACTANTS_PER_REACTION,
                });
            }
            if r.products.len() > MAX_PRODUCTS_PER_REACTION {
                return Err(ReactionNetworkError::TooManyProducts {
                    reaction: i,
                    got: r.products.len(),
                    max: MAX_PRODUCTS_PER_REACTION,
                });
            }
            let reactants = pack_stoich::<MAX_REACTANTS_PER_REACTION>(&r.reactants)
                .ok_or(ReactionNetworkError::InvalidStoich { reaction: i })?;
            let products = pack_stoich::<MAX_PRODUCTS_PER_REACTION>(&r.products)
                .ok_or(ReactionNetworkError::InvalidStoich { reaction: i })?;
            let reaction = Reaction { reactants, products, k: r.k, freq: r.freq };
            if !reaction.is_well_formed() {
                return Err(ReactionNetworkError::IllFormed { reaction: i });
            }
            out.push(reaction);
            if out.len() >= MAX_REACTIONS_PER_NETWORK { break; }
        }
        Ok(Self { reactions: out })
    }
}

fn pack_stoich<const N: usize>(entries: &[StoichSpec]) -> Option<[StoichEntry; N]> {
    let mut arr = [StoichEntry::EMPTY; N];
    for (slot, s) in arr.iter_mut().zip(entries.iter()) {
        *slot = StoichEntry::new(s.0, s.1)?;
    }
    Some(arr)
}

// ── Errors ──────────────────────────────────────────────────────────────────

#[derive(Debug)]
pub enum ReactionNetworkError {
    Ron(ron::error::SpannedError),
    TooManyReactants { reaction: usize, got: usize, max: usize },
    TooManyProducts { reaction: usize, got: usize, max: usize },
    InvalidStoich { reaction: usize },
    IllFormed { reaction: usize },
}

impl From<ron::error::SpannedError> for ReactionNetworkError {
    fn from(e: ron::error::SpannedError) -> Self { Self::Ron(e) }
}

impl core::fmt::Display for ReactionNetworkError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::Ron(e) => write!(f, "ron parse error: {e}"),
            Self::TooManyReactants { reaction, got, max } =>
                write!(f, "reaction {reaction}: {got} reactants > max {max}"),
            Self::TooManyProducts { reaction, got, max } =>
                write!(f, "reaction {reaction}: {got} products > max {max}"),
            Self::InvalidStoich { reaction } =>
                write!(f, "reaction {reaction}: invalid stoichiometry (zero count or out-of-range species)"),
            Self::IllFormed { reaction } =>
                write!(f, "reaction {reaction}: ill-formed (k≤0, no reactants, or no products)"),
        }
    }
}

impl std::error::Error for ReactionNetworkError {}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layers::reaction::SpeciesId;

    const MINIMAL_RAF: &str = r#"(
        reactions: [
            (reactants: [(0,1),(1,1)], products: [(2,1)], k: 1.0, freq: 50.0),
            (reactants: [(2,1)],       products: [(0,1),(3,1)], k: 0.5, freq: 50.0),
            (reactants: [(3,1),(1,1)], products: [(2,1),(1,1)], k: 0.8, freq: 50.0),
        ]
    )"#;

    #[test]
    fn parse_minimal_raf() {
        let net = ReactionNetwork::from_ron_str(MINIMAL_RAF).expect("parse");
        assert_eq!(net.len(), 3);
        assert!(net.reactions().iter().all(Reaction::is_well_formed));
    }

    #[test]
    fn reaction_id_indexes_correctly() {
        let net = ReactionNetwork::from_ron_str(MINIMAL_RAF).unwrap();
        let (id, r0) = net.iter_indexed().next().unwrap();
        assert_eq!(id, ReactionId(0));
        assert_eq!(r0.reactants_active().next().unwrap().species, SpeciesId(0));
    }

    #[test]
    fn rejects_too_many_reactants() {
        let bad = r#"(reactions: [(reactants: [(0,1),(1,1),(2,1),(3,1),(4,1)], products: [(5,1)], k: 1.0, freq: 0.0)])"#;
        assert!(matches!(
            ReactionNetwork::from_ron_str(bad),
            Err(ReactionNetworkError::TooManyReactants { .. })
        ));
    }

    #[test]
    fn rejects_zero_k() {
        let bad = r#"(reactions: [(reactants: [(0,1)], products: [(1,1)], k: 0.0, freq: 0.0)])"#;
        assert!(matches!(
            ReactionNetwork::from_ron_str(bad),
            Err(ReactionNetworkError::IllFormed { .. })
        ));
    }

    #[test]
    fn rejects_out_of_range_species() {
        let bad = r#"(reactions: [(reactants: [(255,1)], products: [(0,1)], k: 1.0, freq: 0.0)])"#;
        assert!(matches!(
            ReactionNetwork::from_ron_str(bad),
            Err(ReactionNetworkError::InvalidStoich { .. })
        ));
    }

    #[test]
    fn empty_network_is_valid() {
        let net = ReactionNetwork::from_ron_str("(reactions: [])").unwrap();
        assert!(net.is_empty());
    }

    #[test]
    fn from_reactions_filters_ill_formed() {
        let good = Reaction {
            reactants: {
                let mut a = [StoichEntry::EMPTY; MAX_REACTANTS_PER_REACTION];
                a[0] = StoichEntry::new(0, 1).unwrap();
                a
            },
            products: {
                let mut a = [StoichEntry::EMPTY; MAX_PRODUCTS_PER_REACTION];
                a[0] = StoichEntry::new(1, 1).unwrap();
                a
            },
            k: 1.0,
            freq: 0.0,
        };
        let bad = Reaction::default();
        let net = ReactionNetwork::from_reactions([good, bad]);
        assert_eq!(net.len(), 1);
    }
}
