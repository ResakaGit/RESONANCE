//! Derivación de pendiente y aspecto desde altitud (Sprint T3, docs/design/TOPOLOGY.md).
//!
//! Índice row-major `y * width + x`, mismo orden que `generate_heightmap` y `TerrainField::cell_index`.
//!
//! **Gradiente:** celdas interiores (`1..width-1` × `1..height-1`) usan el kernel de Horn (vecindad
//! de Moore ponderada — 8 vecinos). En bordes del grid, diferencias forward/backward en cada eje
//! (sin wrap), coherente con el sprint.

#[inline]
fn expected_len(width: u32, height: u32) -> usize {
    width.max(1) as usize * height.max(1) as usize
}

#[inline]
fn cell_size_safe(cell_size: f32) -> f32 {
    if cell_size.is_finite() {
        cell_size.max(0.001)
    } else {
        1.0
    }
}

#[inline]
fn altitude_at(altitude: &[f32], width: u32, x: u32, y: u32) -> f32 {
    let i = y as usize * width as usize + x as usize;
    let v = altitude[i];
    if v.is_finite() { v } else { 0.0 }
}

/// ∂z/∂x con stencil cardinal (bordes o grids pequeños).
fn dz_dx_cardinal(
    altitude: &[f32],
    width: u32,
    _height: u32,
    x: u32,
    y: u32,
    cell_size: f32,
) -> f64 {
    let w = width.max(1);
    let cs = f64::from(cell_size_safe(cell_size));
    if w <= 1 {
        return 0.0;
    }
    let zc = f64::from(altitude_at(altitude, w, x, y));
    if x == 0 {
        let z1 = f64::from(altitude_at(altitude, w, 1, y));
        (z1 - zc) / cs
    } else if x == w - 1 {
        let zm = f64::from(altitude_at(altitude, w, x - 1, y));
        (zc - zm) / cs
    } else {
        let zp = f64::from(altitude_at(altitude, w, x + 1, y));
        let zm = f64::from(altitude_at(altitude, w, x - 1, y));
        (zp - zm) / (2.0 * cs)
    }
}

/// ∂z/∂y con stencil cardinal (bordes o grids pequeños).
fn dz_dy_cardinal(
    altitude: &[f32],
    width: u32,
    height: u32,
    x: u32,
    y: u32,
    cell_size: f32,
) -> f64 {
    let w = width.max(1);
    let h = height.max(1);
    let cs = f64::from(cell_size_safe(cell_size));
    if h <= 1 {
        return 0.0;
    }
    let zc = f64::from(altitude_at(altitude, w, x, y));
    if y == 0 {
        let z1 = f64::from(altitude_at(altitude, w, x, 1));
        (z1 - zc) / cs
    } else if y == h - 1 {
        let zm = f64::from(altitude_at(altitude, w, x, y - 1));
        (zc - zm) / cs
    } else {
        let zp = f64::from(altitude_at(altitude, w, x, y + 1));
        let zm = f64::from(altitude_at(altitude, w, x, y - 1));
        (zp - zm) / (2.0 * cs)
    }
}

/// Horn (1981): gradiente con vecindad de Moore en celdas interiores; mismo límite que diferencias
/// centrales en rampas lineales.
fn horn_gradient(altitude: &[f32], w: u32, x: u32, y: u32, cs: f64) -> (f64, f64) {
    let z = |dx: i32, dy: i32| -> f64 {
        let xi = (x as i32 + dx) as u32;
        let yi = (y as i32 + dy) as u32;
        f64::from(altitude_at(altitude, w, xi, yi))
    };
    let d = 8.0 * cs;
    let east = z(1, -1) + 2.0 * z(1, 0) + z(1, 1);
    let west = z(-1, -1) + 2.0 * z(-1, 0) + z(-1, 1);
    let dx = (east - west) / d;
    let north = z(-1, 1) + 2.0 * z(0, 1) + z(1, 1);
    let south = z(-1, -1) + 2.0 * z(0, -1) + z(1, -1);
    let dy = (north - south) / d;
    (dx, dy)
}

#[inline]
fn cell_gradient(altitude: &[f32], w: u32, h: u32, x: u32, y: u32, cell_size: f32) -> (f64, f64) {
    let interior = w >= 3 && h >= 3 && x >= 1 && x < w - 1 && y >= 1 && y < h - 1;
    if interior {
        let cs = f64::from(cell_size_safe(cell_size));
        horn_gradient(altitude, w, x, y, cs)
    } else {
        (
            dz_dx_cardinal(altitude, w, h, x, y, cell_size),
            dz_dy_cardinal(altitude, w, h, x, y, cell_size),
        )
    }
}

/// Pendiente en grados (0–90) por celda.
pub fn derive_slope(altitude: &[f32], width: u32, height: u32, cell_size: f32) -> Vec<f32> {
    derive_slope_aspect(altitude, width, height, cell_size).0
}

/// Aspecto en grados [0, 360): 0 = norte, 90 = este, convención GIS (caída máxima).
pub fn derive_aspect(altitude: &[f32], width: u32, height: u32, cell_size: f32) -> Vec<f32> {
    derive_slope_aspect(altitude, width, height, cell_size).1
}

/// Calcula pendiente y aspecto en un solo pase (mismos gradientes).
pub fn derive_slope_aspect(
    altitude: &[f32],
    width: u32,
    height: u32,
    cell_size: f32,
) -> (Vec<f32>, Vec<f32>) {
    let w = width.max(1);
    let h = height.max(1);
    let n = expected_len(w, h);
    assert_eq!(
        altitude.len(),
        n,
        "altitude.len() must equal width*height (row-major)"
    );

    let mut slope = Vec::with_capacity(n);
    let mut aspect = Vec::with_capacity(n);

    for y in 0..h {
        for x in 0..w {
            let (dx, dy) = cell_gradient(altitude, w, h, x, y, cell_size);
            let mag = (dx * dx + dy * dy).sqrt();

            let slope_deg = if mag.is_finite() {
                let rad = mag.atan();
                let d = rad.to_degrees();
                if d.is_finite() {
                    d.clamp(0.0, 90.0) as f32
                } else {
                    0.0
                }
            } else {
                0.0
            };

            // `atan2(-dz_dy, -dz_dx)` = ángulo desde +x (este) CCW; brújula: (90° - θ) mod 360.
            let aspect_deg = if mag < 1e-14 {
                0.0_f32
            } else {
                let theta = f64::atan2(-dy, -dx);
                let g = (90.0 - theta.to_degrees()).rem_euclid(360.0);
                let v = g as f32;
                if v >= 360.0 { 0.0 } else { v }
            };

            slope.push(slope_deg);
            aspect.push(aspect_deg);
        }
    }

    (slope, aspect)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn flat_grid(w: u32, h: u32, z: f32) -> Vec<f32> {
        vec![z; expected_len(w, h)]
    }

    #[test]
    fn flat_constant_zero_slope() {
        let w = 16u32;
        let h = 12u32;
        let alt = flat_grid(w, h, 42.0);
        let s = derive_slope(&alt, w, h, 1.0);
        for &v in &s {
            assert!(v.abs() < 1e-5, "expected ~0, got {v}");
        }
        let (s2, a) = derive_slope_aspect(&alt, w, h, 1.0);
        assert_eq!(s, s2);
        for &v in &a {
            assert!(v.abs() < 1e-5, "plano aspecto 0, got {v}");
        }
    }

    #[test]
    fn ramp_in_x_constant_slope_aspect_east_or_west() {
        let w = 20u32;
        let h = 8u32;
        let cs = 1.0_f32;
        let mut alt = Vec::with_capacity(expected_len(w, h));
        let gain = 0.5_f32;
        for _y in 0..h {
            for x in 0..w {
                alt.push(gain * x as f32);
            }
        }
        let (sl, asp) = derive_slope_aspect(&alt, w, h, cs);
        // z crece hacia +x → caída hacia oeste → 270°
        let y_mid = h / 2;
        let inner_x0 = 5usize;
        let i0 = y_mid as usize * w as usize + inner_x0;
        let inner = sl[i0];
        for x in 5..15 {
            let i = y_mid as usize * w as usize + x;
            assert!(
                (sl[i] - inner).abs() < 0.02,
                "pendiente casi constante en interior, got {} vs {}",
                sl[i],
                inner
            );
            let expected_asp = 270.0_f32;
            assert!(
                (asp[i] - expected_asp).abs() < 1.0,
                "aspecto oeste ~270, got {}",
                asp[i]
            );
        }
        // Rampa decreciente en x → ~90° (este)
        let mut alt2 = Vec::with_capacity(expected_len(w, h));
        for _y in 0..h {
            for x in 0..w {
                alt2.push(-gain * x as f32);
            }
        }
        let (_, asp2) = derive_slope_aspect(&alt2, w, h, cs);
        for x in 5..15 {
            let i = y_mid as usize * w as usize + x;
            assert!(
                (asp2[i] - 90.0).abs() < 1.0,
                "aspecto este ~90, got {}",
                asp2[i]
            );
        }
    }

    #[test]
    fn peak_neighbors_positive_slope() {
        // 5×5: los 8 vecinos de Moore del pico caen en celdas interiores → kernel Horn ve el pico.
        let w = 5u32;
        let h = 5u32;
        let mut alt = flat_grid(w, h, 0.0);
        let cx = 2u32;
        let cy = 2u32;
        alt[cy as usize * w as usize + cx as usize] = 100.0;
        let s = derive_slope(&alt, w, h, 1.0);
        for dy in -1i32..=1 {
            for dx in -1i32..=1 {
                if dx == 0 && dy == 0 {
                    continue;
                }
                let x = (cx as i32 + dx) as u32;
                let y = (cy as i32 + dy) as u32;
                let i = y as usize * w as usize + x as usize;
                assert!(
                    s[i] > 0.01,
                    "peak neighbor must have slope > 0, cell ({x},{y}) got {}",
                    s[i]
                );
            }
        }
    }

    #[test]
    fn slope_in_range_aspect_in_range() {
        let w = 25u32;
        let h = 25u32;
        let mut alt = Vec::with_capacity(expected_len(w, h));
        for y in 0..h {
            for x in 0..w {
                alt.push((x as f32 * 0.3).sin() + (y as f32 * 0.17).cos() * 2.0);
            }
        }
        let (s, a) = derive_slope_aspect(&alt, w, h, 0.75);
        for &v in &s {
            assert!(v.is_finite() && (0.0..=90.0).contains(&v), "slope {v}");
        }
        for &v in &a {
            assert!(v.is_finite() && (0.0..360.0).contains(&v), "aspect {v}");
        }
    }

    #[test]
    fn borders_no_panic_nan_tiny_grids() {
        for (w, h) in [(1u32, 1u32), (2u32, 1u32), (1u32, 5u32), (2u32, 3u32)] {
            let alt = flat_grid(w, h, 1.0);
            let (s, a) = derive_slope_aspect(&alt, w, h, 1.0);
            assert_eq!(s.len(), expected_len(w, h));
            for &v in &s {
                assert!(v.is_finite() && !v.is_nan());
            }
            for &v in &a {
                assert!(v.is_finite() && !v.is_nan());
            }
        }
        let w = 4u32;
        let h = 4u32;
        let mut alt = flat_grid(w, h, 0.0);
        alt[w as usize + 1] = 10.0;
        let (s, a) = derive_slope_aspect(&alt, w, h, 0.5);
        for &v in &s {
            assert!(v.is_finite());
        }
        for &v in &a {
            assert!(v.is_finite());
        }
    }

    #[test]
    fn combined_matches_separate_functions() {
        let w = 17u32;
        let h = 11u32;
        let mut alt = Vec::with_capacity(expected_len(w, h));
        for y in 0..h {
            for x in 0..w {
                alt.push(((x * 7 + y * 13) % 100) as f32 * 0.1);
            }
        }
        let (sc, ac) = derive_slope_aspect(&alt, w, h, 1.25);
        let ss = derive_slope(&alt, w, h, 1.25);
        let aa = derive_aspect(&alt, w, h, 1.25);
        assert_eq!(sc, ss);
        assert_eq!(ac, aa);
    }

    #[test]
    fn determinism_same_input_same_output() {
        let w = 10u32;
        let h = 10u32;
        let alt: Vec<f32> = (0..expected_len(w, h))
            .map(|i| (i as f32 * 1.618).sin())
            .collect();
        let a = derive_slope_aspect(&alt, w, h, 2.0);
        let b = derive_slope_aspect(&alt, w, h, 2.0);
        assert_eq!(a, b);
    }

    #[test]
    #[should_panic(expected = "altitude.len()")]
    fn wrong_length_panics() {
        let _ = derive_slope_aspect(&[1.0, 2.0], 3, 3, 1.0);
    }

    /// Entradas no finitas se tratan como 0 en el muestreo; la salida sigue siendo finita (contrato defensivo).
    #[test]
    fn nan_altitude_still_finite_outputs() {
        let w = 5u32;
        let h = 5u32;
        let mut alt = flat_grid(w, h, 3.0);
        alt[12] = f32::NAN;
        let (s, a) = derive_slope_aspect(&alt, w, h, 1.0);
        for &v in &s {
            assert!(v.is_finite());
        }
        for &v in &a {
            assert!(v.is_finite());
        }
    }
}
