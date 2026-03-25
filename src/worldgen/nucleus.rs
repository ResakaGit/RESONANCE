use bevy::prelude::*;
use serde::{Deserialize, Serialize};

/// Modelo de decaimiento radial de intensidad para propagación de campo.
#[derive(Clone, Copy, Debug, Reflect, PartialEq, Serialize, Deserialize)]
pub enum PropagationDecay {
    InverseSquare,
    InverseLinear,
    Flat,
    Exponential { k: f32 },
}

/// Emisor de energía frecuencial hacia el campo de worldgen (Capa 6 extendida).
#[derive(Component, Clone, Copy, Debug, Reflect, PartialEq)]
#[reflect(Component)]
pub struct EnergyNucleus {
    /// Frecuencia central del núcleo [Hz].
    pub(crate) frequency_hz: f32,
    /// Emisión energética en qe/s.
    pub(crate) emission_rate_qe_s: f32,
    /// Radio de influencia máxima.
    pub(crate) propagation_radius: f32,
    /// Curva de caída por distancia.
    pub(crate) decay: PropagationDecay,
}

impl EnergyNucleus {
    pub fn new(
        frequency_hz: f32,
        emission_rate_qe_s: f32,
        propagation_radius: f32,
        decay: PropagationDecay,
    ) -> Self {
        let frequency_hz = if frequency_hz.is_finite() {
            frequency_hz.max(0.0)
        } else {
            0.0
        };
        let emission_rate_qe_s = if emission_rate_qe_s.is_finite() {
            emission_rate_qe_s.max(0.0)
        } else {
            0.0
        };
        let propagation_radius = if propagation_radius.is_finite() {
            propagation_radius.max(0.0)
        } else {
            0.0
        };
        let decay = match decay {
            PropagationDecay::Exponential { k } if k.is_finite() => {
                PropagationDecay::Exponential { k: k.max(0.0) }
            }
            PropagationDecay::Exponential { .. } => PropagationDecay::Exponential { k: 0.0 },
            other => other,
        };
        Self {
            frequency_hz,
            emission_rate_qe_s,
            propagation_radius,
            decay,
        }
    }

    #[inline]
    pub fn frequency_hz(&self) -> f32 {
        self.frequency_hz
    }

    pub fn set_frequency_hz(&mut self, hz: f32) {
        let next = Self::new(
            hz,
            self.emission_rate_qe_s,
            self.propagation_radius,
            self.decay,
        );
        if *self != next {
            *self = next;
        }
    }

    #[inline]
    pub fn emission_rate_qe_s(&self) -> f32 {
        self.emission_rate_qe_s
    }

    pub fn set_emission_rate_qe_s(&mut self, rate: f32) {
        let next = Self::new(self.frequency_hz, rate, self.propagation_radius, self.decay);
        if *self != next {
            *self = next;
        }
    }

    #[inline]
    pub fn propagation_radius(&self) -> f32 {
        self.propagation_radius
    }

    pub fn set_propagation_radius(&mut self, r: f32) {
        let next = Self::new(self.frequency_hz, self.emission_rate_qe_s, r, self.decay);
        if *self != next {
            *self = next;
        }
    }

    #[inline]
    pub fn decay(&self) -> PropagationDecay {
        self.decay
    }

    pub fn set_decay(&mut self, decay: PropagationDecay) {
        let next = Self::new(
            self.frequency_hz,
            self.emission_rate_qe_s,
            self.propagation_radius,
            decay,
        );
        if *self != next {
            *self = next;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{EnergyNucleus, PropagationDecay};

    #[test]
    fn energy_nucleus_new_clamps_negative_values() {
        let nucleus = EnergyNucleus::new(200.0, -5.0, -10.0, PropagationDecay::InverseLinear);
        assert_eq!(nucleus.frequency_hz(), 200.0);
        assert_eq!(nucleus.emission_rate_qe_s(), 0.0);
        assert_eq!(nucleus.propagation_radius(), 0.0);
    }

    #[test]
    fn energy_nucleus_new_non_finite_values_are_zeroed() {
        let nucleus = EnergyNucleus::new(
            f32::NAN,
            f32::INFINITY,
            f32::NEG_INFINITY,
            PropagationDecay::Flat,
        );
        assert_eq!(nucleus.frequency_hz(), 0.0);
        assert_eq!(nucleus.emission_rate_qe_s(), 0.0);
        assert_eq!(nucleus.propagation_radius(), 0.0);
    }

    #[test]
    fn propagation_decay_exponential_preserves_k_and_is_copy() {
        let decay = PropagationDecay::Exponential { k: 0.42 };
        let copied = decay;
        match copied {
            PropagationDecay::Exponential { k } => assert!((k - 0.42).abs() < f32::EPSILON),
            _ => panic!("expected Exponential variant"),
        }
    }

    #[test]
    fn energy_nucleus_new_clamps_negative_exponential_k() {
        let nucleus =
            EnergyNucleus::new(200.0, 5.0, 10.0, PropagationDecay::Exponential { k: -0.8 });
        match nucleus.decay() {
            PropagationDecay::Exponential { k } => assert_eq!(k, 0.0),
            _ => panic!("expected Exponential variant"),
        }
    }

    #[test]
    fn energy_nucleus_new_non_finite_exponential_k_is_zeroed() {
        let nucleus = EnergyNucleus::new(
            200.0,
            5.0,
            10.0,
            PropagationDecay::Exponential { k: f32::NAN },
        );
        match nucleus.decay() {
            PropagationDecay::Exponential { k } => assert_eq!(k, 0.0),
            _ => panic!("expected Exponential variant"),
        }
    }
}
