# BS-7: Preset RON para Estrategias + Validación de Keys

**Objetivo:** Hacer las estrategias de normalización configurables por RON (data-driven) y cerrar el gap de validación de keys desconocidas en hot reload.

**Estado:** PENDIENTE
**Esfuerzo:** S (~80 LOC)
**Bloqueado por:** BS-6 (NormPipeline en config)
**Desbloquea:** Percepciones metafísicas como data, no código

---

## Problema 1: Estrategias no configurables por RON

Hoy `Rigidity` y `CachePolicy` son configurables por RON. `NormPipeline` (nuevo) no tiene path de deserialización.

### Solución

Añadir `NormPipeline` como campo serializable en `BridgeConfigPartialRon`:

```rust
// bridge/presets/mod.rs
#[derive(Deserialize, Default)]
pub struct BridgeConfigPartialRon {
    pub rigidity: Option<RigidityPreset>,
    pub preset: Option<RigidityPreset>,
    pub bands: Option<Vec<BandDef>>,
    pub hysteresis_margin: Option<f32>,
    pub cache_capacity: Option<usize>,
    pub policy: Option<CachePolicy>,
    pub enabled: Option<bool>,
    pub norm_pipeline: Option<NormPipelineRon>,  // ← NUEVO
}
```

RON format:

```ron
// assets/bridge_config.ron
{
    "density": (
        rigidity: "moderate",
        norm_pipeline: (stages: ["concentration"]),
    ),
    "interference": (
        rigidity: "flexible",
        norm_pipeline: (stages: ["frequency_aligned", "concentration"]),
    ),
    "basal_drain": (
        rigidity: "rigid",
        norm_pipeline: (stages: ["temporal_window", "concentration"]),
    ),
}
```

### NormPipelineRon helper

```rust
/// Formato RON para NormPipeline — convierte a struct Copy en build_config_from_partial.
#[derive(Clone, Debug, Deserialize)]
pub struct NormPipelineRon {
    pub stages: Vec<NormStrategy>,  // Vec solo en deserialización; se copia a [_;4]
}

impl NormPipelineRon {
    pub fn to_pipeline(&self) -> NormPipeline {
        let mut p = NormPipeline::passthrough();
        for (i, &s) in self.stages.iter().take(MAX_NORM_STAGES).enumerate() {
            p = p.then(s);
        }
        p
    }
}
```

---

## Problema 2: Keys desconocidas en RON silenciosas

### Problema actual

`apply_bridge_config_for<B>()` en `presets/mod.rs:352-378`:
- Si RON tiene `"densiry"` (typo), se ignora silenciosamente
- Si RON tiene `"experimental_bridge"` (no registrado), se ignora
- Ningún warning para keys no consumidas

### Solución

Después de aplicar todas las bridge configs, verificar keys no consumidas:

```rust
fn validate_ron_keys(asset: &BridgeConfigAsset) {
    const KNOWN_KEYS: &[&str] = &[
        "density", "temperature", "phase_transition",
        "interference", "dissipation", "drag", "engine",
        "will", "catalysis", "collision_transfer", "osmosis",
        "competition_norm", "evolution_surrogate",
        // BS-4 bridges:
        "basal_drain", "senescence", "awakening",
        "radiation_pressure", "shape_opt", "epigenetic",
    ];
    for key in asset.bridges.keys() {
        if !KNOWN_KEYS.contains(&key.as_str()) {
            bevy::log::warn!(
                target: "bridge_config",
                key = key.as_str(),
                "unknown bridge key in RON — possible typo"
            );
        }
    }
}
```

Llamar en `apply_bridge_config_asset()` después del loop de apply.

---

## Problema 3: Backward compat RON (BS-1 → BS-6 migration)

Si un RON file tiene `norm_strategy: "concentration"` (formato BS-1) en vez de `norm_pipeline`:

```rust
#[derive(Deserialize, Default)]
pub struct BridgeConfigPartialRon {
    // ...
    #[serde(alias = "norm_strategy")]
    pub norm_pipeline: Option<NormPipelineOrStrategy>,
}

#[derive(Deserialize)]
#[serde(untagged)]
pub enum NormPipelineOrStrategy {
    Pipeline(NormPipelineRon),
    Strategy(NormStrategy),  // legacy: single strategy → NormPipeline::single(s)
}

impl NormPipelineOrStrategy {
    pub fn to_pipeline(&self) -> NormPipeline {
        match self {
            Self::Pipeline(p) => p.to_pipeline(),
            Self::Strategy(s) => NormPipeline::single(*s),
        }
    }
}
```

---

## Tests

```rust
// Unit: RON deserialization
ron_partial_with_norm_pipeline_deserializes_correctly
ron_partial_with_norm_strategy_legacy_migrates_to_pipeline
ron_partial_without_norm_defaults_to_concentration
ron_pipeline_empty_stages_is_passthrough
ron_pipeline_four_stages_max_accepted
ron_pipeline_five_stages_truncated_to_four

// Integration: hot reload
hot_reload_applies_norm_pipeline_from_ron
hot_reload_unknown_key_logs_warning
hot_reload_typo_key_logs_warning

// Backward compat
legacy_norm_strategy_ron_produces_single_stage_pipeline
```

---

## Archivos tocados

| Archivo | Cambio |
|---------|--------|
| `src/bridge/strategy.rs` | + NormPipelineRon, + Serialize/Deserialize derives |
| `src/bridge/presets/mod.rs` | + NormPipelineOrStrategy, + validate_ron_keys |
| `assets/bridge_config.ron` | + norm_pipeline fields (si existe) |

---

## Checklist pre-merge

- [ ] RON con `norm_pipeline: (stages: [...])` deserializa correctamente
- [ ] RON con `norm_strategy: "concentration"` migra a pipeline single-stage
- [ ] RON sin norm field → default Concentration
- [ ] Keys desconocidas generan warn! en log
- [ ] `cargo test --lib` verde
- [ ] Hot reload preserva pipeline configurado
