# Resonance — Motor de Simulación Alquímica

Simulación física basada en resonancia de ondas para un MOBA alquímico, construida con **Rust** y **Bevy 0.15 ECS**.

## Concepto

Cada entidad del mundo es un ensamblaje de **14 capas ortogonales** (L0 BaseEnergy — L13 StructuralLink). Las capas son independientes entre sí pero sus interacciones cruzadas producen comportamiento emergente: elementos, estados de materia, hechizos, colisiones y vida surgen de las mismas ecuaciones termodinámicas.

No existen stats hardcodeados como "HP", "ATK" o "DEF". Todo es energía (`qe`), frecuencia (`Hz`), fase (`φ`), densidad (`ρ`) y coherencia (`Eb`).

## Arquitectura

```
src/
├── layers/          14 capas ECS + auxiliares (24 archivos)
├── simulation/      Pipeline FixedUpdate: Input → Thermo → Atomic → Chemical → Metabolic → Morphological
├── entities/        EntityBuilder, arquetipos (spawn_*)
├── blueprint/       Motor matemático puro (equations/, constants/, almanac/)
├── bridge/          Cache optimizer (BridgeCache<B>, 11 equation kinds)
├── eco/             Eco-boundaries, zones, climate
├── geometry_flow/   GF1 flora-tubo (branching stateless)
├── topology/        Terrain: noise, slope, drainage, hydraulics
├── worldgen/        V7: field_grid, nucleus, propagation, materialization
├── rendering/       quantized_color
├── runtime_platform/ 17 sub-módulos (compat 2D/3D, tick, input, camera, HUD, fog)
├── plugins/         SimulationPlugin, LayersPlugin, DebugPlugin
└── events.rs        Eventos Bevy (contrato entre sistemas)
```

- **Diseño:** [docs/design/](./docs/design/) — Especificaciones de alto nivel e índice en [INDEX.md](./docs/design/INDEX.md).
- **Contratos por módulo:** [docs/arquitectura/](./docs/arquitectura/) — 30 blueprints runtime.
- **Backlog activo:** [docs/sprints/](./docs/sprints/) — Tracks con sprints pendientes.
- **Raíz:** [TOPOLOGY_AND_LAYERS.md](./TOPOLOGY_AND_LAYERS.md), [PLANT_SIMULATION.md](./PLANT_SIMULATION.md), [DESIGNING.md](./DESIGNING.md).

## Requisitos

- Rust 1.80+ (`rustup update stable`)
- macOS / Linux / Windows

## Ejecutar

```bash
cargo run
```

El arranque carga worldgen desde `assets/maps/default.ron` (o el mapa indicado por entorno), completa el warmup y luego **un solo héroe** (`demo_level.rs`). Los **colores del campo** son las celdas materializadas V7 (bridge 3D / sprites 2D); los gizmos de debug **no** dibujan esas celdas para no tapar el mosaico.

**Mapa mínimo (grid chico, warmup corto, mover al héroe):**

```bash
RESONANCE_MAP=demo_minimal cargo run
```

**Demo guiada (Terra + presión + grid pequeño):** ver [docs/DEMO_FLOW.md](./docs/DEMO_FLOW.md).

```bash
RESONANCE_MAP=demo_floor cargo run
```

**Demo estratos (Terra suelo + Ventus “atmósfera”, orbes + losa cielo, mosaico de colores):**

```bash
RESONANCE_MAP=demo_strata cargo run
```

**Cuatro flores (grid 32×32, cuatro núcleos Terra-band, interferencia visible en el campo V7):**

```bash
RESONANCE_MAP=four_flowers cargo run
```

**Flor procedural (geometry_flow + pistilo, mapa `flower_demo`):**

```bash
RESONANCE_MAP=flower_demo cargo run
```

**Core demo 3D (por defecto):** `cargo run` sin variables → perfil **`full3d`** (rig 3D, bridge, `CameraRigTarget` al héroe demo). Usá `cargo run` desde la raíz del crate para assets.

**2D / hybrid:** `RESONANCE_RENDER_COMPAT_PROFILE=legacy2d` o `=hybrid` (cámara 2D con zoom en legacy; sync de sprites `EnergyVisual`).

Valores: `legacy2d`, `hybrid`, `full3d` (también `RESONANCE_V6_PROFILE` como alias). Binario suelto sin `CARGO_MANIFEST_DIR`: definí `BEVY_ASSET_ROOT` al path del crate.

## Tests

```bash
cargo test
```

## Licencia

Privado — Todos los derechos reservados.
