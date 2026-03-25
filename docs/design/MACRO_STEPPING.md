# ⛩️ Elite Blueprint: Macro-Stepping & Temporal LOD
**Foco:** Integración Analítica y Escalamiento Entrópico Determinista.
**Fase:** Extensión del Módulo de Entropía (Capa 0 a 7).
**Aprobación Yanagi:** ✅ Validado. Cambiar precisión por predicción estocástica ahorra ciclos L2.

---

## 1. El Problema (Espiral de la Muerte)

En un motor ECS que corre una Simulación Termodinámica en un pipeline de `FixedUpdate` (Mecánica de Newton), el estado avanza integrando ecuaciones de tiempo finito (Euler):
`Energia_Futura = Energia_Actual + (Transferencia_Calorica * dt)`

Acelerar el tiempo de juego (`TimeScale x100`) obligando al procesador a iterar el bucle 100 veces por milisegundo produce **CPU Starvation**. Si por el contrario intentamos simplemente multiplicar `dt` por 100 sin iterar, el cálculo explota matemáticamente: la entidada cede más calor del que posee, derivando en valores negativos de `BaseEnergy` e invalidando la termodinámica.

---

## 2. La Solución "Estadística" (LOD Temporal)

Cuando un evento escapa al foco del observador (Atención Visual $A \to 0$ o lejanía explícita), el *ruido estocástico* de sus colisiones micro se estabiliza. Sus decaimientos dejan de ser erráticos y siguen curvas universales.

En lugar de simular ticks repetitivos, se implementa un modelo **Analítico de Tiempo Cerrado** (Macro-Stepping).

### A. Ecuación Analítica (Ejemplo Disipación Entrópica)
Si la energía cae linealmente a tasa constante en micro, a nivel Macro adopta su límite real (Decaimiento Exponencial):

$$ E(t + \Delta t) = E_{0} \cdot e^{-D_{tasa} \cdot \Delta t} $$

Este cómputo requiere **exactamente el mismo O(1)** que procesar un `FixedUpdate`, pero nos permite usar un $\Delta t$ de *meses* sin escupir un valor negativo o basura irreal.

### B. El Diferencial Metabólico (Acople a Capa 4)

En lugar de imponer un salto de tiempo global bruto, el **Multiplicador de Entropía Interno** se vincula dinámicamente a la capa de metabolismo (`GrowthBudget` / `MatterCoherence`) que ya existe.

- **La "Celeridad de Cambios"**: Una entidad con metabolismo elevado (Ej: *BotanicalSeed* consumiendo mucha Biomasa) presenta derivadas muy pronunciadas. Su límite temporal máximo ($\Delta t_{limite}$) analítico será mucho más corto que el de una masa inerte (`MatterState::Solid`, altísima *bond_energy*, metabolismo $0$).
- **Normalización**: Esto evita estallidos. La capa analítica ajusta el acelerador temporal en proporción inversa a la velocidad de mutación orgánica de la entidad.

### C. El Mecanismo de Intercepción ("Componente Macro")
Las entidades que califiquen para LOD Temporal reciben el `MacroStepping` Flag Component y son ruteadas en el Main Schedule para evitar las tuberías estándares. Su energía, densidad, y capa morfológica se rigen exclusivamente evaluando fórmulas de `equations_macro.rs` orientadas al paso largo del tiempo.

---

## 3. Barreras de Normalización (Safeguards)

Si el multiplicador es gigante (ej: *"Sumar 3 semanas de simulación"*), la entidad no puede simplemente procesar indiscriminadamente. Las matemáticas exponenciales fallan cuando hay **Fases Transitorias** de por medio (Puntos Discretos).

El Macro-Step calculará un límite (Horizonte de Normalización):
1. **Punto Acumulativo de Presupuesto Liebig:** La planta crecería 500 metros en 3 semanas, pero el `NutrientProfile` alcanzaría el límite a los 2 días.
2. **Punto de Estado Material:** A los 40 grados el átomo cambia de *Sólido* a *Gas*.

**Metodología Determinista:**
El ciclo Macro calcula el **Tiempo Restante** ($T_{limite}$) hasta chocar contra ese rango paramétrico usando álgebra inversa:
- Si $\Delta t > T_{limite}$, entonces el Engine aplica un $\Delta t = T_{limite}$, gatilla el cambio de Fase Categórica, vacía nutrientes correspondientes, y evalúa el tiempo restante en la próxima iteración macro.

## 4. Filosofía y Alineación Core (Single Source of Truth: Energía)

El `Macro-Stepping` no es un parche de optimización, es una extensión empírica de la **Filosofía Yanagi** del repositorio sobre la conservación y estado molecular. 

- **La Energía es la SSOT (Single Source of Truth)**: Toda la botánica, el entorno topológico y el color atómico son deducciones inferidas de la `BaseEnergy`. El *Macro-Stepping* **NO** crea variables de "crecimiento saltado" ni "cachés temporales". La matemática analítica del macro-salto manipula estricta y únicamente los fotones absorbidos y los ergios de calor presentes.
- **Pureza Stateless (Componentes Vacíos de Historia)**: La tubería macro (`equations_macro.rs`) recibe la Energía Actual, introduce un multiplicador dinámico algorítmico ($\Delta t$), y devuelve la Energía Final de esa porción temporal. No requiere recordar "cómo" llegó a T0.
- **Determinismo Matemático**: Un salto usa álgebra de cuerpo algebraico cerrado. El motor asegura repetibilidad cross-platform idéntica tanto si un jugador miró a la planta crecer tick a tick, o si la dejó abandonada por 7 ciclos solares, sin estresar el *Random Number Generator*.

---

## 5. Estrategias de Implementación de Alto Nivel

### Diagrama de Tubería Híbrida (Diferencial Metabólico + ECS Routing)
```mermaid
graph TD
    classDef euler fill:#2b3a42,stroke:#ff6b6b,stroke-width:2px;
    classDef macro fill:#102540,stroke:#64ffda,stroke-width:2px;
    
    A[Tick Bevy: Phase::MetabolicLayer] --> B{¿Componente MacroStepTarget?}
    
    B -->|NO (Atención > Umbral)| C[Tubería Micro Euleriana]:::euler
    C --> C1(equations::growth_size_feedback)
    C --> C2(equations::liebig_growth_budget)
    
    B -->|SÍ (Atención = 0)| D[Tubería Macro Asintótica]:::macro
    D --> D1[Lector: Diferencial Metabólico / GrowthBudget]
    D1 --> D2(Solver: Calcular Horizonte Máximo Δt_limite)
    D2 --> D3(equations_macro::asymptotic_growth_eval)
    
    C1 --> E[Reescritura Analítica y Plana de SSOT]
    C2 --> E
    D3 --> E
    
    E -->|BaseEnergy, SpatialVolume, NutrientProfile| F[Phase::MorphologicalLayer Reconstruye Vertices]
```

### Paradigma Testable
La adopción de esta capa es robustamente verificable aplicando **Testing Doble Ciego (Unpaired Asymptotic Testing)**:
1. **Unidad Euleriana Base**: Spawn de un `BotanicalSeed` que se itera por fuerza bruta a `dt = 0.01` unas `100,000` iteraciones (Simulando Euler tick-by-tick).
2. **Unidad Analítica de Salto**: Correr el solver de la capa Macro sobre una semilla gemela con `dt = 1000.0` logrando un cálculo inmediato $O(1)$.
3. **Criterio de Validación CI/CD**: `assert_approx_eq!(BaseEnergy_Micro, BaseEnergy_Macro, epsilon = 1e-4)`. Esta condición certifica matemáticamente que las nuevas abstracciones de *Fast-Forward* jamás desobedecerán la Lógica de Entropía del repo.

### 5.1 Integración con BridgeOptimizer (Caché Cuantizado)

Cumpliendo con las directivas del Módulo de Optimización Base (Sprints B1 a B10), la computación Analítica O(1) del Macro-Step **también se somete a cuantización y caché**:
1. **Delegación a `bridge/`**: Las fórmulas en `equations_macro.rs` no se exponen puras. Se envuelven mediante el *Decorator Trait* (`Bridgeable`) en `bridge/equations_macro_ops.rs`.
2. **Cuantización de Entradas**: En lugar de calcular una caída exponencial (floats crudos) para $\Delta t = 86453.21$ y $E_0 = 120.45$, el Módulo Bridge Cuantiza estos *inputs* a bins redondeados.
3. **Data-Driven Catching (`BridgeCache`)**: El ECS consulta esta tabla O(1). Si otra semilla con el mismo bin metabólico ya hizo un leap de tiempo este frame, el proceso retorna el `Result` purgado de la caché. Evitamos multiplicar $O(N)$ ciclos de coma flotante pesada usando pura memoria estática.

---

## 6. Trade-Offs Evaluados

| Ventaja | Desventaja (Costo Yanagi) |
| :--- | :--- |
| Escalamiento temporal logarítmico (puede calcular 10 años en el mismo tiempo que 1 segundo). | Se pierde la turbulencia microscópica. Si el radio de una macrófita hubiese oscilado entre dos piedras, el análisis cerrado la fundirá contra ambas a la vez obteniendo un promedio aritmético aburrido. |
| El `FixedUpdate` de Bevy no asfixia al main-thread ni induce drop de FPS. | Aumenta el costo de Mantenimiento de Hardware: Todo cambio matemático al sistema base (Capa de Termodinámica) tiene que escribirse por *duplicado* (Un solver para Micro, un solver inverso cerrado para Macro). |
