---
document_id: RD-2.4
title: Risk Control Measures
standard: ISO 14971:2019 §6
version: 1.0
date: 2026-04-02
status: DRAFT
author: Resonance Development Team
---

# Risk Control Measures

## 1. Purpose

This document specifies the risk control measures implemented for each hazard identified in RD-2.2 (Risk Analysis) and evaluated in RD-2.3 (Risk Evaluation). For each hazard, the document defines: the control type, specific implementation with codebase references, effectiveness verification method, and residual risk after control application.

This satisfies ISO 14971:2019 §6 (Risk control).

**Cross-references:**
- RD-2.1 (Risk Management Plan): Defines control types and acceptability criteria
- RD-2.2 (Risk Analysis): Hazard register
- RD-2.3 (Risk Evaluation): Hazard dispositions (Acceptable/ALARP/Inaceptable)
- RD-2.5 (Residual Risk Evaluation): Post-control residual risk assessment

## 2. Control Type Hierarchy

ISO 14971:2019 §6.2 prescribes the following priority order for risk control:

1. **Inherent safety by design** — Eliminate the hazard or reduce probability/severity through design choices
2. **Protective measures (Verification)** — Automated tests, validation, monitoring that detect the hazard
3. **Information for safety** — Labeling, disclaimers, documentation that warn users

For research-only software (IEC 62304 Class A), all three types are applicable. Information controls are emphasized because many hazards relate to misinterpretation rather than software defects.

## 3. Risk Control Register

### H-01: Overreliance on Resistance Predictions

**Pre-control risk:** P3 x S5 = ALARP

| Control | Type | Implementation | File Reference |
|---------|------|---------------|----------------|
| C-01.1 | Information | README.md disclaimers: "Not a clinical tool -- not validated against patient outcomes", "Not a drug discovery pipeline", "Not a substitute for oncology" | `README.md` lines 18-22 |
| C-01.2 | Information | Paper §5 Limitations: explicitly states abstract qe units, no ADME, no TME, qualitative only | `docs/paper/resonance_arxiv.tex` §5 (lines 1258-1272) |
| C-01.3 | Information | CLAUDE.md Honest Scope: "NOT clinical tools. Bozic comparison is qualitative (suppression %, not absolute cell counts or time-to-resistance in weeks)" | `CLAUDE.md` §Drug Models |
| C-01.4 | Information | CLI output disclaimer: "DISCLAIMER: SIMULATED. NOT VETERINARY ADVICE." | `src/use_cases/experiments/pathway_inhibitor_exp.rs` line 1295 |
| C-01.5 | Information | Validation table in README: "Against patient outcomes: Not yet" | `README.md` line 141 |
| C-01.6 | Information | Intended use statement excludes clinicians, patients, trial designers | RD-1.1 §2.3 (Explicitly Excluded Users) |
| C-01.7 | Design | Abstract energy units (qe) prevent direct clinical interpretation — no molar concentrations, no tumor volumes, no survival times in native output | `src/layers/energy.rs` — `BaseEnergy { qe: f32 }` |

**Effectiveness verification:**
- V-01.1: Grep codebase for all user-facing output paths; verify disclaimer presence. Result: disclaimers present at 7 identified locations.
- V-01.2: Review README.md, paper, and CLAUDE.md for consistency of scope limitations. Result: consistent messaging across all locations.
- V-01.3: Verify that no CLI binary produces output labeled as "clinical", "recommended dose", or "treatment plan". Result: no such labels found in any binary in `src/bin/`.

**Residual risk:** P3 x S5 reduced to P2 x S5 = ALARP. Disclaimers reduce probability (users who read documentation will not misuse) but do not eliminate it (users may not read, or may deliberately ignore). Severity remains S5 because the potential for clinical misuse exists regardless of disclaimers. Residual risk is ALARP — acceptable for research-only use because further reduction (access control, clinical validation) is disproportionate to the current use context.

---

### H-02: Energy Conservation Bug

**Pre-control risk:** P1 x S4 = Acceptable

| Control | Type | Implementation | File Reference |
|---------|------|---------------|----------------|
| C-02.1 | Verification | Property-based conservation fuzzing: proptest generates arbitrary energy configurations and verifies conservation invariants | `tests/property_conservation.rs` — 19 proptest cases |
| C-02.2 | Verification | Derived threshold tests verify all lifecycle constants maintain conservation relationships from 4 fundamentals | `src/blueprint/equations/derived_thresholds.rs` — 17 tests |
| C-02.3 | Verification | Full test suite includes conservation assertions in spawn, transfer, and dissipation systems | `cargo test` — 3,113 tests |
| C-02.4 | Design | Axiom 2 (Pool Invariant) structurally enforced: `energy(children) <= energy(parent)` checked in entity spawning | `src/simulation/reproduction/mod.rs` — parent drained, offspring qe <= drained |
| C-02.5 | Design | Axiom 5 (Conservation) enforced: total qe monotonically decreases. Energy is never created, only transferred or dissipated. | Codebase-wide architectural constraint |

**Effectiveness verification:**
- V-02.1: Run `cargo test` — all 3,113 tests pass (0 failures). Conservation-specific tests verified passing.
- V-02.2: Run proptest suite (`tests/property_conservation.rs`) — 19 cases pass with arbitrary inputs.
- V-02.3: Verify no `energy.qe +=` patterns exist without corresponding `energy.qe -=` in the same transaction. Result: energy modifications are always paired or go through pure equation functions.

**Residual risk:** P1 x S4 = Acceptable. No additional controls required. Existing verification controls are comprehensive for this hazard.

---

### H-03: Calibration Bias (Only 4 Profiles)

**Pre-control risk:** P4 x S3 = ALARP

| Control | Type | Implementation | File Reference |
|---------|------|---------------|----------------|
| C-03.1 | Verification | Multi-seed Bozic validation: 10 independent seeds, all 10 confirm combo > mono (threshold >= 80%) | `src/bin/bozic_validation.rs` — 10-seed robustness |
| C-03.2 | Information | README validation table: "Against patient outcomes: Not yet -- calibrated but not validated against longitudinal patient data" | `README.md` line 141 |
| C-03.3 | Information | Paper §5 explicitly states: "4 calibration profiles... do not constitute clinical validation" | `docs/paper/resonance_arxiv.tex` §5 |
| C-03.4 | Information | Clinical calibration file disclaimer: "DISCLAIMER: Calibrated from press reports... NOT from peer-reviewed trial data" | `src/blueprint/equations/clinical_calibration.rs` line 92 |
| C-03.5 | Information | Canine MCT profile documents the toceranib/mRNA mechanism mismatch as a known limitation | Commit `971c7ac` — 5 known limitations added to Rosie case |
| C-03.6 | Verification | Each calibration profile has automated tests verifying internal consistency | `src/blueprint/equations/clinical_calibration.rs` — 21 tests |

**Effectiveness verification:**
- V-03.1: Run `cargo run --release --bin bozic_validation` — 10/10 seeds confirm. Result is structural, not stochastic.
- V-03.2: Review 5 documented limitations for Rosie canine MCT case. All present at commit `971c7ac`.
- V-03.3: Verify no marketing or documentation claims generalization beyond the 4 profiled cancer types. Result: no such claims found.

**Residual risk:** P4 x S3 reduced to P3 x S2 = Acceptable. Multi-seed validation and explicit disclaimers reduce severity from S3 to S2 (minor) because users are warned that calibration is not validation. Probability reduced from P4 to P3 because disclaimers reduce the likelihood that users assume generalizability.

---

### H-04: Determinism Broken

**Pre-control risk:** P1 x S4 = Acceptable

| Control | Type | Implementation | File Reference |
|---------|------|---------------|----------------|
| C-04.1 | Design | Hash-based RNG: `hash_f32_slice` uses `f32::to_bits()` for bit-exact hashing, eliminating +0.0/-0.0/NaN ambiguity | `src/blueprint/equations/determinism.rs` — `hash_f32_slice`, `next_u64`, `unit_f32`, `gaussian_f32` |
| C-04.2 | Design | No external randomness: no `std::rand`, no `getrandom`, no entropy source. All randomness derives from entity index (deterministic seed). | `Cargo.toml` — no rand crate dependency |
| C-04.3 | Design | No shared mutable state: Hard Block #4 ("NO `Arc<Mutex<T>>`"), Hard Block #5 ("NO shared mutable state outside Resources") | `CLAUDE.md` §Hard Blocks |
| C-04.4 | Verification | 23 determinism tests verify bit-exact output for same inputs across function calls | `src/blueprint/equations/determinism.rs` — 23 tests |
| C-04.5 | Verification | Bozic validation reproduces identical results across 10 runs (same seed = same output) | `src/bin/bozic_validation.rs` |

**Effectiveness verification:**
- V-04.1: Run determinism tests — all 23 pass.
- V-04.2: Verify no `use rand` or `use getrandom` anywhere in source. Result: none found.
- V-04.3: Verify no `Arc<Mutex` pattern in source. Result: none found (Hard Block #4 enforced).

**Residual risk:** P1 x S4 = Acceptable. Existing design controls make determinism failure improbable. No additional controls required.

---

### H-05: SOUP Vulnerability (Bevy/rayon CVE)

**Pre-control risk:** P2 x S3 = Acceptable

| Control | Type | Implementation | File Reference |
|---------|------|---------------|----------------|
| C-05.1 | Design | `Cargo.lock` pins all dependency versions — no automatic upgrades | `Cargo.lock` (repository root) |
| C-05.2 | Design | No network dependencies — no tokio, reqwest, hyper, or networking crate. RESONANCE cannot receive network input. | `Cargo.toml` — verified absence |
| C-05.3 | Design | No untrusted input processing — simulation parameters are defined in code or .ron configuration files authored by the developer | All binary entry points in `src/bin/` |
| C-05.4 | Design | Hard Block #2: "NO external crates without approval -- only what's in Cargo.toml" | `CLAUDE.md` §Hard Blocks |
| C-05.5 | Information | SOUP list documented for traceability (planned: formal SOUP analysis in RD-3.2) | `Cargo.toml` — 14 direct dependencies |

**Effectiveness verification:**
- V-05.1: Verify `Cargo.lock` is committed to repository. Result: present and committed.
- V-05.2: Verify no networking crates in dependency tree. Result: bevy includes wgpu (GPU, not network); no HTTP/TCP/UDP crates.
- V-05.3: Check `cargo audit` for known vulnerabilities. **Gap:** `cargo audit` is not currently part of CI. This is documented as a gap for RD-3.

**Residual risk:** P2 x S3 = Acceptable. Existing design controls (no network, no untrusted input, pinned deps) are sufficient for a research tool running on local workstations. Gap: automated CVE scanning (`cargo audit`) should be added to CI.

---

### H-06: Output Misinterpretation

**Pre-control risk:** P3 x S5 = ALARP

| Control | Type | Implementation | File Reference |
|---------|------|---------------|----------------|
| C-06.1 | Information | All controls from H-01 (C-01.1 through C-01.7) apply — disclaimers at 7 locations | See H-01 controls |
| C-06.2 | Information | Calibration profiles explicitly state: "calibrated but not validated against longitudinal patient data" | `README.md` line 141 |
| C-06.3 | Information | Clinical calibration code-level disclaimer: "DISCLAIMER: Calibrated from press reports... NOT from peer-reviewed trial data" | `src/blueprint/equations/clinical_calibration.rs` line 92 |
| C-06.4 | Design | Native output uses abstract units (qe, Hz, dimensionless ratios). Clinical units (nM, days) appear only in calibration profiles, which are explicitly marked as research calibrations. | `src/layers/energy.rs`, `src/blueprint/equations/clinical_calibration.rs` |
| C-06.5 | Information | Output format includes "eff=" prefix (efficiency ratio, dimensionless) and "suppression" with "%" — both are relative metrics, not absolute clinical measurements | `src/use_cases/experiments/pathway_inhibitor_exp.rs` output format |

**Effectiveness verification:**
- V-06.1: Review all CLI binary outputs for unit labeling. Verify that qe is labeled as "qe" (not "Joules" or "concentration"). Result: abstract unit labeling confirmed.
- V-06.2: Verify calibration profile output includes disclaimer text. Result: disclaimer present at `clinical_calibration.rs` line 92.
- V-06.3: Verify no output format uses RECIST criteria, tumor volume, progression-free survival, or other clinical endpoints. Result: no clinical endpoint format found.

**Residual risk:** P3 x S5 reduced to P2 x S4 = ALARP. Disclaimers and abstract units reduce both probability and severity. Residual risk is ALARP — acceptable for research-only use. If intended use changes to SaMD, additional controls (output format redesign, mandatory warnings, user authentication) would be required.

---

### H-07: Model Lacks TME

**Pre-control risk:** P5 x S3 = ALARP

| Control | Type | Implementation | File Reference |
|---------|------|---------------|----------------|
| C-07.1 | Information | Paper §5 Limitations: "does not model ADME pharmacokinetics, tumor microenvironment (vasculature, hypoxia), or adaptive immune response" | `docs/paper/resonance_arxiv.tex` lines 1267-1269 |
| C-07.2 | Information | README "Honest scope" section documents model limitations | `README.md` |
| C-07.3 | Information | CLAUDE.md §Drug Models Honest scope: "No tumor microenvironment" | `CLAUDE.md` §Drug Models |
| C-07.4 | Information | Paper future work lists immune system modeling as unimplemented: "Immune system: Model T-cell, NK-cell interactions as frequency-selective predation" | `docs/paper/resonance_arxiv.tex` line 1282 |
| C-07.5 | Information | No immune system components exist in the 14 ECS layers — the absence itself is a signal to informed users | `src/layers/mod.rs` — no immune, vascular, or ECM layers |

**Effectiveness verification:**
- V-07.1: Verify paper §5 limitations paragraph is present and complete. Result: present at lines 1258-1272.
- V-07.2: Verify no layer in `src/layers/mod.rs` claims to model vasculature, immune, or TME. Result: no such layers.
- V-07.3: This limitation is inherent to the model design and cannot be "tested away." Verification confirms documentation, not elimination.

**Residual risk:** P5 x S3 = ALARP (unchanged). This is an inherent limitation — probability cannot be reduced (every run is affected). Severity is S3 because the limitation is well-documented and understood by the intended user population. Controls are exclusively informational. Residual risk is accepted as ALARP because further reduction would require fundamental model redesign, which is out of scope for the current research-only use.

---

### H-08: Floating Point Precision Errors

**Pre-control risk:** P2 x S2 = Acceptable

| Control | Type | Implementation | File Reference |
|---------|------|---------------|----------------|
| C-08.1 | Design | f32 used consistently end-to-end — no f64-to-f32 truncation, no mixed-precision arithmetic | `src/layers/energy.rs`, all `blueprint/equations/` modules |
| C-08.2 | Design | `to_bits()` conversion for hashing ensures bit-exact determinism despite precision limitations | `src/blueprint/equations/determinism.rs` line 14 |
| C-08.3 | Design | Energy values bounded (0.0 to ~10,000 qe in typical simulations) — stays well within f32 representable range | Simulation dynamics constrained by conservation axioms |
| C-08.4 | Verification | Determinism tests verify identical output across runs — precision errors are at least deterministic | `src/blueprint/equations/determinism.rs` — 23 tests |

**Effectiveness verification:**
- V-08.1: Verify no `as f32` casts from f64 in simulation-critical code. Result: f32 is the native type throughout.
- V-08.2: Verify no denormalized value handling issues. Result: energy values are bounded above zero by physical constraints.

**Residual risk:** P2 x S2 = Acceptable. No additional controls required. f32 precision is standard practice for game engines and simulation tools.

---

### H-09: Axiom Violation Undetected

**Pre-control risk:** P1 x S5 = ALARP

| Control | Type | Implementation | File Reference |
|---------|------|---------------|----------------|
| C-09.1 | Verification | Derived threshold tests: all ~40 lifecycle constants verified to derive correctly from 4 fundamentals | `src/blueprint/equations/derived_thresholds.rs` — 17 tests |
| C-09.2 | Verification | Conservation fuzz tests (Axiom 2, 5): proptest generates arbitrary inputs and verifies conservation | `tests/property_conservation.rs` — 19 proptest cases |
| C-09.3 | Verification | Coulomb/LJ tests (Axiom 7, 8): inverse-square law, Newton 3, charge conservation, frequency alignment | `src/blueprint/equations/coulomb.rs` — 26 tests |
| C-09.4 | Verification | Pathway inhibitor tests: Hill pharmacokinetics, binding affinity, inhibition modes all axiom-derived | `src/blueprint/equations/pathway_inhibitor.rs` — 42 tests |
| C-09.5 | Design | CLAUDE.md INVIOLABLE declaration: "No change, feature, refactor, or optimization may contradict, bypass, or weaken ANY of the 8 axioms" | `CLAUDE.md` §The 8 Foundational Axioms |
| C-09.6 | Verification | Code review role (Verificador) specifically checks: "1) contract 2) math 3) DOD 4) determinism 5) perf 6) tests. Math or determinism doubt = BLOCK" | `CLAUDE.md` §Roles |
| C-09.7 | Verification | Bozic validation serves as an end-to-end axiom compliance check — if axioms were violated, the qualitative prediction would likely fail | `src/bin/bozic_validation.rs` — 10/10 seeds |

**Effectiveness verification:**
- V-09.1: Run all 3,113 tests — pass. Conservation, determinism, and derived threshold tests exercise axiom compliance.
- V-09.2: Review `derived_thresholds.rs` test coverage — all 17 derivation paths tested.
- V-09.3: Verify Verificador review protocol includes axiom compliance check. Result: documented in CLAUDE.md Roles section.
- V-09.4: **Gap acknowledged:** Axiom 6 ("Emergence at Scale") is a meta-rule constraining the developer. It cannot be tested by unit tests. Compliance depends on code review discipline.

**Residual risk:** P1 x S5 = ALARP. Probability remains P1 due to extensive verification. Severity remains S5 because axiom violation would invalidate all output. ALARP is accepted because further reduction would require formal verification (proof assistants), which is disproportionate for Class A research software.

---

### H-10: Documentation Gap

**Pre-control risk:** P3 x S3 = ALARP

| Control | Type | Implementation | File Reference |
|---------|------|---------------|----------------|
| C-10.1 | Information | This regulatory documentation track (RD-1 through RD-7, 50 planned documents) provides systematic documentation | `docs/regulatory/` — 7 subdirectories |
| C-10.2 | Information | README maintained with each major feature addition (latest: commit `971c7ac` adds 5 limitations) | `README.md` |
| C-10.3 | Information | CLAUDE.md serves as comprehensive developer and user reference (>500 lines, regularly updated) | `CLAUDE.md` |
| C-10.4 | Information | Paper published on Zenodo (static, peer-reviewable) | DOI: 10.5281/zenodo.19342036 |
| C-10.5 | Information | Architecture documentation maintained | `docs/ARCHITECTURE.md`, `docs/design/`, `docs/arquitectura/` |
| C-10.6 | Verification | Risk management plan (RD-2.1 §5) establishes review schedule: each major release + annual review | `docs/regulatory/02_risk_management/RISK_MANAGEMENT_PLAN.md` |

**Effectiveness verification:**
- V-10.1: Verify all 5 RD-1 foundation documents are complete. Result: RD-1.1 through RD-1.5 exist in `docs/regulatory/01_foundation/`.
- V-10.2: Verify README, CLAUDE.md, and paper are consistent on key scope claims. Result: all state "NOT clinical tool", abstract qe units, qualitative only.
- V-10.3: **Gap acknowledged:** Documentation is maintained manually. No automated consistency check exists between README, CLAUDE.md, paper, and regulatory docs. Drift is possible.

**Residual risk:** P3 x S3 reduced to P2 x S2 = Acceptable. Regulatory documentation track and periodic reviews reduce both probability and severity. Residual risk is Acceptable.

---

### H-11: Pathway Inhibitor Escape Frequency Used Prescriptively

**Pre-control risk:** P2 x S4 = ALARP

| Control | Type | Implementation | File Reference |
|---------|------|---------------|----------------|
| C-11.1 | Design | `escape_frequency` is an internal function with no CLI endpoint — users must read source code to access it | `src/blueprint/equations/pathway_inhibitor.rs` — internal function |
| C-11.2 | Information | Module-level documentation states "frequency" is a computational proxy, not a measurable biological observable | `CLAUDE.md` §Axiomatic Abiogenesis, §Drug Models Honest scope |
| C-11.3 | Information | Paper §5 explicitly states: "Frequency is a proxy for genetic/epigenetic identity, not a measured spectral property" | `docs/paper/resonance_arxiv.tex` §5, RD-1.1 §5.4 |
| C-11.4 | Information | Intended use excludes drug discovery: "Not a drug discovery pipeline -- does not design molecules" | `README.md` line 21 |

**Effectiveness verification:**
- V-11.1: Verify no CLI binary exposes escape_frequency output directly. Result: no binary in `src/bin/` calls or outputs escape_frequency results to stdout.
- V-11.2: Verify paper and README explain the abstract nature of "frequency". Result: documented in both.

**Residual risk:** P2 x S4 reduced to P1 x S3 = Acceptable. No CLI exposure + disclaimers make prescriptive use very unlikely. Residual risk is Acceptable.

---

### H-12: Batch/Bevy Simulator Divergence

**Pre-control risk:** P2 x S3 = Acceptable

| Control | Type | Implementation | File Reference |
|---------|------|---------------|----------------|
| C-12.1 | Design | Both simulators call the same pure functions in `blueprint/equations/` — shared math layer | `src/blueprint/equations/mod.rs` — single source of truth for all math |
| C-12.2 | Verification | Bridge round-trip tests verify lossless conversion between GenomeBlob and Bevy components | `src/batch/bridge.rs` — round-trip tests |
| C-12.3 | Verification | 156 batch-specific tests verify individual system correctness | `src/batch/` — 156 tests across 33 systems |
| C-12.4 | Design | Constants shared between batch and Bevy via `blueprint/constants/` — single source of truth | `src/blueprint/constants/mod.rs` |

**Effectiveness verification:**
- V-12.1: Run batch tests — all 156 pass.
- V-12.2: Verify `src/batch/systems/` imports equations from `blueprint/equations/`, not reimplementations. Result: confirmed — batch systems call blueprint functions.
- V-12.3: **Gap acknowledged:** No end-to-end equivalence test exists that runs the same scenario in both Bevy and batch mode and compares results tick-by-tick. This is documented as a testing gap.

**Residual risk:** P2 x S3 = Acceptable. Shared math layer makes divergence unlikely. No additional controls required, though an end-to-end equivalence test would further reduce risk.

## 4. Control Effectiveness Summary

| ID | Pre-Control Risk | Control Types | Post-Control Residual Risk | Risk Reduced? |
|----|-----------------|---------------|---------------------------|---------------|
| H-01 | P3 x S5 (ALARP) | Information (7 controls) + Design (1) | P2 x S5 (ALARP) | Yes (P reduced) |
| H-02 | P1 x S4 (Acceptable) | Verification (3) + Design (2) | P1 x S4 (Acceptable) | Already acceptable |
| H-03 | P4 x S3 (ALARP) | Verification (2) + Information (4) | P3 x S2 (Acceptable) | Yes (P and S reduced) |
| H-04 | P1 x S4 (Acceptable) | Design (3) + Verification (2) | P1 x S4 (Acceptable) | Already acceptable |
| H-05 | P2 x S3 (Acceptable) | Design (4) + Information (1) | P2 x S3 (Acceptable) | Already acceptable |
| H-06 | P3 x S5 (ALARP) | Information (5) + Design (1) | P2 x S4 (ALARP) | Yes (P and S reduced) |
| H-07 | P5 x S3 (ALARP) | Information (5) | P5 x S3 (ALARP) | No (inherent limitation) |
| H-08 | P2 x S2 (Acceptable) | Design (3) + Verification (1) | P2 x S2 (Acceptable) | Already acceptable |
| H-09 | P1 x S5 (ALARP) | Verification (5) + Design (1) | P1 x S5 (ALARP) | No (severity irreducible) |
| H-10 | P3 x S3 (ALARP) | Information (5) + Verification (1) | P2 x S2 (Acceptable) | Yes (P and S reduced) |
| H-11 | P2 x S4 (ALARP) | Design (1) + Information (3) | P1 x S3 (Acceptable) | Yes (P and S reduced) |
| H-12 | P2 x S3 (Acceptable) | Design (2) + Verification (2) | P2 x S3 (Acceptable) | Already acceptable |

## 5. New Hazards Introduced by Controls

ISO 14971:2019 §6.4 requires evaluating whether risk controls introduce new hazards.

| Control | Potential New Hazard | Assessment |
|---------|---------------------|------------|
| Disclaimers (C-01.x, C-06.x) | Disclaimer fatigue — excessive warnings cause users to ignore all warnings | Low risk. Disclaimers are concentrated in README and paper, not interleaved with simulation output. Users encounter them at documentation-reading time, not at simulation-running time. |
| Deterministic RNG (C-04.1) | Reduced randomness quality compared to cryptographic RNG | Not applicable. RESONANCE does not require cryptographic randomness. Determinism is a feature, not a limitation, for scientific reproducibility. |
| Cargo.lock pinning (C-05.1) | Stale dependencies — pinned versions may miss security patches | Acknowledged. This is managed by periodic dependency review (planned: RD-3 SOUP analysis). The risk of stale dependencies is lower than the risk of automatic upgrades breaking determinism. |
| Abstract units (C-01.7, C-06.4) | Users confused by abstract units may make conversion errors | Low risk. Calibration profiles provide explicit conversion factors. Native abstract output is clearly labeled. |

No new hazards of significant risk are introduced by the implemented controls.

## 6. Gaps and Open Items

| Gap | Affected Hazard | Planned Resolution | Sprint |
|-----|-----------------|-------------------|--------|
| No `cargo audit` in CI | H-05 | Add automated CVE scanning to CI pipeline | RD-3 |
| No end-to-end Bevy/batch equivalence test | H-12 | Implement tick-by-tick comparison test | Engineering backlog |
| No automated documentation consistency check | H-10 | Consider linting tool for cross-document consistency | RD-5 |
| Axiom 6 not testable by unit tests | H-09 | Accepted limitation — code review is the primary control | N/A |
| Single-developer review independence | H-09 | Temporal separation documented; external review if reclassified | RD-5 |

## 7. Revision History

| Version | Date | Author | Changes |
|---------|------|--------|---------|
| 1.0 | 2026-04-02 | Resonance Development Team | Initial risk control register. 12 hazards with 52 total controls documented. Effectiveness verification for each control. 5 gaps identified with planned resolutions. |

---

**Approvals:**

| Role | Name | Date | Signature |
|------|------|------|-----------|
| Alquimista (Implementer) | Resonance Development Team | 2026-04-02 | _draft -- not signed_ |
| Verificador (Verifier) | _pending_ | _pending_ | _pending_ |
