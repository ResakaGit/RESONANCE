# Sprint ET-15 — Language: Comunicación Simbólica a Distancia

**Módulo:** `src/layers/language.rs` (nuevo), `src/blueprint/equations/emergence/language.rs` (nuevo)
**Tipo:** Nueva capa + ecuaciones puras.
**Tier:** T4-2. **Onda:** C.
**BridgeKind:** `SymbolBridge` — cache Small(64), clave `(vocabulary_band, semantic_hash)`.
**Estado:** ✅ Implementado (2026-03-25)

---

## Objetivo

La comunicación simbólica permite coordinación a distancia y transmisión de información compleja sin contacto físico. Un símbolo es un hash de comportamiento (`u32`) que dos entidades han co-desarrollado mediante interacción repetida. El lenguaje emerge de símbolos compartidos que forman un vocabulario con gramática implícita.

```
symbol_fitness(s) = information_conveyed(s) × reception_rate(s) - encoding_cost(s)
shared_vocabulary(a,b) = |{s : symbol_map(a) ∩ symbol_map(b)}| / max_vocab
communication_efficiency = shared_vocab × signal_range / noise_level
```

---

## Responsabilidades

### ET-15A: Ecuaciones puras

```rust
// src/blueprint/equations/emergence/language.rs

/// Fitness de un símbolo: información transmitida menos costo de encoding.
pub fn symbol_fitness(
    information_bits: f32,    // bits de información que transmite el símbolo
    reception_rate: f32,      // probabilidad de que el receptor lo comprenda [0,1]
    encoding_cost: f32,       // qe/uso
) -> f32 {
    information_bits * reception_rate - encoding_cost
}

/// Vocabulario compartido entre dos entidades: intersección normalizada.
pub fn shared_vocabulary_ratio(
    vocab_a: &[u32],  // hashes de símbolos de A
    vocab_b: &[u32],  // hashes de símbolos de B
) -> f32 {
    if vocab_a.is_empty() || vocab_b.is_empty() { return 0.0; }
    let shared = vocab_a.iter().filter(|s| vocab_b.contains(s)).count();
    shared as f32 / vocab_a.len().max(vocab_b.len()) as f32
}

/// Eficiencia de comunicación: vocabulario compartido × alcance / ruido.
pub fn communication_efficiency(
    shared_ratio: f32,
    signal_range: f32,
    noise_level: f32,
) -> f32 {
    if noise_level <= 0.0 { return shared_ratio * signal_range; }
    shared_ratio * signal_range / (1.0 + noise_level)
}

/// Tasa de deriva semántica: con qué velocidad cambia el significado de un símbolo.
pub fn semantic_drift_rate(
    symbol_usage_frequency: f32,
    population_size: f32,
    isolation_factor: f32,   // 0=conectado, 1=aislado
) -> f32 {
    if symbol_usage_frequency <= 0.0 { return isolation_factor; }
    isolation_factor / (symbol_usage_frequency * population_size.sqrt())
}

/// Complejidad gramatical emergente: cuántos símbolos se combinan por mensaje.
pub fn grammar_complexity(
    vocab_size: u8,
    interaction_frequency: f32,
) -> f32 {
    (vocab_size as f32).ln() * interaction_frequency.sqrt()
}
```

### ET-15B: Componente

```rust
// src/layers/language.rs

/// Capa T4-2: LanguageCapacity — capacidad simbólica de una entidad.
#[derive(Component, Reflect, Debug, Clone)]
#[reflect(Component)]
pub struct LanguageCapacity {
    pub vocabulary:      [u32; MAX_VOCAB_SIZE],  // hashes de símbolos conocidos
    pub vocab_count:     u8,
    pub signal_range:    f32,    // radio de comunicación efectiva
    pub encoding_cost:   f32,    // qe por acto de comunicación
}

pub const MAX_VOCAB_SIZE: usize = 8;

impl LanguageCapacity {
    pub fn vocab_slice(&self) -> &[u32] {
        &self.vocabulary[..self.vocab_count as usize]
    }
    pub fn has_symbol(&self, symbol: u32) -> bool {
        self.vocab_slice().contains(&symbol)
    }
    pub fn add_symbol(&mut self, symbol: u32) -> bool {
        if self.vocab_count as usize >= MAX_VOCAB_SIZE { return false; }
        if self.has_symbol(symbol) { return false; }
        self.vocabulary[self.vocab_count as usize] = symbol;
        self.vocab_count += 1;
        true
    }
}
```

### ET-15C: Sistema

```rust
/// Transmite símbolos entre entidades en rango de señal.
/// Phase::Input, after cultural_transmission_system — lenguaje extiende cultura.
pub fn symbol_transmission_system(
    mut agents: Query<(Entity, &Transform, &mut LanguageCapacity, &mut BaseEnergy)>,
    spatial: Res<SpatialIndex>,
    mut cache: ResMut<BridgeCache<SymbolBridge>>,
    clock: Res<SimulationClock>,
    config: Res<LanguageConfig>,
) {
    if clock.tick_id % config.transmission_interval as u64 != 0 { return; }

    // Snapshot para evitar aliasing
    let snapshot: Vec<(Entity, [f32; 2], [u32; MAX_VOCAB_SIZE], u8)> = agents.iter()
        .map(|(e, t, lc, _)| (e, [t.translation.x, t.translation.z], lc.vocabulary, lc.vocab_count))
        .collect();

    for (entity, transform, mut lang, mut energy) in &mut agents {
        let pos = Vec2::new(transform.translation.x, transform.translation.z);
        let nearby = spatial.query_radius(pos, lang.signal_range);

        for entry in &nearby {
            let target_entity = entry.entity;
            if target_entity == entity { continue; }
            let Some(&(_, _, t_vocab, t_count)) = snapshot.iter()
                .find(|(e, _, _, _)| *e == target_entity) else { continue };

            // Cache: vocabulario compartido por banda de hash
            let cache_key = lang.vocabulary[0].wrapping_add(t_vocab[0]);
            let shared_ratio = if let Some(cached) = cache.get(cache_key) {
                cached
            } else {
                let r = language_eq::shared_vocabulary_ratio(
                    lang.vocab_slice(),
                    &t_vocab[..t_count as usize],
                );
                cache.insert(cache_key, r);
                r
            };

            // Aprender un símbolo del vecino si tiene vocabulario más rico
            if (t_count as usize) > lang.vocab_count as usize && lang.vocab_count < MAX_VOCAB_SIZE as u8 {
                if let Some(&new_sym) = t_vocab[..t_count as usize].iter()
                    .find(|&&s| !lang.has_symbol(s))
                {
                    let fitness = language_eq::symbol_fitness(
                        1.0, shared_ratio, lang.encoding_cost,
                    );
                    if fitness > 0.0 && lang.add_symbol(new_sym) {
                        let new_qe = (energy.qe() - lang.encoding_cost).max(0.0);
                        if energy.qe() != new_qe { energy.set_qe(new_qe); }
                    }
                }
            }
        }
    }
}
```

### ET-15D: Constantes

```rust
pub struct SymbolBridge;
impl BridgeKind for SymbolBridge {}

pub const LANGUAGE_DEFAULT_SIGNAL_RANGE:        f32 = 20.0;
pub const LANGUAGE_DEFAULT_ENCODING_COST:       f32 = 0.2;
pub const LANGUAGE_TRANSMISSION_INTERVAL:       u8  = 5;
pub const LANGUAGE_MAX_VOCAB_SIZE:             usize = 8;
```

---

## Tacticas

- **`[u32; 8]` vocabulario fijo.** Sin Vec — el vocabulario es un array de hashes de símbolos. Cada símbolo es un `u32` (Hard Block 6: sin String).
- **Símbolos como hashes de comportamiento.** El hash `behavior_hash` de `MemeEntry` (ET-3) es reutilizable como símbolo lingüístico. El lenguaje emerge directamente de la cultura.
- **`SymbolBridge` cachea vocabulario compartido.** Calcular intersección de vocabularios es O(n×m) sin caché. Con `(vocab_hash_a, vocab_hash_b)` como clave → hit rate alto para entidades colocalizadas.
- **Deriva semántica sin estado adicional.** El significado deriva cuando entidades aisladas usan el mismo símbolo para contextos distintos. Sin "diccionario" centralizado — el lenguaje es distribuido por naturaleza.
- **Gramática como complejidad emergente.** `grammar_complexity` no es un campo — es una ecuación sobre el estado actual del vocabulario. Coherente con el principio stateless-first.

---

## NO hace

- No implementa sintaxis formal — la "gramática" es implicit en el orden de `behavior_hash`.
- No persiste lenguajes entre sesiones — vocabularios se reconstruyen en cada partida.
- No implementa traducción entre dialectos — `shared_vocabulary_ratio` mide directamente la intersección.

---

## Dependencias

- ET-3 `CulturalMemory` — `behavior_hash` de memes son proto-símbolos.
- ET-8 `CoalitionRegistry` — coaliciones tienen vocabulario compartido alto → comunicación eficiente.
- ET-14 `InstitutionRegistry` — instituciones pueden codificar reglas como símbolos (`rule_hash`).
- `world/SpatialIndex` — detectar entidades en rango de señal.

---

## Criterios de Aceptación

- `symbol_fitness(4.0, 0.8, 1.0)` → `2.2`.
- `symbol_fitness(1.0, 0.5, 2.0)` → `-1.5` (símbolo no viable).
- `shared_vocabulary_ratio(&[1,2,3], &[2,3,4])` → `2/3 ≈ 0.667`.
- `shared_vocabulary_ratio(&[], &[1,2])` → `0.0`.
- `communication_efficiency(0.8, 10.0, 0.5)` → `≈ 5.33`.
- `grammar_complexity(8, 1.0)` → `ln(8) ≈ 2.08`.
- Test: entidad A con símbolo X, B sin él → B aprende X si fitness > 0.
- Test: vocabulario lleno → no aprende más símbolos.
- Test: símbolo ya conocido → no duplicado.
- `cargo test --lib` sin regresión.

---

## Referencias

- ET-3 Cultural Transmission — proto-símbolos (behavior_hash)
- ET-14 Institutions — reglas formalizadas como símbolos
- ET-16 Functional Consciousness — requiere lenguaje para planificación abstracta
- Blueprint §T4-2: "Language", symbol emergence equations
