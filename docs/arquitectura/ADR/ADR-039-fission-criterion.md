# ADR-039: Fission Criterion — Presión interna vs volumen vs decoherencia

**Estado:** Propuesto
**Fecha:** 2026-04-14
**Contexto:** AUTOPOIESIS track, sprint AP-4

## Contexto

Una vesícula contenida (AP-3) crece pero no se replica. Para cerrar el invariante "lo que persiste copió antes de disiparse" hace falta un trigger de fisión emergente. La pregunta: ¿qué condición física dispara la división?

Ninguna respuesta es trivial — la elección define qué tipo de vida emerge.

## Alternativas evaluadas

| Opción | Criterio | Pros | Contras |
|--------|----------|------|---------|
| **A: Volumen** | `if blob.cell_count > FISSION_VOLUME → split` | Simple, determinista, paralela a fisión bacteriana clásica | Threshold arbitrario. No emerge — se decreta el "tamaño máximo". Viola Axiom 6. |
| **B: Presión interna vs cohesión** | `if internal_production / cohesion_capacity > FISSION_PRESSURE_RATIO → split` | Físicamente fundado (Pross "kinetic instability"). El umbral es derivable de las 4 fundamentales. La división emerge cuando el sistema literalmente no puede contenerse. | Requiere computar production rate por blob (extra trabajo). Sensible a discretización del grid. |
| **C: Decoherencia oscilatoria** | `if phase_variance(blob) > FISSION_PHASE_THRESHOLD → split along phase boundary` | Native a Axiom 8. Captura "una región oscila a f₁, otra a f₂ → se separan". | Complejo. Requiere campo de fase per-celda. PCA + clustering = caro. Físicamente menos universal que B. |

## Decision

**Opción B: presión interna vs decay diffusivo (Pross rate/rate).**

> **Revisión 2026-04-14 (AP-6d fix).**  La formulación original
> (`production_extensive / cohesion_capacity`) era dimensionalmente
> inconsistente: numerador `[qe·T⁻¹]`, denominador `[qe]`, ratio `[T⁻¹]`,
> umbral adimensional.  El ratio empírico topaba en ~0.23 sobre formose
> vs. umbral 50 (gap 200×), y dependía del timestep (violación sutil de
> Axiom 6).  La corrección espeja `raf::kinetic_stability`: el decay se
> computa como `DISSIPATION_LIQUID × Σ productos_en_blob`, ambos lados
> del ratio viven en `[qe·T⁻¹]`, el resultado es adimensional y
> timestep-invariante.

```rust
// src/blueprint/equations/fission.rs

pub fn decay_rate(
    blob: &BlobIndex,
    grid: &SpeciesGrid,
    product_mask: &[bool; MAX_SPECIES],
) -> f32 {
    // Σ (productos enmascarados en celdas del blob) × SPECIES_DIFFUSION_RATE
    // Espejo espacial de `kinetic_stability` denominator.  Unidad: [qe·T⁻¹].
}

pub fn pressure_ratio(
    blob: &BlobIndex,
    grid: &SpeciesGrid,
    network: &ReactionNetwork,
    product_mask: &[bool; MAX_SPECIES],
    bandwidth: f32,
) -> f32 {
    let decay = decay_rate(blob, grid, product_mask);
    if decay <= KINETIC_STABILITY_EPSILON { return 0.0; }
    internal_production(blob, grid, network, bandwidth) / decay
}

// Trigger (sin cambios):
if pressure_ratio(blob, …) > FISSION_PRESSURE_RATIO {
    let axis = pinch_axis(&blob.cells);
    apply_fission(grid, blob, axis, lineage_parent);
}
```

Donde `FISSION_PRESSURE_RATIO = DISSIPATION_PLASMA / DISSIPATION_SOLID = 50.0` — derivado de las 4 fundamentales, no calibrado.  La membrana sigue
regulando la **detección** del blob (`find_blobs` via `strength_field`) y la
**retención** de flux (`damped_flux_factor` en `diffuse_species`), pero el
criterio de fisión ya no compara producción contra energía de pared sino
contra la tasa de disipación natural del blob — equivalente físico pero
dimensionalmente correcto.

### Justificación

1. **Físicamente fundado.** Pross (cap. 5 del paper) argumenta que la replicación es un acto termodinámicamente favorable porque libera la presión de un sistema que produce más energía/materia de la que puede contener. La inestabilidad cinética **es** el motor de la fisión.

2. **Axiom 6 respetado.** Ningún número mágico. El threshold **se deriva**: el ratio plasma/sólido es la diferencia máxima entre estados de materia, una propiedad física del simulador, no una elección.

3. **Auto-regulación.** Una closure débil (production rate baja) no se divide nunca → muere por dilución (correcto). Una closure fuerte se divide → ambos hijos heredan la red de reacciones, se difunden, comienza el linaje.

4. **Conservativa.** `apply_fission` es una redistribución espacial pura — `Σ species pre = Σ species post`. La fisión no crea ni destruye materia; solo separa el dominio.

5. **Linaje natural.** Cada fisión emite `FissionEvent { parent_lineage, child_lineages: [u64; 2] }`. El árbol genealógico se construye sin componente extra — solo registro de eventos.

### Por qué NO Opción A

Volumen como trigger es la solución de juego de mesa. Funciona, pero hace que toda vida emergente luzca igual: vesículas que crecen hasta tamaño X y dividen. La opción B permite **diversidad de fenotipos**: closures con baja producción son grandes y raras; con alta producción son chicas y prolíficas. **Esto es lo que el paper predice** (cap. 5: "los que aprenden a reconstruirse son los únicos que siguen existiendo").

### Por qué NO Opción C

Decoherencia es elegante pero introduce un mecanismo nuevo (campo de fase per-celda) sin evidencia empírica clara de cuándo pinchea. B es más simple y físicamente robusto. C queda como extensión futura para un track posterior (¿AP+? "differentiation").

## No viola axiomas

| Axiom | Cumplimiento |
|-------|-------------|
| 1 | Energía conservada en fisión (redistribución, no creación). |
| 2 | Pool invariant: `Σ qe pre = Σ qe post`. |
| 3 | Closures compiten por food → fisión depende de producción → emerge selección. |
| 4 | Revisión 2026-04-14: decay explícitamente `DISSIPATION_LIQUID × Σ productos` — aplicación directa de Axiom 4, no sólo tax post-fisión. `apply_fission` sigue cobrando `DISSIPATION_PLASMA × qe_blob`. |
| 5 | Conservación local con disipación. |
| 6 | **Crítico.** Threshold derivado, no decretado. Volumen NO entra en el criterio.  Revisión 2026-04-14: ratio adimensional ⇒ timestep-invariante (fórmula anterior tenía dependencia implícita de `dt`, violación sutil). |
| 7 | Pinch axis es físicamente coherente con la geometría del blob. `SPECIES_DIFFUSION_RATE` en el decay es la forma infinitesimal de Axiom 7 sobre productos. |
| 8 | Compatible con extensión Opción C en el futuro. `mass_action_rate` en la producción sigue incluyendo `frequency_alignment`. |

## Costos

- `find_blobs` flood-fill: O(grid_size). 256 celdas → < 0.1ms.
- `pressure_ratio` por blob: O(perimeter + cells). Despreciable.
- `apply_fission` por evento: O(blob_cells). Pocos eventos por tick (~0–3).
- Total: < 0.5ms en target hardware.

## Archivos modificados

| Archivo | Cambio |
|---------|--------|
| `src/blueprint/equations/blob_topology.rs` | **NUEVO** — flood-fill, perimeter, PCA |
| `src/blueprint/equations/fission.rs` | **NUEVO** — `pressure_ratio`, `pinch_axis`, `apply_fission` |
| `src/simulation/chemical/fission.rs` | **NUEVO** — system + `FissionEvent` |
| `src/resources/lineage.rs` | **NUEVO** — `LineageRegistry` resource |
| `src/events.rs` | Add `FissionEvent` |

## Tests

- 20 unit tests (flood-fill, PCA, conservation post-fission, threshold behavior)
- 3 integration tests (closure sub-threshold no se divide, supra-threshold se divide ≤50t, ambos hijos vivos post-fission)

## Decisión revisable cuando

- Si AP-5 muestra que la "vida emergente" es monotónica (todo pinchea en eje vertical), evaluar Opción C.
- Si en 3D la PCA 2D no captura ejes complejos, extender a 3D PCA.
- Si los linajes explotan combinatorialmente, considerar GC de lineage IDs sin descendencia activa.
