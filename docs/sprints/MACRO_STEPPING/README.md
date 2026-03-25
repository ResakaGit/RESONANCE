# Sprint Macro-Stepping — LOD Temporal y Análisis Predictivo

Índice maestro: [`../README.md`](../README.md).

Set de sprints alineado a `docs/design/MACRO_STEPPING.md`.

## Formato

- Un sprint = un entregable cohesivo y testeable.
- Patrón decorador/intercepción: cada sprint agrega la vía analítica O(1) pero respeta que la base sigua funcionando en su ciclo *tick-by-tick* cuando no tiene la marca Macro.

## Regla de oro

```text
Con el Macro-Step desactivado, la entidad procesa cada FixedUpdate as given.
La diferencia entre ON (Analítico) y OFF (Euleriano micro) debe ser numéricamente convergente
en los límites de Normalización.
```

## Grafo de dependencias

```text
     M1 (Matemáticas Exponenciales)
      │
  ┌───┴───┐
  ▼       ▼
 M2       M3
(Ruteo   (Barreras de
 ECS)    Normalización)
  │       │
  └─┬───┬─┘
    ▼   ▼
   M4 (Observer Visual / LOD Module)
    │
  ┌─┴─┐
  ▼   ▼
 M5   M6 (Bridge 
(VRAM Optimizer Cache)
 CPU Benchmark)
```

## Índice

| Sprint | Archivo | Módulo | Onda | Estado |
|--------|---------|--------|------|--------|
| M1 | `SPRINT_M1_ANALYTICS.md` | `blueprint/equations_macro` (Funciones Analíticas O(1)) | 0 | ⏸ Pendiente |
| M2 | `SPRINT_M2_ECS_ROUTING.md` | `simulation/macro_stepping` (`MacroStepTarget` component, RunCriteria disjoint) | A | ⏸ Pendiente |
| M3 | `SPRINT_M3_NORMALIZATION.md` | `simulation/horizon_solvers` (Funciones inversas para $\Delta t$ límite) | A | ⏸ Pendiente |
| M4 | `SPRINT_M4_LOD_OBSERVER.md` | `simulation/sensory` (Inyector/Removedor de flag según AttentionGrid) | B | ⏸ Pendiente |
| M5 | `SPRINT_M5_BENCHMARK.md` | Pruebas de aceleración `x50` y divergence tests Euler vs Exponencial | C | ⏸ Pendiente |
| M6 | `SPRINT_M6_BRIDGE_MACRO_CACHE.md` | `bridge/equations_macro_ops` (`Bridgeable` Trait + Cuantización para LOD O(1)) | D | ⏸ Pendiente |

## Referencias y Compatibilidad con Base Code

La base `resonance` garantiza 100% de aislamiento para integrar MACRO_STEPPING porque:
1. **Stateless Core**: Las fórmulas de `equations.rs` no retienen caché. Extenderlas con `equations_macro.rs` (analítico) es arquitectónicamente invisible para el simulador primario.
2. **Scheduling Seguro (`pipeline.rs`)**: Ya estructurado en `Phase` (Termo -> Atomo -> etc.). Se creará un `PhaseMacro` paralelo, excluyente mutuo (mediante un filtro `Without<MacroStepTarget>` en los sistemas base). Esto no impacta el Order of Execution existente.
3. **Desacople Rendering**: El visualizador 3D solo observa Componentes de Geometría/Radio. No importa si los datos cambian bruscamente por un asalto analítico de "5 horas después"; el GPU bridge (`shape_inference.rs`) lo consumirá pasivamente en su propio frame.

---

## Roadmap & Checklists (Sub-Sprints details)

### Tarea 1 / M1: Base Matemática Cerrada (`src/blueprint/equations_macro.rs`)
La termodinámica base de `equations.rs` necesita funciones pares de resolución analítica temporal extendida.
- [ ] **Decaimiento Exponencial Constante**: Crear fórmula $\Delta E$ para tiempo arbitrario.
- [ ] **Alometría Asintótica Temporal**: Desarrollar la integral del radio respecto del tiempo para evitar sobrecrecimiento lineal.

### Tarea 2 / M2: Ruteo Sistémico en ECS (`src/simulation/macro_stepping.rs`)
- [ ] **Componente Analítico**: Incluir marca `MacroStepTarget { tick_rate: f32 }` que defina qué entidades califiquen para simulación macro.
- [ ] **Modificación en Pipeline (`pipeline.rs`)**: Mover todas lógicas estándares a un Set disjunto. Para entidades `MacroStepTarget`, correr *MacroThermodynamicLayer*.

### Tarea 3 / M3 y M4: Diferencial Metabólico y Observer
El acople a la Capa 4 (Metabolismo) y Módulo de Atención.
- [ ] **Solvers Inversos (Horizonte)**: Funciones que al recibir un límite devuelvan el máximo $\Delta t$ seguro tolerado antes de tocarlo.
- [ ] **Diferencial Metabólico**: El CPU usará la capa `GrowthBudget` (Celeridad de mutación biológica de esa entidad) y la energía de enlace (`MatterCoherence`) para tabular el `TickRate` específico limitándolo de forma asintótica y puramente determinista. 
- [ ] **Observer LOD**: Inyectar/borrar el `MacroStepTarget` dependiendo si $A < umbral$ del Grid Perceptivo.

### Tarea 4 / M5 y M6: Bridge Optimization y Benchmark Numérico
- [ ] **Bridge Decorator (M6)**: Envolver `equations_macro.rs` bajo el trait `Bridgeable`. Interceptar los inputs ($\Delta t$, $E_0$) y binificarlos/redondearlos para consultarlos contra el `BridgeCache`. Evitar coma flotante recurrente.
- [ ] **Divergencia O(1) (M5)**: Analizar la exactitud de Euler (1,000,000 FixTicks) contra el resultado transpirado por el `BridgeCache` del Macro-Step Analítico. Debe acercase asintóticamente sin error crítico.
