//! Tunings del minimap (presentación). Mapeo world↔UV en `blueprint::equations`.

/// Margen desde borde inferior-derecho de la ventana (px).
pub const MINIMAP_MARGIN_PX: f32 = 14.0;
/// Lado del panel cuadrado (px).
pub const MINIMAP_SIZE_PX: f32 = 172.0;
/// Inset interior para iconos respecto al borde del panel (px).
pub const MINIMAP_INNER_INSET_PX: f32 = 4.0;
/// Throttle de refresco de iconos (sprint G10: 5–10 frames @ 30Hz).
pub const MINIMAP_UPDATE_EVERY_FRAMES: u32 = 8;
/// Relleno semitransparente del rectángulo de viewport.
pub const MINIMAP_VIEWPORT_FILL_ALPHA: f32 = 0.24;
