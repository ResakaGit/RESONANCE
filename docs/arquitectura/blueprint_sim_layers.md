# Módulo: Simulación en 5 Capas

Blueprint de arquitectura para la escalera termodinámica -> morfológica.
Fuentes:
- `docs/sprints/README.md` (sprint puntual eliminado; ver `src/simulation/pipeline.rs` y fases `Phase::*`)
- `design/SIM_LAYERS.md`

## 1) Frontera y Responsabilidad
- **Qué Resuelve**: Pipeline de ejecución de simulaciones físicas de entidades. Restringe la lógica para que respete causalidad: Calor -> Elementos -> Química -> Vida -> Forma.
- **Qué NO Resuelve**: Esta capa es un orquestador lógico. La física dura se procesa en módulos como `EnergyFieldGrid`.

## 2) Trade-offs y Complejidad (Yanagi)
- **Incidental Complexity vs Definición**: La definición exige tuberías *Stateless* (Funciones Puras). Modelar físicas atómicas y enlaces químicos 1:1 reteniendo estado rompe la directiva y mata el cache L1/L2.
- **Solución Estricta (Yanagi)**: La Capa Atómica y la Capa Química operarán exclusivamente como puras evaluaciones *lock-free* ($O(1)$) sobre el `AlchemicalAlmanac`. Función sin estado: Entran inputs $\to$ salen moléculas y entropía. 
- **Pipeline de Bevy (Tuberías)**:
  - `Set::ThermodynamicLayer` (Luz/Calor $\to$ Potencial Energético y Ondas)
  - `Set::AtomicLayer` (Disponibilidad $\to$ Moléculas Base)
  - `Set::ChemicalLayer` (Ósmosis/Catálisis $\to$ Moléculas Complejas)
  - `Set::MetabolicLayer` (Aplica factor biológico de Liebig $\to$ Presupuesto de Biomasa)
  - `Set::MorphologicalLayer` (Traduce Presupuesto a Parámetros de L-Systems)
