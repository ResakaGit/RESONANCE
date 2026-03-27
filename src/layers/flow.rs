use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use crate::blueprint::constants::DEFAULT_DISSIPATION_RATE;

/// Capa 3: Dinámica — El Flujo y la Entropía
///
/// Tendencia de la energía a desplazarse y disiparse (Segunda Ley de la Termodinámica).
///
/// Ecuaciones clave:
///   posición += velocidad * dt
///   qe -= tasa_disipacion_efectiva * dt
///   tasa_efectiva = tasa_disipacion + coef_friccion * |velocidad|²
///   F_arrastre = -0.5 * viscosidad * densidad * |v| * v
#[derive(Component, Reflect, Debug, Clone, Serialize, Deserialize)]
#[reflect(Component)]
pub struct FlowVector {
    /// Dirección y rapidez (unidades/segundo).
    pub(crate) velocity: Vec2,

    /// Tasa base de disipación: porción de qe que pierde por segundo en el vacío.
    pub(crate) dissipation_rate: f32,
}

impl Default for FlowVector {
    fn default() -> Self {
        Self {
            velocity: Vec2::ZERO,
            dissipation_rate: DEFAULT_DISSIPATION_RATE,
        }
    }
}

impl FlowVector {
    pub fn new(velocity: Vec2, dissipation_rate: f32) -> Self {
        Self {
            velocity,
            dissipation_rate: dissipation_rate.max(0.0),
        }
    }

    #[inline]
    pub fn velocity(&self) -> Vec2 {
        self.velocity
    }

    /// Asigna velocidad; descarta no finitos. Si `max_speed` es `Some`, clampea la magnitud.
    /// No muta si el valor resultante es idéntico (evita falsos positivos de `Changed<FlowVector>`).
    pub fn set_velocity(&mut self, v: Vec2, max_speed: Option<f32>) {
        let mut v = if v.is_finite() { v } else { Vec2::ZERO };
        if let Some(ms) = max_speed {
            if ms.is_finite() && ms >= 0.0 {
                v = v.clamp_length_max(ms);
            }
        }
        if self.velocity != v {
            self.velocity = v;
        }
    }

    /// Suma delta a la velocidad y aplica el mismo saneo que `set_velocity`.
    pub fn add_velocity(&mut self, delta: Vec2, max_speed: Option<f32>) {
        self.set_velocity(self.velocity + delta, max_speed);
    }

    #[inline]
    pub fn dissipation_rate(&self) -> f32 {
        self.dissipation_rate
    }

    pub fn set_dissipation_rate(&mut self, rate: f32) {
        let next = rate.max(0.0);
        if self.dissipation_rate != next {
            self.dissipation_rate = next;
        }
    }

    /// Tasa de disipación efectiva incluyendo fricción cinética.
    pub fn effective_dissipation(&self, friction_coef: f32) -> f32 {
        self.dissipation_rate + friction_coef * self.velocity.length_squared()
    }

    /// Rapidez escalar.
    pub fn speed(&self) -> f32 {
        self.velocity.length()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::blueprint::constants::DEFAULT_DISSIPATION_RATE;

    #[test]
    fn default_is_zero_velocity_and_ssot_dissipation() {
        let f = FlowVector::default();
        assert_eq!(f.velocity(), Vec2::ZERO);
        assert!((f.dissipation_rate() - DEFAULT_DISSIPATION_RATE).abs() < 1e-5);
    }

    #[test]
    fn new_stores_velocity_and_clamps_negative_dissipation() {
        let v = Vec2::new(3.0, -4.0);
        let f = FlowVector::new(v, -2.0);
        assert_eq!(f.velocity(), v);
        assert_eq!(f.dissipation_rate(), 0.0);
    }

    #[test]
    fn zero_velocity_is_valid() {
        let f = FlowVector::new(Vec2::ZERO, 1.0);
        assert_eq!(f.speed(), 0.0);
    }

    #[test]
    fn dissipation_rate_never_negative_via_setter() {
        let mut f = FlowVector::default();
        f.set_dissipation_rate(-10.0);
        assert_eq!(f.dissipation_rate(), 0.0);
    }

    #[test]
    fn set_velocity_idempotent_when_normalized_value_unchanged() {
        let mut f = FlowVector::new(Vec2::new(3.0, 4.0), 0.5);
        f.set_velocity(Vec2::new(3.0, 4.0), None);
        assert_eq!(f.velocity(), Vec2::new(3.0, 4.0));
    }

    #[test]
    fn set_dissipation_rate_idempotent() {
        let mut f = FlowVector::new(Vec2::ZERO, 0.25);
        f.set_dissipation_rate(0.25);
        assert!((f.dissipation_rate() - 0.25).abs() < 1e-6);
    }
}
