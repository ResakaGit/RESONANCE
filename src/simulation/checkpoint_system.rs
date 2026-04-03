//! SF-5: Checkpoint Save/Load systems — on-demand save, Startup load.
//! Pure math lives in `blueprint/checkpoint.rs`. These systems wire it to ECS.

use bevy::prelude::*;

use crate::blueprint::checkpoint::{
    EntitySnapshot, build_checkpoint, checkpoint_from_json, checkpoint_from_ron,
    checkpoint_to_json, checkpoint_to_ron, matter_state_to_u8, u8_to_matter_state,
};
use crate::blueprint::ids::WorldEntityId;
use crate::layers::{BaseEnergy, MatterCoherence, OscillatorySignature, SpatialVolume};
use crate::runtime_platform::simulation_tick::SimulationClock;
use crate::worldgen::map_config::ActiveMapName;

// ─── Types ────────────────────────────────────────────────────────────────────

/// Checkpoint serialisation format.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CheckpointFormat {
    Ron,
    Json,
}

/// SF-5A: Checkpoint configuration. Inserted only if env vars are present.
#[derive(Resource, Debug, Clone)]
pub struct CheckpointConfig {
    /// 0 = manual only; >0 = save every N ticks.
    pub save_interval: u32,
    pub output_dir: String,
    /// If Some, load this path at startup.
    pub load_path: Option<String>,
    pub format: CheckpointFormat,
}

impl Default for CheckpointConfig {
    fn default() -> Self {
        Self {
            save_interval: 0,
            output_dir: "/tmp".to_string(),
            load_path: None,
            format: CheckpointFormat::Ron,
        }
    }
}

impl CheckpointConfig {
    /// Build from environment variables. Returns None if no env vars are set.
    pub fn from_env() -> Option<Self> {
        let save_interval = std::env::var("RESONANCE_CHECKPOINT_SAVE")
            .ok()
            .and_then(|s| s.parse::<u32>().ok())
            .unwrap_or(0);
        let load_path = std::env::var("RESONANCE_CHECKPOINT_LOAD").ok();
        let output_dir =
            std::env::var("RESONANCE_CHECKPOINT_DIR").unwrap_or_else(|_| "/tmp".to_string());

        if save_interval == 0 && load_path.is_none() {
            return None;
        }
        Some(Self {
            save_interval,
            output_dir,
            load_path,
            format: CheckpointFormat::Ron,
        })
    }
}

// ─── Systems ─────────────────────────────────────────────────────────────────

/// SF-5B: Saves a checkpoint to disk when the interval fires.
/// Phase::MorphologicalLayer — state is fully settled.
pub fn checkpoint_save_system(
    config: Option<Res<CheckpointConfig>>,
    clock: Res<SimulationClock>,
    map_name: Option<Res<ActiveMapName>>,
    query: Query<(
        Option<&WorldEntityId>,
        &Transform,
        Option<&BaseEnergy>,
        Option<&SpatialVolume>,
        Option<&OscillatorySignature>,
        Option<&MatterCoherence>,
    )>,
    mut last_save: Local<u64>,
) {
    let Some(cfg) = config else {
        return;
    };
    if cfg.save_interval == 0 {
        return;
    }
    if clock.tick_id % cfg.save_interval as u64 != 0 {
        return;
    }
    if clock.tick_id == *last_save {
        return;
    }

    let map = map_name
        .as_deref()
        .map(|m| m.0.as_str())
        .unwrap_or("unknown");

    let mut snapshots: Vec<EntitySnapshot> = query
        .iter()
        .filter_map(
            |(id_opt, transform, energy_opt, volume_opt, osc_opt, coherence_opt)| {
                let Some(energy) = energy_opt else {
                    return None;
                };
                let id = id_opt.map(|w| w.0).unwrap_or(0);
                let pos = transform.translation;
                Some(EntitySnapshot {
                    id,
                    position: [pos.x, pos.y, pos.z],
                    energy: energy.qe(),
                    radius: volume_opt.map(|v| v.radius).unwrap_or(0.5),
                    frequency: osc_opt.map(|o| o.frequency_hz()).unwrap_or(0.0),
                    phase: osc_opt.map(|o| o.phase()).unwrap_or(0.0),
                    matter_state: coherence_opt
                        .map(|c| matter_state_to_u8(c.state()))
                        .unwrap_or(0),
                    bond_energy: coherence_opt.map(|c| c.bond_energy_eb()).unwrap_or(0.0),
                })
            },
        )
        .collect();

    // Deterministic order by entity id.
    snapshots.sort_unstable_by_key(|s| s.id);

    let checkpoint = build_checkpoint(clock.tick_id, map, &snapshots);

    let (content_result, ext) = match cfg.format {
        CheckpointFormat::Ron => (
            checkpoint_to_ron(&checkpoint).map_err(|e| e.to_string()),
            "ron",
        ),
        CheckpointFormat::Json => (
            checkpoint_to_json(&checkpoint).map_err(|e| e.to_string()),
            "json",
        ),
    };
    let Ok(content) = content_result else {
        return;
    };

    let path = format!(
        "{}/resonance_checkpoint_{}.{}",
        cfg.output_dir, clock.tick_id, ext
    );
    if std::fs::write(&path, content).is_ok() {
        info!(
            "checkpoint saved: {} entities, tick {}",
            snapshots.len(),
            clock.tick_id
        );
        *last_save = clock.tick_id;
    }
}

/// SF-5C: Loads a checkpoint at startup if `CheckpointConfig.load_path` is set.
/// Schedule: Startup, before worldgen warmup.
pub fn checkpoint_load_startup_system(
    mut commands: Commands,
    config: Option<Res<CheckpointConfig>>,
    mut clock: ResMut<SimulationClock>,
) {
    let Some(cfg) = config else {
        return;
    };
    let Some(ref path) = cfg.load_path else {
        return;
    };

    let Ok(content) = std::fs::read_to_string(path) else {
        warn!("checkpoint_load_startup: cannot read {path}");
        return;
    };

    let checkpoint = match cfg.format {
        CheckpointFormat::Ron => checkpoint_from_ron(&content).map_err(|e| e.to_string()),
        CheckpointFormat::Json => checkpoint_from_json(&content).map_err(|e| e.to_string()),
    };
    let Ok(cp) = checkpoint else {
        return;
    };

    clock.tick_id = cp.tick;

    let mut spawned = 0u32;
    for snap in &cp.entities {
        let mut entity = commands.spawn(Transform::from_xyz(
            snap.position[0],
            snap.position[1],
            snap.position[2],
        ));
        if snap.energy > 0.0 {
            entity.insert(BaseEnergy::new(snap.energy));
        }
        if snap.radius > 0.0 {
            entity.insert(SpatialVolume::new(snap.radius));
        }
        if snap.frequency > 0.0 {
            entity.insert(OscillatorySignature::new(snap.frequency, snap.phase));
        }
        if snap.bond_energy > 0.0 {
            let state = u8_to_matter_state(snap.matter_state);
            entity.insert(MatterCoherence::new(state, snap.bond_energy, 1.0));
        }
        spawned += 1;
    }

    info!("checkpoint loaded: {} entities, tick {}", spawned, cp.tick);
}
