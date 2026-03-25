# Sprint SM-5 — Macro impl_bridgeable!

**Módulo:** `src/bridge/`
**Tipo:** Eliminar boilerplate repetido. Macro declarativa que genera impls de `Bridgeable`.
**Onda:** B — Requiere SM-3 (bridge ya reorganizado).
**Estado:** ⏳ Pendiente

## Objetivo

Los 11+ bridge types repiten ~30 líneas idénticas de boilerplate (`normalize`, `cache_key`, `into_cached`, `from_cached`). Solo `compute()` difiere por bridge. Crear una macro `impl_bridgeable!` que genere el boilerplate y solo pida la función `compute`.

## Diagnóstico

### Patrón repetido (por cada bridge type)

```rust
impl Bridgeable for FooBridge {
    type Input = f32;
    type Output = f32;

    fn normalize(&self, input: Self::Input) -> Self::Input {
        normalize_scalar(input, self.min, self.max).0    // ← IDÉNTICO
    }

    fn cache_key(&self, normalized: &Self::Input) -> u64 {
        hash_inputs(&[f32::to_bits(*normalized) as u64]) // ← IDÉNTICO
    }

    fn compute(&self, normalized: Self::Input) -> Self::Output {
        equations::foo_calculation(normalized)            // ← ÚNICO por bridge
    }

    fn into_cached(&self, output: Self::Output) -> CachedValue {
        CachedValue::Scalar(output)                      // ← IDÉNTICO para f32
    }

    fn from_cached(&self, cached: &CachedValue) -> Option<Self::Output> {
        if let CachedValue::Scalar(v) = cached {         // ← IDÉNTICO para f32
            Some(*v)
        } else { None }
    }
}
```

Esto se repite 11 veces con variantes menores (Input tipo, Output tipo, compute body).

### Conteo de boilerplate

| Método | Líneas | Veces repetido | Total duplicado |
|--------|--------|----------------|-----------------|
| `normalize` | ~3 | 11 | 33 |
| `cache_key` | ~3 | 11 | 33 |
| `into_cached` | ~3 | 11 | 33 |
| `from_cached` | ~5 | 11 | 55 |
| **Total** | | | **~154 LOC** |

## Diseño de la macro

```rust
/// Genera impl Bridgeable para un bridge type escalar (f32 → f32).
/// Solo requiere el cuerpo de compute().
macro_rules! impl_bridgeable_scalar {
    (
        $bridge:ty,
        |$self_:ident, $input:ident| $compute:expr
    ) => {
        impl Bridgeable for $bridge {
            type Input = f32;
            type Output = f32;

            fn normalize(&self, input: Self::Input) -> Self::Input {
                normalize_scalar(input, self.min, self.max).0
            }

            fn cache_key(&self, normalized: &Self::Input) -> u64 {
                hash_inputs(&[f32::to_bits(*normalized) as u64])
            }

            fn compute(&self, $input: Self::Input) -> Self::Output {
                let $self_ = self;
                $compute
            }

            fn into_cached(&self, output: Self::Output) -> CachedValue {
                CachedValue::Scalar(output)
            }

            fn from_cached(&self, cached: &CachedValue) -> Option<Self::Output> {
                if let CachedValue::Scalar(v) = cached { Some(*v) } else { None }
            }
        }
    };
}

/// Variante para bridges con Input tuple (f32, f32) → Output custom.
macro_rules! impl_bridgeable_tuple {
    (
        $bridge:ty,
        Input = ($($in_ty:ty),+),
        Output = $out_ty:ty,
        cached_variant = $variant:ident,
        |$self_:ident, $input:ident| $compute:expr,
        |$out:ident| $into_cached:expr,
        |$cached:ident| $from_cached:expr
    ) => {
        impl Bridgeable for $bridge {
            type Input = ($($in_ty),+);
            type Output = $out_ty;

            fn normalize(&self, input: Self::Input) -> Self::Input {
                // Tuple normalization — per bridge customization via fields
                input // default: no normalization for tuples
            }

            fn cache_key(&self, normalized: &Self::Input) -> u64 {
                // Hash tuple fields
                let ($($in_ty),+) = normalized;
                hash_inputs(&[$(f32::to_bits(*$in_ty) as u64),+])
            }

            fn compute(&self, $input: Self::Input) -> Self::Output {
                let $self_ = self;
                $compute
            }

            fn into_cached(&self, $out: Self::Output) -> CachedValue {
                $into_cached
            }

            fn from_cached(&self, $cached: &CachedValue) -> Option<Self::Output> {
                $from_cached
            }
        }
    };
}
```

### Uso (reemplaza ~30 LOC por ~5 LOC por bridge)

```rust
// Antes: 30 LOC
impl Bridgeable for DensityBridge { ... }

// Después: 4 LOC
impl_bridgeable_scalar!(DensityBridge, |_self, input| input);

impl_bridgeable_scalar!(TemperatureBridge, |_self, input| {
    equations::equivalent_temperature(input)
});

impl_bridgeable_scalar!(DissipationBridge, |_self, input| {
    equations::dissipation_effective(input, _self.drag_coeff)
});
```

## Pasos de implementación

### SM-5A: Crear macro en `bridge/macros.rs`

1. Crear `src/bridge/macros.rs`.
2. Implementar `impl_bridgeable_scalar!` para el caso más común (f32 → f32).
3. Implementar `impl_bridgeable_tuple!` para bridges con inputs compuestos.
4. Agregar `#[macro_use] mod macros;` en `bridge/mod.rs` (o `pub(crate) use`).

### SM-5B: Migrar bridges escalares

1. Identificar qué bridges son f32 → f32 (mayoría: Density, Temperature, Dissipation, Drag, Osmosis, Engine).
2. Reemplazar cada `impl Bridgeable for X { ... }` con `impl_bridgeable_scalar!(X, |s, i| ...)`.
3. `cargo test --lib` después de cada bridge migrado.

### SM-5C: Migrar bridges tuple/custom

1. Bridges con Input compuesto: `PhaseTransitionBridge`, `CollisionTransferBridge`, etc.
2. Usar `impl_bridgeable_tuple!` o dejar manual si la customización es demasiado específica.
3. **No forzar.** Si un bridge no encaja en la macro, dejarlo manual. La meta es eliminar boilerplate, no crear abstracciones forzadas.

### SM-5D: Eliminar código muerto

1. Verificar que no quedan impls manuales que la macro reemplazó.
2. `cargo test --lib` final.

## Tácticas

- **Macro declarativa (`macro_rules!`), no procedural.** No necesitamos proc-macros ni syn/quote. Las variantes son finitas y predecibles.
- **Dos variantes bastan.** `scalar` (90% de los bridges) y `tuple` (10%). Si algún bridge no encaja, dejarlo manual.
- **No over-engineer.** Si un bridge necesita normalización custom, overridear el método manualmente tras usar la macro. O simplemente no usar la macro para ese bridge.
- **Tests existentes validan.** Los tests de cada bridge ya existen. La macro no cambia behavior — solo genera el mismo código.

## NO hace

- No cambia el trait `Bridgeable`.
- No cambia la API de `BridgeCache<B>`.
- No añade nuevos bridge types.
- No modifica lógica de `compute()` de ningún bridge.
- No introduce proc-macros ni dependencias nuevas.

## Criterios de aceptación

- `cargo test --lib` pasa sin regresión.
- Al menos 8 de 11 bridge types usan la macro (los 3 más custom pueden quedar manuales).
- LOC total en `bridge/impls/` se reduce ≥30% (estimado: ~154 LOC eliminadas).
- La macro está documentada con `///` y un ejemplo de uso.

## Referencias

- `src/bridge/impls/physics.rs` — impls de Bridgeable (post SM-3)
- `src/bridge/impls/ops.rs` — operaciones bridged (post SM-3)
- `src/bridge/cache.rs` — trait `Bridgeable`, `CachedValue`
- `src/bridge/normalize.rs` — `normalize_scalar`, `hash_inputs`
