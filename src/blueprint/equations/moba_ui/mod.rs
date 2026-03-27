use crate::math_types::{Vec2, Vec3};

// ═══════════════════════════════════════════════
// Cámara MOBA (runtime): geometría pura
// ═══════════════════════════════════════════════

/// Ángulo entre el rayo cámara→foco y el plano horizontal (0° = rayo horizontal, 90° = nadir).
///
/// Con foco en el suelo y cámara al sur (+Z) y elevada: `offset = (0, h, d)` con `h/d = tan(pitch)`.
#[inline]
pub fn moba_camera_offset_from_pitch(pitch_deg_from_horizontal: f32, ground_distance: f32) -> Vec3 {
    // Evita tan(±90°) no acotado; MOBA típico 35–70°.
    let pitch_deg = pitch_deg_from_horizontal.clamp(1.0, 89.0);
    let p = pitch_deg.to_radians();
    let d = ground_distance.max(0.0);
    let h = d * p.tan();
    Vec3::new(0.0, h, d)
}

/// Clamp del foco en plano XZ (componente Y intacta).
#[inline]
pub fn moba_clamp_focus_xz(focus: Vec3, min_xz: Vec2, max_xz: Vec2) -> Vec3 {
    Vec3::new(
        focus.x.clamp(min_xz.x, max_xz.x),
        focus.y,
        focus.z.clamp(min_xz.y, max_xz.y),
    )
}

/// Zoom horizontal: delta en unidades de distancia por tick de rueda (líneas).
#[inline]
pub fn moba_zoom_horizontal_delta(
    current: f32,
    scroll_lines: f32,
    speed: f32,
    min: f32,
    max: f32,
) -> f32 {
    let next = current + scroll_lines * speed;
    next.clamp(min, max)
}

// ═══════════════════════════════════════════════
// Minimap (UI): world XZ ↔ UV + percepción por frecuencia (L2)
// ═══════════════════════════════════════════════

/// `world_xz.x` → mundo X; `world_xz.y` → mundo Z (plano del sim).
///
/// UV en \([0,1]^2\): u crece hacia +X; v=0 arriba del panel = `max_xz.y` (norte visual).
#[inline]
pub fn minimap_world_xz_to_uv(world_xz: Vec2, min_xz: Vec2, max_xz: Vec2) -> Vec2 {
    let dx = max_xz.x - min_xz.x;
    let dz = max_xz.y - min_xz.y;
    let u = if dx.abs() > f32::EPSILON {
        (world_xz.x - min_xz.x) / dx
    } else {
        0.5
    };
    let v = if dz.abs() > f32::EPSILON {
        (max_xz.y - world_xz.y) / dz
    } else {
        0.5
    };
    Vec2::new(u.clamp(0.0, 1.0), v.clamp(0.0, 1.0))
}

/// Inversa de [`minimap_world_xz_to_uv`] para picks en el minimap.
#[inline]
pub fn minimap_uv_to_world_xz(uv: Vec2, min_xz: Vec2, max_xz: Vec2) -> Vec2 {
    let u = uv.x.clamp(0.0, 1.0);
    let v = uv.y.clamp(0.0, 1.0);
    let wx = min_xz.x + u * (max_xz.x - min_xz.x);
    let wz = max_xz.y - v * (max_xz.y - min_xz.y);
    Vec2::new(wx, wz)
}

/// Alpha de icono por “visibilidad” energética: baja frecuencia (Umbra) más tenue, alta (Lux) más opaca.
///
/// Escala logarítmica entre ~20 Hz y ~1000 Hz → alpha \([0.25, 1.0]\).
#[inline]
pub fn minimap_perception_alpha(frequency_hz: f32) -> f32 {
    let f = frequency_hz.max(1.0);
    let lo = 20.0_f32;
    let hi = 1000.0_f32;
    let t = ((f.ln() - lo.ln()) / (hi.ln() - lo.ln())).clamp(0.0, 1.0);
    0.25 + 0.75 * t
}

/// Visibilidad natural de un elemento por frecuencia (L2): escala lineal a 1 kHz, clamp \([0.05, 1]\).
#[inline]
pub fn frequency_visibility(frequency_hz: f32) -> f32 {
    if !frequency_hz.is_finite() {
        return 0.05;
    }
    (frequency_hz / 1000.0).clamp(0.05, 1.0)
}

/// Señal que una fuente emite hacia un perceptor: `qe × visibility(freq) / dist²`.
#[inline]
pub fn perception_signal(source_qe: f32, source_freq: f32, dist_sq: f32) -> f32 {
    let qe = if source_qe.is_finite() {
        source_qe.max(0.0)
    } else {
        0.0
    };
    let d2 = if dist_sq.is_finite() {
        dist_sq.max(1.0)
    } else {
        1.0
    };
    qe * frequency_visibility(source_freq) / d2
}

/// Semiancho visible aproximado en XZ para el rectángulo del viewport en el minimap.
///
/// Heurística a partir del zoom horizontal MOBA y el aspect ratio de ventana (w/h).
#[inline]
pub fn moba_minimap_viewport_half_extents_xz(zoom_horizontal: f32, window_aspect_wh: f32) -> Vec2 {
    let z = zoom_horizontal.max(1.0);
    let base = z * 0.44;
    let ar = window_aspect_wh.max(0.2);
    Vec2::new(base, base / ar)
}
