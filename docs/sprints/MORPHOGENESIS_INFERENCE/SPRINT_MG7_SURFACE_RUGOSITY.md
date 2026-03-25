# Sprint MG-7 — Surface Rugosity (ley cuadrático-cúbica → GF1)

**Módulo:** `src/simulation/morphogenesis.rs` + `src/layers/` (componente nuevo) + `src/geometry_flow/`
**Tipo:** Sistema morfológico + componente SparseSet + extensión controlada del influjo geométrico GF1.
**Onda:** D — Requiere MG-6 (`EntropyLedger`).
**Estado:** ⏳ Pendiente

## Objetivo

Cuando **Q/V** es alto, la física del modelo exige más área de intercambio sin crecer volumen proporcionalmente: traducir eso a **rugosity** ∈ `[RUGOSITY_MIN, RUGOSITY_MAX]` mediante `inferred_surface_rugosity` (MG-1) y propagar a GF1 para fenotipos con pliegues / aletas / radiadores emergentes.

**Resultado emergente:** criatura terrestre con alto Q (300 qe/tick) y bajo V (radio 1.0) en ΔT moderado (120K) → rugosity ≈ 2.8 (pliegues/aletas). Criatura acuática con bajo Q (50 qe/tick) y gran V (radio 2.0) → rugosity ≈ 1.1 (casi lisa, buen hidrodinámico).

## Responsabilidades

### MG-7A: Componente `MorphogenesisSurface`

**Decisión de diseño:** Opción A — componente ECS ≤4 campos, traducido a DTO GF1 por el código que arma la malla.

```rust
/// Rugosidad de superficie inferida por balance termodinámico.
/// Controla complejidad geométrica superficial en GF1.
#[derive(Component, Reflect, Debug, Clone, Copy, PartialEq)]
#[reflect(Component)]
#[component(storage = "SparseSet")]
pub struct MorphogenesisSurface {
    /// Ratio de superficie real vs esfera equivalente [1.0, 4.0].
    /// 1.0 = liso, 1.5–2.5 = pliegues, 2.5–4.0 = aletas/radiadores.
    pub rugosity: f32,
    /// Q/V ratio usado para el cálculo (diagnóstico).
    pub heat_volume_ratio: f32,
}
```

- 2 campos — cumple regla ≤4.
- SparseSet: solo entidades con `EntropyLedger`.
- Registrar en `LayersPlugin` con `Reflect`.
- Implementar `MorphogenesisSurface::new(rugosity: f32, qv: f32) -> Self` con clamp de rugosity a `[RUGOSITY_MIN, RUGOSITY_MAX]`.

**Mapeo a GF1 (`GeometryInfluence`):**
```
rugosity ∈ [1.0, 1.5) → detail *= 1.0  (sin subdivisión extra)
rugosity ∈ [1.5, 2.5) → detail *= 1.0 + (rugosity - 1.5) * 0.5  (subdivisión moderada)
rugosity ∈ [2.5, 4.0] → detail *= 1.5 + (rugosity - 2.5) * 0.33 (subdivisión + protuberancias)
```
Función pura `rugosity_to_detail_multiplier(rugosity: f32) -> f32` en `equations/`. El código que arma `GeometryInfluence` multiplica `detail` existente por este factor.

### MG-7B: `surface_rugosity_system`

```rust
/// Infiere rugosidad de superficie desde balance térmico Q/V.
pub fn surface_rugosity_system(
    mut commands: Commands,
    query: Query<
        (Entity, &EntropyLedger, &SpatialVolume, &AmbientPressure),
        Without<Dead>,
    >,
    mut surface_query: Query<&mut MorphogenesisSurface>,
) {
    for (entity, ledger, volume, pressure) in &query {
        let q_total = ledger.total_heat_generated;
        let r = volume.radius();
        let vol = (4.0 / 3.0) * std::f32::consts::PI * r * r * r;
        let t_core = ...; // misma convención que MG-3/MG-5
        let t_env  = ...; // derivado de AmbientPressure
        let h = constants::DEFAULT_CONVECTION_COEFF;

        let rug = equations::inferred_surface_rugosity(q_total, vol, t_core, t_env, h);
        let qv  = if vol > EPSILON { q_total / vol } else { 0.0 };

        let new_surface = MorphogenesisSurface::new(rug, qv);

        if let Ok(mut existing) = surface_query.get_mut(entity) {
            if (existing.rugosity - new_surface.rugosity).abs() > RUGOSITY_EPSILON {
                *existing = new_surface;
            }
        } else {
            commands.entity(entity).insert(new_surface);
        }
    }
}
```

- **Phase:** `Phase::MorphologicalLayer`.
- **Query:** 4 tipos — `EntropyLedger`, `SpatialVolume`, `AmbientPressure`, `Entity`.
- **`q_total`:** `EntropyLedger.total_heat_generated` — fuente canónica única (MG-6).
- **T_core / T_env / h:** misma convención que MG-3 y MG-5 (documentar tabla de mapeo ECS→escalar).
- **Orden:** `.after(shape_optimization_system)` según contrato de pipeline. Antes de `albedo_inference_system`.
- **Guard:** epsilon configurable `RUGOSITY_EPSILON`.

### MG-7C: Funciones puras

```rust
/// Traduce rugosity a multiplicador de detail para GeometryInfluence.
/// Controla cuánta subdivisión/protuberancia aplica GF1.
pub fn rugosity_to_detail_multiplier(rugosity: f32) -> f32 {
    let r = rugosity.clamp(RUGOSITY_MIN, RUGOSITY_MAX);
    if r < 1.5 {
        1.0
    } else if r < 2.5 {
        1.0 + (r - 1.5) * 0.5
    } else {
        1.5 + (r - 2.5) * 0.33
    }
}
```

- Rango retorno: `[1.0, ~2.0]`. Monotónica creciente.
- Sin discontinuidades (C⁰ continua en los joints).
- `inferred_surface_rugosity` ya existe en MG-1. No se añade aritmética nueva al ecuaciones.

### MG-7D: Constantes

```rust
// --- Morfogénesis: Surface Rugosity System ---
pub const RUGOSITY_EPSILON: f32 = 0.02;             // Guard change detection para rugosity
pub const RUGOSITY_MAX_DETAIL_MULTIPLIER: f32 = 2.0; // Tope de subdivisión en GF1
```

(Constantes `RUGOSITY_MIN`=1.0 y `RUGOSITY_MAX`=4.0 ya en MG-1.)

### MG-7E: Presupuesto geométrico (tri-count)

- **Regla:** `detail_multiplier * base_segments ≤ MAX_SEGMENTS_PER_ENTITY`.
- `MAX_SEGMENTS_PER_ENTITY = 64` (constante en `geometry_flow/` o `constants.rs`).
- Si `detail * multiplier` excede el tope → clamp `detail` al máximo permitido.
- LOD Far: `detail_multiplier = 1.0` siempre (sin subdivisión extra). Near: full multiplier. Mid: `lerp(1.0, multiplier, 0.5)`.

### MG-7F: Integración GF1

- El código que construye `GeometryInfluence` lee `MorphogenesisSurface` si está presente.
- `influence.detail = base_detail * rugosity_to_detail_multiplier(surface.rugosity)`.
- GF1 sigue stateless: recibe `GeometryInfluence` → produce mesh. No lee ECS directamente.
- Si `MorphogenesisSurface` no está presente → `detail` sin modificar (backward compatible).

## Tácticas

- **Determinismo.** Mismos Q, V, T → misma rugosity → mismo detail. Sin RNG.
- **Fenotipos claros.** La tabla de rugosity es intencionalmente simple:
  - 1.0–1.5: liso (medusa, pez torpedo).
  - 1.5–2.5: pliegues (intestino, coral blando).
  - 2.5–4.0: aletas/radiadores (dragón, radiador industrial).
- **Tri-count conservador.** `RUGOSITY_MAX_DETAIL_MULTIPLIER = 2.0` → como máximo el doble de geometría que una entidad lisa. Suficiente para pliegues visibles sin explosión.
- **Guard change detection.** Rugosity cambia lento (depende de Q y V que evolucionan gradualmente). `RUGOSITY_EPSILON = 0.02` evita escrituras innecesarias.
- **Debug overlay.** Opcional en `DebugPlugin`: overlay con `rugosity` y `Q/V` por entidad. No bloqueante para este sprint.

## NO hace

- No implementa albedo (MG-5) ni drag optimizer (MG-4).
- No implementa CFD/FEM.
- No reescribe el motor GF1 — solo influye en `detail`.
- No modifica `EntropyLedger` — solo lo lee.

## Dependencias

- MG-1 (`inferred_surface_rugosity`, constantes `RUGOSITY_MIN`/`RUGOSITY_MAX`).
- MG-6 (`EntropyLedger` — fuente de `total_heat_generated`).
- GF1 (`geometry_flow/`) — `GeometryInfluence` (campo `detail`).
- `src/layers/volume.rs` — `SpatialVolume` (1 campo: `radius`).
- `src/layers/pressure.rs` — `AmbientPressure` (2 campos: `delta_qe_constant`, `terrain_viscosity`).

## Criterios de aceptación

### MG-7A (Componente)
- Test: `MorphogenesisSurface::new(2.5, 10.0)` → `rugosity = 2.5`, `heat_volume_ratio = 10.0`.
- Test: `MorphogenesisSurface::new(0.5, 10.0)` → `rugosity = RUGOSITY_MIN` (1.0) — clamped.
- Test: `MorphogenesisSurface::new(6.0, 10.0)` → `rugosity = RUGOSITY_MAX` (4.0) — clamped.
- Test: `MorphogenesisSurface` es `Copy`, `SparseSet`, `Reflect`.

### MG-7B (Sistema — integración)
- Test: alto Q + bajo V — `EntropyLedger { total_heat: 300.0, ... }`, `SpatialVolume { radius: 1.0 }`, T_core=400, T_env=280, h=10:
  - `vol = 4.19`, `A_sphere = 12.57`, `A_needed = 300 / (10 * 120) = 0.25`.
  - Rugosity = `(0.25 / 12.57).clamp(1.0, 4.0) = 1.0` (sorpresa: poco calor relativo a la superficie).
  - **Caso exigente:** T_core=400, T_env=390, h=10 → ΔT=10 → `A_needed = 300 / (10 * 10) = 3.0`. Rugosity = `(3.0 / 12.57).clamp(1.0, 4.0) ≈ 1.0` (aún bajo).
  - **Caso extremo:** Q=3000, T_core=400, T_env=399, h=10 → ΔT=1 → `A_needed = 300`. Rugosity = `(300 / 12.57).clamp(1.0, 4.0) = 4.0` (aletas máximas).
- Test: bajo Q → rugosity ≈ `RUGOSITY_MIN` (1.0).
  - `EntropyLedger { total_heat: 10.0, ... }`, radius=2.0, T_core=400, T_env=280 → rugosity ≈ 1.0.
- Test: ΔT → 0 (T_core ≈ T_env) → rugosity = `RUGOSITY_MAX` (4.0) — no puede disipar por convección.
- Test: entidad sin `EntropyLedger` → no recibe `MorphogenesisSurface` (backward compatible).

### MG-7C (Funciones puras)
- Test: `rugosity_to_detail_multiplier(1.0)` = `1.0`.
- Test: `rugosity_to_detail_multiplier(1.5)` = `1.0`.
- Test: `rugosity_to_detail_multiplier(2.0)` = `1.25`.
- Test: `rugosity_to_detail_multiplier(2.5)` = `1.5`.
- Test: `rugosity_to_detail_multiplier(4.0)` ≈ `2.0`.
- Test: monotónica — `r1 < r2` → `multiplier(r1) ≤ multiplier(r2)` para 100 muestras en [1.0, 4.0].

### MG-7E (Presupuesto geométrico)
- Test: `detail * rugosity_to_detail_multiplier(RUGOSITY_MAX) ≤ MAX_SEGMENTS_PER_ENTITY` con `detail = 32` (techo base de GF1).
- Test: si `base_detail = 40` y `multiplier = 2.0` → clamped a 64 (no excede).

### General
- `cargo test --lib` sin regresión.
- Todos los tipos con `///` doc-comments.

## Referencias

- `docs/design/MORPHOGENESIS.md` §3.2.5, §6 MG-7, §7 (riesgo explosión geométrica)
- `docs/sprints/MORPHOGENESIS_INFERENCE/README.md` — pipeline MorphologicalLayer
- `docs/sprints/GEOMETRY_FLOW/README.md` — GF1 stateless
- `src/geometry_flow/mod.rs` — `GeometryInfluence` (campo `detail: f32`)
- `src/blueprint/equations/` — `inferred_surface_rugosity()` (MG-1)
