use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use crate::blueprint::constants::{
    FACTION_ALLY_BONUS, FACTION_ENEMY_MALUS, LINK_NEUTRAL_MULTIPLIER,
};

/// Facciones del juego. Determinan alianza y afectan el signo de interferencia.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Reflect, Default, Serialize, Deserialize)]
pub enum Faction {
    #[default]
    Neutral,
    Red,
    Blue,
    Wild,
}

impl Faction {
    /// En un 1v1, retorna la facción oponente canónica. Red ↔ Blue.
    pub fn opponent(self) -> Self {
        match self {
            Faction::Red     => Faction::Blue,
            Faction::Blue    => Faction::Red,
            Faction::Neutral => Faction::Neutral,
            Faction::Wild    => Faction::Wild,
        }
    }
}

/// Tags relacionales para filtros de targeting.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Reflect, Serialize, Deserialize)]
pub enum RelationalTag {
    Ally,
    Enemy,
    Resource,
    Structure,
    Summon,
    Hero,
    Minion,
    Jungle,
}

/// Capa 9: Meta-Contexto e Identidad — Reglas de MOBA
///
/// El "Juego" sobre la simulación. Etiquetas arbitrarias que interceptan las
/// matemáticas físicas para aplicar reglas de diseño (Lore, Facciones, Odio, Ítems).
#[derive(Component, Reflect, Debug, Clone, Serialize, Deserialize)]
#[reflect(Component)]
pub struct MobaIdentity {
    /// Equipo/facción de la entidad.
    pub(crate) faction: Faction,

    /// Tags relacionales para filtros y reglas de targeting.
    pub(crate) relational_tags: Vec<RelationalTag>,

    /// Multiplicador de daño/curación crítico.
    /// Alterado por la Fase (Capa 2) y relaciones de Odio/Afinidad.
    pub(crate) critical_multiplier: f32,
}

impl Default for MobaIdentity {
    fn default() -> Self {
        Self {
            faction: Faction::Neutral,
            relational_tags: Vec::new(),
            critical_multiplier: LINK_NEUTRAL_MULTIPLIER,
        }
    }
}

impl MobaIdentity {
    #[inline]
    pub fn faction(&self) -> Faction {
        self.faction
    }

    #[inline]
    pub fn relational_tags(&self) -> &[RelationalTag] {
        &self.relational_tags
    }

    #[inline]
    pub fn critical_multiplier(&self) -> f32 {
        self.critical_multiplier
    }

    pub fn set_critical_multiplier(&mut self, v: f32) {
        self.critical_multiplier = if v.is_finite() { v.max(0.0) } else { 0.0 };
    }

    /// ¿Son aliados? Misma facción no-neutral.
    pub fn is_ally(&self, other: &MobaIdentity) -> bool {
        self.faction != Faction::Neutral
            && other.faction != Faction::Neutral
            && self.faction == other.faction
    }

    /// ¿Son enemigos? Facciones distintas, ambas no-neutrales.
    pub fn is_enemy(&self, other: &MobaIdentity) -> bool {
        self.faction != Faction::Neutral
            && other.faction != Faction::Neutral
            && self.faction != other.faction
    }

    /// Modificador de interferencia según facción.
    /// Aliados obtienen bonus constructivo, enemigos bonus destructivo.
    pub fn faction_modifier(&self, other: &MobaIdentity) -> f32 {
        if self.is_ally(other) {
            FACTION_ALLY_BONUS
        } else if self.is_enemy(other) {
            FACTION_ENEMY_MALUS
        } else {
            0.0
        }
    }

    pub fn has_tag(&self, tag: RelationalTag) -> bool {
        self.relational_tags.contains(&tag)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::blueprint::constants::{FACTION_ALLY_BONUS, FACTION_ENEMY_MALUS};

    #[test]
    fn faction_modifier_ally_positive() {
        let a = MobaIdentity {
            faction: Faction::Red,
            relational_tags: Vec::new(),
            critical_multiplier: 1.0,
        };
        let b = MobaIdentity {
            faction: Faction::Red,
            relational_tags: Vec::new(),
            critical_multiplier: 1.0,
        };
        assert!((a.faction_modifier(&b) - FACTION_ALLY_BONUS).abs() < 1e-5);
        assert!(a.faction_modifier(&b) > 0.0);
    }

    #[test]
    fn faction_modifier_enemy_negative() {
        let a = MobaIdentity {
            faction: Faction::Red,
            relational_tags: Vec::new(),
            critical_multiplier: 1.0,
        };
        let b = MobaIdentity {
            faction: Faction::Blue,
            relational_tags: Vec::new(),
            critical_multiplier: 1.0,
        };
        assert!((a.faction_modifier(&b) - FACTION_ENEMY_MALUS).abs() < 1e-5);
        assert!(a.faction_modifier(&b) < 0.0);
    }

    #[test]
    fn default_critical_multiplier_positive() {
        let id = MobaIdentity::default();
        assert!(id.critical_multiplier() > 0.0);
        assert!(id.critical_multiplier().is_finite());
    }
}
