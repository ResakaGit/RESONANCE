# AXIOMATIC_CLOSURE — Cross-Axiom Derived Dynamics

**Diseño:** `docs/design/AXIOMATIC_CLOSURE.md`
**Arquitectura:** `docs/arquitectura/blueprint_axiomatic_closure.md`
**Estado:** ⏳ Onda A desbloqueada

Los 8 axiomas fundacionales están individualmente implementados. Este track cierra
las **dinámicas derivadas** — los fenómenos que requieren la composición de dos o más
axiomas y que son responsables de la emergencia más rica de la simulación.

---

## Sprints

| Sprint | Descripción | Eje axiómatico | Esfuerzo | Oleada | Estado |
|--------|-------------|----------------|----------|--------|--------|
| [AC-1](SPRINT_AC1_INTERFERENCE_EXTRACTION.md) | Interference × metabolic extraction | Axioma 3 × 8 | Bajo | A | ⏳ |
| [AC-2](SPRINT_AC2_ENTRAINMENT_SYSTEM.md) | Entrainment system (Kuramoto) | Axioma 8 consecuencia | Medio | B | 🔒 |
| [AC-3](SPRINT_AC3_CULTURE_COHERENCE_LOOP.md) | Culture coherence → imitation loop | Axioma 6 × 8 | Bajo | C | 🔒 |
| [AC-4](SPRINT_AC4_FREQUENCY_ATTENUATION.md) | Frequency purity attenuation with distance | Axioma 7 × 8 | Bajo | A | ⏳ |
| [AC-5](SPRINT_AC5_COOPERATION_EMERGENCE.md) | Cooperation emergence (game theory) | Axioma 3 consecuencia | Alto | D | 🔒 |

---

## Oleadas

### Oleada A — ⏳ Desbloqueada (sin dependencias)

| Sprint | Qué hace | Desbloquea |
|--------|----------|------------|
| AC-1 | Aplica `cos(Δfreq × t)` en fotosíntesis, predación, ósmosis | AC-5 |
| AC-4 | `freq_purity = exp(-d/λ)` en signal propagation | AC-2 |

Los dos pueden ejecutarse en paralelo.

**Por qué AC-1 primero:**
Sin interference en extracción metabólica, la frecuencia de una entidad no tiene
presión de selección ecológica. Es el gap más costoso para la simulación: todos los
sistemas de nicho, coevolución y especialización dependen de que la extracción sea
sensible a la frecuencia.

**Por qué AC-4 primero:**
El entrainment (AC-2) necesita saber a qué distancia opera con alta fidelidad.
Sin AC-4, el radio de coupling del Kuramoto es arbitrario. Con AC-4, se deriva
naturalmente del λ de coherencia de frecuencia.

---

### Oleada B — 🔒 Requiere AC-4

| Sprint | Qué hace | Desbloquea |
|--------|----------|------------|
| AC-2 | Sistema de alignment gradual de frecuencias entre vecinos | AC-3 |

---

### Oleada C — 🔒 Requiere AC-2

| Sprint | Qué hace | Desbloquea |
|--------|----------|------------|
| AC-3 | Cierra el loop: coherencia de grupo modula probabilidad de imitación | — |

---

### Oleada D — 🔒 Requiere AC-1

| Sprint | Qué hace |
|--------|----------|
| AC-5 | Evaluación game-teórica de cooperar vs competir; alianzas y deserción endógenas |

---

## Qué se desbloquea con cada oleada

### Tras AC-1 + AC-4

- **Nicho por frecuencia como física:** Un organismo Terra que intenta predar un Lux
  falla energéticamente. No hay regla — es física.
- **Señales con identidad degradable:** Una señal de frecuencia percibida lejos es borrosa.
  Ya no se puede identificar a un aliado o enemigo a distancia arbitraria.

### Tras AC-2

- **Deriva cultural dinámica:** Los grupos que cohabitan durante muchos ticks convergen
  gradualmente en frecuencia. Emerge especiación por aislamiento geográfico.
- **Coevolución depredador-presa:** Un depredador que siempre caza presas de la misma
  banda tiene presión para converger a esa banda. La presa tiene presión para divergir.
  Es una danza de frecuencias, no un script.

### Tras AC-3

- **Cultura como refuerzo:** Los grupos con alta coherencia de frecuencia se imitan más
  entre sí, lo que aumenta su coherencia, lo que aumenta la imitación. Emerge
  segregación cultural sin programarla.
- **Fricción intercultural:** Un meme de alta eficiencia de extracción se propaga
  rápido dentro de una banda, despacio entre bandas distintas.

### Tras AC-5

- **Alianzas endógenas:** Dos entidades que obtienen más extracción juntas que solas
  forman alianza sin que ningún sistema la "decida". La cooperación es el resultado
  de evaluar costos.
- **Deserción endógena:** Una alianza se disuelve cuando la condición Nash deja de
  cumplirse. No hay lealtad estática.
