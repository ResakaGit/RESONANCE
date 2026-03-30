# BS-1: NormStrategy Enum + Desacople Normalización ↔ Atención

**Objetivo:** Separar el "cómo normalizar" del trait `Bridgeable`. La normalización pasa a ser un enum configurable por `BridgeConfig`, no código hardcodeado por impl.

**Estado:** PENDIENTE
**Esfuerzo:** M (~200 LOC)
**Bloqueado por:** —
**Desbloquea:** BS-4 (bridges nuevos), BS-6 (HOF composition)

---

## Problema

```rust
// HOY: normalize está fundido en cada impl de Bridgeable
impl Bridgeable for InterferenceBridge {
    fn normalize(input, config, band_hint) -> Self::Input {
        // hardcodeado: normalize_scalar + quantize_phase_sector + quantize_time_window
        // no se puede cambiar sin recompilar
    }
}
```

Cada bridge decide su normalización en compile-time. Para cambiarla:
1. Reescribir el impl completo
2. Duplicar el bridge (N bridges × M estrategias = N×M tipos)
3. Recompilar + re-testear todo

---

## Solución

### Paso 1: Crear `bridge/strategy.rs`

```rust
//! Estrategias de normalización desacopladas del trait Bridgeable.
//! Cada estrategia define cómo cuantizar inputs antes del cache lookup.

/// Estrategia de normalización — enum cerrado, exhaustive match, Copy, RON-serializable.
/// Cada variante mapea a una función pura de normalización en `apply_norm_strategy_*`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NormStrategy {
    /// Normalización por bandas escalares + histéresis.
    /// Comportamiento actual de todos los bridges. Default universal.
    #[default]
    Concentration,

    /// Passthrough: sin normalización, cache por bits exactos del input.
    /// Hit rate bajo pero precisión perfecta. Útil para debug/verificación.
    Passthrough,

    /// Alineación por frecuencia elemental (Almanac canonical lookup).
    /// Cada input de frecuencia se snapea al canonical del ElementDef más cercano.
    /// Fallback a Concentration si no hay match en Almanac.
    FrequencyAligned,

    /// Ventana temporal: agrupa inputs por ventana de tiempo (floor(t/window)*window).
    /// Reduce cardinality temporal. Combinar con Concentration para scalar+time.
    TemporalWindow,
}
```

### Paso 2: Funciones puras por estrategia

```rust
/// Normaliza un escalar según la estrategia configurada.
/// Cuantiza inputs al espacio canónico de la estrategia.
#[inline]
pub fn apply_norm_scalar(
    strategy: NormStrategy,
    value: f32,
    bands: &[BandDef],
    hysteresis: f32,
    band_hint: Option<usize>,
) -> (f32, usize) {
    match strategy {
        NormStrategy::Concentration => normalize_scalar(value, bands, hysteresis, band_hint),
        NormStrategy::Passthrough   => (value, 0),
        NormStrategy::FrequencyAligned => normalize_scalar(value, bands, hysteresis, band_hint),
        NormStrategy::TemporalWindow   => normalize_scalar(value, bands, hysteresis, band_hint),
    }
}

/// Normaliza un timestamp según la estrategia.
#[inline]
pub fn apply_norm_time(
    strategy: NormStrategy,
    t: f32,
    window_s: f32,
) -> f32 {
    match strategy {
        NormStrategy::TemporalWindow => quantize_time_window(t, window_s),
        NormStrategy::Passthrough    => t,
        _                            => quantize_time_window(t, window_s),
    }
}
```

### Paso 3: Añadir campo a `BridgeConfig`

```rust
// bridge/config.rs
pub struct BridgeConfig<B: BridgeKind> {
    pub bands: Vec<BandDef>,
    pub hysteresis_margin: f32,
    pub cache_capacity: usize,
    pub policy: CachePolicy,
    pub enabled: bool,
    pub rigidity: Rigidity,
    pub norm_strategy: NormStrategy,  // ← NUEVO
    #[serde(skip)]
    pub _marker: PhantomData<B>,
}
```

**Default:** `NormStrategy::Concentration` — comportamiento actual bit-identical.

### Paso 4: Refactorear macro `impl_bridgeable_scalar!`

```rust
#[macro_export]
macro_rules! impl_bridgeable_scalar {
    ($bridge:ty, |$input:ident| $compute_body:expr) => {
        impl $crate::bridge::decorator::Bridgeable for $bridge {
            type Input = f32;
            type Output = f32;

            #[inline]
            fn normalize(
                input: Self::Input,
                config: &$crate::bridge::config::BridgeConfig<Self>,
                band_hint: Option<usize>,
            ) -> Self::Input {
                // ← DISPATCH POR ESTRATEGIA en vez de hardcode normalize_scalar
                $crate::bridge::strategy::apply_norm_scalar(
                    config.norm_strategy,
                    input,
                    &config.bands,
                    config.hysteresis_margin,
                    band_hint,
                ).0
            }
            // ... cache_key, compute, into_cached, from_cached sin cambios
        }
    };
}
```

### Paso 5: Migrar impls custom (InterferenceBridge, etc.)

Cada `fn normalize()` custom pasa a usar `apply_norm_*`:

```rust
// ANTES (ops.rs)
fn normalize(input: Self::Input, config: &BridgeConfig<Self>, _hint: Option<usize>) -> Self::Input {
    let f1 = normalize_scalar(input.f1, &config.bands, h, None).0;
    let phase1 = quantize_phase_sector(input.phase1, INTERFERENCE_PHASE_SECTORS);
    let t = quantize_time_window(input.t, tw);
    // ...
}

// DESPUÉS
fn normalize(input: Self::Input, config: &BridgeConfig<Self>, _hint: Option<usize>) -> Self::Input {
    let f1 = apply_norm_scalar(config.norm_strategy, input.f1, &config.bands, h, None).0;
    let phase1 = quantize_phase_sector(input.phase1, INTERFERENCE_PHASE_SECTORS);
    let t = apply_norm_time(config.norm_strategy, input.t, tw);
    // ...
}
```

**Impacto:** Con `Concentration` (default), la salida es **bit-identical** al código actual.

---

## Archivos tocados

| Archivo | Cambio |
|---------|--------|
| `src/bridge/strategy.rs` | **NUEVO** — NormStrategy enum + apply_norm_* |
| `src/bridge/config.rs` | + campo `norm_strategy: NormStrategy` |
| `src/bridge/mod.rs` | + `pub mod strategy` + re-exports |
| `src/bridge/macros.rs` | dispatch por estrategia |
| `src/bridge/impls/physics.rs` | migrar normalize |
| `src/bridge/impls/ops.rs` | migrar normalize (5 bridges) |
| `src/bridge/presets/mod.rs` | `norm_strategy` en presets |
| `src/bridge/presets/{physics,combat,ecosystem}.rs` | default `Concentration` |

---

## Tests (TDD — escribir ANTES de implementar)

### Unit tests en `bridge/strategy.rs`

```
apply_norm_scalar_concentration_matches_normalize_scalar
apply_norm_scalar_passthrough_returns_input_unchanged
apply_norm_scalar_frequency_aligned_fallback_to_concentration
apply_norm_time_temporal_window_quantizes_correctly
apply_norm_time_passthrough_returns_t_unchanged
apply_norm_time_concentration_still_quantizes_time
```

### Regression tests en `bridge/decorator.rs`

```
bridge_compute_with_concentration_matches_old_behavior
bridge_compute_with_passthrough_exact_inputs
bridge_compute_with_passthrough_different_inputs_miss
bridge_warmup_record_respects_norm_strategy
```

### Integration en `bridge/impls/physics.rs`

```
bridged_density_concentration_matches_before_refactor
bridged_density_passthrough_matches_equations_exact
bridged_temperature_concentration_matches_before_refactor
```

### Integration en `bridge/impls/ops.rs`

```
interference_concentration_matches_before_refactor
interference_passthrough_exact_values
osmosis_concentration_matches_before_refactor
catalysis_concentration_matches_before_refactor
```

---

## Invariantes

1. **Bit-identical default:** `NormStrategy::Concentration` produce exactamente la misma salida que el código pre-refactor. Test de regresión 1:1 con inputs aleatorios (1000 samples).
2. **Passthrough precision:** `NormStrategy::Passthrough` produce salida === `B::compute(raw_input)`. Zero normalización.
3. **Exhaustive match:** Agregar variante al enum rompe compilación en TODOS los dispatch points. Zero wildcard `_`.
4. **Serde round-trip:** `NormStrategy` serializa/deserializa desde RON sin pérdida.
5. **Config backward-compat:** RON files sin `norm_strategy` deserializan con default `Concentration`.

---

## Checklist pre-merge

- [ ] `cargo test --lib` verde
- [ ] `cargo test --test '*'` verde
- [ ] `NormStrategy::Concentration` bit-identical (1000-sample regression)
- [ ] `NormStrategy::Passthrough` exact (1000-sample regression)
- [ ] Campo `norm_strategy` en `BridgeConfig` con `#[serde(default)]`
- [ ] Sin `_` wildcard en match de `NormStrategy`
- [ ] Doc comments bilingües en `strategy.rs`
- [ ] Re-export en `bridge/mod.rs`
