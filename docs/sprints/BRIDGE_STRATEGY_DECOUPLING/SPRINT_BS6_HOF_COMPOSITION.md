# BS-6: HOF Composition para Strategy Stacking

**Objetivo:** Permitir componer estrategias de normalización como funciones apiladas (Higher-Order Functions). Una entidad no tiene "una estrategia" — tiene un stack de transformaciones que se aplican en orden.

**Estado:** PENDIENTE
**Esfuerzo:** M (~150 LOC)
**Bloqueado por:** BS-1 (NormStrategy enum base)
**Desbloquea:** BS-7 (RON presets), percepciones metafísicas

---

## Problema

BS-1 introduce `NormStrategy` como enum plano. Funciona para estrategias atómicas, pero la visión metafísica requiere **composición**:

```
Apostante "Predador":   FrequencyAligned → Concentration(bandas_anchas)
Apostante "Planta":     TemporalWindow(lento) → Concentration(bandas_finas)
Apostante "Observador": Passthrough (máxima precisión, mínimo cache)
```

Cada apostante es un **pipeline de normalización** — un fold de funciones puras sobre el input.

---

## Solución: NormPipeline como array estático de stages

```rust
// bridge/strategy.rs (extender BS-1)

/// Máximo de stages en un pipeline de normalización.
/// 4 = suficiente para cualquier combinación razonable. Stack-allocated.
pub const MAX_NORM_STAGES: usize = 4;

/// Pipeline de normalización: array fijo de stages aplicados en orden.
/// Cada stage transforma el input antes del cache_key.
///
/// Pipeline vacío (len=0) = Passthrough.
/// Pipeline con 1 stage = equivalente a NormStrategy atómico.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct NormPipeline {
    stages: [NormStrategy; MAX_NORM_STAGES],
    len: u8,
}

impl Default for NormPipeline {
    fn default() -> Self {
        // Default = un solo stage Concentration (comportamiento actual)
        Self {
            stages: [NormStrategy::Concentration, NormStrategy::Passthrough,
                     NormStrategy::Passthrough, NormStrategy::Passthrough],
            len: 1,
        }
    }
}
```

### Builder fluido (composición funcional)

```rust
impl NormPipeline {
    /// Pipeline vacío — passthrough puro.
    pub const fn passthrough() -> Self {
        Self { stages: [NormStrategy::Passthrough; MAX_NORM_STAGES], len: 0 }
    }

    /// Pipeline con un solo stage (shortcut para estrategia atómica).
    pub const fn single(strategy: NormStrategy) -> Self {
        Self {
            stages: [strategy, NormStrategy::Passthrough,
                     NormStrategy::Passthrough, NormStrategy::Passthrough],
            len: 1,
        }
    }

    /// Apila un stage al final del pipeline. Capped a MAX_NORM_STAGES.
    pub const fn then(mut self, stage: NormStrategy) -> Self {
        if (self.len as usize) < MAX_NORM_STAGES {
            self.stages[self.len as usize] = stage;
            self.len += 1;
        }
        self
    }

    /// Aplica el pipeline completo a un escalar.
    /// fold(input, |acc, stage| apply_norm_scalar(stage, acc, ...))
    #[inline]
    pub fn apply_scalar(
        &self,
        value: f32,
        bands: &[BandDef],
        hysteresis: f32,
        hint: Option<usize>,
    ) -> (f32, usize) {
        if self.len == 0 {
            return (value, 0); // passthrough
        }
        let mut current = value;
        let mut band_idx = hint.unwrap_or(0);
        for i in 0..self.len as usize {
            let (next, idx) = apply_norm_scalar(self.stages[i], current, bands, hysteresis, Some(band_idx));
            current = next;
            band_idx = idx;
        }
        (current, band_idx)
    }

    /// Aplica el pipeline a un timestamp.
    #[inline]
    pub fn apply_time(&self, t: f32, window_s: f32) -> f32 {
        if self.len == 0 { return t; }
        let mut current = t;
        for i in 0..self.len as usize {
            current = apply_norm_time(self.stages[i], current, window_s);
        }
        current
    }
}
```

### Integración con BridgeConfig

```rust
// bridge/config.rs — reemplazar norm_strategy por pipeline
pub struct BridgeConfig<B: BridgeKind> {
    pub bands: Vec<BandDef>,
    pub hysteresis_margin: f32,
    pub cache_capacity: usize,
    pub policy: CachePolicy,
    pub enabled: bool,
    pub rigidity: Rigidity,
    pub norm_pipeline: NormPipeline,  // ← reemplaza norm_strategy
    pub _marker: PhantomData<B>,
}
```

### Presets predefinidos (apostantes)

```rust
// bridge/strategy.rs

impl NormPipeline {
    /// Predador: bandas anchas, rápido, impreciso.
    pub const PREDATOR: Self = Self::single(NormStrategy::Concentration);
    // Nota: bandas anchas se configuran en BridgeConfig.rigidity = Flexible

    /// Planta: ventana temporal larga + bandas finas.
    pub const FLORA: Self = NormPipeline::passthrough()
        .then(NormStrategy::TemporalWindow)
        .then(NormStrategy::Concentration);

    /// Observador: máxima precisión.
    pub const OBSERVER: Self = Self::passthrough();

    /// Default universal: Concentration puro (backward-compatible).
    pub const STANDARD: Self = Self::single(NormStrategy::Concentration);
}
```

---

## Propiedades del diseño

| Propiedad | Garantía |
|-----------|----------|
| **Stack-allocated** | `[NormStrategy; 4]` + `u8` = 5 bytes. Zero heap. |
| **Copy** | Pipeline es Copy — se puede pasar por valor sin costo. |
| **Composable** | `.then()` apila stages en compile-time (const fn). |
| **Idempotent default** | `NormPipeline::default()` = un stage Concentration = comportamiento actual. |
| **Bounded** | Max 4 stages. Overflow silencioso (cap). |
| **Deterministic** | Fold secuencial, mismo input → misma salida. |
| **Serializable** | RON: `norm_pipeline: { stages: ["concentration", "temporal_window"], len: 2 }` |

---

## Tests (TDD)

```rust
// Unit tests en bridge/strategy.rs

// Constructores
pipeline_default_is_single_concentration
pipeline_passthrough_len_zero
pipeline_single_has_len_one
pipeline_then_increments_len
pipeline_then_capped_at_max_stages

// apply_scalar
pipeline_passthrough_returns_input_unchanged
pipeline_single_concentration_matches_apply_norm_scalar
pipeline_two_stages_applies_in_order
pipeline_three_stages_fold_correct
pipeline_empty_stages_passthrough

// apply_time
pipeline_temporal_window_quantizes_time
pipeline_passthrough_time_unchanged
pipeline_concentration_then_temporal_composes

// Presets
pipeline_predator_single_concentration
pipeline_flora_two_stages
pipeline_observer_passthrough
pipeline_standard_eq_default

// Regression
pipeline_default_bitidentical_to_bs1_norm_strategy_concentration

// Serde
pipeline_ron_round_trip_preserves_stages
pipeline_missing_field_in_ron_defaults_to_standard
```

---

## Archivos tocados

| Archivo | Cambio |
|---------|--------|
| `src/bridge/strategy.rs` | + NormPipeline, + apply methods, + presets |
| `src/bridge/config.rs` | `norm_strategy` → `norm_pipeline` |
| `src/bridge/macros.rs` | `config.norm_strategy` → `config.norm_pipeline.apply_scalar(...)` |
| `src/bridge/impls/physics.rs` | pipeline dispatch |
| `src/bridge/impls/ops.rs` | pipeline dispatch |
| `src/bridge/impls/metabolic.rs` | pipeline dispatch |
| `src/bridge/presets/*.rs` | `norm_pipeline: NormPipeline::STANDARD` |

---

## Migración BS-1 → BS-6

| BS-1 | BS-6 |
|------|------|
| `config.norm_strategy` | `config.norm_pipeline` |
| `apply_norm_scalar(config.norm_strategy, ...)` | `config.norm_pipeline.apply_scalar(...)` |
| `NormStrategy::Concentration` | `NormPipeline::single(NormStrategy::Concentration)` |
| RON: `norm_strategy: "concentration"` | RON: `norm_pipeline: { stages: ["concentration"] }` |

**Backward compat:** Si RON tiene `norm_strategy` (BS-1 format), deserializar como `NormPipeline::single(strategy)`. Si tiene `norm_pipeline`, usar directamente. `#[serde(alias)]` o migration helper.

---

## Checklist pre-merge

- [ ] `NormPipeline` es Copy + 5 bytes
- [ ] `pipeline.apply_scalar()` fold correcto con 0-4 stages
- [ ] Default pipeline = Concentration (bit-identical)
- [ ] Presets PREDATOR/FLORA/OBSERVER compilados como const
- [ ] RON backward compat (norm_strategy → pipeline migration)
- [ ] 15+ tests verdes
- [ ] `cargo test --lib` verde
