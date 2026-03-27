# Blueprint: Entidades y Arquetipos (`entities/`)

Encapsula la construccion de entidades ECS desde presets coherentes.
Composicion declarativa via `EntityBuilder` + funciones `spawn_*` por arquetipo.
Solo define estado inicial — la dinamica temporal vive en `simulation/`.

## Jerarquia de arquetipos

```mermaid
flowchart TD
    P["Particle<br/>L0+L1+L2"]
    S["Stone<br/>L0+L1+L4"]
    D["Dummy<br/>L0+L1+L4"]

    P --> PR["Projectile<br/>+L3+L8"]
    S --> CR["Crystal<br/>+L2+L3+L5"]

    C["Celula<br/>L0-L5+L9+L12"]
    C --> V["Virus<br/>Celula minimal"]
    C --> FL["Flora<br/>+L13 structural"]

    FL --> R["Rosa"]
    FL --> O["Oak"]
    FL --> M["Moss"]

    C --> PL["Planta<br/>+GrowthBudget+Organs"]

    PL --> MO["MorphoOrganism<br/>+MetabolicGraph+InferenceProfile"]
    MO --> AQ["Aquatic organism"]
    MO --> DE["Desert organism"]
    MO --> FO["Forest organism"]

    AN["Animal<br/>L0-L7+L6+L9+L12<br/>+BehavioralAgent<br/>+ConstructalBodyPlan"]
    AN --> H["Hero<br/>+L8+L10+L11<br/>+Grimoire+Champion"]

    H --> FM["FireMage<br/>Ignis 450Hz"]
    H --> EW["EarthWarrior<br/>Terra 75Hz"]
    H --> PA["PlantAssassin<br/>Umbra 20Hz"]
    H --> LH["LightHealer<br/>Lux 1000Hz"]
    H --> WS["WindShooter<br/>Ventus 700Hz"]
    H --> WT["WaterTank<br/>Aqua 250Hz"]

    B["Biome<br/>L0+L1+L4+L6"]
    LK["LavaKnight<br/>L0-L8"]
    EF["Effect<br/>L0+L3+L10"]

    PO["Pool<br/>EnergyPool+L6"]
    PO --> CP["Competitor<br/>PoolParentLink"]
    PO --> SP["SubPool"]

    style H fill:#e74c3c,color:#fff
    style MO fill:#27ae60,color:#fff
    style PO fill:#3498db,color:#fff
```

## Funciones spawn por modulo

| Modulo | Funcion | Entidad | Capas principales |
|--------|---------|---------|-------------------|
| **catalog.rs** | `spawn_celula` | Celula | L0-L5, L9, L12 |
| | `spawn_virus` | Virus | L0-L4, L9 |
| | `spawn_planta` | Planta | L0-L5, L9, L12, L13 |
| | `spawn_animal` | Animal | L0-L7, L6, L9, L12 + constructal body plan |
| **flora.rs** | `spawn_rosa` | Rosa | Flora + GF1 shape |
| | `spawn_oak` | Oak | Flora + high bond_energy |
| | `spawn_moss` | Moss | Flora + low energy |
| **morphogenesis.rs** | `spawn_aquatic_organism` | Aquatic | MorphoOrganism + Aqua band |
| | `spawn_desert_organism` | Desert | MorphoOrganism + Ignis band |
| | `spawn_forest_organism` | Forest | MorphoOrganism + Terra band |
| **heroes.rs** | `spawn_hero(HeroClass)` | Hero | L0-L9 + L11, L12 |
| **world_entities.rs** | `spawn_effect` | Effect | L0, L3, L10 |
| | `spawn_dummy` | Dummy | L0, L1, L4 |
| | `spawn_projectile` | Projectile | L0-L3, L8 |
| | `spawn_crystal` | Crystal | L0-L5 |
| | `spawn_biome` | Biome | L0, L1, L4, L6 |
| | `spawn_particle` | Particle | L0, L1, L2 |
| | `spawn_stone` | Stone | L0, L1, L4 |
| | `spawn_lava_knight` | LavaKnight | L0-L8 |
| **competition.rs** | `spawn_pool` | Pool | EnergyPool, L6 |
| | `spawn_competitor` | Competitor | PoolParentLink |
| | `spawn_sub_pool` | SubPool | nested pool |

## HeroClass (6 clases)

| Clase | Elemento | Frecuencia | Perfil |
|-------|----------|-----------|--------|
| FireMage | Ignis | 450 Hz | Alta energia, bajo radio |
| EarthWarrior | Terra | 75 Hz | Alta cohesion, alta bond_energy |
| PlantAssassin | Umbra | 20 Hz | Baja energia, alta velocidad |
| LightHealer | Lux | 1000 Hz | Buffer grande, alta visibilidad |
| WindShooter | Ventus | 700 Hz | Largo rango, alta disipacion |
| WaterTank | Aqua | 250 Hz | Maxima cohesion, alta viscosidad |

## EntityBuilder (API fluent)

```rust
EntityBuilder::new()
    .named("FireMage")
    .at(Vec2::new(10.0, 5.0))
    .energy(500.0)           // L0
    .volume(0.8)             // L1
    .wave(element_id)        // L2
    .flow(Vec2::ZERO, 0.01)  // L3
    .matter(Solid, 2000.0)   // L4
    .motor(1500.0, 8.0)      // L5
    .will_default()          // L7
    .identity(Red, vec![Hero], 1.5)  // L9
    .spawn(commands)
```

## Dependencias

- `crate::layers` — todos los componentes de las 14 capas
- `crate::blueprint::constants` — valores por defecto de arquetipos
- `bevy::prelude` — Commands, Entity, Transform

## Invariantes

- Configs respetan invariantes numericas de `layers/` (qe >= 0, radius >= 0.01)
- `spawn_projectile` define flags de colision/despawn
- `spawn_effect` crea entidad Tipo B (L10) — duracion emerge de disipacion
- Derivaciones que dependen de sistemas corren luego en `simulation/`
