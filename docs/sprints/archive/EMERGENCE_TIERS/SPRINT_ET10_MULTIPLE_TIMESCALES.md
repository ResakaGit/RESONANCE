# Sprint ET-10 — Multiple Timescales: Efecto Baldwin y Herencia Epigenética

**Módulo:** `src/layers/timescale.rs` (nuevo), `src/blueprint/equations/emergence/timescale.rs` (nuevo)
**Tipo:** Nueva capa + ecuaciones puras.
**Tier:** T3-1. **Onda:** 0.
**BridgeKind:** `TimescaleBridge` — cache Small(32), clave `(lod_band, timescale_tier)`.
**Estado:** ✅ Implementado (2026-03-25)

---

## Objetivo

Un rasgo puede cambiar en cuatro escalas temporales distintas con velocidades muy diferentes:

```
τ_aprendizaje << τ_epigenético << τ_cultural << τ_genético
τ_a ≈ 10² ticks   τ_e ≈ 10³ ticks   τ_c ≈ 10⁴ ticks   τ_g ≈ 10⁵ ticks
```

El efecto Baldwin: comportamientos aprendidos (τ_a) que aumentan fitness pueden, con el tiempo, volverse instintivos (τ_g) — sin Lamarck, sólo selección. La capa `TimescaleAdapter` integra los cuatro offsets para computar el fenotipo efectivo.

```
effective_trait = genetic_baseline + epigenetic_offset + cultural_offset + learned_offset
fixation_rate(learned) = fitness_delta × selection_pressure × (1 / τ_genetic)
```

---

## Responsabilidades

### ET-10A: Ecuaciones puras

```rust
// src/blueprint/equations/emergence/timescale.rs

/// Fenotipo efectivo integrando los cuatro timescales.
pub fn effective_trait(
    genetic_baseline: f32,
    epigenetic_offset: f32,
    cultural_offset: f32,
    learned_offset: f32,
) -> f32 {
    genetic_baseline + epigenetic_offset + cultural_offset + learned_offset
}

/// Tasa de fijación genética del efecto Baldwin:
/// comportamiento aprendido reduce la necesidad de aprender → presión para que sea innato.
/// fitness_delta: mejora de qe/tick del comportamiento.
/// selection_pressure: fuerza de selección (env_variance / mean_qe).
/// genetic_timescale: ticks por generación.
pub fn baldwin_fixation_rate(
    fitness_delta: f32,
    selection_pressure: f32,
    genetic_timescale: u32,
) -> f32 {
    if genetic_timescale == 0 { return 0.0; }
    fitness_delta * selection_pressure / genetic_timescale as f32
}

/// Peso relativo de cada timescale según la varianza del entorno.
/// Alta varianza → más peso en aprendizaje (respuesta rápida).
/// Baja varianza → más peso en genética (respuesta eficiente).
pub fn timescale_weight(env_variance: f32, timescale_tau: f32) -> f32 {
    let responsiveness = 1.0 / (timescale_tau + 1.0);
    (responsiveness * env_variance).clamp(0.0, 1.0)
}

/// Plasticidad fenotípica: capacidad de responder a cambios en la escala τ.
/// Cuánto puede cambiar el fenotipo desde el baseline genético.
pub fn phenotypic_plasticity(
    max_plastic_range: f32,
    developmental_cost: f32,
    env_predictability: f32,
) -> f32 {
    // Alta predictabilidad → menor necesidad de plasticidad (costosa)
    let need = 1.0 - env_predictability;
    (max_plastic_range * need - developmental_cost).max(0.0)
}

/// Transferencia de offset entre timescales (cómo el aprendido se vuelve cultural).
pub fn timescale_transfer_rate(
    offset_source: f32,
    transfer_coefficient: f32,
    population_density: f32,
) -> f32 {
    offset_source * transfer_coefficient * population_density.sqrt()
}
```

### ET-10B: Componente

```rust
// src/layers/timescale.rs

/// Capa T3-1: TimescaleAdapter — integra cuatro velocidades de cambio fenotípico.
#[derive(Component, Reflect, Debug, Clone)]
#[reflect(Component)]
pub struct TimescaleAdapter {
    pub genetic_baseline:  f32,   // base estable — cambia en τ_g (muy lento)
    pub epigenetic_offset: f32,   // ajuste por entorno — τ_e (ET-6 escribe esto)
    pub cultural_offset:   f32,   // ajuste por imitación — τ_c (ET-3 escribe esto)
    pub learned_offset:    f32,   // ajuste por experiencia — τ_a (ET-1 escribe esto)
}

impl TimescaleAdapter {
    /// Fenotipo efectivo total.
    pub fn effective(&self) -> f32 {
        self.genetic_baseline + self.epigenetic_offset + self.cultural_offset + self.learned_offset
    }
    /// Offsets totales sobre el baseline.
    pub fn total_plasticity(&self) -> f32 {
        self.epigenetic_offset + self.cultural_offset + self.learned_offset
    }
}
```

### ET-10C: Sistema

```rust
/// Aplica efecto Baldwin: convierte learned_offset exitoso en genetic_baseline.
/// Phase::MorphologicalLayer — last in tier T3, after all T2 systems.
pub fn baldwin_fixation_system(
    mut adapters: Query<(&mut TimescaleAdapter, &BaseEnergy)>,
    clock: Res<SimulationClock>,
    config: Res<TimescaleConfig>,
) {
    // Baldwin opera en escala geológica — evaluar cada N ticks
    if clock.tick_id % config.genetic_eval_interval as u64 != 0 { return; }

    for (mut adapter, energy) in &mut adapters {
        let fitness_delta = adapter.learned_offset;
        if fitness_delta <= 0.0 { continue; }  // sólo fijan offsets positivos

        let selection_pressure = energy.qe() / (config.mean_qe + f32::EPSILON);
        let fixation = timescale_eq::baldwin_fixation_rate(
            fitness_delta, selection_pressure, config.genetic_timescale,
        );

        // Gradual: mueve una fracción del learned_offset al genetic_baseline
        let transfer = (fixation * fitness_delta).min(adapter.learned_offset);
        if transfer > f32::EPSILON {
            let new_base   = adapter.genetic_baseline + transfer;
            let new_learn  = adapter.learned_offset   - transfer;
            if adapter.genetic_baseline != new_base   { adapter.genetic_baseline  = new_base;  }
            if adapter.learned_offset   != new_learn  { adapter.learned_offset    = new_learn; }
        }
    }
}

/// Copia cultural_offset de modelos a imitadores (bridge entre ET-3 y TimescaleAdapter).
/// Phase::Input — after cultural_transmission_system.
pub fn cultural_timescale_sync_system(
    mut adapters: Query<(&mut TimescaleAdapter, &CulturalMemory)>,
    config: Res<TimescaleConfig>,
) {
    for (mut adapter, culture) in &mut adapters {
        if culture.meme_count == 0 { continue; }
        // Mejor meme disponible ajusta cultural_offset
        let best_fitness = culture.memes[..culture.meme_count as usize]
            .iter()
            .map(|m| m.estimated_fitness)
            .fold(f32::MIN, f32::max);
        let target_offset = best_fitness * config.cultural_to_trait_factor;
        if (adapter.cultural_offset - target_offset).abs() > f32::EPSILON {
            adapter.cultural_offset = adapter.cultural_offset
                + (target_offset - adapter.cultural_offset) * config.cultural_convergence_rate;
        }
    }
}
```

### ET-10D: Constantes

```rust
pub struct TimescaleBridge;
impl BridgeKind for TimescaleBridge {}

pub const TIMESCALE_GENETIC_EVAL_INTERVAL:    u64 = 1000;   // cada 1000 ticks (generacional)
pub const TIMESCALE_GENETIC_TIMESCALE:        u32 = 50_000; // ticks por generación
pub const TIMESCALE_CULTURAL_CONVERGENCE_RATE: f32 = 0.01;  // convergencia lenta
pub const TIMESCALE_CULTURAL_TO_TRAIT_FACTOR: f32 = 0.1;    // escala fitness → offset
pub const TIMESCALE_MEAN_QE:                   f32 = 500.0;
```

---

## Tacticas

- **Cuatro campos = cuatro velocidades, sin complejidad extra.** `TimescaleAdapter` tiene exactamente 4 campos (Hard Constraint 2). Cada campo es escrito por un sistema diferente (ET-1, ET-3, ET-6, ET-10) — bajo acoplamiento por diseño.
- **Baldwin es el único sistema que escribe `genetic_baseline`.** La "herencia" emerge de la acumulación de transfers en el baseline. Sin genes explícitos, sin genomas, sin crossover.
- **Evaluación generacional.** `TIMESCALE_GENETIC_EVAL_INTERVAL = 1000` ticks. El efecto Baldwin no ocurre en tiempo real — es una presión lenta que acumula. CPU cost ≈ 0.
- **Cache en puerta de entrada.** `TimescaleBridge` cachea `effective_trait()` por banda de LOD — cuando ET-13 GeologicalLOD activa compresión temporal, el sistema lee de caché en lugar de recalcular por cada entidad.

---

## NO hace

- No implementa crossover genético — los genes son un único `f32` baseline, no un cromosoma.
- No modela mutación — la varianza viene del entorno y el aprendizaje, no del genoma.
- No sincroniza entre entidades — el Baldwin es individual. La selección actúa sobre poblaciones via tasas de supervivencia (ET-7 Senescence).

---

## Dependencias

- ET-1 `AssociativeMemory` — escribe `learned_offset` via `record_memory_outcome`.
- ET-3 `CulturalMemory` — escribe `cultural_offset` via `cultural_timescale_sync_system`.
- ET-6 `EpigeneticState` — escribe `epigenetic_offset` via `epigenetic_expression_system`.
- ET-7 `SenescenceProfile` — las entidades que más vivieron acumulan más baldwin fixation.

---

## Criterios de Aceptación

- `effective_trait(10.0, 1.0, 0.5, 0.3)` → `11.8`.
- `effective_trait(10.0, 0.0, 0.0, 0.0)` → `10.0` (sin plasticidad).
- `baldwin_fixation_rate(2.0, 0.5, 10000)` → `0.0001`.
- `timescale_weight(1.0, 100.0)` → `≈ 0.0099` (alta tau → poco peso).
- `timescale_weight(1.0, 1.0)` → `0.5` (tau baja → más peso).
- Test: entidad con learned_offset positivo → genetic_baseline aumenta gradualmente cada EVAL_INTERVAL.
- Test: entidad con learned_offset negativo → genetic_baseline sin cambio.
- Test: cultural_timescale_sync → cultural_offset converge hacia best_meme_fitness × factor.
- `cargo test --lib` sin regresión.

---

## Referencias

- ET-1 Associative Memory — fuente de learned_offset
- ET-3 Cultural Transmission — fuente de cultural_offset
- ET-6 Epigenetic Expression — fuente de epigenetic_offset
- ET-7 Programmed Senescence — supervivencia selecciona sobre genetic_baseline
- Blueprint §T3-1: "Multiple Timescales", Baldwin Effect
