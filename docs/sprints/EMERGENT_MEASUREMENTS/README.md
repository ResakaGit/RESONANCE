# Track: EMERGENT_MEASUREMENTS — Del framework al resultado medido

Resonance tiene 7 escalas vivas, 14+ layers y 3.166 tests verdes. Lo que no tiene (todavía) es una **figura publicable** que muestre un axioma haciendo trabajo real, medido en ensemble, con error bars.

Este track produce esas figuras. No es trabajo de simulación — es trabajo de **experimento sobre la simulación que ya existe**.

## Principio

> Una simulación sin medición en ensemble es una demo. Tres figuras reproducibles con seeds fijos son un paper.

Cada sprint de este track entrega:
- Binarios `measure_<name>` headless y deterministas.
- CSVs crudos + PNGs derivados en `docs/figures/<sprint>/`.
- Un criterio de aceptación numérico (no "se ve bien").
- Si el experimento **no** cumple el criterio → **null result documentado**, no tuning.

## Sprints

| ID | Nombre | Estado | Entrega |
|----|--------|--------|---------|
| EM-1 | Tres figuras emergentes (Kleiber · Lotka-Volterra · Linaje) | Designed | 3 binarios, 3 CSV, 3 PNG |

## Relación con otros tracks

- **AP-6c** desbloquea EM-1.3 (linaje).
- **PV-7** (Hordijk RAF) podrá reutilizar el harness de medición de EM-1.
- Figuras de este track van directo a `docs/sintesis_patron_vida_universo.md` cuando cierren.
