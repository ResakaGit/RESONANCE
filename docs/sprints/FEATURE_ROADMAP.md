# Feature Roadmap — Casos de Uso Curados

> 94K LOC · 2567+ tests · 55 sprints (46 ✅) · 128-node radial field · 8 axiomas · 4 constantes
>
> Última actualización: 2026-03-28

Casos de uso filtrados por viabilidad, impacto, y respeto axiomático.
Cada uno incluye: qué es, quién lo usa, cómo se implementa sin hardcode, y qué ya existe.

**Estado global:** 13/16 implementables hoy. 3 necesitan 1-2 semanas. 1 es ciencia abierta.
**Tracks activos:** GAMEPLAY_SYSTEMS (6 sprints), SURVIVAL_MODE (3 sprints, diseñado).

---

## A. GENERAR GANANCIAS

### A1. Versus Arena — "Mi ecosistema vs el tuyo"

**Qué:** Dos jugadores evolucionan criaturas independientemente (seeds distintas).
Se encuentran en arena neutral. Gana la facción con más qe total al final.

**Quién paga:** Gamers competitivos. El loop es: evolucionar (preparar) → versus (competir) → iterar.

**Monetización:** Seasons con leaderboard. Packs de seeds curadas. Skins de arena (mapas RON).

**Axiomas:** Sin cambio. Facciones en `MobaIdentity`. Victoria por qe total. Genomes via `bridge.rs`.

**Ya existe:**
- Batch evolution completo
- `genome_to_components()` round-trip
- `VictoryNucleus` + `victory_check_system`
- Facciones en L9
- 27 mapas

**Falta:** Lobby (cargar 2 archivos .bin), asignar facciones, pantalla de resultado.

**Esfuerzo:** 1 semana.

---

### A2. Laboratorio de Universos — "¿Cómo sería la vida si...?"

**Qué:** Presets que cambian las constantes fundamentales. Cada preset es un universo alternativo.

- **"Júpiter"** — GRAVITY × 5 → criaturas anchas y bajas
- **"Marte"** — GRAVITY × 0.1 + SOLAR_FLUX × 0.5 → criaturas altas y frágiles
- **"Snowball Earth"** — SEASON_AMPLITUDE = 0.9 → solo sobreviven los con reservas
- **"Bombardeo Tardío"** — ASTEROID_INTERVAL = 200 → extinción constante → reproducción rápida
- **"Edén"** — DISSIPATION × 0.1 → todo vive mucho → sobrepoblación → competencia extrema

**Quién paga:** Educadores, museos, curiosos. Es un "Universe Sandbox" biológico.

**Monetización:** App de pago ($5-15). Expansiones con presets nuevos. API para escuelas.

**Axiomas:** Zero código nuevo. Cada preset es una combinación de constantes existentes en `BatchConfig`.

**Ya existe:** Todas las constantes en `batch/constants.rs`. Solo falta UI de selección.

**Esfuerzo:** 3 días (presets) + 1 semana (UI).

---

### A3. Marketplace de Criaturas — "Evoluciona, vende, colecciona"

**Qué:** Cada `GenomeBlob` tiene un hash único determinista. Criaturas evolucionadas durante
miles de generaciones son raras y valiosas. Los usuarios las evolucionan, comparten, intercambian.

**Quién paga:** Coleccionistas, gamers, NFT market (si se quiere ir por ahí).

**Axiomas:** `GenomeBlob.hash()` ya es determinista. La forma emerge del genome — no se puede falsificar.

**Ya existe:**
- `save_genomes` / `load_genomes`
- `GenomeBlob.hash()` → u64 único
- `creature_builder` → mesh preview

**Falta:** Plataforma web, gallery UI, verificación de autenticidad (seed + gens → reproduce el hash).

**Esfuerzo:** 2-4 semanas (backend + frontend).

---

### A4. Survival Mode — "Sobrevive como una criatura evolucionada"

**Qué:** El jugador elige un genome evolucionado, entra al mundo como esa criatura.
Tiene que sobrevivir: comer, evitar depredadores, reproducirse.
Muere → game over. Score = ticks sobrevividos.

**Quién paga:** Gamers casual. Es un roguelike biológico.

**Axiomas:** El player solo controla `will_intent`. La física decide si sobrevive.

**Ya existe:**
- `InputCommand::MoveToward`
- `WillActuator` (L7)
- `PlayerControlled` marker
- Todo el pipeline de simulación
- Evolución batch para poblar el mundo

**Falta:** `apply_input()` wiring (vacío hoy). Death → game over screen. Score display.

**Sprint track:** [SURVIVAL_MODE](SURVIVAL_MODE/) — 3 sprints diseñados:
- SV-1: Input wiring (5 LOC en `sim_world.rs`) — desbloqueado
- SV-2: Survival binary (`src/bin/survival.rs`) — bloqueado por SV-1
- SV-3: Game over (death + score + restart) — bloqueado por SV-2

**Esfuerzo:** 1 semana.

---

## B. INVESTIGACIÓN

### B1. Paper Machine — "Hipótesis → Experimento → Datos → Publicación"

**Qué:** Workflow para investigadores:
1. Definir hipótesis: "la gravedad alta produce criaturas más anchas"
2. Configurar: `GRAVITY = 0.5` vs `GRAVITY = 5.0`
3. Correr: 500 worlds × 200 gens × 3000 ticks (cada condición)
4. Exportar: CSV con generation, fitness, morphology metrics
5. Comparar: distribución de biases entre condiciones
6. Publicar: gráficos reproducibles desde seed

**Quién lo usa:** Biólogos computacionales, physicists, ALife researchers.

**Axiomas:** El simulador ES el instrumento. Los axiomas son las "leyes del universo" del paper.

**Ya existe:**
- Batch harness con `GenerationStats` por generación
- Determinismo (INV-4) — reproducibilidad perfecta
- Conservation audit (2567 tests)

**Falta:** Export CSV automático de `harness.history`. Plotting tool (o export a Python/R).

**Esfuerzo:** 2 días.

---

### B2. Aislamiento Reproductivo — "¿Emerge especiación sin programarla?"

**Qué:** Crear un mapa con una barrera geográfica (zona sin nutrientes en el medio).
Dos poblaciones evolucionan aisladas. Después de N generaciones, quitar la barrera.
¿Las dos poblaciones son incompatibles (frecuencias divergieron)?

**Quién lo usa:** Biólogos evolutivos. Es el experimento de especiación alopátrica in silico.

**Axiomas:** Axioma 8 — las frecuencias divergen naturalmente si no hay entrainment.
Axioma 3 — interference entre frecuencias divergentes es destructiva → incompatibilidad.

**Ya existe:**
- `nutrient_grid` — se puede vaciar una franja para crear barrera
- `entrainment` — sincronización por proximidad
- `interference()` — mide compatibilidad

**Falta:** Mapa con barrera + medición de divergencia de frecuencia post-barrera.

**Esfuerzo:** 1 día (mapa custom) + 1 día (métrica de divergencia).

---

### B3. Explosión Cámbrica — "¿Qué condiciones la disparan?"

**Qué:** Medir la tasa de innovación morfológica (nuevos peaks, nuevos aspect ratios)
por generación. ¿Hay un "punto de inflexión" donde la diversidad explota?

**Quién lo usa:** Paleobiólogos, complexity scientists.

**Axiomas:** Axioma 6 — emergence at scale. La explosión no se programa — se detecta.

**Ya existe:**
- `detect_peaks()` — cuenta features morfológicos
- `peak_aspect_ratio()` — clasifica formas
- `FitnessReport.species_count` + `diversity`
- `harness.history` — tracking por generación

**Falta:** Métrica de "innovación" = nuevos archetypes morfológicos que no existían en gen anterior.

**Esfuerzo:** 3 días (métrica + análisis).

---

## C. VISTOSO

### C1. Fossil Record — "Timeline visual de 500 generaciones"

**Qué:** Barra horizontal. Cada punto es una generación. El usuario desliza y ve
cómo la forma del organismo dominante cambia gradualmente. Morphing visual en tiempo real.

**Quién lo impresiona:** Todos. Es la imagen que resume todo el proyecto en un GIF.

**Axiomas:** Puro rendering de datos existentes. No afecta física.

**Ya existe:**
- `harness.history` — stats por generación
- `creature_builder` — mesh desde genome

**Falta:** Guardar top genome por generación (1 GenomeBlob extra por gen) + slider UI + mesh interpolation.

**Esfuerzo:** 1 semana.

---

### C2. Petri Dish — "Zoom al campo interno de una criatura"

**Qué:** Click en una entidad → overlay que muestra su campo radial 16×8 como heatmap animado.
Los peaks se forman en vivo. Los joints aparecen como valles oscuros. Las appendages
brillan como protuberancias del heatmap.

**Quién lo impresiona:** Científicos, artistas, curiosos. Es la ventana al "por qué" de cada forma.

**Axiomas:** Solo visualización de `entity.qe_field`. No modifica nada.

**Ya existe:** `qe_field: [[f32; RADIAL]; AXIAL]` en cada entidad. 128 valores listos para renderizar.

**Falta:** Overlay 2D (16×8 grid coloreado) sincronizado con el tick.

**Esfuerzo:** 3 días.

---

### C3. Museo Mode — "Proyección de ecosistema en pared"

**Qué:** Fullscreen sin UI. Cámara orbital automática. Evolución continua.
Cada hora, un asteroide. Cada 10 minutos, nueva generación de criaturas.
El público ve un ecosistema vivo evolucionando en tiempo real.

**Quién lo impresiona:** Museos de ciencia, instalaciones artísticas, lobbies de oficinas tech.

**Axiomas:** Es el simulador corriendo sin input. Todo emerge.

**Ya existe:** `evolve_and_view` con cámara orbital. Solo falta loop infinito + fullscreen.

**Esfuerzo:** 1 día.

---

### C4. Mesh Export → Blender — "Formas orgánicas procedurales"

**Qué:** Exportar los meshes GF1 como OBJ/glTF para usar en 3D art, animación, impresión 3D.

**Quién lo impresiona:** Artistas 3D, diseñadores, makers.

**Axiomas:** No afecta. Es serialización de datos existentes.

**Ya existe:** `build_creature_mesh_with_field()` produce `bevy::Mesh` con positions, normals, UVs.

**Falta:** Serializer Mesh → OBJ (vertex + face export). ~50 LOC.

**Esfuerzo:** 1 día.

---

## D. MERAMENTE INTERESANTE

### D1. Time Machine — "¿Qué habría pasado si...?"

**Qué:** Pausar la simulación. Retroceder 100 ticks. Cambiar una constante.
Ver cómo el universo diverge desde ese punto. Dos timelines en paralelo.

**Quién lo usa:** Curiosos, filósofos, estudiantes de causalidad.

**Axiomas:** INV-4 (determinismo). Guardar snapshot → restaurar → replay con constante modificada.
Mismos axiomas, diferente parámetro. Las dos timelines son universos paralelos legítimos.

**Ya existe:**
- `SimWorld::tick()` puro
- Determinismo bit-exact
- `WorldSnapshot` owned
- Checkpoint save/load (SF-5)

**Falta:** UI de timeline (fork point visual, split-screen rendering, slider temporal).

**Esfuerzo:** 2 semanas.

---

### D2. Convergent Evolution Detector — "¿Dos seeds llegaron a la misma solución?"

**Qué:** Correr 100 seeds independientes. Comparar los genomes finales.
¿Emergió la misma morfología en universos diferentes? Eso es convergencia evolutiva —
la solución óptima es un **atractor** del espacio de genomes.

**Quién lo usa:** Complexity scientists, ALife researchers.

**Axiomas:** Puro análisis de datos. `GenomeBlob.distance()` ya mide similitud.

**Ya existe:** Todo. Solo falta un script que corra N seeds y compare.

**Esfuerzo:** 1 día.

---

### D3. Ecosystem as Music — "Cada criatura es un instrumento"

**Qué:** Cada entidad tiene una frecuencia oscilatoria (L2). Sonificar la simulación:
cada entidad emite un tono a su frecuencia. Entities cercanas hacen acordes.
Interference constructiva = consonancia. Destructiva = disonancia.
El ecosistema suena.

**Quién lo usa:** Artistas sonoros, investigadores de sonificación, instalaciones.

**Axiomas:** Axioma 8 — todo oscila a una frecuencia. El sonido es la representación literal del axioma.

**Ya existe:**
- `OscillatorySignature.frequency_hz` en cada entidad
- `interference()` para acordes
- `SpatialIndex` para proximidad

**Falta:** Backend de audio (Bevy audio o `cpal`). Synthesis engine (sine waves + interference).

**Esfuerzo:** 1 semana.

---

### D4. Genome Programming Language — "Define tu propio axioma"

**Qué:** El usuario escribe una ecuación custom que modifica una constante del universo.
Ejemplo: `GRAVITY = sin(tick × 0.01) × 2.0` — gravedad que oscila.
La ecuación se interpreta en runtime y se aplica cada tick.

**Quién lo usa:** Power users, researchers, modders.

**Axiomas:** El usuario extiende los axiomas, no los rompe. La ecuación es un axioma nuevo.

**Ya existe:** Todas las constantes centralizadas. Hot-reload posible.

**Falta:** Parser de expresiones (mini-DSL o Lua embedding). Sandboxing.

**Esfuerzo:** 2-3 semanas.

---

## Matriz resumen

| ID | Caso de uso | Categoría | Esfuerzo | LOC nuevas | Ready? | ¿Genera $? | ¿Paper? | ¿WOW? |
|----|-------------|-----------|----------|-----------|--------|-----------|---------|-------|
| A1 | Versus Arena | Ganancias | 1 día | ~100 | ✅ | ✅ | | ✅ |
| A2 | Lab de Universos | Ganancias | 3 horas | ~50 | ✅ | ✅ | ✅ | ✅ |
| A3 | Marketplace | Ganancias | 3 sem | ~200+ web | ✅ | ✅ | | |
| A4 | Survival Mode | Ganancias | 1 sem | ~200 | 🔧 sprints | ✅ | | ✅ |
| B1 | Paper Machine | Research | medio día | ~30 | ✅ | | ✅ | |
| B2 | Especiación | Research | 1 día | ~50 | ✅ | | ✅ | ✅ |
| B3 | Explosión Cámbrica | Research | 3 días | ~100 | ✅ | | ✅ | ✅ |
| C1 | Fossil Record | Vistoso | 4 días | ~200 | ✅ impl | | | ✅ |
| C2 | Petri Dish | Vistoso | 1 día | ~50 | ✅ impl | | ✅ | ✅ |
| C3 | Museo Mode | Vistoso | 1 día | ~20 | ✅ impl | ✅ | | ✅ |
| C4 | Mesh Export | Vistoso | 1 día | ~80 | ✅ impl | | | ✅ |
| D1 | Time Machine | Interesante | 2 sem | ~500 | 🔧 UI | | ✅ | ✅ |
| D2 | Convergencia | Interesante | medio día | ~30 | ✅ impl | | ✅ | |
| D3 | Ecosystem Music | Interesante | 1 sem | ~300 | ✅ impl | | | ✅ |
| D4 | Genome DSL | Interesante | 3 sem | ~1000+ | ❌ | ✅ | | |

### Costo total estimado

| Tier | Casos | Esfuerzo total | LOC total |
|------|-------|---------------|-----------|
| **✅ Implementados** | A1,A2,B1-B4,C1-C4,D1-D3 | done | ~2200 LOC |
| **🔧 Parcial** | A4 (SV-2/3 pendientes) | ~3 días | ~200 LOC |
| **🔒 Heavy** | A3 (web), D1 (UI), D4 (DSL) | ~9 semanas | ~2000+ LOC |
| **Total 16** | **13 ✅ · 1 🔧 · 2 🔒** | — | — |

> **Estado:** 13/16 use cases funcionales. Faltan: A3 Marketplace (web), A4 Survival (2 sprints), D1 Time Machine (UI heavy), D4 Genome DSL (research).
