# Earth Telescope Demo — De la Sopa Primordial a la Modernidad

**Documento de implementación para la demo que valida el Multi-Telescopio (ADR-015/016) sobre un modelo de la Tierra con visualización 2D y 3D.**

---

## 1. Qué Es Esta Demo

Una simulación de la Tierra que corre desde abiogénesis hasta ecosistema complejo usando el Multi-Telescopio para saltar eras geológicas estables y simular en detalle las transiciones. El usuario ve el planeta evolucionar en tiempo real con dos modos de visualización:

- **2D (sim_viewer):** Vista plana equirectangular — mapa de calor energético, entidades como puntos, día/noche como banda luminosa que barre.
- **3D (planet_viewer):** Esfera con textura dinámica — rotación planetaria, inclinación axial, estaciones, cámara orbital.

Ambos modos muestran un **dashboard de telescopio** con métricas de régimen (Hurst, Fisher, ρ₁, visibilidad de Englert) y el cono de incertidumbre por nivel.

---

## 2. El Mapa: `earth_real.ron`

Mapa existente, no se modifica. Configuración:

```
Grid:               128×64 celdas (2.8° × 2.8° ≈ 310 km/celda)
Coordenadas:        X = longitud (-180° → +180°), Y = latitud (-90° → +90°)
Cell size:          2.0 unidades mundo
Día:                600 ticks (1 día = 10 segundos a 60 Hz)
Año:                219,000 ticks (365 días)
Inclinación axial:  0.26 rad (23.5° — Tierra real)
Sol:                Direccional, 800 qe/s, solo hemisferio iluminado
Geotérmica:         5 qe/s, 50 Hz, InverseSquare
Atmósfera:          30 qe/s, 600 Hz, InverseLinear
Continentes:        18 núcleos Terra (80-92 Hz), posiciones geográficas reales
Océanos:            7 núcleos Aqua (195-210 Hz)
Warmup:             200 ticks (pre-simulación antes de interacción)
```

El mapa ya tiene día/noche direccional (`solar_emission_qe_s: 800`), estaciones (`axial_tilt: 0.26`), y geografía real.

---

## 3. Lo Que Queremos Demostrar

### 3.1. El Telescopio Funciona

| Escala temporal | Régimen esperado | Comportamiento del telescopio |
|---|---|---|
| Ticks 0-200 (warmup) | Campos propagándose, sin vida | STASIS — K crece rápido |
| Ticks 200-2000 | Abiogénesis: coherence > dissipation → spawn | TRANSITION — K baja, detalle completo |
| Ticks 2000-50,000 | Ecosistema joven, competencia por recursos | POST-TRANSITION → STASIS gradual |
| Ticks 50,000-200,000 | Ecosistema maduro, ciclo día/noche estable | STASIS — K al máximo, niveles crecen |
| Tick 219,000 (1 año) | Ciclo estacional completo | STASIS con oscilación periódica |
| Cada 5000 ticks | Impacto de asteroide | TRANSITION abrupta — K colapsa, niveles se reducen |

### 3.2. La Precisión Converge

Después de 100+ reconciliaciones, la precisión del telescopio (% de PERFECTs) debería superar 80% en fases estables. Las calibration weights deberían estabilizarse.

### 3.3. Conservation Se Mantiene

En ningún momento `total_qe` del mundo (post-reconciliación) excede el valor anterior + input solar. Axioma 5 verificable en dashboard.

### 3.4. Los Niveles Se Adaptan

El stack debería crecer de 1 nivel a 3-4 niveles durante estasis prolongada (ticks 50k-200k) y contraerse a 1-2 niveles tras impactos de asteroide.

---

## 4. Arquitectura del Binario

### 4.1. Un Solo Binario, Dos Modos

```
cargo run --release --bin earth_telescope -- [--mode 2d|3d] [--ticks N] [--speed MULT]
```

| Flag | Default | Descripción |
|---|---|---|
| `--mode` | `3d` | `2d` = sim_viewer equirectangular, `3d` = planet_viewer esfera |
| `--ticks` | `0` (infinito) | Ticks a simular (0 = hasta que el usuario cierre) |
| `--speed` | `1` | Multiplicador de velocidad (1=realtime, 10=10×, 100=geologico) |
| `--levels` | `1` | Niveles iniciales del stack (1=ADR-015, 2-8=multi) |

**Env vars heredados:**
```bash
RESONANCE_MAP=earth_real                     # Mapa (default: earth_real)
RESONANCE_RENDER_COMPAT_PROFILE=full3d       # Para modo 3D (auto-set por --mode)
```

### 4.2. Componentes del Binario

```
earth_telescope.rs
├── setup_app()           → Bevy App con plugins según modo
├── setup_telescope()     → TelescopeStack + ReconciliationHistory como Resources
├── telescope_system()    → Sistema en FixedUpdate que corre tick_telescope_stack_sync
├── metrics_system()      → Computa RegimeMetrics desde EnergyFieldGrid + SimTimeSeries
├── dashboard_system()    → Panel egui con métricas del telescopio
└── main()                → CLI args → setup → app.run()
```

### 4.3. Integración con Bevy (Modo 3D)

```
Plugins:
  DefaultPlugins (ventana, renderer)
  LayersPlugin (14 capas)
  SimulationPlugin (33 sistemas en FixedUpdate)
  SimulationTickPlugin (reloj determinista)
  Compat2d3dPlugin(Full3dVisual) (posiciones XZ, cámara 3D)
  WorldgenPlugin (carga earth_real.ron, spawn nuclei)
  QuantizedColorPlugin (colores por frecuencia)
  DashboardBridgePlugin (SimTickSummary, SimTimeSeries)

Resources nuevos:
  TelescopeStack (multi-nivel)
  ReconciliationHistory (ring buffer 256)
  TelescopeConfig (K bounds, grow/shrink factors)
  CalibrationConfig (learning rate, weight bounds)
  EarthTelescopeDemoConfig (speed, mode, levels)
```

### 4.4. Integración con Bevy (Modo 2D)

Mismo binario pero con `MinimalPlugins` + `SimulationPlugin` (sin DefaultPlugins). Renderizado via `frame_buffer::render_frame()` → terminal o pixel_viewer. Dashboard via terminal ASCII o egui si `--features pixel_viewer`.

---

## 5. Visualización 2D: Mapa Equirectangular

### 5.1. Qué se ve

```
┌──────────────────────────────────────────────────────┐
│                  EARTH TELESCOPE DEMO                 │
│  Tick: 54,201  |  Pop: 847  |  QE: 12,450           │
│  Regime: STASIS  |  K: 64  |  Levels: 3/8           │
│  Accuracy: 94%  |  Corrections: 6%  |  H: 0.78      │
├──────────────────────────────────────────────────────┤
│                                                       │
│  ████████░░░░░░░████████████████░░░░░░░░████████████ │
│  ████████░░░░░░░████████████████░░░░░░░░████████████ │
│  ███·····░░░░░░░████··██████████░░░░░░░░█████·····██ │
│  ███·····░░░░░░░████··██████████░░░░░░░░█████·····██ │
│  ████████░░░░░░░████████████████░░░░░░░░████████████ │
│  ████████░░░░░░░████████████████░░░░░░░░████████████ │
│                                                       │
│  █=energía alta  ·=entidades  ░=noche  ▓=telescopio  │
├──────────────────────────────────────────────────────┤
│  [QE ▇▇▇▇▇▇▇▇▅▅▅▅▃▃▃▃▁▁]  [POP ▇▇▇▅▅▇▇▇▅▅▃▃▁▁▇▇] │
│  [H  ▇▇▇▇▇▇▇▇▇▇▇▇▇▇▇▇▇▇]  [ρ₁  ▃▃▃▃▃▃▃▅▅▅▅▃▃▃▃▃] │
│  [V  ░░░░▃▃▅▅▇▇█████████]  [K   ▁▁▃▃▅▅▇▇████████]  │
└──────────────────────────────────────────────────────┘
```

### 5.2. Elementos visuales

| Elemento | Representación 2D | Fuente de datos |
|---|---|---|
| Energía del campo | Color ramp (negro→azul→cyan→verde→amarillo→blanco) | `EnergyFieldGrid.accumulated_qe` por celda |
| Frecuencia | Tinte de hue (bajo=azul, alto=rojo) | `EnergyFieldGrid.frequency_contributions` |
| Día/noche | Banda vertical de brillo que barre | `DayNightConfig.solar_meridian_x` |
| Estaciones | Gradiente latitudinal de intensidad | `seasonal_irradiance_modifier()` |
| Entidades | Puntos blancos (normales) / cyan (behavioral) | Query `Transform + BaseEnergy` |
| Impacto asteroide | Flash rojo en zona de impacto | `ASTEROID_INTERVAL` cada 5000 ticks |
| Cono telescopio | Gradiente de opacidad por nivel (V de Englert) | `TelescopeStack.levels[i].visibility` |

### 5.3. Dashboard 2D (ASCII o egui)

```
Sparklines de 512 puntos (RingBuffer existente):
  QE total       → SimTimeSeries.qe_history
  Población      → SimTimeSeries.pop_history
  Especies       → SimTimeSeries.species_history

Sparklines nuevos (computados por telescope_system):
  Hurst H        → hurst_dfa() cada 64 ticks
  Autocorrelación ρ₁ → sliding_autocorrelation_lag1()
  Visibilidad V  → speculative_visibility() del nivel más alto
  K adaptativo   → stack.levels[0].k

Texto:
  Régimen actual → regime_label()
  Niveles activos → stack.active_levels
  Alcance total  → stack.total_reach()
  Precisión      → projection_accuracy(history, 10)
  Correcciones   → correction_frequency(history)
```

---

## 6. Visualización 3D: Planeta Esfera

### 6.1. Qué se ve

Una esfera que rota con textura dinámica del campo energético, día/noche realista, y entidades como partículas en la superficie. La cámara orbita lentamente. El dashboard egui se superpone en la esquina.

### 6.2. Componentes 3D

| Componente | Implementación | Referencia |
|---|---|---|
| Esfera | `Sphere::new(5.0).mesh().uv(64, 32)` | `planet_viewer.rs:218` |
| Textura | `Image::new(128, 64, RGBA)` actualizada cada frame | `planet_viewer.rs:update_planet_texture` |
| Material | `StandardMaterial { unlit: true, base_color_texture }` | `planet_viewer.rs:material` |
| Cámara | Orbita a distancia 15.0, velocidad 0.15 rad/s | `planet_viewer.rs:CAMERA_ORBIT_SPEED` |
| Inclinación | `PLANET_TILT_RAD = -0.41` (23.5°) | `planet_viewer.rs:PLANET_TILT_RAD` |
| Iluminación | Sin luz direccional (unlit material) — la textura ya tiene día/noche | Simplifica shader |

### 6.3. Textura Dinámica

Cada frame:

```rust
fn update_texture(
    grid: &EnergyFieldGrid,
    entity_positions: &[(u32, u32, f32)],
    behavioral_positions: &[(u32, u32)],
    telescope_visibility: &[f32; MAX_LEVELS],  // NUEVO: V por nivel
) → Image {
    // 1. Renderizar campo energético como color ramp (existente: render_frame)
    // 2. Aplicar entidades como puntos blancos/cyan (existente)
    // 3. NUEVO: overlay de incertidumbre del telescopio
    //    - Regiones lejos del último colapso: ligeramente transparentes/borrosas
    //    - Regiones cerca del colapso: nítidas
    //    - Intensidad del blur ∝ V del nivel que cubre esa región temporal
}
```

### 6.4. Overlay del Telescopio (3D)

Opción visual para mostrar el cono de incertidumbre:

```
Opción A: Halo de emisión
  - Esfera ligeramente más grande que el planeta (radio × 1.02)
  - Alpha = V del nivel más alto activo
  - Color = cyan translúcido (estasis) / rojo translúcido (transición)
  - Cuando el ancla colapsa: flash breve de blanco (medición cuántica)

Opción B: Líneas de nivel temporal
  - Anillos concéntricos alrededor del punto de colapso
  - Cada anillo = un nivel del telescopio
  - Grosor ∝ K del nivel
  - Opacidad ∝ (1 - V) → más opaco cerca del ancla, más transparente lejos

Opción C: Dual-sphere
  - Esfera interna: estado del ancla (verdad, opaco)
  - Esfera externa: estado del nivel más alto (proyección, semi-transparente)
  - Gap entre ambas = distancia temporal en ticks
```

**Recomendación: Opción A** (más simple, se ve bien, implementable con 1 esfera extra + material alpha).

### 6.5. Dashboard 3D (egui overlay)

```
┌─ Telescope ──────────────────────┐
│ Régimen: STASIS                   │
│ Niveles: 3/8  Alcance: 262,144   │
│ K₀: 16  K₁: 16  K₂: 16          │
│ V₀: 0.00  V₁: 0.24  V₂: 0.71   │
│                                   │
│ Precisión: 94%  Correcciones: 6% │
│ H: 0.78  ρ₁: 0.32  F: 0.04     │
│ λ: -0.031  Coherencia: 450       │
├─ Energy ─────────────────────────┤
│ ▇▇▇▇▇▅▅▅▅▃▃▃▃▁▁▁▁▃▃▅▅▇▇▇▇▇▇  │
├─ Population ─────────────────────┤
│ ▃▃▅▅▇▇▇▇▅▅▃▃▁▁▃▃▅▅▇▇████████  │
└──────────────────────────────────┘
```

Panel egui posicionado en esquina superior derecha. Usa los mismos `SimTickSummary` + `SimTimeSeries` + `StackSummary` que el modo 2D.

---

## 7. Ciclo de Vida de la Demo

### 7.1. Arranque (ticks 0-200)

```
1. App::new() con plugins según modo
2. WorldgenPlugin carga earth_real.ron → grid 128×64, 25 nuclei
3. Warmup: 200 ticks de simulación antes de mostrar
4. TelescopeStack se crea con 1 nivel (ADR-015 mode)
5. Dashboard muestra "WARMUP" durante esta fase
```

### 7.2. Abiogénesis (ticks ~200-2000)

```
El campo energético se propaga. Nuclei emiten.
Día/noche barre (600 ticks por ciclo).
Cuando coherence_gain > dissipation en alguna celda → abiogenesis_system spawna entidad.

Telescopio:
  - Primeras reconciliaciones tienen diffs grandes (SYSTEMIC)
  - K se mantiene bajo (4-16)
  - Calibration weights se ajustan rápido (learning_rate = 0.1)
  - Dashboard muestra: "TRANSITION", K=4, Accuracy ~30%, Corrections ~70%
```

### 7.3. Ecosistema Joven (ticks ~2000-50,000)

```
Entidades compiten por recursos (fotosíntesis, predación).
Reproducción emerge (qe > threshold → offspring).
Frecuencias se diversifican (mutación en reproducción).
Primeros trophic levels (productores → herbívoros → carnívoros).

Telescopio:
  - Diffs bajan gradualmente (LOCAL → PERFECT)
  - K crece adaptativamente (16 → 32 → 64)
  - Primer nivel adicional se agrega (~tick 10,000 si estable)
  - Dashboard muestra: "POST-TRANSITION" → "STASIS"
```

### 7.4. Ecosistema Maduro (ticks ~50,000-200,000)

```
Población estable. Ciclos día/noche y estacionales regulares.
Biodiversidad plateau. Nutrient cycling funcionando.
Impactos de asteroide cada 5000 ticks crean perturbaciones temporales.

Telescopio:
  - K al máximo (64-1024)
  - 2-4 niveles activos
  - Accuracy >90%
  - Impacto de asteroide: K colapsa, niveles se reducen, TRANSITION breve
  - Post-impacto: recovery (POST-TRANSITION), luego STASIS de nuevo
  - Dashboard muestra ciclo de Accuracy: 95% → 30% (impacto) → 95% (recovery)
```

### 7.5. Escala Geológica (ticks >200,000)

```
Con --speed 100, el usuario ve eras pasar.
El telescopio salta grandes tramos estables.
Los impactos y las transiciones estacionales marcan los eventos.

Con 4 niveles × K=64: alcance = 64⁴ ≈ 16M ticks por ciclo de reconciliación.
El usuario puede "avanzar" millones de ticks y ver el resultado.
```

---

## 8. Qué Archivos Se Crean

| Archivo | Tipo | Descripción |
|---|---|---|
| `src/bin/earth_telescope.rs` | **Nuevo** | Binario de la demo (CLI + Bevy app + telescope systems) |

## 9. Qué Archivos Se Reutilizan (sin modificar)

| Archivo | Uso |
|---|---|
| `assets/maps/earth_real.ron` | Mapa de la Tierra con día/noche + estaciones |
| `src/plugins/simulation_plugin.rs` | 33 sistemas en FixedUpdate (el ancla) |
| `src/plugins/layers_plugin.rs` | 14 capas registradas |
| `src/worldgen/systems/startup.rs` | Carga mapa, crea grids, spawn nuclei |
| `src/worldgen/systems/day_night.rs` | Día/noche direccional + estaciones |
| `src/runtime_platform/compat_2d3d/` | Perfil 2D/3D, SimWorldTransformParams |
| `src/runtime_platform/dashboard_bridge.rs` | SimTickSummary, SimTimeSeries, RingBuffer |
| `src/runtime_platform/dashboard_panels.rs` | Panels egui (top bar, charts) |
| `src/rendering/quantized_color/` | Colores por frecuencia |
| `src/viewer/frame_buffer.rs` | render_frame() para 2D |
| `src/batch/telescope/` | Todo el módulo (10 archivos, 205 tests) |
| `src/blueprint/equations/temporal_telescope.rs` | Math pura (61 tests) |
| `src/blueprint/constants/temporal_telescope.rs` | Constantes derivadas |

---

## 10. Cómo Validar

### 10.1. Tests Automáticos (cargo test)

Los 3324 tests existentes cubren la math pura, diffs, cascadas, calibración, y stack. La demo no necesita tests nuevos — valida visualmente lo que los tests prueban numéricamente.

### 10.2. Validación Visual

| Qué observar | Comportamiento correcto | Comportamiento incorrecto |
|---|---|---|
| Energía total | Decrece gradualmente (disipación) salvo input solar | Crece sin fuente → bug Axioma 5 |
| Día/noche | Banda de luz barre de E a W | Estático o invertido → bug day_night |
| Estaciones | Polos oscurecen en invierno | Sin variación latitudinal → tilt=0 |
| Abiogénesis | Entidades aparecen donde coherence > dissipation | Aparecen aleatoriamente → bug abiogenesis |
| Impacto asteroide | Flash + caída de población + recovery | Sin efecto → ASTEROID_INTERVAL=0 |
| K adaptativo | Crece en estasis, baja en transición | Siempre alto → no detecta transiciones |
| Niveles | Crecen en estasis prolongada | Nunca crecen → should_add_level bug |
| Precisión | Mejora con el tiempo (calibración) | Empeora → calibration_bridge bug |
| Visibilidad | V=0 en ancla, V→1 en niveles altos | V constante → speculative_visibility bug |

### 10.3. Validación Cuantitativa (modo headless)

```bash
# Correr 100,000 ticks sin GPU, exportar métricas
cargo run --release --bin earth_telescope -- --mode headless --ticks 100000 --levels 4

# Output esperado:
# tick=100000 total_qe=XXXX pop=XXX species=XX
# telescope: levels=4 reach=16777216 accuracy=0.92 corrections=0.08
# regime: STASIS H=0.76 rho1=0.28 lambda=-0.03
# axiom5: total_qe_max=YYYY (never exceeded initial + solar_input)
```

### 10.4. Comparación Telescopio vs Sin Telescopio

```bash
# Sin telescopio (baseline): 100,000 ticks tick-a-tick
time cargo run --release --bin earth_telescope -- --mode headless --ticks 100000 --levels 0

# Con telescopio (4 niveles): mismo resultado, menos tiempo
time cargo run --release --bin earth_telescope -- --mode headless --ticks 100000 --levels 4

# Verificar: mismo estado final (ancla garantiza verdad)
# Verificar: --levels 4 es más rápido que --levels 0
```

---

## 11. Hitos de la Demo

| Hito | Criterio | Verificación |
|---|---|---|
| H-1: Arranque | App inicia, mapa cargado, warmup completo | Ventana abierta, grid visible |
| H-2: Día/noche | Banda de luz barre, estaciones modulan | Visual: polos oscurecen en invierno |
| H-3: Abiogénesis | Entidades aparecen espontáneamente | Puntos blancos en zonas de alta coherencia |
| H-4: Telescopio activo | Dashboard muestra métricas del telescopio | K > 4, régimen clasificado |
| H-5: Multi-nivel | Stack crece a 2+ niveles en estasis | Dashboard: Levels > 1, Reach crece |
| H-6: Impacto + recovery | Asteroide causa TRANSITION → recovery → STASIS | K baja y vuelve a subir |
| H-7: Precisión convergente | Accuracy > 80% tras 100 reconciliaciones | Dashboard: Accuracy trending up |
| H-8: Conservation | total_qe nunca excede initial + solar | Dashboard: QE sparkline monótona (± solar) |
| H-9: Escala geológica | Con --speed 100, se ven eras pasar | Ticks avanzan rápido, ecosistema estable |
| H-10: 2D y 3D | Ambos modos muestran lo mismo con diferente render | Comparar visualmente |
