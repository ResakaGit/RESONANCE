use bevy::prelude::*;

const DEFAULT_INFERENCE_RESILIENCE: f32 = 0.5;

/// Perfil de inferencia morfo-conductual desacoplado de etiquetas taxonómicas.
#[derive(Component, Reflect, Debug, Clone, Copy, PartialEq)]
#[reflect(Component, PartialEq)]
pub struct InferenceProfile {
    pub growth_bias: f32,
    pub mobility_bias: f32,
    pub branching_bias: f32,
    pub resilience: f32,
}

impl Default for InferenceProfile {
    fn default() -> Self {
        Self {
            growth_bias: 1.0,
            mobility_bias: 0.5,
            branching_bias: 0.5,
            resilience: DEFAULT_INFERENCE_RESILIENCE,
        }
    }
}

impl InferenceProfile {
    pub fn new(growth_bias: f32, mobility_bias: f32, branching_bias: f32, resilience: f32) -> Self {
        Self {
            growth_bias: sanitize_norm(growth_bias),
            mobility_bias: sanitize_norm(mobility_bias),
            branching_bias: sanitize_norm(branching_bias),
            resilience: sanitize_norm(resilience),
        }
    }

    /// `resilience` con fallback al default del perfil si el componente falta (EA4, EA7, …).
    #[inline]
    pub fn resilience_effective(profile: Option<&Self>) -> f32 {
        profile.map(|p| p.resilience).unwrap_or(DEFAULT_INFERENCE_RESILIENCE)
    }
}

/// Capacidades ejecutables por reducer; evita etiquetas fijas "planta/animal".
#[derive(Component, Reflect, Debug, Clone, Copy, PartialEq, Eq)]
#[reflect(Component, PartialEq)]
pub struct CapabilitySet {
    pub flags: u8,
}

impl Default for CapabilitySet {
    fn default() -> Self {
        Self {
            flags: Self::GROW | Self::MOVE | Self::BRANCH | Self::ROOT,
        }
    }
}

impl CapabilitySet {
    pub const GROW: u8 = 1 << 0;
    pub const MOVE: u8 = 1 << 1;
    pub const BRANCH: u8 = 1 << 2;
    pub const ROOT: u8 = 1 << 3;
    pub const SENSE: u8 = 1 << 4;
    pub const ARMOR: u8 = 1 << 5;
    pub const REPRODUCE: u8 = 1 << 6;
    pub const PHOTOSYNTH: u8 = 1 << 7;

    pub fn new(flags: u8) -> Self {
        Self { flags }
    }

    #[inline]
    pub fn has(self, capability: u8) -> bool {
        self.flags & capability != 0
    }

    #[inline]
    pub fn can_grow(self) -> bool {
        self.has(Self::GROW)
    }

    #[inline]
    pub fn can_branch(self) -> bool {
        self.has(Self::BRANCH)
    }

    #[inline]
    pub fn can_sense(self) -> bool {
        self.has(Self::SENSE)
    }

    #[inline]
    pub fn can_armor(self) -> bool {
        self.has(Self::ARMOR)
    }

    #[inline]
    pub fn can_reproduce(self) -> bool {
        self.has(Self::REPRODUCE)
    }

    #[inline]
    pub fn can_photosynth(self) -> bool {
        self.has(Self::PHOTOSYNTH)
    }
}

/// Intención de crecimiento inferida (transient): separación inferencia/reducción.
#[derive(Component, Reflect, Debug, Clone, Copy, PartialEq)]
#[reflect(Component, PartialEq)]
#[component(storage = "SparseSet")]
pub struct GrowthIntent {
    pub delta_radius: f32,
    pub confidence: f32,
    pub structural_stability: f32,
}

impl GrowthIntent {
    pub fn new(delta_radius: f32, confidence: f32, structural_stability: f32) -> Self {
        Self {
            delta_radius: sanitize_non_negative(delta_radius),
            confidence: sanitize_norm(confidence),
            structural_stability: sanitize_norm(structural_stability),
        }
    }
}

/// D8: Marker for organ manifest re-inference after significant InferenceProfile change.
#[derive(Component, Reflect, Debug, Clone, Copy, Default)]
#[reflect(Component)]
#[component(storage = "SparseSet")]
pub struct PendingMorphRebuild;

/// Clase trófica para transformaciones energéticas data-driven.
#[repr(u8)]
#[derive(Reflect, Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum TrophicClass {
    #[default]
    PrimaryProducer = 0,
    Herbivore = 1,
    Omnivore = 2,
    Carnivore = 3,
    Detritivore = 4,
}

/// Contrato energético genérico para entidades vivas sin branch por especie.
#[derive(Reflect, Debug, Clone, Copy, PartialEq)]
pub struct AnimalSpec {
    pub trophic: TrophicClass,
    pub metabolic_efficiency: f32,
    pub mobility_bias: f32,
    pub armor_bias: f32,
    pub sensor_bias: f32,
    pub reproduction_bias: f32,
    pub resilience: f32,
}

impl AnimalSpec {
    pub fn new(
        trophic: TrophicClass,
        metabolic_efficiency: f32,
        mobility_bias: f32,
        armor_bias: f32,
        sensor_bias: f32,
        reproduction_bias: f32,
        resilience: f32,
    ) -> Self {
        Self {
            trophic,
            metabolic_efficiency: sanitize_norm(metabolic_efficiency),
            mobility_bias: sanitize_norm(mobility_bias),
            armor_bias: sanitize_norm(armor_bias),
            sensor_bias: sanitize_norm(sensor_bias),
            reproduction_bias: sanitize_norm(reproduction_bias),
            resilience: sanitize_norm(resilience),
        }
    }
}

/// Contexto exógeno normalizado para sandbox ambiental.
#[derive(Reflect, Debug, Clone, Copy, PartialEq)]
pub struct EnvContext {
    pub food_density_t: f32,
    pub predation_pressure_t: f32,
    pub temperature_t: f32,
    pub medium_density_t: f32,
    pub competition_t: f32,
}

impl EnvContext {
    pub fn new(
        food_density_t: f32,
        predation_pressure_t: f32,
        temperature_t: f32,
        medium_density_t: f32,
        competition_t: f32,
    ) -> Self {
        Self {
            food_density_t: sanitize_norm(food_density_t),
            predation_pressure_t: sanitize_norm(predation_pressure_t),
            temperature_t: sanitize_norm(temperature_t),
            medium_density_t: sanitize_norm(medium_density_t),
            competition_t: sanitize_norm(competition_t),
        }
    }
}

#[inline]
fn sanitize_non_negative(value: f32) -> f32 {
    if value.is_finite() {
        value.max(0.0)
    } else {
        0.0
    }
}

#[inline]
fn sanitize_norm(value: f32) -> f32 {
    if value.is_finite() {
        value.clamp(0.0, 1.0)
    } else {
        0.0
    }
}

#[cfg(test)]
mod tests {
    use super::{AnimalSpec, CapabilitySet, EnvContext, GrowthIntent, InferenceProfile, TrophicClass};

    #[test]
    fn inference_profile_new_clamps_biases() {
        let p = InferenceProfile::new(-1.0, 2.0, f32::NAN, 0.7);
        assert_eq!(p.growth_bias, 0.0);
        assert_eq!(p.mobility_bias, 1.0);
        assert_eq!(p.branching_bias, 0.0);
        assert_eq!(p.resilience, 0.7);
    }

    #[test]
    fn capability_set_default_enables_growth() {
        assert!(CapabilitySet::default().can_grow());
    }

    #[test]
    fn capability_set_branch_flag() {
        let b = CapabilitySet::new(CapabilitySet::BRANCH);
        assert!(b.can_branch());
        let r = CapabilitySet::new(CapabilitySet::ROOT);
        assert!(!r.can_branch());
    }

    #[test]
    fn capability_set_extended_flags_do_not_collide() {
        let all_bits = CapabilitySet::GROW
            | CapabilitySet::MOVE
            | CapabilitySet::BRANCH
            | CapabilitySet::ROOT
            | CapabilitySet::SENSE
            | CapabilitySet::ARMOR
            | CapabilitySet::REPRODUCE
            | CapabilitySet::PHOTOSYNTH;
        assert_eq!(all_bits, u8::MAX);
    }

    #[test]
    fn capability_set_can_sense_respects_flag() {
        assert!(CapabilitySet::new(CapabilitySet::SENSE).can_sense());
        assert!(!CapabilitySet::new(CapabilitySet::ROOT).can_sense());
    }

    #[test]
    fn capability_set_default_does_not_enable_new_li1_flags() {
        let default = CapabilitySet::default();
        assert!(!default.can_sense());
        assert!(!default.can_armor());
        assert!(!default.can_reproduce());
        assert!(!default.can_photosynth());
    }

    #[test]
    fn capability_set_has_matches_specific_methods() {
        let set = CapabilitySet::new(CapabilitySet::ARMOR | CapabilitySet::PHOTOSYNTH);
        assert_eq!(set.has(CapabilitySet::ARMOR), set.can_armor());
        assert_eq!(set.has(CapabilitySet::PHOTOSYNTH), set.can_photosynth());
        assert_eq!(set.has(CapabilitySet::REPRODUCE), set.can_reproduce());
    }

    #[test]
    fn growth_intent_new_sanitizes_fields() {
        let i = GrowthIntent::new(-10.0, 3.0, f32::NAN);
        assert_eq!(i.delta_radius, 0.0);
        assert_eq!(i.confidence, 1.0);
        assert_eq!(i.structural_stability, 0.0);
    }

    #[test]
    fn resilience_effective_none_matches_default() {
        assert_eq!(
            InferenceProfile::resilience_effective(None),
            InferenceProfile::default().resilience
        );
        let p = InferenceProfile::new(0.5, 0.0, 0.5, 0.82);
        assert_eq!(InferenceProfile::resilience_effective(Some(&p)), 0.82);
    }

    #[test]
    fn animal_spec_new_clamps_fields_to_unit_interval() {
        let spec = AnimalSpec::new(TrophicClass::Carnivore, -1.0, 2.0, f32::NAN, 0.5, 3.0, -0.2);
        assert_eq!(spec.trophic, TrophicClass::Carnivore);
        assert_eq!(spec.metabolic_efficiency, 0.0);
        assert_eq!(spec.mobility_bias, 1.0);
        assert_eq!(spec.armor_bias, 0.0);
        assert_eq!(spec.sensor_bias, 0.5);
        assert_eq!(spec.reproduction_bias, 1.0);
        assert_eq!(spec.resilience, 0.0);
    }

    #[test]
    fn env_context_new_clamps_all_signals() {
        let ctx = EnvContext::new(1.2, -1.0, f32::NAN, 0.4, 2.0);
        assert_eq!(ctx.food_density_t, 1.0);
        assert_eq!(ctx.predation_pressure_t, 0.0);
        assert_eq!(ctx.temperature_t, 0.0);
        assert_eq!(ctx.medium_density_t, 0.4);
        assert_eq!(ctx.competition_t, 1.0);
    }
}
