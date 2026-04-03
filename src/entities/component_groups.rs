//! Component group factories — pure functions returning tuples for entity composition.
//!
//! Each factory is stateless, pure, and composable via Bevy's nested tuple bundles.
//! Use these instead of manually listing 8-12 components at each spawn site.
//!
//! ```rust,ignore
//! commands.spawn((
//!     physical_components(qe, radius, freq, pos),
//!     behavior_components(caps, profile),
//!     trophic_components(TrophicClass::Herbivore, 12.0),
//!     lifecycle_components(clock.tick_id, true),
//! ));
//! ```

use bevy::prelude::*;

use crate::blueprint::constants::SENESCENCE_DEFAULT_STRATEGY;
use crate::blueprint::equations::{
    bond_from_energy, conductivity_from_state, dissipation_from_state, matter_state_from_density,
};
use crate::layers::organ::LifecycleStageCache;
use crate::layers::senescence::SenescenceProfile;
use crate::layers::{
    AlchemicalEngine, BaseEnergy, BehaviorCooldown, BehaviorIntent, BehavioralAgent, CacheScope,
    CapabilitySet, FlowVector, HasInferredShape, InferenceProfile, MatterCoherence,
    MorphogenesisShapeParams, OscillatorySignature, PerformanceCachePolicy, SpatialVolume,
    TrophicClass, TrophicConsumer, TrophicState, WillActuator,
};
use crate::math_types::Vec2;

// ─── Physical: existence in space (L0-L4 + Transform) ────────────────────────

/// Core physical existence: energy, volume, frequency, flow, matter, position.
/// Every entity needs this. No behavior, no lifecycle — just physics.
pub fn physical_components(
    qe: f32,
    radius: f32,
    frequency_hz: f32,
    position: Vec2,
) -> (
    BaseEnergy,
    SpatialVolume,
    OscillatorySignature,
    FlowVector,
    MatterCoherence,
    Transform,
    GlobalTransform,
) {
    let volume = radius.max(0.01);
    let _density = qe / (volume * volume);
    let state = matter_state_from_density(qe, volume * volume);
    let bond = bond_from_energy(qe);
    let conductivity = conductivity_from_state(state);
    let dissipation = dissipation_from_state(state);
    (
        BaseEnergy::new(qe),
        SpatialVolume::new(radius),
        OscillatorySignature::new(frequency_hz, 0.0),
        FlowVector::new(Vec2::ZERO, dissipation),
        MatterCoherence::new(state, bond, conductivity),
        Transform::from_xyz(position.x, 0.0, position.y),
        GlobalTransform::default(),
    )
}

// ─── Behavior: decision-making agent (L7 + D1) ──────────────────────────────

/// Full behavioral stack: agent marker, intent, cooldown, capabilities, profile.
/// Entities with this can move, sense, forage, hunt, flee.
pub fn behavior_components(
    caps: u8,
    profile: InferenceProfile,
) -> (
    BehavioralAgent,
    BehaviorIntent,
    BehaviorCooldown,
    CapabilitySet,
    InferenceProfile,
) {
    (
        BehavioralAgent,
        BehaviorIntent::default(),
        BehaviorCooldown::default(),
        CapabilitySet::new(caps),
        profile,
    )
}

// ─── Trophic: feeding chain ──────────────────────────────────────────────────

/// Trophic consumer + satiation state. Enables foraging and predation.
pub fn trophic_components(
    class: TrophicClass,
    intake_rate: f32,
) -> (TrophicConsumer, TrophicState) {
    (
        TrophicConsumer::new(class, intake_rate),
        TrophicState::new(0.5),
    )
}

// ─── Lifecycle: aging + morphology ───────────────────────────────────────────

/// Senescence + shape inference + morphology params.
/// `is_mobile`: true = fauna (shorter life, faster turnover), false = flora/terrain.
pub fn lifecycle_components(
    tick_birth: u64,
    is_mobile: bool,
) -> (
    SenescenceProfile,
    HasInferredShape,
    LifecycleStageCache,
    MorphogenesisShapeParams,
    PerformanceCachePolicy,
) {
    let coeff = if is_mobile {
        crate::blueprint::constants::senescence_coeff_fauna()
    } else {
        crate::blueprint::constants::senescence_coeff_flora()
    };
    let max_age = if is_mobile {
        crate::blueprint::constants::senescence_max_age_fauna()
    } else {
        crate::blueprint::constants::senescence_max_age_flora()
    };
    (
        SenescenceProfile {
            tick_birth,
            senescence_coeff: coeff,
            max_viable_age: max_age,
            strategy: SENESCENCE_DEFAULT_STRATEGY,
        },
        HasInferredShape,
        LifecycleStageCache::default(),
        MorphogenesisShapeParams::default(),
        PerformanceCachePolicy {
            enabled: true,
            scope: CacheScope::StableWindow,
            version_tag: 1,
            dependency_signature: 0,
        },
    )
}

// ─── Motor: movement physics (L5 + L7) ──────────────────────────────────────

/// Alchemical engine (energy buffer) + will actuator (movement intent).
/// Enables physical movement through the world.
pub fn motor_components(
    buf_max: f32,
    input_valve: f32,
    output_valve: f32,
    initial_buffer: f32,
) -> (AlchemicalEngine, WillActuator) {
    (
        AlchemicalEngine::new(buf_max, input_valve, output_valve, initial_buffer),
        WillActuator::default(),
    )
}

// ─── Terrain: materialized tile senescence ───────────────────────────────────

/// Senescence for terrain tiles (longer life, slower metabolism).
pub fn terrain_senescence(tick_birth: u64) -> SenescenceProfile {
    SenescenceProfile {
        tick_birth,
        senescence_coeff: crate::blueprint::constants::senescence_coeff_materialized(),
        max_viable_age: crate::blueprint::constants::senescence_max_age_materialized(),
        strategy: SENESCENCE_DEFAULT_STRATEGY,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn physical_components_produces_valid_energy() {
        let (energy, _, _, _, _, _, _) = physical_components(100.0, 1.0, 85.0, Vec2::ZERO);
        assert!((energy.qe() - 100.0).abs() < 1e-5);
    }

    #[test]
    fn behavior_components_includes_agent_marker() {
        let (agent, _, _, caps, _) =
            behavior_components(0x03, InferenceProfile::new(0.5, 0.5, 0.5, 0.5));
        let _ = agent; // BehavioralAgent is a unit struct
        assert!(caps.has(0x01)); // GROW
        assert!(caps.has(0x02)); // MOVE
    }

    #[test]
    fn lifecycle_fauna_shorter_than_flora() {
        let (fauna_sen, _, _, _, _) = lifecycle_components(0, true);
        let (flora_sen, _, _, _, _) = lifecycle_components(0, false);
        assert!(fauna_sen.max_viable_age < flora_sen.max_viable_age);
    }

    #[test]
    fn terrain_senescence_longest() {
        let terrain = terrain_senescence(0);
        let (fauna, _, _, _, _) = lifecycle_components(0, true);
        assert!(terrain.max_viable_age > fauna.max_viable_age);
    }

    #[test]
    fn trophic_components_initial_satiation() {
        let (_, state) = trophic_components(TrophicClass::Herbivore, 12.0);
        assert!((state.satiation - 0.5).abs() < 1e-5);
    }
}
