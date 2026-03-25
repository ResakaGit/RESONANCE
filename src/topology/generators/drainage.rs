//! D8 flow direction, acumulación y relleno de depresiones (priority-flood) — Sprint T4, docs/design/TOPOLOGY.md.

use std::collections::BinaryHeap;

use bevy::math::Vec2;

/// Vecinos D8 en orden fijo (determinista ante empates de altitud).
const D8_DX: [i32; 8] = [-1, 0, 1, -1, 1, -1, 0, 1];
const D8_DY: [i32; 8] = [-1, -1, -1, 0, 0, 1, 1, 1];

/// Distancia celda-centro a vecino D8 (1 o √2) — desempata cardinales vs diagonales en rampas.
const D8_DIST: [f32; 8] = [
    std::f32::consts::SQRT_2,
    1.0,
    std::f32::consts::SQRT_2,
    1.0,
    1.0,
    std::f32::consts::SQRT_2,
    1.0,
    std::f32::consts::SQRT_2,
];

#[inline]
fn cell_count(width: u32, height: u32) -> usize {
    width as usize * height as usize
}

#[inline]
fn index(x: u32, y: u32, width: u32) -> usize {
    y as usize * width as usize + x as usize
}

#[inline]
fn altitude_at(altitude: &[f32], x: u32, y: u32, width: u32) -> f32 {
    altitude[index(x, y, width)]
}

/// Paso D8 de **máxima pendiente** en espacio de rejilla: `(z0 - z_v) / d` con `d` ∈ {1, √2} (unidades celda).
/// Empate = primer índice `k` en orden D8. Usado también por erosión hidráulica para no desviar el destino del flujo.
pub(super) fn steepest_downslope_step(
    altitude: &[f32],
    width: u32,
    height: u32,
    x: u32,
    y: u32,
) -> Option<(i32, i32, f32)> {
    let z0 = altitude_at(altitude, x, y, width);
    if !z0.is_finite() {
        return None;
    }
    let w = width as i32;
    let h = height as i32;
    let mut best_slope = 0.0_f32;
    let mut best_k: Option<usize> = None;
    for k in 0..8 {
        let nx = x as i32 + D8_DX[k];
        let ny = y as i32 + D8_DY[k];
        if nx < 0 || ny < 0 || nx >= w || ny >= h {
            continue;
        }
        let nz = altitude_at(altitude, nx as u32, ny as u32, width);
        if !nz.is_finite() {
            continue;
        }
        let drop = z0 - nz;
        if drop <= 0.0 {
            continue;
        }
        let slope = drop / D8_DIST[k];
        if slope > best_slope {
            best_slope = slope;
            best_k = Some(k);
        }
    }
    let k = best_k?;
    Some((D8_DX[k], D8_DY[k], D8_DIST[k]))
}

/// Dirección de flujo D8: vector unitario hacia el vecino de máxima pendiente de descenso (véase [`steepest_downslope_step`]).
/// Si ningún vecino es más bajo, `ZERO`.
pub fn compute_flow_direction(altitude: &[f32], width: u32, height: u32) -> Vec<Vec2> {
    let n = cell_count(width, height);
    debug_assert_eq!(altitude.len(), n);
    let mut out = vec![Vec2::ZERO; n];

    for y in 0..height {
        for x in 0..width {
            let Some((dx, dy, _)) = steepest_downslope_step(altitude, width, height, x, y) else {
                continue;
            };
            let vx = dx as f32;
            let vy = dy as f32;
            let len_sq = vx * vx + vy * vy;
            if len_sq <= 0.0 {
                continue;
            }
            let inv_len = len_sq.sqrt().recip();
            out[index(x, y, width)] = Vec2::new(vx * inv_len, vy * inv_len);
        }
    }
    out
}

/// Interpreta un vector de flujo D8 (salida de [`compute_flow_direction`]) como offset de celda destino.
fn downstream_cell(x: u32, y: u32, width: u32, height: u32, flow: Vec2) -> Option<(u32, u32)> {
    if !flow.is_finite() || flow.length_squared() < 1e-20 {
        return None;
    }
    let s = std::f32::consts::FRAC_1_SQRT_2;
    // Mismas direcciones que `compute_flow_direction` (cardinal + diagonal × s).
    let candidates: [(i32, i32, Vec2); 8] = [
        (-1, -1, Vec2::new(-s, -s)),
        (-1, 0, Vec2::new(-1.0, 0.0)),
        (-1, 1, Vec2::new(-s, s)),
        (0, -1, Vec2::new(0.0, -1.0)),
        (0, 1, Vec2::new(0.0, 1.0)),
        (1, -1, Vec2::new(s, -s)),
        (1, 0, Vec2::new(1.0, 0.0)),
        (1, 1, Vec2::new(s, s)),
    ];
    let mut best: Option<(i32, i32)> = None;
    let mut best_d = f32::INFINITY;
    for (dx, dy, dir) in candidates {
        let d = (flow - dir).length_squared();
        if d < best_d {
            best_d = d;
            best = Some((dx, dy));
        }
    }
    let (dx, dy) = best?;
    if best_d > 0.05 {
        return None;
    }
    let nx = x as i32 + dx;
    let ny = y as i32 + dy;
    if nx < 0 || ny < 0 || nx >= width as i32 || ny >= height as i32 {
        return None;
    }
    Some((nx as u32, ny as u32))
}

/// Acumulación por orden topológico (de mayor a menor altitud): cada celda aporta 1; el caudal se suma aguas abajo.
///
/// **Nota:** En pendientes con drenaje, la suma de todas las acumulaciones es **> n** (típico en GIS). La suma es exactamente `n` solo si no hay propagación (p. ej. meseta sin vecino más bajo).
pub fn compute_flow_accumulation(
    altitude: &[f32],
    flow: &[Vec2],
    width: u32,
    height: u32,
) -> Vec<f32> {
    let n = cell_count(width, height);
    debug_assert_eq!(altitude.len(), n);
    debug_assert_eq!(flow.len(), n);

    let mut acc = vec![1.0_f32; n];
    let mut order: Vec<u32> = (0..n as u32).collect();
    order.sort_by(|&ia, &ib| altitude[ib as usize].total_cmp(&altitude[ia as usize]));

    for idx in order {
        let x = idx % width;
        let y = idx / width;
        let f = flow[idx as usize];
        if let Some((nx, ny)) = downstream_cell(x, y, width, height, f) {
            let j = index(nx, ny, width);
            acc[j] += acc[idx as usize];
        }
    }
    acc
}

/// `f32` → bits con orden total creciente (para min-heap vía `BinaryHeap<Reverse<...>>`).
#[inline]
fn f32_total_order_bits(f: f32) -> u32 {
    let b = f.to_bits();
    if b & 0x8000_0000 != 0 {
        !b
    } else {
        b ^ 0x8000_0000
    }
}

/// Rellena depresiones cerradas con **priority-flood** (Barnes et al.) para que el drenaje pueda alcanzar el borde.
pub fn fill_pits(altitude: &mut [f32], width: u32, height: u32) {
    let n = cell_count(width, height);
    if altitude.len() != n || width == 0 || height == 0 {
        return;
    }

    let mut visited = vec![false; n];
    type HeapKey = (u32, u32, u32);
    let mut heap: BinaryHeap<std::cmp::Reverse<HeapKey>> = BinaryHeap::new();

    let push_edge = |heap: &mut BinaryHeap<std::cmp::Reverse<HeapKey>>,
                     visited: &mut [bool],
                     alt: &mut [f32],
                     x: u32,
                     y: u32,
                     w: u32| {
        let i = index(x, y, w);
        if visited[i] {
            return;
        }
        visited[i] = true;
        let z = alt[i];
        let z = if z.is_finite() { z } else { 0.0 };
        alt[i] = z;
        let bits = f32_total_order_bits(z);
        heap.push(std::cmp::Reverse((bits, x, y)));
    };

    for x in 0..width {
        push_edge(&mut heap, &mut visited, altitude, x, 0, width);
        push_edge(
            &mut heap,
            &mut visited,
            altitude,
            x,
            height.saturating_sub(1),
            width,
        );
    }
    for y in 1..height.saturating_sub(1) {
        push_edge(&mut heap, &mut visited, altitude, 0, y, width);
        push_edge(
            &mut heap,
            &mut visited,
            altitude,
            width.saturating_sub(1),
            y,
            width,
        );
    }

    while let Some(std::cmp::Reverse((_bits, cx, cy))) = heap.pop() {
        let ce = altitude[index(cx, cy, width)];
        if !ce.is_finite() {
            continue;
        }
        for k in 0..8 {
            let nx = cx as i32 + D8_DX[k];
            let ny = cy as i32 + D8_DY[k];
            if nx < 0 || ny < 0 || nx >= width as i32 || ny >= height as i32 {
                continue;
            }
            let nx = nx as u32;
            let ny = ny as u32;
            let ni = index(nx, ny, width);
            if visited[ni] {
                continue;
            }
            visited[ni] = true;
            let mut z = altitude[ni];
            if !z.is_finite() {
                z = 0.0;
            }
            if z < ce {
                z = ce;
            }
            altitude[ni] = z;
            let bits = f32_total_order_bits(z);
            heap.push(std::cmp::Reverse((bits, nx, ny)));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn flat_alt(w: u32, h: u32, z: f32) -> Vec<f32> {
        vec![z; (w * h) as usize]
    }

    #[test]
    fn flat_terrain_flow_zero_everywhere() {
        let w = 7u32;
        let h = 5u32;
        let alt = flat_alt(w, h, 10.0);
        let flow = compute_flow_direction(&alt, w, h);
        assert!(flow.iter().all(|&v| v == Vec2::ZERO));
    }

    /// Rampa: altitud decrece en +x → flujo hacia -X (vecino a la izquierda más bajo).
    #[test]
    fn ramp_decreasing_in_x_flows_to_negative_x() {
        let w = 8u32;
        let h = 4u32;
        let mut alt = Vec::with_capacity((w * h) as usize);
        for _y in 0..h {
            for x in 0..w {
                alt.push(x as f32);
            }
        }
        let flow = compute_flow_direction(&alt, w, h);
        let expected = Vec2::new(-1.0, 0.0);
        for y in 1..h - 1 {
            for x in 1..w - 1 {
                let f = flow[index(x, y, w)];
                assert!(
                    (f - expected).length_squared() < 1e-6,
                    "cell ({x},{y}) flow {f:?} expected {expected:?}"
                );
            }
        }
    }

    /// Pico central: flujo diverge desde el centro hacia afuera.
    #[test]
    fn central_peak_flow_diverges_outward() {
        let w = 7u32;
        let h = 7u32;
        let cx = 3u32;
        let cy = 3u32;
        let mut alt = vec![0.0_f32; (w * h) as usize];
        for y in 0..h {
            for x in 0..w {
                let dx = x as f32 - cx as f32;
                let dy = y as f32 - cy as f32;
                alt[index(x, y, w)] = 50.0 - (dx * dx + dy * dy).sqrt();
            }
        }
        let flow = compute_flow_direction(&alt, w, h);
        // A la izquierda del pico el descenso sigue bajando hacia afuera (−X).
        let fl = flow[index(cx - 1, cy, w)];
        assert!(fl.x < -0.3, "expected westward flow, got {fl:?}");
        // A la derecha, hacia +X.
        let fr = flow[index(cx + 1, cy, w)];
        assert!(fr.x > 0.3, "expected eastward flow, got {fr:?}");
    }

    /// Valle en V: acumulación máxima en la celda más baja del fondo.
    #[test]
    fn central_valley_max_accumulation_at_bottom() {
        let w = 9u32;
        let h = 9u32;
        let cx = 4u32;
        let cy = 4u32;
        let mut alt = vec![0.0_f32; (w * h) as usize];
        for y in 0..h {
            for x in 0..w {
                let dx = x as f32 - cx as f32;
                let dy = y as f32 - cy as f32;
                alt[index(x, y, w)] = (dx * dx + dy * dy).sqrt();
            }
        }
        let flow = compute_flow_direction(&alt, w, h);
        let acc = compute_flow_accumulation(&alt, &flow, w, h);
        let max_i = acc
            .iter()
            .enumerate()
            .max_by(|(_, a), (_, b)| a.total_cmp(b))
            .expect("acc non-empty")
            .0;
        let mx = max_i % w as usize;
        let my = max_i / w as usize;
        assert!(
            (mx as i32 - cx as i32).abs() <= 1 && (my as i32 - cy as i32).abs() <= 1,
            "max acc at ({mx},{my}), expected near center ({cx},{cy})"
        );
    }

    #[test]
    fn ridge_cells_accumulation_one() {
        let w = 7u32;
        let h = 7u32;
        let cx = 3u32;
        let cy = 3u32;
        let mut alt = vec![0.0_f32; (w * h) as usize];
        for y in 0..h {
            for x in 0..w {
                let dx = x as f32 - cx as f32;
                let dy = y as f32 - cy as f32;
                alt[index(x, y, w)] = 50.0 - (dx * dx + dy * dy).sqrt();
            }
        }
        let flow = compute_flow_direction(&alt, w, h);
        let acc = compute_flow_accumulation(&alt, &flow, w, h);
        assert!((acc[index(cx, cy, w)] - 1.0).abs() < 1e-5);
    }

    /// Con `compute_flow_direction` en rampa, la suma de acumulaciones supera `n` (típico GIS).
    #[test]
    fn accumulation_sum_exceeds_cell_count_on_monotone_slope() {
        let w = 6u32;
        let h = 4u32;
        let mut alt = Vec::new();
        for _y in 0..h {
            for x in 0..w {
                alt.push(x as f32);
            }
        }
        let flow = compute_flow_direction(&alt, w, h);
        let acc = compute_flow_accumulation(&alt, &flow, w, h);
        let sum: f32 = acc.iter().sum();
        assert!(
            sum > (w * h) as f32 + 0.5,
            "expected sum > n with propagation, got sum={sum} n={}",
            w * h
        );
    }

    /// Con flujo nulo en todas las celdas, cada una conserva su unidad → suma = n.
    #[test]
    fn accumulation_sum_equals_cells_when_no_downstream_propagation() {
        let w = 6u32;
        let h = 5u32;
        let alt = flat_alt(w, h, 3.0);
        let flow = vec![Vec2::ZERO; (w * h) as usize];
        let acc = compute_flow_accumulation(&alt, &flow, w, h);
        let sum: f32 = acc.iter().sum();
        assert!((sum - (w * h) as f32).abs() < 1e-4);
    }

    #[test]
    fn accumulation_each_cell_at_least_one() {
        let w = 5u32;
        let h = 5u32;
        let mut alt = flat_alt(w, h, 0.0);
        alt[index(2, 2, w)] = 10.0;
        let flow = compute_flow_direction(&alt, w, h);
        let acc = compute_flow_accumulation(&alt, &flow, w, h);
        assert!(acc.iter().all(|&a| a >= 1.0));
    }

    #[test]
    fn fill_pits_removes_interior_local_minima() {
        let w = 5u32;
        let h = 5u32;
        let mut alt = vec![10.0_f32; (w * h) as usize];
        alt[index(2, 2, w)] = 1.0;
        fill_pits(&mut alt, w, h);
        let zc = altitude_at(&alt, 2, 2, w);
        assert!(
            zc >= 10.0 - 1e-5,
            "pit should be raised to spill level, got {zc}"
        );
        for y in 1..h - 1 {
            for x in 1..w - 1 {
                let z0 = altitude_at(&alt, x, y, w);
                let mut min_n = f32::INFINITY;
                for k in 0..8 {
                    let nx = x as i32 + D8_DX[k];
                    let ny = y as i32 + D8_DY[k];
                    if nx < 0 || ny < 0 || nx >= w as i32 || ny >= h as i32 {
                        continue;
                    }
                    let nz = altitude_at(&alt, nx as u32, ny as u32, w);
                    min_n = min_n.min(nz);
                }
                assert!(
                    z0 >= min_n - 1e-4,
                    "local depression at ({x},{y}) z={z0} min_neighbor={min_n}"
                );
            }
        }
    }
}
