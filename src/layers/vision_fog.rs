//! Componentes auxiliares para Fog of War (G12): proveedores de visión y máscara por equipo.
//! No sustituyen L9; el equipo de juego sigue siendo [`crate::layers::Faction`].

use bevy::math::Vec2;
use bevy::prelude::*;

/// Entidad que revela celdas en el [`crate::world::FogOfWarGrid`] (héroes, torres, wards).
/// `max_radius` acota el stamp en el grid (MVP G12). `sensitivity` reserva umbral para futuro filtro con
/// [`crate::blueprint::equations::perception_signal`] (L0+L2); hoy no lo lee ningún sistema.
#[derive(Component, Reflect, Debug, Clone, Copy)]
#[reflect(Component)]
pub struct VisionProvider {
    max_radius: f32,
    sensitivity: f32,
    team: u8,
}

impl VisionProvider {
    #[inline]
    pub fn new(max_radius: f32, sensitivity: f32, team: u8) -> Self {
        Self {
            max_radius: if max_radius.is_finite() {
                max_radius.max(0.0)
            } else {
                0.0
            },
            sensitivity: if sensitivity.is_finite() {
                sensitivity.max(0.0)
            } else {
                0.0
            },
            team,
        }
    }

    #[inline]
    pub fn max_radius(&self) -> f32 {
        self.max_radius
    }

    #[inline]
    pub fn sensitivity(&self) -> f32 {
        self.sensitivity
    }

    #[inline]
    pub fn team(&self) -> u8 {
        self.team
    }
}

/// Última posición en el plano de sim usada para stamp/unstamp en el grid (evita drift de refcount).
#[derive(Component, Reflect, Debug, Clone, Copy)]
#[reflect(Component)]
pub struct VisionFogAnchor {
    pub last_plane: Vec2,
    pub has_last: bool,
}

impl Default for VisionFogAnchor {
    fn default() -> Self {
        Self {
            last_plane: Vec2::ZERO,
            has_last: false,
        }
    }
}

/// Bloquea visión (terreno, muros). Reservado para LOS futuro; sin sistemas en G12.
#[derive(Component, Reflect, Debug, Clone, Copy, Default)]
#[component(storage = "SparseSet")]
#[reflect(Component)]
pub struct VisionBlocker;

/// Bits por equipo MoBA (0 = Red, 1 = Blue): si el bit está en 1, la entidad se oculta a ese observador.
#[derive(Component, Reflect, Debug, Clone, Copy, Default, PartialEq, Eq)]
#[component(storage = "SparseSet")]
#[reflect(Component)]
pub struct FogHiddenMask(pub u8);

impl FogHiddenMask {
    #[inline]
    pub fn hidden_from_team(self, team: u8) -> bool {
        (self.0 & (1u8 << team)) != 0
    }
}
