# ADR-018: Lab Universal — Refactor para casos de uso reales

**Estado:** Aceptado
**Fecha:** 2026-04-12
**Contexto:** LAB_UI_REFACTOR track (LR-1 a LR-4)

## Contexto

El lab binary (`src/bin/lab.rs`) tiene 7 de 20 experiments accesibles. 13 experiments
escritos, testeados y con BDD están inaccesibles salvo por CLI individual. El sprint
LR propuso 4 sub-sprints pero LR-1 y LR-4 ya están implementados (state machine +
match dispatch). Los gaps reales son: experiments faltantes, controles genéricos,
CSV incompleto, y UI que no escala a 15+ experiments.

## Decisión 1: Mantener match exhaustivo (rechazar ExperimentDef)

**Opciones evaluadas:**

| Criterio | Match exhaustivo | ExperimentDef (fn pointers) |
|----------|-----------------|----------------------------|
| Exhaustividad | Compilador fuerza cobertura | Runtime — campo olvidado = bug silencioso |
| Agregar experiment | 4 match arms (~20 LOC) | 1 struct (~10 LOC) |
| Tipo safety | Cada report tiene su propio tipo | Necesita Box<dyn Any> o enum wrapper |
| Coding rules | OK | Borderline con "no trait objects" |
| Mantenimiento | Compilador guía qué falta | Silencioso si olvidás un campo |

**Decisión:** Match exhaustivo. El compilador es la red de seguridad. Agregar un
experiment nuevo son 4 match arms que el compilador te obliga a escribir. El
ExperimentDef ahorra verbosidad pero pierde la garantía de completitud.

## Decisión 2: Categorización de experiments

15 experiments en lista plana es inutilizable. Categorías:

| Categoría | Experiments | Motivo |
|-----------|-------------|--------|
| Core Simulation | Lab, Fermi, Speciation, Cambrian, Debate, Convergence, Personal | Simulación base, emergencia |
| Drug & Therapy | Cancer Therapy, Pathway Inhibitor | Farmacodinámica, resistencia |
| Paper Validation | Zhang 2022, Sharma 2010, Foo & Michor 2009, Michor 2005, Unified Axioms | Validación científica peer-reviewed |
| Physics | Particle Lab | Coulomb + LJ, moléculas emergentes |

Headers visuales en la UI separan las categorías. Cada categoría es un grupo de
radio buttons bajo un `ui.heading()`.

## Decisión 3: Scope de Ablation/Ensemble

Ablation y Ensemble solo aplican a experiments que usan `ExperimentReport` (Lab,
Fermi, Speciation, Cambrian, Debate, Convergence, Personal). Los paper experiments
tienen configs propias incompatibles con el pipeline de ablation.

**Decisión:** Ocultar Run Mode selector cuando el experiment seleccionado no soporta
Ablation/Ensemble. Forzar `RunMode::Single` al seleccionar un paper/drug/physics
experiment.

## Decisión 4: Experiments excluidos

| Module | Razón de exclusión |
|--------|--------------------|
| `fossil` | Solo snapshots de genomas — valor bajo sin visualización 3D |
| `mesh_export` | Retorna OBJ string, no es un experiment |
| `sonification` | Retorna WAV bytes, no es un experiment |
| `versus` | Heurístico sin simulación completa |
| `paper_hill_ccle` | Sin `run()`, constantes de calibración |

Estos pueden agregarse en el futuro si se justifica la UI.

## Consecuencias

### Se gana
- 15 experiments accesibles (vs 7) sin tocar terminal
- Categorización visual que escala
- CSV export completo para todos los experiments
- Controles contextuales por experiment
- Un lab usable por investigadores

### Se pierde
- Archivo crece de ~950 a ~1600 LOC (aceptable para un binario)
- 60 match arms en 4 lugares (pero exhaustivos — el compilador los mantiene)
- Ablation/Ensemble no disponible para paper experiments (limitación inherente)

### Riesgo
- Archivo grande → candidato a split futuro si supera 2000 LOC
- Mitigación: funciones de rendering son independientes, extraíbles sin cambiar API

## Relación con otros ADRs
- ADR-013 (Stateless Experiment Contract): todos los experiments siguen config→report
- ADR-017 (Cache Integration): sin relación directa
