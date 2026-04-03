# ADR-012: Frequency as Mutation/Identity Proxy in Validations

**Status:** Accepted
**Date:** 2026-04-02
**Deciders:** Resonance Development Team
**Context of:** Paper validation track — mapping RESONANCE frequency to paper mutation rates

## Context

Published papers model drug resistance via mutation rate `u` (probability per cell division of gaining resistance). RESONANCE has no mutation rate — it has frequency heterogeneity. When a cell's frequency is far from the drug's target frequency, the drug effect is reduced (low alignment α). Resistance is statistical, not genetic.

The 5 paper validations need a formal mapping between frequency drift and mutation rate.

## Decision

Frequency offset from drug target is the analog of a resistance mutation. The mapping is:

```
Paper: cell mutates at rate u → gains resistance gene → drug doesn't bind
RESONANCE: cell reproduces with frequency drift → offspring freq far from drug → low α → drug ineffective
```

Specifically:
- **Mutation rate u** ↔ frequency drift magnitude during reproduction (`mutation_sigma`)
- **Resistant mutant** ↔ entity with `|freq - drug_freq| > COHERENCE_BANDWIDTH`
- **Pre-existing resistance** ↔ initial frequency spread in population
- **Fitness cost of resistance** ↔ frequency-shifted entities have suboptimal alignment with nutrient sources

This is NOT a claim that frequency IS mutation. It is a claim that frequency heterogeneity produces the same population-level dynamics as mutation-based resistance: rare pre-existing variants, selection under drug pressure, clonal expansion of resistant subpopulation.

## Consequences

### Positive
- Enables direct comparison with 5 published mathematical oncology models
- The mapping is falsifiable: if frequency drift produces qualitatively different dynamics than mutation, the validation will fail
- Preserves Axiom 8 (oscillatory nature) as the mechanism — no need to add a "mutation" system

### Negative
- Frequency drift is continuous; real mutations are discrete events. This smooths over stochastic effects that matter at small population sizes
- No equivalent of "back mutation" (resistance reversion) unless frequency drifts back — which is statistically rare
- Sharma 2010 persisters involve epigenetic (reversible) tolerance, not genetic mutation — frequency drift is closer to epigenetics than to mutation, which is actually a better fit

### Risks
- If a paper's prediction depends on the discrete, stochastic nature of mutation (e.g., Luria-Delbrück fluctuation), continuous frequency drift may not reproduce it

## Alternatives Considered

| Alternative | Why Rejected |
|------------|-------------|
| Add explicit mutation system (bit-flip in genome) | Breaks Axiom 6 (no top-down programming of resistance); adds mechanism not derived from axioms |
| Use variable genome mutations instead of frequency drift | VariableGenome exists but operates at a different abstraction level; would couple paper validations to genome system |
| Refuse to compare with mutation-based papers | Eliminates 80% of published resistance literature from validation |

## References

- CLAUDE.md Axiom 8 — "Every concentration oscillates at frequency f"
- `src/blueprint/equations/determinism.rs` — `gaussian_f32()` used for reproduction frequency drift
- `src/simulation/reproduction/mod.rs` — offspring inherits mutated InferenceProfile
- `src/blueprint/equations/pathway_inhibitor.rs` — `binding_affinity()` uses Gaussian alignment
- ADR-004 — abstract energy units decision (upstream of this)
