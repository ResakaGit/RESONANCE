//! SF-7 — Integration: Replay determinista + verificación end-to-end.
//!
//! Cuatro contratos que verifican la composición de SF-1..SF-6:
//!
//! - **SF-7A** checkpoint_roundtrip_deterministic      — RON ↔ JSON cross-format bit-idéntico
//! - **SF-7B** propagation_wave_front_is_causal        — velocidad finita se mantiene en N ticks
//! - **SF-7C** health_system_converges_on_stable_energy — sistema Bevy refleja estado real
//! - **SF-7D** csv_tick_sequence_is_monotonic          — export ticksmonotónicos, columnas correctas

use bevy::prelude::*;

use resonance::blueprint::checkpoint::{
    EntitySnapshot, build_checkpoint, checkpoint_from_json, checkpoint_from_ron,
    checkpoint_to_json, checkpoint_to_ron,
};
use resonance::blueprint::equations::{
    PROPAGATION_SPEED_CELLS_PER_TICK, propagation_front_radius, propagation_intensity_at_tick,
};
use resonance::layers::BaseEnergy;
use resonance::simulation::observability::{
    SimulationAlerts, SimulationHealthDashboard, export_dashboard_csv_row, simulation_health_system,
};

// ─── SF-7A: Checkpoint roundtrip determinista ─────────────────────────────────

/// Builds a repeatable set of 4 entity snapshots sorted by id.
fn canonical_snapshots() -> Vec<EntitySnapshot> {
    vec![
        EntitySnapshot { id: 1,  position: [-8.0, 0.0,  4.0], energy: 150.0, radius: 1.2, frequency: 75.0,  phase: 0.0,  matter_state: 0, bond_energy: 500.0 },
        EntitySnapshot { id: 7,  position: [ 0.0, 0.0,  0.0], energy:  80.5, radius: 0.8, frequency: 440.0, phase: 1.57, matter_state: 1, bond_energy: 200.0 },
        EntitySnapshot { id: 13, position: [ 5.0, 0.0, -3.0], energy: 320.0, radius: 2.0, frequency: 880.0, phase: 3.14, matter_state: 2, bond_energy:  75.0 },
        EntitySnapshot { id: 42, position: [12.0, 0.0,  8.0], energy:   0.5, radius: 0.3, frequency: 200.0, phase: 0.78, matter_state: 3, bond_energy:   1.0 },
    ]
}

/// RON and JSON roundtrips produce byte-identical f32 values for all entity fields.
/// Cross-format consistency: same data regardless of serialization format.
#[test]
fn checkpoint_roundtrip_deterministic() {
    let snaps = canonical_snapshots();
    let cp = build_checkpoint(100, "signal_demo", &snaps);

    let ron_str = checkpoint_to_ron(&cp).expect("checkpoint_to_ron");
    let json_str = checkpoint_to_json(&cp).expect("checkpoint_to_json");

    let from_ron  = checkpoint_from_ron(&ron_str).expect("checkpoint_from_ron");
    let from_json = checkpoint_from_json(&json_str).expect("checkpoint_from_json");

    assert_eq!(from_ron.tick, 100);
    assert_eq!(from_ron.map_name, "signal_demo");
    assert_eq!(from_ron.entities.len(), 4);
    assert_eq!(from_json.entities.len(), from_ron.entities.len());

    for (ron_e, json_e) in from_ron.entities.iter().zip(from_json.entities.iter()) {
        assert_eq!(ron_e.id, json_e.id, "id mismatch");
        assert_eq!(
            ron_e.energy.to_bits(), json_e.energy.to_bits(),
            "energy not bit-identical across formats for id={}", ron_e.id,
        );
        assert_eq!(
            ron_e.frequency.to_bits(), json_e.frequency.to_bits(),
            "frequency not bit-identical across formats for id={}", ron_e.id,
        );
        assert_eq!(ron_e.matter_state, json_e.matter_state, "matter_state mismatch for id={}", ron_e.id);
    }
}

/// Two checkpoints built from the same snapshots are PartialEq-identical.
/// Verifies build_checkpoint is pure (no hidden state).
#[test]
fn checkpoint_build_is_pure() {
    let snaps = canonical_snapshots();
    let cp_a = build_checkpoint(50, "map_a", &snaps);
    let cp_b = build_checkpoint(50, "map_a", &snaps);
    assert_eq!(cp_a, cp_b, "build_checkpoint must be pure — same inputs → same output");
}

/// Checkpoint entity order is preserved after RON roundtrip.
/// The save system sorts by id; this verifies deserialization preserves that order.
#[test]
fn checkpoint_entity_order_preserved_after_ron_roundtrip() {
    let mut snaps = canonical_snapshots();
    snaps.sort_unstable_by_key(|e| e.id);
    let cp = build_checkpoint(1, "order_test", &snaps);
    let ron_str = checkpoint_to_ron(&cp).expect("ron");
    let restored = checkpoint_from_ron(&ron_str).expect("from_ron");
    let ids: Vec<u32> = restored.entities.iter().map(|e| e.id).collect();
    let mut sorted = ids.clone();
    sorted.sort_unstable();
    assert_eq!(ids, sorted, "entity order must be preserved after roundtrip (sorted by id)");
}

// ─── SF-7B: Causalidad de propagación wavefront ───────────────────────────────

/// Verifica que la propiedad de causalidad se mantiene en toda una secuencia de N ticks:
/// - Celdas a distancia d > front_radius tienen intensidad cero.
/// - Celdas a distancia d ≤ front_radius tienen intensidad > 0 (source > 0).
#[test]
fn propagation_wave_front_is_causal() {
    let speed    = PROPAGATION_SPEED_CELLS_PER_TICK;
    let source   = 500.0_f32;
    let decay    = 0.05_f32;
    let damping  = 0.98_f32;
    // Test distances: from very close to well beyond max front at tick 15.
    let distances: [f32; 6] = [1.0, 4.0, 8.0, 16.0, 22.0, 30.0];

    for tick in 1u32..=15 {
        let front_radius = propagation_front_radius(speed, tick);

        for &d in &distances {
            let intensity = propagation_intensity_at_tick(source, d, decay, front_radius, damping, tick);

            if d > front_radius {
                assert_eq!(
                    intensity, 0.0,
                    "tick={tick} dist={d:.1} front={front_radius:.1}: causality violated — signal arrived early",
                );
            } else {
                assert!(
                    intensity > 0.0,
                    "tick={tick} dist={d:.1} front={front_radius:.1}: signal must be non-zero inside front",
                );
            }
        }
    }
}

/// Frente de onda crece linealmente — cada tick avanza exactamente SPEED celdas.
#[test]
fn propagation_front_grows_by_speed_each_tick() {
    let speed = PROPAGATION_SPEED_CELLS_PER_TICK;
    for tick in 1u32..=20 {
        let radius = propagation_front_radius(speed, tick);
        let expected = speed * tick as f32;
        assert!(
            (radius - expected).abs() < 1e-4,
            "tick={tick}: expected front={expected}, got={radius}",
        );
    }
}

/// La señal fuera del frente permanece en cero aunque el source sea máximo.
#[test]
fn propagation_no_signal_beyond_front_at_any_tick() {
    let speed = PROPAGATION_SPEED_CELLS_PER_TICK;
    for tick in 0u32..=30 {
        let front = propagation_front_radius(speed, tick);
        let beyond = front + 0.01;
        let intensity = propagation_intensity_at_tick(1000.0, beyond, 0.01, front, 0.99, tick);
        assert_eq!(
            intensity, 0.0,
            "tick={tick} front={front:.2}: signal must be zero beyond front (dist={beyond:.2})",
        );
    }
}

// ─── SF-7C: Health system refleja estado de entidades ────────────────────────

fn make_health_app() -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.init_resource::<SimulationHealthDashboard>();
    app.init_resource::<SimulationAlerts>();
    app.add_systems(Update, simulation_health_system);
    app
}

/// Sistema sin entidades: drift = 0, saturation = 0, tick_count avanza.
#[test]
fn health_system_empty_world_stable() {
    let mut app = make_health_app();
    app.update();
    let dash = app.world().resource::<SimulationHealthDashboard>();
    assert_eq!(dash.tick_count, 1, "tick_count must advance each run");
    assert_eq!(dash.drift_rate, 0.0, "empty world has zero drift");
    assert_eq!(dash.saturation_index, 0.0, "empty world has zero saturation");
}

/// Con N entidades estables: segundo tick tiene drift cero (energía no cambió).
#[test]
fn health_system_converges_on_stable_energy() {
    let mut app = make_health_app();

    for _ in 0..5 {
        app.world_mut().spawn(BaseEnergy::new(100.0));
    }

    app.update(); // tick 1: prev=0 → curr=500, drift guarded by zero-prev
    app.update(); // tick 2: prev=500 → curr=500, drift = 0
    app.update(); // tick 3: stable

    let dash = app.world().resource::<SimulationHealthDashboard>();
    assert_eq!(dash.tick_count, 3);
    assert_eq!(
        dash.drift_rate, 0.0,
        "stable energy must produce zero drift — got {}",
        dash.drift_rate,
    );
    let alerts = app.world().resource::<SimulationAlerts>();
    assert!(!alerts.conservation_violated, "no conservation violation in stable world");
    assert!(!alerts.critical_drift, "no critical drift in stable world");
}

/// tick_count es monotónico a través de múltiples actualizaciones.
#[test]
fn health_system_tick_count_is_monotonic() {
    let mut app = make_health_app();
    let n = 10u64;
    for _ in 0..n { app.update(); }
    let dash = app.world().resource::<SimulationHealthDashboard>();
    assert_eq!(dash.tick_count, n, "tick_count must equal number of system runs");
}

// ─── SF-7D: CSV export — secuencia monotónica, columnas correctas ─────────────

/// Genera filas CSV para ticks [1, 50, 100] y verifica:
/// - Primer campo (tick) es monotónicamente creciente.
/// - Cada fila tiene exactamente 4 campos separados por comas.
/// - Los valores son parseable como f64/u64 (no corruptos).
#[test]
fn csv_tick_sequence_is_monotonic() {
    let ticks = [1u64, 50, 100];
    let mut rows: Vec<String> = Vec::with_capacity(ticks.len());

    for &t in &ticks {
        let dash = SimulationHealthDashboard {
            tick_count:        t,
            conservation_error: 0.0,
            drift_rate:         0.001 * t as f32,
            saturation_index:   0.5,
        };
        rows.push(export_dashboard_csv_row(&dash));
    }

    let mut prev_tick: i64 = -1;
    for row in &rows {
        let fields: Vec<&str> = row.split(',').collect();
        assert_eq!(
            fields.len(), 4,
            "each CSV row must have 4 fields, got {}: '{row}'",
            fields.len(),
        );
        let tick: i64 = fields[0].trim().parse().expect("tick field must be parseable as integer");
        assert!(
            tick > prev_tick,
            "CSV ticks must be monotonically increasing: prev={prev_tick} current={tick}",
        );
        // Validate remaining fields are parseable floats.
        for (i, f) in fields[1..].iter().enumerate() {
            f.trim().parse::<f64>().unwrap_or_else(|_| {
                panic!("CSV field {} is not a valid float: '{f}' in row '{row}'", i + 1)
            });
        }
        prev_tick = tick;
    }
}

/// Una fila CSV para tick=0 (estado inicial) comienza con "0,".
#[test]
fn csv_default_dashboard_row_starts_with_zero() {
    let dash = SimulationHealthDashboard::default();
    let row = export_dashboard_csv_row(&dash);
    assert!(row.starts_with("0,"), "default dashboard tick must be 0, got: '{row}'");
}
