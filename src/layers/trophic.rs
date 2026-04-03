use bevy::prelude::*;

use super::inference::TrophicClass;

/// Consumidor trófico: clase + capacidad de intake base.
#[derive(Component, Reflect, Debug, Clone, PartialEq)]
#[reflect(Component)]
pub struct TrophicConsumer {
    pub class: TrophicClass,
    pub intake_rate: f32,
}

impl TrophicConsumer {
    pub fn new(class: TrophicClass, intake_rate: f32) -> Self {
        Self {
            class,
            intake_rate: sanitize_positive(intake_rate),
        }
    }

    #[inline]
    pub fn is_predator(&self) -> bool {
        matches!(self.class, TrophicClass::Carnivore | TrophicClass::Omnivore)
    }

    #[inline]
    pub fn is_herbivore(&self) -> bool {
        matches!(self.class, TrophicClass::Herbivore | TrophicClass::Omnivore)
    }

    #[inline]
    pub fn is_decomposer(&self) -> bool {
        matches!(self.class, TrophicClass::Detritivore)
    }
}

/// Estado trófico transitorio: saciedad del consumidor.
#[derive(Component, Reflect, Debug, Clone)]
#[component(storage = "SparseSet")]
pub struct TrophicState {
    pub satiation: f32,
}

impl Default for TrophicState {
    fn default() -> Self {
        Self { satiation: 0.5 }
    }
}

impl TrophicState {
    pub fn new(satiation: f32) -> Self {
        Self {
            satiation: sanitize_unit(satiation),
        }
    }
}

#[inline]
fn sanitize_positive(value: f32) -> f32 {
    if value.is_finite() {
        value.max(0.0)
    } else {
        0.0
    }
}

#[inline]
fn sanitize_unit(value: f32) -> f32 {
    if value.is_finite() {
        value.clamp(0.0, 1.0)
    } else {
        0.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn trophic_consumer_carnivore_is_predator() {
        let c = TrophicConsumer::new(TrophicClass::Carnivore, 1.0);
        assert!(c.is_predator());
        assert!(!c.is_herbivore());
        assert!(!c.is_decomposer());
    }

    #[test]
    fn trophic_consumer_omnivore_is_both() {
        let c = TrophicConsumer::new(TrophicClass::Omnivore, 1.0);
        assert!(c.is_predator());
        assert!(c.is_herbivore());
    }

    #[test]
    fn trophic_consumer_herbivore_not_predator() {
        let c = TrophicConsumer::new(TrophicClass::Herbivore, 1.0);
        assert!(!c.is_predator());
        assert!(c.is_herbivore());
    }

    #[test]
    fn trophic_consumer_detritivore_is_decomposer() {
        let c = TrophicConsumer::new(TrophicClass::Detritivore, 1.0);
        assert!(c.is_decomposer());
    }

    #[test]
    fn trophic_consumer_sanitizes_negative_intake() {
        let c = TrophicConsumer::new(TrophicClass::Herbivore, -5.0);
        assert_eq!(c.intake_rate, 0.0);
    }

    #[test]
    fn trophic_consumer_sanitizes_nan_intake() {
        let c = TrophicConsumer::new(TrophicClass::Herbivore, f32::NAN);
        assert_eq!(c.intake_rate, 0.0);
    }

    #[test]
    fn trophic_state_default_half_satiation() {
        let s = TrophicState::default();
        assert!((s.satiation - 0.5).abs() < f32::EPSILON);
    }

    #[test]
    fn trophic_state_new_clamps_satiation() {
        let s = TrophicState::new(1.5);
        assert_eq!(s.satiation, 1.0);
        let s2 = TrophicState::new(-0.5);
        assert_eq!(s2.satiation, 0.0);
    }
}
