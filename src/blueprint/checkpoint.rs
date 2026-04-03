use serde::{Deserialize, Serialize};

use crate::blueprint::MatterState;

/// Schema version for checkpoint format.
pub const CHECKPOINT_VERSION: u32 = 1;
pub const CHECKPOINT_FILE_EXTENSION_RON: &str = ".checkpoint.ron";
pub const CHECKPOINT_FILE_EXTENSION_JSON: &str = ".checkpoint.json";

/// Serializable checkpoint of the full world state.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct WorldCheckpoint {
    pub version: u32,
    pub tick: u64,
    pub map_name: String,
    pub entities: Vec<EntitySnapshot>,
}

/// Snapshot of an entity with its core layer data.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct EntitySnapshot {
    pub id: u32,
    pub position: [f32; 3],
    pub energy: f32,
    pub radius: f32,
    pub frequency: f32,
    pub phase: f32,
    pub matter_state: u8,
    pub bond_energy: f32,
}

/// Builds a WorldCheckpoint from raw slices of entity data (pure, no ECS).
pub fn build_checkpoint(
    tick: u64,
    map_name: &str,
    snapshots: &[EntitySnapshot],
) -> WorldCheckpoint {
    WorldCheckpoint {
        version: CHECKPOINT_VERSION,
        tick,
        map_name: map_name.to_string(),
        entities: snapshots.to_vec(),
    }
}

/// Encodes a MatterState as u8 for snapshot storage.
pub fn matter_state_to_u8(state: MatterState) -> u8 {
    match state {
        MatterState::Solid => 0,
        MatterState::Liquid => 1,
        MatterState::Gas => 2,
        MatterState::Plasma => 3,
    }
}

/// Decodes a u8 back to MatterState; defaults to Solid for unknown values.
pub fn u8_to_matter_state(v: u8) -> MatterState {
    match v {
        0 => MatterState::Solid,
        1 => MatterState::Liquid,
        2 => MatterState::Gas,
        3 => MatterState::Plasma,
        _ => MatterState::Solid,
    }
}

/// Serializes checkpoint to RON string.
pub fn checkpoint_to_ron(checkpoint: &WorldCheckpoint) -> Result<String, ron::Error> {
    ron::ser::to_string_pretty(checkpoint, ron::ser::PrettyConfig::default())
}

/// Deserializes checkpoint from RON string.
pub fn checkpoint_from_ron(data: &str) -> Result<WorldCheckpoint, ron::error::SpannedError> {
    ron::from_str(data)
}

/// Serializes checkpoint to JSON string.
pub fn checkpoint_to_json(checkpoint: &WorldCheckpoint) -> Result<String, serde_json::Error> {
    serde_json::to_string_pretty(checkpoint)
}

/// Deserializes checkpoint from JSON string.
pub fn checkpoint_from_json(data: &str) -> Result<WorldCheckpoint, serde_json::Error> {
    serde_json::from_str(data)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_snapshot() -> EntitySnapshot {
        EntitySnapshot {
            id: 42,
            position: [1.0, 2.0, 3.0],
            energy: 100.0,
            radius: 0.5,
            frequency: 75.0,
            phase: 1.57,
            matter_state: matter_state_to_u8(MatterState::Liquid),
            bond_energy: 500.0,
        }
    }

    fn sample_checkpoint() -> WorldCheckpoint {
        build_checkpoint(
            10,
            "test_map",
            &[
                sample_snapshot(),
                EntitySnapshot {
                    id: 7,
                    position: [0.0, 0.0, 0.0],
                    energy: 50.0,
                    radius: 1.0,
                    frequency: 200.0,
                    phase: 0.0,
                    matter_state: matter_state_to_u8(MatterState::Solid),
                    bond_energy: 1000.0,
                },
                EntitySnapshot {
                    id: 99,
                    position: [-5.0, 10.0, 0.5],
                    energy: 0.0,
                    radius: 0.1,
                    frequency: 900.0,
                    phase: 3.14,
                    matter_state: matter_state_to_u8(MatterState::Plasma),
                    bond_energy: 0.0,
                },
            ],
        )
    }

    #[test]
    fn build_checkpoint_sets_version_and_entity_count() {
        let cp = sample_checkpoint();
        assert_eq!(cp.version, CHECKPOINT_VERSION);
        assert_eq!(cp.tick, 10);
        assert_eq!(cp.map_name, "test_map");
        assert_eq!(cp.entities.len(), 3);
    }

    #[test]
    fn ron_roundtrip_preserves_checkpoint() {
        let cp = sample_checkpoint();
        let ron_str = checkpoint_to_ron(&cp).expect("serialize to RON");
        let restored = checkpoint_from_ron(&ron_str).expect("deserialize from RON");
        assert_eq!(cp, restored);
    }

    #[test]
    fn json_roundtrip_preserves_checkpoint() {
        let cp = sample_checkpoint();
        let json_str = checkpoint_to_json(&cp).expect("serialize to JSON");
        let restored = checkpoint_from_json(&json_str).expect("deserialize from JSON");
        assert_eq!(cp, restored);
    }

    #[test]
    fn matter_state_roundtrip_all_variants() {
        for (state, expected_u8) in [
            (MatterState::Solid, 0u8),
            (MatterState::Liquid, 1),
            (MatterState::Gas, 2),
            (MatterState::Plasma, 3),
        ] {
            let encoded = matter_state_to_u8(state);
            assert_eq!(encoded, expected_u8);
            let decoded = u8_to_matter_state(encoded);
            assert_eq!(decoded, state);
        }
    }

    #[test]
    fn u8_to_matter_state_unknown_defaults_to_solid() {
        assert_eq!(u8_to_matter_state(255), MatterState::Solid);
        assert_eq!(u8_to_matter_state(4), MatterState::Solid);
    }

    #[test]
    fn snapshot_energy_roundtrip_via_ron() {
        let snap = sample_snapshot();
        let cp = build_checkpoint(0, "rt", &[snap.clone()]);
        let ron_str = checkpoint_to_ron(&cp).expect("ron");
        let restored = checkpoint_from_ron(&ron_str).expect("from ron");
        let restored_snap = &restored.entities[0];
        assert!((restored_snap.energy - snap.energy).abs() < 1e-5);
        assert!((restored_snap.frequency - snap.frequency).abs() < 1e-5);
        assert!((restored_snap.phase - snap.phase).abs() < 1e-5);
        assert_eq!(restored_snap.matter_state, snap.matter_state);
    }

    #[test]
    fn empty_checkpoint_roundtrip() {
        let cp = build_checkpoint(0, "empty", &[]);
        let ron_str = checkpoint_to_ron(&cp).expect("ron");
        let restored = checkpoint_from_ron(&ron_str).expect("from ron");
        assert_eq!(restored.entities.len(), 0);
        assert_eq!(restored.version, CHECKPOINT_VERSION);
    }

    #[test]
    fn json_produces_parseable_output() {
        let cp = sample_checkpoint();
        let json_str = checkpoint_to_json(&cp).expect("json");
        assert!(json_str.contains("\"version\""));
        assert!(json_str.contains("\"test_map\""));
        assert!(json_str.contains("\"entities\""));
    }
}
