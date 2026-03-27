# Sprint ET-12 — Continental Drift: Modificación Tectónica del Terreno

**Módulo:** `src/simulation/emergence/tectonics.rs` (nuevo), `src/blueprint/equations/emergence/tectonics.rs` (nuevo)
**Tipo:** Ecuaciones puras + sistema de modificación lenta del EnergyFieldGrid.
**Tier:** T3-3. **Onda:** B.
**BridgeKind:** `TectonicBridge` — cache Small(32), clave `(plate_id, stress_band)`.
**Estado:** ✅ Implementado (2026-03-25)

---

## Contexto: qué ya existe

- `topology/` — generación de terreno: noise, slope, drainage, hydraulics, classifier. Lee `EnergyFieldGrid` para clasificar celdas.
- `worldgen/field_grid.rs::EnergyFieldGrid` — grid 32×32 mutable. `drain_cell` / `cell_qe`. Ya usada por ET-4 Infrastructure.
- `TerrainMutationEvent` — registrado en `simulation/bootstrap.rs`. Señal de que el terreno cambió.
- ET-4 `InfrastructureGrid` — precedente de Resource paralelo a EnergyFieldGrid.

**Lo que NO existe:**
1. Placas tectónicas como entidades de Resource con velocidad de deriva.
2. Acumulación de estrés entre placas y eventos de liberación (volcanes/seísmos).
3. Modificación de `EnergyFieldGrid` por actividad tectónica (no sólo metabólica).
4. `TectonicEvent` — señal de modificación geológica.

---

## Objetivo

Las placas tectónicas derivan lentamente modificando el mapa de energía. El estrés acumulado en bordes de placa se libera en eventos discretos (seísmos, erupciones) que modifican `EnergyFieldGrid` localmente. Los organismos que viven en zonas activas enfrentan un entorno dinámico impredecible.

```
drift(plate) = drift_velocity × Δt
stress(boundary) += contact_force × Δt
if stress > threshold: earthquake(boundary) → field_qe_delta(±amplitude)
```

---

## Responsabilidades

### ET-12A: Ecuaciones puras

```rust
// src/blueprint/equations/emergence/tectonics.rs

/// Estrés acumulado en el borde entre dos placas.
pub fn boundary_stress(
    relative_velocity: f32,  // velocidad relativa de las placas
    contact_length: f32,     // longitud del borde en contacto (celdas)
    friction_coeff: f32,
) -> f32 {
    relative_velocity * contact_length * friction_coeff
}

/// Amplitud del evento sísmico al liberar estrés acumulado.
pub fn seismic_amplitude(stress_released: f32, depth_factor: f32) -> f32 {
    (stress_released * depth_factor).sqrt()
}

/// Delta de qe en una celda tras un evento sísmico.
/// distance: distancia en celdas al epicentro.
/// Ley de potencias: atenúa con la distancia al cuadrado.
pub fn seismic_qe_delta(amplitude: f32, distance: f32, is_constructive: bool) -> f32 {
    let base = amplitude / (1.0 + distance.powi(2));
    if is_constructive { base } else { -base }
}

/// Uplift geológico: incremento de qe base de una celda por actividad volcánica.
pub fn volcanic_qe_uplift(magma_flux: f32, eruption_efficiency: f32) -> f32 {
    magma_flux * eruption_efficiency
}

/// Erosión: reducción de qe base de una celda expuesta a estrés tectónico.
pub fn tectonic_erosion(cell_qe: f32, erosion_rate: f32) -> f32 {
    cell_qe * erosion_rate
}
```

### ET-12B: Tipos

```rust
// src/simulation/emergence/tectonics.rs

/// Resource: estado de las placas tectónicas.
#[derive(Resource, Debug)]
pub struct TectonicState {
    pub plates:       [TectonicPlate; MAX_PLATES],
    pub plate_count:  u8,
    pub global_tick:  u64,   // ticks de tiempo geológico transcurridos
}

#[derive(Debug, Clone, Copy, Default)]
pub struct TectonicPlate {
    pub plate_id:         u8,
    pub drift_velocity_x: f32,  // celdas/tick (muy pequeño — 0.00001)
    pub drift_velocity_z: f32,
    pub stress_accum:     f32,  // estrés acumulado sin liberar
}

pub const MAX_PLATES: usize = 4;

/// Evento: liberación tectónica que modifica el campo.
#[derive(Event, Debug, Clone)]
pub struct TectonicEvent {
    pub epicenter_cell: u32,
    pub amplitude:      f32,
    pub is_volcanic:    bool,   // true=uplift, false=destructivo
    pub tick_id:        u64,
}
```

### ET-12C: Sistemas

```rust
/// Acumula estrés tectónico y dispara eventos al superar umbral.
/// Phase::MorphologicalLayer — last, operación lenta.
pub fn tectonic_stress_system(
    mut state: ResMut<TectonicState>,
    mut tectonic_events: EventWriter<TectonicEvent>,
    mut terrain_events:  EventWriter<TerrainMutationEvent>,
    clock: Res<SimulationClock>,
    config: Res<TectonicConfig>,
) {
    if clock.tick_id % config.tectonic_eval_interval as u64 != 0 { return; }
    state.global_tick += 1;

    for plate in state.plates[..state.plate_count as usize].iter_mut() {
        // Acumular estrés por deriva
        let speed = (plate.drift_velocity_x.powi(2) + plate.drift_velocity_z.powi(2)).sqrt();
        let stress = tectonic_eq::boundary_stress(speed, config.mean_contact_length, config.friction);
        plate.stress_accum += stress;

        if plate.stress_accum > config.stress_threshold {
            // Liberar: determinar epicentro aleatorio (deterministico con tick_id)
            let seed = (clock.tick_id ^ plate.plate_id as u64).wrapping_mul(6364136223846793005);
            let epicenter = (seed % 1024) as u32;
            let amplitude = tectonic_eq::seismic_amplitude(plate.stress_accum, config.depth_factor);

            tectonic_events.send(TectonicEvent {
                epicenter_cell: epicenter,
                amplitude,
                is_volcanic: (seed >> 32) % 5 == 0,  // 20% son volcánicos
                tick_id: clock.tick_id,
            });
            terrain_events.send(TerrainMutationEvent { cell: epicenter, tick: clock.tick_id });
            plate.stress_accum = 0.0;
        }
    }
}

/// Aplica el delta de qe en celdas afectadas por eventos tectónicos.
/// Phase::MorphologicalLayer — after tectonic_stress_system.
pub fn tectonic_field_mutation_system(
    mut field: ResMut<EnergyFieldGrid>,
    mut events: EventReader<TectonicEvent>,
    config: Res<TectonicConfig>,
) {
    for ev in events.read() {
        for cell_idx in 0u32..1024 {
            let row_e = (ev.epicenter_cell / 32) as f32;
            let col_e = (ev.epicenter_cell % 32) as f32;
            let row_c = (cell_idx / 32) as f32;
            let col_c = (cell_idx % 32) as f32;
            let dist  = ((row_c - row_e).powi(2) + (col_c - col_e).powi(2)).sqrt();

            if dist > config.seismic_radius { continue; }

            let delta = if ev.is_volcanic {
                tectonic_eq::volcanic_qe_uplift(ev.amplitude, config.eruption_efficiency)
                    / (1.0 + dist.powi(2))
            } else {
                tectonic_eq::seismic_qe_delta(ev.amplitude, dist, false)
            };

            field.drain_cell(cell_idx, -delta);  // drain negativo = add
        }
    }
}
```

### ET-12D: Constantes

```rust
pub struct TectonicBridge;
impl BridgeKind for TectonicBridge {}

pub const TECTONIC_EVAL_INTERVAL:    u64 = 500;    // cada 500 ticks
pub const TECTONIC_STRESS_THRESHOLD: f32 = 100.0;
pub const TECTONIC_FRICTION:         f32 = 0.5;
pub const TECTONIC_DEPTH_FACTOR:     f32 = 10.0;
pub const TECTONIC_ERUPTION_EFF:     f32 = 0.3;
pub const TECTONIC_SEISMIC_RADIUS:   f32 = 8.0;    // celdas afectadas
pub const MAX_PLATES:                usize = 4;
```

---

## Tacticas

- **`TerrainMutationEvent` como puente tectónica↔topología.** El evento ya existe en `simulation/bootstrap.rs`. ET-12 lo emite; `topology/` puede escucharlo para reclasificar celdas. Desacoplamiento limpio.
- **Deriving epicentro deterministamente.** `seed = tick_id ^ plate_id × prime` da pseudo-aleatoriedad reproducible — invariant INV-4 (determinismo) preservado.
- **Evaluación cada 500 ticks.** La deriva tectónica es geológicamente lenta. 500 ticks por evaluación = CPU cost negligible.
- **MAX_PLATES = 4 array fijo.** No Vec. Sin heap en Resource de estado tectónico.

---

## NO hace

- No genera visuales de terreno — eso es GS-7 VisualContract + topología rendering.
- No modela subducción o colisión de placas — `drift_velocity` es suficiente para el gameplay.
- No implementa clima basado en placas — ET-11 MultiscaleSignal cubre el gradiente climático.

---

## Dependencias

- `worldgen/field_grid.rs::EnergyFieldGrid` — target de modificación.
- `simulation/bootstrap.rs::TerrainMutationEvent` — evento ya registrado, ET-12 lo emite.
- ET-4 `InfrastructureGrid` — infraestructura construida puede destruirse por seísmos (futuro).
- ET-13 `GeologicalLOD` — comprime múltiples tectonic_ticks cuando LOD es bajo.

---

## Criterios de Aceptación

- `boundary_stress(0.001, 10.0, 0.5)` → `0.005`.
- `seismic_amplitude(100.0, 10.0)` → `≈ 31.6`.
- `seismic_qe_delta(30.0, 0.0, false)` → `-30.0` (epicentro destructivo).
- `seismic_qe_delta(30.0, 5.0, true)` → `≈ 1.15` (atenuado por distancia, constructivo).
- Test: estrés > threshold → `TectonicEvent` emitido.
- Test: evento volcánico → celdas cercanas aumentan qe.
- Test: evento sísmico → celdas cercanas reducen qe.
- Test: determinismo — misma seed produce mismo epicentro.
- `cargo test --lib` sin regresión.

---

## Referencias

- `src/topology/` — clasificador de terreno (escucha TerrainMutationEvent)
- `src/worldgen/field_grid.rs` — EnergyFieldGrid
- ET-4 Infrastructure — precedente de Resource paralelo
- Blueprint §T3-3: "Continental Drift", tectonic stress equations
