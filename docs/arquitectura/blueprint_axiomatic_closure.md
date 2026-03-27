# Blueprint: Axiomatic Closure — Cross-Axiom Dynamics

**Módulo:** múltiples (`equations/`, `simulation/metabolic/`, `simulation/reactions.rs`, `layers/`)
**Tipo:** Runtime contracts — qué lee, qué escribe, en qué fase, qué invariante protege
**Track sprints:** `docs/sprints/AXIOMATIC_CLOSURE/`
**Diseño:** `docs/design/AXIOMATIC_CLOSURE.md`

---

## 1. Contexto: gaps en la composición axiomática

Los 8 axiomas tienen implementación individual. Lo que falta son los sistemas que
implementan sus *consecuencias compositivas* — los fenómenos que solo existen cuando
dos axiomas operan juntos.

Audit de gaps (2026-03-25):

| ID | Composición | Ecuación existe | Sistema activo | Gap |
|----|-------------|----------------|----------------|-----|
| AC-1 | Axioma 3 × 8 | ✓ en catalysis | ✗ metabolic | Interference no modula extracción |
| AC-2 | Axioma 8 consequence | ✓ Kuramoto | ✗ entre vecinos | Solo entrainment al host |
| AC-3 | Axioma 6 × 8 | ✓ comprehensivo | ✗ retroalimentado | Cultura desacoplada de frecuencia |
| AC-4 | Axioma 7 × 8 | ✗ | ✗ | Señales no pierden coherencia con distancia |
| AC-5 | Axioma 3 game theory | ✓ esqueleto | ✗ | Cooperación sin evaluación |

---

## 2. AC-1 — Interference × Metabolic Extraction

### Contrato

**Lee:**
- `OscillatorySignature { frequency_hz, phase }` — extractor y objetivo
- `SimulationTick` — tiempo actual (t para el coseno)
- `EnergyPool` — disponible en el objetivo
- `ExtractionProfile` — tipo de extracción (proporcional, greedy, etc.)

**Escribe:**
- Modifica el quantum extraído antes de aplicarlo al pool objetivo
- No escribe componentes nuevos — modula el resultado de funciones existentes

**Fase:** `Phase::MetabolicLayer`, within photosynthesis/trophic/osmosis systems

**Invariante:**
```
extracted_qe_final = extracted_qe_raw × interference_clamp01

donde interference_clamp01 = cos(2π × Δfreq × t + Δphase).max(0.0)
    rango: [0.0, 1.0]   — metabolic interference no puede ser negativa
    rango catalysis: [-1.0, 1.0]  — distinto, no mezclar
```

**Por qué no [-1, 1] aquí:**
La extracción metabólica es acceso diferenciado al recurso, no daño activo.
`interference < 0` significaría que el depredador devuelve energía a la presa —
que viola Axioma 3. El clamp a 0 representa fallo de extracción, no inversión.

### Nuevas funciones puras (en `blueprint/equations/energy_competition/`)

```rust
/// Modifica el quantum de extracción por la interferencia oscilatoria entre
/// extractor y objetivo. Retorna el factor [0.0, 1.0].
/// Ecuación: cos(Δfreq × t + Δphase).max(0.0)
pub fn metabolic_interference_factor(
    extractor_freq: f32, extractor_phase: f32,
    target_freq: f32,    target_phase: f32,
    t: f32,
) -> f32

/// Aplica el factor a la cantidad extraída.
/// extracted_raw: resultado de extract_* antes de este módulo.
pub fn apply_metabolic_interference(
    extracted_raw: f32,
    factor: f32,
) -> f32
```

### Integración en sistemas existentes

Los tres sistemas que deben llamar a `apply_metabolic_interference`:

1. `photosynthesis_system` — extracción del campo de energía ambiental
   - El "target" es el `EnergyFieldGrid` cell; su frecuencia = banda del terreno
   - Factor de interferencia Terra(75Hz) vs Terra-ground → ~1.0 (nativo)
   - Factor Ignis(450Hz) vs Terra-ground(75Hz) → pequeño (Ignis en terreno Terra = hambre)

2. `trophic_predation_system` — extracción depredador → presa
   - Target = presa con su `OscillatorySignature`
   - Depredadores resonantes extraen con máxima eficiencia
   - Depredadores inarmónicos extraen casi nada → presión de especialización

3. `osmosis_system` — difusión entre entidades adyacentes
   - Target = entidad vecina
   - Misma banda → alta difusión (equilibrio rápido dentro de la especie)
   - Bandas distintas → difusión lenta (barrera de mezcla)

---

## 3. AC-2 — Entrainment System

### Contrato

**Lee:**
- `OscillatorySignature { frequency_hz, phase }` — propia y de vecinos
- `Transform` — posición propia y vecinos
- `BaseEnergy { qe }` — qe de vecinos como peso de acoplamiento
- `SpatialIndex` — vecinos dentro de `ENTRAINMENT_RADIUS`
- `SimulationTick` — para la condición Kuramoto

**Escribe:**
- `OscillatorySignature::frequency_hz` — mutado gradualmente hacia media ponderada
- `OscillatorySignature::phase` — ajustado por entrainment parcial

**Fase:** `Phase::MetabolicLayer`, after `homeostasis_system`, before `reactions_system`

**Invariante:**
```
|frequency_hz_after - frequency_hz_before| ≤ ENTRAINMENT_MAX_DELTA_PER_TICK

frequency_hz stays in [freq_min_band, freq_max_band]:
    No entrainment puede sacar una entidad de su banda elemental
    (las bandas son energéticamente estables, el entrainment es gradual)

Condición Kuramoto requerida:
    |ω_target - ω_self| × 2π < coupling_strength(qe_target, dist)
    → Si la brecha es demasiado grande, no hay acoplamiento
    → Protege la identidad de elementos distintos
```

### Nuevas funciones puras (en `blueprint/equations/emergence/` — cultura.rs ya tiene base)

```rust
/// Peso de acoplamiento del Kuramoto: decrece con distancia, crece con qe fuente.
pub fn kuramoto_coupling(source_qe: f32, distance: f32, base_coupling: f32) -> f32

/// Delta de frecuencia por un paso de entrainment.
/// Retorna el cambio Δω_i por interacción con una fuente.
pub fn entrainment_delta(
    self_freq: f32, source_freq: f32,
    coupling: f32, max_delta: f32,
) -> f32

/// Suma de deltas de múltiples fuentes. Stack array, max N vecinos.
pub fn aggregate_entrainment<const N: usize>(
    self_freq: f32,
    sources: &[(f32, f32)], // (freq, coupling)
) -> f32
```

### Sistema a implementar

```rust
/// Gradual frequency alignment toward neighboring entities (Kuramoto model).
/// Phase::MetabolicLayer, after homeostasis_system.
pub fn entrainment_system(
    mut oscillators: Query<(
        Entity, &mut OscillatorySignature, &Transform, &BaseEnergy,
    )>,
    all_oscillators: Query<(Entity, &OscillatorySignature, &Transform, &BaseEnergy)>,
    spatial: Res<SpatialIndex>,
    config: Res<EntrainmentConfig>,
    tick: Res<SimulationTick>,
) {
    // Por cada entidad: query vecinos dentro de ENTRAINMENT_RADIUS
    // Para cada vecino: calcular coupling (qe_vecino, distancia)
    // Si condición Kuramoto cumplida: sumar delta
    // Aplicar delta clampeado a MAX_DELTA_PER_TICK
    // Guardar nuevo frequency_hz con change-detection guard
}
```

### Constantes nuevas

```rust
// src/blueprint/constants/ — nuevo shard entrainment.rs
pub const ENTRAINMENT_RADIUS: f32 = 8.0;           // cells
pub const ENTRAINMENT_BASE_COUPLING: f32 = 0.05;   // fracción de gap por tick
pub const ENTRAINMENT_MAX_DELTA_PER_TICK: f32 = 0.5; // Hz por tick
pub const ENTRAINMENT_MIN_QE_THRESHOLD: f32 = 5.0;  // qe mínimo para ser fuente
```

---

## 4. AC-3 — Culture Coherence Loop

### Contrato

**Lee:**
- `OscillatorySignature { frequency_hz }` de todos los miembros del grupo
- `CulturalMemory { memes }` — memoria cultural del target
- `PackMembership` o `Faction` — identificador de grupo

**Escribe:**
- Modifica la probabilidad de imitar en `cultural_transmission_system`
- No escribe nuevo componente — actualiza el criterio de `should_imitate()`

**Fase:** `Phase::MetabolicLayer`, within cultural transmission system

**Nueva ecuación (en `blueprint/equations/emergence/culture.rs`)**

```rust
/// Factor de afinidad de frecuencia para imitación.
/// Alta afinidad → más probabilidad de imitar.
/// Rango [0.0, 1.0] donde 1.0 = misma banda.
pub fn frequency_imitation_affinity(self_freq: f32, target_freq: f32) -> f32 {
    let normalized_gap = (target_freq - self_freq).abs() / FREQ_BAND_WIDTH_HZ;
    (-normalized_gap * IMITATION_FREQ_DECAY).exp()
}

/// Coherencia del grupo como bonus multiplicativo a la imitación.
/// Grupos con alta coherencia son más "contagiosos".
pub fn group_coherence_imitation_bonus(group_frequencies: &[f32]) -> f32 {
    let coherence = group_frequency_coherence(group_frequencies);
    1.0 + coherence * COHERENCE_IMITATION_MULTIPLIER
}
```

**Cambio en `should_imitate()` existente:**

```rust
// Antes (solo fitness):
fitness_ratio > IMITATION_THRESHOLD

// Después (fitness × frequencia × coherencia):
let affinity = frequency_imitation_affinity(self_freq, target_freq);
let group_bonus = group_coherence_imitation_bonus(&group_freqs);
fitness_ratio * affinity * group_bonus > IMITATION_THRESHOLD
```

---

## 5. AC-4 — Frequency Purity Attenuation with Distance

### Contrato

**Lee:**
- `OscillatorySignature { frequency_hz }` — fuente
- `distance` — entre fuente y receptor
- Config: `FREQ_COHERENCE_DECAY_RATE` (λ en e^(-d/λ))

**Escribe:**
- Nuevo retorno: `(amplitude: f32, freq_purity: f32)` de `propagation_intensity_at_tick`
- `freq_purity` usado en: entrainment (reduce coupling), catalysis (reduce accuracy), perception

**Fase:** Se integra en las ecuaciones puras — no nuevo sistema, sino extensión de los existentes

**Nueva función (en `blueprint/equations/signal_propagation.rs`)**

```rust
/// Pureza de frecuencia recibida a distancia.
/// A distancia 0: purity = 1.0 (frecuencia exacta)
/// A distancia > λ: purity decae — receptor solo puede inferir banda aproximada
/// Ecuación: exp(-distance / λ_coherence)
pub fn frequency_purity_at_distance(distance: f32, lambda_coherence: f32) -> f32 {
    (-distance / lambda_coherence.max(0.001)).exp()
}

/// Frecuencia percibida con ruido proporcional a la pérdida de pureza.
/// A baja pureza: la frecuencia aparente puede estar dentro de ±noise_band de la real.
/// Esta función retorna el factor de modulación para el coupling de entrainment.
pub fn perceived_freq_coupling_factor(freq_purity: f32) -> f32 {
    freq_purity   // [0, 1] — directo: baja pureza = bajo coupling
}
```

**Constante nueva:**
```rust
pub const FREQ_COHERENCE_DECAY_LAMBDA: f32 = 15.0;  // cells — coherencia cae a 1/e a 15 cells
// Nota: λ_coherence < λ_amplitude recomendado para que
// la identidad de frecuencia sea un bien más escaso que la señal en sí
```

**Integración en sistemas:**
1. `entrainment_system` (AC-2): al calcular `kuramoto_coupling()`, multiplica por `freq_purity`
2. `reactions_system` (catalysis): la interferencia de spells aplica `freq_purity` del lanzador
3. `behavior_evaluate_threat`: la detección de frecuencia de amenazas degrada con distancia

---

## 6. AC-5 — Cooperation Emergence (game theory)

### Contrato (diseño; implementación en Oleada D)

**Lee:**
- `ExtractionProfile` — tipo y capacidad de extracción de cada entidad
- `EnergyPool` — disponible en el pool objetivo disputado
- `PackMembership` — grupo actual
- Neighbourhood desde `SpatialIndex`

**Escribe:**
- Propone alianza: evento `AllianceProposedEvent { initiator, target, expected_gain_delta }`
- Propone deserción: evento `AllianceDefectEvent { defector, group_id, reason }`

**Ecuación central (ya existe en `blueprint/equations/emergence/symbiosis.rs`)**

```rust
pub fn is_symbiosis_stable(
    a_with_b: f32, a_without_b: f32,
    b_with_a: f32, b_without_a: f32,
) -> bool

// Para el sistema:
// a_with_b = extraction_estimate(a, pool, group_size + 1)
// a_without_b = extraction_estimate(a, pool, group_size_solo = 1)
```

**El sistema evalúa, no impone:**
- Si `is_symbiosis_stable()` para un par → emit `AllianceProposedEvent`
- El sistema existente de packs acepta o rechaza en base a facción/energy
- Defección cuando la condición deja de cumplirse por N ticks consecutivos

---

## 7. Orden de implementación y dependencias

```
Onda A (paralelo, sin dependencias):
    AC-1: Interference × Metabolic Extraction
    AC-4: Frequency Purity Attenuation

Onda B (requiere AC-4):
    AC-2: Entrainment System
    (AC-4 da rango correcto al coupling de Kuramoto)

Onda C (requiere AC-2):
    AC-3: Culture Coherence Loop
    (el loop es vacuo si las frecuencias no evolucionan)

Onda D (requiere AC-1):
    AC-5: Cooperation Emergence
    (los costos de extracción solo son reales cuando interference modula)
```

---

## 8. Invariantes globales preservados

```
Axioma 4 (Dissipation):
    interference_factor en extracción ∈ [0.0, 1.0]
    → extracted_final ≤ extracted_raw (nunca aumenta por interferencia)

Axioma 5 (Conservation):
    El pool parent conserva: qe_pool -= Σ extracted_final
    → extracted_final reducido por interference no viola conservation

Axioma 7 (Distance):
    freq_purity_at_distance es monotónica decreciente en distance
    → no hay distancias donde la pureza "rebota"

Axioma 8 (Oscillatory):
    frequency_hz ∈ [FREQ_MIN, FREQ_MAX] tras entrainment
    → banda elemental como atractor: entrainment no puede sacar de banda
    → si el entrainment empuja fuera, se clampea al borde de banda
```

---

## 9. Archivos que se tocan

| Archivo | AC-1 | AC-2 | AC-3 | AC-4 | AC-5 |
|---------|------|------|------|------|------|
| `blueprint/equations/energy_competition/metabolic_interference.rs` (nuevo) | ✓ | | | | |
| `blueprint/equations/emergence/culture.rs` | | | ✓ | | |
| `blueprint/equations/emergence/entrainment.rs` (nuevo) | | ✓ | | | |
| `blueprint/equations/signal_propagation.rs` | | | | ✓ | |
| `blueprint/constants/entrainment.rs` (nuevo) | | ✓ | | ✓ | |
| `simulation/metabolic/photosynthesis.rs` | ✓ | | | | |
| `simulation/metabolic/trophic.rs` | ✓ | | | | |
| `simulation/thermodynamic/osmosis.rs` | ✓ | | | | |
| `simulation/metabolic/social_communication.rs` | | | ✓ | | ✓ |
| `simulation/reactions.rs` | | | | ✓ | |
| `simulation/` — nuevo `entrainment.rs` | | ✓ | | | |
| `simulation/` — nuevo `cooperation.rs` | | | | | ✓ |
| `events.rs` | | | | | ✓ |
