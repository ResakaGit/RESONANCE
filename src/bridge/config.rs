//! Configuración del Bridge Optimizer: bandas, políticas y validación.
//! `Resource` por genérico `B` para inyección `Res<BridgeConfig<B>>` (sprint B8).

use core::marker::PhantomData;

use bevy::prelude::Resource;
use serde::{Deserialize, Serialize};

/// Marcador de tipo por ecuación/puente — aislamiento en compile time para `BridgeConfig<B>` / `BridgeCache<B>`.
/// El trait de decorador con ecuación es `crate::bridge::Bridgeable` (`bridge/decorator.rs`, sprint B3).
pub trait BridgeKind: Send + Sync + 'static {}

/// Marcadores por ecuación — routing compile-time de `BridgeConfig<B>` / `BridgeCache<B>`.
/// Ver `docs/arquitectura/blueprint_layer_bridge_optimizer.md` §13 y sprint B8.

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct DensityBridge;
impl BridgeKind for DensityBridge {}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct TemperatureBridge;
impl BridgeKind for TemperatureBridge {}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct PhaseTransitionBridge;
impl BridgeKind for PhaseTransitionBridge {}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct InterferenceBridge;
impl BridgeKind for InterferenceBridge {}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct DissipationBridge;
impl BridgeKind for DissipationBridge {}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct DragBridge;
impl BridgeKind for DragBridge {}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct EngineBridge;
impl BridgeKind for EngineBridge {}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct WillBridge;
impl BridgeKind for WillBridge {}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct CatalysisBridge;
impl BridgeKind for CatalysisBridge {}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct CollisionTransferBridge;
impl BridgeKind for CollisionTransferBridge {}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct OsmosisBridge;
impl BridgeKind for OsmosisBridge {}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct EvolutionSurrogateBridge;
impl BridgeKind for EvolutionSurrogateBridge {}

/// Definición de una banda de normalización: rango half-open salvo la última (cerrada en `max`).
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct BandDef {
    pub min: f32,
    pub max: f32,
    /// Valor representativo para cache / cómputo cuantizado.
    pub canonical: f32,
    /// Banda de equilibrio (afecta políticas de sesgo en capas superiores).
    pub stable: bool,
}

/// Rigidez predefinida — tuning rápido antes de RON completo (sprint B8).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Rigidity {
    Rigid,
    #[default]
    Moderate,
    Flexible,
    Transparent,
}

/// Política de evicción / llenado de cache (implementación en sprint B2/B7).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CachePolicy {
    #[default]
    Lru,
    Lfu,
    ContextFill,
}

/// Nombre usado en blueprints (`EvictionPolicy`); mismo tipo que `CachePolicy`.
pub type EvictionPolicy = CachePolicy;

/// Error al validar bandas antes de usar `BridgeConfig`.
#[derive(Clone, Debug, PartialEq)]
pub enum BandValidationError {
    Empty,
    UnsortedMin {
        index: usize,
    },
    InvalidRange {
        index: usize,
    },
    GapBetweenBands {
        left: usize,
        gap_low: f32,
        gap_high: f32,
    },
    OverlapBetweenBands {
        left: usize,
    },
}

/// Configuración por ecuación — genérica sobre el puente (`B`).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(bound(serialize = "", deserialize = ""))]
pub struct BridgeConfig<B: BridgeKind> {
    pub bands: Vec<BandDef>,
    pub hysteresis_margin: f32,
    pub cache_capacity: usize,
    pub policy: CachePolicy,
    pub enabled: bool,
    pub rigidity: Rigidity,
    #[serde(skip)]
    pub _marker: PhantomData<B>,
}

impl<B: BridgeKind> Resource for BridgeConfig<B> {}

impl<B: BridgeKind> BridgeConfig<B> {
    pub fn new(
        bands: Vec<BandDef>,
        hysteresis_margin: f32,
        cache_capacity: usize,
        policy: CachePolicy,
        enabled: bool,
        rigidity: Rigidity,
    ) -> Result<Self, BandValidationError> {
        validate_bands(&bands)?;
        Ok(Self {
            bands,
            hysteresis_margin,
            cache_capacity,
            policy,
            enabled,
            rigidity,
            _marker: PhantomData,
        })
    }
}

/// Requisitos: `min` no decreciente; cada banda `min <= max`; contigüidad half-open:
/// para `i < n-1` se exige `max_i == min_{i+1}` (sin solapes ni huecos).
pub fn validate_bands(bands: &[BandDef]) -> Result<(), BandValidationError> {
    if bands.is_empty() {
        return Err(BandValidationError::Empty);
    }
    for (i, b) in bands.iter().enumerate() {
        if b.min > b.max {
            return Err(BandValidationError::InvalidRange { index: i });
        }
        if i > 0 && bands[i - 1].min > b.min {
            return Err(BandValidationError::UnsortedMin { index: i });
        }
    }
    for i in 0..bands.len() - 1 {
        let left = &bands[i];
        let right = &bands[i + 1];
        if left.max > right.min {
            return Err(BandValidationError::OverlapBetweenBands { left: i });
        }
        if left.max < right.min {
            return Err(BandValidationError::GapBetweenBands {
                left: i,
                gap_low: left.max,
                gap_high: right.min,
            });
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bridge_config_new_rejects_invalid_bands() {
        let bad = vec![
            BandDef {
                min: 0.0,
                max: 2.0,
                canonical: 1.0,
                stable: true,
            },
            BandDef {
                min: 1.0,
                max: 3.0,
                canonical: 2.0,
                stable: true,
            },
        ];
        assert!(
            BridgeConfig::<DensityBridge>::new(
                bad,
                0.5,
                64,
                CachePolicy::Lru,
                true,
                Rigidity::Moderate,
            )
            .is_err()
        );
    }

    #[test]
    fn bridge_config_new_accepts_contiguous_bands() {
        let good = vec![
            BandDef {
                min: 0.0,
                max: 1.0,
                canonical: 0.5,
                stable: true,
            },
            BandDef {
                min: 1.0,
                max: 2.0,
                canonical: 1.5,
                stable: true,
            },
        ];
        let cfg = BridgeConfig::<DensityBridge>::new(
            good,
            0.5,
            64,
            CachePolicy::Lru,
            true,
            Rigidity::Moderate,
        );
        assert!(cfg.is_ok());
    }
}
