# ADR-043: `SpeciesGrid` como Resource ECS + puente a `AlchemicalInjector`

**Estado:** Propuesto
**Fecha:** 2026-04-15
**Contexto:** AUTOPOIESIS Integration (Sprint AI, ítem AI-1)
**ADRs relacionados:** ADR-037 (substrate), ADR-040 (streaming sim), ADR-038 (membrane), ADR-044, ADR-045

## 1. Contexto y problema

- Módulos afectados:
  - `src/layers/species_grid.rs` (SpeciesGrid, hoy data struct pura)
  - `src/layers/injector.rs:16` (`AlchemicalInjector`)
  - `src/worldgen/field_grid.rs:12` (`EnergyFieldGrid`)
  - `src/simulation/pipeline.rs` (phase `ChemicalLayer`)

La química AP-* (mass-action + RAF) opera sobre `SpeciesGrid`, un struct
denso `[f32; MAX_SPECIES]` por celda.  El simulador principal opera sobre
`EnergyFieldGrid` (qe continuo) con química qe-based por resonancia
(Ax 8) vía `AlchemicalInjector`.

Ambas químicas son **correctas independientemente** pero paralelas:
nada en el pipeline Bevy lee `SpeciesGrid`, y nada en `SoupSim` escribe
a `EnergyFieldGrid`.  Consecuencia: una fisión formose no afecta al
campo de energía planetario, y la iluminación solar no afecta la
cinética formose.

Se necesita un puente direccional (species → qe) que:
1. Exponga `SpeciesGrid` al world Bevy **sin duplicar datos**
2. Inyecte qe al `EnergyFieldGrid` respetando Ax 8 (alineación freq)
3. Sea determinístico y testeable
4. No rompa el harness AP-* existente (AP-5 proptest, ADR-040 byte-equivalence)

## 2. Alternativas

| Opción | Descripción | Pros | Contras |
|---|---|---|---|
| **A · `Resource` wrapper** | `SpeciesGridResource(pub SpeciesGrid)` + system que inyecta a `EnergyFieldGrid` | Mínima invasividad, reusa struct existente, 1-way dep | Acopla `layers/species_grid.rs` a Bevy por `#[derive(Resource)]` |
| B · Component-per-cell | Cada celda del grid como entity con `SpeciesCell` component | ECS-native, queries flexibles | Rompe cache locality (grid denso 32×32 = 1024 entities/cell update), viola coding rule "max 4 fields" si incluye freq, pierde SoA |
| C · Entity bridge + message | Sistema intermedio copia `SpeciesGrid` state → entities cada tick | Desacoplado | O(N_cells) copia por tick, overhead significativo, doble fuente de verdad |
| D · No hacer nada | Dos mundos paralelos indefinidos | Cero trabajo | Gap #1 del audit persiste; claim "stack continuo" falso |

## 3. Decisión

**Opción A — `SpeciesGridResource` wrapper + sistema de inyección en phase `ChemicalLayer`.**

### Estructura

```rust
// src/layers/species_grid.rs (append)
#[derive(Resource)]
pub struct SpeciesGridResource(pub SpeciesGrid);

// src/layers/reaction_network.rs (append)
#[derive(Resource)]
pub struct ReactionNetworkResource(pub ReactionNetwork);
```

### Sistema de inyección

```rust
// src/simulation/chemical/species_to_qe.rs (nuevo)

/// Proyecta Σ species_qe por celda → injection al EnergyFieldGrid.
/// Axiom 8: la frecuencia agregada de los productos alinea con el bandwidth
/// del injector — sólo especies en resonancia con el campo local aportan.
pub fn species_to_qe_injection_system(
    species: Res<SpeciesGridResource>,
    network: Res<ReactionNetworkResource>,
    mut field: ResMut<EnergyFieldGrid>,
    clock: Res<SimulationClock>,
) {
    let grid = &species.0;
    let (w, h) = (grid.width(), grid.height());
    let dt = clock.dt();
    for y in 0..h {
        for x in 0..w {
            let cell = grid.cell(x, y);
            let cell_qe: f32 = cell.species.iter().sum();
            if cell_qe <= 0.0 { continue; }
            let mean_freq = network.0.mean_product_frequency(&cell.species);
            let alignment = frequency_alignment(
                mean_freq, field.freq_at(x, y),
                REACTION_FREQ_BANDWIDTH_DEFAULT,
            );
            field.add_qe(x, y, cell_qe * alignment * SPECIES_TO_QE_COUPLING * dt);
        }
    }
}
```

### Constante de acoplamiento

```rust
// src/blueprint/constants/chemistry.rs (append)
/// Fracción de species_qe que se proyecta a qe de campo por tick.
/// Derivado: `SPECIES_TO_QE_COUPLING = DISSIPATION_LIQUID` — la misma
/// tasa a la que los productos difunden lateralmente (Ax 7) es la tasa
/// a la que se "pierden" como qe inyectable al campo (conservación local).
pub const SPECIES_TO_QE_COUPLING: f32 = DISSIPATION_LIQUID;
```

### Registro en pipeline

```rust
// src/plugins/layers.rs
app.add_systems(
    FixedUpdate,
    species_to_qe_injection_system.in_set(Phase::ChemicalLayer),
);
```

## 4. Justificación

1. **Axiom-compliant.** `SPECIES_TO_QE_COUPLING` deriva de `DISSIPATION_LIQUID` (Ax 4+7).  `alignment` usa `frequency_alignment` existente (Ax 8).  Cero constantes nuevas.
2. **No destructivo.** `SoupSim` y harness AP-* intactos — si el resource no existe en el world, el system no corre (`Res` falla silenciosamente con `get()`, o usar `Option<Res<...>>`).
3. **Test determinismo trivial.** Mismo seed + mismo resource inicial → misma inyección.
4. **Cache-friendly.** `SpeciesGrid` sigue siendo SoA contigua — el wrapper no cambia layout.
5. **Phase correcta.** `ChemicalLayer` es donde vive toda la química qe-based; el sistema nuevo encaja naturalmente.

## 5. No viola axiomas

| Ax | Cumplimiento |
|---|---|
| 1 | Todo qe: species_qe → qe del campo, misma unidad |
| 2 | Pool invariant respetado: lo inyectado se resta de la fuente (species_qe se dissipa vía `step_grid_reactions` existente) |
| 3 | Competencia preservada: reacciones compiten en `SoupSim`; el campo sólo recibe el neto |
| 4 | Dissipation: `SPECIES_TO_QE_COUPLING = DL` — aplicación directa |
| 5 | Conservación ≤ monotónica (sin creación de qe) |
| 6 | Sistema calibrable sin hardcoding — todo deriva de fundamentales |
| 7 | `frequency_alignment` con bandwidth es la forma infinitesimal |
| 8 | Núcleo del bridge: sólo inyecta donde freq alinea |

## 6. Costos

- Compilación: ≤1% (2 resources + 1 system)
- Runtime: O(w×h) por tick en ChemicalLayer ≈ 0.1 ms en grid 32×32
- Memoria: +0 bytes (wrapper transparent)
- Complejidad nueva: 1 function, 1 constante, 2 resources

## 7. Archivos modificados

| Archivo | Cambio |
|---|---|
| `src/layers/species_grid.rs` | + `SpeciesGridResource` wrapper + `#[derive(Resource)]` |
| `src/layers/reaction_network.rs` | + `ReactionNetworkResource` wrapper |
| `src/simulation/chemical/species_to_qe.rs` | **NUEVO** sistema de inyección |
| `src/simulation/chemical/mod.rs` | + `mod species_to_qe;` |
| `src/plugins/layers.rs` | + registro system |
| `src/blueprint/constants/chemistry.rs` | + `SPECIES_TO_QE_COUPLING` |
| `src/layers/reaction_network.rs` | + `mean_product_frequency()` helper |

## 8. Tests

- **Unit:** `species_to_qe_injection_system_respects_freq_alignment` — celda con freq desalineada ⇒ injection = 0.
- **Unit:** `coupling_derives_from_dissipation_liquid` — `SPECIES_TO_QE_COUPLING == DISSIPATION_LIQUID` assert const.
- **Integration:** `formose_spot_injects_qe_to_field` — seed 0 + spot → `field.total_qe()` crece monotónicamente en los primeros 100 ticks.
- **Regression:** `ap5_proptest_unaffected_without_resource` — sin `SpeciesGridResource` en el world, AP-5 proptest sigue pasando byte-idéntico.
- **Determinism:** `two_runs_same_seed_same_injection` — sin flaky.

## 9. Decisión revisable cuando

- Si AI-3 (calibración) concluye que las dos químicas divergen y una debe deprecarse, este ADR queda obsoleto si mass-action es la descartada.
- Si el overhead del sistema supera 1 ms en grids >128×128, considerar SIMD o skip-si-sin-spot.
- Si aparece un consumer del bridge inverso (qe → species), considerar ampliar a bidireccional; por ahora fuera de scope.
