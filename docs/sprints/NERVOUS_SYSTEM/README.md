# Track: NERVOUS_SYSTEM — Red de señales emergente

Células especializadas como "neuronas" se conectan via StructuralLink (L13, ya existe).
Señales se propagan por cadenas de links. Velocidad ∝ bond_energy (mielinización emerge).
Decisiones = cuál señal llega primero al WillActuator.

**Invariante:** Ninguna neurona se programa. Emerge de alta frequency + baja resilience + links.

---

## Qué ya existe

| Componente | Archivo | Uso en NS |
|-----------|---------|-----------|
| `StructuralLink` (L13) | `layers/structural_link.rs` | Axón = spring joint entre células |
| `OscillatorySignature` (L2) | `layers/oscillatory.rs` | Señal = frequency pulse |
| `SensoryAwareness` | `layers/behavior.rs` | Input sensorial → trigger |
| `WillActuator` (L7) | `layers/will.rs` | Output motor → acción |
| `EpigeneticState` | `layers/epigenetics.rs` | Diferenciación neuronal |
| Multicelularidad (MC) | `equations/multicellular.rs` | Colonia = organismo con neuronas |

## Sprints (4)

| Sprint | Nombre | Esfuerzo | Bloqueado por | Entregable |
|--------|--------|----------|---------------|------------|
| [NS-1](SPRINT_NS1_SIGNAL_PROPAGATION.md) | Signal Propagation | 1 sem | MC ✅ | `propagate_signal()` — frequency pulse por cadena de links |
| [NS-2](SPRINT_NS2_ACTIVATION_THRESHOLD.md) | Activation Threshold | 1 sem | NS-1 | `neuron_fires()` — signal > threshold → propaga o no |
| [NS-3](SPRINT_NS3_REFLEX_ARC.md) | Reflex Arc | 1 sem | NS-2 | Sensory → signal chain → WillActuator (reflex completo) |
| [NS-4](SPRINT_NS4_BATCH_WIRING.md) | Batch Integration | 1 sem | NS-3 | Signal propagation en batch + observabilidad |

## Arquitectura de archivos

```
src/blueprint/
├── equations/
│   ├── neural_signal.rs         ← NS-1/2: propagate_signal, neuron_fires, signal_speed
│   └── reflex_arc.rs            ← NS-3: sensory_to_motor, reflex_latency
├── constants/
│   └── neural.rs                ← Constantes: threshold, propagation_speed, decay
src/batch/systems/
│   └── neural.rs                ← NS-4: signal_propagation_step batch system
```

## Axiomas

| Axioma | Cómo aplica |
|--------|-------------|
| 4 | Signal disipa energía al propagarse (Axiom 4: no 100% transmission) |
| 7 | Signal decae con distancia (link length) |
| 8 | Signal = frequency pulse. Neuronas con freq compatible amplifican. Incompatibles atenúan. |
| 6 | Red nerviosa emerge de cuáles links se refuerzan (Hebb, ya implementado MGN-6) |

## Constantes derivadas

| Constante | Derivación |
|-----------|-----------|
| `SIGNAL_PROPAGATION_SPEED` | `1.0 / DISSIPATION_SOLID` — cuántos links por tick |
| `NEURON_ACTIVATION_THRESHOLD` | `KLEIBER_EXPONENT` — 0.75 del máximo signal |
| `SIGNAL_DECAY_PER_LINK` | `DISSIPATION_SOLID × 4` — pérdida por salto |
| `SIGNAL_COST` | `DISSIPATION_SOLID × 2` — qe cost por signal emitido |

## Tests por sprint

| Sprint | Tests de contrato | Tests de lógica | Tests de error |
|--------|:-:|:-:|:-:|
| NS-1 | signal conserva energía, decae por link | velocidad ∝ bond_energy, decay ∝ distance | NaN signal, zero links |
| NS-2 | threshold respetado, sub-threshold no propaga | freq alignment amplifica, misalign atenúa | threshold=0 fires always |
| NS-3 | reflex input→output funcional | latencia ∝ chain length, faster reflexes = more links | no sensory = no reflex |
| NS-4 | batch conserva energy, deterministic | signal count observable | empty world no panic |

## Esfuerzo total: ~4 semanas, ~400 LOC, ~50 tests
