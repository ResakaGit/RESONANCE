use std::fs;
use std::path::PathBuf;

use bevy::prelude::{Entity, Vec2};

use crate::layers::compute_interference_total;
use crate::runtime_platform::contracts::{Pose2, SpatialCandidatePair};
use crate::runtime_platform::spatial_index_backend::{
    Grid2DSpatialBroadphase, SpatialBroadphase, SpatialPose,
};

fn project_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn read(path: &str) -> String {
    let full = project_root().join(path);
    fs::read_to_string(&full).unwrap_or_else(|e| panic!("cannot read {}: {e}", full.display()))
}

fn lcg_step(state: &mut u64) -> u64 {
    *state = state.wrapping_mul(6364136223846793005).wrapping_add(1);
    *state
}

fn replay_hash(seed: u64) -> u64 {
    let mut rng = seed;
    let mut hash = 0_u64;
    let mut grid = Grid2DSpatialBroadphase::new(5.0);

    for tick in 0..30_u64 {
        grid.clear();
        for i in 0..32_u32 {
            let rx = ((lcg_step(&mut rng) >> 16) % 10_000) as f32 / 200.0 - 25.0;
            let ry = ((lcg_step(&mut rng) >> 16) % 10_000) as f32 / 200.0 - 25.0;
            let r = 0.2 + (((lcg_step(&mut rng) >> 16) % 100) as f32 / 250.0);
            grid.insert(
                SpatialPose::Pose2(Pose2::new(Vec2::new(rx, ry), r)),
                Entity::from_raw(i + 1),
                r,
            );
        }

        let pairs = grid.candidate_pairs();
        for pair in pairs {
            hash ^= pair.a.to_bits().wrapping_mul(31);
            hash ^= pair.b.to_bits().wrapping_mul(131);
            hash = hash.rotate_left(7);
        }

        let interf =
            compute_interference_total(450.0, 0.3, 700.0, 1.1, tick as f32 * (1.0 / 60.0), 0.05);
        hash ^= interf.to_bits() as u64;
    }

    hash
}

#[test]
fn determinism_replay_same_seed_same_state() {
    let a = replay_hash(0xA11CE);
    let b = replay_hash(0xA11CE);
    assert_eq!(a, b);
}

#[test]
fn collision_order_invariant() {
    let mk_entries = || {
        vec![
            (Entity::from_raw(1), Vec2::new(0.0, 0.0), 1.1),
            (Entity::from_raw(2), Vec2::new(0.7, 0.0), 1.1),
            (Entity::from_raw(3), Vec2::new(1.4, 0.0), 1.1),
            (Entity::from_raw(4), Vec2::new(2.2, 0.0), 1.1),
        ]
    };

    let mut left = Grid2DSpatialBroadphase::new(5.0);
    for (e, p, r) in mk_entries() {
        left.insert(SpatialPose::Pose2(Pose2::new(p, r)), e, r);
    }

    let mut right = Grid2DSpatialBroadphase::new(5.0);
    for (e, p, r) in mk_entries().into_iter().rev() {
        right.insert(SpatialPose::Pose2(Pose2::new(p, r)), e, r);
    }

    let lp = left.candidate_pairs();
    let rp = right.candidate_pairs();
    assert_eq!(lp, rp);
    assert!(lp.contains(&SpatialCandidatePair::new(
        Entity::from_raw(1),
        Entity::from_raw(2)
    )));
}

#[test]
fn reactions_use_sim_clock() {
    let reactions = read("src/simulation/reactions.rs");
    assert!(
        !reactions.contains("elapsed_secs("),
        "reactions must not use wall-clock"
    );
    assert!(
        reactions.contains("SimulationElapsed"),
        "reactions must depend on SimulationElapsed"
    );
}

#[test]
fn input_phase_no_gameplay_mutation() {
    let input = read("src/simulation/input.rs");
    let start = input
        .find("pub fn grimoire_cast_intent_system")
        .expect("missing grimoire_cast_intent_system");
    let end = input
        .find("pub fn grimoire_cast_resolve_system")
        .expect("missing grimoire_cast_resolve_system");
    let intent_section = &input[start..end];

    assert!(
        !intent_section.contains("spawn_projectile("),
        "Input intent must not spawn entities"
    );
    assert!(
        !intent_section.contains("engine.current_buffer -="),
        "Input intent must not consume engine buffer"
    );
    assert!(
        intent_section.contains("EventWriter<GrimoireProjectileCastPending>"),
        "Input intent must emit event"
    );
}

#[test]
fn component_field_budget_guard() {
    let layers_dir = project_root().join("src/layers");
    let entries = fs::read_dir(&layers_dir)
        .unwrap_or_else(|e| panic!("cannot read {}: {e}", layers_dir.display()));

    for entry in entries {
        let path = entry.expect("entry").path();
        if path.extension().and_then(|e| e.to_str()) != Some("rs") {
            continue;
        }
        let src = fs::read_to_string(&path)
            .unwrap_or_else(|e| panic!("cannot read {}: {e}", path.display()));
        let lines: Vec<&str> = src.lines().collect();

        let mut i = 0usize;
        while i < lines.len() {
            if lines[i].contains("#[derive(Component") {
                let mut j = i + 1;
                while j < lines.len() && !lines[j].contains("pub struct ") {
                    j += 1;
                }
                if j >= lines.len() {
                    break;
                }
                let header = lines[j].trim();
                if !header.contains('{') {
                    i = j + 1;
                    continue;
                }
                let name = header
                    .strip_prefix("pub struct ")
                    .and_then(|s| s.split('{').next())
                    .map(str::trim)
                    .unwrap_or("UnknownComponent");
                let mut fields = 0usize;
                j += 1;
                while j < lines.len() {
                    let ln = lines[j].trim();
                    if ln.starts_with('}') {
                        break;
                    }
                    if ln.starts_with("pub ") && ln.contains(':') {
                        fields += 1;
                    }
                    j += 1;
                }
                assert!(
                    fields <= 4,
                    "Component {} in {} exceeds DoD field budget: {} fields",
                    name,
                    path.display(),
                    fields
                );
                i = j;
            }
            i += 1;
        }
    }
}

#[test]
fn hex_boundary_render_snapshot_only() {
    let render = read("src/runtime_platform/render_bridge_3d/mod.rs");
    let start = render
        .find("fn sync_visual_from_sim_system")
        .expect("missing sync_visual_from_sim_system");
    let end = render[start..]
        .find("// @hex_boundary:sync_visual_end")
        .map(|i| start + i)
        .expect("missing @hex_boundary:sync_visual_end marker after sync_visual_from_sim_system");
    let sync = &render[start..end];

    assert!(
        sync.contains("snapshot: Res<V6RenderSnapshot>"),
        "sync visual must consume explicit snapshot"
    );
    assert!(
        !sync.contains("&BaseEnergy")
            && !sync.contains("&MatterCoherence")
            && !sync.contains("&AlchemicalInjector"),
        "sync visual must not read core layers directly"
    );
}

#[test]
fn intent_projection_is_adapter_only() {
    let projection = read("src/runtime_platform/intent_projection_3d/mod.rs");
    let start = projection
        .find("pub fn project_intent_to_resource_system")
        .expect("missing project_intent_to_resource_system");
    let end = projection
        .find("pub fn apply_projected_intent_to_will_system")
        .expect("missing apply_projected_intent_to_will_system");
    let only_projection = &projection[start..end];

    assert!(
        !only_projection.contains("Query<&mut WillActuator"),
        "projection adapter must not mutate Layer 7"
    );
}
