# Track: COSMIC_TELESCOPE — Del Big Bang a la Vida, un Zoom a la Vez

Simulación multi-escala con colapso observacional. El usuario ve el universo
desde fuera y hace zoom hasta nivel molecular. Solo el nivel observado se simula
a resolución completa — el resto corre coarsened o congelado.

**Non-goal:** Simular 10^80 partículas. Eso es imposible.
**Goal:** Que un observador pueda viajar desde el Big Bang hasta ver una proteína
plegándose dentro de un organismo, en un planeta, en un sistema estelar, en un
cluster galáctico — todo derivado de los 4 fundamentales.

**Invariant:** Cada transición de escala respeta Pool Invariant (Axiom 2),
Dissipation (Axiom 4), y Conservation (Axiom 5). No se crea energía al hacer zoom.

**ADR:** [ADR-036](../../arquitectura/ADR/ADR-036-cosmic-telescope.md)

---

## Current State (baseline)

| Qué existe | Dónde | Limitación |
|------------|-------|------------|
| TelescopeStack (8 niveles temporales) | `batch/telescope/stack.rs` | Solo temporal, escala fija |
| SimWorldFlat (~100KB, 1024 entities) | `batch/arena.rs` | Una sola escala |
| LOD Near/Mid/Far | `worldgen/lod.rs` | Espacial 2D, no multi-escala |
| TensionField (gravedad) | `layers/tension_field.rs` | Solo intra-mundo |
| Abiogenesis | `simulation/abiogenesis/mod.rs` | Funciona, no conectada a escala superior |
| Go model + REMD paralelo | `batch/systems/remd.rs` | Funciona, no conectado a escala superior |
| particle_lab | `batch/systems/particle_forces.rs` | Coulomb + LJ emergente |
| personal_universe | `bin/personal_universe.rs` | Seed determinista, solo texto |
| EnergyFieldGrid | `worldgen/field_grid.rs` | Escala planetaria fija |

---

## Phase 0: Foundation — Data Model + State Machine

**Goal:** Infraestructura de escalas sin simulación nueva. Solo el andamiaje.

| Sprint | Name | Status | Deliverable |
|--------|------|--------|-------------|
| [CT-0](SPRINT_CT0_SCALE_HIERARCHY.md) | Scale Hierarchy | ⏳ Diseñado | `ScaleLevel` enum, `ScaleManager` resource, `CosmicState` |
| [CT-1](SPRINT_CT1_ZOOM_ENGINE.md) | Zoom Collapse Engine | ⏳ Diseñado | Zoom-in/out events, axiom-constrained inference, seed branching |

**Milestone:** `ScaleManager` puede transicionar entre niveles con estado inferido.
Tests: Pool Invariant preservado en zoom-in/out round-trip.

## Phase 1: Escalas Extremas — Cosmológico + Molecular

**Goal:** Las dos puntas: Big Bang y proteínas. El medio ya existe (worldgen/abiogenesis).

| Sprint | Name | Status | Deliverable |
|--------|------|--------|-------------|
| [CT-2](SPRINT_CT2_COSMOLOGICAL.md) | Cosmological Scale | 🔒 CT-0 | N-body gravitacional, cluster formation, dt_cosmo |
| [CT-3](SPRINT_CT3_MOLECULAR_BRIDGE.md) | Molecular Bridge | 🔒 CT-1 | Organismo → proteínas, fold_go integrado como zoom S3→S4 |

**Milestone:** Big Bang produce clusters estables. Zoom en organismo muestra proteínas.

## Phase 2: Puentes Intermedios — Estelar + Planetario

**Goal:** Conectar cosmológico con lo que ya existe (worldgen).

| Sprint | Name | Status | Deliverable |
|--------|------|--------|-------------|
| [CT-4](SPRINT_CT4_STELLAR.md) | Stellar Scale | 🔒 CT-2 | Cluster → estrellas + gas, protoplanetas, nucleosíntesis como freq |
| [CT-5](SPRINT_CT5_PLANETARY_BRIDGE.md) | Planetary Bridge | 🔒 CT-4 | Estrella → planetas, conexión a EnergyFieldGrid existente |

**Milestone:** Zoom continuo Cosmo → Estelar → Planetario → Ecológico funciona.

## Phase 3: Background + Performance

**Goal:** Los niveles no observados siguen vivos. Performance elite.

| Sprint | Name | Status | Deliverable |
|--------|------|--------|-------------|
| [CT-6](SPRINT_CT6_BACKGROUND_COARSENING.md) | Background Coarsening | 🔒 CT-5 | Niveles no observados a tasa reducida, conservación verificada |
| [CT-7](SPRINT_CT7_TEMPORAL_INTEGRATION.md) | Temporal Integration | 🔒 CT-6 | TelescopeStack por escala, proyección temporal dentro de cada nivel |

**Milestone:** 5 niveles simultáneos, <16ms por frame en release.

## Phase 4: Visualización + Multiverso

**Goal:** La experiencia del "dios observando el universo".

| Sprint | Name | Status | Deliverable |
|--------|------|--------|-------------|
| [CT-8](SPRINT_CT8_VISUALIZATION.md) | Scale-Aware Rendering | 🔒 CT-6 | Cámara con transición suave, HUD por escala, fade in/out |
| [CT-9](SPRINT_CT9_MULTIVERSE.md) | Multiverse Seeds | 🔒 CT-1 | MultiverseLog, branch comparison, probabilistic observables |

**Milestone:** Binario `cosmic_telescope` — del Big Bang a la vida en una sesión.

---

## Dependency Chain

```
CT-0 (Scale Hierarchy) ──→ CT-1 (Zoom Engine)
        │                       │
        ↓                       ↓
CT-2 (Cosmological)       CT-3 (Molecular Bridge)
        │                       │
        ↓                       │
CT-4 (Stellar)                  │
        │                       │
        ↓                       │
CT-5 (Planetary Bridge) ←───────┘
        │
        ↓
CT-6 (Background Coarsening)
        │
   ┌────┴────┐
   ↓         ↓
CT-7       CT-8 (Visualization)
(Temporal)   │
             ↓
         CT-9 (Multiverse)
```

## File Architecture

```
src/
├── cosmic/                          ← NEW: módulo principal
│   ├── mod.rs                       ← CT-0: ScaleLevel, CosmicState, CosmicPlugin
│   ├── scale_manager.rs             ← CT-0: ScaleManager resource, transitions
│   ├── zoom.rs                      ← CT-1: ZoomIn/ZoomOut events, inference
│   ├── inference.rs                 ← CT-1: axiom-constrained state generation
│   └── multiverse.rs                ← CT-9: MultiverseLog, branch tracking
│
├── cosmic/scales/                   ← NEW: per-scale simulation
│   ├── cosmological.rs              ← CT-2: N-body gravitacional
│   ├── stellar.rs                   ← CT-4: formación estelar
│   └── coarsening.rs                ← CT-6: background tick reduction
│
├── cosmic/bridges/                  ← NEW: transiciones entre escalas
│   ├── cosmo_to_stellar.rs          ← CT-4: cluster → estrellas
│   ├── stellar_to_planetary.rs      ← CT-5: estrella → planetas
│   ├── planetary_to_ecological.rs   ← CT-5: planeta → worldgen (wrapper)
│   └── ecological_to_molecular.rs   ← CT-3: organismo → proteínas
│
├── blueprint/equations/
│   └── scale_inference.rs           ← CT-1: pure math para inferencia multi-escala
│
├── bin/
│   └── cosmic_telescope.rs          ← CT-8: binario principal
│
└── batch/telescope/
    └── stack.rs                     ← CT-7: extender para per-scale stacks
```

## Axiom Compliance Matrix

Cada sprint debe verificar que NO viola ningún axioma:

| Axiom | Cómo se preserva | Sprint que verifica |
|-------|-------------------|---------------------|
| 1 (Todo es qe) | Todas las entidades en todas las escalas son qe | CT-0 |
| 2 (Pool Invariant) | `sum(children.qe) <= parent.qe` en zoom-in | CT-1 |
| 3 (Competition) | Emerge de Axiom 8 en cada escala | CT-2, CT-3 |
| 4 (Dissipation) | Zoom-in pierde energía. Coarsening pierde energía | CT-1, CT-6 |
| 5 (Conservation) | `total_qe` monotone decreasing across all scales | CT-6 |
| 6 (Emergence at Scale) | S_{n+1} emerge de S_n, no al revés | CT-1, CT-5 |
| 7 (Distance Attenuation) | Gravedad InverseSquare en S0-S1, field decay en S2-S3 | CT-2, CT-4 |
| 8 (Oscillatory) | Frecuencias heredadas con bandwidth. Alignment en todas las escalas | CT-1, CT-3 |
