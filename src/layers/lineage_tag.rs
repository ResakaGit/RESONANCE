//! AI-2 (ADR-044): `LineageTag` — identidad de linaje per-entity.
//! AI-2 (ADR-044): `LineageTag` — per-entity lineage identity.
//!
//! Componente que asocia una entity ECS con su linaje en el árbol genealógico
//! AP-* (`hash_to_lineage` / `child_lineage` en `equations/fission.rs`).
//!
//! Stateless · Copy · 8 bytes.  Sin lógica más allá de almacenar el u64.
//! Queryable: `Query<(Entity, &LineageTag)>` para cualquier filtro genealógico.

use bevy::prelude::*;

/// Tag de linaje heredado de un evento de fisión AP-*.
///
/// `0` reservado para "sopa primordial sin linaje asignado" (convención
/// `apply_fission` / `hash_to_lineage` en ADR-039 §5 + ADR-041 §3).
/// El spawner `on_fission_spawn_entity` rechaza eventos con ambos children=0.
#[derive(Component, Reflect, Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[reflect(Component)]
pub struct LineageTag(pub u64);

impl LineageTag {
    /// Constructor explícito.  No clamping — el caller es responsable de no
    /// pasar `0` para entities reales.
    #[inline]
    pub const fn new(id: u64) -> Self { Self(id) }

    /// `true` ⇔ pre-linaje (sopa primordial).  No corresponde a una entity
    /// concreta — usar como filtro defensivo.
    #[inline]
    pub const fn is_primordial(self) -> bool { self.0 == 0 }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn primordial_is_zero() {
        assert!(LineageTag::new(0).is_primordial());
        assert!(!LineageTag::new(1).is_primordial());
        assert!(!LineageTag::new(u64::MAX).is_primordial());
    }

    #[test]
    fn equality_and_hash_by_value() {
        use std::collections::HashSet;
        let mut s = HashSet::new();
        s.insert(LineageTag::new(42));
        s.insert(LineageTag::new(42));
        s.insert(LineageTag::new(43));
        assert_eq!(s.len(), 2);
    }
}
