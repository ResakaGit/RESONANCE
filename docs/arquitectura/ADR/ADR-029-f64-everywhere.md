# ADR-029: f64-Everywhere Migration for MD Pipeline

**Status:** Proposed | **Sprint:** MD_REFACTOR R3 | **Date:** 2026-04-13

## Context

The MD pipeline mixes f32 and f64:
- `EntitySlot` fields: f32 (legacy batch)
- `Topology` params (BondParams, AngleParams): f32
- Standalone MD worlds (LjWorld, PeptideWorld): f64
- `bonded.rs` functions: f32 params, f32 return
- `bonded_forces.rs`: casts f32→f64 and back

This creates truncation at every boundary. For 10M-step simulations, accumulated
truncation causes measurable energy drift.

## Options

### Option A: Migrate BondParams/AngleParams/DihedralParams to f64
- **Pro:** Eliminates all truncation in bonded force pipeline
- **Con:** Doubles memory for topology (but topology is cold data, not hot)
- **Effort:** 2 days
- **Breaking:** Yes — all code using BondParams needs update

### Option B: Keep f32 params, f64 intermediate computation
- **Pro:** No API change, minimal memory impact
- **Con:** Truncation still happens at param storage boundary
- **Effort:** 0 (current state)

### Option C: Generic `<F: Float>` trait for all equations
- **Pro:** Supports both f32 and f64 without duplication
- **Con:** Complexity explosion (generics everywhere), compile time
- **Effort:** 2 weeks
- **Violates:** KISS principle, Coding Rule "no premature abstraction"

## Recommendation

**Option A.** Topology is cold data — memory is irrelevant. The bonded force
pipeline is the hottest path in MD. Every f32↔f64 cast is wasted cycles and
precision. Make the cut once, cleanly.

Legacy batch (SimWorldFlat, EntitySlot) keeps f32 — it doesn't run MD.

## Axiom compliance

Axiom 1 (Everything is Energy): higher precision in energy computation
reduces phantom energy creation/destruction from truncation.
Axiom 2 (Pool Invariant): f64 makes conservation checks more meaningful.
