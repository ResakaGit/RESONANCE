---
document_id: RD-4.1
title: Software Validation Plan
standard: FDA General Principles of Software Validation (2002), GAMP 5 (2008)
version: 1.0
date: 2026-04-02
status: DRAFT
author: Resonance Development Team
---

# Software Validation Plan

## 1. Purpose

This document defines the validation strategy for RESONANCE, establishing the methods, tools, environments, acceptance criteria, and responsibilities for demonstrating that the software consistently produces results conforming to its intended use (RD-1.1) and software requirements (RD-1.3). It follows FDA General Principles of Software Validation (2002) and GAMP 5 lifecycle approach, adapted for a Class A research simulation tool.

This plan covers three validation layers:

1. **Unit and integration testing** -- verifying individual equations and system interactions.
2. **Property-based and batch testing** -- verifying conservation invariants and headless simulation correctness under arbitrary inputs.
3. **Experiment-level validation** -- verifying that simulation outputs are consistent with published scientific data and stated predictions.

Related documents:

- **RD-1.1** INTENDED_USE.md -- defines what RESONANCE is validated for
- **RD-1.2** SOFTWARE_SAFETY_CLASS.md -- Class A (no patient harm possible)
- **RD-1.3** SOFTWARE_REQUIREMENTS_SPEC.md -- functional requirements
- **RD-4.2** CREDIBILITY_MODEL.md -- ASME V&V 40 credibility assessment
- **RD-4.3** VERIFICATION_REPORT.md -- code verification results
- **RD-4.4** VALIDATION_REPORT.md -- experiment validation results
- **RD-4.5** UNCERTAINTY_ANALYSIS.md -- sensitivity and uncertainty quantification

## 2. Scope

### 2.1 In Scope

| Domain | Description | Location |
|--------|-------------|----------|
| Pure math equations | All functions in `src/blueprint/equations/` (50+ domain files) | Unit tests co-located in each file |
| Conservation invariants | Pool conservation, extraction bounds, energy non-negativity | `tests/property_conservation.rs` (proptest) |
| Determinism | Bit-exact reproducibility across runs and platforms | `src/blueprint/equations/determinism.rs` |
| Derived thresholds | All ~40 lifecycle constants derived from 4 fundamentals | `src/blueprint/equations/derived_thresholds.rs` |
| ECS system integration | Each registered system transforms components correctly | `#[cfg(test)]` in simulation modules |
| Batch simulator | 33 headless systems operating without Bevy | `src/batch/` (156 tests) |
| Drug experiments | Pathway inhibitor, Bozic validation, adaptive therapy, Rosie case | `src/use_cases/experiments/`, `src/bin/` |
| Clinical calibration | 4 calibration profiles mapping qe to published clinical units | `src/blueprint/equations/clinical_calibration.rs` |

### 2.2 Out of Scope

| Domain | Reason |
|--------|--------|
| Rendering and visual output | Not safety-relevant; visual derivation does not affect simulation correctness |
| Keyboard/mouse input handling | Infrastructure, not simulation logic |
| Schedule ordering verification | Bevy scheduler is SOUP; ordering verified by integration test outcomes |
| Performance benchmarks | Performance is a quality attribute, not a functional requirement |
| Patient data handling | No patient data exists in RESONANCE (RD-1.1 Section 5.1) |

## 3. V&V Strategy: Three Layers

### 3.1 Layer 1: Unit Testing (Equation Verification)

**Objective:** Every public function in `src/blueprint/equations/` produces correct output for documented inputs, edge cases, and boundary conditions.

**Method:** `#[cfg(test)] mod tests` blocks co-located with each equation file. Tests are deterministic, require no external state, and complete in milliseconds.

**Coverage target:** 100% of public functions in `blueprint/equations/` have at least one dedicated unit test. Current state: ~1,800 unit tests across 50+ files.

**Test structure per equation:**

| Category | Description | Example |
|----------|-------------|---------|
| Happy path | Nominal inputs, expected output | `basal_rate_is_one` |
| Edge case | Zero, negative, NaN, Inf inputs | `density_zero_radius_returns_zero` |
| Boundary | Threshold crossings, min/max values | `spawn_threshold_one_third` |
| Monotonicity | Output ordering matches physical expectation | `density_thresholds_monotonic` |
| Invariant | Mathematical identities that must hold | `survival_threshold_is_exp_neg2` |
| Derivation chain | Downstream constants consistent with upstream | `branch_is_twice_sustaining` |

**Key equation files and test counts:**

| File | Functions | Tests | Domain |
|------|-----------|-------|--------|
| `derived_thresholds.rs` | ~30 | 17 | Lifecycle constants from 4 fundamentals |
| `pathway_inhibitor.rs` | 11 | 32 | Drug inhibition (Competitive/Noncompetitive/Uncompetitive) |
| `coulomb.rs` | ~10 | 26 | Particle physics (Coulomb + LJ) |
| `variable_genome.rs` | ~20 | 62 | Variable-length genome operations |
| `metabolic_genome.rs` | ~25 | 80 | Gene-to-metabolism pipeline |
| `protein_fold.rs` | ~12 | 27 | HP lattice folding |
| `codon_genome.rs` | ~15 | 28 | Codon translation |
| `multicellular.rs` | ~15 | 33 | Cell adhesion, colony detection |
| `determinism.rs` | 6 | ~23 | Hash-based RNG |
| `clinical_calibration.rs` | ~10 | 21 | Unit conversion (qe to nM/days/cells) |
| `conservation.rs` | ~8 | See proptest | Energy conservation |
| `sensitivity.rs` | 5 | ~8 | Sensitivity analysis utilities |
| `exact_cache.rs` | 3 | ~6 | Zero-loss precomputed values |
| `awakening.rs` | ~4 | ~8 | Coherence-driven awakening |
| `radiation_pressure.rs` | ~3 | ~5 | Frequency-modulated surplus redistribution |

### 3.2 Layer 2: Property-Based and Integration Testing

**Objective:** Conservation invariants hold under arbitrary valid inputs; ECS systems produce correct transformations when composed.

#### 3.2.1 Property-Based Testing (proptest)

**Location:** `tests/property_conservation.rs`

**Properties verified:**

| Property | Test | Axiom |
|----------|------|-------|
| `is_valid_qe` accepts all finite non-negative f32 | `prop_valid_qe_is_finite_non_negative` | Axiom 1 |
| NaN/Inf always invalid | `prop_nan_inf_always_invalid` | Axiom 1 |
| `has_invalid_values` consistent with `is_valid_qe` | `prop_has_invalid_values_consistent_with_is_valid` | Axiom 1 |
| Global conservation error non-negative | `prop_global_conservation_error_non_negative` | Axiom 5 |
| No overshoot implies zero conservation error | `prop_global_conservation_no_overshoot_means_zero` | Axiom 5 |
| Per-pool conservation error non-negative | `prop_per_pool_conservation_error_non_negative` | Axiom 2 |
| `pool_next_tick` never negative | `prop_pool_next_tick_never_negative` | Axiom 1 |
| `pool_next_tick` monotone in intake | `prop_pool_next_tick_monotone_in_intake` | Axiom 1 |
| Dissipation loss bounded by pool | `prop_dissipation_loss_bounded_by_pool` | Axiom 4 |
| Proportional extraction bounded | `prop_extract_proportional_bounded` | Axiom 2 |
| Proportional extraction sum <= available | `prop_extract_proportional_sum_le_available` | Axiom 2 |
| Greedy extraction bounded | `prop_extract_greedy_bounded` | Axiom 2 |
| Competitive extraction non-negative and finite | `prop_extract_competitive_non_negative_finite` | Axiom 3 |
| Aggressive extraction bounded | `prop_extract_aggressive_bounded` | Axiom 3 |
| Regulated extraction non-negative | `prop_extract_regulated_non_negative` | Axiom 2 |
| Relative fitness in [0, 1] | `prop_relative_fitness_in_unit_range` | Axiom 3 |
| Scaled extractions sum <= available | `prop_scale_extractions_invariant` | Axiom 5 |
| Ticks to collapse consistent | `prop_ticks_to_collapse_consistent` | Axiom 4 |

**Configuration:** proptest default (256 cases per property). Shrinking enabled for counterexample minimization.

#### 3.2.2 Integration Testing (ECS Systems)

**Method:** `MinimalPlugins` Bevy app, spawn only required components, run ONE `FixedUpdate` tick, assert expected delta on target components.

**Coverage target:** Each registered simulation system (see `CLAUDE.md` Module Map) has at least one integration test verifying its transformation.

**Location:** `#[cfg(test)]` blocks in `src/simulation/` modules.

**Approximate counts:**

| Module | Tests | Coverage |
|--------|-------|----------|
| `simulation/thermodynamic/` | ~80 | Physics, pre-physics, sensory |
| `simulation/metabolic/` | ~60 | Basal drain, senescence, trophic |
| `simulation/emergence/` | ~120 | Theory of mind, symbiosis, niche, epigenetics, culture, entrainment |
| `simulation/lifecycle/` | ~50 | Body plan, shape inference |
| `simulation/reproduction/` | ~40 | Flora seed, fauna offspring |
| `simulation/awakening.rs` | ~15 | Coherence threshold |
| `worldgen/` | ~200 | Nucleus, propagation, materialization, terrain |

### 3.3 Layer 3: Experiment-Level Validation

**Objective:** End-to-end simulation experiments produce outputs consistent with published scientific predictions and stated claims.

**Method:** Stateless experiment harnesses (`src/use_cases/experiments/`) that configure, run, and report. Multi-seed robustness via repeated execution with distinct seeds.

**Experiments:**

| Experiment | Published Reference | File | Acceptance Criterion |
|------------|-------------------|------|---------------------|
| Exp 1-3 | Zenodo paper (DOI: 10.5281/zenodo.19342036) | `docs/paper/resonance_arxiv.tex` | Reproduces published results |
| Exp 4: Pathway inhibition | None (internal) | `pathway_inhibitor_exp.rs` | Monotonic dose-response across 10 seeds |
| Exp 5: Bozic 2013 | Bozic et al. 2013 (eLife) | `pathway_inhibitor_exp.rs`, `bozic_validation.rs` | combo > mono, combo > double_dose, 10/10 seeds |
| Exp 6: Adaptive therapy | Gatenby et al. 2009 (Cancer Res) | Use case harness | Growth stabilization at zero, >= 7/10 seeds |
| Exp 7: Rosie case | London 2003, London 2009 | Use case harness | Partial response, 5/5 seeds |

**Acceptance criteria per experiment:**

- **Bozic 2013:** `efficiency(combo_AB) < efficiency(mono_A)` AND `efficiency(combo_AB) < efficiency(double_A)` in >= 8/10 independent seeds. Suppression within published qualitative range.
- **Pathway inhibition:** Strictly monotonic dose-response (higher concentration -> lower efficiency) across all 10 seeds.
- **Adaptive therapy:** Growth rate stabilizes at or below zero by generation 40 in >= 7/10 seeds.
- **Rosie case:** Tumor population decreases >= 30% from peak by generation 60 in >= 4/5 seeds.

## 4. Tools and Infrastructure

### 4.1 Test Framework

| Tool | Purpose | Version |
|------|---------|---------|
| `cargo test` | Primary test runner | Rust stable 2024 edition (MSRV 1.85) |
| `proptest` | Property-based fuzzing | Pinned in `Cargo.lock` |
| `cargo bench --bench batch_benchmark` | Performance regression (informational) | Rust stable |

### 4.2 Build and CI

| Environment | Configuration | Determinism |
|-------------|---------------|-------------|
| Developer workstation (macOS) | `cargo test` (debug mode) | Bit-exact (hash-based RNG) |
| Developer workstation (Linux) | `cargo test` (debug mode) | Bit-exact (hash-based RNG) |
| CI headless | `cargo test` (release mode) | Bit-exact (hash-based RNG) |
| Batch simulation (rayon) | `cargo run --release --bin bozic_validation` | Deterministic per seed |
| Headless rendering | `cargo run --bin headless_sim -- --ticks N` | Deterministic per seed |

### 4.3 Determinism Guarantee

RESONANCE uses a custom hash-based RNG (`src/blueprint/equations/determinism.rs`) with no dependency on `std::rand` or any external randomness source. The RNG is:

- **PCG-like state step:** `state' = state * 6364136223846793005 + 1442695040888963407` (wrapping arithmetic)
- **Bit-exact:** `hash_f32_slice` uses `f32::to_bits()` to distinguish +0.0 from -0.0 and handle NaN deterministically
- **Platform-independent:** No floating-point operation ordering depends on hardware (all f32, no f64 intermediates)
- **Verified:** 23 unit tests in `determinism.rs` confirm identical output across invocations

This means: same seed, same commit, same parameters -> identical simulation output on any supported platform.

## 5. Test Naming Convention

All test functions follow the pattern: `<function_name>_<condition>_<expected_result>`

Examples:

- `basal_rate_is_one` -- `basal_drain_rate()` returns 1.0
- `density_thresholds_monotonic` -- liquid < gas < plasma
- `spawn_threshold_one_third` -- `spawn_potential_threshold()` equals 1/3
- `prop_pool_next_tick_never_negative` -- proptest: pool_next_tick >= 0 for all valid inputs
- `coulomb_force_inverse_square` -- force scales as 1/r^2

Property tests are prefixed with `prop_` to distinguish from deterministic tests.

## 6. Regression Policy

### 6.1 Zero Tolerance for Test Failures

All 3,113 tests must pass before any code change is merged. A single test failure blocks the merge. There is no concept of "known failing tests" or "expected failures" (except the 1 `#[ignore]` test, which is explicitly excluded from the pass gate).

### 6.2 Regression Detection

| Trigger | Action |
|---------|--------|
| New equation added | At least one unit test required in the same file before merge |
| Existing equation modified | All existing tests must pass; new tests added for new behavior |
| New system registered | At least one integration test verifying the system's transformation |
| New experiment added | Pipeline test covering configuration, execution, and output validation |
| Constant changed | All derived threshold tests must pass; sensitivity analysis updated if fundamental |
| Dependency updated | Full `cargo test` run; Cargo.lock updated atomically |

### 6.3 Test Execution Cadence

| Event | Scope | Expected Duration |
|-------|-------|-------------------|
| Every commit (local) | `cargo test` (full suite) | ~36 seconds |
| Before merge | `cargo test` (full suite, release mode) | ~36 seconds |
| Weekly | `cargo run --release --bin bozic_validation` (10-seed) | ~95 seconds |
| On fundamental constant change | Full suite + all experiments + sensitivity sweep | ~5 minutes |

## 7. Traceability

### 7.1 Requirements to Tests

Each software requirement (RD-1.3) is traceable to one or more tests. The traceability matrix is maintained in RD-3.1 (TRACEABILITY_MATRIX.md). For this validation plan, the mapping principle is:

| Requirement Type | Test Layer | Example |
|-----------------|------------|---------|
| Mathematical correctness | Layer 1 (unit) | `derived_thresholds.rs` tests verify algebraic derivations |
| Conservation invariants | Layer 2 (proptest) | `property_conservation.rs` fuzzes extraction functions |
| System behavior | Layer 2 (integration) | `simulation/metabolic/basal_drain.rs` tests verify drain computation |
| Experiment predictions | Layer 3 (experiment) | `bozic_validation.rs` verifies combo > mono |
| Determinism | Layer 1 + 2 | `determinism.rs` unit tests + proptest hash consistency |
| Safety (no patient harm) | Layer 1 (unit) | No patient data structures exist (verified by absence) |

### 7.2 Axiom to Test Mapping

| Axiom | Enforcing Tests | Location |
|-------|----------------|----------|
| 1: Everything is Energy | `prop_valid_qe_is_finite_non_negative`, all qe-range tests | proptest, unit tests |
| 2: Pool Invariant | `prop_extract_proportional_sum_le_available`, `prop_scale_extractions_invariant` | proptest |
| 3: Competition as Primitive | `prop_extract_competitive_non_negative_finite` | proptest |
| 4: Dissipation | `prop_dissipation_loss_bounded_by_pool`, `senescence_scales_with_dissipation` | proptest, unit tests |
| 5: Conservation | `prop_global_conservation_error_non_negative`, `prop_global_conservation_no_overshoot_means_zero` | proptest |
| 7: Distance Attenuation | `coulomb_force_inverse_square`, all attenuation tests | unit tests |
| 8: Oscillatory Nature | `frequency_alignment` tests, `gaussian_frequency_alignment` tests | unit tests |

## 8. Roles and Responsibilities

| Role | Responsibility | CLAUDE.md Mapping |
|------|---------------|-------------------|
| Alquimista | Writes tests for new code; ensures Layer 1 coverage | Code author |
| Observador | Reviews test adequacy during code review; flags missing coverage | Reviewer |
| Verificador | Confirms all tests pass; validates test-to-requirement traceability | PR reviewer |
| Planificador | Defines experiment acceptance criteria before execution | Sprint planner |

## 9. Acceptance Criteria Summary

| Layer | Criterion | Current Status |
|-------|-----------|---------------|
| Layer 1 (Unit) | 100% public fn coverage in `equations/` | Met (~1,800 tests) |
| Layer 2 (Property) | All conservation properties hold under 256 random cases | Met (18 proptest properties) |
| Layer 2 (Integration) | Each registered system has >= 1 integration test | Met (~800 tests) |
| Layer 3 (Experiment) | Each experiment meets its acceptance criterion (Section 3.3) | Met (Bozic 10/10, pathway monotonic, adaptive 7/10, Rosie partial) |
| Cross-cutting (Determinism) | Same seed produces bit-exact same output | Met (23 determinism tests) |
| Cross-cutting (Regression) | 0 test failures at all times | Met (3,113 pass, 0 fail) |

## 10. Codebase References

| Reference | File Path |
|-----------|-----------|
| Test runner | `cargo test` (Rust stable 2024) |
| Property tests | `tests/property_conservation.rs` |
| Determinism | `src/blueprint/equations/determinism.rs` |
| Derived thresholds | `src/blueprint/equations/derived_thresholds.rs` |
| Pathway inhibitor | `src/blueprint/equations/pathway_inhibitor.rs` |
| Bozic validation | `src/bin/bozic_validation.rs` |
| Batch simulator | `src/batch/` (19 files, 33 systems, 156 tests) |
| Clinical calibration | `src/blueprint/equations/clinical_calibration.rs` |
| Sensitivity utilities | `src/blueprint/equations/sensitivity.rs` |
| Exact cache | `src/blueprint/equations/exact_cache.rs` |
| Experiment harness | `src/use_cases/experiments/pathway_inhibitor_exp.rs` |
| Headless simulator | `src/bin/headless_sim.rs` |

## 11. Revision History

| Version | Date | Author | Changes |
|---------|------|--------|---------|
| 1.0 | 2026-04-02 | Resonance Development Team | Initial validation plan. Three-layer strategy, acceptance criteria, regression policy, tool inventory. |

---

**Approvals:**

| Role | Name | Date | Signature |
|------|------|------|-----------|
| Author | Resonance Development Team | 2026-04-02 | _draft -- not signed_ |
| Reviewer | _pending_ | _pending_ | _pending_ |
| Approver | _pending_ | _pending_ | _pending_ |
