# Track: AUTOPOIESIS — Bucle autocatalítico cerrado, persistente, replicable

Cierre operacional del arco descrito en [`docs/sintesis_patron_vida_universo.md`](../../sintesis_patron_vida_universo.md). El simulador ya cumple los caps. 3 (universo disipativo) y 5-termo/química (autocatálisis posible). Falta el invariante destilado del cap. 10:

> **Lo que persiste es aquello que encontró una forma de copiarse antes de disiparse.**

Este track lo demuestra como **test ejecutable**, no como aspiración.

---

## Principio fundacional

Resonance ya tiene los ingredientes (qe, dissipation, frequency, distance attenuation). Faltan tres cosas para cerrar el bucle:

1. **Sustrato químico explícito** — concentraciones de especies + reacciones con cinética (no solo partículas individuales).
2. **Detección de cierre topológico** — identificar cuándo un subconjunto de reacciones forma un set autocatalítico (Kauffman RAF).
3. **Membrana + fisión emergentes** — contención espacial y replicación cuando la producción interna excede la cohesión de borde (Pross "kinetic stability").

Sin esto, el simulador puede formar moléculas pero no demuestra autopoiesis. Con esto, se cierra el arco: una sopa aleatoria → un bucle cerrado → contención → replicación → linaje.

---

## Sprints

### Onda 0 — Sustrato (ADR-037)

| Sprint | Nombre | Esfuerzo | Bloqueado por | Entregable |
|--------|--------|----------|---------------|------------|
| [AP-0](SPRINT_AP0_REACTION_NETWORK.md) | Reaction Network Substrate | 1.5 sem | — | `[f32; MAX_SPECIES]` per cell, `Reaction { reactants, products, k }`, mass-action stepping |

### Onda A — Detección (sin ADR — algoritmo puro)

| Sprint | Nombre | Esfuerzo | Bloqueado por | Entregable |
|--------|--------|----------|---------------|------------|
| [AP-1](SPRINT_AP1_CLOSURE_DETECTOR.md) | Autocatalytic Closure Detector | 1 sem | AP-0 | RAF-finder via Tarjan SCC, pure fn |
| [AP-2](SPRINT_AP2_KINETIC_STABILITY.md) | Kinetic Stability Metric | 0.5 sem | AP-1 | `kinetic_stability = reconstruction_rate / decay_rate` (Pross) |

### Onda B — Contención + Replicación

| Sprint | Nombre | Esfuerzo | Bloqueado por | Entregable | ADR |
|--------|--------|----------|---------------|------------|-----|
| [AP-3](SPRINT_AP3_MEMBRANE_FIELD.md) | Emergent Membrane | 1.5 sem | AP-0 | Cohesión por gradiente de densidad de productos (sin componente Membrana) | ADR-038 |
| [AP-4](SPRINT_AP4_FISSION_TRIGGER.md) | Fission Trigger | 1 sem | AP-3 | Fisión cuando `producción_interna > cohesión_membrana` | ADR-039 |

### Onda C — Validación (sin ADR — operacionalización)

| Sprint | Nombre | Esfuerzo | Bloqueado por | Entregable |
|--------|--------|----------|---------------|------------|
| [AP-5](SPRINT_AP5_PERSISTENCE_PROPTEST.md) ✅ | Persistence Property Test | 1 sem | AP-4 | proptest: closure persiste ⟺ se replicó o sostuvo producción ≥ decay |
| [AP-6a](SPRINT_AP6_AUTOPOIETIC_LAB.md) ✅ | `autopoietic_lab` headless + DOT | 0.5 sem | AP-5 | CLI stdlib, `SoupReport` JSON + `to_dot()` Graphviz |
| AP-6b ✅ | `--network <ron>` loader + `run_soup_with_network` | 0.25 sem | AP-6a | `raf_minimal.ron` wired; formose/GARD deferred a AP-6b2 (requieren citas) |
| AP-6b2 ✅ | formose + hypercycle canonical networks | 0.25 sem | AP-6b | Breslow 1959 / Kauffman 1986 + Eigen-Schuster 1977 con citas. GARD diferido (no mass-action). |
| AP-6c (designed) | Viz 2D Bevy + egui | 1 sem | AP-6b | Heatmap species + membrane + lineage tree UI |

**Total:** 7 sprints · ~8 semanas · ~2,000 LOC · ~150 tests · 3 ADRs.

---

## Dependency chain

```
AP-0 ──┬── AP-1 ── AP-2
       └── AP-3 ── AP-4 ── AP-5 ── AP-6a ── AP-6b ── AP-6c
```

AP-1/AP-2 paralelos con AP-3/AP-4 una vez AP-0 cierra.

---

## Axiomas (todos respetados)

| Axioma | Cómo aplica |
|--------|-------------|
| 1 | Especies son packets de qe localizados en celdas. |
| 2 | `sum(species_qe[cell]) ≤ cell_qe_capacity` — pool invariant. |
| 3 | Reacciones compiten por reactivos en común — extracción proporcional. |
| 4 | Cada paso de reacción disipa `(1 - efficiency) × qe_consumido`. |
| 5 | Conservación: `qe_in - qe_dissipated = qe_out`. |
| 6 | **Clave.** Membrana, fisión, linaje EMERGEN — ningún componente "Cell" o "Membrane" predefinido. |
| 7 | Difusión inter-celular atenúa con distancia. |
| 8 | Reactividad modulada por alineación de frecuencia (catálisis selectiva). |

---

## Constantes (todas derivadas de las 4 fundamentales)

| Constante | Derivación |
|-----------|-----------|
| `REACTION_EFFICIENCY` | `1.0 - DISSIPATION_LIQUID = 0.98` |
| `DIFFUSION_RATE` | `DISSIPATION_LIQUID = 0.02/tick` |
| `MEMBRANE_DENSITY_THRESHOLD` | `solid_density_threshold()` (de `derived_thresholds.rs`) |
| `FISSION_PRESSURE_RATIO` | `DISSIPATION_PLASMA / DISSIPATION_SOLID = 50.0` |
| `RAF_MIN_CYCLE_LENGTH` | 3 (Kauffman, no derivable — propiedad topológica) |
| `KINETIC_STABILITY_THRESHOLD` | 1.0 (reconstrucción ≥ decay para considerar persistente) |

---

## ADRs

| ADR | Tema | Sprints |
|-----|------|---------|
| [ADR-037](../../arquitectura/ADR/ADR-037-reaction-network-substrate.md) | Modelado de sustrato químico: SoA por celda vs entidad-por-molécula vs componente-por-especie | AP-0 |
| [ADR-038](../../arquitectura/ADR/ADR-038-emergent-membrane.md) | Membrana sin componente: cohesión emergente por TensionField + gradiente de densidad | AP-3 |
| [ADR-039](../../arquitectura/ADR/ADR-039-fission-criterion.md) | Criterio de fisión: presión interna vs volumen vs decoherencia oscilatoria | AP-4 |

---

## Invariantes del track

1. **Zero "Cell" component.** Una "célula" es un patrón espacial detectado, no una entidad declarada.
2. **Zero "Membrane" component.** La membrana es el iso-contour del gradiente de densidad de productos sobre el grid.
3. **Zero hardcoded reactions.** Las reacciones se cargan desde `assets/reactions/*.ron` o se generan aleatoriamente para tests.
4. **Property test es el contrato.** AP-5 es el test que define "autopoiesis" en código. Si pasa, el cap. 10 está operacionalizado.
5. **Stateless equations.** Toda fn en `blueprint/equations/reaction_*.rs` es `(inputs) → output`.
6. **Cache-friendly SoA.** `species: [f32; 32]` por celda, contiguous.
7. **Determinista.** Misma seed + misma topología de red → mismos linajes en t=10⁴.

---

## Criterio de "100%"

Track cerrado cuando:

- [ ] `cargo run --release --bin autopoietic_lab -- --headless --soup random --seed 42 --ticks 100000` reporta ≥ 1 closure persistente y ≥ 1 fission.
- [ ] AP-5 proptest pasa con 1000 sopas aleatorias.
- [ ] PV-7 (paper validation Kauffman RAF) added a `paper_validation` binary.
- [ ] `docs/sintesis_patron_vida_universo.md` § 10 referencia este track como su demostración.

Cuando esto pasa, el simulador deja de ser "una máquina de disipar" y pasa a ser "una máquina que disipa **copiándose**" — el salto que el documento describe.
