# Reglas de Diseño por System

Cada system nuevo debe cumplir estas reglas. El Verificador (rol Observador) audita contra esta lista.

---

## R-1: Signatura Canónica

Todo system sigue este template:

```rust
/// [Qué transforma y por qué — una línea].
pub fn system_name(
    mut target_query: Query<(&mut Target, &Source), (Without<Dead>, With<RequiredMarker>)>,
    readonly_query: Query<&OtherSource>,
    config: Res<Config>,               // Nunca ResMut si solo lees
    time: Res<Time<Fixed>>,            // Si necesitas dt
    mut events: EventWriter<MyEvent>,  // Si emite eventos
) {
    let dt = time.delta_secs();
    for (mut target, source) in &mut target_query {
        let result = equations::my_calculation(source.field, config.param, dt);
        if target.field != result {
            target.field = result;
        }
    }
}
```

---

## R-2: Ecuación Primero, System Después

**Workflow obligatorio**:
1. Escribir la ecuación pura en `blueprint/equations/{dominio}/mod.rs`
2. Escribir tests unitarios de la ecuación (edge cases, invariants)
3. Extraer constantes a `blueprint/constants/{dominio}.rs`
4. LUEGO escribir el system que llama a la ecuación
5. Escribir test de integración del system (MinimalPlugins, 1 update)

Nunca al revés. La ecuación es la fuente de verdad.

---

## R-3: Presupuesto de Queries

| Complejidad del System | Max Query Types | Max Components en Query |
|------------------------|-----------------|------------------------|
| Simple (1 transformación) | 1-2 queries | 3 components |
| Medio (cross-entity) | 2-3 queries | 4 components |
| Complejo (spatial) | 3 queries + 1 Resource | 4 components |

Si excedes esto, descomponer en chain.

---

## R-4: Clasificación de Systems

Cada system se clasifica en una de estas categorías:

### A) Transformer (80% de systems)
- Lee componente(s), calcula, escribe componente(s)
- Sin side effects más allá de la mutación
- Determinista

### B) Emitter (10%)
- Lee estado, decide si emitir evento
- Nunca muta componentes directamente
- Produce `EventWriter<T>`

### C) Consumer (5%)
- Lee eventos, muta componentes en respuesta
- Consume `EventReader<T>`
- Siempre `.after(emitter)` explícito

### D) Initializer (5%)
- Inserta componentes en entidades que los necesitan
- Usa `Added<T>` o `Without<T>` filters
- Corre una vez por entidad, no por frame

---

## R-5: Phase Assignment Matrix

| Dominio | Phase | Justificación |
|---------|-------|---------------|
| Input/Will/Decision | `Phase::Input` | Antes de todo cálculo |
| Termodinámica/Contención/Osmosis | `Phase::ThermodynamicLayer` | Transferencia de energía |
| Movimiento/Colisión/Tensión | `Phase::AtomicLayer` | Fuerzas y posiciones |
| Reacciones/Catálisis/Homeostasis | `Phase::ChemicalLayer` | Transformaciones de estado |
| Metabolismo/Trófico/Competencia | `Phase::MetabolicLayer` | Balance energético |
| Morfología/Lifecycle/Reproducción | `Phase::MorphologicalLayer` | Forma y ciclo de vida |

**Regla**: Si un system lee output de otro, DEBE estar en Phase posterior o encadenado con `.after()`.

---

## R-6: Throttling Obligatorio para N² Operations

Si un system hace spatial queries o cross-entity comparisons:

```rust
#[derive(Resource, Default)]
pub struct MySystemCursor {
    next_index: usize,
}

const MAX_PER_FRAME: usize = 128;

pub fn my_spatial_system(
    query: Query<(Entity, &Transform, &SpatialVolume)>,
    spatial: Res<SpatialIndex>,
    mut cursor: ResMut<MySystemCursor>,
    mut scratch: Local<Vec<Entity>>,
) {
    let entities: Vec<_> = query.iter().collect();
    let start = cursor.next_index;
    let end = (start + MAX_PER_FRAME).min(entities.len());

    for i in start..end {
        let (entity, transform, volume) = entities[i];
        scratch.clear();
        spatial.query_radius(transform.translation.truncate(), volume.radius * 2.0, &mut scratch);
        // ... process neighbors
    }

    cursor.next_index = if end >= entities.len() { 0 } else { end };
}
```

---

## R-7: Change Detection Guard

Antes de mutar un componente, verificar que el valor cambió:

```rust
// OBLIGATORIO
let new_val = equations::compute(source);
if target.field != new_val {
    target.field = new_val;
}
```

**Por qué**: Bevy marca componentes como `Changed` en cualquier `&mut` access, aunque el valor no cambie. Esto trigger downstream systems innecesariamente.

---

## R-8: Naming Convention

```
{dominio}_{acción}_{scope}_system

Ejemplos:
  behavior_decision_will_system          // D1: decide intent
  trophic_intake_herbivore_system        // D2: herbívoro absorbe
  locomotion_energy_drain_system         // D3: movimiento drena
  homeostasis_frequency_adapt_system     // D4: adapta frecuencia
  sensory_threat_detection_system        // D5: detecta amenazas
  social_pack_formation_system           // D6: forma manada
  reproductive_isolation_guard_system    // D7: guard de especiación
  morphology_adaptive_organ_system       // D8: órganos se adaptan
  ecology_carrying_capacity_system       // D9: límite poblacional
```

---

## R-9: Error Budget por System

| Tipo | Budget | Enforcement |
|------|--------|-------------|
| Entity missing component | `let-else continue` | Compile-time (Query filters) |
| Division by zero | `equations::` guards | `DIVISION_GUARD_EPSILON` |
| NaN/Inf propagation | `finite_non_negative()` | Post-calculation clamp |
| Entity despawned mid-frame | `query.get(entity) else continue` | Runtime guard |
| Resource missing | `Res<T>` (Bevy panics if missing) | Registration in bootstrap |

---

## R-10: Documentation Budget

| Elemento | Requerido | Formato |
|----------|-----------|---------|
| System function | `///` 1 línea | Qué transforma y por qué |
| Ecuación function | `///` 1 línea + `/// f(x) = ...` | Fórmula incluida |
| Constante | `///` 1 línea solo si no-obvio | Skip si el nombre es autoexplicativo |
| Component | `///` 1 línea | Propósito de la capa |
| Inline comment | Solo math no-obvia | Invariants, no obviedades |

**No agregar**: docstrings a imports, comments a `use`, headers de archivo, separadores decorativos.
