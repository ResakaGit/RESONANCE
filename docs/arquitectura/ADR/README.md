# Architecture Decision Records (ADR)

Key design decisions for the RESONANCE project. ADR-001 through ADR-008 cover regulatory documentation decisions. ADR-009 through ADR-013 cover paper validation design.

## Regulatory Documentation (ADR-001 to ADR-008)

| ADR | Title | Status | Date |
|-----|-------|--------|------|
| [001](ADR-001-safety-class-a.md) | IEC 62304 Safety Class A Classification | Accepted | 2026-04-02 |
| [002](ADR-002-research-tool-positioning.md) | Research Tool Positioning (Not SaMD) | Accepted | 2026-04-02 |
| [003](ADR-003-git-as-qms.md) | Git as QMS Infrastructure | Accepted | 2026-04-02 |
| [004](ADR-004-abstract-energy-units.md) | Abstract Energy Units (qe) Instead of Molar Concentrations | Accepted | 2026-04-02 |
| [005](ADR-005-competence-through-delivery.md) | Competence-Through-Delivery Model (No Formal Training Certificates) | Accepted | 2026-04-02 |
| [006](ADR-006-trunk-based-development.md) | Trunk-Based Development as Change Control | Accepted | 2026-04-02 |
| [007](ADR-007-github-actions-ci.md) | GitHub Actions for CI/CD Pipeline | Accepted | 2026-04-02 |
| [008](ADR-008-alcoa-via-git.md) | ALCOA+ Compliance via Git (No Electronic Signatures) | Accepted | 2026-04-02 |

## Paper Validation (ADR-009 to ADR-013)

| ADR | Title | Status | Date |
|-----|-------|--------|------|
| [009](ADR-009-zero-coupling-validation.md) | Zero-Coupling Paper Validation | Accepted | 2026-04-02 |
| [010](ADR-010-qualitative-first-validation.md) | Qualitative-First Validation Strategy | Accepted | 2026-04-02 |
| [011](ADR-011-multi-comparator-strategy.md) | Multi-Comparator Validation (5 Papers, Not 1 Deep) | Accepted | 2026-04-02 |
| [012](ADR-012-frequency-as-mutation-proxy.md) | Frequency as Mutation/Identity Proxy | Accepted | 2026-04-02 |
| [013](ADR-013-stateless-experiment-contract.md) | Stateless Experiment Contract (Config → Report) | Accepted | 2026-04-02 |

## Energy Architecture (ADR-015+)

| ADR | Title | Status | Date |
|-----|-------|--------|------|
| [015](ADR-015-temporal-telescope.md) | Temporal Telescope — Dual-Timeline Speculative Execution with Reconciliation | Accepted | 2026-04-04 |
| [016](ADR-016-multi-telescope.md) | Multi-Telescope — Quantum-Inspired Hierarchical Speculative Execution | Accepted | 2026-04-04 |

## Autopoiesis (ADR-037+)

| ADR | Title | Status | Date |
|-----|-------|--------|------|
| [037](ADR-037-reaction-network-substrate.md) | Reaction Network Substrate (SpeciesGrid SoA) | Accepted | 2026-04-10 |
| [038](ADR-038-emergent-membrane.md) | Emergent Membrane via Product Density Gradient | Accepted | 2026-04-10 |
| [039](ADR-039-fission-criterion.md) | Fission Criterion — Pressure vs Decay (revisado 2026-04-15) | Accepted | 2026-04-10 |
| [040](ADR-040-streaming-soup-sim.md) | Streaming `SoupSim` — incremental stepper | Accepted | 2026-04-14 |
| [041](ADR-041-lineage-in-soup-report.md) | Lineage Tracking in SoupReport | Accepted | 2026-04-14 |
| [042](ADR-042-bevy-viz-layout.md) | Bevy Viz Layout for Autopoietic Lab | Propuesto | 2026-04-14 |

## Autopoiesis Integration (ADR-043+)

| ADR | Title | Status | Date |
|-----|-------|--------|------|
| [043](ADR-043-species-grid-as-resource.md) | `SpeciesGrid` como Resource ECS + puente a `AlchemicalInjector` | Propuesto | 2026-04-15 |
| [044](ADR-044-protocell-to-entity-spawn.md) | Protocell → Entity ECS · `FissionEvent` Observer | Propuesto | 2026-04-15 |
| [045](ADR-045-chemistry-canonical-choice.md) | Elección canónica · alchemical vs mass-action | Aceptado (Camino 1) | 2026-04-15 |
