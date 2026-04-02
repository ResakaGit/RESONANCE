# ADR-004: Abstract Energy Units (qe) Instead of Molar Concentrations

**Status:** Accepted
**Date:** 2026-04-02
**Deciders:** Resonance Development Team
**Context of:** Simulation Ontology and Clinical Applicability Boundaries

## Context

RESONANCE simulates emergent life from 8 axioms and 4 fundamental constants. The simulation spans 10 levels of biological hierarchy: molecular bonds (Coulomb + Lennard-Jones), cellular metabolism (metabolic graph), organ morphogenesis (constructal body plan), organismal behavior (awakening + inference), population dynamics (reproduction + mutation), ecosystem interactions (trophic + symbiosis), and beyond.

A fundamental design decision is required: should the simulation's base unit be a physically measurable quantity (moles, Daltons, Joules, nanometers) or an abstract energy quantum?

This decision propagates to every equation in `blueprint/equations/` (45+ domain files), every constant in `blueprint/constants/`, and every output interpretation. It also directly affects regulatory documentation -- specifically the Credibility Assessment (ASME V&V 40) and the Limitations & Applicability Scope.

## Decision

Use **abstract energy quanta (qe)** as the fundamental unit for all simulation quantities. The qe is dimensionless and scale-free. All 8 axioms operate on qe. All derived thresholds in `derived_thresholds.rs` are computed in qe.

Physical interpretation is provided through **calibration bridges** -- external mapping functions that translate qe dynamics to specific biological scales. Four clinical calibration profiles exist (CML, prostate, NSCLC, canine MCT), each mapping qe thresholds to observable clinical parameters.

The abstraction is not a limitation -- it is the core architectural feature. It enables:
- The same Coulomb + LJ equations to model molecular bonds and ecosystem energy transfer.
- The same Kleiber scaling (mass^0.75) to apply across 27 orders of magnitude.
- The same dissipation rates to govern cellular metabolism and stellar energy cycles.

## Consequences

### Positive
- Scale-free physics: one equation set covers molecular to ecosystem dynamics without unit conversion layers.
- No coupling to specific biological scales -- the simulation is general-purpose, not locked to e.g., mammalian cell biology.
- Conservation tracking (Axiom 5) is exact in qe -- no floating-point unit conversion errors accumulate.
- Enables the 4-constant derivation chain: all lifecycle thresholds derive algebraically from KLEIBER + DISSIPATION_RATIOS + BANDWIDTH + DENSITY_SCALE.

### Negative
- Cannot make quantitative clinical predictions without an external calibration step. qe = 100 has no inherent physical meaning.
- Users may misinterpret qe values as physical quantities (e.g., assuming qe maps linearly to cell count or drug concentration).
- Every output, visualization, and publication requires an "abstract units" disclaimer.
- Bozic 2013 validation is qualitative (suppression percentages, not absolute cell counts or time-to-resistance in weeks).

### Risks
- If the field moves toward quantitative digital twins with physically measurable units, RESONANCE's abstract units may be seen as a limitation rather than a feature. Mitigation: calibration bridges demonstrate that physical mapping is possible without changing the core engine.
- Reviewers may dismiss results as "not real biology" due to abstract units. Mitigation: the Zenodo paper explicitly addresses this as a design choice, not an oversight.

## Alternatives Considered

| Alternative | Why Rejected |
|------------|-------------|
| Molar concentrations (mol/L) | Locks the simulation to molecular scale. Cannot model macro emergence (organisms, ecosystems) without a separate equation set. Breaks the axiom that "everything is energy." |
| Dimensionless ratios (normalized 0-1) | Loses conservation tracking -- ratios don't sum. Cannot enforce Pool Invariant (Axiom 2) or total energy monotonic decrease (Axiom 5). |
| SI units (Joules, meters, seconds) | Overspecifies the simulation. Axioms become scale-dependent. The 4 fundamental constants would need per-scale variants. Kleiber scaling would require explicit unit bookkeeping. |
| Dual-unit system (qe internally, SI externally) | Adds a mandatory conversion layer to every output path. Conversion errors become a new failure mode. Complexity cost exceeds benefit for a research tool. |

## References

- `CLAUDE.md` -- Axiom 1: "Everything is Energy"
- `src/blueprint/equations/derived_thresholds.rs` -- All lifecycle constants from 4 fundamentals
- `docs/regulatory/01_foundation/RD-6.3` -- Limitations & Applicability Scope
- `docs/regulatory/01_foundation/RD-4.2` -- ASME V&V 40 Credibility Assessment, Section 8 (Applicability)
- `src/blueprint/equations/coulomb.rs` -- Coulomb + LJ with qe-derived constants
- `CLAUDE.md` -- "Honest scope (all levels)" and "Abstract qe units (not molar concentrations)"
