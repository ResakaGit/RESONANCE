# DC-5: Simulation↔Worldgen Boundary Hardening

**Objetivo:** Establecer una frontera clara entre simulation/ y worldgen/ mediante: (1) API semántica para `EnergyFieldGrid`, (2) extracción del system chaining de prephysics, (3) contratos de ownership.

**Estado:** PENDIENTE
**Esfuerzo:** A (~12 archivos, ~200 LOC refactor + API nueva)
**Bloqueado por:** DC-3 ✅
**Desbloquea:** Estabilidad de fronteras, testing aislado de worldgen

---

## Análisis profundo del codebase

### cell_xy_mut en simulation/ — 5 operaciones semánticas

104 llamadas a `cell_xy_mut` en 25 archivos. En simulation/ se reducen a 5 operaciones:

| Operación | Sistemas | Archivos | Calls |
|-----------|----------|----------|-------|
| **register_entity** | abiogenesis, materialization | abiogenesis/mod.rs:200,315 | 2 |
| **drain_qe** | nutrient_uptake, trophic, osmosis, nucleus_recycling | 4 archivos | ~15 |
| **deposit_qe** | nutrient_return, osmosis, radiation_pressure | 4 archivos | ~10 |
| **drain_nutrient** | trophic, ecology_dynamics, nutrient_depletion | 3 archivos | ~8 |
| **read_cell_state** | sensory, morpho_adaptation, awakening, shape_inference | 5 archivos | ~10 |

### prephysics.rs — 2 funciones de registro

```
register_worldgen_core_prephysics_chain():      // líneas 42-87
  → 21 sistemas encadenados (todos worldgen propios)
  → Phase::ThermodynamicLayer, run_if Playing

register_grimoire_and_spatial_index():          // líneas 91-112
  → channeling_grimoire_emit_system            (simulation/)
  → grimoire_cast_resolve_system               (simulation/)
  → update_spatial_index_system                (world/)
  → Phase::ThermodynamicLayer, run_if Playing+Active
```

**Solo `register_grimoire_and_spatial_index` importa de simulation/** — los 21 sistemas de worldgen core son legítimos.

---

## Diseño DC-5A: Facade API semántica para EnergyFieldGrid

### Principio: `pub(crate) cell_xy_mut`, `pub` métodos semánticos

Los sistemas de simulation/ usan 5 operaciones. Cada una se convierte en un método con nombre que describe la intención, valida invariantes, y encapsula el acceso:

```rust
// worldgen/field_grid.rs — extender impl EnergyFieldGrid

// === Read API (ya existente, mantener pub) ===
pub fn cell_xy(&self, x: u32, y: u32) -> Option<&EnergyCell>     // sin cambios
pub fn cell_at(&self, world_pos: Vec2) -> Option<&EnergyCell>     // sin cambios
pub fn world_pos(&self, x: u32, y: u32) -> Option<Vec2>          // sin cambios
pub fn world_to_cell(&self, pos: Vec2) -> Option<(u32, u32)>     // sin cambios

// === Write API (NUEVO — facade semántica) ===

/// Registra una entidad materializada en una celda.
pub fn register_materialized(&mut self, x: u32, y: u32, entity: Entity) -> bool {
    if let Some(cell) = self.cell_xy_mut(x, y) {
        cell.materialized_entity = Some(entity);
        true
    } else {
        false
    }
}

/// Desregistra una entidad de una celda (post-death/despawn).
pub fn unregister_materialized(&mut self, x: u32, y: u32, entity: Entity) -> bool {
    if let Some(cell) = self.cell_xy_mut(x, y) {
        if cell.materialized_entity == Some(entity) {
            cell.materialized_entity = None;
            true
        } else {
            false
        }
    } else {
        false
    }
}

/// Drena qe de una celda. Retorna el qe efectivamente drenado (cap al disponible).
/// Axiom 5: energy never created — drain is bounded.
pub fn drain_qe(&mut self, x: u32, y: u32, amount: f32) -> f32 {
    if let Some(cell) = self.cell_xy_mut(x, y) {
        let drained = amount.min(cell.accumulated_qe).max(0.0);
        cell.accumulated_qe -= drained;
        drained
    } else {
        0.0
    }
}

/// Deposita qe en una celda. Axiom 1: energy is fungible.
pub fn deposit_qe(&mut self, x: u32, y: u32, amount: f32) {
    if let Some(cell) = self.cell_xy_mut(x, y) {
        cell.accumulated_qe += amount.max(0.0);
    }
}

/// Vista read-only consolidada de una celda.
pub fn cell_state(&self, x: u32, y: u32) -> Option<CellStateView> {
    self.cell_xy(x, y).map(|c| CellStateView {
        qe: c.accumulated_qe,
        frequency: c.dominant_frequency_hz,
        matter_state: c.matter_state,
        materialized: c.materialized_entity,
    })
}

// === Internal API (worldgen-only) ===
pub(crate) fn cell_xy_mut(&mut self, x: u32, y: u32) -> Option<&mut EnergyCell>  // restricción
pub(crate) fn cell_at_mut(&mut self, world_pos: Vec2) -> Option<&mut EnergyCell>  // restricción
```

### CellStateView (struct de lectura)

```rust
/// Vista read-only de una celda energética. No expone EnergyCell.
#[derive(Debug, Clone, Copy)]
pub struct CellStateView {
    pub qe: f32,
    pub frequency: f32,
    pub matter_state: MatterState,
    pub materialized: Option<Entity>,
}
```

### NutrientFieldGrid — misma estrategia

```rust
// worldgen/nutrient_field.rs — extender

/// Drena nutrientes de una celda. Retorna perfil drenado real.
pub fn drain_nutrient(&mut self, x: usize, y: usize, carbon: f32, nitrogen: f32, phosphorus: f32) -> (f32, f32, f32) { ... }

/// Deposita nutrientes en una celda (nutrient return on death).
pub fn deposit_nutrient(&mut self, x: usize, y: usize, carbon: f32, nitrogen: f32, phosphorus: f32) { ... }

/// Vista read-only de nutrientes en una celda.
pub fn nutrient_state(&self, x: usize, y: usize) -> Option<NutrientStateView> { ... }
```

---

## Diseño DC-5B: Extraer system chaining de prephysics

### Análisis: solo 1 función necesita migración

`register_grimoire_and_spatial_index()` (prephysics.rs:91-112) importa:
- `crate::simulation::ability_targeting::channeling_grimoire_emit_system`
- `crate::simulation::input::grimoire_cast_resolve_system`
- `crate::world::space::update_spatial_index_system`

Los 21 sistemas en `register_worldgen_core_prephysics_chain()` son todos de worldgen — legítimos.

### Solución: mover grimoire chain al InputPlugin

```rust
// ANTES (prephysics.rs:91-112):
pub fn register_grimoire_and_spatial_index(app: &mut App, schedule: impl ScheduleLabel + Clone) {
    app.add_systems(schedule, (
        channeling_grimoire_emit_system,
        grimoire_cast_resolve_system,
    ).chain()
     .in_set(Phase::ThermodynamicLayer)
     .run_if(in_state(GameState::Playing).and(in_state(PlayState::Active))));

    app.add_systems(schedule, update_spatial_index_system
        .in_set(Phase::ThermodynamicLayer)
        .after(grimoire_cast_resolve_system)
        .run_if(in_state(GameState::Playing).and(in_state(PlayState::Active))));
}

// DESPUÉS:
// Eliminar register_grimoire_and_spatial_index de prephysics.rs
// Mover a plugins/input_plugin.rs:
app.add_systems(FixedUpdate, (
    channeling_grimoire_emit_system,
    grimoire_cast_resolve_system,
).chain()
 .in_set(Phase::ThermodynamicLayer)
 .run_if(in_state(GameState::Playing).and(in_state(PlayState::Active))));

// Mover spatial index a plugins/atomic_plugin.rs (o thermodynamic_plugin.rs):
app.add_systems(FixedUpdate, update_spatial_index_system
    .in_set(Phase::ThermodynamicLayer)
    .after(grimoire_cast_resolve_system)
    .run_if(in_state(GameState::Playing).and(in_state(PlayState::Active))));
```

### Verificar: ¿quién llama register_grimoire_and_spatial_index?

Debe ser `pipeline.rs` o un plugin. Verificar y migrar el call site.

---

## Plan de ejecución (5 commits)

### Commit 1: CellStateView + read API + tests

- Añadir `CellStateView` struct a field_grid.rs
- Añadir `cell_state()` method
- Tests: bounds, values, None for OOB
- `cargo test` pasa

### Commit 2: Mutation facade (drain, deposit, register)

- Añadir `drain_qe()`, `deposit_qe()`, `register_materialized()`, `unregister_materialized()`
- Tests: conservation (deposit then drain = original), cap at available, register/unregister round-trip
- `cargo test` pasa (sin migrar consumidores aún — coexisten)

### Commit 3: Migrar consumidores de simulation/

- `cell_xy_mut` → `pub(crate)` — esto rompe compilation de simulation/
- Migrar cada sistema a usar facade API
- Para osmosis (caso complejo): mantener `cell_xy_mut` accesible via pub(crate) helper en worldgen, crear wrapper para osmosis
- `cargo test` pasa

### Commit 4: Extraer grimoire chain de prephysics

- Mover grimoire systems a InputPlugin
- Mover spatial_index a AtomicPlugin/ThermodynamicPlugin
- Eliminar `register_grimoire_and_spatial_index` de prephysics.rs
- Eliminar imports de `crate::simulation` en prephysics.rs
- `cargo test` pasa

### Commit 5: NutrientFieldGrid facade + documentation

- Añadir drain_nutrient/deposit_nutrient/nutrient_state a NutrientFieldGrid
- Documentar ownership contracts
- Grep validation

---

## Testing TDD (3 capas)

### Capa 1: Unitario — facade methods

```rust
#[test]
fn drain_qe_returns_actual_drained() {
    let mut grid = EnergyFieldGrid::new(4, 4, 1.0, Vec2::ZERO);
    grid.deposit_qe(1, 1, 50.0);
    assert!((grid.drain_qe(1, 1, 30.0) - 30.0).abs() < 1e-6);
}

#[test]
fn drain_qe_caps_at_available() {
    let mut grid = EnergyFieldGrid::new(4, 4, 1.0, Vec2::ZERO);
    grid.deposit_qe(1, 1, 10.0);
    assert!((grid.drain_qe(1, 1, 100.0) - 10.0).abs() < 1e-6);
}

#[test]
fn drain_qe_never_negative() {
    let mut grid = EnergyFieldGrid::new(4, 4, 1.0, Vec2::ZERO);
    let _ = grid.drain_qe(1, 1, 50.0);
    assert!(grid.cell_state(1, 1).unwrap().qe >= 0.0);
}

#[test]
fn deposit_then_drain_conservation() {
    let mut grid = EnergyFieldGrid::new(4, 4, 1.0, Vec2::ZERO);
    grid.deposit_qe(2, 2, 100.0);
    let d1 = grid.drain_qe(2, 2, 40.0);
    let d2 = grid.drain_qe(2, 2, 40.0);
    let remaining = grid.cell_state(2, 2).unwrap().qe;
    assert!((d1 + d2 + remaining - 100.0).abs() < 1e-5);
}

#[test]
fn register_unregister_round_trip() {
    let mut grid = EnergyFieldGrid::new(4, 4, 1.0, Vec2::ZERO);
    let e = Entity::from_raw(42);
    assert!(grid.register_materialized(1, 1, e));
    assert_eq!(grid.cell_state(1, 1).unwrap().materialized, Some(e));
    assert!(grid.unregister_materialized(1, 1, e));
    assert_eq!(grid.cell_state(1, 1).unwrap().materialized, None);
}

#[test]
fn unregister_wrong_entity_noop() {
    let mut grid = EnergyFieldGrid::new(4, 4, 1.0, Vec2::ZERO);
    let a = Entity::from_raw(1);
    let b = Entity::from_raw(2);
    grid.register_materialized(1, 1, a);
    assert!(!grid.unregister_materialized(1, 1, b));
    assert_eq!(grid.cell_state(1, 1).unwrap().materialized, Some(a));
}

#[test]
fn cell_state_oob_returns_none() {
    let grid = EnergyFieldGrid::new(4, 4, 1.0, Vec2::ZERO);
    assert!(grid.cell_state(100, 100).is_none());
}
```

### Capa 2: Integración — sistemas migrados

```rust
#[test]
fn abiogenesis_uses_register_api() {
    // Setup: MinimalPlugins + grid + abiogenesis prerequisites
    // Assert: materialized entity registered via register_materialized
}

#[test]
fn nutrient_depletion_uses_drain_api() {
    // Setup: grid + entity with NutrientProfile
    // Assert: nutrients decrease via drain_nutrient
}
```

### Capa 3: Orquestación — startup→gameplay funciona

```rust
fn run_startup_gameplay_test<A>(ticks: u32, assert_fn: A)
where A: FnOnce(&World) {
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, SimulationPlugin));
    for _ in 0..ticks { app.update(); }
    assert_fn(app.world());
}

#[test]
fn full_lifecycle_with_facade_api() {
    run_startup_gameplay_test(20, |world| {
        let grid = world.resource::<EnergyFieldGrid>();
        // Verify entities materialized
        // Verify energy conservation
    });
}
```

---

## Axiomas respetados

| Axioma | Cómo se respeta |
|--------|-----------------|
| 1 (Energy) | `drain_qe` / `deposit_qe` — qe transferido, no creado |
| 2 (Pool) | `drain_qe` caps at available — child ≤ parent |
| 4 (Dissipation) | drain retorna menos que requested si cell tiene menos |
| 5 (Conservation) | `deposit + drain + remaining = original` (test explícito) |

---

## Riesgo: osmosis usa acceso directo complejo

`osmotic_diffusion_system` (osmosis.rs) tiene ~15 llamadas a `cell_xy_mut` con patterns complejos (double-buffered deltas, frequency mixing). **Estrategia:**

1. Mantener `cell_xy_mut` como `pub(crate)` (no eliminarlo)
2. Osmosis vive en `simulation/thermodynamic/` pero opera sobre el grid con pattern de doble buffer
3. Crear un `OsmoticTransfer` struct que encapsula los deltas, y un método `apply_osmotic_transfers(&mut self, transfers: &[OsmoticTransfer])` en EnergyFieldGrid
4. Si la complejidad es excesiva, osmosis puede usar un helper en worldgen que expone la operación de "transfer between cells":

```rust
/// Transfiere qe entre dos celdas con pérdida por disipación (Axiom 4).
pub fn transfer_qe_between_cells(
    &mut self, from: (u32, u32), to: (u32, u32), amount: f32, loss_rate: f32,
) -> f32 {
    let drained = self.drain_qe(from.0, from.1, amount);
    let transferred = drained * (1.0 - loss_rate);
    self.deposit_qe(to.0, to.1, transferred);
    drained - transferred // energy lost to dissipation
}
```

Esto preserva Axiom 4 (loss_rate > 0) y Axiom 5 (conservation: drained = transferred + lost).

---

## Criterios de cierre

- [ ] `cargo test` — 0 failures
- [ ] `cell_xy_mut` es `pub(crate)` (no `pub`)
- [ ] Facade methods: drain_qe, deposit_qe, register_materialized, unregister_materialized, cell_state
- [ ] 7+ tests unitarios para facade
- [ ] Conservation test: deposit + drain + remaining = original
- [ ] `grep "use crate::simulation" src/worldgen/systems/prephysics.rs` — 0 resultados
- [ ] Grimoire chain registrado en InputPlugin, no en prephysics
- [ ] Ownership documentation en worldgen/contracts.rs
- [ ] Ningún `// DEBT:` introducido
