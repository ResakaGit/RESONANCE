//! Constantes de tuning — cámara MOBA (sin fórmulas; ver `blueprint::equations`).

/// Pitch por defecto (~58° entre rayo cámara→foco y plano horizontal).
pub const DEFAULT_MOBA_PITCH_DEG: f32 = 58.0;

/// Distancia horizontal inicial foco→cámara (eje +Z local al offset).
pub const DEFAULT_MOBA_ZOOM_HORIZONTAL: f32 = 22.0;
