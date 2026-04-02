# PV-2: Sharma et al. 2010 — Drug-Tolerant Persisters

**Objetivo:** Reproducir los hallazgos cuantitativos de Sharma et al. 2010 (Cell): una fracción pequeña (~0.3%) de células cancerosas sobrevive terapia citotóxica, muestra 100× menos sensibilidad, y recupera sensibilidad en ~9 doublings tras retirar droga.

**Estado:** PENDIENTE
**Esfuerzo:** Medio
**Bloqueado por:** —

---

## Paper

- **Cita:** Sharma SV, et al. "A chromatin-mediated reversible drug-tolerant state in cancer cell subpopulations." *Cell* 141(1):69-80 (2010)
- **DOI:** 10.1016/j.cell.2010.02.027
- **Acceso:** Open access (PMC2851638)

## Datos cuantitativos

| Métrica | Valor |
|---------|-------|
| Persister fraction (DTPs) | ~0.3% de PC9 |
| Sensitivity reduction | >100-fold |
| Recovery (DTPs → sensitive) | ~9 doublings |
| Recovery (DTEPs → sensitive) | ~90 doublings (~30 passages) |
| CD133+ enrichment | 2% parental → 100% DTPs |
| DTP detection | 9 days post-exposure |
| DTEP clones after 30 days | ~50 |
| DTPs resuming proliferation | ~20% in drug |

## Mapeo a RESONANCE

| Paper | RESONANCE |
|-------|-----------|
| DTPs (persisters) | Quiescent entities (growth_bias < 0.05) |
| 100× sensitivity reduction | Frequency far from drug target (low α) |
| Recovery en 9 doublings | Epigenetic adaptation reverts expression_mask |
| CD133+ enrichment | High-coherence entities survive preferentially |
| Persister fraction 0.3% | Initial quiescent_fraction in config |

## Entregables

### 1. `src/use_cases/experiments/paper_sharma2010.rs`

```rust
pub struct SharmaConfig {
    pub total_cells: u8,              // 64 (scaled from millions)
    pub quiescent_fraction: f32,      // 0.003 (0.3%)
    pub quiescent_growth_bias: f32,   // <0.05 (slow cycling)
    pub quiescent_freq_offset: f32,   // Far from drug → low α
    pub drug_freq: f32,
    pub drug_potency: f32,            // High — cytotoxic
    pub drug_start_gen: u32,
    pub drug_stop_gen: u32,           // Remove drug for recovery phase
    pub recovery_gens: u32,           // Observe recovery
    pub worlds: usize,
    pub generations: u32,
    pub ticks_per_gen: u32,
}

pub struct SharmaReport {
    pub initial_population: f32,
    pub post_drug_survivors: f32,        // Should be ~0.3% of initial
    pub persister_fraction: f32,         // survivors / initial
    pub sensitivity_ratio: f32,          // drug effect on persisters vs sensitive
    pub recovery_gen: Option<u32>,       // Gen where sensitivity restores
    pub recovery_doublings: f32,         // Doublings until sensitive again
    pub timeline_population: Vec<f32>,   // Pop per gen
    pub timeline_persister_frac: Vec<f32>,
    pub persister_fraction_matches: bool,  // 0.1% < fraction < 1%
    pub recovery_matches: bool,            // recovers within 5-15 doublings
}
```

**Protocolo:**
1. Gens 0–drug_start: crecimiento libre
2. Gens drug_start–drug_stop: terapia citotóxica alta potencia
3. Gens drug_stop–end: retirada de droga, observar recovery

### 2. Tests BDD (≥7 tests)

```
sharma_persister_fraction_between_01_and_1_percent
sharma_bulk_population_killed_by_drug          // >90% muere
sharma_persisters_survive_drug                  // fracción sobrevive
sharma_persisters_less_sensitive_than_bulk      // sensitivity ratio >10×
sharma_sensitivity_recovers_after_drug_removal
sharma_recovery_within_15_doublings             // Paper: 9, accept <15
sharma_result_robust_across_5_seeds
```

---

## Scope

**Entra:** 1 archivo .rs, tests inline
**NO entra:** Modelar CD133, epigenética molecular real, chromatin remodeling
