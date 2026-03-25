//! Smoke + contrato frente a `docs/DEMO_FLOW.md`: mapas demo parsean, validan y
//! conservan grid/origen/warmup/núcleos esperados.

use std::fs;
use std::path::PathBuf;

use resonance::worldgen::map_config::{MapConfig, parse_map_config, validate_map_config};

fn read_map_ron(name: &str) -> String {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let path = root.join("assets/maps").join(format!("{name}.ron"));
    fs::read_to_string(&path).unwrap_or_else(|e| panic!("leer {path:?}: {e}"))
}

fn load_validated(name: &str) -> MapConfig {
    let raw = read_map_ron(name);
    let cfg = parse_map_config(&raw).unwrap_or_else(|e| panic!("parse {name}: {e}"));
    validate_map_config(&cfg).unwrap_or_else(|errs| panic!("validate {name}: {errs:?}"));
    cfg
}

#[test]
fn demo_minimal_matches_demo_flow_contract() {
    let c = load_validated("demo_minimal");
    assert_eq!(c.width_cells, 10);
    assert_eq!(c.height_cells, 10);
    assert!((c.cell_size - 2.0).abs() < f32::EPSILON);
    assert_eq!(c.origin, [-10.0, -10.0]);
    assert_eq!(c.warmup_ticks, Some(12));
    assert_eq!(c.seed, None);
    assert_eq!(c.nuclei.len(), 1);
    assert_eq!(c.nuclei[0].name, "terra_minimal");
    assert!(c.nuclei[0].ambient_pressure.is_none());
    assert!(c.seasons.is_empty());
}

#[test]
fn demo_floor_matches_demo_flow_contract() {
    let c = load_validated("demo_floor");
    assert_eq!(c.width_cells, 24);
    assert_eq!(c.height_cells, 24);
    assert!((c.cell_size - 2.0).abs() < f32::EPSILON);
    assert_eq!(c.origin, [-24.0, -24.0]);
    assert_eq!(c.seed, Some(42));
    assert_eq!(c.nuclei.len(), 1);
    assert_eq!(c.nuclei[0].name, "terra_demo_day");
    assert!(c.nuclei[0].ambient_pressure.is_some());
    assert!(c.seasons.is_empty());
}

#[test]
fn demo_strata_matches_demo_flow_contract() {
    let c = load_validated("demo_strata");
    assert_eq!(c.width_cells, 40);
    assert_eq!(c.height_cells, 40);
    assert!((c.cell_size - 2.0).abs() < f32::EPSILON);
    assert_eq!(c.origin, [-40.0, -40.0]);
    assert_eq!(c.seed, Some(7));
    assert_eq!(c.nuclei.len(), 2);
    assert_eq!(c.nuclei[0].name, "terra_suelo");
    assert_eq!(c.nuclei[1].name, "ventus_atmosfera");
    assert!((c.nuclei[1].frequency_hz - 700.0).abs() < 0.01);
    assert!(c.nuclei[0].ambient_pressure.is_none());
    assert!(c.nuclei[1].ambient_pressure.is_none());
    assert!(c.seasons.is_empty());
}

#[test]
fn default_map_still_loads() {
    let c = load_validated("default");
    assert!(!c.nuclei.is_empty());
}

#[test]
fn proving_grounds_map_matches_sprint_contract() {
    let c = load_validated("proving_grounds");
    assert_eq!(c.width_cells, 64);
    assert_eq!(c.height_cells, 64);
    assert!((c.cell_size - 2.0).abs() < f32::EPSILON);
    assert_eq!(c.origin, [-64.0, -64.0]);
    assert_eq!(c.warmup_ticks, Some(80));
    assert_eq!(c.seed, Some(314159));
    assert_eq!(c.nuclei.len(), 7);
    let names: Vec<_> = c.nuclei.iter().map(|n| n.name.as_str()).collect();
    assert!(names.contains(&"terra_nexus"));
    assert!(names.contains(&"ignis_forge"));
    assert!(names.contains(&"lux_sanctum"));
    assert!(c.seasons.is_empty());
    assert!(!c.fog_of_war);
}

#[test]
fn demo_river_plateau_map_ok() {
    let c = load_validated("demo_river_plateau");
    assert_eq!(c.playfield_margin_cells, Some(2));
    assert_eq!(c.nuclei.len(), 1);
}

#[test]
fn four_flowers_map_ok() {
    let c = load_validated("four_flowers");
    assert_eq!(c.width_cells, 32);
    assert_eq!(c.height_cells, 32);
    assert!((c.cell_size - 2.0).abs() < f32::EPSILON);
    assert_eq!(c.origin, [-32.0, -32.0]);
    assert_eq!(c.warmup_ticks, Some(24));
    assert_eq!(c.seed, Some(4404));
    assert_eq!(c.nuclei.len(), 4);
    let names: Vec<_> = c.nuclei.iter().map(|n| n.name.as_str()).collect();
    assert!(names.contains(&"flower_nw"));
    assert!(names.contains(&"flower_ne"));
    assert!(names.contains(&"flower_sw"));
    assert!(names.contains(&"flower_se"));
    assert!(c.seasons.is_empty());
    assert!(!c.fog_of_war);
}

#[test]
fn flower_demo_map_ok() {
    let c = load_validated("flower_demo");
    assert_eq!(c.width_cells, 24);
    assert_eq!(c.height_cells, 24);
    assert!((c.cell_size - 2.0).abs() < f32::EPSILON);
    assert_eq!(c.origin, [-24.0, -24.0]);
    assert_eq!(c.warmup_ticks, Some(20));
    assert_eq!(c.seed, Some(4242));
    assert_eq!(c.nuclei.len(), 2);
    assert_eq!(c.nuclei[0].name, "terra_flower_bed");
    assert_eq!(c.nuclei[1].name, "lux_soft");
    assert!(c.seasons.is_empty());
    assert!(!c.fog_of_war);
}

#[test]
fn round_world_rosa_map_ok() {
    let c = load_validated("round_world_rosa");
    assert_eq!(c.width_cells, 20);
    assert_eq!(c.height_cells, 20);
    assert!((c.cell_size - 2.0).abs() < f32::EPSILON);
    assert_eq!(c.origin, [-20.0, -20.0]);
    assert_eq!(c.warmup_ticks, Some(36));
    assert_eq!(c.seed, Some(60062));
    assert_eq!(c.nuclei.len(), 4);
    assert_eq!(c.nuclei[0].name, "terra_pole");
    assert_eq!(c.nuclei[1].name, "lux_sky");
    assert_eq!(c.nuclei[2].name, "aqua_moisture");
    assert_eq!(c.nuclei[3].name, "flora_boost");
    assert!((c.nuclei[1].frequency_hz - 1000.0).abs() < 0.01);
    assert!(c.seasons.is_empty());
    assert!(!c.fog_of_war);
}

#[test]
fn layer_ladder_map_ok() {
    let c = load_validated("layer_ladder");
    assert_eq!(c.width_cells, 24);
    assert_eq!(c.height_cells, 24);
    assert!((c.cell_size - 2.0).abs() < f32::EPSILON);
    assert_eq!(c.origin, [-24.0, -24.0]);
    assert_eq!(c.warmup_ticks, Some(20));
    assert_eq!(c.seed, Some(5150));
    assert_eq!(c.nuclei.len(), 2);
    assert_eq!(c.nuclei[0].name, "terra_ladder_bed");
    assert_eq!(c.nuclei[1].name, "lux_ladder_field");
    assert!((c.nuclei[1].frequency_hz - 1000.0).abs() < 0.01);
    assert!(c.seasons.is_empty());
    assert!(!c.fog_of_war);
}
