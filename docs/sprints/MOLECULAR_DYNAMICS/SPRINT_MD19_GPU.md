# MD-19: GPU Acceleration

**Effort:** 2 months | **Blocked by:** MD-7 | **Blocks:** —

## Problem

CPU force computation is O(N) with cell list but has a large constant factor.
For N > 10K particles (solvated proteins, large systems), CPU is too slow
for production runs.

## Solution

Port force computation kernel to GPU via wgpu compute shaders.
Only the force computation loop moves to GPU — the rest stays CPU.

## Decision Gate

**Before starting MD-19:**
1. Profile: is force computation actually the bottleneck (>90% wall time)?
2. Is N > 10K actually needed for the folding milestone?
3. Would SIMD (std::simd or packed_simd) give sufficient speedup?

**Not in critical path.** All milestones through MD-17 are achievable on CPU
with N < 5000 (Go model, small proteins).

## Alternatives to GPU

### SIMD (lower complexity)

```rust
use std::simd::f64x4;
// Process 4 pairs simultaneously
let dx = f64x4::from_array([dx0, dx1, dx2, dx3]);
let r_sq = dx * dx + dy * dy;
// ...
```

4-8x speedup for force computation. No GPU complexity.

### Rayon parallelism (already in batch)

Cell list can be parallelized: each cell computed independently.
Already have rayon in batch pipeline. Extend to MD forces.

## Implementation (if GPU chosen)

### `batch/gpu.rs`

- wgpu compute pipeline setup
- WGSL shader for LJ + Coulomb force computation
- Data transfer: positions/charges to GPU, forces back to CPU
- Double-buffering for overlap
- Fallback to CPU if GPU unavailable

## Acceptance Criteria

- [x] Decision: GPU, SIMD, or skip
- [x] Chosen approach implemented
- [x] N=10K feasible at > 1000 steps/second
- [x] Force results match CPU within epsilon
