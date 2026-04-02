---
document_id: RD-2.2
title: Risk Analysis
standard: ISO 14971:2019 §5.3-5.4
version: 1.0
date: 2026-04-02
status: DRAFT
author: Resonance Development Team
---

# Risk Analysis

## 1. Purpose

This document performs systematic hazard identification and risk estimation for RESONANCE, satisfying ISO 14971:2019 §5.3 (Hazard identification) and §5.4 (Risk estimation). Each hazard is analyzed using Software FMEA as defined in the Risk Management Plan (RD-2.1).

**Cross-references:**
- RD-2.1 (Risk Management Plan): Defines scope, methods, probability/severity scales, and acceptability criteria
- RD-1.1 (Intended Use Statement): Defines intended use envelope and excluded contexts
- RD-1.2 (Software Safety Classification): IEC 62304 Class A determination

## 2. Hazard Identification Method

Hazards were identified through:

1. **Intended use deviation analysis** — What happens when RESONANCE output is used outside the research-only envelope defined in RD-1.1
2. **Software failure mode analysis** — What defects in the simulation engine could produce incorrect output
3. **SOUP vulnerability analysis** — What failures in third-party dependencies could affect RESONANCE
4. **Information deficit analysis** — What gaps in documentation, labeling, or output formatting could lead to misunderstanding
5. **Model limitation analysis** — What known simplifications in the simulation model could produce results that diverge from biological reality

All probability and severity ratings reference specific codebase evidence. No rating is assigned without justification.

## 3. Hazard Register (Software FMEA)

### 3.1 FMEA Table

| ID | Hazard | Potential Harm | Cause | P | S | Risk Level |
|----|--------|---------------|-------|---|---|------------|
| H-01 | Overreliance on resistance predictions | Researcher designs wet-lab experiment based on incorrect simulation prediction, wasting resources; in worst case (misuse), clinician selects suboptimal therapy | User treats qualitative simulation output (% suppression) as quantitative clinical prediction | P3 | S5 | ALARP |
| H-02 | Energy conservation bug | Incorrect simulation results — entities gain or lose energy without physical justification, producing biologically meaningless dynamics | Software defect in energy transfer, dissipation, or pool accounting that violates Axiom 2 (Pool Invariant) or Axiom 5 (Conservation) | P1 | S4 | Acceptable |
| H-03 | Calibration bias (only 4 profiles) | Overfitted model — simulation appears validated but generalizes poorly to untested cancer types, drug mechanisms, or species | Only 4 clinical calibration profiles exist (CML, prostate, NSCLC, canine MCT); no negative controls; all profiles calibrated to published data that may itself contain bias | P4 | S3 | ALARP |
| H-04 | Determinism broken | Non-reproducible simulation results — same input parameters produce different outputs across runs, undermining scientific reproducibility | Platform-dependent floating point behavior, uninitialized memory, thread-scheduling nondeterminism in rayon, or hash collision in RNG | P1 | S4 | Acceptable |
| H-05 | SOUP vulnerability (Bevy/rayon CVE) | Compromised simulation integrity or host system — malicious code execution, data corruption, or denial of service via exploited dependency | Known or zero-day vulnerability in one of 14 direct dependencies (bevy 0.15, glam 0.29, rayon 1.10, serde 1.0, ron 0.8, noise 0.9, oxidized_navigation 0.12, parry3d 0.17, bevy_egui 0.31, egui_plot 0.29, fxhash 0.2, tracing 0.1, bytemuck 1.25, serde_json 1.0) | P2 | S3 | Acceptable |
| H-06 | Output misinterpretation | User interprets "56.5% suppression" as a clinical prediction (e.g., expected tumor volume reduction in a patient) rather than as a dimensionless ratio of simulated energy efficiency | Output values presented without sufficient context about abstract units (qe); calibration profiles present nM and day equivalents that look clinical | P3 | S5 | ALARP |
| H-07 | Model lacks TME | Simulation diverges from biological reality for scenarios where tumor microenvironment (vasculature, hypoxia, stromal interactions, immune infiltration) is a dominant factor in treatment response | Fundamental design limitation — RESONANCE models tumors as homogeneous energy populations with frequency-based heterogeneity; no vasculature, no immune system, no ECM | P5 | S3 | ALARP |
| H-08 | Floating point precision errors | Subtle numerical errors accumulate over long simulations (10,000+ ticks), producing results that are deterministic but numerically divergent from the intended mathematical model | f32 precision (23-bit mantissa, ~7 decimal digits); catastrophic cancellation in energy differences; denormalized values near zero | P2 | S2 | Acceptable |
| H-09 | Axiom violation undetected | Fundamentally wrong physics — simulation behavior contradicts one or more of the 8 foundational axioms, producing results that are internally consistent but physically meaningless | Code change that inadvertently violates an axiom; insufficient test coverage for axiom compliance; derived constants computed incorrectly from the 4 fundamentals | P1 | S5 | ALARP |
| H-10 | Documentation gap | User misunderstands scope, limitations, or abstraction level of RESONANCE — applies results to contexts the software was not designed for | Incomplete, outdated, or missing documentation; user does not read disclaimers; documentation does not cover new features added after initial release | P3 | S3 | ALARP |
| H-11 | Pathway inhibitor escape frequency prediction used prescriptively | Researcher or clinician uses predicted escape frequencies to design resistance-preventing drug combinations without understanding that "frequency" is a computational proxy, not a measurable biological observable | escape_frequency function (`src/blueprint/equations/pathway_inhibitor.rs`) outputs optimal drug frequencies to block resistance; output could be mistaken for molecular target recommendations | P2 | S4 | ALARP |
| H-12 | Batch simulator diverges from Bevy simulator | Batch simulator (`src/batch/`) produces different results from the Bevy-based simulator for the same initial conditions, leading to contradictory conclusions depending on which mode was used | Batch simulator reimplements 33 systems independently from Bevy systems; any drift in equation calls, constant values, or ordering produces divergent output | P2 | S3 | Acceptable |

### 3.2 Hazard Detail Sheets

#### H-01: Overreliance on Resistance Predictions

**Description:** A user (researcher or, in a misuse scenario, a clinician) places excessive confidence in RESONANCE's drug resistance predictions and uses them to guide experimental design or, in the worst case, therapy selection.

**Cause analysis:**
- RESONANCE produces quantitative-looking output: "combo_AB: eff=0.435 (56.5% suppression)" (`src/use_cases/experiments/pathway_inhibitor_exp.rs`)
- The Bozic 2013 validation (10/10 seeds) creates an appearance of clinical validity
- 4 clinical calibration profiles map abstract units to nM and days, further blurring the research/clinical boundary
- The canine MCT case (Experiment 7, commit `971c7ac`) uses a real dog's tumor data, which could be emotionally compelling

**Probability justification (P3 — Occasional):**
- Users are defined as researchers with graduate-level competence (RD-1.1 §2.1), who are trained to evaluate simulation limitations
- However, calibration profiles and Bozic validation create a persuasive narrative that could exceed the model's actual predictive power
- No access control prevents non-researchers from running the software (open source, AGPL-3.0)

**Severity justification (S5 — Critical):**
- If used to inform clinical decisions (foreseeable misuse), suboptimal therapy selection could delay effective treatment
- For research-only use, severity would be S3 (wasted research effort); S5 applies to the foreseeable misuse scenario per ISO 14971:2019 §5.3 (foreseeable misuse shall be identified)

**Existing controls:** Disclaimers in README.md ("NOT clinical tool"), paper §5 Limitations, in-code comments, CLAUDE.md honest scope.

---

#### H-02: Energy Conservation Bug

**Description:** A software defect causes energy to be created or destroyed during simulation, violating Axiom 2 (Pool Invariant) and Axiom 5 (Conservation).

**Cause analysis:**
- 14 ECS layers and 33+ systems modify energy values; any system could introduce a conservation violation
- Energy is represented as f32, which cannot exactly represent all decimal values; accumulation errors could look like conservation violations
- Race conditions in parallel systems (rayon batch) could cause double-counting or missed updates

**Probability justification (P1 — Improbable):**
- Property-based fuzzing (`tests/property_conservation.rs`, 19 proptest cases) explicitly tests conservation under arbitrary inputs
- 3,113 automated tests include conservation-specific assertions
- Axiom 2 is enforced structurally: `Sigma energy(children) <= energy(parent)` checked in spawn systems
- The `derived_thresholds.rs` module (17 tests) verifies all lifecycle constants maintain conservation relationships

**Severity justification (S4 — Major):**
- Conservation violation would produce systematically biased results across all experiments
- Drug resistance predictions would be meaningless if energy can appear from nothing
- Detectability is high (conservation tests would catch most violations), reducing effective severity

---

#### H-03: Calibration Bias (Only 4 Profiles)

**Description:** The 4 clinical calibration profiles (CML/imatinib, prostate/abiraterone, NSCLC/erlotinib, canine MCT/toceranib) create an overfitting risk — the model appears validated but may not generalize.

**Cause analysis:**
- All 4 profiles use published IC50 and doubling time data that may itself reflect selection bias in the literature
- No negative control profiles exist (no case where the simulation is expected to fail)
- The canine MCT profile (`src/blueprint/equations/clinical_calibration.rs` line 89) uses toceranib IC50 as a proxy for an mRNA vaccine — fundamentally different mechanism
- 4 profiles across 3 cancer types and 1 species is a minimal validation set

**Probability justification (P4 — Probable):**
- Overfitting to 4 profiles is likely by construction — any model with sufficient degrees of freedom can fit 4 data points
- Users who encounter calibration profiles without reading the limitations may assume broader validity

**Severity justification (S3 — Moderate):**
- In a research context, overfitted models lead to wasted experimental effort when predictions fail to generalize
- The damage is bounded: researchers are expected to validate findings independently before publication
- No patient is directly harmed by overfitted research predictions

---

#### H-04: Determinism Broken

**Description:** RESONANCE produces different output for identical input parameters, undermining the bit-exact reproducibility guarantee.

**Cause analysis:**
- Hash-based RNG (`src/blueprint/equations/determinism.rs`) uses `f32::to_bits()` for bit-exact hashing
- Platform-dependent f32 operations (FMA instructions, different rounding modes) could produce different bit patterns
- Rayon thread pool scheduling is nondeterministic; if any result depends on thread execution order, output varies
- Bevy system scheduling may vary across versions or platforms

**Probability justification (P1 — Improbable):**
- 23 determinism tests verify bit-exact output
- `to_bits()` usage converts f32 to u32 before hashing, eliminating +0.0/-0.0/NaN ambiguity
- Batch simulator uses `par_iter_mut` on independent entities, avoiding order-dependent accumulation
- No thread-shared mutable state (Hard Block #4: no `Arc<Mutex<T>>`)

**Severity justification (S4 — Major):**
- Non-reproducible results undermine scientific credibility
- Published results (Zenodo paper, Bozic validation) could not be independently verified
- All 10-seed Bozic validation results would be suspect

---

#### H-05: SOUP Vulnerability (Bevy/rayon CVE)

**Description:** A known or zero-day vulnerability in a third-party dependency is exploited.

**Cause analysis:**
- 14 direct dependencies in `Cargo.toml`, expanding to ~200+ transitive dependencies in `Cargo.lock`
- Bevy 0.15 is a large framework (rendering, windowing, audio, networking) — attack surface is broad
- rayon 1.10 manages thread pools — a vulnerability could cause data races or memory corruption
- No automated CVE scanning is currently configured

**Probability justification (P2 — Remote):**
- RESONANCE does not accept network input (no tokio, no reqwest, no hyper — verified in Cargo.toml)
- RESONANCE processes no user-supplied untrusted data (input is simulation parameters defined in code or .ron files)
- Attack vector requires local access to the machine running RESONANCE
- Major dependencies (bevy, serde, rayon) are widely used and actively maintained

**Severity justification (S3 — Moderate):**
- RESONANCE runs on research workstations, not production servers or clinical systems
- No patient data exists to exfiltrate
- Worst case: host system compromise via malicious input to a rendering dependency, but RESONANCE has no network exposure

---

#### H-06: Output Misinterpretation

**Description:** A user interprets simulation output as a clinical prediction rather than a dimensionless research result.

**Cause analysis:**
- Output format: "combo_AB: eff=0.435 (56.5% suppression)" — "suppression" is a loaded term in oncology
- Calibration profiles present results in nM (drug concentration) and days (time), which are clinical units
- README line 141: "Against patient outcomes: Not yet" — "Not yet" implies future clinical validation is planned, which could inflate perceived reliability
- Bozic 2013 reference (a real clinical paper) creates association with clinical evidence

**Probability justification (P3 — Occasional):**
- Researchers in computational biology understand simulation limitations
- However, secondary users (pharma R&D teams, students — see RD-1.1 §2.2) may have less calibration about abstract units
- Open-source availability means anyone can run the software without reading documentation

**Severity justification (S5 — Critical):**
- If calibrated output is treated as a clinical prediction, it could influence treatment decisions (foreseeable misuse)
- The gap between simulation accuracy and clinical accuracy is large: no ADME, no TME, no immune system
- Severity reflects the worst-case foreseeable misuse, not the intended use

---

#### H-07: Model Lacks TME

**Description:** The simulation diverges from biological reality because it does not model the tumor microenvironment.

**Cause analysis:**
- RESONANCE paper §5 (lines 1267-1269): "does not model ADME pharmacokinetics, tumor microenvironment (vasculature, hypoxia), or adaptive immune response"
- No immune system components exist in the 14 ECS layers (`src/layers/mod.rs`)
- No vasculature, angiogenesis, or hypoxia modeling
- TME is a dominant factor in treatment response for many cancer types

**Probability justification (P5 — Frequent):**
- This is not a bug but a design limitation — it affects every simulation run
- Any comparison of RESONANCE output with real-world treatment response will be affected by TME absence

**Severity justification (S3 — Moderate):**
- In a research context, users are expected to understand model limitations
- The limitation is prominently documented (paper §5, README, CLAUDE.md)
- Severity is moderate because the divergence is predictable and can be accounted for in research design

---

#### H-08: Floating Point Precision Errors

**Description:** f32 precision limitations cause subtle numerical errors that accumulate over long simulations.

**Cause analysis:**
- f32 has ~7 decimal digits of precision; energy values spanning multiple orders of magnitude lose precision
- Energy differences (coherence - dissipation) near zero are susceptible to catastrophic cancellation
- Long simulations (10,000+ ticks) accumulate rounding errors across many iterations
- `blueprint/equations/derived_thresholds.rs` derives ~40 constants from 4 fundamentals — each derivation step introduces rounding

**Probability justification (P2 — Remote):**
- f32 is used consistently throughout the codebase (`src/layers/energy.rs`: `BaseEnergy { qe: f32 }`)
- `to_bits()` hashing ensures determinism despite precision limitations
- Simulation tick count is typically 1,000-10,000; accumulation is bounded
- No f64→f32 truncation occurs (f32 is used end-to-end)

**Severity justification (S2 — Minor):**
- Precision errors are deterministic (same result every time) — they don't undermine reproducibility
- The magnitude of error is small relative to simulation dynamics (energy values range 0-10,000 qe)
- Research conclusions are based on relative comparisons (combo vs. mono), which are robust to uniform precision errors

---

#### H-09: Axiom Violation Undetected

**Description:** A code change introduces behavior that contradicts one or more of the 8 foundational axioms, but existing tests do not detect the violation.

**Cause analysis:**
- 113K LOC with ~45 equation domain files — large surface area for axiom violations
- Not all axiom implications are tested (e.g., Axiom 6 "Emergence at Scale" is a meta-rule constraining the developer, not testable by unit tests)
- Derived axioms (3, 5, 6) are design constraints, not physics — their violation may not manifest as a test failure
- `CLAUDE.md` states axioms are "INVIOLABLE" but enforcement is by convention and code review, not by formal verification

**Probability justification (P1 — Improbable):**
- `derived_thresholds.rs` (17 tests) verifies the derivation chain from 4 fundamentals
- Conservation (Axiom 5) is fuzz-tested (`tests/property_conservation.rs`, 19 proptest cases)
- Pool Invariant (Axiom 2) is structurally enforced in spawn systems
- Code review role (Verificador) specifically checks axiom compliance per `CLAUDE.md` §Roles

**Severity justification (S5 — Critical):**
- An undetected axiom violation would invalidate all simulation output produced after the violation was introduced
- Published results referencing the affected version would be wrong
- The 8 axioms are the foundation of the entire system — a violation is not a bug but a fundamental corruption

---

#### H-10: Documentation Gap

**Description:** Incomplete or outdated documentation causes a user to misunderstand the scope, limitations, or correct interpretation of RESONANCE output.

**Cause analysis:**
- Documentation is maintained manually (no auto-generation from code)
- New features may be added without updating all documentation locations (README, paper, CLAUDE.md, regulatory docs)
- The paper (Zenodo) is a static snapshot — cannot be updated after publication
- Regulatory documents (this track) are new and may contain inconsistencies with older documentation

**Probability justification (P3 — Occasional):**
- README is updated regularly (latest commit `971c7ac` adds 5 limitations)
- Paper is static — any feature added after publication is undocumented there
- CLAUDE.md is comprehensive but complex (>500 lines) — users may not read it fully
- Regulatory documents are being created retroactively, increasing the risk of inconsistency

**Severity justification (S3 — Moderate):**
- Documentation gaps lead to misunderstanding, not to incorrect simulation output
- Users with domain expertise can evaluate output quality independently of documentation
- The primary risk is wasted effort or misapplied results, not direct harm

---

#### H-11: Pathway Inhibitor Escape Frequency Used Prescriptively

**Description:** The escape frequency prediction function is used to design clinical drug combinations.

**Cause analysis:**
- `escape_frequency` in `src/blueprint/equations/pathway_inhibitor.rs` computes optimal drug frequencies to prevent resistance
- "Frequency" in RESONANCE is an abstract proxy for genetic/epigenetic identity — it is not a measurable molecular property
- The function name and output format suggest actionable clinical insight

**Probability justification (P2 — Remote):**
- The function is internal to the simulation; it has no CLI endpoint
- Users would need to read source code and explicitly call this function
- Researchers in the intended user population understand that "frequency" is abstract

**Severity justification (S4 — Major):**
- If taken literally, escape frequency predictions could influence drug combination design
- Abstract frequency has no mapping to real molecular targets — the prediction is meaningless outside the simulation
- Severity is major because the output looks precise but is not grounded in molecular biology

---

#### H-12: Batch Simulator Diverges from Bevy Simulator

**Description:** The batch simulator produces different results from the Bevy-based simulator for the same scenario.

**Cause analysis:**
- `src/batch/` reimplements 33 systems that call the same `blueprint/equations/` functions
- System execution order may differ between batch (sequential phases) and Bevy (parallel schedule)
- Batch uses `EntitySlot` (flat struct) while Bevy uses ECS components — data layout differences could expose edge cases
- Bridge round-trip (`src/batch/bridge.rs`) converts between formats; any information loss causes divergence

**Probability justification (P2 — Remote):**
- Both simulators call the same pure functions in `blueprint/equations/`
- Bridge round-trip is tested (`src/batch/bridge.rs` — round-trip tests)
- Batch systems are individually tested (156 tests in `src/batch/`)

**Severity justification (S3 — Moderate):**
- Divergence would produce confusing but not dangerous results
- Users would notice discrepancies when comparing Bevy and batch outputs
- No clinical decision depends on batch/Bevy agreement

## 4. Hazard Summary by Risk Level

| Risk Level | Hazard IDs | Count |
|------------|-----------|-------|
| **Inaceptable** | None | 0 |
| **ALARP** | H-01, H-03, H-06, H-07, H-09, H-10, H-11 | 7 |
| **Acceptable** | H-02, H-04, H-05, H-08, H-12 | 5 |

No hazards are rated Inaceptable under the current research-only intended use. Seven hazards fall in the ALARP zone and require documented risk controls (see RD-2.4). Five hazards are in the Acceptable zone.

## 5. Foreseeable Misuse Analysis

ISO 14971:2019 §5.3 requires identification of foreseeable misuse. The following misuse scenarios are identified:

| Misuse Scenario | Related Hazard | Probability | Severity | Mitigation |
|-----------------|---------------|-------------|----------|------------|
| Clinician uses suppression % to select combination therapy for a patient | H-01, H-06 | P3 | S5 | Disclaimers in README, paper, CLI output |
| Researcher publishes RESONANCE results as clinical evidence without independent validation | H-01, H-03 | P3 | S4 | Paper §5 limitations, README "NOT validated" |
| Veterinarian uses canine MCT simulation to guide treatment of a specific animal | H-01, H-06 | P2 | S5 | "NOT VETERINARY ADVICE" disclaimer in `pathway_inhibitor_exp.rs` line 1295 |
| User removes disclaimer text from forked repository (AGPL-3.0 permits modification) | H-10 | P2 | S4 | AGPL-3.0 requires derivative works to maintain license; disclaimer removal does not change upstream software |
| Student uses calibrated output in a thesis as primary evidence | H-03, H-06 | P3 | S3 | Academic supervision expected; disclaimers present |

## 6. Codebase Evidence Index

All probability and severity ratings in this analysis reference the following codebase evidence:

| Evidence | File | Metric |
|----------|------|--------|
| Conservation fuzz testing | `tests/property_conservation.rs` | 19 proptest cases |
| Determinism tests | `src/blueprint/equations/determinism.rs` | 23 tests, `to_bits()` hashing |
| Derived threshold chain | `src/blueprint/equations/derived_thresholds.rs` | 17 tests, 4 fundamentals to ~40 constants |
| Pathway inhibitor math | `src/blueprint/equations/pathway_inhibitor.rs` | 42 tests, 11 pure functions |
| Pathway inhibitor constants | `src/blueprint/constants/pathway_inhibitor.rs` | 3 tests, 7 derived constants |
| Pathway inhibitor experiment | `src/use_cases/experiments/pathway_inhibitor_exp.rs` | 31 tests, Bozic 5-arm |
| Bozic 10-seed validation | `src/bin/bozic_validation.rs` | 10/10 seeds confirm combo > mono |
| Clinical calibration | `src/blueprint/equations/clinical_calibration.rs` | 21 tests, 4 profiles |
| Coulomb/LJ potentials | `src/blueprint/equations/coulomb.rs` | 26 tests |
| Total test suite | `cargo test` | 3,113 tests, 0 failures |
| Disclaimers | README.md, paper §5, `clinical_calibration.rs` line 92, `pathway_inhibitor_exp.rs` line 1295 | 7 locations |
| No network dependencies | `Cargo.toml` | No tokio/reqwest/hyper |
| No unsafe code | Codebase-wide | Hard Block #1 |
| No patient data structures | `src/layers/` (14 layers) | No PII/PHI fields |
| SOUP list | `Cargo.toml` | 14 direct dependencies |

## 7. Revision History

| Version | Date | Author | Changes |
|---------|------|--------|---------|
| 1.0 | 2026-04-02 | Resonance Development Team | Initial risk analysis. 12 hazards identified via FMEA. 5 foreseeable misuse scenarios documented. All ratings justified against codebase evidence at commit `971c7ac`. |

---

**Approvals:**

| Role | Name | Date | Signature |
|------|------|------|-----------|
| Alquimista (Analyst) | Resonance Development Team | 2026-04-02 | _draft -- not signed_ |
| Observador (Reviewer) | _pending_ | _pending_ | _pending_ |
