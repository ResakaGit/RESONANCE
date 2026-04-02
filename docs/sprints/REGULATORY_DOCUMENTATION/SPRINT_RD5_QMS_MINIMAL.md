# RD-5: Quality Management System — ISO 13485 Minimal Viable

**Objetivo:** Crear el QMS mínimo viable: Quality Manual + 6 procedimientos obligatorios de ISO 13485. Documentar sobre las prácticas existentes (sprints, roles, CLAUDE.md) sin inventar burocracia.

**Estado:** ✅ COMPLETADO (2026-04-02)
**Esfuerzo:** Medio (7 documentos, estructura estandarizada)
**Bloqueado por:** RD-1 (Quality Policy necesita Intended Use)
**Desbloquea:** RD-7 (release package requiere QMS)

---

## Entregables

### 1. Manual de Calidad (`QUALITY_MANUAL.md`)

**Estándar:** ISO 13485 §4.2.2
**Contenido:**
- Alcance del QMS: desarrollo de software de simulación biomédica
- Exclusiones justificadas: §7.3 purchasing (open-source, no suppliers), §7.5.1 production (software, no manufacturing)
- Referencia a los 6 procedimientos + regulatory docs
- Organigrama (roles de CLAUDE.md: Alquimista, Observador, Planificador, Verificador)
- Interacción de procesos (sprint cycle → design → code → test → review → archive)

### 2. Política de Calidad + Objetivos (`QUALITY_POLICY.md`)

**Estándar:** ISO 13485 §5.3, §5.4.1
**Contenido:**
- Compromiso: código axiomáticamente correcto, reproducible, auditable
- Objetivos medibles:
  - Zero test regressions en cada sprint
  - 100% axiom compliance (verified by tests)
  - Determinismo bit-exact verificado en cada release
  - Documentación actualizada con cada sprint (sprint docs como evidencia)

### 3. Control de Documentos (`DOCUMENT_CONTROL.md`)

**Estándar:** ISO 13485 §4.2.4
**Contenido:**
- Documentos controlados: CLAUDE.md, ARCHITECTURE.md, sprint docs, regulatory docs
- Aprobación: merge to main = approved (trunk-based, commit hash = version)
- Distribución: GitHub repository (public)
- Cambios: Git diff + commit message (auditable)
- Retención: Git history (permanent, immutable)

### 4. Control de Registros (`RECORD_CONTROL.md`)

**Estándar:** ISO 13485 §4.2.5
**Contenido:**
- Registros de calidad: test results (cargo test output), sprint archive docs, benchmark results
- Almacenamiento: Git repository
- Recuperación: `git log`, `git show`, sprint archive
- Período de retención: indefinido (Git immutable history)

### 5. Auditoría Interna (`INTERNAL_AUDIT.md`)

**Estándar:** ISO 13485 §8.2.4
**Contenido:**
- Frecuencia: cada sprint track (sprint closure = audit checkpoint)
- Método: grep-based verification (como DC-1 closure criteria)
- Checklist: axiom compliance, test count, 0 warnings, DoD met
- Registros: sprint README con checkmarks ✅
- Ejemplo real: DECOUPLING_AUDIT cerrado con grep verification de 7 criterios

### 6. Control de Producto No Conforme (`NONCONFORMING_PRODUCT.md`)

**Estándar:** ISO 13485 §8.3
**Contenido:**
- Non-conformance = test failure, axiom violation, or safety rule breach
- Detection: `cargo test`, `cargo check`, grep verification
- Disposition: fix before merge (trunk-based — no broken main)
- Record: commit que introduce el fix + sprint doc que lo documenta
- Ejemplo: DC-4 encontró hardcoded 0.016 dt → fixed → documented in sprint archive

### 7. CAPA (Corrective and Preventive Action) (`CAPA_PROCEDURE.md`)

**Estándar:** ISO 13485 §8.5.2-8.5.3
**Contenido:**
- Corrective: fix the bug (standard dev cycle)
- Preventive: add test to prevent recurrence + add grep criterion to sprint closure
- Root cause analysis: documented in sprint doc "Problema" section
- Effectiveness verification: test suite must pass + grep criteria green
- Ejemplo: conservation violation → proptest fuzz added → now prevents all future conservation bugs

---

## Scope definido

**Entra:** 7 documentos en `docs/regulatory/05_quality_system/`
**NO entra:** Implementación de QMS electrónico; certificación ISO 13485; auditoría por terceros

## Criterios de cierre

- [ ] Quality Manual con alcance, exclusiones, organigrama, procesos
- [ ] 6 procedimientos documentados con referencias a prácticas existentes
- [ ] Cada procedimiento incluye ≥1 ejemplo real del codebase/sprints
- [ ] Objetivos de calidad son medibles y verificables por cargo test/grep
