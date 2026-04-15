# Resonance — Claude Code Instructions

## Project

Emergent life simulation in Rust/Bevy 0.15. Everything is energy (qe). 14 orthogonal ECS layers. 100% emergent behavior — no scripts, no templates.

**Paper:** https://zenodo.org/records/19342036 | **Repo:** https://github.com/ResakaGit/RESONANCE | **License:** AGPL-3.0

## Stack (Hard Constraints)

| Layer | Tech | Version |
|-------|------|---------|
| Language | Rust | stable 2024, MSRV 1.85 |
| Engine | Bevy | 0.15.x |
| Math | glam 0.29 | `math_types.rs` (decoupled from bevy::math) |
| Async | None | Bevy schedule only |

## Architecture

- **Layered ECS with Vertical Slices.** Components = domain, systems = use cases, Bevy = infrastructure. NOT hexagonal.
- **Pipeline:** `FixedUpdate` phases in `simulation/pipeline.rs`: `SimulationClockSet` → `Input` → `ThermodynamicLayer` → `AtomicLayer` → `ChemicalLayer` → `MetabolicLayer` → `MorphologicalLayer`.
- **Pure math** in `blueprint/equations/`. NEVER inline formulas in systems.
- **Constants** in `{module}/constants.rs`. All derived from 4 fundamentals via `blueprint/equations/derived_thresholds.rs`.
- **Stateless-first:** Pure functions + Resources. Components hold state, systems transform it.

## The 8 Axioms (INVIOLABLE)

No change may contradict, bypass, or weaken ANY axiom. If a change conflicts with an axiom, the change is WRONG. No exceptions. No DEBT. No "temporary" violations.

### Primitive (independent, irreducible)

1. **Everything is Energy** — All entities are qe. No separate HP/mana/stats.
2. **Pool Invariant** — `sum(children) <= parent`. Conservation absolute.
4. **Dissipation** — All processes lose energy. No 100% efficiency.
7. **Distance Attenuation** — Interaction intensity monotonically decreasing in distance.
8. **Oscillatory Nature** — Every concentration oscillates at frequency f. Interaction modulated by `cos(df*t + dphi)`.

### Derived (guard rails, zero simulation effect)

3. **Competition** — From Axiom 8 applied to energy transfer.
5. **Conservation** — From Axiom 2 + 4. Total qe monotonically decreases.
6. **Emergence at Scale** — Behavior at scale N = consequence of N-1. No top-down programming.

## The 4 Fundamental Constants

Everything else is algebraically derived from these. See `blueprint/equations/derived_thresholds.rs`.

| Constant | Value | Type |
|----------|-------|------|
| `KLEIBER_EXPONENT` | 0.75 | Physics (metabolic scaling) |
| `DISSIPATION_{SOLID,LIQUID,GAS,PLASMA}` | 0.005, 0.02, 0.08, 0.25 | Physics (2nd Law rates, ratio 1:4:16:50) |
| `COHERENCE_BANDWIDTH` | 50.0 Hz | Calibration (frequency window) |
| `DENSITY_SCALE` | 20.0 | Calibration (spatial normalization) |

Never touch KLEIBER or dissipation ratios for calibration. Only BANDWIDTH + DENSITY_SCALE are tunable.

## Coding Rules

1. English identifiers only
2. Max 4 fields per component — split into layers if more
3. One system, one transformation — no god-systems (>5 component types)
4. `SparseSet` for transient components
5. Guard change detection: `if val != new { val = new; }` or `set_if_neq`
6. Chain events: `.before()` or `.chain()`. Never unordered
7. Phase assignment required for every gameplay system
8. Math in `blueprint/equations/` — systems call pure fns
9. Component group factories for spawning (prefer over EntityBuilder)
10. Constants centralized per module
11. `With<T>`/`Without<T>` over `Option<&T>` for filter-only queries
12. Minimal query width — only request components you read/write
13. No `Vec<T>` in components unless genuinely variable-length
14. `#[derive(Component, Reflect, Debug, Clone)]` + `app.register_type::<T>()`

## Hard Blocks

**Absolute (never violate):**
1. NO `unsafe`
2. NO external crates without approval
3. NO `async`/`await`
4. NO `Arc<Mutex<T>>` — use `Resource` or `Local`
5. NO shared mutable state outside Resources

**Strong defaults (violate only with `// DEBT: <reason>`):**
6. NO `HashMap` in hot paths — sorted `Vec` or Entity indexing
7. NO `String` in components — enums, `u32` IDs, or `&'static str`
8. NO `Box<dyn Trait>` in components
9. NO `#[derive(Bundle)]` — Bevy 0.15 uses tuples or `#[require(...)]`
10. NO `ResMut` when `Res` suffices
11. NO `unwrap()`/`expect()`/`panic!()` in systems — `let-else` or `if-let`
12. NO inline formulas in systems
13. NO storing derived values as components
14. NO trait objects for game logic
15. NO component methods with side effects
16. NO `Entity` as persistent/network ID
17. NO systems in `Update` for gameplay — `FixedUpdate` + `Phase` only

## Communication

- **Spanish default.** English tech terms inline.
- **Tone:** Peer-to-peer, direct, professional. Answer first, explain second.
- **Brevity:** If it fits in 3 lines, don't use 10.
- **Reference layers by ID:** "L3 FlowVector", not "the velocity component".
- **Roles:** Alquimista (code), Observador (review), Planificador (planning), Verificador (PR — PASS/WARN/BLOCK).

## Inference Protocol

### Critique first
Before implementing, evaluate: Is this the right thing? Is there a simpler alternative? What does it cost? What breaks?

### Propose alternatives
For non-trivial decisions: Option A (what was asked + tradeoffs), Option B (recommended alternative), Option C (don't do it at all).

### Auto-trigger pushback on
- Premature abstraction
- Scope creep disguised as architecture
- Perfectionism loops over shipping
- Claims of "emergence" without test/demo evidence
- Orphan components no system reads/writes

### Priority hierarchy
1. Playable/fun (highest)
2. Simulation correctness
3. Architecture respect
4. Coding rules (lowest)

## Testing

```
cargo test                                          # ~3,166 tests
cargo bench --bench batch_benchmark                 # performance
cargo run --bin headless_sim -- --ticks N --out x.ppm  # headless (no GPU)
RESONANCE_MAP={name} cargo run                      # named maps
cargo run --release --bin bozic_validation          # drug validation (~95s)
cargo run --release --bin paper_validation           # 6 papers + PV-6
```

- **Unit:** `#[cfg(test)]` in `blueprint/equations/`. Name: `fn_condition_expected`.
- **Integration:** `MinimalPlugins`, spawn minimal components, ONE update, assert delta.
- **Property:** `tests/property_conservation.rs` (proptest).

## Binaries (26 post-cleanup 2026-04-15) — `cargo run --release --bin <name>`

**Viz:** `lab` (15 exp, 25 maps) · `headless_sim --ticks N --out x.ppm` · `planet_viewer` · `earth_telescope` · `survival` (WASD)
**Evo:** `evolve` · `fermi`(B1) · `speciation`(B2) · `cambrian`(B3) · `debate`(B4) · `convergence`(D2) · `versus`(A1)
**Drug:** `cancer_therapy` · `pathway_inhibitor` · `adaptive_therapy` · `bozic_validation` · `paper_validation`(PV-1→6)
**MD:** `lj_fluid`(MD-4) · `peptide_vacuum`(MD-9) · `peptide_solvated`(MD-14) · `fold_go`(MD-17 REMD) · `particle_lab`
**Cosmic:** `cosmic_telescope` (CT-8 3D viewer S0→S4, click zoom, multiverse bar) · `cosmic_telescope_headless` (CT-8 CI validation) · `cosmic_bigbang` (CT-2 cluster formation)
**Autopoiesis:** `autopoietic_lab` (AP-6 mass-action chemistry, formose/hypercycle, `--live` ANSI render, `--ppm` heatmap, `--out-dir` artifacts)

## Experiments (`src/use_cases/experiments/`) — Config→Report, pure fns

| Module | ID | Validates | Source |
|--------|----|-----------|--------|
| `paper_zhang2022` | PV-1 | Adaptive therapy prostate (Lotka-Volterra) | Zhang 2022 eLife |
| `paper_sharma2010` | PV-2 | Drug-tolerant persisters ~0.3% | Sharma 2010 Cell |
| `paper_hill_ccle` | PV-3 | Hill n=2 vs GDSC/CCLE | Garnett+Barretina 2012 |
| `paper_foo_michor2009` | PV-4 | Continuous vs pulsed therapy | Foo&Michor 2009 PLoS |
| `paper_michor2005` | PV-5 | Biphasic CML imatinib | Michor 2005 Nature |
| `paper_unified_axioms` | PV-6 | All from 4 fundamentals (4/6 pass) | Internal boundary |
| `autopoiesis` | AP-0..6d | Mass-action RAF + emergent membrane + fission (gas/liquid threshold) | Kauffman/Pross/Breslow/Eigen |
| `cancer_therapy` | — | Freq drift + quiescent stem resistance | L1 cytotoxic |
| `pathway_inhibitor_exp` | — | Metabolic compensation resistance | L3 inhibitor |
| `lj_fluid` | MD-4 | 2D LJ thermodynamics | Toxvaerd77+Smit91 |
| `peptide_vacuum` | MD-9 | Ramachandran phi/psi | AMBER-like |
| `peptide_solvated` | MD-14 | Solvated RDF+stability | TIP3P+SHAKE+Ewald |
| `particle_lab` | — | Coulomb+LJ→molecules | Ax 1,7,8 |
| `fermi`/`speciation`/`cambrian`/`debate`/`convergence` | B1-4,D2 | Evo dynamics | — |
| `lab`/`versus` | A1 | Sandbox wrappers | — |

## 14 ECS Layers (`src/layers/`)

L0 `BaseEnergy` qe:f32 · L1 `SpatialVolume` radius:f32 · L2 `OscillatorySignature` freq,phase:f32 · L3 `FlowVector` vel:Vec2,diss:f32 · L4 `MatterCoherence` state,bond_e,cond · L5 `AlchemicalEngine` buffer,in_v,out_v · L6 `AmbientPressure` delta_qe,viscosity · L7 `WillActuator` dir,channel_t · L8 `AlchemicalInjector` proj_qe,freq,rad · L9 `MobaIdentity` faction,tags · L10 `ResonanceLink` target,mult · L11 `TensionField` strength,falloff · L12 `Homeostasis` target_f,cost · L13 `StructuralLink` rest_l,k,break_s

**Two coexisting chemistries (ADR-045 Camino 1 — coexistencia 2026-04-15):**
- **L4/L5/L8** modelan química emergente Ax 8 (resonancia de frecuencias) — canónica para escalas planetaria+ (`planet_viewer`, `lab`, `earth_telescope`).
- **AP-* mass-action** (`SpeciesGrid`, `ReactionNetwork`) provee química explícita con estequiometría — canónica para autopoiesis (`autopoietic_lab`) + validación contra papers (Breslow, Eigen, Kauffman).
- **Bridge AI** (ADR-043, ADR-044) acopla ambas: cuando `SoupSim` está cargado, sus species emiten qe al `EnergyFieldGrid` via `species_to_qe_injection_system` (Phase::ChemicalLayer) y sus fisiones spawnean entities ECS con `BaseEnergy + OscillatorySignature + LineageTag` via `on_fission_spawn_entity`.

**Components AI-* (autopoiesis integration):**
- `LineageTag(u64)` — identidad de linaje per-entity (ADR-044). `is_primordial()` ⇔ `0` (sopa pre-fisión).

## Caches

- `KleiberCache` (SparseSet) — `r^0.75`, dirty on L1 change
- `GompertzCache` (SparseSet) — exact death tick u64, dirty on L0/L2
- `Converged<T>` (SparseSet) — iterative convergence flag
- `exact_cache` (`blueprint/equations/`) — Kleiber+Gompertz+alignment pure fns
- Bridge cache — `Vec` (N≤256) or `FxHashMap` (N>256), LRU/LFU/ARC

## Bridges (`src/bridge/`, 29)

**Physics(13):** Density Temperature PhaseTransition Interference Dissipation Drag Engine Will Catalysis CollisionTransfer Osmosis EvolutionSurrogate CompetitionNorm
**Emergence(11):** AssociativeMemory OtherModel MemeSpread FieldMod Symbiosis Epigenetic Senescence Coalition NicheOverlap Timescale AggSignal
**Advanced(5):** Tectonic LODPhysics Institution Symbol SelfModel

**Autopoiesis Integration (`src/simulation/`, ADR-043+044+045, 2 systems + 1 bridge module):**
- `species_to_qe.rs` — AI-1 bridge: `SpeciesGrid` → `EnergyFieldGrid` qe injection (Ax 8 alignment). `SPECIES_TO_QE_COUPLING = DISSIPATION_LIQUID` derivado.
- `autopoiesis_bridge.rs` — AI-2: `SoupSimResource` + `FissionEventCursor` + 3 systems (`step_soup_sim_system`, `emit_fission_events_system`, `on_fission_spawn_entity`). Cap `MAX_FISSION_EVENTS_PER_TICK = 4`.
- `events.rs` — `FissionEvent` event con tick/parent/children/centroid/mean_freq/qe_per_child.
- Pipeline chain: `step_soup_sim → species_to_qe → emit_events → spawn` en `Phase::ChemicalLayer`. Opt-in via `Option<Res<SoupSim>>` — sin sopa AP cargada, no-op silencioso.

## Telescope (`src/batch/telescope/`, ADR-015) — dual-timeline: Anchor(truth) + K-tick projection

`mod`=state machine (Project→Reconcile→Correct→Idle) · `activation`=regime metrics+LOD · `pipeline`=`tick_telescope_sync` · `projection`=analytic forward (entity,world,grids) · `diff`=classify(Perfect/Local/Systemic) · `cascade`=correction · `calibration_bridge`=diff→weights · `stack`=K hierarchy [1,10,100,1000]

## Sprints (9 active + 58 archived)

PV(5) pending: paper benchmarks · GS(6) wave2: MOBA · PC(7) designed: emergent atoms · NS(4) designed: signals · EI(3) designed: prediction · TU(4) designed: tools · EL(4) designed: language · CV(4) designed: civilization · **PP(9) designed: plant physiology (pigment, curvature, volatiles, phototropism, phenology, organ senescence, pollination) — ADR-033/034/035** · MD(19) **done**: protein folding · R(6ph) planned: PME/SETTLE/r-RESPA · LR(4) **done**: lab UI · BS(3/7) **done**: cache decoupling · **CT(10) done**: cosmic telescope S0→S4 (ADR-036, `src/cosmic/`) · **AP(7) done**: autopoiesis (ADR-037→041 + ADR-039 revisión 2026-04-15-b gas/liquid threshold) · **AI(3) done 2026-04-15**: integration AP↔main sim (ADR-043 species→qe bridge, ADR-044 fission→entity spawn, ADR-045 Camino 1 coexistencia) · 58 archived tracks

## Key References

- `docs/ARCHITECTURE.md` — canonical architecture doc
- `docs/design/AXIOMATIC_CLOSURE.md` — cross-axiom compositions
- `docs/design/` — design specs | `docs/arquitectura/` — module contracts (32 ADRs)
- `docs/regulatory/AUDIT_CHECKLIST.md` — regulatory index (IEC 62304, ISO 14971, etc.)

## Workflow Skills (`~/.claude/skills/`)

- `/create-sprint` — new sprint doc under `docs/sprints/<TRACK>/`. Gates Stages 0-2 (ready) before accepting items; Stages 3-7 run-time.
- `/create-adr` — new ADR under `docs/arquitectura/ADR/`. Explores repo first, cites `file:line`, flags `[ASSUMPTION]` when unverified.
- `/implement-sprint-item` — execution standard: ready-check → tier A/B/C/D/E → tier-testing → 15-60min cycles → in-cycle refactor → Step-6 fix protocol (L1 symptom → L5 one-sentence) → non-negotiable merge checklist.
Chain: `create-sprint` → (arch-significant item?) `create-adr` → `implement-sprint-item`.
