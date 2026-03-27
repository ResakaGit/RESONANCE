# Sprint 14 — Motor Termodinámico de Color Cuantizado (GPU)

**Módulo:** `src/rendering/quantized_color/` (nuevo) + `assets/shaders/quantized_color.wgsl` (nuevo)
**Tipo:** Rendering — bridge stateless entre ECS y GPU.
**Onda:** H — Depende de Sprint 05 (funciones puras), Sprint 08 (EnergyVisual), Sprint 13 (LOD).
**Estado:** ⏳ Pendiente

## Objetivo

Implementar la proyección cuantizada de energía → color en GPU, usando paletas pre-computadas
en VRAM y un factor de precisión derivado de la distancia a cámara. O(1), branchless, determinista.

## Responsabilidades

### Sub-sprint 14A: Generación de Paletas (CPU, Startup)

- Módulo `palette_gen.rs`: funciones puras que generan `PaletteBlock` desde el `AlchemicalAlmanac`.
- Usa las funciones de `visual_derivation` (Sprint 05) para derivar colores.
- Cada `ElementDef` del Almanac produce una paleta de `n_max` colores.
- Las paletas se consolidan en un `PaletteRegistry` (Resource).
- Startup system (`palette_startup_system`) genera y sube a VRAM como `StorageBuffer`.
- Hot-reload del Almanac regenera la paleta afectada.

### Sub-sprint 14B: Factor de Precisión (CPU, Update)

- Sistema `factor_precision_system` en `Update`.
- Para cada entidad con `Materialized` + `Transform`:
  - Calcular distancia a la cámara activa (query `Camera` + `Transform`).
  - Mapear distancia → `ρ ∈ (0, 1]` usando bandas Near/Mid/Far del Sprint 13.
  - Escribir `ρ` en un nuevo componente ligero `QuantizedPrecision(f32)`.
- Si no hay cámara activa (tests/warmup): `ρ = 1.0` (máxima fidelidad).

### Sub-sprint 14C: Empaquetado de Payload (CPU, Update)

- Sistema `pack_visual_payload_system` en `Update`, después de `factor_precision_system`.
- Para cada entidad con `Materialized` + `BaseEnergy` + `QuantizedPrecision`:
  - `energia_interna` = clamp(qe / QE_REFERENCE, 0.0, 1.0).
  - `factor_precision` = `QuantizedPrecision.0`.
  - `n_max_id` = derivado del `WorldArchetype` del componente `Materialized`.
  - Empaquetar en `VisualPayload` y escribir al instance buffer.

### Sub-sprint 14D: Shader WGSL (GPU, Render)

- Archivo `assets/shaders/quantized_color.wgsl`.
- Fragment shader que implementa los 3 pasos de cuantización (branchless).
- Lee `VisualPayload` desde instance buffer.
- Lee `palettes` desde storage buffer (bind group).
- Emite `vec4<f32>` como color final del fragmento.
- Integración con Bevy material pipeline via `ExtendedMaterial` o custom `MaterialPlugin`.

## Tácticas

- **Reusar Sprint 05 exactamente.** No inventar colores nuevos en las paletas. `generate_palette` llama a `derive_color`, `derive_emission`, `derive_opacity` del Sprint 05. Las paletas son la discretización de esas funciones.
- **`QuantizedPrecision` es SparseSet.** Se añade/remueve por frame según visibilidad. No causa archetype thrashing porque es `SparseSet`.
- **El shader NO tiene condicionales.** Si el designer quiere branchless puro, usar `max(1.0, ...)` , `floor()`, `ceil()` (que son instrucciones nativas de GPU). Cero `if/else`.
- **Medir antes de optimizar.** Sprint 14D puede ser innecesario si el proyecto permanece en 2D/gizmos. Implementar 14A-14C primero (son útiles para CPU-side LOD de color) y 14D cuando el render bridge 3D esté preparado.
- **Fallback a magenta.** Si `n_max_id` apunta a una paleta no cargada, el shader emite `vec4(1, 0, 1, 1)` (magenta visible). No hace discard, no falla silenciosamente.
- **Respetar la coexistencia con Sprint 08.** El `EnergyVisual` sigue existiendo. El motor cuantizado opera como una ruta alternativa de rendering. Ambos se alimentan de las mismas funciones puras de Sprint 05.
- **REUTILIZAR `WorldgenPerfSettings` y `LodBand`.** `factor_precision_system` NO inventa sus propias bandas de distancia. Lee `WorldgenLodContext.focus_world` y las constantes `LOD_NEAR_MAX`/`LOD_MID_MAX` de `worldgen/lod.rs` para derivar `ρ`. La banda Near → `ρ=1.0`, Mid → interpolación, Far → `ρ=ρ_min`.
- **`quantized_index()` como early-out CPU-side.** Antes de llamar `derive_color` en `visual_derivation_update_changed_system`, comparar `quantized_index` actual vs previo. Si no cambió → skip. Esto reduce la presión sobre `max_visual_derivation_per_frame` sin modificar el sistema existente.
- **Para modo 2D (actual), `factor_precision` alimenta un CPU-side quantize.** Antes de tener el shader WGSL, se puede usar la función `quantized_index()` en CPU para reducir recálculos de `derive_color` — si `quantized_index` no cambió desde el frame anterior, skip el `derive_color`. Esto ya es una victoria de performance sin shader custom.

## Demarcación con Sprint 13 (capas de rendimiento existentes)

> **Regla:** Este sprint NO duplica las optimizaciones de Sprint 13. Opera en una capa distinta.

| Sprint 13 (existente) | Sprint 14 (nuevo) | Relación |
|-----------------------|-------------------|----------|
| `MaterializationCellCache` — cachea qué **forma** spawnear | Paletas — cachea qué **colores** posibles tiene un material | Ortogonales: forma ≠ color |
| LOD Near/Mid/Far — controla **frecuencia de tick** | `factor_precision(ρ)` — controla **resolución cromática** | Complementarios: Sprint 14 **lee** las bandas de Sprint 13 |
| `max_visual_derivation_per_frame` — presupuesto CPU | `quantized_index()` — early-out O(1) | Sprint 14 **reduce presión** sobre el presupuesto existente |
| `Changed<BaseEnergy>` filter — skip si no cambió | Shader GPU — no usa change detection | Independientes: GPU stateless |

```
Sprint 14 REUTILIZA: WorldgenPerfSettings, WorldgenLodContext, LodBand
Sprint 14 NO DUPLICA: MaterializationCellCache, max_visual_derivation_per_frame
Sprint 14 COMPLEMENTA: reduce recálculos CPU via early-out cuantizado
```

## NO hace

- No cambia la simulación, propagación ni materialización.
- No modifica el Almanac ni las capas ECS.
- No implementa PBR completo (eso es render bridge V6).
- No altera el Terrain Mesher (la integración es por la frontera `TerrainVisuals`).
- No reemplaza `MaterializationCellCache` (cachea forma, no color).
- No inventa nuevas bandas LOD (reutiliza Near/Mid/Far de Sprint 13).

## Dependencias

- Sprint 05 (`visual_derivation`): funciones puras de color — SSOT.
- Sprint 08 (`visual_system`): `EnergyVisual` componente — coexistencia.
- Sprint 13 (`performance_lod`): bandas Near/Mid/Far — reusar constantes.
- Sprint 01 (`contracts`): `WorldArchetype`, `Materialized` — tipos.
- `blueprint/almanac.rs`: `AlchemicalAlmanac`, `ElementDef`.
- `bevy::render`: StorageBuffer, custom material, shader pipeline.

## Criterio de aceptación

### 14A (Paletas)
- Test: `generate_palette` produce exactamente `n_max` colores para Ignis.
- Test: Color en índice 0 es oscuro (baja energía), color en índice n_max-1 es brillante (alta energía).
- Test: Paleta de Terra produce tonos marrones. Paleta de Aqua produce tonos azules.
- Test: `generate_palette` no hace panic con `n_max = 1` ni con `n_max = 0` (edge case → fallback).

### 14B (Factor de precisión)
- Test: entidad a distancia 0 → `ρ = 1.0`.
- Test: entidad a distancia > Far → `ρ = ρ_min` (> 0).
- Test: sin cámara → `ρ = 1.0` (fallback).
- Test: `ρ` es monótonamente decreciente con la distancia.

### 14C (Payload)
- Test: `VisualPayload` correctamente empaquetado desde `BaseEnergy(100)` + `QuantizedPrecision(0.5)`.
- Test: `energia_interna` está clampado a `[0, 1]` aun con `qe` desbordado.
- Test: `n_max_id` apunta a una paleta válida del `PaletteRegistry`.

### 14D (Shader)
- Test CPU: `quantized_index(0.52, 0.1, 100) == 49` (ejemplo del blueprint).
- Test CPU: `quantized_index(enorm, 1.0, n_max) == floor(enorm * (n_max-1))` (máxima fidelidad = sin cuantización extra).
- Test CPU: `quantized_index(cualquier_enorm, cualquier_rho, n_max) < n_max` (nunca out-of-bounds).
- Visual: demo app mostrando transición de color al acercar/alejar cámara (cuando render bridge 3D esté listo).

### General
- `cargo test` pasa.
- `PaletteRegistry.total_vram_bytes()` < 200 KB para el set completo.
- No regresión en los tests de Sprint 05, 08, 13.

## Referencia

- `docs/design/QUANTIZED_COLOR_ENGINE.md`
- `docs/arquitectura/blueprint_quantized_color.md`
- `docs/arquitectura/blueprint_v7.md` sección 5
