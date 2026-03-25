use bevy::prelude::*;

/// Capa V5: política local de memoización e invalidación por entidad.
///
/// Define si una entidad participa de la optimización de cómputo repetido y con qué
/// alcance de cache. No calcula física; solo declara contrato operativo para sistemas V5.
#[derive(Component, Reflect, Debug, Clone)]
#[reflect(Component)]
pub struct PerformanceCachePolicy {
    /// Activa/desactiva la política en runtime para esta entidad.
    pub enabled: bool,
    /// Alcance permitido de reutilización de resultados.
    pub scope: CacheScope,
    /// Versión de ecuaciones/contrato usada para invalidar cache.
    pub version_tag: u32,
    /// Firma compacta de dependencias relevantes para invalidación selectiva.
    pub dependency_signature: u16,
}

impl Default for PerformanceCachePolicy {
    fn default() -> Self {
        Self {
            enabled: false,
            scope: CacheScope::FrameLocal,
            version_tag: 1,
            dependency_signature: 0,
        }
    }
}

/// Alcance de validez de una entrada cacheada.
#[derive(Reflect, Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum CacheScope {
    /// Válido solo durante el tick/frame actual.
    #[default]
    FrameLocal,
    /// Válido por una ventana corta de ticks cuando la key canónica no cambia.
    StableWindow,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_policy_disabled_frame_local() {
        let p = PerformanceCachePolicy::default();
        assert!(!p.enabled);
        assert_eq!(p.scope, CacheScope::FrameLocal);
        assert_eq!(p.version_tag, 1);
        assert_eq!(p.dependency_signature, 0);
    }

    #[test]
    fn cache_scope_default_is_frame_local() {
        assert_eq!(CacheScope::default(), CacheScope::FrameLocal);
    }

    #[test]
    fn stable_window_distinct_from_frame_local() {
        assert_ne!(CacheScope::StableWindow, CacheScope::FrameLocal);
    }
}
