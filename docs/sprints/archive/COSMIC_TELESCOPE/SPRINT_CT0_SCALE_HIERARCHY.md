# CT-0: Scale Hierarchy — Data Model + State Machine

**Esfuerzo:** M (2–3 sesiones)
**Bloqueado por:** nada
**ADR:** ADR-036 §D1

## Objetivo

Definir el modelo de datos para la jerarquía de escalas espaciales. Sin simulación
nueva — solo el andamiaje que los sprints posteriores rellenan.

## Precondiciones

- `SimWorldFlat` funcional (`batch/arena.rs`)
- TelescopeStack funcional (`batch/telescope/stack.rs`)

## Entregables

### E1: `ScaleLevel` enum + metadata

```rust
// src/cosmic/mod.rs

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Reflect)]
pub enum ScaleLevel {
    Cosmological,  // S0: clusters, dt ~ 10^6
    Stellar,       // S1: estrellas, dt ~ 10^4
    Planetary,     // S2: superficie, dt ~ 100
    Ecological,    // S3: vida, dt = 1 tick
    Molecular,     // S4: átomos, dt = 0.005
}

impl ScaleLevel {
    pub fn dt_ratio(&self) -> f64;          // dt relativo a S3 (base)
    pub fn parent(&self) -> Option<Self>;   // S0 no tiene padre
    pub fn child(&self) -> Option<Self>;    // S4 no tiene hijo
    pub fn depth(&self) -> u8;              // 0..4
}
```

### E2: `CosmicState` — estado de una escala activa

```rust
// src/cosmic/scale_manager.rs

pub struct ScaleInstance {
    pub level: ScaleLevel,
    pub world: Box<SimWorldFlat>,
    pub parent_entity_id: Option<u32>,  // qué entidad del nivel superior se expandió
    pub zoom_seed: u64,                 // determinista
    pub age_ticks: u64,                 // ticks simulados en este nivel
    pub frozen: bool,                   // background congelado
}

#[derive(Resource)]
pub struct ScaleManager {
    pub observed: ScaleLevel,           // nivel con resolución completa
    pub instances: Vec<ScaleInstance>,   // niveles activos (max 5)
}
```

### E3: `CosmicPlugin` — registro en Bevy

```rust
pub struct CosmicPlugin;

impl Plugin for CosmicPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<ScaleLevel>()
            .init_resource::<ScaleManager>()
            .add_event::<ZoomInEvent>()
            .add_event::<ZoomOutEvent>();
    }
}
```

## Tasks

- [ ] Crear `src/cosmic/mod.rs` con `ScaleLevel`, `CosmicPlugin`
- [ ] Crear `src/cosmic/scale_manager.rs` con `ScaleInstance`, `ScaleManager`
- [ ] Registrar `CosmicPlugin` en app (feature-gated si necesario)
- [ ] Tests unitarios:
  - `scale_level_parent_child_consistency`
  - `scale_level_depth_ordering`
  - `scale_manager_default_is_ecological` (nivel actual del juego)
  - `dt_ratio_monotone_with_depth`
- [ ] Definir `ZoomInEvent` y `ZoomOutEvent` (structs, sin handler aún)

## Criterios de aceptación

1. `ScaleLevel` cubre S0–S4 con parent/child navegación
2. `ScaleManager` puede instanciar y destruir niveles
3. 0 warnings, 0 clippy, tests pasan
4. No toca ningún archivo existente excepto `lib.rs` (agregar `pub mod cosmic`)
