# Distribution Posts — Copy & Paste

Reemplazar `[ARXIV_URL]` con el link real (ej: `https://arxiv.org/abs/2603.xxxxx`).
Repo: `https://github.com/ResakaGit/RESONANCE`

---

## 1. Hacker News (Show HN)

### Título (max 80 chars):
```
Show HN: Life emerges from 4 constants — cancer resistance included (109K LOC Rust)
```

### URL field:
```
[ARXIV_URL]
```

### Primer comentario (postear inmediatamente después):
```
Author here. Resonance is a simulation engine where everything — from molecular bonds to chemotherapy resistance — derives from 4 physical constants:

- Kleiber exponent (0.75) — metabolic scaling
- Dissipation rates (0.005→0.25) — Second Law per matter state
- Coherence bandwidth (50 Hz) — oscillatory interaction window
- Density scale (20.0) — spatial normalization

All ~40 lifecycle thresholds are computed algebraically. Zero tuning. Zero per-species parameters.

The drug model: a drug increases the dissipation rate of cells whose frequency matches the target. Cells with different frequencies survive. Resistance isn't programmed — it's a statistical consequence of frequency heterogeneity in the tumor population. Results are consistent with Bozic et al. 2013 (eLife).

The simulation stack (each level emerges from the previous):
Energy → matter → molecules (Coulomb+LJ) → abiogenesis → variable genomes (4→32 genes) → codon-based genetic code → proto-proteins → metabolic networks → multicellularity → social emergence

Try it:
  cargo run --release --bin lab              # Universal lab with egui dashboard
  cargo run --release --bin cancer_therapy   # Tumor dynamics → CSV
  cargo run --release --bin survival         # Play as an evolved creature
  cargo test                                 # 2,994 tests

Code: https://github.com/ResakaGit/RESONANCE (AGPL-3.0)
Paper: [ARXIV_URL]

Happy to answer questions about the axioms, the derivation chain, or the oncology model.
```

---

## 2. Reddit r/rust

### Título:
```
Emergent life simulation — 109K LOC Rust, 2994 tests, 4 constants → molecules, genomes, cancer resistance
```

### Body:
```
I built Resonance, a simulation engine where life emerges from 4 fundamental constants and 8 axioms. Everything is Rust + Bevy 0.15 ECS.

**What it does:** Define physics. Press play. Watch life emerge.

- 14 orthogonal ECS layers, 6-phase deterministic pipeline
- Variable-length genomes grow from 4 to 32 genes
- Codon-based genetic code (64 codons → 8 amino acids, evolvable)
- Metabolic networks with Hebbian rewiring
- Multicellularity via Union-Find colony detection
- Chemotherapy resistance emerges from the same equations that form molecular bonds
- All ~40 lifecycle constants algebraically derived from 4 numbers

**Stack:** Rust 2024 edition, Bevy 0.15, glam, rayon, bevy_egui, egui_plot. No unsafe. Deterministic (same seed = identical f32 bits).

**Try it:**
```bash
cargo run --release --bin lab              # egui dashboard with 8 experiments
cargo run --release --bin survival         # WASD survival mode
cargo run --release --bin cancer_therapy -- --out tumor.csv
cargo test                                 # 2,994 tests
```

**The drug model:** A drug increases dissipation rate (Second Law). Cells with frequencies far from the target survive via Gaussian attenuation. Resistance emerges from heterogeneity, not from a programmed gene.

Paper: [ARXIV_URL]
Repo: https://github.com/ResakaGit/RESONANCE (AGPL-3.0)

Feedback welcome — especially on the axiomatic derivation chain.
```

---

## 3. Reddit r/compsci

### Título:
```
Emergent life from 4 constants: simulation engine deriving cancer resistance from thermodynamic first principles
```

### Body:
```
Paper: [ARXIV_URL]

We present an open-source simulation engine (109K LOC Rust, 2,994 tests) where ~40 lifecycle parameters are algebraically derived from 4 fundamental constants.

Key result: the same equations that produce molecular bonds also produce chemotherapy resistance dynamics consistent with Bozic et al. 2013 (eLife). No cell-type-specific parameterization — resistance emerges from oscillatory frequency heterogeneity under selective pressure.

The 4 constants: Kleiber exponent (0.75), dissipation rates per matter state (0.005-0.25), coherence bandwidth (50 Hz), density scale (20.0).

10 levels of biological organization emerge bottom-up: energy → matter → molecules → abiogenesis → genomes → genetic code → proteins → metabolic networks → multicellularity → social behavior.

Code: https://github.com/ResakaGit/RESONANCE (AGPL-3.0)

Interested in feedback from computational biology and ALife researchers on whether this axiomatic approach has merit for drug resistance modeling.
```

---

## 4. Reddit r/bioinformatics

### Título:
```
Simulating chemotherapy resistance from first principles — no cell-type parameters, results consistent with Bozic 2013
```

### Body:
```
I built a simulation engine (Rust, open-source AGPL-3.0) where drug resistance emerges from fundamental physics rather than cell-type-specific rules.

**Drug mechanism:** Increases dissipation rate (thermodynamic Second Law). Frequency alignment determines selectivity:

effect = Potency × exp(-Δf²/2B²) × base_dissipation

**What emerges (not programmed):**
- Monotherapy at moderate potency → tumor persists (resistance rate consistent with Bozic et al. 2013)
- Sigmoidal dose-response (Hill equation behavior, emergent)
- Quiescent stem cells reactivate when tumor burden drops
- Normal tissue regenerates during drug holidays
- Clonal diversity increases under selective pressure

**What's different:** The same 4 constants that determine molecular bonding also determine drug efficacy. No pharmacology module. No resistance genes. Resistance = statistical consequence of frequency diversity.

**Try it:**
```bash
cargo run --release --bin lab  # egui dashboard, select "Cancer Therapy"
cargo run --release --bin cancer_therapy -- --potency 0.5 --out resistance.csv
```

Paper: [ARXIV_URL]
Code: https://github.com/ResakaGit/RESONANCE

Looking for feedback from computational oncology researchers. Would love to collaborate on clinical calibration.
```

---

## 5. Reddit r/artificial (Artificial Life)

### Título:
```
10 levels of biological organization from 4 constants: molecules, genomes, proteins, metabolism, multicellularity, drug resistance
```

### Body:
```
I built Resonance — a simulation engine where 10 levels of biological organization emerge bottom-up from 8 axioms and 4 fundamental constants.

The hierarchy (each level emerges from the previous):

0. Energy fields
1. Matter states (solid/liquid/gas/plasma from density thresholds)
2. Molecular bonding (Coulomb + Lennard-Jones + frequency alignment)
3. Abiogenesis (life where coherence > dissipation)
4. Variable genomes (4→32 genes via duplication/deletion)
5. Genetic code (64 codons → 8 amino acids, evolvable)
6. Proto-proteins (HP lattice folding, active sites)
7. Metabolic networks (DAG with Hebbian rewiring, competition)
8. Multicellularity (adhesion, colonies, differential expression)
9. Social emergence (theory of mind, coalitions, cultural transmission)

The 4 constants: Kleiber exponent (0.75), dissipation rates (0.005-0.25), coherence bandwidth (50 Hz), density scale (20.0). Everything else is derived.

The most surprising result: chemotherapy resistance emerges from the same frequency alignment that determines molecular bonding.

**22 binaries including:**
- `cargo run --release --bin lab` — universal lab with Live 2D simulation
- `cargo run --release --bin survival` — play as an evolved creature (WASD)
- `cargo run --release --bin fermi` — how many random universes develop life?

Paper: [ARXIV_URL]
Code: https://github.com/ResakaGit/RESONANCE (109K LOC Rust, 2,994 tests, AGPL-3.0)
```

---

## 6. Twitter/X Thread

```
I derived chemotherapy resistance from 4 physical constants.

Not a metaphor. The same equations that form molecular bonds predict why tumors survive treatment.

109,000 lines of Rust. 2,994 tests. Zero hardcoded species.

Thread ↓
```

```
1/ The 4 constants:
• Kleiber exponent: 0.75 (metabolism ∝ mass^0.75)
• Dissipation rates: 0.005→0.25 (Second Law per matter state)
• Coherence bandwidth: 50 Hz (oscillatory window)
• Density scale: 20.0 (spatial normalization)

From these, ~40 lifecycle thresholds are DERIVED algebraically. Not tuned. Computed.
```

```
2/ 10 levels of biology emerge bottom-up:

Energy → matter → molecules (Coulomb) → life (abiogenesis) → genomes (4→32 genes) → genetic code (64 codons) → proteins (lattice fold) → metabolic networks (Hebb) → multicellularity → social behavior

Each from the previous. Nothing programmed.
```

```
3/ The drug model:

A drug increases dissipation rate (Second Law):
effect = Potency × alignment(f_drug, f_cell) × base_rate

Cells with different frequencies? Low alignment → they survive.

That's resistance. Emergent. Not coded.
```

```
4/ Results match Bozic et al. 2013 (eLife):
• Moderate monotherapy → tumor persists
• Stem cells cause relapse after initial response
• Clonal diversity increases under treatment

All from 4 constants. No oncology parameters.
```

```
5/ You can try it right now:

cargo run --release --bin lab
→ egui dashboard with Cancer Therapy experiment

cargo run --release --bin survival
→ play as an evolved creature (WASD)

Paper: [ARXIV_URL]
Code: github.com/ResakaGit/RESONANCE
License: AGPL-3.0

Feedback welcome. Especially from oncologists.
```

---

## 7. LinkedIn

```
I just published a paper on arXiv: an open-source simulation engine where chemotherapy resistance emerges from 4 physical constants.

No cell-type parameters. No hardcoded resistance genes. The same equations that form molecular bonds predict tumor survival under treatment.

Key results:
→ 10 levels of biological organization emerge bottom-up
→ Drug resistance matches Bozic et al. 2013 (eLife)
→ 109,000 lines of Rust, 2,994 tests, AGPL-3.0

The engine includes a universal lab (egui dashboard) with experiments for cancer therapy, speciation, convergent evolution, and the Fermi paradox — all from the same 4 constants.

Paper: [ARXIV_URL]
Code: https://github.com/ResakaGit/RESONANCE

Looking to connect with computational oncology researchers for clinical calibration. If you work in drug resistance modeling, I'd love to hear your perspective.

#ComputationalBiology #OpenSource #Rust #CancerResearch #ArtificialLife
```

---

## Posting Schedule

| Día | Plataforma | Hora óptima | Nota |
|-----|-----------|-------------|------|
| Día 1 (post-arXiv) | Hacker News | 9-10 AM EST, Mar-Jue | Show HN. NO fines de semana. |
| Día 1 (+2h) | r/rust | Después de HN | Cross-link al HN thread |
| Día 1 (tarde) | Twitter/X thread | 12-2 PM EST | Con screenshot del lab UI |
| Día 2 | r/compsci + r/artificial | 9 AM EST | Academic audience |
| Día 2 | LinkedIn | Mediodía | Professional network |
| Día 3 | r/bioinformatics | 9 AM EST | Domain specialists |

**Reglas:**
- NUNCA publicar antes de que arXiv muestre el paper (24-48h post-submit)
- Incluir screenshot del lab UI (Cancer Therapy chart) en Twitter y Reddit
- Responder a TODOS los comentarios las primeras 6 horas
- NO cross-postear el mismo texto en múltiples subreddits — cada uno tiene su versión
