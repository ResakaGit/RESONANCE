//! Normalización escalar por bandas + histéresis. Stateless, sin alloc en hot path.
//! Cuantización Vec2: sector angular (atan2 + floor) + lookup de unitarios canónicos (8/16/32).
//! Convención de rango escalar: `[min, max)` para todas salvo la última `[min, max]` cerrada.

use core::f32::consts::TAU;

use crate::math_types::Vec2;

use crate::bridge::config::BandDef;

// SSOT en `bridge::constants`; reexport para rutas `bridge::normalize::VEC2_*`.
pub use crate::bridge::constants::{VEC2_DIRECTION_ZERO_EPS_SQ, VEC2_STATIC_SECTOR};

/// Unitarios canónicos por sector: índice `i` = ángulo `i * 2π/8` desde +X (E → NE → …).
#[allow(clippy::approx_constant, clippy::excessive_precision)]
pub const CANONICAL_DIRECTIONS_8: [Vec2; 8] = [
    Vec2::new(1.0000000000_f32, 0.0000000000_f32),
    Vec2::new(0.7071067812_f32, 0.7071067812_f32),
    Vec2::new(0.0000000000_f32, 1.0000000000_f32),
    Vec2::new(-0.7071067812_f32, 0.7071067812_f32),
    Vec2::new(-1.0000000000_f32, 0.0000000000_f32),
    Vec2::new(-0.7071067812_f32, -0.7071067812_f32),
    Vec2::new(-0.0000000000_f32, -1.0000000000_f32),
    Vec2::new(0.7071067812_f32, -0.7071067812_f32),
];

/// Subdivisión equidistante; sector `i` alinea con `i * 2π/16` desde +X.
#[allow(clippy::approx_constant, clippy::excessive_precision)]
pub const CANONICAL_DIRECTIONS_16: [Vec2; 16] = [
    Vec2::new(1.0000000000_f32, 0.0000000000_f32),
    Vec2::new(0.9238795325_f32, 0.3826834324_f32),
    Vec2::new(0.7071067812_f32, 0.7071067812_f32),
    Vec2::new(0.3826834324_f32, 0.9238795325_f32),
    Vec2::new(0.0000000000_f32, 1.0000000000_f32),
    Vec2::new(-0.3826834324_f32, 0.9238795325_f32),
    Vec2::new(-0.7071067812_f32, 0.7071067812_f32),
    Vec2::new(-0.9238795325_f32, 0.3826834324_f32),
    Vec2::new(-1.0000000000_f32, 0.0000000000_f32),
    Vec2::new(-0.9238795325_f32, -0.3826834324_f32),
    Vec2::new(-0.7071067812_f32, -0.7071067812_f32),
    Vec2::new(-0.3826834324_f32, -0.9238795325_f32),
    Vec2::new(-0.0000000000_f32, -1.0000000000_f32),
    Vec2::new(0.3826834324_f32, -0.9238795325_f32),
    Vec2::new(0.7071067812_f32, -0.7071067812_f32),
    Vec2::new(0.9238795325_f32, -0.3826834324_f32),
];

/// Subdivisión equidistante; sector `i` alinea con `i * 2π/32` desde +X.
#[allow(clippy::approx_constant, clippy::excessive_precision)]
pub const CANONICAL_DIRECTIONS_32: [Vec2; 32] = [
    Vec2::new(1.0000000000_f32, 0.0000000000_f32),
    Vec2::new(0.9807852804_f32, 0.1950903220_f32),
    Vec2::new(0.9238795325_f32, 0.3826834324_f32),
    Vec2::new(0.8314696123_f32, 0.5555702330_f32),
    Vec2::new(0.7071067812_f32, 0.7071067812_f32),
    Vec2::new(0.5555702330_f32, 0.8314696123_f32),
    Vec2::new(0.3826834324_f32, 0.9238795325_f32),
    Vec2::new(0.1950903220_f32, 0.9807852804_f32),
    Vec2::new(0.0000000000_f32, 1.0000000000_f32),
    Vec2::new(-0.1950903220_f32, 0.9807852804_f32),
    Vec2::new(-0.3826834324_f32, 0.9238795325_f32),
    Vec2::new(-0.5555702330_f32, 0.8314696123_f32),
    Vec2::new(-0.7071067812_f32, 0.7071067812_f32),
    Vec2::new(-0.8314696123_f32, 0.5555702330_f32),
    Vec2::new(-0.9238795325_f32, 0.3826834324_f32),
    Vec2::new(-0.9807852804_f32, 0.1950903220_f32),
    Vec2::new(-1.0000000000_f32, 0.0000000000_f32),
    Vec2::new(-0.9807852804_f32, -0.1950903220_f32),
    Vec2::new(-0.9238795325_f32, -0.3826834324_f32),
    Vec2::new(-0.8314696123_f32, -0.5555702330_f32),
    Vec2::new(-0.7071067812_f32, -0.7071067812_f32),
    Vec2::new(-0.5555702330_f32, -0.8314696123_f32),
    Vec2::new(-0.3826834324_f32, -0.9238795325_f32),
    Vec2::new(-0.1950903220_f32, -0.9807852804_f32),
    Vec2::new(-0.0000000000_f32, -1.0000000000_f32),
    Vec2::new(0.1950903220_f32, -0.9807852804_f32),
    Vec2::new(0.3826834324_f32, -0.9238795325_f32),
    Vec2::new(0.5555702330_f32, -0.8314696123_f32),
    Vec2::new(0.7071067812_f32, -0.7071067812_f32),
    Vec2::new(0.8314696123_f32, -0.5555702330_f32),
    Vec2::new(0.9238795325_f32, -0.3826834324_f32),
    Vec2::new(0.9807852804_f32, -0.1950903220_f32),
];

/// Contiene `value` en la banda `index` según la convención half-open.
#[inline]
pub fn band_contains_half_open(value: f32, bands: &[BandDef], index: usize) -> bool {
    let b = &bands[index];
    let last = index + 1 == bands.len();
    if last {
        value >= b.min && value <= b.max
    } else {
        value >= b.min && value < b.max
    }
}

/// Búsqueda binaria O(log N): mayor índice con `min <= value`, luego comprueba contención.
pub fn band_index_of(value: f32, bands: &[BandDef]) -> Option<usize> {
    if bands.is_empty() {
        return None;
    }
    let n = bands.len();
    let mut lo = 0usize;
    let mut hi = n - 1;
    let mut best: Option<usize> = None;
    while lo <= hi {
        let mid = (lo + hi) / 2;
        if bands[mid].min <= value {
            best = Some(mid);
            lo = mid + 1;
        } else if mid == 0 {
            break;
        } else {
            hi = mid - 1;
        }
    }
    let idx = best?;
    if band_contains_half_open(value, bands, idx) {
        return Some(idx);
    }
    None
}

/// Normaliza un escalar al canónico de su banda; aplica histéresis si hay hint del tick previo.
/// Valores fuera del dominio cubierto se proyectan a la banda extrema más cercana.
///
/// # Panics
///
/// Si `bands` está vacío — usar siempre bandas validadas (`validate_bands` / `BridgeConfig::new`).
pub fn normalize_scalar(
    value: f32,
    bands: &[BandDef],
    hysteresis: f32,
    current_band_hint: Option<usize>,
) -> (f32, usize) {
    assert!(
        !bands.is_empty(),
        "normalize_scalar: bands must be non-empty"
    );
    if let Some(h) = current_band_hint {
        if h < bands.len() {
            let b = &bands[h];
            let low = b.min - hysteresis;
            let high = b.max + hysteresis;
            if value >= low && value <= high {
                return (b.canonical, h);
            }
        }
    }
    if let Some(i) = band_index_of(value, bands) {
        return (bands[i].canonical, i);
    }
    if value < bands[0].min {
        return (bands[0].canonical, 0);
    }
    let last = bands.len() - 1;
    (bands[last].canonical, last)
}

/// Redondea a N decimales — claves de cache más uniformes (sprint B2).
#[inline]
pub fn quantize_precision(value: f32, precision_decimals: u8) -> f32 {
    let p = 10f32.powi((precision_decimals as i32).min(9));
    (value * p).round() / p
}

/// Índice de sector angular en `[0, sectors)`; `VEC2_STATIC_SECTOR` si `dir ≈ 0` o `sectors < 2`.
///
/// Bines: `floor((atan2(y,x) normalizado a [0,2π)) / (2π/sectors))`.
#[inline]
pub fn direction_sector(dir: Vec2, sectors: u8) -> u8 {
    if sectors < 2 {
        return VEC2_STATIC_SECTOR;
    }
    if !dir.x.is_finite() || !dir.y.is_finite() {
        return VEC2_STATIC_SECTOR;
    }
    if dir.length_squared() <= VEC2_DIRECTION_ZERO_EPS_SQ {
        return VEC2_STATIC_SECTOR;
    }
    let angle = dir.y.atan2(dir.x);
    let mut a = angle;
    if a < 0.0 {
        a += TAU;
    }
    let n = sectors as f32;
    let mut s = ((a / TAU) * n).floor() as i32;
    if s >= sectors as i32 {
        s = sectors as i32 - 1;
    }
    if s < 0 {
        s = 0;
    }
    s as u8
}

#[inline]
fn canonical_unit_for_sector_index(sector: u8, sectors: u8) -> Vec2 {
    debug_assert!((sector as u32) < (sectors as u32));
    match sectors {
        8 => CANONICAL_DIRECTIONS_8[sector as usize],
        16 => CANONICAL_DIRECTIONS_16[sector as usize],
        32 => CANONICAL_DIRECTIONS_32[sector as usize],
        _ => Vec2::from_angle((sector as f32 / sectors as f32) * TAU),
    }
}

/// Cuantiza dirección al unitario canónico del **bin angular** `[k·2π/N, (k+1)·2π/N)` (lookup 8/16/32; resto `from_angle`).
#[inline]
pub fn normalize_direction(dir: Vec2, sectors: u8) -> Vec2 {
    let s = direction_sector(dir, sectors);
    if s == VEC2_STATIC_SECTOR {
        return Vec2::ZERO;
    }
    canonical_unit_for_sector_index(s, sectors)
}

/// Cuantiza magnitud no negativa con las mismas bandas que escalares (sin histéresis).
///
/// # Panics
///
/// Si `bands` está vacío — igual que `normalize_scalar`.
#[inline]
pub fn normalize_magnitude(magnitude: f32, bands: &[BandDef]) -> f32 {
    if !magnitude.is_finite() {
        return 0.0;
    }
    let v = magnitude.max(0.0);
    let (c, _) = normalize_scalar(v, bands, 0.0, None);
    c
}

/// Dirección cuantizada (sectores) × magnitud cuantizada (bandas).
#[inline]
pub fn normalize_vec2(vec: Vec2, sectors: u8, magnitude_bands: &[BandDef]) -> Vec2 {
    let unit = normalize_direction(vec, sectors);
    let len = vec.length();
    let m = normalize_magnitude(len, magnitude_bands);
    unit * m
}

/// Key cache Vec2: empaqueta sector de dirección (incl. `VEC2_STATIC_SECTOR`) y banda de magnitud.
/// Layout: 16 bits bajos = `magnitude_band`, bits superiores = `sector` (hasta 255; sin colisión con banda si N bandas < 2¹⁶).
#[inline]
pub fn vec2_cache_key(sector: u8, magnitude_band: u16) -> u64 {
    ((sector as u64) << 16) | (magnitude_band as u64)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::blueprint::constants::{GAS_TRANSITION, LIQUID_TRANSITION, SOLID_TRANSITION};
    use crate::bridge::config::{BandDef, BandValidationError, validate_bands};
    use bevy::math::Vec2;

    /// Bandas tipo doc: 47.3 cae en [40, 55) → canónico 50.
    fn bands_density_example() -> Vec<BandDef> {
        vec![
            BandDef {
                min: 40.0,
                max: 55.0,
                canonical: 50.0,
                stable: true,
            },
            BandDef {
                min: 55.0,
                max: 100.0,
                canonical: 80.0,
                stable: false,
            },
        ]
    }

    #[test]
    fn normalize_47_3_matches_canonical_band() {
        let bands = bands_density_example();
        let (c, i) = normalize_scalar(47.3, &bands, 0.0, None);
        assert_eq!(i, 0);
        assert_eq!(c, 50.0);
    }

    #[test]
    fn hysteresis_keeps_band_near_edge() {
        // Dos bandas contiguas [0,10), [10,20) — última cerrada si es fin único
        let bands = vec![
            BandDef {
                min: 0.0,
                max: 10.0,
                canonical: 5.0,
                stable: true,
            },
            BandDef {
                min: 10.0,
                max: 20.0,
                canonical: 15.0,
                stable: true,
            },
        ];
        validate_bands(&bands).unwrap();
        // Sin hint: 10.0 está en banda 1 (min 10 <= 10 < 20 half-open en primera? 10 está en segunda)
        let (_c1, i1) = normalize_scalar(10.0, &bands, 0.0, None);
        assert_eq!(i1, 1);
        // Con hint 0 y h=1.0: 10.5 <= 10+1 en sticky de banda 0? max_0=10, high=11, 10.5 en [ -1, 11 ] sí → se queda banda 0
        let (c_stay, i_stay) = normalize_scalar(10.5, &bands, 1.0, Some(0));
        assert_eq!(i_stay, 0);
        assert_eq!(c_stay, 5.0);
    }

    #[test]
    fn hysteresis_breaks_beyond_margin() {
        let bands = vec![
            BandDef {
                min: 0.0,
                max: 10.0,
                canonical: 5.0,
                stable: true,
            },
            BandDef {
                min: 10.0,
                max: 30.0,
                canonical: 20.0,
                stable: true,
            },
        ];
        validate_bands(&bands).unwrap();
        // hint 0, h=1 → sticky hasta max+h=11
        let (_c, i) = normalize_scalar(11.5, &bands, 1.0, Some(0));
        assert_eq!(i, 1);
    }

    #[test]
    fn out_of_range_maps_to_extreme_band() {
        let bands = vec![
            BandDef {
                min: 10.0,
                max: 20.0,
                canonical: 15.0,
                stable: true,
            },
            BandDef {
                min: 20.0,
                max: 30.0,
                canonical: 25.0,
                stable: true,
            },
        ];
        validate_bands(&bands).unwrap();
        let (c_lo, i_lo) = normalize_scalar(0.0, &bands, 0.0, None);
        assert_eq!(i_lo, 0);
        assert_eq!(c_lo, 15.0);
        let (c_hi, i_hi) = normalize_scalar(500.0, &bands, 0.0, None);
        assert_eq!(i_hi, 1);
        assert_eq!(c_hi, 25.0);
    }

    #[test]
    fn validate_bands_rejects_gap_and_overlap() {
        let gap = vec![
            BandDef {
                min: 0.0,
                max: 1.0,
                canonical: 0.5,
                stable: true,
            },
            BandDef {
                min: 2.0,
                max: 3.0,
                canonical: 2.5,
                stable: true,
            },
        ];
        assert!(matches!(
            validate_bands(&gap),
            Err(BandValidationError::GapBetweenBands { .. })
        ));
        let overlap = vec![
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
        assert!(matches!(
            validate_bands(&overlap),
            Err(BandValidationError::OverlapBetweenBands { .. })
        ));
    }

    /// Bandas en temperatura equivalente alineadas a `SOLID_TRANSITION`, `LIQUID_TRANSITION`, `GAS_TRANSITION` (eb=1).
    fn temperature_bands_eb1() -> Vec<BandDef> {
        vec![
            BandDef {
                min: 0.0,
                max: SOLID_TRANSITION,
                canonical: SOLID_TRANSITION * 0.5,
                stable: true,
            },
            BandDef {
                min: SOLID_TRANSITION,
                max: LIQUID_TRANSITION,
                canonical: (SOLID_TRANSITION + LIQUID_TRANSITION) * 0.5,
                stable: true,
            },
            BandDef {
                min: LIQUID_TRANSITION,
                max: GAS_TRANSITION,
                canonical: (LIQUID_TRANSITION + GAS_TRANSITION) * 0.5,
                stable: true,
            },
            BandDef {
                min: GAS_TRANSITION,
                max: f32::MAX,
                canonical: GAS_TRANSITION * 1.5,
                stable: false,
            },
        ]
    }

    #[test]
    fn default_temperature_bands_match_engine_constants() {
        let bands = temperature_bands_eb1();
        validate_bands(&bands).unwrap();
        assert_eq!(bands[0].max, SOLID_TRANSITION);
        assert_eq!(bands[1].min, SOLID_TRANSITION);
        assert_eq!(bands[2].min, LIQUID_TRANSITION);
        assert_eq!(bands[3].min, GAS_TRANSITION);
    }

    #[test]
    fn band_index_of_binary_matches_linear_many_bands() {
        let mut bands = Vec::with_capacity(128);
        for i in 0..127 {
            let v = i as f32;
            bands.push(BandDef {
                min: v,
                max: v + 1.0,
                canonical: v + 0.5,
                stable: true,
            });
        }
        bands.push(BandDef {
            min: 127.0,
            max: 128.0,
            canonical: 127.5,
            stable: true,
        });
        validate_bands(&bands).unwrap();

        let linear = |value: f32| -> Option<usize> {
            bands
                .iter()
                .enumerate()
                .find(|(i, _)| band_contains_half_open(value, &bands, *i))
                .map(|(i, _)| i)
        };

        for step in 0..1000 {
            let value = step as f32 * 0.128;
            assert_eq!(band_index_of(value, &bands), linear(value));
        }
    }

    #[test]
    fn quantize_precision_rounds() {
        assert!((quantize_precision(47.34567, 2) - 47.35).abs() < 1e-4);
    }

    // --- Sprint B6: Vec2 ---

    #[test]
    fn normalize_direction_east_8_sectors() {
        let u = normalize_direction(Vec2::new(1.0, 0.0), 8);
        assert!((u - Vec2::X).length() < 1e-5);
        assert_eq!(direction_sector(Vec2::new(1.0, 0.0), 8), 0);
    }

    #[test]
    fn normalize_direction_ne_8_sectors() {
        let u = normalize_direction(Vec2::new(0.7, 0.7), 8);
        let ne = CANONICAL_DIRECTIONS_8[1];
        assert!((u - ne).length() < 1e-5);
        assert_eq!(direction_sector(Vec2::new(0.7, 0.7), 8), 1);
    }

    #[test]
    fn vec2_zero_is_static_sector() {
        assert_eq!(direction_sector(Vec2::ZERO, 8), VEC2_STATIC_SECTOR);
        assert_eq!(normalize_direction(Vec2::ZERO, 8), Vec2::ZERO);
    }

    #[test]
    fn eight_sectors_partition_full_circle() {
        for i in 0..8 {
            let mid = (i as f32 + 0.5) * TAU / 8.0;
            let v = Vec2::new(mid.cos(), mid.sin());
            assert_eq!(direction_sector(v, 8), i as u8);
        }
    }

    #[test]
    fn vec2_cache_key_differs_by_sector() {
        let a = vec2_cache_key(0, 0);
        let b = vec2_cache_key(1, 0);
        assert_ne!(a, b);
        let c = vec2_cache_key(VEC2_STATIC_SECTOR, 0);
        assert_ne!(a, c);
    }

    #[test]
    fn vec2_cache_key_differs_by_magnitude_band() {
        assert_ne!(vec2_cache_key(3, 0), vec2_cache_key(3, 1));
    }

    #[test]
    fn direction_sector_non_finite_is_static() {
        assert_eq!(
            direction_sector(Vec2::new(f32::NAN, 0.0), 8),
            VEC2_STATIC_SECTOR
        );
        assert_eq!(
            direction_sector(Vec2::new(f32::INFINITY, 0.0), 8),
            VEC2_STATIC_SECTOR
        );
    }

    #[test]
    fn normalize_magnitude_clamps_negative_and_matches_scalar() {
        let bands = bands_density_example();
        let m = normalize_magnitude(-10.0, &bands);
        let (c, _) = normalize_scalar(0.0, &bands, 0.0, None);
        assert_eq!(m, c);
        let m2 = normalize_magnitude(47.3, &bands);
        let (c2, _) = normalize_scalar(47.3, &bands, 0.0, None);
        assert_eq!(m2, c2);
    }

    #[test]
    fn normalize_magnitude_nan_is_zero() {
        let bands = bands_density_example();
        assert_eq!(normalize_magnitude(f32::NAN, &bands), 0.0);
    }

    fn assert_sectors_partition(n: u8, table: &[Vec2]) {
        for i in 0..n {
            let mid = (i as f32 + 0.5) * TAU / n as f32;
            let v = Vec2::new(mid.cos(), mid.sin());
            assert_eq!(direction_sector(v, n), i);
            let u = normalize_direction(v, n);
            assert!((u - table[i as usize]).length() < 1e-5, "sector {i} n={n}");
        }
    }

    #[test]
    fn sixteen_sectors_partition_centers() {
        assert_sectors_partition(16, &CANONICAL_DIRECTIONS_16);
    }

    #[test]
    fn thirty_two_sectors_partition_centers() {
        assert_sectors_partition(32, &CANONICAL_DIRECTIONS_32);
    }

    #[test]
    fn normalize_vec2_same_direction_different_magnitude_different_band() {
        let bands = vec![
            BandDef {
                min: 0.0,
                max: 5.0,
                canonical: 2.0,
                stable: true,
            },
            BandDef {
                min: 5.0,
                max: 100.0,
                canonical: 50.0,
                stable: true,
            },
        ];
        validate_bands(&bands).unwrap();
        let dir = Vec2::new(1.0, 0.0);
        let v_slow = dir * 2.0;
        let v_fast = dir * 50.0;
        assert_eq!(direction_sector(v_slow, 8), direction_sector(v_fast, 8));
        let out_slow = normalize_vec2(v_slow, 8, &bands);
        let out_fast = normalize_vec2(v_fast, 8, &bands);
        assert!((out_slow - Vec2::X * 2.0).length() < 1e-4);
        assert!((out_fast - Vec2::X * 50.0).length() < 1e-4);
        assert_ne!(out_slow, out_fast);
    }

    #[test]
    fn normalize_direction_symmetry_near_east() {
        let s0 = direction_sector(Vec2::X, 8);
        let s1 = direction_sector(Vec2::new(0.99, 0.01), 8);
        assert_eq!(s0, s1);
    }
}
