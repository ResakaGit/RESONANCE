# Sprint AP-6: `autopoietic_lab` Binary — Visualización + CI

**ADR:** —
**Esfuerzo:** 1.5 semanas
**Bloqueado por:** AP-5
**Desbloquea:** Track cerrado

## Contexto

AP-5 prueba el invariante en CI. Falta el binario que un humano puede mirar para verificar visualmente: blob → fission → linaje → árbol genealógico. También un modo headless reproducible para benchmarks y validación regulatoria.

## Entregable

1. `cargo run --release --bin autopoietic_lab` — viz 2D
   - Heatmap multicapa: species concentration (R/G/B = 3 especies dominantes), membrane_strength (alpha overlay), closure boundary (contour line)
   - Línea de tiempo: K-stability per closure activa
   - Árbol de linaje: nodos por fission_event
   - Controles: pause/play, speed 0.25x–4x, seed selector, network selector (`raf_minimal.ron`, `random_seed_42.ron`, etc.)

2. `cargo run --release --bin autopoietic_lab -- --headless --soup random --seed S --ticks T --out report.json`
   - Sin GPU
   - Outputs: `report.json` (SoupReport de AP-5), `lineage.dot` (graphviz), `kstab.csv`

3. Asset library: 5 redes de reacciones predefinidas en `assets/reactions/`
   - `raf_minimal.ron` (3 reacciones, ciclo manual)
   - `formose.ron` (red formose autocatalítica clásica, Kauffman 1986)
   - `gard_minimal.ron` (GARD model, Lancet 2018)
   - `random_seed_{42,1337,2026}.ron` (sopas reproducibles)

## Tareas

| # | Tarea | Archivo | Tests |
|---|-------|---------|-------|
| 1 | `bin/autopoietic_lab.rs` esqueleto | `src/bin/autopoietic_lab.rs` | — |
| 2 | Viz 2D heatmap species + membrana | `src/viz/autopoietic_view.rs` | 2 |
| 3 | UI controls (egui) | `src/viz/autopoietic_ui.rs` | 1 |
| 4 | Lineage tree renderer | `src/viz/lineage_tree.rs` | 2 |
| 5 | Headless mode con CLI clap | `src/bin/autopoietic_lab.rs` | 2 integration |
| 6 | DOT exporter para árbol | `src/use_cases/experiments/autopoiesis/dot_export.rs` | 3 |
| 7 | Asset RONs | `assets/reactions/*.ron` | — |
| 8 | Update `src/bin/README.md` y track README | docs | — |

## Criterios de aceptación

- [ ] Binary corre en Win/Linux a ≥30 fps con grid 128×128 + red de 64 reacciones
- [ ] `--headless --ticks 100000 --seed 42` completa en < 30s
- [ ] `report.json` tiene schema validado por AP-5 `SoupReport`
- [ ] `formose.ron` produce ≥1 closure visible en < 5000 ticks
- [ ] Árbol de linaje renderiza correctamente para sopa con ≥3 fissions
- [ ] Sin uso de `unwrap`/`expect` en runtime
- [ ] Doc `docs/sprints/AUTOPOIESIS/README.md` actualizado: track marcado ✅
- [ ] Referencia agregada en `docs/sintesis_patron_vida_universo.md` § 10 → "demostración: `cargo run --bin autopoietic_lab`"

## Cierre del arco

Cuando este sprint cierra:

- El gap del 50% identificado en el análisis del documento queda resuelto.
- `paper_validation` puede agregar PV-7 (Hordijk-Steel RAF benchmark).
- El simulador puede afirmar, con prueba ejecutable: **"lo que persiste copió antes de disiparse"**.

---

## Estado real (split ejecutado) — 2026-04-14

El sprint original se dividió en sub-ítems durante la ejecución.  Ver
`docs/sprints/AUTOPOIESIS/README.md` para el mapa autoritativo:

- **AP-6a** ✅ — headless CLI stdlib + `SoupReport` JSON + `to_dot()` (commit `7cf76c4`)
- **AP-6b** ✅ — `--network <ron>` loader + `run_soup_with_network` (commit `9452ab3`)
- **AP-6b2** ✅ — formose + hypercycle canónicos con citas (commit `362da7f`).
  GARD diferido: no es mass-action (Segré-Lancet 2000 es compositional/statistical).
- **AP-6c** ❌ — Viz Bevy + egui.  Diseñado (ADR-042) pero sin código.

### Criterios de aceptación — mapping al estado real

| Criterio original | Estado | Notas |
|---|---|---|
| Binary ≥30 fps grid 128×128 + 64 reactions | ❌ bloqueado | Requiere AP-6c |
| `--headless --ticks 100000 --seed 42` < 30 s | ✅ parcial | Ejecuta subsegundo a 5k; 100k no cronometrado |
| `report.json` con schema `SoupReport` | ✅ | AP-6a |
| `formose.ron` ≥1 closure < 5000 ticks | ✅ | seed 42, grid 16×16, 5k ticks → `n_closures_final=1`, `dissipated=32.5` |
| Árbol de linaje ≥3 fissions | ❌ inalcanzable | **Ver Finding F-1** |
| Sin `unwrap`/`expect` en runtime | ✅ | AP-6a stdlib-only |
| `docs/sprints/AUTOPOIESIS/README.md` marcado ✅ | ⚠️ | Track cierra con AP-6c + Finding F-1 resuelto |
| Referencia en `sintesis_patron_vida_universo.md` §10 | ❌ | Pendiente |

### Entregables descartados vs. plan original

- `gard_minimal.ron` — descartado en AP-6b2 (GARD no es mass-action).
  Sustituido por `hypercycle.ron` (Eigen-Schuster 1977) como ciclo catalítico canónico.
- `random_seed_{42,1337,2026}.ron` — nunca materializados; el harness
  deriva la red desde `--seed` de forma determinística (`random_reaction_network`).

---

## Findings del sprint

### F-1 · Bug dimensional en `pressure_ratio` — RESUELTO 2026-04-14 (AP-6d)

**Síntoma original.** Sweep exhaustivo sobre formose+hypercycle × 576 combos
daba `max pressure_ratio ≈ 0.23`, umbral 50 inalcanzable (gap 200×).

**Causa raíz.** La fórmula `internal_production / cohesion_capacity` mezclaba
unidades: numerador `[qe · T⁻¹]`, denominador `[qe]`, ratio `[T⁻¹]` comparado
contra un umbral adimensional.  Además era timestep-dependiente (violación
sutil de Axioma 6 — el criterio no debe depender de la discretización).

**Fix.** Reescritura de `pressure_ratio` como `internal_production / decay_rate`,
ambos `[qe · T⁻¹]`, ratio adimensional.  Espeja `raf::kinetic_stability`
(mismo patrón Pross reconstruction/decay aplicado al blob) — no introduce
primitivas nuevas, reutiliza el patrón ya probado del codebase.

- `decay_rate(blob, grid, product_mask) = DISSIPATION_LIQUID × Σ productos_en_blob`
- `pressure_ratio = internal_production / decay_rate`
- Tests añadidos en `equations/fission.rs`:
  - `decay_rate_sums_masked_products_scaled_by_diffusion`
  - `pressure_ratio_is_size_invariant_for_uniform_blobs`
  - `pressure_ratio_is_dimensionless_and_crosses_fission_threshold_under_overdrive`
    (validación directa: overdrive → ratio > 50)

**Efecto medido.** `max_ratio` pasó de **0.23 → 13.3** en steady-state
formose spot-seeded (mismo input, nueva fórmula).  El criterio ahora es
físicamente alcanzable; ADR-039 actualizado con §Revisión 2026-04-14.

### F-1a · Gap kinético residual en sistema cerrado — deferred

**Síntoma.** Aún con fórmula corregida, formose+hypercycle en sopa cerrada
(food único seeded al t=0, sin replenishment) converge a K ≈ 13.  El umbral
50 exige un régimen 4× más overdriven del que la kinética del sustrato
sostiene sin input externo.  Gap 4× ≠ 200× — órdenes de magnitud distintos.

**Causa.** Kauffman RAF + Pross asumen food continuo o forzamiento
termodinámico externo.  Nuestro harness siembra food una vez; concentraciones
cruzan el régimen de overdrive sólo transientemente.

**Opciones follow-up** (AP-6e propuesto, fuera de scope AP-6):
- Food replenishment constante en el spot (nuevo `SoupConfig::food_flux: f32`)
- Kinetics canónicas más agresivas (revisar k values de `formose.ron` vs.
  Breslow 1959 rate constants literales)
- Threshold alternativo derivado (e.g. `DISSIPATION_GAS/DISSIPATION_LIQUID = 4`
  — transición gas→líquido en vez de plasma→sólido; requiere justificación
  física en ADR-039 §threshold)

**Implicación sprint.** Item 2/3 originales ("≥3 fissions", "≥1 fission en
100k ticks") siguen sin evidencia ejecutable.  El bug que los hacía
literalmente imposibles (F-1) está resuelto; queda una calibración
empírica que pertenece a un sprint de continuación.

### F-2 · AP-5 proptest falla en seed=9 (pre-existing)

Ejecutar `cargo test --release --test property_autopoiesis` actualmente
reporta `surviving_closures_satisfy_persistence_contract` FAILED
(`seed=9`: `k_mean=0.0002`, `pressure_events=0`).  Reproduce en main
sin modificaciones de este sprint — deuda heredada, no regresión.
Track README.md:49 marca AP-5 ✅; requiere actualización o fix del
contrato de persistencia.

---

## Qué quedó entregado en este item

1. Feature `SoupConfig::food_spot_radius: Option<usize>` — siembra
   centrada `(2r+1)×(2r+1)` para romper simetría traslacional.
2. Flags CLI `--spot R` y `--food-qe Q` en `autopoietic_lab`.
3. Test de regresión: `spot_seeded_formose_produces_nonzero_membrane_gradient`
   (uniform ⇒ `max_s = 0`, spot ⇒ `max_s > 0`).
4. Documentación de Findings F-1/F-2 (arriba).

**No entregado (bloqueado por F-1):** fixture con ≥3 fissions, criterio
"≥1 fission" random soup.
