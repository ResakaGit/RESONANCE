# Sprint AC-3 — Culture Coherence Loop

**Módulo:** `src/blueprint/equations/emergence/culture.rs` (extensión), `src/simulation/metabolic/social_communication.rs`
**Tipo:** Ecuaciones puras (2 funciones) + modificación de un sistema existente
**Eje axiomático:** Axioma 6 × Axioma 8 → cultura como coherencia de frecuencia emergente
**Estado:** 🔒 Requiere AC-2
**Oleada:** C

---

## Contexto: qué ya existe

**Lo que SÍ existe — y está bien diseñado:**

- `blueprint/equations/emergence/culture.rs` — comprehensive:
  - `group_frequency_coherence(&[f32])` → `1 - CV(frequencies)` ∈ [0, 1]
  - `internal_synthesis_rate()` → fracción de pares constructivos en el grupo
  - `pattern_resilience()` → ratio post/pre-perturbación
  - `cultural_phase()` → Gas | Liquid | Solid según coherencia
  - `culture_emergent()` → AND gate de 5 condiciones (coherencia, síntesis, resiliencia, tamaño, conectividad)
  - `entrainment_possible()` → condición Kuramoto

- `simulation/metabolic/social_communication.rs` — `cultural_transmission_system`:
  - Escanea entidades con `CulturalMemory`, busca vecinos, copia memes por fitness
  - `should_imitate(meme, adopter_energy, model_energy, threshold)` → bool
  - `MemeAdoptedEvent` emitido correctamente

**El gap:**

`should_imitate()` usa **solo fitness de extracción** como criterio. No usa frecuencia.
Un meme se propaga si el portador extrae más que el receptor — independientemente de
si están en la misma banda. El resultado: la cultura se propaga sin fricción cross-cultural.

Con AC-2 (entrainment) activo, los grupos ahora tienen coherencia dinámica medible.
Este sprint cierra el loop: **alta coherencia → más fácil imitar**.

---

## Objetivo

Modificar `should_imitate()` para incluir dos factores adicionales:

1. **Afinidad de frecuencia:** La probabilidad de imitar decrece si las frecuencias difieren.
   Un organismo Terra no imita fácilmente a un Lux, incluso si el Lux extrae más.

2. **Bonus de coherencia de grupo:** Los grupos con alta coherencia interna son más
   "contagiosos" — sus memes se propagan con mayor facilidad entre sí.

```
p_imitate = base_fitness_ratio
          × frequency_imitation_affinity(self_freq, model_freq)
          × group_coherence_imitation_bonus(group_frequencies)
          > IMITATION_THRESHOLD
```

---

## Responsabilidades

### AC-3A: Ecuaciones puras

```rust
// src/blueprint/equations/emergence/culture.rs — agregar al módulo existente

/// Afinidad de frecuencia para imitación cultural.
/// Alta afinidad entre entidades de la misma banda → imitación fluida.
/// Baja afinidad entre bandas distintas → fricción cultural.
/// Ecuación: exp(-|Δfreq| / BAND_WIDTH × DECAY_FACTOR)
/// Rango: (0.0, 1.0] donde 1.0 = frecuencias idénticas.
pub fn frequency_imitation_affinity(self_freq: f32, model_freq: f32) -> f32 {
    let normalized_gap = (model_freq - self_freq).abs() / CULTURE_FREQ_BAND_WIDTH;
    (-normalized_gap * CULTURE_FREQ_AFFINITY_DECAY).exp()
}

/// Bonus multiplicativo a imitación basado en coherencia del grupo del modelo.
/// Grupos coherentes son más "contagiosos" — sus memes se propagan más fácil.
/// coherence: resultado de group_frequency_coherence() ∈ [0, 1]
/// Rango del bonus: [1.0, 1.0 + COHERENCE_IMITATION_MAX_BONUS]
pub fn group_coherence_imitation_bonus(coherence: f32) -> f32 {
    1.0 + coherence.clamp(0.0, 1.0) * CULTURE_COHERENCE_IMITATION_BONUS
}
```

### AC-3B: Constantes

```rust
// src/blueprint/constants/ — agregar a constants de cultura (o nuevo shard culture.rs)

/// Ancho de banda "efectivo" para el cálculo de afinidad cultural.
/// Un gap de BAND_WIDTH reduce la afinidad a exp(-DECAY_FACTOR).
/// Default: 100 Hz — dentro de una banda (~50 Hz ancho) la afinidad es alta.
pub const CULTURE_FREQ_BAND_WIDTH: f32 = 100.0;

/// Velocidad de caída de afinidad con el gap de frecuencia.
pub const CULTURE_FREQ_AFFINITY_DECAY: f32 = 2.0;

/// Máximo bonus de coherencia sobre la probabilidad de imitación.
/// Con coherencia=1.0: la probabilidad base se multiplica por (1 + MAX_BONUS).
pub const CULTURE_COHERENCE_IMITATION_BONUS: f32 = 0.5;
```

### AC-3C: Modificación del sistema de transmisión cultural

```rust
// src/simulation/metabolic/social_communication.rs
// En cultural_transmission_system, donde se evalúa should_imitate:

// ANTES (simplificado):
if should_imitate(meme, adopter_energy.qe(), model_energy.qe(), IMITATION_THRESHOLD) {
    adopter_memory.adopt(meme);
    events.write(MemeAdoptedEvent { adopter, model });
}

// DESPUÉS — extender la condición:
let freq_affinity = culture_eq::frequency_imitation_affinity(
    adopter_osc.frequency_hz(),
    model_osc.frequency_hz(),
);

// Recolectar frecuencias del grupo del modelo para coherencia
// (limitado a CULTURE_MAX_GROUP_SAMPLE entidades para no ser O(n²) sin bounds)
let group_freqs: [f32; CULTURE_MAX_GROUP_SAMPLE] = collect_group_frequencies(
    model, &pack_members, &all_oscillators,
);
let coherence = culture_eq::group_frequency_coherence(&group_freqs);
let coherence_bonus = culture_eq::group_coherence_imitation_bonus(coherence);

// Fitness ratio existente × nuevos factores
let adjusted_fitness_ratio = fitness_ratio * freq_affinity * coherence_bonus;

if adjusted_fitness_ratio > IMITATION_THRESHOLD {
    adopter_memory.adopt(meme);
    events.write(MemeAdoptedEvent { adopter, model });
}
```

**Constante adicional:**
```rust
pub const CULTURE_MAX_GROUP_SAMPLE: usize = 12;  // máximo de entidades del grupo para sample de coherencia
```

---

## No hace

- No crea nuevo componente de "cultura" — `CulturalMemory` y `MemeAdoptedEvent` ya existen.
- No cambia la estructura del meme ni del `CulturalMemory`.
- No implementa fricción cultural activa (penalización) — solo modula probabilidad pasiva.
- No requiere que `group_frequency_coherence()` se cachee — se calcula on-demand con
  array fijo de sample.

---

## Criterios de aceptación

### AC-3A (Ecuaciones)

```
frequency_imitation_affinity(75.0, 75.0)     → 1.0       (misma frecuencia)
frequency_imitation_affinity(75.0, 100.0)    → > 0.6     (misma banda aprox)
frequency_imitation_affinity(75.0, 450.0)    → < 0.01    (bandas distintas)

group_coherence_imitation_bonus(0.0)         → 1.0       (sin bonus)
group_coherence_imitation_bonus(1.0)         → 1.0 + MAX_BONUS
group_coherence_imitation_bonus(0.5)         → 1.0 + 0.5 × MAX_BONUS
```

### AC-3C (Sistema)

Test (MinimalPlugins + AC-2 activo):
- Dos grupos Terra (alta coherencia interna): memes se propagan rápido dentro del grupo.
- Un agente Terra intenta imitar a un Lux (alta extracción): `adjusted_fitness_ratio` reducido por `freq_affinity ≈ 0` → no imita.
- Grupo de Terra + 1 Aqua intruso: memes del grupo Terra no se propagan al Aqua incluso si son más fit.

### General

- `cargo test --lib` sin regresión.
- `cultural_transmission_system` sigue pasando sus tests previos.
- Sin Vec allocation nueva (group_freqs como array fijo).

---

## Lo que emerge

Tras AC-1 + AC-2 + AC-3:

1. **Especiación cultural:** Los grupos que cohabitan se sincronizan en frecuencia (AC-2).
   Una vez sincronizados, sus memes se propagan más fácil entre ellos (AC-3). Grupos
   geográficamente separados divergen en frecuencia → divergen en cultura → fricción
   al contacto.

2. **Auto-refuerzo cultural:** Alta coherencia → más imitación → más coherencia.
   Este loop positivo produce que la cultura "cristaliza" — un grupo que alcanza
   coherencia alta es muy estable.

3. **Barreras culturales sin programar:** Un Terra y un Lux no comparten cultura
   aunque convivan, porque la afinidad de frecuencia es casi 0. La segregación
   cultural no fue diseñada — emerge de la física.

---

## Dependencias

- AC-2 — Entrainment system (las frecuencias deben ser dinámicas para que la coherencia evolucione)
- `blueprint/equations/emergence/culture.rs` — `group_frequency_coherence()`, `should_imitate()` (extensión)
- `simulation/metabolic/social_communication.rs` — `cultural_transmission_system` (modificación)
- `layers/oscillatory.rs` — `frequency_hz()` getter

---

## Referencias

- `src/blueprint/equations/emergence/culture.rs` — todas las ecuaciones de cultura existentes
- `src/simulation/metabolic/social_communication.rs` — `cultural_transmission_system`, `should_imitate()`
- `docs/design/AXIOMATIC_CLOSURE.md §3 Tier 2 AC-3` — Culture coherence design
- Axioma 6: "behavior of a system at scale N is consequence of interactions at scale N-1"
- Axioma 8: "Culture: emergent frequency coherence in a group — not programmed, derived from entrainment"
