//! AP-3: `ClosureMembraneMask` — Resource que marca qué especies forman membrana.
//! AP-3: `ClosureMembraneMask` — Resource flagging membrane-forming species.
//!
//! **Puente AP-1 → AP-3.**  Un detector de closures (AP-1, orquestado por AP-6)
//! escribe aquí los productos de las closures vivas.  El pipeline de difusión
//! (AP-0 + AP-3) consulta la mask para construir el campo de damping por celda.
//!
//! No es componente — es Resource global, consistente con `ReactionNetwork` y
//! `SpeciesGrid` (ambos Resources del mundo químico).  El `system` que lo
//! puebla cada tick se wirea en AP-6 (`autopoietic_lab`), no aquí — el sprint
//! AP-3 sólo define la estructura de datos y su API pura.
//!
//! Axiom 6: membrana emerge; mask es sólo la traducción del "conjunto de
//! productos vivos" al grid — no declara membrana, sólo habilita lectura.

use bevy::prelude::*;

use crate::blueprint::constants::chemistry::MAX_SPECIES;
use crate::blueprint::equations::raf::Closure;
use crate::layers::reaction_network::ReactionNetwork;

/// Máscara global de especies-producto.
/// `mask[s] == true` ⇔ la especie `s` es producto de alguna closure viva y
/// por tanto contribuye a la densidad de membrana.  Vacía ⇒ sin damping.
#[derive(Resource, Clone, Debug)]
pub struct ClosureMembraneMask {
    mask: [bool; MAX_SPECIES],
}

impl Default for ClosureMembraneMask {
    fn default() -> Self { Self { mask: [false; MAX_SPECIES] } }
}

impl ClosureMembraneMask {
    /// Mask vacía — sin closures ⇒ damping = 1 en todo el grid.
    pub fn new() -> Self { Self::default() }

    /// Borra todos los bits.  Llamar al inicio del tick de detección si se
    /// quiere estado instantáneo (no acumulativo) de las closures vivas.
    #[inline]
    pub fn clear(&mut self) { self.mask = [false; MAX_SPECIES]; }

    /// Acceso a la mask como array fijo — pensado para `compute_membrane_field`.
    #[inline]
    pub fn as_array(&self) -> &[bool; MAX_SPECIES] { &self.mask }

    /// `true` si ninguna especie está marcada (⇔ sin membrana activa).
    pub fn is_empty(&self) -> bool { !self.mask.iter().any(|&b| b) }

    /// Cuenta de bits activos — métricas/tests.
    pub fn count(&self) -> usize { self.mask.iter().filter(|&&b| b).count() }

    /// Marca los productos de `closure` como especies-membrana.
    /// `ReactionId` fuera de rango se ignora silenciosamente (la red pudo
    /// haberse re-cargado dejando el hash de la closure obsoleto).
    pub fn mark_closure_products(&mut self, closure: &Closure, network: &ReactionNetwork) {
        for rid in &closure.reactions {
            let Some(rx) = network.get(*rid) else { continue };
            for e in rx.products_active() {
                self.mask[e.species.index()] = true;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::blueprint::equations::raf::raf_closures;
    use crate::layers::reaction::SpeciesId;

    /// RAF mínima (mismo shape que `assets/reactions/raf_minimal.ron`):
    /// food = {A=0, B=1}; productos = {C=2, D=3}.
    ///   r0: A+B → C;  r1: C → A+D;  r2: D+B → B+C.
    fn minimal_raf_network() -> ReactionNetwork {
        let spec = r#"(reactions: [
            (reactants: [(0,1),(1,1)], products: [(2,1)],       k: 1.0, freq: 50.0),
            (reactants: [(2,1)],       products: [(0,1),(3,1)], k: 0.5, freq: 50.0),
            (reactants: [(3,1),(1,1)], products: [(1,1),(2,1)], k: 0.8, freq: 50.0),
        ])"#;
        ReactionNetwork::from_ron_str(spec).unwrap()
    }

    fn food() -> [SpeciesId; 2] {
        [SpeciesId::new(0).unwrap(), SpeciesId::new(1).unwrap()]
    }

    #[test]
    fn default_is_empty() {
        let m = ClosureMembraneMask::default();
        assert!(m.is_empty());
        assert_eq!(m.count(), 0);
    }

    #[test]
    fn mark_closure_products_sets_only_product_species() {
        let net = minimal_raf_network();
        let food = food();
        let closures = raf_closures(&net, &food);
        assert!(!closures.is_empty(), "red minimal debe producir ≥1 closure");

        let mut mask = ClosureMembraneMask::new();
        mask.mark_closure_products(&closures[0], &net);

        // Productos de la RAF: A(0), C(2), D(3), B(1) — B aparece como producto
        // de r2 (B cataliza). La mask refleja *productos aparecidos*, no food
        // fresca: aquí todos cuatro aparecen como productos de alguna reacción.
        let bits = mask.as_array();
        assert!(bits[2] && bits[3], "C y D son productos puros de la closure");
        assert!(mask.count() >= 2);
    }

    #[test]
    fn clear_resets_mask() {
        let net = minimal_raf_network();
        let closures = raf_closures(&net, &food());
        assert!(!closures.is_empty());
        let mut mask = ClosureMembraneMask::new();
        mask.mark_closure_products(&closures[0], &net);
        assert!(!mask.is_empty());
        mask.clear();
        assert!(mask.is_empty());
        assert_eq!(mask.count(), 0);
    }

    #[test]
    fn invalid_reaction_id_is_ignored() {
        use crate::blueprint::equations::raf::Closure;
        use crate::layers::reaction_network::ReactionId;

        let net = minimal_raf_network();
        let mut mask = ClosureMembraneMask::new();
        let stale = Closure {
            reactions: vec![ReactionId(999)],
            species: vec![],
            hash: 0,
        };
        mask.mark_closure_products(&stale, &net); // no panic
        assert!(mask.is_empty());
    }

    #[test]
    fn multiple_closures_union_mask_bits() {
        let net = minimal_raf_network();
        let closures = raf_closures(&net, &food());
        let mut mask = ClosureMembraneMask::new();
        for c in &closures {
            mask.mark_closure_products(c, &net);
        }
        // La red minimal: C(2) y D(3) son productos en la closure.
        assert!(mask.as_array()[2]);
        assert!(mask.as_array()[3]);
    }
}
