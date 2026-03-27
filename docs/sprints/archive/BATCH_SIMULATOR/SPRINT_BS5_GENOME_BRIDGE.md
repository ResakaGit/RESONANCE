# Sprint BS-5 — Genome Bridge: Batch ↔ Bevy Round-Trip

**Modulo:** `src/batch/bridge.rs`
**Tipo:** Conversion layer entre GenomeBlob y componentes Bevy.
**Onda:** BS-4 → BS-5.
**Estado:** ⏳ Pendiente

---

## Contexto: que ya existe (post BS-4)

- `GenomeBlob` con `from_slot()`, `apply()`, `mutate()`, `crossover()`.
- `GeneticHarness` completo con loop evolutivo.
- `SimWorld` (Bevy) con `tick()`, `snapshot()`, `EntityBuilder`.
- `InferenceProfile`, `MobaIdentity`, `TrophicConsumer` — componentes Bevy existentes.

---

## Objetivo

Crear funciones de conversion bidireccional entre `GenomeBlob` (batch, sin Bevy) y componentes Bevy (ECS). Garantizar que el round-trip es lossless: `genome → bevy → genome = identical`.

---

## Responsabilidades

### BS-5A: Batch → Bevy

```rust
// src/batch/bridge.rs

use crate::layers::*;
use crate::blueprint::constants;

/// Convierte GenomeBlob en tupla de componentes Bevy para spawn.
pub fn genome_to_components(genome: &GenomeBlob) -> (
    BaseEnergy,
    SpatialVolume,
    OscillatorySignature,
    FlowVector,
    MatterCoherence,
    AlchemicalEngine,
    InferenceProfile,
    // + archetype-specific components via separate fn
) {
    (
        BaseEnergy::new(constants::DEFAULT_BASE_ENERGY),
        SpatialVolume::new(constants::DEFAULT_RADIUS),
        OscillatorySignature::new(
            constants::frequency_for_archetype(genome.archetype), 0.0,
        ),
        FlowVector::default(),
        MatterCoherence::new(MatterState::Liquid, 1.0, 0.1),
        AlchemicalEngine::new(
            0.0,
            constants::DEFAULT_ENGINE_MAX,
            constants::DEFAULT_INPUT_VALVE,
            constants::DEFAULT_OUTPUT_VALVE,
        ),
        InferenceProfile::new(
            genome.growth_bias,
            genome.mobility_bias,
            genome.branching_bias,
            genome.resilience,
        ),
    )
}

/// Componentes adicionales segun archetype.
pub fn archetype_extras(genome: &GenomeBlob) -> Vec<Box<dyn std::any::Any>> {
    // NOT trait objects in ECS — esto retorna tuplas segun archetype
    // Implementacion real usa match + spawn especifico
    todo!("match genome.archetype → spawn_flora/spawn_animal/etc extras")
}
```

### BS-5B: Bevy → Batch

```rust
/// Extrae GenomeBlob desde componentes Bevy.
pub fn components_to_genome(
    profile: &InferenceProfile,
    identity: Option<&MobaIdentity>,
    trophic: Option<&TrophicConsumer>,
    osc: &OscillatorySignature,
) -> GenomeBlob {
    GenomeBlob {
        archetype: infer_archetype(identity, trophic),
        trophic_class: trophic.map(|t| t.class() as u8).unwrap_or(0),
        growth_bias: profile.growth_bias(),
        mobility_bias: profile.mobility_bias(),
        branching_bias: profile.branching_bias(),
        resilience: profile.resilience(),
    }
}

fn infer_archetype(
    identity: Option<&MobaIdentity>,
    trophic: Option<&TrophicConsumer>,
) -> u8 {
    // 0=inert, 1=flora, 2=fauna, 3=cell, 4=virus
    match trophic {
        Some(t) if t.class() == TrophicClass::PrimaryProducer => 1,
        Some(t) if t.class() == TrophicClass::Carnivore => 2,
        Some(_) => 3,
        None => 0,
    }
}
```

### BS-5C: Serialization a disco

```rust
/// Serializa Vec<GenomeBlob> a archivo binario.
/// Format: [u32 count][GenomeBlob; count] — little-endian, repr(C).
pub fn save_genomes(genomes: &[GenomeBlob], path: &std::path::Path) -> std::io::Result<()> {
    let count = genomes.len() as u32;
    let mut buf = Vec::with_capacity(4 + genomes.len() * std::mem::size_of::<GenomeBlob>());
    buf.extend_from_slice(&count.to_le_bytes());
    for g in genomes {
        let bytes: &[u8] = unsafe {
            std::slice::from_raw_parts(g as *const _ as *const u8, std::mem::size_of::<GenomeBlob>())
        };
        buf.extend_from_slice(bytes);
    }
    std::fs::write(path, buf)
}

/// Deserializa Vec<GenomeBlob> desde archivo binario.
pub fn load_genomes(path: &std::path::Path) -> std::io::Result<Vec<GenomeBlob>> { ... }
```

**Nota:** `save_genomes` es el unico lugar donde `unsafe` es aceptable — `GenomeBlob` es `repr(C)` + `Copy` + no padding. Tests verifican round-trip.

---

## NO hace

- No modifica SimWorld ni SimConfig — solo provee funciones de conversion.
- No spawna entidades — el caller (SimWorld o test) decide cuando.
- No implementa rayon — eso es BS-6.

---

## Dependencias

- BS-4 — `GenomeBlob` tipos completos.
- `crate::layers` — componentes Bevy para conversion.
- `crate::blueprint::constants` — defaults para componentes.
- `crate::entities::builder` — referencia para que componentes necesita cada archetype.

---

## Criterios de aceptacion

### BS-5A (Round-trip)
- `components_to_genome(genome_to_components(g)) == g` para todo `GenomeBlob` valido.
- Los 4 biases (growth, mobility, branching, resilience) son bit-exact en round-trip.
- `archetype` se preserva: flora→1, fauna→2, cell→3, virus→4.

### BS-5C (Serialization)
- `load_genomes(save_genomes(genomes))` == `genomes` (round-trip disco).
- Archivo vacio → `load_genomes` retorna `Ok(vec![])`.
- Archivo corrupto → `load_genomes` retorna `Err`.

### Integracion
- Genomes evolucionados en batch se pueden spawnar en `SimWorld` Bevy sin panic.
- Entidades spawneadas tienen los componentes correctos segun archetype.

---

## Referencias

- `docs/arquitectura/blueprint_batch_simulator.md` §6 — contrato de integracion
- `src/entities/archetypes/catalog.rs` — spawn functions per archetype
- `src/layers/inference.rs` — InferenceProfile component
