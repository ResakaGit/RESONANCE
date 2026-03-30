# Visualization Pipeline — UI/UX para Simulación Científica

> Diseño de interfaces para visualizar, controlar, y exportar simulaciones.
> Estándares: PhysiCell Studio, COPASI, Grafana científico, FDA/EMA reporting.
> Stack: Bevy 0.15 + bevy_egui 0.31 + egui_plot + egui_heatmap.

---

## Principios

```
1. La visualización NO es la simulación. Son capas ortogonales.
2. Cada panel es stateless: lee Resources/Components, renderiza, no muta física.
3. Controles → Resources → sistemas leen en el siguiente tick. Zero acoplamiento directo.
4. Export reproducible: mismo seed + params → mismos gráficos → mismos datos.
```

---

## Arquitectura: 4 capas

```
┌──────────────────────────────────────────────────────┐
│  Capa 4: Paneles de Dominio (cancer, ecology, etc.)  │
│  Composición de widgets genéricos para use cases.     │
├──────────────────────────────────────────────────────┤
│  Capa 3: Widgets Científicos                         │
│  TimeSeries, Heatmap, EntityInspector, VPC plot      │
│  (egui_plot, egui_heatmap, custom)                   │
├──────────────────────────────────────────────────────┤
│  Capa 2: Layout Framework                            │
│  Tabs, SidePanel, Drawer, SplitView                  │
│  (bevy_egui panels, state-driven visibility)         │
├──────────────────────────────────────────────────────┤
│  Capa 1: Data Bridge (read-only)                     │
│  SimulationSnapshot Resource, updated per tick        │
│  (stateless: systems write, panels read)             │
└──────────────────────────────────────────────────────┘
```

**Regla:** Capa N solo importa Capa N-1. Nunca lateral. Nunca hacia arriba.

---

## Stack Técnico

| Crate | Versión | Para qué |
|---|---|---|
| `bevy_egui` | 0.31-0.33 | egui dentro de Bevy (panels, widgets, input) |
| `egui` | 0.29.x | Immediate-mode UI (sliders, text, combobox) |
| `egui_plot` | latest compatible | Time-series, scatter, bar charts en tiempo real |
| `egui_heatmap` | 0.4.x (verificar compat egui 0.29) | Campos de energía/nutrientes como heatmap. Fallback: custom widget con `egui::Painter` si incompatible |

**No se necesitan:** Grafana (overhead de infra), Plotters (rendering pipeline separado), web frameworks.

---

## Capa 1: Data Bridge

### Resources (separados por responsabilidad)

```rust
/// Aggregados instantáneos de la simulación. Actualizado al final de cada tick.
/// Instant simulation aggregates. Updated at the end of each tick.
#[derive(Resource, Default)]
pub struct SimTickSummary {
    pub tick:           u64,
    pub total_qe:       f32,
    pub alive_count:    u32,
    pub species_count:  u8,
}

/// Historial de time series para gráficos. Ring buffer stack-allocated.
/// Time series history for charts. Stack-allocated ring buffer.
#[derive(Resource)]
pub struct SimTimeSeries {
    pub qe_history:      RingBuffer<f32>,
    pub pop_history:     RingBuffer<f32>,
    pub species_history: RingBuffer<f32>,
}

/// Ring buffer stack-allocated. Copy-friendly. Cache-friendly (contiguous memory).
pub struct RingBuffer<T: Copy + Default> {
    data: [T; 512],
    head: usize,
    len:  usize,
}
```

**Contrato:** `SimTickSummary` para status bar y panels puntuales. `SimTimeSeries` para gráficos temporales. Separados porque tienen frecuencias de lectura distintas (status bar = cada frame, gráficos = cada render del widget).

### Update system (Phase::MorphologicalLayer, end of tick)

```rust
/// Agrega datos de la simulación en Resources read-only para el UI.
/// Aggregates simulation data into read-only Resources for the UI.
fn update_sim_tick_summary(
    mut summary: ResMut<SimTickSummary>,
    mut series: ResMut<SimTimeSeries>,
    clock: Res<SimulationClock>,
    query: Query<&BaseEnergy, With<BaseEnergy>>,
) {
    summary.tick = clock.tick_id;
    let (mut total, mut alive) = (0.0_f32, 0_u32);
    for energy in &query {
        if energy.qe() > 0.0 { total += energy.qe(); alive += 1; }
    }
    summary.total_qe = total;
    summary.alive_count = alive;
    series.qe_history.push(total);
    series.pop_history.push(alive as f32);
}
```

**Contrato:** Los paneles de UI NUNCA ejecutan queries ECS. Solo leen `Res<SimTickSummary>` y `Res<SimTimeSeries>`.

---

## Capa 2: Layout Framework

### Estructura de pestañas (inspirada en PhysiCell Studio)

```
┌─────────────────────────────────────────────────────────────┐
│ [Simulation] [Parameters] [Analysis] [Export]               │
├────────────┬────────────────────────────────────────────────┤
│            │                                                │
│  Control   │          Viewport / Charts                     │
│  Panel     │                                                │
│  (left)    │  ┌──────────────────┐  ┌────────────────────┐ │
│            │  │  3D/2D Viewport  │  │  Time Series       │ │
│  ▸ Speed   │  │  (Bevy render)   │  │  (egui_plot)       │ │
│  ▸ Preset  │  │                  │  │                    │ │
│  ▸ Phase   │  └──────────────────┘  └────────────────────┘ │
│  ▸ Entity  │  ┌──────────────────┐  ┌────────────────────┐ │
│            │  │  Energy Heatmap  │  │  Population Chart  │ │
│            │  │  (egui_heatmap)  │  │  (egui_plot)       │ │
│            │  └──────────────────┘  └────────────────────┘ │
├────────────┴────────────────────────────────────────────────┤
│ Status: tick 1234 | 60 FPS | 42 entities | qe: 5420.3     │
└─────────────────────────────────────────────────────────────┘
```

### Tab enum (state-driven)

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
enum DashboardTab {
    #[default]
    Simulation,
    Parameters,
    Analysis,
    Export,
}
```

---

## Capa 3: Widgets Científicos

### Widget 1: Time Series Plot

```
Eje X: tick (ventana deslizante de 512 ticks)
Eje Y: valor (auto-scale)
Líneas: total_qe (azul), alive_count (verde), species (naranja)
Interacción: hover → tooltip con valor exacto. Scroll → zoom temporal.
```

**Implementación:** `egui_plot::Plot` con `Line` series desde `RingBuffer`.

### Widget 2: Energy Field Heatmap

```
Grilla: width × height del EnergyFieldGrid
Valor: accumulated_qe per cell
Color: gradiente frío→caliente (0 = azul oscuro, max = rojo)
Interacción: hover → cell coords + qe value. Click → inspect cell.
```

**Implementación:** `egui_heatmap` navigable. Datos desde `Res<EnergyFieldGrid>` (excepción documentada: es Resource read-only, O(1) access, copiar 16KB/tick al snapshot no justifica el costo).

### Widget 3: Entity Inspector

```
Selección: entity list en panel lateral (scrollable, filtrable por archetype)
  o Tab para ciclar entre entidades cercanas al player.
Display:
  - qe, radius, frequency_hz
  - matter_state, capabilities
  - InferenceProfile (growth, mobility, branching, resilience)
  - SenescenceProfile (age, max_age)
  - Trophic class
```

**Implementación:** `egui::SidePanel::right` con `egui::Grid`. Selección por lista (no raycast — evita acoplamiento con cámara/render). El sistema de inspector lee Entity via `SelectedEntity` Resource, no via click en viewport.

### Widget 4: Parameter Control Panel

```
Sección: Physics
  ├─ Simulation speed    [slider 0.1× - 10×]
  ├─ Tick rate           [slider 1-120 Hz]

Sección: Presets
  ├─ Universe            [dropdown: Earth, Jupiter, Mars, Eden, Hell]
  ├─ Load preset         [button]

Sección: View
  ├─ Show grid           [checkbox]
  ├─ Show trajectories   [checkbox]
  ├─ Color mode          [dropdown: frequency, qe, trophic, age]
  ├─ Camera mode         [dropdown: orbital, follow player, top-down]
```

**Patrón Grafana:** Cada control escribe a un `Resource`, nunca muta la simulación directamente.

```rust
/// Configuración de visualización. Escrita por UI, leída por render systems.
/// Visualization config. Written by UI, read by render systems.
#[derive(Resource)]
pub struct ViewConfig {
    pub show_grid:        bool,
    pub show_trajectories: bool,
    pub color_mode:       ColorMode,
    pub camera_mode:      CameraMode,
}

/// Configuración de velocidad. Escrita por UI, leída por pipeline clock.
/// Speed config. Written by UI, read by pipeline clock.
#[derive(Resource)]
pub struct SimSpeedConfig {
    pub time_scale: f32,  // 1.0 = normal, 0.1 = slow-mo, 10.0 = fast
    pub paused:     bool,
}
```

**Separación:** `ViewConfig` es rendering-only (no afecta simulación). `SimSpeedConfig` afecta `Time<Fixed>` (afecta simulación). Responsabilidades distintas → Resources distintos.

### Widget 5: Visual Predictive Check (FDA/EMA)

Para use case cancer:

```
Eje X: generación (tiempo)
Eje Y: conteo de células cancerosas
Bandas: percentil 5-95 de simulaciones (fill semi-transparente)
Línea central: mediana
Overlay: datos observados (puntos rojos)
```

**Implementación:** `egui_plot::PlotPoints` para scatter + `egui_plot::Polygon` para bandas.

---

## Capa 4: Paneles de Dominio

### Panel Cancer Therapy

```
┌───────────────────────────────────────────────────────────┐
│ CANCER THERAPY DASHBOARD                                  │
├────────────┬──────────────────────────────────────────────┤
│ Drug       │  ┌─────────────────────┐                    │
│ ▸ Potency  │  │ Cancer vs Normal    │  Time series:      │
│   [2.0]    │  │ ██████             │  cancer count       │
│ ▸ BW [50]  │  │ ████               │  normal count       │
│ ▸ Start    │  └─────────────────────┘  resistance index   │
│   [gen 5]  │  ┌─────────────────────┐                    │
│ ▸ Pause    │  │ Frequency Drift    │  Cancer freq vs     │
│   [0=cont] │  │                    │  drug target freq   │
│            │  └─────────────────────┘                    │
│ Biology    │  ┌─────────────────────┐                    │
│ ▸ Quiesc.  │  │ VPC: Tumor Volume  │  FDA-style:         │
│   [0.05]   │  │ (if ensemble)      │  5th-95th band      │
│ ▸ Immune   │  └─────────────────────┘  + observed overlay │
│   [5]      │                                              │
├────────────┴──────────────────────────────────────────────┤
│ Status: gen 42/100 | cancer: 8.3 | resist: 0.45 | drug ON│
└───────────────────────────────────────────────────────────┘
```

### Panel Ecology

```
┌───────────────────────────────────────────────────────────┐
│ ECOLOGY DASHBOARD                                         │
├────────────┬──────────────────────────────────────────────┤
│ Metrics    │  ┌─────────────────────┐                    │
│ ▸ Species  │  │ Population Dynamics │  By trophic class   │
│ ▸ Trophic  │  │ (stacked area)     │  producer/herb/carn │
│ ▸ Biomass  │  └─────────────────────┘                    │
│ ▸ Diversity│  ┌─────────────────────┐                    │
│            │  │ Spatial Distribution│  Entity positions    │
│ Field      │  │ (heatmap + scatter) │  on energy field    │
│ ▸ Nutrient │  └─────────────────────┘                    │
│ ▸ Energy   │  ┌─────────────────────┐                    │
│ ▸ Frequency│  │ Fitness Landscape   │  4D bias space      │
│            │  │ (growth vs mobility)│  colored by fitness  │
│            │  └─────────────────────┘                    │
└───────────────────────────────────────────────────────────┘
```

### Panel Fermi / Astrobiology

```
┌───────────────────────────────────────────────────────────┐
│ FERMI PARADOX DASHBOARD                                   │
├────────────┬──────────────────────────────────────────────┤
│ Parameters │  ┌─────────────────────┐                    │
│ ▸ Universes│  │ Life Probability    │  Bar: % with life   │
│   [1000]   │  │ by parameter range  │  % with complex     │
│ ▸ Gravity  │  └─────────────────────┘                    │
│   range    │  ┌─────────────────────┐                    │
│ ▸ Solar    │  │ Parameter Space     │  2D heatmap:        │
│   range    │  │ (gravity × solar)   │  color = species    │
│ ▸ Asteroid │  │ → species count     │  count at each      │
│   range    │  └─────────────────────┘  (gravity, solar)   │
└───────────────────────────────────────────────────────────┘
```

---

## Implementación: Sprints

| Sprint | Entregable | Costo | Deps |
|---|---|---|---|
| **VIS-1** | bevy_egui integration + SimulationSnapshot Resource + tab layout | M (200 LOC) | — |
| **VIS-2** | Time series widget (egui_plot) + population/qe/species charts | M (150 LOC) | VIS-1 |
| **VIS-3** | Energy field heatmap (egui_heatmap) + nutrient overlay | M (150 LOC) | VIS-1 |
| **VIS-4** | Parameter control panel (sliders, presets, view config) | M (200 LOC) | VIS-1 |
| **VIS-5** | Entity inspector (click → detail panel) | S (100 LOC) | VIS-1 |
| **VIS-6** | Cancer dashboard (domain panel composing VIS-2/3/4/5) | M (150 LOC) | VIS-2,3,4 |
| **VIS-7** | Ecology dashboard + Fermi dashboard | M (200 LOC) | VIS-2,3,4 |
| **VIS-8** | FDA/EMA VPC plot + export PDF/PNG | L (300 LOC) | VIS-2,6 |

**Total: ~1450 LOC, ~8 semanas (secuencial) o ~4 semanas (paralelo VIS-2/3/4).**

```
VIS-1 ──→ VIS-2 ──→ VIS-6 ──→ VIS-8
  │    ──→ VIS-3 ──→ VIS-7
  │    ──→ VIS-4
  └────→ VIS-5
```

---

## Estándares FDA/EMA implementados

| Requisito | Widget | Sprint |
|---|---|---|
| Concentration-time profiles (linear + semi-log) | Time series con toggle log scale | VIS-2 |
| Visual Predictive Check (5th-95th percentile bands) | VPC plot con polygon fill | VIS-8 |
| Predicted vs observed overlay | Scatter + line overlay | VIS-8 |
| Parameter sensitivity analysis | Tornado plot desde `ablate()` results | VIS-8 |
| Cmax/AUC comparison tables | egui::Grid con métricas tabuladas | VIS-6 |
| Dual-scale (linear + log) | Toggle button en time series widget | VIS-2 |
| Export data + figures | PNG screenshot + CSV (ya implementado) | VIS-8 |

---

## Cargo.toml additions

```toml
[dependencies]
bevy_egui = "0.31"
egui_plot = "0.29"
# egui_heatmap — evaluar si 0.4.x es compatible con egui 0.29
```

**Evaluación de riesgo:** bevy_egui 0.31 es la última compatible con Bevy 0.15. Si se upgradea Bevy a 0.16+, necesitará bevy_egui 0.34+. El código de paneles es independiente del motor — migración sería renaming de APIs, no reescritura.

---

## Patrón de comunicación UI ↔ Simulación

```
UI Panel System (Update schedule)
  │
  ├── ResMut<ViewConfig>         ← sliders/checkboxes de visualización
  ├── ResMut<SimSpeedConfig>     ← slider de velocidad, pause toggle
  ├── Res<SimTickSummary>        ← datos instantáneos (read-only)
  └── Res<SimTimeSeries>         ← historial para gráficos (read-only)

Simulation Systems (FixedUpdate schedule)
  │
  ├── Res<SimSpeedConfig>        ← lee velocidad para ajustar Time<Fixed>
  ├── ResMut<SimTickSummary>     ← escribe aggregados al final del tick
  └── ResMut<SimTimeSeries>      ← push to ring buffers al final del tick

Render Systems (Update schedule, after UI)
  │
  └── Res<ViewConfig>            ← lee config visual (grid, color mode, camera)
```

**Flujo unidireccional por Resource:**
- `ViewConfig`: UI escribe → Render lee (zero sim involvement)
- `SimSpeedConfig`: UI escribe → Sim lee (zero render involvement)
- `SimTickSummary` / `SimTimeSeries`: Sim escribe → UI lee (zero write from UI)

**Zero acoplamiento.** Cada Resource tiene exactamente un writer y N readers. No hay bidireccionalidad.
