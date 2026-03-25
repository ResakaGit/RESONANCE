/// ══════════════════════════════════════════════════════════════
/// Recetas: descripciones declarativas de efectos y transmutaciones.
///
/// Una receta describe QUÉ hace un efecto, sin especificar el target.
/// El target se resuelve en runtime cuando el efecto se aplica.
/// ══════════════════════════════════════════════════════════════
use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use crate::layers::ModifiedField;

/// Receta de efecto: descripción pre-target de un ResonanceLink.
///
/// Cuando un proyectil impacta o una habilidad se activa, la receta
/// se materializa como una entidad-efecto (Capa 10) vinculada al target.
#[derive(Clone, Debug, Reflect, Serialize, Deserialize)]
pub struct EffectRecipe {
    /// Qué campo del target se modifica.
    pub field: ModifiedField,

    /// Magnitud del modificador (ej. 0.5 = slow, 2.0 = haste).
    pub magnitude: f32,

    /// Energía inicial de la entidad-efecto (combustible).
    pub fuel_qe: f32,

    /// Tasa de disipación de la entidad-efecto (determina duración = fuel_qe / dissipation).
    pub dissipation: f32,
}

impl EffectRecipe {
    /// Duración aproximada del efecto en segundos.
    pub fn duration_secs(&self) -> f32 {
        if self.dissipation <= 0.0 {
            f32::INFINITY
        } else {
            self.fuel_qe / self.dissipation
        }
    }
}

/// Dirección de transmutación frecuencial.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Reflect, Serialize, Deserialize)]
pub enum TransmuteDir {
    /// Sube la frecuencia del target.
    Up,
    /// Baja la frecuencia del target.
    Down,
    /// Empuja la frecuencia del target hacia la del caster.
    TowardCaster,
}
