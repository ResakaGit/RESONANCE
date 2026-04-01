# Resonance — Architecture

> 110K LOC · 3,066 tests · Rust 2024 / Bevy 0.15 · AGPL-3.0
>
> Paper: https://zenodo.org/records/19342036

## Overview

Resonance is an emergent life simulation engine where everything derives from **8 axioms** and **4 fundamental constants**. No behavior is programmed — life, evolution, drug resistance, and therapy strategies emerge from energy interactions.

```
                    ┌─────────────────────────┐
                    │   4 Fundamental Constants │
                    │ Kleiber · Dissipation ×4 │
                    │ Bandwidth · Density Scale │
                    └────────────┬────────────┘
                                 │ derives ~40 thresholds
                    ┌────────────▼────────────┐
                    │    8 Axioms (physics)     │
                    │ Energy · Pool · Dissip.   │
                    │ Distance · Oscillatory    │
                    └────────────┬────────────┘
                                 │
          ┌──────────┬───────────┼───────────┬──────────┐
          ▼          ▼           ▼           ▼          ▼
    ┌──────────┐ ┌────────┐ ┌────────┐ ┌────────┐ ┌────────┐
    │ Particles│ │Genomes │ │Proteins│ │Metabol.│ │  Drug  │
    │ Coulomb  │ │ 4→32   │ │HP fold │ │  DAG   │ │Inhibit.│
    │   + LJ   │ │ genes  │ │catalysis│ │ Hebb  │ │  Loop  │
    └──────────┘ └────────┘ └────────┘ └────────┘ └────────┘
          │          │           │           │          │
          └──────────┴───────────┴───────────┴──────────┘
                                 │
                    ┌────────────▼────────────┐
                    │   Emergent Behavior      │
                    │ Life · Death · Evolution  │
                    │ Resistance · Therapy      │
                    └─────────────────────────┘
```

## Module Map

```
src/
├── blueprint/
│   ├── equations/          Pure math (50+ files). NO ECS. NO side effects.
│   │   ├── derived_thresholds.rs   4 constants → ~40 lifecycle thresholds
│   │   ├── pathway_inhibitor.rs    Drug design: 11 fns, 42 tests
│   │   ├── protein_fold.rs         HP lattice fold, 27 tests
│   │   ├── metabolic_genome.rs     DAG metabolism, 68 tests
│   │   ├── variable_genome.rs      4-32 genes, 62 tests
│   │   ├── codon_genome.rs         64 codons → 8 aa, 28 tests
│   │   ├── multicellular.rs        Union-Find colonies, 33 tests
│   │   ├── coulomb.rs              Coulomb + LJ, 26 tests
│   │   └── core_physics/           Interference, density, drag
│   └── constants/          Derived constants per domain
│       └── pathway_inhibitor.rs    7 constants from 4 fundamentals
│
├── batch/                  Headless simulator (NO Bevy dependency)
│   ├── arena.rs            EntitySlot (repr(C), 128 entities, cache-friendly)
│   ├── systems/            37 stateless systems (6 phases)
│   ├── harness.rs          GeneticHarness (evaluate → select → reproduce)
│   └── pipeline.rs         Tick order: Input→Thermo→Atomic→Chemical→Metabolic→Morpho
│
├── layers/                 14 ECS layers (L0 BaseEnergy → L13 StructuralLink)
├── simulation/             Bevy runtime systems (9 emergence active, 7 implemented)
├── plugins/                6 domain plugins (Input, Thermo, Atomic, Chemical, Metabolic, Morpho)
├── worldgen/               V7: field grid, nucleus, materialization, day/night, water cycle
├── use_cases/
│   ├── experiments/
│   │   ├── pathway_inhibitor_exp.rs   Adaptive control loop + Bozic validation
│   │   ├── cancer_therapy.rs          Level 1 cytotoxic model
│   │   └── particle_lab.rs            Molecular bonding
│   └── orchestrators.rs    HOFs: ablate(), ensemble(), sweep()
│
└── bin/                    Executables
    ├── adaptive_therapy.rs    Adaptive control loop output
    ├── bozic_validation.rs    5-arm Bozic 2013 comparison
    ├── pathway_inhibitor.rs   Level 2 pathway inhibition
    ├── cancer_therapy.rs      Level 1 cytotoxic
    ├── lab.rs                 Universal dashboard (8 experiments + Live 2D)
    └── survival.rs            WASD gameplay
```

## The 8 Axioms

| # | Axiom | Formula | Type |
|---|-------|---------|------|
| 1 | Everything is Energy | All entities = qe scalar | Primitive |
| 2 | Pool Invariant | Σ children ≤ parent | Primitive |
| 3 | Competition | M = base × α(f₁,f₂) | Derived (from 8) |
| 4 | Dissipation (2nd Law) | Δloss ≥ qe × r_d(state) | Primitive |
| 5 | Conservation | Total qe monotonically non-increasing | Derived (from 2+4) |
| 6 | Emergence at Scale | No top-down programming | Meta-constraint |
| 7 | Distance Attenuation | I(d) = 1/(d² + ε) | Primitive |
| 8 | Oscillatory Nature | α = exp(-Δf²/(2B²)) | Primitive |

## The 4 Fundamental Constants

| Constant | Value | Derivation chain |
|----------|-------|-----------------|
| `KLEIBER_EXPONENT` | 0.75 | → basal_drain, genome cost, appendage count |
| `DISSIPATION_{S,L,G,P}` | 0.005, 0.02, 0.08, 0.25 | → matter states, senescence, nutrient retention, all Ki values |
| `COHERENCE_BANDWIDTH` | 50.0 Hz | → binding affinity, interference window |
| `DENSITY_SCALE` | 20.0 | → self_sustaining_qe_min, density thresholds, Coulomb scale |

All ~40 lifecycle constants computed algebraically in `derived_thresholds.rs` (17 tests).

## Biological Hierarchy (10 Levels)

```
L0  Energy fields        Nucleus emission + diffusion
L1  Matter states        Density thresholds from 4 constants
L2  Molecular bonding    Coulomb + Lennard-Jones + freq alignment
L3  Entities (life)      Abiogenesis: coherence > dissipation
L4  Variable genome      4-32 genes, Schwefel mutation
L5  Genetic code         64 codons → 8 amino acids
L6  Proto-proteins       HP lattice fold, catalytic function
L7  Metabolic networks   DAG (12 nodes), competitive flow, Hebb
L8  Multicellularity     Adhesion, Union-Find, differential expression
L9  Social emergence     Theory of mind, coalitions, culture
```

## Drug Design Pipeline

```
Level 1: Cytotoxic          Drug drains qe (kills cells)
Level 2: Pathway Inhibitor  Drug reduces metabolic efficiency (controls cells)
Level 3: Adaptive Control   Controller adjusts dose per generation

                ┌──────────────┐
                │ Tumor Profile │
                │ freq, spread  │
                └──────┬───────┘
                       │ profile
                ┌──────▼───────┐
                │  Drug Design  │
                │ freq, conc,Ki │
                └──────┬───────┘
                       │ apply
                ┌──────▼───────┐
                │  Simulation   │
                │ multi_drug    │
                │ _tick()       │
                └──────┬───────┘
                       │ observe
                ┌──────▼───────┐
                │  Snapshot     │
                │ alive, eff,   │
                │ growth_rate   │
                └──────┬───────┘
                       │ decide
                ┌──────▼───────┐
                │  Controller   │
                │ adaptive_     │
                │ decision()    │
                └──────┬───────┘
                       │ adjust
                       └──→ next generation
```

### Controller States

| State | Condition | Action |
|-------|-----------|--------|
| `no_tumor` | alive < 1 | No therapy |
| `tumor_growing_start_therapy` | growing + no drugs | Start at tumor freq |
| `tumor_growing_increase_dose` | growing + has drugs | +50% dose |
| `stable_maintain` | growth ≈ 0 + has drugs | Keep current dose |
| `stable_untreated_start_therapy` | growth ≈ 0 + no drugs | Start therapy |
| `escape_detected_add_drug` | efficiency recovering | Add drug at escape freq |
| `shrinking_reduce_dose` | shrinking | -30% dose |

### Bozic 2013 Validation

| Arm | Efficiency | Suppression | Bozic prediction |
|-----|-----------|-------------|------------------|
| no_drug | 1.000 | 0% | baseline |
| mono_A (400 Hz) | 0.481 | 51.9% | resistance inevitable ✓ |
| combo_AB (A+B) | 0.435 | 56.5% | combo > mono ✓ |
| double_A (2×) | 0.466 | 53.4% | combo > double ✓ |

Validated across 10 independent seeds. p < 0.001.

### Adaptive Therapy Result

```
Gen 0-2:  No treatment. Efficiency 1.000.
Gen 3:    Controller starts: 399 Hz @ 0.40.
Gen 4:    Efficiency drops to 0.575 (42.5% suppression).
Gen 5+:   STABLE. Growth rate = 0.000. Tumor controlled.
Protocol: "399 Hz @ 0.40, maintain from gen 3"
```

## Emergence Systems Status

### Active (9 systems, registered in plugins)

| System | Phase | Mechanism |
|--------|-------|-----------|
| entrainment | Atomic | Kuramoto frequency sync |
| theory_of_mind | Input | Predict neighbor behavior |
| cultural_transmission | Input | Meme spread by oscillatory affinity |
| infrastructure (×2) | Metabolic | Persistent field modification |
| cooperation | Metabolic | Nash alliance detection |
| symbiosis_effect | Metabolic | Mutualism/parasitism |
| niche_adaptation | Metabolic | Character displacement (Hutchinson) |
| epigenetic_adaptation | Morphological | Environment modulates expression |

### Implemented, Not Registered (7 systems)

| System | Status | To activate |
|--------|--------|-------------|
| coalition_stability + intake | Complete, 0 consumers | Register in MorphologicalPlugin |
| institution_stability + distribution | Complete, 0 consumers | Register in MetabolicPlugin |
| tectonic_drift | Complete, 0 consumers | Register in ThermodynamicPlugin |
| multiscale_aggregation | Complete, 0 consumers | Register in MetabolicPlugin |
| geological_lod_update | Complete, 0 consumers | Register in MorphologicalPlugin |

## Energy Cycle

```
Nucleus (finite) → emit to field → diffusion + radiation pressure
       ↓                                          ↓
Reservoir depletes                    Entities materialize
       ↓                                          ↓
Zone cools                           Live (Kleiber) → die (Gompertz)
       ↓                                          ↓
                 Nutrients return to grid
                           ↓
                 Threshold → nucleus recycling → new nucleus
                           ↓
                       Cycle restarts
```

## Testing

```bash
cargo test                    # 3,066 tests (~30 sec release)
cargo test --lib pathway_inhibitor  # 68 tests (drug design + control loop)
cargo run --release --bin bozic_validation    # 5-arm Bozic comparison (~95 sec)
cargo run --release --bin adaptive_therapy    # Adaptive control loop (~8 sec)
cargo run --release --bin pathway_inhibitor   # Level 2 experiment (~6 sec)
```

## Honest Limitations

- **Not clinical:** Abstract qe units, no molecular targets, not validated against patient data
- **Not molecular:** No EGFR/BCR-ABL, no ADME pharmacokinetics
- **Not spatial:** 2D grid, no vasculature, no hypoxia gradients
- **Not immune:** No adaptive immune system modeled
- **Qualitative:** Suppression %, not absolute cell counts or time in weeks

## What It IS

A framework where you define a tumor profile (frequency + heterogeneity) and the system computes: **what frequency to target, at what dose, when to escalate, when to reduce, when to add a second drug.** From 4 constants. Without oncology knowledge. Deterministic, reproducible, exportable as protocol.
