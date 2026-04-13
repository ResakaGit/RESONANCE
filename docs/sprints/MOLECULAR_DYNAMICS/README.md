# Track: MOLECULAR_DYNAMICS — From Emergent Particles to Protein Folding

Road from the current batch particle simulator to a thermodynamically correct
molecular dynamics engine capable of simulating protein folding via coarse-grained
(Go) models, leveraging Resonance's unique frequency-modulated interactions (Axiom 8).

**Non-goal:** Compete with GROMACS/LAMMPS on all-atom accuracy.
**Goal:** A correct MD engine where folding tendency emerges from axiom-derived
interactions — validatable against known thermodynamic properties and folding models.

**Invariant:** All MD additions MUST derive from the 8 axioms. No ad-hoc force terms.

---

## Current State (baseline)

| What exists | Where | Limitation |
|-------------|-------|------------|
| Coulomb + LJ forces | `blueprint/equations/coulomb.rs` (26 tests) | Correct but only pairwise, no cutoff |
| Force accumulation | `batch/systems/particle_forces.rs` | O(N^2), N<=128 |
| Position integration | `batch/systems/atomic.rs:39` | **Euler** (O(dt) error, energy drift) |
| Energy conservation | `batch/pipeline.rs` tests | Axiom 5 enforced, but no Hamiltonian tracking |
| EntitySlot | `batch/arena.rs` | **2D**, **f32**, 28 fields, repr(C) |
| Batch parallel | `batch/batch.rs` | rayon par_iter, millions of worlds |
| Frequency coherence | Axiom 8 everywhere | **Unique** — not in classical MD |

## Architecture Decision

**Extend `batch/`, don't create a parallel engine.**

The batch simulator already has: SoA layout, rayon parallelism, stateless systems,
deterministic hashing, conservation enforcement. Adding MD capabilities means
adding systems and extending EntitySlot — not building from scratch.

**Coordinate upgrade path:** 2D f32 -> 3D f64 is a phased migration, not a flag day.
Early sprints work in 2D f32, later sprints extend dimensions and precision.

---

## Phase 0: Thermodynamic Foundation (Tier 1) — COMPLETE

**Goal:** Make the existing simulator thermodynamically correct.

| Sprint | Name | Status | Deliverable |
|--------|------|--------|-------------|
| [MD-0](SPRINT_MD0_VERLET.md) | Velocity Verlet | **done** | `equations/verlet.rs` (2D + 3D), energy drift < 1e-4 |
| [MD-1](SPRINT_MD1_THERMOSTAT.md) | Langevin Thermostat | **done** | `equations/thermostat.rs`, `batch/systems/thermostat.rs` |
| [MD-2](SPRINT_MD2_PBC.md) | Periodic Boundaries | **done** | `equations/pbc.rs` (2D + 3D f64), `SimWorldFlat.sim_box` |
| [MD-3](SPRINT_MD3_NEIGHBOR_LIST.md) | Neighbor Lists | **done** | `batch/neighbor_list.rs` (CellList + CellList3D) |
| [MD-4](SPRINT_MD4_LJ_VALIDATION.md) | LJ Fluid Validation | **done** | `bin/lj_fluid.rs`, `equations/md_observables.rs` |

**Milestone:** `cargo run --bin lj_fluid` reproduces known thermodynamic properties.

## Phase 1: Molecular Architecture (Tier 2) — COMPLETE

**Goal:** Support bonded interactions and real molecules.

| Sprint | Name | Status | Deliverable |
|--------|------|--------|-------------|
| [MD-5](SPRINT_MD5_BONDED.md) | Bonded Potentials | **done** | `equations/bonded.rs` (2D + 3D), bonds/angles/dihedrals |
| [MD-6](SPRINT_MD6_TOPOLOGY.md) | Molecular Topology | **done** | `batch/topology.rs`, angle/dihedral inference |
| [MD-7](SPRINT_MD7_3D_F64.md) | 3D + f64 Upgrade | **done** | `verlet.rs` 3D, `arena.rs` feature-gated, pbc/neighbor 3D |
| [MD-8](SPRINT_MD8_CUTOFF.md) | Cutoff + Shifted LJ | **done** | Force-shifted LJ in `md_observables.rs` |
| [MD-9](SPRINT_MD9_PEPTIDE.md) | Peptide in Vacuum | **done** | `bin/peptide_vacuum.rs`, alpha basin confirmed |

**Milestone:** Alanine dipeptide samples correct phi/psi (alpha-helix basin).

## Phase 2: Solvation & Force Fields (Tier 2b) — COMPLETE

**Goal:** Simulate molecules in explicit solvent.

| Sprint | Name | Status | Deliverable |
|--------|------|--------|-------------|
| [MD-10](SPRINT_MD10_WATER.md) | Water Model (TIP3P) | **done** | `batch/ff/water.rs` (12 tests), `RdfAccumulator3D` |
| [MD-11](SPRINT_MD11_SHAKE.md) | SHAKE Constraint | **done** | `equations/constraints.rs` (7 tests), SHAKE+RATTLE |
| [MD-12](SPRINT_MD12_EWALD.md) | Ewald Summation | **done** | `equations/ewald.rs` (14 tests), NaCl Madelung validated |
| [MD-13](SPRINT_MD13_FF_LOADER.md) | Force Field Loader | **done** | `batch/ff/mod.rs` + `amber.rs` (20 tests) |
| [MD-14](SPRINT_MD14_SOLVATION.md) | Solvated Peptide | **done** | `bin/peptide_solvated.rs`, `experiments/peptide_solvated.rs` (8 tests) |

**Milestone:** Solvated dipeptide with Ewald + SHAKE stable. O-O RDF accumulated.

## Phase 3: Folding Engine (Tier 3) — COMPLETE

**Goal:** Fold proteins using coarse-grained models with Resonance's frequency modulation.

| Sprint | Name | Status | Deliverable |
|--------|------|--------|-------------|
| [MD-15](SPRINT_MD15_GO_MODEL.md) | Go Model + Axiom 8 | **done** | Residue-level CG, native contacts via frequency alignment |
| [MD-16](SPRINT_MD16_REMD.md) | Replica Exchange MD | **done** | N replicas at different T, swap by Metropolis |
| [MD-17](SPRINT_MD17_FOLD_VALIDATE.md) | Folding Validation | **done** | Fold villin headpiece (35 res), compare RMSD vs. PDB |
| [MD-18](SPRINT_MD18_ANALYSIS.md) | Analysis Suite | **done** | RMSD, Rg, contact maps, PMF |
| [MD-19](SPRINT_MD19_GPU.md) | Parallel Forces | **done** | Rayon-parallel Go forces, serial fallback N<64 |

**Milestone:** Go-model villin headpiece folds to within 5A RMSD of native structure.

---

## Dependency Chain

```
MD-0 (Verlet) ─────┬──→ MD-1 (Thermostat)
                    ├──→ MD-2 (PBC) ──→ MD-3 (Neighbors)
                    │                         │
                    └──────────────────┬──────┘
                                       ↓
                                  MD-4 (LJ Validation)
                                       │
                    ┌──────────────────┬┘
                    ↓                  ↓
              MD-5 (Bonded)      MD-8 (Cutoff)
                    │
              ┌─────┼──────┐
              ↓     ↓      ↓
         MD-6    MD-7    MD-9 (Peptide)
        (Topo)  (3D/f64)    │
              │     │        │
              ↓     ↓        ↓
        MD-10 (Water)   MD-15 (Go Model) ──→ MD-16 (REMD) ──→ MD-17 (Fold)
              │                                                     │
        MD-11 (SHAKE)                                         MD-18 (Analysis)
              │
        MD-12 (Ewald) ──→ MD-13 (FF Loader) ──→ MD-14 (Solvated)
```

## File Architecture

```
src/blueprint/
├── equations/
│   ├── coulomb.rs              ← EXISTS: Coulomb + LJ (extend with cutoff)
│   ├── verlet.rs               ← MD-0: velocity_verlet_step, leapfrog_step
│   ├── thermostat.rs           ← MD-1: langevin_kick, berendsen_rescale, maxwell_boltzmann
│   ├── bonded.rs               ← MD-5: harmonic_bond, harmonic_angle, dihedral_proper/improper
│   ├── ewald.rs                ← MD-12: real_space, reciprocal_space, self_correction
│   ├── constraints.rs          ← MD-11: shake_iteration, rattle_velocity
│   ├── go_model.rs             ← MD-15: native_contact_energy, frequency_modulated_contact
│   └── md_analysis.rs          ← MD-18: rmsd, radius_of_gyration, rdf, contact_map
├── constants/
│   └── molecular_dynamics.rs   ← MD-0+: k_B derived from fundamentals, LJ cutoff, etc.
src/batch/
├── arena.rs                    ← MD-7: EntitySlot position/velocity → [f64; 3]
├── topology.rs                 ← MD-6: BondGraph, ResidueTable, AtomType mapping
├── pbc.rs                      ← MD-2: Box dimensions, wrap, minimum_image
├── neighbor_list.rs            ← MD-3: CellList, VerletList, rebuild trigger
├── systems/
│   ├── atomic.rs               ← MD-0: movement_integrate → velocity_verlet
│   ├── particle_forces.rs      ← MD-3/8: add cutoff, use neighbor list
│   ├── bonded_forces.rs        ← MD-5: bond/angle/dihedral force accumulation
│   ├── thermostat.rs           ← MD-1: Langevin friction + random kick
│   ├── constraints.rs          ← MD-11: SHAKE post-integration correction
│   └── remd.rs                 ← MD-16: replica exchange swap logic
├── ff/                         ← MD-13: force field parameter loading
│   ├── mod.rs
│   ├── amber.rs
│   └── opls.rs
src/bin/
├── lj_fluid.rs                 ← MD-4: LJ fluid validation binary
├── peptide_vacuum.rs           ← MD-9: peptide simulation
└── fold_go.rs                  ← MD-17: Go-model folding binary
```

---

## Axiom Mapping

Every MD addition must trace back to an axiom:

| MD Feature | Axiom | Derivation |
|------------|-------|------------|
| Velocity Verlet | 1, 4 | Energy is the only quantity; dissipation bounds drift |
| Thermostat | 4 | 2nd Law: system tends to thermal equilibrium via dissipation |
| PBC | 7 | Distance attenuation requires a finite domain |
| Neighbor list | 7 | Interactions decay — beyond r_cut, contribution < epsilon |
| Bonded potentials | 8 | Harmonic = small-amplitude limit of oscillatory interaction |
| Ewald summation | 7, 8 | Long-range Coulomb respects distance attenuation in periodic domain |
| SHAKE | 2 | Pool invariant: constrained DOF don't create energy |
| Go model | 8 | **Native contacts encoded as frequency alignment** — original |
| REMD | 4 | Multiple dissipation rates sample the free energy landscape |
| Water model | 1, 4 | Water is energy (qe) with specific dissipation properties |

### The Axiom 8 Differentiator (Go Model)

Classical Go models use a contact map: if residues i,j are in contact in the native
structure, they attract. The interaction is binary (contact/no contact) with a LJ-like
potential depth.

**Resonance Go model:** Each residue has a frequency (from its amino acid type or
position in sequence). Native contacts are frequency-coherent pairs:

```
E_contact(i,j) = epsilon_native * alignment(f_i, f_j) * LJ(r_ij, sigma_ij)

where alignment = exp(-0.5 * ((f_i - f_j) / COHERENCE_BANDWIDTH)^2)
```

Non-native contacts have random frequencies → low alignment → weak attraction.
This means:
- Folding is frequency-selective (Axiom 8)
- Misfolded states have low coherence (detectable, not just high energy)
- Mutation = frequency shift → quantifiable effect on folding stability
- Drug binding (existing pathway_inhibitor) extends naturally to folding context

This is **not** how Go models work in the literature. It's original.

---

## Risk Register

### R1. f32 Precision Drift (Phase 0, critical)

**Problem:** MD accumulates numerical error over millions of steps. f32 has ~7 decimal
digits. Velocity Verlet helps (symplectic → bounded drift) but long trajectories will
lose precision in absolute coordinates.

**Strategy:**
- Phase 0-1: Stay in f32. Validate drift is < 0.01% over 10K ticks with Verlet.
- If drift exceeds threshold: use **coordinate centering** (subtract center of mass
  each N steps) to keep absolute values small. This is standard in production MD.
- Phase 1 (MD-7): Full migration to f64 for positions/velocities/forces.
  EntitySlot grows but cache line pressure stays manageable (~400 bytes/entity in 3D f64).

**Mitigation test:** `verlet_energy_drift_10k_steps` — run 10K ticks, measure
(E_final - E_initial) / E_initial. MUST be < 1e-4.

### R2. Thermostat Destroys Conservation (Phase 0)

**Problem:** A thermostat injects/removes energy. Axiom 5 says total qe decreases.
How do we reconcile?

**Strategy:** The thermostat is the "environment" — an implicit infinite heat bath.
Axiom 5 applies to the **closed** system (sim + bath). The sim alone can gain energy
from the bath. Document this as an axiom extension: the simulation box is an **open
subsystem** when a thermostat is active.

**Implementation:** Langevin thermostat adds friction (−gamma*v, dissipation = Axiom 4)
and random kicks (thermal fluctuation). The friction term satisfies Axiom 4 directly.
The random term is the heat bath coupling. Track `qe_injected_by_thermostat` as an
accounting field — conservation becomes: `E_kinetic + E_potential + E_dissipated - E_injected = E_initial`.

**Mitigation test:** Without random kicks (gamma > 0, T_target = 0), system must cool
to zero kinetic energy monotonically (pure dissipation, Axiom 4 strict).

### R3. PBC Minimum Image Breaks Distance Attenuation (Phase 0)

**Problem:** With periodic boundaries, an atom at (0,0) and one at (L-epsilon, 0)
are actually neighbors (distance = epsilon, not L-epsilon). The minimum image
convention handles this, but it means "distance" is no longer Euclidean in the
naive sense. Does this violate Axiom 7?

**Strategy:** No. Axiom 7 says interaction decays with **physical** distance. In a
periodic system, the physical distance IS the minimum image distance. The topology
is a torus, and distance on a torus is well-defined. The axiom holds — just on a
different manifold.

**Implementation:** `minimum_image(dr, box_length) → dr - box_length * round(dr / box_length)`.
Applied in force computation before distance calculation. All existing force functions
(`coulomb_force`, `lennard_jones`) receive the corrected distance — no changes needed
to the equations themselves.

**Mitigation test:** Two particles at opposite edges of the box feel attraction
(minimum image distance < r_cut), not repulsion (raw distance > r_cut).

### R4. Bonded Forces Break O(N^2) Symmetry (Phase 1)

**Problem:** Current forces are all pairwise non-bonded (loop over all pairs).
Bonded forces (bonds, angles, dihedrals) only apply to connected atoms — you
need a connectivity graph, not an N^2 sweep.

**Strategy:** Separate bonded from non-bonded force computation.
- Non-bonded: existing `accumulate_forces` + neighbor list (MD-3)
- Bonded: iterate over topology graph (bond list, angle list, dihedral list)
- Both write to the same force accumulator per atom
- Total force = bonded + non-bonded (superposition principle)

**Implementation:** `Topology { bonds: Vec<(u16, u16, BondParams)>, angles: Vec<(u16, u16, u16, AngleParams)>, ... }` as a `Resource` or field in `SimWorldFlat`. Bonded forces iterate this list, not the N^2 pairs.

**Risk:** Topology must be immutable during force computation (no bond breaking mid-step).
Bond formation/breaking happens in a separate phase (MorphologicalLayer).

### R5. Go Model Frequency Assignment is Arbitrary (Phase 3)

**Problem:** In classical Go models, native contacts come from the PDB structure.
In our frequency-modulated version, each residue needs a frequency. How to assign?

**Strategy (3 options, to be decided in MD-15):**

**Option A: Sequence-derived frequencies.**
Each amino acid type gets a base frequency (20 values). Position in sequence adds
a small offset. Native contacts between residues i,j are coherent because the PDB
structure determines which (freq_i, freq_j) pairs are within COHERENCE_BANDWIDTH.

**Option B: Structure-derived frequencies.**
Run a normal Go model first (binary contacts). Assign frequencies post-hoc such that
native contacts maximize alignment and non-native contacts minimize it. This is
reverse-engineering — less elegant but guaranteed to work.

**Option C: Evolutionary assignment.**
Use the batch genetic harness (already exists) to evolve frequency assignments that
produce correct folding. This is the most "Resonance-native" approach — let the
simulation find its own frequency code.

**Decision deferred to MD-15.** All three are implementable. Option C is the most
interesting for publication but slowest to converge.

### R6. Ewald Summation Complexity (Phase 2)

**Problem:** Ewald splits Coulomb into real-space (short-range, convergent) and
reciprocal-space (long-range, Fourier). The reciprocal part needs FFT over a 3D grid.
This is the most complex single algorithm in the track.

**Strategy:**
- Start with bare Ewald (no PME). Complexity O(N^{3/2}) — acceptable for N < 5000.
- If N > 5000 needed: implement PME (Particle Mesh Ewald) using a 3D grid + FFT.
  This requires an FFT implementation (use `rustfft` crate — needs approval per
  Hard Block #2).
- **Alternative:** For Go models (Phase 3), Coulomb is often not needed (contacts
  are short-range). Ewald may be skippable entirely for the folding milestone.

**Decision gate:** Before starting MD-12, evaluate whether the Phase 3 milestone
requires Ewald or if cutoff Coulomb + reaction field correction suffices.

### R7. GPU Acceleration Scope (Phase 3)

**Problem:** For N > 10K particles, CPU is too slow. GPU compute via wgpu is the
standard path but adds significant complexity (shader code, memory management,
synchronization).

**Strategy:**
- **Not in critical path.** All milestones through MD-17 are achievable on CPU
  with N < 5000 (Go model, small proteins).
- MD-19 is isolated — it accelerates existing algorithms, doesn't add new physics.
- If GPU is needed earlier: profile first, identify the bottleneck (force computation
  is >90% of wall time), port only that kernel.

**Alternative:** Before GPU, try SIMD via `std::simd` (nightly) or `packed_simd`.
Force computation is embarrassingly vectorizable. 4-8x speedup possible without
GPU complexity.

### R8. 2D to 3D Migration Breaks Existing Tests (Phase 1)

**Problem:** EntitySlot uses `[f32; 2]` for position/velocity. All 33 existing
batch systems assume 2D. Moving to 3D touches everything.

**Strategy:** Phased migration with backward compatibility:
1. Add `position_3d: [f64; 3]` and `velocity_3d: [f64; 3]` to EntitySlot alongside
   existing 2D fields. MD systems use 3D; legacy systems use 2D.
2. Bridge function: `fn pos_2d(slot: &EntitySlot) -> [f32; 2]` reads from 3D and
   projects. Legacy systems don't change.
3. When all legacy systems are migrated (or deprecated for MD context), remove 2D
   fields.

**Risk:** EntitySlot grows by 48 bytes (6 * f64). At 128 entities, this is 6KB —
negligible. At 4096 entities (MD-3), it's 192KB — still fine for L2 cache.

**Mitigation:** Feature gate: `#[cfg(feature = "md_3d")]` on the 3D fields. Default
off for existing batch runs, on for MD binaries.

---

## Effort Summary

| Phase | Sprints | Estimated effort | Cumulative |
|-------|---------|-----------------|------------|
| Phase 0: Thermodynamic Foundation | MD-0..4 | 4 weeks | 4 weeks |
| Phase 1: Molecular Architecture | MD-5..9 | 7 weeks | 11 weeks |
| Phase 2: Solvation & Force Fields | MD-10..14 | 8 weeks | 19 weeks |
| Phase 3: Folding Engine | MD-15..18 | 8 weeks | 27 weeks |
| Phase 3b: GPU (optional) | MD-19 | 8 weeks | 35 weeks |

**Total to folding milestone (without GPU): ~27 weeks (~6.5 months).**
**Total with GPU: ~35 weeks (~8.5 months).**

## Constants (Derived from 4 Fundamentals)

All MD constants MUST derive from the 4 fundamentals or be standard physical constants.

| Constant | Derivation | Sprint |
|----------|-----------|--------|
| `K_BOLTZMANN` | Physical constant (1.380649e-23 J/K). Not derived — universal. | MD-1 |
| `LJ_CUTOFF_RATIO` | 2.5 * sigma. Standard choice (error < 1% with tail correction). | MD-8 |
| `NEIGHBOR_SKIN` | 0.3 * sigma. Rebuild when any atom moves > skin/2. | MD-3 |
| `EWALD_ALPHA` | Optimized per box size: alpha = 5.0 / L_box. Standard. | MD-12 |
| `SHAKE_TOLERANCE` | 1e-6. Iterative convergence criterion. | MD-11 |
| `LANGEVIN_GAMMA` | `DISSIPATION_LIQUID * 10`. Friction from Axiom 4 dissipation rate. | MD-1 |
| `GO_CONTACT_EPSILON` | `DISSIPATION_SOLID * 200`. Same as `BOND_ENERGY_THRESHOLD`. | MD-15 |
| `GO_CONTACT_SIGMA` | `1.0 / DENSITY_SCALE * 3.8`. 3.8 A C-alpha distance, normalized. | MD-15 |

---

## Validation Matrix

| Sprint | Validation | Pass criterion |
|--------|-----------|---------------|
| MD-0 | Energy drift (NVE) | < 1e-4 relative over 10K steps |
| MD-1 | Temperature equilibration | <T> = T_target +/- 2% after 5K steps |
| MD-1 | Velocity distribution | Chi-squared test vs. Maxwell-Boltzmann (p > 0.01) |
| MD-2 | Wrap correctness | Particle at x=L+eps appears at x=eps |
| MD-3 | Neighbor list completeness | All pairs within r_cut found (vs. brute force) |
| MD-4 | LJ equation of state | Pressure within 5% of Johnson et al. 1993 at T*=1.0, rho*=0.8 |
| MD-4 | RDF first peak | r_peak / sigma = 1.0 +/- 0.05 |
| MD-5 | Harmonic bond | Bond length oscillates around r_eq with correct frequency |
| MD-9 | Ramachandran | phi/psi populations match AMBER reference (qualitative) |
| MD-10 | Water density | 0.997 +/- 0.01 g/cm^3 at 300K, 1 atm |
| MD-15 | Go native contacts | Folded state has >80% native contacts |
| MD-17 | Villin RMSD | < 5 A from PDB 1VII after REMD |

---

## Decision Gates

Before starting each phase, evaluate whether to proceed or pivot:

**Gate 0 -> 1 (after MD-4):**
- Is LJ fluid thermodynamically correct?
- Is f32 precision sufficient or do we need f64 immediately?
- Does Verlet drift stay bounded?

**Gate 1 -> 2 (after MD-9):**
- Does peptide in vacuum sample correct conformations?
- Is the Go model viable without explicit solvent (skip Phase 2)?
- If Go model works in vacuum, jump directly to Phase 3 (save 8 weeks).

**Gate 2 -> 3 (after MD-14):**
- Is solvated simulation stable?
- Does Ewald converge for our system sizes?
- Is performance acceptable (> 1 ns/day for 5K atoms on CPU)?

**Shortcut path:** MD-0 -> MD-1 -> MD-2 -> MD-3 -> MD-4 -> MD-5 -> MD-9 -> MD-15 -> MD-17.
This skips Phase 2 entirely (no explicit solvent, no Ewald) and uses implicit solvent
or vacuum Go model. **Estimated: 15 weeks instead of 27.**
