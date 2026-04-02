---
document_id: RD-6.2
title: Clinical Evaluation Report
standard: IMDRF SaMD N41
version: 1.0
date: 2026-04-02
status: DRAFT
author: Resonance Development Team
---

# Clinical Evaluation Report

## 1. Purpose

This report synthesizes all available evidence for RESONANCE against the Clinical Evaluation Plan (RD-6.1). It evaluates whether the evidence supports RESONANCE's intended use as a research simulation tool (IEC 62304 Class A, IMDRF SaMD Category I) and identifies residual gaps.

This report follows the IMDRF SaMD N41 structure: analytical evidence (verification), performance evidence (validation), clinical evidence (contextualization), benefit-risk assessment, and conclusion.

**Cross-references:**

- RD-6.1 (Clinical Evaluation Plan): Defines scope, claims, evidence types, and acceptance criteria
- RD-6.3 (Limitations and Scope Report): Detailed analysis of what RESONANCE cannot do
- RD-6.4 (Reproducibility Protocol): Commands to independently verify all results
- RD-6.5 (Reference Data Registry): External data source catalog with integrity assessment
- RD-1.1 (Intended Use Statement): Research-only scope, excluded users
- RD-1.2 (Software Safety Classification): IEC 62304 Class A

## 2. Product Description

RESONANCE is an emergent life simulation engine where all entities are energy (qe). Behavior emerges from 8 foundational axioms and 4 fundamental constants via 14 orthogonal ECS layers. Drug models (Level 1: cytotoxic; Level 2: pathway inhibitor) simulate therapeutic resistance dynamics using abstract frequency-based binding (Axiom 8) and Hill pharmacokinetics.

| Parameter | Value |
|-----------|-------|
| Codebase | 113K LOC, Rust 2024 / Bevy 0.15 |
| Tests | 3,113 automated (0 failures) |
| License | AGPL-3.0 |
| Paper | https://zenodo.org/records/19342036 |
| Commit | `971c7acb99decde45bf28860e6e10372718c51e2` |
| Safety class | IEC 62304 Class A |
| IMDRF category | Category I |

## 3. Analytical Evidence (Verification)

Analytical evidence demonstrates that RESONANCE correctly implements its stated computational model. This is the strongest evidence category for RESONANCE.

### 3.1 Automated Test Suite

The complete test suite was executed on the evaluation commit:

```
$ cargo test
...
test result: ok. 3113 passed; 0 failed; 1 ignored; 0 measured; 0 filtered out; finished in 35.78s
```

| Metric | Value |
|--------|-------|
| Tests passed | 3,113 |
| Tests failed | 0 |
| Tests ignored | 1 |
| Wall time | ~36 seconds |

Tests cover all critical subsystems:

| Subsystem | Approx. test count | Key files |
|-----------|-------------------|-----------|
| Blueprint equations (pure math) | ~600 | `src/blueprint/equations/*.rs` (45+ domain files) |
| Batch simulator (33 systems) | ~156 | `src/batch/*.rs` |
| Pathway inhibitor | 32+ | `src/blueprint/equations/pathway_inhibitor.rs` |
| Pathway inhibitor experiments | 18+ | `src/use_cases/experiments/pathway_inhibitor_exp.rs` |
| Derived thresholds | 17 | `src/blueprint/equations/derived_thresholds.rs` |
| Clinical calibration | 21 | `src/blueprint/equations/clinical_calibration.rs` |
| Determinism | 23 | `src/blueprint/equations/determinism.rs` |
| Conservation (property-based) | ~20 | `tests/property_conservation.rs` |
| Protein folding | 27 | `src/blueprint/equations/protein_fold.rs` |
| Coulomb/LJ potentials | 26 | `src/blueprint/equations/coulomb.rs` |
| Metabolic genome | 80 | `src/blueprint/equations/metabolic_genome.rs` |
| Variable genome | 62 | `src/blueprint/equations/variable_genome.rs` |
| Codon genome | 28 | `src/blueprint/equations/codon_genome.rs` |
| Multicellular | 27 | `src/blueprint/equations/multicellular.rs` |

### 3.2 Axiom Compliance Verification

Each of the 8 foundational axioms is verified by the test suite and/or the Rust type system:

| Axiom | Verification method | Evidence |
|-------|-------------------|----------|
| 1. Everything is Energy | Type system: all entities use `BaseEnergy { qe: f32 }` | `src/layers/energy.rs` — no alternative HP/mana/stat types exist |
| 2. Pool Invariant | Property-based fuzzing: `sum(children) <= parent` | `tests/property_conservation.rs` — proptest with arbitrary inputs, 0 violations |
| 3. Competition as Primitive | Tested via interference factor in energy transfer | `src/blueprint/equations/core_physics/` tests |
| 4. Dissipation (2nd Law) | 4 dissipation constants tested; all processes lose energy | `src/blueprint/equations/derived_thresholds.rs` — 17 tests |
| 5. Conservation | Total qe monotonically decreases — property-tested | `tests/property_conservation.rs` — `global_conservation_error` fuzzed |
| 6. Emergence at Scale | Meta-rule: no hardcoded behaviors exist in codebase | Architecture review; no trophic-class tags, no behavior scripts |
| 7. Distance Attenuation | Monotonic decrease tested | Spatial interaction tests in `src/blueprint/equations/` |
| 8. Oscillatory Nature | Frequency alignment via `gaussian_frequency_alignment` | `src/blueprint/equations/determinism.rs` — 23 tests |

### 3.3 Conservation Fuzz Testing

Property-based tests (`tests/property_conservation.rs`) use the `proptest` crate to generate arbitrary valid inputs and verify conservation invariants:

| Property | Generator | Result |
|----------|-----------|--------|
| `is_valid_qe` is true for all finite non-negative f32 | `qe_value()`: 0.0 or 0.001..1e6 | Pass (0 failures in arbitrary runs) |
| `has_invalid_values` detects NaN/Inf | `prop::num::f32::ANY` | Pass |
| `global_conservation_error` within tolerance | Multiple extraction strategies | Pass |
| `extract_*` functions never produce negative energy | All 5 extraction types | Pass |
| `pool_next_tick` maintains invariant | Arbitrary qe + rate + siblings | Pass |
| `scale_extractions_to_available` never exceeds budget | Arbitrary extraction vectors | Pass |

No conservation violations have been found in any proptest run. The fuzzer generates hundreds of thousands of test cases per execution.

### 3.4 Determinism Verification

RESONANCE uses hash-based deterministic RNG (`src/blueprint/equations/determinism.rs`) with no external randomness source. Functions:

| Function | Purpose | Tests |
|----------|---------|-------|
| `hash_f32_slice` | Bit-exact hash of f32 array (order-sensitive) | Tested |
| `snapshot_hash` | Hash of energy snapshot for determinism check | Tested |
| `snapshots_match` | Bit-exact comparison of two snapshots | Tested |
| `next_u64` | PCG-like deterministic state step | Tested |
| `unit_f32` | Uniform [0,1) from state (top 24 bits) | Tested |
| `range_f32` | Uniform [min, max) from state | Tested |
| `gaussian_f32` | Gaussian via Box-Muller (deterministic) | Tested |

Verification protocol: Run any experiment binary twice with the same parameters. Output must be byte-identical. See RD-6.4 for exact commands.

### 3.5 Analytical Evidence Assessment

| Criterion | Assessment |
|-----------|-----------|
| Computational correctness | **Strong.** 3,113 tests, 0 failures, property-based fuzzing with no violations. |
| Axiom compliance | **Strong.** All 8 axioms verified by tests and/or type system. |
| Conservation | **Strong.** Property-based fuzzing with arbitrary inputs, no violations found. |
| Determinism | **Strong.** Hash-based RNG, bit-exact reproducibility, 23 dedicated tests. |
| Code safety | **Strong.** Zero `unsafe` blocks, zero `unwrap` in systems (per `CLAUDE.md` Hard Blocks). |

**Conclusion:** Analytical evidence is sufficient to confirm that RESONANCE correctly implements its stated computational model.

## 4. Performance Evidence (Validation)

Performance evidence demonstrates that RESONANCE's output is consistent with established scientific predictions. This section presents quantitative results from all experiments.

### 4.1 Experiment 4: Pathway Inhibition Dose-Response

**Design:** Single-arm dose escalation using Level 2 pathway inhibitor in Competitive mode. Control (no drug) compared against two concentration levels.

**Method:** 100 worlds, 80 generations, 200 ticks/generation, 10 seeds. Metabolic efficiency ratio = mean alive efficiency at final generation / control efficiency.

**Results:**

| Condition | Concentration | Efficiency (mean) | Suppression | Monotonic |
|-----------|--------------|-------------------|-------------|-----------|
| Control | 0.0 | 1.000 | 0.0% | -- |
| Low dose | 0.4 | 0.488 | 51.2% | Yes (vs. control) |
| High dose | 0.8 | 0.471 | 52.9% | Yes (vs. low dose) |

**Assessment:** Dose-response is monotonic as predicted by Hill pharmacokinetics. Higher concentration produces greater suppression. Result is consistent across all 10 seeds. The narrow gap between 0.4 and 0.8 suggests diminishing returns at high occupancy, consistent with Hill saturation kinetics.

**Source:** `src/use_cases/experiments/pathway_inhibitor_exp.rs`

### 4.2 Experiment 5: Bozic 2013 Replication

**Design:** 5-arm parallel protocol replicating the structure of Bozic et al. 2013 (eLife 2:e00747).

**Method:** Two drugs targeting different frequencies (400 Hz and 300 Hz). Arms: no treatment, monotherapy A, monotherapy B, combination A+B, double-dose A (2x concentration). 100 worlds, 80 generations, 200 ticks/generation, 10 independent seeds.

**Results:**

| Arm | Efficiency | Suppression | 10/10 seeds confirm |
|-----|-----------|-------------|---------------------|
| no_drug | 1.000 | 0.0% | -- |
| mono_A (400 Hz) | 0.481 | 51.9% | Yes |
| mono_B (300 Hz) | 0.635 | 36.5% | Yes |
| combo_AB | 0.435 | 56.5% | Yes |
| double_A (2x conc) | 0.466 | 53.4% | Yes |

**Key ordering verified:**

| Prediction | Observed | Status |
|------------|----------|--------|
| combo_AB < mono_A | 0.435 < 0.481 | Confirmed |
| combo_AB < mono_B | 0.435 < 0.635 | Confirmed |
| combo_AB < double_A | 0.435 < 0.466 | Confirmed |
| combo suppresses more than doubling dose | 56.5% > 53.4% | Confirmed |

**Robustness:** 10/10 independent seeds confirm all orderings. The result is structural (derives from the orthogonality of frequency-based targeting), not stochastic.

**Comparison with Bozic 2013:**

| Aspect | Bozic 2013 | RESONANCE | Match |
|--------|-----------|-----------|-------|
| Combo > mono | Predicted (analytical model) | Confirmed (10/10 seeds) | Qualitative match |
| Combo > double dose | Predicted | Confirmed | Qualitative match |
| Mechanism | Explicit resistance mutation probability | Frequency-based metabolic compensation | Different |
| Output metric | Probability of resistance, cell count | Metabolic efficiency ratio | Different |
| Time scale | Weeks/months | Abstract generations | Not comparable |

**Assessment:** RESONANCE replicates the qualitative prediction of Bozic et al. 2013 using a fundamentally different mechanism (energy/frequency-based emergence vs. explicit mutation probability). This is consistent with the hypothesis that the combination advantage is a robust structural property, not an artifact of a specific model.

**Limitations:** (1) Suppression percentages, not absolute cell counts. (2) No time-to-resistance comparison. (3) RESONANCE uses frequency escape (metabolic compensation) rather than explicit point mutations. (4) Efficiency reduction is metabolic suppression, not cell death — these are not equivalent measures.

**Source:** `src/use_cases/experiments/pathway_inhibitor_exp.rs` (function `run_bozic_validation`), `src/bin/bozic_validation.rs`

### 4.3 Experiment 6: Adaptive Therapy Controller

**Design:** Closed-loop feedback controller that profiles the tumor, attacks with targeted drugs, predicts escape frequencies, and adapts drug selection.

**Method:** 20 worlds, 30 generations, 80 ticks/generation, 10 seeds. Controller adds/removes drugs based on growth rate trajectory.

**Results:**

| Metric | Value |
|--------|-------|
| Seeds achieving growth stabilization at zero | 7/10 |
| Seeds with partial stabilization | 2/10 |
| Seeds failing to stabilize | 1/10 |
| Typical stability generation | 8-15 |
| Wall time | ~8 seconds |

**Assessment:** The adaptive controller demonstrates that feedback-based drug modulation can stabilize tumor growth in a majority of cases (7/10). This is consistent with the adaptive therapy hypothesis (Gatenby et al. 2009) — maintaining drug-sensitive cells to compete with resistant cells delays resistance emergence.

**Limitations:** (1) 7/10 is not universal — 30% of seeds do not achieve full stabilization. (2) "Growth rate = 0" is an abstract metric; clinical adaptive therapy targets tumor volume stabilization by RECIST criteria. (3) No pharmacokinetic delay modeled. (4) Controller operates on perfect information (knows exact efficiency of each cell); real adaptive therapy relies on imperfect biomarkers (PSA, imaging).

**Source:** `src/use_cases/experiments/pathway_inhibitor_exp.rs` (function `run_adaptive`), `src/bin/adaptive_therapy.rs`

### 4.4 Experiment 7: Canine Mast Cell Tumor (Rosie Case)

**Design:** Simulation of heterogeneous tumor treatment with a single-target agent, calibrated against canine mast cell tumor parameters.

**Method:** 70% responsive (KIT+) / 30% resistant (KIT-) population, treatment with single-frequency pathway inhibitor (proxy for mRNA vaccine), 5 independent seeds, calibrated via `CANINE_MAST_CELL` profile.

**Results:**

| Observation | Simulation outcome | Clinical observation (press reports) |
|-------------|-------------------|-------------------------------------|
| Single-target suppression | 50-70% efficiency reduction | ~75% tumor volume reduction |
| Resistant fraction persists | Yes (KIT- subpopulation survives) | Yes (incomplete response reported) |
| Combination predicted better | Yes (simulation predicts combo advantage) | Not yet tested clinically |
| Time to partial response | ~2 generations (42 days at 21-day doubling) | ~6 weeks (reported) |

**Assessment:** The simulation produces qualitatively consistent results with the observed partial response. The responsive fraction is suppressed, the resistant fraction persists, and the timeline is roughly consistent when calibrated through the `CANINE_MAST_CELL` profile.

**Limitations (critical):** (1) The mRNA vaccine mechanism (immune-mediated killing) is fundamentally different from kinase inhibition modeled by RESONANCE. (2) Toceranib IC50 is used as a pharmacological proxy for a vaccine with no published IC50. (3) "Press reports" (Japan Times, Fortune, March 2026) are the source for observed outcomes, not peer-reviewed clinical data. (4) RESONANCE models metabolic suppression, not immune-mediated tumor clearance. (5) This is a simulation exercise, not veterinary validation.

**Source:** `src/blueprint/equations/clinical_calibration.rs`

### 4.5 Performance Evidence Assessment

| Experiment | Claim supported | Strength | Key limitation |
|-----------|----------------|----------|----------------|
| Exp 4: Dose-response | C-3 (monotonic dose-response) | Strong (10/10 seeds, consistent with Hill theory) | Narrow gap at high occupancy |
| Exp 5: Bozic 2013 | C-2 (combo > mono) | Strong (10/10 seeds, structural result) | Qualitative only; different mechanism |
| Exp 6: Adaptive therapy | C-4 (growth stabilization) | Moderate (7/10 seeds) | 30% failure rate; abstract metric |
| Exp 7: Rosie case | C-6 (partial response) | Weak (mechanism mismatch, press reports) | Fundamental mechanism difference |

## 5. Clinical Evidence (Contextualization)

### 5.1 Calibration Profile Verification

All 4 calibration profiles have been verified against published values:

| Profile | Parameter | Published value | RESONANCE value | Test |
|---------|-----------|----------------|-----------------|------|
| CML / imatinib | Doubling time | 4 days (Bozic 2013, Table 1) | 4.0 days | `cml_doubling_time_is_4_days` |
| CML / imatinib | IC50 | 260 nM (Druker et al. 2001) | 260.0 nM | `cml_imatinib_ic50_is_260nm` |
| Prostate / abiraterone | Doubling time | 30 days (Gatenby 2009) | 30.0 days | `prostate_doubling_time_is_30_days` |
| NSCLC / erlotinib | Doubling time | 7 days (typical EGFR-mutant) | 7.0 days | In test suite |
| NSCLC / erlotinib | IC50 | 20 nM (EGFR-mutant literature) | 20.0 nM | In test suite |
| Canine MCT / toceranib | Doubling time | 21 days (London & Seguin 2003) | 21.0 days | `mast_cell_doubling_21_days` |
| Canine MCT / toceranib | IC50 | 40 nM (London et al. 2009) | 40.0 nM | `mast_cell_toceranib_ic50_40nm` |

All calibration conversion functions have round-trip tests (e.g., `roundtrip_nm_concentration`). The calibration module has 21 passing tests.

### 5.2 Biological Scaling Validation

The Kleiber exponent (0.75) used in RESONANCE is supported by:

| Reference | Finding | Application in RESONANCE |
|-----------|---------|-------------------------|
| Kleiber M. 1947 | Metabolic rate proportional to mass^0.75 across species | `KLEIBER_EXPONENT = 0.75` in `derived_thresholds.rs` |
| West et al. 1997 (Science 276:122) | General model deriving 3/4-power law from fractal distribution networks | Justifies the exponent as a biological universal, not a calibration artifact |

The exponent governs basal metabolic drain (`basal_drain_rate`), matter state thresholds, and all derived lifecycle constants. Its use is well-grounded in established biology.

### 5.3 Clinical Evidence Assessment

| Criterion | Assessment |
|-----------|-----------|
| Calibration correctness | **Strong.** All parameters match published values; 21 unit tests pass. |
| Biological scaling foundation | **Strong.** Kleiber exponent validated across 27 orders of magnitude in published literature. |
| Clinical predictive validity | **Not established.** No comparison with patient-level outcomes. |
| Prospective predictions | **Not established.** All evidence is retrospective or near-retrospective. |

**Conclusion:** Clinical evidence is sufficient to confirm that calibration profiles accurately reflect published parameters. Clinical predictive validity is not claimed and not established.

## 6. Benefit-Risk Assessment

### 6.1 Benefits

| Benefit | Evidence | Beneficiary |
|---------|----------|-------------|
| Hypothesis generation for combination therapy strategies | Experiment 5: structural combo advantage in 10/10 seeds | Computational biology researchers |
| Exploration of adaptive therapy dynamics | Experiment 6: growth stabilization in 7/10 seeds | Mathematical oncology researchers |
| Dose-response behavior from first principles | Experiment 4: monotonic response consistent with Hill theory | Pharmacology researchers |
| Open-source, deterministic, reproducible | 3,113 tests, bit-exact RNG, AGPL-3.0 | All research users |
| Calibration framework for contextualizing abstract results | 4 profiles with 21 tests, published DOIs | Research communicators |

### 6.2 Risks

| Risk | Severity | Probability | Mitigation |
|------|----------|-------------|------------|
| Researcher overinterprets simulation as clinical evidence | Medium | Low (intended users are trained researchers) | Disclaimers in README, paper, code; RD-6.3 limitations report |
| Clinician uses output for treatment decisions | High | Very low (no clinical interface, no patient data input) | Intended use exclusion (RD-1.1); disclaimers; IMDRF Category I |
| Incorrect simulation output misleads research direction | Low | Low (3,113 tests; property-based fuzzing; deterministic) | Test suite; reproducibility protocol (RD-6.4) |
| Calibration profiles mistaken for clinical validation | Medium | Low | Explicit "NOT validated" disclaimers; RD-6.3 |
| Rosie case interpreted as veterinary guidance | Medium | Very low | In-code disclaimer: "NOT VETERINARY ADVICE" |

### 6.3 Benefit-Risk Determination

**For the intended use (research tool, IMDRF Category I):** The benefit-risk balance is **favorable**.

- Benefits are substantial for the intended user population (hypothesis generation, open-source reproducibility).
- Risks to patients are negligible when the tool is used within its intended use (no patient in the decision loop).
- Residual risks (misinterpretation) are mitigated by pervasive disclaimers and the intended-use restriction.

**For clinical use (hypothetical, not the intended use):** The benefit-risk balance would be **unfavorable**. Clinical use would require pharmacokinetic modeling, tumor microenvironment, immune system, and prospective validation against patient outcomes — none of which exist. See RD-1.1 §6 for reclassification analysis.

## 7. Claim-by-Claim Evaluation Summary

| Claim | Evidence type | Strength | Verdict |
|-------|-------------|----------|---------|
| C-1: Deterministic, conservation-correct simulation | Analytical | Strong | **Supported.** 3,113 tests, conservation fuzz, determinism verified. |
| C-2: Combo > mono (Bozic qualitative replication) | Performance | Strong | **Supported.** 10/10 seeds, structural result. Qualitative only. |
| C-3: Monotonic dose-response | Performance | Strong | **Supported.** 10/10 seeds, consistent with Hill theory. |
| C-4: Adaptive therapy stabilizes growth | Performance | Moderate | **Partially supported.** 7/10 seeds. Abstract metric. |
| C-5: Calibration contextualizes abstract output | Clinical | Strong | **Supported.** 21 tests, published DOIs, correct conversions. |
| C-6: Rosie case partial response | Performance + Clinical | Weak | **Weakly supported.** Mechanism mismatch, press report source. |

## 8. Evidence Gaps and Residual Limitations

### 8.1 Gaps Identified

| Gap ID | Description | Impact | Severity | Resolution path |
|--------|-------------|--------|----------|----------------|
| G-1 | No patient-level outcome validation | Cannot claim clinical predictive validity | High | Collaboration with clinical trial teams |
| G-2 | No prospective predictions validated | Cannot claim predictive power | Medium | Pre-register predictions, validate against future data |
| G-3 | No independent replication | Cannot rule out systematic bias | Medium | Open-source; reproducibility protocol published |
| G-4 | No ADME pharmacokinetics | Cannot model drug exposure dynamics | Medium | Architecture supports extension; not planned |
| G-5 | No tumor microenvironment | Results may not hold when TME dominates | Medium | Architecture supports extension; not planned |
| G-6 | No immune system | Cannot model immune-mediated responses | High (for Rosie case) | Documented as future work in paper |
| G-7 | Efficiency reduction is not cell death | Metabolic suppression is not equivalent to tumor volume reduction | Medium | Explicitly documented in all outputs |

### 8.2 Accepted Limitations

The following limitations are inherent to the RESONANCE model and are accepted as design constraints for the research-tool use case:

- Abstract energy units (qe) are not biophysical measurements
- Frequency is a computational proxy, not a biological observable
- Hill coefficient fixed at n=2 for all drugs (real drugs vary)
- No spatial tumor structure (homogeneous population model in drug experiments)
- Drug concentration is static (no time decay, no half-life)
- Population entities represent aggregate cells, not individual cells

All limitations are documented in detail in RD-6.3 (Limitations and Scope Report).

## 9. Conclusion

### 9.1 Summary

RESONANCE demonstrates strong analytical evidence (3,113 tests, conservation fuzz, determinism), moderate-to-strong performance evidence (Bozic replication in 10/10 seeds, monotonic dose-response, adaptive therapy in 7/10 seeds), and adequate clinical contextualization (4 calibration profiles with published DOIs and 21 tests).

### 9.2 Determination

**RESONANCE is suitable for its intended use as a research tool for exploring emergent therapeutic resistance dynamics.**

It is **not suitable** for:

- Clinical decision-making
- Patient diagnosis or treatment selection
- Drug dosing recommendations
- Regulatory submission as clinical evidence (without additional validation)
- Veterinary treatment guidance

### 9.3 Recommendations

1. Maintain the research-only intended use (IMDRF Category I).
2. Continue to strengthen disclaimers in all public-facing materials.
3. Publish the reproducibility protocol (RD-6.4) alongside the codebase to enable independent replication.
4. If clinical application is ever pursued, conduct prospective validation against patient-level outcome data and reclassify per IMDRF SaMD N10.
5. Monitor for misuse (e.g., citation in clinical contexts without appropriate caveats).

## 10. Codebase References

All quantitative claims in this report are traceable to the RESONANCE codebase at commit `971c7acb99decde45bf28860e6e10372718c51e2`. Verification commands are provided in RD-6.4 (Reproducibility Protocol).

| Reference | File | Verification |
|-----------|------|--------------|
| Test suite | All `src/**/*.rs` test modules | `cargo test` |
| Conservation fuzz | `tests/property_conservation.rs` | `cargo test property_conservation` |
| Determinism | `src/blueprint/equations/determinism.rs` | `cargo test determinism` |
| Pathway inhibitor math | `src/blueprint/equations/pathway_inhibitor.rs` | `cargo test pathway_inhibitor` |
| Bozic validation | `src/bin/bozic_validation.rs` | `cargo run --release --bin bozic_validation` |
| Adaptive therapy | `src/bin/adaptive_therapy.rs` | `cargo run --release --bin adaptive_therapy` |
| Clinical calibration | `src/blueprint/equations/clinical_calibration.rs` | `cargo test clinical_calibration` |
| Derived thresholds | `src/blueprint/equations/derived_thresholds.rs` | `cargo test derived_thresholds` |
| Experiment harness | `src/use_cases/experiments/pathway_inhibitor_exp.rs` | `cargo test pathway_inhibitor_exp` |

## 11. Revision History

| Version | Date | Author | Change Description |
|---------|------|--------|--------------------|
| 1.0 | 2026-04-02 | Resonance Development Team | Initial clinical evaluation report. Analytical evidence strong, performance evidence moderate-to-strong, clinical evidence adequate for research use. Benefit-risk favorable for intended use. 7 evidence gaps identified. All claims traced to commit `971c7ac`. |

---

**Approvals:**

| Role | Name | Date | Signature |
|------|------|------|-----------|
| Author | Resonance Development Team | 2026-04-02 | _draft -- not signed_ |
| Reviewer | _pending_ | _pending_ | _pending_ |
| Approver | _pending_ | _pending_ | _pending_ |
