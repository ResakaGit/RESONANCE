//! AP-0: Reaction — átomos de la red autocatalítica.
//! AP-0: Reaction — atoms of the autocatalytic network.
//!
//! `SpeciesId` es un `u8` con sentinel (`NONE`) para slots vacíos en arrays fijos.
//! `StoichEntry` empaqueta `(species, stoich_count)` en 2 bytes.
//! `Reaction` es `Copy` de 24 bytes, diseñada para caber en cache y moverse barato.
//!
//! Axiom 1: cada reacción transforma `qe` entre canales (especies).
//! Axiom 4: nunca 100% eficiente (ver `REACTION_EFFICIENCY`).
//! Axiom 8: tasa modulada por alineación de frecuencia con la celda.

use serde::{Deserialize, Serialize};

use crate::blueprint::constants::chemistry::{
    MAX_PRODUCTS_PER_REACTION, MAX_REACTANTS_PER_REACTION, MAX_SPECIES, SPECIES_ID_NONE,
};

// ── SpeciesId ───────────────────────────────────────────────────────────────

/// Identificador de especie química. `u8` con `SpeciesId::NONE` como sentinel.
/// Species identifier.
#[repr(transparent)]
#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug, Ord, PartialOrd, Serialize, Deserialize)]
#[serde(transparent)]
pub struct SpeciesId(pub u8);

impl SpeciesId {
    pub const NONE: Self = Self(SPECIES_ID_NONE);

    /// Construye `SpeciesId` validando rango. `None` si `id >= MAX_SPECIES`.
    #[inline]
    pub const fn new(id: u8) -> Option<Self> {
        if (id as usize) < MAX_SPECIES { Some(Self(id)) } else { None }
    }

    #[inline] pub const fn is_none(self) -> bool { self.0 == SPECIES_ID_NONE }
    #[inline] pub const fn is_some(self) -> bool { !self.is_none() }
    #[inline] pub const fn raw(self) -> u8 { self.0 }

    /// Índice para acceso a arrays `[_; MAX_SPECIES]`.
    /// **Panic** (debug) si la especie es `NONE` — siempre testear `is_some()` antes.
    #[inline]
    pub fn index(self) -> usize {
        debug_assert!(self.is_some(), "SpeciesId::index() on NONE");
        self.0 as usize
    }
}

// ── StoichEntry ─────────────────────────────────────────────────────────────

/// Pareja (especie, coeficiente estequiométrico) empaquetada en 2 bytes.
/// `count == 0` o `species.is_none()` significan "slot vacío".
#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub struct StoichEntry {
    pub species: SpeciesId,
    pub count: u8,
}

impl StoichEntry {
    pub const EMPTY: Self = Self { species: SpeciesId::NONE, count: 0 };

    /// Constructor seguro: `None` si `species_id` fuera de rango o `count == 0`.
    #[inline]
    pub const fn new(species_id: u8, count: u8) -> Option<Self> {
        if count == 0 { return None; }
        match SpeciesId::new(species_id) {
            Some(s) => Some(Self { species: s, count }),
            None => None,
        }
    }

    #[inline]
    pub const fn is_active(self) -> bool {
        self.count > 0 && self.species.is_some()
    }
}

impl Default for StoichEntry {
    fn default() -> Self { Self::EMPTY }
}

// ── Reaction ────────────────────────────────────────────────────────────────

/// Reacción química: reactivos → productos, constante `k` y frecuencia `freq`.
/// `Copy`, 24 bytes. Orden de slots en arrays es irrelevante — `is_active()` filtra.
#[derive(Copy, Clone, Debug, Serialize, Deserialize)]
pub struct Reaction {
    pub reactants: [StoichEntry; MAX_REACTANTS_PER_REACTION],
    pub products: [StoichEntry; MAX_PRODUCTS_PER_REACTION],
    /// Constante cinética (mass-action). `> 0`.
    pub k: f32,
    /// Frecuencia resonante de la reacción (Hz). Catálisis frequency-aligned.
    pub freq: f32,
}

impl Reaction {
    /// Itera reactivos activos (filtrando slots vacíos).
    #[inline]
    pub fn reactants_active(&self) -> impl Iterator<Item = StoichEntry> + '_ {
        self.reactants.iter().copied().filter(|e| e.is_active())
    }

    /// Itera productos activos.
    #[inline]
    pub fn products_active(&self) -> impl Iterator<Item = StoichEntry> + '_ {
        self.products.iter().copied().filter(|e| e.is_active())
    }

    /// Reaccion válida: al menos un reactivo, al menos un producto, `k > 0`.
    pub fn is_well_formed(&self) -> bool {
        self.k.is_finite()
            && self.k > 0.0
            && self.freq.is_finite()
            && self.reactants_active().next().is_some()
            && self.products_active().next().is_some()
    }
}

impl Default for Reaction {
    fn default() -> Self {
        Self {
            reactants: [StoichEntry::EMPTY; MAX_REACTANTS_PER_REACTION],
            products: [StoichEntry::EMPTY; MAX_PRODUCTS_PER_REACTION],
            k: 0.0,
            freq: 0.0,
        }
    }
}

// Compile-time layout guard: Reaction must stay cheap to Copy.
const _: () = assert!(core::mem::size_of::<Reaction>() <= 32, "Reaction ≤ 32 bytes");
const _: () = assert!(core::mem::size_of::<StoichEntry>() == 2, "StoichEntry == 2 bytes");

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn species_id_none_sentinel() {
        let n = SpeciesId::NONE;
        assert!(n.is_none());
        assert!(!n.is_some());
    }

    #[test]
    fn species_id_valid_range() {
        assert!(SpeciesId::new(0).is_some());
        assert!(SpeciesId::new((MAX_SPECIES - 1) as u8).is_some());
        assert!(SpeciesId::new(MAX_SPECIES as u8).is_none());
        assert!(SpeciesId::new(SPECIES_ID_NONE).is_none());
    }

    #[test]
    fn stoich_entry_empty_inactive() {
        assert!(!StoichEntry::EMPTY.is_active());
        assert_eq!(StoichEntry::default(), StoichEntry::EMPTY);
    }

    #[test]
    fn stoich_entry_new_rejects_zero_count() {
        assert!(StoichEntry::new(0, 0).is_none());
        assert!(StoichEntry::new(0, 1).is_some());
    }

    #[test]
    fn stoich_entry_new_rejects_out_of_range_species() {
        assert!(StoichEntry::new(SPECIES_ID_NONE, 1).is_none());
    }

    #[test]
    fn reaction_default_not_well_formed() {
        assert!(!Reaction::default().is_well_formed());
    }

    #[test]
    fn reaction_well_formed_requires_k_positive() {
        let mut r = Reaction::default();
        r.reactants[0] = StoichEntry::new(0, 1).unwrap();
        r.products[0] = StoichEntry::new(1, 1).unwrap();
        r.k = 0.0;
        assert!(!r.is_well_formed());
        r.k = 1.0;
        assert!(r.is_well_formed());
    }

    #[test]
    fn reaction_active_iterators_skip_empty_slots() {
        let mut r = Reaction::default();
        r.reactants[0] = StoichEntry::new(0, 1).unwrap();
        r.reactants[2] = StoichEntry::new(3, 2).unwrap();
        r.products[1] = StoichEntry::new(5, 1).unwrap();
        r.k = 1.0;
        assert_eq!(r.reactants_active().count(), 2);
        assert_eq!(r.products_active().count(), 1);
    }

    #[test]
    fn reaction_fits_in_cache_line_pair() {
        assert!(core::mem::size_of::<Reaction>() <= 32);
    }

    #[test]
    fn serde_roundtrip_species_id() {
        let s = SpeciesId(7);
        let txt = ron::to_string(&s).unwrap();
        // `#[serde(transparent)]` → se serializa como el u8 interno.
        assert_eq!(txt, "7");
        let back: SpeciesId = ron::from_str(&txt).unwrap();
        assert_eq!(back, s);
    }
}
