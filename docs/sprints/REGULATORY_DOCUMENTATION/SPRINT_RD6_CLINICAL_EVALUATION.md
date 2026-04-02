# RD-6: Clinical Evaluation + Reproducibility + Limitations

**Objetivo:** Documentar formalmente la evaluación clínica, el protocolo de reproducibilidad, el registro de datos de referencia, y el informe de limitaciones. Consolida la evidencia científica dispersa en README, paper, y binarios.

**Estado:** ✅ COMPLETADO (2026-04-02)
**Esfuerzo:** Medio (5 documentos, consolidación de material existente)
**Bloqueado por:** RD-4 (credibility model informa la evaluación clínica)
**Desbloquea:** Publicaciones, partnerships, submissions

---

## Entregables

### 1. Plan de Evaluación Clínica (`CLINICAL_EVALUATION_PLAN.md`)

**Estándar:** IMDRF SaMD N41, EU MDR Annex XIV
**Contenido:**
- Evidencia analítica: tests de verificación (3,113), conservation proofs
- Evidencia de rendimiento: Bozic validation, multi-seed robustness
- Evidencia clínica: calibración contra London 2003/2009, Gatenby 2009
- Gold standard comparators: Bozic et al. 2013 (eLife) predictions
- Gaps identificados: no patient-level data, no prospective studies

### 2. Informe de Evaluación Clínica (`CLINICAL_EVALUATION_REPORT.md`)

**Estándar:** IMDRF SaMD N41
**Contenido:**

**Evidencia analítica:**
- 3,113 automated tests → code correctness
- Axiom compliance → 8/8 verified
- Conservation fuzz → no violations found in 10^6 random inputs

**Evidencia de rendimiento:**
- Bozic 5-arm protocol: combo_AB(56.5%) > mono_A(51.9%) > mono_B(36.5%) > no_drug(0%)
- 10/10 seeds confirm → structural result
- Adaptive therapy framing (Gatenby 2009): qualitative agreement

**Calibración contra datos publicados:**
- London 2003: canine MCT, vinblastine + prednisolone response curve
- London 2009: updated MCT staging
- Gatenby 2009: adaptive therapy concept
- Bozic 2013: combination therapy exponential advantage

**Limitaciones (explicit):**
- Abstract qe units, not molar concentrations
- No molecular targets (no EGFR, no BCR-ABL)
- No tumor microenvironment
- Not validated against patient-level outcomes
- Bozic comparison is qualitative (suppression %, not time-to-resistance)

### 3. Informe de Limitaciones y Alcance (`LIMITATIONS_REPORT.md`)

**Estándar:** ASME V&V 40 + ética científica
**Contenido:**
- **Lo que el modelo PUEDE hacer:** explorar dinámicas de resistencia, comparar estrategias cualitivamente, generar hipótesis
- **Lo que el modelo NO PUEDE hacer:** predecir respuesta individual, determinar dosis, reemplazar ensayos clínicos
- **Supuestos del modelo:** everything-is-energy, no molecular resolution, frequency as proxy for target binding
- **Condiciones de falla:** model diverges from reality when TME dominates, when stochastic events (immune response) are critical, when pharmacokinetics matter
- **Honest scope per module:**
  - Level 1 (cytotoxic): qe drain only, no mechanism resolution
  - Level 2 (pathway inhibitor): competitive/noncompetitive/uncompetitive, but abstract binding
  - Calibration: 4 profiles, qualitative fit, not quantitative prediction

### 4. Protocolo de Reproducibilidad (`REPRODUCIBILITY_PROTOCOL.md`)

**Estándar:** Buena práctica científica + ASME V&V 40
**Contenido:**
- **Requisitos de entorno:** Rust stable 2024 (MSRV 1.85), cargo build --release
- **Comandos exactos por experimento:**
  ```bash
  # Bozic validation (10 seeds, ~95 sec)
  cargo run --release --bin bozic_validation

  # Cancer therapy (cytotoxic)
  cargo run --release --bin cancer_therapy

  # Headless simulation (PPM output)
  cargo run --release --bin headless_sim -- --ticks 10000 --scale 8 --out world.ppm

  # Full test suite
  cargo test
  ```
- **Verificación de determinismo:**
  ```bash
  # Run twice, compare output
  cargo run --release --bin bozic_validation > run1.txt 2>&1
  cargo run --release --bin bozic_validation > run2.txt 2>&1
  diff run1.txt run2.txt  # Must be empty
  ```
- **Seed specification:** deterministic from entity index (no RNG crate)
- **Expected outputs:** tables with reference values for each experiment

### 5. Registro de Datos de Referencia (`REFERENCE_DATA_REGISTRY.md`)

**Estándar:** ASME V&V 40
**Contenido:**

| Dataset | Source | DOI/URL | License | Used in | Integrity |
|---------|--------|---------|---------|---------|-----------|
| Bozic 2013 predictions | eLife 2:e00747 | 10.7554/eLife.00747 | CC-BY | Exp 7, bozic_validation | Qualitative comparison |
| Gatenby 2009 adaptive therapy | Cancer Research 69:4894 | 10.1158/0008-5472 | — | Adaptive framing | Conceptual only |
| London 2003 canine MCT | JAAHA 39:489 | — | — | Rosie calibration | Dose-response curve |
| London 2009 MCT update | Vet Comp Oncology 7:31 | — | — | Rosie staging | Grade correlation |
| Kleiber 0.75 | Kleiber 1947, West 1997 | — | — | KLEIBER_EXPONENT | Biological universal |

---

## Scope definido

**Entra:** 5 documentos en `docs/regulatory/06_clinical/`
**NO entra:** Nuevos experimentos; recolección de datos clínicos; prospective studies

## Criterios de cierre

- [ ] Clinical Evaluation Report con 3 tipos de evidencia (analítica, rendimiento, clínica)
- [ ] Limitations Report con "CAN do" / "CANNOT do" / "assumptions" / "failure conditions"
- [ ] Reproducibility Protocol con comandos copy-paste para cada experimento
- [ ] Reference Data Registry con ≥5 datasets, DOIs, y integrity assessment
- [ ] Cada claim de validación referencia output concreto de bin/
