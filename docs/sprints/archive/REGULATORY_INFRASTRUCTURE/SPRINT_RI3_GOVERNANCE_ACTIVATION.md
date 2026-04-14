# RI-3: Governance + Monitoring Activation

**Objetivo:** Activar los mecanismos de gobernanza y monitoreo que los documentos regulatorios definen pero que aun no estan operativos. Cierra los gaps #5 (CCB), #7 (training), #8 (monitoring) y parcialmente #2 (review records via quarterly review).

**Estado:** ✅ COMPLETADO (2026-04-02)
**Esfuerzo:** Bajo (5 entregables, documentos de gobernanza + configuracion GitHub)
**Bloqueado por:** RI-1 (CI debe estar activa para que el monitoreo tenga baseline)
**Desbloquea:** --

---

## Entregables

### 1. Change Control Board Charter

**Archivo:** `docs/regulatory/05_quality_system/CCB_CHARTER.md`
**Contenido:**
- **Membership:** Verificador (chair) + Observador (miembro permanente). Alquimista y Planificador como invitados segun agenda.
- **Scope de decision:** cambios a documentos regulatorios, reclasificacion de seguridad, excepciones a DOD, actualizaciones de SOUP/SBOM.
- **Proceso:** propuesta via PR con label `ccb-review` → revision en sprint review o async → disposicion (APPROVE/REJECT/DEFER) documentada en PR comments.
- **Cadencia:** per-sprint review (sincrono con sprint closure). Quarterly para revisiones de riesgo.
- **Escalation:** si Verificador y Observador discrepan → Planificador decide. Si afecta safety class → requiere unanimidad.
- **Registro:** decisiones en PR history + `REVIEW_LOG.md` (creado en RI-2).

**Evidencia de cierre:** documento existe, roles asignados, primer PR con label `ccb-review` procesado.

### 2. Training Matrix

**Archivo:** `docs/regulatory/05_quality_system/TRAINING_MATRIX.md`
**Contenido:**

| Rol | Competencia requerida | Evidencia aceptada | Estado |
|-----|----------------------|-------------------|--------|
| Alquimista | Rust/Bevy, axiomas, DOD | Sprint closures (code merged + tests green) | Activo |
| Observador | Review protocol, math verification | PR review history (>= 10 reviews) | Activo |
| Planificador | Architecture, orthogonality, sprint design | Sprint track designs (>= 3 tracks) | Activo |
| Verificador | Full DOD, determinism, perf, Bevy 0.15 | PASS/WARN/BLOCK verdicts (>= 5 PRs) | Activo |

- **Modelo:** competence-through-delivery (evidencia = artefactos producidos, no certificados formales).
- **Upgrade path:** si reclasificacion a Class B+ → training formal requerido (IEC 62304, ISO 14971 workshops). Documentar como gap abierto con timeline.
- **Refresh:** anual o cuando se agregue nueva normativa al scope.

**Evidencia de cierre:** documento existe con evidencia real por rol (links a PRs/sprints).

### 3. Monitoring Activation (GitHub Issues + Dependabot)

**3a. Issue templates** (`.github/ISSUE_TEMPLATE/`):
- `bug_report.yml` — fields: description, reproduction steps, expected behavior, severity
- `feature_request.yml` — fields: description, axiom impact, layers affected
- `regulatory_feedback.yml` — fields: document affected, standard reference, gap description, proposed resolution

**3b. Dependabot alerts:** ya habilitado en RI-1. Verificar que alerts llegan y son triageados.

**3c. Quarterly review schedule:**
- Q3 2026 (primera): revisar risk file (RD-2.*), SOUP analysis (RD-3.2), SBOM (RD-3.3), post-production monitoring (RD-2.7)
- Registrar en calendario + GitHub Issue milestone

**Evidencia de cierre:** 3 issue templates existen, Dependabot activo, milestone Q3-2026-REVIEW creado.

### 4. First Quarterly Review Template

**Archivo:** `docs/regulatory/05_quality_system/QUARTERLY_REVIEW_TEMPLATE.md`
**Contenido:**
- **Fecha:** Q3 2026 (julio-septiembre)
- **Scope:** RD-2.1-2.7 (risk file), RD-3.2 (SOUP), RD-3.3 (SBOM), RD-2.7 (post-production monitoring)
- **Checklist:**
  - [ ] Risk analysis actualizado con nuevos hazards identificados
  - [ ] SOUP versions verificadas contra `Cargo.lock`
  - [ ] SBOM regenerado si hubo cambios de dependencias
  - [ ] Post-production: issues recibidos, advisories publicados, acciones tomadas
  - [ ] CCB disposition de cada hallazgo
- **Output:** acta de revision + acciones correctivas si aplica

**Evidencia de cierre:** template existe, primera review agendada.

### 5. Actualizar RD-5.8, RD-2.7, RD-5.5

**RD-5.8 (Competence Records):** referenciar `TRAINING_MATRIX.md` como artefacto formal de evidencia de competencia.
**RD-2.7 (Post-Production Monitoring):** referenciar issue templates + Dependabot + quarterly review como canales activos.
**RD-5.5 (Internal Audit):** referenciar CCB charter + quarterly review template como mecanismos de audit activo.

**Evidencia de cierre:** 3 documentos actualizados con referencias cruzadas.

---

## Scope definido

**Entra:** CCB charter, training matrix, issue templates, quarterly review template, 3 doc updates
**NO entra:** contratacion de auditor externo, certificacion ISO 13485, training formal externo, helpdesk

## Criterios de cierre

- [ ] `CCB_CHARTER.md` existe con membership, scope, proceso, cadencia
- [ ] `TRAINING_MATRIX.md` existe con evidencia por rol
- [ ] 3 issue templates en `.github/ISSUE_TEMPLATE/` (bug, feature, regulatory)
- [ ] Dependabot alerts activo y verificado
- [ ] Milestone `Q3-2026-REVIEW` creado en GitHub
- [ ] `QUARTERLY_REVIEW_TEMPLATE.md` existe con checklist
- [ ] RD-5.8, RD-2.7, RD-5.5 actualizados con referencias a nuevos artefactos
