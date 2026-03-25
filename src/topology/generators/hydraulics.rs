//! Erosión hidráulica determinista sobre heightmap — Sprint T4, docs/design/TOPOLOGY.md.
//!
//! Modelo **simplificado** respecto al texto narrativo del sprint: por ciclo, una celda aleatoria
//! transfiere masa al vecino de máxima pendiente (mismo criterio que `drainage::steepest_downslope_step`).
//! `evaporation` reparte el volumen erosionado entre vecino y misma celda sin pérdida de masa total.

use serde::{Deserialize, Serialize};

/// Parámetros de erosión para RON / tuning (T9).
#[derive(Copy, Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct ErosionParams {
    pub cycles: u32,
    pub strength: f32,
    pub deposition_rate: f32,
    pub evaporation: f32,
}

impl Default for ErosionParams {
    fn default() -> Self {
        Self {
            cycles: 0,
            strength: 0.3,
            deposition_rate: 0.5,
            evaporation: 0.1,
        }
    }
}

/// PRNG determinista (SplitMix64) — sin dependencia `rand`.
#[derive(Clone, Copy)]
struct SplitMix64(u64);

impl SplitMix64 {
    fn new(seed: u64) -> Self {
        Self(seed)
    }

    fn next_u64(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(0x9e37_79b9_7f4a_7c15);
        let mut z = self.0;
        z = (z ^ (z >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
        z ^ (z >> 31)
    }

    fn gen_usize_below(&mut self, n: usize) -> usize {
        if n == 0 {
            return 0;
        }
        (self.next_u64() as usize) % n
    }

    /// [0, 1)
    fn gen_f32_unit(&mut self) -> f32 {
        (self.next_u64() >> 40) as f32 / (1u64 << 24) as f32
    }
}

#[inline]
fn idx(x: u32, y: u32, w: u32) -> usize {
    y as usize * w as usize + x as usize
}

#[inline]
fn alt_at(a: &[f32], x: u32, y: u32, w: u32) -> f32 {
    a[idx(x, y, w)]
}

/// N ciclos de erosión hidráulica local: sedimento baja al vecino más bajo; masa conservada en sólido.
///
/// `evaporation`: fracción del volumen erosionado que se redeposita en la misma celda (no se pierde masa del terreno).
/// `deposition_rate`: escala del paso de erosión respecto a `strength` y pendiente local.
pub fn erode_hydraulic(
    altitude: &mut [f32],
    width: u32,
    height: u32,
    cell_size: f32,
    params: &ErosionParams,
    seed: u64,
) {
    let n = width as usize * height as usize;
    if altitude.len() != n || params.cycles == 0 {
        return;
    }
    let cs = if cell_size.is_finite() {
        cell_size.max(0.001)
    } else {
        1.0
    };
    let strength = if params.strength.is_finite() {
        params.strength.max(0.0)
    } else {
        0.0
    };
    let dep = if params.deposition_rate.is_finite() {
        params.deposition_rate.max(0.0)
    } else {
        0.0
    };
    let evap = if params.evaporation.is_finite() {
        params.evaporation.clamp(0.0, 1.0)
    } else {
        0.0
    };

    let mut rng = SplitMix64::new(seed);
    for _ in 0..params.cycles {
        let i = rng.gen_usize_below(n);
        let x = (i % width as usize) as u32;
        let y = (i / width as usize) as u32;
        let z0 = alt_at(altitude, x, y, width);
        if !z0.is_finite() {
            continue;
        }
        let Some((dx, dy, d_grid)) =
            super::drainage::steepest_downslope_step(altitude, width, height, x, y)
        else {
            continue;
        };
        let nx = x as i32 + dx;
        let ny = y as i32 + dy;
        let j = idx(nx as u32, ny as u32, width);
        let z1 = altitude[j];
        let drop = (z0 - z1).max(0.0);
        if drop <= 1e-20 {
            continue;
        }
        // Misma métrica que D8: Δz / (d_grid * cell_size).
        let horizontal = d_grid * cs;
        let slope = drop / horizontal;
        let jitter = 0.5 + 0.5 * rng.gen_f32_unit();
        let mut eroded = strength * dep * slope * cs * jitter * 0.01;
        if !eroded.is_finite() {
            continue;
        }
        eroded = eroded.min(drop * 0.5);
        if eroded < 1e-12 {
            continue;
        }
        let stay = eroded * evap;
        let to_nb = eroded - stay;
        altitude[i] = z0 - eroded + stay;
        altitude[j] = z1 + to_nb;
        if !altitude[i].is_finite() {
            altitude[i] = z0;
        }
        if !altitude[j].is_finite() {
            altitude[j] = z1;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn zero_cycles_is_noop() {
        let w = 12u32;
        let h = 10u32;
        let mut a: Vec<f32> = (0..w * h).map(|i| i as f32 * 0.1).collect();
        let snapshot = a.clone();
        let p = ErosionParams {
            cycles: 0,
            strength: 1.0,
            deposition_rate: 1.0,
            evaporation: 0.0,
        };
        erode_hydraulic(&mut a, w, h, 1.0, &p, 42);
        assert_eq!(a, snapshot);
    }

    #[test]
    fn seed_42_deterministic() {
        let w = 16u32;
        let h = 16u32;
        let base: Vec<f32> = (0..(w as usize * h as usize))
            .map(|i| {
                let x = (i % w as usize) as f32;
                let y = (i / w as usize) as f32;
                20.0 + x * 0.3 + y * 0.1 + ((x * 0.7 + y * 1.1).sin())
            })
            .collect();
        let p = ErosionParams {
            cycles: 80,
            strength: 0.8,
            deposition_rate: 0.6,
            evaporation: 0.15,
        };
        let mut a = base.clone();
        let mut b = base.clone();
        erode_hydraulic(&mut a, w, h, 2.0, &p, 42);
        erode_hydraulic(&mut b, w, h, 2.0, &p, 42);
        assert_eq!(a, b);
    }

    #[test]
    fn after_erosion_all_finite() {
        let w = 20u32;
        let h = 20u32;
        let mut a: Vec<f32> = (0..w * h).map(|i| (i % 7) as f32 * 2.5).collect();
        let p = ErosionParams {
            cycles: 200,
            strength: 0.5,
            deposition_rate: 0.9,
            evaporation: 0.2,
        };
        erode_hydraulic(&mut a, w, h, 1.5, &p, 999);
        assert!(a.iter().all(|&v| v.is_finite()));
    }

    #[test]
    fn valleys_deepen_or_max_drops() {
        let w = 24u32;
        let h = 24u32;
        let mut a: Vec<f32> = (0..(w as usize * h as usize))
            .map(|i| {
                let x = (i % w as usize) as f32 * 0.2;
                let y = (i / w as usize) as f32 * 0.2;
                100.0 - x * x - y * y + (x * 3.1).sin() * 2.0
            })
            .collect();
        let min0 = a.iter().cloned().fold(f32::INFINITY, f32::min);
        let max0 = a.iter().cloned().fold(f32::NEG_INFINITY, f32::max);
        let p = ErosionParams {
            cycles: 400,
            strength: 1.2,
            deposition_rate: 1.0,
            evaporation: 0.05,
        };
        erode_hydraulic(&mut a, w, h, 2.0, &p, 12345);
        let min1 = a.iter().cloned().fold(f32::INFINITY, f32::min);
        let max1 = a.iter().cloned().fold(f32::NEG_INFINITY, f32::max);
        assert!(
            min1 < min0 || max1 < max0,
            "expected more carved relief: min {min0}->{min1} max {max0}->{max1}"
        );
    }

    #[test]
    fn erosion_conserves_sum_altitude() {
        let w = 18u32;
        let h = 14u32;
        let mut a: Vec<f32> = (0..(w as usize * h as usize))
            .map(|i| 30.0 + (i % 13) as f32 * 0.07 + (i / 13) as f32 * 0.03)
            .collect();
        let sum0: f32 = a.iter().sum();
        let p = ErosionParams {
            cycles: 300,
            strength: 0.6,
            deposition_rate: 0.85,
            evaporation: 0.12,
        };
        erode_hydraulic(&mut a, w, h, 2.0, &p, 777);
        let sum1: f32 = a.iter().sum();
        assert!(
            (sum0 - sum1).abs() < 1e-3,
            "masa total altitud: {sum0} vs {sum1}"
        );
    }

    #[test]
    fn erosion_params_serde_roundtrip() {
        let p = ErosionParams {
            cycles: 50,
            strength: 0.31,
            deposition_rate: 0.4,
            evaporation: 0.07,
        };
        let json = serde_json::to_string(&p).unwrap();
        let back: ErosionParams = serde_json::from_str(&json).unwrap();
        assert_eq!(p, back);
    }
}
