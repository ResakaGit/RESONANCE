# Axiomatic Closure — Cross-Axiom Derived Dynamics

**Nivel:** Diseño de alto nivel (sin implementación)
**Relacionado:** `docs/sprints/AXIOMATIC_CLOSURE/`, `docs/arquitectura/blueprint_axiomatic_closure.md`

---

## 0. El problema

Los 8 axiomas fundacionales están implementados individualmente. Cada uno tiene
componentes, ecuaciones y sistemas. Sin embargo, los axiomas no son independientes:
generan **fenómenos derivados que requieren la composición de dos o más axiomas**.

Esta clase de dinámicas — que llamamos *cross-axiom dynamics* — es precisamente donde
emerge la simulación más rica. Sin ellas, los axiomas coexisten pero no se potencian.

---

## 1. Mapa de dinámicas derivadas ausentes

```
Axioma 3 (Competition) × Axioma 8 (Frequency)
    → interaction magnitude = base × interference_factor
    → IMPL: en catalysis/spells ✓  |  en extracción metabólica ✗

Axioma 8 consecuencia — Entrainment
    → gradual alignment of frequency between interacting systems
    → IMPL: hacia 1 host (Homeostasis) ✓  |  entre vecinos ✗

Axioma 7 (Distance) × Axioma 8 (Frequency)
    → señales pierden coherencia de frecuencia con la distancia
    → IMPL: atenuación de amplitud ✓  |  atenuación de pureza de frecuencia ✗

Axioma 6 (Emergence) × Axioma 8 (Frequency) → Cultura
    → culture = emergent frequency coherence in a group
    → IMPL: ecuaciones de coherencia ✓  |  loop coherencia ↔ imitación ✗

Axioma 3 consecuencia — Cooperation
    → cooperation emerges when E[ΔE|cooperate] > E[ΔE|compete]
    → IMPL: packs estructurales ✓  |  evaluación game-teórica ✗
```

---

## 2. La jerarquía de impacto

### Tier 1 — Físicamente fundamental (bloquean emergencia)

**AC-1: Interference × Metabolic Extraction**

El axioma 3 establece: "interaction magnitude = base × interference_factor".
Esto se aplica en catálisis (spells). No se aplica en:
- Fotosíntesis (extracción del campo de energía ambiental)
- Predación (extracción de presas)
- Ósmosis (difusión entre entidades adyacentes)

Sin esto, dos entidades con frecuencias incompatibles extraen igual que dos resonantes.
Se elimina toda presión de selección de frecuencia en la ecología.

**Lo que esto produce cuando se implementa:**
```
Terra (75 Hz) depredando entidad Terra (75 Hz):
    interference = cos(0) = 1.0  → 100% eficiencia

Terra (75 Hz) depredando entidad Lux (1000 Hz):
    interference ≈ cos(2π × 925 × t) → media ≈ 0  → ~0% eficiencia neta

Consecuencia: depredadores se especializan en la frecuencia de su presa
              o mueren. Nicho ecológico como física, no como regla.
```

**AC-4: Frequency Purity Attenuation with Distance**

El axioma 7 establece que la interacción decrece con la distancia. Esto aplica
a amplitud de señales. La *pureza de frecuencia* (coherencia oscilatoria) también
debe degradar con la distancia.

Sin esto:
- Un depredador puede detectar la frecuencia exacta de una presa a distancia infinita
- El entrainment entre vecinos funciona igual a 1 metro que a 100 metros
- Los horizontes de información son sólo de amplitud, no de identidad

La ecuación: `freq_purity_received(dist) = exp(-dist / λ_coherence)`

Un receptor que percibe una señal degradada sólo puede inferir la frecuencia con
incertidumbre proporcional a la distancia. Esto fuerza proximidad para interacciones
de alta fidelidad.

### Tier 2 — Emergencia social (bloquean cultura y alianzas)

**AC-2: Entrainment System**

El axioma 8 establece que los sistemas en interacción alinean gradualmente sus
frecuencias. Actualmente `Homeostasis` alinea una entidad hacia su *host* de entorno,
pero no entre vecinos.

Sin este sistema, las frecuencias son estáticas tras el spawn. Toda entidad Terra
permanece Terra para siempre. No hay deriva cultural, no hay entidades que crucen
bandas por exposición prolongada, no hay coevolución de frecuencia entre depredador
y presa.

Modelo de Kuramoto simplificado:
```
dω_i/dt = Σ_j K × sin(ω_j - ω_i) × entrainment_weight(dist_ij, qe_j)

donde:
    K                   = coupling strength (∝ qe de la entidad que "tira")
    entrainment_weight  = decrece con distancia (AC-4 integrado)
    |ω_j - ω_i| < gap   = condición Kuramoto para que haya acoplamiento
```

**AC-3: Culture Coherence Loop**

Las ecuaciones de cultura son comprehensivas (`emergence/culture.rs`). Calculan
coherencia de grupo, tasa de síntesis interna, resiliencia de patrón, fase cultural
(Gas/Liquid/Solid). Pero este valor nunca retroalimenta el sistema de transmisión
de memes.

La ecuación `should_imitate()` usa fitness de extracción como criterio. No penaliza
imitar a entidades fuera de banda de frecuencia. No bonifica imitar a entidades
con alta coherencia de grupo.

Cerrar este loop:
```
p_imitate(target) = base_p × fitness_ratio × frequency_affinity(self, target)
                  × group_coherence_bonus(target_group)
```

Con esto, la cultura como coherencia de frecuencia es auto-reforzante: grupos con
alta coherencia se copian más entre sí → mayor coherencia → mayor presión de copia.
Emerge segregación cultural sin programarla.

### Tier 3 — Game theory (alto impacto, alta complejidad)

**AC-5: Cooperation Emergence**

El axioma 3 establece la condición de cooperación:
```
cooperate when E[ΔE_A | cooperate] > E[ΔE_A | compete]
```

Los packs actuales se forman por fórmula (misma facción, proximidad). No evalúan
si la cooperación es energéticamente ventajosa. Un depredador solitario y un pack
de tres siempre se forman si están cerca, sin considerar si el pool objetivo los
sustenta a todos.

Implementar esto requiere:
- Estimar `extraction_solo` vs `extraction_in_group` para cada entidad por tick
- Evaluar `is_symbiosis_stable()` para cada par candidato
- Trigger de alianza/deserción por cambio de condición

---

## 3. La cadena de desbloqueo

```
Sin AC-1 (interference on extraction):
    → La frecuencia no tiene presión de selección ecológica
    → AC-2 (entrainment) produce movimiento de frecuencia sin consecuencias
    → AC-5 (cooperation) no puede calcular costos reales de extracción mixta

Sin AC-4 (frequency attenuation):
    → AC-2 (entrainment) puede actuar a distancia arbitraria
    → Los horizontes de información son incompletos
    → Las señales de frecuencia no degradan realistamente

Sin AC-2 (entrainment):
    → AC-3 (culture loop) calcula coherencia de un conjunto estático
    → No hay deriva cultural dinámica
    → La cultura es observable pero no activa

Sin AC-3 (culture loop):
    → La coherencia de grupo no retroalimenta comportamiento
    → La cultura no selecciona ni presiona

Sin AC-5 (cooperation):
    → Los grupos son estructurales, no game-teóricos
    → No hay deserción endógena ni alianzas emergentes
```

Orden recomendado de implementación:
```
Onda A: AC-1 + AC-4   (paralelos, sin dependencias)
Onda B: AC-2          (requiere AC-4 para range correcto)
Onda C: AC-3          (requiere AC-2 para coherencia dinámica)
Onda D: AC-5          (requiere AC-1 para costos reales)
```

---

## 4. Invariantes que deben preservarse

Del blueprint axiomático, estos invariantes deben mantenerse tras implementar
cada cross-axiom dynamic:

```
De Axioma 4 (Dissipation):
    Aplicar interference_factor en extracción NO puede resultar en extracción < 0
    → interference_factor debe clampearse a [0, 1] para extracción (no a [-1, 1])
    → interference negativa = cancelación de interacción, no drenaje inverso

De Axioma 5 (Conservation):
    Σ qe(children) ≤ qe(parent) debe preservarse aunque interference module extraction
    → el multiplier reduce lo que se toma, no lo que hay disponible

De Axioma 8:
    interference_factor para catálisis (spells) puede ser [-1, 1] (daño y curación)
    interference_factor para extracción metabólica es [0, 1] (acceso vs no acceso)
    Son dos semánticas distintas del mismo coseno

De Axioma 7 (Distance):
    Atenuación de pureza de frecuencia debe ser monotónica decreciente con distancia
    → a distancia 0: purity = 1.0  (frecuencia exacta)
    → a distancia ∞: purity → 0   (frecuencia irreconocible)
    → λ_coherence < λ_amplitude?  posible — coherencia se pierde antes que la señal
```

---

## 5. Referencia a implementación

- Sprint track: `docs/sprints/AXIOMATIC_CLOSURE/`
- Runtime contracts: `docs/arquitectura/blueprint_axiomatic_closure.md`
- Ecuaciones existentes: `src/blueprint/equations/emergence/culture.rs`, `src/blueprint/equations/signal_propagation.rs`, `src/blueprint/equations/core_physics/mod.rs`
- Sistemas relacionados: `src/simulation/reactions.rs` (catalysis), `src/simulation/metabolic/` (extraction), `src/layers/oscillatory.rs`, `src/layers/homeostasis.rs`
