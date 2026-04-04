# ADR-016: Multi-Telescope — Quantum-Inspired Hierarchical Speculative Execution

**Status:** Accepted (implemented 2026-04-04)
**Date:** 2026-04-04
**Deciders:** Resonance Development Team
**Context of:** ADR-015 (Temporal Telescope), batch pipeline, geological timescale simulation
**Extends:** ADR-015 (single Anchor + single Telescope → N-level quantum-inspired hierarchy)

## Context

ADR-015 introduced a dual-timeline system: one Anchor (ground truth) and one Telescope (speculative projection). This works for moderate time-skips (K ≈ 16-1024 ticks). Reaching geological timescales (abiogenesis → modernity ≈ 10¹² ticks) requires a fundamentally different model.

The solution operates at four layers simultaneously — from the most abstract to the most concrete:

```
Layer 4: METAPHYSICS     → Structure of temporal hierarchy (Whitehead, Leibniz, Plotinus, Hegel)
Layer 3: QUANTUM PHYSICS → Wave-particle duality, collapse, decoherence (Tonomura, Englert, Zurek)
Layer 2: CLASSICAL PHYSICS → Multigrid, RESPA, renormalization, center manifold (Brandt, Wilson, Haken)
Layer 1: SOFTWARE         → Cache, stateless functions, zero-heap, axiom-preserving ECS (Resonance)
```

Each layer constrains and informs the ones below it. The metaphysics defines the structure; the quantum physics defines the behavior; the classical physics guarantees convergence; the software implements it without breaking axioms.

## Decision

Extend ADR-015 to an N-level **Telescope Stack** where:

- **Unobserved ticks are waves** — superpositions of possible states, not specific states
- **Anchor arrival is measurement** — collapses the wave to one reality, destroys alternatives
- **Re-emanation replaces correction** — higher levels are rebuilt from collapse, not patched
- **Precision converges as anchor approaches** — Englert's D² + V² ≤ 1 governs the uncertainty cone
- **Conservation is guaranteed by the anchor** — the telescope is always disposable

---

## Layer 4: Metaphysical Structure

Five traditions converge on the same hierarchical pattern:

**Whitehead — Nested Actual Occasions:** Each telescope level is a "society" at a coarser temporal grain. Level 0's ticks are atomic actual occasions. Each higher level prehends (incorporates) lower levels as already-integrated data. Each level adds genuine interpretive structure — it is not merely a summary. The concrescence (coming together) at each level produces something new.

**Leibniz — Hierarchy of Monads:** Higher monads perceive more of the universe but at lower resolution (petites perceptions). Level 0 sees 128 entities × all fields. Level N sees the regime class over K^N ticks. The Anchor is the "dominant monad" — it perceives with maximum clarity and calibrates all others. The telescope levels are subordinate monads whose perceptions are confused but total.

**Plotinus — Emanation and Epistrophe:** The critical insight: **emanation does not diminish the source**. Projection flows downward (The One → Nous → Soul → Matter): Anchor → Level 1 → Level 2 → ... Each level is the FULL content of the anchor but in increasingly differentiated form. Correction flows upward (epistrophe): each level yearns to return to its source. The collapse is the return.

**Buddhist Pratītyasamutpāda — Dependent Origination:** The radical challenge: **no single level is fundamental**. All levels co-arise. The anchor depends on the telescope's predictions (to allocate computational resources). The telescope depends on the anchor's corrections (to calibrate). Indra's Net: each level reflects all other levels. This demands bidirectional information flow.

**Hegel — Dialectical Aufhebung:** Each collapse is a synthesis: the telescope's projection is simultaneously cancelled (replaced by anchor truth) and preserved (structural insights captured in NormalizerWeights). The weights encode the accumulated wisdom of all prior thesis-projection / antithesis-correction pairs. The dialectic has no final resting point — each synthesis becomes a new thesis.

---

## Layer 3: Quantum Physics — Wave, Collapse, Decoherence

### The Experimental Foundation

**Tonomura et al. (1989, Hitachi Central Research Laboratory, Tokyo):** Single electrons build up an interference pattern one at a time. 10 electrons: random dots. 70,000 electrons: full wave interference. Each electron passes through BOTH paths simultaneously. Until measurement, there is no "one path."

**Jacques et al. (2007, Institut d'Optique):** Wheeler's delayed-choice experiment realized. The decision to measure which-path is made AFTER the photon passes the slits. The photon never "decided" — the question of path is meaningless until measurement context exists.

**Zurek (2003, quantum Darwinism):** The environment continuously "measures" the system. Decoherence is not a discrete event — it is gradual loss of quantum coherence through environmental interaction. The states that survive are the "fittest" — classical states we observe.

### The Three Quantum Principles

**Principle 1: Superposition — Unobserved ticks are waves, not particles.**

Ticks not yet validated by an anchor exist as a distribution of possible states — not one specific SimWorldFlat. The telescope's projection is the eigenstate with maximum |ψ|² (most probable future), but other futures exist as latent possibilities.

The telescope does NOT project "the future." It projects the **most probable future given current tendencies**. The distinction is load-bearing.

**Principle 2: Collapse — Anchor arrival destroys alternatives.**

When the anchor reaches a projected tick, it collapses the superposition to one reality. The telescope levels ahead of the anchor are DESTROYED and rebuilt from the collapsed state (re-emanation, Plotinus). They are not "corrected" — they cease to exist.

This is cleaner than cascade correction: no residual error accumulates because each re-emanation starts fresh from ground truth. The DiffReport becomes a post-hoc measurement of projection quality (learning signal), not a correction vector.

**Principle 3: Decoherence — Precision converges as anchor approaches.**

The Englert duality relation (1996) governs the uncertainty:

```
D² + V² ≤ 1

D = distinguishability (information from anchor, confidence)
V = visibility (speculative coherence, uncertainty)
```

Mapped to the telescope stack:

```
D(level) = e^{-ticks_to_anchor / coherence_length}
V(level) = sqrt(1 - D²)

Level 0 (anchor):     D=1.0, V=0.0  → certainty, zero speculation
Level 1 (K=16):       D≈0.97, V≈0.24 → mostly certain
Level 2 (K=256):      D≈0.71, V≈0.71 → equal certainty and speculation
Level 5 (K=10⁶):     D≈0.001, V≈1.0 → pure wave, trend only
Level 8 (K=4×10⁹):   D≈0, V≈1.0 → onda pura, tendencia de régimen
```

The coherence length depends on regime stability:

```
coherence_length = f(H, ρ₁, λ_max)
  Stasis: coherence_length large → can see far with confidence
  Transition: coherence_length small → only near anchor is reliable
```

### The Quantum Zeno Effect (Anchoring Frequency Limit)

University of Tokyo demonstrated: frequent measurement prevents quantum evolution. Applied: if K=1 (anchor every tick), the telescope never speculates — maximum precision, zero acceleration. The optimal K balances the Zeno limit (too frequent → no speedup) with decoherence (too infrequent → no precision).

---

## Layer 2: Classical Physics — Convergence Guarantees

### Why Error Converges (Not Explodes)

Six independent frameworks predict convergence:

**1. Multigrid (Brandt 1977):** Error converges geometrically: ‖e_k‖ ≤ ρ^k × ‖e₀‖, ρ ≈ 0.1-0.2 per V-cycle. High-frequency error eliminated by smoothing; low-frequency by coarse-grid correction.

**2. RESPA (Tuckerman et al. 1992):** Symplectic multi-timescale integration. Conservation error bounded: |H(t) - H_shadow| = O(dt^p). Does NOT grow over time. Maps to Axiom 5.

**3. Renormalization Group (Wilson 1971):** Coarse-graining eliminates irrelevant operators (entity fluctuations) exponentially. Only relevant operators (regime metrics) survive at coarse scales.

**4. Parareal (Lions, Maday, Turinici 2001):** Cheap propagator G (telescope) + expensive propagator F (anchor) in predictor-corrector loop. Superlinear convergence on bounded intervals.

**5. Center Manifold (Haken 1983):** Near equilibrium, dynamics collapses onto low-dimensional manifold. Off-manifold perturbations decay as e^{-γt}. The telescope's projection space IS the center manifold.

**6. Born-Oppenheimer (1927):** Adiabatic separation of timescales. Error ∝ (m_fast/M_slow)^{1/4}. Breaks down at conical intersections (= regime transitions in our system).

### The Error Formula

```
E_total(k) = E_projection × ρ^k           (calibration: geometric convergence)
           + E_off_manifold × e^{-γ×K}     (center manifold: exponential decay)
           + E_conservation × K × diss_rate (physics: linear, bounded by Axiom 4)
```

With re-emanation (collapse + rebuild instead of correction):
- First term becomes **zero after each collapse** (fresh re-emanation, no residual)
- Second term stays (physics)
- Third term stays (physics)

**Re-emanation eliminates error accumulation between levels.**

---

## Layer 1: Software Architecture — Cache-Powered, Axiom-Preserving

### Axiom Compliance

| Axiom | Status | Mechanism |
|-------|--------|-----------|
| 1 (Everything is Energy) | ✅ | All levels manipulate qe only |
| 2 (Pool Invariant) | ✅ | Telescope doesn't spawn; anchor enforces per-tick |
| 3 (Competition) | ✅ | Anchor runs full interference (Axiom 8); telescope approximates via frequency-aware decay |
| 4 (Dissipation) | ✅ | Conservation-bounded projection: clamp(base_decay, current_qe). Anchor enforces exactly. |
| 5 (Conservation) | ✅ | After collapse, world = anchor (full simulation, conserved). Telescope disposable. |
| 6 (Emergence) | ✅ | Telescope doesn't program behavior; projects tendencies. Emergence comes from anchor. |
| 7 (Distance Attenuation) | ✅ | Decoherence cone: visibility decays with distance from anchor. |
| 8 (Oscillatory) | ✅ | Frequency-aware decay rate: effective_dissipation × (1 - solar_resonance × efficiency). |

### The Five Precision Strategies

**Strategy 1: Conservation-Bounded Projection (Axiom 4+5)**

```rust
// After project_qe, clamp to prevent energy creation.
let base_decay = dissipation_n_ticks(qe, rate, K);
let projected = project_qe(base_decay, trend, metrics, weights, K);
let final_qe = projected.clamp(base_decay, current_qe);
// Monotonically decreasing through levels. Never exceeds input.
```

**Strategy 2: Frequency-Aware Decay Rate (Axiom 8)**

```rust
// Modulate dissipation by solar resonance — O(1) per entity.
let resonance = gaussian_frequency_alignment(entity_freq, SOLAR_FREQUENCY, SOLAR_BANDWIDTH);
let effective_dissipation = entity.dissipation * (1.0 - resonance * PHOTOSYNTHESIS_EFFICIENCY);
// Resonant entities decay less (subsidized by photosynthesis).
// Disonant entities decay more.
```

**Strategy 3: Collapse + Re-Emanation (replaces cascade correction)**

```rust
// When anchor arrives at tick T:
// DESTROY all telescope levels (they are waves that collapsed).
// RE-EMANATE from anchor truth:
for level in 0..active_levels {
    let source = if level == 0 { &anchor } else { &stack[level-1].world };
    stack[level].world = project_world(source, &metrics, &weights, stack[level].k);
}
// Zero error accumulation — each level starts fresh.
```

**Strategy 4: Uncertainty Cone (Englert D²+V²≤1)**

```rust
// Each level knows its speculative visibility.
pub fn speculative_visibility(ticks_to_anchor: u64, coherence_length: f32) -> f32 {
    let d = (-ticks_to_anchor as f32 / coherence_length).exp();
    (1.0 - d * d).sqrt().clamp(0.0, 1.0)
}
// V ≈ 1: pure wave (trend only, no detail). V ≈ 0: collapsed (full detail).
// The user sees the center of the cone (most probable state).
```

**Strategy 5: Event Signature Propagation (discrete event tracking)**

```rust
// Hash of structural events (births, deaths, transitions) per tramo.
pub fn event_signature(births: u16, deaths: u16, transitions: u16) -> u64 {
    hash_f32_slice(&[births as f32, deaths as f32, transitions as f32])
}
// If anchor signature ≠ telescope signature → structural change → force re-emanation.
// If signatures match → only numerical drift → lighter correction.
```

### Cache Architecture

The system is powered by cache at every level:

| Cache | What it stores | Invalidation | Layer |
|-------|---------------|-------------|-------|
| `KleiberCache` | vol_factor per entity | radius change | Entity-level (existing) |
| `GompertzCache` | death_tick per entity | never (sealed at birth) | Entity-level (existing) |
| `BridgeCache` | normalized equation outputs | LRU eviction | Equation-level (existing) |
| `Converged<T>` | env_hash when stable | environment change | Convergence (existing) |
| `NormalizerWeights` | calibrated projection weights | each reconciliation | Telescope-level (ADR-015) |
| `ReconciliationHistory` | last 256 reconciliation records | ring buffer overflow | Learning-level (ADR-015) |
| `TelescopeStack` | N projected worlds + states | **collapse** (re-emanation) | Multi-level (NEW) |

The collapse event is the **universal invalidation signal**: when the anchor arrives, ALL telescope caches above the collapse point are invalidated and rebuilt. This is the quantum measurement — it destroys the cached wave function and forces fresh computation from ground truth.

### Implementation

```rust
pub const MAX_LEVELS: usize = 8;  // 16⁸ ≈ 4.3 × 10⁹ ticks

/// Nivel del telescopio. Contiene estado proyectado + incertidumbre.
/// Telescope level. Contains projected state + uncertainty.
pub struct TelescopeLevel {
    pub state: TelescopeState,
    pub projected_world: SimWorldFlat,
    pub k: u32,
    pub visibility: f32,           // V from Englert: 0=collapsed, 1=pure wave
}

/// Stack de telescopios. Array fijo, zero-heap.
/// Telescope stack. Fixed array, zero-heap.
pub struct TelescopeStack {
    pub levels: [TelescopeLevel; MAX_LEVELS],
    pub active_levels: u8,
    pub coherence_length: f32,     // regime-dependent, from metrics
}
```

### Collapse Algorithm (replaces bubble-up)

```
fn collapse_and_emanate(stack, anchor, metrics, scratch):
  // 1. COLLAPSE: anchor is the measurement. Wave function dies.
  stack.levels[0].projected_world = anchor.clone()
  stack.levels[0].visibility = 0.0  // fully collapsed

  // 2. CALIBRATE: learn from the collapsed wave
  for level in 0..stack.active_levels:
    let diff = world_diff(&old_projected[level], &new_truth_at_level)
    let record = ReconciliationRecord { diff.class, ... }
    stack.levels[level].state.weights = calibrate(&record, &weights, &history)
    history.push(record)

  // 3. RE-EMANATE: rebuild all levels from ground truth
  for level in 1..stack.active_levels:
    let source = &stack.levels[level-1].projected_world
    let k = optimal_k(&metrics, &stack.levels[level].state.weights, K_MIN, K_MAX)
    stack.levels[level].projected_world = project_world(source, &metrics, &weights, k)
    stack.levels[level].k = k
    // Uncertainty grows with distance from anchor
    let ticks_to_anchor = (0..=level).map(|l| stack.levels[l].k as u64).product()
    stack.levels[level].visibility = speculative_visibility(ticks_to_anchor, stack.coherence_length)

  // 4. ADAPTIVE LEVELS: grow or shrink stack
  if should_add_level(stack, metrics) { stack.active_levels += 1 }
  if should_remove_level(stack) { stack.active_levels -= 1 }
```

### Why Re-Emanation is Better Than Cascade Correction

| Aspect | Cascade (ADR-016 v1) | Re-Emanation (quantum) |
|--------|---------------------|----------------------|
| Error between levels | Accumulates (residual from patch) | **Zero** (fresh projection each time) |
| Complexity | O(N × affected_entities) | O(N × projection_cost) |
| Axiom 4 compliance | Cascade can redistribute energy incorrectly | **Always compliant** (project_world uses base_decay) |
| Determinism | Cascade order affects result | **Order-independent** (each level only depends on level below) |
| Cache invalidation | Partial (only patched entities) | **Total** (all levels rebuilt) |
| Conceptual clarity | "Patch the wave" | **"Collapse the wave, re-emanate fresh"** |

---

## Consequences

### Positive
- **Exponential reach, zero accumulated error:** K^N ticks reachable, each collapse resets error to zero
- **Geological timescales:** 8 levels × K=16 = 4.3 billion ticks — abiogenesis to modernity
- **Axiom-preserving:** Conservation guaranteed by anchor; telescope is always disposable
- **Self-improving:** Calibration weights learn from each collapse (Hegel's dialectical accumulation)
- **Physically grounded at four layers:** metaphysics, quantum, classical, software all converge
- **Cache-powered:** Every level leverages existing cache infrastructure (KleiberCache, GompertzCache, BridgeCache, Converged<T>)
- **Uncertainty quantified:** Englert's V tells you exactly how much to trust each level

### Negative
- 8 × SimWorldFlat ≈ 800KB memory (acceptable)
- Re-emanation rebuilds ALL levels on each collapse (O(N × project_world cost) — but project_world is O(entities), not O(ticks))
- Conceptual complexity: four-layer theoretical framework

### Risks

**RISK 1: Re-emanation cost**

Each collapse rebuilds N levels. project_world is O(128 entities) ≈ 10μs per level. 8 levels = 80μs. The anchor tick (33 systems) is ≈1ms. Re-emanation is <10% of anchor cost. Acceptable.

**RISK 2: Coherence length calibration**

The coherence_length determines how fast visibility decays. Too short → all levels are V≈1 (useless). Too long → false confidence in distant projections. Mitigation: derive from regime metrics (H, ρ₁, λ_max). The calibration bridge learns the correct coherence_length from reconciliation history.

**RISK 3: Quantum Zeno — anchoring too frequently**

If K_MIN is too small, the system spends all time collapsing and re-emanating, never speculating. Mitigation: K_MIN = 4 (existing). The Zeno effect is already handled by optimal_k.

## Files to Modify

| File | Action | Why |
|------|--------|-----|
| `src/batch/telescope/stack.rs` | **Create** | TelescopeStack, TelescopeLevel, collapse_and_emanate, speculative_visibility, adaptive level management |
| `src/batch/telescope/mod.rs` | **Modify** | Add `pub mod stack;` |
| `src/blueprint/constants/temporal_telescope.rs` | **Modify** | Add MAX_LEVELS, DEFAULT_COHERENCE_LENGTH |
| `src/blueprint/equations/temporal_telescope.rs` | **Modify** | Add speculative_visibility(), conservation_bounded_project(), frequency_aware_decay_rate() |
| `src/batch/telescope/projection.rs` | **Modify** | Apply conservation clamp + frequency-aware decay in project_entity |

## Files NOT Modified

All of ADR-015's core is reused unchanged:
- `diff.rs` — world_diff, DiffReport (used for learning signal, not correction)
- `calibration_bridge.rs` — calibrate (same learning mechanism)
- `pipeline.rs` — tick_telescope_sync (single-level mode preserved for active_levels=1)
- `activation.rs` — dashboard wiring
- `cascade.rs` — **no longer used for inter-level correction** (kept for intra-level use if needed)

## Success Criteria

1. **Reach:** 8 levels × K=16 reaches 4.3 × 10⁹ ticks
2. **Zero accumulated error:** Each collapse resets projection error (re-emanation, not patching)
3. **Conservation:** Axiom 5 holds — no level ever creates energy. Anchor truth always wins.
4. **Uncertainty quantified:** Each level reports V (Englert visibility). V decreases as anchor approaches.
5. **Axiom 8 compliance:** Frequency-aware decay captures solar resonance in projection
6. **Adaptive levels:** System grows during stasis, shrinks during transitions
7. **ADR-015 compatibility:** With active_levels=1, behavior is identical to ADR-015
8. **Cache coherence:** Collapse invalidates all telescope caches; rebuild is deterministic

## Alternatives Considered

| Alternative | Why Rejected |
|------------|-------------|
| Cascade correction between levels (ADR-016 v1) | Error accumulates across levels. Re-emanation is zero-accumulation by construction. |
| Single telescope with K=10⁹ | Projection over 10⁹ ticks is pure noise. Hierarchy bounds error per level. |
| Many-Worlds (keep all branches) | Memory ∝ branches^N. Exponential in levels. Collapse (discard alternatives) is necessary. |
| Probabilistic sampling (true quantum) | Loses determinism. The anchor IS deterministic — the quantum model is structural, not literal. |

## Scientific References

### Quantum Physics
1. Tonomura, A. et al. (1989). "Demonstration of single-electron buildup of an interference pattern." *Am. J. Phys.*, 57(2), 117-120.
2. Jacques, V. et al. (2007). "Experimental realization of Wheeler's delayed-choice gedanken experiment." *Science*, 315, 966-968.
3. Kim, Y.-H. et al. (1999). "A delayed choice quantum eraser." *Phys. Rev. Lett.*, 84, 1-5.
4. Zurek, W.H. (2003). "Decoherence, einselection, and the quantum origins of the classical." *Rev. Mod. Phys.*, 75, 715.
5. Englert, B.-G. (1996). "Fringe visibility and which-way information." *Phys. Rev. Lett.*, 77, 2154.

### Classical Physics
6. Brandt, A. (1977). "Multi-level adaptive solutions to boundary-value problems." *Math. Comp.*, 31(138), 333-390.
7. Tuckerman, M. et al. (1992). "Reversible multiple time scale molecular dynamics." *J. Chem. Phys.*, 97(3), 1990-2001.
8. Wilson, K.G. (1971). "Renormalization group and critical phenomena." *Phys. Rev. B*, 4(9), 3174.
9. Lions, J.-L. et al. (2001). "A 'parareal in time' discretization of PDE's." *C.R. Acad. Sci. Paris*, 332(7), 661-668.
10. Haken, H. (1983). *Synergetics: Introduction and Advanced Topics.* Springer.
11. Born, M. & Oppenheimer, R. (1927). "Zur Quantentheorie der Molekeln." *Annalen der Physik*, 389(20), 457-484.
12. Feynman, R.P. & Hibbs, A.R. (1965). *Quantum Mechanics and Path Integrals.* McGraw-Hill.
13. Bejan, A. (1996). "Constructal-theory network of conducting paths." *Int. J. Heat Mass Transfer*, 40(4), 799-816.

### Philosophy & Metaphysics
14. Whitehead, A.N. (1929). *Process and Reality.* — Nested actual occasions, concrescence, prehension.
15. Leibniz, G.W. (1714). *Monadology.* — Hierarchy of monads, petites perceptions, dominant monad.
16. Plotinus (c. 270 CE). *Enneads.* — Emanation (One→Nous→Soul→Matter), epistrophe (return).
17. Nāgārjuna (c. 150 CE). *Mūlamadhyamakakārikā.* — Pratītyasamutpāda, no privileged level.
18. Hegel, G.W.F. (1807). *Phänomenologie des Geistes.* — Dialectical Aufhebung.
19. Buddhist Abhidhamma. — Anicca (impermanence), santati (continuity), Indra's Net.

### Information Theory
20. Shannon, C.E. (1959). "Coding theorems for a discrete source with a fidelity criterion." *IRE Nat. Conv. Rec.*, 7(4), 142-163.
21. Mallat, S. (1989). "A theory for multiresolution signal decomposition." *IEEE Trans. PAMI*, 11(7), 674-693.

## Codebase References

- ADR-015: `docs/arquitectura/ADR/ADR-015-temporal-telescope.md`
- `src/batch/telescope/` — 8 modules from ADR-015 (all reusable)
- `src/blueprint/equations/temporal_telescope.rs` — pure math (47 tests)
- `src/blueprint/constants/temporal_telescope.rs` — derived constants
- `src/batch/arena.rs` — SimWorldFlat (Clone, ~100KB)
- `src/blueprint/equations/macro_analytics.rs` — exponential_decay (never exceeds input)
- `src/blueprint/equations/batch_stepping.rs` — dissipation_n_ticks, neighbors_within_radius
- `src/blueprint/equations/determinism.rs` — gaussian_frequency_alignment (for frequency-aware decay)
- `src/bridge/cache.rs` — BridgeCache (LRU, transient, per-timeline instances)
- `src/layers/kleiber_cache.rs`, `gompertz_cache.rs` — entity-level caches (survive across ticks)
- `src/layers/converged.rs` — Converged<T> with env_hash (invalidated by collapse)
