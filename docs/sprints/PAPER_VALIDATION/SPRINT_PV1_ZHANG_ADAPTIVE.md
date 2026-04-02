# PV-1: Zhang et al. 2022 — Adaptive Therapy Prostate Cancer

**Objetivo:** Reproducir el resultado principal de Zhang et al. 2022 (eLife): terapia adaptativa extiende TTP 2.3× vs terapia continua (33.5 vs 14.3 meses) en cáncer de próstata metastásico.

**Estado:** PENDIENTE
**Esfuerzo:** Medio (1 módulo + 1 binary + tests)
**Bloqueado por:** —

---

## Paper

- **Cita:** Zhang J, et al. "Evolution-based mathematical models significantly prolong response to abiraterone in metastatic castrate-resistant prostate cancer." *eLife* 11:e76284 (2022)
- **DOI:** 10.7554/eLife.76284
- **Acceso:** Open access

## Datos cuantitativos del paper

| Métrica | SOC (continuous) | Adaptive | Ratio |
|---------|-----------------|----------|-------|
| Median TTP | 14.3 meses | 33.5 meses | 2.34× |
| Median OS | 31.3 meses | 58.5 meses | 1.87× |
| Tiempo off-therapy | 0% | 46% | — |
| Pacientes | 16 | 17 | — |

**Modelo Lotka-Volterra del paper:**
- 3 subpoblaciones: T+ (sensitive), TP (partially resistant), T- (fully resistant)
- Growth rates: T+ = 0.0278/day, TP = 0.0355/day, T- = 0.0665/day
- Carrying capacity K = 10,000
- Competition coefficients α: 0.4–0.9 entre subpoblaciones
- Protocolo: off cuando PSA baja 50%, on cuando PSA vuelve a baseline

## Mapeo a RESONANCE

| Paper | RESONANCE |
|-------|-----------|
| T+ (sensitive) | Entities con freq ≈ drug freq (alta alignment) |
| TP (partial) | Entities con freq moderadamente desplazada |
| T- (resistant) | Entities con freq lejana al drug |
| Growth rates | Dissipation-dependent growth en batch |
| α competition | Interference factor (Axiom 8) |
| PSA | Total population qe (proxy) |
| On/off protocol | Adaptive controller con threshold-based switching |

## Entregables

### 1. `src/use_cases/experiments/paper_zhang2022.rs`

```rust
pub struct ZhangConfig {
    pub sensitive_count: u8,      // T+ (≈55% initial)
    pub partial_count: u8,        // TP (≈30% initial)
    pub resistant_count: u8,      // T- (≈15% initial)
    pub sensitive_freq: f32,      // Near drug target
    pub partial_freq: f32,        // Offset by ~0.5 bandwidth
    pub resistant_freq: f32,      // Offset by >1 bandwidth
    pub drug_freq: f32,
    pub drug_conc: f32,
    pub psa_off_threshold: f32,   // 0.50 (50% decline → pause)
    pub psa_on_threshold: f32,    // 1.00 (return to baseline → resume)
    pub worlds: usize,
    pub generations: u32,
    pub ticks_per_gen: u32,
}

pub struct ZhangReport {
    pub continuous_ttp_gen: u32,       // Gen where continuous arm progresses
    pub adaptive_ttp_gen: u32,         // Gen where adaptive arm progresses
    pub ttp_ratio: f32,                // adaptive/continuous (target: ≈2.3)
    pub drug_exposure_ratio: f32,      // adaptive/continuous (target: ≈0.54)
    pub adaptive_cycles: u32,          // Number of on/off cycles
    pub timeline_continuous: Vec<f32>, // Efficiency per gen
    pub timeline_adaptive: Vec<f32>,   // Efficiency per gen
    pub prediction_met: bool,          // adaptive TTP > continuous TTP
}
```

**Lógica clave:**
- Dos brazos: continuous (drug always on) vs adaptive (threshold-based on/off)
- TTP = generación donde efficiency cae por debajo de threshold irreversiblemente
- Drug exposure = fracción de generaciones con drug activa

### 2. Tests BDD (≥8 tests)

```
zhang_adaptive_ttp_exceeds_continuous
zhang_ttp_ratio_above_1_5x                // Paper: 2.34×, accept ≥1.5×
zhang_drug_exposure_below_70_percent      // Paper: 54%, accept <70%
zhang_resistant_fraction_grows_under_continuous
zhang_sensitive_fraction_recovers_during_holiday
zhang_adaptive_cycles_at_least_2
zhang_result_robust_across_5_seeds
zhang_continuous_always_progresses
```

### 3. Línea en mod.rs

```rust
pub mod paper_zhang2022;
```

---

## Criterio de éxito

El test principal es: `adaptive TTP > continuous TTP` en ≥8/10 seeds. El ratio exacto (2.34×) no es el target — RESONANCE usa unidades abstractas. Lo que debe ser estructuralmente verdadero:

1. Adaptive therapy prolonga TTP vs continuous
2. Adaptive usa menos droga total
3. Subpoblación sensible se recupera durante drug holidays
4. Resistencia crece más lento bajo adaptive

---

## Scope

**Entra:** 1 archivo .rs, tests inline, 1 línea en mod.rs
**NO entra:** Modificar batch systems, modificar equations, agregar crates, calibración cuantitativa exacta (meses → generaciones)
