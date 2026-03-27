# Sprint GS-6 — Map Energy: El Mapa como Paisaje Energético

**Modulo:** `src/worldgen/systems/node_control.rs` (nuevo), `src/blueprint/equations/node_control.rs` (nuevo), `src/blueprint/constants/node_control.rs` (nuevo)
**Tipo:** Ecuaciones puras + componentes marcadores + sistema de control.
**Onda:** A — Requiere GS-5 (VictoryNucleus) + `EnergyFieldGrid` existente.
**Estado:** ⏳ Pendiente

---

## Contexto: que ya existe

**Lo que SÍ existe:**

- `worldgen/EnergyFieldGrid` — grid 32×32 de qe por celda. Fuente de energía del mapa.
- `worldgen/EnergyNucleus` — generador de campo energético. Ya tiene `BaseEnergy` + `Faction`.
- `simulation/post.rs::faction_identity_system` — rastrea qué facción controla qué.
- `layers/identity.rs::Faction` — equipo de la entidad.
- GS-5 `VictoryNucleus` — el núcleo objetivo. La victoria ya está definida.
- `world/SpatialIndex` — query_radius para entidades en zona.

**Lo que NO existe:**

1. **Nodos de control.** No hay puntos del mapa que puedan ser "capturados" por equipos.
2. **Drain de campo por control.** Los nodos no drenan la `EnergyFieldGrid` hacia el equipo controlador.
3. **Presencia de facción en zona.** Nada verifica si una facción domina un radio del mapa.
4. **Bonus de intake por control.** El `VictoryNucleus` no se beneficia de nodos controlados.
5. **Snowball de ventaja.** No hay mecanismo por el que controlar más mapa amplifica la ventaja energética.

---

## Objetivo

Hacer que el mapa sea un actor de la partida: nodos que drenan energía del campo y la inyectan al núcleo del equipo que los controla. Control territorial → ventaja energética → snowball hacia victoria. Sin reglas explícitas — es física de flujo.

```
control(node, faction) ↔ count(entities_of_faction, radius) > count(enemies, radius)
qe_rate_to_nucleus = node_drain_rate × control_factor(faction_presence, enemy_presence)
```

---

## Responsabilidades

### GS-6A: Ecuaciones de control de nodo

```rust
// src/blueprint/equations/node_control.rs

/// Factor de control: cuánta ventaja tiene la facción A en este nodo.
/// presence_a, presence_b: magnitud de presencia (suma de extraction_capacity).
/// Retorna [0,1]. 1.0 = control total, 0.5 = contested, 0.0 = sin control.
pub fn control_factor(presence_a: f32, presence_b: f32) -> f32 {
    let total = presence_a + presence_b;
    if total <= 0.0 { return 0.0; }
    (presence_a / total).clamp(0.0, 1.0)
}

/// ¿Cuál facción controla el nodo? None = contested.
/// threshold: control_factor mínimo para considerarse "controlado".
pub fn controlling_faction(
    presence_a: f32,
    presence_b: f32,
    threshold: f32,
) -> Option<bool> {  // Some(true) = A, Some(false) = B, None = contested
    let cf = control_factor(presence_a, presence_b);
    if cf >= threshold { return Some(true); }
    if cf <= (1.0 - threshold) { return Some(false); }
    None
}

/// Tasa de drain del nodo: qe/tick que extrae del EnergyFieldGrid hacia el núcleo.
pub fn node_drain_rate(base_rate: f32, cf: f32, field_qe: f32) -> f32 {
    let available = field_qe.min(base_rate);  // no más de lo que hay en el campo
    available * cf
}

/// Bonus de snowball: amplificador de intake del núcleo por nodos controlados.
/// n_controlled: nodos bajo control. Resultado: factor multiplicativo.
pub fn snowball_intake_factor(n_controlled: u8, scaling: f32) -> f32 {
    1.0 + (n_controlled as f32 * scaling)
}
```

### GS-6B: Componente ControlNode

```rust
// src/worldgen/systems/node_control.rs

/// Nodo de control del mapa. SparseSet — pocos nodos por partida (3-5).
/// Comparte posición con una entidad existente del worldgen (campo energético).
#[derive(Component, Reflect, Debug, Clone)]
#[reflect(Component)]
#[component(storage = "SparseSet")]
pub struct ControlNode {
    pub base_drain_rate: f32,   // qe/tick que puede extraer del campo
    pub capture_radius: f32,    // radio de presencia que cuenta para control
    pub field_cell_idx: u32,    // índice en EnergyFieldGrid (row * cols + col)
}

/// Estado de control actual. Actualizado cada tick.
#[derive(Component, Reflect, Debug, Clone, Default)]
#[reflect(Component)]
pub struct NodeControlState {
    pub controlling_faction: Option<Faction>,   // None = contested
    pub control_factor: f32,                    // [0,1]
    pub tick_drain: f32,                        // qe drenado este tick
}

/// Evento emitido cuando un nodo cambia de manos.
#[derive(Event, Debug, Clone)]
pub struct NodeCapturedEvent {
    pub node: Entity,
    pub new_faction: Option<Faction>,   // None = vuelve a contested
    pub tick_id: u64,
}
```

### GS-6C: Sistema de control

```rust
/// Calcula presencia de facciones en cada nodo y actualiza su estado.
/// Phase::MetabolicLayer, after faction_identity_system, before victory_check_system.
pub fn node_control_update_system(
    mut nodes: Query<(Entity, &ControlNode, &Transform, &mut NodeControlState)>,
    entities: Query<(&Transform, &InferenceProfile, &Faction), With<BehavioralAgent>>,
    spatial: Res<SpatialIndex>,
    mut field: ResMut<EnergyFieldGrid>,
    clock: Res<SimulationClock>,
    config: Res<NodeControlConfig>,
    mut events: EventWriter<NodeCapturedEvent>,
) {
    for (node_entity, node, node_transform, mut state) in &mut nodes {
        let center = [node_transform.translation.x, node_transform.translation.z];
        let nearby = spatial.query_radius(center, node.capture_radius);

        // Presencia como suma de extraction_capacity por facción
        let mut presence_a = 0.0f32;
        let mut presence_b = 0.0f32;
        for &e in &nearby {
            let Ok((_, profile, faction)) = entities.get(e) else { continue; };
            match faction {
                Faction::A => presence_a += profile.extraction_capacity(),
                Faction::B => presence_b += profile.extraction_capacity(),
            }
        }

        let cf = node_control_eq::control_factor(presence_a, presence_b);
        let new_faction = node_control_eq::controlling_faction(
            presence_a, presence_b, config.capture_threshold
        ).map(|a| if a { Faction::A } else { Faction::B });

        // Detectar cambio de control
        if state.controlling_faction != new_faction {
            events.send(NodeCapturedEvent {
                node: node_entity,
                new_faction,
                tick_id: clock.tick_id,
            });
        }

        // Drain del campo hacia el núcleo controlador
        let field_qe = field.cell_qe(node.field_cell_idx);
        let drain = node_control_eq::node_drain_rate(node.base_drain_rate, cf, field_qe);
        field.drain_cell(node.field_cell_idx, drain);

        state.controlling_faction = new_faction;
        state.control_factor = cf;
        state.tick_drain = drain;
    }
}

/// Aplica bonus de snowball al intake de VictoryNucleus según nodos controlados.
/// Phase::MetabolicLayer, after node_control_update_system.
pub fn nucleus_node_bonus_system(
    nodes: Query<&NodeControlState>,
    mut nuclei: Query<(&Faction, &mut AlchemicalEngine), With<VictoryNucleus>>,
    config: Res<NodeControlConfig>,
) {
    for (faction, mut engine) in &mut nuclei {
        let controlled: u8 = nodes.iter()
            .filter(|s| s.controlling_faction == Some(*faction))
            .count() as u8;
        let factor = node_control_eq::snowball_intake_factor(controlled, config.snowball_scaling);
        let boosted = engine.base_intake() * factor;
        if engine.intake() != boosted {
            engine.set_intake(boosted);
        }
    }
}
```

### GS-6D: Constantes y config

```rust
// src/blueprint/constants/node_control.rs

/// Control_factor mínimo para considerar un nodo "capturado" (no contested).
pub const NODE_CAPTURE_THRESHOLD: f32 = 0.65;
/// Tasa base de drain por nodo en qe/tick.
pub const NODE_BASE_DRAIN_RATE: f32 = 5.0;
/// Radio de captura por defecto.
pub const NODE_CAPTURE_RADIUS: f32 = 8.0;
/// Factor de snowball por nodo controlado.
pub const NODE_SNOWBALL_SCALING: f32 = 0.15;

#[derive(Resource, Reflect, Debug, Clone)]
#[reflect(Resource)]
pub struct NodeControlConfig {
    pub capture_threshold: f32,
    pub snowball_scaling: f32,
}
```

---

## Tacticas

- **Nodos como entidades worldgen existentes.** `ControlNode` se agrega a entidades del `EnergyFieldGrid` existente vía `commands.entity(e).insert(ControlNode { ... })` en startup. Sin nuevas entidades.
- **Presencia = suma de extraction_capacity.** No hay contador de "unidades en zona" — la física ya define cuánto influye cada entidad.
- **Drain del campo, no del núcleo.** Los nodos extraen de `EnergyFieldGrid`, no de `BaseEnergy` directamente. El campo alimenta a los jugadores, y los jugadores alimentan al núcleo — chain de dependencia emergente.
- **Snowball moderado.** `NODE_SNOWBALL_SCALING = 0.15` → 3 nodos dan +45% intake. Significativo pero recuperable con rollback agresivo.

---

## NO hace

- No define posiciones de nodos — eso es el diseño de mapa (RON assets).
- No implementa visión de nodos — eso es GS-7 (visual contract).
- No implementa healing de nodos — los nodos no se "reparan".
- No agrega pathfinding hacia nodos — los agentes los visitan si su AI lo decide (GS-3).

---

## Dependencias

- GS-5 — `VictoryNucleus`, `GameOutcome` (victoria ya definida).
- `worldgen/EnergyFieldGrid` — fuente de energía del campo.
- `simulation/post.rs::faction_identity_system` — rastreo de facciones.
- `layers/identity.rs::Faction` — equipo.
- `layers/engine.rs::AlchemicalEngine` — intake del núcleo.
- `layers/inference.rs::InferenceProfile` — extraction_capacity para presencia.
- `world/SpatialIndex` — entidades en radio de captura.

---

## Criterios de aceptacion

### GS-6A (Ecuaciones)
- `control_factor(10.0, 0.0)` → `1.0`.
- `control_factor(0.0, 0.0)` → `0.0`.
- `control_factor(7.0, 3.0)` → `0.7`.
- `controlling_faction(7.0, 3.0, 0.65)` → `Some(true)` (A controla).
- `controlling_faction(6.0, 4.0, 0.65)` → `None` (contested).
- `node_drain_rate(5.0, 1.0, 3.0)` → `3.0` (limitado por campo).
- `node_drain_rate(5.0, 0.5, 100.0)` → `2.5`.
- `snowball_intake_factor(3, 0.15)` → `1.45`.

### GS-6B/C (Sistemas)
- Test (MinimalPlugins + GS-5): nodo con presencia A → `NodeControlState.controlling_faction = Some(A)`.
- Test: nodo sin presencia → `controlling_faction = None`.
- Test: `NodeCapturedEvent` emitido al cambiar control.
- Test: `nucleus_node_bonus_system` → intake del núcleo A aumenta con 1 nodo controlado.
- Test: `EnergyFieldGrid` drenado por nodo activo.

### General
- `cargo test --lib` sin regresión.
- Sin String en componentes.

---

## Referencias

- `src/worldgen/field_grid.rs` — `EnergyFieldGrid`
- `src/worldgen/` — `EnergyNucleus` (entidad base)
- `src/simulation/post.rs` — `faction_identity_system`
- `docs/sprints/GAMEPLAY_SYSTEMS/SPRINT_GS5_VICTORY_NUCLEUS.md` — VictoryNucleus
- Blueprint §6: "Map as Energy Landscape", "Node Control Dynamics"
