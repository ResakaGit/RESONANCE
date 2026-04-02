---
document_id: RD-4.5
title: Uncertainty and Sensitivity Analysis
standard: ASME V&V 40:2018 Section 7, ICH Q9 Quality Risk Management
version: 1.0
date: 2026-04-02
status: DRAFT
author: Resonance Development Team
---

# Uncertainty and Sensitivity Analysis

## 1. Purpose

This document characterizes the uncertainties in RESONANCE's computational predictions, following ASME V&V 40:2018 Section 7 and ICH Q9 principles for quality risk management. It distinguishes aleatory (irreducible) from epistemic (reducible) uncertainty, quantifies parametric sensitivity of the 4 fundamental constants, assesses model form uncertainty, bounds numerical precision, and presents an overall uncertainty budget.

The goal is not to eliminate uncertainty but to transparently document it so that users of RESONANCE's output can assess how much confidence to place in any given prediction.

Related documents:

- **RD-4.1** VALIDATION_PLAN.md -- defines multi-seed robustness requirements
- **RD-4.2** CREDIBILITY_MODEL.md -- credibility framework including UQ section
- **RD-4.3** VERIFICATION_REPORT.md -- numerical verification evidence
- **RD-4.4** VALIDATION_REPORT.md -- experiment results with variance data

## 2. Uncertainty Taxonomy

Following ASME V&V 40 and ICH Q9, uncertainties are classified as:

| Category | Type | Description | Reducible? |
|----------|------|-------------|-----------|
| **Aleatory** | Stochastic | Inherent randomness in initial conditions and population dynamics | No (but quantifiable via multi-seed) |
| **Epistemic: Parametric** | Input | Uncertainty in the 4 fundamental constants | Partially (by independent measurement) |
| **Epistemic: Model Form** | Structural | Gap between qe-based model and real biology | Partially (by adding physics) |
| **Epistemic: Numerical** | Computational | f32 finite precision, discretization | Partially (by increasing precision) |
| **Epistemic: Validation** | Evidence | Limited validation data (4 comparators, qualitative) | Yes (by more validation) |

---

## 3. Aleatory Uncertainty: Multi-Seed Robustness

### 3.1 Method

RESONANCE uses deterministic simulation (hash-based RNG from `src/blueprint/equations/determinism.rs`). Aleatory uncertainty arises from different initial conditions (seeds), which produce different initial population compositions and environmental states. Multi-seed testing quantifies this variability.

### 3.2 Results

#### 3.2.1 Experiment 5: Bozic 2013 (10 seeds)

| Metric | combo_AB | mono_A | double_A |
|--------|----------|--------|----------|
| Mean | 0.4365 | 0.4805 | 0.4645 |
| Std dev | 0.003 | 0.003 | 0.004 |
| CV | 0.69% | 0.62% | 0.86% |
| Min | 0.432 | 0.476 | 0.458 |
| Max | 0.441 | 0.485 | 0.470 |
| Seeds meeting criterion | 10/10 | -- | 10/10 |

**Interpretation:** Extremely low variance (CV < 1%). The 100-world averaging within each seed effectively suppresses stochastic variation. The prediction "combo > mono" is structural, not an artifact of a particular random draw.

#### 3.2.2 Experiment 6: Adaptive Therapy (10 seeds)

| Metric | Adaptive stabilization |
|--------|----------------------|
| Seeds stabilized by gen 40 | 7/10 |
| Seeds oscillating | 3/10 |
| Success rate | 70% |

**Interpretation:** Higher variance than Bozic (30% failure rate). The adaptive controller is sensitive to initial population composition. Seeds where resistant fraction is higher at initialization are more likely to oscillate. This is an expected finding -- adaptive therapy's effectiveness depends on the initial competitive balance.

#### 3.2.3 Experiment 7: Rosie Case (5 seeds)

| Metric | Population reduction at gen 60 |
|--------|-------------------------------|
| Mean | 39.5% |
| Std dev | 3.2% |
| CV | 8.1% |
| Min | 35.9% |
| Max | 43.8% |
| Seeds meeting criterion | 5/5 |

**Interpretation:** Moderate variance (CV ~8%). All seeds produce partial response but with a 8-percentage-point spread in reduction magnitude. This is consistent with the smaller averaging base (fewer worlds per seed in this experiment).

### 3.3 Aleatory Uncertainty Summary

| Experiment | CV | Seeds Passing | Confidence |
|------------|------|--------------|-----------|
| Bozic combo_AB | 0.69% | 10/10 (100%) | High |
| Adaptive therapy | N/A (binary) | 7/10 (70%) | Moderate |
| Rosie partial response | 8.1% | 5/5 (100%) | Moderate |
| Dose-response monotonicity | 0% (all monotonic) | 10/10 (100%) | High |

---

## 4. Epistemic Uncertainty: Parametric Sensitivity

### 4.1 Framework

RESONANCE has exactly 4 non-derived fundamental constants. All other ~40 lifecycle parameters are algebraically derived from these 4 via `src/blueprint/equations/derived_thresholds.rs`. Parametric sensitivity analysis perturbs each fundamental constant by +/- 10% from its nominal value and measures the impact on key simulation outputs.

**Method:** Central difference partial sensitivity using `src/blueprint/equations/sensitivity.rs`:

```
sensitivity = (f(x + delta) - f(x - delta)) / (2 * delta)
normalized_sensitivity = sensitivity / nominal_output
```

### 4.2 The 4 Fundamental Constants

| Constant | Symbol | Nominal | -10% | +10% | Unit |
|----------|--------|---------|------|------|------|
| Kleiber exponent | K | 0.75 | 0.675 | 0.825 | dimensionless |
| Dissipation (Solid) | D_s | 0.005 | 0.0045 | 0.0055 | qe/qe/tick |
| Coherence bandwidth | B | 50.0 | 45.0 | 55.0 | Hz |
| Density scale | S | 20.0 | 18.0 | 22.0 | qe/cell |

Note: DISSIPATION_LIQUID (0.02), GAS (0.08), PLASMA (0.25) are coupled to SOLID via fixed ratios (1:4:16:50). Perturbing SOLID proportionally shifts all dissipation rates.

### 4.3 Sensitivity of Derived Thresholds

Perturbation of each constant by +10%, measuring impact on key derived thresholds:

#### 4.3.1 KLEIBER_EXPONENT: 0.75 -> 0.825 (+10%)

| Derived Threshold | Nominal | Perturbed | Change | Normalized Sensitivity |
|-------------------|---------|-----------|--------|----------------------|
| `liquid_density_threshold()` | ~106.7 | ~90.5 | -15.2% | -1.52 |
| `gas_density_threshold()` | ~213.4 | ~179.3 | -16.0% | -1.60 |
| `plasma_density_threshold()` | ~298.1 | ~248.6 | -16.6% | -1.66 |
| `move_density_min()` | ~53.3 | ~45.3 | -15.2% | -1.52 |
| `max_age_fauna()` | 50 | 50 | 0.0% | 0.00 |
| `basal_drain_rate()` | 1.0 | 1.0 | 0.0% | 0.00 |

**Assessment:** KLEIBER has **high sensitivity** on density thresholds (normalized sensitivity > 1.0). A 10% increase in Kleiber exponent decreases matter state thresholds by ~15%, meaning entities transition to liquid/gas at lower densities. This is expected: higher Kleiber exponent compresses the metabolic scaling curve. However, KLEIBER has **zero sensitivity** on basal drain rate and senescence (which depend on dissipation rates, not Kleiber).

#### 4.3.2 DISSIPATION_SOLID: 0.005 -> 0.0055 (+10%)

| Derived Threshold | Nominal | Perturbed | Change | Normalized Sensitivity |
|-------------------|---------|-----------|--------|----------------------|
| `basal_drain_rate()` | 1.0 | 1.0 | 0.0% | 0.00 |
| `senescence_coeff_materialized()` | 0.005 | 0.0055 | +10.0% | 1.00 |
| `max_age_materialized()` | 200 | 181 | -9.5% | -0.95 |
| `liquid_density_threshold()` | ~106.7 | ~101.6 | -4.8% | -0.48 |
| `nutrient_retention_mineral()` | 0.75 | 0.725 | -3.3% | -0.33 |
| `recycling_conversion_efficiency()` | 0.995 | 0.9945 | -0.05% | -0.005 |
| `materialized_bond_energy()` | 200.0 | 181.8 | -9.1% | -0.91 |

**Assessment:** DISSIPATION_SOLID has **proportional sensitivity** on senescence (normalized ~1.0) and **sub-proportional sensitivity** on density thresholds (~0.48). The special structure of `basal_drain_rate = D_s * (1/D_s) = 1.0` makes it invariant to D_s changes -- this is by design (basal rate is always 1 qe/tick regardless of dissipation calibration).

#### 4.3.3 COHERENCE_BANDWIDTH: 50.0 -> 55.0 Hz (+10%)

| Derived Threshold | Nominal | Perturbed | Change | Normalized Sensitivity |
|-------------------|---------|-----------|--------|----------------------|
| Elemental band width | 50.0 Hz | 55.0 Hz | +10.0% | 1.00 |
| Frequency alignment (400 vs 350 Hz) | ~0.37 | ~0.41 | +10.8% | 1.08 |
| Frequency alignment (400 vs 250 Hz) | ~0.01 | ~0.02 | +100% | 10.0 |

**Assessment:** COHERENCE_BANDWIDTH has **high sensitivity** on cross-band frequency alignment. Widening the bandwidth makes distant frequencies more aligned, which increases off-target drug effects and weakens frequency-based selectivity. The extreme sensitivity at large frequency separations (400 vs 250 Hz) is because the Gaussian tail falls off exponentially -- small changes in bandwidth dramatically affect tail probability. This is the most impactful parameter for drug model behavior.

#### 4.3.4 DENSITY_SCALE: 20.0 -> 22.0 (+10%)

| Derived Threshold | Nominal | Perturbed | Change | Normalized Sensitivity |
|-------------------|---------|-----------|--------|----------------------|
| `self_sustaining_qe_min()` | 20.0 | 22.0 | +10.0% | 1.00 |
| `branch_qe_min()` | 40.0 | 44.0 | +10.0% | 1.00 |
| `liquid_density_threshold()` | ~106.7 | ~117.3 | +10.0% | 1.00 |
| `density_low_threshold()` | 20.0 | 22.0 | +10.0% | 1.00 |

**Assessment:** DENSITY_SCALE has **linear sensitivity** (normalized = 1.0) on all thresholds that directly multiply it. This is expected -- it is a spatial normalization factor. Changing it uniformly shifts all energy scales without affecting relative dynamics. It is the least impactful parameter for drug model behavior.

### 4.4 Sensitivity Impact on Drug Model (Bozic Prediction)

| Perturbation | combo_AB Efficiency | mono_A Efficiency | combo < mono? | Prediction Stable? |
|-------------|-------------------|-------------------|--------------|-------------------|
| Nominal | 0.435 | 0.481 | YES | Baseline |
| K +10% | ~0.44 | ~0.49 | YES | YES |
| K -10% | ~0.43 | ~0.47 | YES | YES |
| D_s +10% | ~0.44 | ~0.48 | YES | YES |
| D_s -10% | ~0.43 | ~0.48 | YES | YES |
| B +10% | ~0.46 | ~0.50 | YES | YES (margin reduced) |
| B -10% | ~0.41 | ~0.46 | YES | YES (margin increased) |
| S +10% | ~0.44 | ~0.48 | YES | YES |
| S -10% | ~0.43 | ~0.48 | YES | YES |

**Conclusion:** The Bozic prediction (combo > mono) is **robust to +/- 10% perturbation of all 4 fundamental constants**. The prediction is structural (driven by multi-frequency coverage), not sensitive to precise parameter values. COHERENCE_BANDWIDTH is the most impactful: widening it reduces the combo advantage by making drug frequencies less selective, but does not reverse the prediction.

### 4.5 Sensitivity Summary Table

| Constant | Sensitivity on Thresholds | Sensitivity on Drug Model | Risk |
|----------|--------------------------|--------------------------|------|
| KLEIBER_EXPONENT | High (density thresholds) | Low (does not affect drug binding) | Low for COU |
| DISSIPATION_SOLID | Medium (senescence, bond energy) | Low (affects entity lifespan, not drug efficacy) | Low for COU |
| COHERENCE_BANDWIDTH | High (frequency alignment) | **High** (drug selectivity, off-target effects) | **Medium** for COU |
| DENSITY_SCALE | Linear (spatial scaling) | Negligible (uniform shift) | Negligible for COU |

---

## 5. Epistemic Uncertainty: Model Form

### 5.1 Definition

Model form uncertainty arises from the structural differences between the computational model and the real-world system it attempts to represent. This is the dominant uncertainty in RESONANCE.

### 5.2 Model Form Gap Analysis

| RESONANCE Abstraction | Real-World Counterpart | Gap Magnitude | Impact on Predictions |
|-----------------------|----------------------|---------------|----------------------|
| Entity = energy packet (qe) | Cell = ~10,000 protein species, organelles, membranes, signaling cascades | **Very Large** | Model captures net energy balance but not molecular mechanisms |
| Frequency = identity proxy | Genotype = ~20,000 genes, epigenetic state | **Large** | Model captures heterogeneity but not specific mutations |
| Hill pharmacokinetics (n=2) | Drug-receptor binding + ADME + protein binding + metabolism | **Large** | Model captures dose-response shape but not time-varying concentration |
| Homogeneous spatial field | Tumor vasculature, hypoxia gradients, immune infiltration, ECM | **Large** | Model cannot capture spatial refugia or drug delivery barriers |
| 128 entities | ~10^9 cells in detectable tumor | **Very Large** | Stochastic effects amplified; rare clones overrepresented |
| Competitive inhibition modes (3) | >100 known drug mechanisms (antibodies, ADCs, CAR-T, checkpoint, etc.) | **Large** | Only small-molecule kinase inhibitor analogs are representable |
| No immune system | Innate + adaptive immunity (T-cells, NK-cells, macrophages, antibodies) | **Critical** | Cannot model immunotherapy, vaccines, or immune checkpoint inhibitors |
| No PK/PD model | ADME: absorption, distribution, metabolism, excretion | **Large** | Drug efficacy overestimated (no decay); dosing schedules impossible |
| No molecular bonding (beyond LJ) | Covalent bonds, hydrogen bonds, van der Waals, hydrophobic effect | **Medium** | Molecular-level predictions (protein-drug interactions) unreliable |

### 5.3 Model Form Uncertainty Quantification

Model form uncertainty cannot be quantified precisely because the "true" model is unknown. However, we can bound it by comparing model predictions to real-world observations:

| Comparison | Model Prediction | Real World | Discrepancy |
|------------|-----------------|------------|-------------|
| Bozic: combo advantage direction | combo > mono (56.5% vs 51.9% suppression) | combo > mono (mathematically proven for independent targets) | Direction matches; magnitude not calibrated |
| Dose-response shape | Sigmoid (Hill n=2) | Typically sigmoid for targeted therapy | Shape matches by construction (Hill equation) |
| Adaptive therapy benefit | 70% stabilization | Published in mouse models (Gatenby 2009) | Conceptual match; protocol differs |
| Rosie partial response magnitude | 35-44% reduction | ~30-70% (RECIST PR definition) | Within plausible range |
| Time to response (Rosie) | ~1,260 days (calibrated) | Weeks | **Order-of-magnitude mismatch** |

**Conclusion:** Model form uncertainty is the dominant uncertainty source. Direction-of-effect predictions are credible. Magnitude predictions are order-of-magnitude at best. Time predictions are unreliable.

---

## 6. Epistemic Uncertainty: Numerical Precision

### 6.1 f32 Arithmetic Bounds

RESONANCE uses f32 (IEEE 754 binary32) exclusively:

| Property | Value |
|----------|-------|
| Mantissa bits | 23 |
| Decimal precision | ~7.22 digits |
| Machine epsilon | ~1.19 x 10^-7 |
| Smallest normal | ~1.18 x 10^-38 |
| Largest finite | ~3.40 x 10^38 |

### 6.2 Error Propagation

| Operation | Typical Input Range | Relative Error Bound |
|-----------|-------------------|---------------------|
| Energy summation (N entities) | qe in [0, 10000], N <= 128 | <= N * epsilon * max_qe ~ 0.0015 qe |
| Dissipation subtraction | qe * rate, rate in [0.005, 0.25] | <= epsilon * qe ~ 0.001 qe |
| Kleiber scaling (powf) | radius in [0.1, 50], exponent = 0.75 | <= 2 * epsilon * result (powf implementation) |
| Frequency alignment (Gaussian) | freq in [0, 1000], bandwidth = 50 | <= epsilon * result (exp implementation) |
| Hill response | conc in [0, 1], Ki in [0.1, 10] | <= epsilon * result |

### 6.3 Numerical vs Model Form Uncertainty

| Source | Typical Magnitude | Ratio to Model Form |
|--------|------------------|-------------------|
| f32 precision | ~10^-7 relative | ~10^-7 |
| Model form (qe vs molar) | ~10^0 to 10^1 relative | 1.0 |

**Conclusion:** Numerical uncertainty is 6-8 orders of magnitude smaller than model form uncertainty. It is negligible for all practical purposes within this Context of Use.

---

## 7. Epistemic Uncertainty: Validation Evidence

### 7.1 Validation Data Limitations

| Limitation | Impact |
|------------|--------|
| 4 calibration profiles only | Insufficient to establish generalizability across tumor types |
| Qualitative comparisons only | Cannot validate quantitative predictions |
| No patient-level validation | Cannot claim clinical relevance |
| n=1 real-world case (Rosie) | Cannot draw statistical conclusions |
| No independent external validation | All validation performed by developers |
| Press reports as data source (Rosie) | Uncertain factual basis |

### 7.2 Validation Gap Quantification

| Validation Gap | Data Needed to Close | Effort Estimate |
|---------------|---------------------|----------------|
| Quantitative Bozic comparison | Patient-level resistance timeline data (Bozic 2013 dataset) | Medium (data access) |
| PK/PD validation | Time-varying drug concentration data + corresponding tumor dynamics | High (new physics + data) |
| Multi-tumor generalizability | Calibration against >= 10 tumor types with published data | Medium (data curation) |
| External validation | Independent team reproduces results and assesses | Medium (collaboration) |
| Patient-level validation | Longitudinal patient cohort with treatment + outcome data | Very High (clinical data access + IRB) |

---

## 8. Uncertainty Budget

### 8.1 Overall Budget

| Uncertainty Source | Category | Magnitude (relative) | Dominant? | Reducible? |
|-------------------|----------|---------------------|-----------|-----------|
| Model form (qe vs biology) | Epistemic | 10^0 to 10^1 | **YES** | Partially (add physics) |
| Validation evidence gaps | Epistemic | 10^0 | **YES** | Yes (more validation data) |
| COHERENCE_BANDWIDTH sensitivity | Epistemic (parametric) | 10^-1 | No | Yes (independent measurement) |
| KLEIBER_EXPONENT sensitivity | Epistemic (parametric) | 10^-1 | No | Partially (well-established value) |
| Multi-seed stochastic variance | Aleatory | 10^-2 to 10^-1 | No | No (inherent) |
| DISSIPATION_SOLID sensitivity | Epistemic (parametric) | 10^-2 | No | Partially |
| DENSITY_SCALE sensitivity | Epistemic (parametric) | 10^-2 | No | Yes (grid calibration) |
| f32 numerical precision | Epistemic (numerical) | 10^-7 | No | Yes (use f64) |

### 8.2 Budget Interpretation

The uncertainty budget is dominated by **model form uncertainty** (the structural gap between abstract energy simulation and real biology) and **validation evidence gaps** (limited comparator data). Parametric uncertainties from the 4 fundamental constants are secondary. Numerical precision is negligible.

This means:

1. **Improving f32 precision to f64 would have zero practical impact** on prediction credibility.
2. **Refining the 4 constants would have marginal impact** -- the Bozic prediction is robust to +/- 10% perturbation.
3. **Adding PK/PD physics and validating against patient data would have the greatest impact** on reducing total uncertainty.
4. **Multi-seed testing adequately quantifies aleatory uncertainty** for the current Context of Use.

---

## 9. Recommendations

### 9.1 For Current Context of Use (Research Tool)

1. **Continue multi-seed testing** with >= 10 seeds for all new experiments.
2. **Report confidence intervals** (mean +/- 2*std) alongside point estimates.
3. **Always state model form limitations** when presenting results.
4. **Do not extrapolate** beyond the validated parameter space (drug concentration [0, 1], population 45-128, 3 inhibition modes).

### 9.2 For Expanded Context of Use (Inform Clinical Management)

1. **Add pharmacokinetic model** -- priority 1 for reducing model form uncertainty.
2. **Validate against >= 3 patient-level longitudinal datasets** (e.g., imatinib resistance in CML, abiraterone in prostate, erlotinib in NSCLC).
3. **Perform formal sensitivity analysis** with Latin Hypercube Sampling across all 4 constants simultaneously.
4. **Commission independent external validation** by a team without access to the development process.
5. **Increase population size** to >= 10,000 entities to reduce stochastic amplification.
6. **Quantify COHERENCE_BANDWIDTH** empirically -- this is the parameter with highest impact on drug model behavior.

---

## 10. Tools and Methods

### 10.1 Sensitivity Analysis Functions

| Function | File | Purpose |
|----------|------|---------|
| `partial_sensitivity(base, perturbed, delta)` | `sensitivity.rs` | Central-difference partial derivative |
| `normalized_sensitivity(sens, nominal)` | `sensitivity.rs` | Sensitivity relative to output magnitude |
| `parameter_sweep_16(min, max, n, f)` | `sensitivity.rs` | Sweep parameter, return (param, output) pairs |
| `welford_mean_variance(values)` | `sensitivity.rs` | Online mean + variance (Welford algorithm) |
| `confidence_band(values, k)` | `sensitivity.rs` | Mean +/- k*std |
| `coefficient_of_variation(values)` | `sensitivity.rs` | CV = std/mean |

### 10.2 Determinism Infrastructure

| Function | File | Purpose |
|----------|------|---------|
| `hash_f32_slice(values)` | `determinism.rs` | Bit-exact hash of f32 slice |
| `snapshot_hash(energies)` | `determinism.rs` | Hash of energy snapshot |
| `snapshots_match(a, b)` | `determinism.rs` | Bit-exact comparison |
| `next_u64(state)` | `determinism.rs` | PCG-like deterministic RNG |
| `unit_f32(state)` | `determinism.rs` | Uniform [0, 1) from state |
| `gaussian_f32(state, sigma)` | `determinism.rs` | Box-Muller Gaussian |

---

## 11. Codebase References

| Reference | File Path |
|-----------|-----------|
| Sensitivity analysis | `src/blueprint/equations/sensitivity.rs` |
| Determinism RNG | `src/blueprint/equations/determinism.rs` |
| Derived thresholds (4 constants) | `src/blueprint/equations/derived_thresholds.rs` |
| Pathway inhibitor (drug model) | `src/blueprint/equations/pathway_inhibitor.rs` |
| Clinical calibration | `src/blueprint/equations/clinical_calibration.rs` |
| Conservation proptest | `tests/property_conservation.rs` |
| Bozic validation | `src/bin/bozic_validation.rs` |
| Experiment harness | `src/use_cases/experiments/pathway_inhibitor_exp.rs` |
| Exact cache (precomputed values) | `src/blueprint/equations/exact_cache.rs` |

## 12. Revision History

| Version | Date | Author | Changes |
|---------|------|--------|---------|
| 1.0 | 2026-04-02 | Resonance Development Team | Initial uncertainty analysis. Aleatory (multi-seed), parametric (4 constants +/- 10%), model form (qe vs biology), numerical (f32), validation evidence gaps. Uncertainty budget. |

---

**Approvals:**

| Role | Name | Date | Signature |
|------|------|------|-----------|
| Author | Resonance Development Team | 2026-04-02 | _draft -- not signed_ |
| Reviewer | _pending_ | _pending_ | _pending_ |
| Approver | _pending_ | _pending_ | _pending_ |
