# ADR-022: 3D/f64 Migration Strategy — Feature Gate, Not Flag Day

**Estado:** Propuesto
**Fecha:** 2026-04-13
**Contexto:** MOLECULAR_DYNAMICS track, sprint [MD-7](../../sprints/MOLECULAR_DYNAMICS/SPRINT_MD7_3D_F64.md)

## Contexto

EntitySlot usa `[f32; 2]` para posicion/velocidad. MD necesita 3D (proteinas
se pliegan en 3D) y f64 (precision acumulada sobre millones de pasos).

## Problema

Migrar a 3D/f64 toca 33+ batch systems. Un "flag day" (cambiar todo a la vez)
es riesgoso y rompe todos los tests existentes.

## Decision

### D1: Feature gate `#[cfg(feature = "md_3d")]`

Los campos 3D se agregan a EntitySlot bajo feature gate. Default off para
batch runs. On para binarios MD. Los 33 sistemas existentes no ven los
campos nuevos y compilan sin cambios.

**Razon:** Zero riesgo de regresion. Migracion incremental.

### D2: LjWorld ya usa f32 arrays propios — migra primero

El standalone `LjWorld` en `lj_fluid.rs` tiene sus propias arrays de posicion/
velocidad. Migrar a `[f64; 3]` ahi no toca EntitySlot. Esto valida la fisica
3D antes de tocar el batch simulator.

**Razon:** Minimo blast radius. Validacion antes de migracion.

### D3: Bridge function `pos_2d()` para backward compatibility

Legacy systems que necesitan 2D leen via `pos_2d(slot) -> [f32; 2]` que
proyecta desde 3D. Cuando todos los sistemas legacy esten migrados, se
remueven los campos 2D.

### D4: NO migrar EntitySlot a f64 para campos no-MD

Solo posicion, velocidad y aceleracion van a f64. Campos biologicos (qe,
frequency_hz, etc.) quedan en f32 — precision suficiente para la simulacion
de vida.

## Alternativas descartadas

**A. Flag day: cambiar todo a [f64; 3]:** Rompe 33+ sistemas, 3000+ tests.
Riesgo inasumible.

**B. Crear un segundo EntitySlot para MD:** Duplica la arena, complica el
spawn/kill, y no reutiliza la infraestructura batch.

## Consecuencias

- EntitySlot crece 48 bytes (6 * f64) con feature gate
- At N=4096: +192KB — cabe en L2 cache
- Tests existentes no cambian (feature off por default)
- LjWorld migra primero como proof of concept
