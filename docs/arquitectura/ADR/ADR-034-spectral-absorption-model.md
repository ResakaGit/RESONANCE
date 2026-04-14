# ADR-034: Spectral Absorption Model — Pigmentación desde física de energía

**Estado:** Propuesto
**Fecha:** 2026-04-13
**Contexto:** PLANT_PHYSIOLOGY track, sprint PP-1

## Contexto

Hoy el color de una entidad viene de `frequency_to_tint_rgb(hz)` — un mapeo
directo decorativo de 6 bandas. Esto no tiene relación con la física de absorción.

En la realidad, el color de un objeto es la frecuencia que **no absorbe**.
Una rosa roja absorbe azul/verde y refleja rojo. El pigmento es un subproducto
de la eficiencia de absorción por banda.

## La contribución de Resonance

El color emerge de la **diferencia entre espectro solar y espectro de absorción**:

```
absorption_freq = organ.frequency                    // Axiom 8: cada órgano oscila
reflected_freq = solar_freq - absorption_freq        // complemento espectral
pigment_rgb = frequency_to_tint_rgb(reflected_freq)
```

Un órgano con `freq=300Hz` absorbe la banda centrada en 300 Hz. El color
reflejado es el complemento. **No hay tabla de "Leaf=verde"** — el color
sale de la frecuencia del órgano, que sale de su estado energético.

**Axiom 8** determina qué frecuencias absorbe. **Axiom 4** garantiza que la
absorción no es perfecta — parte de la energía siempre se refleja (albedo > 0).

## Decision

### D1: Reinterpretar albedo_inference como espectro

**Hoy:** `InferredAlbedo` = escalar [0.05, 0.95] que modula brillo.
**Propuesta:** `InferredAlbedo` + `ReflectedSpectrum` (nuevo component, 1 campo: `reflected_freq_hz: f32`).

No se toca `InferredAlbedo` — se agrega un component nuevo que indica QUÉ
frecuencia se refleja, no solo CUÁNTO.

```
reflected_freq = solar_freq - organ.frequency
```

El mesh tint pasa de `frequency_to_tint_rgb(entity_freq)` a
`frequency_to_tint_rgb(reflected_freq)`. Mismo pipeline, diferente input.

### D2: Per-organ color agnóstico (no per-role)

**Rechazada:** tabla de absorción por OrganRole (Leaf=verde, Petal=rojo).
Viola Axiom 6 — no es emergente.

**Decisión:** Cada órgano tiene su propia frecuencia de oscilación, derivada de:

```
organ_freq = entity_freq × density_modulator(organ)
density_modulator = organ.density / entity_density
```

Un órgano denso (tallo) tiene frecuencia cercana al entity → absorbe banda
amplia → color oscuro/marrón. Un órgano de baja densidad (delgado) tiene
frecuencia desplazada → absorbe banda estrecha → refleja más colores → vivo.

**El color de un fruto cambia cuando madura** porque su `organ_qe` aumenta,
lo cual cambia su densidad, lo cual desplaza su frecuencia de absorción.
Emerge sin programar "verde→rojo al madurar".

### D3: Pure fn stateless

```rust
// blueprint/equations/spectral_absorption.rs
pub fn reflected_frequency(solar_freq: f32, absorption_freq: f32) -> f32
pub fn organ_frequency(entity_freq: f32, organ_density: f32, entity_density: f32) -> f32
pub fn spectral_tint_rgb(reflected_freq: f32, albedo: f32) -> [f32; 3]
```

Cache signature: `organ_density × entity_freq × solar_freq` — cambia solo si
el sol se mueve, el entity muta, o el órgano cambia de densidad. Ideal para cache.

## No viola axiomas

1. **Axiom 1:** Color = energía reflejada. Tiene costo energético (lo absorbido - lo reflejado).
2. **Axiom 4:** Absorción < 100%. Siempre refleja algo (albedo mínimo 0.05).
3. **Axiom 6:** Color emerge de la frecuencia del órgano, no de su rol.
4. **Axiom 8:** Absorción es frequency-selective. Determinada por oscilación del órgano.

## Archivos

| Archivo | Cambio |
|---------|--------|
| `src/layers/reflected_spectrum.rs` | **NUEVO** — `ReflectedSpectrum { reflected_freq_hz: f32 }` SparseSet |
| `src/blueprint/equations/spectral_absorption.rs` | **NUEVO** — 3 pure fns agnósticas |
| `src/simulation/metabolic/morphogenesis.rs` | Extender `albedo_inference_system` → escribir `ReflectedSpectrum` |
| `src/blueprint/equations/entity_shape.rs` | `frequency_to_tint_rgb` ahora lee `reflected_freq` si disponible |
