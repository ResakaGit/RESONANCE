# Sprint SF-4 — Metrics Export System (CSV/JSON a Disco)

**Modulo:** `src/simulation/observability.rs` (extension), `src/blueprint/constants/simulation_foundations.rs`
**Tipo:** Sistema ECS (FixedUpdate) + file I/O.
**Onda:** A — Requiere SF-1.
**Estado:** ⏳ Pendiente

## Contexto: que ya existe

SF-1 produce `SimulationMetricsSnapshot` cada tick. `export_dashboard_csv_row()` ya genera CSV pero nadie la llama. Bridge metrics se logean via `info!()`. Debug HUD muestra metricas en pantalla.

**Lo que NO existe:**
1. **Sistema de export automatico.** Nadie escribe metricas a disco.
2. **Buffer de batch.** Sin buffer, escribir cada tick seria I/O prohibitivo.
3. **CSV/JSON destination configurada.** No hay path de salida definido.
4. **Header row.** El CSV no tiene encabezados.

## Objetivo

Un sistema que batchea metricas y las flushea a disco cada N ticks. Formato CSV para analisis rapido, JSON opcional para tooling externo.

**Resultado:** `RESONANCE_METRICS=1 cargo run` genera `/tmp/resonance_metrics_{timestamp}.csv` con una fila por tick. Abris en cualquier spreadsheet y ves la evolucion del universo.

## Responsabilidades

### SF-4A: Resource `MetricsExportConfig`

```rust
/// Configuracion de export de metricas. Insertado solo si env RESONANCE_METRICS=1.
#[derive(Resource, Debug, Clone)]
pub struct MetricsExportConfig {
    pub output_path: String,
    pub batch_size: u32,
    pub format: MetricsFormat,
    pub enabled: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MetricsFormat {
    Csv,
    Json,
}
```

- Default: `output_path = "/tmp/resonance_metrics_{timestamp}.csv"`, `batch_size = 60`, `format = Csv`.
- Se inserta condicionalmente: `if std::env::var("RESONANCE_METRICS").is_ok()`.

### SF-4B: `metrics_batch_system` (FixedUpdate)

```rust
/// Batchea snapshots y flushea a disco cuando el buffer llena.
pub fn metrics_batch_system(
    snapshot: Res<SimulationMetricsSnapshot>,
    config: Option<Res<MetricsExportConfig>>,
    mut buffer: Local<Vec<String>>,
    mut header_written: Local<bool>,
) { ... }
```

- **Phase:** `Phase::MetabolicLayer`, `.after(metrics_snapshot_system)`.
- Guard: `if config.is_none() || !config.unwrap().enabled { return; }`.
- Cada tick: formatea snapshot → push a buffer.
- Cada `batch_size` ticks: flush buffer a disco via `std::fs::OpenOptions::append()`.
- Primera escritura: incluir header row.
- **No usa async I/O.** Append sincrono de ~4KB cada 60 ticks es <1ms.

### SF-4C: CSV format

```
tick,total_qe,field_occupancy,solid,liquid,gas,plasma,entities,births,deaths,growth_rate,diversity,conservation_error,drift_rate,saturation
0,1200.5,0.45,0.6,0.2,0.15,0.05,42,0,0,0.0,2.1,0.001,0.0,0.8
1,1198.3,0.45,0.6,0.2,0.15,0.05,42,0,1,-0.024,2.1,0.001,0.02,0.79
```

### SF-4D: JSON format (opcional)

```json
{"tick":0,"total_qe":1200.5,"field_occupancy":0.45,"matter":[0.6,0.2,0.15,0.05],"entities":42,"births":0,"deaths":0,"growth_rate":0.0,"diversity":2.1}
```

NDJSON (newline-delimited) para streaming.

### SF-4E: Constantes

```rust
pub const METRICS_EXPORT_BATCH_SIZE: u32 = 60;
pub const METRICS_EXPORT_DEFAULT_PATH: &str = "/tmp/resonance_metrics";
```

## Tacticas

- **Env var gating.** Sin `RESONANCE_METRICS=1`, el sistema hace early return (zero overhead).
- **Local buffer.** `Local<Vec<String>>` evita Resource overhead. Se limpia cada flush.
- **Append mode.** No reescribe el archivo — agrega al final. Permite multiples sesiones.
- **Sincrono.** 4KB cada 60 ticks = negligible. No justifica async.

## NO hace

- No implementa Prometheus/Grafana endpoint.
- No implementa network streaming.
- No crea visualizaciones.
- No exporta per-entity data — solo aggregados globales.

## Dependencias

- SF-1 (`SimulationMetricsSnapshot`, `metrics_snapshot_system`).
- `std::fs` — file I/O estandar.
- `std::env` — env var check.

## Criterios de aceptacion

### SF-4B (Sistema)
- Test: con config enabled + 60 ticks → archivo existe en output_path con 60 filas + header.
- Test: sin config → no crea archivo (zero overhead).
- Test: config con `batch_size = 1` → flush cada tick (para debugging).
- Test: archivo pre-existente → append, no overwrite.

### SF-4C (CSV format)
- Test: header tiene 15 columnas.
- Test: cada fila tiene 15 valores separados por coma.
- Test: valores float con precision 6 decimales.

### General
- `cargo test --lib` sin regresion.
- `RESONANCE_METRICS=1 cargo run` produce archivo CSV legible.

## Referencias

- `src/simulation/observability.rs:90-98` — `export_dashboard_csv_row()` (se depreca, SF-4 lo reemplaza)
- SF-1 — `SimulationMetricsSnapshot`
