//! Constantes de tuning compartidas por normalización Vec2 e interferencia bridge.

/// Sector reservado para velocidad nula (sin dirección). Ver `docs/design/BRIDGE_OPTIMIZER.md` §9.
pub const VEC2_STATIC_SECTOR: u8 = 255;

/// Umbral² para tratar `Vec2` como cero (estático).
pub const VEC2_DIRECTION_ZERO_EPS_SQ: f32 = 1e-20;

/// Ventana temporal de cuantización de interferencia (s). Misma magnitud que en el sprint (~0.1s).
pub const INTERFERENCE_TIME_QUANT_S: f32 = 0.1;

/// Sectores de fase para cuantización (16 = `CANONICAL_DIRECTIONS_16` / blueprint).
pub const INTERFERENCE_PHASE_SECTORS: u8 = 16;
