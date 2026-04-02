---
document_id: RD-4.4
title: Model Validation Report
standard: ASME V&V 40:2018 Section 6
version: 1.0
date: 2026-04-02
status: DRAFT
author: Resonance Development Team
approved_by: PENDING
review_date: PENDING
review_status: PENDING
---

# Model Validation Report

## 1. Purpose

This document reports the results of validation activities for the RESONANCE computational model, following ASME V&V 40:2018 Section 6. It documents each experiment's protocol, parameters, quantitative results, statistical robustness, comparison to published data, and known gaps. All validation evidence is drawn from the codebase at commit `971c7acb99decde45bf28860e6e10372718c51e2`.

Validation here means: comparing model output to independent reference data (published literature or real-world observations) to determine if the model adequately represents the real-world phenomena for its stated Context of Use.

**Context of Use (from RD-4.2 Section 4):** Inform research on therapeutic resistance dynamics. Low decision consequence. Output is qualitative hypotheses, not clinical predictions.

Related documents:

- **RD-4.1** VALIDATION_PLAN.md -- acceptance criteria
- **RD-4.2** CREDIBILITY_MODEL.md -- credibility framework
- **RD-4.3** VERIFICATION_REPORT.md -- code verification results
- **RD-4.5** UNCERTAINTY_ANALYSIS.md -- uncertainty quantification

## 2. Validation Experiments Overview

| Experiment | Reference | Key Prediction | Result | Seeds |
|------------|-----------|---------------|--------|-------|
| Exp 1-3 | Zenodo paper | Basic emergence, conservation, interference | Published | N/A |
| Exp 4 | Internal | Monotonic dose-response | PASS | 10 |
| Exp 5 | Bozic et al. 2013 | Combo > mono suppression | PASS (10/10) | 10 |
| Exp 6 | Gatenby et al. 2009 | Adaptive therapy stabilizes growth | PASS (7/10) | 10 |
| Exp 7 | London 2003, 2009 | Partial response in canine MCT | PASS (5/5) | 5 |

---

## 3. Experiments 1-3: Published in Zenodo Paper

### 3.1 Summary

Experiments 1-3 are documented in the published paper (Zenodo DOI: 10.5281/zenodo.19342036, "Emergent Life from Four Constants: An Axiomatic Simulation Engine"). They cover:

- **Exp 1:** Basic emergence -- entities self-organize from energy field without templates.
- **Exp 2:** Energy conservation -- total system energy monotonically decreases (Axiom 5).
- **Exp 3:** Frequency interference -- interaction strength modulated by frequency alignment (Axiom 8).

These experiments validate the core physics engine. Results are fully documented in the paper and are not repeated here.

**File:** `docs/paper/resonance_arxiv.tex`

---

## 4. Experiment 4: Pathway Inhibition Dose-Response

### 4.1 Objective

Verify that RESONANCE produces a monotonic dose-response curve when drug concentration increases, without explicit programming of dose-response behavior.

### 4.2 Protocol

| Parameter | Value |
|-----------|-------|
| Population | 40 wildtype (freq=400 Hz) + 5 resistant (freq=250 Hz) |
| Drug | Competitive inhibitor targeting Root metabolic node |
| Drug frequency | 400 Hz (matches wildtype) |
| Drug Ki | 1.0 (DISSIPATION_SOLID * 200) |
| Concentration sweep | 0.0, 0.1, 0.2, 0.3, 0.4, 0.5, 0.6, 0.7, 0.8, 0.9, 1.0 |
| Worlds | 100 per concentration step |
| Generations | 80 |
| Ticks per generation | 200 |
| Seeds | 10 independent seeds (42, 43, ..., 51) |
| Treatment start | Generation 5 |
| Nutrient level | 5.0 (scarce) |

### 4.3 Acceptance Criterion

For each seed: efficiency(conc_i) >= efficiency(conc_{i+1}) for all i in [0, 9]. Strict monotonicity across the full concentration sweep.

### 4.4 Results

| Concentration | Avg Efficiency (seed=42) | Suppression |
|--------------|------------------------|-------------|
| 0.0 | 1.000 | 0.0% |
| 0.1 | 0.942 | 5.8% |
| 0.2 | 0.876 | 12.4% |
| 0.3 | 0.801 | 19.9% |
| 0.4 | 0.724 | 27.6% |
| 0.5 | 0.648 | 35.2% |
| 0.6 | 0.576 | 42.4% |
| 0.7 | 0.512 | 48.8% |
| 0.8 | 0.455 | 54.5% |
| 0.9 | 0.408 | 59.2% |
| 1.0 | 0.371 | 62.9% |

**Monotonicity:** Strictly monotonic in all 10 seeds. 10/10 PASS.

**Mechanism:** The dose-response emerges from Hill pharmacokinetics (`hill_response(conc * affinity, Ki, n=2)`) interacting with frequency-based binding (`gaussian_frequency_alignment`). Higher concentration -> higher occupancy -> more efficiency reduction. The curve is sigmoid (characteristic of Hill n=2), not linear.

### 4.5 Known Gaps

- No external reference data for dose-response shape (internal validation only)
- Drug concentration is static per experiment (no PK time-varying decay)
- Absolute efficiency values are in abstract units (qe-based), not biologically calibrated
- The sigmoid shape is expected from the Hill equation -- this validates implementation, not discovery

### 4.6 File References

- Experiment harness: `src/use_cases/experiments/pathway_inhibitor_exp.rs`
- Hill response function: `src/blueprint/equations/pathway_inhibitor.rs` (`hill_response`)
- Binding affinity: `src/blueprint/equations/determinism.rs` (`gaussian_frequency_alignment`)
- Constants: `src/blueprint/constants/pathway_inhibitor.rs`

---

## 5. Experiment 5: Bozic 2013 Replication (Primary Validation)

### 5.1 Objective

Reproduce the qualitative prediction from Bozic et al. 2013 (eLife 2:e00747) that combination therapy has exponential advantage over monotherapy in delaying drug resistance in heterogeneous tumor populations.

### 5.2 Published Reference

**Citation:** Bozic I, Reiter JG, Allen B, Antal T, Chatterjee K, Shah P, Moon YS, Yaqubie A, Kelly N, Le DT, Lipson EJ, Chapman PB, Diaz LA Jr, Vogelstein B, Nowak MA (2013). Evolutionary dynamics of cancer in response to targeted combination therapy. eLife 2:e00747.

**Key prediction:** For tumors with mutation rate ~10^-9 per gene per division, the probability of pre-existing resistance to a combination of two independent drugs is the product of individual resistance probabilities. This yields an exponential (multiplicative) advantage for combination therapy.

**RESONANCE translation:** The model does not simulate mutation rate or pre-existing resistance directly. Instead, it uses a frequency-heterogeneous population where resistant clones (freq=250 Hz) are pre-seeded alongside wildtype (freq=400 Hz). Drugs target specific frequencies. The prediction tested is: combination therapy (two drugs at different frequencies) suppresses more than monotherapy or double-dose monotherapy.

### 5.3 Protocol

| Parameter | Value |
|-----------|-------|
| Population | 40 wildtype (freq=400 Hz, qe=80) + 5 resistant (freq=250 Hz, qe=80) |
| Drug A | Competitive, freq=400 Hz, conc=0.8, Ki=1.0, target=Root |
| Drug B | Competitive, freq=350 Hz, conc=0.8, Ki=1.0, target=Root |
| Arms | no_drug, mono_A, mono_B, combo_AB, double_A |
| double_A | Drug A at conc=1.6 (double dose, same frequency) |
| Worlds | 100 |
| Generations | 80 |
| Ticks per generation | 200 |
| Treatment start | Generation 5 |
| Nutrient level | 5.0 |
| Seeds | 10 independent seeds (42-51) |

### 5.4 Results

#### 5.4.1 Primary Results (Seed = 42)

| Arm | Final Efficiency | Suppression vs Baseline | Rank |
|-----|-----------------|------------------------|------|
| no_drug | 1.000 | 0.0% | 5 (worst) |
| mono_A | 0.481 | 51.9% | 3 |
| mono_B | 0.635 | 36.5% | 4 |
| combo_AB | 0.435 | 56.5% | 1 (best) |
| double_A | 0.466 | 53.4% | 2 |

**Key comparisons:**

| Comparison | Result | Bozic Prediction |
|------------|--------|-----------------|
| combo_AB < mono_A | 0.435 < 0.481 | YES -- combo outperforms best monotherapy |
| combo_AB < mono_B | 0.435 < 0.635 | YES -- combo outperforms weaker monotherapy |
| combo_AB < double_A | 0.435 < 0.466 | YES -- combo outperforms double dose of single drug |

#### 5.4.2 Multi-Seed Robustness

| Seed | combo_AB | mono_A | combo < mono_A? | double_A | combo < double? |
|------|----------|--------|----------------|----------|----------------|
| 42 | 0.435 | 0.481 | YES | 0.466 | YES |
| 43 | 0.441 | 0.479 | YES | 0.462 | YES |
| 44 | 0.438 | 0.483 | YES | 0.468 | YES |
| 45 | 0.432 | 0.477 | YES | 0.460 | YES |
| 46 | 0.440 | 0.485 | YES | 0.470 | YES |
| 47 | 0.436 | 0.480 | YES | 0.464 | YES |
| 48 | 0.433 | 0.478 | YES | 0.461 | YES |
| 49 | 0.439 | 0.482 | YES | 0.467 | YES |
| 50 | 0.437 | 0.484 | YES | 0.469 | YES |
| 51 | 0.434 | 0.476 | YES | 0.458 | YES |

**Score: 10/10 seeds confirm both predictions.** Threshold: >= 8/10 (80%). Result: 100%.

#### 5.4.3 Statistical Summary

| Metric | combo_AB | mono_A | double_A |
|--------|----------|--------|----------|
| Mean efficiency | 0.4365 | 0.4805 | 0.4645 |
| Std dev | 0.003 | 0.003 | 0.004 |
| CV (coefficient of variation) | 0.7% | 0.6% | 0.9% |
| Min | 0.432 | 0.476 | 0.458 |
| Max | 0.441 | 0.485 | 0.470 |

**Observation:** Variance is very low across seeds (CV < 1%). The result is structural (determined by population composition and drug parameters), not stochastic. This high reproducibility is consistent with 100-world averaging within each seed.

### 5.5 Interpretation

RESONANCE reproduces the Bozic 2013 qualitative prediction: combination therapy (two drugs at different frequencies) suppresses a heterogeneous population more than monotherapy or double-dose monotherapy. The mechanism in RESONANCE is:

1. **mono_A:** Strongly suppresses wildtype (freq=400, near drug_A freq=400) but leaves resistant (freq=250) unaffected.
2. **mono_B:** Moderately suppresses wildtype (freq=400, moderately near drug_B freq=350) but leaves resistant (freq=250) partially unaffected.
3. **combo_AB:** Suppresses wildtype via drug_A AND provides partial coverage of resistant via drug_B (freq=350 is closer to 250 than drug_A's 400).
4. **double_A:** Doubles the dose of drug_A (conc=1.6) but still cannot reach resistant clones at freq=250 -- diminishing returns on a single frequency target.

This is qualitatively analogous to Bozic's argument: two drugs targeting independent resistance mechanisms have multiplicative advantage because the probability of dual resistance is the product of individual resistance probabilities.

### 5.6 Known Gaps

| Gap | Description | Impact |
|-----|-------------|--------|
| Qualitative, not quantitative | Suppression percentages (56.5% vs published resistance probability calculations) are not directly comparable | Cannot validate specific numerical predictions |
| Abstract frequency vs genetic resistance | RESONANCE uses frequency distance as proxy for resistance; Bozic models point mutations | Mechanism is analogous, not identical |
| Static drug concentration | Bozic's model considers pharmacokinetics; RESONANCE uses constant concentration | Overestimates drug efficacy (no drug decay) |
| Small population | 45 entities vs ~10^9 cells in Bozic's model | Stochastic effects amplified; resistant fraction (5/45 = 11%) higher than realistic (~10^-4) |
| No TME | Bozic considers spatial heterogeneity; RESONANCE uses homogeneous field | Spatial refugia not captured |
| Pre-seeded resistance | Bozic models spontaneous mutation to resistance; RESONANCE pre-seeds resistant clones | Removes the stochastic mutation component entirely |

### 5.7 File References

- 5-arm experiment: `src/use_cases/experiments/pathway_inhibitor_exp.rs`
- Bozic validation binary: `src/bin/bozic_validation.rs`
- Pathway inhibitor equations: `src/blueprint/equations/pathway_inhibitor.rs`
- Bliss independence: `src/blueprint/equations/pathway_inhibitor.rs` (`bliss_independence`)

---

## 6. Experiment 6: Adaptive Therapy Controller

### 6.1 Objective

Demonstrate that an adaptive therapy strategy (modulating drug dose based on tumor growth rate) can stabilize tumor populations longer than continuous maximum-dose therapy, consistent with the conceptual framework of Gatenby et al. 2009.

### 6.2 Published Reference

**Citation:** Gatenby RA, Silva AS, Gillies RJ, Frieden BR (2009). Adaptive therapy. Cancer Research 69(11):4894-4903.

**Key insight:** Continuous maximum-dose therapy selects for resistant clones by eliminating sensitive competitors. Adaptive therapy maintains a population of drug-sensitive cells that compete with resistant cells, delaying resistance emergence.

### 6.3 Protocol

| Parameter | Value |
|-----------|-------|
| Population | 40 wildtype (freq=400 Hz) + 5 resistant (freq=250 Hz) |
| Continuous arm | Drug A at conc=0.8, constant for all generations |
| Adaptive arm | Drug A, concentration modulated by proportional controller: increase when growth rate > 0, decrease when growth rate < 0 |
| Controller | `conc_next = clamp(conc_current + gain * growth_rate, 0.0, 1.0)`, gain = 0.5 |
| Worlds | 100 |
| Generations | 100 |
| Ticks per generation | 200 |
| Treatment start | Generation 5 |
| Seeds | 10 independent seeds |
| Success criterion | Growth rate <= 0 by generation 40 (tumor stabilized) |

### 6.4 Results

| Seed | Adaptive: growth stabilized? | Continuous: growth controlled? |
|------|---------------------------|-------------------------------|
| 42 | YES (gen 35) | NO (escapes gen 52) |
| 43 | YES (gen 38) | YES (gen 45, borderline) |
| 44 | YES (gen 32) | NO (escapes gen 48) |
| 45 | NO (oscillates) | NO (escapes gen 44) |
| 46 | YES (gen 36) | NO (escapes gen 55) |
| 47 | YES (gen 40) | YES (gen 60, borderline) |
| 48 | NO (oscillates) | NO (escapes gen 50) |
| 49 | YES (gen 34) | NO (escapes gen 46) |
| 50 | YES (gen 37) | YES (gen 55, borderline) |
| 51 | NO (oscillates) | NO (escapes gen 42) |

**Adaptive stabilization: 7/10 seeds.** Threshold: >= 7/10 (70%). PASS.

**Continuous control: 3/10 seeds** (and those 3 are borderline -- control is lost later).

**Interpretation:** Adaptive therapy achieves growth stabilization in 70% of seeds by maintaining competitive pressure from sensitive cells. Continuous therapy eliminates sensitive cells early, allowing resistant clones to expand unopposed. The 3 adaptive failures show oscillatory behavior (controller gain too aggressive for those initial conditions).

### 6.5 Known Gaps

| Gap | Description |
|-----|-------------|
| Conceptual comparison only | Gatenby 2009 describes a specific adaptive protocol with PSA-based switching; RESONANCE uses a simple proportional controller |
| No PK/PD | Drug effect is instantaneous; no absorption/clearance dynamics |
| No tumor volume metric | Growth rate is entity count, not RECIST-measurable tumor volume |
| Controller not optimized | Proportional gain = 0.5 is a fixed choice; optimal gain depends on population dynamics |
| 30% failure rate | 3/10 seeds oscillate without stabilizing -- reveals sensitivity to initial conditions |

### 6.6 File References

- Adaptive controller: implemented in experiment harness (use case module)
- Drug model: `src/blueprint/equations/pathway_inhibitor.rs`
- Batch stepping: `src/blueprint/equations/batch_stepping.rs`

---

## 7. Experiment 7: Rosie Case (Canine Mast Cell Tumor)

### 7.1 Objective

Simulate a canine mast cell tumor case calibrated to published veterinary oncology data (London & Seguin 2003, London et al. 2009) and assess whether the model qualitatively reproduces the observed partial response.

### 7.2 Published References

**London CA, Seguin B (2003).** Mast cell tumors in the dog. Vet Clin North Am Small Anim Pract 33(3):473-489.
- Intermediate-grade canine MCT doubling time: ~21 days.

**London CA, Malpas PB, Wood-Follis SL, et al. (2009).** Multi-center, placebo-controlled, double-blind, randomized study of oral toceranib phosphate (Palladia), a receptor tyrosine kinase inhibitor, for the treatment of dogs with recurrent (either local or distant) mast cell tumor following surgical excision. Clin Cancer Res 15(11):3856-3865.
- Toceranib IC50 for KIT-mutant mast cell: ~40 nM.
- Overall response rate: 42.8% (partial + complete response).

### 7.3 Real-World Case

A canine mast cell tumor case ("Rosie") treated with an experimental mRNA vaccine. Reported in press (Japan Times, Fortune, March 2026). Outcome: partial response (tumor shrinkage but not complete elimination). Press reports, not peer-reviewed clinical data.

### 7.4 Protocol

| Parameter | Value | Source |
|-----------|-------|--------|
| Calibration profile | `CANINE_MAST_CELL` | `clinical_calibration.rs` |
| Days per generation | 21 | London & Seguin 2003 |
| IC50 (toceranib proxy) | 40 nM | London et al. 2009 |
| Cells per entity | ~781,000 (10^8 / 128) | Estimated from tumor size |
| Mutation rate | 3 x 10^-9 | Canine somatic rate |
| Population | 128 entities (wildtype at KIT-mutant frequency) |
| Drug | Competitive inhibitor, freq matching KIT-mutant band |
| Concentration | 0.8 (normalized) |
| Generations | 100 |
| Ticks per generation | 200 |
| Seeds | 5 |
| Success criterion | Population reduction >= 30% from peak by generation 60 |

### 7.5 Calibration Disclaimer

The toceranib IC50 (40 nM) is used as a pharmacological proxy for the mRNA vaccine potency. This is a fundamental mechanism mismatch:

- **Toceranib:** Small molecule kinase inhibitor (blocks KIT receptor signaling).
- **mRNA vaccine:** Immune-mediated killing (trains immune system to recognize tumor antigens).

The model has no immune system. The mRNA vaccine's mechanism (antigen presentation, T-cell activation, antibody-dependent cytotoxicity) is abstracted as frequency-selective energy drain. This is a gross simplification suitable only for qualitative hypothesis exploration.

**This simulation is NOT veterinary advice.** (Explicit disclaimer in `pathway_inhibitor_exp.rs` line 1295.)

### 7.6 Results

| Seed | Peak Population | Pop at Gen 60 | Reduction | Partial Response? |
|------|----------------|--------------|-----------|-------------------|
| 42 | 128 | 78 | 39.1% | YES |
| 43 | 128 | 72 | 43.8% | YES |
| 44 | 128 | 82 | 35.9% | YES |
| 45 | 128 | 75 | 41.4% | YES |
| 46 | 128 | 80 | 37.5% | YES |

**Score: 5/5 seeds achieve partial response (>= 30% reduction).** Threshold: >= 4/5. PASS.

**Average reduction:** 39.5% (range: 35.9% - 43.8%).

### 7.7 Comparison to Real-World Observation

| Metric | RESONANCE | Real World (London 2009) | Real World (Rosie) |
|--------|-----------|------------------------|-------------------|
| Response category | Partial response | 42.8% ORR (PR+CR) | Partial response |
| Reduction magnitude | 35-44% | 30-100% (PR threshold: 30%) | Unknown (press report) |
| Mechanism | Frequency-selective energy drain | Kinase inhibition (toceranib) / Immune (vaccine) | mRNA vaccine |
| Time to response | ~60 generations (~1,260 days calibrated) | Weeks to months | Weeks |

**Qualitative consistency:** The model produces a partial response consistent with the observed outcome. The magnitude (35-44%) falls within the biologically plausible range for MCT treatment. However, the calibrated time to response (~1,260 days) is much longer than the real-world response time (weeks), indicating that the time calibration is not accurate for this case. This may reflect the use of tumor doubling time (21 days) as the generation length, which conflates growth dynamics with treatment response dynamics.

### 7.8 Known Gaps

| Gap | Description | Impact |
|-----|-------------|--------|
| n=1 | Single real-world case, not a clinical trial | Cannot draw statistical conclusions |
| Mechanism mismatch | Toceranib IC50 proxy for mRNA vaccine | Drug model does not capture immune-mediated killing |
| Press reports | Rosie case details from press, not peer-reviewed | Uncertain factual basis |
| No immune model | RESONANCE has no immune system layers | Cannot model T-cell, NK-cell, or antibody responses |
| Time calibration inaccurate | Calibrated response time (~1,260 days) >> real response time (weeks) | Time axis is unreliable |
| Homogeneous population | No spatial heterogeneity | Cannot model tumor microenvironment effects on drug delivery |
| KIT mutation prevalence | ~30% of canine MCT (London 2009) | Model assumes 100% KIT-mutant |

### 7.9 File References

- Calibration profile: `src/blueprint/equations/clinical_calibration.rs` (lines 89-108, `CANINE_MAST_CELL`)
- Experiment harness: `src/use_cases/experiments/pathway_inhibitor_exp.rs`
- Paper discussion: `docs/paper/resonance_arxiv.tex`
- Rosie case commit: `1e795c2` (feat), `971c7ac` (limitations honesty pass)

---

## 8. Cross-Experiment Summary

### 8.1 Acceptance Criteria Status

| Criterion | Threshold | Result | Status |
|-----------|-----------|--------|--------|
| Bozic combo > mono | >= 8/10 seeds | 10/10 | PASS |
| Bozic combo > double_dose | >= 8/10 seeds | 10/10 | PASS |
| Dose-response monotonicity | 10/10 seeds | 10/10 | PASS |
| Adaptive growth stabilization | >= 7/10 seeds | 7/10 | PASS |
| Rosie partial response | >= 4/5 seeds | 5/5 | PASS |

### 8.2 Validation Evidence Strength

| Evidence Type | ASME V&V 40 Level | RESONANCE Status |
|---------------|-------------------|-----------------|
| Comparison to published data (Bozic) | Quantitative comparison | Qualitative agreement -- suppression direction, not magnitude |
| Comparison to clinical strategy (Gatenby) | Conceptual consistency | Strategy class comparison only |
| Comparison to pharmacological data (London) | Calibration | IC50 and doubling time used; not validated against patient trajectories |
| Real-world case (Rosie) | Case study | n=1, mechanism proxy, press report source |

### 8.3 Overall Validation Assessment

The validation evidence is **adequate for the stated Context of Use** (inform research, low decision consequence). The model:

- Reproduces the qualitative direction of Bozic's prediction with high confidence (10/10 seeds).
- Demonstrates biologically plausible dose-response behavior.
- Shows conceptual consistency with adaptive therapy strategies.
- Achieves qualitative agreement with a real-world case observation.

The validation evidence is **inadequate for**:

- Quantitative prediction of resistance timelines.
- Patient-specific treatment selection.
- Regulatory submission evidence (without substantial additional validation).
- Veterinary clinical guidance.

---

## 9. Limitations Honest Assessment

### 9.1 Fundamental Limitations

1. **Abstract energy units (qe):** All predictions are in abstract units. Calibration to clinical units introduces unconstrained degrees of freedom. The mapping is illustrative, not validated.

2. **No molecular targets:** RESONANCE uses frequency as a proxy for genetic identity. It cannot model specific resistance mutations (T315I for imatinib, T790M for erlotinib, C481S for ibrutinib). Frequency distance is an analogy, not a biophysical measurement.

3. **No pharmacokinetics (PK/PD):** Drug concentration is static per experiment. Real drugs have time-varying concentration governed by absorption, distribution, metabolism, and excretion. This means RESONANCE overestimates sustained drug efficacy and cannot model dosing schedules.

4. **No tumor microenvironment:** The model operates on a homogeneous spatial field. Real tumors have vasculature (limiting drug delivery), hypoxia (creating resistant niches), immune infiltration (modulating response), and extracellular matrix (affecting cell migration).

5. **Small population size:** 45-128 simulated entities vs ~10^9 real tumor cells. Stochastic effects are amplified. Rare events (mutation to resistance) are overrepresented. Population dynamics differ qualitatively at 10^9 vs 10^2 scale.

### 9.2 Comparison-Specific Limitations

6. **Bozic comparison is qualitative.** Suppression percentages are not calibrated to absolute cell counts or time-to-resistance in weeks. The comparison validates direction (combo > mono), not magnitude.

7. **Gatenby comparison is conceptual.** RESONANCE's proportional controller is not the specific adaptive protocol described in Gatenby 2009. The comparison validates the strategy class (modulated vs continuous), not the specific protocol.

8. **Rosie case uses proxy IC50.** Toceranib (kinase inhibitor) IC50 used as proxy for mRNA vaccine potency. Fundamentally different mechanisms. Single case (n=1). Source is press reports.

9. **Calibration profiles are qualitative fits.** 4 profiles with 3 free parameters each. No formal goodness-of-fit metric. No confidence intervals on calibrated predictions.

10. **Not validated against patient-level outcomes.** README.md line 141: "Against patient outcomes: Not yet." This is the single most important limitation.

---

## 10. Codebase References

| Reference | File Path |
|-----------|-----------|
| Pathway inhibitor experiment | `src/use_cases/experiments/pathway_inhibitor_exp.rs` |
| Bozic validation binary | `src/bin/bozic_validation.rs` |
| Pathway inhibitor equations | `src/blueprint/equations/pathway_inhibitor.rs` |
| Pathway inhibitor constants | `src/blueprint/constants/pathway_inhibitor.rs` |
| Clinical calibration profiles | `src/blueprint/equations/clinical_calibration.rs` |
| Derived thresholds | `src/blueprint/equations/derived_thresholds.rs` |
| Determinism | `src/blueprint/equations/determinism.rs` |
| Batch simulator | `src/batch/` |
| Zenodo paper | https://zenodo.org/records/19342036 |
| Rosie case commit (feat) | `1e795c2` |
| Rosie case commit (honesty pass) | `971c7ac` |
| README disclaimers | `README.md` lines 18-22, 141 |

## 11. Revision History

| Version | Date | Author | Changes |
|---------|------|--------|---------|
| 1.0 | 2026-04-02 | Resonance Development Team | Initial validation report. 4 experiments documented with protocol, results, comparison, and gaps. Honest assessment of all limitations. |

---

**Approvals:**

| Role | Name | Date | Signature |
|------|------|------|-----------|
| Author | Resonance Development Team | 2026-04-02 | _draft -- not signed_ |
| Reviewer | _pending_ | _pending_ | _pending_ |
| Approver | _pending_ | _pending_ | _pending_ |
