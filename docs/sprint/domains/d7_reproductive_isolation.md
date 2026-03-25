# D7: Reproductive Isolation & Speciation

**Prioridad**: P1
**Phase**: `Phase::MorphologicalLayer` (con reproduction)
**Dependencias**: D2 (trophic role), population equations, lifecycle
**Systems**: 3

---

## Motivación Científica

La especiación requiere **aislamiento reproductivo**: dos poblaciones dejan de cruzarse y divergen. Los mecanismos principales son:

1. **Pre-cigótico**: Diferencia fenotípica impide apareamiento (frecuencias incompatibles)
2. **Ecológico**: Nichos distintos → timing/ubicación distintos
3. **Genético**: Drift acumula diferencias que hacen offspring inviables

En Resonance, la "frecuencia" (L2) es el análogo del ADN. Dos entidades con frecuencias muy distintas no pueden resonar constructivamente → offspring tendría frecuencia mezclada que no cae en ninguna banda → inviable.

---

## Ecuaciones Nuevas

```
src/blueprint/equations/speciation/mod.rs (NUEVO)
```

### E1: `phenotypic_distance(a: &InferenceProfile, b: &InferenceProfile) -> f32`
```
d = sqrt((a.growth - b.growth)² + (a.mobility - b.mobility)² +
         (a.branching - b.branching)² + (a.resilience - b.resilience)²)
```
Distancia euclidiana en espacio fenotípico 4D.

### E2: `frequency_compatibility(freq_a: f32, freq_b: f32, band_width: f32) -> f32`
```
delta = |freq_a - freq_b|
compatibility = (1 - delta / band_width).max(0.0)
```
Compatibilidad = 1 si misma frecuencia, 0 si diferencia > band_width.

### E3: `reproductive_viability(phenotypic_distance: f32, frequency_compat: f32) -> bool`
```
viable = phenotypic_distance < MAX_PHENOTYPIC_DISTANCE
       AND frequency_compat > MIN_FREQUENCY_COMPATIBILITY
```

### E4: `genetic_drift_per_generation(population_size: u32, base_drift: f32) -> f32`
```
drift = base_drift / sqrt(population_size)
```
Wright-Fisher: drift inversamente proporcional a sqrt(N).

---

## Constantes

```rust
pub const MAX_PHENOTYPIC_DISTANCE: f32 = 1.2;         // Beyond this, can't reproduce
pub const MIN_FREQUENCY_COMPATIBILITY: f32 = 0.3;     // Below this, offspring inviable
pub const GENETIC_DRIFT_BASE: f32 = 0.05;             // Max bias shift per generation
pub const SPECIATION_CHECK_INTERVAL: u32 = 60;        // Check every 60 ticks
pub const NICHE_FREQUENCY_SPECIALIZATION_RATE: f32 = 0.001; // Frequency drift toward local band
```

---

## Systems (3)

### S1: `reproductive_isolation_guard_system` (Transformer)
**Phase**: MorphologicalLayer, before reproduction_spawn_system
**Reads**: InferenceProfile, OscillatorySignature, Transform, SpatialIndex
**Writes**: ReproductionEligibility (SparseSet marker: eligible or not)
**Logic**:
1. For each entity ready to reproduce
2. Find nearest mate candidate (same faction, opposite or same sex if applicable)
3. Compute phenotypic_distance + frequency_compatibility
4. If viable → mark eligible. If not → block reproduction.

### S2: `genetic_drift_system` (Transformer)
**Phase**: MorphologicalLayer, every 60 ticks
**Reads**: InferenceProfile, OscillatorySignature (population stats from census)
**Writes**: InferenceProfile (tiny perturbation)
**Logic**: Apply `genetic_drift_per_generation()` scaled by local population size. Drift direction biased toward local conditions (frequency band of dominant cell).

### S3: `niche_frequency_specialization_system` (Transformer)
**Phase**: MorphologicalLayer, every 60 ticks
**Reads**: OscillatorySignature, Transform, EnergyFieldGrid (local dominant frequency)
**Writes**: OscillatorySignature (tiny drift toward local band)
**Logic**: Entities slowly drift toward the dominant frequency of their cell. Creates frequency clustering → ecological niches → eventual speciation.

---

## Tests

- `phenotypic_distance_identical_is_zero`
- `phenotypic_distance_max_divergence_is_two`
- `frequency_compatibility_same_freq_is_one`
- `frequency_compatibility_far_apart_is_zero`
- `reproductive_viability_blocks_distant_phenotypes`
- `genetic_drift_smaller_in_large_populations`
- `niche_specialization_drifts_toward_local_frequency`
