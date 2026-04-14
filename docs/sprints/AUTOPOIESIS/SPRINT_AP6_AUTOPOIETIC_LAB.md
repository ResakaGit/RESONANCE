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
