# ADR-035: Volatile Field Protocol — Señalización química via emisión de energía

**Estado:** Propuesto
**Fecha:** 2026-04-13
**Contexto:** PLANT_PHYSIOLOGY track, sprints PP-6, PP-8

## Contexto

No existe señalización química entre entidades. Un volátil (fragancia) es
energía emitida al campo a frecuencia específica — mecánicamente idéntico a un
núcleo de energía temporal, pero emitido por un órgano en vez de por el terreno.

El sistema de `nucleus_emission` + `sensory_perception` ya existe. Solo falta
que órganos puedan emitir al `NutrientFieldGrid`.

## Decision

### D1: Volátil = escritura en NutrientFieldGrid con decay GAS

**Rechazada:** solo Petal/Fruit emiten. Viola Axiom 6 — no es agnóstico.

**Decisión:** Cualquier órgano con exceso de qe y densidad bajo el umbral de
gas emite al grid. La condición es puramente física:

```
can_emit(organ) = organ.qe > maintenance_cost(organ)
                  AND organ_density(organ) < GAS_DENSITY_THRESHOLD
emission_rate = overflow_qe × dissipation_rate(matter_state) × volatile_efficiency
field[cell] += emission_rate × attenuation(distance)    ← Axiom 7
field[cell].volatile_freq = organ.frequency              ← Axiom 8
organ.qe -= emission_rate / volatile_efficiency          ← Axiom 4 (cuesta)
```

Un órgano denso (tallo) no emite porque su densidad > GAS_THRESHOLD.
Un órgano de baja densidad con exceso de energía emite naturalmente.
**No se dice "pétalo emite"** — cualquier estructura gaseosa con overflow lo hace.

**Decay:** El campo se disipa a `DISSIPATION_GAS = 0.08/tick` — correcto
biológicamente (volátiles son efímeros, se difunden y degradan rápido).

### D2: Detección por alignment (Axiom 8)

Entidad con `SENSE` capability detecta señales en el grid si:

```
alignment = exp(-Δf² / (2 × bandwidth²))
perceived = field_value × alignment
```

Solo entidades con frecuencia alineada al volátil lo "perciben". Esto emerge la
especificidad sin programar relaciones especie↔especie.

### D3: Costo energético (Axiom 4)

Emitir volátil **cuesta energía** al órgano. Entidad con poca energía deja de
emitir → no atrae → no se reproduce → presión selectiva. Emerge sin script.

### D4: Canal separado en el grid

`NutrientFieldGrid` hoy tiene C, N, P, W (4 canales). Se agrega un 5to canal:
`volatile_signal: f32` + `volatile_freq: f32` por celda.

## No viola axiomas

1. **Axiom 1:** Volátil = qe emitido. Cuesta energía producirlo.
2. **Axiom 2:** `organ_qe` se reduce. Pool invariant conservado.
3. **Axiom 4:** Se disipa a tasa GAS (0.08). Efímero por diseño.
4. **Axiom 6:** Qué órgano emite emerge de densidad, no de rol.
5. **Axiom 7:** Intensidad decae con distancia (attenuation).
6. **Axiom 8:** Solo frecuencias alineadas lo detectan.

## Archivos

| Archivo | Cambio |
|---------|--------|
| `src/batch/scratch.rs` | `NutrientFieldGrid` + campos `volatile_signal`, `volatile_freq` |
| `src/blueprint/equations/volatile_emission.rs` | **NUEVO** — `can_emit`, `emission_rate`, `volatile_decay`, `perceive_volatile` |
| `src/simulation/metabolic/volatile_emission.rs` | **NUEVO** — system en ChemicalLayer |
| `src/simulation/thermodynamic/sensory.rs` | Extender con lectura de volatile grid |
