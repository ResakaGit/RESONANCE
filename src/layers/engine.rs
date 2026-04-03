use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use crate::blueprint::constants::{
    ENGINE_DEFAULT_INPUT_VALVE, ENGINE_DEFAULT_MAX_BUFFER, ENGINE_DEFAULT_OUTPUT_VALVE,
    ENGINE_EFFICIENCY_FALLOFF, ENGINE_EFFICIENCY_FREQ_DIVISOR, ENGINE_MASTERY_BONUS,
    LINK_NEUTRAL_MULTIPLIER, OVERLOAD_FACTOR,
};
use crate::blueprint::equations;
use crate::blueprint::{AlchemicalAlmanac, ElementId};

/// Capa 5: Enrutamiento — El Motor Abierto
/// Layer 5: Routing — The Open Engine
///
/// Capacitor entre campo de energía (L0) y habilidades (L8).
/// Capacitor between energy field (L0) and abilities (L8).
#[derive(Component, Reflect, Debug, Clone, Serialize, Deserialize)]
#[reflect(Component)]
pub struct AlchemicalEngine {
    /// "Maná" acumulado (qe aislado del campo principal).
    pub(crate) current_buffer: f32,

    /// Capacidad máxima antes de sobrecargarse.
    pub(crate) max_buffer: f32,

    /// Tasa máxima de absorción por segundo (intake).
    pub(crate) input_valve: f32,

    /// Tasa máxima de expulsión por segundo (cast speed).
    pub(crate) output_valve: f32,
}

impl Default for AlchemicalEngine {
    fn default() -> Self {
        Self {
            current_buffer: 0.0,
            max_buffer: ENGINE_DEFAULT_MAX_BUFFER,
            input_valve: ENGINE_DEFAULT_INPUT_VALVE,
            output_valve: ENGINE_DEFAULT_OUTPUT_VALVE,
        }
    }
}

impl AlchemicalEngine {
    pub fn new(max_buffer: f32, input: f32, output: f32, initial: f32) -> Self {
        Self {
            current_buffer: initial.clamp(0.0, max_buffer),
            max_buffer: max_buffer.max(0.0),
            input_valve: input.max(0.0),
            output_valve: output.max(0.0),
        }
    }

    #[inline]
    pub fn buffer_level(&self) -> f32 {
        self.current_buffer
    }

    #[inline]
    pub fn buffer_cap(&self) -> f32 {
        self.max_buffer
    }

    #[inline]
    pub fn valve_in_rate(&self) -> f32 {
        self.input_valve
    }

    #[inline]
    pub fn intake(&self) -> f32 {
        self.input_valve
    }

    #[inline]
    pub fn base_intake(&self) -> f32 {
        self.input_valve
    }

    pub fn set_intake(&mut self, rate: f32) {
        self.input_valve = rate.max(0.0);
    }

    #[inline]
    pub fn valve_out_rate(&self) -> f32 {
        self.output_valve
    }

    /// Espacio libre en el buffer.
    pub fn free_space(&self) -> f32 {
        (self.max_buffer - self.current_buffer).max(0.0)
    }

    /// Calcula cuánta energía puede absorber este tick.
    pub fn available_intake(&self, dt: f32, qe_available: f32) -> f32 {
        let max_tick = self.input_valve * dt;
        max_tick.min(qe_available).min(self.free_space())
    }

    /// Variante TL6: intake efectivo usando escalamiento alométrico por radio.
    pub fn available_intake_allometric(&self, dt: f32, qe_available: f32, radius: f32) -> f32 {
        equations::engine_intake_allometric(
            self.input_valve,
            dt,
            qe_available,
            self.current_buffer,
            self.max_buffer,
            radius,
        )
    }

    /// Absorbe energía al buffer. Retorna cuánto absorbió realmente.
    pub fn absorb(&mut self, amount: f32) -> f32 {
        let absorbed = amount.min(self.free_space());
        if absorbed > 0.0 {
            self.current_buffer += absorbed;
        }
        absorbed
    }

    /// Consume energía del buffer para una habilidad. Retorna cuánto consumió realmente.
    pub fn consume(&mut self, amount: f32, dt: f32) -> f32 {
        let max_tick = self.output_valve * dt;
        let consumed = amount.min(self.current_buffer).min(max_tick);
        if consumed > 0.0 {
            self.current_buffer -= consumed;
        }
        consumed
    }

    /// ¿Está el buffer lleno?
    pub fn is_full(&self) -> bool {
        self.current_buffer >= self.max_buffer
    }

    /// ¿Está el buffer sobrecargado? (puede pasar por inyección externa)
    pub fn is_overloaded(&self) -> bool {
        self.current_buffer > self.max_buffer * OVERLOAD_FACTOR
    }

    /// Resta `amount` del buffer si hay saldo; usado por gasto instantáneo (grimoire).
    pub fn try_subtract_buffer(&mut self, amount: f32) -> bool {
        let amt = amount.max(0.0);
        if self.current_buffer < amt {
            return false;
        }
        if amt > 0.0 {
            self.current_buffer -= amt;
        }
        true
    }

    /// Alias semántico: drenaje por cast (misma lógica que `try_subtract_buffer`).
    #[inline]
    pub fn drain_buffer(&mut self, amount: f32) -> bool {
        self.try_subtract_buffer(amount)
    }
}

/// Max mastered elements per entity.
const MAX_MASTERED: usize = 4;
/// Max discovered compounds per entity.
const MAX_COMPOUNDS: usize = 4;

/// Extensión del motor: identidad alquímica de la entidad.
///
/// Fixed-size arrays (no heap). Entities master ≤4 elements, discover ≤4 compounds.
#[derive(Component, Reflect, Debug, Clone)]
#[reflect(Component)]
pub struct AlchemicalForge {
    /// Elementos dominados + compuestos descubiertos (fixed-size, no heap).
    pub(crate) mastered_elements: [ElementId; MAX_MASTERED],
    pub(crate) mastered_count: u8,
    pub(crate) discovered_compounds: [ElementId; MAX_COMPOUNDS],
    pub(crate) discovered_count: u8,
    /// Multiplicador global de eficiencia de creación.
    pub creation_bonus: f32,
}

impl Default for AlchemicalForge {
    fn default() -> Self {
        Self {
            mastered_elements: [ElementId::default(); MAX_MASTERED],
            mastered_count: 0,
            creation_bonus: LINK_NEUTRAL_MULTIPLIER,
            discovered_compounds: [ElementId::default(); MAX_COMPOUNDS],
            discovered_count: 0,
        }
    }
}

impl AlchemicalForge {
    pub fn new(primary_element: ElementId) -> Self {
        let mut s = Self::default();
        s.mastered_elements[0] = primary_element;
        s.mastered_count = 1;
        s
    }

    /// Active mastered elements slice.
    pub fn mastered(&self) -> &[ElementId] {
        &self.mastered_elements[..self.mastered_count as usize]
    }

    /// Eficiencia de creación para un elemento dado.
    ///
    ///   1.0 - (|f_caster - f_target| / ENGINE_EFFICIENCY_FREQ_DIVISOR) × ENGINE_EFFICIENCY_FALLOFF + mastery_bonus
    pub fn creation_efficiency(&self, target: ElementId, almanac: &AlchemicalAlmanac) -> f32 {
        let f_target = almanac.get(target).map(|d| d.frequency_hz).unwrap_or(0.0);
        let f_caster = self
            .mastered()
            .first()
            .and_then(|id| almanac.get(*id))
            .map(|d| d.frequency_hz)
            .unwrap_or(f_target);

        let base = 1.0
            - ((f_caster - f_target).abs() / ENGINE_EFFICIENCY_FREQ_DIVISOR)
                * ENGINE_EFFICIENCY_FALLOFF;
        let mastery_bonus = if self.mastered_elements.contains(&target) {
            ENGINE_MASTERY_BONUS
        } else {
            0.0
        };

        (base + mastery_bonus)
            .mul_add(self.creation_bonus, 0.0)
            .clamp(0.0, 1.0)
    }

    /// Registra un compuesto como descubierto.
    pub fn discover(&mut self, compound: ElementId) {
        let count = self.discovered_count as usize;
        if count < MAX_COMPOUNDS && !self.discovered_compounds[..count].contains(&compound) {
            self.discovered_compounds[count] = compound;
            self.discovered_count += 1;
        }
    }

    /// Promueve un compuesto descubierto a dominado (mastered).
    pub fn master(&mut self, element: ElementId) {
        let m_count = self.mastered_count as usize;
        if m_count < MAX_MASTERED && !self.mastered_elements[..m_count].contains(&element) {
            self.mastered_elements[m_count] = element;
            self.mastered_count += 1;
        }
        // Si era descubierto, ya no necesita estar ahí — compact in place
        let mut write = 0usize;
        for read in 0..self.discovered_count as usize {
            if self.discovered_compounds[read] != element {
                self.discovered_compounds[write] = self.discovered_compounds[read];
                write += 1;
            }
        }
        self.discovered_count = write as u8;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::blueprint::constants::{
        ENGINE_DEFAULT_MAX_BUFFER, LINK_NEUTRAL_MULTIPLIER, OVERLOAD_FACTOR,
    };
    use crate::blueprint::{AlchemicalAlmanac, ElementDef, ElementId};
    use crate::layers::MatterState;

    #[test]
    fn absorb_respects_free_space() {
        let mut m = AlchemicalEngine::new(1000.0, 10.0, 50.0, 0.0);
        let took = m.absorb(100.0);
        assert!((took - 100.0).abs() < 1e-5);
        assert!((m.buffer_level() - 100.0).abs() < 1e-5);
    }

    #[test]
    fn absorb_clamps_to_max_buffer() {
        let mut m = AlchemicalEngine::new(1000.0, 10.0, 50.0, 0.0);
        let took = m.absorb(2000.0);
        assert!((took - ENGINE_DEFAULT_MAX_BUFFER).abs() < 1e-4);
        assert!((m.buffer_level() - ENGINE_DEFAULT_MAX_BUFFER).abs() < 1e-4);
    }

    #[test]
    fn consume_respects_buffer_and_output_cap() {
        let mut m = AlchemicalEngine::new(1000.0, 10.0, 50.0, 100.0);
        let c1 = m.consume(50.0, 1.0);
        assert!((c1 - 50.0).abs() < 1e-5);
        assert!((m.buffer_level() - 50.0).abs() < 1e-5);
    }

    #[test]
    fn available_intake_allometric_respects_floor_for_zero_radius() {
        let m = AlchemicalEngine::new(100.0, 0.01, 10.0, 0.0);
        let intake = m.available_intake_allometric(1.0, 1.0, 0.0);
        assert!(intake > 0.0);
    }

    #[test]
    fn consume_large_request_drains_until_buffer_empty_with_enough_dt() {
        let mut m = AlchemicalEngine::new(1000.0, 10.0, 50.0, 100.0);
        let c = m.consume(200.0, 2.0);
        assert!((c - 100.0).abs() < 1e-5);
        assert_eq!(m.buffer_level(), 0.0);
    }

    #[test]
    fn buffer_level_reflects_current_buffer() {
        let m = AlchemicalEngine::new(100.0, 10.0, 10.0, 42.0);
        assert!((m.buffer_level() - 42.0).abs() < 1e-5);
    }

    #[test]
    fn is_overloaded_when_buffer_exceeds_factor() {
        let mut m = AlchemicalEngine::new(1000.0, 10.0, 50.0, 0.0);
        assert!(!m.is_overloaded());
        // absorb() está acotado por max_buffer; la sobrecarga es estado externo al buffer normal.
        m.current_buffer = m.buffer_cap() * OVERLOAD_FACTOR + 1.0;
        assert!(m.is_overloaded());
    }

    #[test]
    fn creation_efficiency_high_when_frequencies_match() {
        let terra = ElementId::from_name("Terra");
        let almanac = AlchemicalAlmanac::from_defs(vec![ElementDef {
            name: "Terra".to_string(),
            symbol: "Terra".to_string(),
            atomic_number: 0,
            frequency_hz: 75.0,
            freq_band: (50.0, 84.0),
            bond_energy: 1000.0,
            conductivity: 0.5,
            visibility: 0.5,
            matter_state: MatterState::Solid,
            electronegativity: 0.0,
            ionization_ev: 0.0,
            color: (0.0, 0.5, 0.0),
            is_compound: false,
            phenology: None,
            hz_identity_weight: crate::blueprint::FIELD_EAC4_HZ_IDENTITY_WEIGHT_RON_ONLY,
        }]);
        let forge = AlchemicalForge::new(terra);
        let eff_same = forge.creation_efficiency(terra, &almanac);
        assert!(eff_same >= LINK_NEUTRAL_MULTIPLIER);
    }

    #[test]
    fn creation_efficiency_lower_when_target_far_in_spectrum() {
        let terra = ElementId::from_name("Terra");
        let ventus = ElementId::from_name("Ventus");
        let almanac = AlchemicalAlmanac::from_defs(vec![
            ElementDef {
                name: "Terra".to_string(),
                symbol: "Terra".to_string(),
                atomic_number: 0,
                frequency_hz: 75.0,
                freq_band: (50.0, 84.0),
                bond_energy: 1000.0,
                conductivity: 0.5,
                visibility: 0.5,
                matter_state: MatterState::Solid,
                electronegativity: 0.0,
                ionization_ev: 0.0,
                color: (0.0, 0.5, 0.0),
                is_compound: false,
                phenology: None,
                hz_identity_weight: crate::blueprint::FIELD_EAC4_HZ_IDENTITY_WEIGHT_RON_ONLY,
            },
            ElementDef {
                name: "Ventus".to_string(),
                symbol: "Ventus".to_string(),
                atomic_number: 0,
                frequency_hz: 700.0,
                freq_band: (600.0, 800.0),
                bond_energy: 1000.0,
                conductivity: 0.5,
                visibility: 0.5,
                matter_state: MatterState::Gas,
                electronegativity: 0.0,
                ionization_ev: 0.0,
                color: (0.5, 0.5, 1.0),
                is_compound: false,
                phenology: None,
                hz_identity_weight: crate::blueprint::FIELD_EAC4_HZ_IDENTITY_WEIGHT_RON_ONLY,
            },
        ]);
        let forge = AlchemicalForge::new(terra);
        let eff_far = forge.creation_efficiency(ventus, &almanac);
        let eff_self = forge.creation_efficiency(terra, &almanac);
        assert!(eff_far < eff_self);
    }
}
