use bevy::prelude::{Color, Entity, Vec2, Vec3};

use crate::runtime_platform::core_math_agnostic::{clamp_unit, normalize_or_zero};

/// Revisión congelada del contrato compartido V6.
pub const V6_CONTRACTS_REV: u32 = 1;

/// Bit para acción primaria en `button_mask`.
pub const BUTTON_PRIMARY_ACTION: u16 = 1 << 0;

/// Snapshot de input discreto del jugador/controlador.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct IntentSnapshot {
    /// Vector XY normalizado o cero.
    pub movement_xy: Vec2,
    /// Botones discretos codificados como bitset.
    pub button_mask: u16,
    /// Tick opcional para alineación temporal de backend/frontend.
    pub tick_id: Option<u64>,
}

impl IntentSnapshot {
    pub fn new(movement_xy: Vec2, button_mask: u16, tick_id: Option<u64>) -> Self {
        Self {
            movement_xy: normalize_or_zero(movement_xy),
            button_mask,
            tick_id,
        }
    }

    pub fn primary_action(&self) -> bool {
        (self.button_mask & BUTTON_PRIMARY_ACTION) != 0
    }
}

/// Intención final para locomoción 3D (plano XZ).
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct WillIntent3D {
    /// Dirección en XZ normalizada o cero.
    pub direction_xz: Vec2,
    /// Intensidad del input (0..1).
    pub magnitude: f32,
}

impl WillIntent3D {
    pub fn new(direction_xz: Vec2, magnitude: f32) -> Self {
        Self {
            direction_xz: normalize_or_zero(direction_xz),
            magnitude: clamp_unit(magnitude),
        }
    }

    pub fn zero() -> Self {
        Self::default()
    }
}

/// Pose mínima 2D para broadphase.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Pose2 {
    pub position: Vec2,
    pub radius: f32,
}

impl Pose2 {
    pub fn new(position: Vec2, radius: f32) -> Self {
        Self {
            position,
            radius: radius.max(0.0),
        }
    }
}

/// Pose mínima 3D para broadphase.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Pose3 {
    pub position: Vec3,
    pub radius: f32,
}

impl Pose3 {
    pub fn new(position: Vec3, radius: f32) -> Self {
        Self {
            position,
            radius: radius.max(0.0),
        }
    }
}

/// Par candidato de espacialización/collisión ordenado canónicamente.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SpatialCandidatePair {
    pub a: Entity,
    pub b: Entity,
}

impl SpatialCandidatePair {
    pub fn new(left: Entity, right: Entity) -> Self {
        if left.to_bits() < right.to_bits() {
            Self { a: left, b: right }
        } else {
            Self { a: right, b: left }
        }
    }
}

/// Contacto de colisión (stub inicial del contrato V6).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CollisionContact {
    pub a: Entity,
    pub b: Entity,
    pub normal: Vec3,
    pub penetration: Option<f32>,
}

impl CollisionContact {
    pub fn new(a: Entity, b: Entity, normal: Vec3, penetration: Option<f32>) -> Self {
        Self {
            a,
            b,
            normal,
            penetration: penetration.map(|depth| depth.max(0.0)),
        }
    }
}

/// Pod mínimo para render: datos ya **derivados** en capture (hex boundary).
/// La sim no lee esto; solo el sistema `sync_visual_from_sim_system`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct VisualEntityPod {
    pub sim_entity: Entity,
    pub translation: Vec3,
    pub scale: Vec3,
    pub base_color: Color,
    pub emissive: Color,
    pub perceptual_roughness: f32,
    pub metallic: f32,
}

/// Snapshot post-simulación para puente visual (hex boundary).
#[derive(Debug, Clone, PartialEq, Default)]
pub struct SimStateSnapshot {
    pub tick_id: Option<u64>,
    pub pods: Vec<VisualEntityPod>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn spatial_candidate_pair_orders_entities_ascending() {
        let e_low = Entity::from_raw(1);
        let e_high = Entity::from_raw(9);
        let pair = SpatialCandidatePair::new(e_high, e_low);
        assert_eq!(pair.a, e_low);
        assert_eq!(pair.b, e_high);
    }

    #[test]
    fn intent_snapshot_new_normalizes_non_zero_vector() {
        let snapshot = IntentSnapshot::new(Vec2::new(3.0, 4.0), 0, Some(10));
        assert!((snapshot.movement_xy.length() - 1.0).abs() < 1e-6);
    }

    #[test]
    fn will_intent3d_new_clamps_magnitude_to_unit_interval() {
        let low = WillIntent3D::new(Vec2::X, -1.0);
        let high = WillIntent3D::new(Vec2::X, 5.0);
        assert_eq!(low.magnitude, 0.0);
        assert_eq!(high.magnitude, 1.0);
    }
}
