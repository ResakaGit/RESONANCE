# ADR-045: Elección canónica · alchemical (Ax 8) vs mass-action (AP-*)

**Estado:** Propuesto (decisión pendiente de spike AI-3)
**Fecha:** 2026-04-15
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

- [ ] Spike AI-3 corrido sobre 32 seeds
- [ ] CSV `results/chemistry_equivalence_{alchemical,mass_action}.csv` commiteado
- [ ] Script de análisis (`scripts/chemistry_equivalence_report.py` o test Rust) que computa las 3 métricas
- [ ] Veredicto: Camino 1 / 2 / 3 escrito en este ADR
- [ ] Si Camino 2 o 3: sprint follow-up de deprecation creado

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
