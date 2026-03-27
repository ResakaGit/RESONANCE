# Track — Energy Competition: Pools Jerárquicos y Extracción Competitiva

**Blueprint:** Blueprint Energy Competition Layer (documento conceptual)
**Alineación:** Axioma 1 (todo es energía) + Axioma 2 (pool invariant) + Axioma 3 (competencia como única interacción primitiva). Filosofía "math in equations/" + "one system, one transformation" + stateless-first.
**Metodología:** TDD, funciones puras en `blueprint/equations/energy_competition/`, Writer pattern para contabilidad, DOD estricto.

---

## Objetivo del track

Implementar el sistema de **pools de energía jerárquicos** donde entidades hijas extraen energía de pools padres mediante **funciones de extracción** tipadas (Proporcional, Greedy, Competitiva, Agresiva, Regulada). La competencia es la interacción primitiva; cooperación, simbiosis y equilibrio emergen de las condiciones de extracción. Conservación estricta: `Sigma energy(children) <= energy(parent)` en todo tick.

**Resultado jugable:** un ecosistema donde criaturas compiten por energía de zonas del mapa. Poblaciones oscilan naturalmente (Lotka-Volterra emergente). Parásitos colapsan hosts. Organismos regulados alcanzan homeostasis. Todo sin lógica explícita — emerge de las funciones de extracción interactuando con pools finitos.

---

## Principio fundamental

> La competencia no se programa. Se resuelve. Es la consecuencia inevitable de N agentes extrayendo de un pool finito. Todo lo demás — cooperación, parasitismo, equilibrio, colapso — es un patrón emergente de la distribución de energía.

El modelo es **escala-invariante** (Matryoshka): célula, órgano, organismo, población usan la misma mecánica. Un hijo con una función de extracción, compartiendo un pool padre.

```
Parent Pool (finite qe)
  │
  ├── Child A: extract_fn_a(pool, state) → claimed_a
  ├── Child B: extract_fn_b(pool, state) → claimed_b
  └── Child C: extract_fn_c(pool, state) → claimed_c
  │
  ▼ Invariant enforcement
  Σ claimed ≤ pool
  loss = pool × dissipation_rate   (loss > 0 always — second law)
  pool(t+1) = pool(t) + intake - Σ claimed - loss
```

---

## Grafo de dependencias

```
EC-1 (Pool Equations)                    ── Onda 0 — bloqueante para todos
     │
     ├──► EC-2 (Pool Components)         ── Onda A
     │         │
     ├──► EC-3 (Extraction Registry)     ── Onda A (paralelo con EC-2)
     │         │
     │         ▼
     │    EC-4 (Pool Distribution System)── Onda B (requiere EC-2 + EC-3)
     │         │
     │         ├──► EC-5 (Competition    ── Onda C
     │         │         Dynamics)
     │         │
     │         ├──► EC-6 (Conservation   ── Onda C (paralelo con EC-5)
     │         │         Ledger)
     │         │
     │         └──► EC-7 (Scale-Invariant── Onda D (requiere EC-5 + EC-6)
     │                    Composition)
     │
     └──► EC-8 (Integration Demo)        ── Onda E (requiere todo)
```

## Ondas de ejecución

| Onda | Sprints | Qué habilita |
|------|---------|-------------|
| **0** | EC-1 | Funciones puras: conservación, extracción (5 tipos), disipación, fitness |
| **A** | EC-2, EC-3 (paralelo) | Componentes de pool + registro de funciones de extracción |
| **B** | EC-4 | Sistema de distribución: el tick de extracción competitiva |
| **C** | EC-5, EC-6 (paralelo) | Dinámica de competencia (matriz, dominancia, colapso) + ledger de conservación |
| **D** | EC-7 | Composición escala-invariante: fitness de padre inferido de hijos |
| **E** | EC-8 | Demo integrada, EntityBuilder, benchmark |

## Índice de sprints

| Sprint | Archivo | Módulo principal | Onda | Dependencias | Estado |
|--------|---------|-----------------|------|--------------|--------|
| [EC-1](SPRINT_EC1_POOL_EQUATIONS.md) | Pool Equations | `src/blueprint/equations/energy_competition/pool_equations.rs` | 0 | — | ✅ |
| [EC-2](SPRINT_EC2_POOL_COMPONENTS.md) | Pool Components | `src/layers/energy_pool.rs`, `src/layers/pool_link.rs` | A | EC-1 | ✅ |
| [EC-3](SPRINT_EC3_EXTRACTION_REGISTRY.md) | Extraction Registry | `src/blueprint/equations/energy_competition/extraction.rs` | A | EC-1 | ✅ |
| [EC-4](SPRINT_EC4_POOL_DISTRIBUTION_SYSTEM.md) | Pool Distribution System | `src/simulation/metabolic/pool_distribution.rs` | B | EC-2, EC-3 | ✅ |
| [EC-5](SPRINT_EC5_COMPETITION_DYNAMICS.md) | Competition Dynamics | `src/blueprint/equations/energy_competition/dynamics.rs`, `src/simulation/metabolic/competition_dynamics.rs` | C | EC-4 | ✅ |
| [EC-6](SPRINT_EC6_CONSERVATION_LEDGER.md) | Conservation Ledger | `src/layers/pool_ledger.rs`, `src/simulation/metabolic/pool_conservation.rs` | C | EC-4 | ✅ |
| [EC-7](SPRINT_EC7_SCALE_INVARIANT_COMPOSITION.md) | Scale-Invariant Composition | `src/blueprint/equations/energy_competition/scale.rs`, `src/simulation/metabolic/scale_composition.rs` | D | EC-5, EC-6 | ✅ |
| [EC-8](SPRINT_EC8_INTEGRATION_DEMO.md) | Integration Demo | `src/entities/archetypes/competition.rs`, `tests/energy_competition_integration.rs` | E | Todos | ✅ |

---

## Paralelismo seguro

| | EC-1 | EC-2 | EC-3 | EC-4 | EC-5 | EC-6 | EC-7 | EC-8 |
|---|---|---|---|---|---|---|---|---|
| **EC-2** | | — | ✅ | | | | | |
| **EC-3** | | ✅ | — | | | | | |
| **EC-5** | | | | | — | ✅ | | |
| **EC-6** | | | | | ✅ | — | | |

EC-2 y EC-3 son paralelos (Onda A): no comparten archivos de escritura.
EC-5 y EC-6 son paralelos (Onda C): no comparten archivos de escritura.

---

## Invariantes del track

1. **Pool invariant absoluto.** `Sigma energy(children) <= energy(parent)` verificado post-tick. Violación = bug, no estado válido.
2. **Dissipation floor.** `loss >= pool * dissipation_rate`, `dissipation_rate in (0, 1)`. Ningún proceso es 100% eficiente.
3. **Math in equations/.** Toda ecuación de extracción, conservación, fitness y dinámica es función pura en `blueprint/equations/energy_competition/`. Sistemas solo orquestan queries y llaman puras.
4. **Max 4 campos por componente.** `EnergyPool` tiene 4 campos. `PoolParentLink` tiene 3 campos. Cumple DOD.
5. **SparseSet para componentes de competencia.** `PoolParentLink`, `CompetitionLedger` — solo entidades en jerarquía activa.
6. **Guard change detection.** Todo sistema verifica `if old != new` antes de mutar.
7. **Determinismo.** Orden de extracción determinista (topológico por `Entity` index). Misma seed → misma distribución.
8. **Sin RNG.** Mismos inputs → misma distribución de energía.
9. **Backward compatible.** Entidades sin `EnergyPool` → `BaseEnergy` funciona exacto como hoy. Cero regresión.
10. **Phase assignment.** Sistemas de distribución → `.in_set(Phase::MetabolicLayer)`. Ledger → `.after(pool_distribution_system)`.

## Contrato de pipeline EC

```
FixedUpdate:
  SimulationClockSet
  → Phase::Input
  → Phase::ThermodynamicLayer     ← existente (thermal, dissipation)
  → Phase::AtomicLayer
  → Phase::ChemicalLayer
  → Phase::MetabolicLayer         ← pool_intake_system
                                   ← pool_distribution_system (.after intake)
                                   ← pool_conservation_system (.after distribution)
                                   ← competition_dynamics_system (.after conservation)
                                   ← scale_composition_system (.after dynamics)
  → Phase::MorphologicalLayer
```

---

## Relación con sistemas existentes

| Sistema existente | Interacción con EC |
|---|---|
| `BaseEnergy` (L0) | EC **no reemplaza** BaseEnergy. `EnergyPool` extiende el concepto: una entidad puede tener `BaseEnergy` (su qe propio) + `EnergyPool` (pool distribuible a hijos) |
| `competitive_exclusion` (EA7) | El drain plano de EA7 se puede expresar como Type III extraction. EC generaliza y subsume |
| `trophic` systems | Predación = Type II (Greedy) extraction de prey pool. EC provee el framework; trophic lo instancia |
| `reproduction` | Parent → seed transfer = one-shot extraction. EC modela la relación persistente post-spawn |
| `EntropyLedger` (MG-6) | El ledger de conservación EC es análogo pero para pools, no para DAGs metabólicos |
| `ContainedIn` | EC usa `PoolParentLink` (ownership), no `ContainedIn` (spatial inference). Ortogonales |

---

## Ejemplo motivador: Ecosistema competitivo

```
Zona_Bosque entity:
  L0: BaseEnergy = 10000 qe
  EnergyPool: pool=10000, capacity=10000, intake_rate=50/tick (sol), dissipation_rate=0.001

  Child: Arbol_A
    PoolParentLink: parent=Zona_Bosque, extraction=Type_III(fitness=0.6)
    BaseEnergy = 300 qe
    → extract = 10000 × (0.6 / (0.6+0.3+0.1)) = 6000 → clamped by capacity

  Child: Arbol_B
    PoolParentLink: parent=Zona_Bosque, extraction=Type_III(fitness=0.3)
    BaseEnergy = 200 qe
    → extract = 10000 × (0.3 / 1.0) = 3000 → clamped

  Child: Parasito
    PoolParentLink: parent=Zona_Bosque, extraction=Type_IV(aggression=0.8, damage=0.1)
    BaseEnergy = 50 qe
    → extract = 10000 × 0.8 = 8000 → clamped
    → pool_damage = 8000 × 0.1 = 800 → capacity decreases

  ▼ pool_distribution_system (deterministic order)
  Total fitness demand > pool → proportional scaling
  Parasito degrades capacity → next tick, less available for all
  If unchecked → host collapse (pool → 0) → all children starve
  If Arbol_A has Type_V (regulated) → throttles when pool_ratio < 0.3
  → natural oscillation emerges: Lotka-Volterra without explicit implementation
```

---

## Referencias cruzadas

- Blueprint Energy Competition Layer — documento conceptual (axiomas, pool model, extraction types)
- `docs/design/MORPHOGENESIS.md` — Matrioska functional composition
- `src/layers/energy.rs` — BaseEnergy (L0), EnergyOps
- `src/layers/entropy_ledger.rs` — Precedente de ledger derivado (MG-6)
- `src/simulation/lifecycle/competitive_exclusion.rs` — EA7 competition drain
- `src/blueprint/equations/trophic.rs` — Prey extraction model
- `src/simulation/reproduction/` — Parent-child energy transfer
- `DESIGNING.md` — Axioma energético y filosofía de capas
