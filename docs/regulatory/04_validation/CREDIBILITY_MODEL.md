---
document_id: RD-4.2
title: Computational Model Credibility Assessment
standard: ASME V&V 40:2018, FDA Guidance — Assessing the Credibility of Computational Modeling and Simulation in Medical Device Submissions (2023)
version: 1.0
date: 2026-04-02
status: DRAFT
author: Resonance Development Team
approved_by: PENDING
review_date: PENDING
review_status: PENDING
---

# Computational Model Credibility Assessment

## 1. Purpose

This document presents a credibility assessment of the RESONANCE computational model following the structure prescribed by ASME V&V 40:2018 and the FDA Guidance on Credibility of Computational Modeling and Simulation in Medical Device Submissions (December 2023). It evaluates the model's fitness for its stated Context of Use (COU), documents verification and validation evidence, quantifies uncertainty, and defines the applicability domain.

RESONANCE is a Class A research simulation tool (RD-1.2). This credibility assessment is performed voluntarily as best practice, not as a regulatory requirement. If RESONANCE's intended use evolves toward SaMD (RD-1.5, Section 5), this document would serve as the foundation for a regulatory submission.

Related documents:

- **RD-1.1** INTENDED_USE.md
- **RD-1.2** SOFTWARE_SAFETY_CLASS.md
- **RD-4.1** VALIDATION_PLAN.md
- **RD-4.3** VERIFICATION_REPORT.md
- **RD-4.4** VALIDATION_REPORT.md
- **RD-4.5** UNCERTAINTY_ANALYSIS.md

## 2. Model Overview

RESONANCE is an emergent life simulation engine where all entities are composed of abstract energy (qe). 14 orthogonal ECS layers define entity composition. 8 foundational axioms govern all interactions. 4 fundamental constants parameterize the physics. All behavior emerges from energy interactions -- no scripted behavior, no templates, no hardcoded trophic classes.

**Architecture:** Rust 2024 / Bevy 0.15 ECS, 113K LOC, 3,113 automated tests, AGPL-3.0.

**Drug models:** Two levels of drug simulation (cytotoxic + pathway inhibitor) using Hill pharmacokinetics with frequency-based binding (Axiom 8). Three inhibition modes (Competitive, Noncompetitive, Uncompetitive). Population-level dynamics with reproduction, mutation, and selection.

**Publication:** Zenodo DOI: 10.5281/zenodo.19342036 -- "Emergent Life from Four Constants: An Axiomatic Simulation Engine."

**Commit hash:** `971c7acb99decde45bf28860e6e10372718c51e2`

---

## 3. ASME V&V 40 Structure

The following sections correspond directly to V&V 40 Sections 4 through 8.

---

## 4. Context of Use (V&V 40 Section 4)

### 4.1 Question of Interest

**Primary question:** Does combination drug therapy produce greater population suppression than monotherapy or double-dose monotherapy in a frequency-heterogeneous population with emergent resistance?

**Secondary questions:**

1. Does monotonic dose-response emerge from frequency-based binding without explicit programming?
2. Can an adaptive therapy controller (modulating drug concentration based on growth rate) stabilize tumor populations better than continuous maximum-dose therapy?
3. Does the model qualitatively reproduce the partial response observed in a real-world canine mast cell tumor case?

### 4.2 Role of the Computational Model

**Role: INFORM (not control, not predict)**

RESONANCE is used to generate qualitative hypotheses about therapeutic resistance dynamics. Its role is to inform research planning -- specifically, to explore which combination strategies might warrant further investigation in vitro or in vivo. It does not predict patient outcomes, calculate dosing, or recommend treatment.

Per IMDRF SaMD N10 (RD-1.1, Section 6): RESONANCE is Category I (Inform, Non-serious).

### 4.3 Decision Consequence

**Low.** If the model produces an incorrect prediction, the consequence is misdirected research effort (investigating a combination strategy that turns out to be suboptimal). No patient is harmed. No clinical decision is affected. The maximum cost is wasted computation time and researcher attention.

| Consequence Dimension | Assessment |
|----------------------|------------|
| Patient safety | None -- no patient in the loop |
| Clinical decision | None -- output is not clinically actionable |
| Research direction | Low -- incorrect predictions lead to wasted exploration, detected during in vitro/in vivo validation |
| Financial | Minimal -- computational cost of experiments is < 2 minutes |
| Regulatory | None -- research tool, no submission |

### 4.4 Regulatory Consequence

**Minimal.** RESONANCE is not submitted to any regulatory authority. This credibility assessment is voluntary. If the model were used as evidence in a regulatory submission, the credibility bar would increase proportionally to the decision consequence (per V&V 40 risk-informed approach).

### 4.5 Credibility Goal (Risk-Informed)

Per V&V 40 Figure 3 (risk-informed credibility framework): low decision consequence requires **moderate** verification and **low-to-moderate** validation evidence. RESONANCE exceeds this baseline with 3,113 automated tests, property-based fuzzing, multi-seed robustness testing, and comparison to published data from 4 independent sources.

---

## 5. Verification (V&V 40 Section 5)

### 5.1 Code Verification

**Objective:** Confirm that the mathematical equations implemented in code faithfully represent the intended mathematical model.

#### 5.1.1 Test Coverage

| Category | Count | Location |
|----------|-------|----------|
| Unit tests (pure math) | ~1,800 | `src/blueprint/equations/` (50+ files) |
| Integration tests (ECS systems) | ~800 | `src/simulation/` modules |
| Batch simulator tests | 156 | `src/batch/` (33 systems, no Bevy) |
| Worldgen tests | ~200 | `src/worldgen/` |
| Property-based (proptest) | 18 properties x 256 cases = ~4,608 executions | `tests/property_conservation.rs` |
| Clinical calibration tests | 21 | `src/blueprint/equations/clinical_calibration.rs` |
| **Total** | **3,113 tests** | `cargo test` -- 0 failures, 35.78s |

#### 5.1.2 Code-to-Equation Traceability

Every public function in `src/blueprint/equations/` includes a doc comment specifying the mathematical formula it implements. Each function has at least one co-located unit test verifying the formula against known values.

Example (from `derived_thresholds.rs`):

```
/// `liquid_threshold = (LIQUID/SOLID)^(1/KLEIBER) x DENSITY_SCALE`
pub fn liquid_density_threshold() -> f32 { ... }

#[test] fn density_thresholds_monotonic() { assert!(gas > liquid > 0.0); }
```

#### 5.1.3 Axiom Compliance Mapping

| Axiom | Implementation | Enforcing Tests |
|-------|---------------|----------------|
| 1: Everything is Energy | All entities are `BaseEnergy { qe: f32 }` | `prop_valid_qe_is_finite_non_negative` |
| 2: Pool Invariant | `scale_extractions_to_available()` enforces sum <= pool | `prop_scale_extractions_invariant` |
| 3: Competition | `extract_competitive()` + `interference_factor()` | `prop_extract_competitive_non_negative_finite` |
| 4: Dissipation | `dissipation_loss()` always positive; rates per matter state | `prop_dissipation_loss_bounded_by_pool`, `senescence_scales_with_dissipation` |
| 5: Conservation | `global_conservation_error()` = 0 when no overshoot | `prop_global_conservation_no_overshoot_means_zero` |
| 7: Distance Attenuation | `coulomb_force()` proportional to 1/r^2 | `coulomb_force_inverse_square` (26 tests in `coulomb.rs`) |
| 8: Oscillatory Nature | `gaussian_frequency_alignment()` modulates all interactions | frequency alignment tests in `determinism.rs`, `pathway_inhibitor.rs` |

### 5.2 Numerical Verification

#### 5.2.1 Floating-Point Determinism

RESONANCE uses f32 exclusively (no f64 intermediates). The custom RNG in `src/blueprint/equations/determinism.rs` provides:

- **`hash_f32_slice()`**: Converts f32 to bits via `to_bits()` before hashing, ensuring +0.0 and -0.0 produce distinct hashes and NaN handling is deterministic.
- **`next_u64()`**: PCG-like state step with wrapping arithmetic. No floating-point operations in the state transition.
- **`unit_f32()`**: Extracts top 24 bits for mantissa quality, producing uniform [0, 1) without precision loss.

**Verification:** 23 unit tests confirm identical output across invocations. `snapshot_hash()` and `snapshots_match()` verify bit-exact simulation state.

#### 5.2.2 f32 Precision Bounds

RESONANCE operates in f32 (23-bit mantissa, ~7.2 decimal digits of precision). Key implications:

| Operation | Precision Concern | Mitigation |
|-----------|------------------|------------|
| Energy summation | Accumulation error O(n * epsilon) | `scale_extractions_to_available()` clamps sum to available |
| Dissipation subtraction | Catastrophic cancellation near zero | `.max(0.0)` guards prevent negative energy |
| `powf()` in Kleiber scaling | Implementation-defined precision | `exact_cache.rs` precomputes `kleiber_volume_factor()` once at growth event |
| Trigonometric in frequency alignment | Platform variation in `cos()` | Acceptable: alignment is a soft modulation, not a threshold |

**Quantitative bound:** For a population of 128 entities, energy summation accumulates at most ~128 * 2^-23 * max_qe relative error. At max_qe = 10,000, this is ~0.15 qe -- negligible relative to typical entity energy (80-200 qe).

### 5.3 Software Quality Assurance

#### 5.3.1 Hard Blocks (CLAUDE.md)

| Rule | Verification Method |
|------|-------------------|
| NO `unsafe` | Codebase search: 0 occurrences in runtime code |
| NO external crates without approval | `Cargo.toml` review: all dependencies approved |
| NO `async`/`await` | Codebase search: 0 occurrences |
| NO `Arc<Mutex<T>>` | Codebase search: 0 occurrences in simulation code |
| NO shared mutable state outside Resources | Codebase search: no `static mut`, no `lazy_static! { Mutex }` |

#### 5.3.2 Coding Standards

17 coding rules + 10 aesthetic guidelines enforced by convention and code review. See `CLAUDE.md` Coding Rules and Code Aesthetic sections.

#### 5.3.3 Sprint Methodology

Development follows a sprint-based methodology with defined closure criteria. Completed sprints are archived in `docs/sprints/archive/` with grep-verified closure evidence.

---

## 6. Validation (V&V 40 Section 6)

### 6.1 Validation Strategy

Validation compares model predictions against independent data sources. RESONANCE's validation evidence consists of:

1. **Direct comparison** to published predictions (Bozic et al. 2013).
2. **Qualitative consistency** with published clinical strategies (Gatenby et al. 2009).
3. **Calibration** against published pharmacological data (London 2003, 2009).
4. **Retrospective case study** (Rosie canine MCT, partial response).

### 6.2 Validation Activities

#### 6.2.1 Experiment 5: Bozic 2013 Replication (Primary Validation)

**Reference:** Bozic et al. 2013, "Evolutionary dynamics of cancer in response to targeted combination therapy," eLife 2:e00747.

**Published prediction:** Combination therapy has exponential advantage over monotherapy in delaying resistance.

**RESONANCE protocol:** 5-arm experiment (no_drug, mono_A, mono_B, combo_AB, double_A). 100 worlds, 80 generations, 200 ticks/generation. Wildtype frequency = 400 Hz, resistant = 250 Hz. Drug targets Root metabolic node.

**Quantitative results:**

| Arm | Efficiency | Suppression | Prediction Met? |
|-----|-----------|-------------|----------------|
| no_drug | 1.000 | 0.0% (baseline) | N/A |
| mono_A | 0.481 | 51.9% | -- |
| mono_B | 0.635 | 36.5% | -- |
| combo_AB | 0.435 | 56.5% | combo > mono_A: YES |
| double_A | 0.466 | 53.4% | combo > double: YES |

**Robustness:** 10 independent seeds, 10/10 confirm `combo_AB < mono_A` and `combo_AB < double_A` with >= 80% threshold. Result is structural, not stochastic.

**File:** `src/use_cases/experiments/pathway_inhibitor_exp.rs`, `src/bin/bozic_validation.rs`

**Validation metric:** Binary (combo > mono: yes/no) across seeds. Quantitative suppression percentages reported but not calibrated to absolute cell counts.

**Known gaps:**

- Comparison is qualitative (suppression %, not absolute resistance timelines in days/weeks)
- RESONANCE uses abstract qe, not molar drug concentrations
- No tumor microenvironment (TME) in model
- Population size (128 entities) is orders of magnitude smaller than real tumors (~10^9 cells)

#### 6.2.2 Experiment 4: Pathway Inhibition Dose-Response

**Reference:** Internal (no external comparator). Tests fundamental model behavior.

**Published prediction:** Dose-response should be monotonic (higher drug concentration -> lower cell efficiency).

**RESONANCE protocol:** Sweep drug concentration from 0.0 to 1.0 in 10 steps. 10 seeds per concentration. Record population-average metabolic efficiency.

**Result:** Strictly monotonic dose-response across all 10 seeds. No non-monotonic artifacts.

**Validation metric:** Monotonicity (efficiency_i >= efficiency_{i+1} for all i).

**File:** `src/use_cases/experiments/pathway_inhibitor_exp.rs`

#### 6.2.3 Experiment 6: Adaptive Therapy Controller

**Reference:** Gatenby et al. 2009, "Adaptive therapy," Cancer Research 69(11):4894-4903.

**Published prediction:** Adaptive (modulated) therapy can delay resistance longer than continuous maximum-dose therapy by maintaining a competitive sensitive population.

**RESONANCE protocol:** Controller modulates drug concentration based on tumor growth rate (increase when growing, decrease when shrinking). Compared against continuous maximum-dose arm. 10 seeds, 100 generations.

**Result:** Growth rate stabilizes at or below zero by generation 40 in 7/10 seeds. Continuous therapy loses control in 6/10 seeds by generation 60.

**Validation metric:** Fraction of seeds achieving growth stabilization (target: >= 70%).

**Known gaps:**

- RESONANCE does not model pharmacokinetics (drug concentration is instantaneous, not governed by ADME)
- The controller is a simple proportional feedback, not the specific protocol described in Gatenby 2009
- Comparison is conceptual (adaptive vs continuous strategy), not quantitative

#### 6.2.4 Experiment 7: Rosie Case (Canine MCT)

**Reference:** London & Seguin 2003 (mast cell tumor biology), London et al. 2009 (toceranib phase I).

**Published data:** Canine mast cell tumor with intermediate grade, partial response to mRNA vaccine (reported in press: Japan Times, Fortune, March 2026).

**RESONANCE protocol:** Calibration profile uses toceranib IC50 (40 nM) as pharmacological proxy for mRNA vaccine potency. 128 entities, 100 generations, wildtype at KIT-mutant frequency. 5 seeds.

**Result:** Tumor population decreases ~35-45% from peak by generation 60 across 5 seeds. Consistent with "partial response" (30-70% reduction by RECIST-like criteria).

**Validation metric:** Population reduction >= 30% from peak (partial response).

**Known gaps:**

- Toceranib IC50 used as proxy for mRNA vaccine (fundamentally different mechanism: kinase inhibition vs immune-mediated killing)
- Calibrated from press reports, not peer-reviewed trial data
- No immune system modeled (immune-mediated killing is abstracted as frequency-selective energy drain)
- Single case validation (n=1 real-world comparison)
- NOT veterinary advice (explicit disclaimer in `pathway_inhibitor_exp.rs` line 1295)

### 6.3 Validation Metrics Summary

| Metric | Type | Threshold | Status |
|--------|------|-----------|--------|
| Bozic combo > mono | Binary (yes/no per seed) | >= 8/10 seeds | 10/10 PASS |
| Bozic combo > double_dose | Binary (yes/no per seed) | >= 8/10 seeds | 10/10 PASS |
| Dose-response monotonicity | Binary per seed | 10/10 seeds | 10/10 PASS |
| Adaptive growth stabilization | Binary per seed | >= 7/10 seeds | 7/10 PASS |
| Rosie partial response | Binary per seed | >= 4/5 seeds | 5/5 PASS |

### 6.4 Known Validation Gaps

| Gap | Severity | Impact on Credibility |
|-----|----------|-----------------------|
| No quantitative comparison to resistance timelines | High | Model cannot predict when resistance emerges (only that it does) |
| No PK/PD model | High | Drug concentration is static, not time-varying |
| No TME model | Medium | Spatial heterogeneity, hypoxia, and immune infiltration are absent |
| Abstract qe units | Medium | Cannot convert predictions to molar concentrations without calibration |
| Small population size (128 entities vs 10^9 cells) | Medium | Stochastic effects amplified; rare resistant clones overrepresented |
| n=1 real-world comparison (Rosie) | Medium | Insufficient for statistical validation |
| No molecular targets | High | Cannot model specific resistance mutations (T315I, T790M, etc.) |
| Calibration profiles are qualitative fits | Medium | Mapping to clinical units introduces unconstrained degrees of freedom |

---

## 7. Uncertainty Quantification (V&V 40 Section 7)

Full details in RD-4.5. Summary below.

### 7.1 Multi-Seed Robustness

All experiments are run with multiple independent seeds (5-10). Acceptance thresholds require >= 70-80% seed agreement. This quantifies aleatory (stochastic) uncertainty within the model.

**Bozic 2013:** 10/10 seeds agree (100%). Coefficient of variation for combo_AB efficiency across seeds: < 5%.

**Adaptive therapy:** 7/10 seeds agree (70%). Higher variance reflects sensitivity to initial population composition.

### 7.2 Parametric Uncertainty

The 4 fundamental constants are the only non-derived parameters:

| Constant | Nominal | +/- 10% Range | Sensitivity |
|----------|---------|---------------|-------------|
| KLEIBER_EXPONENT | 0.75 | 0.675 - 0.825 | High: affects all metabolic scaling |
| DISSIPATION_SOLID | 0.005 | 0.0045 - 0.0055 | Medium: shifts basal drain, senescence |
| COHERENCE_BANDWIDTH | 50.0 Hz | 45.0 - 55.0 Hz | Low-Medium: affects elemental band width |
| DENSITY_SCALE | 20.0 | 18.0 - 22.0 | Low: spatial normalization only |

Formal sensitivity analysis methodology: central difference partial derivatives using `src/blueprint/equations/sensitivity.rs`. Framework defined in RD-4.5.

### 7.3 Model Form Uncertainty

RESONANCE uses abstract energy units (qe), not molar concentrations. The gap between qe-based predictions and real-world outcomes is unquantified and potentially large.

| Model Form Assumption | Real-World Counterpart | Gap |
|-----------------------|----------------------|-----|
| Entity = energy packet (qe) | Cell = complex biological system | Large: no organelles, no signaling cascades |
| Frequency = identity proxy | Genotype/phenotype | Moderate: captures heterogeneity but not specific mutations |
| Hill pharmacokinetics (n=2) | ADME + receptor binding | Large: no absorption, distribution, metabolism, excretion |
| Homogeneous spatial field | Tumor microenvironment | Large: no vasculature, hypoxia, immune infiltration |
| 128 entities | ~10^9 tumor cells | Large: stochastic amplification of rare events |

### 7.4 Numerical Uncertainty

f32 arithmetic introduces precision-limited errors. See Section 5.2.2 for quantitative bounds. Numerical uncertainty is negligible relative to model form uncertainty.

---

## 8. Applicability (V&V 40 Section 8)

### 8.1 Valid For

| Application | Confidence | Evidence |
|-------------|-----------|----------|
| Exploring qualitative resistance dynamics (combo vs mono) | Moderate | Bozic 2013 (10/10 seeds), dose-response monotonicity |
| Comparing therapeutic strategy classes (continuous vs adaptive) | Low-Moderate | Gatenby 2009 conceptual consistency (7/10 seeds) |
| Generating hypotheses about frequency-heterogeneous populations | Moderate | Structural: emerges from axioms without programming |
| Teaching emergent dynamics and ECS architecture | High | 3,113 tests, deterministic, fully documented |
| Demonstrating that 4 constants can derive ~40 lifecycle thresholds | High | 17 algebraic tests in `derived_thresholds.rs` |

### 8.2 NOT Valid For

| Application | Reason | Reference |
|-------------|--------|-----------|
| Clinical dosing decisions | Abstract qe units, no PK/PD, no molecular targets | RD-1.1 Section 5.5 |
| Patient-specific treatment selection | No patient data input, no biomarker analysis | RD-1.1 Section 5.2 |
| Regulatory submission evidence (without further V&V) | Qualitative validation only; no quantitative agreement with patient outcomes | This document, Section 6.4 |
| Predicting resistance timeline (days/weeks) | No time calibration validated against longitudinal data | Section 6.4 |
| Modeling specific molecular targets (EGFR, BCR-ABL) | No molecular resolution | RD-1.1 Section 5.4 |
| Modeling immune response | No immune system layers | RD-1.1 Section 5.6 |
| Predicting rare mutation events | Population size (128) amplifies stochastic effects | Section 7.3 |
| Veterinary treatment guidance | Rosie case is simulation only, uses proxy IC50 | `pathway_inhibitor_exp.rs` disclaimer |

### 8.3 Conditions for Expanded Applicability

To move from "inform research" to "inform clinical management" (IMDRF Category II), RESONANCE would require:

1. **Pharmacokinetic model** -- time-varying drug concentration with ADME parameters.
2. **Molecular resolution** -- specific resistance mutations (e.g., T315I for imatinib, T790M for erlotinib).
3. **Tumor microenvironment** -- vasculature, hypoxia gradients, immune cell interactions.
4. **Quantitative validation** -- comparison to patient-level longitudinal data (e.g., PSA time series for prostate, RECIST measurements for solid tumors).
5. **Larger population sizes** -- at least 10^4 entities to reduce stochastic artifacts.
6. **Independent external validation** -- by a team other than the developers.
7. **Formal regulatory assessment** -- FDA pre-submission (Q-Sub) to clarify evidence requirements.

---

## 9. Credibility Assessment Summary

| V&V 40 Element | Required Level (Low Decision Consequence) | Achieved Level | Status |
|----------------|------------------------------------------|---------------|--------|
| Context of Use (Section 4) | Clearly stated | Clearly stated with negative scope | ADEQUATE |
| Code Verification (Section 5.1) | Moderate | 3,113 tests, zero unsafe, code-to-equation traceability | EXCEEDS |
| Numerical Verification (Section 5.2) | Moderate | f32 determinism, bit-exact hashing, precision bounds | ADEQUATE |
| Software QA (Section 5.3) | Moderate | Hard blocks, coding standards, sprint methodology | ADEQUATE |
| Validation (Section 6) | Low-Moderate | Bozic 10/10, dose-response, adaptive 7/10, Rosie 5/5 | ADEQUATE |
| Uncertainty Quantification (Section 7) | Low | Multi-seed robustness, parametric framework defined | ADEQUATE |
| Applicability (Section 8) | Clearly bounded | Valid/not-valid tables with explicit conditions | ADEQUATE |

**Overall credibility assessment:** ADEQUATE for the stated Context of Use (inform research, low decision consequence). The model produces qualitatively credible predictions for therapeutic resistance dynamics under the documented limitations. It is NOT credible for clinical decision-making, quantitative resistance timeline prediction, or patient-specific treatment selection.

---

## 10. Codebase References

| Reference | File Path |
|-----------|-----------|
| 4 fundamental constants | `src/blueprint/equations/derived_thresholds.rs` (lines 14-34) |
| Derived thresholds (17 tests) | `src/blueprint/equations/derived_thresholds.rs` (lines 331-438) |
| Determinism RNG | `src/blueprint/equations/determinism.rs` |
| Pathway inhibitor equations | `src/blueprint/equations/pathway_inhibitor.rs` |
| Pathway inhibitor constants | `src/blueprint/constants/pathway_inhibitor.rs` |
| Bozic validation binary | `src/bin/bozic_validation.rs` |
| Experiment harness | `src/use_cases/experiments/pathway_inhibitor_exp.rs` |
| Clinical calibration | `src/blueprint/equations/clinical_calibration.rs` |
| Property-based tests | `tests/property_conservation.rs` |
| Sensitivity utilities | `src/blueprint/equations/sensitivity.rs` |
| Coulomb physics | `src/blueprint/equations/coulomb.rs` |
| Conservation equations | `src/blueprint/equations/conservation.rs` |
| Batch simulator | `src/batch/` (19 files) |
| Zenodo paper | https://zenodo.org/records/19342036 |

## 11. Revision History

| Version | Date | Author | Changes |
|---------|------|--------|---------|
| 1.0 | 2026-04-02 | Resonance Development Team | Initial credibility assessment. V&V 40 structure, Context of Use, verification/validation evidence, uncertainty quantification, applicability domain. |

---

**Approvals:**

| Role | Name | Date | Signature |
|------|------|------|-----------|
| Author | Resonance Development Team | 2026-04-02 | _draft -- not signed_ |
| Reviewer | _pending_ | _pending_ | _pending_ |
| Approver | _pending_ | _pending_ | _pending_ |
