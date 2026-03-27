# Sprint ET-6 — Epigenetic Expression: Fenotipo Modular por Ambiente

**Módulo:** `src/layers/epigenetics.rs` (nuevo), `src/blueprint/equations/emergence/epigenetics.rs` (nuevo)
**Tipo:** Nueva capa + ecuaciones puras.
**Tier:** T2-2. **Onda:** A.
**BridgeKind:** `EpigeneticBridge` — cache Small(64), clave `(env_band, gene_mask_hash)`.
**Estado:** ✅ Implementado (2026-03-25)

---

## Objetivo

El mismo `InferenceProfile` (genotipo) produce fenotipos distintos según las condiciones energéticas del entorno. En entornos de baja energía, se silencian genes costosos (órganos que no se expresan). La adaptación es reversible dentro de una vida — más rápida que la evolución genética.

```
express(gene_i) iff E[benefit(gene_i) | environment] > expression_cost_i
phenotype = InferenceProfile × expression_mask (element-wise)
```

---

## Responsabilidades

### ET-6A: Ecuaciones

```rust
// src/blueprint/equations/emergence/epigenetics.rs

/// ¿Debe expresarse este gen dado el entorno energético?
pub fn should_express_gene(
    gene_benefit: f32,        // E[qe/tick] que aporta este gen expresado
    expression_cost: f32,     // qe/tick para mantenerlo expresado
    env_energy_ratio: f32,    // field_qe / mean_field_qe — qué tan rico es el entorno
) -> bool {
    gene_benefit * env_energy_ratio > expression_cost
}

/// Fenotipo efectivo: producto del genotipo base por la máscara de expresión.
/// genotype_val: valor del gen en InferenceProfile.
/// expression_level: [0,1] — 0 = silenciado, 1 = expresión completa.
pub fn effective_phenotype(genotype_val: f32, expression_level: f32) -> f32 {
    genotype_val * expression_level.clamp(0.0, 1.0)
}

/// Costo de silenciar un gen (reconfiguración metabólica).
pub fn silencing_cost(gene_complexity: f32, silencing_rate: f32) -> f32 {
    gene_complexity * silencing_rate
}

/// Velocidad de respuesta epigenética: cuántos ticks para ajustarse al nuevo entorno.
pub fn epigenetic_lag(expression_current: f32, expression_target: f32, adaptation_speed: f32) -> f32 {
    expression_current + (expression_target - expression_current) * adaptation_speed
}
```

### ET-6B: Componente

```rust
// src/layers/epigenetics.rs

/// Capa T2-2: EpigeneticState — máscara de expresión sobre InferenceProfile.
#[derive(Component, Reflect, Debug, Clone)]
#[reflect(Component)]
pub struct EpigeneticState {
    pub expression_mask: [f32; 4],   // [0,1] por dimensión del InferenceProfile
    pub adaptation_speed: f32,        // velocidad de cambio de expresión
    pub silencing_cost:   f32,        // qe por silenciamiento activo
    pub env_sample_rate:  u8,         // cada cuántos ticks re-samplea el entorno
}
```

### ET-6C: Sistema

```rust
/// Ajusta la máscara de expresión epigenética según el entorno de energía local.
/// Phase::MorphologicalLayer — modifica fenotipos antes de visual_contract_sync.
pub fn epigenetic_expression_system(
    mut agents: Query<(&Transform, &mut EpigeneticState, &mut InferenceProfile, &mut BaseEnergy)>,
    field: Res<EnergyFieldGrid>,
    config: Res<EpigeneticConfig>,
    clock: Res<SimulationClock>,
) {
    for (transform, mut epi, mut profile, mut energy) in &mut agents {
        if clock.tick_id % epi.env_sample_rate as u64 != 0 { continue; }

        let cell_qe = field.cell_qe_at_world(transform.translation.x, transform.translation.z);
        let env_ratio = (cell_qe / config.mean_field_qe).clamp(0.0, 2.0);

        let mut total_cost = 0.0f32;

        // Ajustar cada dimensión de expresión
        for i in 0..4 {
            let benefit = profile.gene_benefit(i);
            let cost = profile.gene_cost(i);
            let target = if epigenetic_eq::should_express_gene(benefit, cost, env_ratio) {
                1.0f32
            } else {
                0.0f32
            };
            let new_level = epigenetic_eq::epigenetic_lag(
                epi.expression_mask[i], target, epi.adaptation_speed,
            );
            if (epi.expression_mask[i] - new_level).abs() > f32::EPSILON {
                total_cost += epigenetic_eq::silencing_cost(
                    profile.gene_complexity(i), epi.silencing_cost,
                );
                epi.expression_mask[i] = new_level;
                // Escribir fenotipo efectivo en InferenceProfile (sistema hace la escritura — Hard Block 14)
                let new_bias = epigenetic_eq::effective_phenotype(benefit, new_level);
                if (profile.gene_benefit(i) - new_bias).abs() > f32::EPSILON {
                    profile.set_bias(i, new_bias);
                }
            }
        }

        let new_qe = (energy.qe() - total_cost).max(0.0);
        if energy.qe() != new_qe { energy.set_qe(new_qe); }
    }
}
```

### ET-6D: Constantes

```rust
pub struct EpigeneticBridge;
impl BridgeKind for EpigeneticBridge {}

pub const EPIGENETIC_DEFAULT_ADAPTATION_SPEED: f32 = 0.05;  // lento — días en ticks
pub const EPIGENETIC_DEFAULT_SILENCING_COST: f32 = 0.5;
pub const EPIGENETIC_MEAN_FIELD_QE: f32 = 200.0;             // referencia de entorno "normal"
pub const EPIGENETIC_DEFAULT_SAMPLE_RATE: u8 = 16;           // re-samplea cada 16 ticks
```

---

## Tacticas

- **Throttle por `env_sample_rate`.** El entorno cambia lentamente. Re-samplear cada 16 ticks reduce carga 16×.
- **`expression_mask: [f32; 4]`** modula las 4 dimensiones principales de `InferenceProfile`. Array fijo, sin Vec.
- **`InferenceProfile::set_bias(i, val)` setter puro** — el sistema calcula `effective_phenotype(benefit, level)` y llama `set_bias`. Hard Block 14: sin cómputo en métodos de componente.
- **Plasticidad fenotípica emerge sin programarla.** Mismos genes → distintos fenotipos en distintos entornos.

---

## Criterios de Aceptación

- `should_express_gene(5.0, 1.0, 1.0)` → `true`. `should_express_gene(0.5, 1.0, 0.5)` → `false`.
- `effective_phenotype(10.0, 0.5)` → `5.0`. `effective_phenotype(10.0, 0.0)` → `0.0`.
- `epigenetic_lag(1.0, 0.0, 0.05)` → `0.95` (convergencia lenta).
- Test: entidad en entorno rico → expression_mask ≈ [1,1,1,1].
- Test: entidad en entorno pobre → genes costosos silenciados (expression_mask[i] → 0).
- Test: reactivación al volver a entorno rico → mask se recupera (reversible).
- `cargo test --lib` sin regresión.

---

## Referencias

- `src/layers/inference.rs::InferenceProfile` — genotipo base
- `src/worldgen/field_grid.rs::EnergyFieldGrid` — señal ambiental
- Blueprint §T2-2: "Epigenetic Expression"
