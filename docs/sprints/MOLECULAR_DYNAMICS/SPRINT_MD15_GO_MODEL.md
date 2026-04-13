# MD-15: Go Model + Axiom 8 Frequency Modulation

**Effort:** 3 weeks | **Blocked by:** MD-9 | **Blocks:** MD-16, MD-17

**ADR:** [ADR-023 Frequency-Modulated Go Model](../../arquitectura/ADR/ADR-023-frequency-modulated-go-model.md)

## The Original Contribution

Classical Go models (Taketomi, Ueda & Go, 1975) define native contacts from a PDB
structure and assign identical attraction strength to all native pairs. Non-native
pairs only repel.

**Resonance Go model:** Each residue has a characteristic frequency. Native contacts
are frequency-coherent pairs (Axiom 8). Non-native contacts have mismatched frequencies
and thus weaker attraction. This makes:

- Folding = frequency synchronization (not just energy minimization)
- Misfolded states detectable by coherence measurement (not just RMSD)
- Mutations = frequency shifts with quantifiable effects on stability
- Drug binding (pathway_inhibitor) extends naturally to folding context

**This does not exist in the literature.** It is the publishable differentiator.

## Theory

### Classical Go energy

```
E = sum_{native_pairs} epsilon * [5*(sigma/r)^12 - 6*(sigma/r)^10]
  + sum_{non-native}   epsilon_repel * (sigma/r)^12
  + sum_{bonds}        k_bond * (r - r0)^2
  + sum_{angles}       k_angle * (theta - theta0)^2
  + sum_{dihedrals}    k_phi * [1 - cos(phi - phi0)]
```

All native pairs have the same epsilon. The contact map is binary.

### Resonance Go energy

```
E = sum_{native_pairs}  epsilon * alignment(f_i, f_j) * [5*(sigma/r)^12 - 6*(sigma/r)^10]
  + sum_{non-native}    epsilon_repel * (sigma/r)^12
  + bonded terms (same as classical)
```

Where alignment is the Axiom 8 frequency coherence:

```
alignment(f_i, f_j) = exp(-0.5 * ((f_i - f_j) / COHERENCE_BANDWIDTH)^2)
```

- Native contacts with coherent frequencies: alignment ~ 1.0 (full attraction)
- Native contacts with incoherent frequencies: alignment ~ 0.0 (reduced attraction)
- Non-native contacts: only repulsion (no alignment dependence)

This turns the binary contact map into a **graded coherence landscape**.

## Frequency Assignment Strategies

Three options, to be evaluated empirically:

### Strategy A: Amino Acid Type Frequencies

Each of the 20 amino acid types gets a base frequency:

```rust
pub fn amino_acid_frequency(aa: AminoAcid) -> f64 {
    match aa {
        Ala => 100.0,  Arg => 105.0,  Asn => 110.0,  Asp => 115.0,
        Cys => 120.0,  Gln => 125.0,  Glu => 130.0,  Gly =>  95.0,
        His => 135.0,  Ile => 140.0,  Leu => 145.0,  Lys => 150.0,
        Met => 155.0,  Phe => 160.0,  Pro =>  90.0,  Ser => 165.0,
        Thr => 170.0,  Trp => 175.0,  Tyr => 180.0,  Val => 185.0,
    }
}
```

Frequencies are chosen so that residues forming native contacts have frequencies
within COHERENCE_BANDWIDTH (50 Hz). This is a design constraint: the frequency
assignment must be compatible with the native structure.

**Advantage:** Simple, deterministic, amino-acid-specific.
**Risk:** May not produce correct alignment for all native contacts.

### Strategy B: Structure-Derived Optimization

1. Build classical Go model (binary contacts).
2. Optimize frequency assignment to maximize:
   `sum_{native} alignment(f_i, f_j) - sum_{non-native} alignment(f_i, f_j)`
3. Use gradient descent or simulated annealing on frequency vector.

**Advantage:** Guaranteed to produce correct alignments.
**Risk:** Frequencies lose physical meaning; becomes a fitting exercise.

### Strategy C: Evolutionary (Batch Harness)

Use the existing genetic harness (`batch/harness.rs`):
- Genome = frequency assignment for each residue
- Fitness = fraction of native contacts formed at low temperature
- Evolve frequencies that fold the protein

**Advantage:** Most "Resonance-native". Frequencies emerge, not assigned.
**Risk:** Slow convergence. Requires many folding simulations per generation.

### Decision

**Implement all three.** Strategy A as default. B and C as experiments.
The paper can compare all three and show that C (emergent) converges to similar
frequencies as A (designed), which would be a strong result.

## Implementation

### 1. Pure math: `blueprint/equations/go_model.rs`

```rust
/// Go model 10-12 potential for native contact.
/// V = epsilon * [5*(sigma/r)^12 - 6*(sigma/r)^10]
pub fn go_native_potential(r: f64, sigma: f64, epsilon: f64) -> f64

/// Go model force (derivative of potential).
pub fn go_native_force(r: f64, sigma: f64, epsilon: f64) -> f64

/// Frequency-modulated Go potential (Axiom 8).
/// V = epsilon * alignment(f_i, f_j) * go_native_potential(r, sigma)
pub fn go_axiom8_potential(r: f64, sigma: f64, epsilon: f64, f_i: f64, f_j: f64, bandwidth: f64) -> f64

/// Frequency-modulated force.
pub fn go_axiom8_force(r: f64, sigma: f64, epsilon: f64, f_i: f64, f_j: f64, bandwidth: f64) -> f64

/// Contact map from PDB coordinates.
/// Native contact: C-alpha distance < cutoff (typically 8 A) and |i - j| >= 3.
pub fn native_contact_map(ca_positions: &[[f64; 3]], cutoff: f64) -> Vec<(u16, u16, f64)>

/// Amino acid frequency assignment (Strategy A).
pub fn amino_acid_frequency(aa_type: u8) -> f64

/// Optimize frequencies for maximum native coherence (Strategy B).
pub fn optimize_frequencies(
    contact_map: &[(u16, u16, f64)],
    n_residues: usize,
    bandwidth: f64,
    iterations: usize,
) -> Vec<f64>
```

### 2. Topology: coarse-grained residue representation

Each residue = one particle (C-alpha position). This lives in the existing
topology system (MD-6):

```rust
pub struct GoTopology {
    pub n_residues: usize,
    pub sequence: Vec<u8>,            // amino acid types
    pub native_contacts: Vec<(u16, u16, f64)>,  // (i, j, sigma_ij)
    pub frequencies: Vec<f64>,        // per-residue frequency
    pub bond_length: f64,             // C-alpha to C-alpha (~3.8 A)
    pub bond_k: f64,                  // harmonic spring constant
}
```

### 3. Force system: `batch/systems/go_forces.rs`

```rust
pub fn go_model_forces(world: &mut SimWorldFlat, topo: &GoTopology) {
    // Bonded: sequential C-alpha pairs
    for i in 0..topo.n_residues - 1 {
        let force = harmonic_bond_force(r, topo.bond_length, topo.bond_k);
        // apply to i and i+1
    }

    // Native contacts: frequency-modulated Go potential
    for &(i, j, sigma_ij) in &topo.native_contacts {
        let f_i = topo.frequencies[i as usize];
        let f_j = topo.frequencies[j as usize];
        let force = go_axiom8_force(r, sigma_ij, GO_CONTACT_EPSILON, f_i, f_j, COHERENCE_BANDWIDTH);
        // apply to i and j
    }

    // Non-native: repulsion only (no frequency dependence)
    // Use cell list for efficiency
}
```

### 4. PDB loader: `batch/ff/pdb.rs`

Minimal PDB parser for C-alpha coordinates:

```rust
pub struct PdbStructure {
    pub residues: Vec<Residue>,
    pub ca_positions: Vec<[f64; 3]>,
}

pub struct Residue {
    pub name: [u8; 3],    // "ALA", "GLY", etc.
    pub aa_type: u8,       // 0-19
    pub chain: u8,
}

pub fn parse_pdb_ca(pdb_text: &str) -> PdbStructure
```

Only parse ATOM records where atom name = " CA ". Minimal, no full PDB support.

### 5. Coherence observables

New analysis for frequency-modulated folding:

```rust
/// Fraction of native contacts formed (distance < 1.2 * sigma_ij).
pub fn native_contact_fraction(positions: &[[f64; 3]], contacts: &[(u16, u16, f64)]) -> f64

/// Average frequency coherence of formed native contacts.
pub fn folding_coherence(
    positions: &[[f64; 3]],
    contacts: &[(u16, u16, f64)],
    frequencies: &[f64],
    bandwidth: f64,
) -> f64

/// Coherence spectrum: histogram of alignment values for formed contacts.
pub fn coherence_spectrum(
    positions: &[[f64; 3]],
    contacts: &[(u16, u16, f64)],
    frequencies: &[f64],
    bandwidth: f64,
    n_bins: usize,
) -> Vec<f64>
```

These observables are unique to Resonance's Go model and form the basis of the
paper's analysis.

## Validation

| Test | Criterion |
|------|-----------|
| `go_potential_minimum_at_sigma` | V(sigma) < V(r) for r != sigma |
| `go_force_zero_at_sigma` | F(sigma) = 0 (equilibrium) |
| `axiom8_full_alignment_matches_classical` | alignment=1 → same as classical Go |
| `axiom8_zero_alignment_kills_attraction` | alignment=0 → only repulsion |
| `contact_map_symmetric` | (i,j) in map iff (j,i) in map |
| `contact_map_excludes_neighbors` | |i-j| < 3 excluded |
| `frequency_optimization_increases_native_coherence` | coherence_after > coherence_before |
| `native_fraction_100_at_native_structure` | Q = 1.0 when positions = PDB |
| `native_fraction_low_at_random_coil` | Q < 0.3 for random positions |
| `folding_coherence_correlates_with_Q` | Pearson(coherence, Q) > 0.7 |

## Risks

### Frequency assignment doesn't produce folding

**Problem:** If frequencies are assigned such that non-native contacts have
high alignment, the protein misfolds to a frequency-coherent but structurally
wrong state.

**Mitigation:** Strategy B guarantees correct alignment by construction.
If Strategy A fails, fall back to B and analyze why. The failure itself would
be interesting (reveals which sequence positions need frequency constraints).

### Go model too simple for meaningful frequency effects

**Problem:** Classical Go models already fold perfectly. Adding frequency
modulation might not change the result measurably.

**Mitigation:** Measure folding kinetics (time to fold) and thermodynamic
stability (melting temperature). Even if both models fold, the pathways and
stability may differ. Also test with deliberately wrong frequencies — the
classical model still folds, but the frequency model should NOT.

### PDB file format parsing edge cases

**Problem:** PDB format is notoriously messy (multiple models, alternate
conformations, HETATM, insertion codes).

**Mitigation:** Only parse " CA " atoms from ATOM records. Ignore everything
else. Test with 3 known structures: 1VII (villin), 2GB1 (protein G), 1L2Y (Trp-cage).
Include test PDB files in `assets/pdb/`.

## Acceptance Criteria

- [x] `blueprint/equations/go_model.rs` with Go potentials + Axiom 8 modulation (>= 10 tests)
- [x] `batch/ff/pdb.rs` minimal PDB parser
- [x] `batch/systems/go_forces.rs` force computation
- [x] All 3 frequency strategies implemented (A default, B/C experimental)
- [x] Coherence observables (native_fraction, folding_coherence, spectrum)
- [x] Test PDB files in `assets/pdb/` (villin, protein G, Trp-cage)
- [x] Classical Go (alignment=1 constant) folds correctly as baseline
- [x] Frequency-modulated Go with Strategy A folds at least 1 of 3 test proteins
- [x] Wrong frequencies demonstrably impair folding (control experiment)
