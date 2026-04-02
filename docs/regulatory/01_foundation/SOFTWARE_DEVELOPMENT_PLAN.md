---
document_id: RD-1.4
title: Software Development Plan
standard: IEC 62304:2006+Amd1:2015 §5.1
version: 1.0
date: 2026-04-02
status: DRAFT
author: Resonance Development Team
---

# Software Development Plan

## 1. Purpose

This document defines the processes, methods, tools, roles, and deliverables governing the development, verification, and maintenance of RESONANCE — an emergent life simulation engine in Rust/Bevy. It satisfies IEC 62304:2006+Amd1:2015 §5.1 (Software Development Planning) and serves as the binding reference for all software lifecycle activities.

RESONANCE is developed iteratively through sprint-based cycles. This plan codifies the practices already embedded in the codebase and project infrastructure, retroactively formalizing them for regulatory traceability.

**Scope of application:**

- All source code under `src/` (113K LOC, Rust 2024 edition)
- All pure math in `src/blueprint/equations/` (50+ domain files)
- All batch simulation code in `src/batch/` (33 stateless systems, zero Bevy dependency)
- All test infrastructure (3,113 tests across unit, integration, property, and batch layers)
- Configuration files (`Cargo.toml`, `Cargo.lock`, `assets/maps/*.ron`)
- Sprint documentation (`docs/sprints/`, `docs/sprints/archive/`)
- Design specifications (`docs/design/`, `docs/arquitectura/`)

**Out of scope:** Rendering-only code in `Update` schedule (visual derivation, not simulation logic), third-party crate internals (SOUP — see RD-3).

## 2. Development Model

### 2.1 Iterative Sprint-Based Lifecycle

RESONANCE follows an iterative, sprint-based development model. Each sprint is a self-contained unit of work with explicit scope, deliverables, and closure criteria. This model maps to IEC 62304 §5.1.1(a) — the software development life cycle model.

**Sprint lifecycle:**

```
Scope Definition → Design → Implementation → Testing → Review → Archive
      │                                                          │
      └──── Sprint README.md ────────────────────────── Closure criteria verified by grep
```

**Phases within each sprint:**

| Phase | Activity | Artefact |
|-------|----------|----------|
| Scope | Define objectives, entregables, what enters/exits | Sprint README.md |
| Design | Decompose into layers, validate orthogonality, identify equations | Design notes in sprint doc |
| Implement | Write code respecting CLAUDE.md rules, 14 layers, Phase assignment | `src/` files |
| Test | Unit tests (pure math), integration tests (MinimalPlugins), property tests | `#[cfg(test)]` modules |
| Review | Verificador checklist: contract, math, DOD, determinism, perf, tests | PASS/WARN/BLOCK verdict |
| Archive | Verify closure criteria (grep-based), move to `docs/sprints/archive/` | Archived sprint doc |

**Reference:** Sprint methodology is documented in `docs/sprints/SPRINT_PRIMER.md`. Active sprints are tracked in `docs/sprints/README.md`. Completed sprints (49 tracks, 78 sprints archived) reside in `docs/sprints/archive/`.

### 2.2 Sprint Tracking and Waves

Sprints are organized into tracks (thematic groupings) and waves (dependency-ordered batches). Waves enforce that blocking dependencies are satisfied before downstream sprints begin.

| Wave | Status | Tracks |
|------|--------|--------|
| Wave 1 | Completed (2026-03-25) | SIMULATION_FOUNDATIONS, GAMEPLAY_SYSTEMS (partial), SIMULATION_QUALITY |
| Wave 2 | Unblocked | GAMEPLAY_SYSTEMS (GS-2, GS-4, GS-6) |
| Wave 3 | Blocked by Wave 2 | GAMEPLAY_SYSTEMS (GS-7, GS-8, GS-9) |
| Wave 4 | Blocked by Wave 3 | DEMO (Proving Grounds) |

Current state: 112 total sprints across all tracks. 78 archived, 33 active/designed, 1 blocked.

### 2.3 Regulatory Documentation Track

The REGULATORY_DOCUMENTATION track (RD-1 through RD-7) introduces formal regulatory artefacts as sprints within the same methodology. This plan (RD-1.4) is a deliverable of sprint RD-1.

**Sprint reference:** `docs/sprints/REGULATORY_DOCUMENTATION/SPRINT_RD1_REGULATORY_FOUNDATION.md`

## 3. Roles and Responsibilities

Four roles govern the development process. Each maps to IEC 62304 responsibilities.

| Role | RESONANCE Name | Responsibility | IEC 62304 Mapping |
|------|---------------|----------------|-------------------|
| Developer | **Alquimista** | Write code respecting 14 layers, Phase assignment, equations in `blueprint/equations/`. Output: impact analysis, code, plugin registration. | Software Developer (§5.1.1(d)) |
| Reviewer | **Observador** | Review for DOD violations, math correctness, pipeline ordering, performance, Bevy 0.15 compliance. | Software Reviewer (§5.5.2) |
| Planner | **Planificador** | Decompose features into layers, validate orthogonality, define interaction matrix. Output: data flow, systems, events, equations, archetypes, risks. | Software Architect / Project Lead (§5.1.1(b)) |
| Verifier | **Verificador** | PR-level verification following strict sequence: (1) contract, (2) math, (3) DOD, (4) determinism, (5) performance, (6) tests. Verdict: PASS / WARN / BLOCK. Math or determinism doubt triggers BLOCK. | Software Verifier (§5.7) |

**Role assignment:** Roles are functional, not organizational. A single contributor may hold multiple roles across different sprints but must not self-verify (the Verificador for a sprint must not be the same person who wrote the code under review).

**Escalation:** BLOCK verdicts from the Verificador halt the sprint. Resolution requires re-implementation and re-verification. No BLOCK may be overridden without written justification.

**Reference:** Role definitions are codified in `CLAUDE.md` §Roles.

## 4. Coding Standards

### 4.1 Governing Document

All source code must conform to the coding standards defined in `CLAUDE.md` at the repository root. This file is version-controlled and constitutes the normative reference for code quality.

### 4.2 The 14 Coding Rules

| # | Rule | Rationale |
|---|------|-----------|
| 1 | English identifiers only | External linter translates Spanish to English |
| 2 | Max 4 fields per component | Prevents component bloat; enforces layer decomposition |
| 3 | One system, one transformation | No god-systems (>5 component types) |
| 4 | `SparseSet` for transient components | Buffs, markers, one-shot flags use sparse storage |
| 5 | Guard change detection | `if val != new { val = new; }` or `set_if_neq` |
| 6 | Chain events with ordering | `.before()` or `.chain()` — never unordered producers/consumers |
| 7 | Phase assignment required | Every gameplay system must be in `.in_set(Phase::X)` |
| 8 | Math in `blueprint/equations/` | Systems call pure functions; no inline formulas |
| 9 | Component group factories for spawning | Pure functions returning tuples; composable nested bundles |
| 10 | Constants in constants modules | Tuning values centralized per module; algorithmic arrays stay in-file |
| 11 | `With<T>`/`Without<T>` over `Option<&T>` | Use filter-only queries for components not read |
| 12 | Minimal query width | Only request components actually read/written |
| 13 | No `Vec<T>` in components | Unless genuinely variable-length |
| 14 | Full derive set on components | `#[derive(Component, Reflect, Debug, Clone)]` + `register_type` |

### 4.3 Hard Blocks — Absolute Prohibitions

These rules have zero tolerance. No exceptions, no DEBT justification accepted.

| Block | Prohibition | Justification |
|-------|-------------|---------------|
| HB-1 | NO `unsafe` | Memory safety is non-negotiable for simulation correctness |
| HB-2 | NO external crates without approval | Only dependencies listed in `Cargo.toml` are permitted |
| HB-3 | NO `async`/`await` | Bevy schedule only; no tokio/async-std |
| HB-4 | NO `Arc<Mutex<T>>` | Use `Resource` or `Local` for shared state |
| HB-5 | NO shared mutable state outside Resources | No `static mut`, no `lazy_static! { Mutex }` |

### 4.4 Hard Blocks — Strong Defaults

These rules may be violated only with inline `// DEBT: <reason>` justification. Unjustified violations are zero tolerance.

| Block | Default | Condition for Exception |
|-------|---------|------------------------|
| SD-1 | NO `HashMap` in hot paths | Prove it is hot with a benchmark; sorted `Vec` or entity indexing preferred |
| SD-2 | NO `String` in components | Use enums, `u32` IDs, or `&'static str` |
| SD-3 | NO `Box<dyn Trait>` in components | Use enums for closed sets |
| SD-4 | NO `#[derive(Bundle)]` | Bevy 0.15 uses tuples or `#[require(...)]` |
| SD-5 | NO `ResMut` when `Res` suffices | Minimize write locks |
| SD-6 | NO `unwrap()`/`expect()`/`panic!()` in systems | `let-else` or `if-let`; tests are exempt |
| SD-7 | NO inline formulas in systems | All math must reside in `blueprint/equations/` |
| SD-8 | NO storing derived values as components | Compute at point of use |
| SD-9 | NO trait objects for game logic | Components + systems only |
| SD-10 | NO component methods with side effects | Pure `&self` only; systems perform mutations |
| SD-11 | NO `Entity` as persistent ID | Strong newtype IDs required |
| SD-12 | NO gameplay systems in `Update` schedule | `FixedUpdate` + `Phase` for determinism (except visual derivation) |

### 4.5 Axiom Inviolability

The 8 foundational axioms and 4 fundamental constants are the constitution of the project. No change, feature, refactor, or optimization may contradict, bypass, or weaken any axiom. If a proposed change conflicts with an axiom, the change is wrong — not the axiom. This constraint is absolute and supersedes all other coding rules.

**Reference:** Axioms and constants are defined in `CLAUDE.md` §The 8 Foundational Axioms and §The 4 Fundamental Constants. Derived thresholds are implemented in `src/blueprint/equations/derived_thresholds.rs` (17 tests).

## 5. Development Environment

### 5.1 Language and Toolchain

| Component | Specification | Notes |
|-----------|--------------|-------|
| Language | Rust | 2024 edition |
| MSRV | 1.85 | Minimum Supported Rust Version |
| Engine | Bevy | 0.15.x (with `serialize` feature) |
| Math | glam 0.29 | Engine-agnostic via `src/math_types.rs` |
| Parallelism | rayon 1.10 | Batch simulator only; no async runtime |
| Serialization | serde 1 + ron 0.8 + serde_json 1 | Configuration and export |
| Hashing | fxhash 0.2 | Fast deterministic hashing |
| Terrain | noise 0.9 | Procedural noise generation |
| Navigation | oxidized_navigation 0.12 | Pathfinding |
| Collision | parry3d 0.17 | Physics collision |
| UI | bevy_egui 0.31 + egui_plot 0.29 | Debug/dashboard overlay |

### 5.2 Development Tools

| Tool | Purpose | Invocation |
|------|---------|------------|
| `cargo check` | Fast compilation verification | CI gate: must pass |
| `cargo clippy` | Lint analysis | CI gate: must pass with no warnings |
| `cargo test` | Run all 3,113 tests | CI gate: must pass; target <60 sec |
| `cargo bench --bench batch_benchmark` | Performance benchmarks | criterion HTML reports |
| `cargo run --bin headless_sim` | Headless simulation to PPM | No GPU required |

### 5.3 Supported Platforms

| Platform | Status | Notes |
|----------|--------|-------|
| macOS (Darwin, ARM64) | Primary development | Full GPU rendering + headless |
| Linux (x86_64) | Supported | Full GPU rendering + headless |
| Headless (no GPU) | Supported | `headless_sim` binary; batch simulator |

## 6. Configuration Management

### 6.1 Version Control

| Aspect | Practice |
|--------|----------|
| System | Git |
| Hosting | GitHub (`ResakaGit/RESONANCE`) |
| Branch strategy | Trunk-based development on `main` |
| Commit history | 134 commits (2025-03-25 to 2026-04-02) |
| Latest commit | `971c7ac` |

### 6.2 Dependency Management

| Mechanism | Purpose |
|-----------|---------|
| `Cargo.toml` | Declares dependencies with semver constraints |
| `Cargo.lock` | Pins exact dependency versions for reproducibility |
| Crate approval | New external crates require explicit approval (Hard Block HB-2) |

All dependencies are listed in `Cargo.toml`. The complete dependency tree is locked via `Cargo.lock`, ensuring bit-exact reproducibility of builds across environments.

### 6.3 Configuration Files

| File | Purpose | Format |
|------|---------|--------|
| `Cargo.toml` | Package metadata, dependencies, features, build profiles | TOML |
| `Cargo.lock` | Pinned dependency graph | TOML (auto-generated) |
| `assets/maps/*.ron` | World map definitions | RON (Rusty Object Notation) |
| `CLAUDE.md` | Coding standards, axioms, architecture reference | Markdown |

### 6.4 Versioning

The software version is declared in `Cargo.toml` (`version = "0.1.0"`). Semantic versioning (semver) is followed. The Rust edition is `2024` with MSRV `1.85`.

### 6.5 Asset and Binary Tracking

Evolved genome binaries (`assets/evolved/*.bin`) are tracked in version control for experiment reproducibility. Map configurations (`assets/maps/*.ron`) are likewise version-controlled. 22 binary targets are defined across `Cargo.toml` `[[bin]]` sections.

## 7. Verification and Validation Strategy

### 7.1 Overview

The V&V strategy is structured in four layers, each targeting a different scope and confidence level. This satisfies IEC 62304 §5.5 (Software Integration and Integration Testing) and §5.7 (Software Verification).

```
Layer 4: Headless Simulation (full pipeline, no GPU)
Layer 3: Property Tests (proptest fuzzing, conservation invariants)
Layer 2: Integration Tests (MinimalPlugins + spawn + update + assert)
Layer 1: Unit Tests (pure math in blueprint/equations/)
```

### 7.2 Layer 1 — Unit Tests (Pure Math)

**Scope:** All pure functions in `src/blueprint/equations/` (50+ domain files).

**Method:** `#[cfg(test)] mod tests` co-located with source. Each function is tested with:
- Boundary inputs (`qe=0`, `radius=0`, `frequency=0`)
- Invariant preservation (conservation, monotonicity, bounds)
- Known analytical results (e.g., LJ zero-crossing at r = 2^(1/6)sigma)
- Edge cases specific to the domain

**Naming convention:** `<function>_<condition>_<expected>` — e.g., `density_zero_radius_returns_zero`.

**No mocks.** Pure functions have no dependencies to mock.

**Key test files and counts:**

| File | Tests | Domain |
|------|-------|--------|
| `blueprint/equations/metabolic_genome.rs` | 80 | DAG metabolism, Hebb rewiring |
| `blueprint/equations/variable_genome.rs` | 62 | 4-32 gene genomes, Kleiber cost |
| `blueprint/equations/pathway_inhibitor.rs` | 32 | Drug inhibition, Hill kinetics |
| `blueprint/equations/codon_genome.rs` | 28 | Triplet translation, silent mutations |
| `blueprint/equations/protein_fold.rs` | 27 | HP lattice fold, contact maps |
| `blueprint/equations/multicellular.rs` | 27 | Cell adhesion, Union-Find colonies |
| `blueprint/equations/coulomb.rs` | 26 | Coulomb + Lennard-Jones potentials |
| `blueprint/equations/derived_thresholds.rs` | 17 | 4 constants to 40 thresholds |

### 7.3 Layer 2 — Integration Tests

**Scope:** Bevy ECS systems operating on real component data.

**Method:** Test harness using `MinimalPlugins`:
1. Create Bevy `App` with `MinimalPlugins`
2. Spawn entities with only the components needed for the test
3. Run ONE `app.update()` cycle
4. Assert delta on output components

**Constraint:** Each integration test exercises exactly one system transformation. No multi-tick chained tests (those are headless simulation).

### 7.4 Layer 3 — Property Tests (Proptest)

**Scope:** Conservation invariants and mathematical properties under arbitrary inputs.

**Method:** `proptest` crate (dev-dependency) with strategies generating random but valid inputs.

**Key file:** `tests/property_conservation.rs` — fuzzes conservation and pool equations with arbitrary inputs to verify:
- Energy conservation (sum never increases)
- Pool invariant (children never exceed parent)
- Dissipation positivity (loss is always non-negative)

### 7.5 Layer 4 — Batch Tests

**Scope:** The batch simulator (`src/batch/`), which runs the full simulation pipeline without Bevy.

**Statistics:** 156 tests covering 33 stateless systems, arena operations, genome manipulation, harness evolution, and bridge round-trips (GenomeBlob to Bevy components and back).

**Key characteristic:** Zero Bevy dependency. These tests verify simulation correctness independent of the ECS engine.

### 7.6 Headless Simulation

**Invocation:** `cargo run --bin headless_sim -- --ticks 10000 --scale 8 --out world.ppm`

**Purpose:** End-to-end verification of the full simulation pipeline producing a PPM image. Exercises all 6 phases (SimulationClockSet through MorphologicalLayer) without GPU rendering. Used for visual regression and long-run stability validation.

### 7.7 Benchmark Suite

**Tool:** criterion 0.5 (dev-dependency)

**Invocation:** `cargo bench --bench batch_benchmark`

**Purpose:** Performance regression detection. Measures throughput of batch simulation (worlds/second) and individual equation evaluation. HTML reports generated automatically.

### 7.8 Experiment Validation

Specific experiments serve as high-level validation of emergent behavior:

| Experiment | Binary | Validation Target |
|------------|--------|-------------------|
| Bozic 2013 | `bozic_validation` | Combo > mono therapy (10/10 seeds) |
| Pathway Inhibitor | `pathway_inhibitor` | 3 inhibition modes, off-target binding |
| Cancer Therapy | `cancer_therapy` | Cytotoxic Hill pharmacokinetics |
| Adaptive Therapy | `adaptive_therapy` | Control loop adapts to population state |
| Rosie Case | (via experiment framework) | Canine mast cell tumor partial response |

**Reference:** `src/use_cases/experiments/pathway_inhibitor_exp.rs` (18 tests), `src/use_cases/experiments/cancer_therapy.rs`.

### 7.9 Test Execution Summary

| Layer | Count | Location | Bevy Required | Execution Time |
|-------|-------|----------|---------------|----------------|
| Unit (pure math) | ~2,400 | `src/blueprint/equations/` | No | <10 sec |
| Integration (ECS) | ~400 | `src/simulation/`, `src/worldgen/` | Yes (MinimalPlugins) | <15 sec |
| Property (proptest) | ~50 | `tests/property_conservation.rs` | No | <5 sec |
| Batch | 156 | `src/batch/` | No | <5 sec |
| **Total** | **~3,113** | | | **<35 sec** |

Gate: `cargo test` must pass with zero failures before any sprint closure.

## 8. Software Architecture

### 8.1 Canonical Reference

The full software architecture is documented in `docs/ARCHITECTURE.md`. This section provides a summary for regulatory context.

### 8.2 Architectural Pattern

**Pattern:** Layered ECS with Vertical Slices. This is NOT hexagonal architecture. Components are the domain, systems are the use cases, Bevy is the infrastructure.

**Stateless-first:** Pure functions + Resources. Components hold state, systems transform it. All mathematical logic resides in `src/blueprint/equations/` as pure functions with no side effects.

### 8.3 The 14 Orthogonal ECS Layers

All entities are defined by composition of up to 14 orthogonal layers:

| Layer | Name | Purpose |
|-------|------|---------|
| L0 | BaseEnergy | Existence — qe (energy quantum) |
| L1 | SpatialVolume | Spatial extent — radius |
| L2 | OscillatorySignature | Wave signature — frequency, phase |
| L3 | FlowVector | Flow — velocity, dissipation |
| L4 | MatterCoherence | Structural integrity — state, bond energy |
| L5 | AlchemicalEngine | Mana processor — buffer, valves |
| L6 | AmbientPressure | Terrain — delta_qe, viscosity |
| L7 | WillActuator | Will — intent, channeling |
| L8 | AlchemicalInjector | Spell payload — projected qe, forced freq |
| L9 | MobaIdentity | Game rules — faction, tags, crit |
| L10 | ResonanceLink | Buff/debuff — effect to target |
| L11 | TensionField | Gravity/magnetic force at distance |
| L12 | Homeostasis | Frequency adaptation with qe cost |
| L13 | StructuralLink | Spring joint between entities |

**Orthogonality invariant:** Each layer has its own update rule, is orthogonal to all others, and its removal does not affect entities that lack it.

### 8.4 Simulation Pipeline

Simulation runs in `FixedUpdate` with a deterministic fixed timestep (`Time<Fixed>` / `SimulationTickPlugin`). Systems are assigned to ordered phases via `SystemSet`:

```
SimulationClockSet
  → Phase::Input
    → Phase::ThermodynamicLayer
      → Phase::AtomicLayer
        → Phase::ChemicalLayer
          → Phase::MetabolicLayer
            → Phase::MorphologicalLayer
```

Visual derivation (non-deterministic) runs in `Update`, strictly after simulation.

**Reference:** `src/simulation/pipeline.rs` (scheduling), `src/simulation/mod.rs` (Phase enum).

### 8.5 Module Decomposition

```
src/
├── blueprint/          Types, equations, constants, almanac, morphogenesis
│   ├── equations/      Pure math (50+ files, zero side effects)
│   └── constants/      Derived constants per domain
├── batch/              Headless batch simulator (no Bevy, rayon parallel)
├── layers/             14 ECS layers + auxiliaries (24+ files)
├── simulation/         Runtime systems (pipeline, emergence, lifecycle, metabolic)
├── entities/           EntityBuilder, archetypes, composition
├── plugins/            Bevy plugins (Simulation, Layers, Debug, Morphological)
├── worldgen/           V7 world generation (nucleus, field, materialization)
├── use_cases/          Experiments, orchestrators, export
├── rendering/          Quantized color system
├── runtime_platform/   Platform compatibility (tick, input, camera, HUD)
├── math_types.rs       Engine-agnostic glam re-exports
└── bin/                22 executable binaries
```

## 9. Risk Management Integration

### 9.1 Cross-Reference

A full risk management file per ISO 14971 will be produced in sprint RD-2 (`docs/sprints/REGULATORY_DOCUMENTATION/SPRINT_RD2_RISK_MANAGEMENT.md`). This section documents how the development plan integrates preventive risk controls.

### 9.2 Hard Blocks as Preventive Controls

The Hard Blocks defined in §4.3 and §4.4 function as design-level risk mitigations:

| Risk | Control | Hard Block |
|------|---------|------------|
| Memory corruption leading to incorrect simulation results | Zero `unsafe` code | HB-1 |
| Supply chain vulnerability via unvetted dependencies | Crate approval process | HB-2 |
| Non-deterministic simulation output | No `async`/`await`, no shared mutable state | HB-3, HB-4, HB-5 |
| Simulation state corruption via concurrent mutation | No `Arc<Mutex>`, no `static mut` | HB-4, HB-5 |
| Incorrect mathematical results from inline formulas | All math in `blueprint/equations/` with unit tests | SD-7 (Coding Rule 8) |
| Derived values drifting from source of truth | No storing derived values as components | SD-8 |
| Non-reproducible results across runs | Deterministic RNG (`determinism.rs`: hash-based, no `std::rand`) | HB-3, SD-12 |

### 9.3 Axiom Guard Rails

The 8 foundational axioms serve as architectural risk controls. Three derived axioms (Competition, Conservation, Emergence at Scale) exist explicitly as guard rails for design decisions:

- **Axiom 5 (Conservation):** Prevents creation of energy, ensuring simulation stability.
- **Axiom 6 (Emergence at Scale):** Prevents hardcoded behaviors, faction tags, and scripted templates — constraining the developer, not the physics.
- **Axiom 3 (Competition):** Prevents bypass of interference mechanics.

Axiom violations are treated as BLOCK-level findings during Verificador review.

### 9.4 Intended Use Boundaries

RESONANCE is a research tool, not a clinical decision support system. This boundary is enforced through:
- Disclaimers in README.md ("NOT a clinical tool", "Not validated against patient outcomes")
- Abstract energy units (qe), not molar concentrations or patient-specific data
- No molecular targets (no EGFR/BCR-ABL) — frequency-based abstractions only
- Paper (Zenodo) §5 Limitations explicitly documenting honest scope

## 10. Deliverables per Sprint

Each sprint must produce the following artefacts before closure:

### 10.1 Required Deliverables

| Deliverable | Location | Verification |
|-------------|----------|--------------|
| Sprint README.md | `docs/sprints/{TRACK}/SPRINT_{ID}.md` | Exists with scope, entregables, closure criteria |
| Source code | `src/` | Compiles (`cargo check`), lints clean (`cargo clippy`) |
| Unit tests | Co-located `#[cfg(test)] mod tests` | `cargo test` passes |
| Integration tests (if ECS systems added) | Co-located or `tests/` | `cargo test` passes |
| Closure criteria verification | Sprint README.md checkboxes | Each criterion verified by grep or test |
| Plugin registration (if new systems) | Appropriate plugin in `src/plugins/` | System runs in correct Phase |

### 10.2 Sprint Closure Process

1. **All closure criteria checkboxes checked** in the sprint README.md
2. **`cargo test` green** — zero failures across all 3,113+ tests
3. **`cargo clippy` clean** — no warnings
4. **Grep verification** — closure criteria that reference specific code patterns are verified by grep against the codebase (e.g., "no `unwrap()` in systems" verified by `grep -r 'unwrap()' src/simulation/`)
5. **Verificador review** — contract, math, DOD, determinism, performance, tests. PASS required.
6. **Archive** — sprint doc moved to `docs/sprints/archive/{TRACK}/`

### 10.3 Sprint Documentation Template

Each sprint README.md follows a consistent structure:

```
# {SPRINT_ID}: {Title}
Objetivo: ...
Estado: PENDIENTE | EN PROGRESO | COMPLETADO
Esfuerzo: Bajo | Medio | Alto
Bloqueado por: ...
Desbloquea: ...

## Entregables
(numbered list with standard reference, content description, source files)

## Scope definido
Entra: ...
NO entra: ...

## Criterios de cierre
- [ ] (verifiable criteria with grep or test reference)
```

## 11. Traceability

### 11.1 Traceability Chain

IEC 62304 §5.1.1(e) requires traceability between software requirements, design, implementation, and verification. RESONANCE maintains the following traceability chain:

```
Axiom / Requirement (CLAUDE.md, SRS)
  → Sprint doc (docs/sprints/{TRACK}/SPRINT_{ID}.md)
    → Source files (src/{module}/{file}.rs)
      → Tests (#[cfg(test)] mod tests in same file)
        → Closure criteria (grep verification in sprint doc)
          → Archive (docs/sprints/archive/{TRACK}/)
```

### 11.2 Forward Traceability (Requirement to Code)

Each sprint README.md lists specific entregables with references to source files. Example from AXIOMATIC_INFERENCE sprint:

```
Entregable: derived_thresholds module
  → File: src/blueprint/equations/derived_thresholds.rs
  → Tests: 17 tests in same file
  → Constants: ALL lifecycle thresholds derived from 4 fundamentals
```

### 11.3 Backward Traceability (Code to Requirement)

Source files reference their originating sprint or axiom via:
- Module-level doc comments citing the governing axiom
- Sprint ID references in commit messages
- `CLAUDE.md` §Key Files mapping files to their architectural role

### 11.4 Test Traceability

Test names encode their target function and expected behavior: `<function>_<condition>_<expected>`. This naming convention enables automated traceability from test to requirement by function name.

### 11.5 Full Traceability Matrix

A complete requirements-to-tests traceability matrix will be produced in sprint RD-3 (`docs/sprints/REGULATORY_DOCUMENTATION/SPRINT_RD3_TRACEABILITY_SOUP.md`). The current plan establishes the methodology; RD-3 populates the matrix.

## 12. Maintenance and Problem Resolution

### 12.1 Corrective Actions

Defects discovered post-sprint-closure are addressed through the same sprint methodology:

1. **Identify** — defect reported via issue or discovered during development
2. **Scope** — create a new sprint (or add to an active sprint) with the defect as a closure criterion
3. **Fix** — implement the correction following all coding rules and hard blocks
4. **Verify** — add regression test covering the defect's root cause
5. **Close** — sprint closure with Verificador review

### 12.2 Regression Prevention

Every defect fix MUST include a regression test. The test is co-located with the source module and follows the naming convention `<function>_<defect_condition>_<correct_behavior>`.

The full test suite (`cargo test`, 3,113+ tests) serves as the regression gate. No sprint may close with a failing test.

### 12.3 Preventive Actions

The Inference Protocol defined in `CLAUDE.md` §Inference Protocol (Strict) mandates proactive critique:

- **Critique First, Validate Second:** Every change is evaluated for cost, breakage, and alternatives before implementation.
- **Red Lines — Auto-Trigger Critique:** Premature abstraction, scope creep, perfectionism loops, missing gameplay evidence, and orphan components trigger automatic pushback.
- **Challenge Assumptions:** Design decisions driven by aesthetic preference over functional need are flagged.

### 12.4 Configuration Change Control

Changes to the 4 fundamental constants or 8 axioms are prohibited by design. Changes to `CLAUDE.md` coding rules require explicit justification and are tracked in version control.

Changes to `Cargo.toml` dependencies require crate approval (Hard Block HB-2).

## 13. Codebase References

### 13.1 Governing Documents

| Document | Path | Content |
|----------|------|---------|
| Coding Standards & Axioms | `CLAUDE.md` | 14 coding rules, hard blocks, 8 axioms, 4 constants, roles |
| Architecture | `docs/ARCHITECTURE.md` | Canonical module map, pipeline, drug models, emergence status |
| Sprint Primer | `docs/sprints/SPRINT_PRIMER.md` | 7 rules, pipeline, file structure, merge checklist |
| Sprint Backlog | `docs/sprints/README.md` | Active/archived sprints, wave tracking, global metrics |
| Sprint Archive | `docs/sprints/archive/README.md` | Completed sprint index with test counts and dates |

### 13.2 Key Source Files

| File | Role |
|------|------|
| `src/simulation/pipeline.rs` | Scheduling and phase ordering |
| `src/simulation/mod.rs` | `Phase` enum, `InputChannelSet` |
| `src/blueprint/equations/mod.rs` | Pure math facade (50+ domain re-exports) |
| `src/blueprint/equations/derived_thresholds.rs` | All lifecycle constants from 4 fundamentals |
| `src/blueprint/equations/determinism.rs` | Hash-based RNG (no `std::rand`), bit-exact |
| `src/blueprint/constants/mod.rs` | Physics constants facade |
| `src/layers/mod.rs` | Layer re-exports (14 orthogonal layers) |
| `src/batch/mod.rs` | Batch simulator entry point (33 systems, zero Bevy) |
| `src/math_types.rs` | Engine-agnostic glam re-exports |
| `Cargo.toml` | Package metadata, dependencies, features |
| `Cargo.lock` | Pinned dependency graph |

### 13.3 Test Infrastructure

| File / Directory | Content |
|------------------|---------|
| `src/blueprint/equations/*.rs` | Co-located `#[cfg(test)]` unit tests |
| `src/batch/**/*.rs` | 156 batch simulator tests |
| `tests/property_conservation.rs` | Proptest fuzzing for conservation invariants |
| `src/use_cases/experiments/pathway_inhibitor_exp.rs` | 18 experiment validation tests |

### 13.4 Regulatory Documents (This Track)

| Document ID | Title | Path |
|-------------|-------|------|
| RD-1.1 | Intended Use | `docs/regulatory/01_foundation/INTENDED_USE.md` |
| RD-1.2 | Software Safety Classification | `docs/regulatory/01_foundation/SOFTWARE_SAFETY_CLASS.md` |
| RD-1.3 | Software Requirements Specification | `docs/regulatory/01_foundation/SOFTWARE_REQUIREMENTS_SPEC.md` |
| RD-1.4 | Software Development Plan | `docs/regulatory/01_foundation/SOFTWARE_DEVELOPMENT_PLAN.md` (this document) |
| RD-1.5 | Regulatory Strategy | `docs/regulatory/01_foundation/REGULATORY_STRATEGY.md` |

## 14. Revision History

| Version | Date | Author | Changes |
|---------|------|--------|---------|
| 1.0 | 2026-04-02 | Resonance Development Team | Initial release — retroactive formalization of existing development practices per IEC 62304 §5.1 |
