# ADR-015: Temporal Telescope — Dual-Timeline Speculative Execution with Reconciliation

**Status:** Accepted (implemented 2026-04-04)
**Date:** 2026-04-04
**Deciders:** Resonance Development Team
**Context of:** Batch pipeline, macro_analytics, GeologicalLOD, tick_fast, SimTimeSeries

## Context

Resonance simulates energy dynamics tick-by-tick. Every tick computes dissipation, metabolism, reproduction, predation, and field propagation for all entities. This is correct but wasteful: natural systems spend most of their time in quasi-equilibrium (stasis), punctuated by brief transitions.

The same pattern repeats at every scale:

| Scale | Stasis | Transition | Ratio |
|-------|--------|------------|-------|
| Stellar evolution | Main sequence (10 Gyr) | Red giant transition (1 Gyr) | 10:1 |
| Glacial cycles | Glacial period (90 kyr) | Deglaciation (10 kyr) | 9:1 |
| Ecosystem succession | Climax forest (centuries) | Disturbance (years) | 100:1 |
| Cell metabolism | Homeostasis (hours) | Stress response (minutes) | 60:1 |
| Earthquakes | Stress accumulation (years) | Rupture (seconds) | 10⁷:1 |

**Self-similarity:** The macro and micro behave identically at different scales. Measurable via the Hurst exponent (H ≈ 0.7-0.9 in natural systems). Trends persist across scales (Hurst 1951, Peng et al. 1994).

**Existing infrastructure in Resonance:**

| Component | File | What it does |
|-----------|------|-------------|
| `tick_fast()` | `batch/pipeline.rs` | O(1) analytical stepping for isolated entities (5-10× speedup) |
| `predict_death_ticks()` | `blueprint/equations/batch_stepping.rs` | Closed-form death time prediction |
| `macro_analytics.rs` | `blueprint/equations/` | Exponential decay + allometric growth solvers |
| `GeologicalLOD` | `simulation/emergence/geological_lod.rs` | Tick compression 1×/10×/100×/1000× (implemented, not active) |
| `MultiscaleSignalGrid` | `simulation/emergence/multiscale.rs` | 3-level spatial aggregation (implemented, not active) |
| `RingBuffer` / `SimTimeSeries` | `runtime_platform/dashboard_bridge.rs` | 512-tick sliding window of qe, population, species |
| `Converged<T>` | `layers/converged.rs` | Generic convergence marker with environment hashing |
| `field_converged` | `batch/systems/internal_field.rs` | Per-entity diffusion convergence (AS-2) |
| `SimWorldFlat` | `batch/arena.rs` | Flat Copy-friendly struct (~100KB), memcpy = instant snapshot |
| `rayon` parallelism | `batch/batch.rs` | N worlds in parallel already (WorldBatch) |
| `BridgeCache` | `bridge/cache.rs` | LRU cache with band normalization, transient and stateless |
| `exact_cache.rs` | `blueprint/equations/` | Zero-loss precompute: kleiber_volume_factor, exact_death_tick, frequency_alignment_exact |
| `KleiberCache` + `GompertzCache` | `layers/` | Per-entity caches valid across ticks |
| `checkpoint_system` | `simulation/` | RON/JSON snapshot save/load |

The pieces exist. They are not connected into a unified framework.

## Decision

Introduce a **Temporal Telescope** — a dual-timeline speculative execution system. Two timelines run simultaneously:

- **Anchor (background):** Full tick-by-tick simulation. The ground truth. Runs in a background thread at full fidelity — no shortcuts.
- **Telescope (foreground):** Analytical projection into the future. Instantaneous. The user sees this immediately. Informed by statistical rules that improve projection accuracy.

When the Anchor catches up, it reconciles with the Telescope. Differences cascade locally. The reconciliation history trains the Telescope to project better over time.

### The Analogy

```
CPU branch prediction:   Predict branch → execute ahead → if correct, free → if wrong, flush
Optimistic concurrency:  Execute transaction → detect conflicts → resolve after
Git branching:           Fork (speculate) → work independently → rebase (reconcile)
This system:             Project future → simulate past in background → diff → cascade corrections
```

### Why Dual-Timeline Eliminates Classical Risks

A single-timeline skip must answer: "Is it safe to skip?" — a prediction problem (fundamentally hard: chaos, CSD, information loss). The dual-timeline converts this to: "What changed?" — a reconciliation problem (fundamentally easy: diff + propagate).

| Classical risk | Why it disappears |
|---|---|
| False stasis | Anchor simulates ALL ticks. If there's a transition, Anchor finds it. |
| Desynchronization | Anchor executes all interactions. Missing ones in Telescope are reconciled. |
| Determinism | Anchor IS the ground truth. Telescope is disposable. |

### Architecture: Two Timelines

```
Tick 1000: Current state S₁₀₀₀
           ┌──────────────────────────────────────────┐
           │ TELESCOPE (foreground, instantaneous)     │
           │                                           │
           │ 1. Read metrics: ρ₁, H, F, λ, E          │
           │ 2. Interpret regime via normalizer rules   │
           │ 3. Project K ticks ahead analytically      │
           │ 4. Result: S̃₁₀₆₄ (speculative state)     │
           │ → User sees S̃₁₀₆₄ immediately             │
           └──────────────────────────────────────────┘
           ┌──────────────────────────────────────────┐
           │ ANCHOR (background thread)                │
           │                                           │
           │ tick 1001 → 1002 → ... → 1064             │
           │ Full simulation: dissipation, metabolism,  │
           │ reproduction, predation, interference      │
           │ Result: S₁₀₆₄ (ground truth)              │
           └──────────────────────────────────────────┘
                              │
                              ▼
           ┌──────────────────────────────────────────┐
           │ RECONCILIATION (when Anchor reaches 1064) │
           │                                           │
           │ 1. DIFF: S₁₀₆₄ vs S̃₁₀₆₄                 │
           │ 2. CLASSIFY: local diff or systemic?      │
           │ 3. CASCADE: correct affected entities     │
           │ 4. LEARN: record (metrics, K, accuracy)   │
           └──────────────────────────────────────────┘
```

### Reconciliation Protocol

```
DIFF:
  For each entity: compare qe, position, frequency, alive/dead
  For grid: compare per-cell accumulated_qe

CLASSIFY:
  PERFECT (diff < 0.5% all entities):
    → No correction needed
    → Record: "with these metrics, K=64 works"
    → Next K = K × 1.5 (more aggressive)

  LOCAL (2-5 entities differ > 2%):
    → Correct those entities + their causal neighborhood
    → Cascade radius = max(interaction_radii) of affected entities
    → Record: "telescope missed [type/region], adjust rule weights"
    → K unchanged

  SYSTEMIC (> 10% entities differ):
    → Replace S̃ with S entirely (full correction)
    → Record: "K=64 too aggressive with these metrics"
    → Next K = K / 2

CASCADE (for LOCAL diffs):
  1. Identify affected entities: those in diff + their neighbors within interaction radius
  2. For each affected entity: recompute state from Anchor's ground truth
  3. Propagate: if corrected entity's qe changed > ε, check ITS neighbors
  4. Stop when corrections are below ε (damped propagation)
  Cost: O(affected × avg_neighbors), NOT O(total_entities)
```

### Rules as Projection Normalizers (Not Go/No-Go Gates)

The statistical rules from classical CSD detection serve a different purpose: they **tune how the Telescope projects**, not whether it projects.

#### Hurst Normalizer (persistence of trends)

```
H = Hurst exponent via DFA over sliding window

H > 0.7 (persistent): qe_proj(t+K) = qe(t) + K × (dqe/dt) × H
  → Extrapolate current trend aggressively. "What's happening will continue."

H ≈ 0.5 (random walk): qe_proj(t+K) = qe(t) + noise
  → Don't extrapolate. "No information about direction."

H < 0.3 (anti-persistent): qe_proj(t+K) = qe(t) - K × (dqe/dt) × (1-H)
  → Reverse trend. "What's happening will reverse."

Infrastructure: SimTimeSeries.qe_history (RingBuffer, 512 entries, already tracked per tick)
```

#### Fisher Normalizer (distributional sensitivity)

```
F(t) = Σ_cells (1/p_c) × (Δp_c/Δt)²    where p_c = qe_c / Σqe

F low and stable:
  → Project constant energy distribution
  → Entities maintain their relative qe proportions

F high or rising:
  → Project redistribution: compute the new distributional equilibrium
  → Use F's trend to estimate redistribution velocity
  → Entities far from new equilibrium get larger corrections in projection

Infrastructure: EnergyFieldGrid already stores per-cell qe. Cost: O(grid_cells) per tick.
```

#### Autocorrelation Normalizer (system inertia)

```
ρ₁ = lag-1 autocorrelation of qe_total over sliding window

ρ₁ = 0.9 → high inertia:
  → Project with 90% current state + 10% trend
  → "System resists change, keep almost the same"

ρ₁ = 0.3 → responsive:
  → Project with 30% current state + 70% theoretical equilibrium
  → "System adjusts fast, converge to fixed point"

Infrastructure: SimTimeSeries.qe_history (RingBuffer, already tracked)
```

#### Lyapunov Normalizer (confidence horizon)

```
λ_max estimated via shadow simulation (periodic, every M ticks):
  Perturb one entity's qe by ε = 10⁻⁶ on SimWorldFlat copy
  Run both copies for W ticks
  λ_max ≈ (1/W) × ln(|divergence| / ε)

λ_max = -0.05 (strongly contractive):
  → Confidence horizon = |1/λ| = 20 ticks
  → Project up to K = 200 safely (10× horizon, paths converge)

λ_max = -0.001 (weakly contractive):
  → Horizon = 1000 ticks
  → K = 64 safe, K = 2000 not

λ_max = +0.01 (expansive):
  → Horizon = 100 ticks divergence
  → K = 8 max, with growing uncertainty bounds

Infrastructure: SimWorldFlat is Copy-friendly (~100KB). Shadow = memcpy. Cost: ~100KB + W ticks.
```

#### McTaggart Normalizer (discrete event density)

```
E = (death_rate + birth_rate + transition_rate) × K × population

E < 0.5: Deterministic projection (no events expected)
E ≈ 1:   Include most probable event with probability p
          Project two alternatives: with_event(p), without_event(1-p)
          Show user the more probable one
E > 5:   Include Poisson distribution of events
          Project weighted-average state
          Anchor reconciles with actual events

Infrastructure:
  - predict_death_ticks() — exact death timing
  - REPRODUCTION_THRESHOLD = 50.0 — birth timing estimable
  - ASTEROID_INTERVAL = 5000 — catastrophe timing deterministic
```

#### Entropy Acceleration Normalizer (reorganization speed)

```
H(t) = -Σ_cells p_c × ln(p_c)    (Shannon entropy of energy distribution)

|d²H/dt²| ≈ 0: Entropy production steady → project with simple extrapolation
|d²H/dt²| > ε: Entropy accelerating → project conservatively, reduce K
                System is reorganizing, projection accuracy degrades

Infrastructure: EnergyFieldGrid per-cell qe. Cost: O(grid_cells) per tick.
```

### The Feedback Loop (Calibration Over Time)

Each reconciliation generates a training datum:

```
Input:  metrics at fork time (ρ₁, H, F, λ_max, E, σ², population, K_used)
Output: projection accuracy (% entities with diff < ε)

After N reconciliations:
  → Telescope LEARNS which metric combinations predict good projections
  → Not ML — empirical threshold calibration via feedback

Example evolution:
  Reconciliation #1:   ρ₁=0.3, H=0.8, K=64  → diff=0.1%  → PERFECT
  Reconciliation #2:   ρ₁=0.3, H=0.8, K=96  → diff=0.3%  → OK
  Reconciliation #3:   ρ₁=0.3, H=0.8, K=128 → diff=4.2%  → TOO FAR
  Reconciliation #4:   ρ₁=0.5, H=0.7, K=64  → diff=8.1%  → TOO FAR
  → Learns: "with ρ₁>0.4, reduce K. With H<0.75, reduce K further"

  Reconciliation #50:  ρ₁=0.85, H=0.6, K=16 → diff=0.2%  → well calibrated
  → Telescope knows exactly how far to project for each regime
```

Storage: `Vec<ReconciliationRecord>` — bounded ring buffer (last 256 reconciliations). Each record: 7 metrics (f32) + K (u32) + accuracy (f32) = 36 bytes. Total: ~9KB.

### Adaptive K (How Far to Project)

```
Start: K = 16 ticks
After 4 consecutive PERFECT reconciliations: K = K × 1.5
After 1 LOCAL reconciliation: K unchanged
After 1 SYSTEMIC reconciliation: K = K / 2
Floor: K_min = 4
Ceiling: K_max = min(1024, anchor_throughput_ticks_per_second)

The ceiling is physical: K cannot exceed what the Anchor can process
before the user needs the next projection.
```

### Equilibrium Drift (What the Telescope Tracks)

The equilibrium point itself moves because:
- NucleusReservoir depletes → less input → equilibrium drops
- Entities adapt (epigenetic, entrainment) → efficiency changes → equilibrium shifts
- Seasons modulate input → equilibrium oscillates with period = year
- Asteroid impacts reset population → temporary equilibrium break

```
qe_eq(t) = E_in(t) / dissipation_rate_effective(t)
d(qe_eq)/dt = rate of equilibrium drift

Small drift → Telescope projects far (equilibrium quasi-static)
Large drift → Telescope projects conservatively (equilibrium moving)
```

### Scale Invariance (Why It Works at All Scales)

Natural systems exhibit self-organized criticality (Bak 1996):
- Event sizes: P(S) ∝ S^(-τ), τ ≈ 1.2-2.0
- Waiting times: P(T) ∝ T^(-β), β ≈ 0.7-1.5
- Power spectrum: S(f) ∝ 1/f^β, β ≈ 1.0 (pink noise)
- Hurst H > 0.5: trends persist, stasis tends to continue

The same Telescope + Anchor pattern works at every scale because the statistical structure is scale-free.

## Consequences

### Positive
- **Perceived instant** time-travel — user sees projected future immediately
- **Guaranteed correctness** — Anchor is full-fidelity ground truth, always catches up
- **Self-improving** — each reconciliation calibrates the Telescope's projection accuracy
- **No false positives/negatives** — no go/no-go gate to get wrong; Anchor verifies everything
- **Local corrections** — cascade affects only the causal neighborhood of diffs, not the entire world
- **Composes with tick_fast** — isolated entities use O(1) analytical stepping in BOTH timelines
- **Scientifically valuable outputs** — regime metrics (H, ρ₁, F, λ) characterize the simulation dynamics
- **Builds on existing infrastructure** — SimWorldFlat (memcpy), rayon (parallelism), macro_analytics (solvers), checkpoint_system (snapshots)

### Negative
- Two timelines = 2× memory for SimWorldFlat (~200KB total — negligible)
- Background thread consumes CPU even when not needed
- Reconciliation cascade has variable cost (usually O(1), worst case O(n))
- User sees speculative state that may "shimmer" on correction (visual artifact)

### Risks — Deep Analysis (Physics + Metaphysics + Infrastructure)

---

#### RISK 1: Projection Divergence — Telescope Projects Wrong Future

**The physics:** Near a bifurcation, the dominant eigenvalue λ of the Jacobian approaches zero (Wissel 1984). The system becomes unpredictable — small perturbations grow. Critical Slowing Down (Scheffer et al. 2009).

**The metaphysics:** Heraclitus — "πάντα ῥεῖ" (everything flows). Stability is an illusion produced by balanced opposing tensions. A river *looks* stable but IS the flow. Never measure stasis by comparing states — measure *rates* and *rate-of-rates*. Buddhist *anicca*: the appearance of permanence is *santati* (continuity through rapid succession), not unconditioned stability. *Jarā* (invisible decay) operates within apparent stillness.

**Mitigation — Rules as normalizers (not gates):**

The rules don't block projection — they **improve** it. And the Anchor catches whatever the rules miss.

- **Heraclitus Criterion (ρ₁):** High autocorrelation → project with more inertia. Rising ρ₁ → reduce K (system approaching criticality, projection horizon shrinks).
- **Fisher Spike (F):** Rising F → project distributional shift, not stasis. Catches shape changes that mean/variance miss.
- **Entropy Acceleration (d²H/dt²):** Accelerating entropy → project conservatively, system reorganizing.

**Why this is better than blocking:** Even a "wrong" projection is valuable — the Anchor catches the truth, the diff teaches the Telescope what it missed, and future projections of similar regimes improve. A blocked skip teaches nothing.

**Infrastructure:**
- `SimTimeSeries.qe_history` (RingBuffer, 512 entries) — ρ₁ computation
- `EnergyFieldGrid` — Fisher and entropy computation per cell
- `BridgeCache` band normalization — quantized energy distribution

---

#### RISK 2: Cascade Explosion — Correction Propagates Too Far

**The physics:** Principle of locality — interactions have bounded range (causal cone). Over N ticks, causal radius = N × d_max. Corrections can only propagate within this cone.

**The metaphysics:** Leibniz's Monadology — independent entities ("monads without windows") evolve independently. The degree to which entities fail to be monads (coupling > 0) is the degree to which corrections cascade. Whitehead's *prehension*: entities ARE their relations — removing relations doesn't simplify, it destroys. But Resonance interactions have finite range, so the cascade is bounded.

**Mitigation — 3 bounds:**

**Bound 1 — Causal Cone Limit (hard, exact):**
```
Cascade radius ≤ K × d_max
d_max = max(PREDATION_RANGE=3, PACK_SCAN_RADIUS=8, CULTURE_SCAN_RADIUS=10)

For K=64, d_max=10: cascade radius ≤ 640 world units
But: most corrections are far smaller (damped by attenuation — Axiom 7)

Infrastructure: is_isolated() + ISOLATION_RANGE_SQ already compute causal reach
```

**Bound 2 — Damped Propagation (practical):**
```
Each cascade step: correction_magnitude × attenuation_factor
Attenuation: distance-dependent (Axiom 7), frequency-dependent (Axiom 8)

Correction of 5% qe at entity A → 5% × attenuation at neighbor B → ...
Typically dies within 2-3 hops (exponential decay of correction magnitude)
Stop when correction < ε (0.1% of entity qe)
```

**Bound 3 — Fiedler Decomposition (structural):**
```
If interaction graph has low Fiedler value (λ₂ < ε):
  → Graph is nearly disconnected
  → Corrections in component C₁ do NOT propagate to C₂
  → Cascade is confined to the connected component of the diff

Infrastructure: SpatialIndex, NeighborBuffer, MultiscaleSignalGrid (dormant)
```

**Worst case:** Systemic diff (>10% entities) → replace entire speculative state with Anchor state. Cost = O(1) memcpy of SimWorldFlat. No cascade needed.

---

#### RISK 3: Anchor Falls Behind — Background Can't Keep Up

**The physics:** The Anchor must process K ticks before the user needs the next projection. If the Anchor is slower than real-time, the speculative gap grows unboundedly.

**The metaphysics:** McTaggart's A-series vs B-series. The Anchor operates in A-series time (each tick must be processed in order — becoming matters). The Telescope operates in B-series time (states are related by static functions — order doesn't matter). The A-series is inherently slower because it cannot be compressed.

**Mitigation:**

```
K_max = anchor_throughput × acceptable_latency

If Anchor processes 1000 ticks/second and acceptable latency = 1 second:
  K_max = 1000

If Anchor can't keep up at current K:
  → Reduce K until Anchor catches up within acceptable_latency
  → In batch mode: Anchor uses tick_fast() for isolated entities (5-10× speedup)
  → In batch mode: rayon parallelism for Anchor (already available)

Infrastructure:
  - tick_fast() — O(1) stepping for isolated entities within Anchor
  - rayon — multi-core parallelism for Anchor thread
  - WorldBatch — already runs N worlds in parallel
```

**Floor guarantee:** K_min = 4 ticks. Even if Anchor is slow, the Telescope always projects at least 4 ticks ahead. The user never waits for the Anchor — they see the Telescope immediately.

---

#### The Master Inequality (Information-Theoretic Unification)

All risks reduce to **information loss during temporal compression:**

```
Projection_error ∝ h_KS × K                         (chaos: bits/tick of unpredictable evolution)
                  + Σ I(Cᵢ; Cⱼ) × K                  (coupling: missed interactions)
                  + Fisher_anomaly × K                  (undetected distributional shifts)

But: error is ALWAYS caught by the Anchor.
The only question is: how often does the Anchor need to correct?

Correction_frequency ∝ Projection_error / tolerance

Low error (good rules) → rare corrections → smooth user experience
High error (bad rules) → frequent corrections → shimmer, but still CORRECT
```

| Component | Role | Key Metric | Infrastructure |
|-----------|------|------------|----------------|
| Telescope | Project future | Normalizer-weighted analytical extrapolation | macro_analytics, exact_cache |
| Anchor | Ground truth | Full tick-by-tick simulation | SimWorldFlat, tick/tick_fast |
| Rules | Improve projection | ρ₁, H, F, λ_max, E, d²H/dt² | RingBuffer, EnergyFieldGrid |
| Reconciliation | Catch errors | Diff + cascade + learn | memcpy, SpatialIndex |
| Feedback | Self-improve | accuracy_history → adjust K and rule weights | ReconciliationRecord ring buffer |

## Files to Modify

| File | Action | Why |
|------|--------|-----|
| `src/blueprint/equations/temporal_telescope.rs` | **Create** | Pure math: normalizer rules (Hurst DFA, Fisher, sliding variance/autocorrelation, λ_max estimation, entropy acceleration), diff functions, cascade propagation |
| `src/blueprint/constants/temporal_telescope.rs` | **Create** | Thresholds: K_min, K_max, cascade_ε, correction classifications (PERFECT/LOCAL/SYSTEMIC), normalizer weights |
| `src/batch/telescope.rs` | **Create** | `TelescopeState` (dual timeline controller), `ReconciliationRecord`, `ProjectionNormalizers`, adaptive K logic, feedback loop |
| `src/batch/pipeline.rs` | **Modify** | Fork: Anchor continues tick-by-tick, Telescope projects analytically. Reconciliation point when Anchor catches up. |
| `src/batch/arena.rs` | **Modify** | `SimWorldFlat::diff(other) → DiffReport` — per-entity field comparison. `SimWorldFlat::cascade(diff, spatial_index)` — local correction propagation. |
| `src/runtime_platform/dashboard_bridge.rs` | **Modify** | Expose telescope metrics: regime, K, projection accuracy, correction frequency, normalizer values |
| `src/simulation/emergence/geological_lod.rs` | **Modify** | Wire GeologicalLOD to telescope K (dynamic LOD = Anchor's tick compression for distant regions) |
| `src/simulation/emergence/multiscale.rs` | **Modify** | Feed regional aggregation into Fisher/entropy normalizers |

## Files NOT Modified

| File | Why |
|------|-----|
| `src/blueprint/equations/macro_analytics.rs` | Already has O(1) solvers — Telescope calls them, doesn't change them |
| `src/batch/systems/*.rs` | 33 batch systems unchanged — Anchor runs them as-is |
| `src/layers/*.rs` | No new ECS components — dual timeline is pipeline-level |
| `src/blueprint/equations/derived_thresholds.rs` | Fundamental constants unchanged |
| `src/sim_world.rs` | INV-7 unchanged — Anchor is full simulation, Telescope is disposable |
| `src/blueprint/equations/exact_cache.rs` | Already has exact_death_tick, kleiber_volume_factor — Telescope uses them |
| `src/bridge/cache.rs` | BridgeCache is transient/stateless — each timeline maintains its own instance |
| `src/layers/converged.rs` | Converged<T> env_hash — used as-is for skip-across-ticks validation |

## Implementation Plan

```
Phase 1 — Pure Math (no behavior change)
  1. Create temporal_telescope.rs equations:
     - sliding_variance(window) → f32
     - sliding_autocorrelation_lag1(window) → f32
     - hurst_dfa(window, min_box=8, max_box=128) → f32
     - estimate_lambda_max(rho1, dt) → f32
     - shannon_entropy(distribution) → f32
     - fisher_information(distribution, prev_distribution, dt) → f32
     - project_qe(current, trend, H, K) → f32   (Hurst-normalized extrapolation)
  2. Create diff functions:
     - entity_diff(slot_a, slot_b) → EntityDiff { qe_delta, pos_delta, alive_mismatch }
     - world_diff(world_a, world_b) → DiffReport { perfect, local, systemic }
  3. Create constants with derived thresholds
  4. Unit tests: known time series → expected H, σ², ρ₁, F

Phase 2 — Dual Timeline (batch)
  5. Implement SimWorldFlat::fork() → (anchor_copy, telescope_copy)
  6. Telescope: project K ticks via normalizer-weighted analytical solvers
  7. Anchor: continue tick()/tick_fast() on the copy
  8. Reconciliation: diff + classify + cascade
  9. Feedback loop: ReconciliationRecord storage + adaptive K
  10. Integration tests: Anchor and Telescope agree on known dynamics

Phase 3 — Parallelism
  11. Anchor runs in background thread (rayon::spawn or std::thread)
  12. Telescope projects on main thread (instantaneous)
  13. Reconciliation via channel: Anchor sends DiffReport when caught up
  14. Adaptive K respects Anchor throughput ceiling

Phase 4 — Activation of Dormant Systems
  15. Wire GeologicalLOD to telescope K (dynamic LOD)
  16. Wire MultiscaleSignalGrid into Fisher/entropy normalizers
  17. Property tests: Telescope + Anchor final state matches pure Anchor final state

Phase 5 — Dashboard + Bevy Integration
  18. Expose telescope metrics to SimTimeSeries
  19. Regime indicator: PROJECTING / RECONCILING / CORRECTING
  20. Interactive "fast-forward to next event" button
  21. Visual indicator for corrected entities (brief highlight on reconciliation)
```

## Success Criteria

1. **Perceived speed:** User sees projected future within 1ms of requesting advance
2. **Correctness:** After reconciliation, state matches pure Anchor (zero permanent divergence)
3. **Convergence:** Projection accuracy improves over time (correction frequency decreases)
4. **Cascade locality:** >90% of corrections affect <5% of entities
5. **Conservation:** INV-7 holds on Anchor timeline (Telescope is disposable, doesn't need INV-7)
6. **Throughput:** Anchor processes ≥500 ticks/second (batch mode, single world)
7. **Calibration:** After 100 reconciliations, mean projection accuracy >95%

## Alternatives Considered

| Alternative | Why Rejected |
|------------|-------------|
| Single-timeline skip with go/no-go gates | Must predict if skip is safe — a fundamentally hard problem (chaos, CSD). 60-90% detection rate means 10-40% undetected errors with no correction mechanism. Dual timeline catches ALL errors via reconciliation. |
| Fixed tick compression (always 10×) | Can't adapt. Either skips too much (loses precision at transitions) or too little (no speedup during stasis). |
| Only entity-level skip (extend tick_fast) | Works for isolated entities but not densely interacting populations. Dual timeline skips at world level. |
| Event-driven simulation (skip to next event) | Requires knowing future events a priori. In emergent systems, events arise from interactions — cannot pre-compute. |
| Statistical surrogate model (ML) | Loses emergent property guarantee. Can miss novel dynamics. Telescope uses exact analytical solvers + Anchor as ground truth. |
| Just run faster (optimize per-tick) | Orthogonal. Dual timeline composes with per-tick optimization. tick_fast() in Anchor already provides 5-10× for isolated entities. |

## Scientific References

### Physics & Mathematics
1. Bak, P., Tang, C. & Wiesenfeld, K. (1987). "Self-organized criticality." *Phys. Rev. Lett.*, 59(4), 381-384.
2. Scheffer, M. et al. (2009). "Early-warning signals for critical transitions." *Nature*, 461, 53-59.
3. Kevrekidis, I.G. et al. (2004). "Equation-free multiscale computation." *AIChE Journal*, 50(7), 1346-1355.
4. Peng, C.-K. et al. (1994). "Mosaic organization of DNA nucleotides." *Phys. Rev. E*, 49(2), 1685.
5. Hurst, H.E. (1951). "Long-term storage capacity of reservoirs." *Trans. ASCE*, 116, 770-799.
6. Gillespie, D.T. (2001). "Approximate accelerated stochastic simulation." *J. Chem. Phys.*, 115(4), 1716.
7. Gould, S.J. & Eldredge, N. (1977). "Punctuated equilibria." *Paleobiology*, 3(2), 115-151.
8. Gutenberg, B. & Richter, C.F. (1944). "Frequency of earthquakes in California." *Bull. Seismol. Soc. Am.*, 34(4), 185.
9. Holling, C.S. (1973). "Resilience and stability of ecological systems." *Annu. Rev. Ecol. Syst.*, 4, 1-23.
10. Dakos, V. et al. (2012). "Methods for detecting early warnings of critical transitions." *PLOS ONE*, 7(7), e41010.
11. Wissel, C. (1984). "A universal law of the characteristic return time near thresholds." *Oecologia*, 65, 101-107.
12. Frieden, B.R. (2004). *Science from Fisher Information.* Cambridge University Press.
13. Simon, H.A. (1962). "The Architecture of Complexity." *Proc. American Philosophical Society*, 106(6), 467-482.
14. Eckmann, J.-P. & Ruelle, D. (1985). "Ergodic theory of chaos and strange attractors." *Rev. Mod. Phys.*, 57, 617.
15. Pesin, Ya.B. (1977). "Characteristic Lyapunov exponents and smooth ergodic theory." *Russian Mathematical Surveys*, 32(4), 55-114.

### Philosophy & Metaphysics
16. Heraclitus (c. 500 BCE). Fragments DK B49a, B91 — flux doctrine: stability as dynamic balance of opposing tensions.
17. Leibniz, G.W. (1714). *Monadology* — pre-established harmony: independent entities synchronized without causal interaction.
18. Whitehead, A.N. (1929). *Process and Reality* — actual occasions, prehension, fallacy of misplaced concreteness.
19. McTaggart, J.M.E. (1908). "The Unreality of Time." *Mind*, 17(68), 457-474 — A-series (becoming) vs B-series (static order).
20. Bergson, H. (1907). *Creative Evolution* — durée as irreducible creative novelty per moment of time.
21. Buddhist Abhidhamma — *anicca* (impermanence), *santati* (continuity through rapid succession), *jarā* (invisible decay within apparent stability).

## Codebase References

- `src/batch/pipeline.rs` — `tick_fast()`, isolated entity O(1) stepping
- `src/batch/arena.rs` — `SimWorldFlat`, flat Copy struct (~100KB), `update_total_qe()`
- `src/batch/batch.rs` — `WorldBatch`, rayon parallelism for N worlds
- `src/blueprint/equations/macro_analytics.rs` — exponential decay, allometric growth solvers
- `src/blueprint/equations/batch_stepping.rs` — `predict_death_ticks()`, `is_isolated()`
- `src/blueprint/equations/exact_cache.rs` — `exact_death_tick()`, `kleiber_volume_factor()`, `frequency_alignment_exact()`
- `src/runtime_platform/dashboard_bridge.rs` — `RingBuffer` (512), `SimTimeSeries`
- `src/simulation/emergence/geological_lod.rs` — tick compression levels (dormant)
- `src/simulation/emergence/multiscale.rs` — 3-level spatial aggregation (dormant)
- `src/layers/converged.rs` — `Converged<T>` generic convergence marker with env_hash
- `src/batch/systems/internal_field.rs` — `field_converged` (AS-2 optimization)
- `src/batch/census.rs` — `PopulationCensus`, `EntitySnapshot`
- `src/batch/harness.rs` — `GenerationStats`, evolutionary history
- `src/bridge/cache.rs` — `BridgeCache`, LRU with band normalization, transient/stateless
- `src/layers/kleiber_cache.rs` — per-entity Kleiber volume factor cache
- `src/layers/gompertz_cache.rs` — per-entity death tick cache (eternal post-birth)
- `src/simulation/checkpoint_system.rs` — RON/JSON snapshot save/load
- `src/batch/scratch.rs` — `NeighborBuffer`, thread-local scratch pads
