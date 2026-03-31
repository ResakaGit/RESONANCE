# Distribution Posts — Copy & Paste

Paper: https://zenodo.org/records/19342036
Repo: https://github.com/ResakaGit/RESONANCE

---

## 1. Hacker News (Show HN)

### Título (max 80 chars):
```
Show HN: Simulation where cancer resistance emerges from thermodynamics (Rust, AGPL)
```

### URL field:
```
https://zenodo.org/records/19342036
```

### Primer comentario:
```
Author here. Resonance is a simulation engine where biological phenomena — including drug resistance — emerge from 4 physical constants and 8 thermodynamic axioms.

The 4 constants:
- Kleiber exponent (0.75) — metabolic scaling law
- Dissipation rates (0.005→0.25) — Second Law per matter state
- Coherence bandwidth (50 Hz) — oscillatory interaction window
- Density scale (20.0) — spatial normalization

All ~40 lifecycle thresholds are algebraically derived from these. No tuning per species.

The drug model: a drug increases dissipation rate (Second Law). Cells whose oscillatory frequency differs from the drug target experience less effect (Gaussian attenuation). The surviving cells reproduce, shifting the population frequency. This is emergent resistance — not programmed.

The resistance dynamics are consistent with predictions from Bozic et al. 2013 (eLife) for monotherapy, but this is a theoretical model — not clinically validated. It needs calibration against real tumor growth data to be useful for medicine.

What you can try:
  cargo run --release --bin lab              # Dashboard with 8 experiments including cancer therapy
  cargo run --release --bin survival         # Play as an evolved creature (WASD)
  cargo test                                 # 2,994 tests, 109K LOC

Code: https://github.com/ResakaGit/RESONANCE (AGPL-3.0)

Looking for feedback on the axiomatic approach and collaborators for clinical calibration.
```

---

## 2. Reddit r/rust

### Título:
```
Emergent life simulation engine in Rust — 109K LOC, 2994 tests, Bevy 0.15 ECS + egui dashboard
```

### Body:
```
I built Resonance, a simulation engine where biological organization emerges bottom-up from 4 constants and 8 axioms. Rust + Bevy 0.15 ECS.

**Architecture:**
- 14 orthogonal ECS layers (energy, volume, oscillation, flow, coherence, engine, pressure, will, injector, identity, link, tension, homeostasis, structure)
- 6-phase deterministic FixedUpdate pipeline
- Batch simulator: millions of worlds in parallel (rayon), deterministic (same seed = identical f32 bits)
- egui dashboard with real-time charts (bevy_egui 0.31 + egui_plot)
- Zero unsafe. AGPL-3.0.

**What emerges from the physics (not programmed):**
- Variable-length genomes (4→32 genes via duplication/deletion)
- Codon-based genetic code (64 codons → 8 amino acids, evolvable)
- Metabolic networks with Hebbian rewiring
- Multicellularity via colony detection
- Drug resistance under selective pressure

**Try it:**
```bash
cargo run --release --bin lab              # egui dashboard, 8 experiments + Live 2D
cargo run --release --bin survival         # WASD survival mode
cargo run --release --bin cancer_therapy -- --out tumor.csv
cargo test                                 # 2,994 tests
```

**Rust-specific highlights:**
- `repr(C)` flat entity for batch (EntitySlot, cache-friendly)
- RingBuffer<f32> stack-allocated [f32; 512] for dashboard time series
- Generic BridgeCache<B: BridgeKind> with compile-time type isolation
- HOF orchestrators: `ablate(closure)`, `ensemble()`, `sweep(closure, closure)`
- Zero-alloc CSV export via `write!` to String buffer

Paper: https://zenodo.org/records/19342036
Repo: https://github.com/ResakaGit/RESONANCE

Feedback welcome on the ECS architecture and the axiomatic derivation approach.
```

---

## 3. Reddit r/compsci

### Título:
```
Emergent biological organization from thermodynamic first principles — simulation engine with drug resistance dynamics
```

### Body:
```
Paper: https://zenodo.org/records/19342036

We present an open-source simulation engine (109K LOC Rust, 2,994 tests) where ~40 lifecycle parameters are algebraically derived from 4 fundamental constants.

10 levels of biological organization emerge bottom-up without type-specific rules:
energy → matter states → molecular bonding → abiogenesis → variable genomes → genetic code → protein folding → metabolic networks → multicellularity → social behavior

Key observation: under selective pressure from a frequency-targeted dissipation increase (modeling drug action), the simulated population develops resistance dynamics consistent with theoretical predictions from Bozic et al. 2013 (eLife) for monotherapy failure.

Limitations: this is a theoretical model operating on abstract energy units, not calibrated to specific drugs or cell lines. Clinical utility requires validation against real tumor growth curves — which we have not done.

The 4 constants: Kleiber exponent (0.75), dissipation rates (0.005-0.25), coherence bandwidth (50 Hz), density scale (20.0).

Code: https://github.com/ResakaGit/RESONANCE (AGPL-3.0)

Looking for feedback from computational biology and ALife researchers.
```

---

## 4. Reddit r/bioinformatics

### Título:
```
Open-source engine for simulating drug resistance from thermodynamic first principles (Rust, AGPL-3.0)
```

### Body:
```
I built a simulation engine where drug resistance emerges from fundamental physics rather than cell-type-specific rules.

**Drug mechanism:** Increases dissipation rate (thermodynamic Second Law), modulated by frequency alignment (Gaussian selectivity) and Hill dose-response:

effect = Hill(alignment(f_drug, f_cell, bandwidth)) × base_dissipation

**What emerges (not programmed):**
- Moderate monotherapy → tumor persists (frequency-mismatched clones survive)
- Sigmoidal dose-response curve (Hill equation behavior)
- Quiescent stem cells reactivate when tumor burden drops below threshold
- Clonal diversity increases under sustained selective pressure

**Honest limitations:**
- The model operates on abstract energy units (qe), not molar concentrations
- "Frequency" is a simulation abstraction, not a direct biological observable
- Results are consistent with Bozic et al. 2013 predictions but NOT validated against clinical data
- No ADME, no molecular targets, no tissue-specific pharmacology

**What it IS useful for:** exploring how resistance dynamics emerge from population heterogeneity without assuming specific resistance mechanisms. It's a hypothesis generator, not a clinical tool.

**Try it:**
```bash
cargo run --release --bin lab  # select "Cancer Therapy", adjust potency/bandwidth
cargo run --release --bin cancer_therapy -- --potency 0.5 --gens 50 --out resistance.csv
```

Paper: https://zenodo.org/records/19342036
Code: https://github.com/ResakaGit/RESONANCE

Would value feedback from computational oncology researchers. Looking for collaborators on calibration against real datasets.
```

---

## 5. Reddit r/artificial (Artificial Life)

### Título:
```
Bottom-up emergence of 10 levels of biological organization from 4 thermodynamic constants
```

### Body:
```
I built Resonance — a simulation engine where 10 levels of biological organization emerge from 8 axioms and 4 fundamental constants. No per-level rules. Each level is a consequence of the previous.

The hierarchy:
0. Energy fields (continuous qe distribution)
1. Matter states (density thresholds → solid/liquid/gas/plasma)
2. Molecular bonding (Coulomb + Lennard-Jones + frequency alignment)
3. Abiogenesis (life where coherence gain > dissipation cost)
4. Variable genomes (4→32 genes via duplication/deletion)
5. Genetic code (64 codons → 8 amino acids, evolvable mapping)
6. Proto-proteins (HP lattice folding, emergent active sites)
7. Metabolic networks (directed acyclic graph with Hebbian rewiring)
8. Multicellularity (frequency-based adhesion, Union-Find colonies, differential expression)
9. Social behavior (theory of mind, Nash coalitions, cultural transmission)

The 4 constants: Kleiber exponent (0.75), dissipation rates (0.005-0.25), coherence bandwidth (50 Hz), density scale (20.0).

The most unexpected result: applying selective pressure (frequency-targeted dissipation increase, modeling a drug) produces resistance dynamics — from the same equations that determine molecular bonding at level 2.

The engine includes a lab with Live 2D visualization where you can watch the simulation run in real time, and a survival mode where you control one creature with WASD.

Paper: https://zenodo.org/records/19342036
Code: https://github.com/ResakaGit/RESONANCE (109K LOC Rust, 2,994 tests, AGPL-3.0)
```

---

## 6. Twitter/X Thread

```
I built a simulation where cancer resistance emerges from the same physics that forms molecular bonds.

4 constants. 8 axioms. 109K lines of Rust. 2,994 tests. No hardcoded biology.

Here's what happens ↓
```

```
1/ The 4 constants:
• Kleiber exponent: 0.75 (metabolic scaling)
• Dissipation rates: 0.005→0.25 (Second Law)
• Coherence bandwidth: 50 Hz
• Density scale: 20.0

From these, ~40 thresholds are derived algebraically. Not tuned. Computed.
```

```
2/ 10 levels emerge bottom-up:

Energy → matter → molecules → life → genomes → genetic code → proteins → metabolic networks → multicellularity → social behavior

Each from the previous. Zero top-down programming.
```

```
3/ The drug model:

A drug increases dissipation (Second Law). Frequency alignment determines selectivity.

Cells with mismatched frequencies survive → reproduce → population shifts.

That's resistance. Emergent from physics.
```

```
4/ Consistent with Bozic et al. 2013 (eLife): monotherapy at moderate dose → tumor persists.

But honest caveat: this is a theoretical model. Not clinically validated. Useful for exploring resistance dynamics, not for prescribing treatment.
```

```
5/ Try it:

cargo run --release --bin lab
→ Dashboard with cancer therapy experiment + Live 2D

cargo run --release --bin survival
→ Play as an evolved creature

Paper: https://zenodo.org/records/19342036
Code: github.com/ResakaGit/RESONANCE (AGPL-3.0)

Feedback welcome — especially from computational biologists.
```

---

## 7. LinkedIn

```
I published a paper on Zenodo presenting Resonance — an open-source simulation engine where biological phenomena emerge from 4 thermodynamic constants.

The engine derives ~40 lifecycle thresholds algebraically. No per-species tuning. 10 levels of biological organization emerge bottom-up: from energy fields to molecular bonds to genomes to metabolic networks to multicellularity.

Under selective pressure (frequency-targeted dissipation increase), the simulated population develops drug resistance dynamics consistent with theoretical predictions from Bozic et al. 2013 (eLife).

Important caveat: this is a theoretical model, not a clinical tool. It needs calibration against real tumor growth data to be medically useful.

The project includes a universal lab with egui dashboard, survival gameplay mode, and CSV export for data analysis — all built in Rust (109K LOC, 2,994 tests, AGPL-3.0).

Paper: https://zenodo.org/records/19342036
Code: https://github.com/ResakaGit/RESONANCE

Looking to connect with computational biology researchers interested in axiomatic approaches to emergence and drug resistance modeling.

#ComputationalBiology #OpenSource #Rust #Simulation #ArtificialLife
```

---

## Posting Schedule

| Día | Plataforma | Hora óptima | Nota |
|-----|-----------|-------------|------|
| Hoy | Hacker News | 9-10 AM EST, Mar-Jue | Show HN. Paper ya está en Zenodo. |
| Hoy (+2h) | r/rust | Después de HN | Enfoque en arquitectura Rust |
| Hoy (tarde) | Twitter/X thread | 12-2 PM EST | Con screenshot del lab Cancer Therapy chart |
| Día 2 | r/compsci + r/artificial | 9 AM EST | Enfoque académico |
| Día 2 | LinkedIn | Mediodía | Red profesional |
| Día 3 | r/bioinformatics | 9 AM EST | Especialistas de dominio |

**Reglas:**
- Screenshot del lab UI (Cancer Therapy chart mostrando resistance) en Twitter y Reddit
- Responder a TODOS los comentarios las primeras 6 horas
- NO cross-postear el mismo texto — cada subreddit tiene su versión
- Ser honesto sobre limitaciones en CADA post — la comunidad científica castiga el hype
- Si preguntan "can this cure cancer?" → responder "No. It can help understand resistance dynamics."
