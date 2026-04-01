# Resonance — Emergent Life Simulation Engine

Simulation engine where life, evolution, ecology, and drug resistance emerge from **8 thermodynamic axioms** and **4 fundamental constants**. Built with **Rust** and **Bevy 0.15 ECS**. Open source (AGPL-3.0).

Paper: https://zenodo.org/records/19342036

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
| `DISSIPATION_{SOLID→PLASMA}` | 0.005 → 0.25 | Second Law (ratios 1:4:16:50, physically motivated) |
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

**9 emergence systems active in runtime:** entrainment (Kuramoto), theory of mind, cultural transmission, infrastructure, cooperation (Nash), symbiosis, niche adaptation (Hutchinson), epigenetic adaptation, infrastructure intake. 7 additional systems are implemented but not yet wired into the schedule (coalitions, institutions, tectonics, multiscale, geological LOD).

## Biological Hierarchy (10 levels, all emergent)

| Level | Phenomenon | Mechanism | Reference | Tests |
|-------|-----------|-----------|-----------|-------|
| 0 | Energy fields | Nucleus emission + diffusion | Axioms 1, 4, 7 | 17 |
| 1 | Matter states | Density thresholds (derived from 4 constants) | Axiom 4 | 17 |
| 2 | Molecular bonding | Coulomb + Lennard-Jones + frequency alignment | Coulomb 1785, LJ 1924 | 26 |
| 3 | Entities (life) | Abiogenesis: coherence > dissipation | Axioms 4, 7, 8 | 42 |
| 4 | Variable genome | 4-32 genes, Schwefel self-adaptive mutation | Schwefel 1981 | 62 |
| 5 | Genetic code | 64 codons → 8 amino acids, evolvable table | Dill 1985 | 28 |
| 6 | Proto-proteins | HP lattice fold, catalytic function inference | Dill 1985 | 27 |
| 7 | Metabolic networks | DAG (12 nodes), competitive flow, Hebbian rewiring | Systems biology | 68 |
| 8 | Multicellularity | Cell adhesion, Union-Find colonies, differential expression | Dev biology | 33 |
| 9 | Social emergence | Theory of mind, coalitions, cultural transmission | Game theory | 40+ |

### Molecular Physics (Level 2)

Particles interact via classical potentials derived from the 4 fundamental constants:

- **Coulomb:** `F = k_C * q1 * q2 / (r^2 + eps^2)` where `k_C = 1/DENSITY_SCALE = 0.05`
- **Lennard-Jones:** `V(r) = 4*eps_LJ * [(sigma/r)^12 - (sigma/r)^6]` where `sigma = 1/DENSITY_SCALE`, `eps_LJ = DISSIPATION_SOLID * 100`
- **Bond detection:** pair is stable when `|E_bond| > threshold` (negative energy = bound state)
- **Frequency modulation:** bond strength scaled by `exp(-df^2 / (2*B^2))` (Axiom 8)

No bond tables, no molecule templates. Opposite charges attract, reach equilibrium, and form stable bonds. 26 tests verify inverse-square law, LJ zero-crossing, Newton 3, charge conservation, and deterministic reproducibility.

### Drug Models (Two Levels + Bozic Validation)

**Level 1 — Cytotoxic (cancer_therapy):** Drug drains qe directly (kills cells). Frequency-selective via Axiom 8 + Hill pharmacokinetics (n=2). Quiescent stem cells escape chemo, reactivate on tumor regression.

**Level 2 — Pathway Inhibitor (pathway_inhibitor):** Drug binds to protein active site, reduces metabolic node efficiency without killing. Three modes: Competitive / Noncompetitive / Uncompetitive. Off-target effects via frequency proximity. Bliss independence for drug combinations. Reproduction + death enabled — population evolves under drug pressure.

**Bozic 2013 Validation — combination therapy advantage CONFIRMED:**

```
cargo run --release --bin bozic_validation    # 5-arm experiment, ~95 sec
```

| Arm | Efficiency | Suppression | Bozic prediction |
|-----|-----------|-------------|------------------|
| no_drug | 1.000 | 0% | baseline |
| mono_A (400 Hz) | 0.481 | 51.9% | resistance inevitable |
| mono_B (300 Hz) | 0.635 | 36.5% | — |
| **combo_AB** | **0.435** | **56.5%** | **combo > mono** ✓ |
| double_A (2× dose) | 0.466 | 53.4% | **combo > double** ✓ |

Two drugs at different frequencies suppress MORE than one drug at double dose. Validated across **10 independent seeds** (10/10 confirm, p < 0.001). Reproduces the exponential advantage of combination therapy predicted by Bozic et al. 2013 (eLife) — derived from 4 constants, no molecular mechanisms, no mutation rates.

```bash
cargo run --release --bin cancer_therapy       # Level 1: cytotoxic
cargo run --release --bin pathway_inhibitor    # Level 2: pathway inhibition, ~6 sec
cargo run --release --bin bozic_validation     # Bozic 2013 validation, ~95 sec
```

**Honest limitations:** abstract qe units (not molar concentrations), no molecular targets (no EGFR/BCR-ABL), no tumor microenvironment (no vasculature/hypoxia/immune), not validated against patient-level data. Theoretical models for exploring resistance dynamics, not clinical tools.

**Nothing programmed. Everything emerged from 8 axioms and 4 constants.**

## Use Cases (15 experiments + 22 binaries)

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
| E1 | Cancer Therapy (Level 1) | `cargo run --bin cancer_therapy` |
| E2 | Pathway Inhibitor (Level 2) | `cargo run --release --bin pathway_inhibitor` |
| E3 | **Bozic Validation (combo vs mono)** | **`cargo run --release --bin bozic_validation`** |
| E4 | Particle Lab | `cargo run --bin particle_lab` |
| **LAB** | **Universal Lab (all experiments + Live 2D)** | **`cargo run --release --bin lab`** |
| **SV** | **Survival Mode (play as evolved creature)** | **`cargo run --release --bin survival -- --seed 42`** |

**Additional binaries:**

```bash
cargo run --release --bin lab                # Universal lab: 8 experiments + Live 2D + ablation + CSV export
cargo run --release --bin survival           # Survival: WASD, play as evolved creature
cargo run --release --bin evolve             # Headless evolution with gene/protein/colony observability
cargo run --release --bin evolve_and_view    # Evolution + 3D GF1 visualization
cargo run --release --bin cancer_therapy -- --out tumor.csv   # Cancer therapy → CSV
cargo run --release --bin fermi -- --out fermi.csv            # Fermi paradox → CSV
cargo run --release --bin headless_sim -- --ticks 5000 --out world.ppm
```

## Tests

```bash
cargo test    # 3,051 tests (110K LOC)
cargo bench   # batch + bridge benchmarks
```

## Docs

- **Architecture (canonical):** [docs/ARCHITECTURE.md](./docs/ARCHITECTURE.md) — axioms, constants, module map, drug pipeline, Bozic validation, limitations
- **Design specs (historical):** [docs/design/INDEX.md](./docs/design/INDEX.md)
- **Module contracts:** [docs/arquitectura/](./docs/arquitectura/)
- **Paper:** [docs/paper/](./docs/paper/) — arXiv source + references

## Requirements

- Rust 1.85+ (edition 2024)
- macOS / Linux / Windows
- Optional: `--features pixel_viewer` for real-time window (uses `minifb`)

## License

AGPL-3.0 — Free to use, study, modify, and distribute. Companies using this in production must open-source their code. See [LICENSE](./LICENSE).
