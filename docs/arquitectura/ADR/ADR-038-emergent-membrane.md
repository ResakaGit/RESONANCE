# ADR-038: Emergent Membrane — Cohesión sin componente Membrana

**Estado:** Propuesto
**Fecha:** 2026-04-14
**Contexto:** AUTOPOIESIS track, sprint AP-3

## Contexto

Una RAF detectada (AP-1) se difunde y se diluye sin contención espacial. Para que una closure persista debe mantener `[reactants] > umbral_de_autosostén` localmente. Eso requiere algún tipo de **frontera** que reduzca el flux de salida.

Pero declarar un componente `Membrane { thickness, permeability }` viola Axiom 6 (top-down design). Tampoco es fiel al cap. 6 del paper:

> Una llama no tiene membrana — tiene gradiente de oxígeno. Una vesícula lipídica emerge porque las cabezas polares minimizan energía libre frente al agua. La membrana es **un patrón observable**, no un objeto declarado.

## Alternativas evaluadas

| Opción | Modelo | Pros | Contras |
|--------|--------|------|---------|
| **A: Componente `Membrane`** | Entity envoltura con `permeability: [f32; MAX_SPECIES]` | Explícito, fácil de razonar | Top-down. Decide qué especies cruzan ANTES de simular. Viola Axiom 6. |
| **B: Cohesión emergente por gradiente de densidad** | Damping de difusión proporcional a `‖∇(Σ products)‖ × bond_energy_avg`. Sin componente nuevo. | Fiel a Axiom 6. Reutiliza TensionField (L11) conceptualmente. Parametrización mínima (1 constante: MEMBRANE_DAMPING). Membrana se forma, se cierra y se rompe automáticamente. | Más sutil de debuggear: el "objeto membrana" no existe en ECS, hay que inferirlo del campo. |
| **C: Membrana topológica** | Tag celdas como "interior" cuando su species mask matches closure products; flux entre interior/exterior tiene un tax fijo. | Determinista, fácil. | El tax es arbitrario (calibración manual). No emerge — se decreta. Híbrido pobre entre A y B. |

## Decision

**Opción B: cohesión emergente por gradiente de densidad de productos.**

```rust
// src/blueprint/equations/membrane.rs

pub fn local_gradient(
    grid: &[CellState], stride: usize,
    cell: (u16, u16), species_mask: &[bool],
) -> Vec2 {
    // ∇ discreto sobre Σ_{s ∈ mask} grid[cell].species[s]
    // Sobel 3x3 o forward-difference simple
}

pub fn membrane_strength(gradient_norm: f32, bond_energy_avg: f32) -> f32 {
    gradient_norm * bond_energy_avg * (1.0 - DISSIPATION_LIQUID)
}

pub fn damped_diffusion(raw_flux: f32, membrane_strength: f32) -> f32 {
    raw_flux * (-membrane_strength * MEMBRANE_DAMPING).exp()
}
```

### Justificación

1. **Axiom 6 (emergence) intacto.** No hay componente `Membrane`. Solo hay un campo escalar derivado en cada tick. La "vesícula" es lo que un humano observa al mirar el heatmap; el simulador no sabe qué es.

2. **Reuso conceptual de L11 (TensionField).** L11 ya modela cohesión en otro contexto (acoplamiento entre entidades cercanas). Aquí aplicamos la misma intuición a especies químicas — sin acoplar literalmente al componente, pero respetando el patrón.

3. **Auto-rotura.** Si la closure muere (production rate cae), el gradiente se relaja, el damping cae a 0, la vesícula se "abre" naturalmente. No hay caso especial de "membrane breaks" — simplemente desaparece la condición que la sostenía.

4. **Compatible con AP-4 (fission).** El blob_topology de AP-4 es flood-fill sobre `membrane_strength > THRESHOLD` — directo de leer del campo emergente.

5. **Parametrización mínima.** Una sola constante (`MEMBRANE_DAMPING`), idealmente derivable de las 4 fundamentales. Propuesta: `MEMBRANE_DAMPING = 1.0 / DISSIPATION_LIQUID = 50.0`.

### Lo que NO hace este ADR

- **No** crea `Membrane` component, resource, ni event.
- **No** asume bicapa lipídica ni química específica.
- **No** asume que toda closure produce membrana — solo ocurre si los productos son densos y tienen bond_energy alto.

## No viola axiomas

| Axiom | Cumplimiento |
|-------|-------------|
| 1 | Membrana = manifestación del campo de qe distribuido. |
| 2 | Damping reduce flux **out** pero no destruye qe — pool invariant intacto. |
| 4 | El factor `(1 - DISSIPATION_LIQUID)` es disipación explícita. |
| 6 | **Crítico.** Membrana no se declara; emerge del gradiente. |
| 7 | Damping exponencial = atenuación con distancia (efectiva). |
| 8 | `bond_energy_avg` es frequency-modulated en composiciones futuras. |

## Riesgos

- **R1: Damping exponencial puede crear vesículas demasiado estables.** Mitigación: cap superior `damped_flux ≥ raw_flux × 0.01` para garantizar que algo siempre escapa (Axiom 4 estricto). Empíricamente verificable en AP-5.
- **R2: Detección de blobs requiere flood-fill cada tick.** Mitigación: se hace en AP-4 cada N=50 ticks, no por tick.

## Archivos modificados

| Archivo | Cambio |
|---------|--------|
| `src/blueprint/equations/membrane.rs` | **NUEVO** — 3 pure fns |
| `src/blueprint/equations/reaction_kinetics.rs` | Modificar `diffuse_species` para aceptar `damping_field: Option<&[f32]>` |
| `src/simulation/chemical/membrane_inference.rs` | **NUEVO** — system que computa `membrane_strength_field` cada tick |
| `src/batch/scratch.rs` | Channel viz `membrane_strength: f32` por celda (solo viz, no afecta sim) |

## Tests

- 12 unit tests (gradient correctness, damping monotonicity, blob coherence)
- 3 integration tests (regression sopa homogénea, blob estable 100 ticks, ruptura tras matar food)

## Decisión revisable cuando

- En 3D la formulación de gradient cambia (6-vecino + Sobel 3D).
- Si el track propone bilayers explícitos (lipid analog), reabrir A vs B.
