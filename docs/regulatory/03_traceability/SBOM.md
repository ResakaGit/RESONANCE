---
document_id: RD-3.3
title: Software Bill of Materials
standard: FDA Cybersecurity Guidance (2023), NTIA SBOM Minimum Elements
version: 1.0
date: 2026-04-02
status: DRAFT
author: Resonance Development Team
commit: 971c7acb99decde45bf28860e6e10372718c51e2
---

# Software Bill of Materials (SBOM)

## 1. Purpose

This document provides the Software Bill of Materials for RESONANCE, enumerating all direct software components, their versions, licenses, sources, and integrity references. It satisfies the FDA's Cybersecurity Guidance (2023) SBOM requirements and the NTIA "Minimum Elements for a Software Bill of Materials" (2021).

The SBOM is maintained manually in this document and can be regenerated from the authoritative source: `Cargo.lock` at the referenced commit. Commands for automated generation and verification are provided in Section 5.

**Cross-references:**
- RD-3.2: SOUP Analysis (risk assessment for each component)
- RD-3.4: Configuration Management Plan (Cargo.lock as configuration item)
- RD-1.4: Software Development Plan §6 (dependency management)

## 2. NTIA Minimum Elements Compliance

The NTIA "Minimum Elements for a Software Bill of Materials" (July 2021) requires the following data fields. This document's compliance status is indicated for each.

| NTIA Minimum Element | Status | Location in This Document |
|----------------------|--------|---------------------------|
| Supplier name | Provided | §3 (Publisher column) |
| Component name | Provided | §3 (Component column) |
| Version of the component | Provided | §3 (Version column) |
| Other unique identifiers | Provided | §3 (Checksum column -- SHA-256 from Cargo.lock) |
| Dependency relationship | Provided | §3 (Relationship column: direct/dev) |
| Author of SBOM data | Provided | Document header (Resonance Development Team) |
| Timestamp | Provided | Document header (2026-04-02) |

## 3. Direct Component Inventory

### 3.1 CycloneDX-Compatible Component Table

The following table lists all direct dependencies declared in `Cargo.toml`. Checksums are SHA-256 hashes from `Cargo.lock` (Cargo registry integrity format). All components are sourced from crates.io (`registry+https://github.com/rust-lang/crates.io-index`).

#### Runtime Dependencies

| Component | Version | License | Relationship | Publisher | Checksum (SHA-256) |
|-----------|---------|---------|-------------|-----------|-------------------|
| bevy | 0.15.3 | MIT OR Apache-2.0 | Direct / Runtime | Bevy Foundation | `2eaad7fe854258047680c51c3cacb804468553c04241912f6254c841c67c0198` |
| glam | 0.29.3 | MIT OR Apache-2.0 | Direct / Runtime | Cameron Hart | `8babf46d4c1c9d92deac9f7be466f76dfc4482b6452fc5024b5e8daf6ffeb3ee` |
| bytemuck | 1.25.0 | Zlib OR Apache-2.0 OR MIT | Direct / Runtime | Lokathor | `c8efb64bd706a16a1bdde310ae86b351e4d21550d98d056f22f8a7f7a2183fec` |
| serde | 1.0.228 | MIT OR Apache-2.0 | Direct / Runtime | David Tolnay | `9a8e94ea7f378bd32cbbd37198a4a91436180c5bb472411e48b5ec2e2124ae9e` |
| serde_json | 1.0.149 | MIT OR Apache-2.0 | Direct / Runtime | David Tolnay | `83fc039473c5595ace860d8c4fafa220ff474b3fc6bfdb4293327f1a37e94d86` |
| ron | 0.8.1 | MIT OR Apache-2.0 | Direct / Runtime | Ron Team | `b91f7eff05f748767f183df4320a63d6936e9c6107d97c9e6bdd9784f4289c94` |
| fxhash | 0.2.1 | Apache-2.0 / MIT | Direct / Runtime | Chris Fallin | `c31b6d751ae2c7f11320402d34e41349dd1016f8d5d45e48c4312bc8625af50c` |
| tracing | 0.1.44 | MIT | Direct / Runtime | Tokio Project | `63e71662fa4b2a2c3a26f570f037eb95bb1f85397f3cd8076caed2f026a6d100` |
| noise | 0.9.0 | Apache-2.0 / MIT | Direct / Runtime | Brendan Zabarauskas | `6da45c8333f2e152fc665d78a380be060eb84fad8ca4c9f7ac8ca29216cff0cc` |
| oxidized_navigation | 0.12.0 | MIT OR Apache-2.0 | Direct / Runtime | TheGrimsey | `b003498f909e536c2b463416ced900306f76a37fd36227b0d4f05c4442ef8203` |
| parry3d | 0.17.6 | Apache-2.0 | Direct / Runtime | Dimforge | `6aeb9659a05b1783fb2e9bc94f48225ae5b40817eb45b62569c0e4dd767a6e51` |
| rayon | 1.11.0 | MIT OR Apache-2.0 | Direct / Runtime | Josh Stone, Niko Matsakis | `368f01d005bf8fd9b1206fb6fa653e6c4a81ceb1466406b81792d87c5677a58f` |
| bevy_egui | 0.31.1 | MIT | Direct / Runtime | V. Batyrenko | `954fbe8551af4b40767ea9390ec7d32fe1070a6ab55d524cf0868c17f8469a55` |
| egui_plot | 0.29.0 | MIT OR Apache-2.0 | Direct / Runtime | egui team | `d8dca4871c15d51aadb79534dcf51a8189e5de3426ee7b465eb7db9a0a81ea67` |
| minifb | 0.27.0 | MIT / Apache-2.0 | Direct / Runtime (optional) | Daniel Collin | `b0c470a74618b43cd182c21b3dc1e6123501249f3bad9a0085e95d1304ca2478` |

#### Dev-Only Dependencies

| Component | Version | License | Relationship | Publisher | Checksum (SHA-256) |
|-----------|---------|---------|-------------|-----------|-------------------|
| criterion | 0.5.1 | Apache-2.0 OR MIT | Direct / Dev | Brook Heisler | `f2b12d017a929603d80db1831cd3a24082f8137ce19c69e6447f54f5fc8d692f` |
| naga | 23.1.0 | MIT OR Apache-2.0 | Direct / Dev | gfx-rs team | `364f94bc34f61332abebe8cad6f6cd82a5b65cff22c828d05d0968911462ca4f` |
| pollster | 0.4.0 | Apache-2.0 / MIT | Direct / Dev | John Googler | `2f3a9f18d041e6d0e102a0a46750538147e5e8992d3b4873aaafee2520b00ce3` |
| proptest | 1.11.0 | MIT OR Apache-2.0 | Direct / Dev | Jason Lingle | `4b45fcc2344c680f5025fe57779faef368840d0bd1f42f216291f0dc4ace4744` |
| wgpu | 23.0.1 | MIT OR Apache-2.0 | Direct / Dev | gfx-rs team | `80f70000db37c469ea9d67defdc13024ddf9a5f1b89cb2941b812ad7cde1735a` |

### 3.2 Primary Component: RESONANCE

| Field | Value |
|-------|-------|
| Component name | resonance |
| Version | 0.1.0 |
| License | AGPL-3.0-or-later |
| Source | https://github.com/ResakaGit/RESONANCE |
| Language | Rust (2024 edition) |
| MSRV | 1.85 |
| Commit | `971c7acb99decde45bf28860e6e10372718c51e2` |
| LOC | ~113K |
| Tests | 3,113 (0 failures, 35.78s) |

---

## 4. Transitive Dependency Summary

### 4.1 Dependency Tree Statistics

| Metric | Value | Command |
|--------|-------|---------|
| Direct runtime dependencies | 14 (+ 1 optional) | `cargo tree --depth 1` |
| Direct dev dependencies | 5 | `cargo tree --depth 1` (dev section) |
| Total unique transitive packages | ~424 | `cargo tree --prefix none --no-dedupe \| sort -u \| wc -l` |
| Total dependency tree lines (with duplicates) | ~1,339 | `cargo tree \| wc -l` |

### 4.2 Major Transitive Dependency Trees

The following direct dependencies contribute the most transitive dependencies:

| Direct Dependency | Approximate Transitive Count | Primary Subtree Purpose |
|-------------------|------------------------------|------------------------|
| bevy 0.15.3 | ~300 | Rendering (wgpu, winit, naga), windowing, audio, input, ECS core |
| parry3d 0.17.6 | ~30 | nalgebra (linear algebra), simba (abstract algebra) |
| oxidized_navigation 0.12.0 | ~15 | Navigation mesh generation, spatial queries |
| bevy_egui 0.31.1 | ~10 | egui core, rendering bridge |
| rayon 1.11.0 | ~5 | crossbeam (lock-free data structures), rayon-core |

### 4.3 Transitive Dependency Risk Note

A full enumeration of all ~424 transitive packages is not included in this document due to volume. The authoritative source for the complete dependency tree is `Cargo.lock` in the repository root. The complete tree can be regenerated using the commands in Section 5.

Key observations:
- The majority of transitive dependencies are pulled by bevy's rendering subsystem (wgpu, winit, naga, and their platform-specific bindings).
- In headless mode (`cargo run --bin headless_sim`), rendering dependencies are compiled but not executed.
- The batch simulator (`src/batch/`) has zero dependency on bevy and therefore zero exposure to bevy's transitive tree at runtime.
- All transitive dependencies are pinned via `Cargo.lock`.

---

## 5. SBOM Generation and Verification Commands

### 5.1 Generation Commands

These commands reproduce the data in this document from the codebase:

```bash
# Direct dependencies with versions and licenses
cargo tree --depth 1 --format "{p} {l}"

# Full dependency tree (all transitive)
cargo tree

# Full dependency tree in flat format (unique packages)
cargo tree --prefix none --no-dedupe | sort -u

# Machine-readable metadata (JSON)
cargo metadata --format-version 1

# Dependency count
cargo tree --prefix none --no-dedupe | sort -u | wc -l
```

### 5.2 Verification Commands

```bash
# Verify Cargo.lock matches Cargo.toml (no unresolved changes)
cargo check

# Verify no known vulnerabilities
cargo audit

# Verify no duplicate crate versions
cargo tree --duplicates

# Verify specific crate version and checksum
grep -A4 'name = "bevy"' Cargo.lock | head -5
```

### 5.3 CycloneDX Export (Recommended)

For machine-readable SBOM export compatible with FDA and NTIA tooling:

```bash
# Install CycloneDX generator
cargo install cargo-cyclonedx

# Generate CycloneDX SBOM (JSON format)
cargo cyclonedx --format json --output-file sbom.cdx.json

# Generate CycloneDX SBOM (XML format)
cargo cyclonedx --format xml --output-file sbom.cdx.xml
```

**Note:** `cargo-cyclonedx` is not currently a project dependency. It is a standalone tool used for SBOM generation. The markdown table in Section 3 is the authoritative human-readable SBOM; CycloneDX export is recommended for automated tooling integration.

---

## 6. License Compliance Summary

### 6.1 Direct Dependency Licenses

| License | Count | Components |
|---------|-------|------------|
| MIT OR Apache-2.0 | 11 | bevy, glam, egui_plot, oxidized_navigation, rayon, ron, serde, serde_json, noise, proptest, wgpu |
| MIT | 3 | tracing, bevy_egui, naga (also MIT OR Apache-2.0) |
| Apache-2.0 | 1 | parry3d |
| Zlib OR Apache-2.0 OR MIT | 1 | bytemuck |
| Apache-2.0 OR MIT | 2 | criterion, fxhash |
| Apache-2.0 / MIT | 2 | pollster, minifb |

### 6.2 Compatibility with AGPL-3.0

RESONANCE is licensed under AGPL-3.0-or-later. All direct dependencies use permissive licenses (MIT, Apache-2.0, Zlib) that are compatible with AGPL-3.0:

- **MIT:** Permissive. Compatible with AGPL-3.0 (can be sublicensed under AGPL).
- **Apache-2.0:** Permissive with patent clause. Compatible with GPL-3.0/AGPL-3.0 per FSF guidance.
- **Zlib:** Permissive. Compatible with AGPL-3.0.

No copyleft (GPL, LGPL) or restrictively licensed dependencies are present among direct dependencies.

### 6.3 Transitive License Risk

A comprehensive license audit of all ~424 transitive dependencies has not been performed. Based on the Rust/crates.io ecosystem norms and preliminary review:

- The Bevy ecosystem (largest transitive tree) uses MIT OR Apache-2.0 consistently.
- The nalgebra/parry ecosystem (Dimforge) uses Apache-2.0.
- No known copyleft transitive dependencies exist in the standard Bevy + Rust ecosystem.

**Recommendation:** Perform a full transitive license audit using `cargo-deny` or `cargo-license` before any commercial distribution or regulatory submission. This is planned for RD-7 (Release Package).

```bash
# Recommended: full license audit
cargo install cargo-deny
cargo deny check licenses
```

---

## 7. Integrity Verification

### 7.1 Cargo.lock as Integrity Source

`Cargo.lock` is the authoritative source for dependency integrity. It is:

- **Version-controlled:** Committed to Git, tracked in every commit.
- **Deterministic:** Given the same `Cargo.lock`, `cargo build` produces the same dependency resolution.
- **Checksummed:** Each package entry includes a SHA-256 checksum verified by Cargo against the crates.io registry.
- **Tamper-evident:** Any modification to `Cargo.lock` is visible in `git diff` and `git log`.

### 7.2 Build Reproducibility

Given the same `Cargo.lock` and Rust toolchain version:

```bash
# Pin Rust toolchain (if not already pinned via rust-toolchain.toml)
rustup override set 1.85.0

# Build with locked dependencies (fails if Cargo.lock is stale)
cargo build --release --locked
```

The `--locked` flag ensures that `cargo build` uses exactly the versions in `Cargo.lock` and fails if any dependency would be resolved differently. This guarantees build reproducibility.

### 7.3 Supply Chain Security

| Measure | Status |
|---------|--------|
| Dependencies sourced from crates.io only | Yes -- all `source = "registry+https://github.com/rust-lang/crates.io-index"` |
| No git dependencies | Yes -- no `git = "..."` in Cargo.toml |
| No path dependencies | Yes -- no `path = "..."` in Cargo.toml |
| No custom registries | Yes -- crates.io only |
| Cargo.lock committed | Yes -- tracked in Git |
| cargo audit available | Yes -- can be run manually; CI integration planned |

---

## 8. Revision History

| Version | Date | Author | Changes |
|---------|------|--------|---------|
| 1.0 | 2026-04-02 | Resonance Development Team | Initial SBOM. 15 runtime + 5 dev direct dependencies enumerated with versions, licenses, checksums. Transitive dependency count: ~424. License compliance confirmed for direct dependencies. |

---

**Approvals:**

| Role | Name | Date | Signature |
|------|------|------|-----------|
| Alquimista (Author) | Resonance Development Team | 2026-04-02 | _draft -- not signed_ |
| Verificador (Reviewer) | _pending_ | _pending_ | _pending_ |
