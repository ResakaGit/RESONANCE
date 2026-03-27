# Sprint R5 — Sensitivity and Uncertainty

## Objetivo

Cuantificar sensibilidad de parámetros e incertidumbre para evitar overfitting de tuning.

## Rol dentro de SRP

Expone qué parámetros realmente gobiernan el sistema y cuáles son frágiles ante ruido de entrada.

## Entregables

- Barridos sistemáticos de parámetros.
- Ranking de parámetros críticos.
- Bandas de confianza para métricas clave.
- Límites recomendados por parámetro.

## Tareas

- Ejecutar barridos (ablation/Monte Carlo).
- Identificar top parámetros críticos por impacto.
- Definir bandas de confianza.
- Agregar alertas de inestabilidad.

## DoD

- Reporte de sensibilidad con top 10 parámetros críticos.
- Límites recomendados por parámetro documentados.
- Re-ejecución del barrido con seed fija reproduce ranking principal.

## Referencias

- `src/blueprint/equations.rs`
- `src/simulation/evolution_surrogate.rs`

