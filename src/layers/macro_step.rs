use bevy::prelude::*;

/// LOD marker — entity receives O(1) analytical simulation for `ticks_remaining` ticks.
/// Added when an entity moves out of active LOD range; removed by `apply_macro_step`.
#[derive(Component, Reflect, Debug, Clone)]
#[reflect(Component)]
#[component(storage = "SparseSet")]
pub struct MacroStepTarget {
    pub ticks_remaining: u32,
    pub last_qe: f32,
    pub decay_rate: f32,
}

impl MacroStepTarget {
    pub fn new(ticks_remaining: u32, last_qe: f32, decay_rate: f32) -> Self {
        Self { ticks_remaining, last_qe, decay_rate: decay_rate.max(0.0) }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_guards_negative_decay_rate() {
        let s = MacroStepTarget::new(10, 100.0, -0.5);
        assert_eq!(s.decay_rate, 0.0);
    }

    #[test]
    fn new_stores_fields() {
        let s = MacroStepTarget::new(5, 50.0, 0.01);
        assert_eq!(s.ticks_remaining, 5);
        assert!((s.last_qe - 50.0).abs() < 1e-6);
        assert!((s.decay_rate - 0.01).abs() < 1e-6);
    }
}
