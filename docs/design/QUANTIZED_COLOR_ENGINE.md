# BLUEPRINT — Motor Termodinámico de Color Cuantizado

> **Axioma**: El color no es una propiedad — es una **frecuencia** proyectada. La GPU no necesita
> saber qué es una flor, un río o una roca. Solo recibe energía y un factor de precisión.
> El color es el último eslabón del camino de vuelta a la Capa 0.

> **Relación con V7**: Este blueprint **extiende** la derivación visual de V7 (Sprint 05/08) sin
> reemplazarla. V7 define **qué** propiedades visuales tiene una entidad (`EnergyVisual`).
> Este blueprint define **cómo** la GPU las renderiza **eficientemente** según la distancia.

---

## 0. Motivación

El sistema actual (Sprint 05 + Sprint 08) computa color, escala, emisión y opacidad en CPU (ECS `Update`), y lo asigna al sprite/material. Esto funciona, pero tiene dos limitaciones:

1. **No hay LOD de color.** Cada entidad recibe la máxima fidelidad cromática independientemente de si está a 2 metros o a 200 metros de la cámara.
2. **No hay coherencia de caché GPU.** Miles de polígonos vecinos con energías minimamente distintas producen miles de colores distintos → miles de cache misses en la GPU.

El Motor de Color Cuantizado resuelve ambos problemas con una sola idea:
> **Cuantizar la energía normalizada según la distancia a la cámara, mapeándola a un bloque finito de colores pre-computados en VRAM.**

---

## 1. Separación Estricta de Responsabilidades (DoD)

```
┌────────────────────────────────────────────────────────────────────┐
│                    CAPA SUPERIOR (CPU / Bevy ECS)                  │
│                                                                    │
│  1. Frustum Culling:  ¿el objeto está en el cono de visión?       │
│     → NO visible → no enviar a GPU (zero cost)                    │
│     → SÍ visible → continuar                                      │
│                                                                    │
│  2. factor_precision (ρ):  f(distancia_cámara)                    │
│     distancia → ρ ∈ (0, 1]                                        │
│     → cerca (d < Near): ρ = 1.0  (máxima fidelidad)               │
│     → lejos (d > Far):  ρ = ρ_min (mínima fidelidad, > 0)        │
│     → intermedio: interpolación lineal o smoothstep               │
│                                                                    │
│  3. energia_interna (Enorm):  BaseEnergy normalizada [0, 1]       │
│     Enorm = clamp(qe / QE_REFERENCE, 0.0, 1.0)                   │
│                                                                    │
│  4. n_max_id:  índice a la paleta pre-computada en VRAM           │
│     → derivado de WorldArchetype o ElementBand del Almanac        │
│                                                                    │
│  5. Empaqueta VisualPayload → inyecta al pipeline de render       │
└────────────────────────────────────────────────────────────────────┘
                              │
                        GPU Pipeline
                              │
                              ▼
┌────────────────────────────────────────────────────────────────────┐
│              MÓDULO STATELESS (Fragment Shader / GPU)              │
│                                                                    │
│  Recibe por vértice/instancia:                                     │
│    energia_interna: f32                                            │
│    factor_precision: f32                                           │
│    n_max_id: u32                                                   │
│                                                                    │
│  Computa O(1), sin condicionales (branchless):                    │
│    S = max(1, ceil(Nmax * ρ))                                     │
│    Eq = floor(Enorm * S) / S                                      │
│    Índice = floor(Eq * (Nmax - 1))                                │
│    Color = palette[n_max_id][Índice]                               │
│                                                                    │
│  NO sabe qué es una cámara, una flor, un bioma.                   │
│  Solo transforma números en un puntero a una paleta.              │
└────────────────────────────────────────────────────────────────────┘
```

### Regla DoD: Sin estado bidireccional

La GPU **nunca** escribe de vuelta al ECS. El flujo es estrictamente **unidireccional**:
- ECS (CPU) → calcula `VisualPayload`
- GPU → lee payload → lee paleta → emite color
- Fin. Sin feedback loops.

---

## 2. La Física de Estados Finitos (Nmax)

### Concepto

La onda energética de un material no es infinita. La estructura molecular de un "tipo de materia" en Resonance (definida por su `WorldArchetype` o `ElementBand`) solo permite un número **finito** de variaciones cromáticas perceptibles antes de que la diferencia sea indistinguible.

**Nmax** = el límite físico de tonos cacheados para ese material.

### Determinación de Nmax

Nmax se determina por la **paleta del Almanac**:

| Fuente | Nmax típico | Justificación |
|--------|-------------|---------------|
| Elemento puro (ej. Ignis) | 64-128 | Gradiente de un solo hue con variación de saturación/luminosidad |
| Compuesto (ej. Swamp) | 32-64 | Mezcla de dos elementos → menos variación perceptible |
| Terreno neutro | 16-32 | Tierra/roca → pocos tonos posibles |
| Plasma/Emisivo | 128-256 | Amplio rango de luminosidad, alta variación visual |

Estos valores son **constantes de tuning por paleta**, definidas en los `.ron` del Almanac o en una tabla auxiliar.

### Bloque de Memoria (VRAM)

Cada paleta es un arreglo contiguo de `Nmax` colores (4 bytes RGBA cada uno):

```
Paleta "Ignis" (Nmax = 128):
  [0]: RGBA(64, 0, 0, 255)      ← Enorm ≈ 0.0 (casi sin energía)
  [1]: RGBA(80, 10, 0, 255)
  ...
  [63]: RGBA(255, 128, 0, 255)   ← Enorm ≈ 0.5 (media energía)
  ...
  [127]: RGBA(255, 255, 200, 255)← Enorm ≈ 1.0 (máxima energía)

Footprint: 128 × 4 bytes = 512 bytes en VRAM.
```

Total para 20 paletas: ~10 KB en VRAM. **Despreciable.**

---

## 3. El Algoritmo de Cuantización (Módulo Stateless GPU)

### Matemática Formal

Sea:
- `ρ` = factor_precision inyectado por la cámara, `ρ ∈ (0, 1]`
- `Enorm` = energía normalizada, `Enorm ∈ [0, 1]`
- `Nmax` = colores máximos de la paleta del material

**Paso A — Subdivisiones activas:**

```
S = max(1, ⌈Nmax · ρ⌉)
```

Si `Nmax = 100` y `ρ = 0.1`: el motor trabaja con `S = 10` bloques.
Si `ρ = 1.0`: `S = 100` (máxima fidelidad).

**Paso B — Cuantizar la energía:**

```
Eq = ⌊Enorm · S⌋ / S
```

Si `Enorm = 0.52` y `S = 10`: la matemática lo trunca a `0.50`.
Hemos eliminado la micro-variación imperceptible a esa distancia.

**Paso C — Índice de paleta:**

```
Índice = ⌊Eq · (Nmax - 1)⌋
```

El shader lee directamente de `palette[Índice]`.

### Implementación WGSL (Bevy)

```wgsl
// ── Fragment Shader: quantized_color.wgsl ──

struct VisualPayload {
    energia_interna: f32,   // Enorm ∈ [0, 1]
    factor_precision: f32,  // ρ ∈ (0, 1]
    n_max: u32,             // Nmax de esta paleta
    palette_offset: u32,    // offset en el buffer global de paletas
};

@group(1) @binding(0) var<storage, read> palettes: array<vec4<f32>>;
@group(1) @binding(1) var<uniform> payload: VisualPayload;

fn quantized_color(payload: VisualPayload) -> vec4<f32> {
    let n_max_f = f32(payload.n_max);

    // Paso A: subdivisiones activas (branchless)
    let s = max(1.0, ceil(n_max_f * payload.factor_precision));

    // Paso B: cuantizar energía
    let eq = floor(payload.energia_interna * s) / s;

    // Paso C: índice de paleta
    let idx = u32(floor(eq * (n_max_f - 1.0)));

    // Lookup directo en VRAM
    return palettes[payload.palette_offset + idx];
}
```

### Propiedades del Algoritmo

| Propiedad | Valor |
|-----------|-------|
| Complejidad | O(1) — sin loops, sin branches |
| Memoria por paleta | Nmax × 16 bytes (vec4<f32>) |
| Latencia | 1 lookup de memoria |
| Determinismo | Total — misma entrada = mismo color |

---

## 4. Ventajas de Rendimiento y Manejo de Memoria

### 4.1 Coherencia Espacial de Caché GPU

Cuando `ρ = 0.1`, obligamos a que miles de polígonos vecinos (que antes tenían variaciones minúsculas de energía) calculen **exactamente el mismo `Eq`**. Todos piden el mismo `Índice` de memoria:

```
Sin cuantización (ρ=1.0):
  Polígono A: Enorm=0.501 → Color #FF8001
  Polígono B: Enorm=0.502 → Color #FF8002
  Polígono C: Enorm=0.503 → Color #FF8003
  → 3 cache-lines distintas → 3 fetches

Con cuantización (ρ=0.1, S=10):
  Polígono A: Enorm=0.501 → Eq=0.50 → Índice=49 → Color palette[49]
  Polígono B: Enorm=0.502 → Eq=0.50 → Índice=49 → Color palette[49]
  Polígono C: Enorm=0.503 → Eq=0.50 → Índice=49 → Color palette[49]
  → 1 cache-line → 1 fetch → hit de L1 para los siguientes
```

### 4.2 Zero Multiplicación de Texturas

No necesitás crear diferentes texturas para diferentes distancias. Usás el **mismo bloque de Nmax colores en VRAM** (unos pocos KB), y es la **matemática stateless** la que "agrupa" los punteros dinámicamente.

### 4.3 Transición Continua

Al acercarte, `ρ` sube a 1.0. La matemática permite automáticamente que `S = Nmax`. La energía de la entidad recupera toda su fidelidad visual, revelando los Nmax tonos posibles de manera natural y determinista.

No hay "pop". No hay corte abrupto. Es una función continua de la distancia.

### 4.4 Presupuesto de VRAM

| Escenario | Paletas | Nmax avg | VRAM |
|-----------|---------|----------|------|
| MVP (6 elementos puros) | 6 | 64 | 6 × 64 × 16B = 6 KB |
| Full (6 puros + 10 compuestos + terreno) | 17 | 96 | 17 × 96 × 16B ≈ 26 KB |
| Máximo teórico | 30 | 256 | 30 × 256 × 16B ≈ 120 KB |

**Despreciable** incluso para hardware modesto.

---

## 5. El Contrato de Interfaz (VisualPayload)

### Payload CPU → GPU (por instancia visual)

```rust
/// El payload que el ECS le inyecta a la GPU para procesar el lote visual.
///
/// Empaquetado como uniform buffer o instance attribute según la estrategia
/// de batching que use el render bridge.
///
/// INVARIANTE: energia_interna ∈ [0, 1], factor_precision ∈ (0, 1].
/// INVARIANTE: n_max_id apunta a una paleta válida cargada en VRAM.
#[repr(C)]
#[derive(Clone, Copy, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct VisualPayload {
    /// Estado termodinámico normalizado.
    /// Derivado de BaseEnergy.qe / QE_REFERENCE.
    pub energia_interna: f32,

    /// Factor de precisión inyectado por la cámara.
    /// 0.0+ (lejos) → 1.0 (cerca).
    /// Calculado en CPU por el LOD system.
    pub factor_precision: f32,

    /// ID de la paleta pre-computada en VRAM.
    /// Derivado del WorldArchetype o ElementBand.
    pub n_max_id: u32,

    /// Padding para alineación de 16 bytes en GPU.
    pub _padding: u32,
}
```

### Paleta pre-computada (VRAM)

```rust
/// Paleta pre-computada por el sistema de startup.
///
/// Generada UNA VEZ al arrancar (o al cambiar de season/config).
/// Subida a un storage buffer en VRAM.
/// El fragment shader la indexa por (n_max_id, Índice).
pub struct PaletteBlock {
    /// ID único de la paleta (corresponde a un WorldArchetype o ElementBand).
    pub id: u32,

    /// Número máximo de colores (Nmax para este material).
    pub n_max: u32,

    /// Colores pre-computados usando derive_color con energías uniformemente
    /// espaciadas de 0.0 a 1.0.
    /// Generados por las funciones puras de V7 Sprint 05.
    pub colors: Vec<[f32; 4]>,
}
```

---

## 6. Generación de Paletas (Puente CPU → GPU)

### Principio

Las paletas NO se inventan — se **derivan** de las mismas funciones puras de V7:

```rust
/// Genera la paleta pre-computada para un elemento del Almanac.
/// Usa exactamente las mismas funciones de visual_derivation (Sprint 05).
///
/// STATELESS — función pura.
pub fn generate_palette(
    element: &ElementDef,
    n_max: u32,
    almanac: &AlchemicalAlmanac,
) -> PaletteBlock {
    let colors: Vec<[f32; 4]> = (0..n_max)
        .map(|i| {
            let enorm = i as f32 / (n_max - 1).max(1) as f32;

            // Simular la energía correspondiente a este nivel
            let density = enorm * REFERENCE_DENSITY * 2.0;
            let temperature = equations::equivalent_temperature(density);
            let state = equations::state_from_temperature(temperature, element.bond_energy);

            // Usar las funciones puras de Sprint 05
            let color = derive_color_for_element(element, enorm);
            let emission = derive_emission(temperature, state);
            let opacity = derive_opacity(density, state);

            // Mezclar emisión en el color
            let r = (color.r + emission * 0.5).min(1.0);
            let g = (color.g + emission * 0.3).min(1.0);
            let b = (color.b + emission * 0.2).min(1.0);

            [r, g, b, opacity]
        })
        .collect();

    PaletteBlock {
        id: element.id,
        n_max,
        colors,
    }
}
```

### Cuándo se regeneran

| Evento | Acción |
|--------|--------|
| Startup | Generar todas las paletas desde el Almanac |
| Season change | Regenerar paletas afectadas (si los colores del Almanac cambian) |
| Hot reload del Almanac | Regenerar la paleta del elemento modificado |

**Costo:** O(Nmax × num_paletas) ≈ microsegundos. Una sola vez.

---

## 7. Integración con Pipeline Existente

### Posición en el flujo

```
V7 Sprint 05: derive_color/scale/emission (funciones puras)
    │
    ├──▶ Sprint 08: visual_derivation_system (CPU, por entidad)     ← HOY, se mantiene
    │    └── Escribe EnergyVisual → render bridge
    │
    └──▶ ESTE BLUEPRINT: generate_palette() (CPU, startup)          ← NUEVO
         └── Sube PaletteBlock[] a VRAM
              │
              ├── factor_precision_system (CPU, Update)             ← NUEVO
              │   └── Calcula ρ por entidad desde distancia a cámara
              │
              └── quantized_color.wgsl (GPU, fragment)              ← NUEVO
                  └── Lee payload → indexa paleta → emite color
```

### Coexistencia con Sprint 08

El sistema actual de Sprint 08 (`visual_derivation_system` + `visual_sync_to_render_system`)
sigue siendo válido para:
- Entidades cercanas donde `ρ = 1.0` (máxima fidelidad, el shader simplemente indexa toda la paleta)
- Modo 2D (sprites) donde el fragment shader custom no aplica
- Debug/desarrollo donde queremos ver el color CPU-side

El motor cuantizado es una **optimización de rendering** que opera en paralelo.
El `EnergyVisual` sigue siendo la SSOT del color en CPU.
El shader cuantizado es una **proyección alternativa** para rendering 3D de alto rendimiento.

### Integración con Terrain Mesher

El `ProceduralTerrainMesher` (Blueprint existente) recibe `TerrainVisuals.vertex_colors`.
Con este blueprint, esos colores pueden calcularse usando la misma cuantización:

```rust
// En la frontera Terrain → Mesher:
// vertex_colors[i] = palette[palette_id][quantized_index(enorm, rho, nmax)]
// donde rho viene del LOD del chunk del terreno
```

---

## 7.5 Demarcación con Capas de Rendimiento Existentes (Sprint 13)

> **Regla:** Este blueprint NO duplica las optimizaciones del Sprint 13.
> Opera en una capa distinta. La tabla siguiente demarca responsabilidades.

### Mapa de capas existentes vs Color Cuantizado

| Capa existente (Sprint 13) | Qué controla | Fase | ¿Sprint 14 la toca? |
|----------------------------|-------------|------|---------------------|
| `MaterializationCellCache` | Cachea resultado de `materialize_cell()` (qué **arquetipo/forma** spawnear) por firma de celda | `FixedUpdate` | **NO** — cachea forma, no color |
| LOD Near/Mid/Far (`lod.rs`) | **Frecuencia de tick** de materialización (Near=cada tick, Mid=cada 4, Far=cada 16) | `FixedUpdate` | **REUTILIZA** — `factor_precision` lee las mismas bandas para calcular `ρ` |
| Cull distance (150u) | Corta materialización **completa** más allá de 150u | `FixedUpdate` | **NO** — entidad culled no llega al shader |
| `max_visual_derivation_per_frame` | **Presupuesto CPU** de entidades a recalcular `EnergyVisual` por frame | `Update` | **MANTIENE** como fallback para modo 2D/sprites |
| `Changed<BaseEnergy>` filter | Skip de `derive_color` si la energía **no cambió** | `Update` | **NO** — el shader GPU no usa change detection |

### Capa NUEVA del Color Cuantizado (Sprint 14)

| Concepto nuevo | Qué controla | No duplica porque... |
|----------------|-------------|---------------------|
| Paletas pre-computadas | Discretiza los Nmax colores posibles en startup | No existe hoy. `derive_color` se computa cada vez. Las paletas son lookup estático. |
| `factor_precision (ρ)` | Resolución cromática según distancia | LOD existente decide **si** materializar/derivar. `ρ` decide **con cuánta fidelidad** colorear. Ortogonales. |
| `quantized_index()` | Mapea Enorm a índice de paleta O(1) | No existe hoy. `derive_color` interpola el Almanac cada vez, `quantized_index` es un puntero. |
| Shader WGSL | Proyecta color en GPU | No existe hoy. Todo el color se computa en CPU. |

### Regla de integración

```
Sprint 14 REUTILIZA:
  - WorldgenPerfSettings (constantes Near/Mid/Far/cull_distance)
  - LodBand (para derivar ρ desde la banda ya clasificada)

Sprint 14 NO DUPLICA:
  - MaterializationCellCache (arquitectura, no color)
  - max_visual_derivation_per_frame (se mantiene para 2D)
  - Changed<T> filters (irrelevantes en GPU)

Sprint 14 COMPLEMENTA:
  - Con paletas CPU-side, quantized_index() actúa como early-out:
    si el índice no cambió desde el frame anterior, skip derive_color().
    Esto REDUCE la presión sobre max_visual_derivation_per_frame.
```

---

## 8. Posición Filosófica: ¿Por qué esto es coherente con "Todo es Energía"?

### El color como última derivación

```
Capa 0 (Energía)
  → Capa 2 (Frecuencia)
    → Almanac (Identidad elemental)
      → derive_color() (Sprint 05: función pura)
        → Paleta pre-computada (Startup: discretización finita)
          → Cuantización (GPU: proyección por distancia)
            → Pixel final (Observador)
```

Cada paso es una **proyección stateless** de la capa anterior.
El color nunca se "inventa" — emerge de la energía, pasa por la frecuencia,
se discretiza por los límites físicos del material (Nmax),
y se proyecta según la capacidad de observación (distancia).

### El observador determina la resolución

Esto es filosóficamente coherente con Resonance:
- **Cerca** → ves el detalle termodinámico completo (S = Nmax).
- **Lejos** → ves la esencia energética promediada (S = pocos bloques).
- **Fuera de vista** → no ves nada (Frustum Culling, cero costo).

La resolución visual es **literalmente una función matemática** de la distancia
y el límite atómico del objeto. No hay magia, no hay heurísticas.

### Nmax como propiedad física

Nmax NO es un parámetro artístico arbitrario. Es el **límite de estados estables**
del material a nivel molecular. Una roca tiene pocos estados cromáticos posibles.
Un plasma tiene muchos. Esto se deriva del `bond_energy` y `conductivity` del Almanac.

---

## 9. Resumen Ejecutivo

```
Este blueprint NO agrega capas nuevas al modelo.
Este blueprint NO modifica el ECS ni la simulación.
Este blueprint NO reemplaza la derivación visual de V7.

Agrega:
  1. Un módulo CPU stateless que genera paletas desde el Almanac (startup).
  2. Un componente por-instancia (VisualPayload) con 3 floats.
  3. Un fragment shader WGSL O(1) branchless que cuantiza e indexa.
  4. Un sistema ECS (factor_precision_system) que calcula ρ por distancia.

El resultado:
  - Coherencia masiva de caché GPU (miles de polígonos → mismo color).
  - Transición continua sin pop-in (ρ sube → más fidelidad).
  - Zero texturas adicionales (solo paletas de KB en VRAM).
  - La resolución visual es una función determinista de energía × distancia.

Es la proyección final del axioma:
  todo es energía → el color es su frecuencia observada.
```

---

## 10. Referencia cruzada

- `docs/design/V7.md` — Sección 5: Derivación visual
- `docs/arquitectura/blueprint_v7.md` — Sección 5: Funciones puras de derivación
- `src/worldgen/visual_derivation.rs`, `src/worldgen/systems/visual.rs`, `src/worldgen/systems/performance.rs` — derivación visual + LOD (sprints V7 05/08/13 cerrados; docs eliminados)
- `docs/sprints/BLUEPRINT_V7/README.md` — backlog V7 (06, 07, 14)
- `docs/design/TERRAIN_MESHER.md` — Integración con terreno
- `docs/arquitectura/blueprint_geometry_flow.md` — Geometry Flow (no afectado)
- `docs/arquitectura/blueprint_quantized_color.md` — Contrato de módulo (acompaña)
