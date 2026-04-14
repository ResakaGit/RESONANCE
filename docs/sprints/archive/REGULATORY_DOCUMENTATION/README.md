# Track: REGULATORY_DOCUMENTATION — Compliance-Ready Technical Documentation

**Objetivo:** Crear la documentación técnica completa que posiciona a RESONANCE para cumplimiento regulatorio (IEC 62304, ISO 14971, ASME V&V 40, FDA CMS Guidance, GAMP 5). Documentar retroactivamente sobre la base de código existente (3,113 tests, 110K LOC, paper Zenodo, 8 axiomas verificados).

**Estado:** ✅ COMPLETADO (2026-04-02) — 37/37 documentos + índice maestro
**Bloqueado por:** Nada (track independiente, puramente documental)
**Desbloquea:** Submissions regulatorias, partnerships farmacéuticos, publicaciones con evidencia V&V

---

## Gap Analysis (auditoría 2026-04-02)

| Estado | Documentos | Porcentaje |
|--------|-----------|------------|
| ✅ Existe | 1 | 2% |
| ⚠️ Parcial | 17 | 34% |
| ❌ No existe | 32 | 64% |
| **Total** | **50** | **100%** |

---

## Marco Normativo

| Eje | Estándares |
|-----|-----------|
| Software médico | IEC 62304:2006+Amd1:2015, ISO 14971:2019, ISO 13485:2016, IEC 82304-1, IMDRF SaMD |
| Validación computacional | GAMP 5 2ª ed., FDA General Principles, 21 CFR Part 11, ICH Q9/Q10 |
| Simulación y modelado | ASME V&V 40:2018, FDA CMS Guidance 2023, ISO/IEC 25010:2023 |

---

## 7 Sprints en 4 Oleadas

| Sprint | Descripción | Docs | Esfuerzo | Bloqueado por |
|--------|-------------|------|----------|---------------|
| [RD-1](SPRINT_RD1_REGULATORY_FOUNDATION.md) | Clasificación + Intended Use + Risk Plan + SRS + Dev Plan | 5 docs | Alto | — |
| [RD-2](SPRINT_RD2_RISK_MANAGEMENT.md) | Risk Management File completo (ISO 14971) | 6 docs | Alto | RD-1 |
| [RD-3](SPRINT_RD3_TRACEABILITY_SOUP.md) | Matriz trazabilidad + SOUP analysis + SBOM | 4 docs | Medio | RD-1 |
| [RD-4](SPRINT_RD4_VV_CREDIBILITY.md) | Validation Plan + Credibility Model (ASME V&V 40) + V&V Reports | 5 docs | Alto | RD-1, RD-3 |
| [RD-5](SPRINT_RD5_QMS_MINIMAL.md) | Quality Manual + 6 procedimientos obligatorios ISO 13485 | 7 docs | Medio | RD-1 |
| [RD-6](SPRINT_RD6_CLINICAL_EVALUATION.md) | Evaluación clínica + Reproducibilidad + Limitaciones | 5 docs | Medio | RD-4 |
| [RD-7](SPRINT_RD7_DATA_INTEGRITY_RELEASE.md) | Part 11 compliance + Audit trail + Release package + Cybersecurity | 5 docs | Medio | RD-5 |

**Total: 7 sprints, 37 documentos nuevos + 13 actualizaciones de parciales = 50 entregables.**

---

## Grafo de dependencias

```
RD-1 (foundation) ──┬──→ RD-2 (risk file)
                    ├──→ RD-3 (traceability + SOUP) ──→ RD-4 (V&V + credibility)
                    ├──→ RD-5 (QMS) ──────────────────→ RD-7 (data integrity + release)
                    └──→ RD-6 (clinical) ←── RD-4
```

## Oleadas de ejecución

- **Oleada 0:** RD-1 (fundación regulatoria — desbloquea todo)
- **Oleada 1:** RD-2 + RD-3 + RD-5 (paralelos — riesgos, trazabilidad, QMS)
- **Oleada 2:** RD-4 + RD-6 (paralelos — V&V y evaluación clínica, post-trazabilidad)
- **Oleada 3:** RD-7 (release package — cierre)

## Matriz de paralelismo

| | RD-1 | RD-2 | RD-3 | RD-4 | RD-5 | RD-6 | RD-7 |
|---|---|---|---|---|---|---|---|
| **RD-1** | — | blocks | blocks | blocks | blocks | blocks | blocks |
| **RD-2** | — | — | ∥ | — | ∥ | — | — |
| **RD-3** | — | ∥ | — | blocks | ∥ | — | — |
| **RD-4** | — | — | — | — | ∥ | blocks | — |
| **RD-5** | — | ∥ | ∥ | ∥ | — | ∥ | blocks |
| **RD-6** | — | — | — | — | ∥ | — | — |
| **RD-7** | — | — | — | — | — | ∥ | — |

---

## Inventario de fortalezas existentes

Estos activos del codebase aceleran la documentación retroactiva:

| Activo | Dónde | Qué alimenta |
|--------|-------|---------------|
| 3,113 tests automatizados | `cargo test` | Evidencia de verificación (IEC 62304 §5.5-5.7) |
| Determinismo bit-exact | `blueprint/equations/determinism.rs` | Reproducibilidad (ASME V&V 40) |
| Paper Zenodo | `docs/paper/resonance_arxiv.tex` | Modelo de credibilidad + evaluación clínica |
| 8 axiomas formalizados | CLAUDE.md + `derived_thresholds.rs` | Especificación matemática del modelo |
| Validación Bozic 2013 | `bin/bozic_validation.rs` | Evidencia de validación contra datos publicados |
| 4 perfiles de calibración | README.md | Protocolo de calibración parcial |
| Disclaimers explícitos | README.md | Controles de riesgo informales |
| ARCHITECTURE.md | `docs/ARCHITECTURE.md` | Diseño arquitectónico (IEC 62304 §5.3) |
| Diseño detallado | `docs/design/*.md` | Diseño detallado parcial (IEC 62304 §5.4) |
| CHANGELOG | CHANGELOG.md | Gestión de configuración parcial |
| Cargo.lock | raíz | SOUP/SBOM base |
| Sprint history | `docs/sprints/archive/` | Trazabilidad de desarrollo |

---

## Ubicación de documentos

```
docs/regulatory/
├── 01_foundation/
│   ├── INTENDED_USE.md               (RD-1)
│   ├── SOFTWARE_SAFETY_CLASS.md      (RD-1)
│   ├── SOFTWARE_DEVELOPMENT_PLAN.md  (RD-1)
│   ├── SOFTWARE_REQUIREMENTS_SPEC.md (RD-1)
│   └── REGULATORY_STRATEGY.md        (RD-1)
├── 02_risk_management/
│   ├── RISK_MANAGEMENT_PLAN.md       (RD-2)
│   ├── RISK_ANALYSIS.md              (RD-2)
│   ├── RISK_EVALUATION.md            (RD-2)
│   ├── RISK_CONTROLS.md              (RD-2)
│   ├── RESIDUAL_RISK.md              (RD-2)
│   └── RISK_MANAGEMENT_REPORT.md     (RD-2)
├── 03_traceability/
│   ├── TRACEABILITY_MATRIX.md        (RD-3)
│   ├── SOUP_ANALYSIS.md              (RD-3)
│   ├── SBOM.md                       (RD-3)
│   └── CONFIGURATION_MANAGEMENT.md   (RD-3)
├── 04_validation/
│   ├── VALIDATION_PLAN.md            (RD-4)
│   ├── CREDIBILITY_MODEL.md          (RD-4)
│   ├── VERIFICATION_REPORT.md        (RD-4)
│   ├── VALIDATION_REPORT.md          (RD-4)
│   └── UNCERTAINTY_ANALYSIS.md       (RD-4)
├── 05_quality_system/
│   ├── QUALITY_MANUAL.md             (RD-5)
│   ├── QUALITY_POLICY.md             (RD-5)
│   ├── DOCUMENT_CONTROL.md           (RD-5)
│   ├── RECORD_CONTROL.md             (RD-5)
│   ├── INTERNAL_AUDIT.md             (RD-5)
│   ├── NONCONFORMING_PRODUCT.md      (RD-5)
│   └── CAPA_PROCEDURE.md             (RD-5)
├── 06_clinical/
│   ├── CLINICAL_EVALUATION_PLAN.md   (RD-6)
│   ├── CLINICAL_EVALUATION_REPORT.md (RD-6)
│   ├── LIMITATIONS_REPORT.md         (RD-6)
│   ├── REPRODUCIBILITY_PROTOCOL.md   (RD-6)
│   └── REFERENCE_DATA_REGISTRY.md    (RD-6)
├── 07_release/
│   ├── PART11_COMPLIANCE.md          (RD-7)
│   ├── DATA_INTEGRITY_POLICY.md      (RD-7)
│   ├── AUDIT_TRAIL.md                (RD-7)
│   ├── CYBERSECURITY_PLAN.md         (RD-7)
│   └── RELEASE_PACKAGE.md            (RD-7)
└── AUDIT_CHECKLIST.md                (índice maestro — este doc)
```

---

## Invariantes del track

1. **Retroactivo, no ficticio.** Cada documento se basa en la realidad del codebase. Si algo no existe, se documenta como gap — no se inventa.
2. **Auditable.** Cada claim referencia archivo:línea, test name, o commit hash.
3. **Versionado.** Cada documento incluye header con versión, fecha, autor, estado de revisión.
4. **Independiente del IDE.** Todo en Markdown — renderizable en GitHub, exportable a PDF.
5. **Clasificación honesta.** Si RESONANCE es research-only, se documenta así. Si aspira a SaMD, se documenta el gap.

---

## Criterios de cierre del track

- [x] 50 documentos creados o actualizados en `docs/regulatory/` — 43 docs + 1 índice = 44 archivos, ~15,400 líneas
- [x] AUDIT_CHECKLIST.md con ✅/⚠️/❌ actualizado por documento — 43/43 DONE
- [x] README.md actualizado con test count corregido (3,113)
- [x] CLAUDE.md actualizado con referencia a docs/regulatory/
- [x] Zero documentos con placeholders vacíos — cada uno tiene contenido sustantivo
- [x] Cada claim técnico referencia ≥1 fuente en el codebase
