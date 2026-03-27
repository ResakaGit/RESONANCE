use bevy::prelude::*;

use crate::blueprint::constants::WILL_MOVEMENT_INTENT_SQ_EPSILON;
use serde::{Deserialize, Serialize};

use crate::blueprint::ElementId;
use crate::blueprint::recipes::{EffectRecipe, TransmuteDir};
/// Límite de slots en grimorio (evita crecimiento ilimitado del Vec).
pub const MAX_GRIMOIRE_ABILITIES: usize = 64;

/// Capa 7: Agencia y Locomoción — El Actuador
///
/// La voluntad inyectada en el motor. Traduce inputs (teclado/IA) en órdenes
/// de drenaje de la Capa 5 para generar vectores en la Capa 3.
#[derive(Component, Reflect, Debug, Clone, Serialize, Deserialize)]
#[reflect(Component)]
pub struct WillActuator {
    /// Vector normalizado del input (WASD / Click / IA).
    pub(crate) movement_intent: Vec2,

    /// Si está usando la válvula de salida (canalizando habilidad).
    pub(crate) channeling_ability: bool,

    /// Índice del slot activo en el Grimoire (None si no canaliza).
    pub(crate) active_slot: Option<usize>,

    /// Intención social: dirección de agrupamiento o exploración colectiva.
    /// Escrito por GS-4 (pack cohesion) y ET-16 (long-range planning). Vec2::ZERO = sin presión social.
    pub(crate) social_intent: Vec2,
}

impl Default for WillActuator {
    fn default() -> Self {
        Self {
            movement_intent: Vec2::ZERO,
            channeling_ability: false,
            active_slot: None,
            social_intent: Vec2::ZERO,
        }
    }
}

impl WillActuator {
    #[inline]
    pub fn movement_intent(&self) -> Vec2 {
        self.movement_intent
    }

    pub fn set_movement_intent(&mut self, v: Vec2) {
        self.movement_intent = if v.is_finite() { v } else { Vec2::ZERO };
    }

    #[inline]
    pub fn channeling_ability(&self) -> bool {
        self.channeling_ability
    }

    pub fn set_channeling_ability(&mut self, v: bool) {
        self.channeling_ability = v;
    }

    #[inline]
    pub fn active_slot(&self) -> Option<usize> {
        self.active_slot
    }

    pub fn set_active_slot(&mut self, slot: Option<usize>) {
        self.active_slot = slot;
    }

    /// Intención social: dirección de cohesión de grupo o exploración planificada.
    #[inline]
    pub fn social_intent(&self) -> Vec2 {
        self.social_intent
    }

    pub fn set_social_intent(&mut self, v: Vec2) {
        self.social_intent = if v.is_finite() { v } else { Vec2::ZERO };
    }

    /// ¿Tiene intención de moverse?
    pub fn wants_to_move(&self) -> bool {
        self.movement_intent.length_squared() > WILL_MOVEMENT_INTENT_SQ_EPSILON
    }

    /// ¿Puede moverse? (No si está canalizando)
    pub fn can_move(&self) -> bool {
        !self.channeling_ability && self.wants_to_move()
    }
}

/// Qué tipo de entidad/efecto produce una habilidad al activarse.
#[derive(Clone, Debug, Reflect, Serialize, Deserialize)]
pub enum AbilityOutput {
    /// Proyectil que viaja y colisiona (catálisis).
    Projectile {
        element_id: ElementId,
        radius: f32,
        speed: f32,
        effect: Option<EffectRecipe>,
    },

    /// Invoca una entidad persistente (torreta, dron, golem).
    /// El template se resuelve en runtime via TemplateRegistry.
    Summon {
        template_name: String,
        max_active: u8,
    },

    /// Transmuta la frecuencia del target.
    Transmute {
        freq_shift_per_sec: f32,
        max_shift: f32,
        direction: TransmuteDir,
    },

    /// Buff/debuff sobre sí mismo.
    SelfBuff { effect: EffectRecipe },

    /// Zona estática de área.
    Zone { template_name: String },

    /// Entidad que se activa al contacto.
    Pickup {
        template_name: String,
        on_contact: EffectRecipe,
    },

    /// Múltiples proyectiles en abanico (campos aplanados, sin Box).
    Barrage {
        element_id: ElementId,
        radius: f32,
        speed: f32,
        effect: Option<EffectRecipe>,
        count: u8,
        spread_angle: f32,
    },
}

/// Objetivo resuelto para un cast (dato puro; vive en eventos y en `Channeling`).
#[derive(Clone, Debug, Reflect, Serialize, Deserialize, PartialEq)]
pub enum AbilityTarget {
    None,
    Point(Vec3),
    Entity(Entity),
    Direction(Vec3),
}

/// Modo de selección de objetivo (MOBA). Sin timers: el “cooldown” emerge del buffer L5.
#[derive(Clone, Debug, Reflect, Serialize, Deserialize, PartialEq)]
pub enum TargetingMode {
    /// Auto-cast (ej. buff sobre sí).
    NoTarget,
    /// Click en suelo dentro de `range`.
    PointTarget { range: f32 },
    /// Click en entidad (homing / single-target; resolución futura).
    UnitTarget { range: f32 },
    /// Skillshot en línea desde el caster.
    DirectionTarget { range: f32 },
    /// AoE: centro por click dentro de `range`, radio `radius`.
    AreaTarget { radius: f32, range: f32 },
}

/// Costo, targeting y canalización física del inyector (no es cooldown almacenado).
#[derive(Clone, Debug, Reflect, Serialize, Deserialize)]
pub struct AbilityCastSpec {
    pub cost_qe: f32,
    pub targeting: TargetingMode,
    /// Tiempo mínimo de emisión L8; 0 = instantáneo.
    pub min_channeling_secs: f32,
}

impl Default for AbilityCastSpec {
    fn default() -> Self {
        Self {
            cost_qe: 0.0,
            targeting: TargetingMode::NoTarget,
            min_channeling_secs: 0.0,
        }
    }
}

/// Slot de habilidad individual dentro de un Grimoire (≤4 campos top-level).
#[derive(Clone, Debug, Reflect, Serialize, Deserialize)]
pub struct AbilitySlot {
    /// Nombre de la habilidad (debug / UI).
    pub name: String,
    /// Qué produce la habilidad al activarse.
    pub output: AbilityOutput,
    pub cast: AbilityCastSpec,
}

impl AbilitySlot {
    #[inline]
    pub fn cost_qe(&self) -> f32 {
        self.cast.cost_qe
    }

    #[inline]
    pub fn targeting(&self) -> &TargetingMode {
        &self.cast.targeting
    }

    #[inline]
    pub fn min_channeling_secs(&self) -> f32 {
        self.cast.min_channeling_secs
    }
}

/// Capa 7b: Grimorio — Catálogo de habilidades de la entidad.
#[derive(Component, Reflect, Debug, Clone, Default)]
#[reflect(Component)]
pub struct Grimoire {
    pub(crate) abilities: Vec<AbilitySlot>,
}

impl Grimoire {
    /// Iteración de solo lectura sobre slots (API pública fuera del crate).
    #[inline]
    pub fn abilities(&self) -> &[AbilitySlot] {
        &self.abilities
    }

    /// Inserta una habilidad en el grimorio.
    pub fn push_ability(&mut self, ability: AbilitySlot) -> bool {
        if self.abilities.len() >= MAX_GRIMOIRE_ABILITIES {
            return false;
        }
        self.abilities.push(ability);
        true
    }
}

/// Marker: despawnea el spell en el primer contacto exitoso.
///
/// Usado por projectiles “OneShot” generados desde el `Grimoire`.
#[derive(Component, Reflect, Debug, Clone, Copy, Default)]
#[component(storage = "SparseSet")]
#[reflect(Component)]
pub struct DespawnOnContact;

/// Componente: una entidad-efecto persistente se spawnea cuando el spell
/// impacta un target en runtime.
#[derive(Component, Reflect, Debug, Clone)]
#[component(storage = "SparseSet")]
#[reflect(Component)]
pub struct OnContactEffect {
    pub recipe: EffectRecipe,
}

/// Marker: enlaza `AlchemicalInjector.projected_qe` con la energía actual
/// (`BaseEnergy.qe`) del spell.
///
/// Así, la energía modificada por contención/canales termina afectando
/// el daño/transferencia calculada por catálisis.
#[derive(Component, Reflect, Debug, Clone, Copy, Default)]
#[reflect(Component)]
pub struct ProjectedQeFromEnergy;

/// Inyector físico canalizando; al expirar se emite el cast pendiente (SparseSet).
#[derive(Component, Reflect, Debug, Clone)]
#[component(storage = "SparseSet")]
#[reflect(Component)]
pub struct Channeling {
    pub remaining_secs: f32,
    pub slot_index: usize,
    pub target: AbilityTarget,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layers::ModifiedField;

    #[test]
    fn movement_intent_roundtrip() {
        let mut w = WillActuator::default();
        let v = Vec2::new(0.6, -0.8);
        w.set_movement_intent(v);
        assert_eq!(w.movement_intent(), v);
    }

    #[test]
    fn grimoire_push_respects_max_slots() {
        let mut g = Grimoire::default();
        let slot = AbilitySlot {
            name: "test".to_string(),
            output: AbilityOutput::SelfBuff {
                effect: EffectRecipe {
                    field: ModifiedField::DissipationMultiplier,
                    magnitude: 1.0,
                    fuel_qe: 1.0,
                    dissipation: 1.0,
                },
            },
            cast: AbilityCastSpec {
                cost_qe: 1.0,
                targeting: TargetingMode::NoTarget,
                min_channeling_secs: 0.0,
            },
        };
        for _ in 0..MAX_GRIMOIRE_ABILITIES {
            assert!(g.push_ability(slot.clone()));
        }
        assert!(!g.push_ability(slot));
        assert_eq!(g.abilities().len(), MAX_GRIMOIRE_ABILITIES);
    }

    #[test]
    fn channeling_blocks_can_move() {
        let mut w = WillActuator::default();
        w.set_movement_intent(Vec2::X);
        assert!(w.can_move());
        w.set_channeling_ability(true);
        assert!(!w.can_move());
    }
}
