# Sprint MG-8 — Integración, Demo y Benchmark

**Módulo:** `src/entities/builder.rs`, `src/entities/archetypes.rs`, `assets/maps/`, `benches/`, `src/worldgen/map_config.rs`
**Tipo:** Composición end-to-end + contenido + validación de rendimiento.
**Onda:** E — Requiere cadena completa MG-1 → MG-7.
**Estado:** ⏳ Pendiente

## Objetivo

Demostrar en un mapa reproducible que la morfología y el color **emergen** del acoplamiento termodinámico (sin authoring manual de "forma de pez" o "color arena"). Integrar `EntityBuilder` con API fluent para `MetabolicGraph` / ledger / albedo inferido, y fijar presupuesto de CPU (< 1 ms para 100 entidades con DAG completo).

**Resultado jugable:** `RESONANCE_MAP=morphogenesis_demo cargo run` muestra tres biomas con organismos cuya forma, color y textura emergen de la termodinámica. Fusiforme + oscuro en agua; claro + radiadores en desierto; forma intermedia en bosque.

## Responsabilidades

### MG-8A: Mapa `morphogenesis_demo.ron`

```ron
// assets/maps/morphogenesis_demo.ron
MapConfig(
    name: "morphogenesis_demo",
    description: "Demo de morfogénesis inferida: tres biomas contrastantes",
    zones: [
        Zone(
            name: "deep_ocean",
            bounds: Rect(min: (-50.0, -50.0), max: (0.0, 50.0)),
            biome: Water,
            params: ZoneParams(
                viscosity: 1000.0,       // ρ agua densa
                solar_irradiance: 5.0,   // poca luz
                delta_qe: 0.0,
                temperature_offset: -80.0, // T_env bajo
            ),
            spawns: [
                SpawnEntry(archetype: "aquatic_organism", count: 15),
            ],
        ),
        Zone(
            name: "scorched_desert",
            bounds: Rect(min: (10.0, -50.0), max: (60.0, 0.0)),
            biome: Desert,
            params: ZoneParams(
                viscosity: 1.2,          // ρ aire
                solar_irradiance: 100.0, // máxima irradiancia
                delta_qe: -2.0,         // estrés térmico
                temperature_offset: 100.0, // T_env alto
            ),
            spawns: [
                SpawnEntry(archetype: "desert_plant", count: 15),
                SpawnEntry(archetype: "desert_creature", count: 10),
            ],
        ),
        Zone(
            name: "temperate_forest",
            bounds: Rect(min: (10.0, 10.0), max: (60.0, 50.0)),
            biome: Forest,
            params: ZoneParams(
                viscosity: 1.2,
                solar_irradiance: 40.0,  // intermedia
                delta_qe: 1.0,          // ligeramente favorable
                temperature_offset: 0.0,
            ),
            spawns: [
                SpawnEntry(archetype: "forest_plant", count: 20),
            ],
        ),
    ],
)
```

- Tres biomas con parámetros contrastantes: agua densa (ρ=1000, I=5), desierto (ρ=1.2, I=100), bosque (ρ=1.2, I=40).
- Registrar en `RESONANCE_MAP` / `map_config.rs`.

### MG-8B: Arquetipos (archetypes.rs)

Mínimo tres entidades con DAG distinto. Cada una con `OrganManifest` y `MetabolicGraph` inferido.

```rust
/// Organismo acuático: fusiforme, oscuro, aletas laterales.
/// Manifest: Core + Stem + Fin×2 + Sensory.
/// Resultado esperado: fineness > 3.0, albedo < 0.3, rugosity ≈ 1.0.
pub fn spawn_aquatic_organism(commands: &mut Commands, position: Vec2) -> Entity {
    EntityBuilder::new()
        .energy(500.0)
        .volume(2.0)
        .flow(Vec2::new(4.0, 0.0), 0.05)
        .ambient(0.0, 1000.0)  // agua densa
        .with_organ_manifest(OrganManifest::from_roles(&[
            Core, Stem, Fin, Fin, Sensory,
        ]))
        .with_metabolic_graph_inferred(400.0, 280.0) // T_core, T_env
        .spawn(commands)
}

/// Planta desértica: compacta, clara, radiadores.
/// Manifest: Root + Stem + Leaf×2 + Thorn.
/// Resultado esperado: fineness ≈ 1.5, albedo > 0.7, rugosity > 2.0.
pub fn spawn_desert_plant(commands: &mut Commands, position: Vec2) -> Entity {
    EntityBuilder::new()
        .energy(200.0)
        .volume(1.0)
        .flow(Vec2::ZERO, 0.02)
        .ambient(-2.0, 1.2)  // desierto, aire
        .with_organ_manifest(OrganManifest::from_roles(&[
            Root, Stem, Leaf, Leaf, Thorn,
        ]))
        .with_metabolic_graph_inferred(450.0, 380.0) // T alto
        .spawn(commands)
}

/// Planta de bosque: forma intermedia, color medio.
/// Manifest: Root + Stem + Leaf×3 + Fruit.
/// Resultado esperado: fineness ≈ 2.0, albedo ≈ 0.4, rugosity ≈ 1.5.
pub fn spawn_forest_plant(commands: &mut Commands, position: Vec2) -> Entity {
    EntityBuilder::new()
        .energy(300.0)
        .volume(1.5)
        .flow(Vec2::ZERO, 0.03)
        .ambient(1.0, 1.2)  // bosque, aire
        .with_organ_manifest(OrganManifest::from_roles(&[
            Root, Stem, Leaf, Leaf, Leaf, Fruit,
        ]))
        .with_metabolic_graph_inferred(380.0, 300.0)
        .spawn(commands)
}
```

### MG-8C: EntityBuilder — API de morfogénesis

```rust
// entities/builder.rs — nuevos métodos fluent

impl EntityBuilder {
    /// Infiere y adjunta MetabolicGraph desde el OrganManifest ya configurado.
    /// Requiere que .with_organ_manifest() se haya llamado antes.
    /// Usa metabolic_graph_from_manifest(manifest, t_core, t_env) de MG-2.
    pub fn with_metabolic_graph_inferred(mut self, t_core: f32, t_env: f32) -> Self {
        // Construye grafo desde manifest existente
        let graph = equations::metabolic_graph_from_manifest(
            self.organ_manifest.as_ref().expect("manifest required"),
            t_core, t_env,
        );
        self.metabolic_graph = Some(graph);
        self
    }

    /// Adjunta MetabolicGraph construido manualmente (vía MetabolicGraphBuilder).
    pub fn with_metabolic_graph(mut self, graph: MetabolicGraph) -> Self {
        self.metabolic_graph = Some(graph);
        self
    }
}
```

- Campo `metabolic_graph: Option<MetabolicGraph>` en `EntityBuilder`.
- `spawn()` inserta `MetabolicGraph` si present. Los sistemas de MG-3→MG-7 se encargan de insertar `EntropyLedger`, `InferredAlbedo`, `MorphogenesisShapeParams`, `MorphogenesisSurface` en runtime.
- No violar SparseSet/Reflect — los componentes derivados los insertan los sistemas, no el builder.

### MG-8D: Benchmark

```rust
// benches/morphogenesis_benchmark.rs
use criterion::{criterion_group, criterion_main, Criterion};

/// Benchmark de la pipeline de morfogénesis (simulación pura, sin rendering).
/// Scope: metabolic_graph_step → entropy_constraint → entropy_ledger
///        → shape_optimization → surface_rugosity → albedo_inference.
/// Excluye: mesh rebuild / tessellation GF1 (medir aparte con --features=full_visual_pipeline).
fn morphogenesis_pipeline_100_entities(c: &mut Criterion) {
    c.bench_function("mg_pipeline_100_entities", |b| {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        // Registrar sistemas MG-3 → MG-7
        // Spawn 100 entidades con DAG de 12 nodos (techo)
        // Pre-warm: 1 update
        b.iter(|| {
            app.update(); // 1 tick de FixedUpdate
        });
    });
}

criterion_group!(benches, morphogenesis_pipeline_100_entities);
criterion_main!(benches);
```

- **Scope explícito:** sistemas `metabolic_graph_step` → `entropy_constraint` → `entropy_ledger` → `shape_optimization` → `surface_rugosity` → `albedo_inference`. 6 sistemas medidos.
- **Excluye:** rebuild de `Mesh`, tessellation GF1, rendering. Solo pipeline de simulación.
- **Entidades:** 100 con DAG al techo MG-2 (≤12 nodos, ≤16 aristas).
- **Umbral:** < 1 ms en hardware de referencia (documentar CPU en header del bench).
- **CI:** umbral relativo `±20%` vs baseline. Sin HW fijo en CI → solo regresión relativa.

### MG-8E: Registro de mapa

```rust
// worldgen/map_config.rs
// Agregar a la tabla de mapas:
"morphogenesis_demo" => MapConfig::from_file("assets/maps/morphogenesis_demo.ron"),
```

- `RESONANCE_MAP=morphogenesis_demo cargo run` arranca sin panic.

### MG-8F: Tabla de fenotipos esperados

| Arquetipo | Bioma | fineness | albedo | rugosity | Fenotipo visual |
|-----------|-------|----------|--------|----------|-----------------|
| `aquatic_organism` | Agua densa (ρ=1000) | > 3.0 | < 0.3 | ≈ 1.0 | Fusiforme, oscuro, liso |
| `desert_plant` | Desierto (I=100) | ≈ 1.5 | > 0.7 | > 2.0 | Compacta, clara, pliegues |
| `desert_creature` | Desierto (I=100) | ≈ 2.0 | > 0.6 | > 2.0 | Ligeramente alargada, clara, crestas |
| `forest_plant` | Bosque (I=40) | ≈ 2.0 | ≈ 0.4 | ≈ 1.5 | Intermedia, verde-medio, poco rugosa |

Estos rangos son **criterios de test**, no solo documentación. Verificar con tests de integración.

## Tácticas

- **No regresión.** Entidades legacy sin `MetabolicGraph` se ven y simulan exactamente igual. Query vacío = no-op.
- **Composición, no herencia.** El builder compone capas; los sistemas componen transformaciones. No hay "tipo de criatura" — solo composición de layers.
- **Tests antes de lo visual.** Los tests numéricos de la tabla de fenotipos son la fuente de verdad. Las capturas visuales complementan, no reemplazan.
- **Benchmark primero.** Si el benchmark falla (>1ms), optimizar antes de pulir lo visual. Opciones: BridgeCache (MG-4F, MG-5), LOD Far freeze, skip de sistemas para entidades lejanas.
- **Documentación mínima en código.** Comentarios español en puntos de integración críticos; inglés en identificadores.

## NO hace

- No redefine ecuaciones (solo uso).
- No abre scope MOBA (habilidades, daño directo por nodo) — eso es producto futuro.
- No implementa nuevos sistemas — solo wiring e integración de los existentes.
- No crea nuevos componentes — usa los definidos en MG-2→MG-7.

## Dependencias

- MG-1 → MG-7 (cadena completa).
- `src/entities/builder.rs` — EntityBuilder (patrón fluent existente: `mut self → Self`).
- `src/entities/archetypes.rs` — funciones `spawn_*`.
- `src/worldgen/map_config.rs` — registro de mapas.
- `assets/maps/` — archivos RON.

## Criterios de aceptación

### MG-8A (Mapa)
- Test: `MapConfig::from_file("assets/maps/morphogenesis_demo.ron")` carga sin error.
- Test: 3 zonas con nombres `"deep_ocean"`, `"scorched_desert"`, `"temperate_forest"`.
- Test: parámetros de zona coinciden con tabla (viscosity, irradiance, etc.).

### MG-8B (Arquetipos)
- Test: `spawn_aquatic_organism` → entidad tiene `MetabolicGraph` con ≥5 nodos.
- Test: `spawn_desert_plant` → entidad tiene `MetabolicGraph` con ≥5 nodos.
- Test: `spawn_forest_plant` → entidad tiene `MetabolicGraph` con ≥6 nodos.
- Test: ningún arquetipo paniquea al construir.

### MG-8C (EntityBuilder)
- Test: `.with_metabolic_graph_inferred(400.0, 280.0)` con manifest válido → `MetabolicGraph` insertado en spawn.
- Test: `.with_metabolic_graph(graph)` con grafo manual → insertado sin cambio.
- Test: `.with_metabolic_graph_inferred()` sin `.with_organ_manifest()` previo → panic controlado (expect).
- Test: cadena completa `.energy().volume().flow().ambient().with_organ_manifest().with_metabolic_graph_inferred().spawn()` → entidad válida con todas las capas.

### MG-8D (Benchmark)
- Test: 100 entidades (12 nodos cada una) → pipeline MG completo < 1 ms (valor medido en bench).
- Test: benchmark reproducible — desviación estándar < 10% entre runs.

### MG-8E (Fenotipos — tabla numérica)
- Test (integración, 10 ticks): `aquatic_organism` → `fineness_ratio > 3.0`.
- Test (integración, 10 ticks): `aquatic_organism` → `albedo < 0.3`.
- Test (integración, 10 ticks): `aquatic_organism` → `rugosity < 1.3`.
- Test (integración, 10 ticks): `desert_plant` → `albedo > 0.7`.
- Test (integración, 10 ticks): `desert_plant` → `rugosity > 2.0`.
- Test (integración, 10 ticks): `forest_plant` → `albedo ∈ [0.25, 0.55]`.
- Test (integración, 10 ticks): `forest_plant` → `rugosity ∈ [1.0, 2.0]`.

### MG-8F (Smoke test)
- `RESONANCE_MAP=morphogenesis_demo cargo run` arranca sin panic y muestra entidades en pantalla.
- `cargo test --lib` sin regresión.

### Checklist visual (complemento, no gate)
- [ ] Organismo acuático visualmente alargado y oscuro.
- [ ] Planta desértica visualmente compacta y clara.
- [ ] Planta de bosque intermedia.
- [ ] Capturas adjuntas al PR.

## Referencias

- `docs/design/MORPHOGENESIS.md` §6 MG-8, §5 (valor jugable)
- `docs/sprints/MORPHOGENESIS_INFERENCE/README.md` — ejemplo motivador criatura acuática
- `src/entities/builder.rs` — EntityBuilder (patrón fluent: `energy()`, `volume()`, `flow()`, `ambient()`)
- `src/entities/archetypes.rs` — funciones `spawn_*` existentes
- `src/worldgen/map_config.rs` — registro de mapas `RESONANCE_MAP`
- `CLAUDE.md` — EntityBuilder, mapas `assets/maps/*.ron`
