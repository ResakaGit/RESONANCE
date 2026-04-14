# ADR-024: REMD Swap Criterion — Metropolis on Temperature, Not Coordinates

**Estado:** Propuesto
**Fecha:** 2026-04-13
**Contexto:** MOLECULAR_DYNAMICS track, sprints [MD-16](../../sprints/MOLECULAR_DYNAMICS/SPRINT_MD16_REMD.md), [MD-17](../../sprints/MOLECULAR_DYNAMICS/SPRINT_MD17_FOLD_VALIDATE.md)

## Contexto

REMD necesita intercambiar informacion entre replicas a distintas temperaturas.
Dos opciones: intercambiar coordenadas o intercambiar temperaturas.

## Decision

### D1: Intercambiar temperaturas, no coordenadas

Cada replica mantiene sus coordenadas/velocidades y solo intercambia la
temperatura asignada. Velocidades se re-escalan: `v_new = v * sqrt(T_new/T_old)`.

**Razon:** Menos datos transferidos, mas simple, y evita problemas de
sincronizacion de velocidades. Standard en la literatura (Sugita & Okamoto 1999).

### D2: Metropolis criterion con detailed balance

```
Delta = (beta_i - beta_j) * (E_i - E_j)
P_swap = min(1, exp(Delta))
```

**Razon:** Garantiza muestreo canonico correcto en cada replica. Validable
via detailed balance tests.

### D3: Geometric temperature ladder

```
T_i = T_min * (T_max / T_min)^(i / (N-1))
```

**Razon:** Acceptance ratio uniforme a lo largo del ladder. Optimo para
proteinas con distribucion de energia log-normal.

### D4: Deterministic swap via hash-based RNG

Swap acceptance usa RNG determinista (splitmix64 seeded from tick + replica
pair indices). Reproducibilidad bit-exact.

## Consecuencias

- 8-16 replicas (configurable)
- Target acceptance ratio: 20-50%
- Swap frequency: cada 100-500 steps (configurable)
- Output: histograma de acceptance ratio por par de replicas
