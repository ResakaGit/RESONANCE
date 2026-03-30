# Resonance — Emergent Life Simulation Engine

Energy-first simulation where life, behavior, and ecosystems emerge from **8 axioms** and **4 fundamental constants**. Built with **Rust** and **Bevy 0.15 ECS**.

## What It Does

Define the laws of physics. Press play. Watch life emerge.

- **253 entities** materialize from energy fields — no templates, no scripting
- **17 behavioral agents** gain agency when coherence exceeds dissipation
- **Multicelular clusters** form via axiomatic cell division + structural bonds
- **Day/night rotation**, seasonal tilt, water cycle — all axiom-derived
- **Zero hardcoded behavior** — everything is energy (`qe`), frequency (`Hz`), and interference

## The 8 Axioms

1. **Everything is Energy** — All entities are qe. No HP/mana/stats.
2. **Pool Invariant** — `Σ energy(children) ≤ energy(parent)`.
3. **Competition** — Derived from oscillatory interference.
4. **Dissipation (2nd Law)** — All processes lose energy.
5. **Conservation** — Energy never created, only transferred/dissipated.
6. **Emergence at Scale** — No top-down programming.
7. **Distance Attenuation** — Interaction decays with distance.
8. **Oscillatory Nature** — Every concentration oscillates at frequency f.

## The 4 Fundamental Constants

| Constant | Value | Source |
|----------|-------|--------|
| `KLEIBER_EXPONENT` | 0.75 | Biological universal (metabolic scaling) |
| `DISSIPATION_{SOLID→PLASMA}` | 0.005 → 0.25 | Second Law (empirical ratios 1:4:16:50) |
| `COHERENCE_BANDWIDTH` | 50.0 Hz | Frequency observation window |
| `DENSITY_SCALE` | 20.0 | Spatial normalization |

All ~40 lifecycle constants are **algebraically derived** from these 4 via `blueprint/equations/derived_thresholds.rs`.

## Quick Start

```bash
# Default demo
cargo run

# Earth simulation (48×48, day/night, infinite sun)
RESONANCE_MAP=earth cargo run --release

# Planetary simulation (128×128, seasons, water cycle)
RESONANCE_MAP=earth_128 cargo run --release

# Real-time viewer (pixel window)
RESONANCE_MAP=earth cargo run --release --features pixel_viewer --bin sim_viewer -- --render window --scale 10

# Terminal viewer (no GPU)
RESONANCE_MAP=earth cargo run --release --bin sim_viewer -- --render terminal

# Headless (sim → PPM image)
cargo run --bin headless_sim -- --ticks 5000 --scale 8 --out world.ppm
```

## Architecture

```
src/
├── blueprint/       Pure math: equations/, constants/, derived_thresholds.rs
│   └── equations/   45+ domain files — ALL simulation math lives here
├── layers/          14 ECS layers (L0 BaseEnergy → L13 StructuralLink)
├── simulation/      FixedUpdate pipeline (6 phases), abiogenesis, lifecycle, metabolic, emergence
├── worldgen/        Field grid, nucleus propagation, materialization, day/night, water cycle
├── entities/        Component group factories, archetypes
├── batch/           Headless batch simulator (millions of worlds, genetic evolution)
├── viewer/          Terminal + pixel window real-time visualization
├── plugins/         SimulationPlugin + 6 domain plugins
├── eco/             Eco-boundaries, climate
├── topology/        Procedural terrain
└── rendering/       Quantized color engine
```

## Planetary Simulation

```ron
// assets/maps/earth_128.ron
(
  width_cells: 128,
  height_cells: 128,
  day_period_ticks: Some(1200.0),
  year_period_ticks: Some(24000.0),
  axial_tilt: Some(0.26),
  self_sustaining_qe: Some(10.0),
  emission_scale: Some(7.0),
  // ...
)
```

Features:
- **Toroidal topology** — grid wraps in both axes (planetary surface)
- **Day/night** — solar meridian sweeps X axis, cosine falloff
- **Seasons** — axial tilt oscillates sub-solar latitude over year period
- **Water cycle** — evaporation from hot cells, precipitation on cold
- **Emission scaling** — nucleus power scales with grid area
- **Injectable anchor** — `self_sustaining_qe` tunes the threshold of life per map

## Energy Cycle (Closed Loop)

```
Nucleus (finite/infinite) → emits to field → diffusion + radiation pressure
    ↓                                                    ↓
Reservoir depletes                          Entities materialize
    ↓                                                    ↓
Zone cools                                 Live (Kleiber drain) → die (Gompertz)
    ↓                                                    ↓
                    Nutrients return to grid (conservation)
                                 ↓
                    Threshold → nucleus recycling → new nucleus
                                 ↓
                           Cycle restarts
```

## Emergence Pipeline

```
Energy field accumulates → coherence > dissipation → abiogenesis (entity spawns)
    → awakening (BehavioralAgent when potential > 1/3)
    → axiomatic cell division (valley ≤ 0 in internal field)
    → StructuralLink between children (multicelularity)
    → specialization (InferenceProfile diverges by energy fraction)
    → pack formation + cooperative hunting (√N bonus)
    → cultural transmission + coalition stability
```

No step is programmed. Each emerges from the previous via axiom-derived thresholds.

## Simulation Stack (levels 0→10, all emergent)

```
Energy (qe) → matter states → metabolism → reproduction → variable genome (4→32 genes)
→ codon-based genetic code (64→8 amino, evolucionable) → proto-proteins (lattice fold)
→ metabolic networks (DAG 12 nodes, competition, Hebb, catalysis)
→ multicellularity (adhesion, colonies, differential expression)
→ epigenetics (environment silences genes) → 16 tiers social emergence
→ bilateral morphology (128 nodes, appendages)
```

**Nothing programmed. Everything emerged from 8 axioms and 4 constants.**

## Use Cases (13/16 implemented)

| ID | Use Case | Command |
|----|----------|---------|
| A1 | Versus Arena | `cargo run --bin versus` |
| A2 | Universe Lab | `cargo run --bin universe_lab -- --preset jupiter` |
| B1 | Fermi Paradox | `cargo run --bin fermi` |
| B2 | Allopatric Speciation | `cargo run --bin speciation` |
| B3 | Cambrian Explosion | `cargo run --bin cambrian` |
| B4 | Debate Settler | `cargo run --bin debate` |
| C1 | Fossil Record | `cargo run --bin fossil_record` |
| C2 | Petri Dish | `cargo run --bin petri_dish` |
| C3 | Museum Mode | `cargo run --bin museum` |
| C4 | Mesh Export (OBJ) | `cargo run --bin mesh_export` |
| D1 | Personal Universe | `cargo run --bin personal_universe -- "your name"` |
| D2 | Convergent Evolution | `cargo run --bin convergence` |
| D3 | Ecosystem Music | `cargo run --bin ecosystem_music` |

**Additional binaries:**

```bash
cargo run --bin evolve              # Headless evolution with gene/protein/colony observability
cargo run --bin evolve_and_view     # Evolution + 3D GF1 visualization
cargo run --bin headless_sim        # Full sim → PPM image (no GPU)
cargo run --bin sim_viewer          # Real-time viewer (terminal or pixel window)
```

## Tests

```bash
cargo test    # 2,834 tests (87K+ LOC)
cargo bench   # batch + bridge benchmarks
```

## Docs

- **Design specs:** [docs/design/INDEX.md](./docs/design/INDEX.md)
- **Module contracts:** [docs/arquitectura/](./docs/arquitectura/)
- **Sprint backlog:** [docs/sprints/](./docs/sprints/)
- **Planetary simulation:** [docs/design/PLANETARY_SIMULATION.md](./docs/design/PLANETARY_SIMULATION.md)

## Requirements

- Rust 1.85+ (edition 2024)
- macOS / Linux / Windows
- Optional: `--features pixel_viewer` for real-time window (uses `minifb`)

## License

Private — All rights reserved.
