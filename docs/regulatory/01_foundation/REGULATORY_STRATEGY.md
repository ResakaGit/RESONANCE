---
document_id: RD-1.5
title: Regulatory Strategy
standard: IMDRF SaMD Framework, FDA De Novo Guidance
version: 1.0
date: 2026-04-02
status: DRAFT
author: Resonance Development Team
---

# Regulatory Strategy

## 1. Purpose

This document defines the regulatory positioning of RESONANCE and charts the pathway from its current status as a **research-only simulation tool** to potential future classification as Software as a Medical Device (SaMD). It serves three functions:

1. **Positioning:** Establish, with evidence, that RESONANCE is not currently SaMD and therefore not subject to mandatory medical device regulation.
2. **Preparedness:** Identify which regulatory standards are voluntarily adopted and where gaps remain, so that a future transition to SaMD (if warranted) is achievable without a ground-up documentation effort.
3. **Credibility:** Provide pharma partners, journal reviewers, and grant agencies with a formal regulatory strategy that demonstrates disciplined development practice.

This document does not claim or imply that RESONANCE meets any regulatory standard in full. Where compliance is partial, that is stated explicitly.

---

## 2. Current Positioning: Research Tool

### 2.1 RESONANCE Is Not SaMD

Under the IMDRF definition (IMDRF/SaMD WG/N10R4:2013), Software as a Medical Device is software intended to be used for one or more medical purposes without being part of a hardware medical device. The critical phrase is **"intended to be used."**

RESONANCE does not meet this definition because:

- **Intended use is research exploration, not clinical decision-making.** RESONANCE simulates emergent life dynamics using abstract energy (qe) units. It does not accept patient data, does not output treatment recommendations, and does not interface with clinical workflows.
- **Outputs are not actionable in a clinical context.** Results are expressed as dimensionless suppression percentages (e.g., "56.5% suppression"), not molar concentrations, tumor volumes, progression-free survival, or any clinically interpretable metric.
- **No molecular resolution.** The model operates on abstract frequency-based binding (Axiom 8), not molecular targets (EGFR, BCR-ABL, PD-L1). Drug models use Hill pharmacokinetics with abstract parameters, not pharmacokinetic/pharmacodynamic (PK/PD) profiles.
- **Explicit disclaimers are pervasive.** README, paper (Zenodo DOI: 10.5281/zenodo.19342036), binary outputs, and architecture documentation all state: "NOT a clinical tool," "NOT validated against patient outcomes," and "NOT a substitute for clinical trials."

### 2.2 Voluntary Compliance Rationale

Although RESONANCE is exempt from mandatory regulatory compliance, the development team voluntarily documents against the following standards as best practice:

| Standard | Rationale for Voluntary Adoption |
|----------|----------------------------------|
| IEC 62304 (software lifecycle) | Establishes credibility that software development follows a recognized process |
| ISO 14971 (risk management) | Demonstrates systematic identification of what could go wrong if misused |
| ASME V&V 40 (computational credibility) | Required by FDA CMS Guidance 2023 for any computational model submitted as evidence |
| ISO 13485 (QMS) | Provides structure for disciplined development; required if SaMD is pursued |
| 21 CFR Part 11 (electronic records) | Git-based audit trail already satisfies many requirements by design |

### 2.3 Benefits of Voluntary Compliance

- **Publication support.** Journals increasingly require V&V documentation for computational models. ASME V&V 40 compliance strengthens manuscript credibility.
- **Pharma partnership readiness.** Pharmaceutical R&D partners performing in silico modeling require suppliers to demonstrate software quality. ISO 13485 documentation accelerates due diligence.
- **Future optionality.** If RESONANCE's drug resistance models prove scientifically valuable enough to inform clinical trial design, the gap to SaMD classification is reduced from ~50 documents to incremental additions.
- **Liability protection.** Documented disclaimers + formal risk analysis provide defense against claims of misrepresentation.

---

## 3. IMDRF SaMD Classification Analysis

### 3.1 IMDRF Framework Summary

The IMDRF SaMD classification framework (N12R2:2014) uses a two-axis risk matrix:

| | **Treat or diagnose** | **Drive clinical management** | **Inform clinical management** |
|---|---|---|---|
| **Critical** | IV | III | II |
| **Serious** | III | II | I |
| **Non-serious** | II | I | I |

### 3.2 RESONANCE Classification (If It Were SaMD)

**State of healthcare situation: Non-serious.**
RESONANCE operates in a research context where no patient is directly affected by its outputs. Even if outputs were used to inform therapy selection, the software would not be the sole basis for a treatment decision.

**Significance to healthcare decision: Inform.**
RESONANCE does not treat, diagnose, or drive clinical management. At most, it would inform a researcher's understanding of resistance dynamics. It does not provide patient-specific recommendations.

**Result: Category I** (lowest risk, if reclassified as SaMD).

### 3.3 Decision Tree Walkthrough

```
Q1: Does the software meet the definition of SaMD?
    → Does it have a medical purpose?
      Current: NO. Purpose is research simulation.
      Hypothetical: YES, if intended to inform therapy selection.
    → Is it independent of hardware?
      YES. Pure software (Rust/Bevy, no device interface).

Q2: What is the intended medical purpose?
    → NOT treat or diagnose (no patient interface)
    → NOT drive clinical management (no recommendations)
    → Inform clinical management (at most: "combination therapy may outperform monotherapy for this resistance profile")

Q3: What is the state of healthcare situation?
    → Non-serious (research hypothesis generation, not patient-facing)

Q4: Classification result?
    → Category I (Non-serious × Inform)
```

### 3.4 Why Category I Still Matters

Even Category I SaMD requires:
- Quality Management System (ISO 13485 or equivalent)
- Risk management (ISO 14971)
- Software lifecycle documentation (IEC 62304 Class A minimum)
- Clinical evaluation proportionate to risk
- Post-market surveillance

RESONANCE currently lacks the formal documentation for most of these, though many underlying practices (testing, determinism, disclaimers) already exist informally. The gap is documentation, not practice.

---

## 4. If RESONANCE Becomes SaMD: Regulatory Pathways

### 4.1 FDA (United States)

**Pathway: De Novo Classification (21 CFR 860.260)**

There is no predicate device for an emergent life simulation engine that models drug resistance dynamics from first principles. The 510(k) pathway requires a substantially equivalent predicate; none exists. De Novo is the appropriate pathway for novel, low-to-moderate-risk devices.

| Item | Detail |
|------|--------|
| Classification | Class II (De Novo default) |
| Product code | New (no existing code for energy-based drug resistance simulation) |
| Review division | OHT7 (Digital Health Center of Excellence) |
| Pre-submission | **Strongly recommended.** Novel computational model + no predicate = high uncertainty about FDA expectations. A Q-Sub meeting would clarify acceptable validation evidence. |
| Clinical evidence | Analytical validation (tests) + computational credibility (ASME V&V 40) + published literature comparisons. No clinical trial expected for Category I/inform-only. |
| Timeline estimate | 12-18 months from pre-submission to decision (De Novo average: 150-300 review days + sponsor preparation) |
| Critical gaps | Formal SRS (IEC 62304 format), traceability matrix, clinical evaluation report, Part 11 assessment, SBOM |

**FDA CMS Guidance 2023 (Credibility of Computational Modeling and Simulation):**
This guidance is directly applicable. It requires a credibility assessment based on the Context of Use (COU), verification evidence, validation evidence, and uncertainty quantification. RESONANCE's existing Bozic validation, multi-seed robustness, and deterministic RNG provide a foundation, but the evidence needs to be organized per V&V 40 structure.

### 4.2 European Union

**Pathway: MDR Class IIa (Rule 11)**

Under EU MDR 2017/745, software intended to provide information used to make decisions with diagnostic or therapeutic purposes is classified per Rule 11. For non-serious situations with inform-only significance, Rule 11 yields Class IIa.

| Item | Detail |
|------|--------|
| Classification | Class IIa (Rule 11: software providing information for therapeutic decisions, non-serious situation) |
| Conformity assessment | Annex IX (QMS + technical documentation) via Notified Body |
| CE marking | Required |
| Technical documentation | Annex II + Annex III requirements: intended purpose, risk management, V&V, clinical evaluation, GSPR checklist |
| Notified Body | Must be MDR-designated with software competence. Limited availability; lead times 6-12 months for audit scheduling. |
| UDI | Required (Unique Device Identification) |
| EUDAMED registration | Required |
| Post-market surveillance | PMS plan + periodic safety update report (PSUR) |
| Timeline estimate | 18-24 months (technical file preparation + Notified Body queue + audit + certification) |
| Critical gaps | All gaps listed in Section 5, plus GSPR checklist, EU-specific clinical evaluation per MEDDEV 2.7/1 rev 4, PMS plan |

### 4.3 Health Canada

**Pathway: Class II Medical Device**

Health Canada follows the IMDRF SaMD framework directly. Category I SaMD maps to Class II under the Medical Devices Regulations (SOR/98-282).

| Item | Detail |
|------|--------|
| Classification | Class II |
| License | Medical Device Establishment License (MDEL) + Device License |
| Audit framework | MDSAP (Medical Device Single Audit Program) — ISO 13485 audit by recognized auditing organization |
| Timeline estimate | 12-18 months (MDSAP audit + Health Canada review) |
| Critical gaps | ISO 13485 QMS (formal, auditable), MDSAP-ready procedures |

### 4.4 Pathway Comparison Summary

| Dimension | FDA De Novo | EU MDR IIa | Health Canada II |
|-----------|------------|------------|-----------------|
| Effort to prepare | High (novel pathway, no predicate) | Very high (Notified Body, GSPR) | High (MDSAP audit) |
| Cost estimate | $100K-300K (regulatory + legal) | $150K-400K (NB fees + CE process) | $80K-200K (MDSAP + license) |
| Timeline | 12-18 months | 18-24 months | 12-18 months |
| Recurring obligations | Annual reporting, post-market | PSUR, vigilance, NB surveillance | MDSAP re-audit, incident reporting |
| Recommended first? | Yes (Q-Sub provides early feedback) | No (start after FDA clarity) | Parallel with FDA via MDSAP |

---

## 5. Gap Analysis Summary

### 5.1 Overall Documentation Status

| Status | Documents | Percentage |
|--------|-----------|------------|
| Exists | 1 | 2% |
| Partial (practices exist, formal docs missing) | 17 | 34% |
| Missing (no documentation or practice) | 32 | 64% |
| **Total required** | **50** | **100%** |

### 5.2 IEC 62304:2006+Amd1:2015 — Software Lifecycle

| What Exists | Codebase Reference |
|-------------|-------------------|
| 3,113 automated tests (unit, integration, property) | `cargo test` — 36 sec runtime |
| Architecture documentation | `docs/ARCHITECTURE.md` |
| Detailed design per module | `docs/design/*.md` (6 specs), `docs/arquitectura/*.md` (4 contracts) |
| Coding standards | `CLAUDE.md` — Hard Blocks, Coding Rules, Bevy 0.15 Patterns |
| Sprint-based development with closure criteria | `docs/sprints/archive/` — completed sprints with grep-verified criteria |
| Defined roles (Alquimista, Observador, Planificador, Verificador) | `CLAUDE.md` Roles section |

| What Is Missing | IEC 62304 Clause |
|-----------------|------------------|
| Formal Software Development Plan (SDP) | 5.1 |
| Software Requirements Specification (SRS) in standard format | 5.2 |
| Formal architecture design document (current is close but not clause-mapped) | 5.3 |
| Detailed design traceability to requirements | 5.4 |
| Software unit verification report (tests exist but no formal report) | 5.5 |
| Software integration testing plan and report | 5.6 |
| System testing plan and report | 5.7 |
| Software release documentation | 5.8 |
| Software maintenance plan | 6 |
| Software configuration management plan (Git is used but not formally documented) | 8 |
| Software problem resolution process | 9 |

### 5.3 ISO 14971:2019 — Risk Management

| What Exists | Codebase Reference |
|-------------|-------------------|
| Disclaimers ("NOT clinical tool") in all user-facing outputs | `README.md`, paper Section 5, binary CLI outputs |
| Known limitations documented honestly | `README.md` "Honest scope" section, paper Section 5 |
| 5 documented limitations in Rosie case | Commit `971c7ac` |
| Hazard awareness (overconfidence, calibration bias, no TME) | `README.md`, `CLAUDE.md` drug model sections |

| What Is Missing | ISO 14971 Clause |
|-----------------|------------------|
| Risk management plan (scope, criteria, methods, schedule) | 5.1 |
| Systematic hazard identification (FMEA or equivalent) | 5.3-5.4 |
| Risk evaluation against defined acceptability criteria | 5.5 |
| Risk control measures formally documented and verified | 6 |
| Residual risk evaluation | 7 |
| Risk management report | 8 |
| Production and post-production information review | 9 |

### 5.4 ISO 13485:2016 — Quality Management System

| What Exists | Codebase Reference |
|-------------|-------------------|
| Sprint methodology with defined deliverables and closure criteria | `docs/sprints/` directory structure |
| Four defined roles with responsibilities | `CLAUDE.md` Roles table |
| Coding standards enforced by convention | `CLAUDE.md` Hard Blocks (17 rules) |
| Change history via Git | `git log` — full immutable history |
| Nonconformance detection via test suite | `cargo test` — all 3,113 must pass before merge |

| What Is Missing | ISO 13485 Clause |
|-----------------|------------------|
| Quality Manual | 4.2.2 |
| Quality Policy and objectives | 5.3, 5.4.1 |
| Document control procedure | 4.2.4 |
| Record control procedure | 4.2.5 |
| Internal audit procedure | 8.2.4 |
| Control of nonconforming product procedure | 8.3 |
| CAPA procedure | 8.5.2-8.5.3 |
| Management review records | 5.6 |

### 5.5 ASME V&V 40:2018 — Computational Credibility

| What Exists | Codebase Reference |
|-------------|-------------------|
| Bozic 2013 validation (5-arm, 10/10 seeds) | `src/bin/bozic_validation.rs` |
| 4 clinical calibration profiles (London, Gatenby, Bozic, Rosie) | `README.md` clinical calibration section |
| Deterministic simulation (bit-exact, hash-based RNG) | `src/blueprint/equations/determinism.rs` |
| Conservation property testing (proptest) | `tests/property_conservation.rs` |
| 4 fundamental constants with full derivation chain | `src/blueprint/equations/derived_thresholds.rs` (17 tests) |
| Published paper with experiments | Zenodo DOI: 10.5281/zenodo.19342036 |

| What Is Missing | V&V 40 Section |
|-----------------|----------------|
| Formal Context of Use (COU) statement | 4.1 |
| Credibility assessment framework (risk-informed) | 4 |
| Verification report (tests exist, formal report does not) | 5 |
| Validation report (results exist, formal report does not) | 6 |
| Uncertainty quantification (multi-seed exists, formal analysis does not) | 7 |
| Applicability assessment (informal in README, no formal doc) | 8 |

### 5.6 21 CFR Part 11 — Electronic Records and Signatures

| What Exists | Codebase Reference |
|-------------|-------------------|
| Immutable audit trail | Git history (every change attributed, timestamped) |
| Record reproducibility | Deterministic simulation — same seed produces identical output |
| Configuration control | `Cargo.lock` pins all dependency versions |
| Validation evidence | 3,113 tests + property-based fuzzing |

| What Is Missing | Part 11 Section |
|-----------------|-----------------|
| Formal compliance assessment document | 11.10 (all subsections) |
| Access control documentation | 11.10(d) |
| Authority checks (formal, beyond Git) | 11.10(g) |
| Personnel training records | 11.10(i) |
| Electronic signature procedures | 11.50, 11.70 |
| Open/closed system assessment | 11.30 |

---

## 6. Regulatory Documentation Roadmap

The documentation effort is structured as 7 sprints (RD-1 through RD-7), organized in 4 execution waves following a dependency graph.

### 6.1 Sprint Overview

| Sprint | Focus | Documents | Effort | Blocked By |
|--------|-------|-----------|--------|------------|
| **RD-1** | Regulatory Foundation | 5 (Intended Use, Safety Class, SRS, SDP, this Strategy) | High | None |
| **RD-2** | Risk Management File (ISO 14971) | 6 (Plan, Analysis, Evaluation, Controls, Residual, Report) | High | RD-1 |
| **RD-3** | Traceability + SOUP + SBOM | 4 (Matrix, SOUP Analysis, SBOM, Config Mgmt) | Medium | RD-1 |
| **RD-4** | V&V + Credibility Model (ASME V&V 40) | 5 (Plan, Credibility, Verification Rpt, Validation Rpt, Uncertainty) | High | RD-1, RD-3 |
| **RD-5** | QMS Minimal (ISO 13485) | 7 (Quality Manual, Policy, Doc Control, Record Control, Audit, NC, CAPA) | Medium | RD-1 |
| **RD-6** | Clinical Evaluation | 5 (Plan, Report, Limitations, Reproducibility, Reference Data) | Medium | RD-4 |
| **RD-7** | Data Integrity + Release | 5 (Part 11, Data Integrity Policy, Audit Trail, Cybersecurity, Release Pkg) | Medium | RD-5 |

**Total: 37 new documents + 13 updates to partial documents = 50 deliverables.**

### 6.2 Execution Waves

```
Wave 0: RD-1 (foundation — unblocks everything)
Wave 1: RD-2 + RD-3 + RD-5 (parallel — risk, traceability, QMS)
Wave 2: RD-4 + RD-6 (parallel — V&V/credibility and clinical evaluation)
Wave 3: RD-7 (release package — closure)
```

### 6.3 Dependency Graph

```
RD-1 (foundation) ──┬──→ RD-2 (risk file)
                    ├──→ RD-3 (traceability + SOUP) ──→ RD-4 (V&V + credibility)
                    ├──→ RD-5 (QMS) ──────────────────→ RD-7 (data integrity + release)
                    └──→ RD-6 (clinical) ←── RD-4
```

### 6.4 Location

All documents reside under `docs/regulatory/` with subdirectories per sprint:

```
docs/regulatory/
├── 01_foundation/         (RD-1: 5 docs)
├── 02_risk_management/    (RD-2: 6 docs)
├── 03_traceability/       (RD-3: 4 docs)
├── 04_validation/         (RD-4: 5 docs)
├── 05_quality_system/     (RD-5: 7 docs)
├── 06_clinical/           (RD-6: 5 docs)
├── 07_release/            (RD-7: 5 docs)
└── AUDIT_CHECKLIST.md     (master index)
```

---

## 7. Decision Points

### 7.1 The Central Decision: Research Tool vs. SaMD

RESONANCE is currently a research tool. The decision to pursue SaMD classification is **not yet warranted** and should be deferred until at least one of the following triggers occurs.

### 7.2 Reclassification Triggers

| Trigger | Description | Evidence Required |
|---------|-------------|-------------------|
| **Clinical use claim** | Marketing or documentation claims RESONANCE can inform therapy selection for specific patients | Any statement implying patient-level applicability |
| **Pharma partnership with clinical intent** | A partner intends to use RESONANCE outputs to support a regulatory submission (e.g., IND, NDA supplement) | Contract specifying clinical decision support |
| **Direct patient data input** | RESONANCE is modified to accept patient tumor genomic profiles as input | Code change: patient data ingestion module |
| **Treatment recommendation output** | RESONANCE is modified to output specific drug recommendations or dosing | Code change: recommendation engine |
| **Regulatory inquiry** | FDA, competent authority, or Notified Body contacts the team regarding RESONANCE's classification | Official correspondence |

### 7.3 Decision Timeline

| Milestone | Action | Deadline |
|-----------|--------|----------|
| RD-1 complete | Formal Intended Use locked. Classification decision documented. | Before any external partnership |
| First pharma contact | Evaluate whether partner's use case triggers SaMD. If yes, initiate pre-submission. | At point of contact |
| Molecular resolution added | If RESONANCE adds molecular targets (EGFR, BCR-ABL), re-evaluate intended use. | At design decision |
| Patient data integration | SaMD classification is triggered. Begin De Novo preparation. | At design decision |
| Annual review | Re-evaluate positioning against IMDRF criteria regardless of triggers. | Every 12 months |

### 7.4 What Does NOT Trigger Reclassification

- Publishing the Zenodo paper (research dissemination, not clinical claims)
- Adding new abstract drug models (still abstract qe, not molecular)
- Increasing test count or validation experiments (strengthens research credibility, not clinical intent)
- Headless batch simulation for parameter sweeps (computational tool, not clinical workflow)
- Bozic/Gatenby/London qualitative comparisons (literature comparison, not patient prediction)

---

## 8. Strengths Inventory

These existing codebase assets accelerate regulatory documentation. Each represents work that does not need to be created from scratch.

### 8.1 Verification Evidence

| Asset | Location | Regulatory Value |
|-------|----------|-----------------|
| 3,113 automated tests | `cargo test` (36 sec) | IEC 62304 5.5-5.7: software unit/integration verification |
| Property-based conservation fuzz | `tests/property_conservation.rs` | ASME V&V 40 5: numerical verification under arbitrary inputs |
| Deterministic RNG (hash-based, no std::rand) | `src/blueprint/equations/determinism.rs` | Reproducibility guarantee (V&V 40 5, Part 11 11.10(b)) |
| 4 fundamental constants with derivation chain | `src/blueprint/equations/derived_thresholds.rs` (17 tests) | Traceability: all thresholds derived, not hardcoded |
| Zero `unsafe` policy | `CLAUDE.md` Hard Block 1 | Memory safety guarantee (IEC 62304 5.5.3) |

### 8.2 Validation Evidence

| Asset | Location | Regulatory Value |
|-------|----------|-----------------|
| Bozic 2013 5-arm validation (10/10 seeds) | `src/bin/bozic_validation.rs` | ASME V&V 40 6: validation against published data |
| 4 clinical calibration profiles | `README.md` | ASME V&V 40 6: multiple independent comparators |
| Published paper (Zenodo, 7 experiments) | DOI: 10.5281/zenodo.19342036 | Peer review evidence, literature context |
| Rosie case (canine MCT, partial response) | Commit `971c7ac`, `1e795c2` | Real-world case calibration with honest limitations |

### 8.3 Process Evidence

| Asset | Location | Regulatory Value |
|-------|----------|-----------------|
| Architecture documentation | `docs/ARCHITECTURE.md` | IEC 62304 5.3: architectural design |
| 6 detailed design specifications | `docs/design/*.md` | IEC 62304 5.4: detailed design |
| 4 module contracts | `docs/arquitectura/*.md` | Interface specifications |
| Sprint methodology with closure criteria | `docs/sprints/archive/` | ISO 13485 7.3: design control evidence |
| 17 coding rules + 5 absolute hard blocks | `CLAUDE.md` | IEC 62304 5.1: coding standards |
| 4 defined roles with responsibilities | `CLAUDE.md` Roles table | ISO 13485 5.5.1: responsibility and authority |
| Git immutable history | `.git/` | 21 CFR Part 11 11.10(e): audit trail |
| `Cargo.lock` dependency pinning | `Cargo.lock` | IEC 62304 8: configuration management |

### 8.4 Risk Control Evidence

| Asset | Location | Regulatory Value |
|-------|----------|-----------------|
| Pervasive disclaimers | README, paper, CLI outputs | ISO 14971 6: information for safety |
| Known limitations documented (5 in Rosie, paper Section 5) | `README.md`, paper | ISO 14971 7: residual risk disclosure |
| Abstract units (qe, not molar) prevent clinical misinterpretation | Entire codebase | ISO 14971 6: inherent safety by design |
| No patient data ingestion | Entire codebase (no I/O for patient records) | ISO 14971 6: inherent safety by design |

---

## 9. Risk of NOT Documenting

Even as a research tool, failing to create regulatory-grade documentation carries concrete risks.

### 9.1 Credibility Risk

- **Journal rejection.** Computational biology journals increasingly require V&V documentation. Without formal credibility assessment (ASME V&V 40), reviewers may dismiss RESONANCE as "just a simulation" despite 3,113 tests and Bozic validation.
- **arXiv endorsement.** The cs.NE endorsement process benefits from evidence of rigorous methodology. Formal documentation strengthens the case.

### 9.2 Partnership Risk

- **Pharma due diligence.** Pharmaceutical companies evaluating computational tools require ISO 13485 QMS evidence (or equivalent). Without it, RESONANCE is excluded from consideration regardless of technical merit.
- **CRO partnerships.** Contract research organizations working under GxP cannot use tools without documented validation.

### 9.3 Publication Risk

- **Reproducibility challenges.** Without a formal reproducibility protocol, independent researchers may fail to reproduce results due to environment differences, even though RESONANCE is deterministic. Formal documentation prevents this.
- **Citation disputes.** Without clear scope documentation, downstream users may cite RESONANCE as clinical evidence, creating reputational risk.

### 9.4 Liability Risk

- **Misuse without documentation.** If a researcher uses RESONANCE outputs to justify a clinical decision and harm results, the absence of formal disclaimers and risk analysis weakens the defense that misuse was foreseeable and mitigated.
- **Regulatory inquiry.** If a regulatory body questions RESONANCE's classification, the absence of a formal intended use statement and classification analysis creates ambiguity that formal documentation resolves.

### 9.5 Technical Debt Risk

- **Retroactive documentation is harder.** The further the codebase evolves without parallel documentation, the more expensive retroactive documentation becomes. The current moment (113K LOC, 3,113 tests, published paper, 4 calibration profiles) is the optimal time to document — the system is mature enough to be substantive but not so large that documentation is intractable.

---

## 10. Codebase References

The following files and directories are referenced throughout this strategy. All paths are relative to the repository root.

### Source Code

| File | Relevance |
|------|-----------|
| `src/blueprint/equations/determinism.rs` | Hash-based deterministic RNG — reproducibility foundation |
| `src/blueprint/equations/derived_thresholds.rs` | All lifecycle constants derived from 4 fundamentals (17 tests) |
| `src/blueprint/equations/pathway_inhibitor.rs` | Drug pathway model (11 pure functions, 32 tests) |
| `src/blueprint/constants/pathway_inhibitor.rs` | Pathway inhibitor constants (7 derived, 3 tests) |
| `src/use_cases/experiments/pathway_inhibitor_exp.rs` | Experiment harness + Bozic validation (18 tests) |
| `src/use_cases/experiments/cancer_therapy.rs` | Cytotoxic drug model (Hill pharmacokinetics) |
| `src/bin/bozic_validation.rs` | 10-seed Bozic 2013 validation binary |
| `src/bin/headless_sim.rs` | Headless simulation (PPM output, no GPU) |
| `src/batch/` | Batch simulator (33 systems, 156 tests, rayon parallel) |
| `tests/property_conservation.rs` | Property-based conservation fuzzing (proptest) |

### Documentation

| File | Relevance |
|------|-----------|
| `CLAUDE.md` | Coding standards, hard blocks, roles, axioms, constants |
| `docs/ARCHITECTURE.md` | Canonical architecture documentation |
| `docs/design/*.md` | 6 detailed design specifications |
| `docs/arquitectura/*.md` | 4 module contract documents |
| `docs/paper/resonance_arxiv.tex` | arXiv paper source (3 experiments, 11 references) |
| `docs/sprints/archive/` | Completed sprint documentation with closure evidence |
| `docs/sprints/REGULATORY_DOCUMENTATION/` | This regulatory track: 7 sprint definitions |

### Configuration

| File | Relevance |
|------|-----------|
| `Cargo.toml` | Direct dependencies (SOUP list) |
| `Cargo.lock` | Pinned dependency tree (SBOM source) |
| `README.md` | Disclaimers, clinical calibration profiles, known limitations |

---

## 11. Revision History

| Version | Date | Author | Changes |
|---------|------|--------|---------|
| 1.0 | 2026-04-02 | Resonance Development Team | Initial draft. Regulatory positioning, IMDRF classification, pathway analysis, gap analysis, roadmap. |
