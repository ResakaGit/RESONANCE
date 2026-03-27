# Sprint AC-2 — Entrainment System (Kuramoto)

**Módulo:** `src/simulation/` (nuevo `entrainment.rs`), `src/blueprint/equations/emergence/`
**Tipo:** Ecuaciones puras (extensión) + sistema nuevo
**Eje axiomático:** Axioma 8 consecuencia — gradual alignment of frequency
**Estado:** 🔒 Requiere AC-4
**Oleada:** B

---

## Contexto: qué ya existe

**Lo que SÍ existe:**

- `layers/oscillatory.rs` — `OscillatorySignature { frequency_hz, phase }`. Componente con getters/setters.
- `layers/homeostasis.rs` — `Homeostasis { adapt_rate, target_freq }`. Ya hace entrainment hacia 1 host.
- `blueprint/equations/emergence/culture.rs:172` — `entrainment_possible(freq_a, freq_b, coupling_strength)`:
  implementa la condición Kuramoto `|ω_a - ω_b| × 2π < coupling_strength`.
- `simulation/thermodynamic/structural_runtime.rs` — `homeostasis_system`: alinea hacia host de entorno.
- `world/mod.rs` — `SpatialIndex::query_radius()` para vecinos.
- AC-4 (prereq) — `entrainment_coupling_at_distance()` da el coupling correcto.

**Lo que NO existe:**

1. **Entrainment multi-vecino.** Solo hay 1-a-1 con el host. No hay sistema que escanee
   vecinos dentro de un radio y aplique el modelo Kuramoto.
2. **Delta de frecuencia por vecino.** No hay función que calcule `Δω_i` por un step
   desde múltiples fuentes.
3. El entrainment actual en `homeostasis` no muta `frequency_hz` directamente — muta
   `target_freq` y el sistema de homeostasis aplica la presión. Necesitamos que el
   entrainment entre vecinos también respete este canal.

---

## Objetivo

Implementar el modelo de Kuramoto simplificado para frecuencias entre entidades vecinas.
Cada entidad escanea vecinos dentro de `ENTRAINMENT_RADIUS`, y recibe presión gradual
para alinear su frecuencia.

```
dω_i/dt ≈ Σ_j  coupling(qe_j, dist_ij) × sin(ω_j - ω_i)

donde:
    coupling(qe_j, dist) = entrainment_coupling_at_distance(BASE_COUPLING × qe_factor, dist, λ)
    condición Kuramoto:   |ω_j - ω_i| < GAP_MAX  (sin esto, bandas distintas no se afectan)
    Δω por tick:          clampeado a MAX_DELTA_PER_TICK (evolución gradual, no instantánea)
```

---

## Responsabilidades

### AC-2A: Ecuaciones puras (extensión de culture.rs)

```rust
// src/blueprint/equations/emergence/entrainment.rs  (nuevo — separar de culture.rs)

use crate::blueprint::constants::entrainment::*;
use crate::blueprint::equations::signal_propagation::entrainment_coupling_at_distance;

/// Delta de frecuencia que ejerce una fuente sobre el receptor en un tick.
/// Implementa un paso del modelo de Kuramoto simplificado.
/// Retorna Δω en Hz por tick (puede ser negativo o positivo).
pub fn entrainment_delta_from_source(
    self_freq: f32,
    source_freq: f32,
    coupling: f32,  // ya modulado por distancia (de AC-4)
) -> f32 {
    let gap = source_freq - self_freq;
    // Condición Kuramoto: solo actúa si la brecha es menor que el umbral
    if gap.abs() > ENTRAINMENT_MAX_GAP_HZ {
        return 0.0;
    }
    // sin(gap / normalizer) — para pequeños gaps ≈ gap, da comportamiento lineal cercano
    let normalized = gap / ENTRAINMENT_FREQ_NORMALIZER;
    coupling * normalized.sin()
}

/// Agrega los deltas de hasta N fuentes. Retorna Δω total del tick.
/// Usa slice de tamaño fijo para evitar allocation.
pub fn aggregate_entrainment_delta(
    self_freq: f32,
    sources: &[(f32, f32)],  // (source_freq, coupling)
    max_sources: usize,
) -> f32 {
    sources.iter()
        .take(max_sources)
        .map(|&(src_freq, coupling)| {
            entrainment_delta_from_source(self_freq, src_freq, coupling)
        })
        .sum::<f32>()
        .clamp(-ENTRAINMENT_MAX_DELTA_PER_TICK, ENTRAINMENT_MAX_DELTA_PER_TICK)
}

/// Aplica el delta de entrainment, respetando los límites de banda.
/// La banda de un elemento no puede cruzarse por entrainment.
pub fn apply_entrainment(current_freq: f32, delta: f32, freq_min: f32, freq_max: f32) -> f32 {
    (current_freq + delta).clamp(freq_min, freq_max)
}
```

### AC-2B: Constantes

```rust
// src/blueprint/constants/entrainment.rs  (nuevo)

/// Radio dentro del cual otra entidad puede ejercer presión de entrainment.
pub const ENTRAINMENT_RADIUS: f32 = 8.0;  // cells

/// Coupling base antes de modulación por distancia y qe.
/// Fracción del gap que se cierra por tick en contacto directo.
pub const ENTRAINMENT_BASE_COUPLING: f32 = 0.04;

/// Máximo cambio de frecuencia por tick por todas las fuentes combinadas.
/// Gradual: evita que un grupo homogéneo "aplaste" instantáneamente a un outlier.
pub const ENTRAINMENT_MAX_DELTA_PER_TICK: f32 = 0.3;  // Hz por tick

/// Brecha máxima de Hz para que haya acoplamiento.
/// Protege la identidad de elementos distintos (Terra 75 Hz, Lux 1000 Hz → sin acoplamiento).
/// Se configura para que sólo actúe dentro de la misma banda o bands adyacentes.
pub const ENTRAINMENT_MAX_GAP_HZ: f32 = 60.0;  // Hz

/// Normalizador del seno en la ecuación Kuramoto.
pub const ENTRAINMENT_FREQ_NORMALIZER: f32 = 100.0;

/// qe mínimo de una fuente para que ejerza presión de entrainment.
/// Entidades casi muertas no "jalan" a otras.
pub const ENTRAINMENT_MIN_SOURCE_QE: f32 = 5.0;

/// Máximo de fuentes consideradas por entidad por tick (evita O(n²) en hot path).
pub const ENTRAINMENT_MAX_SOURCES: usize = 8;
```

### AC-2C: Sistema

```rust
// src/simulation/entrainment.rs  (nuevo)

use bevy::prelude::*;
use crate::layers::{OscillatorySignature, BaseEnergy};
use crate::world::SpatialIndex;
use crate::simulation::mod::Phase;
use crate::blueprint::{equations::emergence::entrainment as entrainment_eq, constants::*};
use crate::blueprint::equations::signal_propagation::entrainment_coupling_at_distance;

/// Gradual frequency alignment toward neighboring entities via Kuramoto model.
/// Phase::MetabolicLayer, after homeostasis_system, before reactions_system.
pub fn entrainment_system(
    mut oscillators: Query<(Entity, &mut OscillatorySignature, &Transform, &BaseEnergy)>,
    all_oscillators: Query<(Entity, &OscillatorySignature, &Transform, &BaseEnergy)>,
    spatial: Res<SpatialIndex>,
) {
    // Collect solo posiciones + freq + qe para el lookup de vecinos
    // (no mutable, no conflict)
    for (entity, mut osc, transform, energy) in &mut oscillators {
        let pos = transform.translation.truncate();

        // Recolectar vecinos dentro del radio
        let neighbor_entities = spatial.query_radius(pos, ENTRAINMENT_RADIUS);

        // Calcular coupling para cada vecino
        let mut sources: [(f32, f32); ENTRAINMENT_MAX_SOURCES] =
            [(0.0, 0.0); ENTRAINMENT_MAX_SOURCES];
        let mut count = 0;

        for &neighbor_entity in neighbor_entities.iter().take(ENTRAINMENT_MAX_SOURCES) {
            if neighbor_entity == entity { continue; }
            let Ok((_, neighbor_osc, neighbor_transform, neighbor_energy)) =
                all_oscillators.get(neighbor_entity) else { continue; };

            if neighbor_energy.qe() < ENTRAINMENT_MIN_SOURCE_QE { continue; }

            let dist = transform.translation.distance(neighbor_transform.translation);
            let coupling = entrainment_coupling_at_distance(
                ENTRAINMENT_BASE_COUPLING, dist, FREQ_COHERENCE_DECAY_LAMBDA,
            );
            sources[count] = (neighbor_osc.frequency_hz(), coupling);
            count += 1;
        }

        let delta = entrainment_eq::aggregate_entrainment_delta(
            osc.frequency_hz(),
            &sources[..count],
            ENTRAINMENT_MAX_SOURCES,
        );

        if delta.abs() < 0.001 { continue; }  // change detection guard

        let new_freq = entrainment_eq::apply_entrainment(
            osc.frequency_hz(), delta,
            osc.band_freq_min(), osc.band_freq_max(),
        );
        osc.set_frequency_hz(new_freq);
    }
}
```

**Nota sobre `band_freq_min/max`:** `OscillatorySignature` necesita exponer el rango
de su banda elemental para que `apply_entrainment` pueda clampear. Si el componente
no lo tiene, se puede derivar de la frecuencia actual con una lookup a las constantes
de bandas elementales.

### AC-2D: Registro en plugin

```rust
// src/plugins/simulation_plugin.rs o metabolic_plugin.rs — agregar:

app.add_systems(
    FixedUpdate,
    entrainment_system
        .in_set(Phase::MetabolicLayer)
        .after(homeostasis_system)
        .before(reactions_system),
);
```

---

## No hace

- No modifica la lógica de `homeostasis_system` (que sigue alineando hacia el host de entorno).
- No implementa "conversión de elemento" — las bandas actúan como atractores. Cruzar de
  Terra a Aqua requeriría acumulación enorme de presión (lo que en la práctica no ocurre
  con el coupling bajo).
- No guarda historial de entrainment — es completamente stateless por tick.
- No requiere nueva componente.

---

## Criterios de aceptación

### AC-2A (Ecuaciones)

```
entrainment_delta_from_source(75.0, 75.0, 0.1)    → 0.0  (misma frec, sin delta)
entrainment_delta_from_source(75.0, 80.0, 0.1)    → pequeño positivo (empuja hacia 80)
entrainment_delta_from_source(75.0, 135.0, 0.1)   → 0.0  (gap > MAX_GAP_HZ=60, sin acoplamiento)
entrainment_delta_from_source(75.0, 450.0, 0.1)   → 0.0  (bandas distintas, sin acoplamiento)
aggregate clamp: si sum > MAX_DELTA_PER_TICK       → clampeado

apply_entrainment(75.0, 5.0, 50.0, 100.0)         → 80.0
apply_entrainment(75.0, 30.0, 50.0, 100.0)        → 100.0  (clampeado al max de banda)
```

### AC-2C (Sistema)

Test (MinimalPlugins):
- 3 entidades Terra con freq [74, 75, 76] Hz → tras 100 ticks convergen hacia 75 Hz.
- 1 entidad Terra (75 Hz) + 1 entidad Lux (1000 Hz), distance 5 → sin cambio (gap > MAX).
- 2 entidades Terra (75 Hz) a distance > ENTRAINMENT_RADIUS × 3 → sin cambio (lejos).
- Después de AC-4 integrada: coupling cae con distancia → convergencia más lenta a distancia 8 que a 0.

### General

- `cargo test --lib` sin regresión.
- Sin Vec allocation in hot path (sources array fijo).
- OscillatorySignature.frequency_hz no sale de banda elemental tras entrainment.

---

## Dependencias

- AC-4 — `entrainment_coupling_at_distance()` (coupling modulado por distancia)
- `layers/oscillatory.rs` — `set_frequency_hz()`, `band_freq_min()`, `band_freq_max()` (verificar API)
- `blueprint/equations/emergence/culture.rs:172` — `entrainment_possible()` como referencia
- `world/mod.rs` — `SpatialIndex::query_radius()`
- `simulation/thermodynamic/structural_runtime.rs` — `homeostasis_system` (ordering reference)

---

## Referencias

- `src/layers/oscillatory.rs` — `OscillatorySignature` API
- `src/blueprint/equations/emergence/culture.rs:172` — condición Kuramoto existente
- `docs/design/AXIOMATIC_CLOSURE.md §3 Tier 2` — Entrainment impact analysis
- `docs/arquitectura/blueprint_axiomatic_closure.md §3` — Entrainment contract
- Axioma 8: "The gradual alignment of frequency between interacting systems (entrainment)"
