# Módulo: Sensorial & Atención (Systemic LOD)

Blueprint de arquitectura para la implementación del LOD predictivo basado en atención ($A$).
Fuentes:
- `docs/sprints/README.md` (sprint puntual eliminado; LOD worldgen en `src/worldgen/lod.rs`, bridge métricas)
- `design/SENSORY_ATTENTION.md`

## 1) Frontera y Responsabilidad
- **Qué Resuelve**: Determina qué porciones del grid de simulación (Sectores) necesitan ejecutarse a máxima frecuencia (60Hz) y cuáles pueden degradarse a 10Hz, 1Hz, o suspenderse (0Hz) en base a funciones puras de transducción sensorial.
- **Qué NO Resuelve**: Este módulo no renderiza nada ni simula colisiones. Solo dicta el `TickRate` de las entidades según su métrica $A$.

## 2) Trade-offs y Complejidad (Yanagi)
- **Implementación Segura**: Agrupar entidades en `RunCriteria` de Bevy basados en un contador de "ticks since last update". Si $A = 0.1$, el sector hace skip de 9 updates completos y en el décimo ejecuta con $\Delta t \times 10$.
- **Riesgo**: Si un misil entra a un sector dormido, el despertar debe ser inmediato. El sistema sensorial evalúa energía (el misil tiene un perfil cinético alto), por ende el receptor que "siente" la vibración sube la atención del sector antes de la colisión.
- **Solución Numérica**: Mantener un "Grid de Atención" de baja resolución (ej. `macro_cells`) que propaga la $A$ como onda predictiva amortiguada.

## 3) Integración Lógica con Optimizaciones Existentes
El framework actual (`WorldgenPerfSettings`) opera por distancia geométrica pura. El LOD Sensorial la reemplaza/modula:
- **Sobrescritura del Fallback Euclidiano**: La distancia solo actúa como fallback. Si $A$ (Atención perceptiva) está computado, $A$ es absoluto. Coordenadas cercanas pero ocultas (oclución) tendrán $A \approx 0$, habilitando su degradación máxima pese a estar cerca de la cámara.
- **Normalización de Frecuencia/Energía (`factor_precision`)**: El rango de normalización (la pérdida de mantisa o *step resolution* en el shader o simulación) obedece a $A$. Rangos finos en $A \approx 1.0$, normalizaciones agresivas (agrupando por rangos, ej. round(E / 10)*10) cuando $A < 0.5$.
