# RD-2: Risk Management File — ISO 14971:2019 Completo

**Objetivo:** Crear el expediente de gestión de riesgos completo según ISO 14971:2019. Incluye plan, análisis, evaluación, controles, riesgo residual global, e informe final.

**Estado:** ✅ COMPLETADO (2026-04-02)
**Esfuerzo:** Alto (6 documentos, requiere análisis sistemático de peligros)
**Bloqueado por:** RD-1 (Intended Use + Safety Class determinan alcance del análisis)
**Desbloquea:** RD-6 (evaluación clínica necesita el expediente de riesgos)

---

## Entregables

### 1. Plan de Gestión de Riesgos (`RISK_MANAGEMENT_PLAN.md`)

**Estándar:** ISO 14971 §5.1
**Contenido:**
- Alcance: RESONANCE como simulador de dinámicas de resistencia a fármacos
- Criterios de aceptabilidad de riesgo (ALARP — As Low As Reasonably Practicable)
- Métodos: FMEA (Failure Mode and Effects Analysis) para software
- Calendario de revisiones (cada release major, mínimo anual)
- Responsabilidades (roles del equipo vs ISO 14971 §4.3)

### 2. Análisis de Riesgos (`RISK_ANALYSIS.md`)

**Estándar:** ISO 14971 §5.3-5.4
**Contenido — Peligros identificados:**

| ID | Peligro | Daño potencial | Causa | Probabilidad | Severidad |
|----|---------|----------------|-------|-------------|-----------|
| H-01 | Sobreconfianza en predicción de resistencia | Paciente recibe terapia subóptima basada en simulación | Uso clínico sin validación patient-level | Improbable (si research-only) | Seria |
| H-02 | Bug en conservación de energía | Simulación produce resultados incorrectos | Axiom 5 violation no detectada | Remota (3,113 tests + conservation fuzz) | Moderada |
| H-03 | Sesgo en calibración | Modelo calibrado solo a 4 perfiles → overfitting | Datos limitados | Posible | Moderada |
| H-04 | Determinismo roto | Resultados no reproducibles entre runs | Hash collision, floating point | Remota (hash_f32_slice verified) | Menor |
| H-05 | Dependencia SOUP con vulnerabilidad | Bevy/rayon/serde CVE no parcheada | Supply chain | Posible | Variable |
| H-06 | Malinterpretación de output | Usuario interpreta "% suppression" como predicción clínica | UI/UX sin disclaimer visible | Posible | Seria |
| H-07 | Modelo no captura microentorno tumoral | Resultados divergen de realidad biológica | Limitación del modelo (no TME) | Probable | Moderada |

**Fuentes para severity/probability:** README disclaimers, paper §5 Limitations, Bozic validation scope.

### 3. Evaluación de Riesgos (`RISK_EVALUATION.md`)

**Estándar:** ISO 14971 §5.5
- Comparación de cada riesgo contra criterios de aceptabilidad definidos en Plan
- Clasificación: Aceptable / ALARP / Inaceptable
- Research-only → la mayoría son Aceptable (no hay paciente en el loop)
- SaMD → H-01, H-06 son Inaceptables sin controles adicionales

### 4. Medidas de Control de Riesgos (`RISK_CONTROLS.md`)

**Estándar:** ISO 14971 §6
**Controles implementados:**

| Riesgo | Control | Tipo | Implementación |
|--------|---------|------|----------------|
| H-01 | Disclaimers "NOT clinical tool" | Información | README.md, paper §5, bin outputs |
| H-02 | 3,113 tests + conservation fuzz (proptest) | Verificación | `tests/property_conservation.rs` |
| H-03 | Multi-seed validation (10/10 Bozic) | Validación | `bin/bozic_validation.rs` |
| H-04 | Deterministic RNG (hash-based) | Diseño | `blueprint/equations/determinism.rs` |
| H-05 | Cargo.lock pinned deps | Diseño | `Cargo.lock` |
| H-06 | Explicit units in output ("% suppression, not molar") | Información | README honesty pass |
| H-07 | Documented limitations | Información | Paper §5, README "Honest scope" |

### 5. Evaluación de Riesgo Residual Global (`RESIDUAL_RISK.md`)

**Estándar:** ISO 14971 §7
- Balance beneficio-riesgo considerando controles implementados
- Para research-only: riesgo residual aceptable (no hay paciente)
- Para SaMD: riesgo residual requiere evaluación clínica formal (→ RD-6)

### 6. Informe de Gestión de Riesgos (`RISK_MANAGEMENT_REPORT.md`)

**Estándar:** ISO 14971 §8
- Confirmación de completitud del proceso
- Todas las actividades del plan ejecutadas
- Riesgos residuales documentados y aceptados
- Referencia cruzada a controles implementados

---

## Scope definido

**Entra:** 6 documentos del Risk Management File en `docs/regulatory/02_risk_management/`
**NO entra:** Post-market surveillance (cubierto en RD-7), evaluación clínica (RD-6)

## Criterios de cierre

- [ ] 6 documentos con contenido sustantivo
- [ ] ≥7 peligros identificados con probability/severity
- [ ] Cada control referencia implementación concreta en codebase
- [ ] Balance beneficio-riesgo documentado para clasificación actual
