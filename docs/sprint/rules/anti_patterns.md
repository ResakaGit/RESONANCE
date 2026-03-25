# Anti-Patterns: Zero Tolerance

Reglas de evitación para todos los systems del sprint. Cada anti-patrón incluye el **por qué** y la **alternativa correcta**.

---

## AP-1: God System (>5 component types en un Query)

**Síntoma**: `Query<(&A, &B, &C, &D, &E, &F, &mut G)>`

**Por qué es malo**: Viola Single Responsibility. Acopla 7+ tipos. Imposible testear unitariamente. Bloquea paralelismo del schedule (Bevy no puede ejecutar en paralelo si dos systems comparten &mut).

**Alternativa**: Descomponer en cadena de systems más pequeños conectados por `.chain()` o eventos. Cada system lee ≤3 tipos y escribe ≤1.

```rust
// MAL
fn god_system(q: Query<(&Energy, &Volume, &Flow, &Matter, &Engine, &Will, &mut Identity)>) { ... }

// BIEN
fn read_energy_flow(q: Query<(&Energy, &Flow)>, mut cache: ResMut<EnergyFlowCache>) { ... }
fn apply_identity_delta(mut q: Query<&mut Identity>, cache: Res<EnergyFlowCache>) { ... }
// Registrar: read_energy_flow.pipe(apply_identity_delta).in_set(Phase::X)
```

---

## AP-2: Inline Math en Systems

**Síntoma**: `let result = source.qe * 0.5 * (1.0 - target.frequency / 450.0);`

**Por qué es malo**: Fórmula no testeable aisladamente. Constantes mágicas. Difícil auditar correctitud.

**Alternativa**: Función pura en `blueprint/equations/{dominio}/mod.rs` + constante en `blueprint/constants/{dominio}.rs`.

```rust
// MAL (en system)
let cost = mass * velocity.length_squared() * 0.5 * terrain_factor;

// BIEN (en equations/locomotion/mod.rs)
pub fn locomotion_energy_cost(mass: f32, speed: f32, terrain_factor: f32) -> f32 {
    mass * speed * speed * LOCOMOTION_KINETIC_FACTOR * terrain_factor
}
// En constants/locomotion.rs
pub const LOCOMOTION_KINETIC_FACTOR: f32 = 0.5;
```

---

## AP-3: Estado Mutable Compartido fuera de Resources

**Síntoma**: `static mut`, `lazy_static! { Mutex }`, `Arc<Mutex<T>>`

**Por qué es malo**: Data race. Bevy ya gestiona concurrencia vía schedule. Mutex destruye la ventaja del ECS.

**Alternativa**: `Resource` para estado global, `Local<T>` para estado por-system.

---

## AP-4: Allocation en Hot Path

**Síntoma**: `Vec::new()` o `String::from()` dentro de un `for` que itera entidades.

**Por qué es malo**: Heap allocation por frame × N entidades = GC pressure + cache misses.

**Alternativa**: Pre-allocar en `Local<Vec<T>>` y hacer `.clear()` al inicio del frame. O usar arrays fijos `[T; N]`.

```rust
// MAL
for entity in &query {
    let mut neighbors = Vec::new();  // Alloc por entidad por frame
    spatial.query_radius(pos, radius, &mut neighbors);
}

// BIEN
fn my_system(mut scratch: Local<Vec<Entity>>, ...) {
    for entity in &query {
        scratch.clear();
        spatial.query_radius(pos, radius, &mut scratch);
    }
}
```

---

## AP-5: Polling/Busy-Wait en Systems

**Síntoma**: System que no hace nada el 99% de frames pero siempre corre.

**Por qué es malo**: Waste de CPU. Contaminación de change detection.

**Alternativa**: Run conditions (`run_if`), change detection (`Changed<T>`, `Added<T>`), o eventos.

```rust
// MAL
fn check_homeostasis(q: Query<&Homeostasis>) {
    for h in &q {
        if !h.enabled { continue; }  // Skip 90% de entidades
        // ...
    }
}

// BIEN
fn homeostasis_system(q: Query<&Homeostasis, (Changed<BaseEnergy>, With<Homeostasis>)>) {
    // Solo corre en entidades cuya energía cambió Y tienen Homeostasis
}
// O con run_if:
app.add_systems(FixedUpdate, homeostasis_system
    .in_set(Phase::ChemicalLayer)
    .run_if(any_with_component::<Homeostasis>));
```

---

## AP-6: ResMut Cuando Res Basta

**Síntoma**: `config: ResMut<MyConfig>` en system que solo lee.

**Por qué es malo**: Write lock impide paralelismo. Bevy schedule serializa todos los systems con `ResMut` al mismo Resource.

**Alternativa**: Usar `Res<T>` siempre que no se escriba.

---

## AP-7: Derived Values Almacenados como Components

**Síntoma**: `Density` component que es `qe / volume`. `Temperature` component.

**Por qué es malo**: Se desincroniza del source of truth. Double bookkeeping. Bug factory.

**Alternativa**: Calcular en point-of-use via ecuación pura.

```rust
// MAL
#[derive(Component)]
struct Density(f32);  // ¿Quién lo actualiza? ¿Cuándo? ¿Si cambia qe? ¿Si cambia radius?

// BIEN
let density = equations::density(energy.qe(), volume.radius());
```

**Excepción**: Valores caros de computar que no cambian cada frame (ej: `MetabolicGraph`, `InferredAlbedo`). Estos se recalculan con change detection y se marcan SparseSet.

---

## AP-8: Eventos Desordenados

**Síntoma**: Productor y consumidor en el mismo Phase sin `.chain()` ni `.before()`/`.after()`.

**Por qué es malo**: Non-deterministic. Bevy puede ejecutar en cualquier orden dentro de un set.

**Alternativa**: Siempre encadenar productor → consumidor.

```rust
// MAL
app.add_systems(FixedUpdate, (
    emit_hunger_event,
    handle_hunger_event,  // ¿Se ejecuta antes o después?
).in_set(Phase::MetabolicLayer));

// BIEN
app.add_systems(FixedUpdate, (
    emit_hunger_event,
    handle_hunger_event,
).chain().in_set(Phase::MetabolicLayer));
```

---

## AP-9: Component con >4 Fields

**Síntoma**: `struct BehaviorState { intent, target, timer, cooldown, memory, last_seen, threat_level }`

**Por qué es malo**: Viola DOD. Componentes grandes = cache misses (todo el struct se carga cuando solo necesitas 1 field). Imposible componer ortogonalmente.

**Alternativa**: Split en componentes ortogonales de ≤4 fields.

```rust
// MAL
struct BehaviorState { intent, target, timer, cooldown, memory, last_seen, threat_level }

// BIEN
struct BehaviorIntent { intent: Intent, target: Option<Entity> }                    // 2 fields
struct BehaviorCooldown { timer: f32, cooldown: f32 }                               // 2 fields
struct BehaviorMemory { last_seen_entity: Option<Entity>, threat_level: f32 }       // 2 fields
```

---

## AP-10: `unwrap()`/`expect()`/`panic!()` en Systems

**Síntoma**: `let target = query.get(entity).unwrap();`

**Por qué es malo**: Entity puede haber sido despawned entre queries. Panic crashea el frame completo.

**Alternativa**: `let-else` o `if-let`.

```rust
// MAL
let target = query.get(entity).unwrap();

// BIEN
let Ok(target) = query.get(entity) else { continue; };
// o
let Some(target) = query.get(entity).ok() else { return; };
```

---

## AP-11: HashMap en Hot Path

**Síntoma**: `HashMap<Entity, f32>` actualizado por frame en loop de entidades.

**Por qué es malo**: Hash + allocation + cache-unfriendly. Entity ya es un índice.

**Alternativa**: `SpatialIndex` existente (grid-based), sorted `Vec`, o Bevy queries directas.

---

## AP-12: Box<dyn Trait> para Game Logic

**Síntoma**: `Box<dyn BehaviorStrategy>` en component.

**Por qué es malo**: Heap indirection. vtable lookup. No serializable. Incompatible con Reflect.

**Alternativa**: Enum cerrado con match exhaustivo.

```rust
// MAL
struct Behavior(Box<dyn BehaviorStrategy>);

// BIEN
enum BehaviorMode {
    Idle,
    Forage { target_pos: Vec2 },
    Flee { threat: Entity },
    Hunt { prey: Entity },
    Reproduce,
}
```

---

## AP-13: Tests que Dependen del Schedule Order

**Síntoma**: Test que hace `app.update()` N veces y espera orden específico.

**Por qué es malo**: Frágil ante reorganización de pipeline. El test debería testear la transformación, no el schedule.

**Alternativa**: Testear ecuaciones puras aisladamente. Para systems, hacer 1 update y verificar delta.

---

## AP-14: `f64` en Game Math

**Síntoma**: `let result: f64 = ...`

**Por qué es malo**: Desperdicio de ancho de banda. GPU no lo soporta nativo. Inconsistencia con `bevy::math` (glam usa f32).

**Alternativa**: `f32` siempre. Si necesitas precision, reformula el algoritmo.

---

## AP-15: Feature Flags / Backwards Compatibility Shims

**Síntoma**: `#[cfg(feature = "new_behavior")]` o `if config.use_v2 { ... } else { ... }`

**Por qué es malo**: Bifurcación de código. Doble testing. Complexity debt.

**Alternativa**: Cambiar el código directamente. Si necesitas rollback, git revert.
