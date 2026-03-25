# BLUEPRINT — Especificación Técnica de Resonance

---

## 1. Filosofía de Diseño

Resonance no modela un juego con stats tradicionales. Modela una **simulación termodinámica** donde las reglas de juego (MOBA) son una capa de meta-contexto encima de la física.

La unidad fundamental es el **quanto de energía** (`qe`). Todo lo demás — vida, daño, curación, elementos, estados de materia, maná — son derivaciones matemáticas de `qe`, frecuencia y geometría espacial.

### Principio Ortogonal

Las **14 capas** (L0 BaseEnergy — L13 StructuralLink) son **ortogonales**: cada una puede existir independientemente. Una entidad puede tener cualquier subconjunto de capas. El comportamiento emerge de las **interacciones cruzadas** entre capas, procesadas por los sistemas ECS.

> **Nota (2026-03):** Este documento fue escrito con 10 capas originales. Las capas L10–L13 (ResonanceLink, TensionField, Homeostasis, StructuralLink) se añadieron en V4. La tabla canónica de 14 capas está en `CLAUDE.md`.

```
Entidad = Σ(Capas seleccionadas)
Comportamiento = f(Capa_i × Capa_j × ... × tiempo)
```

---

## 2. Mapa de Módulos

```
src/
├── capas/
│   ├── mod.rs                         Re-exporta los 10 componentes
│   ├── capa0_energia_base.rs          EnergiaBase { qe }
│   ├── capa1_volumen_espacial.rs      VolumenEspacial { radio }
│   ├── capa2_firma_oscilatoria.rs     FirmaOscilatoria { frecuencia_hz, fase }
│   ├── capa3_vector_flujo.rs          VectorFlujo { velocidad, tasa_disipacion }
│   ├── capa4_coherencia_materia.rs    CoherenciaMateria { estado, eb, conductividad }
│   ├── capa5_motor_alquimico.rs       MotorAlquimico { buffer, max, entrada, salida }
│   ├── capa6_presion_ambiental.rs     PresionAmbiental { delta_qe, viscosidad }
│   ├── capa7_actuador_voluntad.rs     ActuadorVoluntad { intencion, canalizando }
│   ├── capa8_inyector_alquimico.rs    InyectorAlquimico { qe_proy, freq_forzada, radio }
│   └── capa9_identidad_moba.rs        IdentidadMoba { faccion, tags, mult_critico }
│
├── sistemas/
│   ├── mod.rs                         ConjuntoSimulacion (SystemSets)
│   ├── voluntad_entrada.rs            Input → ActuadorVoluntad
│   ├── presion_entorno.rs             Bioma → EnergiaBase, VectorFlujo
│   ├── motor_procesamiento.rs         EnergiaBase ↔ MotorAlquimico
│   ├── disipacion.rs                  Entropía: VectorFlujo → EnergiaBase
│   ├── movimiento.rs                  ActuadorVoluntad → VectorFlujo → Transform
│   ├── colision_interferencia.rs      Interferencia de ondas entre pares
│   ├── transiciones_estado.rs         Densidad → EstadoMateria
│   ├── resolucion_catalisis.rs        InyectorAlquimico → objetivos
│   └── identidad_faccion.rs           Facción, puntuación, despawn
│
├── bundles/
│   ├── mod.rs
│   ├── heroe.rs                       Capas 0–9 (todas)
│   ├── proyectil.rs                   Capas 0,1,2,3,8
│   ├── cristal.rs                     Capas 0,1,2,4,5
│   ├── celda_bioma.rs                 Capas 0,1,6
│   └── hechizo.rs                     Capas 0,1,2,3,8
│
├── blueprint/
│   ├── mod.rs
│   ├── ecuaciones.rs                  Funciones puras fn(f32...) → f32
│   ├── constantes.rs                  Todos los valores de tuning
│   └── tablas_elemento.rs             Bandas de frecuencia elemental
│
├── eventos/
│   ├── mod.rs
│   ├── colision_evento.rs             ColisionEvento
│   ├── transicion_fase_evento.rs      TransicionFaseEvento
│   ├── catalisis_evento.rs            CatalisisEvento
│   └── muerte_evento.rs               MuerteEvento, CausaMuerte
│
├── plugins/
│   ├── mod.rs
│   ├── capas_plugin.rs                Registro de tipos Reflect
│   ├── simulacion_plugin.rs           Sistemas + ordenamiento
│   └── debug_plugin.rs                Gizmos + marcador
│
├── lib.rs                             Declaración del árbol de módulos
└── main.rs                            Entry point + setup_mundo
```

---

## 3. Especificación por Capa

---

### CAPA 0 — EnergiaBase (El Cuanto)

**Módulo:** `src/capas/capa0_energia_base.rs`

**Propósito:** La existencia pura. Define cuánta "sustancia" tiene una entidad antes de darle forma o comportamiento. Es el equivalente termodinámico de HP: cuando `qe = 0`, la entidad deja de existir.

**Componente:**
```rust
pub struct EnergiaBase {
    pub qe: f32,  // Quanta de energía (Joules mágicos)
}
```

**Valor base:** `qe = 100.0`

**Invariante:** `qe >= 0.0` — clampeado en todo sistema que lo modifique.

**Métodos clave:**
- `drenar(cantidad) → f32` — resta energía, nunca por debajo de 0, retorna cuánto drenó realmente
- `inyectar(cantidad)` — suma energía
- `esta_muerto() → bool` — `qe <= 0.0`

**Interacciones cruzadas:**
| Con Capa | Relación |
|----------|----------|
| 1 | `densidad = qe / volumen` — la densidad es derivada, no almacenada |
| 3 | La disipación drena `qe` cada tick |
| 5 | El motor sifona `qe` al buffer de maná |
| 6 | La presión ambiental inyecta o drena `qe` |
| 8 | Los hechizos transfieren `qe` entre entidades |

---

### CAPA 1 — VolumenEspacial (El Espacio)

**Módulo:** `src/capas/capa1_volumen_espacial.rs`

**Propósito:** Contenedor espacial de la energía. Define el radio de colisión y permite calcular la densidad (cantidad derivada).

**Componente:**
```rust
pub struct VolumenEspacial {
    pub radio: f32,  // Radio en unidades de mundo
}
```

**Valor base:** `radio = 1.0` (esfera unitaria)

**Ecuación fundamental:**
```
V = (4/3) × π × r³
ρ = qe / V
```

**Métodos clave:**
- `volumen() → f32` — calcula (4/3)πr³
- `densidad(qe) → f32` — calcula ρ = qe / V

**Interacciones cruzadas:**
| Con Capa | Relación |
|----------|----------|
| 0 | `ρ = qe / V` — densidad derivada |
| 3 | El arrastre depende de la densidad: `F_drag = -0.5 × visc × ρ × |v| × v` |
| 4 | La temperatura equivalente se calcula desde la densidad: `T = ρ / k` |
| 8 | Define el radio de colisión para impacto de hechizos |

---

### CAPA 2 — FirmaOscilatoria (El Tiempo y la Forma)

**Módulo:** `src/capas/capa2_firma_oscilatoria.rs`

**Propósito:** La energía concentrada oscila. La frecuencia define el "elemento" y la fase determina la alineación. Es el corazón del sistema de combate: dos entidades interactúan constructiva o destructivamente según su relación de ondas.

**Componente:**
```rust
pub struct FirmaOscilatoria {
    pub frecuencia_hz: f32,  // Frecuencia primaria (Hz)
    pub fase: f32,           // Desplazamiento de fase [0, 2π)
}
```

**Valor base:** `frecuencia_hz = 1000.0` (Lux/Neutral), `fase = 0.0`

**Ecuación fundamental — Interferencia:**
```
I(a, b, t) = cos(2π × |f_a - f_b| × t + (φ_a - φ_b))
```

**Interpretación del resultado:**
| Valor de I | Significado | Efecto en gameplay |
|------------|-------------|-------------------|
| I → +1.0 | Constructiva | Resonancia, amplificación, curación |
| I → -1.0 | Destructiva | Daño, aniquilación, oposición |
| I → 0.0 | Ortogonal | Sin interacción significativa |

**Tabla de Elementos (bandas de frecuencia):**
| Elemento | Freq Min | Freq Max | Freq Central | Color RGB |
|----------|----------|----------|-------------|-----------|
| Umbra | 10 | 30 | 20 | (0.15, 0.0, 0.3) |
| Terra | 50 | 100 | 75 | (0.55, 0.35, 0.1) |
| Aqua | 200 | 300 | 250 | (0.0, 0.4, 0.9) |
| Ignis | 400 | 500 | 450 | (1.0, 0.3, 0.0) |
| Ventus | 600 | 800 | 700 | (0.7, 0.95, 0.7) |
| Lux | 900 | 1100 | 1000 | (1.0, 1.0, 0.8) |

**Pureza elemental:** Qué tan centrada está la frecuencia dentro de su banda.
```
pureza = 1.0 - |freq - freq_central| / (rango / 2)
```
Pureza 1.0 = centro exacto. Pureza 0.0 = borde de banda.

**Ventaja elemental (ciclo):**
```
Ignis > Ventus > Terra > Aqua > Ignis   (×1.5 / ×0.7)
Lux ↔ Umbra                              (×2.0 mutuamente)
Mismo vs Mismo                            (×0.5 resistencia)
```

**Gaps entre bandas** (ej. 100–200 Hz): frecuencias que no pertenecen a ningún elemento. Son "Vacío" — entidades híbridas o inestables.

---

### CAPA 3 — VectorFlujo (El Flujo y la Entropía)

**Módulo:** `src/capas/capa3_vector_flujo.rs`

**Propósito:** La Segunda Ley de la Termodinámica del juego. Toda energía tiende a disiparse y moverse. Define velocidad y la tasa de sangrado entrópico.

**Componente:**
```rust
pub struct VectorFlujo {
    pub velocidad: Vec2,        // Dirección y rapidez (unidades/s)
    pub tasa_disipacion: f32,   // qe perdido por segundo en vacío
}
```

**Valor base:** `velocidad = (0,0)`, `tasa_disipacion = 5.0`

**Ecuaciones fundamentales:**

1. **Integración de posición:**
```
posición += velocidad × dt
```

2. **Disipación efectiva (entropía + fricción cinética):**
```
d_eff = d_base + COEF_FRICCION × |v|²
qe -= d_eff × dt
```

3. **Fuerza de arrastre (del terreno):**
```
F_drag = -0.5 × viscosidad × ρ × |v| × v
```

4. **Integración de velocidad:**
```
v_new = v_old + (F_total / masa_efectiva) × dt
masa_efectiva = qe  (la energía actúa como inercia)
```

**Constantes:** `COEF_FRICCION = 0.01`, `VELOCIDAD_MAXIMA_GLOBAL = 50.0`

**Interacciones cruzadas:**
| Con Capa | Relación |
|----------|----------|
| 0 | La disipación drena qe. qe actúa como masa para la inercia |
| 1 | La densidad escala el arrastre |
| 4 | El estado de materia limita la velocidad (Sólido=0, Líquido≤5, Gas/Plasma=ilimitado) |
| 6 | La viscosidad del terreno multiplica el arrastre |
| 7 | La intención de voluntad produce fuerza que modifica velocidad |

---

### CAPA 4 — CoherenciaMateria (La Coherencia Estructural)

**Módulo:** `src/capas/capa4_coherencia_materia.rs`

**Propósito:** El "freno" a la disipación. Es el contenedor que resiste el estrés de las Capas 2 y 3. Si la energía interna supera la energía de enlace, la entidad cambia de estado o colapsa.

**Componente:**
```rust
pub enum EstadoMateria { Solido, Liquido, Gas, Plasma }

pub struct CoherenciaMateria {
    pub estado: EstadoMateria,
    pub energia_enlace_eb: f32,       // Resistencia a cambiar de estado
    pub conductividad_termica: f32,   // [0.0, 1.0] cuánto calor deja pasar
}
```

**Valor base (Roca):** `estado = Solido`, `eb = 5000.0`, `conductividad = 0.2`

**Ecuación de transición de fase:**
```
T_equiv = ρ / k_boltzmann_juego

T < 0.3 × Eb  →  Sólido
T < 1.0 × Eb  →  Líquido
T < 3.0 × Eb  →  Gas
T ≥ 3.0 × Eb  →  Plasma
```

**Efectos por estado:**
| Estado | Vel. Máx | Mult. Disipación | Comportamiento |
|--------|----------|-----------------|----------------|
| Sólido | 0.0 (fijado) | ×0.2 | Alto daño colisión, baja entropía |
| Líquido | 5.0 | ×0.5 | Fluye alrededor de obstáculos |
| Gas | Ilimitado | ×1.5 | Atraviesa sólidos |
| Plasma | Ilimitado | ×3.0 | Máximo daño, emite radiación (Capa 8) |

**Constantes:** `K_BOLTZMANN_JUEGO = 1.0`, `TRANSICION_SOLIDO = 0.3`, `TRANSICION_LIQUIDO = 1.0`, `TRANSICION_GAS = 3.0`

**Interacciones cruzadas:**
| Con Capa | Relación |
|----------|----------|
| 0+1 | Densidad → temperatura → transición de fase |
| 3 | Estado limita velocidad. Estado multiplica disipación |
| 8 | Interferencia destructiva debilita `eb` (factor 0.05) |
| Colisiones | `conductividad_termica` escala la transferencia de energía entre entidades |

---

### CAPA 5 — MotorAlquimico (El Motor Abierto)

**Módulo:** `src/capas/capa5_motor_alquimico.rs`

**Propósito:** Transforma la materia muerta en un sistema procesador. Es un **capacitor** entre el campo de energía bruto (Capa 0) y el uso activo de habilidades (Capa 8). Introduce el concepto de "Maná" separado de la estructura.

**Componente:**
```rust
pub struct MotorAlquimico {
    pub buffer_actual: f32,     // Maná acumulado
    pub buffer_maximo: f32,     // Capacidad antes de sobrecarga
    pub valvula_entrada: f32,   // Tasa de absorción (qe/s)
    pub valvula_salida: f32,    // Tasa de expulsión (qe/s) — cast speed
}
```

**Valor base:** `buffer = 0.0`, `max = 1000.0`, `entrada = 10.0`, `salida = 50.0`

**Ciclo por tick:**
```
intake = min(valvula_entrada × dt, qe_disponible, buffer_maximo - buffer_actual)
qe -= intake
buffer_actual += intake
```

**Arquetipos de motor:**
| Perfil | buffer_max | entrada | salida | Estilo de juego |
|--------|-----------|---------|--------|----------------|
| Burst | Grande | Pequeña | Grande | Ráfaga potente, recuperación lenta |
| Sustain | Pequeño | Grande | Pequeña | Flujo constante pero débil |
| Tank | Medio | Grande | Pequeña | Absorción pasiva, poca ofensiva |
| Glass Cannon | Pequeño | Pequeña | Enorme | Un disparo devastador, luego indefenso |

**Sobrecarga:** Si `buffer_actual > buffer_maximo × 1.5`, la entidad explota (MuerteEvento::Sobrecarga).

**Interacciones cruzadas:**
| Con Capa | Relación |
|----------|----------|
| 0 | Sifona qe del campo al buffer |
| 7 | La potencia del actuador escala con buffer/max |
| 8 | El buffer financia el lanzamiento de hechizos |
| 6 | La presión ambiental puede inyectar qe al campo (que luego el motor absorbe) |

---

### CAPA 6 — PresionAmbiental (Topología Macroscópica)

**Módulo:** `src/capas/capa6_presion_ambiental.rs`

**Propósito:** Modificadores ambientales a gran escala. Cada celda de bioma aplica presión constante sobre las entidades que la pisan.

**Componente:**
```rust
pub struct PresionAmbiental {
    pub delta_qe_constante: f32,   // qe/s inyectado (positivo) o robado (negativo)
    pub viscosidad_terreno: f32,   // Multiplicador de fricción (1.0 = neutral)
}
```

**Valor base (Llanura):** `delta_qe = 0.0`, `viscosidad = 1.0`

**Biomas predefinidos:**
| Bioma | delta_qe | viscosidad | Efecto |
|-------|---------|-----------|--------|
| Llanura | 0.0 | 1.0 | Neutral |
| Volcán | -5.0 | 2.0 | Drena energía, terreno espeso |
| Línea Ley | +10.0 | 0.5 | Inyecta energía, terreno fluido |
| Pantano | -1.0 | 3.0 | Drena poco, muy viscoso |
| Tundra | -2.0 | 1.5 | Drena moderado, algo viscoso |
| Desierto | -3.0 | 1.2 | Drena bastante, algo viscoso |

**Aplicación por tick:**
```
entidad.qe += bioma.delta_qe_constante × dt
entidad.velocidad *= 1.0 / (1.0 + (viscosidad - 1.0) × dt)
```

**Espacialidad:** Se aplica cuando `distancia(entidad, bioma) < bioma.radio + entidad.radio`. Las entidades tipo bioma llevan el marcador `MarcadorBioma` para excluirse de la interacción.

---

### CAPA 7 — ActuadorVoluntad (El Actuador)

**Módulo:** `src/capas/capa7_actuador_voluntad.rs`

**Propósito:** La voluntad inyectada en el motor. Puente entre input (teclado/IA) y la simulación física. NO modifica velocidad directamente — produce una **fuerza**.

**Componente:**
```rust
pub struct ActuadorVoluntad {
    pub intencion_movimiento: Vec2,    // Vector normalizado del input
    pub canalizando_habilidad: bool,   // true mientras canaliza
    pub habilidad_id: Option<u32>,     // ID de la habilidad en canalización
}
```

**Valor base:** `intencion = (0,0)`, `canalizando = false`

**Ecuación de fuerza de voluntad:**
```
eficiencia = buffer_actual / buffer_maximo    (clamped [0, 1])
F_voluntad = intención × POTENCIA_MOTOR_BASE × eficiencia
```

**Constante:** `POTENCIA_MOTOR_BASE = 100.0`

**Restricciones:**
- Mientras `canalizando_habilidad = true`, el movimiento se suprime
- La salida del MotorAlquimico se redirige al InyectorAlquimico durante canalización
- Entidades en estado Sólido (Capa 4) ignoran la fuerza de voluntad

**Input mapping (sistema `voluntad_entrada`):**
- WASD / Flechas → `intencion_movimiento` (normalizado para que diagonal no sea más rápida)
- Espacio → `canalizando_habilidad`
- Solo afecta entidades con tag `Heroe`

---

### CAPA 8 — InyectorAlquimico (El Gestor de Reacciones Emergentes)

**Módulo:** `src/capas/capa8_inyector_alquimico.rs`

**Propósito:** El evento de alteración de la realidad. No es un componente pasivo — es un **arquetipo dinámico** que se spawea como entidad independiente cuando se lanza una habilidad.

**Componente:**
```rust
pub struct InyectorAlquimico {
    pub qe_proyectado: f32,       // Energía a forzar en el objetivo
    pub frecuencia_forzada: f32,  // Elemento que intenta imponer
    pub radio_influencia: f32,    // Radio del área de efecto
}
```

**Valor base (Ataque Básico):** `qe = 50.0`, `freq = 1000.0`, `radio = 0.5`

**Ciclo de vida de un hechizo:**
```
1. Héroe canaliza → MotorAlquimico.buffer drena qe_proyectado
2. Se spawea entidad hechizo con: EnergiaBase, FirmaOscilatoria, VectorFlujo, InyectorAlquimico
3. El hechizo vive como objeto físico independiente (sujeto a todas las capas)
4. Al superponerse con un objetivo, se resuelve la catálisis
5. Cuando qe → 0, el hechizo se despawnea
```

**Resolución de catálisis:**
```
I = interferencia(f_hechizo, φ_hechizo, f_objetivo, φ_objetivo, t)

Si I > UMBRAL_CONSTRUCTIVO (0.5):
    objetivo.qe += qe_proyectado × I × mult_critico
    objetivo.freq_hz += (freq_forzada - objetivo.freq_hz) × 0.1    ← resonance lock

Si I < UMBRAL_DESTRUCTIVO (-0.5):
    objetivo.qe -= qe_proyectado × |I|
    objetivo.eb *= (1.0 + I × 0.05)                                ← debilitamiento

Si |I| > UMBRAL_CRITICO (0.9):
    resultado × multiplicador_critico                              ← golpe crítico
```

**Presets elementales:**
| Elemento | Frecuencia | Factory |
|----------|-----------|---------|
| Ignis | 450.0 | `InyectorAlquimico::fuego(qe, radio)` |
| Aqua | 250.0 | `InyectorAlquimico::hielo(qe, radio)` |
| Terra | 75.0 | `InyectorAlquimico::tierra(qe, radio)` |
| Ventus | 700.0 | `InyectorAlquimico::viento(qe, radio)` |
| Umbra | 20.0 | `InyectorAlquimico::sombra(qe, radio)` |
| Lux | 1000.0 | `InyectorAlquimico::luz(qe, radio)` |

---

### CAPA 9 — IdentidadMoba (Reglas de MOBA)

**Módulo:** `src/capas/capa9_identidad_moba.rs`

**Propósito:** El "Juego" sobre la simulación. Etiquetas arbitrarias que interceptan las matemáticas físicas para aplicar reglas de diseño. NO participa en física.

**Componente:**
```rust
pub enum Faccion { Neutral, Roja, Azul, Salvaje }
pub enum TagRelacional { Aliado, Enemigo, Recurso, Estructura, Invocacion, Heroe, Minion, Jungla }

pub struct IdentidadMoba {
    pub faccion: Faccion,
    pub tags_relacionales: Vec<TagRelacional>,
    pub multiplicador_critico: f32,
}
```

**Valor base:** `faccion = Neutral`, `tags = []`, `mult_critico = 1.0`

**Lógica de facción:**
```
Aliados:   misma facción, ambas no-neutral → modificador +0.2 (bonus constructivo)
Enemigos:  distinta facción, ambas no-neutral → modificador -0.2 (bonus destructivo)
Neutral:   sin modificador
```

El modificador se suma al valor de interferencia (Capa 2) y se clampea a [-1, 1]. Esto significa que aliados tienden a curarse mutuamente y enemigos tienden a dañarse.

**Tags relacionales:** Filtros de targeting para habilidades. Ejemplo: "curar solo Aliado+Heroe", "dañar solo Enemigo+Estructura".

**Marcador de puntuación:** Recurso global `Marcador { puntos_roja, puntos_azul, bajas_roja, bajas_azul }`.

---

### CAPA 7b — Grimoire (Extensión del Actuador)

**Módulo:** `src/capa/voluntad.rs` (co-localizado con ActuadorVoluntad)

**Propósito:** Catálogo de habilidades disponibles para una entidad con voluntad. NO es una capa separada — es una extensión de Capa 7 que define QUÉ puede hacer la voluntad, a qué costo, y con qué elemento.

**Componente:**
```rust
pub struct Grimoire {
    pub slots: [AbilitySlot; 4],  // 4 slots de habilidad (QWER)
}

pub struct AbilitySlot {
    pub cost_qe: f32,              // Buffer drenado al castear
    pub forced_frequency: f32,     // Elemento del hechizo (Hz)
    pub influence_radius: f32,     // Radio de efecto
    pub min_buffer_threshold: f32, // Buffer mínimo requerido para activar
    pub apply_effect: Option<EffectRecipe>,  // Si aplica efecto temporal (Capa 10)
}
```

**Valor base (Ataque Básico):** `cost = 30.0`, `freq = 1000.0`, `radius = 0.5`, `threshold = 30.0`

**Cooldown emergente:**

El cooldown NO es un timer explícito. Emerge de la Capa 5 (Motor):

```
cooldown_natural = cost_qe / input_valve

Ejemplo MagoFuego:
  cost_qe = 80, input_valve = 8 qe/s
  cooldown = 80 / 8 = 10 segundos

Ejemplo AsesinoPlantar:
  cost_qe = 40, input_valve = 20 qe/s
  cooldown = 40 / 20 = 2 segundos
```

El jugador puede lanzar la habilidad cuando `buffer_actual >= min_buffer_threshold`. Si el motor absorbe más rápido (por bioma de Línea Ley o buff), el cooldown se acorta naturalmente.

**Activación:**
```
1. Jugador presiona Q (slot 0)
2. Sistema verifica: buffer >= slot.min_buffer_threshold
3. Si sí: ActuadorVoluntad.channeling = true, ability_id = 0
4. Motor drena: buffer -= slot.cost_qe
5. Se spawea hechizo con: freq = slot.forced_frequency, radius = slot.influence_radius
6. Si slot.apply_effect es Some → se spawea entidad-efecto (Capa 10)
```

**Presets por clase de héroe:**

| Clase | Slot 0 (Q) | Slot 1 (W) | Slot 2 (E) | Slot 3 (R) |
|-------|-----------|-----------|-----------|-----------|
| MagoFuego | Bola de fuego (80qe, 450Hz, r=0.8) | Muro de fuego (120qe, 450Hz, r=3.0) | Sprint ígneo (40qe, effect:Haste) | Explosión (200qe, 450Hz, r=5.0) |
| TanqueAgua | Oleada (60qe, 250Hz, r=2.0) | Escudo (50qe, effect:Shield) | Maremoto (100qe, 250Hz, r=4.0) | Tsunami (180qe, 250Hz, r=6.0) |
| AsesinoPlantar | Daga umbra (40qe, 20Hz, r=0.3) | Blink (30qe, teleport) | Veneno (35qe, effect:Poison) | Ejecución (150qe, 20Hz, r=0.5) |

**Interacciones cruzadas:**
| Con Capa | Relación |
|----------|----------|
| 5 (Motor) | `buffer >= threshold` para activar. `cost_qe` se drena del buffer. Cooldown = `cost / input_valve`. |
| 7 (Voluntad) | `channeling` y `ability_id` referencian el slot activo. |
| 8 (Inyector) | Los parámetros del slot configuran el hechizo spawneado. |
| 10 (Enlace) | `apply_effect` define la entidad-efecto que se spawea al impactar. |

---

### CAPA 10 — EnlaceResonancia (Efectos Temporales como Entidades)

**Módulo:** `src/capa/enlace.rs`

**Propósito:** Modificación temporal de las propiedades de otra entidad. Un efecto temporal no es un "debuff con timer" — es una **entidad Tipo B** cuya energía sostiene la modificación. Cuando su `qe → 0` (por disipación natural), la modificación desaparece.

**Tipo de capa:** Tipo B — la entidad-efecto ES el enlace. Tiene su propio ciclo de vida energético.

**Componente:**
```rust
pub struct ResonanceLink {
    pub target: Entity,               // Entidad cuya propiedad se modifica
    pub modified_field: ModifiedField, // Qué propiedad se multiplica
    pub magnitude: f32,               // Factor multiplicativo (0.5 = mitad, 2.0 = doble)
}

pub enum ModifiedField {
    VelocityMultiplier,      // Capa 3: velocidad máxima efectiva
    DissipationMultiplier,   // Capa 3: tasa de sangrado entrópico
    BondEnergyMultiplier,    // Capa 4: resistencia a cambio de fase (shield)
    ConductivityMultiplier,  // Capa 4: transferencia de calor en colisión
    MotorIntakeMultiplier,   // Capa 5: velocidad de absorción de qe
    MotorOutputMultiplier,   // Capa 5: velocidad de lanzamiento
}
```

**Valor base (efecto genérico):** `magnitude = 1.0` (sin efecto)

**Nivel en el árbol:** Nivel 2 (Acción), Tipo B.

**Dependencias en el árbol:**
```
Capa 0 (EnergiaBase)    → El qe del efecto es su "combustible". Cuando se agota, muere.
Capa 3 (VectorFlujo)    → La disipación del efecto determina su duración.
```

**Ecuación de duración:**
```
duración_base = qe_inicial / tasa_disipación

Ejemplo Slow:
  qe = 30, disipación = 10 qe/s
  duración = 30 / 10 = 3 segundos

Ejemplo Shield:
  qe = 50, disipación = 10 qe/s
  duración = 50 / 10 = 5 segundos
```

**Ecuación de aplicación:**

El sistema `resonance_link_system` aplica los modificadores activos cada frame:

```
Para cada entidad-efecto con ResonanceLink:
  Si target existe Y efecto.qe > QE_MIN:
    property_efectiva(target) = property_base(target) × magnitude
  Si efecto.qe <= QE_MIN:
    La entidad-efecto muere → modificación desaparece
```

**Acumulación de efectos:**

Múltiples efectos del mismo `ModifiedField` sobre el mismo `target` se MULTIPLICAN:

```
Slow ×0.5 + Slow ×0.7 = velocidad × 0.5 × 0.7 = velocidad × 0.35
Shield ×2.0 + Shield ×1.5 = bond_energy × 2.0 × 1.5 = bond_energy × 3.0
```

**Ciclo de vida completo:**
```
1. CREACIÓN
   Hechizo impacta al target.
   Catálisis resuelve el daño/curación normal.
   Si el slot tiene apply_effect → se spawea entidad-efecto:
     - EnergiaBase(qe_efecto)           ← combustible
     - VectorFlujo(vel=ZERO, disip)     ← tasa de decaimiento, no se mueve
     - ResonanceLink(target, field, mag) ← qué modifica y cuánto

2. VIDA
   El efecto pierde qe cada tick por disipación (como toda entidad).
   La propiedad del target se multiplica por magnitude.
   El efecto es INVISIBLE al target (no ocupa espacio, sin VolumenEspacial).
   El efecto NO colisiona (sin VolumenEspacial → fuera del SpatialIndex).

3. INTERACCIONES
   Si el target está en Línea Ley (Capa 6, +qe/s):
     → El efecto dura MÁS (la ley line inyecta qe al efecto? No — el efecto
        no tiene PresionAmbiental. Los biomas solo afectan entidades con
        VolumenEspacial. Los efectos son inmunes a biomas.)

   Si alguien lanza un hechizo CONSTRUCTIVO al efecto:
     → El efecto gana qe → dura más.
     Pero: el efecto no tiene VolumenEspacial, así que no es targeteable
     por colisión directa. Solo puede ser afectado por:
     - Habilidades de "purge" que buscan entidades-efecto por target
     - Interferencia destructiva dirigida

4. DESTRUCCIÓN
   qe → 0 por disipación → entidad muere → modificación desaparece.
   O: sistema de "purge/cleanse" drena todo el qe del efecto de golpe.
```

**Presets de efectos:**

| Efecto | ModifiedField | Magnitude | qe | Disipación | Duración | Elemento |
|--------|---------------|-----------|-----|------------|----------|----------|
| Slow (Hielo) | VelocityMultiplier | 0.5 | 30 | 10 | 3.0s | Aqua (250 Hz) |
| Haste (Viento) | VelocityMultiplier | 1.5 | 40 | 10 | 4.0s | Ventus (700 Hz) |
| Shield (Tierra) | BondEnergyMultiplier | 2.0 | 50 | 10 | 5.0s | Terra (75 Hz) |
| Weaken (Sombra) | BondEnergyMultiplier | 0.3 | 25 | 5 | 5.0s | Umbra (20 Hz) |
| Mana Drain | MotorIntakeMultiplier | 0.0 | 20 | 10 | 2.0s | Umbra (20 Hz) |
| Cast Speed | MotorOutputMultiplier | 2.0 | 30 | 10 | 3.0s | Lux (1000 Hz) |
| Poison (DoT) | DissipationMultiplier | 3.0 | 40 | 8 | 5.0s | Umbra (20 Hz) |
| Armor | ConductivityMultiplier | 0.2 | 60 | 10 | 6.0s | Terra (75 Hz) |

**Interacciones cruzadas:**
| Con Capa | Relación |
|----------|----------|
| 0 | El qe del efecto es su combustible. Drenar el qe del efecto lo destruye (purge). |
| 3 | La disipación del efecto determina su duración natural. `vel = ZERO` siempre. |
| 3 (target) | `VelocityMultiplier` escala la velocidad máxima del target. |
| 3 (target) | `DissipationMultiplier` escala la disipación del target (poison = más entropía). |
| 4 (target) | `BondEnergyMultiplier` escala la energía de enlace (shield/weaken). |
| 4 (target) | `ConductivityMultiplier` escala la conductividad (armor). |
| 5 (target) | `MotorIntakeMultiplier` escala la absorción de qe (mana drain). |
| 5 (target) | `MotorOutputMultiplier` escala la velocidad de lanzamiento (cast speed). |
| 7b (Grimoire) | El slot define `apply_effect` que spawea la entidad-efecto al impactar. |
| 8 | La catálisis del hechizo original spawea el efecto como paso adicional. |

---

### CAPA 11 — CampoTension (Fuerzas a Distancia)

**Módulo:** `src/capa/campo_tension.rs`

**Propósito:** Aplicar aceleración continua a entidades vecinas sin colisión directa, combinando masa efectiva (`qe`) e interferencia oscilatoria para atracción/repulsión.

**Tipo de capa:** Tipo A — vive en la entidad emisora del campo.

**Componente:**
```rust
pub struct CampoTension {
    pub radius: f32,
    pub gravity_gain: f32,
    pub magnetic_gain: f32,
    pub falloff_mode: FieldFalloffMode,
}
```

**Contrato matemático:**
```
F_grav ∝ (qe_emisor × qe_objetivo) / dist²
F_mag  ∝ interference(f_emisor, p_emisor, f_objetivo, p_objetivo, t)
F_total = gravity_gain * F_grav + magnetic_gain * F_mag
```

**Reglas duras:**
- Clamp de distancia mínima para evitar singularidades.
- Salida siempre finita (`!NaN`, `!Inf`).
- Sin vecinos en radio: costo cercano a cero.

**Interacciones cruzadas:**
| Con Capa | Relación |
|----------|----------|
| 0 (EnergiaBase) | Usa `qe` como masa efectiva para magnitud gravitatoria. |
| 2 (FirmaOscilatoria) | Usa `interference()` para sesgo atractivo/repulsivo. |
| 3 (VectorFlujo) | Acumula fuerza y curva trayectoria. |
| 1 (VolumenEspacial) | Usa distancia/radio para vecindad y atenuación. |

---

### CAPA 12 — HomeostasisFrecuencial (Adaptación Energética)

**Módulo:** `src/capa/homeostasis.rs`

**Propósito:** Adaptar gradualmente `frequency_hz` hacia una banda estable del entorno, con costo energético explícito.

**Tipo de capa:** Tipo A — capacidad de autorregulación en runtime.

**Componente:**
```rust
pub struct HomeostasisFrecuencial {
    pub adapt_rate_hz: f32,
    pub qe_cost_per_hz: f32,
    pub stability_band_hz: f32,
    pub enabled: bool,
}
```

**Contrato matemático:**
```
delta_hz = clamp(target_hz - current_hz, adapt_rate_hz * dt)
qe_cost  = abs(delta_hz) * qe_cost_per_hz

si qe >= qe_cost:
  frequency_hz += delta_hz
  qe -= qe_cost
si qe < qe_cost:
  no hay adaptación parcial (determinismo)
```

**Reglas duras:**
- Sin presupuesto energético, no adapta.
- Sin presión hostil sostenida, no hay drift permanente.
- Clamp + histeresis para evitar jitter.

**Interacciones cruzadas:**
| Con Capa | Relación |
|----------|----------|
| 0 (EnergiaBase) | Drena `qe` por cada Hz ajustado. |
| 2 (FirmaOscilatoria) | Escribe `frequency_hz` de manera acotada. |
| 6 (PresionAmbiental) | El entorno define la presión que dispara adaptación. |
| 5 (Motor) | Compite por presupuesto energético total de la entidad. |

---

### CAPA 13 — EnlaceEstructural (Topología Multicuerpo)

**Módulo:** `src/capa/enlace_estructural.rs`

**Propósito:** Unir entidades con restricción física tipo resorte y permitir ruptura determinista bajo estrés.

**Tipo de capa:** Tipo B — vínculo explícito entre nodos.

**Componente:**
```rust
pub struct EnlaceEstructural {
    pub target: Entity,
    pub rest_length: f32,
    pub stiffness: f32,
    pub break_stress: f32,
}
```

**Contrato matemático:**
```
F_resorte = stiffness * (dist - rest_length)
stress += abs(dist - rest_length) + carga_termica_compartida

si stress > break_stress:
  emitir StructuralLinkBreakEvent
  remover enlace
```

**Reglas duras:**
- No fusiona entidades: conserva individualidad por capas.
- Bajo carga extrema, ruptura determinista.
- No bypass de ecuaciones inline en sistemas.

**Interacciones cruzadas:**
| Con Capa | Relación |
|----------|----------|
| 1 (VolumenEspacial) | Mide separación real entre nodos. |
| 3 (VectorFlujo) | Aplica fuerza de restricción por frame. |
| 0 (EnergiaBase) | Puede transferir carga/energía entre nodos enlazados. |
| 6 (PresionAmbiental) | El estrés térmico del medio acelera ruptura. |

---

### Percepción (Sistema, NO capa)

**Módulo:** `src/simulacion/pre_fisica.rs` (sistema) + `src/mundo/percepcion.rs` (resource)

**Propósito:** Determinar qué entidades son visibles para cada facción. NO es una capa porque se DERIVA de capas existentes (0, 2) y distancia.

**Principio energético:** Toda entidad con `FirmaOscilatoria` emite una onda. La intensidad de esa emisión depende de su energía (`qe`) y su frecuencia. La detección ocurre cuando la señal supera un umbral.

**Ecuación de señal:**
```
visibility(freq) = tabla de visibilidad por elemento
signal = source.qe × visibility(source.freq) / distance²
detected = signal > DETECTION_THRESHOLD
```

**Tabla de visibilidad por frecuencia:**
| Elemento | Rango Hz | Visibilidad | Razón |
|----------|----------|-------------|-------|
| Umbra | 10-30 | 0.1 | Oscilación lenta, casi indetectable |
| Terra | 50-100 | 0.3 | Vibración baja, discreta |
| Aqua | 200-300 | 0.5 | Emisión moderada |
| Ignis | 400-500 | 0.7 | Emisión alta, energía radiante |
| Ventus | 600-800 | 0.8 | Oscilación rápida, fácil de sentir |
| Lux | 900-1100 | 1.0 | Máxima emisión, imposible de ocultar |

**Constantes:**
| Constante | Valor | Uso |
|-----------|-------|-----|
| `DETECTION_THRESHOLD` | 0.5 | Señal mínima para detección |
| `MAX_VISION_RADIUS` | 30.0 | Distancia máxima de percepción (cap) |

**Resource:**
```rust
pub struct PerceptionCache {
    visible_by_faction: HashMap<Faction, HashSet<Entity>>,
}

impl PerceptionCache {
    pub fn is_visible(&self, faction: Faction, target: Entity) -> bool;
    pub fn visible_enemies(&self, faction: Faction) -> &HashSet<Entity>;
    pub fn visible_allies(&self, faction: Faction) -> &HashSet<Entity>;
}
```

**Sistema:**
```
perception_system (Phase::PrePhysics):
  Para cada entidad con FirmaOscilatoria + MobaIdentity:
    Para cada entidad cercana (SpatialIndex.query_radius(MAX_VISION)):
      signal = target.qe × visibility(target.freq) / dist²
      Si signal > DETECTION_THRESHOLD:
        Marcar target como visible para la facción del perceiver

  Visión compartida: aliados comparten lo que ven.
```

**Gameplay emergente:**
| Mecánica | Cómo emerge |
|----------|-------------|
| Stealth natural de Umbra | `visibility(20Hz) = 0.1` → 10× más difícil de detectar que Lux |
| Ward/Centinela | Entidad con `qe` alto + `freq` Lux → detecta en radio grande |
| Arbusto | Bioma con sistema que reduce `qe` aparente de entidades dentro (no la qe real, solo para el cálculo de percepción) |
| Revelar | Hechizo Lux constructivo que inyecta qe al target → más detectable |
| Fade (desaparecer) | Drenar tu propio qe temporalmente (peligroso — te acercás a la muerte) |

---

## 4. Plano Ortogonal — Matriz de Interacciones

Esta es la tabla maestra que define qué capa afecta a qué capa y a través de qué ecuación.

```
        C0    C1    C2    C3    C4    C5    C6    C7    C8    C9
  C0    ·     ρ     ·     ←d    ·     ↔i    ←δ    ·     ←qe   ·
  C1    ρ     ·     ·     drag  T→    ·     col   ·     col   ·
  C2    ·     ·     I()   ·     ·     ·     ·     ·     cat   ±fac
  C3    d→    drag  ·     ·     lim   ·     visc  F←    ·     ·
  C4    ·     T←    ·     lim   ·     ·     ·     sol   deb   ·
  C5    i↔    ·     ·     ·     ·     ·     δ→    pot   →qe   ·
  C6    δ→    col   ·     visc  ·     δ→    ·     ·     ·     ·
  C7    ·     ·     ·     F→    sol   pot   ·     ·     cast  ·
  C8    qe→   col   cat   ·     deb   qe←   ·     cast  ·     crit
  C9    ·     ·     ±fac  ·     ·     ·     ·     ·     crit  ·
```

**Leyenda:**
| Símbolo | Significado |
|---------|-------------|
| `ρ` | Densidad = qe / volumen |
| `d→` / `←d` | Disipación drena qe |
| `I()` | Función de interferencia cos(2π\|Δf\|t + Δφ) |
| `drag` | Fuerza de arrastre -0.5·visc·ρ·\|v\|·v |
| `T→` / `T←` | Temperatura equivalente → transición de fase |
| `lim` | Estado de materia limita velocidad |
| `i↔` / `↔i` | Motor sifona qe al buffer y viceversa |
| `δ→` / `←δ` | Presión ambiental inyecta/drena qe |
| `visc` | Viscosidad del terreno modifica arrastre |
| `F→` / `F←` | Fuerza de voluntad → velocidad |
| `sol` | Sólido bloquea movimiento |
| `pot` | Eficiencia del motor escala fuerza |
| `cat` | Resolución de catálisis (hechizo → objetivo) |
| `deb` | Debilitamiento de enlace por interferencia destructiva |
| `cast` | Canalización: motor → inyector |
| `qe→` / `←qe` | Transferencia directa de energía |
| `col` | Radio de colisión |
| `±fac` | Modificador de facción al signo de interferencia |
| `crit` | Multiplicador crítico escala resultado |

---

## 5. Grafo de Dependencias entre Sistemas

```
                    ┌──────────────┐
                    │   ENTRADA    │
                    │              │
                    │ voluntad_    │
                    │ entrada      │
                    └──────┬───────┘
                           │
                    ┌──────▼───────┐
                    │  FISICA_PRE  │
                    │              │
                    │ presion_     │
                    │ entorno      │
                    │              │
                    │ motor_       │
                    │ procesamiento│
                    └──────┬───────┘
                           │
                    ┌──────▼───────┐
                    │ FISICA_CORE  │
                    │              │
                    │ disipacion   │
                    │              │
                    │ movimiento   │
                    │              │
                    │ colision_    │
                    │ interferencia│
                    └──────┬───────┘
                           │
                    ┌──────▼───────┐
                    │  REACCIONES  │
                    │              │
                    │ transiciones_│
                    │ estado       │
                    │              │
                    │ resolucion_  │
                    │ catalisis    │
                    └──────┬───────┘
                           │
                    ┌──────▼───────┐
                    │ FISICA_POST  │
                    │              │
                    │ identidad_   │
                    │ faccion      │
                    │ (despawn,    │
                    │  marcador)   │
                    └──────────────┘
```

**Orden estricto:** `Entrada → FisicaPre → FisicaCore → Reacciones → FisicaPost`

Dentro de cada conjunto, los sistemas son **paralelos** (Bevy los puede ejecutar concurrentemente si no hay conflictos de datos).

---

## 6. Arquetipos de Entidad

### Héroe (Capas 0–9, todas)

El único arquetipo con las 10 capas. Es el ensamblaje completo.

| Clase | qe | radio | freq | eb | buffer_max | entrada | salida | disip | crit |
|-------|----|-------|------|----|-----------|---------|--------|-------|------|
| MagoFuego | 500 | 0.8 | 450 | 2000 | 1500 | 8 | 80 | 3 | 1.8 |
| GuerreroTierra | 800 | 1.2 | 75 | 8000 | 500 | 15 | 30 | 2 | 1.5 |
| AsesinoPlantar | 300 | 0.5 | 20 | 1500 | 800 | 20 | 100 | 8 | 2.5 |
| SanadorLuz | 400 | 0.9 | 1000 | 3000 | 2000 | 12 | 60 | 2 | 1.5 |
| TiradorViento | 350 | 0.7 | 700 | 2500 | 1000 | 10 | 70 | 4 | 1.5 |
| TanqueAgua | 1000 | 1.5 | 250 | 10000 | 300 | 5 | 20 | 1 | 1.5 |

### Proyectil (Capas 0, 1, 2, 3, 8)

Ligero, balístico. Sin coherencia (energía pura), sin motor (no regenera), sin voluntad (trayectoria fija). Alta disipación para que decaiga.

### Cristal (Capas 0, 1, 2, 4, 5)

Nodo de recurso pasivo. Sólido, acumula energía lentamente. Puede ser "minado" por interferencia constructiva.

### Celda de Bioma (Capas 0, 1, 6)

Tile estático con presión ambiental. Radio grande. Lleva `MarcadorBioma`.

### Hechizo AoE (Capas 0, 1, 2, 3, 8)

Spawneado al lanzar habilidad. Puede ser estático (zona) o viajero (proyectil). Corta vida por alta disipación. Lleva `MarcadorHechizo { caster }`.

---

## 7. Eventos

| Evento | Campos | Emitido por |
|--------|--------|-------------|
| `ColisionEvento` | entity_a, entity_b, interferencia, qe_transferido | `colision_interferencia` |
| `TransicionFaseEvento` | entity, estado_anterior, estado_nuevo | `transiciones_estado` |
| `CatalisisEvento` | caster, target, spell, interferencia, qe_aplicado | `resolucion_catalisis` |
| `MuerteEvento` | entity, causa | `disipacion`, `motor_procesamiento`, `resolucion_catalisis` |

**Causas de muerte:**
- `Disipacion` — entropía natural
- `Destruccion` — interferencia destructiva
- `Aniquilacion` — destrucción mutua
- `ColapsoEstructural` — enlace roto
- `Sobrecarga` — explosión de maná

---

## 8. Constantes de Tuning

Todas centralizadas en `src/blueprint/constantes.rs`:

| Constante | Valor | Uso |
|-----------|-------|-----|
| `COEF_FRICCION` | 0.01 | Escala disipación con \|v\|² |
| `K_BOLTZMANN_JUEGO` | 1.0 | Convierte densidad → temperatura |
| `TRANSICION_SOLIDO` | 0.3 | Umbral T/eb para sólido |
| `TRANSICION_LIQUIDO` | 1.0 | Umbral T/eb para líquido |
| `TRANSICION_GAS` | 3.0 | Umbral T/eb para gas/plasma |
| `UMBRAL_CONSTRUCTIVO` | 0.5 | I > esto → efecto constructivo |
| `UMBRAL_DESTRUCTIVO` | -0.5 | I < esto → efecto destructivo |
| `UMBRAL_CRITICO` | 0.9 | \|I\| > esto → golpe crítico |
| `FACTOR_RESONANCIA_LOCK` | 0.1 | Velocidad de convergencia de freq |
| `FACTOR_DEBILITAMIENTO_ENLACE` | 0.05 | Tasa de debilitamiento de eb |
| `POTENCIA_MOTOR_BASE` | 100.0 | Fuerza base del actuador |
| `VELOCIDAD_MAXIMA_GLOBAL` | 50.0 | Cap anti-exploit |
| `FACTOR_SOBRECARGA` | 1.5 | buffer > max×1.5 → explosión |
| `QE_MINIMO_EXISTENCIA` | 0.01 | Debajo de esto, la entidad muere |

---

## 9. Secuencia de Implementación Recomendada

**Fase 1 (Completada):** Esqueleto — archivos, componentes, sistemas vacíos, compila.

**Fase 2:** Rendering básico — sprites/meshes para visualizar entidades, cámara seguimiento.

**Fase 3:** Gameplay loop — input completo, lanzamiento de habilidades, spawning de hechizos.

**Fase 4:** IA básica — sistema de decisión para entidades no-jugador.

**Fase 5:** Balance — ajustar constantes, agregar inspector en runtime.

**Fase 6:** Red/Multijugador — serialización de estado, rollback.
