//! Macros declarativas para reducir boilerplate en impls del trait [`Bridgeable`].
//!
//! Dos variantes según cuánto del impl es idéntico:
//!
//! - [`impl_bridgeable_scalar!`] — bridge f32 → f32 con normalización escalar estándar.
//!   Genera la impl completa; solo requiere `$bridge` y el cuerpo de `compute`.
//!
//! - [`impl_bridgeable_scalar_io!`] — bridge Input → f32 donde `normalize` y `cache_key`
//!   son custom pero `into_cached` / `from_cached` son siempre `CachedValue::Scalar`.
//!   Genera solo esos dos métodos; `normalize`, `cache_key` y `compute` se proveen inline.

/// Genera la impl completa de `Bridgeable` para un bridge cuyo Input y Output son `f32`,
/// usando `normalize_scalar` estándar y `CachedValue::Scalar` para caché.
///
/// # Uso
///
/// ```rust,ignore
/// impl_bridgeable_scalar!(DensityBridge, |normalized| normalized);
/// impl_bridgeable_scalar!(TemperatureBridge, |normalized| equations::equivalent_temperature(normalized));
/// ```
///
/// Equivale a escribir manualmente las ~30 LOC de normalize/cache_key/compute/into_cached/from_cached.
#[macro_export]
macro_rules! impl_bridgeable_scalar {
    ($bridge:ty, |$input:ident| $compute_body:expr) => {
        impl $crate::bridge::decorator::Bridgeable for $bridge {
            type Input = f32;
            type Output = f32;

            #[inline]
            fn normalize(
                input: Self::Input,
                config: &$crate::bridge::config::BridgeConfig<Self>,
                band_hint: Option<usize>,
            ) -> Self::Input {
                $crate::bridge::normalize::normalize_scalar(
                    input,
                    &config.bands,
                    config.hysteresis_margin,
                    band_hint,
                )
                .0
            }

            #[inline]
            fn cache_key(normalized: Self::Input) -> u64 {
                $crate::bridge::decorator::hash_inputs(&[f32::to_bits(normalized) as u64])
            }

            #[inline]
            fn compute($input: Self::Input) -> Self::Output {
                $compute_body
            }

            #[inline]
            fn into_cached(value: Self::Output) -> $crate::bridge::cache::CachedValue {
                $crate::bridge::cache::CachedValue::Scalar(value)
            }

            #[inline]
            fn from_cached(
                value: $crate::bridge::cache::CachedValue,
            ) -> Option<Self::Output> {
                if let $crate::bridge::cache::CachedValue::Scalar(v) = value {
                    Some(v)
                } else {
                    None
                }
            }
        }
    };
}

/// Genera solo `into_cached` y `from_cached` para bridges cuyo Output es `f32`,
/// dejando `normalize`, `cache_key` y `compute` al caller.
///
/// Útil cuando el Input es un struct compuesto (normalización custom) pero la salida
/// cacheada es siempre `CachedValue::Scalar`.
///
/// # Uso
///
/// ```rust,ignore
/// impl Bridgeable for InterferenceBridge {
///     type Input = InterferenceEquationInput;
///     type Output = f32;
///
///     fn normalize(...) -> Self::Input { /* custom */ }
///     fn cache_key(...) -> u64 { /* custom */ }
///     fn compute(...) -> Self::Output { /* custom */ }
///
///     impl_bridgeable_scalar_io!();
/// }
/// ```
#[macro_export]
macro_rules! impl_bridgeable_scalar_io {
    () => {
        #[inline]
        fn into_cached(value: Self::Output) -> $crate::bridge::cache::CachedValue {
            $crate::bridge::cache::CachedValue::Scalar(value)
        }

        #[inline]
        fn from_cached(
            value: $crate::bridge::cache::CachedValue,
        ) -> Option<Self::Output> {
            if let $crate::bridge::cache::CachedValue::Scalar(v) = value {
                Some(v)
            } else {
                None
            }
        }
    };
}

pub use impl_bridgeable_scalar;
pub use impl_bridgeable_scalar_io;
