# Resonance — Emergent Life Simulation Engine

Simulation engine where life, evolution, and therapeutic strategies emerge from **8 axioms** and **4 fundamental constants**. Built with **Rust** and **Bevy 0.15 ECS**. Open source (AGPL-3.0).

Paper: https://zenodo.org/records/19342036 | Repo: https://github.com/ResakaGit/RESONANCE

## What It Does

Define the laws of physics. Press play. Watch life emerge. Design therapeutic strategies from first principles.

- **10-level biological hierarchy** — from energy fields to social emergence, all emergent
- **Pathway-level drug design** — inhibit specific metabolic pathways without killing cells
- **Adaptive therapy controller** — profiles tumor, selects frequency + dose, stabilizes growth at zero
- **Bozic 2013 validated** — combination > monotherapy, confirmed 10/10 independent seeds
- **Clinically calibrated** — output in nM, days, cell count (3 tumor profiles from published data)
- **3,095 automated tests** — deterministic, bit-exact reproducible

## What It Is NOT

- **Not a clinical tool** — not validated against patient outcomes
- **Not a drug discovery pipeline** — does not design molecules
- **Not a substitute for oncology** — a simulator for exploring therapeutic strategies

**Clinically calibrated** — simulation output maps to real units (nM, days, cell count) via published data. Three tumor profiles: CML/imatinib (Bozic 2013), prostate/abiraterone (Gatenby 2009), NSCLC/erlotinib. Example: "399 Hz @ 0.40, gen 3" → "Imatinib 104 nM, start day 12, doubling time 4 → 7.5 days."

## The 8 Axioms

| # | Axiom | Type |
|---|-------|------|
| 1 | Everything is Energy — all entities are qe | Primitive |
| 2 | Pool Invariant — Σ children ≤ parent | Primitive |
| 3 | Competition — derived from oscillatory interference | Derived |
| 4 | Dissipation (2nd Law) — all processes lose energy | Primitive |
| 5 | Conservation — energy never created | Derived |
| 6 | Emergence at Scale — no top-down programming | Meta |
| 7 | Distance Attenuation — interaction decays with distance | Primitive |
| 8 | Oscillatory Nature — every entity oscillates at frequency f | Primitive |

## The 4 Fundamental Constants

| Constant | Value | Source |
|----------|-------|--------|
| `KLEIBER_EXPONENT` | 0.75 | Biological universal (metabolic scaling) |
| `DISSIPATION_{SOLID→PLASMA}` | 0.005 → 0.25 | Second Law (ratios 1:4:16:50, physically motivated) |
| `COHERENCE_BANDWIDTH` | 50.0 Hz | Frequency observation window |
| `DENSITY_SCALE` | 20.0 | Spatial normalization |

All ~40 lifecycle constants are **algebraically derived** from these 4 via `derived_thresholds.rs`. Zero hardcoded values.

## Validated Results

### Experiment 4: Pathway Inhibition

Drug reduces metabolic efficiency **without killing cells**. Dose-response validated across 10 independent seeds.

| Concentration | Efficiency | Suppression |
|---------------|-----------|-------------|
| 0.0 (control) | 1.000 | 0% |
| 0.4 | 0.488 | 51.2% |
| 0.8 | 0.471 | 52.9% |

### Experiment 5: Bozic 2013 Validation

Combination therapy advantage **confirmed** (10/10 seeds, p < 0.001):

| Arm | Efficiency | Suppression | Prediction |
|-----|-----------|-------------|------------|
| no_drug | 1.000 | 0% | baseline |
| mono_A (400 Hz) | 0.481 | 51.9% | resistance inevitable ✓ |
| **combo_AB** | **0.435** | **56.5%** | **combo > mono ✓** |
| double_A (2×) | 0.466 | 53.4% | **combo > double ✓** |

### Experiment 6: Adaptive Therapy Controller

Feedback loop stabilizes tumor growth at **zero net expansion**:

```
Gen 0-2:  No treatment. Efficiency 1.000.
Gen 3:    Controller starts: 399 Hz @ 0.40.
Gen 4:    Efficiency drops to 0.575 (42.5% suppression).
Gen 5+:   STABLE. Growth rate = 0.000. Tumor controlled.
Protocol: "399 Hz @ 0.40, maintain from gen 3"
```

Validated across 10 seeds: stabilizes in ≥7/10, suppresses in ≥7/10.

### Calibrated Output (CML/imatinib)

| Simulation | Clinical (Bozic 2013) |
|-----------|----------------------|
| Gen 3, 399 Hz @ 0.40 | Day 12, imatinib 104 nM |
| Efficiency 0.536 | Doubling time 4 → 7.5 days |
| 128 entities | ~10⁹ cells |
| Combo A+B @ 0.8 | 208 nM + 208 nM, day 20 |

### Experiment 7: Rosie Case (Canine Mast Cell Tumor)

Simulation of a real-world case: personalized mRNA cancer vaccine for a dog (press reports, March 2026). Tumor profile: 70% KIT+ responsive, 30% KIT- resistant. Calibrated with published veterinary oncology data (London 2003, London 2009).

| Observed (real) | Predicted (simulation) | Match |
|----------------|----------------------|-------|
| Mono vaccine → 75% tumor reduction | Mono → efficiency drops significantly | ✓ |
| Some tumors didn't respond | Resistant fraction persists (eff > 0.05) | ✓ |
| Surgery needed after vaccine | Mono insufficient to eliminate | ✓ |
| Second target not tried | **Combo (KIT+ & KIT-) suppresses more than mono** | Prediction |

Validated across 5 seeds. Partial response is structural, not stochastic.

**Calibrated (canine mast cell):** 21-day doubling, toceranib IC50 = 40 nM proxy, ~10⁸ cells.

*DISCLAIMER: Simulated from press reports. NOT peer-reviewed data. NOT veterinary advice.*

### Clinical Calibration Profiles

| Tumor | Drug | Doubling | IC50 | Source |
|-------|------|---------|------|--------|
| CML | Imatinib | 4 days | 260 nM | Bozic 2013 |
| Prostate | Abiraterone | 30 days | 5.1 nM | Gatenby 2009 |
| NSCLC | Erlotinib | 7 days | 20 nM | EGFR mutant |
| Canine mast cell | Toceranib (proxy) | 21 days | 40 nM | London 2009 |

### Scientific Validation Summary

| Criterion | Status |
|-----------|--------|
| Reproducibility | ✓ Bit-exact determinism, any machine |
| Controls | ✓ No-drug baseline + fixed-dose comparison |
| Multi-seed | ✓ 10 seeds per experiment (Exp 4, 5, 6) |
| Falsifiability | ✓ All BDD tests could have failed |
| Pre-registration | ✓ Assertions written before execution |
| Dose-response monotonicity | ✓ 5/5 seeds |
| Against published prediction | ✓ Bozic 2013 combo advantage confirmed |
| Clinical calibration | ✓ 3 profiles (CML, prostate, NSCLC) from published IC50 + doubling times |
| Against patient outcomes | **Not yet** — calibrated but not validated against longitudinal patient data |

## Biological Hierarchy (10 levels, all emergent)

| Level | Phenomenon | Mechanism | Tests |
|-------|-----------|-----------|-------|
| 0 | Energy fields | Nucleus emission + diffusion | 17 |
| 1 | Matter states | Density thresholds (derived) | 17 |
| 2 | Molecular bonding | Coulomb + Lennard-Jones | 26 |
| 3 | Entities (life) | Abiogenesis: coherence > dissipation | 42 |
| 4 | Variable genome | 4-32 genes, Schwefel mutation | 62 |
| 5 | Genetic code | 64 codons → 8 amino acids | 28 |
| 6 | Proto-proteins | HP lattice fold, catalytic function | 27 |
| 7 | Metabolic networks | DAG, competitive flow, Hebb | 68 |
| 8 | Multicellularity | Union-Find, differential expression | 33 |
| 9 | Social emergence | Theory of mind, coalitions, culture | 40+ |

## Quick Start

```bash
cargo run                                          # Default demo
cargo run --release --bin adaptive_therapy         # Adaptive therapy controller (~8 sec)
cargo run --release --bin bozic_validation          # Bozic 2013 5-arm validation (~95 sec)
cargo run --release --bin pathway_inhibitor         # Pathway inhibition (~6 sec)
cargo run --release --bin cancer_therapy            # Level 1 cytotoxic
cargo run --release --bin lab                       # Universal dashboard
cargo run --release --bin survival -- --seed 42     # Play as evolved creature
RESONANCE_MAP=earth cargo run --release             # Earth simulation
```

## Tests

```bash
cargo test --release    # 3,095 tests (110K LOC, ~34 sec)
cargo bench             # batch + bridge benchmarks
```

## Architecture

```
src/
├── blueprint/equations/    Pure math (50+ files, 0 side effects)
│   ├── pathway_inhibitor.rs   Drug design: 14 fns, 42 tests
│   ├── derived_thresholds.rs  4 constants → ~40 thresholds
│   └── ...                    protein_fold, metabolic_genome, coulomb, etc.
├── batch/                  Headless simulator (NO Bevy, rayon parallel)
├── layers/                 14 ECS layers
├── simulation/             9 active emergence systems, 7 implemented not registered
├── use_cases/experiments/  6 validated experiments
└── bin/                    25 executables
```

## Docs

- **Architecture (canonical):** [docs/ARCHITECTURE.md](./docs/ARCHITECTURE.md)
- **Paper:** [docs/paper/](./docs/paper/) — arXiv source (6 experiments, 12 references)
- **Design specs:** [docs/design/](./docs/design/) — code-referenced historical specs

## Requirements

- Rust 1.85+ (edition 2024)
- macOS / Linux / Windows
- No GPU required

## License

AGPL-3.0 — Free to use, study, modify, and distribute. See [LICENSE](./LICENSE).
