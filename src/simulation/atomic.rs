use bevy::prelude::{Entity, Query};

use crate::blueprint::{AlchemicalAlmanac, ElementId};
use crate::layers::{BaseEnergy, OscillatorySignature};

/// Vista atómica efímera para inferencia química sin estado cacheado.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MatterProjection {
    StableBand(ElementId),
    Unstable,
}

/// Adapter O(1): proyecta el "tipo de materia" de una entidad en este tick.
pub struct MatterLense;

impl MatterLense {
    pub fn project(
        energy: &BaseEnergy,
        sig: &OscillatorySignature,
        almanac: &AlchemicalAlmanac,
    ) -> MatterProjection {
        if energy.qe() <= 0.0 {
            return MatterProjection::Unstable;
        }
        let projection = almanac
            .find_stable_band_id(sig.frequency_hz())
            .map(MatterProjection::StableBand)
            .unwrap_or(MatterProjection::Unstable);
        projection
    }

    pub fn project_entity(
        entity: Entity,
        energies: &Query<&BaseEnergy>,
        oscillatory: &Query<&OscillatorySignature>,
        almanac: &AlchemicalAlmanac,
    ) -> Option<MatterProjection> {
        let energy = energies.get(entity).ok()?;
        let sig = oscillatory.get(entity).ok()?;
        Some(Self::project(energy, sig, almanac))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::blueprint::ElementDef;
    use crate::layers::MatterState;

    #[test]
    fn matter_lense_projects_stable_band_when_frequency_matches() {
        let almanac = AlchemicalAlmanac::from_defs(vec![ElementDef {
            name: "Testium".to_string(),
            symbol: "Testium".to_string(),
            atomic_number: 1,
            frequency_hz: 100.0,
            freq_band: (90.0, 110.0),
            bond_energy: 10.0,
            conductivity: 0.5,
            visibility: 0.5,
            matter_state: MatterState::Solid,
            electronegativity: 1.0,
            ionization_ev: 1.0,
            color: (0.2, 0.2, 0.2),
            is_compound: false,
            phenology: None,
            hz_identity_weight: crate::blueprint::FIELD_EAC4_HZ_IDENTITY_WEIGHT_RON_ONLY,
        }]);
        let projection = MatterLense::project(
            &BaseEnergy::new(10.0),
            &OscillatorySignature::new(100.0, 0.0),
            &almanac,
        );
        assert!(matches!(projection, MatterProjection::StableBand(_)));
    }

    #[test]
    fn matter_lense_returns_unstable_when_energy_is_zero() {
        let almanac = AlchemicalAlmanac::default();
        let projection = MatterLense::project(
            &BaseEnergy::new(0.0),
            &OscillatorySignature::new(100.0, 0.0),
            &almanac,
        );
        assert_eq!(projection, MatterProjection::Unstable);
    }
}
