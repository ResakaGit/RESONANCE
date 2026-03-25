# Blueprint V7: Materialización Visual por Composición Energética

> **Axioma V7**: Si todo es energía, todo lo que VES es una proyección de la energía.
> El color es frecuencia. La opacidad es densidad. La forma es estado de materia.
> La textura es coherencia. El mundo visual es el mundo energético visto a través de un lente stateless.

---

## 0. Motivación

El modelo actual (v6) define cómo la energía EXISTE, se FORMA, ACTÚA, y se TRANSFIERE.
Pero el aspecto visual de las entidades (árboles, ríos, volcanes, cristales) se define manualmente:
`spawn_biome(BiomeType::Volcano)` → hardcodeado.

**V7 invierte esto**: el mapa se GENERA SOLO a partir de la energía.

- Un diseñador de mapa solo coloca **núcleos de energía** (posición, frecuencia, intensidad).
- La energía se propaga por el espacio formando un **campo**.
- Donde la energía se acumula, la **materia se materializa** según reglas stateless.
- El **aspecto** de cada cosa emerge de su composición energética.

Resultado:
- Distintos mapas por partida → cambiás los núcleos.
- Distintos mapas entre parches/seasons → cambiás la frecuencia/intensidad de los núcleos.
- Zero trabajo artístico por bioma → las reglas generan el mundo.

---

## 1. La Cadena Completa

```
NÚCLEOS                    CAMPO                  MATERIALIZACIÓN           VISUAL
(semillas de energía)      (acumulación espacial)  (puente stateless)        (derivación pura)
     │                          │                        │                       │
     ▼                          ▼                        ▼                       ▼
┌──────────┐            ┌──────────────┐          ┌──────────────┐        ┌──────────────┐
│ Nucleus  │  emana →   │ EnergyField  │  lee →   │ Materialization│ crea → │ VisualDeriv  │
│ (Capa 6+)│  qe+freq   │ Grid         │  reglas  │ Rules         │ ents   │ System       │
│          │  por tick   │ (Resource)   │  puras   │ (stateless)   │        │ (stateless)  │
└──────────┘            └──────────────┘          └──────────────┘        └──────────────┘
                              │                                                  │
                        dissipation                                        color, mesh,
                        (Segunda Ley)                                      scale, emission
```

---

## 2. Núcleos de Energía — Extension de Capa 6

### Concepto

Un **núcleo** es una entidad de Capa 6 (Tipo B — ES presión ambiental) extendida con:
- La **frecuencia** que emana (qué elemento domina la zona).
- La **tasa de emisión** (cuánta energía inyecta al campo por segundo).
- El **radio de propagación** (hasta dónde llega la energía).
- El **modelo de decaimiento** (cómo decae con la distancia).

### Componente

```rust
/// Extensión de Capa 6: fuente de energía que alimenta el EnergyFieldGrid.
///
/// Un núcleo NO modifica entidades directamente (eso sigue siendo AmbientPressure).
/// Un núcleo INYECTA energía frecuencial en el campo espacial.
#[derive(Component, Reflect, Debug, Clone)]
#[reflect(Component)]
pub struct EnergyNucleus {
    /// Frecuencia dominante que emana (Hz). Define qué elemento domina la zona.
    pub frequency_hz: f32,

    /// Tasa de emisión: qe inyectada al campo por segundo.
    pub emission_rate: f32,

    /// Radio máximo de propagación (unidades de mundo).
    pub propagation_radius: f32,

    /// Modelo de decaimiento de la energía con la distancia.
    pub decay: PropagationDecay,
}

#[derive(Clone, Copy, Debug, Reflect)]
pub enum PropagationDecay {
    /// Intensidad = emission / distance². Realista.
    InverseSquare,
    /// Intensidad = emission / distance. Suave, mapas más uniformes.
    InverseLinear,
    /// Intensidad constante dentro del radio, 0 fuera. Zonas definidas.
    Flat,
    /// Intensidad = emission * e^(-k * distance). Transición gradual.
    Exponential { k: f32 },
}
```

### Relación con Capa 6 existente

Un núcleo **coexiste** con `AmbientPressure` en la misma entidad:
- `AmbientPressure` → afecta entidades que la pisan (gameplay directo).
- `EnergyNucleus` → alimenta el campo espacial (generación de mundo).

```
Entidad "Volcán":
  AmbientPressure { delta_qe: -5.0, viscosity: 2.0 }    ← gameplay
  EnergyNucleus { freq: 450, rate: 100, radius: 15, .. } ← worldgen
  SpatialVolume { radius: 8.0 }
  BaseEnergy { qe: 500.0 }
  OscillatorySignature { freq: 450, phase: 0 }
```

### Diseño de Mapa = Colocación de Núcleos

```rust
/// Configuración de mapa: solo núcleos.
/// El resto emerge.
pub struct MapConfig {
    pub nuclei: Vec<NucleusConfig>,
    pub world_size: Vec2,
    pub warmup_ticks: u32,    // ticks de propagación antes de materializar
}

pub struct NucleusConfig {
    pub position: Vec2,
    pub frequency_hz: f32,       // elemento dominante
    pub emission_rate: f32,      // intensidad
    pub propagation_radius: f32, // alcance
    pub decay: PropagationDecay,
    // Opcional: parámetros de gameplay de Capa 6
    pub ambient_pressure: Option<AmbientPressureConfig>,
}
```

**Cambiar el mapa entre seasons** = cambiar `MapConfig`:

| Season  | Cambio en núcleos                        | Resultado visual                    |
|---------|------------------------------------------|-------------------------------------|
| Verano  | +Ignis intensity, -Aqua radius           | Más lava, ríos se encogen           |
| Invierno| +Aqua intensity, +Terra bond             | Más agua/hielo, terreno más sólido  |
| Eclipse | +Umbra emission, -Lux emission           | Mundo oscuro, sombras dominan       |
| Bloom   | +Ventus + Terra, balance equilibrado     | Praderas amplias, viento constante  |

---

## 3. Campo de Energía — EnergyFieldGrid (Resource)

### Concepto

Un grid espacial que acumula la energía emanada por todos los núcleos. Cada celda almacena:
- Cuánta energía hay (qe acumulada).
- Qué frecuencia domina (mezcla de contribuciones).
- La temperatura equivalente (derivada de densidad).
- El estado de materia resultante.

### Estructura

```rust
/// Recurso global: grid de energía acumulada en el espacio.
///
/// Alimentado por EnergyNucleus cada tick.
/// Consumido por MaterializationRules para generar el mundo.
/// NO es un componente — es estado del MUNDO, no de una entidad.
#[derive(Resource)]
pub struct EnergyFieldGrid {
    pub cells: Vec<EnergyCell>,
    pub cell_size: f32,          // metros por celda (resolución)
    pub width: u32,              // celdas en X
    pub height: u32,             // celdas en Y
    pub origin: Vec2,            // esquina inferior-izquierda en mundo
}

/// Estado energético acumulado en una celda del grid.
#[derive(Clone, Debug, Default)]
pub struct EnergyCell {
    /// Energía total acumulada en esta celda.
    pub accumulated_qe: f32,

    /// Contribuciones frecuenciales (frecuencia, intensidad).
    /// La frecuencia dominante = la de mayor intensidad.
    pub frequency_contributions: Vec<FrequencyContribution>,

    /// Frecuencia dominante (cache, recalculada cada tick).
    pub dominant_frequency_hz: f32,

    /// Pureza de la frecuencia dominante [0,1].
    /// 1.0 = una sola fuente domina. 0.0 = mezcla caótica.
    pub purity: f32,

    /// Temperatura equivalente (derivada).
    pub temperature: f32,

    /// Estado de materia derivado.
    pub matter_state: MatterState,

    /// Entidad materializada en esta celda (si existe).
    pub materialized_entity: Option<Entity>,
}

#[derive(Clone, Debug)]
pub struct FrequencyContribution {
    pub source: Entity,          // el núcleo que contribuye
    pub frequency_hz: f32,
    pub intensity: f32,          // qe/s que llega a esta celda
}
```

### Pipeline de Actualización

```
                        ┌─────────────────┐
                        │  propagate_      │
  EnergyNucleus ──────▶│  nuclei_system   │──▶ EnergyFieldGrid (write)
  (Query: nuclei)       │  (FixedUpdate)   │
                        └─────────────────┘
                                 │
                        ┌────────▼────────┐
                        │  dissipate_     │
                        │  field_system   │──▶ EnergyFieldGrid (mutate: Segunda Ley)
                        │  (FixedUpdate)  │
                        └─────────────────┘
                                 │
                        ┌────────▼────────┐
                        │  derive_cell_   │
                        │  state_system   │──▶ EnergyFieldGrid (derive: temp, state, purity)
                        │  (FixedUpdate)  │
                        └─────────────────┘
```

### Ecuaciones del Campo

```rust
/// Intensidad de un núcleo en un punto del espacio.
/// Función pura — stateless.
pub fn nucleus_intensity_at(
    nucleus_pos: Vec2,
    cell_pos: Vec2,
    emission_rate: f32,
    propagation_radius: f32,
    decay: PropagationDecay,
) -> f32 {
    let distance = (cell_pos - nucleus_pos).length();
    if distance > propagation_radius {
        return 0.0;
    }

    match decay {
        PropagationDecay::InverseSquare => {
            emission_rate / distance.max(0.5).powi(2)
        }
        PropagationDecay::InverseLinear => {
            emission_rate / distance.max(0.5)
        }
        PropagationDecay::Flat => {
            emission_rate
        }
        PropagationDecay::Exponential { k } => {
            emission_rate * (-k * distance).exp()
        }
    }
}

/// Frecuencia dominante y pureza de una celda con múltiples contribuciones.
/// Función pura — stateless.
pub fn resolve_dominant_frequency(
    contributions: &[FrequencyContribution],
) -> (f32, f32) {
    if contributions.is_empty() {
        return (0.0, 0.0);
    }

    let total_intensity: f32 = contributions.iter().map(|c| c.intensity).sum();
    if total_intensity <= 0.0 {
        return (0.0, 0.0);
    }

    // Frecuencia dominante = promedio ponderado por intensidad.
    let weighted_freq: f32 = contributions
        .iter()
        .map(|c| c.frequency_hz * c.intensity)
        .sum::<f32>() / total_intensity;

    // Pureza = cuánto domina la contribución más fuerte.
    let max_intensity = contributions
        .iter()
        .map(|c| c.intensity)
        .fold(0.0f32, f32::max);

    let purity = max_intensity / total_intensity;

    (weighted_freq, purity)
}

/// Disipación del campo: cada celda pierde energía cada tick (Segunda Ley).
pub fn field_dissipation(accumulated_qe: f32, decay_rate: f32, dt: f32) -> f32 {
    (accumulated_qe - decay_rate * dt).max(0.0)
}
```

### Interferencia entre Núcleos

Cuando dos núcleos de frecuencias distintas se superponen en una celda,
su interferencia determina QUÉ emerge en esa zona:

| Interferencia | Resultado | Ejemplo |
|---------------|-----------|---------|
| Constructiva (freq similares) | Amplificación → mayor densidad → materia más sólida | Dos núcleos Terra = montaña |
| Destructiva (freq opuestas) | Cancelación → zona "vacía" o inestable | Ignis + Aqua = vapor/niebla |
| Ortogonal (freq distintas) | Coexistencia → mezcla de elementos | Terra + Aqua = pantano |

La interferencia ya existe en `equations::interference()` — la reutilizamos:

```rust
/// Interferencia entre dos contribuciones frecuenciales en una celda.
pub fn cell_interference(a: &FrequencyContribution, b: &FrequencyContribution, t: f32) -> f32 {
    equations::interference(a.frequency_hz, 0.0, b.frequency_hz, 0.0, t)
}
```

---

## 4. Reglas de Materialización — El Puente Stateless

### Filosofía

> Una capa stateless que funcione simplemente como un puente con reglas.
> Al final las capas son eso — el camino hasta la pureza de energía — la Capa 0.

La capa de materialización:
- **NO almacena estado propio.**
- **LEE** el `EnergyFieldGrid` + `AlchemicalAlmanac`.
- **PRODUCE** comandos de spawn/despawn + propiedades visuales.
- **ES** una función pura: `f(energy_state) → visual_world`.
- **PUEDE** re-ejecutarse desde cero y producir el mismo resultado.

### La Tabla de Materialización

Cada combinación de (elemento dominante, estado de materia, densidad) produce
un tipo de entidad/aspecto:

```rust
/// Resultado de materialización para una celda.
/// Función PURA — sin side effects, sin estado.
pub struct MaterializationResult {
    pub archetype: WorldArchetype,
    pub base_color: Color,
    pub scale: f32,
    pub emission: f32,
    pub opacity: f32,
}

/// Arquetipos de mundo procedural.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WorldArchetype {
    // Terra
    Mountain,       // Terra + Solid + alta densidad
    Rock,           // Terra + Solid + media densidad
    Dirt,           // Terra + Solid + baja densidad
    Mud,            // Terra + Liquid + baja densidad
    DustCloud,      // Terra + Gas

    // Aqua
    DeepWater,      // Aqua + Liquid + alta densidad
    ShallowWater,   // Aqua + Liquid + baja densidad
    Ice,            // Aqua + Solid
    Fog,            // Aqua + Gas
    Steam,          // Aqua + Plasma

    // Ignis
    LavaFlow,       // Ignis + Plasma + alta densidad
    Ember,          // Ignis + Plasma + baja densidad
    Ash,            // Ignis + Solid + baja densidad
    Smoke,          // Ignis + Gas
    MagmaRock,      // Ignis + Solid + alta densidad

    // Ventus
    WindCurrent,    // Ventus + Gas
    Vortex,         // Ventus + Gas + alta densidad
    CalmAir,        // Ventus + Gas + baja densidad

    // Lux
    Crystal,        // Lux + Solid
    LightPool,      // Lux + Liquid
    Radiance,       // Lux + Plasma
    Shimmer,        // Lux + Gas

    // Umbra
    Shadow,         // Umbra + Gas
    VoidPool,       // Umbra + Liquid
    Obsidian,       // Umbra + Solid + alta densidad
    DarkMist,       // Umbra + Gas + baja densidad

    // Compuestos (interferencia entre elementos)
    Swamp,          // Terra + Aqua (ortogonal)
    Oasis,          // Aqua + Lux (constructiva parcial)
    Tundra,         // Aqua + Terra (densidad baja, baja temperatura)
    VolcanicBeach,  // Ignis + Terra (alta temperatura, sólido)
    StormZone,      // Ventus + Aqua (interferencia destructiva)

    // Vacío
    Void,           // densidad < umbral mínimo
}
```

### La Función de Materialización

```rust
/// SSOT de materialización: energy_state → world_archetype.
/// STATELESS — función pura, sin dependencia de ECS.
pub fn materialize_cell(
    cell: &EnergyCell,
    almanac: &AlchemicalAlmanac,
) -> Option<MaterializationResult> {
    // Umbral mínimo: sin suficiente energía, no hay nada.
    if cell.accumulated_qe < MIN_MATERIALIZATION_QE {
        return None;
    }

    // Derivar propiedades físicas de la celda.
    let cell_volume = CELL_SIZE * CELL_SIZE * CELL_HEIGHT; // volumen de la celda
    let density = cell.accumulated_qe / cell_volume;
    let temperature = equations::equivalent_temperature(density);

    // Buscar elemento dominante en el almanac.
    let element = almanac.find_stable_band(cell.dominant_frequency_hz);

    // Estado de materia según temperatura y bond_energy del elemento.
    let bond_energy = element.map(|e| e.bond_energy).unwrap_or(1000.0);
    let state = equations::state_from_temperature(temperature, bond_energy);

    // Clasificar densidad relativa.
    let density_class = classify_density(density);

    // Buscar arquetipo en la tabla.
    let archetype = lookup_archetype(
        cell.dominant_frequency_hz,
        state,
        density_class,
        cell.purity,
        almanac,
    );

    // Derivar propiedades visuales.
    let base_color = derive_color(cell, element);
    let scale = derive_scale(density, state);
    let emission = derive_emission(temperature, state);
    let opacity = derive_opacity(density, state);

    Some(MaterializationResult {
        archetype,
        base_color,
        scale,
        emission,
        opacity,
    })
}

#[derive(Clone, Copy, Debug)]
pub enum DensityClass { Low, Medium, High }

fn classify_density(density: f32) -> DensityClass {
    if density < DENSITY_LOW_THRESHOLD {
        DensityClass::Low
    } else if density < DENSITY_HIGH_THRESHOLD {
        DensityClass::Medium
    } else {
        DensityClass::High
    }
}
```

### Reglas de Compuestos por Interferencia

Cuando una celda tiene contribuciones de **múltiples elementos** (pureza < 0.7),
la materialización usa interferencia para decidir el resultado:

```rust
/// Resolver materialización compuesta cuando hay mezcla de elementos.
/// STATELESS.
pub fn resolve_compound(
    contributions: &[FrequencyContribution],
    state: MatterState,
    density_class: DensityClass,
    t: f32,
) -> WorldArchetype {
    if contributions.len() < 2 {
        return WorldArchetype::Void;
    }

    // Ordenar por intensidad descendente.
    let mut sorted = contributions.to_vec();
    sorted.sort_by(|a, b| b.intensity.partial_cmp(&a.intensity).unwrap());

    let primary = &sorted[0];
    let secondary = &sorted[1];

    // Interferencia entre los dos dominantes.
    let interference = equations::interference(
        primary.frequency_hz, 0.0,
        secondary.frequency_hz, 0.0,
        t,
    );

    // Mapear por combinación de bandas.
    match (band_of(primary.frequency_hz), band_of(secondary.frequency_hz)) {
        (Band::Terra, Band::Aqua) | (Band::Aqua, Band::Terra) => {
            if interference > 0.3 { WorldArchetype::Swamp }
            else if state == MatterState::Solid { WorldArchetype::Tundra }
            else { WorldArchetype::Mud }
        }
        (Band::Ignis, Band::Terra) | (Band::Terra, Band::Ignis) => {
            if state == MatterState::Plasma { WorldArchetype::LavaFlow }
            else { WorldArchetype::VolcanicBeach }
        }
        (Band::Aqua, Band::Lux) | (Band::Lux, Band::Aqua) => {
            WorldArchetype::Oasis
        }
        (Band::Ventus, Band::Aqua) | (Band::Aqua, Band::Ventus) => {
            if interference < -0.3 { WorldArchetype::StormZone }
            else { WorldArchetype::Fog }
        }
        _ => {
            // Default: el dominante manda.
            WorldArchetype::Void
        }
    }
}
```

---

## 5. Derivación Visual — Colores y Formas desde Energía

### Principio

> El color de algo es su frecuencia. La forma de algo es su estado de materia.
> La intensidad de algo es su densidad. La pureza de algo es cuánto domina un elemento.

### Funciones Puras de Derivación

```rust
/// Color base: interpolación entre el color del elemento y gris neutro,
/// ponderado por la pureza de la frecuencia.
///
/// Pureza 1.0 → color puro del elemento.
/// Pureza 0.0 → gris neutro (mezcla caótica).
/// STATELESS.
pub fn derive_color(cell: &EnergyCell, element: Option<&ElementDef>) -> Color {
    let base = element
        .map(|e| Color::srgb(e.color.0, e.color.1, e.color.2))
        .unwrap_or(Color::srgb(0.5, 0.5, 0.5));

    let neutral = Color::srgb(0.4, 0.4, 0.4);

    // Lerp entre neutral y color del elemento según pureza.
    color_lerp(neutral, base, cell.purity)
}

/// Escala visual: función de la densidad y estado de materia.
/// Sólidos crecen con densidad. Gases se expanden. Líquidos son uniformes.
/// STATELESS.
pub fn derive_scale(density: f32, state: MatterState) -> f32 {
    match state {
        MatterState::Solid => (density / REFERENCE_DENSITY).sqrt().clamp(0.3, 3.0),
        MatterState::Liquid => 1.0, // uniformes
        MatterState::Gas => (REFERENCE_DENSITY / density.max(0.01)).sqrt().clamp(0.5, 5.0),
        MatterState::Plasma => (density / REFERENCE_DENSITY).clamp(0.8, 2.0),
    }
}

/// Emisión lumínica: solo Plasma y Gas caliente emiten.
/// STATELESS.
pub fn derive_emission(temperature: f32, state: MatterState) -> f32 {
    match state {
        MatterState::Plasma => (temperature * 0.01).clamp(0.0, 1.0),
        MatterState::Gas => (temperature * 0.002).clamp(0.0, 0.3),
        _ => 0.0,
    }
}

/// Opacidad: sólidos son opacos, gases son transparentes.
/// STATELESS.
pub fn derive_opacity(density: f32, state: MatterState) -> f32 {
    match state {
        MatterState::Solid => 1.0,
        MatterState::Liquid => 0.85,
        MatterState::Gas => (density / REFERENCE_DENSITY).clamp(0.1, 0.6),
        MatterState::Plasma => 0.9,
    }
}
```

### Tabla Visual Ejemplo

| Elemento | Estado  | Densidad | Color              | Escala  | Emisión | Resultado Visual     |
|----------|---------|----------|--------------------|---------|---------|----------------------|
| Terra    | Solid   | Alta     | Marrón oscuro      | Grande  | 0       | Montaña              |
| Terra    | Solid   | Media    | Marrón             | Normal  | 0       | Terreno/piedra       |
| Terra    | Solid   | Baja     | Marrón claro       | Pequeña | 0       | Tierra suelta        |
| Aqua     | Liquid  | Alta     | Azul profundo      | Normal  | 0       | Agua profunda        |
| Aqua     | Solid   | Alta     | Azul pálido/blanco | Normal  | 0       | Hielo                |
| Aqua     | Gas     | Baja     | Blanco translúcido | Expand  | 0       | Niebla               |
| Ignis    | Plasma  | Alta     | Naranja brillante  | Normal  | Alta    | Lava                 |
| Ignis    | Gas     | Baja     | Gris rojizo        | Expand  | Baja    | Humo                 |
| Ignis    | Solid   | Alta     | Negro rojizo       | Normal  | 0       | Roca volcánica       |
| Lux      | Solid   | Alta     | Blanco dorado      | Normal  | Media   | Cristal              |
| Lux      | Plasma  | Alta     | Blanco puro        | Normal  | Máxima  | Radiancia            |
| Umbra    | Solid   | Alta     | Violeta oscuro     | Normal  | 0       | Obsidiana            |
| Umbra    | Gas     | Baja     | Negro translúcido  | Expand  | 0       | Sombra               |
| Ventus   | Gas     | Media    | Cian tenue         | Expand  | 0       | Corriente de viento  |

---

## 6. Pipeline Completo de Generación

### Fase 1: Startup (Warmup)

```
1. Cargar MapConfig (nuclei positions, frequencies, intensities)
2. Spawnear entidades-núcleo (EnergyNucleus + AmbientPressure + SpatialVolume)
3. Inicializar EnergyFieldGrid vacío
4. Ejecutar N ticks de propagación (warmup)
   └─ Cada tick:
      a) propagate_nuclei_system  → inyectar energía al grid
      b) dissipate_field_system   → disipar energía (Segunda Ley)
      c) derive_cell_state_system → calcular temp/state/purity por celda
5. Ejecutar materialization_system → spawnear entidades donde hay materia
6. Ejecutar visual_derivation_system → asignar color/scale/emission
```

### Fase 2: Runtime (Tick a Tick)

```
Phase::ThermodynamicLayer (cadena worldgen, prephysics.rs):
  1. propagate_nuclei_system      ← alimentar el campo
  2. dissipate_field_system       ← Segunda Ley
  3. derive_cell_state_system     ← derivar estado por celda
  4. … eco_boundaries, materialization_delta_system, flush visual, etc.

Update (register_visual_derivation_pipeline):
  visual_derivation_* + shape/morphology ← color/scale de entidades (no FixedUpdate)
```

### Fase 3: Season/Patch Change

```
1. Recibir nuevo MapConfig (o delta de cambios a núcleos)
2. Modificar EnergyNucleus de entidades existentes (freq, rate, radius)
3. Los sistemas de campo NATURALMENTE adaptan el grid
4. Las reglas de materialización NATURALMENTE adaptan el mundo
5. En N ticks, el mapa ha "transicionado" orgánicamente
```

Esto significa que un cambio de season no es un corte abrupto —
es una EVOLUCIÓN del mapa. Los jugadores ven cómo la lava retrocede,
el agua avanza, los cristales aparecen, todo en tiempo real.

---

## 7. Posición en el Árbol de Capas

La V7 introduce conceptos que se integran así en el árbol existente:

```
                              Capa 0
                             ENERGÍA
                          "¿Cuánta hay?"
                               │
              ┌────────────────┼────────────────┐
              │                │                │
           Capa 1           Capa 2           Capa 5
          ESPACIO            ONDA            MOTOR
              │                │                │
         ┌────┴────┐           │           ┌────┴────┐
         │         │           │           │         │
      Capa 3    Capa 4         │        Capa 7    Capa 8
      FLUJO    MATERIA         │       VOLUNTAD  INYECTOR
                               │
                            Capa 9
                          IDENTIDAD

  FUENTES EXTERNAS:
              Capa 6 — PRESIÓN AMBIENTAL
              ┌────────────────────────────┐
              │  + EnergyNucleus (V7)      │ ← extensión, no capa nueva
              └────────────────────────────┘

  ENTIDADES-EFECTO:
              Capa 10 — ENLACE DE RESONANCIA

  CAMPOS DE FUERZA:
              Capa 11 — CAMPO DE TENSIÓN

  ADAPTACIÓN:
              Capa 12 — HOMEOSTASIS

  VÍNCULOS:
              Capa 13 — ENLACE ESTRUCTURAL

  ┌──────────────────────────────────────────┐
  │         V7 — MATERIALIZACIÓN             │
  │     (NO es capa — es PUENTE STATELESS)   │
  │                                          │
  │  EnergyFieldGrid ← Resource (estado del  │
  │                    mundo, no de entidad)  │
  │                                          │
  │  materialize_cell() ← función pura       │
  │  derive_color()     ← función pura       │
  │  derive_scale()     ← función pura       │
  │                                          │
  │  Lee: Capas 0,1,2,4 + Almanac           │
  │  Escribe: spawn/despawn + render props   │
  │  Almacena: NADA (stateless)              │
  └──────────────────────────────────────────┘
```

### ¿Por qué NO es una capa?

Aplicamos los 5 tests de DESIGNING.md:

| Test | Resultado | Razón |
|------|-----------|-------|
| 1. Pregunta de energía | ❌ No responde una pregunta nueva | "¿Cómo se ve?" no es una pregunta sobre la energía — es una PROYECCIÓN de la energía |
| 2. Dependencia | N/A | No depende de capas específicas — lee TODAS las capas |
| 3. Tipo A o B | Ninguno | No es propiedad de entidad (A) ni entidad en sí (B) |
| 4. Entropía | N/A | No introduce ni consume energía |
| 5. Interferencia | N/A | No puede ser afectada por interferencia |

**La materialización es un OBSERVADOR** del sistema energético.
No participa en él. No lo modifica. Solo lo PROYECTA en forma visual.

Es el lente por el cual el jugador VE la energía.

---

## 8. Componentes Render-Only (V7)

Las entidades materializadas por V7 llevan componentes de solo lectura
para el sistema de rendering:

```rust
/// Marker: esta entidad fue creada por el sistema de materialización.
/// Se despawnea si la celda del campo pierde energía suficiente.
#[derive(Component, Reflect, Debug, Clone)]
#[reflect(Component)]
pub struct Materialized {
    /// Coordenadas de la celda en el grid que la originó.
    pub cell_x: u32,
    pub cell_y: u32,
    /// Arquetipo derivado.
    pub archetype: WorldArchetype,
}

/// Propiedades visuales derivadas de la composición energética.
/// Recalculadas cada tick por visual_derivation_system.
/// Los sistemas de rendering LEEN este componente — nunca lo escriben.
#[derive(Component, Reflect, Debug, Clone)]
#[reflect(Component)]
pub struct EnergyVisual {
    /// Color derivado de frecuencia × pureza.
    pub color: Color,
    /// Escala derivada de densidad × estado.
    pub scale: f32,
    /// Emisión luminosa derivada de temperatura × estado.
    pub emission: f32,
    /// Opacidad derivada de densidad × estado.
    pub opacity: f32,
}
```

Estos componentes son **de solo lectura para rendering**.
El sistema de materialización los ESCRIBE, nunca los lee.
No participan en la simulación.

---

## 9. Mapas Dinámicos en Runtime

### Escenario: El jugador destruye un núcleo

```
1. Héroe ataca entidad-núcleo → núcleo pierde qe
2. Cuando qe < QE_MIN → núcleo muere (DeathEvent)
3. propagate_nuclei_system deja de recibir contribuciones de ese núcleo
4. dissipate_field_system drena la energía restante en las celdas
5. materialization_delta_system detecta celdas bajo umbral → despawnea entidades
6. El terreno alrededor del núcleo destruido DESAPARECE orgánicamente
```

### Escenario: Nuevo núcleo aparece (invocación, item, evento)

```
1. Se spawnea entidad con EnergyNucleus (freq=250 Aqua, rate=50)
2. propagate_nuclei_system empieza a inyectar Aqua en celdas cercanas
3. En ~5 ticks: celdas cercanas superan umbral de materialización
4. materialization_delta_system spawnea entidades-agua alrededor
5. Visual: un lago "crece" orgánicamente desde el punto de invocación
```

### Escenario: Dos núcleos colisionan (overlap de campos)

```
Núcleo A: Ignis (450 Hz, rate=80)
Núcleo B: Aqua (250 Hz, rate=60)

Zona de overlap:
  - Interferencia destructiva → pureza baja → mezcla caótica
  - Resultado visual: niebla/vapor (Steam)
  - Si Ignis domina por intensidad → smoke/ceniza
  - Si Aqua domina → agua caliente / géiser

La "frontera" entre biomas no se diseña — EMERGE de la interferencia.
```

---

## 10. Organización de Código

### Nuevos archivos

```
src/
  worldgen/                          ← NUEVO módulo
    mod.rs                           ← re-exports
    nucleus.rs                       ← EnergyNucleus component
    field_grid.rs                    ← EnergyFieldGrid resource + EnergyCell
    propagation.rs                   ← ecuaciones puras de propagación
    materialization_rules.rs         ← funciones puras stateless
    visual_derivation.rs             ← funciones puras de color/scale/emission
    archetypes.rs                    ← WorldArchetype enum + lookup tables
    map_config.rs                    ← MapConfig, NucleusConfig (data-driven)

  simulation/
    worldgen_propagation.rs          ← sistemas ECS (propagate, dissipate, derive)
    worldgen_materialization.rs      ← sistemas ECS (materialize, despawn, update visual)
```

### Principio de separación

| Módulo | Contiene | Dependencia de ECS |
|--------|----------|-------------------|
| `worldgen/propagation.rs` | Funciones puras de campo | NINGUNA |
| `worldgen/materialization_rules.rs` | Funciones puras de materialización | NINGUNA |
| `worldgen/visual_derivation.rs` | Funciones puras de color/forma | NINGUNA |
| `worldgen/systems/*.rs` | Sistemas que llaman a las funciones puras | Bevy ECS |

Las funciones puras son **100% testeables** sin Bevy.
Los sistemas son **wiring** que conecta ECS con funciones puras.

---

## 11. Constantes de Tuning

```rust
// ── V7: Materialización ──

/// Energía mínima en una celda para materializar algo.
pub const MIN_MATERIALIZATION_QE: f32 = 10.0;

/// Resolución del grid (metros por celda).
pub const FIELD_CELL_SIZE: f32 = 2.0;

/// Altura implícita de una celda (para cálculo de volumen/densidad).
pub const FIELD_CELL_HEIGHT: f32 = 2.0;

/// Tasa de disipación del campo por segundo.
pub const FIELD_DECAY_RATE: f32 = 1.0;

/// Densidad de referencia para escala visual.
pub const REFERENCE_DENSITY: f32 = 50.0;

/// Umbrales de clasificación de densidad.
pub const DENSITY_LOW_THRESHOLD: f32 = 20.0;
pub const DENSITY_HIGH_THRESHOLD: f32 = 100.0;

/// Pureza mínima para considerar un elemento "puro" (vs compuesto).
pub const PURITY_THRESHOLD: f32 = 0.7;

/// Ticks de warmup para generación inicial del mapa.
pub const WARMUP_TICKS: u32 = 60;
```

---

## 12. Conexión con el Almanac

El `AlchemicalAlmanac` ya tiene todo lo que V7 necesita:

| Campo del Almanac | Uso en V7 |
|-------------------|-----------|
| `frequency_hz` | Centro de la banda del elemento |
| `freq_band` | Rango de estabilidad para clasificar la celda |
| `color` | Color base del elemento para derivación visual |
| `visibility` | Factor de opacidad/emisión |
| `bond_energy` | Determina transiciones de fase |
| `conductivity` | Afecta propagación entre celdas adyacentes |
| `matter_state` | Estado natural del elemento |

**No se necesitan cambios en el Almanac.** V7 lo consume tal cual existe.

---

## 13. Resumen Ejecutivo

```
V7 NO agrega capas nuevas al modelo.
V7 agrega UN componente (EnergyNucleus) como extensión de Capa 6.
V7 agrega UN resource (EnergyFieldGrid) como estado del mundo.
V7 agrega FUNCIONES PURAS stateless que traducen energía → mundo visual.
V7 agrega SISTEMAS que conectan ECS con funciones puras.

El resultado:
- El mapa se genera solo a partir de núcleos.
- Los colores/formas emergen de la composición energética.
- Distintos mapas = distintos núcleos.
- Seasons/patches = modificar parámetros de núcleos.
- Zero arte manual por bioma.
- Todo sigue la Segunda Ley: la energía decae, el mundo evoluciona.

El puente stateless es la PUREZA del modelo:
  lee energía → produce mundo → no almacena nada.
  Es el camino de vuelta a la Capa 0.
```

---

## 14. Roadmap de Implementación Sugerido

| Sprint | Entregable | Validación |
|--------|-----------|------------|
| S1 | `EnergyNucleus` component + `EnergyFieldGrid` resource + `propagate_nuclei_system` | Test: núcleo inyecta energía en celdas cercanas |
| S2 | `dissipate_field_system` + `derive_cell_state_system` | Test: campo alcanza equilibrio, Segunda Ley funciona |
| S3 | `materialization_rules.rs` (funciones puras) + `WorldArchetype` | Test unitario: (freq, density, state) → archetype correcto |
| S4 | `materialization_system` (spawn entidades desde grid) | Test: warmup + materialización produce mundo coherente |
| S5 | `visual_derivation.rs` (funciones puras de color) | Test unitario: derive_color/scale/emission correctos |
| S6 | `visual_derivation_system` + integración con render bridge | Demo visual: mundo coloreado por energía |
| S7 | Interferencia entre núcleos + compuestos | Test: overlap de campos produce biomas mixtos |
| S8 | `MapConfig` data-driven + season transitions | Demo: cambiar config → mundo evoluciona en runtime |
| S9 | Destrucción/creación de núcleos en gameplay | Demo: destruir núcleo → terreno desaparece |
| S10 | Performance: LOD del grid + cache de materialización | Benchmark: grid 200×200 a 60fps |
