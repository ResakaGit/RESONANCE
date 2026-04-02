---
document_id: RD-6.4
title: Reproducibility Protocol
standard: ASME V&V 40, Good Scientific Practice
version: 1.0
date: 2026-04-02
status: DRAFT
author: Resonance Development Team
---

# Reproducibility Protocol

## 1. Purpose

This document provides exact, copy-paste instructions for independently reproducing every quantitative claim made by RESONANCE. Any researcher with the required environment can verify every experiment described in the Clinical Evaluation Report (RD-6.2) by executing the commands in this document and comparing output against the reference values provided.

RESONANCE is deterministic: identical inputs produce bit-identical outputs on any machine that meets the environment requirements. This protocol exploits that property to enable full reproducibility.

**Cross-references:**

- RD-6.2 (Clinical Evaluation Report): Contains the claims being reproduced
- RD-6.1 (Clinical Evaluation Plan): Defines acceptance criteria for evidence
- `src/blueprint/equations/determinism.rs`: Hash-based RNG implementation (23 tests)

## 2. Environment Requirements

### 2.1 Software Requirements

| Component | Requirement | Verification command |
|-----------|------------|---------------------|
| Rust toolchain | Stable 2024 edition, MSRV 1.85 | `rustc --version` (must report >= 1.85.0) |
| Cargo | Matching Rust version | `cargo --version` |
| Git | Any recent version | `git --version` |
| Operating system | Linux (x86_64), macOS (x86_64 or aarch64), or Windows (x86_64) | `uname -a` or `systeminfo` |
| Disk space | >= 2 GB free (for compilation artifacts) | `df -h .` |
| RAM | >= 4 GB (8 GB recommended for Bozic validation) | `free -h` or `sysctl hw.memsize` |

### 2.2 Hardware Requirements

| Component | Minimum | Recommended |
|-----------|---------|-------------|
| CPU | 2 cores | 8+ cores (Bozic validation uses rayon parallelism) |
| RAM | 4 GB | 8 GB |
| GPU | Not required (all experiments run headless) | Optional (for visual simulation) |

### 2.3 Environment Setup

```bash
# 1. Clone the repository
git clone https://github.com/ResakaGit/RESONANCE.git
cd RESONANCE

# 2. Checkout the evaluation commit
git checkout 971c7acb99decde45bf28860e6e10372718c51e2

# 3. Verify Rust toolchain
rustc --version
# Expected: rustc 1.85.0 (or higher stable release)

# 4. Build in release mode (required for experiment binaries)
cargo build --release
# Expected: Compiling resonance v0.1.0 (...)
# Expected: Finished `release` profile [optimized] target(s)

# 5. Verify dependencies are locked
cat Cargo.lock | head -5
# Expected: Cargo.lock exists and is checked in (pinned dependencies)
```

### 2.4 Build Verification

After `cargo build --release`, verify the build produced the expected binaries:

```bash
ls target/release/bozic_validation
ls target/release/cancer_therapy
ls target/release/pathway_inhibitor
ls target/release/adaptive_therapy
ls target/release/headless_sim
```

All five binaries must exist. If any is missing, the build failed partially — check `cargo build --release` output for errors.

## 3. Experiment Reproduction

### 3.1 Experiment 0: Full Test Suite

**Claim:** 3,113 tests pass with 0 failures.

```bash
cargo test 2>&1 | tail -5
```

**Expected output (last line):**

```
test result: ok. 3113 passed; 0 failed; 1 ignored; 0 measured; 0 filtered out; finished in XX.XXs
```

The exact wall time varies by machine. The test count (3113), failure count (0), and ignored count (1) must match exactly.

**Targeted test subsets:**

```bash
# Conservation fuzz tests (proptest)
cargo test property_conservation

# Pathway inhibitor pure math (32+ tests)
cargo test pathway_inhibitor -- --test-threads=1

# Clinical calibration (21 tests)
cargo test clinical_calibration

# Determinism (23 tests)
cargo test determinism

# Derived thresholds (17 tests)
cargo test derived_thresholds
```

### 3.2 Experiment 4: Pathway Inhibition Dose-Response

**Claim:** Monotonic dose-response — higher concentration produces greater suppression.

```bash
cargo run --release --bin pathway_inhibitor
```

**Expected reference values:**

| Condition | Efficiency | Suppression | Tolerance |
|-----------|-----------|-------------|-----------|
| Control (no drug) | 1.000 | 0.0% | Exact |
| Concentration 0.4 | ~0.488 | ~51.2% | +/- 0.02 |
| Concentration 0.8 | ~0.471 | ~52.9% | +/- 0.02 |

**Verification criteria:**

1. Control efficiency equals 1.000 (exact).
2. Low-dose efficiency < 1.000 (drug suppresses).
3. High-dose efficiency < low-dose efficiency (monotonic).
4. Both treatment efficiencies within tolerance of reference values.

**Note:** Because the default seed is deterministic, exact values should match across runs on the same platform. Cross-platform variation (if any) would appear in the least significant decimal places due to floating-point ordering differences in parallel reductions. The tolerance accounts for this.

### 3.3 Experiment 5: Bozic 2013 Replication (5-Arm)

**Claim:** Combination therapy suppresses more than monotherapy or doubled monotherapy, in 10/10 independent seeds.

```bash
cargo run --release --bin bozic_validation
```

**Expected wall time:** ~95 seconds (varies by CPU).

**Expected reference values:**

| Arm | Efficiency | Suppression | Tolerance |
|-----|-----------|-------------|-----------|
| no_drug | 1.000 | 0.0% | Exact |
| mono_A (400 Hz) | 0.481 | 51.9% | +/- 0.02 |
| mono_B (300 Hz) | 0.635 | 36.5% | +/- 0.02 |
| combo_AB | 0.435 | 56.5% | +/- 0.02 |
| double_A (2x conc) | 0.466 | 53.4% | +/- 0.02 |

**Verification criteria (all must hold):**

1. `combo_AB.efficiency < mono_A.efficiency` (combo suppresses more than mono A)
2. `combo_AB.efficiency < mono_B.efficiency` (combo suppresses more than mono B)
3. `combo_AB.efficiency < double_A.efficiency` (combo suppresses more than doubled dose)
4. `mono_A.efficiency < no_drug.efficiency` (mono A suppresses vs. control)
5. `mono_B.efficiency < no_drug.efficiency` (mono B suppresses vs. control)
6. All orderings confirmed in 10/10 seeds (check final summary output)

The binary prints a per-generation timeline followed by a summary table. The summary table shows final efficiency, alive count, resistance detection, and resistance generation for each arm.

### 3.4 Experiment 6: Adaptive Therapy Controller

**Claim:** Feedback-based drug modulation stabilizes growth at zero in a majority of seeds.

```bash
cargo run --release --bin adaptive_therapy
```

**Expected wall time:** ~8 seconds.

**Expected reference values:**

| Metric | Expected | Tolerance |
|--------|----------|-----------|
| Final stability | "stable" or "partial" | -- |
| Stability generation | 8-15 (when achieved) | +/- 5 |
| Max drugs used | 1-3 | -- |

**Verification criteria:**

1. Binary runs without error.
2. Output shows per-generation growth rate trajectory.
3. Growth rate approaches zero in later generations.
4. Summary reports final stability status.

**Note:** The adaptive therapy controller is stochastic in the sense that different configurations of initial conditions (driven by the deterministic seed) may produce different stability outcomes. The claim is 7/10 seeds — run the experiment multiple times with different configs if verifying the 7/10 claim. The default config runs a single seed.

To verify the multi-seed claim, the unit tests in the source file provide the authoritative check:

```bash
cargo test adaptive -- --test-threads=1
```

### 3.5 Experiment 7: Canine Mast Cell Tumor (Rosie Case)

**Claim:** Simulation predicts partial response with resistant fraction persistence.

The Rosie case is not a standalone binary. It is verified through the clinical calibration test suite:

```bash
cargo test clinical_calibration -- --test-threads=1
```

**Expected output:** 21 tests pass, including:

| Test name | What it verifies |
|-----------|-----------------|
| `mast_cell_doubling_21_days` | Doubling time = 21 days (London & Seguin 2003) |
| `mast_cell_toceranib_ic50_40nm` | IC50 = 40 nM (London et al. 2009) |
| `rosie_6_weeks_is_2_generations` | 42 days / 21 days = 2 generations |
| `rosie_responsive_resistant_split` | 70% responsive / 30% resistant |
| `rosie_calibrated_protocol` | 0.40 concentration = 16 nM at 21-day generation |
| `rosie_tumor_cell_count` | 128 entities = ~10^8 cells |
| `rosie_snapshot_at_response` | Day 42, doubling time extended to >80 days |

### 3.6 Headless Simulation

**Claim:** RESONANCE can run a full simulation without GPU and produce image output.

```bash
cargo run --release --bin headless_sim -- --ticks 10000 --scale 8 --out world.ppm
```

**Expected output:**

- File `world.ppm` created in current directory
- PPM image file (viewable with any image viewer: `open world.ppm` on macOS, `xdg-open world.ppm` on Linux)
- File size > 0 bytes

```bash
ls -la world.ppm
# Expected: -rw-r--r-- ... world.ppm (file exists, non-zero size)
file world.ppm
# Expected: world.ppm: Netpbm image data, size = ...
```

## 4. Determinism Verification Protocol

### 4.1 Principle

RESONANCE uses no external randomness. All stochasticity derives from `src/blueprint/equations/determinism.rs`, which implements a PCG-like deterministic state step seeded from entity indices. Running the same binary with the same parameters on the same platform must produce byte-identical output.

### 4.2 Procedure

```bash
# Run Bozic validation twice, capture output
cargo run --release --bin bozic_validation > run1.txt 2>&1
cargo run --release --bin bozic_validation > run2.txt 2>&1

# Compare outputs (excluding timing lines if present)
diff run1.txt run2.txt
```

**Expected result:** `diff` produces no output (files are identical).

If `diff` shows differences, they should only appear in wall-time measurements (which depend on system load). All numerical values (efficiency, suppression, alive count) must be identical.

**Stricter check (numeric values only):**

```bash
# Extract only numeric result lines
grep -E "^[0-9]|eff|alive|combo|mono|no_drug|double" run1.txt > run1_nums.txt
grep -E "^[0-9]|eff|alive|combo|mono|no_drug|double" run2.txt > run2_nums.txt
diff run1_nums.txt run2_nums.txt
```

**Expected result:** Empty diff.

### 4.3 Cross-Platform Verification

Due to floating-point operation ordering differences in parallel reductions (rayon), cross-platform results may differ in the least significant bits. This is expected and does not indicate a determinism failure. The orderings (combo < mono, combo < double) must hold on all platforms.

To verify platform-specific determinism, run the same binary twice on the same machine and confirm byte-identical numerical output.

## 5. Seed Specification

### 5.1 Determinism Model

RESONANCE does not use a random number generator crate. All randomness derives from entity index via hash-based deterministic functions:

```rust
// src/blueprint/equations/determinism.rs
pub fn next_u64(state: u64) -> u64 {
    state.wrapping_mul(6_364_136_223_846_793_005)
         .wrapping_add(1_442_695_040_888_963_407)
}
```

The seed for each entity is derived from its index (spawn order). The seed for each world in the batch simulator is derived from the experiment seed combined with the world index.

### 5.2 Default Seeds

| Experiment | Default seed | Source |
|-----------|-------------|--------|
| Pathway inhibitor (Exp 4) | 42 | `InhibitorConfig::default()` in `pathway_inhibitor_exp.rs` |
| Bozic validation (Exp 5) | 42 | `BozicValidationConfig::default()` in `pathway_inhibitor_exp.rs` |
| Adaptive therapy (Exp 6) | 42 (inherited from Bozic config) | `BozicValidationConfig::default()` |
| Headless sim | Deterministic from spawn order | Entity index-based |

### 5.3 Multi-Seed Runs

The Bozic validation binary runs 10 seeds internally (seed 42 through 51, or entity-index-derived variants). The "10/10 seeds confirm" claim is verified within the single binary execution — the user does not need to manually run 10 separate executions.

For the adaptive therapy controller, multi-seed verification is performed by the test suite:

```bash
cargo test adaptive_multi_seed -- --test-threads=1
```

## 6. Expected Output Reference Tables

### 6.1 Full Test Suite Summary

```
test result: ok. 3113 passed; 0 failed; 1 ignored; 0 measured; 0 filtered out
```

### 6.2 Bozic Validation Summary (Default Seed)

```
arm        | eff   | suppression
-----------|-------|------------
no_drug    | 1.000 | 0.0%
mono_A     | 0.481 | 51.9%
mono_B     | 0.635 | 36.5%
combo_AB   | 0.435 | 56.5%
double_A   | 0.466 | 53.4%
```

All values may vary by +/- 0.02 across platforms due to parallel reduction ordering. Orderings are invariant.

### 6.3 Dose-Response Summary (Default Seed)

```
concentration | efficiency | suppression
-------------|-----------|------------
0.0 (control) | 1.000    | 0.0%
0.4           | ~0.488   | ~51.2%
0.8           | ~0.471   | ~52.9%
```

### 6.4 Clinical Calibration Test Summary

```
running 21 tests
...
test result: ok. 21 passed; 0 failed; 0 ignored; 0 measured
```

Key verified values:

| Test | Verified value |
|------|---------------|
| CML doubling time | 4.0 days |
| CML IC50 | 260.0 nM |
| Prostate doubling time | 30.0 days |
| Canine MCT doubling time | 21.0 days |
| Canine MCT IC50 | 40.0 nM |
| Rosie: 6 weeks = 2 generations | 42 / 21 = 2 |
| Rosie: 70/30 split of 45 entities | 32 responsive, 13 resistant |

## 7. Troubleshooting

### 7.1 Build Failures

| Symptom | Cause | Solution |
|---------|-------|----------|
| `error: edition 2024 is not supported` | Rust version < 1.85 | `rustup update stable` |
| `Cargo.lock mismatch` | Dependencies not locked | Ensure `Cargo.lock` is checked in; run `cargo update` only if instructed |
| `linker errors` | Missing system libraries (Bevy dependencies) | Install system libraries per Bevy 0.15 docs: `libudev-dev`, `libasound2-dev` (Linux) |
| `out of memory during compilation` | < 4 GB RAM | Close other programs; use `cargo build --release -j 2` to limit parallelism |

### 7.2 Test Failures

| Symptom | Cause | Solution |
|---------|-------|----------|
| Test count differs from 3,113 | Wrong commit | `git log --oneline -1` must show `971c7ac` |
| Specific test fails | Platform-specific float difference | Check if failure is in tolerance (<1e-5). Report if not. |
| `proptest` tests time out | Slow machine | Increase timeout: `PROPTEST_MAX_SHRINK_TIME=300 cargo test property_conservation` |

### 7.3 Experiment Output Differs

| Symptom | Cause | Solution |
|---------|-------|----------|
| Efficiency values differ by > 0.02 | Wrong commit or modified source | Verify clean checkout: `git status` shows no modifications |
| Ordering differs (combo > mono) | Critical bug or wrong binary | Rebuild: `cargo build --release` and re-run |
| Wall time very different | Expected; depends on hardware | Only numerical values matter, not timing |
| `diff` shows timing differences only | Normal; system load affects wall time | Use numeric-only diff (Section 4.2) |

### 7.4 Platform-Specific Notes

**macOS (Apple Silicon / aarch64):** Floating-point results are deterministic within platform. Cross-platform comparison with x86_64 may show LSB differences due to different FMA instruction behavior. Orderings are invariant.

**Linux (x86_64):** Reference platform. All reference values in this document were generated on x86_64.

**Windows:** Supported but less tested. Ensure Rust stable toolchain is installed via `rustup`. Use PowerShell or WSL2 for command execution.

## 8. Codebase References

| Reference | File |
|-----------|------|
| Deterministic RNG | `src/blueprint/equations/determinism.rs` |
| Bozic validation binary | `src/bin/bozic_validation.rs` |
| Adaptive therapy binary | `src/bin/adaptive_therapy.rs` |
| Pathway inhibitor binary | `src/bin/pathway_inhibitor.rs` |
| Headless simulation binary | `src/bin/headless_sim.rs` |
| Cancer therapy binary | `src/bin/cancer_therapy.rs` |
| Experiment harness | `src/use_cases/experiments/pathway_inhibitor_exp.rs` |
| Clinical calibration | `src/blueprint/equations/clinical_calibration.rs` |
| Conservation fuzz tests | `tests/property_conservation.rs` |
| Derived thresholds | `src/blueprint/equations/derived_thresholds.rs` |
| Default configs | `src/use_cases/experiments/pathway_inhibitor_exp.rs` (InhibitorConfig, BozicValidationConfig) |

## 9. Revision History

| Version | Date | Author | Change Description |
|---------|------|--------|--------------------|
| 1.0 | 2026-04-02 | Resonance Development Team | Initial reproducibility protocol. Environment requirements, exact commands for all experiments, determinism verification procedure, reference output tables, seed specification, troubleshooting guide. All commands verified at commit `971c7ac`. |

---

**Approvals:**

| Role | Name | Date | Signature |
|------|------|------|-----------|
| Author | Resonance Development Team | 2026-04-02 | _draft -- not signed_ |
| Reviewer | _pending_ | _pending_ | _pending_ |
| Approver | _pending_ | _pending_ | _pending_ |
