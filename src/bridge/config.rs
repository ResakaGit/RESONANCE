//! Bridge Optimizer configuration: bands, policies and validation.
//! Generic `Resource` over `B` for `Res<BridgeConfig<B>>` injection (sprint B8).

use core::marker::PhantomData;

use bevy::prelude::Resource;
use serde::{Deserialize, Serialize};

/// Type marker per equation/bridge — compile-time isolation for `BridgeConfig<B>` / `BridgeCache<B>`.
/// Decorator trait with equation: `crate::bridge::Bridgeable` (`bridge/decorator.rs`, sprint B3).
pub trait BridgeKind: Send + Sync + 'static {}

/// Per-equation markers — compile-time routing of `BridgeConfig<B>` / `BridgeCache<B>`.
/// See `docs/arquitectura/blueprint_layer_bridge_optimizer.md` §13 and sprint B8.

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

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct CompetitionNormBridge;
impl BridgeKind for CompetitionNormBridge {}

// ─── Emergence Tiers (ET-1 … ET-16) ────────────────────────────────────────
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct AssociativeMemoryBridge;
impl BridgeKind for AssociativeMemoryBridge {}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct OtherModelBridge;
impl BridgeKind for OtherModelBridge {}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct MemeSpreadBridge;
impl BridgeKind for MemeSpreadBridge {}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct FieldModBridge;
impl BridgeKind for FieldModBridge {}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct SymbiosisBridge;
impl BridgeKind for SymbiosisBridge {}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct EpigeneticBridge;
impl BridgeKind for EpigeneticBridge {}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct SenescenceBridge;
impl BridgeKind for SenescenceBridge {}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct CoalitionBridge;
impl BridgeKind for CoalitionBridge {}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct NicheOverlapBridge;
impl BridgeKind for NicheOverlapBridge {}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct TimescaleBridge;
impl BridgeKind for TimescaleBridge {}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct AggSignalBridge;
impl BridgeKind for AggSignalBridge {}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct TectonicBridge;
impl BridgeKind for TectonicBridge {}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct LODPhysicsBridge;
impl BridgeKind for LODPhysicsBridge {}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct InstitutionBridge;
impl BridgeKind for InstitutionBridge {}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct SymbolBridge;
impl BridgeKind for SymbolBridge {}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct SelfModelBridge;
impl BridgeKind for SelfModelBridge {}

/// Normalization band definition: half-open range except the last (closed at `max`).
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct BandDef {
    pub min: f32,
    pub max: f32,
    /// Representative value for cache / quantized computation.
    pub canonical: f32,
    /// Equilibrium band (affects bias policies in upper layers).
    pub stable: bool,
}

/// Predefined rigidity — quick tuning before full RON (sprint B8).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Rigidity {
    Rigid,
    #[default]
    Moderate,
    Flexible,
    Transparent,
}

/// Eviction / fill policy for bridge caches (B2/B7 sprints).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CachePolicy {
    #[default]
    Lru,
    ContextFill,
}

/// Alias used in blueprints (`EvictionPolicy`); same type as `CachePolicy`.
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

/// Per-equation configuration — generic over the bridge (`B`).
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

/// Requirements: non-decreasing `min`; each band `min <= max`; half-open contiguity:
/// for `i < n-1`, `max_i == min_{i+1}` (no overlaps, no gaps).
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
