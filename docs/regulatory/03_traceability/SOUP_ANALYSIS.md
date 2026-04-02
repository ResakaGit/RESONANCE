---
document_id: RD-3.2
title: SOUP Analysis
standard: IEC 62304:2006+Amd1:2015 §5.3.3
version: 1.0
date: 2026-04-02
status: DRAFT
author: Resonance Development Team
commit: 971c7acb99decde45bf28860e6e10372718c51e2
approved_by: PENDING
review_date: PENDING
review_status: PENDING
---

# SOUP Analysis

## 1. Purpose

This document provides the formal analysis of all Software of Unknown Provenance (SOUP) used by RESONANCE. IEC 62304:2006+Amd1:2015 §5.3.3 requires that SOUP items be identified, their known anomalies documented, and the risk of SOUP anomalies assessed in the context of the software's safety classification.

RESONANCE is classified as IEC 62304 Class A (RD-1.2). For Class A software, IEC 62304 requires:
- Identification of each SOUP item (§5.3.3(a))
- The SOUP item's title, manufacturer, and unique designation (§5.3.3(b))
- Documentation of known anomalies relevant to the software (§5.3.3(c))

For higher classes (B/C), additional requirements apply (functional and performance requirements for each SOUP item, verification of SOUP in the integrated system). These are documented here voluntarily for regulatory preparedness.

**Cross-references:**
- RD-1.2: Software Safety Classification (Class A -- governs required SOUP analysis rigor)
- RD-1.4: Software Development Plan §4.3 (Hard Block HB-2: no external crates without approval)
- RD-2.1: Risk Management Plan (H-05: SOUP dependency with unpatched CVE)
- RD-3.3: SBOM (full dependency tree with hashes)
- RD-3.4: Configuration Management Plan (Cargo.lock pinning strategy)

## 2. SOUP Identification Method

SOUP items are identified from three sources:

1. **`Cargo.toml`** -- declares direct dependencies with semver constraints
2. **`Cargo.lock`** -- pins exact versions of all direct and transitive dependencies
3. **`cargo tree`** -- resolves the complete dependency graph

RESONANCE declares:
- **14 runtime dependencies** (including 1 optional: `minifb`)
- **5 dev-only dependencies** (testing and benchmarking; not shipped in release binaries)
- **~424 unique transitive crate packages** (resolved via `cargo tree --prefix none --no-dedupe | sort -u`)

Dev-dependencies are included in this analysis for completeness but are flagged as non-runtime. They cannot affect simulation output in release builds.

## 3. Runtime SOUP Analysis

### 3.1 bevy

| Field | Value |
|-------|-------|
| **Crate** | bevy |
| **Version** | 0.15.3 |
| **License** | MIT OR Apache-2.0 |
| **Source** | crates.io |
| **Checksum** | `2eaad7fe854258047680c51c3cacb804468553c04241912f6254c841c67c0198` |
| **Publisher** | Bevy Foundation / Carter Anderson |
| **Purpose** | ECS engine, rendering, input handling, scheduling, asset loading. Core runtime dependency -- RESONANCE's simulation pipeline, component system, and fixed-timestep scheduler all depend on Bevy. |
| **Used In** | All `src/` modules except `src/batch/` (batch simulator is Bevy-free), `src/blueprint/equations/` (pure math, Bevy-free), and `src/math_types.rs` (glam-only). |
| **Anomaly Risk** | **High** |
| **Justification** | Largest dependency by transitive crate count. Complex codebase (~300K LOC). Active development with breaking changes between minor versions. ECS scheduling bugs could cause non-deterministic system ordering. Rendering bugs are out of simulation scope. |
| **Known CVEs** | None in RustSec Advisory Database as of 2026-04-02. |
| **Mitigation** | Version pinned to 0.15.3 via `Cargo.lock`. Simulation correctness verified by 3,113 tests independent of rendering. Batch simulator (`src/batch/`) provides Bevy-free verification path (199 tests). Bevy scheduling verified via `Phase` system sets (`src/simulation/pipeline.rs`). |

### 3.2 glam

| Field | Value |
|-------|-------|
| **Crate** | glam |
| **Version** | 0.29.3 |
| **License** | MIT OR Apache-2.0 |
| **Source** | crates.io |
| **Checksum** | `8babf46d4c1c9d92deac9f7be466f76dfc4482b6452fc5024b5e8daf6ffeb3ee` |
| **Publisher** | Cameron Hart |
| **Purpose** | Linear algebra primitives (Vec2, Vec3, Quat, f32 operations). Decoupled from Bevy via `src/math_types.rs`. Used in all pure math and spatial calculations. |
| **Used In** | `src/math_types.rs` (re-exports), `src/blueprint/equations/` (spatial math), `src/batch/` (arena spatial ops), `src/worldgen/` (field grid). |
| **Anomaly Risk** | **Low** |
| **Justification** | Pure math library with no I/O, no threading, no unsafe in public API. Widely audited across the Rust gamedev ecosystem. Small API surface relevant to RESONANCE (Vec2, Vec3, basic f32 ops). |
| **Known CVEs** | None in RustSec Advisory Database as of 2026-04-02. |
| **Mitigation** | Version pinned. All spatial math verified by unit tests in `src/blueprint/equations/`. Determinism verified via `f32::to_bits()` hashing in `src/blueprint/equations/determinism.rs`. |

### 3.3 bytemuck

| Field | Value |
|-------|-------|
| **Crate** | bytemuck |
| **Version** | 1.25.0 |
| **License** | Zlib OR Apache-2.0 OR MIT |
| **Source** | crates.io |
| **Checksum** | `c8efb64bd706a16a1bdde310ae86b351e4d21550d98d056f22f8a7f7a2183fec` |
| **Publisher** | Lokathor (Daniel Keep) |
| **Purpose** | Safe transmutation of plain-old-data types. Used for GPU buffer layout (`src/worldgen/cell_field_snapshot/gpu_layout.rs`) and batch arena memory layout (`src/batch/arena.rs`). |
| **Used In** | `src/worldgen/cell_field_snapshot/`, `src/batch/arena.rs`, `src/batch/bridge.rs`. |
| **Anomaly Risk** | **Low** |
| **Justification** | Well-audited crate for safe type casting. Contains `unsafe` internally but exposes safe derive macros. RESONANCE uses `Pod` and `Zeroable` derives only. Not in simulation logic hot path. |
| **Known CVEs** | None in RustSec Advisory Database as of 2026-04-02. |
| **Mitigation** | Version pinned. Usage limited to data layout (GPU buffers, arena). 4 `unsafe` impls in RESONANCE are isolated from simulation logic and documented with `// DEBT:` justification per RS-01. |

### 3.4 serde

| Field | Value |
|-------|-------|
| **Crate** | serde |
| **Version** | 1.0.228 |
| **License** | MIT OR Apache-2.0 |
| **Source** | crates.io |
| **Checksum** | `9a8e94ea7f378bd32cbbd37198a4a91436180c5bb472411e48b5ec2e2124ae9e` |
| **Publisher** | David Tolnay |
| **Purpose** | Serialization/deserialization framework. Used for Bevy component reflection, RON map configuration loading, and JSON/CSV export. |
| **Used In** | Component `#[derive(Serialize, Deserialize)]` across `src/layers/`, map loading in `src/worldgen/map_config.rs`, export in `src/use_cases/export.rs`. |
| **Anomaly Risk** | **Low** |
| **Justification** | Most widely used Rust crate (~200M downloads). Extensively audited. No `unsafe` in derive macros. Stable API (1.x semver). Used only for configuration I/O, not simulation logic. |
| **Known CVEs** | None in RustSec Advisory Database as of 2026-04-02. |
| **Mitigation** | Version pinned. Serialization tested via round-trip tests in `src/batch/bridge.rs` (11 tests). |

### 3.5 serde_json

| Field | Value |
|-------|-------|
| **Crate** | serde_json |
| **Version** | 1.0.149 |
| **License** | MIT OR Apache-2.0 |
| **Source** | crates.io |
| **Checksum** | `83fc039473c5595ace860d8c4fafa220ff474b3fc6bfdb4293327f1a37e94d86` |
| **Publisher** | David Tolnay |
| **Purpose** | JSON serialization. Used in `src/use_cases/export.rs` for JSON export of simulation results. |
| **Used In** | `src/use_cases/export.rs` (JSON output). |
| **Anomaly Risk** | **Low** |
| **Justification** | Companion to serde. Stable, widely audited. Used only for output formatting, not simulation logic. |
| **Known CVEs** | None in RustSec Advisory Database as of 2026-04-02. |
| **Mitigation** | Version pinned. Export functions are stateless adapters verified by 9 unit tests. |

### 3.6 ron

| Field | Value |
|-------|-------|
| **Crate** | ron |
| **Version** | 0.8.1 |
| **License** | MIT OR Apache-2.0 |
| **Source** | crates.io |
| **Checksum** | `b91f7eff05f748767f183df4320a63d6936e9c6107d97c9e6bdd9784f4289c94` |
| **Publisher** | Ron Team (Tamschi) |
| **Purpose** | Rusty Object Notation -- configuration file format. Used for map configuration files (`assets/maps/*.ron`). |
| **Used In** | `src/worldgen/map_config.rs` (map deserialization), `assets/maps/*.ron` (25 map files). |
| **Anomaly Risk** | **Low** |
| **Justification** | Small crate for structured data format. Used only at startup for map loading. Malformed RON files produce deserialization errors caught by Bevy asset loading. |
| **Known CVEs** | None in RustSec Advisory Database as of 2026-04-02. |
| **Mitigation** | Version pinned. Map files version-controlled. Deserialization errors caught at startup. |

### 3.7 fxhash

| Field | Value |
|-------|-------|
| **Crate** | fxhash |
| **Version** | 0.2.1 |
| **License** | Apache-2.0 / MIT |
| **Source** | crates.io |
| **Checksum** | `c31b6d751ae2c7f11320402d34e41349dd1016f8d5d45e48c4312bc8625af50c` |
| **Publisher** | Chris Fallin |
| **Purpose** | Fast non-cryptographic hashing. Used for deterministic hashing in spatial indexing and internal data structures. |
| **Used In** | Spatial indexing, internal HashMap acceleration. Not used in `src/blueprint/equations/determinism.rs` (which uses its own hash-based RNG). |
| **Anomaly Risk** | **Low** |
| **Justification** | Minimal crate (~150 LOC). Pure math, no I/O, no threading. Deterministic for same input on same platform. |
| **Known CVEs** | None in RustSec Advisory Database as of 2026-04-02. |
| **Mitigation** | Version pinned. Simulation determinism verified independently via `hash_f32_slice` in `src/blueprint/equations/determinism.rs` (23 tests). |

### 3.8 tracing

| Field | Value |
|-------|-------|
| **Crate** | tracing |
| **Version** | 0.1.44 |
| **License** | MIT |
| **Source** | crates.io |
| **Checksum** | `63e71662fa4b2a2c3a26f570f037eb95bb1f85397f3cd8076caed2f026a6d100` |
| **Publisher** | Tokio project (Eliza Weisman, David Barsky) |
| **Purpose** | Structured logging and instrumentation. Used for debug-level tracing in simulation systems. |
| **Used In** | Various `src/simulation/` and `src/worldgen/` modules for diagnostic logging. |
| **Anomaly Risk** | **Low** |
| **Justification** | Standard Rust ecosystem logging crate. No effect on simulation output. Logging calls are zero-cost when subscriber is absent (compile-time optimization). |
| **Known CVEs** | None in RustSec Advisory Database as of 2026-04-02. |
| **Mitigation** | Version pinned. Tracing has no side effects on simulation correctness. |

### 3.9 noise

| Field | Value |
|-------|-------|
| **Crate** | noise |
| **Version** | 0.9.0 |
| **License** | Apache-2.0 / MIT |
| **Source** | crates.io |
| **Checksum** | `6da45c8333f2e152fc665d78a380be060eb84fad8ca4c9f7ac8ca29216cff0cc` |
| **Publisher** | Brendan Zabarauskas |
| **Purpose** | Procedural noise generation (Perlin, Simplex, etc.). Used for terrain noise in `src/topology/`. |
| **Used In** | `src/topology/` (terrain generation). |
| **Anomaly Risk** | **Low** |
| **Justification** | Pure math library for noise functions. No I/O, no threading, no unsafe. Deterministic for same seed. Affects terrain appearance, not entity physics. |
| **Known CVEs** | None in RustSec Advisory Database as of 2026-04-02. |
| **Mitigation** | Version pinned. Terrain noise is cosmetic (does not affect simulation conservation or entity physics). |

### 3.10 oxidized_navigation

| Field | Value |
|-------|-------|
| **Crate** | oxidized_navigation |
| **Version** | 0.12.0 |
| **License** | MIT OR Apache-2.0 |
| **Source** | crates.io |
| **Checksum** | `b003498f909e536c2b463416ced900306f76a37fd36227b0d4f05c4442ef8203` |
| **Publisher** | TheGrimsey |
| **Purpose** | Navigation mesh pathfinding (A* on nav meshes). Used for entity movement pathfinding. |
| **Used In** | `src/simulation/pathfinding/` (PathRequestEvent handling). |
| **Anomaly Risk** | **Medium** |
| **Justification** | Complex algorithm (navmesh generation + A* search). Pathfinding errors could cause entities to take incorrect paths, affecting simulation behavior. However, pathfinding is behavioral (not physics/conservation), so errors affect emergent behavior quality, not energy conservation or determinism. |
| **Known CVEs** | None in RustSec Advisory Database as of 2026-04-02. |
| **Mitigation** | Version pinned. Pathfinding is downstream of energy physics (does not affect conservation invariants). Entity movement is bounded by energy budget (basal drain). |

### 3.11 parry3d

| Field | Value |
|-------|-------|
| **Crate** | parry3d |
| **Version** | 0.17.6 |
| **License** | Apache-2.0 |
| **Source** | crates.io |
| **Checksum** | `6aeb9659a05b1783fb2e9bc94f48225ae5b40817eb45b62569c0e4dd767a6e51` |
| **Publisher** | Dimforge (Sebastien Crozet) |
| **Purpose** | 3D collision detection. Used for spatial queries and collision backend. |
| **Used In** | Collision detection backend (feature-gated: `v6_collision_backend_3d`). |
| **Anomaly Risk** | **Medium** |
| **Justification** | Complex geometric algorithms. Collision errors could affect entity interactions. However, collision is feature-gated and supplementary to the energy-based interaction model. Core simulation physics (energy transfer, conservation) does not depend on parry3d. |
| **Known CVEs** | None in RustSec Advisory Database as of 2026-04-02. |
| **Mitigation** | Version pinned. Feature-gated (`v6_collision_backend_3d`). Energy physics operates independently of collision detection. |

### 3.12 rayon

| Field | Value |
|-------|-------|
| **Crate** | rayon |
| **Version** | 1.11.0 |
| **License** | MIT OR Apache-2.0 |
| **Source** | crates.io |
| **Checksum** | `368f01d005bf8fd9b1206fb6fa653e6c4a81ceb1466406b81792d87c5677a58f` |
| **Publisher** | Josh Stone, Niko Matsakis |
| **Purpose** | Data parallelism. Used in batch simulator (`src/batch/batch.rs`) for parallel execution of multiple simulation worlds via `par_iter_mut`. |
| **Used In** | `src/batch/batch.rs` (WorldBatch parallel iteration). Not used in Bevy runtime simulation. |
| **Anomaly Risk** | **Medium** |
| **Justification** | Threading library. Thread scheduling non-determinism is inherent but mitigated by the design: each world in the batch is independent (no shared state between worlds). Within a single world, execution is sequential. Rayon only parallelizes across worlds, not within a world. |
| **Known CVEs** | None in RustSec Advisory Database as of 2026-04-02. |
| **Mitigation** | Version pinned. Used only in batch simulator (not Bevy runtime). Each world is independent -- no cross-world state. Within-world determinism verified by batch system tests (199 tests). Results are deterministic per-world regardless of thread scheduling. |

### 3.13 bevy_egui

| Field | Value |
|-------|-------|
| **Crate** | bevy_egui |
| **Version** | 0.31.1 |
| **License** | MIT |
| **Source** | crates.io |
| **Checksum** | `954fbe8551af4b40767ea9390ec7d32fe1070a6ab55d524cf0868c17f8469a55` |
| **Publisher** | Vladyslav Batyrenko |
| **Purpose** | egui integration for Bevy. Provides immediate-mode UI for debug dashboards and lab interfaces. |
| **Used In** | `src/plugins/debug_plugin.rs`, `src/runtime_platform/dashboard_bridge.rs` (debug/dashboard UI). |
| **Anomaly Risk** | **Medium** |
| **Justification** | UI rendering crate. Complex (bridges egui and Bevy rendering). UI bugs affect visual display but not simulation output. All simulation logic runs in `FixedUpdate`; egui runs in `Update`. |
| **Known CVEs** | None in RustSec Advisory Database as of 2026-04-02. |
| **Mitigation** | Version pinned. Excluded from simulation scope (rendering-only, per RD-2.1 §2.3). Does not affect simulation correctness. |

### 3.14 egui_plot

| Field | Value |
|-------|-------|
| **Crate** | egui_plot |
| **Version** | 0.29.0 |
| **License** | MIT OR Apache-2.0 |
| **Source** | crates.io |
| **Checksum** | `d8dca4871c15d51aadb79534dcf51a8189e5de3426ee7b465eb7db9a0a81ea67` |
| **Publisher** | egui team (Emil Ernerfeldt) |
| **Purpose** | Plot/chart rendering for egui. Used in dashboard for time-series visualization. |
| **Used In** | `src/runtime_platform/dashboard_bridge.rs` (time-series plots). |
| **Anomaly Risk** | **Low** |
| **Justification** | Chart rendering only. Small crate extending egui. No effect on simulation logic. |
| **Known CVEs** | None in RustSec Advisory Database as of 2026-04-02. |
| **Mitigation** | Version pinned. Rendering-only; excluded from simulation scope. |

### 3.15 minifb (optional)

| Field | Value |
|-------|-------|
| **Crate** | minifb |
| **Version** | 0.27.0 |
| **License** | MIT / Apache-2.0 |
| **Source** | crates.io |
| **Checksum** | `b0c470a74618b43cd182c21b3dc1e6123501249f3bad9a0085e95d1304ca2478` |
| **Publisher** | Daniel Collin |
| **Purpose** | Minimal framebuffer for headless pixel display. Optional dependency (feature: `pixel_viewer`). |
| **Used In** | Feature-gated `pixel_viewer` mode. Not compiled in default build. |
| **Anomaly Risk** | **Low** |
| **Justification** | Optional, not compiled by default. Display-only (framebuffer). No effect on simulation. |
| **Known CVEs** | None in RustSec Advisory Database as of 2026-04-02. |
| **Mitigation** | Optional feature. Not included in default build. Version pinned when enabled. |

---

## 4. Dev-Only SOUP Analysis

Dev-dependencies are compiled only for `cargo test` and `cargo bench`. They are not included in release binaries and cannot affect simulation output in production.

### 4.1 criterion

| Field | Value |
|-------|-------|
| **Crate** | criterion |
| **Version** | 0.5.1 |
| **License** | Apache-2.0 OR MIT |
| **Source** | crates.io |
| **Checksum** | `f2b12d017a929603d80db1831cd3a24082f8137ce19c69e6447f54f5fc8d692f` |
| **Purpose** | Statistical benchmarking framework. Used for `cargo bench --bench batch_benchmark` and 5 other benchmark targets. |
| **Runtime Impact** | **None.** Dev-dependency only. |
| **Known CVEs** | None. |

### 4.2 naga

| Field | Value |
|-------|-------|
| **Crate** | naga |
| **Version** | 23.1.0 |
| **License** | MIT OR Apache-2.0 |
| **Source** | crates.io |
| **Checksum** | `364f94bc34f61332abebe8cad6f6cd82a5b65cff22c828d05d0968911462ca4f` |
| **Purpose** | Shader language translator. Used in GPU-related tests (`gpu_cell_field_snapshot_palette_dispatch`). |
| **Runtime Impact** | **None.** Dev-dependency only. |
| **Known CVEs** | None. |

### 4.3 pollster

| Field | Value |
|-------|-------|
| **Crate** | pollster |
| **Version** | 0.4.0 |
| **License** | Apache-2.0 / MIT |
| **Source** | crates.io |
| **Checksum** | `2f3a9f18d041e6d0e102a0a46750538147e5e8992d3b4873aaafee2520b00ce3` |
| **Purpose** | Minimal async executor. Used in GPU test harnesses to block on async wgpu operations. |
| **Runtime Impact** | **None.** Dev-dependency only. |
| **Known CVEs** | None. |

### 4.4 proptest

| Field | Value |
|-------|-------|
| **Crate** | proptest |
| **Version** | 1.11.0 |
| **License** | MIT OR Apache-2.0 |
| **Source** | crates.io |
| **Checksum** | `4b45fcc2344c680f5025fe57779faef368840d0bd1f42f216291f0dc4ace4744` |
| **Purpose** | Property-based testing framework. Used in `tests/property_conservation.rs` (19 proptest cases testing conservation invariants). |
| **Runtime Impact** | **None.** Dev-dependency only. |
| **Known CVEs** | None. |
| **Note** | Proptest generates test evidence that directly mitigates H-02 (conservation bug). The framework itself is not runtime SOUP. |

### 4.5 wgpu

| Field | Value |
|-------|-------|
| **Crate** | wgpu |
| **Version** | 23.0.1 |
| **License** | MIT OR Apache-2.0 |
| **Source** | crates.io |
| **Checksum** | `80f70000db37c469ea9d67defdc13024ddf9a5f1b89cb2941b812ad7cde1735a` |
| **Purpose** | WebGPU API implementation. Used in GPU-related tests. |
| **Runtime Impact** | **None.** Dev-dependency only. |
| **Known CVEs** | None. |
| **Note** | wgpu is also a transitive dependency of bevy (runtime). The dev-dependency version (23.0.1) is the same version pulled transitively by bevy 0.15.3. |

---

## 5. Overall SOUP Risk Assessment

### 5.1 Risk Summary

| Risk Level | Crates | Count |
|------------|--------|-------|
| **High** | bevy | 1 |
| **Medium** | oxidized_navigation, parry3d, rayon, bevy_egui | 4 |
| **Low** | glam, bytemuck, serde, serde_json, ron, fxhash, tracing, noise, egui_plot, minifb | 10 |
| **None (dev-only)** | criterion, naga, pollster, proptest, wgpu | 5 |

### 5.2 High-Risk SOUP Assessment

**bevy (0.15.3)** is the only high-risk SOUP item. This risk is structural -- Bevy is the ECS engine upon which the entire runtime simulation depends. Mitigation:

1. **Version pinning:** Bevy is pinned to 0.15.3 via `Cargo.lock`. No automatic updates.
2. **Independent verification:** The batch simulator (`src/batch/`, 199 tests) implements the same physics without Bevy, providing an independent verification path.
3. **Test coverage:** 3,113 tests verify simulation correctness through Bevy's ECS.
4. **Schedule determinism:** All simulation systems use `FixedUpdate` with explicit `Phase` ordering (`src/simulation/pipeline.rs`), eliminating non-deterministic system ordering.
5. **No Bevy in pure math:** All equations in `src/blueprint/equations/` are Bevy-free pure functions.

### 5.3 Transitive Dependency Risk

RESONANCE's 14 direct runtime dependencies resolve to approximately 424 unique transitive crate packages (measured via `cargo tree --prefix none --no-dedupe | sort -u`). The majority are pulled by bevy (rendering, windowing, audio, input handling) and are not exercised in headless/batch mode.

**Key observation:** In headless mode (`cargo run --bin headless_sim`), most of bevy's rendering transitive dependencies are compiled but not executed. The batch simulator (`src/batch/`) has zero Bevy dependency and therefore zero transitive dependency risk from the Bevy tree.

### 5.4 Known CVE Status

As of 2026-04-02, no known CVEs exist in the RustSec Advisory Database for any of RESONANCE's direct dependencies. This was verified by review of https://rustsec.org/advisories/ for each direct dependency.

**Limitation:** This check covers only the RustSec database, which tracks Rust-specific advisories. It does not cover general CVE databases (NVD, GitHub Security Advisories) for transitive native dependencies (e.g., system libraries linked by windowing crates). A full `cargo audit` scan is recommended as part of the monitoring plan (§6).

---

## 6. SOUP Monitoring Plan

### 6.1 Monitoring Tools

| Tool | Purpose | Frequency |
|------|---------|-----------|
| `cargo audit` | Scan `Cargo.lock` against RustSec Advisory Database | Before each release, weekly in CI (when CI is established) |
| `cargo tree --duplicates` | Detect duplicate crate versions (version conflict indicator) | Before each dependency update |
| `cargo outdated` | List dependencies with available updates | Monthly review |
| RustSec Advisory Database (https://rustsec.org/) | Manual review of new advisories | Weekly (RSS or email subscription) |
| GitHub Dependabot alerts | Automated vulnerability alerts for repository dependencies | Continuous (when enabled on GitHub) |

### 6.2 Update Policy

| Scenario | Action |
|----------|--------|
| Security advisory (CVE) for a direct dependency | Evaluate within 48 hours. Update if fix available. If no fix, document mitigation and risk acceptance. |
| Security advisory for a transitive dependency | Evaluate within 1 week. Check if direct dependency update resolves it. |
| New minor/patch version of direct dependency | Evaluate changelog for relevant fixes. Update if low risk. Run full test suite. |
| New major version of direct dependency (esp. Bevy) | Treat as major release. Full regression testing. Dedicated sprint with Verificador review. |
| New dependency addition | Requires approval per Hard Block HB-2 (`CLAUDE.md`). Must be added to this SOUP analysis before merge. |

### 6.3 Acceptance Criteria for SOUP Updates

Before any SOUP update is merged:

1. `cargo test` passes with zero failures (3,113+ tests)
2. `cargo audit` reports no new advisories
3. Batch simulator tests pass (199 tests, Bevy-independent)
4. Determinism tests pass (`src/blueprint/equations/determinism.rs`, 23 tests)
5. Conservation tests pass (`tests/property_conservation.rs`, 19 proptest cases)
6. Updated version and checksum recorded in `Cargo.lock` (committed to Git)
7. This document (RD-3.2) updated with new version, checksum, and CVE status

---

## 7. License Compliance Summary

| License | Crates Using It | Compatible with AGPL-3.0? |
|---------|-----------------|---------------------------|
| MIT | tracing, bevy_egui, fxhash (dual) | Yes |
| MIT OR Apache-2.0 | bevy, glam, egui_plot, oxidized_navigation, rayon, ron, serde, serde_json, noise, criterion, naga, proptest, wgpu | Yes |
| Apache-2.0 | parry3d | Yes |
| Zlib OR Apache-2.0 OR MIT | bytemuck | Yes |
| Apache-2.0 / MIT | pollster | Yes |

All direct dependencies use permissive licenses (MIT, Apache-2.0, Zlib) that are compatible with RESONANCE's AGPL-3.0-or-later license. No copyleft dependencies (GPL, LGPL) are present among direct dependencies.

**Transitive dependency note:** A full license audit of all ~424 transitive dependencies has not been performed. This is recommended for RD-7 (Release Package). Preliminary review indicates all transitive crates in the Bevy ecosystem use MIT or Apache-2.0 licensing.

---

## 8. Revision History

| Version | Date | Author | Changes |
|---------|------|--------|---------|
| 1.0 | 2026-04-02 | Resonance Development Team | Initial SOUP analysis. 15 runtime dependencies + 5 dev dependencies analyzed. Risk levels assigned. Zero known CVEs. Monitoring plan established. |

---

**Approvals:**

| Role | Name | Date | Signature |
|------|------|------|-----------|
| Alquimista (Analyst) | Resonance Development Team | 2026-04-02 | _draft -- not signed_ |
| Verificador (Reviewer) | _pending_ | _pending_ | _pending_ |
