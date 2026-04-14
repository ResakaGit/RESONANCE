# RD-1: Regulatory Foundation — Clasificación + Intended Use + SRS + Dev Plan

**Objetivo:** Crear los 5 documentos fundacionales sin los cuales ningún otro documento regulatorio tiene contexto. Establece QUÉ es RESONANCE, PARA QUIÉN, y CÓMO se desarrolla.

**Estado:** ✅ COMPLETADO (2026-04-02)
**Esfuerzo:** Alto (5 documentos de alta complejidad conceptual)
**Bloqueado por:** —
**Desbloquea:** RD-2, RD-3, RD-4, RD-5, RD-6, RD-7 (todo el track)

---

## Entregables

### 1. Declaración de Uso Previsto (`INTENDED_USE.md`)

**Estándar:** IMDRF SaMD N10, IEC 62304 §4.1
**Contenido:**
- Intended Use Statement formal (qué hace, para quién, en qué contexto)
- Intended Users (investigadores, no clínicos; farmacéuticas en R&D, no bedside)
- Condiciones de uso (research-only con disclaimer; NO point-of-care)
- Lo que RESONANCE NO es (no diagnóstico, no prescripción, no sustituto de ensayos clínicos)
- Clasificación IMDRF: significancia de decisión × estado de salud → categoría de riesgo

**Fuentes en codebase:**
- README.md disclaimers ("NOT a clinical tool", "Not validated against patient outcomes")
- Paper Zenodo §1 Introduction, §5 Limitations
- `docs/paper/resonance_arxiv.tex` — scope declarations

### 2. Clasificación de Seguridad del Software (`SOFTWARE_SAFETY_CLASS.md`)

**Estándar:** IEC 62304 §4.3
**Contenido:**
- Asignación formal: Class A (no injury possible) / B (non-serious) / C (serious)
- Justificación basada en Intended Use
- Si research-only → Class A (el software no puede causar daño porque no informa decisiones clínicas directas)
- Si SaMD → Class B (informar, no controlar) o C (driving clinical management)
- Implicaciones en documentación requerida por clase
- Mapping: Safety Class → requerimientos de IEC 62304 por cláusula

**Decisión clave:** La clasificación determina el rigor documental de TODO el resto del track.

### 3. Especificación de Requisitos de Software (`SOFTWARE_REQUIREMENTS_SPEC.md`)

**Estándar:** IEC 62304 §5.2, GAMP 5 FS
**Contenido:**

**Requisitos funcionales** (derivados retroactivamente del código):
- RF-01: Simular vida emergente desde interacciones energéticas (Axioms 1-8)
- RF-02: Modelar drug-pathway interaction (pathway_inhibitor.rs: Hill pharmacokinetics, 3 inhibition modes)
- RF-03: Modelar drug resistance evolution (cancer_therapy.rs: quiescent stem cells, mutation)
- RF-04: Reproducir Bozic 2013 (combo > mono therapy, 10/10 seeds)
- RF-05: Calibrar contra datos publicados (4 clinical profiles: London, Gatenby, Bozic, Rosie)
- RF-06: Determinismo bit-exact (determinism.rs: hash-based RNG, no std::rand)
- RF-07: Batch simulation (batch/: 33 systems, rayon parallel, no Bevy dependency)
- RF-08: Headless output (headless_sim.rs: PPM image, no GPU)
- RF-09: Energy conservation (Axiom 5: total qe monotonically decreases)
- RF-10: 14-layer ECS composition (layers/: orthogonal components)

**Requisitos de rendimiento:**
- RP-01: 3,113 tests en <60 seg (actual: ~35 seg)
- RP-02: Batch benchmark: N worlds/sec (criterion bench)
- RP-03: Headless sim: 10K ticks en <30 seg

**Requisitos de seguridad/safety:**
- RS-01: Zero `unsafe` en runtime (except justified bytemuck GPU)
- RS-02: Zero shared mutable state (no Arc<Mutex>, no static mut)
- RS-03: Deterministic output (same seed → same result)
- RS-04: Disclaimers visibles (NOT a clinical tool)

**Fuentes en codebase:** CLAUDE.md (rules, axioms, constants), src/ module structure, test names.

### 4. Plan de Desarrollo de Software (`SOFTWARE_DEVELOPMENT_PLAN.md`)

**Estándar:** IEC 62304 §5.1
**Contenido:**
- Modelo de desarrollo: iterativo basado en sprints (docs/sprints/)
- Roles: Alquimista (dev), Observador (review), Planificador (design), Verificador (PR)
- Estándares de codificación: CLAUDE.md Hard Blocks + Coding Rules
- Herramientas: Rust stable 2024, Bevy 0.15, cargo test, cargo bench
- Gestión de configuración: Git, Cargo.lock, semver
- Entregables por sprint: código + tests + diseño + criterios de cierre verificados por grep
- V&V strategy: 3 capas (unitario, integración, orquestación)
- Trazabilidad: cada sprint doc → archivos → tests

**Fuentes:** CLAUDE.md §Roles, §Testing, §Checklists; docs/sprints/ structure.

### 5. Estrategia Regulatoria (`REGULATORY_STRATEGY.md`)

**Estándar:** IMDRF SaMD Framework
**Contenido:**
- Decisión de posicionamiento: Research Tool vs SaMD
- Si Research: documentación como best practice, no obligación
- Si SaMD: pathway regulatorio (FDA 510(k) De Novo, EU MDR Class IIa, etc.)
- Gap analysis actualizado (este documento — la auditoría de 50 items)
- Roadmap de certificación con hitos y dependencias

---

## Scope definido

**Entra:** 5 documentos nuevos en `docs/regulatory/01_foundation/`
**NO entra:** Contenido de los otros 6 sprints; implementación de QMS; evaluación clínica detallada

## Criterios de cierre

- [ ] 5 documentos creados con contenido sustantivo (no placeholders)
- [ ] Safety Classification asignada formalmente con justificación
- [ ] SRS tiene ≥10 requisitos funcionales con referencia a archivo fuente
- [ ] Intended Use incluye declaración formal compatible con IMDRF
- [ ] Dev Plan referencia CLAUDE.md, roles, herramientas, y sprint methodology
