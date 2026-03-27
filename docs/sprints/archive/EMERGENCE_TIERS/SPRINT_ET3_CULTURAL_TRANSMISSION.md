# Sprint ET-3 — Cultural Transmission: Imitación como Adaptación Rápida

**Módulo:** `src/simulation/emergence/culture.rs` (nuevo), `src/blueprint/equations/emergence/culture.rs` (nuevo)
**Tipo:** Ecuaciones puras + sistema de transmisión + evento.
**Tier:** T1-3. **Onda:** A.
**BridgeKind:** `MemeSpreadBridge` — cache Small(64), clave `(behavior_hash, population_density_band)`.
**Estado:** ✅ Implementado (2026-03-25)

---

## Contexto: que ya existe

- ET-1 `AssociativeMemory`, ET-2 `OtherModelSet` — fundación de aprendizaje individual.
- `layers/social_communication.rs::PackMembership` — grupos de entidades (canal de transmisión).
- `simulation/behavior.rs::BehaviorMode` — el "comportamiento" que se transmite es una decisión de qué hacer con la energía.
- `world/SpatialIndex` — `query_radius` para detectar quién está en rango de imitación.

**Lo que NO existe:**
1. Mecanismo de copia de comportamiento entre entidades no-emparentadas.
2. `MemeEntry` — representación de un comportamiento transmisible con fitness.
3. Diferenciación cultural regional (mismos genes, distinto comportamiento por historia local).
4. `MemeAdoptedEvent` — señal de que una entidad adoptó un comportamiento.

---

## Objetivo

Un organismo que observa a otro con mayor éxito energético imita su comportamiento. La imitación es más rápida que la evolución genética: `T_cultural << T_genetic`. Poblaciones regionales con la misma genética pero historia distinta divergen culturalmente.

```
meme_fitness(B) = E[extraction_improvement | adopting B] - adoption_cost - maintenance_cost
spread_rate(B)  = meme_fitness(B) × contact_rate × imitation_probability
```

---

## Responsabilidades

### ET-3A: Ecuaciones puras

```rust
// src/blueprint/equations/emergence/culture.rs

/// Fitness de un comportamiento (meme): mejora esperada de extracción menos costos.
pub fn meme_fitness(
    extraction_improvement: f32,  // ΔE[qe/tick] al adoptar el comportamiento
    adoption_cost: f32,           // qe gastado en adoptar
    maintenance_cost: f32,        // qe/tick para mantener el comportamiento
) -> f32 {
    extraction_improvement - adoption_cost - maintenance_cost
}

/// Tasa de propagación de un meme en una población.
pub fn spread_rate(fitness: f32, contact_rate: f32, imitation_prob: f32) -> f32 {
    if fitness <= 0.0 { return 0.0; }
    fitness * contact_rate * imitation_prob
}

/// ¿Vale la pena imitar? Compara fitness del comportamiento observado con el propio.
pub fn should_imitate(
    observer_current_rate: f32,    // qe/tick actual del observador
    target_observed_rate: f32,     // qe/tick del modelo a imitar
    adoption_cost: f32,
    uncertainty: f32,              // [0,1] — incertidumbre de la observación
) -> bool {
    let expected_gain = (target_observed_rate - observer_current_rate) * (1.0 - uncertainty);
    expected_gain > adoption_cost
}

/// Deriva cultural por aislamiento: distancia entre nichos culturales de dos poblaciones.
/// behavior_vectors: 4D vector de preferencias energéticas normalizadas.
pub fn cultural_distance(behavior_a: [f32; 4], behavior_b: [f32; 4]) -> f32 {
    behavior_a.iter()
        .zip(behavior_b.iter())
        .map(|(a, b)| (a - b).powi(2))
        .sum::<f32>()
        .sqrt()
}
```

### ET-3B: Tipos

```rust
// src/simulation/emergence/culture.rs

/// Comportamiento transmisible: identificado por hash, con fitness estimada.
#[derive(Debug, Clone, Copy, Default, Reflect)]
pub struct MemeEntry {
    pub behavior_hash: u32,         // hash del comportamiento (BehaviorMode discriminant + params)
    pub estimated_fitness: f32,     // fitness observada en modelos imitados
    pub adoption_tick: u64,         // cuándo se adoptó
    pub spread_count: u8,           // cuántas veces lo transmitiste a otros
}

/// Capa T1-3: CulturalMemory — comportamientos aprendidos por imitación.
#[derive(Component, Reflect, Debug, Clone)]
#[reflect(Component)]
pub struct CulturalMemory {
    pub memes:       [MemeEntry; MAX_MEMES],
    pub meme_count:  u8,
    pub imitation_radius: f32,       // radio de observación cultural
    pub imitation_prob:   f32,       // probabilidad de imitar si fitness > propia
}

pub const MAX_MEMES: usize = 4;

/// Evento emitido cuando una entidad adopta un comportamiento por imitación.
#[derive(Event, Debug, Clone)]
pub struct MemeAdoptedEvent {
    pub adopter: Entity,
    pub source:  Entity,
    pub behavior_hash: u32,
    pub tick_id: u64,
}
```

### ET-3C: Sistema

```rust
/// Propaga comportamientos entre entidades en rango de imitación.
/// Phase::Input, in_set(EmergenceTier1Set), after theory_of_mind_update_system.
pub fn cultural_transmission_system(
    mut imitators: Query<(
        Entity, &Transform, &mut CulturalMemory, &BaseEnergy,
    ), With<BehavioralAgent>>,
    models: Query<(Entity, &Transform, &BaseEnergy, &CulturalMemory)>,
    spatial: Res<SpatialIndex>,
    clock: Res<SimulationClock>,
    mut events: EventWriter<MemeAdoptedEvent>,
    config: Res<CultureConfig>,
) {
    for (imitator_entity, transform, mut culture, energy) in &mut imitators {
        let pos = Vec2::new(transform.translation.x, transform.translation.z);
        let nearby = spatial.query_radius(pos, culture.imitation_radius);

        for entry in &nearby {
            let target_entity = entry.entity;
            if target_entity == imitator_entity { continue; }
            let Ok((_, _, target_energy, target_culture)) = models.get(target_entity) else { continue; };

            // Sólo imitar si el modelo tiene mayor éxito energético
            if !culture_eq::should_imitate(
                energy.qe(), target_energy.qe(),
                config.adoption_cost, config.observation_uncertainty,
            ) { continue; }

            // Adoptar el meme más exitoso del modelo
            if let Some(best_meme) = target_culture.memes[..target_culture.meme_count as usize]
                .iter()
                .max_by(|a, b| a.estimated_fitness.partial_cmp(&b.estimated_fitness).unwrap())
            {
                // Verificar que no lo tenemos ya
                let already_has = culture.memes[..culture.meme_count as usize]
                    .iter()
                    .any(|m| m.behavior_hash == best_meme.behavior_hash);
                if already_has { continue; }

                if (culture.meme_count as usize) < MAX_MEMES {
                    let idx = culture.meme_count as usize;
                    culture.memes[idx] = MemeEntry {
                        behavior_hash: best_meme.behavior_hash,
                        estimated_fitness: best_meme.estimated_fitness,
                        adoption_tick: clock.tick_id,
                        spread_count: 0,
                    };
                    culture.meme_count += 1;
                    events.send(MemeAdoptedEvent {
                        adopter: imitator_entity,
                        source: target_entity,
                        behavior_hash: best_meme.behavior_hash,
                        tick_id: clock.tick_id,
                    });
                }
            }
        }
    }
}
```

### ET-3D: Constantes

```rust
// src/blueprint/constants/emergence/culture.rs
pub const CULTURE_DEFAULT_IMITATION_RADIUS: f32 = 10.0;
pub const CULTURE_DEFAULT_IMITATION_PROB: f32 = 0.3;
pub const CULTURE_ADOPTION_COST: f32 = 1.0;   // qe por adopción
pub const CULTURE_OBSERVATION_UNCERTAINTY: f32 = 0.2;

pub struct MemeSpreadBridge;
impl BridgeKind for MemeSpreadBridge {}
```

---

## Tacticas

- **BridgeCache para spread_rate.** La misma combinación `(behavior_hash, population_density_band)` se repite para todos los agentes en la misma zona. Cache hit rate alto.
- **MemeAdoptedEvent como señal de trazabilidad.** Permite rastrear árboles filogenéticos culturales — importante para SF-7 (replay) y observabilidad.
- **Sin String en MemeEntry.** `behavior_hash` es un `u32` que codifica el tipo de comportamiento. Determinista, compacto.
- **Regional divergence emerge sola.** Poblaciones geográficamente separadas no se imitan entre sí → divergen aunque tengan mismos genes. Sin programar diferenciación regional.

---

## NO hace

- No implementa lenguaje simbólico — eso es ET-15.
- No modifica InferenceProfile genético — la transmisión cultural es horizontal, no vertical.
- No programa qué comportamientos son "buenos" — el fitness se observa en el qe del modelo.

---

## Dependencias

- ET-1, ET-2 — fundación de aprendizaje individual.
- `layers/social_communication.rs::PackMembership` — canal de transmisión preferente.
- `world/SpatialIndex` — detectar entidades en rango de imitación.
- `blueprint/equations/emergence/culture.rs` — ecuaciones puras.

---

## Criterios de Aceptación

### ET-3A
- `meme_fitness(5.0, 1.0, 0.5)` → `3.5`.
- `meme_fitness(-1.0, 1.0, 0.5)` → negativo (comportamiento perjudicial).
- `spread_rate(-1.0, 5.0, 0.5)` → `0.0` (memes negativos no se propagan).
- `should_imitate(100.0, 200.0, 1.0, 0.0)` → `true`.
- `should_imitate(200.0, 100.0, 1.0, 0.0)` → `false`.
- `cultural_distance([1,0,0,0], [0,1,0,0])` → `√2 ≈ 1.414`.
- `cultural_distance([1,0,0,0], [1,0,0,0])` → `0.0`.

### ET-3C
- Test: agente con qe bajo observa agente con qe alto → `MemeAdoptedEvent` emitido.
- Test: agente con qe mayor observa agente con qe menor → no imita.
- Test: `meme_count == MAX_MEMES` → no falla, no adopta más.
- Test: mismo meme no adoptado dos veces.

### General
- `cargo test --lib` sin regresión. Sin Vec/String en componentes.

---

## Referencias

- ET-1, ET-2 — cadena de aprendizaje individual
- `src/layers/social_communication.rs` — PackMembership como canal
- Blueprint §T1-3: "Cultural Transmission", meme fitness equation
