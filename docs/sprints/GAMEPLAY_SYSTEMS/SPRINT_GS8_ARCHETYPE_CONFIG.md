# Sprint GS-8 — Archetype Config: Personaje como Configuración de Física

**Modulo:** `src/entities/archetypes/` (extensión), `assets/characters/*.ron` (nuevo), `src/blueprint/constants/archetype_config.rs` (nuevo)
**Tipo:** RON loader + tipos + spawn desde config.
**Onda:** B — Requiere GS-5 (física de partida) + `entities/archetypes/` existente.
**Estado:** ⏳ Pendiente

---

## Contexto: que ya existe

**Lo que SÍ existe:**

- `entities/archetypes/` — funciones `spawn_*` que construyen entidades con composición de 14 capas.
- `src/entities/builder.rs::EntityBuilder` — API fluente para construir entidades por capas.
- `layers/inference.rs::InferenceProfile` — morfología inferida (masa, radio, densidad). Derivado de física.
- `layers/oscillatory.rs::OscillatorySignature` — frecuencia base del personaje.
- `layers/identity.rs::Faction` — equipo.
- `blueprint/constants/` — constantes de física por módulo. Ya separadas por dominio.
- `serde` + `ron` en Cargo.toml — serialización ya disponible.

**Lo que NO existe:**

1. **ArchetypeConfig.** No hay struct que represente "un personaje" como configuración data-driven.
2. **RON assets de personajes.** No hay `assets/characters/*.ron` con stats de balanceo.
3. **Spawn desde archivo.** Las funciones `spawn_*` usan constantes hardcoded — no RON.
4. **Balance sin código.** Cambiar el daño de un personaje requiere recompilar.
5. **Variantes de arquetipo.** No hay múltiples configuraciones del mismo arquetipo base (ej: Tank/Bruiser/Assassin desde la misma física).

---

## Objetivo

Separar la configuración de personaje (qué parámetros de física) del código (cómo se construye la entidad). Un `ArchetypeConfig` RON define todos los escalares tuneables. Las funciones `spawn_from_config` los leen y construyen la entidad. Balance = editar un archivo `.ron`.

```
assets/characters/resonance_tank.ron → ArchetypeConfig → spawn_from_config() → Entity(14 capas)
```

---

## Responsabilidades

### GS-8A: Tipo ArchetypeConfig

```rust
// src/entities/archetypes/config.rs

/// Configuración completa de un arquetipo como datos RON.
/// Todos los campos son f32/u8/bool — sin String, sin Box.
/// Máx por bloque lógico = 4 campos (regla de componente aplicada a config structs).
#[derive(Debug, Clone, Reflect)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize))]
pub struct ArchetypeConfig {
    pub identity: ArchetypeIdentity,
    pub energy: EnergyConfig,
    pub oscillatory: OscillatoryConfig,
    pub matter: MatterConfig,
    pub engine: EngineConfig,
    pub locomotion: LocomotionConfig,
    pub combat: CombatConfig,
    pub social: SocialConfig,
}

#[derive(Debug, Clone, Reflect)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize))]
pub struct ArchetypeIdentity {
    pub archetype_id: u32,   // strong ID numérico
    pub faction: Faction,
    pub pack_role: PackRole,
    pub is_hero: bool,       // true = jugador controla; false = AI
}

#[derive(Debug, Clone, Reflect)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize))]
pub struct EnergyConfig {
    pub base_qe: f32,
    pub max_qe: f32,
    pub regen_rate: f32,
    pub extraction_capacity: f32,
}

#[derive(Debug, Clone, Reflect)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize))]
pub struct OscillatoryConfig {
    pub frequency_hz: f32,
    pub phase_offset: f32,
    pub resonance_bandwidth: f32,
    pub adaptation_rate: f32,
}

#[derive(Debug, Clone, Reflect)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize))]
pub struct MatterConfig {
    pub structural_integrity: f32,
    pub bond_energy: f32,
    pub inertial_mass: f32,
    pub volume_radius: f32,
}

#[derive(Debug, Clone, Reflect)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize))]
pub struct EngineConfig {
    pub buffer_capacity: f32,
    pub intake_rate: f32,
    pub valve_open: f32,
    pub efficiency: f32,
}

#[derive(Debug, Clone, Reflect)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize))]
pub struct LocomotionConfig {
    pub max_velocity: f32,
    pub drag_coefficient: f32,
    pub turn_rate: f32,
    pub sprint_factor: f32,
}

#[derive(Debug, Clone, Reflect)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize))]
pub struct CombatConfig {
    pub attack_radius: f32,
    pub sensory_radius: f32,
    pub flee_threshold_qe: f32,
    pub hunt_min_qe: f32,
}

#[derive(Debug, Clone, Reflect)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize))]
pub struct SocialConfig {
    pub pack_id: u32,
    pub cohesion_weight: f32,
    pub flee_weight: f32,
    pub communication_radius: f32,
}
```

### GS-8B: Función spawn_from_config

```rust
// src/entities/archetypes/spawn.rs

/// Spawna una entidad completa desde ArchetypeConfig.
/// Respeta las 14 capas vía EntityBuilder. No hardcodea valores.
pub fn spawn_from_config(
    commands: &mut Commands,
    config: &ArchetypeConfig,
    position: Vec2,
    clock: &SimulationClock,
) -> Entity {
    let id = WorldEntityId::next_from_clock(clock);

    EntityBuilder::new(commands)
        .with_identity(id, config.identity.faction, config.identity.pack_role)
        .with_energy(
            config.energy.base_qe,
            config.energy.max_qe,
            config.energy.regen_rate,
        )
        .with_oscillatory(
            config.oscillatory.frequency_hz,
            config.oscillatory.phase_offset,
        )
        .with_matter(
            config.matter.structural_integrity,
            config.matter.bond_energy,
            config.matter.inertial_mass,
        )
        .with_volume(config.matter.volume_radius)
        .with_engine(
            config.engine.buffer_capacity,
            config.engine.intake_rate,
        )
        .with_locomotion(
            config.locomotion.max_velocity,
            config.locomotion.drag_coefficient,
        )
        .with_inference(InferenceProfile::from_config(&config.combat))
        .with_social(
            config.social.pack_id,
            config.identity.pack_role,
        )
        .at_position(position)
        .build()
}
```

### GS-8C: Loader de RON

```rust
// src/entities/archetypes/loader.rs

/// Resource que almacena configs cargados desde assets.
/// Indexado por archetype_id — Vec ordenado, sin HashMap.
#[derive(Resource, Default, Debug)]
pub struct ArchetypeRegistry {
    pub configs: Vec<ArchetypeConfig>,   // ordenado por archetype_id
}

impl ArchetypeRegistry {
    pub fn get(&self, id: u32) -> Option<&ArchetypeConfig> {
        self.configs.iter().find(|c| c.identity.archetype_id == id)
    }
    pub fn insert(&mut self, config: ArchetypeConfig) {
        let id = config.identity.archetype_id;
        if let Some(idx) = self.configs.iter().position(|c| c.identity.archetype_id == id) {
            self.configs[idx] = config;
        } else {
            self.configs.push(config);
            self.configs.sort_unstable_by_key(|c| c.identity.archetype_id);
        }
    }
}

/// Sistema startup: carga todos los .ron de assets/characters/.
/// Corre en Startup, antes de spawn de entidades.
pub fn load_archetype_configs_system(
    mut registry: ResMut<ArchetypeRegistry>,
    asset_server: Res<AssetServer>,  // sólo en contexto app
) {
    // Lee los paths desde ARCHETYPE_CONFIG_PATHS (constante con lista de archivos)
    for path in ARCHETYPE_CONFIG_PATHS {
        // En headless/test: load directo desde bytes embebidos
        // En app: asset_server.load()
        let _ = (registry.as_mut(), asset_server.as_ref(), path);
    }
}
```

### GS-8D: Assets RON (ejemplos)

```ron
// assets/characters/resonance_tank.ron
ArchetypeConfig(
    identity: ArchetypeIdentity(
        archetype_id: 1,
        faction: A,
        pack_role: Leader,
        is_hero: false,
    ),
    energy: EnergyConfig(
        base_qe: 800.0,
        max_qe: 1200.0,
        regen_rate: 2.0,
        extraction_capacity: 15.0,
    ),
    oscillatory: OscillatoryConfig(
        frequency_hz: 220.0,
        phase_offset: 0.0,
        resonance_bandwidth: 50.0,
        adaptation_rate: 0.1,
    ),
    matter: MatterConfig(
        structural_integrity: 1.0,
        bond_energy: 500.0,
        inertial_mass: 10.0,
        volume_radius: 2.5,
    ),
    engine: EngineConfig(
        buffer_capacity: 200.0,
        intake_rate: 25.0,
        valve_open: 0.8,
        efficiency: 0.9,
    ),
    locomotion: LocomotionConfig(
        max_velocity: 4.0,
        drag_coefficient: 0.15,
        turn_rate: 90.0,
        sprint_factor: 1.5,
    ),
    combat: CombatConfig(
        attack_radius: 6.0,
        sensory_radius: 20.0,
        flee_threshold_qe: 100.0,
        hunt_min_qe: 200.0,
    ),
    social: SocialConfig(
        pack_id: 1,
        cohesion_weight: 1.5,
        flee_weight: 1.0,
        communication_radius: 15.0,
    ),
)
```

---

## Tacticas

- **Config como datos puros.** `ArchetypeConfig` es `#[derive(Deserialize)]` sin lógica. Toda la lógica de construcción está en `spawn_from_config`. Separación total dato/comportamiento.
- **Sin HashMap en registry.** `ArchetypeRegistry` usa Vec + sort determinista. Lookup por `find` — O(n) con n < 20. Correcto.
- **Campos divididos en 8 sub-structs de máx 4 campos.** La regla "max 4 fields per component" aplica igual a config structs para legibilidad y extensibilidad.
- **Fallback a constantes.** Si `ArchetypeRegistry` no tiene el config de un ID, `spawn_from_config` usa valores default de `blueprint/constants/`. No pánico.

---

## NO hace

- No define qué personajes están en cada mapa — eso es el RON del mapa (worldgen).
- No implementa selección de personaje en UI — eso es resonance-app.
- No versiona configs (migración de RON) — out of scope.
- No implementa hot-reload de configs en runtime — sólo carga en Startup.

---

## Dependencias

- `entities/archetypes/` — funciones `spawn_*` existentes (GS-8 las extiende, no reemplaza).
- `src/entities/builder.rs::EntityBuilder` — API de construcción.
- `layers/inference.rs::InferenceProfile` — `from_config()` helper nuevo.
- `layers/identity.rs::Faction`, `PackRole` — enums existentes.
- `serde` + `ron` — ya en Cargo.toml.
- GS-4 — `PackDynamicsConfig` para `SocialConfig` mapping.

---

## Criterios de aceptacion

### GS-8A (Config)
- `ArchetypeConfig` deserializa desde RON sin pánico.
- Todos los sub-structs tienen máx 4 campos.
- Sin `String` en ningún campo — sólo `f32`, `u32`, `u8`, `bool`, enums.

### GS-8B (Spawn)
- Test: `spawn_from_config` con config tank → entidad con `BaseEnergy.qe() == config.energy.base_qe`.
- Test: `spawn_from_config` con config assassin → `OscillatorySignature.frequency_hz() == config.oscillatory.frequency_hz`.
- Test: dos configs distintos → entidades con propiedades distintas.

### GS-8C (Registry)
- `ArchetypeRegistry::get(id)` → `Some` si existe, `None` si no.
- `insert` × N → configs ordenados por archetype_id.
- Sin pánico si RON no existe (fallback a default).

### General
- `cargo test --lib` sin regresión.
- Sin `HashMap`. Sin `String` en componentes ni config structs.

---

## Referencias

- `src/entities/archetypes/` — funciones spawn existentes
- `src/entities/builder.rs` — EntityBuilder API
- `src/layers/inference.rs` — InferenceProfile
- `assets/maps/*.ron` — patrón RON existente en el proyecto
- Blueprint §8: "Character as Physics Configuration"
- `docs/design/GAMEDEV_PATTERNS.md` — arquetipo pattern
