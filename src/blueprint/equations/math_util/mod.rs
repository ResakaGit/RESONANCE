// ═══════════════════════════════════════════════
// Fenología visual (EA8) — entradas en [0, 1]; no finitos en entradas → 0 (salvo `normalize_range` en value).
// ═══════════════════════════════════════════════

/// Normaliza `value` al intervalo [0, 1] con saturación respecto a `[range_min, range_max]`.
/// Si `range_min` o `range_max` no son finitos → 0. Si `value` no es finito → 0.
/// Rango invertido se ordena. Si el ancho efectivo es ~0: `value < lo` → 0, si no → 1.
#[inline]
pub fn normalize_range(value: f32, range_min: f32, range_max: f32) -> f32 {
    if !value.is_finite() {
        return 0.0;
    }
    if !range_min.is_finite() || !range_max.is_finite() {
        return 0.0;
    }
    let (lo, hi) = if range_min <= range_max {
        (range_min, range_max)
    } else {
        (range_max, range_min)
    };
    let denom = hi - lo;
    // Solo rango puntual (o no positivo): escalón; ancho pequeño pero >0 sigue siendo lineal.
    if denom <= 0.0 || !denom.is_finite() {
        return if value < lo { 0.0 } else { 1.0 };
    }
    ((value - lo) / denom).clamp(0.0, 1.0)
}
