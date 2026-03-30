# Track: MULTICELLULARITY — Organismos multicelulares emergentes

Entidades se unen vía StructuralLink (L13, ya existe) para formar organismos compuestos.
Cada célula expresa genes diferentes (epigenética, ya existe).
Especialización emerge: borde=defensa, interior=metabolismo.
División del trabajo emerge de physics, no de templates.

**Invariante:** Nadie le dice a una célula "sé piel" o "sé hígado".
La especialización es consecuencia de posición + presión + energía.
Axiom 6 estricto.

---

## Qué ya existe (no se toca)

| Componente | Archivo | Estado | Rol en multicelularidad |
|-----------|---------|--------|------------------------|
| `StructuralLink` (L13) | `layers/structural_link.rs` | ✅ 4 fields | Spring joint entre entidades → adhesión celular |
| `EpigeneticState` | `layers/epigenetics.rs` | ✅ expression_mask | Expresión diferencial por célula |
| `MetabolicGraph` | `layers/metabolic_graph.rs` | ✅ DAG 12 nodos | Red metabólica interna por célula |
| `NicheProfile` | `layers/niche.rs` | ✅ 4D hypervolume | Especialización ecológica → celular |
| `SymbiosisLink` | `layers/symbiosis.rs` | ✅ SparseSet | Mutualismo entre células = cooperación |
| `collision_interference_system` | `simulation/thermodynamic/physics.rs` | ✅ | Detección de contacto → trigger adhesión |
| `structural_constraint_system` | `simulation/thermodynamic/` | ✅ | Mantiene spring joints → mantiene tejido |
| Batch: `tension_field_apply` | `batch/systems/atomic.rs` | ✅ | Fuerzas entre entidades |
| Batch: `collision` | `batch/systems/atomic.rs` | ✅ | Detección de contacto |

## Sprints (5)

| Sprint | Nombre | Esfuerzo | Bloqueado por | Entregable |
|--------|--------|----------|---------------|------------|
| [MC-1](SPRINT_MC1_ADHESION.md) | Cell Adhesion | Medio | — | `adhesion_affinity()` — cuándo dos células se unen |
| [MC-2](SPRINT_MC2_COLONY_DETECTION.md) | Colony Detection | Bajo | MC-1 | `detect_colonies()` — clusters de cells linked |
| [MC-3](SPRINT_MC3_POSITION_SIGNAL.md) | Positional Signaling | Medio | MC-2 | `positional_gradient()` — señal borde/interior |
| [MC-4](SPRINT_MC4_DIFFERENTIAL_EXPRESSION.md) | Differential Expression | Medio | MC-3 | Borde cells silencian growth, expresan shell |
| [MC-5](SPRINT_MC5_BATCH_WIRING.md) | Batch Integration | Medio | MC-4 | Adhesión + colonia + expresión en batch loop |

---

## Dependency chain

```
StructuralLink (L13) ✅ + EpigeneticState (ET-6) ✅
    │
    ▼
MC-1: Cell Adhesion (cuándo se unen)
    │
    ▼
MC-2: Colony Detection (quién es grupo)
    │
    ▼
MC-3: Positional Signaling (señal borde/interior)
    │
    ▼
MC-4: Differential Expression (genes diferentes por posición)
    │
    ▼
MC-5: Batch wiring (todo corre en evolución)
```

## Arquitectura de archivos

```
src/
├── blueprint/
│   ├── equations/
│   │   ├── cell_adhesion.rs        ← MC-1: adhesion_affinity, bond_strength
│   │   ├── colony_detection.rs     ← MC-2: detect_colonies, colony_size
│   │   └── positional_signal.rs    ← MC-3: gradient_from_position, border_signal
│   └── constants/
│       └── multicellular.rs        ← MC-1→4: adhesion threshold, signal range, etc.
├── batch/
│   └── systems/
│       └── multicellular.rs        ← MC-5: adhesion + colony + expression batch systems
├── layers/
│   └── structural_link.rs          ← YA EXISTE (L13) — no se modifica
└── simulation/
    └── emergence/
        └── multicellularity.rs     ← MC-5: Bevy systems (Future, no en este track)
```

## Patrones por rol

| Rol | Patrón | Ejemplo |
|-----|--------|---------|
| **Ecuación adhesión** | `fn(cell_a, cell_b) → f32`, stateless | `adhesion_affinity(freq_a, freq_b, distance) → [0,1]` |
| **Detección colonia** | `fn(links) → [colony_id; N]`, graph traversal | `detect_colonies(adjacency) → cluster assignments` |
| **Señal posicional** | `fn(colony, cell_pos) → f32`, gradient | `border_signal(colony_center, cell_pos, colony_radius) → [0,1]` |
| **Expresión diferencial** | `fn(signal, mask) → mask'`, modulator | `modulate_expression(border_signal, mask) → new_mask` |
| **Cache** | Struct por colonia, computado 1×/tick | `ColonyPhenotype { size, center, border_cells, interior_cells }` |

## El mecanismo completo

```
Tick N:
  1. Dos células colisionan
  2. adhesion_affinity(freq_a, freq_b, distance) > threshold? (Axiom 8: frequency match)
  3. SI → StructuralLink creado entre ellas (L13, spring joint)

Tick N+1:
  4. detect_colonies() encuentra cluster de 3+ cells linked
  5. positional_gradient() computa señal borde/interior para cada célula
  6. Células BORDE: expression_mask modifica → silencia growth, expresa resilience (shell)
  7. Células INTERIOR: expression_mask modifica → silencia resilience, expresa growth (core)
  8. → ESPECIALIZACIÓN EMERGE: borde=defensa, interior=metabolismo
  9. → DIVISION DEL TRABAJO sin programar roles

Evolución:
  10. Organismos multicelulares con especialización → mejor fitness
  11. → selección natural favorece adhesión + expresión diferencial
  12. → multicelularidad se propaga
  13. → tamaño del organismo crece
  14. → más especialización → más fitness → ciclo positivo
```

## Axiomas en cada sprint

| Sprint | Ax1 | Ax3 | Ax4 | Ax6 | Ax7 | Ax8 |
|--------|:---:|:---:|:---:|:---:|:---:|:---:|
| MC-1 | cells=energy | — | adhesion has cost | bond emerges | distance decay | freq match |
| MC-2 | colony=energy pool | — | — | topology emerges | — | — |
| MC-3 | signal=energy gradient | — | signal dissipates | gradient emerges | signal decays | — |
| MC-4 | — | cells compete for role | expression costs | specialization emerges | position matters | freq→role |
| MC-5 | — | — | maintenance cost | — | — | — |

## Constantes derivadas

| Constante | Valor | Derivación |
|-----------|-------|-----------|
| `ADHESION_THRESHOLD` | 0.75 | `KLEIBER_EXPONENT` (same as metabolic scaling) |
| `ADHESION_FREQ_BANDWIDTH` | 50.0 | `COHERENCE_BANDWIDTH` (4th fundamental) |
| `ADHESION_COST` | 0.01 | `DISSIPATION_SOLID × 2` (maintaining bond) |
| `MIN_COLONY_SIZE` | 3 | `MIN_GENES - 1` (minimum for gradient) |
| `BORDER_SIGNAL_DECAY` | 0.1 | `DISSIPATION_SOLID × 20` (signal range) |
| `EXPRESSION_MODULATION_RATE` | 0.05 | `DISSIPATION_SOLID × 10` (how fast cells specialize) |

---

## Resumen de cambios

| Archivo | Tipo | Cambio |
|---------|------|--------|
| `blueprint/equations/multicellular.rs` | Nuevo | adhesion_affinity, bond_strength, detect_colonies (Union-Find), border_signal, positional_gradient, modulate_expression, specialization_index. 27 tests. |
| `blueprint/constants/multicellular.rs` | Nuevo | 8 constantes derivadas de DISSIPATION_SOLID + KLEIBER. |
| `batch/systems/multicellular.rs` | Nuevo | multicellular_step: adhesion→colony→gradient→expression→cost. 6 tests. |
| `batch/pipeline.rs` | Mod | +multicellular_step en MorphologicalLayer. |
| `batch/harness.rs` | Mod | +multicellular_rate en GenerationStats + colony detection en observabilidad. |
