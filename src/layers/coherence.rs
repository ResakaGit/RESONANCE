use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use crate::blueprint::constants::{
    DEFAULT_BOND_ENERGY, DEFAULT_THERMAL_CONDUCTIVITY, DISSIPATION_MULT_GAS,
    DISSIPATION_MULT_LIQUID, DISSIPATION_MULT_PLASMA, DISSIPATION_MULT_SOLID,
    VELOCITY_LIMIT_LIQUID,
};

/// Estados de la materia con implicaciones de gameplay:
///   Solido:  sin velocidad (fijado), alto daño colisión, baja disipación
///   Liquido: velocidad limitada, conductividad moderada, fluye alrededor de obstáculos
///   Gas:     sin límite de velocidad, alta disipación, atraviesa sólidos
///   Plasma:  máximo daño, máxima disipación, emite radiación (Capa 8)
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Reflect, Default, Deserialize, Serialize)]
pub enum MatterState {
    #[default]
    Solid,
    Liquid,
    Gas,
    Plasma,
}

/// Capa 4: Materia — La Coherencia Estructural
///
/// El "freno" a la disipación. Contenedor que resiste el estrés de las Capas 2 y 3.
/// Si la energía interna supera la energía de enlace, la entidad colapsa o muta.
///
/// Transiciones de fase (basadas en densidad → temperatura equivalente):
///   T_equiv = densidad / k_boltzmann_juego
///   T < 0.3 * eb → Sólido
///   T < 1.0 * eb → Líquido
///   T < 3.0 * eb → Gas
///   T >= 3.0 * eb → Plasma
#[derive(Component, Reflect, Debug, Clone, Serialize, Deserialize)]
#[reflect(Component)]
pub struct MatterCoherence {
    /// Estado actual de la materia.
    pub(crate) state: MatterState,

    /// Energía de enlace: resistencia a cambiar de estado (fundirse/romperse).
    pub(crate) bond_energy_eb: f32,

    /// Conductividad térmica [0.0, 1.0]: cuánto calor deja pasar sin absorber.
    pub(crate) thermal_conductivity: f32,
}

impl Default for MatterCoherence {
    fn default() -> Self {
        Self {
            state: MatterState::Solid,
            bond_energy_eb: DEFAULT_BOND_ENERGY,
            thermal_conductivity: DEFAULT_THERMAL_CONDUCTIVITY,
        }
    }
}

impl MatterCoherence {
    pub fn new(state: MatterState, eb: f32, conductivity: f32) -> Self {
        Self {
            state,
            bond_energy_eb: eb.max(0.0),
            thermal_conductivity: conductivity.clamp(0.0, 1.0),
        }
    }

    #[inline]
    pub fn state(&self) -> MatterState {
        self.state
    }

    pub fn set_state(&mut self, s: MatterState) {
        if self.state != s { self.state = s; }
    }

    #[inline]
    pub fn bond_energy_eb(&self) -> f32 {
        self.bond_energy_eb
    }

    pub fn set_bond_energy_eb(&mut self, eb: f32) {
        let v = eb.max(0.0);
        if self.bond_energy_eb != v { self.bond_energy_eb = v; }
    }

    #[inline]
    pub fn thermal_conductivity(&self) -> f32 {
        self.thermal_conductivity
    }

    pub fn set_thermal_conductivity(&mut self, k: f32) {
        let v = k.clamp(0.0, 1.0);
        if self.thermal_conductivity != v { self.thermal_conductivity = v; }
    }

    /// Daño estructural normalizado [0,1]: proxy de degradación física.
    pub fn structural_damage(&self) -> f32 {
        match self.state {
            MatterState::Solid  => 0.0,
            MatterState::Liquid => 0.33,
            MatterState::Gas    => 0.66,
            MatterState::Plasma => 1.0,
        }
    }

    /// Multiplicador de velocidad máxima según estado.
    pub fn velocity_limit(&self) -> Option<f32> {
        match self.state {
            MatterState::Solid => Some(0.0),
            MatterState::Liquid => Some(VELOCITY_LIMIT_LIQUID),
            MatterState::Gas => None,
            MatterState::Plasma => None,
        }
    }

    /// Multiplicador de disipación según estado.
    pub fn dissipation_multiplier(&self) -> f32 {
        match self.state {
            MatterState::Solid => DISSIPATION_MULT_SOLID,
            MatterState::Liquid => DISSIPATION_MULT_LIQUID,
            MatterState::Gas => DISSIPATION_MULT_GAS,
            MatterState::Plasma => DISSIPATION_MULT_PLASMA,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::blueprint::constants::DEFAULT_BOND_ENERGY;

    #[test]
    fn default_is_solid_with_positive_bond_energy() {
        let c = MatterCoherence::default();
        assert_eq!(c.state(), MatterState::Solid);
        assert!(c.bond_energy_eb().is_finite());
        assert!(c.bond_energy_eb() > 0.0);
        assert_eq!(c.bond_energy_eb(), DEFAULT_BOND_ENERGY);
    }

    #[test]
    fn velocity_limit_solid_is_zero_liquid_is_positive_gas_unbounded() {
        let solid = MatterCoherence::new(MatterState::Solid, 1000.0, 0.5);
        let liq = MatterCoherence::new(MatterState::Liquid, 1000.0, 0.5);
        let gas = MatterCoherence::new(MatterState::Gas, 1000.0, 0.5);
        assert_eq!(solid.velocity_limit(), Some(0.0));
        let liq_lim = liq.velocity_limit().expect("liquid capped");
        assert!(liq_lim.is_finite() && liq_lim > 0.0);
        assert!(gas.velocity_limit().is_none());
    }

    #[test]
    fn unbounded_velocity_limit_exceeds_solid_cap() {
        let solid = MatterCoherence::new(MatterState::Solid, 1000.0, 0.5);
        let gas = MatterCoherence::new(MatterState::Gas, 1000.0, 0.5);
        let s = solid.velocity_limit().unwrap_or(0.0);
        let g = gas.velocity_limit().unwrap_or(f32::INFINITY);
        assert!(g > s);
    }

    #[test]
    fn plasma_dissipation_exceeds_solid() {
        let s = MatterCoherence::new(MatterState::Solid, 1000.0, 0.5);
        let p = MatterCoherence::new(MatterState::Plasma, 1000.0, 0.5);
        assert!(p.dissipation_multiplier() > s.dissipation_multiplier());
    }

    #[test]
    fn every_matter_state_has_non_negative_finite_dissipation_multiplier() {
        for state in [
            MatterState::Solid,
            MatterState::Liquid,
            MatterState::Gas,
            MatterState::Plasma,
        ] {
            let c = MatterCoherence::new(state, 1000.0, 0.5);
            let m = c.dissipation_multiplier();
            assert!(m.is_finite() && m >= 0.0);
        }
    }

    #[test]
    fn liquid_velocity_cap_is_strictly_positive() {
        let c = MatterCoherence::new(MatterState::Liquid, 1000.0, 0.5);
        let lim = c.velocity_limit().expect("liquid");
        assert!(lim > 0.0);
    }
}
