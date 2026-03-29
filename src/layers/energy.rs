use bevy::ecs::system::SystemParam;
use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use crate::blueprint::constants::{DEFAULT_BASE_ENERGY, QE_MIN_EXISTENCE};
use crate::events::{DeathCause, DeathEvent};

/// Capa 0: Magnitud Base — El Cuanto
/// Layer 0: Base Magnitude — The Quantum
///
/// La existencia pura. Define cuánta "sustancia" hay antes de darle forma o comportamiento.
/// Pure existence. Defines how much "substance" there is before shape or behavior.
///
/// Es el HP termodinámico: cuando `qe` llega a 0, la entidad se disipa o muere.
/// Thermodynamic HP: when `qe` reaches 0, the entity dissipates or dies.
///
/// Invariante / Invariant: qe >= 0.0 (clamped in every system that modifies it)
#[derive(Component, Reflect, Debug, Clone, Serialize, Deserialize)]
#[reflect(Component)]
pub struct BaseEnergy {
    /// Quanta de energía (Joules mágicos). Unidad fundamental de transferencia.
    pub(crate) qe: f32,
}

impl Default for BaseEnergy {
    fn default() -> Self {
        Self {
            qe: DEFAULT_BASE_ENERGY,
        }
    }
}

impl BaseEnergy {
    #[inline]
    pub fn qe(&self) -> f32 {
        self.qe
    }

    pub fn new(qe: f32) -> Self {
        Self { qe: qe.max(0.0) }
    }

    /// Drena energía, nunca por debajo de 0.
    pub fn drain(&mut self, amount: f32) -> f32 {
        let drained = amount.min(self.qe);
        self.qe -= drained;
        drained
    }

    /// Inyecta energía.
    pub fn inject(&mut self, amount: f32) {
        self.qe += amount.max(0.0);
    }

    pub fn is_dead(&self) -> bool {
        self.qe <= 0.0
    }

    /// Asigna `qe` directamente, clampeado a `[0, ∞)`.
    /// Usar sólo en sistemas que calculan el nuevo valor externamente
    /// (ET-5 symbiosis, ET-6 epigenetics, ET-7 senescence, etc.).
    pub fn set_qe(&mut self, val: f32) {
        self.qe = val.max(0.0);
    }
}

#[derive(SystemParam)]
pub struct EnergyOps<'w, 's> {
    query: Query<'w, 's, &'static mut BaseEnergy>,
    deaths: EventWriter<'w, DeathEvent>,
}

impl<'w, 's> EnergyOps<'w, 's> {
    pub fn drain(&mut self, entity: Entity, amount: f32, cause: DeathCause) -> f32 {
        let clamped = amount.max(0.0);
        let Ok(energy_ref) = self.query.get(entity) else {
            return 0.0;
        };
        let qe_before = energy_ref.qe();
        let drained = clamped.min(qe_before);

        // Entidad viva sin drenaje: no tocar `Mut` (evita `Changed<BaseEnergy>` falso).
        if drained == 0.0 && qe_before >= QE_MIN_EXISTENCE {
            return 0.0;
        }
        // Ya disipada (qe=0) y sin drenaje: idempotente; evita `Mut` y spam de `DeathEvent`.
        if drained == 0.0 && qe_before == 0.0 {
            return 0.0;
        }

        if let Ok(mut energy) = self.query.get_mut(entity) {
            energy.qe -= drained;

            if energy.qe < QE_MIN_EXISTENCE {
                energy.qe = 0.0;
                self.deaths.send(DeathEvent { entity, cause });
            }
        }
        drained
    }

    pub fn inject(&mut self, entity: Entity, amount: f32) {
        if amount <= 0.0 {
            return;
        }
        if let Ok(mut energy) = self.query.get_mut(entity) {
            energy.qe += amount;
        }
    }

    pub fn qe(&self, entity: Entity) -> Option<f32> {
        self.query.get(entity).ok().map(|e| e.qe())
    }

    pub fn is_alive(&self, entity: Entity) -> bool {
        self.qe(entity)
            .map(|qe| qe > QE_MIN_EXISTENCE)
            .unwrap_or(false)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::blueprint::constants::DEFAULT_BASE_ENERGY;
    use crate::events::DeathCause;
    use bevy::prelude::{App, EventReader, MinimalPlugins, Res, Resource, Update};

    #[test]
    fn inject_increases_qe() {
        let mut e = BaseEnergy::new(100.0);
        e.inject(50.0);
        assert!((e.qe() - 150.0).abs() < 1e-5);
    }

    #[test]
    fn inject_negative_does_not_reduce_qe() {
        let mut e = BaseEnergy::new(100.0);
        e.inject(-10.0);
        assert!((e.qe() - 100.0).abs() < 1e-5);
    }

    #[test]
    fn drain_reduces_qe_and_returns_drained() {
        let mut e = BaseEnergy::new(100.0);
        let got = e.drain(50.0);
        assert!((got - 50.0).abs() < 1e-5);
        assert!((e.qe() - 50.0).abs() < 1e-5);
    }

    #[test]
    fn drain_clamps_to_zero() {
        let mut e = BaseEnergy::new(100.0);
        e.drain(200.0);
        assert_eq!(e.qe(), 0.0);
    }

    #[test]
    fn inject_nan_does_not_poison_qe() {
        let mut e = BaseEnergy::new(100.0);
        e.inject(f32::NAN);
        assert!(e.qe().is_finite());
        assert!((e.qe() - 100.0).abs() < 1e-5);
    }

    #[test]
    fn new_clamps_negative_to_zero() {
        let e = BaseEnergy::new(-50.0);
        assert_eq!(e.qe(), 0.0);
    }

    #[test]
    fn drain_overdraw_returns_only_available() {
        let mut e = BaseEnergy::new(30.0);
        let got = e.drain(100.0);
        assert!((got - 30.0).abs() < 1e-5, "should return available, not requested: {got}");
        assert_eq!(e.qe(), 0.0);
    }

    #[test]
    fn default_matches_ssot_constant() {
        let e = BaseEnergy::default();
        assert!((e.qe() - DEFAULT_BASE_ENERGY).abs() < 1e-5);
    }

    #[derive(Resource, Clone, Copy)]
    struct DrainTestTarget(Entity);

    fn drain_zero_on_target(mut ops: EnergyOps, t: Res<DrainTestTarget>) {
        let _ = ops.drain(t.0, 0.0, DeathCause::Dissipation);
    }

    fn assert_no_death_event(mut ev: EventReader<crate::events::DeathEvent>) {
        assert_eq!(
            ev.read().count(),
            0,
            "drain(0) con qe=0 no debe re-emitir DeathEvent"
        );
    }

    #[test]
    fn energy_ops_drain_zero_on_dead_entity_skips_component_mutation() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_event::<crate::events::DeathEvent>();
        let id = app.world_mut().spawn(BaseEnergy::new(0.0)).id();
        app.insert_resource(DrainTestTarget(id));
        app.add_systems(
            Update,
            (drain_zero_on_target, assert_no_death_event).chain(),
        );
        app.update();
    }
}
