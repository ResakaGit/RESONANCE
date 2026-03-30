# Track: EMERGENT_LANGUAGE — Comunicación simbólica emergente

Entidades emiten señales (frequency pulses) con "significado" aprendido.
Vocabulario crece por innovación + imitación (cultural_transmission, ya existe).
Composicionalidad: combinar 2 señales en 1 significado nuevo.

**Invariante:** Ninguna palabra se programa. Emerge de señal + asociación + imitación.

---

## Qué ya existe

| Componente | Uso |
|-----------|-----|
| `LanguageCapacity` (4 fields) | vocabulary[8], signal_range, encoding_cost, vocab_count |
| `cultural_transmission_system` | Meme spread by imitation (ET-3) |
| `OscillatorySignature` (L2) | Signal carrier (frequency pulse) |
| `SensoryAwareness` | Signal receiver |
| `coalition_stability_system` | Group identity (shared language = coalition marker) |

## Sprints (4)

| Sprint | Nombre | Esfuerzo | Bloqueado por | Entregable |
|--------|--------|----------|---------------|------------|
| [EL-1](SPRINT_EL1_SIGNAL_EMISSION.md) | Signal Emission | 1 sem | NS ✅ | `emit_signal()` — freq pulse con "label" del vocabulario |
| [EL-2](SPRINT_EL2_ASSOCIATION.md) | Signal-Event Association | 1 sem | EL-1 | `learn_association()` — señal + evento cercano → memoria |
| [EL-3](SPRINT_EL3_COMPOSITIONALITY.md) | Compositionality | 1 sem | EL-2 | `compose_signals()` — 2 señales → 1 significado combinado |
| [EL-4](SPRINT_EL4_BATCH_WIRING.md) | Batch Integration | 1 sem | EL-3 | Language en batch + vocab_size_mean observable |

## Arquitectura

```
src/blueprint/equations/
├── language_signal.rs           ← EL-1/2/3: emit_signal, learn_association, compose_signals
src/blueprint/constants/
├── language.rs                  ← Constantes derivadas
src/batch/systems/
├── language.rs                  ← EL-4: batch systems
```

## Axiomas

| Axioma | Cómo aplica |
|--------|-------------|
| 4 | Emitir señal cuesta energía (encoding_cost). Vocabulario grande cuesta más mantener. |
| 7 | Signal range limitado por distancia (signal_range en LanguageCapacity). |
| 8 | Signal = frequency pulse. Composición = interference de 2 frecuencias → freq emergente. |
| 6 | Vocabulario emerge de qué asociaciones sobreviven selección. |

## Constantes derivadas

| Constante | Derivación |
|-----------|-----------|
| `SIGNAL_EMISSION_COST` | `DISSIPATION_SOLID × 3` |
| `ASSOCIATION_LEARNING_RATE` | `DISSIPATION_SOLID × 10` |
| `VOCAB_MAINTENANCE_COST` | `DISSIPATION_SOLID × vocab_count` (Kleiber-like) |
| `COMPOSITION_THRESHOLD` | `KLEIBER_EXPONENT` — min association strength para combinar |

## Esfuerzo total: ~4 semanas, ~350 LOC, ~40 tests
