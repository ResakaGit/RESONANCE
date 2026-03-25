# Sprint MG-5 — Albedo Inference (Balance radiativo → α)

**Módulo:** `src/simulation/morphogenesis.rs` + `src/layers/` (componente nuevo) + `src/blueprint/equations/`
**Tipo:** Sistema morfológico + componente marker SparseSet + función pura auxiliar.
**Onda:** C — Paralelo con MG-4 y MG-6 **como planificación de sprints**; en **runtime** el ledger (MG-6) corre en `MetabolicLayer` antes que este sistema en `MorphologicalLayer`.
**Estado:** ⏳ Pendiente

## Objetivo

Derivar **albedo superficial α** desde balance térmico usando `inferred_albedo` (MG-1). Fuente de Q metabólico: tras MG-6, usar **solo** `EntropyLedger.total_heat_generated` (garantía de una sola definición de ΣQ). Antes del merge de MG-6, leer agregado provisional `MetabolicGraph.total_entropy_rate * T_core` como proxy, documentando equivalencia temporal.

Exponer α en `InferredAlbedo` según `docs/arquitectura/blueprint_morphogenesis_inference.md` §2.

**Resultado emergente:** criatura caliente (Q=300) bajo sol intenso (I=80) → α ≈ 0.85 (blanca, refleja). Criatura fría (Q=20) en cueva (I≈0) → α = 0.5 (fallback neutral).

## Responsabilidades

### MG-5A: Componente `InferredAlbedo`

```rust
/// Albedo inferido por balance radiativo. α ∈ [ALBEDO_MIN, ALBEDO_MAX].
/// Solo entidades con MetabolicGraph lo reciben.
#[derive(Component, Reflect, Debug, Clone, Copy, PartialEq)]
#[reflect(Component)]
#[component(storage = "SparseSet")]
pub struct InferredAlbedo {
    /// Reflectancia superficial [0.05, 0.95].
    pub albedo: f32,
}
```

- 1 campo — cumple regla ≤4.
- SparseSet: solo entidades "vivas complejas".
- Registrar en `LayersPlugin` con `Reflect`.
- Implementar `InferredAlbedo::new(albedo: f32) -> Self` con clamp a `[ALBEDO_MIN, ALBEDO_MAX]`.

### MG-5B: Función pura auxiliar `irradiance_effective_for_albedo`

```rust
/// Extrae irradiancia solar efectiva de un IrradianceReceiver para el cálculo de albedo.
/// Unidades consistentes con fotosíntesis/eco (photon_density * absorbed_fraction).
/// Si el componente no existe → retorna 0.0 (sin sol).
pub fn irradiance_effective_for_albedo(
    photon_density: f32,
    absorbed_fraction: f32,
) -> f32 {
    (photon_density * absorbed_fraction).max(0.0)
}
```

- Función pura en `equations/`. Sin conversión de unidades — `IrradianceReceiver.photon_density` ya está en unidades de irradiancia del modelo.
- Si `photon_density = 0` o `absorbed_fraction = 0` → `I = 0` → albedo cae a fallback en `inferred_albedo`.

### MG-5C: `albedo_inference_system`

```rust
/// Infiere albedo desde balance radiativo: Q_met, irradiancia, geometría, convección.
pub fn albedo_inference_system(
    mut commands: Commands,
    query: Query<
        (Entity, &MetabolicGraph, &SpatialVolume, &AmbientPressure,
         Option<&IrradianceReceiver>, Option<&EntropyLedger>),
        Without<Dead>,
    >,
    mut albedo_query: Query<&mut InferredAlbedo>,
) {
    for (entity, graph, volume, pressure, irradiance, ledger) in &query {
        // 1. Q metabólico: preferir EntropyLedger si existe, sino proxy del grafo.
        let q_met = match ledger {
            Some(l) => l.total_heat_generated,
            None    => graph.total_entropy_rate * t_core,  // proxy temporal
        };
        // 2. Irradiancia efectiva.
        let i_solar = irradiance
            .map(|ir| equations::irradiance_effective_for_albedo(ir.photon_density, ir.absorbed_fraction))
            .unwrap_or(0.0);
        // 3. Geometría: A_proj ≈ π * r², A_surf ≈ 4π * r².
        let r = volume.radius();
        let proj_area = std::f32::consts::PI * r * r;
        let surf_area = 4.0 * std::f32::consts::PI * r * r;
        // 4. T_core, T_env: misma convención que MG-3.
        let t_core = equations::equivalent_temperature(...);
        let t_env  = ...;
        // 5. Albedo.
        let alpha = equations::inferred_albedo(
            q_met, i_solar, proj_area,
            constants::DEFAULT_EMISSIVITY, t_core, t_env,
            surf_area, constants::DEFAULT_CONVECTION_COEFF,
        );
        // 6. Guard + insert/update.
        if let Ok(mut existing) = albedo_query.get_mut(entity) {
            if (existing.albedo - alpha).abs() > ALBEDO_EPSILON {
                existing.albedo = alpha;
            }
        } else {
            commands.entity(entity).insert(InferredAlbedo::new(alpha));
        }
    }
}
```

- **Phase:** `Phase::MorphologicalLayer`.
- **Query:** 4 tipos obligatorios + 2 opcionales (justificación: `IrradianceReceiver` y `EntropyLedger` pueden no existir, handled con `Option`).
- **Pura:** toda aritmética delegada a `equations::inferred_albedo` (MG-1). Sin fórmulas inline.
- **Orden:**
  - **Con MG-7:** `.after(surface_rugosity_system)` (contrato de pipeline).
  - **Sin MG-7 (staging):** `.after(shape_optimization_system)` — re-enlazar al mergear MG-7.
- **Fallback sin IrradianceReceiver:** `I = 0` → `inferred_albedo` retorna `ALBEDO_FALLBACK` (0.5). Determinista.

### MG-5D: Constantes

```rust
// --- Morfogénesis: Albedo System ---
pub const ALBEDO_EPSILON: f32 = 0.005;              // Guard change detection para α
```

(Constantes de albedo min/max/fallback, emissivity, convección ya definidas en MG-1.)

### MG-5E: Integración visual

- El sistema existente `shape_color_inference_system` **lee** `InferredAlbedo` si está presente.
- Blend documentado: `luminosity_final = luminosity_base * (0.3 + 0.7 * albedo)`. Matiz elemental (Hz) se preserva; solo cambia luminosidad.
- Si `InferredAlbedo` no está presente → color sin modificar (backward compatible).

## Tácticas

- **Clamp duro** en la pura y en el componente. Doble barrera: `inferred_albedo` clampea internamente + `InferredAlbedo::new` clampea al construir. Imposible tener α fuera de rango.
- **Sin RNG.** Determinismo total.
- **Proxy temporal consciente.** El proxy `total_entropy_rate * T_core` sobrestima Q si hay entropía por activación (vs solo disipación). Documentar en código: "// TODO: replace with EntropyLedger.total_heat_generated when MG-6 merges".
- **Una sola definición de Q.** Tras merge de MG-6, la fuente canónica es `EntropyLedger.total_heat_generated` — eliminar el branch proxy y dejar solo el path del ledger.
- **Test de coherencia post-MG-6:** α computado con proxy vs α computado con ledger deben coincidir dentro de `5%` para el caso estándar (3-nodo, flujo estable).

## NO hace

- No reimplementa `field_linear_rgb_from_hz_purity` ni interferencia compuesta.
- No implementa ledger ni cadena Writer (MG-6).
- No implementa rugosity (MG-7).
- No modifica el cálculo de irradiancia existente — solo lee.

## Dependencias

- MG-1 (`inferred_albedo`, constantes albedo/radiación/convección).
- MG-3 (grafo actualizado con `total_entropy_rate`; fuente de Q proxy).
- MG-6 (`EntropyLedger`) — fuente canónica de Q post-merge.
- `src/layers/irradiance.rs` — `IrradianceReceiver` (2 campos: `photon_density`, `absorbed_fraction`).
- `src/layers/volume.rs` — `SpatialVolume` (1 campo: `radius`).
- `src/layers/pressure.rs` — `AmbientPressure` (2 campos: `delta_qe_constant`, `terrain_viscosity`).

## Criterios de aceptación

### MG-5A (Componente)
- Test: `InferredAlbedo::new(0.7)` → `albedo = 0.7`.
- Test: `InferredAlbedo::new(-0.5)` → `albedo = ALBEDO_MIN` (0.05).
- Test: `InferredAlbedo::new(1.5)` → `albedo = ALBEDO_MAX` (0.95).
- Test: `InferredAlbedo` es `Copy`, `SparseSet`, `Reflect`.

### MG-5B (Función auxiliar)
- Test: `irradiance_effective_for_albedo(50.0, 0.8)` → `40.0`.
- Test: `irradiance_effective_for_albedo(0.0, 0.8)` → `0.0`.
- Test: `irradiance_effective_for_albedo(50.0, 0.0)` → `0.0`.
- Test: `irradiance_effective_for_albedo(-5.0, 0.8)` → `0.0` (clamp negativo).

### MG-5C (Sistema — integración)
- Test: organismo con alto Q (q_met=300) + alto I solar (photon_density=100, absorbed=0.8) + T_core=500, T_env=280 → α > 0.7 (criatura caliente en desierto → clara).
- Test: organismo con bajo Q (q_met=20) + bajo I (photon_density=5, absorbed=0.5) + T_core=350, T_env=300 → α < 0.3 (criatura fría en sombra → oscura).
- Test: I_solar = 0 (sin `IrradianceReceiver`) → α = `ALBEDO_FALLBACK` (0.5).
- Test: entidad sin `MetabolicGraph` → no recibe `InferredAlbedo` (backward compatible, no panic).
- Test: entidad con `MetabolicGraph` pero sin `IrradianceReceiver` → fallback, no panic.
- Test: α siempre ∈ `[ALBEDO_MIN, ALBEDO_MAX]` para combinación de inputs extremos (Q=0, Q=10000, I=0, I=1000).

### MG-5E (Visual)
- Test: `luminosity_final` con albedo=0.05 → `0.3 + 0.7 * 0.05 = 0.335` (oscuro).
- Test: `luminosity_final` con albedo=0.95 → `0.3 + 0.7 * 0.95 = 0.965` (claro).
- Test: sin `InferredAlbedo` → luminosity sin modificar.

### General
- `cargo test --lib` sin regresión.
- Todos los tipos con `///` doc-comments.

## Referencias

- `docs/design/MORPHOGENESIS.md` §3.3 (tabla `albedo_inference_system`), §6 MG-5
- `docs/arquitectura/blueprint_morphogenesis_inference.md` §2 (`InferredAlbedo`)
- `src/layers/irradiance.rs` — `IrradianceReceiver` (SparseSet, 2 campos)
- `src/blueprint/equations/` — `inferred_albedo()` (MG-1), `equivalent_temperature()`
- `docs/sprints/MORPHOGENESIS_INFERENCE/README.md` — contrato de pipeline MG
