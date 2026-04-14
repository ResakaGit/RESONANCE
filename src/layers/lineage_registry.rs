//! AP-4: `LineageRegistry` — árbol genealógico emergente de closures.
//! AP-4: `LineageRegistry` — emergent genealogy tree of closures.
//!
//! **Puente AP-4 → AP-5/6.**  Cada `apply_fission` genera dos `lineage_id`
//! hijos; este Resource los persiste con `(parent, birth_tick)` para que:
//!   - AP-5 pueda asertar invariantes de linaje (hijos distintos de padre).
//!   - AP-6 pueda reconstruir árboles y reportar "supervivientes".
//!
//! **Zero HashMap** (regla repo Hard Block #6): backing = `Vec` ordenada por
//! `lineage_id`, binary-search para lookup.  Inserción O(n) — fisión es rara
//! (≤ pocos eventos por tick), por lo que amortizado es trivial.
//!
//! Axiom 6: el árbol no se programa — cada nodo aparece como consecuencia
//! de un evento físico (fisión) registrado reactivamente.

use bevy::prelude::*;

use crate::blueprint::equations::fission::FissionOutcome;

/// Registro de un linaje individual.  Max 2 campos (data, no Component).
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct LineageRecord {
    /// Linaje padre.  `0` convencional para raíces (sin padre conocido).
    pub parent: u64,
    /// Tick en el que nació el linaje.
    pub birth_tick: u64,
}

/// Resource global: árbol genealógico de todas las closures que han existido.
/// Storage: `Vec` ordenada por `lineage_id` — binary-search para O(log n) lookup.
#[derive(Resource, Clone, Debug, Default)]
pub struct LineageRegistry {
    records: Vec<(u64, LineageRecord)>,
}

impl LineageRegistry {
    pub fn new() -> Self { Self::default() }
    #[inline] pub fn len(&self) -> usize { self.records.len() }
    #[inline] pub fn is_empty(&self) -> bool { self.records.is_empty() }

    /// Registra el nacimiento de `lineage`.  Si ya existe, **no-op** — el primer
    /// registro gana (las fisiones son eventos únicos; re-registrar sería un bug
    /// del caller, no algo a silenciar con mutación).
    pub fn record_birth(&mut self, lineage: u64, parent: u64, birth_tick: u64) {
        match self.records.binary_search_by_key(&lineage, |r| r.0) {
            Ok(_) => {}
            Err(pos) => self.records.insert(
                pos,
                (lineage, LineageRecord { parent, birth_tick }),
            ),
        }
    }

    /// Registra ambos hijos de un `FissionOutcome`.  Conveniencia.
    pub fn record_fission(
        &mut self,
        outcome: &FissionOutcome,
        parent: u64,
        birth_tick: u64,
    ) {
        self.record_birth(outcome.lineage_a, parent, birth_tick);
        self.record_birth(outcome.lineage_b, parent, birth_tick);
    }

    /// Lookup por `lineage_id`.  `None` si no registrado.
    pub fn get(&self, lineage: u64) -> Option<&LineageRecord> {
        self.records
            .binary_search_by_key(&lineage, |r| r.0)
            .ok()
            .map(|i| &self.records[i].1)
    }

    #[inline]
    pub fn parent_of(&self, lineage: u64) -> Option<u64> {
        self.get(lineage).map(|r| r.parent)
    }

    /// `true` si `candidate` es ancestro (transitivo) de `lineage`.  Útil
    /// para invariantes de AP-5 ("ambos hijos comparten ancestro = padre").
    pub fn is_ancestor_of(&self, candidate: u64, lineage: u64) -> bool {
        let mut cur = lineage;
        // Guardia contra ciclos (malformación).  Máximo = #linajes registrados.
        for _ in 0..self.records.len() {
            let Some(p) = self.parent_of(cur) else { return false };
            if p == candidate { return true; }
            if p == 0 { return false; }
            cur = p;
        }
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::blueprint::equations::fission::child_lineage;

    #[test]
    fn new_is_empty() {
        let r = LineageRegistry::new();
        assert!(r.is_empty());
        assert_eq!(r.len(), 0);
        assert!(r.get(42).is_none());
    }

    #[test]
    fn record_birth_persists_lookup() {
        let mut r = LineageRegistry::new();
        r.record_birth(7, 1, 100);
        let rec = r.get(7).expect("lineage 7 should be present");
        assert_eq!(rec.parent, 1);
        assert_eq!(rec.birth_tick, 100);
        assert_eq!(r.parent_of(7), Some(1));
    }

    #[test]
    fn record_birth_is_idempotent_first_wins() {
        let mut r = LineageRegistry::new();
        r.record_birth(7, 1, 100);
        r.record_birth(7, 999, 999); // segundo intento: no-op
        let rec = r.get(7).unwrap();
        assert_eq!(rec.parent, 1);
        assert_eq!(rec.birth_tick, 100);
        assert_eq!(r.len(), 1);
    }

    #[test]
    fn record_fission_persists_both_children_with_shared_parent() {
        let parent = 42_u64;
        let tick = 11_u64;
        let outcome = FissionOutcome {
            cells_a: vec![],
            cells_b: vec![],
            lineage_a: child_lineage(parent, tick, 0),
            lineage_b: child_lineage(parent, tick, 1),
            dissipated_qe: 0.0,
        };
        let mut r = LineageRegistry::new();
        r.record_fission(&outcome, parent, tick);
        assert_eq!(r.len(), 2);
        assert_eq!(r.parent_of(outcome.lineage_a), Some(parent));
        assert_eq!(r.parent_of(outcome.lineage_b), Some(parent));
    }

    #[test]
    fn is_ancestor_of_walks_the_chain() {
        // Cadena: root → gen1 → gen2.
        let mut r = LineageRegistry::new();
        r.record_birth(10, 0, 1);  // root (parent=0)
        r.record_birth(20, 10, 2); // hijo de 10
        r.record_birth(30, 20, 3); // nieto
        assert!(r.is_ancestor_of(10, 30));
        assert!(r.is_ancestor_of(20, 30));
        assert!(r.is_ancestor_of(10, 20));
        assert!(!r.is_ancestor_of(30, 10), "descendant is not ancestor");
        assert!(!r.is_ancestor_of(99, 30), "unknown lineage not ancestor");
    }
}
