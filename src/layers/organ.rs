use bevy::prelude::*;

use crate::blueprint::equations::BranchRole;

/// Cantidad máxima de órganos inferidos por entidad en un tick.
pub const MAX_ORGANS_PER_ENTITY: usize = 16;

/// Rol funcional de un órgano inferido; no es un componente ECS.
#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Default, Reflect)]
pub enum OrganRole {
    #[default]
    Stem = 0,
    Root = 1,
    Core = 2,
    Leaf = 3,
    Petal = 4,
    Sensory = 5,
    Thorn = 6,
    Shell = 7,
    Fruit = 8,
    Bud = 9,
    Limb = 10,
    Fin = 11,
}

/// Fase funcional del ciclo de vida inferido.
#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Default, Reflect)]
pub enum LifecycleStage {
    #[default]
    Dormant = 0,
    Emerging = 1,
    Growing = 2,
    Mature = 3,
    Reproductive = 4,
    Declining = 5,
}

/// Clase geométrica base para sintetizar órganos.
#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Default, Reflect)]
pub enum GeometryPrimitive {
    #[default]
    Tube = 0,
    FlatSurface = 1,
    PetalFan = 2,
    Bulb = 3,
}

/// Cache de fase de ciclo de vida para evitar flickeo entre ticks.
#[derive(Component, Clone, Copy, Debug, Default, Reflect, PartialEq, Eq)]
#[reflect(Component, PartialEq)]
#[component(storage = "SparseSet")]
pub struct LifecycleStageCache {
    pub stage: LifecycleStage,
    pub ticks_in_stage: u16,
    pub candidate_stage: Option<LifecycleStage>,
    pub candidate_ticks: u16,
}

/// Especificación de órgano inferido y efímero (se consume para geometría).
#[derive(Clone, Copy, Debug, PartialEq, Reflect)]
pub struct OrganSpec {
    role: OrganRole,
    primitive: GeometryPrimitive,
    count: u8,
    scale_factor: f32,
}

impl OrganSpec {
    /// Constructor canónico: la primitiva siempre deriva del rol.
    pub fn new(role: OrganRole, count: u8, scale_factor: f32) -> Self {
        let count = if count == 0 { 1 } else { count };
        let scale_factor = if scale_factor.is_finite() {
            scale_factor.max(0.0)
        } else {
            0.0
        };
        Self {
            role,
            primitive: role.primitive(),
            count,
            scale_factor,
        }
    }

    #[inline]
    pub fn role(self) -> OrganRole {
        self.role
    }

    #[inline]
    pub fn primitive(self) -> GeometryPrimitive {
        self.primitive
    }

    #[inline]
    pub fn count(self) -> u8 {
        self.count
    }

    #[inline]
    pub fn scale_factor(self) -> f32 {
        self.scale_factor
    }
}

/// Manifesto efímero de órganos inferidos para una entidad.
#[derive(Clone, Debug, PartialEq, Reflect)]
pub struct OrganManifest {
    organs: [OrganSpec; MAX_ORGANS_PER_ENTITY],
    len: usize,
    stage: LifecycleStage,
}

impl Default for OrganManifest {
    fn default() -> Self {
        let default_spec = OrganSpec::new(OrganRole::Stem, 1, 0.0);
        Self {
            organs: [default_spec; MAX_ORGANS_PER_ENTITY],
            len: 0,
            stage: LifecycleStage::Dormant,
        }
    }
}

impl OrganManifest {
    #[inline]
    pub fn new(stage: LifecycleStage) -> Self {
        Self {
            stage,
            ..Self::default()
        }
    }

    #[inline]
    pub fn push(&mut self, spec: OrganSpec) -> bool {
        if self.len >= MAX_ORGANS_PER_ENTITY {
            return false;
        }
        self.organs[self.len] = spec;
        self.len += 1;
        true
    }

    #[inline]
    pub fn as_slice(&self) -> &[OrganSpec] {
        &self.organs[..self.len]
    }

    #[inline]
    pub fn iter(&self) -> impl Iterator<Item = &OrganSpec> {
        self.as_slice().iter()
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.len
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    #[inline]
    pub fn clear(&mut self) {
        self.len = 0;
    }

    #[inline]
    pub fn stage(&self) -> LifecycleStage {
        self.stage
    }

    #[inline]
    pub fn set_stage(&mut self, stage: LifecycleStage) {
        self.stage = stage;
    }
}

/// Tabla de mapeo `OrganRole -> GeometryPrimitive` sin branching en hot path.
pub const ORGAN_ROLE_PRIMITIVE: [GeometryPrimitive; OrganRole::COUNT] = [
    GeometryPrimitive::Tube,
    GeometryPrimitive::Tube,
    GeometryPrimitive::Tube,
    GeometryPrimitive::FlatSurface,
    GeometryPrimitive::PetalFan,
    GeometryPrimitive::Bulb,
    GeometryPrimitive::Tube,
    GeometryPrimitive::FlatSurface,
    GeometryPrimitive::Bulb,
    GeometryPrimitive::Bulb,
    GeometryPrimitive::Tube,
    GeometryPrimitive::FlatSurface,
];

impl OrganRole {
    pub const COUNT: usize = OrganRole::Fin as usize + 1;

    #[inline]
    pub const fn primitive(self) -> GeometryPrimitive {
        ORGAN_ROLE_PRIMITIVE[self as usize]
    }
}

const _: () = assert!(ORGAN_ROLE_PRIMITIVE.len() == OrganRole::COUNT);

impl From<BranchRole> for OrganRole {
    fn from(value: BranchRole) -> Self {
        match value {
            BranchRole::Stem => Self::Stem,
            BranchRole::Leaf => Self::Leaf,
            BranchRole::Thorn => Self::Thorn,
        }
    }
}

impl TryFrom<OrganRole> for BranchRole {
    type Error = &'static str;

    fn try_from(value: OrganRole) -> Result<Self, Self::Error> {
        match value {
            OrganRole::Stem => Ok(BranchRole::Stem),
            OrganRole::Leaf => Ok(BranchRole::Leaf),
            OrganRole::Thorn => Ok(BranchRole::Thorn),
            _ => Err("organ role has no BranchRole equivalent"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{
        GeometryPrimitive, LifecycleStage, OrganManifest, OrganRole, OrganSpec, MAX_ORGANS_PER_ENTITY,
        ORGAN_ROLE_PRIMITIVE,
    };
    use crate::blueprint::equations::BranchRole;

    #[test]
    fn organ_role_covers_twelve_variants() {
        assert_eq!(OrganRole::COUNT, 12);
    }

    #[test]
    fn branch_role_round_trip_for_compatible_roles() {
        let role = OrganRole::from(BranchRole::Stem);
        assert_eq!(BranchRole::try_from(role), Ok(BranchRole::Stem));

        let role = OrganRole::from(BranchRole::Leaf);
        assert_eq!(BranchRole::try_from(role), Ok(BranchRole::Leaf));

        let role = OrganRole::from(BranchRole::Thorn);
        assert_eq!(BranchRole::try_from(role), Ok(BranchRole::Thorn));
    }

    #[test]
    fn lifecycle_stage_order_is_coherent() {
        assert!((LifecycleStage::Dormant as u8) < (LifecycleStage::Emerging as u8));
        assert!((LifecycleStage::Emerging as u8) < (LifecycleStage::Growing as u8));
        assert!((LifecycleStage::Growing as u8) < (LifecycleStage::Mature as u8));
        assert!((LifecycleStage::Mature as u8) < (LifecycleStage::Reproductive as u8));
        assert!((LifecycleStage::Reproductive as u8) < (LifecycleStage::Declining as u8));
    }

    #[test]
    fn role_primitive_table_covers_every_role() {
        assert_eq!(ORGAN_ROLE_PRIMITIVE.len(), OrganRole::COUNT);
        assert_eq!(OrganRole::Stem.primitive(), GeometryPrimitive::Tube);
        assert_eq!(OrganRole::Root.primitive(), GeometryPrimitive::Tube);
        assert_eq!(OrganRole::Core.primitive(), GeometryPrimitive::Tube);
        assert_eq!(OrganRole::Leaf.primitive(), GeometryPrimitive::FlatSurface);
        assert_eq!(OrganRole::Petal.primitive(), GeometryPrimitive::PetalFan);
        assert_eq!(OrganRole::Sensory.primitive(), GeometryPrimitive::Bulb);
        assert_eq!(OrganRole::Thorn.primitive(), GeometryPrimitive::Tube);
        assert_eq!(OrganRole::Shell.primitive(), GeometryPrimitive::FlatSurface);
        assert_eq!(OrganRole::Fruit.primitive(), GeometryPrimitive::Bulb);
        assert_eq!(OrganRole::Bud.primitive(), GeometryPrimitive::Bulb);
        assert_eq!(OrganRole::Limb.primitive(), GeometryPrimitive::Tube);
        assert_eq!(OrganRole::Fin.primitive(), GeometryPrimitive::FlatSurface);
    }

    #[test]
    fn branch_role_try_from_non_legacy_roles_returns_err() {
        assert!(BranchRole::try_from(OrganRole::Root).is_err());
        assert!(BranchRole::try_from(OrganRole::Petal).is_err());
        assert!(BranchRole::try_from(OrganRole::Limb).is_err());
    }

    #[test]
    fn organ_manifest_accepts_up_to_max_organs() {
        let mut manifest = OrganManifest::default();
        for _ in 0..MAX_ORGANS_PER_ENTITY {
            assert!(manifest.push(OrganSpec::new(OrganRole::Leaf, 1, 0.7)));
        }
        assert!(!manifest.push(OrganSpec::new(OrganRole::Bud, 1, 0.2)));
        assert_eq!(manifest.len(), MAX_ORGANS_PER_ENTITY);
    }

    #[test]
    fn organ_manifest_as_slice_tracks_len_and_insertion_order() {
        let mut manifest = OrganManifest::default();
        assert!(manifest.is_empty());
        assert!(manifest.push(OrganSpec::new(OrganRole::Stem, 1, 1.0)));
        assert!(manifest.push(OrganSpec::new(OrganRole::Leaf, 2, 0.5)));
        let slice = manifest.as_slice();
        assert_eq!(slice.len(), 2);
        assert_eq!(slice[0].role(), OrganRole::Stem);
        assert_eq!(slice[1].role(), OrganRole::Leaf);
        manifest.clear();
        assert!(manifest.is_empty());
    }

    #[test]
    fn organ_spec_new_derives_primitive_from_role() {
        let spec = OrganSpec::new(OrganRole::Petal, 0, f32::NAN);
        assert_eq!(spec.primitive(), GeometryPrimitive::PetalFan);
        assert_eq!(spec.count(), 1);
        assert_eq!(spec.scale_factor(), 0.0);
    }

    #[test]
    fn organ_manifest_new_and_stage_setter_work() {
        let mut manifest = OrganManifest::new(LifecycleStage::Growing);
        assert_eq!(manifest.stage(), LifecycleStage::Growing);
        manifest.set_stage(LifecycleStage::Mature);
        assert_eq!(manifest.stage(), LifecycleStage::Mature);
        assert_eq!(manifest.iter().count(), 0);
    }
}
