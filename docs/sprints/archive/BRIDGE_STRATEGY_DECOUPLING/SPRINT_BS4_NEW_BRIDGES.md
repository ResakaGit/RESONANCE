# BS-4: Bridges Nuevos — 6 Sistemas sin Cache

**Objetivo:** Extender el Bridge Optimizer a los 6 sistemas que computan cada tick sin cache. Todas las ecuaciones son puras, estables, y altamente cacheables.

**Estado:** PENDIENTE
**Esfuerzo:** L (~350 LOC)
**Bloqueado por:** BS-1 (NormStrategy enum — los nuevos bridges nacen con estrategia configurable)
**Desbloquea:** BS-5 (tests)

---

## Los 6 candidatos

| # | Sistema | Ecuación | Inputs | Estabilidad | Hit rate esperado |
|---|---------|----------|--------|-------------|-------------------|
| 1 | `basal_drain_system` | `radius^0.75 × age_factor` (Kleiber) | radius, age | Alta (radius cambia lento) | 90% |
| 2 | `senescence_death_system` | Gompertz hazard: `coeff × e^(coeff × age)` | age, coeff | Alta (coeff constante) | 95% |
| 3 | `awakening_system` | `coherence > threshold` | coherence, qe | Media (coherence fluctúa) | 75% |
| 4 | `radiation_pressure_system` | frequency alignment + transfer | freq, qe, distance | Media-alta | 80% |
| 5 | `shape_optimization_system` | `bounded_fineness_descent` | fineness, drag, cost | Alta (converge rápido) | 90% |
| 6 | `epigenetic_adaptation_system` | `env × expression_mask` | env[4], mask[4] | Alta (env cambia lento) | 85% |

**Impacto estimado:** ~1500 cómputos/tick eliminados en mundo de 512 entidades.

---

## Bridge 1: BasalDrainBridge

### Ecuación

```
drain_rate = BASAL_DRAIN_RATE × radius^KLEIBER × age_dependent_factor(age, coeff)
```

Kleiber (0.75) es constante universal. `age_dependent_factor` es monotónica creciente. Radius cambia solo con growth (lento).

### Implementación

```rust
// bridge/config.rs
pub struct BasalDrainBridge;
impl BridgeKind for BasalDrainBridge {}

// bridge/impls/metabolic.rs (NUEVO)
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BasalDrainInput {
    pub radius: f32,
    pub age_ticks: u64,
    pub senescence_coeff: f32,
}

impl Bridgeable for BasalDrainBridge {
    type Input = BasalDrainInput;
    type Output = f32;

    fn normalize(input: Self::Input, config: &BridgeConfig<Self>, hint: Option<usize>) -> Self::Input {
        Self::Input {
            radius: apply_norm_scalar(config.norm_strategy, input.radius, &config.bands, config.hysteresis_margin, hint).0,
            age_ticks: (input.age_ticks / 100) * 100,  // quantize age to 100-tick windows
            senescence_coeff: input.senescence_coeff,    // constant per entity, no normalize
        }
    }

    fn cache_key(n: Self::Input) -> u64 {
        hash_inputs(&[f32::to_bits(n.radius) as u64, n.age_ticks, f32::to_bits(n.senescence_coeff) as u64])
    }

    fn compute(n: Self::Input) -> Self::Output {
        equations::basal_drain_rate(n.radius, n.age_ticks, n.senescence_coeff)
    }

    impl_bridgeable_scalar_io!();
}
```

### Bandas default (Concentration)

```rust
// presets/metabolic.rs (NUEVO)
const BASAL_DRAIN_MOD: &[BandDef] = &[
    BandDef { min: 0.0,  max: 0.5,  canonical: 0.25, stable: true  },
    BandDef { min: 0.5,  max: 2.0,  canonical: 1.0,  stable: true  },
    BandDef { min: 2.0,  max: 5.0,  canonical: 3.5,  stable: true  },
    BandDef { min: 5.0,  max: 20.0, canonical: 10.0, stable: false },
    BandDef { min: 20.0, max: 100.0,canonical: 50.0, stable: false },
];
```

---

## Bridge 2: SenescenceBridge

### Ecuación

```
hazard = coeff × e^(coeff × age_years)
survival_probability = e^(-hazard)
die if survival_probability < threshold
```

`coeff` es constante por entity type (materializado, flora, fauna). `age_years` incrementa monótonamente. Altamente cacheable — misma edad ± ventana = misma probabilidad.

### Implementación

```rust
pub struct SenescenceHazardBridge;
impl BridgeKind for SenescenceHazardBridge {}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SenescenceInput {
    pub age_ticks: u64,
    pub coeff: f32,
}

// Normalización: age quantizada a ventanas de 50 ticks, coeff exacto (constante per-type)
// Hit rate: ~95% — solo cambia al cruzar ventana de edad
```

---

## Bridge 3: AwakeningBridge

### Ecuación

```
potential = (coherence - dissipation) / (coherence - dissipation + qe)
awaken if potential > threshold
```

Coherence fluctúa más que las anteriores pero el threshold es constante (axiom-derived). Cache por bandas de coherence + qe.

### Implementación

```rust
pub struct AwakeningBridge;
impl BridgeKind for AwakeningBridge {}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct AwakeningInput {
    pub coherence: f32,
    pub dissipation: f32,
    pub qe: f32,
}
// Normalización: 3 scalares por bandas independientes
// Hit rate: ~75% — coherence más volátil
```

---

## Bridge 4: RadiationPressureBridge

### Ecuación

```
alignment = gaussian_frequency_alignment(f_center, f_cell, bandwidth)
transfer = qe × alignment × pressure_rate × attenuation(distance)
```

Grid processing (per-cell). Alto volumen de llamadas. Frequency alignment es el cuello de botella — `exp()` call.

### Implementación

```rust
pub struct RadiationPressureAlignmentBridge;
impl BridgeKind for RadiationPressureAlignmentBridge {}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RadPressureInput {
    pub f_center: f32,
    pub f_cell: f32,
    pub bandwidth: f32,
}
// Solo cachea la parte cara (alignment = exp(-x²))
// Transfer se computa inline (multiplicación trivial)
// Hit rate: ~80% — frecuencias se repiten por zona
```

---

## Bridge 5: ShapeOptimizationBridge

### Ecuación

```
fineness_next = bounded_fineness_descent(fineness, drag, cost, dt)
```

Converge en ~10-20 ticks. Después, fineness estable → cache hit perpetuo.

### Implementación

```rust
pub struct ShapeOptBridge;
impl BridgeKind for ShapeOptBridge {}

// Input: (fineness: f32, drag: f32, cost: f32)
// Normalización: 3 bandas escalares
// Hit rate: ~90% — converge rápido y se queda
```

---

## Bridge 6: EpigeneticBridge

### Ecuación

```
expression[dim] = lerp(current, target, rate × dt)
target = f(env_context[dim])
```

`env_context` cambia lento (ambiente). `rate` es constante. Alta repetición.

### Implementación

```rust
pub struct EpigeneticBridge;
impl BridgeKind for EpigeneticBridge {}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct EpigeneticInput {
    pub current: f32,
    pub target: f32,
    pub rate: f32,
}
// Normalización: current y target por bandas, rate exacto
// Hit rate: ~85% — env cambia lento
```

---

## Archivos nuevos/tocados

| Archivo | Cambio |
|---------|--------|
| `src/bridge/config.rs` | + 6 marker structs |
| `src/bridge/impls/metabolic.rs` | **NUEVO** — BasalDrain, Senescence, Awakening, Epigenetic |
| `src/bridge/impls/worldgen.rs` | **NUEVO** — RadiationPressureAlignment |
| `src/bridge/impls/morphological.rs` | **NUEVO** — ShapeOpt |
| `src/bridge/impls/mod.rs` | + pub mod metabolic, worldgen, morphological |
| `src/bridge/presets/metabolic.rs` | **NUEVO** — bandas para 4 bridges |
| `src/bridge/presets/worldgen.rs` | **NUEVO** — bandas para RadPressure |
| `src/bridge/presets/morphological.rs` | **NUEVO** — bandas para ShapeOpt |
| `src/bridge/presets/mod.rs` | + impl_bridge_defaults, + register loops |
| `src/bridge/context_fill.rs` | + 6 bridges en macros |
| `src/bridge/metrics.rs` | + 6 bridges en macros |
| `src/simulation/metabolic/basal_drain.rs` | + bridge_compute call |
| `src/simulation/metabolic/senescence_death.rs` | + bridge_compute call |
| `src/simulation/awakening.rs` | + bridge_compute call |
| `src/worldgen/systems/radiation_pressure.rs` | + bridge_compute call |
| `src/simulation/lifecycle/shape_optimization.rs` | + bridge_compute call |
| `src/simulation/emergence/epigenetic_adaptation.rs` | + bridge_compute call |

---

## Tests (TDD)

### Por cada bridge nuevo (6×5 = 30 tests mínimo):

```
{bridge}_concentration_matches_exact_equation
{bridge}_passthrough_matches_exact_bitwise
{bridge}_cache_hit_on_same_band
{bridge}_cache_miss_on_different_band
{bridge}_disabled_bypasses_completely
```

### Integration (sistema con bridge):

```
basal_drain_system_with_bridge_reduces_computation_count
senescence_system_with_bridge_same_death_tick
awakening_system_with_bridge_same_awakening_threshold
radiation_pressure_with_bridge_conserves_energy
shape_opt_with_bridge_converges_same_fineness
epigenetic_with_bridge_same_expression_trajectory
```

---

## Invariantes

1. **Conservation:** Bridges metabólicos NUNCA alteran la suma total de energía drenada. Regresión test: `Σ drain(bridged) == Σ drain(exact) ± ε_banda`.
2. **Determinism:** Mismos inputs → mismos outputs. Bridges nuevos con `NormStrategy::Passthrough` = bit-identical.
3. **Phase lifecycle:** Los 6 bridges nuevos participan en Warmup→Filling→Active.
4. **Axiom compliance:** Cache de senescence no viola Axiom 4 (dissipation siempre ocurre). Cache es optimización, no bypass.

---

## Checklist pre-merge

- [ ] 6 marker structs en config.rs
- [ ] 6 Bridgeable impls con norm_strategy dispatch
- [ ] 6 preset definitions con bandas calibradas
- [ ] 6 bridges en context_fill macros (scan, each, clear)
- [ ] 6 bridges en metrics macros (collect, summary)
- [ ] 30+ unit tests (5 per bridge)
- [ ] 6 integration tests (1 per sistema)
- [ ] `cargo test --lib` verde
- [ ] `cargo bench --bench batch_benchmark` sin regresión >5%
