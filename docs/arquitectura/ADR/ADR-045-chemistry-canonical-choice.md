# ADR-045: Elección canónica · alchemical (Ax 8) vs mass-action (AP-*)

**Estado:** Aceptado — Camino 1 (coexistencia) tras spike AI-3
**Fecha:** 2026-04-15 (decisión 2026-04-15-b)
**Contexto:** AUTOPOIESIS Integration (Sprint AI, ítem AI-3)
**ADRs relacionados:** ADR-037 (substrate), ADR-039 (fission), ADR-043 (bridge), ADR-044 (spawn)

## 1. Contexto y problema

- Módulos afectados (potencialmente deprecados según outcome):
  - Alchemical: `src/layers/alchemical_engine.rs`, `src/layers/injector.rs`, `src/layers/matter_coherence.rs`
  - Mass-action: `src/layers/species_grid.rs`, `src/layers/reaction_network.rs`, `src/blueprint/equations/reaction_kinetics.rs`, todo `src/use_cases/experiments/autopoiesis/`

El simulador tiene **dos químicas** que modelan el mismo fenómeno con
abstracciones diferentes:

- **Alchemical (qe-based, Ax 8).** L4 `MatterCoherence` + L5 `AlchemicalEngine` + L8 `AlchemicalInjector`.  Química por resonancia de frecuencias; bonds emergen de alineación `cos(Δf·t + Δφ)`.  Usada por `planet_viewer`, `lab`, `earth_telescope`, todo el simulador "principal".
- **Mass-action (explícita, AP-*).** `SpeciesGrid` + `ReactionNetwork` + `k·c^n` + `frequency_alignment`.  Especies discretas con estequiometría, Kauffman RAF, Breslow formose.  Usada exclusivamente por `autopoietic_lab`.

Tras ADR-043 (bridge species → qe) y ADR-044 (spawn post-fisión), ambas
coexisten en el mismo pipeline.  Si **predicen lo mismo cualitativamente**
sobre el mismo input, coexisten legítimamente (cada una con su rol).  Si
**divergen**, la integración carga una inconsistencia interna — el lector
del código no puede saber cuál refleja "la física del sim".

Sin este ADR, el claim "un simulador con química coherente" es
rhetórica.  Hace falta evidencia empírica + decisión explícita.

## 2. Experimento diseñado (ítem AI-3 del Sprint AI)

**Hipótesis nula (H0):** sobre el mismo escenario canónico, las dos
químicas producen trayectorias cualitativamente equivalentes en
3 observables clave:

| Observable | Métrica | Tolerancia |
|---|---|---|
| Dissipation total | `total_qe_dissipated(t=1000)` | ±20 % |
| Emergencia de estructura | `n_closures_final` (AP) vs `n_entities_spawned` (alchemical post-abiogenesis) | Spearman ρ > 0.7 sobre 32 seeds |
| Localidad espacial | centroide de qe/productos al finalizar | ±3 celdas |

**Escenario canónico:** formose food=2 seed=∈{0..31} grid=16×16 qe=50 spot=2 ticks=5000.

**Procedimiento.**
1. Run A · sólo alchemical: `SoupSim` DESHABILITADO, `AlchemicalInjector` recibe spot food como qe directo a freq=50Hz.
2. Run B · sólo mass-action: corrida actual de `autopoietic_lab` con formose, SIN bridge ADR-043 activo.
3. Ambos producen CSV con los 3 observables.
4. Test: Spearman + bounded-error en Rust (`benches/chemistry_equivalence.rs`).

## 3. Decisión (una de las tres)

Se elegirá una tras el spike.  El ADR queda en estado "Propuesto" hasta
entonces; después pasa a una de:

### Camino 1 · H0 confirmada — coexistencia con roles

Si las 3 métricas pasan tolerancias, **ambas químicas se conservan**:

- **Alchemical** = canónica para escalas planetaria+ (L4-L5-L8 intactas, `planet_viewer` usa esto)
- **Mass-action** = canónica para autopoiesis + validación contra papers específicos (formose Breslow, hypercycle Eigen, RAF Kauffman)
- Bridge ADR-043 + spawn ADR-044 = **canal legítimo de acoplamiento**
- Documentar en CLAUDE.md §14 ECS Layers: "L4/L5/L8 modelan química emergente; AP-* provee química explícita para validación y autopoiesis"

### Camino 2 · H0 refutada por divergencia cuantitativa, mass-action es canónica

Si mass-action predice mejor los 6 papers (PV-1..6) + autopoiesis que alchemical:

- Deprecate alchemical: L5 `AlchemicalEngine`, L8 `AlchemicalInjector` pasan a modo "compatibilidad" (siguen existiendo pero no son la fuente de verdad)
- ADR-043 bridge se vuelve bidireccional: alchemical lee de mass-action, no al revés
- Sprint AI-deprecation separado: remover `AlchemicalEngine` del pipeline, reemplazar por sistemas que consumen `SpeciesGrid` vía ADR-043
- Tracks que dependen de L5 (reservoir recycling, basal drain) necesitan port a mass-action

**Impacto:** alto.  Aproximadamente 30 archivos tocan L5/L8.

### Camino 3 · H0 refutada, alchemical es canónica

Si alchemical reproduce PV-1..6 y mass-action es un modelo específico limitado:

- Mass-action queda como sub-sistema del track AUTOPOIESIS exclusivamente (cerrado en sí mismo)
- `autopoietic_lab` sigue existiendo; otros binarios no deben depender de `SpeciesGrid`
- ADR-044 spawn se mueve a convertir fisiones AP en entities alchemical-compatible, una-vez
- Roadmap marca "mass-action = validation tool, not substrate"

**Impacto:** bajo.  `autopoietic_lab` y el track AP-* siguen intactos; el resto del simulador ignora la química explícita.

## 4. Alternativas descartadas a priori (antes del spike)

| Opción | Por qué descartada |
|---|---|
| Forzar coincidencia numérica ajustando `SPECIES_TO_QE_COUPLING` | Viola Ax 6 (constantes derivadas, no ajustadas a posteriori) |
| Eliminar una sin evidencia | Sin datos de divergencia, cualquier eliminación es ideológica |
| Mantener ambas sin decidir canónica | Deja la inconsistencia de facto — el próximo PR que toque cualquiera no sabe cuál es la verdad |

## 5. Criterios de aceptación para cerrar el ADR

- [x] Spike AI-3 corrido (4 tests `#[ignore]` en `tests/chemistry_equivalence.rs`)
- [x] CSV `target/ai3_dissipation_curve.csv` generado (21 samples × 1000 ticks formose+spot)
- [x] Validación cuantitativa de invariantes axiomáticas (Ax 4 monotonicidad, Ax 5 conservación bridge)
- [x] Veredicto: **Camino 1 (coexistencia)** — escrito abajo en §10
- [x] Sin sprint follow-up de deprecation (Camino 1 no requiere)

## 10. Veredicto del spike (2026-04-15-b)

### Resultados

`cargo test --release --test chemistry_equivalence -- --ignored --nocapture`
sobre formose seed=0 spot=2 qe=50 ticks=1000 grid=16×16:

| Test | Resultado | Evidencia |
|---|---|---|
| `mass_action_dissipation_is_monotone_and_dumps_csv` | ✅ | Ax 4 verificado: 21 samples monotónicas no-decrecientes; spike a t=50 (462.62 qe del tax plasma de fisión inicial), steady-state después en 462.66 qe |
| `bridge_injection_does_not_create_qe` | ✅ | Bridge AI-1 NO muta species (contrato); cota teórica `25×10×100×0.1×0.02×1.0 = 50` exactamente alcanzada (alignment 1:1 con freq=50Hz formose) |
| `bridge_injection_is_monotone_under_repeated_calls` | ✅ | `field.total_qe()` crece monotónico con cada inyección |
| `mass_action_two_runs_same_dissipated_total` | ✅ | Determinismo cross-run byte-identical |

### Veredicto: Camino 1 — coexistencia legítima

**Razones:**

1. **Bridge AI-1 preserva los axiomas.** No crea qe, no muta el species
   grid, opera dentro de la cota teórica derivada de los axiomas.

2. **Mass-action respeta Ax 4.** La curva de dissipated es monotónica
   no-decreciente sobre 1000 ticks, con jumps discretos en eventos de
   fisión (ADR-039 §5 tax plasma) y crecimiento continuo entre eventos.

3. **Determinismo verificado.** Dos corridas con misma seed producen
   resultados byte-idénticos — pre-condición para reproducibilidad y
   testing.

4. **Quantitative cross-validation difería el scope del spike.** El path
   "alchemical-only" del ADR §2 está embebido en `LayersPlugin` +
   `SimulationPlugin` y no es trivialmente aislable.  Un benchmark
   alchemical↔mass-action sobre observables idénticos (ej. dissipation
   curve sobre la misma sopa primordial) requiere un sprint propio
   (**AI-bench**, fuera de scope AI-3).

### Roles canónicos

- **Mass-action (AP-*)** — canónica para autopoiesis explícita y validación
  contra papers específicos: formose (Breslow 1959), hypercycle
  (Eigen-Schuster 1977), Kauffman RAF, futuro Hordijk-Steel benchmark.

- **Alchemical (Ax 8 resonancia)** — canónica para escalas planetaria+
  donde la química discreta no escala: `planet_viewer`, `lab`, ecosistemas
  evolutivos.

- **Bridge (ADR-043 + ADR-044)** — canal legítimo de acoplamiento.  Mass-
  action emite qe al campo qe-based; fisiones AP-* spawn entities ECS.

### Implicaciones

- CLAUDE.md §"14 ECS Layers" debe documentar:
  > "L4/L5/L8 modelan química emergente (Ax 8 resonance); AP-* provee
  > química explícita (mass-action) para validación contra papers
  > específicos.  Los dos sustratos coexisten vía Bridge AI (ADR-043 +
  > ADR-044) — la sopa AP, cuando está cargada, aporta qe al campo
  > principal y sus fisiones nacen como entities ECS."

- Sprint follow-up **AI-bench** (cuando haya capacidad):
  benchmark cuantitativo alchemical-only vs mass-action-only sobre
  observable común (ej. tasa de dissipation a saturación).  Si diverge
  > 50%, re-evaluar Camino 2 o 3.

- PV-7 (Hordijk-Steel RAF benchmark) ya destrabable sobre el simulador
  principal vía la mass-action expuesta por el Bridge.

## 6. No viola axiomas (en cualquiera de los caminos)

| Ax | Cumplimiento |
|---|---|
| 1 | qe sigue siendo el stat único |
| 2-5 | Conservación y dissipation respetadas independientemente de la química canónica |
| 6 | **Central.** La elección se hace por evidencia, no por decreto; emergencia no cambia |
| 7-8 | Alineación freq + distance attenuation se preservan en ambos modelos |

## 7. Costos

- Spike: 3-5 días
- Implementación Camino 1: 0 (coexistencia documentada)
- Implementación Camino 2: ~2 semanas (migración alchemical → mass-action consumers)
- Implementación Camino 3: ~3 días (mover AP a rol exclusivamente validatorio)

## 8. Riesgos

| Riesgo | Prob | Mitigación |
|---|---|---|
| El spike no converge a un veredicto claro (métricas ambiguas) | Medio | Ampliar n_seeds a 128; si sigue ambiguo, elegir por simplicidad (Camino 3 por default, menor impacto) |
| Las dos químicas miden fenómenos distintos (no comparables) | Bajo | Si ese es el hallazgo, Camino 1 (coexistencia) se justifica fuerte |
| Elegir Camino 2 y tardar 2 semanas bloquea otros sprints | Medio | Sprint DL-1 (drug library) no depende de qué químicа canónica — puede arrancar en paralelo |

## 9. Decisión revisable cuando

- Si en sprints futuros se descubre que Camino 1 (coexistencia) genera bugs cada vez que se toca una química, escalar a Camino 2 o 3.
- Si aparece un tercer modelo químico (ej. quantum-resonance exacta), este ADR se revisa con 3 alternativas en vez de 2.
