use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use crate::blueprint::ElementId;

// ---------------------------------------------------------------------------
// WorldgenReady — señaliza que warmup completó (DC-3)
// ---------------------------------------------------------------------------

/// Señaliza que worldgen completó su warmup y el mundo está materializado.
/// Insertada por worldgen_warmup_system. Leída por simulation para transicionar.
///
/// Signals that worldgen warmup completed and the world is materialized.
#[derive(Resource, Debug, Default)]
pub struct WorldgenReady {
    /// Tick en el que se completó el warmup.
    pub completed_at_tick: u64,
}
use crate::eco::contracts::{TransitionType, ZoneClass};
use crate::layers::MatterState;
use crate::worldgen::archetypes::WorldArchetype;
use crate::worldgen::constants::{MAX_FREQUENCY_CONTRIBUTIONS, MIN_CONTRIBUTION_INTENSITY};

fn placeholder_entity() -> Entity {
    Entity::PLACEHOLDER
}

/// Contribución frecuencial de una fuente hacia una celda del grid.
#[derive(Clone, Copy, Debug, Reflect, PartialEq, Serialize, Deserialize)]
pub struct FrequencyContribution {
    /// Entidad fuente que emitió la contribución.
    #[serde(skip, default = "placeholder_entity")]
    pub(crate) source_entity: Entity,
    /// Frecuencia emitida [Hz].
    pub(crate) frequency_hz: f32,
    /// Intensidad aportada [qe].
    pub(crate) intensity_qe: f32,
}

impl FrequencyContribution {
    pub fn new(source_entity: Entity, frequency_hz: f32, intensity_qe: f32) -> Self {
        let frequency_hz = if frequency_hz.is_finite() {
            frequency_hz.max(0.0)
        } else {
            0.0
        };
        let intensity_qe = if intensity_qe.is_finite() {
            intensity_qe.max(0.0)
        } else {
            0.0
        };
        Self {
            source_entity,
            frequency_hz,
            intensity_qe,
        }
    }

    #[inline]
    pub fn source_entity(&self) -> Entity {
        self.source_entity
    }

    #[inline]
    pub fn frequency_hz(&self) -> f32 {
        self.frequency_hz
    }

    #[inline]
    pub fn intensity_qe(&self) -> f32 {
        self.intensity_qe
    }
}

/// Las dos contribuciones de mayor intensidad (empate estable por índice de `source_entity`).
pub fn top_two(
    contributions: &[FrequencyContribution],
) -> Option<(FrequencyContribution, FrequencyContribution)> {
    let mut ranked = Vec::with_capacity(MAX_FREQUENCY_CONTRIBUTIONS);
    for c in contributions.iter().copied() {
        if c.intensity_qe.is_finite() && c.intensity_qe > 0.0 {
            ranked.push(c);
        }
    }
    if ranked.len() < 2 {
        return None;
    }
    ranked.sort_by(|a, b| {
        b.intensity_qe
            .total_cmp(&a.intensity_qe)
            .then_with(|| a.source_entity.index().cmp(&b.source_entity.index()))
    });
    Some((ranked[0], ranked[1]))
}

/// Estado energético resumido de una celda de campo.
#[derive(Clone, Debug, Reflect, Serialize, Deserialize)]
pub struct EnergyCell {
    /// Energía acumulada en la celda [qe].
    pub accumulated_qe: f32,
    /// Contribuciones activas por frecuencia (acotadas por constante).
    pub frequency_contributions: Vec<FrequencyContribution>,
    /// Frecuencia dominante resultante [Hz].
    pub dominant_frequency_hz: f32,
    /// Pureza en [0, 1]: dominio de una sola frecuencia.
    pub purity: f32,
    /// Temperatura equivalente derivada de la densidad.
    pub temperature: f32,
    /// Estado de materia derivado.
    pub matter_state: MatterState,
    /// Entidad materializada en esta celda; limpiar si la entidad ya no existe.
    #[serde(skip, default)]
    pub materialized_entity: Option<Entity>,
}

impl Default for EnergyCell {
    fn default() -> Self {
        Self {
            accumulated_qe: 0.0,
            frequency_contributions: Vec::with_capacity(MAX_FREQUENCY_CONTRIBUTIONS),
            dominant_frequency_hz: 0.0,
            purity: 0.0,
            temperature: 0.0,
            matter_state: MatterState::Solid,
            materialized_entity: None,
        }
    }
}

impl EnergyCell {
    /// Vista de solo lectura de contribuciones guardadas.
    pub fn frequency_contributions(&self) -> &[FrequencyContribution] {
        &self.frequency_contributions
    }

    /// Capacidad reservada para contribuciones.
    pub fn frequency_capacity(&self) -> usize {
        self.frequency_contributions.capacity()
    }

    /// Inserta una contribución relevante con capacidad acotada.
    ///
    /// Si la celda ya está llena, reemplaza la contribución más débil
    /// solo cuando la nueva tiene mayor intensidad.
    pub fn push_contribution_bounded(&mut self, contribution: FrequencyContribution) {
        if !contribution.intensity_qe.is_finite()
            || contribution.intensity_qe < MIN_CONTRIBUTION_INTENSITY
        {
            return;
        }

        if self.frequency_contributions.len() < MAX_FREQUENCY_CONTRIBUTIONS {
            self.frequency_contributions.push(contribution);
            return;
        }

        if let Some((idx, weakest)) = self
            .frequency_contributions
            .iter()
            .enumerate()
            .min_by(|(_, a), (_, b)| a.intensity_qe.total_cmp(&b.intensity_qe))
            && contribution.intensity_qe > weakest.intensity_qe
        {
            self.frequency_contributions[idx] = contribution;
        }
    }
}

/// Resultado puro de **forma** (WorldArchetype): color/escala/emisión viven en `EnergyVisual` vía `visual_derivation`.
#[derive(Clone, Debug, Reflect)]
pub struct MaterializationResult {
    pub archetype: WorldArchetype,
}

/// Marca entidades creadas por materialización del grid.
#[derive(Component, Clone, Copy, Debug, Reflect)]
#[reflect(Component)]
pub struct Materialized {
    pub cell_x: i32,
    pub cell_y: i32,
    pub archetype: WorldArchetype,
}

/// Celda en frontera ecológica (Eco-Boundaries): hints para render sin entidades extra.
#[derive(Component, Clone, Copy, Debug, PartialEq, Reflect)]
#[reflect(Component, PartialEq)]
pub struct BoundaryVisual {
    pub transition_type: TransitionType,
    pub gradient_factor: f32,
    pub zone_a: ZoneClass,
    pub zone_b: ZoneClass,
}

/// Marca rebuild prioritario de `EnergyVisual` tras invalidación en `materialization_delta_system`
/// (misma frame de sim, antes de `Update`, sin consumir presupuesto visual).
#[derive(Component, Clone, Copy, Debug, Default, Reflect)]
#[reflect(Component)]
pub struct PendingEnergyVisualRebuild;

/// Propiedades visuales derivadas de energía para el renderer.
#[derive(Component, Clone, Debug, Reflect, PartialEq)]
#[reflect(Component, PartialEq)]
pub struct EnergyVisual {
    pub color: Color,
    pub scale: f32,
    pub emission: f32,
    pub opacity: f32,
}

/// Parámetros para fenología visual (EA8): referencia al elemento en almanaque + techos de normalización.
#[derive(Component, Clone, Copy, Debug, Reflect, PartialEq)]
#[reflect(Component, PartialEq)]
#[component(storage = "SparseSet")]
pub struct PhenologyVisualParams {
    pub element_id: ElementId,
    /// Techo para `growth_t` (biomasa o `accumulated_qe` proxy).
    pub growth_norm_ceiling: f32,
    pub qe_reference: f32,
    pub epsilon: f32,
}

/// Fase previa para histeresis (evita flicker).
#[derive(Component, Clone, Copy, Debug, Default, Reflect, PartialEq)]
#[reflect(Component, PartialEq)]
#[component(storage = "SparseSet")]
pub struct PhenologyPhaseCache {
    pub prev_phase: f32,
}

#[cfg(test)]
mod tests {
    use super::{EnergyCell, FrequencyContribution};
    use crate::worldgen::constants::{MAX_FREQUENCY_CONTRIBUTIONS, MIN_CONTRIBUTION_INTENSITY};
    use bevy::prelude::Entity;

    #[test]
    fn frequency_contribution_negative_intensity_is_clamped_to_zero() {
        let contribution = FrequencyContribution::new(Entity::from_raw(1), 100.0, -42.0);
        assert_eq!(contribution.intensity_qe(), 0.0);
    }

    #[test]
    fn energy_cell_default_reserves_expected_capacity() {
        let cell = EnergyCell::default();
        assert!(cell.frequency_capacity() >= MAX_FREQUENCY_CONTRIBUTIONS);
    }

    #[test]
    fn energy_cell_discards_tiny_contribution() {
        let mut cell = EnergyCell::default();
        let tiny = FrequencyContribution::new(
            Entity::from_raw(1),
            100.0,
            MIN_CONTRIBUTION_INTENSITY / 2.0,
        );
        cell.push_contribution_bounded(tiny);
        assert!(cell.frequency_contributions().is_empty());
    }

    #[test]
    fn energy_cell_keeps_max_bounded_contributions() {
        let mut cell = EnergyCell::default();
        for i in 0..(MAX_FREQUENCY_CONTRIBUTIONS + 3) {
            let contribution = FrequencyContribution::new(
                Entity::from_raw(i as u32 + 1),
                100.0 + i as f32,
                1.0 + i as f32,
            );
            cell.push_contribution_bounded(contribution);
        }
        assert_eq!(
            cell.frequency_contributions().len(),
            MAX_FREQUENCY_CONTRIBUTIONS
        );
    }

    #[test]
    fn frequency_contribution_non_finite_values_are_zeroed() {
        let contribution = FrequencyContribution::new(Entity::from_raw(1), f32::NAN, f32::INFINITY);
        assert_eq!(contribution.frequency_hz(), 0.0);
        assert_eq!(contribution.intensity_qe(), 0.0);
    }

    #[test]
    fn frequency_contribution_negative_frequency_is_clamped_to_zero() {
        let contribution = FrequencyContribution::new(Entity::from_raw(7), -440.0, 3.0);
        assert_eq!(contribution.frequency_hz, 0.0);
        assert_eq!(contribution.intensity_qe, 3.0);
    }

    #[test]
    fn energy_cell_replaces_weakest_when_full_and_new_is_stronger() {
        let mut cell = EnergyCell::default();
        for i in 0..MAX_FREQUENCY_CONTRIBUTIONS {
            let contribution = FrequencyContribution::new(
                Entity::from_raw((i + 1) as u32),
                100.0 + i as f32,
                (i + 1) as f32,
            );
            cell.push_contribution_bounded(contribution);
        }

        cell.push_contribution_bounded(FrequencyContribution::new(
            Entity::from_raw(999),
            777.0,
            999.0,
        ));

        assert_eq!(
            cell.frequency_contributions().len(),
            MAX_FREQUENCY_CONTRIBUTIONS
        );
        assert!(
            cell.frequency_contributions()
                .iter()
                .any(|entry| entry.source_entity() == Entity::from_raw(999))
        );
        assert!(
            !cell
                .frequency_contributions()
                .iter()
                .any(|entry| (entry.intensity_qe() - 1.0).abs() < f32::EPSILON)
        );
    }

    #[test]
    fn energy_cell_does_not_replace_when_full_and_new_is_weaker_or_equal() {
        let mut cell = EnergyCell::default();
        for i in 0..MAX_FREQUENCY_CONTRIBUTIONS {
            let contribution = FrequencyContribution::new(
                Entity::from_raw((i + 1) as u32),
                100.0 + i as f32,
                (i + 1) as f32,
            );
            cell.push_contribution_bounded(contribution);
        }

        let before = cell.frequency_contributions().to_vec();
        cell.push_contribution_bounded(FrequencyContribution::new(
            Entity::from_raw(500),
            700.0,
            0.5,
        ));
        cell.push_contribution_bounded(FrequencyContribution::new(
            Entity::from_raw(501),
            701.0,
            1.0,
        ));

        assert_eq!(cell.frequency_contributions(), before.as_slice());
    }

    #[test]
    fn energy_cell_discards_non_finite_contributions() {
        let mut cell = EnergyCell::default();
        cell.push_contribution_bounded(FrequencyContribution::new(
            Entity::from_raw(1),
            100.0,
            f32::NAN,
        ));
        cell.push_contribution_bounded(FrequencyContribution::new(
            Entity::from_raw(2),
            100.0,
            f32::INFINITY,
        ));
        cell.push_contribution_bounded(FrequencyContribution::new(
            Entity::from_raw(3),
            100.0,
            f32::NEG_INFINITY,
        ));
        assert!(cell.frequency_contributions().is_empty());
    }

    #[test]
    fn energy_cell_accepts_contribution_exactly_at_min_threshold() {
        let mut cell = EnergyCell::default();
        let edge =
            FrequencyContribution::new(Entity::from_raw(1), 100.0, MIN_CONTRIBUTION_INTENSITY);
        cell.push_contribution_bounded(edge);
        assert_eq!(cell.frequency_contributions().len(), 1);
    }

    #[test]
    fn energy_cell_clone_has_no_shared_mutable_state() {
        let mut original = EnergyCell::default();
        original.push_contribution_bounded(FrequencyContribution::new(
            Entity::from_raw(1),
            100.0,
            1.0,
        ));

        let mut cloned = original.clone();
        cloned.push_contribution_bounded(FrequencyContribution::new(
            Entity::from_raw(2),
            200.0,
            2.0,
        ));

        assert_eq!(original.frequency_contributions().len(), 1);
        assert_eq!(cloned.frequency_contributions().len(), 2);
    }

    #[test]
    fn frequency_contribution_new_preserves_entity_and_frequency() {
        let contribution = FrequencyContribution::new(Entity::from_raw(42), 432.1, 12.0);
        assert_eq!(contribution.source_entity(), Entity::from_raw(42));
        assert_eq!(contribution.frequency_hz(), 432.1);
        assert_eq!(contribution.intensity_qe(), 12.0);
    }
}
