//! FBM sobre Perlin: heightmap base determinista (Sprint T2, docs/design/TOPOLOGY.md).

use std::cmp::Ordering;

use crate::math_types::Vec2;
use noise::{NoiseFn, Perlin};
use serde::{Deserialize, Serialize};

use crate::topology::constants::{ALTITUDE_MAX_DEFAULT, ALTITUDE_MIN_DEFAULT};

fn default_min_height() -> f32 {
    ALTITUDE_MIN_DEFAULT
}

fn default_max_height() -> f32 {
    ALTITUDE_MAX_DEFAULT
}

/// Parámetros de noise para RON / tuning (T9); Copy y sin estado.
#[derive(Copy, Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct NoiseParams {
    pub octaves: u32,
    pub frequency: f64,
    pub amplitude: f64,
    pub lacunarity: f64,
    pub persistence: f64,
    /// Rango tras `normalize_heightmap` (metros).
    #[serde(default = "default_min_height")]
    pub min_height: f32,
    #[serde(default = "default_max_height")]
    pub max_height: f32,
}

impl Default for NoiseParams {
    fn default() -> Self {
        Self {
            octaves: 6,
            frequency: 0.01,
            amplitude: 100.0,
            lacunarity: 2.0,
            persistence: 0.5,
            min_height: ALTITUDE_MIN_DEFAULT,
            max_height: ALTITUDE_MAX_DEFAULT,
        }
    }
}

/// Mezcla determinista `u64` → semilla del crate `noise` (`u32`).
///
/// **Nota:** `Perlin` del crate solo admite `u32`; dos valores `u64` distintos pueden
/// colisionar (misma salida). Para reproducción bit-a-bit basta con fijar el `u64` completo
/// en persistencia; la entropía efectiva del ruido es la de 32 bits.
#[inline]
fn perlin_seed(seed: u64) -> u32 {
    let lo = seed as u32;
    let hi = (seed >> 32) as u32;
    lo ^ hi.rotate_left(17) ^ lo.wrapping_mul(0x85eb_ca6b)
}

/// Punto de muestreo en mundo: **centro de celda**, misma convención que `TerrainField::cell_to_world` / `EnergyFieldGrid::world_pos`.
#[inline]
fn cell_center_world(x: u32, y: u32, cell_size: f32, origin: Vec2) -> (f64, f64) {
    let wx = origin.x + (x as f32 + 0.5) * cell_size;
    let wy = origin.y + (y as f32 + 0.5) * cell_size;
    (f64::from(wx), f64::from(wy))
}

/// Genera altitudes en row-major (`y * width + x`). Secuencial, determinista.
///
/// FBM: por octava `n`, suma `amplitude * persistence^n * perlin([wx, wy] * frequency * lacunarity^n)`.
///
/// Muestreo en **centro de celda** (`origin + (i + 0.5) * cell_size`), igual que
/// [`crate::topology::TerrainField::cell_to_world`] / `EnergyFieldGrid::world_pos`.
pub fn generate_heightmap(
    width: u32,
    height: u32,
    cell_size: f32,
    origin: Vec2,
    seed: u64,
    params: &NoiseParams,
) -> Vec<f32> {
    let w = width.max(1);
    let h = height.max(1);
    let cs = if cell_size.is_finite() {
        cell_size.max(0.001)
    } else {
        1.0
    };
    let n_cells = w as usize * h as usize;
    let mut out = Vec::with_capacity(n_cells);
    let perlin = Perlin::new(perlin_seed(seed));
    // Tope defensivo: RON/hostil no debe producir Inf en frecuencia acumulada.
    let octaves = params.octaves.max(1).min(32);
    let frequency = if params.frequency.is_finite() {
        params.frequency
    } else {
        0.01
    };
    let amplitude = if params.amplitude.is_finite() {
        params.amplitude
    } else {
        0.0
    };
    let lacunarity = if params.lacunarity.is_finite() && params.lacunarity > 0.0 {
        params.lacunarity
    } else {
        2.0
    };
    let persistence = if params.persistence.is_finite() && params.persistence >= 0.0 {
        params.persistence
    } else {
        0.5
    };

    for y in 0..h {
        for x in 0..w {
            let (wx, wy) = cell_center_world(x, y, cs, origin);
            let mut sum = 0.0_f64;
            let mut f = frequency;
            let mut amp_oct = 1.0_f64;
            for _ in 0..octaves {
                if !f.is_finite() {
                    break;
                }
                let nx = wx * f;
                let ny = wy * f;
                if !nx.is_finite() || !ny.is_finite() {
                    break;
                }
                let sample = perlin.get([nx, ny]);
                sum += amplitude * amp_oct * sample;
                f *= lacunarity;
                amp_oct *= persistence;
            }
            let out_v = sum as f32;
            out.push(if out_v.is_finite() { out_v } else { 0.0 });
        }
    }
    out
}

/// Reescala valores al intervalo `[min_height, max_height]`.
///
/// Si `min_height` / `max_height` no son finitos o `min_height >= max_height`, **no modifica** el slice.
/// Si no hay muestras finitas, o el rango crudo es despreciable, rellena con el punto medio del intervalo válido.
pub fn normalize_heightmap(altitude: &mut [f32], min_height: f32, max_height: f32) {
    if altitude.is_empty() {
        return;
    }
    if !min_height.is_finite() || !max_height.is_finite() {
        return;
    }
    if !matches!(min_height.partial_cmp(&max_height), Some(Ordering::Less)) {
        return;
    }
    let mid = min_height + (max_height - min_height) * 0.5;
    let mut raw_min = f32::INFINITY;
    let mut raw_max = f32::NEG_INFINITY;
    for &v in altitude.iter() {
        if v.is_finite() {
            raw_min = raw_min.min(v);
            raw_max = raw_max.max(v);
        }
    }
    if !raw_min.is_finite() {
        altitude.fill(mid);
        return;
    }
    let span = raw_max - raw_min;
    let mag = raw_min.abs().max(raw_max.abs()).max(1.0);
    let eps = (1e-6_f32).max(f32::EPSILON * mag);
    if !span.is_finite() || span <= eps {
        altitude.fill(mid);
        return;
    }
    let range = max_height - min_height;
    let scale = range / span;
    if !scale.is_finite() {
        altitude.fill(mid);
        return;
    }
    for v in altitude.iter_mut() {
        *v = if v.is_finite() {
            let mapped = min_height + (*v - raw_min) * scale;
            if mapped.is_finite() { mapped } else { mid }
        } else {
            mid
        };
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn assert_all_finite(slice: &[f32]) {
        for &v in slice {
            assert!(v.is_finite(), "expected finite, got {v:?}");
        }
    }

    #[test]
    fn seed_42_is_deterministic() {
        let p = NoiseParams {
            octaves: 4,
            frequency: 0.02,
            amplitude: 50.0,
            lacunarity: 2.0,
            persistence: 0.45,
            ..NoiseParams::default()
        };
        let a = generate_heightmap(32, 32, 1.0, Vec2::ZERO, 42, &p);
        let b = generate_heightmap(32, 32, 1.0, Vec2::ZERO, 42, &p);
        assert_eq!(a, b);
    }

    #[test]
    fn distinct_seeds_produce_different_heightmaps() {
        let p = NoiseParams::default();
        let a = generate_heightmap(24, 24, 2.0, Vec2::new(1.0, -3.0), 42, &p);
        let b = generate_heightmap(24, 24, 2.0, Vec2::new(1.0, -3.0), 43, &p);
        assert_ne!(a, b);
    }

    /// Variación local media (|Δx|+|Δy|) suele ser menor con una sola octava que con seis.
    #[test]
    fn single_octave_smoother_than_six() {
        let base = NoiseParams {
            octaves: 1,
            frequency: 0.01,
            amplitude: 80.0,
            lacunarity: 2.0,
            persistence: 0.5,
            ..NoiseParams::default()
        };
        let mut detailed = base;
        detailed.octaves = 6;
        let w = 48u32;
        let h = 48u32;
        let smooth = generate_heightmap(w, h, 1.0, Vec2::ZERO, 99, &base);
        let fine = generate_heightmap(w, h, 1.0, Vec2::ZERO, 99, &detailed);
        assert_all_finite(&smooth);
        assert_all_finite(&fine);

        fn mean_neighbor_variation(v: &[f32], width: u32, height: u32) -> f32 {
            let mut acc = 0.0_f32;
            let mut n = 0u32;
            for y in 0..height {
                for x in 0..width {
                    let i = (y * width + x) as usize;
                    let c = v[i];
                    if x + 1 < width {
                        acc += (c - v[i + 1]).abs();
                        n += 1;
                    }
                    if y + 1 < height {
                        acc += (c - v[i + width as usize]).abs();
                        n += 1;
                    }
                }
            }
            acc / n.max(1) as f32
        }

        let vs = mean_neighbor_variation(&smooth, w, h);
        let vf = mean_neighbor_variation(&fine, w, h);
        assert!(
            vf > vs,
            "expected more fine detail with 6 octaves: vs={vs} vf={vf}"
        );
    }

    #[test]
    fn normalize_maps_exactly_to_range() {
        let mut v = vec![0.0_f32, 0.5, 1.0, 0.25];
        normalize_heightmap(&mut v, -50.0, 200.0);
        assert!((v.iter().cloned().fold(f32::INFINITY, f32::min) + 50.0).abs() < 1e-4);
        assert!((v.iter().cloned().fold(f32::NEG_INFINITY, f32::max) - 200.0).abs() < 1e-3);
        assert_all_finite(&v);
    }

    #[test]
    fn normalize_after_generate_touches_endpoints() {
        let p = NoiseParams {
            octaves: 4,
            ..NoiseParams::default()
        };
        let mut v = generate_heightmap(40, 40, 1.0, Vec2::ZERO, 11, &p);
        normalize_heightmap(&mut v, -50.0, 200.0);
        let mn = v.iter().cloned().fold(f32::INFINITY, f32::min);
        let mx = v.iter().cloned().fold(f32::NEG_INFINITY, f32::max);
        assert!((mn + 50.0).abs() < 1e-3, "min got {mn}");
        assert!((mx - 200.0).abs() < 1e-3, "max got {mx}");
        assert_all_finite(&v);
    }

    #[test]
    fn normalize_invalid_range_is_noop() {
        let mut v = vec![1.0_f32, 2.0, 3.0];
        let snapshot = v.clone();
        normalize_heightmap(&mut v, 10.0, 5.0);
        assert_eq!(v, snapshot);
    }

    #[test]
    fn dimensions_zero_normalize_to_one_cell() {
        let p = NoiseParams::default();
        let v = generate_heightmap(0, 0, 1.0, Vec2::ZERO, 1, &p);
        assert_eq!(v.len(), 1);
        assert_all_finite(&v);
    }

    #[test]
    fn octaves_zero_equivalent_to_one() {
        let mut p = NoiseParams::default();
        p.octaves = 0;
        let a = generate_heightmap(12, 12, 1.0, Vec2::ZERO, 8, &p);
        let mut p1 = p;
        p1.octaves = 1;
        let b = generate_heightmap(12, 12, 1.0, Vec2::ZERO, 8, &p1);
        assert_eq!(a, b);
    }

    #[test]
    fn noise_params_json_without_min_max_uses_blueprint_defaults() {
        use crate::topology::constants::{ALTITUDE_MAX_DEFAULT, ALTITUDE_MIN_DEFAULT};
        let json =
            r#"{"octaves":3,"frequency":0.02,"amplitude":10.0,"lacunarity":2.0,"persistence":0.5}"#;
        let p: NoiseParams = serde_json::from_str(json).expect("deserialize");
        assert_eq!(p.min_height, ALTITUDE_MIN_DEFAULT);
        assert_eq!(p.max_height, ALTITUDE_MAX_DEFAULT);
    }

    #[test]
    fn noise_params_serde_roundtrip_json() {
        let p = NoiseParams {
            octaves: 5,
            frequency: 0.03,
            amplitude: 42.0,
            lacunarity: 2.2,
            persistence: 0.4,
            min_height: -10.0,
            max_height: 90.0,
        };
        let json = serde_json::to_string(&p).expect("serialize");
        let back: NoiseParams = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(p, back);
    }

    #[test]
    fn generated_values_are_finite() {
        let p = NoiseParams::default();
        let v = generate_heightmap(100, 100, 1.5, Vec2::new(-10.0, 5.0), 7, &p);
        assert_eq!(v.len(), 10_000);
        assert_all_finite(&v);
    }

    /// No blocker de CI: marcar ignore si el host es lento; documenta presupuesto T2.
    #[test]
    #[ignore = "local benchmark: 100×100 should be <100ms on dev machine"]
    fn generate_100x100_under_100ms() {
        let p = NoiseParams::default();
        let start = std::time::Instant::now();
        let _ = generate_heightmap(100, 100, 1.0, Vec2::ZERO, 1, &p);
        assert!(start.elapsed().as_millis() < 100);
    }

    #[test]
    fn default_params_match_blueprint_tuning_order_of_magnitude() {
        let d = NoiseParams::default();
        assert_eq!(d.octaves, 6);
        assert!((d.frequency - 0.01).abs() < 1e-9);
        assert!((d.amplitude - 100.0).abs() < 1e-9);
    }

    #[test]
    fn world_coords_match_terrain_field_cell_center() {
        use crate::topology::TerrainField;
        let tf = TerrainField::new(5, 5, 2.0, Vec2::new(1.0, -2.0), 0);
        let p = NoiseParams {
            octaves: 1,
            frequency: 0.1,
            amplitude: 1.0,
            lacunarity: 2.0,
            persistence: 0.5,
            ..NoiseParams::default()
        };
        let seed = 123u64;
        let v = generate_heightmap(5, 5, 2.0, Vec2::new(1.0, -2.0), seed, &p);
        let perlin = Perlin::new(perlin_seed(seed));
        let c = tf.cell_to_world(2, 3);
        let wx = f64::from(c.x);
        let wy = f64::from(c.y);
        let expected = (p.amplitude * perlin.get([wx * p.frequency, wy * p.frequency])) as f32;
        let i = tf.cell_index(2, 3);
        assert!(
            (v[i] - expected).abs() < 1e-4,
            "v={} exp={}",
            v[i],
            expected
        );
    }
}
