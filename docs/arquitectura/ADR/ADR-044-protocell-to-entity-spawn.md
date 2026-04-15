# ADR-044: Protocell → Entity ECS · `FissionEvent` Observer

**Estado:** Propuesto
**Fecha:** 2026-04-15
**Contexto:** AUTOPOIESIS Integration (Sprint AI, ítem AI-2)
**ADRs relacionados:** ADR-039 (fission criterion), ADR-041 (lineage in report), ADR-043, ADR-045

## 1. Contexto y problema

- Módulos afectados:
  - `src/use_cases/experiments/autopoiesis/soup_sim.rs` (hoy registra `FissionEventRecord` en `Vec` interno)
  - `src/simulation/chemical/fission.rs` (sistema actual sólo cuenta `pressure_events` legacy)
  - `src/layers/*` (candidato a recibir `LineageTag` component)
  - `src/simulation/pipeline.rs` (phase `ChemicalLayer`)

Cuando un blob AP cruza `pressure_ratio > FISSION_PRESSURE_RATIO`
(ahora = 4 tras AP-6d), `SoupSim` dispara un `FissionEventRecord` que
queda en `Vec<FissionEventRecord>` dentro del stepper y aparece en el
`SoupReport` al finalizar.

**Problema:** esa fisión no produce ninguna entity en el world Bevy.
La "célula" que nació por autopoiesis es invisible al resto del
simulador (L6 metabolismo, L9 identidad, L10 resonance links, etc.
nunca la ven).  El gap #2 del audit.

Necesitamos:
1. Un event bus ECS con el record de la fisión
2. Un observer que spawn dos entities con los atributos heredados
3. Una forma de trackear linaje post-spawn (`LineageTag` componente)
4. No romper `SoupSim` como "stepper sin Bevy" (ADR-040 §2)

## 2. Alternativas

| Opción | Descripción | Pros | Contras |
|---|---|---|---|
| **A · Event bus + Observer** | `FissionEvent` Bevy Event; Observer en phase `ChemicalLayer` consume y spawna | Desacoplado, testeable, ECS-native | 2 pasos lógicos (record → event → spawn), overhead mínimo |
| B · Commands desde SoupSim | `SoupSim::step` recibe `&mut Commands` y spawna directo | 1-paso | Acopla stepper a Bevy, viola ADR-040 §2 "Bevy-free stepper" |
| C · Polling por system | System lee `SoupSim.fission_events()` cada tick y spawna los nuevos | Sin event bus | Estado implícito (qué eventos ya procesé), duplica responsabilidad |
| D · Component-per-blob | Cada blob es entity desde el inicio | ECS-native completo | Viola ADR-038 §Axiom 6 — blobs emergen como patrón, no como entity predeclarada |

## 3. Decisión

**Opción A — Event bus + Observer, con bridge resource para transportar records entre `SoupSim` y el world ECS.**

> **Revisión durante implementación 2026-04-15.**
> - `FissionEventRecord` enriquecido con `centroid: (f32, f32)` y `qe_per_child: f32` (computados en `SoupSim::step` antes de fisión, cuando el blob aún existe).  Backward-compatible vía `#[serde(default)]`.
> - El bridge vive en `src/simulation/autopoiesis_bridge.rs` (no `chemical/fission_bridge.rs`) — patrón flat consistente con `species_to_qe.rs` de AI-1.
> - `ReactionNetworkResource` / `SpeciesGridResource` / `SoupSimResource` wrappers no necesarios para los dos primeros (ya derivan `Resource`).  Solo `SoupSimResource(pub SoupSim)` se introduce.
> - `Cap MAX_FISSION_EVENTS_PER_TICK = 4` (8 entities/tick max) protege contra cascadas patológicas, con `warn!` log si se satura.
> - `step_soup_sim_system` agregado al chain — necesario para que el sim avance dentro del pipeline ECS (fuera del binario standalone).

### Bridge: `PendingFissionEvents` resource

`SoupSim` sigue siendo Bevy-free (ADR-040 §2).  Un sistema adaptador
lee `SoupSim::fission_events()` tras cada step, compara contra el
contador procesado anterior, y emite los nuevos como `FissionEvent`.

```rust
// src/simulation/chemical/fission_bridge.rs (nuevo)

#[derive(Resource, Default)]
pub struct FissionEventCursor { last_processed: usize }

pub fn emit_fission_events_system(
    sim: Res<SoupSimResource>,
    mut cursor: ResMut<FissionEventCursor>,
    mut events: EventWriter<FissionEvent>,
    network: Res<ReactionNetworkResource>,
    species: Res<SpeciesGridResource>,
) {
    let records = sim.0.fission_events();
    for record in records[cursor.last_processed..].iter() {
        let (cx, cy) = record_centroid(record, &species.0);
        let mean_freq = mean_product_frequency(record, &network.0, &species.0);
        let qe_per_child = (total_qe_in_region(record, &species.0)
            * (1.0 - DISSIPATION_PLASMA))
            * 0.5; // ADR-039 §apply_fission, mitad por hijo
        events.send(FissionEvent {
            tick: record.tick,
            parent_lineage: record.parent,
            children_lineages: record.children,
            centroid: Vec2::new(cx, cy),
            mean_freq,
            qe_per_child,
        });
    }
    cursor.last_processed = records.len();
}
```

### Event + Observer

```rust
// src/events/fission.rs (nuevo)

#[derive(Event, Clone, Debug, Reflect)]
#[reflect(Debug)]
pub struct FissionEvent {
    pub tick: u64,
    pub parent_lineage: u64,
    pub children_lineages: [u64; 2],
    pub centroid: Vec2,
    pub mean_freq: f32,
    pub qe_per_child: f32,
}

// src/layers/lineage_tag.rs (nuevo)
#[derive(Component, Reflect, Debug, Clone, Copy)]
#[reflect(Component)]
pub struct LineageTag(pub u64);
```

### Observer

```rust
// src/simulation/chemical/spawn_on_fission.rs (nuevo)

pub fn on_fission_spawn_entity(
    mut commands: Commands,
    mut events: EventReader<FissionEvent>,
    params: Res<SimWorldTransformParams>,
) {
    for ev in events.read() {
        for &lineage in &ev.children_lineages {
            let world_pos = params.cell_to_world(ev.centroid);
            commands.spawn((
                BaseEnergy::new(ev.qe_per_child),
                OscillatorySignature::new(ev.mean_freq, 0.0),
                LineageTag(lineage),
                Transform::from_translation(world_pos.extend(0.0)),
                StateScoped(GameState::Playing),
                Name::new(format!("cell_lin{:08x}", (lineage & 0xFFFF_FFFF))),
            ));
        }
    }
}
```

### Registro

```rust
// src/plugins/layers.rs
app.add_event::<FissionEvent>()
   .register_type::<LineageTag>()
   .register_type::<FissionEvent>()
   .init_resource::<FissionEventCursor>()
   .add_systems(
       FixedUpdate,
       (emit_fission_events_system, on_fission_spawn_entity)
           .chain()
           .in_set(Phase::ChemicalLayer),
   );
```

## 4. Justificación

1. **ADR-040 §2 respetado.** `SoupSim` sigue Bevy-free.  El bridge vive en un módulo adaptador.
2. **Axiom 6 intacto.** Entity spawnea **cuando el criterio físico se cumple** (ratio>4 ⇒ fisión ⇒ record ⇒ event ⇒ spawn).  Cero reglas top-down.
3. **Axiom 2 (pool invariant).** `qe_per_child = (Σ qe_blob × (1-DISSIPATION_PLASMA)) / 2` — conserva la partición que ya hace `apply_fission` (ADR-039 §5).
4. **Axiom 8.** `OscillatorySignature::new(mean_freq, 0.0)` hereda la freq promedio de los productos, no se decreta.
5. **Lineage preservado.** `LineageTag(u64)` encapsula el hash determinístico de `child_lineage(parent, tick, side)` (ADR-041 §3) — queryable, debuggable.
6. **Determinismo.** Cursor avanza secuencialmente; dos runs mismo seed ⇒ mismos spawns en el mismo tick.

## 5. No viola axiomas

| Ax | Cumplimiento |
|---|---|
| 1 | Entity spawneada tiene `BaseEnergy` — qe como único stat |
| 2 | `qe_per_child` conserva partición de `apply_fission`; total pre = total post + tax plasma |
| 3 | Entidades spawneadas compiten igual que cualquier otra BehavioralAgent-like |
| 4 | Tax `DISSIPATION_PLASMA` ya incluido en `apply_fission`; el spawn no crea qe |
| 5 | Sin creación neta |
| 6 | **Núcleo.** Spawn sólo si pressure emerge > threshold; cero reglas "en tick T spawnear X" |
| 7 | `Transform` coloca en el centroide real del blob, no posición arbitraria |
| 8 | `mean_freq` heredada de productos, no fija |

## 6. Costos

- Compilación: +1 event, +1 component, +2 systems
- Runtime: O(nuevos_events_por_tick) ≈ ≤4 por tick en escenarios realistas (fission rate bajo por diseño)
- Memoria: LineageTag = 8 B/entity
- Complejidad: event bus + cursor + observer. Bien aislado.

## 7. Archivos modificados

| Archivo | Cambio |
|---|---|
| `src/events/fission.rs` | **NUEVO** `FissionEvent` |
| `src/layers/lineage_tag.rs` | **NUEVO** `LineageTag` component |
| `src/layers/mod.rs` | + `mod lineage_tag;` |
| `src/simulation/chemical/fission_bridge.rs` | **NUEVO** `emit_fission_events_system` + `FissionEventCursor` |
| `src/simulation/chemical/spawn_on_fission.rs` | **NUEVO** `on_fission_spawn_entity` |
| `src/simulation/chemical/mod.rs` | + 2 mods |
| `src/plugins/layers.rs` | + event + component registro + systems |
| `src/use_cases/experiments/autopoiesis/mod.rs` | + `SoupSimResource` wrapper si se usa (opcional) |

## 8. Tests

- **Unit:** `fission_event_cursor_advances_exactly_once_per_record`
- **Unit:** `lineage_tag_reflects_child_lineage_hash` — spawn tras record ⇒ `LineageTag.0 == record.children[0]`
- **Integration:** `formose_spot_end_to_end_spawns_two_entities` — seed 0 + spot + calibrado (ratio>4) ⇒ world tiene 2+ entities con `LineageTag` distintos
- **Integration:** `cell_qe_conservation` — `Σ LineageTag.qe_per_child == (pre_qe × (1-DP))`
- **Regression:** sin fission, cero entities spawneadas (`assert_eq!(count_with_tag, 0)`)

## 9. Decisión revisable cuando

- Si ADR-045 elige mass-action como canónica, este ADR puede absorber la responsabilidad de ser el único spawn-path de vida en el sim (reemplazaría `abiogenesis_system` actual).
- Si aparecen fisiones en cascada (hijo fisiona inmediatamente) y el observer satura `Commands`, agregar `MAX_SPAWN_PER_TICK=4`.
- `LineageTag` puede evolucionar a `Lineage { id: u64, parent: u64, birth_tick: u64 }` si se necesitan queries genealógicas — fuera de scope AI-2.
