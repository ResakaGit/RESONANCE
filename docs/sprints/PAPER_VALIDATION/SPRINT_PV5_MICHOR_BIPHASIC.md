# PV-5: Michor et al. 2005 — Biphasic CML Decline

**Objetivo:** Reproducir el hallazgo de Michor et al. 2005 (Nature): la respuesta de CML a imatinib muestra declive bifásico — fase rápida (células diferenciadas, slope ~0.05/day) seguida de fase lenta (progenitores/stem, slope ~0.005/day). Las stem cells leucémicas NO son eliminadas.

**Estado:** PENDIENTE
**Esfuerzo:** Medio-Alto (requiere modelar 2 compartimentos con cinéticas distintas)
**Bloqueado por:** —

---

## Paper

- **Cita:** Michor F, Hughes TP, Iwasa Y, et al. "Dynamics of chronic myeloid leukaemia." *Nature* 435:1267-1270 (2005)
- **DOI:** 10.1038/nature03669
- **Acceso:** Parcial (Nature, pero números clave en literatura derivada)

## Datos cuantitativos

| Compartimento | Slope (declive/día) | Turnover |
|--------------|-------------------|----------|
| Diferenciadas (TD) | 0.03–0.05 | Alto (días) |
| Progenitoras (P) | 0.004–0.007 | Bajo (semanas) |
| Stem (S) | ~0 (no eliminadas) | Muy bajo (meses) |
| **Ratio slopes** | **~6:1 a 10:1** | — |

**169 pacientes CML** en imatinib. Modelo de 4 compartimentos: S → P → D → TD.

**Predicción clave:** Imatinib no elimina stem cells leucémicas → recaída inevitable si se suspende droga.

## Mapeo a RESONANCE

| Paper | RESONANCE |
|-------|-----------|
| TD (terminally differentiated) | Entities alta freq, high dissipation, bulk population |
| P (progenitors) | Entities freq intermedia, moderate dissipation |
| S (stem cells) | Entities baja freq, low dissipation, quiescent (growth_bias < 0.05) |
| Imatinib | Drug targeting high-freq compartment (alignment alto con TD, bajo con S) |
| Biphasic decline | Population timeline muestra 2 slopes distinguibles |
| Slope ratio | Ratio de slopes ≈ 6-10× entre fase 1 y fase 2 |

**Insight clave:** La frecuencia en RESONANCE actúa como proxy del compartimento. Stem cells tienen frecuencia lejana al drug target → baja alignment → baja respuesta. Esto reproduce el mecanismo biológico (stem cells no expresan el target BCR-ABL al mismo nivel) sin programarlo explícitamente.

## Entregables

### 1. `src/use_cases/experiments/paper_michor2005.rs`

```rust
pub struct MichorConfig {
    pub differentiated_count: u8,     // 70% — bulk (alta freq, cerca del drug)
    pub progenitor_count: u8,         // 20% — intermedio
    pub stem_count: u8,               // 10% — quiescent (freq lejana)
    pub differentiated_freq: f32,     // Cerca del drug target
    pub progenitor_freq: f32,         // Offset moderado
    pub stem_freq: f32,               // Offset grande
    pub stem_growth_bias: f32,        // <0.05 (quiescent)
    pub drug_freq: f32,               // Imatinib target
    pub drug_potency: f32,
    pub worlds: usize,
    pub generations: u32,
    pub ticks_per_gen: u32,
}

pub struct MichorReport {
    pub timeline_total: Vec<f32>,          // Total population per gen
    pub timeline_differentiated: Vec<f32>, // TD compartment per gen
    pub timeline_progenitor: Vec<f32>,     // P compartment per gen
    pub timeline_stem: Vec<f32>,           // S compartment per gen
    pub phase1_slope: f32,                 // Decline rate fase rápida
    pub phase2_slope: f32,                 // Decline rate fase lenta
    pub slope_ratio: f32,                  // phase1/phase2 (target: 6-10×)
    pub stem_cells_survive: bool,          // S > 0 al final
    pub biphasic_detected: bool,           // 2 slopes distinguibles
    pub inflection_gen: u32,               // Gen donde cambia el slope
}

pub fn run_michor(config: &MichorConfig) -> MichorReport { ... }

/// Detecta punto de inflexión en curva de declive (cambio de slope).
fn detect_inflection(timeline: &[f32]) -> Option<(usize, f32, f32)> { ... }
```

### 2. Tests BDD (≥8 tests)

```
michor_total_population_declines_under_drug
michor_differentiated_decline_faster_than_progenitor
michor_stem_cells_survive_treatment              // Clave: stem ≠ 0 al final
michor_biphasic_decline_detected                 // 2 slopes distinguibles
michor_slope_ratio_between_3_and_15              // Paper: 6-10, accept 3-15
michor_phase1_slope_greater_than_phase2
michor_inflection_point_exists
michor_result_robust_across_5_seeds
```

---

## Criterio de éxito

1. **Biphasic decline detectable** — la curva de population vs gen muestra 2 regímenes lineales
2. **Slope ratio 3-15×** — Paper: 6-10×, aceptamos rango más amplio por abstracción
3. **Stem cells sobreviven** — fraction > 0 al final del tratamiento
4. **Robusto** — ≥4/5 seeds muestran el patrón

---

## Scope

**Entra:** 1 archivo .rs, 3-compartment population model, inflection detection, tests
**NO entra:** 4-compartment ODE solver, BCR-ABL molecular modeling, pharmacokinetics de imatinib
