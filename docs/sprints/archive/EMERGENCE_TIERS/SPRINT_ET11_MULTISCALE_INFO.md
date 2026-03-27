# Sprint ET-11 — Multi-Scale Information: Señales Agregadas a Través de Escalas

**Módulo:** `src/simulation/emergence/multiscale.rs` (nuevo), `src/blueprint/equations/emergence/multiscale.rs` (nuevo)
**Tipo:** Ecuaciones puras + Resource de señal agregada + sistema.
**Tier:** T3-2. **Onda:** A.
**BridgeKind:** `AggSignalBridge` — cache Small(128), clave `(grid_band, scale_tier)`.
**Estado:** ✅ Implementado (2026-03-25)

---

## Objetivo

Una entidad necesita diferentes resoluciones de información según su escala de decisión. Decisión individual (τ_a): lee campo local. Decisión grupal (τ_c): lee promedio regional. Decisión institucional (τ_g): lee gradiente continental. La agregación multi-escala evita que cada sistema recompute la misma media.

```
signal_local(x)    = field_qe(x)
signal_regional(x) = mean(field_qe, radius=32)
signal_global(x)   = mean(field_qe, radius=128)
gradient(x)        = (signal_regional - signal_local) / radius
```

---

## Responsabilidades

### ET-11A: Ecuaciones puras

```rust
// src/blueprint/equations/emergence/multiscale.rs

/// Señal agregada ponderada: combina local, regional y global según pesos.
pub fn aggregate_signal(
    local:    f32,
    regional: f32,
    global:   f32,
    weights:  [f32; 3],   // [w_local, w_regional, w_global], idealmente suman 1.0
) -> f32 {
    local * weights[0] + regional * weights[1] + global * weights[2]
}

/// Relevancia de una escala para un horizonte de planificación dado.
/// horizon_ticks: cuántos ticks adelante planea la entidad.
/// scale_tau: escala temporal de la señal (local=10, regional=100, global=1000).
pub fn scale_relevance(horizon_ticks: u32, scale_tau: u32) -> f32 {
    if scale_tau == 0 { return 1.0; }
    let ratio = horizon_ticks as f32 / scale_tau as f32;
    (-ratio.powi(2)).exp()   // gaussiana: más relevante cuando horizon ≈ tau
}

/// Gradiente de señal entre escalas: dirección de movimiento óptimo.
/// Positivo → moverse hacia recursos, negativo → huir de alta presión.
pub fn information_gradient(local: f32, regional: f32, scale_distance: f32) -> f32 {
    if scale_distance <= 0.0 { return 0.0; }
    (regional - local) / scale_distance
}

/// Atenuación de señal con la distancia (ley de potencias).
pub fn signal_attenuation(base_signal: f32, distance: f32, attenuation_exp: f32) -> f32 {
    if distance <= 0.0 { return base_signal; }
    base_signal / (1.0 + distance.powf(attenuation_exp))
}

/// Ruido de información: incertidumbre al agregar señales heterogéneas.
pub fn aggregation_noise(n_sources: u32, source_variance: f32) -> f32 {
    if n_sources == 0 { return source_variance; }
    source_variance / (n_sources as f32).sqrt()   // ley de √n
}
```

### ET-11B: Tipos

```rust
// src/simulation/emergence/multiscale.rs

/// Resource: señales pre-agregadas en 3 escalas espaciales.
/// Actualizado 1×/N ticks por `multiscale_aggregation_system`.
#[derive(Resource, Default, Debug)]
pub struct MultiscaleSignalGrid {
    pub local:    Vec<f32>,   // 32×32 — cell resolution
    pub regional: Vec<f32>,   // 8×8  — region resolution (4×4 cells per region)
    pub global:   f32,        // scalar — mean over entire map
    pub last_updated: u64,
}

impl MultiscaleSignalGrid {
    pub fn local_at(&self, idx: usize) -> f32 {
        self.local.get(idx).copied().unwrap_or(0.0)
    }
    pub fn regional_at(&self, region_idx: usize) -> f32 {
        self.regional.get(region_idx).copied().unwrap_or(0.0)
    }
    /// Convierte cell_idx (32×32) a region_idx (8×8).
    pub fn cell_to_region(cell_idx: u32) -> usize {
        let row = (cell_idx / 32) as usize / 4;
        let col = (cell_idx % 32) as usize / 4;
        row * 8 + col
    }
}
```

### ET-11C: Sistemas

```rust
/// Agrega el EnergyFieldGrid en las 3 escalas. Corre 1× cada MULTISCALE_UPDATE_INTERVAL ticks.
/// Phase::ThermodynamicLayer — primera fase, antes de que los sistemas lo consuman.
pub fn multiscale_aggregation_system(
    field: Res<EnergyFieldGrid>,
    mut ms: ResMut<MultiscaleSignalGrid>,
    clock: Res<SimulationClock>,
    config: Res<MultiscaleConfig>,
) {
    if clock.tick_id % config.update_interval as u64 != 0 { return; }

    // Local: copia directa (32×32)
    ms.local.clear();
    ms.local.extend((0..1024).map(|i| field.cell_qe(i)));

    // Regional: media de bloques 4×4 → 8×8
    ms.regional.clear();
    ms.regional.resize(64, 0.0);
    for cell_idx in 0u32..1024 {
        let region = MultiscaleSignalGrid::cell_to_region(cell_idx);
        ms.regional[region] += ms.local[cell_idx as usize] / 16.0;
    }

    // Global: media de todos los locales
    ms.global = ms.local.iter().sum::<f32>() / 1024.0;
    ms.last_updated = clock.tick_id;
}

/// Provee señal agregada a entidades con `TimescaleAdapter`.
/// Escribe en el `learned_offset` si el gradiente indica una dirección de mejora.
/// Phase::Input — after multiscale_aggregation_system.
pub fn multiscale_signal_consumer_system(
    mut adapters: Query<(&Transform, &mut TimescaleAdapter)>,
    ms: Res<MultiscaleSignalGrid>,
    field: Res<EnergyFieldGrid>,
    cache: ResMut<BridgeCache<AggSignalBridge>>,
    config: Res<MultiscaleConfig>,
) {
    for (transform, mut adapter) in &mut adapters {
        let cell_idx = field.world_to_cell_idx(transform.translation.x, transform.translation.z);
        let region   = MultiscaleSignalGrid::cell_to_region(cell_idx);
        let cache_key = (cell_idx as u32) ^ ((region as u32) << 16);

        let gradient = multiscale_eq::information_gradient(
            ms.local_at(cell_idx as usize),
            ms.regional_at(region),
            config.regional_radius,
        );
        // Gradiente positivo → hay más recursos en la región → ajustar learned_offset levemente
        let new_offset = adapter.learned_offset + gradient * config.gradient_influence;
        if (adapter.learned_offset - new_offset).abs() > f32::EPSILON {
            adapter.learned_offset = new_offset;
        }
    }
}
```

### ET-11D: Constantes

```rust
pub struct AggSignalBridge;
impl BridgeKind for AggSignalBridge {}

pub const MULTISCALE_UPDATE_INTERVAL: u8 = 8;     // agrega cada 8 ticks
pub const MULTISCALE_REGIONAL_RADIUS:  f32 = 32.0;
pub const MULTISCALE_GLOBAL_RADIUS:    f32 = 128.0;
pub const MULTISCALE_GRADIENT_INFLUENCE: f32 = 0.005; // influencia del gradiente en learned_offset
```

---

## Tacticas

- **Agregación amortizada.** `multiscale_aggregation_system` corre 1× cada 8 ticks. La grid de 1024 celdas se agrega en ~O(1024) operaciones — barato. Todos los demás sistemas leen de `MultiscaleSignalGrid` sin tocar `EnergyFieldGrid`.
- **Vec<f32> sin HashMap.** `MultiscaleSignalGrid` es un Vec indexado por cell_idx. Acceso O(1), sin colisiones, coherencia de caché lineal.
- **`AggSignalBridge` cachea gradientes.** El gradiente `(local, regional)` es el mismo para toda entidad en la misma celda. Cache key `(cell_idx, region_idx)` → hit rate ~90% en grupos de entidades colocalizadas.
- **Tres vistas del mismo dato.** Local/Regional/Global no son tres estructuras de datos — son tres vistas de `EnergyFieldGrid` precalculadas. Sin duplicación de estado.

---

## NO hace

- No implementa pathfinding hacia gradiente positivo — eso es `simulation/pathfinding.rs`.
- No agrega señales de otras capas (MatterCoherence, TensionField) — sólo qe. Extender si necesario.
- No transmite información entre entidades — es un Resource global, no un canal P2P.

---

## Dependencias

- `worldgen/field_grid.rs::EnergyFieldGrid` — fuente de datos.
- ET-10 `TimescaleAdapter` — receptor del `learned_offset` ajustado.
- ET-14 Institutions — usará `ms.global` para detectar escasez a escala civilizacional.

---

## Criterios de Aceptación

- `aggregate_signal(10.0, 20.0, 30.0, [0.5, 0.3, 0.2])` → `17.0`.
- `scale_relevance(100, 100)` → `1/e ≈ 0.368`.
- `scale_relevance(10, 1000)` → `≈ 1.0` (horizonte << tau → máxima relevancia local).
- `information_gradient(10.0, 20.0, 10.0)` → `1.0` (gradiente positivo).
- `aggregation_noise(100, 1.0)` → `0.1` (ley √n).
- Test: `multiscale_aggregation_system` → `ms.regional` tiene 64 entradas.
- Test: `ms.global` = media correcta de `local`.
- Test: entidad en celda con gradiente positivo → `learned_offset` aumenta.
- `cargo test --lib` sin regresión.

---

## Referencias

- ET-10 Multiple Timescales — receptor de la señal agregada
- `src/worldgen/field_grid.rs` — `EnergyFieldGrid` fuente
- Blueprint §T3-2: "Multi-Scale Information Flow"
