use serde::{Deserialize, Serialize};

use crate::layers::will::{AbilityCastSpec, AbilityOutput, TargetingMode};

/// Ability serializable que viaja como dato puro (MCP/IA -> runtime).
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct AbilityDef {
    pub name: String,
    pub description: String,
    pub cost_qe: f32,
    pub cooldown_estimate: f32,
    pub output: AbilityOutput,
    pub frequency_override: Option<f32>,
    pub phase_offset: Option<f32>,
    pub max_range: f32,
    pub max_duration_s: f32,
    pub validated: bool,
    pub checksum: u32,
}

impl AbilityDef {
    pub fn to_slot(&self) -> crate::layers::will::AbilitySlot {
        let targeting = match &self.output {
            AbilityOutput::SelfBuff { .. } => TargetingMode::NoTarget,
            _ => TargetingMode::PointTarget {
                range: self.max_range.max(1.0),
            },
        };
        crate::layers::will::AbilitySlot {
            name: self.name.clone(),
            output: self.output.clone(),
            cast: AbilityCastSpec {
                cost_qe: self.cost_qe,
                targeting,
                min_channeling_secs: 0.0,
            },
        }
    }
}
