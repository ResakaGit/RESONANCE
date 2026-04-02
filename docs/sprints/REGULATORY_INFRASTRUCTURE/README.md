# Track: REGULATORY_INFRASTRUCTURE — Closing Structural Gaps from Documentation Audit

**Objetivo:** Implementar la infraestructura de proceso y tooling necesaria para cerrar los 8 gaps estructurales identificados en AUDIT_CHECKLIST.md S5.2. Convertir la documentacion regulatoria de REGULATORY_DOCUMENTATION (37 docs) en un sistema auditable con firmas, CI, gobernanza y monitoreo activo.

**Estado:** PENDIENTE
**Bloqueado por:** Nada (track independiente)
**Desbloquea:** Submissions regulatorias formales, auditorias externas, reclasificacion a SaMD Class B+

---

## Gaps cerrados (AUDIT_CHECKLIST.md S5.2)

| # | Gap | Severidad | Sprint |
|---|-----|-----------|--------|
| 1 | No formal approval signatures | Medium | RI-2 |
| 2 | No review records per document | Medium | RI-2, RI-3 |
| 3 | No GPG commit signing enforced | Low | RI-1 (parcial), RI-2 |
| 4 | No automated CI/CD pipeline | Low | RI-1 |
| 5 | No formal change control board | Medium | RI-3 |
| 6 | Abstract units only (qe, not molar) | Informational | N/A (design decision, not a gap) |
| 7 | No formal training certificates | Low | RI-3 |
| 8 | Monitoring channels defined but not active | Low | RI-3 |

---

## 3 Sprints en orden de dependencia

| Sprint | Descripcion | Esfuerzo | Bloqueado por | Gaps cerrados |
|--------|-------------|----------|---------------|---------------|
| [RI-1](SPRINT_RI1_CICD_PIPELINE.md) | CI/CD Pipeline + Branch Protection | Medio | -- | #4, parcial #3, parcial #1 |
| [RI-2](SPRINT_RI2_SIGNING_IDENTITY.md) | Commit Signing + Document Approval Workflow | Medio | RI-1 | #3, #1, #2 |
| [RI-3](SPRINT_RI3_GOVERNANCE_ACTIVATION.md) | Governance + Monitoring Activation | Bajo | RI-1 | #5, #7, #8, parcial #2 |

---

## Grafo de dependencias

```
RI-1 (CI/CD + branch protection) ──┬──> RI-2 (signing + approval workflow)
                                   └──> RI-3 (governance + monitoring)
```

RI-2 y RI-3 son paralelos entre si. Ambos dependen de RI-1 porque:
- RI-2 necesita branch protection activa para enforcement de GPG signing
- RI-3 necesita CI verde para que el monitoring tenga sentido

---

## Scope del track

**Entra:**
- GitHub Actions workflows
- Branch protection rules
- GPG signing enforcement
- Document approval fields (YAML frontmatter)
- Validation scripts
- CCB charter
- Training matrix
- Issue templates
- Quarterly review schedule

**NO entra:**
- Cambios a `src/` (zero code changes)
- Nuevos documentos regulatorios (eso fue REGULATORY_DOCUMENTATION)
- Certificacion ISO 13485 formal
- Contratacion de auditor externo

---

## Criterios de cierre del track

- [ ] CI pipeline verde en GitHub Actions (`cargo test` + `cargo clippy` + `cargo audit`)
- [ ] Branch protection activa en `main` (PR required, CI pass, 1 review)
- [ ] GPG signing enforced en commits a `main`
- [ ] 43 documentos regulatorios con campos `approved_by` + `review_date` en frontmatter
- [ ] Script de validacion ejecutable que verifica approval fields
- [ ] CCB charter existe con membership y decision authority
- [ ] Training matrix existe con evidencia de competencia por rol
- [ ] GitHub Issues templates configurados (bug, feature, regulatory feedback)
- [ ] Primera quarterly review agendada (Q3 2026)
- [ ] Gaps #1-#5, #7-#8 de AUDIT_CHECKLIST.md S5.2 cerrados o con evidencia de mitigacion
