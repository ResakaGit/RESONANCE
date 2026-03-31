# Track: LAB_UI_REFACTOR — Composición minimalista del Lab

**Objetivo:** Refactorizar el lab para que cada experimento defina su propio panel de control, vista central, acción de run, y export. El lab solo compone. Zero lógica condicional dispersa.

**Estado:** ACTIVO
**Bloqueado por:** Nada
**Desbloquea:** Demos profesionales, capturas limpias, UI usable por no-programadores

---

## Problema actual

El lab tiene lógica condicional dispersa:
- `is_live` check en 3 lugares distintos
- `render_shared_params` muestra Worlds/Gens/Ticks para Live 2D donde no aplican
- `render_experiment_params` solo maneja CancerTherapy, el resto dice "No additional parameters"
- Run/Export visible cuando no debería
- Live 2D sin controles (pause, speed, reset, map selector)
- No hay máquina de estados — flags booleanos sueltos

## Solución: Experiment como composición de 4 funciones

```rust
/// Cada experimento define qué se muestra y qué se ejecuta.
/// Stateless: funciones puras que reciben &mut Ui + &Resources.
struct ExperimentDef {
    name:           &'static str,
    render_controls: fn(&mut egui::Ui, &mut LabParams, &mut CancerParams),
    render_central:  fn(&mut egui::Ui, &LabState, ...),
    run:             fn(&LabParams, &CancerParams) -> LabResult,
    to_csv:          fn(&LabResult) -> String,
}
```

El lab solo hace:
```rust
let def = experiment_def(params.experiment);
(def.render_controls)(ui, params, cancer);
(def.render_central)(ui, state);
```

Zero `if is_live`. Zero `match params.experiment` dispersos.

---

## Sprints (4)

| Sprint | Descripción | Costo | Deps |
|--------|-------------|-------|------|
| **LR-1** | LabMode state machine (Batch/Live) + contextual panel visibility | S | — |
| **LR-2** | Live 2D controls (pause, speed, reset, map selector, view layer) | M | LR-1 |
| **LR-3** | Per-experiment control panel (contextual params, no shared junk) | M | LR-1 |
| **LR-4** | ExperimentDef composition pattern (trait-like dispatch via enum) | M | LR-3 |

---

## LR-1: LabMode state machine

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum LabMode {
    /// Batch experiment: configure → run → see results → export
    Batch { experiment: BatchExperiment, run_mode: RunMode },
    /// Live simulation: real-time Bevy runtime visualization
    Live { paused: bool },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum BatchExperiment {
    Lab, Fermi, Speciation, Cambrian, Debate, Convergence, CancerTherapy,
}
```

Transiciones válidas:
```
Batch(any) ←→ Live    (switch experiment selector)
Batch(A) → Batch(B)   (switch experiment within batch)
Live.paused ←→ Live.running  (pause/resume button)
```

Transiciones inválidas (compilador previene):
- Live + RunMode (Ablation/Ensemble no aplican a live)
- Batch + pause/speed (no aplican a batch)

El panel izquierdo muestra controles según `LabMode`:
- `Batch` → Preset, Seed, Worlds, Gens, Ticks, RunMode, Run, Export
- `Live` → Map, Pause/Resume, Speed, Reset, View Layer

---

## LR-2: Live 2D controls

Panel izquierdo cuando Live está activo:

```
Simulation (Live)
  Map: [dropdown: genesis_validation, earth, demo_animal, ...]
  [Pause] / [Resume]
  Speed: [slider 0.5× — 5×]
  [Reset World]

View Layer
  ○ Energy (qe heatmap)
  ○ Frequency (dominant Hz → hue)
  ○ Nutrient (nutrient field overlay)
  ○ Temperature (matter state boundaries)

Status
  Tick: 1234
  Alive: 716
  Total qe: 52340
```

Cada view layer pinta el mismo grid con datos distintos de `EnergyCell`:
- Energy: `cell.accumulated_qe` → brightness
- Frequency: `cell.dominant_frequency_hz` → hue
- Nutrient: `NutrientFieldGrid.cell_xy()` → green intensity
- Temperature: `cell.temperature` → blue(cold) to red(hot)

---

## LR-3: Per-experiment control panel

Cada experiment muestra SOLO los parámetros que le aplican:

| Experiment | Panel de control |
|---|---|
| Universe Lab | Preset, Seed, Worlds, Gens, Ticks |
| Fermi | Universes (=Worlds), Gens, Ticks |
| Speciation | Preset, Seed A, Seed B, Gens, Ticks |
| Cambrian | Preset, Seed, Worlds, Gens, Ticks, Explosion threshold |
| Debate | Preset, Seeds (=Worlds), Gens, Ticks |
| Convergence | Preset, Seeds, Gens, Ticks, Convergence threshold |
| Cancer Therapy | Potency, Bandwidth, Start gen, Worlds, Gens, Ticks |
| Live 2D | Map, Speed, View layer (no batch params) |

---

## LR-4: ExperimentDef pattern

```rust
fn controls_for(mode: &LabMode) -> fn(&mut egui::Ui, &mut AllParams) {
    match mode {
        LabMode::Batch { experiment, .. } => match experiment {
            Lab => render_lab_controls,
            Fermi => render_fermi_controls,
            CancerTherapy => render_cancer_controls,
            // ...
        },
        LabMode::Live { .. } => render_live_controls,
    }
}

fn central_for(mode: &LabMode) -> fn(&mut egui::Ui, &LabState, ...) {
    match mode {
        LabMode::Batch { .. } => render_batch_results,
        LabMode::Live { .. } => render_live_2d,
    }
}
```

El lab main loop:
```rust
let controls_fn = controls_for(&mode);
let central_fn = central_for(&mode);

SidePanel::left(|ui| controls_fn(ui, &mut params));
CentralPanel(|ui| central_fn(ui, &state));
```

Zero `if is_live`. Zero flags dispersos. Cada nuevo experiment se añade como un variant + 2 funciones. El lab no cambia.

---

## Visualización correcta por experimento (profesional)

| Experiment | Panel central profesional |
|---|---|
| **Universe Lab** | Fitness curve (best+mean) + top genomes table + diversity sparkline |
| **Fermi** | Bar chart: % universes con N species. Tabla: total, life, complex. Histogram de species distribution. |
| **Speciation** | Frequency divergence over time (2 líneas que se separan). Cross-interference decay. YES/NO badge. |
| **Cambrian** | Diversity curve + species curve overlay. Vertical line en explosion gen. Before/after delta highlight. |
| **Debate** | 3 gauges: life rate, complexity rate, cooperation signal. Per-seed scatter plot. |
| **Convergence** | Genome 4D bias scatter (growth×mobility, colored by fitness). Cluster visualization. Distance matrix heatmap. |
| **Cancer** | Cancer(red)+Normal(green) population over time. Resistance index over time. Drug ON/OFF shading. Summary badges: Eliminated, Resistance gen, Relapse gen. |
| **Live 2D** | Full-panel heatmap del campo. Layer toggle. Entity overlay. Status bar con métricas en vivo. |

---

## Orden de implementación

```
LR-1 (state machine) → LR-3 (contextual panels) → LR-4 (composition pattern) → LR-2 (live controls)
```

LR-1 primero porque elimina todos los `is_live` flags y establece la base.
LR-3 antes de LR-4 porque necesita definir qué controles existen para cada experiment.
LR-2 último porque es el más visual y depende de que el framework de composición esté listo.
