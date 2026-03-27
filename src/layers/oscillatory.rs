use bevy::ecs::system::SystemParam;
use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use std::f32::consts::PI;

use crate::blueprint::constants::DEFAULT_FREQUENCY_HZ;

use crate::blueprint::equations;
use crate::layers::MobaIdentity;
use crate::runtime_platform::simulation_tick::SimulationElapsed;

/// Capa 2: Resonancia — El Tiempo y la Forma
///
/// La energía concentrada oscila. La frecuencia define el "elemento" y la fase
/// determina la alineación para cálculos de interferencia.
///
/// Interferencia entre dos osciladores:
///   I(a,b) = cos(2π * |f_a - f_b| * t + (φ_a - φ_b))
///
/// I ~ +1.0 → constructiva (resonancia, amplificación, curación)
/// I ~ -1.0 → destructiva (daño, aniquilación, oposición)
/// I ~  0.0 → ortogonal (sin interacción)
#[derive(Component, Reflect, Debug, Clone, Serialize, Deserialize)]
#[reflect(Component)]
pub struct OscillatorySignature {
    /// Frecuencia primaria de oscilación (Hz).
    /// Bandas elementales:
    ///   Umbra:  10-30    | Terra: 50-100  | Aqua: 200-300
    ///   Ignis: 400-500   | Ventus: 600-800 | Lux: 900-1100
    pub(crate) frequency_hz: f32,

    /// Desplazamiento de fase en radianes [0, 2π).
    /// Usado para calcular interferencias y golpes críticos.
    pub(crate) phase: f32,
}

impl Default for OscillatorySignature {
    fn default() -> Self {
        Self {
            frequency_hz: DEFAULT_FREQUENCY_HZ, // Neutral/Cinética (banda Lux)
            phase: 0.0,
        }
    }
}

impl OscillatorySignature {
    pub fn new(frequency_hz: f32, phase: f32) -> Self {
        Self {
            frequency_hz: frequency_hz.max(0.0),
            phase: phase.rem_euclid(2.0 * PI),
        }
    }

    #[inline]
    pub fn frequency_hz(&self) -> f32 {
        self.frequency_hz
    }

    pub fn set_frequency_hz(&mut self, hz: f32) {
        let next = if hz.is_finite() { hz.max(0.0) } else { 0.0 };
        if self.frequency_hz != next {
            self.frequency_hz = next;
        }
    }

    #[inline]
    pub fn phase(&self) -> f32 {
        self.phase
    }

    pub fn set_phase(&mut self, phase: f32) {
        let next = phase.rem_euclid(2.0 * PI);
        if self.phase != next {
            self.phase = next;
        }
    }

    /// Calcula la interferencia con otra firma oscilatoria en el instante `t`.
    /// Retorna un valor en [-1.0, 1.0].
    #[inline]
    pub fn interference(&self, other: &OscillatorySignature, t: f32) -> f32 {
        equations::interference(
            self.frequency_hz,
            self.phase,
            other.frequency_hz,
            other.phase,
            t,
        )
    }
}

/// SSOT de interferencia total:
/// 1) interferencia física pura
/// 2) modificador de facción
/// 3) clamp canónico [-1, 1]
#[inline]
pub fn compose_interference(raw: f32, faction_mod: f32) -> f32 {
    (raw + faction_mod).clamp(-1.0, 1.0)
}

/// Camino único para calcular interferencia entre dos osciladores.
#[inline]
pub fn compute_interference_total(
    freq_a: f32,
    phase_a: f32,
    freq_b: f32,
    phase_b: f32,
    t: f32,
    faction_mod: f32,
) -> f32 {
    let raw = equations::interference(freq_a, phase_a, freq_b, phase_b, t);
    compose_interference(raw, faction_mod)
}

#[derive(SystemParam)]
pub struct InterferenceOps<'w, 's> {
    waves: Query<'w, 's, &'static OscillatorySignature>,
    identities: Query<'w, 's, &'static MobaIdentity>,
    sim_elapsed: Option<Res<'w, SimulationElapsed>>,
    time: Res<'w, Time>,
}

impl<'w, 's> InterferenceOps<'w, 's> {
    #[inline]
    fn phase_time_secs(&self) -> f32 {
        self.sim_elapsed
            .as_ref()
            .map(|sim| sim.secs)
            .unwrap_or_else(|| self.time.elapsed_secs())
    }

    pub fn between(&self, a: Entity, b: Entity) -> Option<f32> {
        let wave_a = self.waves.get(a).ok()?;
        let wave_b = self.waves.get(b).ok()?;

        let t = self.phase_time_secs();
        let faction_mod = match (self.identities.get(a).ok(), self.identities.get(b).ok()) {
            (Some(id_a), Some(id_b)) => id_a.faction_modifier(id_b),
            _ => 0.0,
        };

        Some(compute_interference_total(
            wave_a.frequency_hz(),
            wave_a.phase(),
            wave_b.frequency_hz(),
            wave_b.phase(),
            t,
            faction_mod,
        ))
    }

    pub fn raw(&self, a: Entity, b: Entity) -> Option<f32> {
        let wave_a = self.waves.get(a).ok()?;
        let wave_b = self.waves.get(b).ok()?;

        let t = self.phase_time_secs();
        Some(equations::interference(
            wave_a.frequency_hz(),
            wave_a.phase(),
            wave_b.frequency_hz(),
            wave_b.phase(),
            t,
        ))
    }

    pub fn critical_multiplier(&self, entity: Entity) -> f32 {
        self.identities
            .get(entity)
            .ok()
            .map(|id| id.critical_multiplier())
            .unwrap_or(1.0)
    }

    pub fn elapsed(&self) -> f32 {
        self.phase_time_secs()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn compose_interference_clamps_canonical_range() {
        assert_eq!(compose_interference(0.9, 0.5), 1.0);
        assert_eq!(compose_interference(-0.9, -0.5), -1.0);
        assert_eq!(compose_interference(0.2, -0.1), 0.1);
    }

    #[test]
    fn compute_interference_total_matches_composed_equation() {
        let raw = equations::interference(450.0, 0.2, 700.0, 1.1, 0.5);
        let total = compute_interference_total(450.0, 0.2, 700.0, 1.1, 0.5, 0.15);
        assert!((total - compose_interference(raw, 0.15)).abs() < 1e-6);
    }

    #[test]
    fn interference_stays_in_unit_range_for_typical_frequencies() {
        let a = OscillatorySignature::new(100.0, 0.0);
        let b = OscillatorySignature::new(200.0, 1.0);
        let i = a.interference(&b, 0.25);
        assert!(i >= -1.0 - 1e-5 && i <= 1.0 + 1e-5);
        assert!(i.is_finite());
    }

    #[test]
    fn oscillatory_interference_delegates_to_equations_ssot() {
        let a = OscillatorySignature::new(111.0, 0.4);
        let b = OscillatorySignature::new(333.0, 2.1);
        let t = 0.37;
        let from_method = a.interference(&b, t);
        let from_eq =
            equations::interference(a.frequency_hz(), a.phase(), b.frequency_hz(), b.phase(), t);
        assert_eq!(from_method, from_eq);
    }

    #[test]
    fn set_frequency_hz_is_noop_when_value_matches() {
        let mut s = OscillatorySignature::new(200.0, 0.5);
        s.set_frequency_hz(200.0);
        assert!((s.frequency_hz() - 200.0).abs() < 1e-6);
        assert!((s.phase() - 0.5).abs() < 1e-6);
    }
}
