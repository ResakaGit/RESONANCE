# RI-2: Commit Signing + Document Approval Workflow

**Objetivo:** Establecer identidad criptografica en commits y un workflow de aprobacion formal para documentos regulatorios. Cierra los gaps #3 (GPG signing), #1 (approval signatures) y #2 (review records).

**Estado:** ✅ COMPLETADO (2026-04-02)
**Esfuerzo:** Medio (5 entregables, proceso + tooling)
**Bloqueado por:** RI-1 (branch protection debe estar activa para enforcement)
**Desbloquea:** --

---

## Entregables

### 1. GPG Key Setup + Enforcement

**Pasos:**
- Generar GPG key par (o usar SSH signing, GitHub lo soporta desde 2022)
- Configurar `git config --global commit.gpgsign true`
- Subir public key a GitHub > Settings > SSH and GPG keys
- Branch protection rule: "Require signed commits" en `main`

**Nota:** SSH signing (`allowed_signers`) es mas simple que GPG para equipos pequenos. Ambos cumplen 21 CFR Part 11 S11.10(a) (firma electronica vinculada a individuo).

**Evidencia de cierre:** `git log --show-signature HEAD` muestra "Good signature" en commits nuevos.

### 2. Document Approval Workflow (YAML Frontmatter)

**Formato:** agregar campos de aprobacion al frontmatter YAML existente de los 43 docs regulatorios:

```yaml
---
document_id: RD-X.Y
title: ...
version: 1.0
date: 2026-04-02
status: APPROVED          # DRAFT → REVIEW → APPROVED
author: Resonance Development Team
approved_by: [name/role]  # NEW: quien aprobo
review_date: 2026-XX-XX   # NEW: fecha de revision
review_status: APPROVED    # NEW: PENDING | IN_REVIEW | APPROVED | REJECTED
reviewer_notes: ""         # NEW: notas del revisor (opcional)
---
```

**Proceso:** merge a `main` con frontmatter `status: APPROVED` = documento aprobado. El commit hash + GPG signature = firma electronica.

**Evidencia de cierre:** al menos 5 docs regulatorios actualizados con nuevos campos como prueba de concepto.

### 3. Review Log Template

**Archivo:** `docs/regulatory/REVIEW_LOG.md`
**Contenido:** tabla centralizada de revisiones por documento:

| Doc ID | Reviewer | Date | Disposition | Commit | Notes |
|--------|----------|------|-------------|--------|-------|
| RD-1.1 | [role] | YYYY-MM-DD | APPROVED | abc1234 | -- |

**Alternativa:** review log per-document (en el propio frontmatter). El log centralizado es mejor para auditorias porque permite vista panoramica.

**Evidencia de cierre:** template existe con al menos 5 entradas reales.

### 4. Script de Validacion de Approval Fields

**Archivo:** `scripts/validate_regulatory_frontmatter.sh` (o `.py`)
**Funcion:** recorrer `docs/regulatory/**/*.md`, parsear frontmatter YAML, verificar:
- `approved_by` presente y no vacio
- `review_date` presente y formato valido
- `review_status` in {PENDING, IN_REVIEW, APPROVED}
- `status` != DRAFT si `review_status` == APPROVED

**Exit code:** 0 si todo ok, 1 si hay gaps. Integrable en CI (futuro).

**Evidencia de cierre:** script ejecuta y reporta estado de los 43 docs.

### 5. Actualizar RD-5.3 y RD-7.1

**RD-5.3 (Document Control):** agregar seccion describiendo el nuevo workflow de aprobacion (frontmatter + GPG + PR merge).
**RD-7.1 (Part 11 Compliance):** agregar evidencia de firma electronica (GPG/SSH signing en commits) como control tecnico de 21 CFR Part 11 S11.10.

**Evidencia de cierre:** ambos documentos actualizados con referencia a los nuevos artefactos.

---

## Scope definido

**Entra:** GPG/SSH setup, frontmatter updates, review log, validation script, 2 doc updates
**NO entra:** PKI corporativo, certificados X.509, firma digital avanzada (eIDAS), hardware tokens

## Criterios de cierre

- [ ] Commits a `main` firmados (GPG o SSH) y verificables en GitHub
- [ ] Branch protection rule "Require signed commits" activa
- [ ] 43 docs regulatorios con campos `approved_by`, `review_date`, `review_status` en frontmatter
- [ ] `REVIEW_LOG.md` existe con >= 5 entradas reales
- [ ] Script de validacion ejecuta y reporta 0 gaps en approval fields
- [ ] RD-5.3 y RD-7.1 actualizados con referencia al nuevo workflow
