---
document_id: RD-1.6
title: Medical Device File Index
standard: ISO 13485:2016 §4.2.3
version: 1.0
date: 2026-04-02
status: DRAFT
author: Resonance Development Team
approved_by: PENDING
review_date: PENDING
review_status: PENDING
---

# Medical Device File Index

## 1. Purpose

This document constitutes the Medical Device File (MDF / Expediente de Dispositivo) for RESONANCE, as required by ISO 13485:2016 §4.2.3. The MDF is a master index that points to all product-related documentation --- it does not duplicate content but provides a single navigable reference to every document in the regulatory file.

**Important:** RESONANCE is classified as a **research tool** (IMDRF SaMD Category I, IEC 62304 Class A). It is not currently regulated as a medical device. This MDF is maintained voluntarily as best practice for transparency, pharma partnership readiness, and future regulatory optionality. If RESONANCE were to be positioned as a medical device, this file would serve as the starting point for regulatory review.

**Codebase context:**
- Repository: `https://github.com/ResakaGit/RESONANCE`
- Commit: `971c7acb99decde45bf28860e6e10372718c51e2`
- LOC: ~113K | Tests: 3,113 | Language: Rust 2024 / Bevy 0.15 | License: AGPL-3.0

## 2. Product Description

### 2.1 Product Identity

| Field | Value |
|-------|-------|
| Product name | RESONANCE |
| Version | 0.1.0 (pre-1.0) |
| Description | Emergent life simulation engine where all behavior derives from energy interactions governed by 8 foundational axioms and 4 fundamental constants |
| Technology | Rust stable 2024 edition (MSRV 1.85), Bevy 0.15 ECS engine, glam 0.29 math |
| Operating model | Desktop workstation or HPC cluster. No network, no cloud, no SaaS. Fully offline. |
| Source | Open-source, AGPL-3.0, compiled locally by user |
| Paper | Zenodo DOI 10.5281/zenodo.19342036 |

### 2.2 What RESONANCE Does

RESONANCE simulates emergent life dynamics, therapeutic resistance evolution, and drug interaction strategies from first principles. It operates exclusively on abstract energy units (qe) --- not on molar concentrations, molecular structures, or patient-derived data. It produces qualitative predictions suitable for informing preclinical research directions.

**Key capabilities:**
- Emergent life from 14 orthogonal ECS layers composed by energy state
- Pathway inhibitor model (3 inhibition modes, Bliss independence for combinations)
- Cytotoxic drug model (Hill pharmacokinetics, quiescent stem cell escape)
- Bozic 2013 qualitative reproduction (combination > monotherapy, 10/10 seeds)
- 4 clinical calibration profiles (CML, prostate, NSCLC, canine MCT)
- Batch parallel simulator (millions of worlds, no GPU dependency)
- Bit-exact deterministic output (hash-based RNG, no external randomness)

### 2.3 What RESONANCE Is NOT

Not a clinical tool, not a drug discovery pipeline, not a substitute for oncology, not a pharmacokinetic model, not a molecular simulator, not a tumor microenvironment model. See RD-1.1 §5 for exhaustive negative scope with codebase evidence.

## 3. MDF Section Index

### 3.1 Intended Use and Classification

| Section | Document | Doc ID | File Path |
|---------|----------|--------|-----------|
| Intended use statement | Intended Use Statement | RD-1.1 | `docs/regulatory/01_foundation/INTENDED_USE.md` |
| Safety classification | Software Safety Classification | RD-1.2 | `docs/regulatory/01_foundation/SOFTWARE_SAFETY_CLASS.md` |
| Regulatory strategy | Regulatory Strategy | RD-1.5 | `docs/regulatory/01_foundation/REGULATORY_STRATEGY.md` |

**Classification summary:** IMDRF SaMD Category I (Inform, Non-serious). IEC 62304 Class A (no contribution to hazardous situation). Research-only intended use.

### 3.2 Requirements

| Section | Document | Doc ID | File Path |
|---------|----------|--------|-----------|
| Software requirements | Software Requirements Specification | RD-1.3 | `docs/regulatory/01_foundation/SOFTWARE_REQUIREMENTS_SPEC.md` |
| User requirements | User Requirements Specification | RD-4.6 | `docs/regulatory/04_validation/USER_REQUIREMENTS_SPEC.md` |

**Requirements summary:** 17 functional requirements (RF-01 through RF-17), 3 performance requirements (RP-01 through RP-03), 5 safety requirements (RS-01 through RS-05), 3 interface requirements (RI-01 through RI-03). All traced to axioms and implementation files.

### 3.3 Design and Architecture

| Section | Document | Doc ID | File Path |
|---------|----------|--------|-----------|
| Software development plan | Software Development Plan | RD-1.4 | `docs/regulatory/01_foundation/SOFTWARE_DEVELOPMENT_PLAN.md` |
| Architecture (canonical) | ARCHITECTURE.md | --- | `docs/ARCHITECTURE.md` |
| Coding standards | CLAUDE.md | --- | `CLAUDE.md` |
| Design specifications | Design docs | --- | `docs/design/*.md` (6 files) |
| Module contracts | Contract docs | --- | `docs/arquitectura/*.md` (4 files) |

**Architecture summary:** 14 orthogonal ECS layers (L0 BaseEnergy through L13 StructuralLink). FixedUpdate pipeline with 6 Phases. Pure math in `blueprint/equations/` (45+ domain files). Stateless-first design. Sprint-based iterative development with 4 roles.

### 3.4 Verification and Validation

| Section | Document | Doc ID | File Path |
|---------|----------|--------|-----------|
| Validation plan | Validation Plan | RD-4.1 | `docs/regulatory/04_validation/VALIDATION_PLAN.md` |
| Credibility model | Credibility Model | RD-4.2 | `docs/regulatory/04_validation/CREDIBILITY_MODEL.md` |
| Verification report | Verification Report | RD-4.3 | `docs/regulatory/04_validation/VERIFICATION_REPORT.md` |
| Validation report | Validation Report | RD-4.4 | `docs/regulatory/04_validation/VALIDATION_REPORT.md` |
| Uncertainty analysis | Uncertainty Analysis | RD-4.5 | `docs/regulatory/04_validation/UNCERTAINTY_ANALYSIS.md` |
| User requirements spec | User Requirements Specification | RD-4.6 | `docs/regulatory/04_validation/USER_REQUIREMENTS_SPEC.md` |

**V&V summary:** 3,113 automated tests (0 failures). Unit tests (pure math in blueprint/equations/), integration tests (MinimalPlugins app), property tests (proptest fuzzing for conservation invariants), batch tests (156+ tests, 33 systems). Bozic 2013 validated 10/10 seeds. 4 calibration profiles with 21 tests.

### 3.5 Risk Management

| Section | Document | Doc ID | File Path |
|---------|----------|--------|-----------|
| Risk management plan | Risk Management Plan | RD-2.1 | `docs/regulatory/02_risk_management/RISK_MANAGEMENT_PLAN.md` |
| Risk analysis | Risk Analysis | RD-2.2 | `docs/regulatory/02_risk_management/RISK_ANALYSIS.md` |
| Risk evaluation | Risk Evaluation | RD-2.3 | `docs/regulatory/02_risk_management/RISK_EVALUATION.md` |
| Risk controls | Risk Controls | RD-2.4 | `docs/regulatory/02_risk_management/RISK_CONTROLS.md` |
| Residual risk | Residual Risk Assessment | RD-2.5 | `docs/regulatory/02_risk_management/RESIDUAL_RISK.md` |
| Risk management report | Risk Management Report | RD-2.6 | `docs/regulatory/02_risk_management/RISK_MANAGEMENT_REPORT.md` |
| Post-production monitoring | Post-Production Monitoring Plan | RD-2.7 | `docs/regulatory/02_risk_management/POST_PRODUCTION_MONITORING.md` |

**Risk summary:** 12 hazards identified via software FMEA. 52 controls implemented. Post-control: 8 Acceptable, 4 ALARP, 0 Unacceptable. Overall residual risk acceptable for research-only use.

### 3.6 Traceability and Configuration

| Section | Document | Doc ID | File Path |
|---------|----------|--------|-----------|
| Traceability matrix | Traceability Matrix | RD-3.1 | `docs/regulatory/03_traceability/TRACEABILITY_MATRIX.md` |
| SOUP analysis | SOUP Analysis | RD-3.2 | `docs/regulatory/03_traceability/SOUP_ANALYSIS.md` |
| SBOM | Software Bill of Materials | RD-3.3 | `docs/regulatory/03_traceability/SBOM.md` |
| Configuration management | Configuration Management Plan | RD-3.4 | `docs/regulatory/03_traceability/CONFIGURATION_MANAGEMENT.md` |

**Traceability summary:** Requirements (RF/RP/RS/RI) traced to implementation files and verification tests. 14 runtime dependencies pinned via Cargo.lock. 25 binary targets. Git-based configuration management.

### 3.7 Labeling and Instructions for Use

| Section | Document | Location |
|---------|----------|----------|
| Quick start | README.md §Getting Started | `README.md` |
| Disclaimers | README.md lines 18--22 | `README.md` |
| Validation status | README.md line 141 | `README.md` |
| Headless usage | README.md + `src/bin/headless_sim.rs` | CLI `--help` |
| Map configuration | `assets/maps/*.ron` | `src/worldgen/map_config.rs` |
| Coding standards (developer) | CLAUDE.md | `CLAUDE.md` |

**Gap acknowledged:** There is no formal "Instructions for Use" (IFU) document separate from README.md. For a research tool distributed as source code, the README serves as the de facto IFU. If RESONANCE were reclassified as a medical device, a standalone IFU conforming to IEC 62366-1 (usability engineering) would be required.

### 3.8 Quality Management System

| Section | Document | Doc ID | File Path |
|---------|----------|--------|-----------|
| Quality manual | Quality Manual | RD-5.1 | `docs/regulatory/05_quality_system/QUALITY_MANUAL.md` |
| Quality policy | Quality Policy | RD-5.2 | `docs/regulatory/05_quality_system/QUALITY_POLICY.md` |
| Document control | Document Control Procedure | RD-5.3 | `docs/regulatory/05_quality_system/DOCUMENT_CONTROL.md` |
| Record control | Record Control Procedure | RD-5.4 | `docs/regulatory/05_quality_system/RECORD_CONTROL.md` |
| Internal audit | Internal Audit Procedure | RD-5.5 | `docs/regulatory/05_quality_system/INTERNAL_AUDIT.md` |
| Nonconforming product | Nonconforming Product Procedure | RD-5.6 | `docs/regulatory/05_quality_system/NONCONFORMING_PRODUCT.md` |
| CAPA | CAPA Procedure | RD-5.7 | `docs/regulatory/05_quality_system/CAPA_PROCEDURE.md` |
| Competence records | Personnel Competence and Training Records | RD-5.8 | `docs/regulatory/05_quality_system/COMPETENCE_RECORDS.md` |

### 3.9 Clinical Evaluation

| Section | Document | Doc ID | File Path |
|---------|----------|--------|-----------|
| Clinical evaluation plan | Clinical Evaluation Plan | RD-6.1 | `docs/regulatory/06_clinical/CLINICAL_EVALUATION_PLAN.md` |
| Clinical evaluation report | Clinical Evaluation Report | RD-6.2 | `docs/regulatory/06_clinical/CLINICAL_EVALUATION_REPORT.md` |
| Limitations report | Limitations Report | RD-6.3 | `docs/regulatory/06_clinical/LIMITATIONS_REPORT.md` |
| Reproducibility protocol | Reproducibility Protocol | RD-6.4 | `docs/regulatory/06_clinical/REPRODUCIBILITY_PROTOCOL.md` |
| Reference data registry | Reference Data Registry | RD-6.5 | `docs/regulatory/06_clinical/REFERENCE_DATA_REGISTRY.md` |

**Clinical summary:** RESONANCE is a research tool, not a clinical device. Clinical evaluation documents are maintained for scientific credibility and pharma partnership readiness. Bozic 2013 comparison is qualitative (suppression percentages, not absolute cell counts). Calibration profiles are not validated against patient outcomes.

### 3.10 Regulatory Compliance and Data Integrity

| Section | Document | Doc ID | File Path |
|---------|----------|--------|-----------|
| Part 11 compliance | Part 11 Compliance Assessment | RD-7.1 | `docs/regulatory/07_release/PART11_COMPLIANCE.md` |
| Data integrity | Data Integrity Policy | RD-7.2 | `docs/regulatory/07_release/DATA_INTEGRITY_POLICY.md` |
| Audit trail | Audit Trail Procedure | RD-7.3 | `docs/regulatory/07_release/AUDIT_TRAIL.md` |
| Cybersecurity | Cybersecurity Plan | RD-7.4 | `docs/regulatory/07_release/CYBERSECURITY_PLAN.md` |
| Release package | Release Package Definition | RD-7.5 | `docs/regulatory/07_release/RELEASE_PACKAGE.md` |

### 3.11 Software Maintenance and Problem Resolution

| Section | Document | Doc ID | File Path |
|---------|----------|--------|-----------|
| Maintenance plan | Software Maintenance Plan | RD-1.7 | `docs/regulatory/01_foundation/SOFTWARE_MAINTENANCE_PLAN.md` |
| Problem resolution | Software Problem Resolution Process | RD-1.8 | `docs/regulatory/01_foundation/PROBLEM_RESOLUTION.md` |

### 3.12 Audit Index

| Section | Document | Doc ID | File Path |
|---------|----------|--------|-----------|
| Audit checklist | Regulatory Documentation Audit Checklist | RD-INDEX | `docs/regulatory/AUDIT_CHECKLIST.md` |

## 4. Document Count Summary

| Sprint | Category | Documents | Status |
|--------|----------|-----------|--------|
| RD-1 | Foundation | 8 (original 5 + RD-1.6, RD-1.7, RD-1.8) | Complete |
| RD-2 | Risk Management | 7 (original 6 + RD-2.7) | Complete |
| RD-3 | Traceability | 4 | Complete |
| RD-4 | Validation | 6 (original 5 + RD-4.6) | Complete |
| RD-5 | Quality System | 8 (original 7 + RD-5.8) | Complete |
| RD-6 | Clinical Evaluation | 5 | Complete |
| RD-7 | Data Integrity + Release | 5 | Complete |
| --- | Audit Index | 1 | Complete |
| **Total** | | **44** (37 original + 6 audit gap + 1 index) | |

## 5. Known Gaps

| Gap | Severity | Resolution Path |
|-----|----------|-----------------|
| No formal Instructions for Use (IFU) | Low | README.md serves as de facto IFU. Standalone IFU required only if reclassified as medical device. |
| No formal approval signatures on any document | Medium | All 44 documents are DRAFT. Digital signature workflow not yet implemented. |
| No IEC 62366-1 usability engineering file | Low | Not required for Class A research tool. Required if clinical use pursued. |
| No EU MDR Annex I (GSPR) systematic mapping | Low | Required only if EU market entry planned. |
| MDF is voluntary | Informational | RESONANCE is not currently regulated. MDF maintained for transparency and readiness. |

## 6. Revision History

| Version | Date | Author | Changes |
|---------|------|--------|---------|
| 1.0 | 2026-04-02 | Resonance Development Team | Initial MDF index. 44 documents cataloged across 7 sprints + audit index. All claims traced to codebase at commit `971c7ac`. |
