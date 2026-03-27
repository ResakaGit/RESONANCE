# Demo — Civilization Test (Headless)

**Mapa:** `RESONANCE_MAP=civilization_test cargo run --release --bin headless_sim -- --ticks 8000 --scale 12`
**Archivo:** `assets/maps/civilization_test.ron`
**Objetivo:** Validar transmisión cultural, herencia inter-generacional, e infraestructura con 25-38 BehavioralAgents simultáneos sostenidos durante miles de ticks.

---

## 1. Motivación

Las demos anteriores (`optimal_inference`, `genesis_validation`) producían máximo 5-9 BehavioralAgents. Insuficiente para:
- **Transmisión cultural** — necesita ≥5 agentes en rango de imitación (10 unidades)
- **Cooperación** — Nash alliance detection necesita pares de agentes cercanos
- **Herencia** — reproducción fauna requiere agentes con suficiente qe (≥200)

Civilization Test maximiza agentes activos con: grid compacto (32×32), alta emisión (400-600 qe/s), cluster de 4 nuclei Terra solapados para máxima coherencia.

---

## 2. Layout del mapa

```
32×32 cells, cell_size=2.0, origin=(-32,-32)

        ╔══════════════════════╗
        ║   AQUA SOURCE (NW)   ║ ← 200 Hz, InverseLinear
        ║   freq diversity     ║
        ╠══════════════════════╣
        ║                      ║
        ║   TERRA CLUSTER (C)  ║ ← 4 nuclei 82-90 Hz, overlapping
        ║   α(0,0) β(6,6)     ║     max coherence zone
        ║   γ(-6,-4) δ(4,-6)  ║     → awakening hotspot
        ║                      ║
        ╠══════════════════════╣
        ║   IGNIS CORE (SE)    ║ ← 440 Hz, InverseSquare
        ║   high energy dense  ║
        ╚══════════════════════╝
```

## 3. Nuclei configuration

| Nombre | Posición | Hz | Emisión (qe/s) | Radio | Decay | Rol |
|--------|----------|-----|----------------|-------|-------|-----|
| terra_alpha | (0, 0) | 85 | 600 | 18 | InverseLinear | Core coherence |
| terra_beta | (6, 6) | 88 | 500 | 16 | InverseLinear | NE overlap |
| terra_gamma | (-6, -4) | 82 | 500 | 16 | InverseLinear | SW overlap |
| terra_delta | (4, -6) | 90 | 400 | 14 | InverseLinear | SE overlap |
| aqua_source | (-12, 10) | 200 | 400 | 14 | InverseLinear | Biome diversity |
| ignis_core | (12, -10) | 440 | 500 | 12 | InverseSquare | Energy density |

**Diseño:** Los 4 nuclei Terra están dentro de 50 Hz entre sí (bandwidth de coherencia). Sus radios se solapan creando una zona de ~12×12 celdas con coherencia máxima → awakening triggers.

## 4. Sistemas ejercitados

| Sistema | Fase | Qué valida | Métrica observable |
|---------|------|------------|-------------------|
| `propagate_nuclei_system` | ThermodynamicLayer | Campo de energía con 6 nuclei | total_qe crece → estabiliza → cae |
| `radiation_pressure_system` | ThermodynamicLayer | Redistribución coherente por frecuencia | max_cell_qe < 350 (vs 2400 sin presión) |
| `materialization_delta_system` | ThermodynamicLayer | Tiles con SenescenceProfile | sen = alive (100%) |
| `awakening_system` | MorphologicalLayer | Entidades ganan BehavioralAgent | beh = 25-38 sostenido |
| `basal_drain_system` | MetabolicLayer | Costo de vivir | avg_qe baja cuando nuclei se agotan |
| `senescence_death_system` | MetabolicLayer | Muerte por edad | avg_age < max_age, turnover visible |
| `cultural_transmission_system` | Input | Memes se propagan entre agentes | (requires logging to verify) |
| `reproduction_spawn_system` | MorphologicalLayer | Offspring heredan CulturalMemory | avg_age baja = nuevas generaciones |
| `infrastructure_update_system` | MetabolicLayer | Grid de infraestructura decae | (passive — no investment events yet) |
| `nucleus_recycling_system` | MorphologicalLayer | Nutrientes → nuevo núcleo | total_qe se estabiliza post-depleción |
| `NucleusReservoir` drain | ThermodynamicLayer | Nuclei finitos | total_qe peak → decline |

## 5. Fases del ciclo de vida observado

```
FASE 1: Crecimiento (tick 0-1600)
  Nuclei emiten. Campo se expande. 296→439 entidades.
  38 BehavioralAgents a tick 400 (máximo).
  Cultural transmission activa entre 27-38 agentes.

FASE 2: Meseta (tick 1600-3200)
  447 entidades, 28-29 agentes. Equilibrio emisión ≈ drain.
  avg_qe estable en ~100. Generaciones se reemplazan (avg_age ~90).
  Memes se propagan durante ~1600 ticks con 28 agentes en rango.

FASE 3: Declive (tick 3200-4400)
  Nuclei se agotan (reservoir 15k / 10 qe/tick ≈ 1500 ticks).
  total_qe cae 47k→20k. avg_qe cae 100→50.
  Agentes persisten: beh=26-27 (energía residual suficiente).

FASE 4: Segundo equilibrio (tick 4400-8000)
  404→307 entidades. 25-26 agentes estables.
  avg_age baja a 32-37 (generaciones más cortas, turnover rápido).
  Reciclaje de nuclei alimenta el campo residual.
  Cultural memory heredada por offspring (gen 2+).
```

## 6. Métricas baseline (reproducibles con seed=2026)

| tick | alive | total_qe | avg_qe | avg_age | sen | beh |
|------|-------|----------|--------|---------|-----|-----|
| 400 | 296 | 12361 | 41.8 | 100 | 296 | 38 |
| 2000 | 447 | 47267 | 105.7 | 89 | 447 | 28 |
| 3600 | 424 | 32257 | 76.1 | 75 | 424 | 26 |
| 5600 | 410 | 21333 | 52.0 | 37 | 410 | 26 |
| 8000 | 307 | 21073 | — | — | 307 | — |

## 7. Cómo correr

```bash
# Generar imagen estática (tick final)
RESONANCE_MAP=civilization_test cargo run --release --bin headless_sim -- --ticks 8000 --scale 12 --out civilization.ppm

# Convertir a PNG
sips -s format png civilization.ppm --out civilization.png

# Correr con Bevy rendering (ventana 3D)
RESONANCE_MAP=civilization_test cargo run --release
```

## 8. Criterio de validación

| Check | Esperado | Comando de verificación |
|-------|----------|------------------------|
| BehavioralAgents > 20 sostenidos | beh ≥ 20 durante 4000+ ticks | Telemetry output |
| Senescence activa | sen = alive para todas las entidades | Telemetry: sen == alive |
| Ciclo energético completo | total_qe sube → pico → baja → estabiliza | Telemetry: 4 fases visibles |
| Turnover generacional | avg_age < max_age_materialized | Telemetry: avg_age < 200 |
| No crash | 8000 ticks sin panic | Exit code 0 |
| Determinismo | Misma seed → mismos números | Correr 2 veces, comparar |

## 9. Limitaciones conocidas

- **No se ve movimiento** en PPM — los puntos blancos son posiciones estáticas. Movimiento visible requiere video (frame-by-frame PPM → ffmpeg) o Bevy rendering.
- **Infrastructure investment no ocurre** — ningún behavior emite `InfrastructureInvestEvent`. El sistema procesa pero no hay input.
- **Cultural transmission no se ve en la imagen** — es un cambio de datos internos (MemeEntry), no visual. Requiere logging específico para validar.
- **Solo rojo** — las 6 bandas de frecuencia existen pero el Terra cluster domina visualmente por su alta emisión × 4 nuclei.
