# RD-4: Validation & Verification + Computational Credibility Model

**Objetivo:** Documentar formalmente la evidencia de V&V y construir el modelo de credibilidad computacional según ASME V&V 40:2018 y FDA CMS Guidance 2023.

**Estado:** ✅ COMPLETADO (2026-04-02)
**Esfuerzo:** Alto (5 documentos, requiere análisis de los 3,113 tests + validaciones existentes)
**Bloqueado por:** RD-1 (SRS), RD-3 (trazabilidad)
**Desbloquea:** RD-6 (evaluación clínica requiere credibilidad del modelo)

---

## Entregables

### 1. Plan de Validación del Software (`VALIDATION_PLAN.md`)

**Estándar:** FDA General Principles of Software Validation, GAMP 5
**Contenido:**
- Estrategia V&V de 3 capas (ya implementada):
  - **Capa 1: Unitario** — pure fns en `blueprint/equations/`, boundary inputs (qe=0, radius=0)
  - **Capa 2: Integración** — MinimalPlugins + spawn + update + assert delta
  - **Capa 3: Orquestación** — full pipeline, HOF harness, N updates
- Criterios de aceptación por capa:
  - Capa 1: 100% coverage de funciones públicas en equations/
  - Capa 2: Cada sistema registrado tiene ≥1 integration test
  - Capa 3: Cada experimento validado tiene ≥1 pipeline test
- Herramientas: cargo test (lib + integration + doc), cargo bench (criterion), proptest (fuzz)
- Ambientes: dev (macOS/Linux), CI (headless), batch (rayon)

### 2. Modelo de Credibilidad Computacional (`CREDIBILITY_MODEL.md`)

**Estándar:** ASME V&V 40:2018 §4, FDA CMS Guidance 2023
**Estructura según V&V 40:**

**§4.1 Context of Use (COU):**
- Question of Interest: ¿Cuál es la ventaja terapéutica de combinación vs monoterapia en resistencia?
- Role of Model: Informar (no controlar). Predice tendencia, no valor absoluto.
- Decision Consequence: Baja (research planning, no patient treatment)
- Regulatory Consequence: Mínima si research-only; media si informa diseño de ensayos

**§5 Verification (Code → Equations):**
- 3,113 tests automatizados verifican que el código implementa las ecuaciones correctamente
- Determinismo bit-exact (same input → same output, verified by hash)
- Conservation fuzz: `tests/property_conservation.rs` (proptest con inputs arbitrarios)
- 8 axiomas con ≥10 tests cada uno
- 4 constantes fundamentales con derivation chain verified (17 tests en `derived_thresholds.rs`)

**§6 Validation (Model → Reality):**
- **Bozic 2013 (eLife):** Combo > mono therapy — 10/10 seeds confirm (structural, not stochastic)
- **Gatenby 2009 (Cancer Research):** Adaptive therapy framing
- **London 2003/2009 (Veterinary Oncology):** Canine mast cell tumor profiles
- **Limitation:** All comparisons are qualitative (% suppression, not absolute cell counts)
- **Limitation:** No tumor microenvironment, no molecular targets (EGFR, BCR-ABL)
- **Limitation:** Abstract qe units, not molar concentrations

**§7 Uncertainty Quantification:**
- Multi-seed robustness: 10 independent seeds, ≥80% threshold for structural claims
- Sensitivity: 4 fundamental constants → all derived thresholds (chain documented)
- Parametric uncertainty: KLEIBER (0.75 ± biological literature), DISSIPATION ratios (1:4:16:50 ± calibration)

**§8 Applicability:**
- Valid for: exploring resistance dynamics, qualitative comparison of therapeutic strategies
- NOT valid for: dosing decisions, patient-specific predictions, regulatory submissions without additional validation

### 3. Informe de Verificación del Modelo (`VERIFICATION_REPORT.md`)

**Estándar:** ASME V&V 40 §5
**Contenido:**
- Inventario completo de tests por módulo:
  - `blueprint/equations/`: ~1,200 unit tests (pure math verification)
  - `batch/`: 156 tests (headless simulation systems)
  - `simulation/`: ~1,500 tests (ECS integration)
  - `tests/`: property-based (conservation fuzz)
  - `worldgen/`: ~200 tests (field generation)
- Código → Ecuación mapping: cada fn en equations/ tiene §doc comment con fórmula + test
- Determinism proof: `hash_f32_slice` + `next_u64` + `gaussian_f32` verified bit-exact
- Axiom compliance: each axiom mapped to enforcing tests

### 4. Informe de Validación del Modelo (`VALIDATION_REPORT.md`)

**Estándar:** ASME V&V 40 §6
**Contenido:**
- Experiment 1-6: Published in Zenodo paper
- Experiment 7 (Rosie): Canine mast cell tumor, partial response validated
- Bozic validation: 5-arm protocol, combo_AB suppression > mono_A, mono_B, double_A
- Quantitative results: tables with efficiency values per arm per seed
- Statistical robustness: 10/10 seeds confirm (no parametric statistics needed — structural result)
- Known gaps: no TME, no pharmacokinetics (PK), no molecular resolution

### 5. Análisis de Incertidumbre y Sensibilidad (`UNCERTAINTY_ANALYSIS.md`)

**Estándar:** ASME V&V 40 §7, ICH Q9
**Contenido:**
- Parameter sensitivity: perturb each of 4 fundamentals ±10%, measure output delta
- Multi-seed variance: standard deviation of efficiency across 10 seeds
- Model form uncertainty: qe-based (abstract) vs molar (physical) — qualitative gap analysis
- Numerical uncertainty: f32 precision limits (bounded by deterministic hashing)

---

## Scope definido

**Entra:** 5 documentos en `docs/regulatory/04_validation/`
**NO entra:** Ejecutar nuevos experimentos de validación; automatizar sensitivity analysis

## Criterios de cierre

- [ ] Credibility Model sigue estructura ASME V&V 40 §4-8
- [ ] Verification Report lista tests por módulo con conteo
- [ ] Validation Report documenta todos los experimentos con resultados cuantitativos
- [ ] Uncertainty Analysis incluye multi-seed + parameter sensitivity framework
- [ ] Cada claim referencia ≥1 archivo fuente o test name
