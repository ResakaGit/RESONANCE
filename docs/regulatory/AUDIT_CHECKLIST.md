---
document_id: RD-INDEX
title: Regulatory Documentation Audit Checklist
version: 1.0
date: 2026-04-02
status: DRAFT
author: Resonance Development Team
---

# Regulatory Documentation Audit Checklist

## 1. Purpose

This is the **master audit index** for all regulatory documentation in the RESONANCE project. It provides a single point of reference for auditors, reviewers, and development team members to verify completeness of the documentation set across all 7 sprints (37 documents).

Related documents:
- **Sprint plan:** `docs/sprints/REGULATORY_DOCUMENTATION/README.md`
- **Architecture:** `docs/ARCHITECTURE.md`
- **Codebase instructions:** `CLAUDE.md`

---

## 2. Document Inventory

### RD-1: Foundation (8 documents)

| Doc ID | Title | Standard | File Path | Status | Notes |
|--------|-------|----------|-----------|--------|-------|
| RD-1.1 | Intended Use Statement | IMDRF SaMD N10, IEC 62304 S4.1 | `docs/regulatory/01_foundation/INTENDED_USE.md` | DONE | Research-only positioning established |
| RD-1.2 | Software Safety Classification | IEC 62304 S4.3, IMDRF SaMD Risk | `docs/regulatory/01_foundation/SOFTWARE_SAFETY_CLASS.md` | DONE | Class A (no patient harm pathway) |
| RD-1.3 | Software Requirements Specification | IEC 62304 S5.2 | `docs/regulatory/01_foundation/SOFTWARE_REQUIREMENTS_SPEC.md` | DONE | Requirements traced to axioms |
| RD-1.4 | Software Development Plan | IEC 62304 S5.1 | `docs/regulatory/01_foundation/SOFTWARE_DEVELOPMENT_PLAN.md` | DONE | Iterative development model |
| RD-1.5 | Regulatory Strategy | IMDRF SaMD Framework, FDA De Novo | `docs/regulatory/01_foundation/REGULATORY_STRATEGY.md` | DONE | Roadmap from research to SaMD |
| RD-1.6 | Medical Device File Index | ISO 13485 S4.2.3 | `docs/regulatory/01_foundation/MEDICAL_DEVICE_FILE.md` | DONE | Master index to all product documentation |
| RD-1.7 | Software Maintenance Plan | IEC 62304 S6 | `docs/regulatory/01_foundation/SOFTWARE_MAINTENANCE_PLAN.md` | DONE | Sprint-based maintenance, regression testing |
| RD-1.8 | Software Problem Resolution Process | IEC 62304 S9 | `docs/regulatory/01_foundation/PROBLEM_RESOLUTION.md` | DONE | Severity classification, investigation, resolution |

### RD-2: Risk Management (7 documents)

| Doc ID | Title | Standard | File Path | Status | Notes |
|--------|-------|----------|-----------|--------|-------|
| RD-2.1 | Risk Management Plan | ISO 14971 S3-S4 | `docs/regulatory/02_risk_management/RISK_MANAGEMENT_PLAN.md` | DONE | Scope, criteria, responsibilities |
| RD-2.2 | Risk Analysis | ISO 14971 S5, Annex C | `docs/regulatory/02_risk_management/RISK_ANALYSIS.md` | DONE | Hazard identification + estimation |
| RD-2.3 | Risk Evaluation | ISO 14971 S6 | `docs/regulatory/02_risk_management/RISK_EVALUATION.md` | DONE | Acceptability determination |
| RD-2.4 | Risk Controls | ISO 14971 S7 | `docs/regulatory/02_risk_management/RISK_CONTROLS.md` | DONE | Mitigation measures |
| RD-2.5 | Residual Risk Assessment | ISO 14971 S8 | `docs/regulatory/02_risk_management/RESIDUAL_RISK.md` | DONE | Post-control risk levels |
| RD-2.6 | Risk Management Report | ISO 14971 S9 | `docs/regulatory/02_risk_management/RISK_MANAGEMENT_REPORT.md` | DONE | Summary of risk file |
| RD-2.7 | Post-Production Monitoring Plan | ISO 14971 S9 | `docs/regulatory/02_risk_management/POST_PRODUCTION_MONITORING.md` | DONE | Monitoring channels, evaluation criteria, review frequency |

### RD-3: Traceability (4 documents)

| Doc ID | Title | Standard | File Path | Status | Notes |
|--------|-------|----------|-----------|--------|-------|
| RD-3.1 | Traceability Matrix | IEC 62304 S5.6, ISO 13485 S7.3.7 | `docs/regulatory/03_traceability/TRACEABILITY_MATRIX.md` | DONE | Requirement -> code -> test links |
| RD-3.2 | SOUP Analysis | IEC 62304 S8 | `docs/regulatory/03_traceability/SOUP_ANALYSIS.md` | DONE | Third-party component risk |
| RD-3.3 | Software Bill of Materials | NTIA SBOM minimum, IEC 62304 S8 | `docs/regulatory/03_traceability/SBOM.md` | DONE | Full dependency inventory |
| RD-3.4 | Configuration Management Plan | IEC 62304 S5.1.9, ISO 13485 S4.2.4 | `docs/regulatory/03_traceability/CONFIGURATION_MANAGEMENT.md` | DONE | Git-based CM procedures |

### RD-4: Validation (6 documents)

| Doc ID | Title | Standard | File Path | Status | Notes |
|--------|-------|----------|-----------|--------|-------|
| RD-4.1 | Validation Plan | ASME V&V 40 S5, GAMP 5 | `docs/regulatory/04_validation/VALIDATION_PLAN.md` | DONE | Scope, strategy, acceptance criteria |
| RD-4.2 | Credibility Model | ASME V&V 40:2018, FDA CMS Guidance 2023 | `docs/regulatory/04_validation/CREDIBILITY_MODEL.md` | DONE | Context of Use assessment |
| RD-4.3 | Verification Report | IEC 62304 S5.5-S5.7, GAMP 5 | `docs/regulatory/04_validation/VERIFICATION_REPORT.md` | DONE | Unit/integration/property test evidence |
| RD-4.4 | Validation Report | ASME V&V 40, FDA CMS Guidance | `docs/regulatory/04_validation/VALIDATION_REPORT.md` | DONE | Bozic validation + calibration |
| RD-4.5 | Uncertainty Analysis | ASME V&V 40 S7, GUM | `docs/regulatory/04_validation/UNCERTAINTY_ANALYSIS.md` | DONE | Parameter sensitivity + epistemic gaps |
| RD-4.6 | User Requirements Specification | GAMP 5 2nd Ed. | `docs/regulatory/04_validation/USER_REQUIREMENTS_SPEC.md` | DONE | 7 user needs mapped to RF/RP/RS/RI requirements |

### RD-5: Quality System (8 documents)

| Doc ID | Title | Standard | File Path | Status | Notes |
|--------|-------|----------|-----------|--------|-------|
| RD-5.1 | Quality Manual | ISO 13485 S4.2.2 | `docs/regulatory/05_quality_system/QUALITY_MANUAL.md` | DONE | QMS overview |
| RD-5.2 | Quality Policy | ISO 13485 S5.3 | `docs/regulatory/05_quality_system/QUALITY_POLICY.md` | DONE | Policy statement + objectives |
| RD-5.3 | Document Control Procedure | ISO 13485 S4.2.4 | `docs/regulatory/05_quality_system/DOCUMENT_CONTROL.md` | DONE | Approval, versioning, archival |
| RD-5.4 | Record Control Procedure | ISO 13485 S4.2.5 | `docs/regulatory/05_quality_system/RECORD_CONTROL.md` | DONE | Retention, access, disposal |
| RD-5.5 | Internal Audit Procedure | ISO 13485 S8.2.4 | `docs/regulatory/05_quality_system/INTERNAL_AUDIT.md` | DONE | Audit schedule + criteria |
| RD-5.6 | Nonconforming Product | ISO 13485 S8.3 | `docs/regulatory/05_quality_system/NONCONFORMING_PRODUCT.md` | DONE | NC detection + disposition |
| RD-5.7 | CAPA Procedure | ISO 13485 S8.5.2-S8.5.3 | `docs/regulatory/05_quality_system/CAPA_PROCEDURE.md` | DONE | Corrective/preventive actions |
| RD-5.8 | Personnel Competence and Training Records | ISO 13485 S6.2, ISO 14971 S4.3 | `docs/regulatory/05_quality_system/COMPETENCE_RECORDS.md` | DONE | Competence per role, training sources, gaps |

### RD-6: Clinical Evaluation (5 documents)

| Doc ID | Title | Standard | File Path | Status | Notes |
|--------|-------|----------|-----------|--------|-------|
| RD-6.1 | Clinical Evaluation Plan | MDR Annex XIV, MEDDEV 2.7/1 rev 4 | `docs/regulatory/06_clinical/CLINICAL_EVALUATION_PLAN.md` | DONE | Evaluation scope + methodology |
| RD-6.2 | Clinical Evaluation Report | MDR Annex XIV, MEDDEV 2.7/1 rev 4 | `docs/regulatory/06_clinical/CLINICAL_EVALUATION_REPORT.md` | DONE | Literature + simulation evidence |
| RD-6.3 | Limitations Report | FDA CMS Guidance S6, ASME V&V 40 | `docs/regulatory/06_clinical/LIMITATIONS_REPORT.md` | DONE | Honest scope boundaries |
| RD-6.4 | Reproducibility Protocol | ASME V&V 40 S6, FDA CMS Guidance | `docs/regulatory/06_clinical/REPRODUCIBILITY_PROTOCOL.md` | DONE | Deterministic reproducibility |
| RD-6.5 | Reference Data Registry | ASME V&V 40 S5.4, ICH E9 | `docs/regulatory/06_clinical/REFERENCE_DATA_REGISTRY.md` | DONE | Comparator data catalog |

### RD-7: Data Integrity + Release (5 documents)

| Doc ID | Title | Standard | File Path | Status | Notes |
|--------|-------|----------|-----------|--------|-------|
| RD-7.1 | Part 11 Compliance Assessment | 21 CFR Part 11, EU Annex 11 | `docs/regulatory/07_release/PART11_COMPLIANCE.md` | DONE | All §11.10 subsections assessed |
| RD-7.2 | Data Integrity Policy | ALCOA+ (WHO/EMA/FDA harmonized) | `docs/regulatory/07_release/DATA_INTEGRITY_POLICY.md` | DONE | All 9 ALCOA+ principles mapped |
| RD-7.3 | Audit Trail Procedure | 21 CFR Part 11 S11.10(e) | `docs/regulatory/07_release/AUDIT_TRAIL.md` | DONE | Git-based trail + review procedure |
| RD-7.4 | Cybersecurity Plan | FDA S524B FD&C, IMDRF Cybersecurity | `docs/regulatory/07_release/CYBERSECURITY_PLAN.md` | DONE | STRIDE threat model, no PHI/PII |
| RD-7.5 | Release Package | IEC 62304 S5.8 | `docs/regulatory/07_release/RELEASE_PACKAGE.md` | DONE | 12 release criteria, semver |

---

## 3. Summary Statistics

| Sprint | Description | Documents | Created | Pending | Completion |
|--------|-------------|-----------|---------|---------|------------|
| RD-1 | Foundation | 8 | 8 | 0 | 100% |
| RD-2 | Risk Management | 7 | 7 | 0 | 100% |
| RD-3 | Traceability | 4 | 4 | 0 | 100% |
| RD-4 | Validation | 6 | 6 | 0 | 100% |
| RD-5 | Quality System | 8 | 8 | 0 | 100% |
| RD-6 | Clinical Evaluation | 5 | 5 | 0 | 100% |
| RD-7 | Data Integrity + Release | 5 | 5 | 0 | 100% |
| **Total** | | **43** | **43** | **0** | **100%** |

---

## 4. Standards Coverage Matrix

### 4.1 IEC 62304:2006+Amd1:2015 (Medical Device Software -- Software Life Cycle Processes)

| Clause | Description | Covered By |
|--------|-------------|------------|
| S4.1 | Quality management system | RD-5.1, RD-5.2 |
| S4.3 | Software safety classification | RD-1.2 |
| S5.1 | Software development planning | RD-1.4 |
| S5.1.9 | Configuration management | RD-3.4 |
| S5.2 | Software requirements analysis | RD-1.3 |
| S5.3 | Software architectural design | RD-1.3 (refs ARCHITECTURE.md) |
| S5.4 | Software detailed design | RD-1.3 (refs design/*.md) |
| S5.5 | Software unit implementation and verification | RD-4.3 |
| S5.6 | Software integration and integration testing | RD-4.3, RD-3.1 |
| S5.7 | Software system testing | RD-4.3, RD-4.4 |
| S5.8 | Software release | RD-7.5 |
| S6 | Software maintenance | RD-1.7 (dedicated maintenance plan) |
| S7 | Software risk management | RD-2.1 through RD-2.7 |
| S8 | Software of unknown provenance (SOUP) | RD-3.2, RD-3.3 |
| S9 | Software problem resolution | RD-1.8, RD-3.4 |

### 4.2 ISO 14971:2019 (Medical Devices -- Application of Risk Management)

| Clause | Description | Covered By |
|--------|-------------|------------|
| S3 | General requirements for risk management | RD-2.1 |
| S4 | Risk analysis | RD-2.1, RD-2.2 |
| S5 | Risk evaluation | RD-2.3 |
| S6 | Risk control | RD-2.4 |
| S7 | Evaluation of overall residual risk | RD-2.5 |
| S8 | Risk management review | RD-2.6 |
| S9 | Production and post-production activities | RD-2.7 (dedicated post-production monitoring plan) |
| S10 | Risk management file | RD-2.1 through RD-2.7 (complete set) |
| Annex C | Hazard identification (informative) | RD-2.2 |

### 4.3 ISO 13485:2016 (Medical Devices -- Quality Management Systems)

| Clause | Description | Covered By |
|--------|-------------|------------|
| S4.1 | Quality management system | RD-5.1 |
| S4.2.2 | Quality manual | RD-5.1 |
| S4.2.3 | Medical device file | RD-1.6 |
| S4.2.4 | Document control | RD-5.3, RD-3.4 |
| S4.2.5 | Record control | RD-5.4 |
| S5.3 | Quality policy | RD-5.2 |
| S5.4 | Quality objectives | RD-5.2 |
| S5.6 | Management review | RD-5.5 |
| S6.2 | Human resources --- competence, training | RD-5.8 |
| S7.1 | Planning of product realization | RD-1.4 |
| S7.3 | Design and development | RD-1.3, RD-1.4 |
| S7.3.7 | Design traceability | RD-3.1 |
| S7.4 | Purchasing (SOUP) | RD-3.2, RD-3.3 |
| S8.2.4 | Internal audit | RD-5.5 |
| S8.3 | Control of nonconforming product | RD-5.6 |
| S8.5.2 | Corrective action | RD-5.7 |
| S8.5.3 | Preventive action | RD-5.7 |

### 4.4 ASME V&V 40:2018 (Assessing Credibility of Computational Modeling and Simulation)

| Section | Description | Covered By |
|---------|-------------|------------|
| S4 | Context of use | RD-4.2 |
| S5 | Model credibility assessment planning | RD-4.1, RD-4.2 |
| S5.4 | Reference data requirements | RD-6.5 |
| S6 | Verification and validation activities | RD-4.3, RD-4.4, RD-6.4 |
| S7 | Uncertainty quantification | RD-4.5 |
| S8 | Model adequacy assessment | RD-4.2, RD-6.3 |

### 4.5 21 CFR Part 11 (Electronic Records; Electronic Signatures)

| Section | Description | Covered By |
|---------|-------------|------------|
| S11.10(a) | Validation of systems | RD-7.1, RD-4.3 |
| S11.10(b) | Accurate record generation | RD-7.1 |
| S11.10(c) | Record protection | RD-7.1, RD-7.2 |
| S11.10(d) | Limiting system access | RD-7.1 |
| S11.10(e) | Secure audit trails | RD-7.3 |
| S11.10(f) | Operational checks | RD-7.1 |
| S11.10(g) | Authority checks | RD-7.1 |
| S11.10(i) | Training documentation | RD-7.1 |
| S11.10(k) | System documentation | RD-7.1 |
| S11.50 | Signature manifestation | RD-7.1 |
| S11.70 | Signature/record linking | RD-7.1 |

### 4.6 IMDRF SaMD (Software as a Medical Device)

| Document | Description | Covered By |
|----------|-------------|------------|
| N10R4:2013 | SaMD definition and key definitions | RD-1.1, RD-1.5 |
| N12R2:2014 | SaMD possible framework | RD-1.2, RD-1.5 |
| N23:2015 | SaMD application of QMS | RD-5.1, RD-1.5 |
| N41:2017 | SaMD clinical evaluation | RD-6.1, RD-6.2 |
| Cybersecurity | Principles and practices | RD-7.4 |

### 4.7 GAMP 5 (2nd Edition -- A Risk-Based Approach to Compliant GxP Computerized Systems)

| Concept | Description | Covered By |
|---------|-------------|------------|
| Software categorization | Category 1 (infrastructure), 3 (non-configured), 5 (custom) | RD-1.2 |
| User requirements specification | User needs mapped to system requirements | RD-4.6 |
| V-model life cycle | Planning, specification, verification, reporting | RD-1.4, RD-4.1, RD-4.3 |
| Risk-based testing | Risk assessment drives test intensity | RD-2.1, RD-4.1 |
| Data integrity | ALCOA+ principles | RD-7.2 |
| Operational phase | Change control, periodic review | RD-5.3, RD-5.5 |
| Supplier assessment | SOUP/third-party qualification | RD-3.2 |

### 4.8 FDA CMS Guidance (Assessing Credibility of Computational Modeling and Simulation -- 2023)

| Section | Description | Covered By |
|---------|-------------|------------|
| Context of Use | COU definition and risk-informed assessment | RD-4.2 |
| Verification | Code verification evidence | RD-4.3 |
| Validation | Comparator data and model-reality agreement | RD-4.4, RD-6.5 |
| Uncertainty quantification | Sensitivity analysis + parameter uncertainty | RD-4.5 |
| Applicability | Domain of validity and extrapolation limits | RD-4.2, RD-6.3 |
| Adequacy | Overall model fitness-for-purpose | RD-4.2, RD-6.2 |

---

## 5. Known Gaps

### 5.1 Resolved Gaps (Audit Supplement)

The following gaps from the original 37-document set have been addressed by 6 supplementary documents:

| Gap | Resolved By | Doc ID |
|-----|-------------|--------|
| No medical device file (ISO 13485 S4.2.3) | Medical Device File Index | RD-1.6 |
| No software maintenance plan (IEC 62304 S6) | Software Maintenance Plan | RD-1.7 |
| No software problem resolution (IEC 62304 S9) | Software Problem Resolution Process | RD-1.8 |
| No post-production monitoring (ISO 14971 S9) | Post-Production Monitoring Plan | RD-2.7 |
| No user requirements specification (GAMP 5) | User Requirements Specification | RD-4.6 |
| No competence/training records (ISO 13485 S6.2) | Personnel Competence and Training Records | RD-5.8 |

### 5.2 Structural Gaps Across All Documents

| Gap | Severity | Affected Documents | Resolution |
|-----|----------|--------------------|------------|
| No formal approval signatures | Medium | All 43 | Implement document approval workflow (digital signatures or Git-tag-based approval) |
| No review records | Medium | All 43 | Add formal review log per document (reviewer, date, disposition) |
| No GPG commit signing enforced | Low | RD-7.1, RD-7.3 | Enable GPG signing in repository settings |
| No automated CI/CD pipeline | Low | RD-7.5 | Implement GitHub Actions for `cargo test` + `cargo audit` |
| No formal change control board | Medium | RD-5.3 | Define CCB membership and decision authority |
| Abstract units only (qe, not molar) | Informational | RD-6.2, RD-6.3 | Fundamental design decision -- not a gap to close but a scope boundary to communicate |
| No formal training certificates | Low | RD-5.8 | Competence-through-delivery model documented; formal training if reclassified |
| Monitoring channels defined but not active | Low | RD-2.7 | Pre-1.0 with 0 users; first review Q3 2026 or first external issue |

### 5.3 Standards Coverage Gaps

| Standard | Gap | Resolution Path |
|----------|-----|-----------------|
| IEC 82304-1 (Health software) | Not addressed | Low priority -- applies if RESONANCE becomes a standalone health software product |
| ICH Q9/Q10 | Partially covered via GAMP 5 alignment | Full coverage if pharma deployment pursued |
| EU MDR Annex I (GSPR) | Not systematically mapped | Required only if EU market entry planned |
| ISO/IEC 25010:2023 (Software quality) | Not explicitly referenced | Can be mapped to existing SRS (RD-1.3) |
| GAMP 5 IQ/OQ/PQ protocols | Not addressed as separate documents | IQ covered by RD-6.4 (Reproducibility Protocol: build + install + verify). OQ covered by RD-4.3 (Verification Report: 3,113 tests = operational qualification). PQ covered by RD-4.4 (Validation Report: Bozic + calibration = performance qualification). Formal IQ/OQ/PQ separation only required for GxP-regulated deployment environments. |

---

## 6. Verification Commands

Commands to verify all regulatory documentation files exist on disk:

```bash
# Sprint RD-1: Foundation (8 files)
ls -la docs/regulatory/01_foundation/INTENDED_USE.md
ls -la docs/regulatory/01_foundation/SOFTWARE_SAFETY_CLASS.md
ls -la docs/regulatory/01_foundation/SOFTWARE_REQUIREMENTS_SPEC.md
ls -la docs/regulatory/01_foundation/SOFTWARE_DEVELOPMENT_PLAN.md
ls -la docs/regulatory/01_foundation/REGULATORY_STRATEGY.md
ls -la docs/regulatory/01_foundation/MEDICAL_DEVICE_FILE.md
ls -la docs/regulatory/01_foundation/SOFTWARE_MAINTENANCE_PLAN.md
ls -la docs/regulatory/01_foundation/PROBLEM_RESOLUTION.md

# Sprint RD-2: Risk Management (7 files)
ls -la docs/regulatory/02_risk_management/RISK_MANAGEMENT_PLAN.md
ls -la docs/regulatory/02_risk_management/RISK_ANALYSIS.md
ls -la docs/regulatory/02_risk_management/RISK_EVALUATION.md
ls -la docs/regulatory/02_risk_management/RISK_CONTROLS.md
ls -la docs/regulatory/02_risk_management/RESIDUAL_RISK.md
ls -la docs/regulatory/02_risk_management/RISK_MANAGEMENT_REPORT.md
ls -la docs/regulatory/02_risk_management/POST_PRODUCTION_MONITORING.md

# Sprint RD-3: Traceability (4 files)
ls -la docs/regulatory/03_traceability/TRACEABILITY_MATRIX.md
ls -la docs/regulatory/03_traceability/SOUP_ANALYSIS.md
ls -la docs/regulatory/03_traceability/SBOM.md
ls -la docs/regulatory/03_traceability/CONFIGURATION_MANAGEMENT.md

# Sprint RD-4: Validation (6 files)
ls -la docs/regulatory/04_validation/VALIDATION_PLAN.md
ls -la docs/regulatory/04_validation/CREDIBILITY_MODEL.md
ls -la docs/regulatory/04_validation/VERIFICATION_REPORT.md
ls -la docs/regulatory/04_validation/VALIDATION_REPORT.md
ls -la docs/regulatory/04_validation/UNCERTAINTY_ANALYSIS.md
ls -la docs/regulatory/04_validation/USER_REQUIREMENTS_SPEC.md

# Sprint RD-5: Quality System (8 files)
ls -la docs/regulatory/05_quality_system/QUALITY_MANUAL.md
ls -la docs/regulatory/05_quality_system/QUALITY_POLICY.md
ls -la docs/regulatory/05_quality_system/DOCUMENT_CONTROL.md
ls -la docs/regulatory/05_quality_system/RECORD_CONTROL.md
ls -la docs/regulatory/05_quality_system/INTERNAL_AUDIT.md
ls -la docs/regulatory/05_quality_system/NONCONFORMING_PRODUCT.md
ls -la docs/regulatory/05_quality_system/CAPA_PROCEDURE.md
ls -la docs/regulatory/05_quality_system/COMPETENCE_RECORDS.md

# Sprint RD-6: Clinical Evaluation (5 files)
ls -la docs/regulatory/06_clinical/CLINICAL_EVALUATION_PLAN.md
ls -la docs/regulatory/06_clinical/CLINICAL_EVALUATION_REPORT.md
ls -la docs/regulatory/06_clinical/LIMITATIONS_REPORT.md
ls -la docs/regulatory/06_clinical/REPRODUCIBILITY_PROTOCOL.md
ls -la docs/regulatory/06_clinical/REFERENCE_DATA_REGISTRY.md

# Sprint RD-7: Data Integrity + Release (5 files)
ls -la docs/regulatory/07_release/PART11_COMPLIANCE.md
ls -la docs/regulatory/07_release/DATA_INTEGRITY_POLICY.md
ls -la docs/regulatory/07_release/AUDIT_TRAIL.md
ls -la docs/regulatory/07_release/CYBERSECURITY_PLAN.md
ls -la docs/regulatory/07_release/RELEASE_PACKAGE.md

# Master index
ls -la docs/regulatory/AUDIT_CHECKLIST.md
```

One-liner to count existing vs expected:

```bash
# Count all regulatory .md files (expected: 44 = 43 documents + 1 index)
find docs/regulatory/ -name "*.md" -type f | wc -l

# List any missing files from the expected set
for f in \
  docs/regulatory/01_foundation/{INTENDED_USE,SOFTWARE_SAFETY_CLASS,SOFTWARE_REQUIREMENTS_SPEC,SOFTWARE_DEVELOPMENT_PLAN,REGULATORY_STRATEGY,MEDICAL_DEVICE_FILE,SOFTWARE_MAINTENANCE_PLAN,PROBLEM_RESOLUTION}.md \
  docs/regulatory/02_risk_management/{RISK_MANAGEMENT_PLAN,RISK_ANALYSIS,RISK_EVALUATION,RISK_CONTROLS,RESIDUAL_RISK,RISK_MANAGEMENT_REPORT,POST_PRODUCTION_MONITORING}.md \
  docs/regulatory/03_traceability/{TRACEABILITY_MATRIX,SOUP_ANALYSIS,SBOM,CONFIGURATION_MANAGEMENT}.md \
  docs/regulatory/04_validation/{VALIDATION_PLAN,CREDIBILITY_MODEL,VERIFICATION_REPORT,VALIDATION_REPORT,UNCERTAINTY_ANALYSIS,USER_REQUIREMENTS_SPEC}.md \
  docs/regulatory/05_quality_system/{QUALITY_MANUAL,QUALITY_POLICY,DOCUMENT_CONTROL,RECORD_CONTROL,INTERNAL_AUDIT,NONCONFORMING_PRODUCT,CAPA_PROCEDURE,COMPETENCE_RECORDS}.md \
  docs/regulatory/06_clinical/{CLINICAL_EVALUATION_PLAN,CLINICAL_EVALUATION_REPORT,LIMITATIONS_REPORT,REPRODUCIBILITY_PROTOCOL,REFERENCE_DATA_REGISTRY}.md \
  docs/regulatory/07_release/{PART11_COMPLIANCE,DATA_INTEGRITY_POLICY,AUDIT_TRAIL,CYBERSECURITY_PLAN,RELEASE_PACKAGE}.md \
  docs/regulatory/AUDIT_CHECKLIST.md; do
  [ -f "$f" ] && echo "OK   $f" || echo "MISS $f"
done
```

---

## 7. Revision History

| Version | Date | Author | Changes |
|---------|------|--------|---------|
| 1.0 | 2026-04-02 | Resonance Development Team | Initial creation. 37/37 documents complete (RD-1 through RD-7). Full coverage. |
| 1.1 | 2026-04-02 | Resonance Development Team | Audit supplement: 6 documents added (RD-1.6, RD-1.7, RD-1.8, RD-2.7, RD-4.6, RD-5.8) to close gaps identified by external audit checklist. Total: 43 documents + 1 index = 44 files. Standards coverage updated for IEC 62304 S6/S9, ISO 14971 S9, ISO 13485 S4.2.3/S6.2, GAMP 5 URS. |
