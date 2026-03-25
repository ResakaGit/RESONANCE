# Blueprint: Living Organ Inference — Órganos Stateless para Entidades Vivas

Referencia de contrato alineada al track [`docs/sprints/LIVING_ORGAN_INFERENCE/README.md`](../sprints/LIVING_ORGAN_INFERENCE/README.md), al módulo de inferencia de partes [`blueprint_energy_field_inference.md`](blueprint_energy_field_inference.md), y al ecosistema autopoiético [`blueprint_ecosystem_autopoiesis.md`](blueprint_ecosystem_autopoiesis.md).
Template base: [`00_contratos_glosario.md`](00_contratos_glosario.md).

## 1) Propósito y frontera

- **Qué resuelve:** cerrar la brecha entre energía y forma orgánica. Permite que entidades vivas — plantas y animales — manifiesten **órganos diferenciados** (pétalos, hojas, raíces, espinas, extremidades, ojos) cuya presencia, cantidad y forma se infiere del estado energético, fase de ciclo de vida, y perfil de la entidad. Cada órgano es una **feature geométrica stateless** — computada al vuelo, nunca almacenada como componente.
- **Qué NO resuelve:** no define gameplay de habilidades, facciones MOBA, ni tácticas de combate. No sustituye el pipeline de crecimiento existente (FL1–FL4) ni la materialización V7. No crea fauna autónoma (L15 Tactical Inference). No modifica el campo de energía ni la propagación.
- **Naturaleza:** 12 tipos puros + ~15 funciones puras en `equations.rs` + 3 primitivas geométricas nuevas + 2 sistemas ECS + extensión de 2 existentes.

## 2) Superficie pública (contrato)

### Tipos (LI1)

| Tipo | Ubicación | Naturaleza | Campos |
|------|-----------|------------|--------|
| `OrganRole` | `src/layers/organ.rs` | `#[repr(u8)]` enum, 12 variantes | Stem, Root, Core, Leaf, Petal, Sensory, Thorn, Shell, Fruit, Bud, Limb, Fin |
| `LifecycleStage` | `src/layers/organ.rs` | `#[repr(u8)]` enum, 6 variantes | Dormant, Emerging, Growing, Mature, Reproductive, Declining |
| `GeometryPrimitive` | `src/layers/organ.rs` | `#[repr(u8)]` enum, 4 variantes | Tube, FlatSurface, PetalFan, Bulb |
| `OrganSpec` | `src/layers/organ.rs` | struct, Copy | role, primitive, count (u8), scale_factor (f32) |
| `OrganManifest` | `src/layers/organ.rs` | struct, ArrayVec<OrganSpec, 8> | organs, stage |
| `LifecycleStageCache` | `src/layers/organ.rs` | Component, SparseSet | stage, ticks_in_stage (u16) |
| `AttachmentZone` | `src/worldgen/organ_inference.rs` | enum | Apical, Distributed, Basal, Full |
| `OrganAttachment` | `src/worldgen/organ_inference.rs` | struct, Copy | position, tangent, normal, spine_fraction |
| `OrganPrimitiveParams` | `src/geometry_flow/primitives.rs` | struct, Copy | origin, direction, tangent, base_radius, tint_rgb, qe_norm, detail |

### Funciones puras (LI2, LI3, LI6)

| Función | Ubicación | Firma | Propósito |
|---------|-----------|-------|-----------|
| `infer_lifecycle_stage` | `equations.rs` | (viability, growth_progress, biomass, can_reproduce) → LifecycleStage | Fase de ciclo desde estado |
| `lifecycle_stage_with_hysteresis` | `equations.rs` | (current, ticks, inferred, min_ticks) → (stage, ticks) | Histéresis anti-flickeo |
| `growth_progress` | `equations.rs` | (radius, base_radius, max_factor) → f32 | Progreso normalizado [0,1] |
| `infer_organ_manifest` | `equations.rs` | (stage, capabilities, biases...) → OrganManifest | Manifesto completo |
| `infer_leaf_count` | `equations.rs` | (biomass, growth_bias) → u8 | Conteo hojas |
| `infer_petal_count` | `equations.rs` | (biomass, branching_bias) → u8 | Conteo pétalos (Fibonacci) |
| `infer_thorn_count` | `equations.rs` | (biomass, resilience) → u8 | Conteo espinas |
| `infer_root_count` | `equations.rs` | (biomass, growth_bias) → u8 | Conteo raíces |
| `infer_limb_count` | `equations.rs` | (biomass, mobility_bias) → u8 | Conteo extremidades (pares) |
| `organ_role_modulated_rgb` | `equations.rs` | (field_rgb, OrganRole) → [f32;3] | Modulación visual 12 roles |
| `organ_role_scale` | `equations.rs` | (OrganRole, base_radius) → f32 | Escala por rol |
| `organ_attachment_points` | `organ_inference.rs` | (spine, count, zone) → ArrayVec<OrganAttachment> | Puntos de anclaje |
| `organ_orientation` | `organ_inference.rs` | (role, attachment, energy_dir) → (Vec3, Vec3) | Normal + tangent del órgano |

### Primitivas geométricas (LI4)

| Builder | Ubicación | Primitiva | Usado por |
|---------|-----------|-----------|-----------|
| `build_flat_surface` | `geometry_flow/primitives.rs` | Quad subdividido, doble cara | Leaf, Shell, Fin |
| `build_petal_fan` | `geometry_flow/primitives.rs` | N pétalos en espiral áurea con curvatura cóncava | Petal |
| `build_bulb` | `geometry_flow/primitives.rs` | UV sphere con elongación | Fruit, Bud, Sensory |
| `build_organ_primitive` | `geometry_flow/primitives.rs` | Dispatch por GeometryPrimitive | Todos |

### Sistemas ECS (LI7)

| Sistema | Ubicación | Phase | Reads | Writes |
|---------|-----------|-------|-------|--------|
| `lifecycle_stage_init_system` | `simulation/organ_lifecycle.rs` | MorphologicalLayer | GrowthBudget, CapabilitySet | Insert LifecycleStageCache |
| `lifecycle_stage_inference_system` | `simulation/organ_lifecycle.rs` | MorphologicalLayer | BaseEnergy, SpatialVolume, AllometricRadiusAnchor, GrowthBudget, CapabilitySet, InferenceProfile | Mut LifecycleStageCache |
| `shape_color_inference_system` (ext.) | `worldgen/shape_inference.rs` | MorphologicalLayer | + LifecycleStageCache | Mesh3d (organ pipeline si cache presente) |
| `growth_morphology_system` (ext.) | `worldgen/shape_inference.rs` | MorphologicalLayer | + LifecycleStageCache | Mesh3d rebuild (organ pipeline si cache presente) |

## 3) Invariantes y precondiciones

1. **Órganos nunca son componentes.** `OrganManifest` se computa, consume para generar mesh, y descarta. No persiste entre frames. No hay `Vec<Organ>` almacenado en ninguna entidad.
2. **Sin `match Species`.** Ninguna función pregunta qué tipo de criatura es. El manifesto emerge de `(stage × capabilities × biases × biomass)`. Las tablas de modulación usan `role as usize`, no nombres de especie.
3. **Backward compatible.** Sin `LifecycleStageCache` → pipeline actual exacto. Cero regresión visual para entidades existentes. La migración es opt-in vía inserción del componente.
4. **Determinismo.** Sin RNG en toda la cadena. Mismos inputs → misma malla, mismos colores. Filotaxia usa ángulo áureo constante (`2π(2-φ)`).
5. **Presupuesto compartido.** Cómputo de órganos cuenta contra `SHAPE_INF_MAX_PER_FRAME` existente. No hay presupuesto separado.
6. **Fibonacci para pétalos.** Conteo de pétalos se redondea al Fibonacci más cercano (3, 5, 8, 13). Produce patrones naturales.
7. **Simetría bilateral para extremidades.** Conteo de limbs forzado a pares.
8. **Histéresis para estabilidad.** `LifecycleStageCache` previene cambios de fase < N ticks. Excepción: Declining es inmediato (la muerte no espera).
9. **Modulación visual compatible.** `organ_role_modulated_rgb` para Stem/Leaf/Thorn produce EXACTAMENTE el mismo resultado que `branch_role_modulated_linear_rgb`. Los acentos y pesos son idénticos.

## 4) Comportamiento runtime

### Posición en el pipeline

```
Phase::MetabolicLayer
  ├─ growth_budget_system ✅
  └─ metabolic_stress_death_system ✅

Phase::MorphologicalLayer
  ├─ growth_intent_inference_system ✅
  ├─ allometric_growth_system ✅
  ├─ reproduction_spawn_system ✅
  ├─ abiogenesis_spawn_system ✅
  │
  ├─ lifecycle_stage_init_system        ← NEW (inserta cache en entidades nuevas)
  ├─ lifecycle_stage_inference_system   ← NEW (computa stage, cachea con histéresis)
  │
  ├─ shape_color_inference_system ✅    ← EXTENDED (branch: si cache → organ mesh, si no → existente)
  └─ growth_morphology_system ✅       ← EXTENDED (misma lógica)
```

### Chain ordering

```
growth_budget_system
  → lifecycle_stage_init_system
    → lifecycle_stage_inference_system
      → shape_color_inference_system (+ organ mesh branch)
```

La fase de ciclo debe estar resuelta ANTES de inferir shape, porque el manifesto depende del stage.

### Flujo de datos por entidad (un frame)

```
Entity: Rosa en Reproductive
  │
  ├─ MetabolicLayer: GrowthBudget actualizado (biomass=2.5, limiter=water)
  │
  ├─ MorphologicalLayer:
  │   ├─ lifecycle_stage_init_system: skip (ya tiene cache)
  │   ├─ lifecycle_stage_inference_system:
  │   │     viability = metabolic_viability(200, threshold) = 1.3
  │   │     progress = growth_progress(0.21, 0.08, 3.0) = 0.88
  │   │     inferred = Reproductive (viability > 1.2 AND progress > 0.7 AND REPRODUCE AND biomass > 1.5)
  │   │     hysteresis: ticks = 15 > min(10) → confirm stage
  │   │     cache.stage = Reproductive ✓
  │   │
  │   └─ shape_color_inference_system:
  │         manifest = infer_organ_manifest(Reproductive, caps, biases, biomass=2.5, progress=0.88)
  │           → [Stem ×1, Leaf ×3, Thorn ×1, Petal ×5, Root ×2]
  │         build_organ_mesh():
  │           1. Trunk spine (tube, existing)
  │           2. Branched tree with OrganRole assignments
  │           3. 5 petals at apex (PetalFan, golden angle, opening 57°)
  │           4. 3 leaves distributed (FlatSurface, filotaxia)
  │           5. 1 thorn distributed (thin tube)
  │           6. 2 roots at base (inverted tubes)
  │           7. Flatten → single Mesh3d
  │           8. Bake to entity local space
  │         cost = base(~50) + petals(~60) + leaves(~36) + roots(~24) = ~170
  │         budget remaining: 384 - 170 = 214
```

### LOD interaction

- `detail` (de `QuantizedPrecision.rho`) controla subdivisiones de TODAS las primitivas.
- Lejos → menos subdivisiones en pétalos, menos anillos en bulbos, menos segmentos en hojas.
- Un pétalo a LOD mínimo = 4 triángulos. A LOD máximo = 32 triángulos.

## 5) Implementación y trade-offs

### Memoria

| Dato | Coste | Persistencia |
|------|-------|-------------|
| `LifecycleStageCache` | 3 bytes (u8 + u16), SparseSet | Por entidad con lifecycle |
| `OrganManifest` | ~72 bytes (8 × OrganSpec en stack) | Efímero, un frame |
| Mesh generada | Varía, ~2–8 KB por entidad con órganos | Persistente (en Assets<Mesh>) |

### Coste computacional

| Operación | Coste | Frecuencia |
|-----------|-------|-----------|
| `infer_lifecycle_stage` | O(1) — 6 comparaciones | Cada frame por entidad con cache |
| `infer_organ_manifest` | O(1) — ~12 branches + 5 conteos | Cada rebuild de mesh |
| `build_petal_fan` (5 pétalos, subdiv=4) | ~40 vértices, ~60 triángulos | Cada rebuild |
| `build_flat_surface` (hoja, subdiv=3) | ~12 vértices, ~24 triángulos | Por hoja, cada rebuild |
| `build_bulb` (fruto, 4 rings, 6 sectors) | ~35 vértices, ~48 triángulos | Cada rebuild |

### Trade-offs

- **Órganos efímeros vs cacheados.** Computar OrganManifest cada frame es barato (~ns). Cachear el manifesto añadiría un componente más sin beneficio real — el manifesto solo se usa si la malla se reconstruye, lo cual ya está throttled.
- **Primitivas simples vs subdivisión adaptativa.** Pétalos/hojas usan quads subdivididos, no NURBS. Suficiente para el estilo visual del juego. Subdivisión adaptativa añadiría complejidad sin beneficio visible a la distancia de cámara típica.
- **CapabilitySet u8 vs u16.** 8 bits = 8 capabilities. Suficiente para la feature actual (4 existentes + 4 nuevas). Si se necesitan más → migrar a u16, cambio localizado en un solo struct.

## 6) Fallas y observabilidad

| Falla | Causa | Mitigación |
|-------|-------|-----------|
| Flickeo de stage | Histéresis insuficiente | `LIFECYCLE_HYSTERESIS_TICKS` ajustable; test de estabilidad |
| Pétalos ausentes en Reproductive | `CapabilitySet` sin flag REPRODUCE | Test: Rosa preset incluye REPRODUCE; log warning si stage=Reproductive sin capability |
| Malla excesiva (muchos órganos) | Biomasa alta + todos los capabilities | `MAX_ORGANS_PER_ENTITY = 8`; presupuesto de frame compartido |
| Regresión visual (entidades sin cache) | Pipeline viejo no produce misma malla | Test de no-regresión: mismos inputs sin cache → misma malla que antes |
| Stage Declining sin efecto visual | Manifesto no reduce órganos | Test: Declining + low viability → menos órganos que Mature |
| OrganRole desalineado con BranchRole | Tablas de acento divergen | Test: `organ_role_modulated_rgb(Stem) == branch_role_modulated_linear_rgb(Stem)` |

### Observabilidad

- `LifecycleStageCache` es `Reflect` → inspeccionable en DebugPlugin.
- `ShapeInferenceFrameState.processed_this_frame` incluye coste de órganos → monitorizable.
- Log level `debug`: emitir stage transitions para tuning de umbrales.

## 7) Checklist de atomicidad

- **¿Responsabilidad principal única?** Sí — inferir forma orgánica desde estado energético. No toca gameplay, combate, ni economía.
- **¿Acopla dominios?** Bajo en tipos/puras (solo `equations.rs` + `layers/`). Medio en integración (toca `shape_inference.rs`, pero con branch condicional aislado).
- **¿Necesita split?** No por ahora. Los 7 sprints ya están separados en responsabilidades claras. Si las primitivas crecen mucho → split `geometry_flow/primitives.rs` en archivos por primitiva.

## 8) Referencias cruzadas

| Doc | Relación |
|-----|----------|
| [`blueprint_energy_field_inference.md`](blueprint_energy_field_inference.md) | EPI1–EPI3: muestreo campo → vértice. Los órganos reutilizan EPI3 para color por nodo. |
| [`blueprint_ecosystem_autopoiesis.md`](blueprint_ecosystem_autopoiesis.md) | EA4–EA7: muerte, abiogénesis, reproducción, competencia. El lifecycle stage consume viabilidad de EA4. |
| [`blueprint_emergent_flora.md`](blueprint_emergent_flora.md) | FL1–FL4: irradiancia, nutrientes, crecimiento. El lifecycle stage depende de growth_progress de FL4. |
| [`blueprint_geometry_flow.md`](blueprint_geometry_flow.md) | GF1: spine/tube/branching existente. Las primitivas nuevas (LI4) coexisten con tubes. |
| [`blueprint_layers.md`](blueprint_layers.md) | 14 capas ECS. `LifecycleStageCache` es auxiliar SparseSet, no capa numerada. |
| [`blueprint_simulation.md`](blueprint_simulation.md) | Pipeline de fases. Los sistemas nuevos van en `Phase::MorphologicalLayer`. |
| [`blueprint_blueprint_math.md`](blueprint_blueprint_math.md) | equations.rs: ~15 funciones puras nuevas para lifecycle + organ inference. |
| [`blueprint_quantized_color.md`](blueprint_quantized_color.md) | Paleta cuantizada. `organ_role_modulated_rgb` compone sobre el mismo pipeline de color. |
| [`../sprints/LIVING_ORGAN_INFERENCE/`](../sprints/LIVING_ORGAN_INFERENCE/) | 7 sprints de implementación (LI1–LI7). |
