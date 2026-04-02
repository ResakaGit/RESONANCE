# PV-4: Foo & Michor 2009 — Continuous vs Pulsed Therapy

**Objetivo:** Reproducir la predicción principal de Foo & Michor 2009: existe un dose rate óptimo que minimiza probabilidad de resistencia, y pulsed therapy puede superar a continuous cuando el fitness cost de resistencia es alto.

**Estado:** PENDIENTE
**Esfuerzo:** Medio
**Bloqueado por:** —

---

## Paper

- **Cita:** Foo J, Michor F. "Evolution of resistance to targeted anti-cancer therapies during continuous and pulsed administration strategies." *PLoS Computational Biology* 5(11):e1000557 (2009)
- **DOI:** 10.1371/journal.pcbi.1000557
- **Acceso:** Open access (PLOS)

## Datos cuantitativos

| Parámetro | Rango |
|-----------|-------|
| Mutation rate u | 10⁻⁵ a 5×10⁻¹⁰ |
| Initial population M | 10³ a 10⁹ |
| Growth rate during therapy (sensitive) q₁ | 0.5–1.3 |
| Growth rate during therapy (resistant) q₂ | 1.1–1.25 |
| Treatment cycle K | 28 days |
| Optimal death rate c₁ | ~3.15 (yields <10% resistance) |
| Non-optimal high dose | ~40% resistance |

**Predicción clave:** P(resistencia) = 1 - exp(-u × B) donde B = total divisiones de células sensibles. A mayor dosis, menos divisiones sensibles → menos oportunidad de mutación → menos resistencia. PERO: dosis demasiado alta mata sensibles rápido, dejando nicho abierto para resistentes.

## Mapeo a RESONANCE

| Paper | RESONANCE |
|-------|-----------|
| Mutation rate u | Frequency drift magnitude en reproducción |
| Population M | Entity count en arena |
| q₁, q₂ growth rates | Dissipation-modulated growth en batch |
| Death rate c₁ | Drug potency × alignment |
| Pulsed (K=28 days) | Drug on N gens, off M gens |
| P(resistance) | Fraction of worlds donde resistant domina |

## Entregables

### 1. `src/use_cases/experiments/paper_foo_michor2009.rs`

```rust
pub struct FooMichorConfig {
    pub cell_count: u8,
    pub cell_freq: f32,
    pub mutation_magnitude: f32,     // Analog de u
    pub drug_freq: f32,
    pub dose_levels: Vec<f32>,       // Sweep de potencia
    pub pulse_on_gens: u32,          // Gens con droga
    pub pulse_off_gens: u32,         // Gens sin droga
    pub continuous: bool,            // true = siempre on
    pub worlds: usize,
    pub generations: u32,
    pub ticks_per_gen: u32,
}

pub struct FooMichorReport {
    pub dose_resistance_curve: Vec<(f32, f32)>,  // (dose, P(resistance))
    pub optimal_dose: f32,                        // Dose que minimiza P(res)
    pub continuous_resistance_rate: f32,
    pub pulsed_resistance_rate: f32,
    pub optimal_exists: bool,           // Hay un mínimo, no monotónico
    pub pulsed_beats_continuous: bool,  // En al menos 1 dose level
    pub timeline_continuous: Vec<f32>,
    pub timeline_pulsed: Vec<f32>,
}

pub fn run_foo_michor(config: &FooMichorConfig) -> FooMichorReport { ... }

/// Sweep de dosis: ejecuta N worlds por dose level, reporta P(resistance) por nivel.
pub fn run_dose_resistance_sweep(config: &FooMichorConfig) -> Vec<(f32, f32)> { ... }
```

### 2. Tests BDD (≥7 tests)

```
foo_resistance_increases_with_population_size     // P(res) ∝ M
foo_resistance_decreases_with_low_mutation         // u baja → menos resistencia
foo_optimal_dose_exists                            // Curva no es monotónica
foo_very_high_dose_increases_resistance            // Dosis extrema → nicho vacío → resistentes
foo_pulsed_reduces_resistance_vs_continuous        // Al menos en 1 config
foo_zero_drug_zero_resistance_pressure             // Control: sin droga, no hay selección
foo_result_robust_across_5_seeds
```

---

## Scope

**Entra:** 1 archivo .rs, dose sweep, continuous vs pulsed comparison, tests
**NO entra:** Stochastic branching process exacto, continuous-time Markov chain, solver ODE
