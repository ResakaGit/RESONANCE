# Resonance — Emergent Life Simulation Engine

[![CI](https://github.com/ResakaGit/RESONANCE/actions/workflows/ci.yml/badge.svg)](https://github.com/ResakaGit/RESONANCE/actions/workflows/ci.yml)
[![License: AGPL-3.0](https://img.shields.io/badge/License-AGPL--3.0-blue.svg)](./LICENSE)
[![Tests: 3,166](https://img.shields.io/badge/tests-3%2C166%20passing-brightgreen)]()
[![Papers: 6/6](https://img.shields.io/badge/papers-6%2F6%20qualitative%20match-blue)]()
[![Safety Class: A](https://img.shields.io/badge/IEC%2062304-Class%20A-green)]()

Simulation engine where life, evolution, and therapeutic strategies emerge from **8 axioms** and **4 fundamental constants**. Built with **Rust** and **Bevy 0.15 ECS**. Open source (AGPL-3.0).

Preprint (not peer-reviewed): https://zenodo.org/records/19342036 | Repo: https://github.com/ResakaGit/RESONANCE

> 👉 **New here?** Three demos in 5 minutes: [`docs/DEMOS.md`](./docs/DEMOS.md)

## What It Does

Define the laws of physics. Press play. Watch life emerge. Design therapeutic strategies from first principles.

- **10-level biological hierarchy** — from energy fields to social emergence, all emergent
- **Pathway-level drug design** — inhibit specific metabolic pathways without killing cells
- **Adaptive therapy controller** — feedback loop stabilizes growth within its own model (not clinically validated)
- **Qualitatively consistent with Bozic 2013** — combo > mono reproduced in 10/10 seeds (qualitative, not quantitative — uses own parameters, not Bozic's b/d/u rates)
- **Clinical unit mapping** — post-hoc conversion to nM, days, cell count via published data (not predictive calibration)
- **3,166 automated tests** — deterministic, bit-exact reproducible
- **6 published papers — qualitative match** — structural predictions consistent with Bozic, Zhang, Sharma, GDSC/CCLE, Foo & Michor, Michor

## What It Is NOT

- **Not a clinical tool** — not validated against patient outcomes
- **Not a drug discovery pipeline** — does not design molecules
- **Not a substitute for oncology** — a simulator for exploring therapeutic strategies

**Clinical unit mapping (post-hoc, not predictive)** — simulation output can be converted to clinical units (nM, days, cell count) via published pharmacological data. This is a post-hoc linear mapping, not a prediction. Three profiles: CML/imatinib (Bozic 2013), prostate/abiraterone (Gatenby 2009), NSCLC/erlotinib. Example: "399 Hz @ 0.40, gen 3" maps to "Imatinib 104 nM, start day 12" — but this mapping is arbitrary, not derived from the model.

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

### Experiment 5: Qualitative Comparison with Bozic 2013

Combination therapy advantage reproduced qualitatively (10/10 seeds). **Note:** this uses RESONANCE's own parameters (frequency-based binding, abstract qe units), not Bozic's original parameters (b=0.14/day, d=0.13/day, u=10⁻⁹). The qualitative conclusion (combo > mono > double-dose) is consistent, but the quantitative curves are not directly comparable.

| Arm | Efficiency | Suppression | Prediction |
|-----|-----------|-------------|------------|
| no_drug | 1.000 | 0% | baseline |
| mono_A (400 Hz) | 0.481 | 51.9% | resistance inevitable ✓ |
| **combo_AB** | **0.435** | **56.5%** | **combo > mono ✓** |
| double_A (2×) | 0.466 | 53.4% | **combo > double ✓** |

### Experiment 6: Adaptive Therapy Controller (internal model only)

Feedback loop stabilizes tumor growth at **zero net expansion** within RESONANCE's own model. This demonstrates the controller works on its own rules — it is not validated against patient data or independent simulators:

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

### Experiment 7: Rosie Case — Speculative (Canine Mast Cell Tumor)

**This experiment is speculative, not validated.** Simulates a real-world case from press reports (not peer-reviewed data) about a personalized mRNA cancer vaccine for a dog. Uses population-level parameters (70% KIT+/30% KIT- from London 2003), not the individual animal's tumor profile. The vaccine works via immune-mediated killing; RESONANCE models direct pathway inhibition — fundamentally different mechanisms. Toceranib IC50 used as pharmacological proxy (not mechanism equivalent).

| Observed (real) | Predicted (simulation) | Match |
|----------------|----------------------|-------|
| Mono vaccine → 75% tumor reduction | Mono → efficiency drops 50-70% | Pattern ✓ |
| Some tumors didn't respond | Resistant fraction persists (eff > 0.05) | ✓ |
| Surgery needed after vaccine | Mono insufficient to eliminate | ✓ |
| Second target not tried | **Combo (KIT+ & KIT-) suppresses more than mono** | Prediction |

Validated across 5 seeds. Partial response is structural, not stochastic.

**Calibrated (canine mast cell):** 21-day doubling, toceranib IC50 = 40 nM as pharmacological proxy (not mechanism equivalent — toceranib is a kinase inhibitor, the real vaccine is immune-mediated), ~10⁸ cells.

**Known limitations of this simulation:**
- Efficiency reduction ≠ tumor volume reduction (we measure metabolic suppression, not cell death)
- 70/30 KIT+/KIT- split is from population-level prevalence (London 2003), not Rosie's individual tumor
- No immune system modeled — vaccine is simulated as direct pathway inhibitor, not immune-mediated response
- Frequency is a computational proxy for genetic identity, not a measured biological observable
- The partial response pattern matches but the underlying mechanism differs (direct inhibition vs T-cell mediated killing)

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
| Against published prediction | ✓ 6 independent papers — qualitative structural match (see below) |
| Clinical unit mapping | ✓ 4 profiles (CML, prostate, NSCLC, canine MCT) — post-hoc mapping, not predictive |
| Against patient outcomes | **Not yet** — calibrated but not validated against longitudinal patient data |

### Paper Comparison Suite (6 comparators + unified axiom test)

All comparisons are **qualitative** — structural predictions reproduced within RESONANCE's own model using its own parameters. These are NOT quantitative reproductions of the original papers' exact curves or parameter values. Run: `cargo run --release --bin paper_validation`

| Paper | Prediction tested | RESONANCE result | Match type |
|-------|------------------|-----------------|------------|
| Bozic 2013 (eLife) | Combo > mono therapy | 56.5% vs 51.9% suppression, 10/10 seeds | Qualitative (own params, not Bozic's b/d/u) |
| Zhang 2022 (eLife) | Adaptive TTP > continuous | 1.50× ratio, 3 cycles | Qualitative (Lotka-Volterra, calibrated) |
| Sharma 2010 (Cell) | Drug-tolerant persisters survive + recover | 2% fraction, recovery detected | Pattern match (scaled population) |
| GDSC/CCLE (Nature) | Hill n=2 within empirical distribution | Within IQR and 1σ | Statistical (input validation, not output) |
| Foo & Michor 2009 (PLoS) | Pulsed < continuous resistance | 15% vs 25% | Qualitative (own params) |
| Michor 2005 (Nature) | Biphasic CML decline, stem survive | 8.0× slope ratio | Qualitative (3 subpops, own params) |

### PV-6: Unified Axiom Test (4 constants, zero calibration)

Can 4 numbers reproduce all 6 phenomena without manual tuning? Every parameter derived algebraically from KLEIBER (0.75), DISSIPATION (0.005-0.25), BANDWIDTH (50 Hz), DENSITY_SCALE (20.0).

| Test | Phenomenon | From 4 constants? | Result |
|------|-----------|-------------------|--------|
| T1 | Combo > mono | Drug potency = LIQUID/SOLID ratio | **PASS** |
| T2 | Adaptive > continuous | Fitness cost = LIQUID/GAS ratio | **PASS** |
| T3 | Persisters survive | Quiescent fraction = DISSIPATION_SOLID | **FAIL** |
| T4 | Hill n=2 valid | Within published IQR | **PASS** |
| T5 | Pulsed < continuous | Frequency drift = BANDWIDTH/3 | **PASS** |
| T6 | Biphasic decline | Stem freq offset = 3× BANDWIDTH | **FAIL** |

**4/6 PASS.** The 2 FAILs reveal a real model boundary: the batch simulator's nutrient-driven carrying capacity prevents net-death regimes needed for Sharma (persisters) and Michor (biphasic). Relative comparisons (T1, T2, T5) work because they don't require absolute population decline. This is an honest result — it shows exactly where the axioms are sufficient and where the simulator's ecology model diverges from clinical kill dynamics.

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

## From Particle to Cure — Mechanistic Pipeline

How organs, tumors, and therapeutic strategies emerge from the same 8 axioms without scripting or templates.

### Level 0: Minimal Entity

Every entity starts as `qe` (energy) + `frequency` (identity). Existence requires `qe > 0`. No cell type, no labels — just physics.

### Level 1: Organ Inference

Organs are never declared — they emerge from physics:

```
Energy + Volume + Biomass → Lifecycle Stage (Dormant → Emerging → Growing → Mature → Reproductive → Declining)
    → InferenceProfile (growth_bias, branching_bias) × Environment Viability
    → OrganManifest (up to 12 slots: Stem, Root, Leaf, Thorn, Shell, Limb, Fin...)
    → BodyPlanLayout (bilateral or thermodynamic optimization)
    → GF1 Mesh (torso + organ sub-meshes, merged)
```

Changing temperature shifts `branching_bias` (Bergmann/Allen rules) and reorganizes organs automatically.

### Level 2: Tumor Emergence

A tumor emerges from **frequency and trophic class**, not from a "cancer" label:

| Cell type | Frequency | Trophic | Metabolism |
|-----------|----------|---------|------------|
| Normal | ~250 Hz | Producer | Photosynthesis (freq-aligned) |
| Cancer | ~400 Hz | Detritivore | Direct nutrient scavenge |
| Quiescent stem | ~200 Hz | Detritivore | Dormant (growth_bias ~0.01) |

Cancer cells outcompete because their frequency doesn't align with photosynthetic machinery — they scavenge nutrients directly, growing faster in vascularized environments.

### Level 3: Drug Mechanism (Frequency-Selective)

```
alignment  = exp(-df^2 / (2 * bandwidth^2))              -- Gaussian (Axiom 8)
response   = potency * alignment^n / (EC50^n + alignment^n)  -- Hill n=2
drain      = response * 0.5 qe/tick                        -- applied post-uptake, pre-death
```

Bandwidth controls selectivity: narrow = antibody-like, broad = alkylating agent. Drug targets 400 Hz — kills cancer (alignment ~1.0), spares normal cells at 250 Hz (alignment ~0).

### Level 4: Three Resistance Layers (All Emergent)

| Layer | Mechanism | Timescale | Axioms |
|-------|-----------|-----------|--------|
| **Frequency escape** | Population tail survives drug bandwidth; clonal expansion shifts mean frequency | ~10-20 gen | 3, 8 |
| **Quiescent persistence** | Dormant stems at offset frequency; reactivate when niche empties | Indefinite → relapse | 6, 7, 8 |
| **Metabolic compensation** | Alternative pathway rerouting + epigenetic silencing reduces inhibitor load | Continuous | 4, 6 |

None require random mutation — all emerge from frequency distributions, energy constraints, and competitive dynamics.

### Level 5: Therapeutic Strategies

| Strategy | Why it works (in the model) | Validated against |
|----------|----------------------------|-------------------|
| Combo > Mono | Two frequencies cover more of the resistance spectrum | Bozic 2013 |
| Adaptive > Continuous | Drug holidays let sensitive cells recover and re-suppress resistant | Zhang 2022 |
| Pulsed therapy | Reduced selection pressure slows frequency drift | Foo & Michor 2009 |

### Derived Constants (Zero Manual Calibration)

All therapy parameters derive algebraically from the 4 fundamentals:

| Parameter | Derivation | Value |
|-----------|-----------|-------|
| Drug potency | DISSIPATION_LIQUID / DISSIPATION_SOLID | 4.0 |
| Tumor frequency | 8 * COHERENCE_BANDWIDTH | 400 Hz |
| Resistant offset | 3 * BANDWIDTH | 150 Hz |
| Resistance fitness cost | DISSIPATION_LIQUID / DISSIPATION_GAS | 0.25 |

See `blueprint/equations/derived_thresholds.rs` for the full derivation chain.

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
cargo test --release    # 3,166 tests (113K LOC, ~88 sec)
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
├── use_cases/experiments/  11 validated experiments (6 papers + PV-6 unified)
└── bin/                    26 executables
```

## Documentation

| Document | Description |
|----------|-------------|
| [docs/ARCHITECTURE.md](./docs/ARCHITECTURE.md) | Canonical architecture — axioms, constants, module map, pipeline |
| [docs/regulatory/](./docs/regulatory/) | 46 regulatory documents (IEC 62304, ISO 14971, ISO 13485, ASME V&V 40, FDA CMS 2023, 21 CFR Part 11) |
| [docs/regulatory/AUDIT_CHECKLIST.md](./docs/regulatory/AUDIT_CHECKLIST.md) | Master audit index — 50/50 external checklist items mapped |
| [docs/arquitectura/ADR/](./docs/arquitectura/ADR/) | 13 Architecture Decision Records |
| [docs/paper/](./docs/paper/) | arXiv paper source (7 experiments, 12 references) |
| [docs/design/](./docs/design/) | Code-referenced design specs |
| [docs/sprints/](./docs/sprints/) | Sprint backlog (37 pending) + [archive/](./docs/sprints/archive/) (88 completed) |

## Regulatory Status

RESONANCE is a **research tool**, not a medical device. Voluntary compliance documentation exists for credibility and partnership readiness.

| Standard | Classification | Document |
|----------|---------------|----------|
| IEC 62304 | **Safety Class A** (no injury possible) | [SOFTWARE_SAFETY_CLASS.md](./docs/regulatory/01_foundation/SOFTWARE_SAFETY_CLASS.md) |
| IMDRF SaMD | **Category I** (Non-serious, Inform) | [INTENDED_USE.md](./docs/regulatory/01_foundation/INTENDED_USE.md) |
| ISO 14971 | 12 hazards, 52 controls, all ALARP or Acceptable | [RISK_ANALYSIS.md](./docs/regulatory/02_risk_management/RISK_ANALYSIS.md) |
| ASME V&V 40 | Credibility model complete (§4-8) | [CREDIBILITY_MODEL.md](./docs/regulatory/04_validation/CREDIBILITY_MODEL.md) |
| ISO 13485 | QMS minimal viable (Quality Manual + 6 procedures) | [QUALITY_MANUAL.md](./docs/regulatory/05_quality_system/QUALITY_MANUAL.md) |

**Disclaimer:** This documentation is voluntary best-practice, not regulatory obligation. RESONANCE is not FDA-cleared, CE-marked, or approved for clinical use. See [INTENDED_USE.md](./docs/regulatory/01_foundation/INTENDED_USE.md) and [LIMITATIONS_REPORT.md](./docs/regulatory/06_clinical/LIMITATIONS_REPORT.md).

## CI/CD

Every push to `main` and every PR runs 5 automated checks:

```
cargo check    — compilation
cargo test     — 3,166 tests
cargo clippy   — zero warnings
cargo audit    — no known CVEs
cargo fmt      — formatting
```

Branch protection requires PR + CI pass + review before merge.

## Requirements

- Rust 1.85+ (edition 2024)
- macOS / Linux / Windows
- No GPU required (headless mode available)

## Contributing

See [CONTRIBUTING.md](./CONTRIBUTING.md) for development workflow, coding rules, and how to submit changes.

## Security

See [SECURITY.md](./SECURITY.md) for vulnerability reporting policy.

## License

AGPL-3.0 — Free to use, study, modify, and distribute. Copyright (c) 2026 Augusto Gomez Saa. See [LICENSE](./LICENSE).
