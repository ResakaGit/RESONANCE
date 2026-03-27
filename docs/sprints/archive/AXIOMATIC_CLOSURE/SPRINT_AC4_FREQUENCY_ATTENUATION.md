# Sprint AC-4 — Frequency Purity Attenuation with Distance

**Módulo:** `src/blueprint/equations/signal_propagation.rs` (extensión), `src/blueprint/constants/`
**Tipo:** Ecuación pura + constante + integración en sistemas de percepción y catalysis
**Eje axiomático:** Axioma 7 × Axioma 8
**Estado:** ⏳ Pendiente
**Oleada:** A (sin dependencias, paralelo con AC-1)

---

## Contexto: qué ya existe

**Lo que SÍ existe:**

- `blueprint/equations/signal_propagation.rs` — `propagation_intensity_at_tick()` modela:
  - Amplitud: `source_qe × exp(-decay × dist) × damping^ticks`
  - Frente de onda: señal no existe fuera del frente (simulación de velocidad finita)
  - Estos cubren la parte *cuánto* de la señal llega. No el *qué tan limpia* llega.

**Lo que NO existe:**

1. Ninguna ecuación modela la degradación de pureza de frecuencia con la distancia.
2. Las señales que llegan de lejos se reciben con la frecuencia exacta de la fuente.
3. Un depredador a distancia 100 detecta la frecuencia precisa de una presa a distancia 100.
4. El coupling de entrainment (AC-2) no tiene escala natural de distancia.

---

## Objetivo

La frecuencia de una señal percibida debe ser "borrosa" proporcional a la distancia.
A corta distancia: identidad precisa. A larga distancia: sólo banda aproximada.

Esto produce:
- Radio de acción natural para el entrainment (AC-2)
- Catalysis de largo alcance menos precisa (interferencia menos predecible)
- Depredadores deben acercarse para identificar la frecuencia exacta de la presa
- Información de identidad como recurso escaso que se degrada

```
freq_purity(d) = exp(-d / λ_coherence)
    d=0:    purity=1.0  → frecuencia exacta
    d=λ:    purity≈0.37 → identidad estimada
    d=3λ:   purity≈0.05 → solo banda detectable
```

---

## Responsabilidades

### AC-4A: Ecuación pura

```rust
// src/blueprint/equations/signal_propagation.rs — agregar al módulo existente

/// Pureza de frecuencia percibida a distancia `distance`.
/// Retorna factor [0.0, 1.0] donde:
///   1.0 = frecuencia exacta (contacto directo)
///   0.0 = frecuencia indetectable (demasiado lejos)
/// Ecuación: exp(-distance / lambda_coherence)
/// λ recomendado < λ_amplitude para que la identidad sea más escasa que la señal
pub fn frequency_purity_at_distance(distance: f32, lambda_coherence: f32) -> f32 {
    let lambda = lambda_coherence.max(0.001);
    (-distance / lambda).exp()
}

/// Coupling de entrainment modulado por pureza de frecuencia.
/// A baja pureza, la fuente no puede "tirar" del receptor con fidelidad.
/// Esto da un radio natural al sistema Kuramoto de AC-2.
pub fn entrainment_coupling_at_distance(
    base_coupling: f32,
    distance: f32,
    lambda_coherence: f32,
) -> f32 {
    base_coupling * frequency_purity_at_distance(distance, lambda_coherence)
}
```

### AC-4B: Constantes

```rust
// src/blueprint/constants/ — nuevo shard: signal_coherence.rs

/// Distancia a la que la pureza de frecuencia cae a 1/e ≈ 37%.
/// Más corto que la atenuación de amplitud (PROPAGATION_DECAY_RATE).
/// Justificación: la identidad de frecuencia es más frágil que la presencia de señal.
pub const FREQ_COHERENCE_DECAY_LAMBDA: f32 = 12.0;  // cells

/// Por debajo de esta pureza, la frecuencia es indetectable.
/// Se usa como umbral para: skip interference, skip entrainment.
pub const FREQ_PURITY_PERCEPTION_THRESHOLD: f32 = 0.1;
```

```rust
// src/blueprint/constants/mod.rs — re-exportar
pub mod signal_coherence;
pub use signal_coherence::*;
```

### AC-4C: Integración en behavior (percepción de amenazas)

```rust
// src/simulation/behavior.rs — en BehaviorEvaluateThreat

// ANTES: la frecuencia del objetivo se lee directamente
let target_freq = target_osc.frequency_hz;

// DESPUÉS: la frecuencia se pondera por pureza según distancia
let dist = transform.translation.distance(target_transform.translation);
let purity = signal_propagation_eq::frequency_purity_at_distance(
    dist, FREQ_COHERENCE_DECAY_LAMBDA,
);
// Si pureza < threshold: solo detecto que es "algo", no qué frecuencia
// Uso para: interference estimate en BehaviorDecide
let effective_freq_known = purity >= FREQ_PURITY_PERCEPTION_THRESHOLD;
```

**Nota:** Aquí no se muta `OscillatorySignature`. La pureza es un factor de
*percepción*, no de estado. El receptor puede estar incertidumbre del emisor
sin que el emisor cambie.

### AC-4D: Integración en reactions (catalysis de largo alcance)

```rust
// src/simulation/reactions.rs — en catalysis computation

// DESPUÉS de calcular interference_raw:
let dist = spell_origin.distance(target_pos);
let purity = signal_propagation_eq::frequency_purity_at_distance(
    dist, FREQ_COHERENCE_DECAY_LAMBDA,
);
// Spells de largo alcance tienen interferencia menos predecible
// A baja pureza: el coseno pierde su amplitud pero la incertidumbre aumenta
// Implementación simple: modular el factor por purity
let effective_interference = interference_raw * purity + (1.0 - purity) * CATALYSIS_NEUTRAL;
// CATALYSIS_NEUTRAL = 0.0 (neutral, ni daño ni curación)
```

---

## No hace

- No cambia cómo se propagan los campos energéticos del worldgen (eso es amplitud, no identidad).
- No agrega componente nuevo a ninguna entidad — purity es cálculo puntual.
- No introduce memoria de percepción de frecuencia (eso sería un componente nuevo de LOD sensorial).

---

## Criterios de aceptación

### AC-4A (Ecuaciones)

```
frequency_purity_at_distance(0.0, 12.0)    → 1.0
frequency_purity_at_distance(12.0, 12.0)   → ≈ 0.368   (1/e)
frequency_purity_at_distance(36.0, 12.0)   → ≈ 0.050   (3 lambdas)
frequency_purity_at_distance(100.0, 12.0)  → < 0.01    (efectivamente 0)
frequency_purity_at_distance(0.0, 0.0)     → 1.0       (lambda=0 protegido con .max(0.001))

entrainment_coupling_at_distance(1.0, 12.0, 12.0)  → ≈ 0.368
entrainment_coupling_at_distance(1.0, 0.0,  12.0)  → 1.0
```

### AC-4C (Comportamiento)

Test (MinimalPlugins):
- Entidad a distancia 0 del objetivo: `effective_freq_known = true`.
- Entidad a distancia 100 del objetivo: `effective_freq_known = false` (purity < threshold).
- La transición ocurre en ~`3 × FREQ_COHERENCE_DECAY_LAMBDA` distancia.

### AC-4D (Catalysis)

Test:
- Spell lanzado a distancia 0: `effective_interference ≈ interference_raw`.
- Spell lanzado a distancia 100: `effective_interference ≈ CATALYSIS_NEUTRAL`.
- `cargo test --lib` sin regresión.

---

## Por qué λ_coherence < λ_amplitude

La atenuación de amplitud (presencia de señal) y la atenuación de pureza (identidad
de señal) son fenómenos distintos con escalas distintas.

- Amplitud: ¿llegó algo?
- Pureza: ¿sé exactamente qué era?

En física real (acústica, radio), la identidad se pierde antes que la señal. Un mensaje
llega con ruido antes de desaparecer completamente. En la simulación, esta distinción
produce que los sensores de largo alcance son útiles para detectar presencia pero no
para identificar frecuencia — lo que da valor estratégico a la proximidad.

---

## Dependencias

- `blueprint/equations/signal_propagation.rs` — se extiende, no se reemplaza
- `simulation/behavior.rs` — integración en BehaviorEvaluateThreat
- `simulation/reactions.rs` — integración en catalysis
- `blueprint/constants/mod.rs` — para re-exportar el nuevo shard

Desbloquea:
- **AC-2** (Entrainment): usa `entrainment_coupling_at_distance()` como coupling del Kuramoto

---

## Referencias

- `src/blueprint/equations/signal_propagation.rs:23-67` — propagación existente (amplitud)
- `src/simulation/behavior.rs` — `BehaviorEvaluateThreat` donde se integra
- `src/simulation/reactions.rs:166-173` — catalysis existente
- `docs/design/AXIOMATIC_CLOSURE.md §5` — Frequency Attenuation design
- Axioma 7: "interaction intensity is monotonically decreasing in distance(A,B)"
- Axioma 8: "every energy concentration oscillates at a characteristic frequency"
