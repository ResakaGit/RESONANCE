use bevy::prelude::*;

/// Las 5 funciones primitivas de extracción.
/// Cerrado: no `Box<dyn Trait>`, no trait objects.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Reflect)]
pub enum ExtractionType {
    /// Type I: Fair share (pool / n_siblings).
    Proportional,
    /// Type II: Takes up to capacity limit.
    Greedy,
    /// Type III: Share proportional to relative fitness.
    Competitive,
    /// Type IV: Extracts and damages parent capacity.
    Aggressive,
    /// Type V: Self-regulates based on parent pool state.
    Regulated,
}

/// Vínculo de extracción: esta entidad extrae energía del pool padre.
/// SparseSet: solo entidades en jerarquía activa.
#[derive(Component, Reflect, Debug, Clone, Copy, PartialEq)]
#[reflect(Component)]
#[component(storage = "SparseSet")]
pub struct PoolParentLink {
    parent: Entity,
    extraction_type: ExtractionType,
    primary_param: f32,
}

impl PoolParentLink {
    pub fn new(parent: Entity, extraction_type: ExtractionType, primary_param: f32) -> Self {
        Self { parent, extraction_type, primary_param }
    }

    #[inline]
    pub fn parent(&self) -> Entity { self.parent }
    #[inline]
    pub fn extraction_type(&self) -> ExtractionType { self.extraction_type }
    #[inline]
    pub fn primary_param(&self) -> f32 { self.primary_param }

    pub fn set_parent(&mut self, parent: Entity) { self.parent = parent; }
    pub fn set_extraction_type(&mut self, et: ExtractionType) { self.extraction_type = et; }
    pub fn set_primary_param(&mut self, val: f32) { self.primary_param = val; }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pool_parent_link_is_copy() {
        let a = PoolParentLink::new(
            Entity::from_raw(1),
            ExtractionType::Competitive,
            0.6,
        );
        let b = a;
        assert_eq!(a, b);
    }

    #[test]
    fn getters_return_correct_values() {
        let parent = Entity::from_raw(42);
        let link = PoolParentLink::new(parent, ExtractionType::Greedy, 500.0);
        assert_eq!(link.parent(), parent);
        assert_eq!(link.extraction_type(), ExtractionType::Greedy);
        assert_eq!(link.primary_param(), 500.0);
    }

    #[test]
    fn setters_update_values() {
        let mut link = PoolParentLink::new(
            Entity::from_raw(1),
            ExtractionType::Proportional,
            0.0,
        );
        let new_parent = Entity::from_raw(99);
        link.set_parent(new_parent);
        link.set_extraction_type(ExtractionType::Regulated);
        link.set_primary_param(100.0);
        assert_eq!(link.parent(), new_parent);
        assert_eq!(link.extraction_type(), ExtractionType::Regulated);
        assert_eq!(link.primary_param(), 100.0);
    }

    #[test]
    fn extraction_type_is_copy_eq_hash() {
        let a = ExtractionType::Aggressive;
        let b = a;
        assert_eq!(a, b);

        // Hash trait verified by using in a set
        let mut set = std::collections::HashSet::new();
        set.insert(ExtractionType::Proportional);
        set.insert(ExtractionType::Greedy);
        set.insert(ExtractionType::Competitive);
        set.insert(ExtractionType::Aggressive);
        set.insert(ExtractionType::Regulated);
        assert_eq!(set.len(), 5);
    }

    #[test]
    fn extraction_type_size_is_1_byte() {
        assert_eq!(std::mem::size_of::<ExtractionType>(), 1);
    }

    #[test]
    fn extraction_type_exhaustive_match() {
        // Compile-time: no `_ =>` — compiler catches new variants.
        let check = |et: ExtractionType| -> &str {
            match et {
                ExtractionType::Proportional => "I",
                ExtractionType::Greedy       => "II",
                ExtractionType::Competitive  => "III",
                ExtractionType::Aggressive   => "IV",
                ExtractionType::Regulated    => "V",
            }
        };
        assert_eq!(check(ExtractionType::Proportional), "I");
        assert_eq!(check(ExtractionType::Greedy), "II");
        assert_eq!(check(ExtractionType::Competitive), "III");
        assert_eq!(check(ExtractionType::Aggressive), "IV");
        assert_eq!(check(ExtractionType::Regulated), "V");
    }
}
