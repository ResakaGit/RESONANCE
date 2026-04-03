use bevy::prelude::*;

/// Active behavioral mode with variant-specific data.
#[derive(Reflect, Debug, Clone, PartialEq, Default)]
pub enum BehaviorMode {
    #[default]
    Idle,
    Forage {
        urgency: f32,
    },
    Hunt {
        prey: Entity,
        chase_ticks: u32,
    },
    Flee {
        threat: Entity,
    },
    Reproduce,
    Migrate {
        direction: Vec2,
    },
    /// Nash-optimal focus: the whole team converges on the easiest-to-eliminate target.
    FocusFire {
        target: Entity,
        team_priority: u8,
    },
    /// Tactical regrouping toward a rally position.
    Regroup {
        rally_pos: Vec2,
    },
}

/// Current behavioral decision for an autonomous entity.
#[derive(Component, Reflect, Debug, Clone)]
#[reflect(Component)]
pub struct BehaviorIntent {
    pub mode: BehaviorMode,
    pub target_entity: Option<Entity>,
}

impl Default for BehaviorIntent {
    fn default() -> Self {
        Self {
            mode: BehaviorMode::Idle,
            target_entity: None,
        }
    }
}

/// Marker: entity participates in the behavior decision pipeline.
#[derive(Component, Default, Reflect, Debug, Clone, Copy)]
#[reflect(Component)]
pub struct BehavioralAgent;

/// Tick counters for behavior decision gating.
#[derive(Component, Reflect, Debug, Clone)]
#[reflect(Component)]
#[derive(Default)]
pub struct BehaviorCooldown {
    pub decision_cooldown: u32,
    pub action_cooldown: u32,
}

/// Transient cache: internal energy assessment (S1 → S3).
#[derive(Component, Reflect, Debug, Clone, Copy)]
#[reflect(Component)]
#[component(storage = "SparseSet")]
pub struct EnergyAssessment {
    pub hunger_fraction: f32,
    pub energy_ratio: f32,
    pub biomass: f32,
}

/// Transient cache: spatial awareness of hostile and food entities (S2 → S3).
#[derive(Component, Reflect, Debug, Clone, Copy)]
#[reflect(Component)]
#[component(storage = "SparseSet")]
pub struct SensoryAwareness {
    pub hostile_entity: Option<Entity>,
    pub hostile_distance: f32,
    pub food_entity: Option<Entity>,
    pub food_distance: f32,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn behavior_intent_default_is_idle() {
        let intent = BehaviorIntent::default();
        assert_eq!(intent.mode, BehaviorMode::Idle);
        assert!(intent.target_entity.is_none());
    }

    #[test]
    fn behavior_cooldown_default_fires_immediately() {
        let cd = BehaviorCooldown::default();
        assert_eq!(cd.decision_cooldown, 0);
        assert_eq!(cd.action_cooldown, 0);
    }

    #[test]
    fn behavior_mode_partial_eq() {
        assert_eq!(BehaviorMode::Idle, BehaviorMode::Idle);
        assert_ne!(
            BehaviorMode::Forage { urgency: 0.5 },
            BehaviorMode::Forage { urgency: 0.3 },
        );
        assert_ne!(BehaviorMode::Idle, BehaviorMode::Reproduce);
    }
}
