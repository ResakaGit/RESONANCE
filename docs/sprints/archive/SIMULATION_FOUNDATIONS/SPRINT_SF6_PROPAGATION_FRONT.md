# Sprint SF-6 — Propagacion Multi-Tick (Frente de Onda)

**Modulo:** `src/worldgen/systems/propagation.rs` (modificacion), `src/worldgen/propagation.rs` (extension), `src/worldgen/field_grid.rs` (extension minima)
**Tipo:** Sistema ECS (FixedUpdate) — rewire de propagacion existente.
**Onda:** A — Requiere SF-3. Paralelo con SF-4 y SF-5.
**Estado:** ⏳ Pendiente

## Contexto: que ya existe

`propagate_nuclei_system` emite energia de cada nucleo a TODAS las celdas en radio en UN tick. `diffusion_transfer()` existe pero no se usa. `EnergyFieldGrid` tiene dirty bitset para tracking de celdas modificadas.

La ecuacion `nucleus_intensity_at()` calcula decay espacial correctamente. Lo que falta es la dimension temporal.

**Lo que NO existe:**
1. **Emision gradual.** El nucleo deberia emitir solo a celdas dentro del frente actual.
2. **Difusion inter-celda.** Las celdas deberian difundir a vecinos cada tick (ya existe math, falta wiring).
3. **Tracking de emision.** No se sabe en que tick empezo cada nucleo a emitir.

## Objetivo

Modificar `propagate_nuclei_system` para que use el frente de onda de SF-3. Agregar un sistema de difusion vecinal que corre despues de la propagacion.

**Resultado emergente:** Un nucleo nuevo en tick 100 solo afecta celdas inmediatas. Para tick 108, el frente llego a 8 celdas de radio. Las entidades cercanas reaccionan primero, las lejanas despues. Causalidad espaciotemporal real.

## Responsabilidades

### SF-6A: Resource `PropagationMode`

```rust
/// Controla si la propagacion usa modelo legacy (instant) o multi-tick (frente de onda).
#[derive(Resource, Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum PropagationMode {
    #[default]
    Legacy,      // All-at-once (comportamiento actual)
    WaveFront,   // Multi-tick con velocidad finita
}
```

- Default: `Legacy` → backward compatible.
- Mapas nuevos pueden setear `WaveFront` via config.

### SF-6B: Componente `NucleusEmissionState`

```rust
/// Tick en que este nucleo empezo a emitir (para calcular frente de onda).
#[derive(Component, Clone, Copy, Debug, Default)]
#[component(storage = "SparseSet")]
pub struct NucleusEmissionState {
    pub start_tick: u64,
    pub last_front_radius: f32,
}
```

- 2 campos. SparseSet. Solo entidades con `EnergyNucleus`.
- Se inserta automaticamente al detectar nucleo sin estado.

### SF-6C: Modificacion de `propagate_nuclei_system`

Logica actual (pseudocodigo):
```
for nucleus in nuclei:
    for cell in grid where distance < radius:
        cell.qe += intensity_at(distance)
```

Logica nueva (WaveFront mode):
```
for nucleus in nuclei:
    front = propagation_front_radius(emission_tick, current_tick, speed, cell_size)
    prev_front = emission_state.last_front_radius
    for cell in grid where prev_front < distance <= front:
        intensity = propagation_intensity_at_tick(distance, front, base, decay, damping)
        cell.qe += intensity
    emission_state.last_front_radius = front
```

- Solo emite a la **corona** entre el frente anterior y el nuevo. No re-emite a celdas ya cubiertas.
- `if mode == PropagationMode::Legacy` → comportamiento original sin cambios.
- Guard: `front <= prev_front` → no hay expansion este tick, skip.

### SF-6D: `diffuse_propagation_system` (nuevo)

```rust
/// Difusion lateral entre celdas vecinas (4-connected).
/// Corre despues de propagate_nuclei_system.
pub fn diffuse_propagation_system(
    mode: Res<PropagationMode>,
    mut grid: ResMut<EnergyFieldGrid>,
) { ... }
```

- **Phase:** `Phase::ThermodynamicLayer`, `.after(propagate_nuclei_system)`.
- Guard: `if *mode == PropagationMode::Legacy { return; }`.
- Guard: `if !grid.is_changed() { return; }`.
- Algoritmo:
  1. Iterar celdas dirty (via `dirty_words` bitset).
  2. Por cada celda dirty, calcular `diffusion_delta()` con 4 vecinos.
  3. Aplicar deltas (double-buffer o delta accumulator para evitar order-dependency).
  4. Budget: max `DIFFUSION_BUDGET_MAX` celdas por tick.
- **Double-buffer:** Leer de `accumulated_qe`, acumular deltas en `Vec<f32>`, aplicar al final. Evita que el orden de iteracion afecte el resultado.

### SF-6E: Registro en pipeline

- `PropagationMode` como Resource (default: Legacy).
- `NucleusEmissionState` insertado por `propagate_nuclei_system` si falta.
- `diffuse_propagation_system` en `Phase::ThermodynamicLayer`, `.after(propagate_nuclei_system)`, solo en WaveFront mode.

## Tacticas

- **Legacy por defecto.** `PropagationMode::Legacy` = 100% backward compatible. Zero cambio visual/behavioral.
- **Corona, no disco completo.** Emitir solo en la franja nueva evita re-procesamiento.
- **Double-buffer difusion.** Orden de iteracion no afecta resultado → determinista.
- **Dirty tracking existente.** `EnergyFieldGrid.dirty_words` ya marca celdas modificadas. Reutilizar.

## NO hace

- No modifica `nucleus_intensity_at()` — las ecuaciones SF-3 lo wrappean.
- No implementa propagacion por tipo (sonido vs luz) — todo es qe uniforme.
- No implementa reflexion/refraccion en bordes de terreno.
- No modifica sensory perception — la latencia sensorial emerge del campo con delay.

## Dependencias

- SF-3 (`propagation_front_radius`, `propagation_intensity_at_tick`, `diffusion_delta`, `diffusion_budget`).
- `worldgen/systems/propagation.rs` — `propagate_nuclei_system` (se modifica).
- `worldgen/propagation.rs` — `nucleus_intensity_at()` (se reutiliza).
- `worldgen/field_grid.rs` — `EnergyFieldGrid`, dirty bitset.

## Criterios de aceptacion

### SF-6C (Propagacion multi-tick)
- Test (MinimalPlugins): insertar nucleo en centro de grid 16x16, mode=WaveFront, speed=4 → tick 1: solo celdas a distancia ≤4 tienen qe>0.
- Test: tick 4 → celdas a distancia ≤16 tienen qe>0 (frente completo).
- Test: mode=Legacy → comportamiento identico al original (regresion).
- Test: determinismo — 100 ticks produce mismo estado que segunda ejecucion.

### SF-6D (Difusion)
- Test: grid uniforme (todo qe=100) → difusion delta = 0 (equilibrio).
- Test: grid con gradiente (celda 100, vecino 0) → difusion reduce gradiente.
- Test: budget < total dirty → solo procesa budget celdas.
- Test: mode=Legacy → difusion no corre (early return).

### General
- `cargo test --lib` sin regresion.
- Mapas existentes con mode=Legacy → identico comportamiento visual.

## Referencias

- SF-3 — Ecuaciones de propagacion multi-tick
- `src/worldgen/systems/propagation.rs:81-172` — `propagate_nuclei_system()` (se modifica)
- `src/worldgen/propagation.rs:129-140` — `diffusion_transfer()` (se depreca)
- `src/worldgen/field_grid.rs` — dirty bitset infrastructure
