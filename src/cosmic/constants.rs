//! Constantes centralizadas del stack Cosmic Telescope.
//! Centralized constants for the Cosmic Telescope stack.
//!
//! Convención del proyecto (CLAUDE.md §Coding Rules §10): cada módulo
//! mantiene sus constantes en `{module}/constants.rs`, derivadas desde los
//! fundamentales (`blueprint/equations/derived_thresholds.rs`) cuando aplica.
//!
//! Aquí viven SÓLO valores específicos del dominio multi-escala:
//! - etiquetas para HUD y logs
//! - defaults del Big Bang interactivo (CT-2/CT-8)
//! - bounds de bridges (CT-4, CT-5)
//! - temperaturas de sonda para worldgen (CT-5)
//! - distancias de cámara por escala (CT-8 visual)
//!
//! Física auténtica (dissipation por fase) se importa de `derived_thresholds`.

use super::ScaleLevel;

// ─── Iteración canónica ────────────────────────────────────────────────────

/// Orden canónico de escalas S0→S4. Usado por HUDs, breadcrumbs y tests para
/// iterar de forma determinista sin depender de `enum` discriminant order.
pub const ALL_SCALES: [ScaleLevel; 5] = [
    ScaleLevel::Cosmological,
    ScaleLevel::Stellar,
    ScaleLevel::Planetary,
    ScaleLevel::Ecological,
    ScaleLevel::Molecular,
];

// ─── Labels ────────────────────────────────────────────────────────────────

/// Tag compacto "S0"…"S4". Útil para HUDs con poco espacio.
#[inline]
pub const fn scale_short(s: ScaleLevel) -> &'static str {
    match s {
        ScaleLevel::Cosmological => "S0",
        ScaleLevel::Stellar => "S1",
        ScaleLevel::Planetary => "S2",
        ScaleLevel::Ecological => "S3",
        ScaleLevel::Molecular => "S4",
    }
}

/// Nombre largo humano-legible ("Cosmological", "Stellar", …).
#[inline]
pub const fn scale_label(s: ScaleLevel) -> &'static str {
    match s {
        ScaleLevel::Cosmological => "Cosmological",
        ScaleLevel::Stellar => "Stellar",
        ScaleLevel::Planetary => "Planetary",
        ScaleLevel::Ecological => "Ecological",
        ScaleLevel::Molecular => "Molecular",
    }
}

// ─── Big Bang interactivo (CT-2 / CT-8) ────────────────────────────────────

/// N. de clusters iniciales para el preset interactivo (suficientes para
/// visualizar diversidad, pocos para mantener <30 FPS holgado).
pub const INTERACTIVE_BIG_BANG_CLUSTERS: usize = 32;

/// Energía total del universo interactivo (qe). Más bajo que el default
/// `CosmoConfig::default_with_seed` para tests/viz rápidos.
pub const INTERACTIVE_BIG_BANG_TOTAL_QE: f64 = 1.0e5;

/// Ticks de warmup cosmológico antes de mostrar — permite que los clusters
/// ganen coherencia mínima (cosmic_gravity relaja posiciones).
pub const INTERACTIVE_BIG_BANG_WARMUP_TICKS: usize = 100;

// ─── Bridges (CT-4 / CT-5) ─────────────────────────────────────────────────

/// Fracción de la qe estelar que forma el disco protoplanetario. Referencia
/// astrofísica: discos típicos ~1 % de la masa del hospedador.
pub const STELLAR_DISK_FRACTION: f64 = 0.01;

/// Fracción de la qe planetaria capturada por el ecosistema (S3).
/// Placeholder honesto — worldgen real vive fuera del bridge.
pub const ECOLOGICAL_CAPTURE_FRACTION: f64 = 0.5;

pub const MIN_STARS_PER_CLUSTER: usize = 20;
pub const MAX_STARS_PER_CLUSTER: usize = 100;
pub const MIN_PLANETS_PER_STAR: usize = 3;
pub const MAX_PLANETS_PER_STAR: usize = 12;

/// Temperatura de sonda para el `MapConfig` placeholder en `build_ecological`.
/// En unidades normalizadas del proyecto (no Kelvin). Valor elegido dentro
/// del rango habitable para que `planet_to_map_config` no rechace el planeta.
pub const ECOLOGICAL_PROBE_TEMPERATURE: f64 = 0.015;

// ─── Viewer 3D (CT-8) ──────────────────────────────────────────────────────

/// Distancia objetivo de la cámara por escala tras una transición. Valores
/// visuales (no físicos); calibrados para que entidades ocupen ~25-60 % de
/// viewport en cada nivel.
#[inline]
pub const fn scale_camera_distance(s: ScaleLevel) -> f32 {
    match s {
        ScaleLevel::Cosmological => 26.0,
        ScaleLevel::Stellar => 22.0,
        ScaleLevel::Planetary => 18.0,
        ScaleLevel::Ecological => 14.0,
        ScaleLevel::Molecular => 12.0,
    }
}

// ─── Tests ─────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_scales_covers_every_level() {
        assert_eq!(ALL_SCALES.len(), 5);
        for (i, s) in ALL_SCALES.iter().enumerate() {
            assert_eq!(s.depth() as usize, i);
        }
    }

    #[test]
    fn labels_are_unique_and_stable() {
        let shorts: Vec<&str> = ALL_SCALES.iter().map(|s| scale_short(*s)).collect();
        let labels: Vec<&str> = ALL_SCALES.iter().map(|s| scale_label(*s)).collect();
        assert_eq!(shorts, ["S0", "S1", "S2", "S3", "S4"]);
        assert_eq!(labels.iter().collect::<std::collections::HashSet<_>>().len(), 5);
    }

    #[test]
    fn camera_distance_monotone_with_depth() {
        let mut prev = f32::INFINITY;
        for s in ALL_SCALES {
            let d = scale_camera_distance(s);
            assert!(d <= prev, "camera distance must decrease with depth");
            prev = d;
        }
    }
}
