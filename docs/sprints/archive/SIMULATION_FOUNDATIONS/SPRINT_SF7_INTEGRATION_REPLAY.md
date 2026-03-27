# Sprint SF-7 — Integracion: Replay Determinista + Verificacion End-to-End

**Modulo:** `tests/sf_integration.rs`, `assets/maps/signal_latency_demo.ron`, `src/world/demos/signal_demo.rs`
**Tipo:** Tests de integracion + demo map.
**Onda:** B — Requiere SF-1 a SF-6.
**Estado:** ⏳ Pendiente

## Contexto: que ya existe despues de SF-1→SF-6

| Pieza | Sprint | Estado post-SF |
|-------|--------|---------------|
| Metricas expandidas | SF-1 | `SimulationMetricsSnapshot` con 13 campos por tick |
| CSV export | SF-4 | `RESONANCE_METRICS=1` genera CSV a disco |
| Serde en layers | SF-2 | 14 layers + grid serializables |
| Checkpoint save/load | SF-5 | `RESONANCE_CHECKPOINT_SAVE=N` guarda, `RESONANCE_CHECKPOINT_LOAD=path` carga |
| Propagacion multi-tick | SF-6 | `PropagationMode::WaveFront` con frente de onda |
| Ecuaciones de latencia | SF-3 | `propagation_front_radius`, `diffusion_delta`, etc. |

**Lo que falta:** Verificar que todo funciona junto. Que un checkpoint cargado produce el mismo estado. Que la propagacion multi-tick es causal. Que las metricas reflejan la realidad.

## Objetivo

Tests de integracion que verifican los 3 pilares en conjunto, mas un demo map que muestra propagacion con latencia visible.

**Resultado:** `cargo test --test sf_integration` pasa. `RESONANCE_MAP=signal_demo cargo run` muestra un frente de onda expandiendose visualmente.

## Responsabilidades

### SF-7A: Test de replay determinista

```rust
#[test]
fn checkpoint_roundtrip_deterministic() {
    // 1. Run simulation 100 ticks → save checkpoint at tick 100.
    // 2. Run same simulation 200 ticks → save state at tick 200 as "reference".
    // 3. Load checkpoint from tick 100 → run 100 ticks → save state at tick 200 as "replay".
    // 4. Assert: reference == replay (bit-for-bit on EnergyFieldGrid + entity qe values).
}
```

- Usa `MinimalPlugins` + `SimulationPlugin` sin rendering.
- Mapa: `default.ron` o mapa test dedicado.
- Comparacion: `EnergyFieldGrid.cells` iguales, `BaseEnergy.qe` por entidad iguales.
- **Este test es la prueba de fuego del determinismo.**

### SF-7B: Test de causalidad de propagacion

```rust
#[test]
fn propagation_wave_front_is_causal() {
    // 1. Grid 32x32, single nucleus at center, mode=WaveFront, speed=4.
    // 2. Run 1 tick → assert: only cells within radius 4 have qe > 0.
    // 3. Run 4 more ticks (total 5) → assert: cells within radius 20 have qe > 0.
    // 4. Assert: cells at radius 25 still have qe == 0 (frente no llego).
    // 5. Run 2 more ticks → assert: radius 25 cells now have qe > 0.
}
```

- Verifica que la informacion no viaja mas rapido que `PROPAGATION_SPEED_CELLS_PER_TICK`.
- Verifica que eventualmente llega (no se pierde).

### SF-7C: Test de metricas vs realidad

```rust
#[test]
fn metrics_snapshot_matches_world_state() {
    // 1. Run 50 ticks.
    // 2. Read SimulationMetricsSnapshot.
    // 3. Manually count entities, sum qe, check matter distribution.
    // 4. Assert: snapshot values match manual count within epsilon.
}
```

- Verifica que `SimulationMetricsSnapshot` no miente.
- Precision: `|snapshot.total_qe - manual_sum| < 0.01`.

### SF-7D: Test de CSV export

```rust
#[test]
fn csv_export_produces_valid_file() {
    // 1. Set RESONANCE_METRICS=1 (or insert MetricsExportConfig directly).
    // 2. Run 120 ticks (2 batches of 60).
    // 3. Read CSV file.
    // 4. Assert: 120 data rows + 1 header.
    // 5. Assert: each row has 15 comma-separated values.
    // 6. Assert: tick column is monotonically increasing.
}
```

### SF-7E: Demo map `signal_latency_demo.ron`

```ron
(
    name: "signal_demo",
    grid_size: (32, 32),
    cell_size: 2.0,
    nuclei: [
        (position: (16, 16), element: "terra", qe: 1200.0, radius: 14),
    ],
    terrain: (enabled: true, seed: 99),
    propagation_mode: "wave_front",
    profile: "full3d",
)
```

- Un solo nucleo en el centro. Propagacion WaveFront.
- El frente de onda es visible como expansion gradual del color del terreno.
- Terrain mesh (IWG-4) refleja los colores inferidos del campo → el frente se ve como una onda de color.

### SF-7F: Demo startup `spawn_signal_demo_startup_system`

```rust
pub fn spawn_signal_demo_startup_system(mut commands: Commands) {
    info!("SF-7 signal demo: single nucleus, wave front propagation");
    // No fauna — pure wave observation.
}
```

- Registrar en `DebugPlugin` con slug `"signal_demo"`.
- Sin fauna — el demo es puramente observar la onda.

## Tacticas

- **Tests como prueba de contrato.** Cada test verifica un invariante del track (determinismo, causalidad, precision, formato).
- **MinimalPlugins para tests.** Sin rendering, sin input, sin camera. Solo simulacion pura.
- **El demo es visual pero el test es numerico.** El demo muestra, el test prueba.

## NO hace

- No implementa herramientas de analisis (graphs, plots).
- No implementa diff entre checkpoints.
- No implementa UI de replay (play/pause/rewind).

## Dependencias

- SF-1 (`SimulationMetricsSnapshot`, `metrics_snapshot_system`).
- SF-2 (`WorldCheckpoint`, serde derives).
- SF-3 (ecuaciones de propagacion).
- SF-4 (`MetricsExportConfig`, `metrics_batch_system`).
- SF-5 (`CheckpointConfig`, save/load systems).
- SF-6 (`PropagationMode::WaveFront`, `diffuse_propagation_system`).

## Criterios de aceptacion

### SF-7A (Determinismo)
- `checkpoint_roundtrip_deterministic` pasa.
- 100 ticks de replay produce estado identico al original.

### SF-7B (Causalidad)
- `propagation_wave_front_is_causal` pasa.
- Celdas fuera del frente tienen qe=0 en ticks tempranos.
- Celdas dentro del frente tienen qe>0.

### SF-7C (Metricas)
- `metrics_snapshot_matches_world_state` pasa.
- total_qe error < 0.01.
- entity_count exacto.

### SF-7D (CSV)
- `csv_export_produces_valid_file` pasa.
- 120 filas + header, 15 columnas, tick monotono.

### SF-7E/F (Demo)
- `RESONANCE_MAP=signal_demo cargo run` → mundo con onda visible expandiendose.
- Frente de onda tarda ~8 segundos en llenar el grid (visual).

### General
- `cargo test --test sf_integration` sin fallos.
- `cargo test --lib` sin regresion.

## Referencias

- Todos los sprints SF-1 a SF-6
- `tests/` — directorio de tests de integracion existentes
- `assets/maps/*.ron` — formato de mapa
- `src/plugins/debug_plugin.rs` — registro de demos
