# Sprint G11 — Strong IDs (Entity → Newtype)

**Tipo:** Refactor — preparacion para networking/save.
**Riesgo:** MEDIO — toca componentes que usan `Entity` como referencia.
**Onda:** B — Independiente. Pre-requisito para networking y G12 (Fog of War).
**Estado:** Pendiente

## Objetivo

Reemplazar usos de `Entity` como ID persistente por newtypes fuertes. `Entity` es un indice de generacion que no es estable entre sesiones ni entre cliente/servidor. Para networking, replays y save/load se necesitan IDs deterministas.

## Estado actual en Resonance

Componentes que usan `Entity` como referencia:
- `ResonanceLink.target: Entity` — referencia al target del buff
- `StructuralLink.target: Entity` — referencia al otro extremo del spring
- `CameraRigTarget` — referencia al hero seguido
- Eventos que incluyen `Entity` (DeathEvent, CollisionEvent, etc.)

En single-player funciona. En networking: el server genera `Entity(42, gen 3)`, el cliente genera `Entity(42, gen 7)` para otra entidad → collision de IDs.

## Responsabilidades

### Paso 1 — Definir ID types

Crear `src/blueprint/ids.rs`:

```rust
/// ID persistente para champions. Estable entre sesiones y cliente/servidor.
#[derive(Component, Copy, Clone, Reflect, Hash, Eq, PartialEq, Debug)]
pub struct ChampionId(pub u32);

/// ID persistente para entidades del mundo (crystals, biomes, structures).
#[derive(Component, Copy, Clone, Reflect, Hash, Eq, PartialEq, Debug)]
pub struct WorldEntityId(pub u32);

/// ID persistente para projectiles y efectos.
#[derive(Component, Copy, Clone, Reflect, Hash, Eq, PartialEq, Debug)]
pub struct EffectId(pub u32);

/// Generador de IDs determinista.
#[derive(Resource)]
pub struct IdGenerator {
    next_champion: u32,
    next_world: u32,
    next_effect: u32,
}

impl IdGenerator {
    pub fn next_champion(&mut self) -> ChampionId {
        let id = ChampionId(self.next_champion);
        self.next_champion += 1;
        id
    }
    // ... similar para world y effect
}
```

### Paso 2 — Agregar IDs al spawn

En `archetypes.rs`, agregar ID al spawn de cada entidad:

```rust
pub fn spawn_hero(..., id_gen: &mut IdGenerator) -> Entity {
    let id = id_gen.next_champion();
    EntityBuilder::new()
        // ... capas existentes ...
        .spawn(commands)
    // Luego:
    commands.entity(entity).insert(id);
    entity
}
```

### Paso 3 — Lookup Resource

```rust
#[derive(Resource, Default)]
pub struct EntityLookup {
    champions: HashMap<ChampionId, Entity>,
    world_entities: HashMap<WorldEntityId, Entity>,
    effects: HashMap<EffectId, Entity>,
}
```

Actualizar con observers (`OnAdd<ChampionId>` → insert en lookup, `OnRemove` → remove).

### Paso 4 — Migrar ResonanceLink y StructuralLink (FUTURO)

**No en este sprint MVP.** En networking sprint, cambiar:
```rust
// Actual
pub struct ResonanceLink { target: Entity, ... }

// Futuro (networking)
pub struct ResonanceLink { target: ChampionId, ... }
```

Esto requiere resolver IDs en cada sistema que usa el target. Es costoso y solo necesario para networking.

### Paso 5 — IdGenerator en spawns

Asegurar que `IdGenerator` se pasa como `ResMut` a sistemas de spawn:
```rust
fn spawn_system(mut commands: Commands, mut id_gen: ResMut<IdGenerator>) {
    let id = id_gen.next_champion();
    // ...
}
```

## Tacticas

- **IDs son additive.** No reemplazar `Entity` en componentes existentes (aun). Solo agregar `ChampionId` como componente extra.
- **Determinismo.** `IdGenerator` es un contador secuencial. Si el orden de spawn es determinista (FixedUpdate), los IDs son deterministas.
- **No migrar Entity en L10/L13 ahora.** `ResonanceLink.target: Entity` y `StructuralLink.target: Entity` siguen usando Entity en este sprint. La migracion a strong IDs es para networking sprint.

## NO hace

- No cambia `ResonanceLink.target` de Entity a strong ID (post-networking).
- No cambia `StructuralLink.target` de Entity a strong ID (post-networking).
- No implementa serialization/save.
- No implementa networking.
- No modifica sistemas existentes (solo agrega componente ID).

## Criterio de aceptacion

- [ ] `ChampionId`, `WorldEntityId`, `EffectId` newtypes existen
- [ ] `IdGenerator` Resource existe y es determinista
- [ ] Todos los champions se spawnean con `ChampionId`
- [ ] `EntityLookup` Resource permite resolver ID → Entity
- [ ] `cargo check` pasa
- [ ] `cargo test` — tests para IdGenerator determinismo, EntityLookup consistency

## Esfuerzo estimado

~2-3 horas. Mayormente mecanico: agregar componente ID a cada funcion de spawn.
