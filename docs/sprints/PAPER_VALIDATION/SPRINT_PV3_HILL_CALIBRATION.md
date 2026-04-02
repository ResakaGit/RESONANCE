# PV-3: GDSC/CCLE — Hill Slope Calibration

**Objetivo:** Validar empiricamente que la asunción n=2 (Hill coefficient) de RESONANCE es razonable comparando contra la distribución real de Hill slopes en ~75,000 dose-response curves publicadas.

**Estado:** PENDIENTE
**Esfuerzo:** Bajo (análisis de datos, no simulación compleja)
**Bloqueado por:** —

---

## Fuentes de datos

### GDSC (Genomics of Drug Sensitivity in Cancer)
- **Cita:** Garnett MJ, et al. *Nature* 483:570-575 (2012). DOI: 10.1038/nature11005
- **Datos:** ~75,000 dose-response experiments, 138 drugs × ~700 cell lines
- **Download:** https://www.cancerrxgene.org/downloads/anova
- **Formato:** CSV con IC50, Hill slope, AUC por combinación drug-cell line

### CCLE (Cancer Cell Line Encyclopedia)
- **Cita:** Barretina J, et al. *Nature* 483:603-607 (2012). DOI: 10.1038/nature11003
- **Datos:** 8-point dose-response, 24 compounds × 481 cell lines
- **Download:** https://depmap.org/portal/ccle/
- **Formato:** CSV

## Pregunta específica

RESONANCE usa Hill coefficient n=2 (cooperative binding) en toda la farmacología. ¿Es esto razonable?

**Hipótesis:** La mediana de Hill slopes en datos reales está entre 1.0 y 3.0, con n=2 dentro de 1 desviación estándar.

## Entregables

### 1. `src/use_cases/experiments/paper_hill_ccle.rs`

```rust
pub struct HillCalibrationConfig {
    pub gdsc_slopes: Vec<f32>,       // Parsed from CSV
    pub ccle_slopes: Vec<f32>,       // Parsed from CSV
}

pub struct HillCalibrationReport {
    pub gdsc_count: usize,
    pub gdsc_median: f32,
    pub gdsc_mean: f32,
    pub gdsc_std: f32,
    pub gdsc_p25: f32,
    pub gdsc_p75: f32,
    pub gdsc_fraction_1_to_3: f32,   // Fraction with slope in [1, 3]
    pub ccle_count: usize,
    pub ccle_median: f32,
    pub ccle_mean: f32,
    pub ccle_std: f32,
    pub n2_within_1_std: bool,       // Is n=2 within 1σ of median?
    pub n2_within_iqr: bool,         // Is n=2 within IQR?
    pub resonance_assumption_valid: bool, // Summary verdict
}

/// Estadísticas de Hill slopes — pure math, no IO.
pub fn analyze_hill_slopes(slopes: &[f32]) -> HillStats { ... }

/// Compara n=2 de RESONANCE contra distribución empírica.
pub fn validate_hill_assumption(gdsc: &[f32], ccle: &[f32]) -> HillCalibrationReport { ... }
```

**Nota:** Los datos se parsean fuera de la función pura. El binary lee CSV y pasa `Vec<f32>` a la función stateless. La función pure NO hace IO.

### 2. `src/bin/paper_validation.rs` (parcial — sección Hill)

```rust
// Lee CSV (descargado previamente), extrae Hill slopes, pasa a analyze
// Si CSV no existe, reporta skip (no falla)
```

### 3. Tests BDD (≥5 tests)

```
hill_stats_correct_for_known_distribution     // [1.0, 2.0, 3.0] → median=2.0
hill_empty_input_returns_zero
hill_n2_within_range_1_to_3_for_sample_data
hill_fraction_calculation_correct
hill_resonance_assumption_default_passes      // Con datos hardcoded del paper summary
```

**Test con datos hardcoded:** Si no hay CSV disponible, usar los estadísticos publicados en los papers (median Hill slope ≈ 1.5-2.5 para targeted therapies) como test de referencia.

---

## Scope

**Entra:** 1 archivo .rs, stats puras (median, mean, std, percentiles), tests
**NO entra:** Descargar datos automáticamente, parsear CSV en la lib (solo en bin), modificar Hill n=2 en equations, ML/regression
