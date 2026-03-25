# Blueprint — Game Development Patterns & Skills

**Scope:** Patrones de desarrollo MOBA en Bevy 0.15, orientados a Resonance.
**Audiencia:** Humanos + modelos de IA (Claude, Cursor) que programen sobre este codebase.
**Filosofia:** Interfaces estandar de MOBA (Dota/LoL). Gameplay 100% emergente por composicion energetica.

---

## 1. Principios de diseño

| # | Principio | Implicacion |
|---|-----------|-------------|
| 1 | **Interfaces estandar, gameplay emergente** | Click-to-move, camara orbital, HUD de abilities, minimap = identico a Dota. Pero damage, status, interacciones = emergen de las 14 capas de energia, no de scripts. |
| 2 | **Data-driven first** | Abilities, heroes, mapas, reacciones: definidos en RON. El codigo define sistemas genericos; los datos definen comportamiento especifico. |
| 3 | **Stateless-first** | Funciones puras + Resources. Componentes ECS contienen estado; sistemas lo transforman sin side-effects ocultos. |
| 4 | **Convention over configuration** | Cada modulo sigue la misma estructura (types, constants, functions, systems, plugin). Un modelo de IA que conoce la convencion puede generar modulos nuevos. |
| 5 | **Determinismo absoluto** | `FixedUpdate` a 30Hz. Sin `thread_rng()`, sin floats acumulados. Tick integer → float solo para render. Habilita replay, netcode lockstep, testing. |

---

## 2. Anti-patrones de Bevy a evitar

### 2.1 — Archetype thrashing por status effects

**Problema:** Agregar/quitar componentes como `Stunned`, `Slowed` mueve la entidad entre archetypes (copia todos los datos del hero ~20 componentes).

**En Resonance:** Cada hero tiene 10+ componentes. Un buff que agrega/quita un component causa migracion de archetype costosa.

**Solucion:** `SparseSet` storage para componentes transitorios:
```rust
#[derive(Component)]
#[component(storage = "SparseSet")]
struct Stunned { remaining_ticks: u32 }
```

**Regla:** Todo componente que se agrega y quita mas de 1 vez por segundo → `SparseSet`.

**Estado en Resonance:** No se usa `SparseSet` en ningun componente actualmente. `DespawnOnContact`, `OnContactEffect`, `SpellMarker`, `PlayerControlled` son candidatos.

---

### 2.2 — Sistemas sin run conditions

**Problema:** Sistemas en `Update` corren en todos los estados del juego (menu, loading, gameplay). Desperdician CPU y causan bugs.

**En Resonance:** No hay `GameState` enum. Todos los sistemas corren siempre.

**Solucion:**
```rust
#[derive(States, Default, Clone, PartialEq, Eq, Hash, Debug)]
enum GameState {
    #[default]
    Loading,
    Playing,
    Paused,
}

app.add_systems(
    FixedUpdate,
    gameplay_systems
        .run_if(in_state(GameState::Playing))
        .in_set(Phase::Physics),
);
```

**Estado en Resonance:** `WorldgenState` (Warming/Propagating/Ready) existe pero no es un `States` de Bevy. No hay `run_if(in_state(...))`.

---

### 2.3 — Eventos sin orden explicito

**Problema:** `EventWriter` y `EventReader` sin `.after()` o `.chain()` = el reader puede consumir eventos del frame anterior o del siguiente.

**En Resonance:** Pipeline usa `.chain()` dentro de cada `Phase`, pero los eventos (`CollisionEvent`, `DeathEvent`, etc.) no tienen orden garantizado cross-phase.

**Solucion:** Siempre encadenar productor → consumidor:
```rust
app.add_systems(FixedUpdate, (
    detect_collisions,      // sends CollisionEvent
    resolve_damage,         // reads CollisionEvent, sends DamageEvent
    check_deaths,           // reads DamageEvent, sends DeathEvent
    process_deaths,         // reads DeathEvent
).chain().in_set(Phase::Reactions));
```

---

### 2.4 — Change detection falsa por `&mut`

**Problema:** Acceder `&mut Component` marca el componente como changed aunque no se modifique. Sistemas con `Changed<T>` se disparan innecesariamente.

**Solucion:**
```rust
// MAL — marca changed siempre
fn bad(mut q: Query<&mut BaseEnergy>) {
    for mut e in &mut q { /* lee pero no modifica */ }
}

// BIEN — solo muta si es necesario
fn good(mut q: Query<&mut BaseEnergy>) {
    for mut e in &mut q {
        let new_val = calculate();
        if e.qe() != new_val {
            e.set_qe(new_val);
        }
    }
}
```

**Estado en Resonance:** Solo 2 archivos usan `is_changed()`. Ningun archivo usa `set_if_neq`. Oportunidad de mejora en sistemas de worldgen visual que usan `Changed<T>`.

---

### 2.5 — Bundles manuales (deprecado en 0.15)

**Problema:** Bevy 0.15 depreco Bundles. El patron nuevo es `#[require(...)]`.

**En Resonance:** Se usa `EntityBuilder` (pattern propio). Funciona pero no aprovecha las ventajas de `#[require]` (validacion en compile-time, auto-insertion recursiva).

**Solucion futura:**
```rust
#[derive(Component)]
#[require(Transform, Visibility, BaseEnergy, SpatialVolume, OscillatorySignature, FlowVector)]
struct AlchemicalEntity;

#[derive(Component)]
#[require(AlchemicalEntity, MatterCoherence, AlchemicalEngine, WillActuator, MobaIdentity)]
struct Champion;
```

**Nota:** `EntityBuilder` sigue siendo valido como API de spawn. La migracion a `#[require]` es opcional y puede hacerse incrementalmente.

---

### 2.6 — Entity como ID persistente

**Problema:** `Entity` es un indice de generacion. No es estable entre sesiones, ni entre cliente/servidor.

**En Resonance:** `StructuralLink.target: Entity`, `ResonanceLink.target: Entity`. Funciona en single-player pero no para networking/save.

**Solucion:** Strong IDs:
```rust
#[derive(Component, Copy, Clone, Reflect, Hash, Eq, PartialEq)]
struct ChampionId(u32);

#[derive(Resource)]
struct IdGenerator { next: u32 }
```

---

### 2.7 — Commands overuse

**Problema:** `Commands` es deferred y requiere exclusive World access al flush. Es costoso comparado con mutacion directa via `Query<&mut T>`.

**Regla:** Usar `Commands` solo para cambios estructurales (spawn, despawn, add/remove components). Para mutaciones de datos, usar `Query<&mut T>` directo.

**En Resonance:** Se usa correctamente en general. Los sistemas de simulacion mutan via queries.

---

## 3. Patrones MOBA esenciales

### 3.1 — Sistema de habilidades (GAS-inspired)

El Gameplay Ability System de Unreal descompone abilities en 3 partes: Cost, Cooldown, Effect. Adaptado para ECS:

```rust
struct AbilitySlot {
    id: AbilityId,
    cooldown_remaining: f32,
    cooldown_total: f32,
    cost_qe: f32,
    targeting: TargetingMode,
    state: AbilityState,
}

enum TargetingMode {
    NoTarget,                          // self-cast (Ember Shield)
    PointTarget { range: f32 },        // skillshot (Fireball)
    UnitTarget { range: f32 },         // targeted (Hex)
    DirectionTarget { range: f32 },    // line skillshot (Lina stun)
    AreaTarget { radius: f32, range: f32 }, // AoE (Ravage)
}

enum AbilityState {
    Ready,
    Casting { timer: f32 },
    OnCooldown { remaining: f32 },
    Disabled,
}
```

**Mapeo a Resonance:** `AbilitySlot` ya existe en `layers/will.rs`. `AbilityOutput` ya tiene variantes (Projectile, SelfBuff, Zone, etc.). Lo que falta: `TargetingMode`, `AbilityState`, cooldown system.

### 3.2 — Status effects como SparseSet

```rust
#[derive(Component)]
#[component(storage = "SparseSet")]
struct Stunned { remaining_ticks: u32 }

#[derive(Component)]
#[component(storage = "SparseSet")]
struct Slowed { factor: f32, remaining_ticks: u32 }

#[derive(Component)]
#[component(storage = "SparseSet")]
struct Silenced { remaining_ticks: u32 }

// Sistema generico para tick-down
fn tick_timed_effect<T: Component + TimedEffect>(
    mut commands: Commands,
    mut query: Query<(Entity, &mut T)>,
) {
    for (entity, mut effect) in &mut query {
        if effect.tick_down() {
            commands.entity(entity).remove::<T>();
        }
    }
}
```

**Mapeo a Resonance:** `ResonanceLink` ya actua como buff/debuff. Pero no es SparseSet y se modela como entidad-efecto separada en vez de componente directo. Ambas aproximaciones son validas; SparseSet es mas eficiente para efectos frecuentes.

### 3.3 — Click-to-move + Pathfinding

| Capa | Crate | Uso |
|------|-------|-----|
| Click → ground position | Bevy built-in (ray-plane) | Ya implementado en `click_to_move/` |
| Pathfinding individual | `oxidized_navigation` o `vleue_navigator` | NavMesh para heroes |
| Pathfinding masivo | `bevy_flowfield_tiles_plugin` | Flow fields para creeps/minions |
| Avoidance local | Boids / RVO2 | Evitar overlap entre unidades |

**En Resonance:** Click-to-move basico existe. No hay pathfinding (los heroes caminan en linea recta). No hay avoidance.

### 3.4 — Camara MOBA

| Comportamiento | Implementacion | Estado en Resonance |
|---------------|----------------|---------------------|
| Free pan (WASD/edge scroll) | Mover `look_at` en plano XZ | Parcial (orbital, no free pan) |
| Lock to hero (Space/Y) | `camera.look_at = hero.position` | Implementado (CameraRigTarget) |
| Minimap click | Teleport `look_at` a posicion | No implementado |
| Zoom (scroll wheel) | Ajustar height manteniendo angulo | No implementado |
| Bounds clamping | Clamp `look_at` al area del mapa | No implementado |
| Angulo fijo | ~55-60 grados desde horizontal | Implementado (pitch range) |

**Nota:** La camara actual es orbital-follow (tipo third-person). Para MOBA estandar (Dota/LoL) debe ser **free-roaming con lock toggle**.

### 3.5 — Fog of War

**Arquitectura server-authoritative:**
```rust
#[derive(Resource)]
struct FogOfWarGrid {
    width: u32,
    height: u32,
    cells: Vec<i32>,  // reference count por team
    cell_size: f32,
}

#[derive(Component)]
struct VisionProvider { radius: f32 }

#[derive(Component)]
struct VisionBlocker;  // muros, terreno alto
```

**Pipeline:**
1. Cada tick, para cada entidad con `VisionProvider`, flood-fill celdas en rango.
2. Reference counting: increment al entrar, decrement al salir. Solo procesar si la entidad cruzo una celda.
3. Server no envia datos de entidades invisibles al cliente.
4. Cliente renderiza: negro (unexplored), gris (explored-not-visible), claro (visible).

**En Resonance:** No implementado. `world/perception.rs` existe pero es sparse.

### 3.6 — Networking (futuro)

| Modelo | Crate Bevy | Para que |
|--------|-----------|----------|
| Server-authoritative | `lightyear` | MOBA standard (info oculta, anti-cheat) |
| Rollback | `bevy_ggrs` | Fighting games (pocos jugadores) |
| Lockstep | Manual | RTS (determinismo total requerido) |

**Prerequisitos para networking:**
1. Determinismo en `FixedUpdate` (Resonance ya lo tiene).
2. Strong IDs para entidades (Resonance NO lo tiene — usa `Entity`).
3. Separacion Input → Simulation → Render (Resonance ya lo tiene).
4. Fog of War (Resonance NO lo tiene).

---

## 4. Patrones de Bevy 0.15 a adoptar

### 4.1 — Required Components

```rust
// Define jerarquia de componentes
#[derive(Component, Default)]
#[require(Transform, Visibility, BaseEnergy, SpatialVolume)]
struct AlchemicalBase;

#[derive(Component)]
#[require(AlchemicalBase, OscillatorySignature, FlowVector)]
struct WaveEntity;

#[derive(Component)]
#[require(WaveEntity, MatterCoherence, AlchemicalEngine, WillActuator)]
struct MobileEntity;

#[derive(Component)]
#[require(MobileEntity, MobaIdentity)]
struct Champion;
```

**Beneficio:** `commands.spawn(Champion)` inserta automaticamente las 10+ capas requeridas. Imposible olvidar un componente.

### 4.2 — Observers para lifecycle

```rust
// Trigger inmediato cuando un champion muere
commands.spawn(Champion)
    .observe(|trigger: Trigger<DeathEvent>, world: &mut World| {
        // Spawn death VFX, start respawn timer
    });

// Global observer para cleanup
app.add_observer(on_death_cleanup);

fn on_death_cleanup(
    trigger: Trigger<OnRemove, BaseEnergy>,
    query: Query<&Name>,
) {
    if let Ok(name) = query.get(trigger.entity()) {
        info!("{name} lost all energy");
    }
}
```

**Cuando usar:** Lifecycle hooks (spawn, death, component add/remove). NO para eventos de alta frecuencia (damage, collision).

### 4.3 — Game States con SubStates

```rust
#[derive(States, Default, Clone, PartialEq, Eq, Hash, Debug)]
enum GameState {
    #[default]
    Loading,
    MainMenu,
    HeroSelect,
    Playing,
    PostGame,
}

#[derive(SubStates, Clone, PartialEq, Eq, Hash, Debug)]
#[source(GameState = GameState::Playing)]
enum PlayState {
    Warmup,      // worldgen propagating
    Active,      // gameplay
    Paused,
}
```

**Mapeo a Resonance:** `WorldgenState` (Warming/Propagating/Ready) se mapea directamente a `PlayState` sub-states.

### 4.4 — StateScoped para cleanup

```rust
commands.spawn((
    Name::new("Minion_Wave_3"),
    StateScoped(GameState::Playing),  // auto-despawn al salir de Playing
    AlchemicalBase::default(),
));
```

**Beneficio:** Zero cleanup code manual. Al cambiar de estado, Bevy despawnea automaticamente.

### 4.5 — Storage strategy por tipo

| Componente Resonance | Storage recomendado | Razon |
|---------------------|--------------------|----|
| BaseEnergy | Table | Iterado cada tick, nunca removido |
| SpatialVolume | Table | Iterado cada tick |
| OscillatorySignature | Table | Iterado cada tick |
| FlowVector | Table | Iterado cada tick |
| MatterCoherence | Table | Iterado cada tick |
| AlchemicalEngine | Table | Iterado cada tick |
| AmbientPressure | Table | Presente toda la vida |
| WillActuator | Table | Presente toda la vida en heroes |
| AlchemicalInjector | Table | Presente en projectiles/injectors toda su vida |
| MobaIdentity | Table | Nunca removido |
| ResonanceLink | Table | Presente toda la vida de la entidad-efecto |
| TensionField | Table | Presente toda la vida |
| Homeostasis | Table | Presente toda la vida |
| StructuralLink | Table | Presente toda la vida |
| **DespawnOnContact** | **SparseSet** | Agrega/quita en contacto |
| **SpellMarker** | **SparseSet** | Tag transitorio |
| **PlayerControlled** | **SparseSet** | Puede cambiar (spec mode) |
| **OnContactEffect** | **SparseSet** | Consumido en contacto |
| **Stunned/Slowed/etc** | **SparseSet** | Status effects transitorios |

---

## 5. Templates para IA

### 5.1 — Template: Nuevo sistema de gameplay

```rust
//! # [Nombre del sistema]
//!
//! [Descripcion de 1 linea].
//! Corre en `Phase::[fase]`, despues de `[dependencia]`.
//!
//! ## Componentes que lee
//! - `ComponentA` — [que significa]
//!
//! ## Componentes que muta
//! - `ComponentB` — [que campo y por que]
//!
//! ## Eventos que emite
//! - `MyEvent` — [cuando se emite]

use bevy::prelude::*;
use crate::layers::*;

/// [Documentacion del sistema — que hace, cuando corre, que invariantes mantiene].
pub fn my_system(
    // Queries: minimas, con With/Without filters
    mut query: Query<(&ComponentA, &mut ComponentB), Without<Dead>>,
    // Events: writer si produce, reader si consume
    mut events: EventWriter<MyEvent>,
    // Resources: Res para lectura, ResMut solo si muta
    config: Res<MyConfig>,
) {
    for (a, mut b) in &mut query {
        let new_value = pure_calculation(a, &config);
        if b.value != new_value {  // guard change detection
            b.value = new_value;
        }
        if some_condition {
            events.send(MyEvent { /* ... */ });
        }
    }
}

/// Funcion pura: testeable sin Bevy.
fn pure_calculation(a: &ComponentA, config: &MyConfig) -> f32 {
    // logica sin side-effects
    0.0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pure_calculation_baseline() {
        let a = ComponentA::default();
        let config = MyConfig::default();
        assert_eq!(pure_calculation(&a, &config), 0.0);
    }
}
```

### 5.2 — Template: Nuevo componente

```rust
use bevy::prelude::*;

/// Capa [N]: [Nombre] — [descripcion de 1 linea].
///
/// [Que representa en el modelo de energia].
/// [Invariantes: rangos validos, relacion con otras capas].
#[derive(Component, Reflect, Debug, Clone)]
#[reflect(Component)]
pub struct MyComponent {
    field_a: f32,
    field_b: f32,
}

impl MyComponent {
    pub fn new(field_a: f32, field_b: f32) -> Self {
        Self {
            field_a: field_a.max(0.0),  // clamp invariante
            field_b,
        }
    }

    pub fn field_a(&self) -> f32 { self.field_a }

    pub fn set_field_a(&mut self, val: f32) {
        self.field_a = val.max(0.0);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_clamps_negative() {
        let c = MyComponent::new(-5.0, 1.0);
        assert_eq!(c.field_a(), 0.0);
    }
}
```

### 5.3 — Template: Nuevo plugin

```rust
use bevy::prelude::*;

pub struct MyPlugin;

impl Plugin for MyPlugin {
    fn build(&self, app: &mut App) {
        // 1. Registrar tipos
        app.register_type::<MyComponent>();

        // 2. Registrar eventos
        app.add_event::<MyEvent>();

        // 3. Registrar resources
        app.init_resource::<MyConfig>();

        // 4. Sistemas con state guard + set
        app.add_systems(
            FixedUpdate,
            (system_a, system_b)
                .chain()
                .in_set(Phase::PrePhysics),
        );
    }
}
```

### 5.4 — Template: Entidad desde RON

```ron
// assets/heroes/fire_mage.hero.ron
(
    name: "FireMage",
    element: "Ignis",
    base_qe: 500.0,
    radius: 0.8,
    matter: (state: Solid, bond_energy: 2000.0, conductivity: 0.6),
    engine: (max_buffer: 1500.0, input_valve: 8.0, output_valve: 80.0),
    abilities: [
        (
            name: "Fireball",
            cost_qe: 50.0,
            cooldown: 8.0,
            targeting: PointTarget(range: 30.0),
            output: Projectile(
                speed: 15.0,
                radius: 0.5,
                forced_freq: 450.0,
                despawn_on_contact: true,
            ),
        ),
    ],
)
```

### 5.5 — Template: Spawn de entidad MOBA

```rust
pub fn spawn_champion(
    commands: &mut Commands,
    def: &ChampionDef,
    faction: Faction,
    pos: Vec2,
    layout: &SimWorldTransformParams,
) -> Entity {
    let entity = EntityBuilder::new()
        .named(&def.name)
        .at(pos)
        .energy(def.base_qe)
        .volume(def.radius)
        .wave(ElementId::from_name(&def.element))
        .flow(Vec2::ZERO, def.dissipation)
        .matter(def.matter.state, def.matter.bond_energy, def.matter.conductivity)
        .motor(
            def.engine.max_buffer,
            def.engine.input_valve,
            def.engine.output_valve,
            def.engine.max_buffer * 0.5,
        )
        .will_default()
        .identity(faction, vec![RelationalTag::Hero], def.critical_multiplier)
        .sim_world_layout(layout)
        .spawn(commands);

    // Grimoire con abilities del RON
    let grimoire = Grimoire::from_defs(&def.abilities);
    commands.entity(entity).insert(grimoire);

    entity
}
```

---

## 6. Checklist de auditoria Resonance vs patrones

| Patron | Estado actual | Prioridad | Sprint sugerido |
|--------|-------------|-----------|----------------|
| `#[require(...)]` para componentes | No usado | Media | Futuro (EntityBuilder funciona) |
| `SparseSet` storage para status | No usado | **Alta** | Proximo sprint |
| `GameState` enum + `run_if` | No existe | **Alta** | Proximo sprint |
| `StateScoped` cleanup | No existe | Media | Con GameState |
| `SubStates` para PlayState | No existe | Media | Con GameState |
| Observers para lifecycle | No usado | Media | Cuando se implemente death/respawn |
| Change detection guards (`set_if_neq`) | No usado | Media | Gradual |
| Event ordering explicito | Parcial (chain dentro de Phase) | Media | Refinar |
| Strong IDs (no Entity) | No existe | Baja (single-player ok) | Pre-networking |
| Pathfinding (NavMesh/flowfield) | No existe | **Alta** | Proximo sprint |
| Camara MOBA (free pan + lock) | Solo orbital follow | **Alta** | Proximo sprint |
| Fog of War | No existe | Baja | Post-networking |
| Cooldown system | No existe | **Alta** | Con abilities |
| HUD de abilities | No existe | **Alta** | Con abilities |
| Targeting system | No existe | **Alta** | Con abilities |
| Minimap | No existe | Media | Post-camara |

---

## 7. Crates recomendados

| Crate | Uso | Notas |
|-------|-----|-------|
| `oxidized_navigation` | NavMesh runtime | Para pathfinding de heroes |
| `bevy_flowfield_tiles_plugin` | Flow fields | Para creeps/minions |
| `lightyear` | Networking | Server-authoritative, client prediction |
| `bevy_egui` | Debug UI / HUD rapido | Inspector + game UI |
| `bevy_inspector_egui` | Inspector de entidades | Debug en desarrollo |
| `leafwing-input-manager` | Input mapping | Keybinds configurables |
| `bevy_mod_debugdump` | Schedule visualization | Ver orden de sistemas |

---

## 8. Decision matrix: Evento vs Resource vs Observer

| Comunicacion | Mecanismo | Por que |
|-------------|-----------|--------|
| Damage dealt | `Event<DamageEvent>` | Alta frecuencia, multiples consumidores |
| Champion muerto | `Event<DeathEvent>` + Observer `OnRemove<BaseEnergy>` | Trigger respawn + VFX |
| Ability cast | `Event<AbilityCastEvent>` | Animation + sound + network |
| Gold/score cambio | `ResMut<Scoreboard>` + `Changed<Scoreboard>` | Singleton global |
| Game timer | `Res<GameClock>` | Singleton, read-only para sistemas |
| Componente agregado (buff) | Observer `OnAdd<StunEffect>` | VFX/SFX inmediato |
| Componente removido (buff expire) | Observer `OnRemove<StunEffect>` | Cleanup VFX |
| Fase del worldgen | `Res<WorldgenState>` | Estado global, leido frecuentemente |

---

## 9. Performance quick reference

| Operacion | Costo | Nota |
|-----------|-------|------|
| Query iteration | O(n matched) | Cache-friendly dentro del archetype |
| Entity spawn | O(1) amortizado | Usar batch si >10 por frame |
| Entity despawn | O(1) | Swap-remove del archetype |
| Component add/remove (Table) | O(components) | Copia TODOS los datos — usar SparseSet para transitorios |
| Component add/remove (SparseSet) | O(1) | Sin migracion de archetype |
| Commands flush | O(pending) | Requiere exclusive World access |
| Changed<T> filter | O(archetypes) | Skippea archetypes enteros sin cambios |
| Spatial query (brute) | O(n^2) | Actual en Resonance para catalisis |
| Spatial query (grid) | O(n + k) | SpatialIndex ya implementado |

---

## 10. Fuentes

- [tbillington/bevy_best_practices](https://github.com/tbillington/bevy_best_practices)
- [Bevy 0.15 Release Notes](https://bevy.org/news/bevy-0-15/)
- [Unofficial Bevy Cheat Book](https://bevy-cheatbook.github.io/)
- [Tainted Coders — Bevy Patterns](https://taintedcoders.com/bevy/ecs)
- [GAS Documentation (Unreal)](https://github.com/tranek/GASDocumentation)
- [Riot Games — Determinism in LoL](https://technology.riotgames.com/news/determinism-league-legends-unified-clock)
- [Riot Games — Fog of War](https://technology.riotgames.com/news/story-fog-and-war)
- [Digital Extinction (Bevy RTS)](https://github.com/DigitalExtinction/Game)
- [Game Programming Patterns — Component](https://gameprogrammingpatterns.com/component.html)
- [lightyear (networking)](https://github.com/cBournhonesque/lightyear)
- [oxidized_navigation](https://github.com/TheGrimsey/oxidized_navigation)
- [Ariel Coppes — ECS Design Decisions](https://arielcoppes.dev/2023/07/13/design-decisions-when-building-games-using-ecs.html)
- [DeepWiki — Bevy ECS Architecture](https://deepwiki.com/bevyengine/bevy/2-entity-component-system-(ecs))
