# Distribution Posts — Copy & Paste

Replace [ZENODO_DOI_URL] with your actual Zenodo DOI link.
Replace [GITHUB_URL] with your GitHub repo URL.

---

## 1. HackerNews (Show HN)

### Title (max 80 chars):
```
Show HN: I derived chemotherapy resistance from 4 physical constants (Rust)
```

### Body (paste in the URL field):
```
[ZENODO_DOI_URL]
```

### First comment (post immediately after):
```
Author here. I built a simulation engine where life emerges from 5 axioms and 4 constants — the Kleiber exponent (0.75), four dissipation rates, a coherence bandwidth, and a density scale.

The same equations that form Coulomb-bonded molecules also produce:
- Variable-length genomes (4→32 genes via duplication)
- Codon-based genetic codes (64 codons → 8 amino acids)
- Metabolic networks with Hebbian rewiring
- Multicellularity with differential expression
- Chemotherapy resistance matching Bozic et al. 2013 (eLife)

No species definitions. No hardcoded behavior. Everything derives algebraically from 4 numbers.

Drug model: a drug increases the dissipation rate (Second Law). Cells whose oscillatory frequency is far from the drug's target survive — resistance emerges from frequency heterogeneity, not from a programmed resistance gene.

Try it:
  cargo run --release --bin particle_lab  (molecules form in 7ms)
  cargo run --release --bin cancer_therapy (tumor dynamics in 2min)
  cargo test  (2,994 tests, 109K LOC Rust)

Paper: [ZENODO_DOI_URL]
Code: [GITHUB_URL]

Happy to answer questions about the axioms, the derivation chain, or the oncology model.
```

---

## 2. Reddit r/rust

### Title:
```
Show: Emergent life simulation engine — 109K LOC, 2994 tests, 5 axioms → molecules, genomes, cancer resistance
```

### Body:
```
I've been building Resonance, a simulation engine where life emerges from 5 axioms and 4 fundamental constants.

**What it does:** Define physics laws. Press play. Watch life emerge.

- 40 charged particles → 21 stable molecules in 7ms (Coulomb + Lennard-Jones + frequency alignment)
- Variable-length genomes grow from 4 to 12 genes over 100 generations
- Chemotherapy resistance emerges from the same equations that form molecular bonds
- 14 orthogonal ECS layers (Bevy 0.15), 6-phase simulation pipeline
- All ~40 lifecycle constants algebraically derived from 4 numbers

**Quick start:**
```bash
cargo run --release --bin particle_lab    # emergent molecules (7ms)
cargo run --release --bin cancer_therapy  # tumor dynamics
cargo run --release --bin evolve          # genome evolution
cargo test                                # 2,994 tests
```

**The drug model is the part I'm most proud of:** A drug doesn't "kill cells" — it increases the dissipation rate (Second Law). Cells with frequencies far from the drug's target survive. Resistance isn't programmed; it's a statistical consequence of oscillatory heterogeneity. Results match Bozic et al. 2013 (eLife).

Paper: [ZENODO_DOI_URL]
Repo: [GITHUB_URL]

Stack: Rust 2024 edition, Bevy 0.15, glam 0.29, rayon. No unsafe. No external RNG crate. Deterministic (same seed = identical f32 bits).

Feedback welcome — especially on the axiomatic approach and whether the derivation chain holds up.
```

---

## 3. Reddit r/compsci

### Title:
```
Emergent Life from Four Constants: an axiomatic simulation engine deriving chemotherapy resistance from first principles
```

### Body:
```
Paper: [ZENODO_DOI_URL]

We present an open-source simulation engine where ~40 lifecycle parameters are algebraically derived from 4 fundamental constants (Kleiber exponent, dissipation rates, coherence bandwidth, density scale).

Key result: the same equations that produce Coulomb-bonded molecules also produce chemotherapy resistance dynamics consistent with Bozic et al. 2013 (eLife). No cell-type-specific parameterization — resistance emerges from oscillatory frequency heterogeneity.

Implementation: 109K LOC Rust, 2,994 tests, deterministic (bit-exact replay), headless batch simulator runs 10^6 world-ticks/sec.

Code: [GITHUB_URL]

Interested in feedback from the computational biology and ALife communities on whether this axiomatic approach has merit for drug resistance modeling.
```

---

## 4. Reddit r/bioinformatics

### Title:
```
Simulating chemotherapy resistance from first principles — no cell-type parameters, results match Bozic 2013
```

### Body:
```
I built a simulation engine (Rust, open-source) where drug resistance emerges from fundamental physics rather than cell-type-specific rules.

**Drug mechanism:** Increases dissipation rate (thermodynamic, not direct cell kill). Cells with frequencies far from drug target frequency experience reduced drug effect via Gaussian alignment:

Δr_d = Potency × exp(-Δf²/2B²) × r_d

**Results:**
- Monotherapy (P=0.31): tumor persists — 100% resistance rate (matches Bozic et al. 2013 prediction for single-agent therapy)
- Dose-response is sigmoidal (Hill equation, emergent — not hardcoded)
- Quiescent stem cells reactivate when tumor burden drops
- Normal tissue regenerates during drug holidays

**What's different:** The same 4 constants that determine molecular bonding also determine drug efficacy. No separate pharmacology module. No resistance genes. Resistance = statistical consequence of frequency diversity in the tumor population.

Paper: [ZENODO_DOI_URL]
Code: [GITHUB_URL]
Try it: `cargo run --release --bin cancer_therapy`

Looking for feedback from computational oncology researchers. Would love to collaborate on clinical validation.
```

---

## 5. Reddit r/artificial (Artificial Life)

### Title:
```
Emergent life from 5 axioms: molecules, genomes, proteins, metabolism, multicellularity, and drug resistance — all from 4 constants
```

### Body:
```
I built Resonance — a simulation engine where 10 levels of biological organization emerge bottom-up from 5 primitive axioms and 4 fundamental constants.

The hierarchy (each level emerges from the previous):

0. Energy fields → 1. Matter states → 2. Particle bonding (Coulomb+LJ)
→ 3. Abiogenesis (coherence > dissipation) → 4. Variable genome (4→32 genes)
→ 5. Genetic code (64 codons → 8 amino acids) → 6. Proto-proteins (HP lattice)
→ 7. Metabolic networks (DAG, Hebbian, competition) → 8. Multicellularity (Union-Find colonies)
→ 9. Social emergence (theory of mind, coalitions)

The 4 constants: Kleiber exponent (0.75), dissipation rates (0.005-0.25), coherence bandwidth (50 Hz), density scale (20.0).

Everything else — all ~40 lifecycle thresholds — is algebraically derived. No tuning. No per-species parameters.

The most surprising result: chemotherapy resistance emerges from the same frequency alignment equation that determines molecular bonding. A drug increases dissipation; cells with mismatched frequencies survive.

Paper: [ZENODO_DOI_URL]
Code: [GITHUB_URL] (109K LOC Rust, 2,994 tests)

Try: `cargo run --release --bin particle_lab` (molecules in 7ms)
```

---

## 6. Twitter/X Thread

```
🧵 I derived chemotherapy resistance from 4 physical constants.

Not a metaphor. The same equations that form molecular bonds also predict why tumors survive monotherapy.

109,000 lines of Rust. 2,994 tests. Zero hardcoded species.

Here's how ↓
```

```
1/ Everything starts with 5 axioms:
- Everything is energy (qe)
- Energy is conserved
- All processes lose energy (2nd Law)
- Interaction decays with distance
- Everything oscillates at a frequency

And 4 constants: Kleiber (0.75), dissipation rates, bandwidth (50 Hz), density scale (20.0).
```

```
2/ From these, ~40 lifecycle thresholds are DERIVED algebraically.

Matter states. Metabolism. Lifespan. Reproduction threshold. All computed, not tuned.

17 unit tests verify the derivation chain.
```

```
3/ Charged particles form molecules:
- Coulomb + Lennard-Jones forces
- Frequency alignment determines bond strength
- 40 particles → 21 bonds, 5 molecule types in 7ms

No bond rules. No templates. Emergent chemistry.
```

```
4/ Life emerges where coherence > dissipation.
Genomes grow from 4 to 12 genes.
Proteins fold on HP lattices.
Metabolic networks rewire via Hebb's rule.
Cells form colonies via Union-Find.

All from the same 4 constants.
```

```
5/ Now the drug:

A drug doesn't "kill cells." It increases dissipation rate:

r'_d = r_d + P × alignment(f_drug, f_cell) × r_d

Cells with frequencies far from the drug's target? Low alignment → they survive.

That's resistance. Not programmed — emergent.
```

```
6/ Result: monotherapy with potency 0.31 → tumor persists (100% resistance).

This matches Bozic et al. 2013 (eLife): single-agent therapy is insufficient.

We got there from 4 constants. No oncology-specific parameters.
```

```
7/ Paper: [ZENODO_DOI_URL]
Code: [GITHUB_URL]

Try it yourself:
cargo run --release --bin particle_lab
cargo run --release --bin cancer_therapy
cargo test  # 2,994 tests

Everything is deterministic. Same seed = same results.

Feedback welcome. Especially from oncologists.
```

---

## Posting Schedule

| Day | Platform | Time (optimal) |
|-----|----------|----------------|
| Day 1 (morning) | HackerNews | 9-10 AM EST (Tue-Thu best) |
| Day 1 (afternoon) | r/rust | After HN post |
| Day 1 (evening) | Twitter thread | After screenshots ready |
| Day 2 | r/compsci + r/artificial | Morning |
| Day 3 | r/bioinformatics | Morning |

Post HN on Tuesday-Thursday 9AM EST for maximum visibility.
Do NOT post on weekends — low traffic.
